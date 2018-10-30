// Copyright (c) 2017-2018 Stefan Lankes, RWTH Aachen University
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

use alloc::rc::Rc;
use alloc::collections::{BTreeMap, VecDeque};
use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering};
use scheduler::task::*;
use logging::*;

static NO_TASKS: AtomicU32 = AtomicU32::new(0);
static TID_COUNTER: AtomicU32 = AtomicU32::new(0);

extern {
    pub fn switch(old_stack: *mut usize, new_stack: usize);
}

pub struct Scheduler {
	/// task id which is currently running
	current_task:  Rc<RefCell<Task>>,
	/// task id of the idle task
	idle_task:  Rc<RefCell<Task>>,
	/// queue of tasks, which are ready
	ready_queue: TaskQueue,
	/// queue of tasks, which are finished and can be released
	finished_tasks: VecDeque<TaskId>,
	// map between task id and task controll block
	tasks: BTreeMap<TaskId, Rc<RefCell<Task>>>
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
			ready_queue: TaskQueue::new(),
			finished_tasks: VecDeque::<TaskId>::new(),
			tasks: tasks
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

	pub fn spawn(&mut self, func: extern fn()) -> TaskId {
		// Create the new task.
		let tid = self.get_tid();
		let task = Rc::new(RefCell::new(Task::new(tid, TaskStatus::TaskReady)));

		task.borrow_mut().create_stack_frame(func);

		// Add it to the task lists.
		self.ready_queue.push(task.clone());
		self.tasks.insert(tid, task);
		NO_TASKS.fetch_add(1, Ordering::SeqCst);

		info!("Creating task {}", tid);

		tid
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

	pub fn schedule(&mut self) {
		// do we have finished tasks? => drop tasks => deallocate implicitly the stack
		match self.finished_tasks.pop_front() {
			Some(id) => {
				if self.tasks.remove(&id).is_none() == true {
					info!("Unable to drop task {}", id);
				}
			},
			_ => {}
		}

		// Get information about the current task.
		let (old_id, old_stack_pointer, current_status) = {
			let mut borrowed = self.current_task.borrow_mut();
			(borrowed.id, &mut borrowed.last_stack_pointer as *mut usize, borrowed.status)
		};

		// do we have a task, which is ready?
		let mut next_task = self.ready_queue.pop();
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
					debug!("Add task {} to ready queue", old_id);
					self.current_task.borrow_mut().status = TaskStatus::TaskReady;
					self.ready_queue.push(self.current_task.clone());
				} else if current_status == TaskStatus::TaskFinished {
					debug!("Task {} finished", old_id);
					self.current_task.borrow_mut().status = TaskStatus::TaskInvalid;
					// release the task later, because the stack is required
					// to call the function "switch"
					// => push id to a queue and release the task later
					self.finished_tasks.push_back(old_id);
				}

				debug!("Switching task from {} to {} (stack {:#X} => {:#X})", old_id, new_id,
					unsafe { *old_stack_pointer }, new_stack_pointer);

				self.current_task = new_task;

				unsafe { switch(old_stack_pointer, new_stack_pointer); }
			},
			_ => {}
		}
	}

	pub fn reschedule(&mut self) {
		self.schedule();
	}
}
