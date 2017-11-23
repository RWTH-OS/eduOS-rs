// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// MIT License
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use consts::*;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::ptr::Shared;
use scheduler::task::*;
use arch::irq::{irq_nested_enable,irq_nested_disable};
use arch::replace_boot_stack;
use logging::*;
use synch::spinlock::*;
use alloc::VecDeque;
use alloc::boxed::Box;
use alloc::btree_map::*;

static TID_COUNTER: AtomicUsize = AtomicUsize::new(0);

extern {
	pub fn switch(old_stack: *const usize, new_stack: usize);
}

pub struct Scheduler {
	/// task which is currently running
	current_task: Shared<Task>,
	/// idle task
	idle_task: Shared<Task>,
	/// queue of tasks, which are finished and can be released
	finished_tasks: SpinlockIrqSave<VecDeque<TaskId>>,
	/// map between task id and task control block
	tasks: SpinlockIrqSave<BTreeMap<TaskId, Shared<Task>>>,
	/// number of tasks managed by the scheduler
	no_tasks: AtomicUsize
}

impl Scheduler {
	/// Create a new scheduler
	pub fn new() -> Scheduler {
		let tid = TaskId::from(0);

		// boot task is implicitly task 0 and and the idle task of core 0
		let idle_box = Box::new(Task::new(tid, TaskStatus::TaskIdle, LOW_PRIO));
		unsafe {

			let rsp = (*idle_box.stack).bottom();
			let ist = (*idle_box.ist).bottom();

			// replace temporary boot stack by the kernel stack of the boot task
			replace_boot_stack(rsp, ist);
		}
		let mut idle_task = unsafe { Shared::new_unchecked(Box::into_raw(idle_box)) };

		let s = Scheduler {
			current_task: idle_task,
			idle_task: idle_task,
			finished_tasks: SpinlockIrqSave::new(VecDeque::new()),
			tasks: SpinlockIrqSave::new(BTreeMap::new()),
			no_tasks: AtomicUsize::new(0)
		};

		let tid = s.get_tid();
		s.tasks.lock().insert(tid, idle_task);

		// consume running boot task as idle task
		unsafe {
			idle_task.as_mut().id = tid;
			idle_task.as_mut().status = TaskStatus::TaskRunning;
		}

		s
	}

	fn get_tid(&self) -> TaskId {
		loop {
			let id = TaskId::from(TID_COUNTER.fetch_add(1, Ordering::SeqCst));

			if self.tasks.lock().contains_key(&id) == false {
				return id;
			}
		}
	}

	/// Spawn a new task
	pub unsafe fn spawn(&mut self, func: extern fn(), base_prio: Priority) -> TaskId {
		let tid: TaskId;

		// do we have finished a task? => reuse it
		match self.finished_tasks.lock().pop_front() {
			None => {
				debug!("create new task control block");
				tid = self.get_tid();
				let mut task = Box::new(Task::new(tid, TaskStatus::TaskReady, base_prio));

				task.create_stack_frame(func);

				let shared_task = &mut Shared::new_unchecked(Box::into_raw(task));
				self.tasks.lock().insert(tid, *shared_task);
			},
			Some(id) => {
				debug!("resuse existing task control block");

				tid = id;
				match self.tasks.lock().get_mut(&tid) {
					Some(task) => {
						// reset old task and setup stack frame
						task.as_mut().status = TaskStatus::TaskReady;
						task.as_mut().base_prio = base_prio;
						task.as_mut().last_stack_pointer = 0;

						task.as_mut().create_stack_frame(func);
					},
					None => panic!("didn't find task")
				}
			}
		}

		info!("create task with id {}", tid);

		// update the number of tasks
		self.no_tasks.fetch_add(1, Ordering::SeqCst);

		tid
	}

	/// Terminate the current task
	pub unsafe fn exit(&mut self) -> ! {
		if self.current_task.as_ref().status != TaskStatus::TaskIdle {
			info!("finish task with id {}", self.current_task.as_ref().id);
			self.current_task.as_mut().status = TaskStatus::TaskFinished;
			// update the number of tasks
			self.no_tasks.fetch_sub(1, Ordering::SeqCst);
		} else {
			panic!("unable to terminate idle task");
		}

		self.reschedule();

		// we should never reach this point
		panic!("exit failed!")
	}

	pub unsafe fn abort(&mut self) -> ! {
			if self.current_task.as_ref().status != TaskStatus::TaskIdle {
				info!("abort task with id {}", self.current_task.as_ref().id);
				self.current_task.as_mut().status = TaskStatus::TaskFinished;
				// update the number of tasks
				self.no_tasks.fetch_sub(1, Ordering::SeqCst);
			} else {
				panic!("unable to terminate idle task");
			}

			self.reschedule();

			// we should never reach this point
			panic!("abort failed!");
	}

	pub fn number_of_tasks(&self) -> usize {
		self.no_tasks.load(Ordering::SeqCst)
	}

	/// Block the current task
	pub unsafe fn block_current_task(&mut self) -> Shared<Task> {
		if self.current_task.as_ref().status == TaskStatus::TaskRunning {
			debug!("block task {}", self.current_task.as_ref().id);

			self.current_task.as_mut().status = TaskStatus::TaskBlocked;
			return self.current_task;
		} else {
			panic!("unable to block task {} with status {:?}",
			       self.current_task.as_ref().id,
			       self.current_task.as_ref().status);
		}
	}

	/// Wakeup a blocked task
	pub unsafe fn wakeup_task(&mut self, mut task: Shared<Task>) {
		if task.as_ref().status == TaskStatus::TaskBlocked {
			debug!("wakeup task {}", task.as_ref().id);

			task.as_mut().status = TaskStatus::TaskReady;
		}
	}

	/// Determines the id of the current task
	#[inline(always)]
	pub fn get_current_taskid(&self) -> TaskId {
		unsafe { self.current_task.as_ref().id }
	}

	/// Determines the start address of the stack
	#[inline(always)]
	pub fn get_current_stack(&self) -> (usize, usize) {
		unsafe {
			((*self.current_task.as_ref().stack).bottom(), (*self.current_task.as_ref().ist).bottom())
		}
	}

	/// Determines the priority of the current task
	#[inline(always)]
	pub fn get_current_priority(&self) -> Priority {
		unsafe { self.current_task.as_ref().prio() }
	}

	/// Determines the priority of the task with the 'tid'
	pub fn get_priority(&self, tid: TaskId) -> Priority {
		let mut prio: Priority = NORMAL_PRIO;

		match self.tasks.lock().get(&tid) {
			Some(task) => prio = unsafe { task.as_ref().prio() },
			None => { info!("didn't find current task"); }
		}

		prio
	}

	unsafe fn get_next_task(&mut self) -> Shared<Task> {
		// candidate with the highest priority found yet
		let mut candidate: Shared<Task> = self.idle_task;

		// search for ready task with highest priority
		for (_id, task) in self.tasks.lock().iter() {
			if (task.as_ref().status == TaskStatus::TaskReady)
			    & (task.as_ref().prio() < candidate.as_ref().prio()) { // inverse comparison
				candidate = *task;
			}
		}

		candidate
	}

	pub unsafe fn schedule(&mut self) {
		// update status of current task
		if self.current_task.as_ref().status == TaskStatus::TaskRunning {
			self.current_task.as_mut().status = TaskStatus::TaskReady;
			self.current_task.as_mut().penalty += SCHEDULING_PENALTY;
		} else if self.current_task.as_ref().status == TaskStatus::TaskFinished {
			self.current_task.as_mut().status = TaskStatus::TaskInvalid;
			// release the task later, because the stack is required
			// to call the function "switch"
			// => push id to a queue and release the task later
			self.finished_tasks.lock().push_back(self.current_task.as_mut().id);
		}

		// update penalties
		for (_id, task) in self.tasks.lock().iter_mut() {
			task.as_mut().penalty /= 2;
		}

		let mut next_task = self.get_next_task();

		// return early if no need for switch
		if next_task.as_ref().id == self.current_task.as_ref().id {
			debug!("no need to switch");
			return;
		}

		debug!("switch task from {} to {}", self.current_task.as_mut().id, next_task.as_ref().id);

		next_task.as_mut().status = TaskStatus::TaskRunning;

		let next_stack_pointer = next_task.as_ref().last_stack_pointer;
		let old_stack_pointer = &self.current_task.as_ref().last_stack_pointer as *const usize;

		self.current_task = Shared::<Task>::from(next_task);

		switch(old_stack_pointer, next_stack_pointer);
	}

	/// Check if a finisched task could be deleted.
	unsafe fn cleanup_tasks(&mut self)
	{
		// do we have finished tasks? => drop first tasks => deallocate implicitly the stack
		match self.finished_tasks.lock().pop_front() {
			Some(id) => {
				match self.tasks.lock().remove(&id) {
					Some(task) => drop(Box::from_raw(task.as_ptr())),
					None => info!("unable to drop task {}", id)
				}
			},
			None => {}
	 	}
	}

	/// Triggers the scheduler to reschedule the tasks
	#[inline(always)]
	pub unsafe fn reschedule(&mut self) {
		// someone want to give up the CPU
		// => we have time to cleanup the system
		self.cleanup_tasks();

		let flags = irq_nested_disable();
		self.schedule();
		irq_nested_enable(flags);
	}
}
