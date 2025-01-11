use crate::arch::drop_user_space;
use crate::arch::mm::{PhysAddr, VirtAddr};
use crate::arch::switch;
use crate::collections::irqsave;
use crate::consts::*;
use crate::errno::*;
use crate::fd::{FileDescriptor, IoInterface};
use crate::io;
use crate::logging::*;
use crate::scheduler::task::*;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::rc::Rc;
use alloc::sync::Arc;
use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering};

static TID_COUNTER: AtomicU32 = AtomicU32::new(0);

pub(crate) struct Scheduler {
	/// task id which is currently running
	current_task: Rc<RefCell<Task>>,
	/// task id of the idle task
	idle_task: Rc<RefCell<Task>>,
	/// queue of tasks, which are ready
	ready_queue: PriorityTaskQueue,
	/// queue of tasks, which are finished and can be released
	finished_tasks: VecDeque<TaskId>,
	/// map between task id and task control block
	tasks: BTreeMap<TaskId, Rc<RefCell<Task>>>,
}

impl Scheduler {
	pub fn new() -> Scheduler {
		let tid = TaskId::from(TID_COUNTER.fetch_add(1, Ordering::SeqCst));
		let idle_task = Rc::new(RefCell::new(Task::new_idle(tid)));
		let mut tasks = BTreeMap::new();

		tasks.insert(tid, idle_task.clone());

		Scheduler {
			current_task: idle_task.clone(),
			idle_task: idle_task.clone(),
			ready_queue: PriorityTaskQueue::new(),
			finished_tasks: VecDeque::<TaskId>::new(),
			tasks,
		}
	}

	fn get_tid(&self) -> TaskId {
		loop {
			let id = TaskId::from(TID_COUNTER.fetch_add(1, Ordering::SeqCst));

			if !self.tasks.contains_key(&id) {
				return id;
			}
		}
	}

	pub fn spawn(&mut self, func: extern "C" fn(), prio: TaskPriority) -> Result<TaskId> {
		let closure = || {
			let prio_number: usize = prio.into().into();

			if prio_number >= NO_PRIORITIES {
				return Err(Error::BadPriority);
			}

			// Create the new task.
			let tid = self.get_tid();
			let task = Rc::new(RefCell::new(Task::new(tid, TaskStatus::Ready, prio)));

			task.borrow_mut().create_stack_frame(func);

			// Add it to the task lists.
			self.ready_queue.push(task.clone());
			self.tasks.insert(tid, task);

			info!("Creating task {}", tid);

			Ok(tid)
		};

		irqsave(closure)
	}

	fn cleanup(&mut self) {
		// destroy user space
		drop_user_space();

		self.current_task.borrow_mut().status = TaskStatus::Finished;
	}

	pub fn exit(&mut self) -> ! {
		let closure = || {
			if self.current_task.borrow().status != TaskStatus::Idle {
				info!("finish task with id {}", self.current_task.borrow().id);
				self.cleanup();
			} else {
				panic!("unable to terminate idle task");
			}
		};

		irqsave(closure);

		self.reschedule();

		// we should never reach this point
		panic!("exit failed!");
	}

	pub fn abort(&mut self) -> ! {
		let closure = || {
			if self.current_task.borrow().status != TaskStatus::Idle {
				info!("abort task with id {}", self.current_task.borrow().id);
				self.cleanup();
			} else {
				panic!("unable to terminate idle task");
			}
		};

		irqsave(closure);

		self.reschedule();

		// we should never reach this point
		panic!("abort failed!");
	}

	pub fn block_current_task(&mut self) -> Rc<RefCell<Task>> {
		let closure = || {
			if self.current_task.borrow().status == TaskStatus::Running {
				debug!("block task {}", self.current_task.borrow().id);

				self.current_task.borrow_mut().status = TaskStatus::Blocked;
				self.current_task.clone()
			} else {
				panic!("unable to block task {}", self.current_task.borrow().id);
			}
		};

		irqsave(closure)
	}

	pub fn wakeup_task(&mut self, task: Rc<RefCell<Task>>) {
		let closure = || {
			if task.borrow().status == TaskStatus::Blocked {
				debug!("wakeup task {}", task.borrow().id);

				task.borrow_mut().status = TaskStatus::Ready;
				self.ready_queue.push(task.clone());
			}
		};

		irqsave(closure);
	}

	pub(crate) fn insert_io_interface(
		&mut self,
		io_interface: Arc<dyn IoInterface>,
	) -> io::Result<FileDescriptor> {
		let new_fd = || -> io::Result<FileDescriptor> {
			let mut fd: FileDescriptor = 0;
			loop {
				if !self.current_task.borrow().fd_map.contains_key(&fd) {
					break Ok(fd);
				} else if fd == FileDescriptor::MAX {
					break Err(io::Error::EOVERFLOW);
				}

				fd = fd.saturating_add(1);
			}
		};

		let fd = new_fd()?;
		self.current_task
			.borrow_mut()
			.fd_map
			.insert(fd, io_interface.clone());

		Ok(fd)
	}

	pub fn remove_io_interface(&self, fd: FileDescriptor) -> io::Result<Arc<dyn IoInterface>> {
		self.current_task
			.borrow_mut()
			.fd_map
			.remove(&fd)
			.ok_or(io::Error::EBADF)
	}

	pub(crate) fn get_io_interface(
		&self,
		fd: FileDescriptor,
	) -> crate::io::Result<Arc<dyn IoInterface>> {
		let closure = || {
			if let Some(io_interface) = self.current_task.borrow().fd_map.get(&fd) {
				Ok(io_interface.clone())
			} else {
				Err(crate::io::Error::ENOENT)
			}
		};

		irqsave(closure)
	}

	pub fn get_current_taskid(&self) -> TaskId {
		irqsave(|| self.current_task.borrow().id)
	}

	/// Determines the start address of the stack
	#[no_mangle]
	pub fn get_current_interrupt_stack(&self) -> VirtAddr {
		irqsave(|| (*self.current_task.borrow().stack).interrupt_top())
	}

	pub fn get_root_page_table(&self) -> PhysAddr {
		self.current_task.borrow().root_page_table
	}

	pub fn set_root_page_table(&self, addr: PhysAddr) {
		self.current_task.borrow_mut().root_page_table = addr;
	}

	pub fn schedule(&mut self) {
		// do we have finished tasks? => drop tasks => deallocate implicitly the stack
		if let Some(id) = self.finished_tasks.pop_front() {
			if self.tasks.remove(&id).is_none() {
				info!("Unable to drop task {}", id);
			}
		}

		// Get information about the current task.
		let (current_id, current_stack_pointer, current_prio, current_status) = {
			let mut borrowed = self.current_task.borrow_mut();
			(
				borrowed.id,
				&mut borrowed.last_stack_pointer as *mut VirtAddr,
				borrowed.prio,
				borrowed.status,
			)
		};

		// do we have a task, which is ready?
		let mut next_task;
		if current_status == TaskStatus::Running {
			next_task = self.ready_queue.pop_with_prio(current_prio);
		} else {
			next_task = self.ready_queue.pop();
		}

		if next_task.is_none()
			&& current_status != TaskStatus::Running
			&& current_status != TaskStatus::Idle
		{
			debug!("Switch to idle task");
			// current task isn't able to run and no other task available
			// => switch to the idle task
			next_task = Some(self.idle_task.clone());
		}

		if let Some(new_task) = next_task {
			let (new_id, new_stack_pointer) = {
				let mut borrowed = new_task.borrow_mut();
				borrowed.status = TaskStatus::Running;
				(borrowed.id, borrowed.last_stack_pointer)
			};

			if current_status == TaskStatus::Running {
				debug!("Add task {} to ready queue", current_id);
				self.current_task.borrow_mut().status = TaskStatus::Ready;
				self.ready_queue.push(self.current_task.clone());
			} else if current_status == TaskStatus::Finished {
				debug!("Task {} finished", current_id);
				self.current_task.borrow_mut().status = TaskStatus::Invalid;
				// release the task later, because the stack is required
				// to call the function "switch"
				// => push id to a queue and release the task later
				self.finished_tasks.push_back(current_id);
			}

			debug!(
				"Switching task from {} to {} (stack {:#X} => {:#X})",
				current_id,
				new_id,
				unsafe { *current_stack_pointer },
				new_stack_pointer
			);

			self.current_task = new_task;

			unsafe {
				switch(current_stack_pointer, new_stack_pointer);
			}
		}
	}

	pub fn reschedule(&mut self) {
		irqsave(|| self.schedule());
	}
}
