// Copyright (c) 2017-2018 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::arch::processor::lsb;
use crate::arch::switch;
use crate::consts::*;
use crate::errno::*;
use crate::logging::*;
use crate::scheduler::task::*;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::rc::Rc;
use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering};

static NO_TASKS: AtomicU32 = AtomicU32::new(0);
static TID_COUNTER: AtomicU32 = AtomicU32::new(0);

pub struct Scheduler {
	/// task id which is currently running
	current_task: Rc<RefCell<Task>>,
	/// task id of the idle task
	idle_task: Rc<RefCell<Task>>,
	/// queue of tasks, which are ready
	ready_queues: [TaskQueue; NO_PRIORITIES],
	/// Bitmap to show, which queue is uesed
	prio_bitmap: u64,
	/// queue of tasks, which are finished and can be released
	finished_tasks: VecDeque<TaskId>,
	// map between task id and task controll block
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
			ready_queues: Default::default(),
			prio_bitmap: 0,
			finished_tasks: VecDeque::<TaskId>::new(),
			tasks: tasks,
		}
	}

	fn get_tid(&self) -> TaskId {
		loop {
			let id = TaskId::from(TID_COUNTER.fetch_add(1, Ordering::SeqCst));

			if self.tasks.contains_key(&id) == false {
				return id;
			}
		}
	}

	pub fn spawn(&mut self, func: extern "C" fn(), prio: TaskPriority) -> Result<TaskId> {
		let prio_number = prio.into() as usize;

		if prio_number >= NO_PRIORITIES {
			return Err(Error::BadPriority);
		}

		// Create the new task.
		let tid = self.get_tid();
		let task = Rc::new(RefCell::new(Task::new(tid, TaskStatus::TaskReady, prio)));

		task.borrow_mut().create_stack_frame(func);

		// Add it to the task lists.
		self.ready_queues[prio_number].push(task.clone());
		self.prio_bitmap |= 1 << prio_number;
		self.tasks.insert(tid, task);
		NO_TASKS.fetch_add(1, Ordering::SeqCst);

		info!("Creating task {}", tid);

		Ok(tid)
	}

	pub fn exit(&mut self) {
		if self.current_task.borrow().status != TaskStatus::TaskIdle {
			info!("finish task with id {}", self.current_task.borrow().id);
			self.current_task.borrow_mut().status = TaskStatus::TaskFinished;
		} else {
			panic!("unable to terminate idle task");
		}

		self.reschedule();
	}

	pub fn get_current_taskid(&self) -> TaskId {
		self.current_task.borrow().id
	}

	// determine the next task, which is ready and priority is a greater than or equal to prio
	fn get_next_task(&mut self, prio: TaskPriority) -> Option<Rc<RefCell<Task>>> {
		let i = lsb(self.prio_bitmap);
		let mut task = None;

		if i <= prio.into() as u64 {
			task = self.ready_queues[i as usize].pop();

			// clear bitmap entry for the priority i if the queus is empty
			if self.ready_queues[i as usize].is_empty() == true {
				self.prio_bitmap &= !(1 << i);
			}
		}

		task
	}

	pub fn schedule(&mut self) {
		// do we have finished tasks? => drop tasks => deallocate implicitly the stack
		match self.finished_tasks.pop_front() {
			Some(id) => {
				if self.tasks.remove(&id).is_none() == true {
					info!("Unable to drop task {}", id);
				}
			}
			_ => {}
		}

		// Get information about the current task.
		let (current_id, current_stack_pointer, current_prio, current_status) = {
			let mut borrowed = self.current_task.borrow_mut();
			(
				borrowed.id,
				&mut borrowed.last_stack_pointer as *mut usize,
				borrowed.prio,
				borrowed.status,
			)
		};

		// do we have a task, which is ready?
		let mut next_task;
		if current_status == TaskStatus::TaskRunning {
			next_task = self.get_next_task(current_prio);
		} else {
			next_task = self.get_next_task(LOW_PRIORITY);
		}

		if next_task.is_none() == true {
			if current_status != TaskStatus::TaskRunning && current_status != TaskStatus::TaskIdle {
				debug!("Switch to idle task");
				// current task isn't able to run and no other task available
				// => switch to the idle task
				next_task = Some(self.idle_task.clone());
			}
		}

		match next_task {
			Some(new_task) => {
				let (new_id, new_stack_pointer) = {
					let mut borrowed = new_task.borrow_mut();
					borrowed.status = TaskStatus::TaskRunning;
					(borrowed.id, borrowed.last_stack_pointer)
				};

				if current_status == TaskStatus::TaskRunning {
					debug!("Add task {} to ready queue", current_id);
					self.current_task.borrow_mut().status = TaskStatus::TaskReady;
					self.ready_queues[current_prio.into() as usize].push(self.current_task.clone());
					self.prio_bitmap |= 1 << current_prio.into() as usize;
				} else if current_status == TaskStatus::TaskFinished {
					debug!("Task {} finished", current_id);
					self.current_task.borrow_mut().status = TaskStatus::TaskInvalid;
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

				switch(current_stack_pointer, new_stack_pointer);
			}
			_ => {}
		}
	}

	pub fn reschedule(&mut self) {
		self.schedule();
	}
}
