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

use core::sync::atomic::{AtomicUsize, Ordering};
use core::ptr::Shared;
use scheduler::task::*;
use arch::irq::{irq_nested_enable,irq_nested_disable};
use logging::*;
use consts::*;
use synch::spinlock::*;
use alloc::VecDeque;
use alloc::boxed::Box;
use alloc::btree_map::*;

static TID_COUNTER: AtomicUsize = AtomicUsize::new(0);

extern {
	pub fn switch(old_stack: *const u64, new_stack: u64);

	/// The boot loader initialize a stack, which is later also required to
	/// to boot other core. Consequently, the kernel has to replace with this
	/// function the boot stack by a new one.
	pub fn replace_boot_stack(stack_bottom: usize);
}

pub struct Scheduler {
	/// task id which is currently running
	current_task: Shared<Task>,
	/// id of the idle task
	idle_task: Shared<Task>,
	/// queues of tasks, which are ready
	ready_queues: SpinlockIrqSave<[TaskQueue; NO_PRIORITIES]>,
	/// queue of tasks, which are finished and can be released
	finished_tasks: SpinlockIrqSave<Option<VecDeque<TaskId>>>,
	/// map between task id and task controll block
	tasks: SpinlockIrqSave<Option<BTreeMap<TaskId, Shared<Task>>>>
}

impl Scheduler {
	pub const fn new() -> Scheduler {
		Scheduler {
			// I know that this is unsafe. But I know also that I initialize
			// the Scheduler (with add_idle_task correctly) before the system schedules task.
			current_task: unsafe { Shared::new_unchecked(0 as *mut Task) },
			idle_task: unsafe { Shared::new_unchecked(0 as *mut Task) },
			ready_queues: SpinlockIrqSave::new([TaskQueue::new(); NO_PRIORITIES]),
			finished_tasks: SpinlockIrqSave::new(None),
			tasks: SpinlockIrqSave::new(None)
		}
	}

	fn get_tid(&self) -> TaskId {
		loop {
			let id = TaskId::from(TID_COUNTER.fetch_add(1, Ordering::SeqCst));

			if self.tasks.lock().as_ref().unwrap().contains_key(&id) == false {
				return id;
			}
		}
	}

	pub unsafe fn add_idle_task(&mut self) {
		// idle task is the first task for the scheduler => initialize queues and btree

		// initialize vector of queues
		*self.finished_tasks.lock() = Some(VecDeque::new());
		*self.tasks.lock() = Some(BTreeMap::new());
		let tid = self.get_tid();

		// boot task is implicitly task 0 and and the idle task of core 0
		let idle_box = Box::new(Task::new(tid, TaskStatus::TaskIdle, LOW_PRIO));
		let bottom = (*idle_box.stack).bottom();
		let idle_shared = Shared::new_unchecked(Box::into_raw(idle_box));

		self.idle_task = idle_shared;
		self.current_task = self.idle_task;

		// replace temporary boot stack by the kernel stack of the boot task
		replace_boot_stack(bottom);

		self.tasks.lock().as_mut().unwrap().insert(tid, idle_shared);
	}

	pub unsafe fn spawn(&mut self, func: extern fn(), prio: Priority) -> TaskId {
		let tid: TaskId;

		// do we have finished a task? => reuse it
		match self.finished_tasks.lock().as_mut().unwrap().pop_front() {
			None => {
				debug!("create new task control block");
				tid = self.get_tid();
				let mut task = Box::new(Task::new(tid, TaskStatus::TaskReady, prio));

				task.create_stack_frame(func);

				let shared_task = &mut Shared::new_unchecked(Box::into_raw(task));
				self.ready_queues.lock()[prio.into() as usize].push_back(shared_task);
				self.tasks.lock().as_mut().unwrap().insert(tid, *shared_task);
			},
			Some(id) => {
				debug!("resuse existing task control block");

				tid = id;
				match self.tasks.lock().as_mut().unwrap().get_mut(&tid) {
					Some(task) => {
						// reset old task and setup stack frame
						task.as_mut().status = TaskStatus::TaskReady;
						task.as_mut().prio = prio;
						task.as_mut().last_stack_pointer = 0;

						task.as_mut().create_stack_frame(func);

						self.ready_queues.lock()[prio.into() as usize].push_back(task);
					},
					None => panic!("didn't find task")
				}
			}
		}

		info!("create task with id {}", tid);

		tid
	}

	pub unsafe fn exit(&mut self) {
		if self.current_task.as_ref().status != TaskStatus::TaskIdle {
			info!("finish task with id {}", self.current_task.as_ref().id);
			self.current_task.as_mut().status = TaskStatus::TaskFinished;
		} else {
			panic!("unable to terminate idle task");
		}

		self.reschedule();
	}

	pub unsafe fn block_current_task(&mut self) -> Shared<Task> {
		if self.current_task.as_ref().status == TaskStatus::TaskRunning {
			debug!("block task {}", self.current_task.as_ref().id);

			self.current_task.as_mut().status = TaskStatus::TaskBlocked;
			return self.current_task;
		} else {
			panic!("unable to block task {}", self.current_task.as_ref().id);
		}
	}

	pub unsafe fn wakeup_task(&mut self, mut task: Shared<Task>) {
		if task.as_ref().status == TaskStatus::TaskBlocked {
			let prio = task.as_ref().prio;

			debug!("wakeup task {}", task.as_ref().id);

			task.as_mut().status = TaskStatus::TaskReady;
			self.ready_queues.lock()[prio.into() as usize]
				.push_back(&mut Shared::new_unchecked(task.as_mut()));
		}
	}

	#[inline(always)]
	pub fn get_current_taskid(&self) -> TaskId {
		unsafe { self.current_task.as_ref().id }
	}

	#[inline(always)]
	pub fn get_current_priority(&self) -> Priority {
		unsafe { self.current_task.as_ref().prio }
	}

	pub fn get_priority(&self, tid: TaskId) -> Priority {
		let mut prio: Priority = NORMAL_PRIO;

		match self.tasks.lock().as_ref().unwrap().get(&tid) {
			Some(task) => prio = unsafe { task.as_ref().prio },
			None => { info!("didn't find current task"); }
		}

		prio
	}

	unsafe fn get_next_task(&mut self) -> Option<Shared<Task>> {
		let mut prio = NO_PRIORITIES as usize;
		let status: TaskStatus;

		// if the current task is runable, check only if a task with
		// higher priority is available
		if self.current_task.as_ref().status == TaskStatus::TaskRunning {
			prio = self.current_task.as_ref().prio.into() as usize + 1;
		}
		status = self.current_task.as_ref().status;

		let mut guard = self.ready_queues.lock();

		for i in 0..prio {
			match guard[i].pop_front() {
				Some(mut task) => {
					task.as_mut().status = TaskStatus::TaskRunning;
					return Some(task)
				},
				None => {}
			}
		}

		if status != TaskStatus::TaskRunning && status != TaskStatus::TaskIdle {
			// current task isn't able to run and no other task available
			// => switch to the idle task
			Some(self.idle_task)
		} else {
			None
		}
	}

	pub unsafe fn schedule(&mut self) {
		// do we have a task, which is ready?
		match self.get_next_task() {
			Some(next_task) => {
				let old_id: TaskId = self.current_task.as_ref().id;

				if self.current_task.as_ref().status == TaskStatus::TaskRunning {
					self.current_task.as_mut().status = TaskStatus::TaskReady;
					self.ready_queues.lock()[self.current_task.as_ref().prio.into() as usize]
						.push_back(&mut self.current_task);
				} else if self.current_task.as_ref().status == TaskStatus::TaskFinished {
					self.current_task.as_mut().status = TaskStatus::TaskInvalid;
					// release the task later, because the stack is required
					// to call the function "switch"
					// => push id to a queue and release the task later
					self.finished_tasks.lock().as_mut().unwrap().push_back(old_id);
				}

				let next_stack_pointer = next_task.as_ref().last_stack_pointer;
				let old_stack_pointer = &self.current_task.as_ref().last_stack_pointer as *const u64;

				self.current_task = next_task;

				debug!("switch task from {} to {}", old_id, next_task.as_ref().id);

				switch(old_stack_pointer, next_stack_pointer);
			},
			None => {}
		}
	}

	unsafe fn cleanup_tasks(&mut self)
	{
		// do we have finished tasks? => drop first tasks => deallocate implicitly the stack
		match self.finished_tasks.lock().as_mut().unwrap().pop_front() {
			Some(id) => {
				match self.tasks.lock().as_mut().unwrap().remove(&id) {
					Some(task) => drop(Box::from_raw(task.as_ptr())),
					None => info!("unable to drop task {}", id)
				}
			},
			None => {}
	 	}
	}

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
