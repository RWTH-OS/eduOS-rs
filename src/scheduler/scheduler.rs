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

use scheduler::task::*;
use consts::*;
use logging::*;
use alloc::VecDeque;

extern {
    pub fn switch(old_stack: *mut u64, new_stack: u64);
}

#[derive(Debug)]
pub enum SchedulerError {
    TooManyTasks
}

#[derive(Debug)]
pub struct Scheduler {
	pub current_task: TaskId,
	pub ready_queue: Option<VecDeque<TaskId>>,
	pub task_table: [Task; MAX_TASKS]
}

impl Scheduler {
	pub const fn new() -> Scheduler {
		Scheduler {
			current_task: TaskId::from(0),
			ready_queue: None,
			task_table: [Task::new(); MAX_TASKS]
		}
	}

	pub fn spawn(&mut self, func: extern fn()) -> Result<TaskId, SchedulerError> {
		for i in 0..MAX_TASKS {
			if self.task_table[i].status == TaskStatus::TaskInvalid {
				self.task_table[i].status = TaskStatus::TaskReady;
				// TaskID == Position in our task table
				self.task_table[i].id = TaskId::from(i);
				self.task_table[i].create_stack_frame(func);
				self.ready_queue.as_mut().unwrap().push_back(TaskId::from(i));

				info!("create task with id {}", self.task_table[i].id);

				return Ok(self.task_table[i].id);
			}
		}

		Err(SchedulerError::TooManyTasks)
	}

	pub fn exit(&mut self) {
		if self.task_table[self.current_task.into()].status != TaskStatus::TaskIdle {
			info!("finish task with id {}", self.task_table[self.current_task.into()].id);
			self.task_table[self.current_task.into()].status = TaskStatus::TaskFinished;
		}

		self.reschedule();
	}

	#[inline(always)]
	pub fn get_current_taskid(&self) -> TaskId {
		self.current_task
	}

	pub fn schedule(&mut self) {
		let old_task: TaskId = self.current_task;

		match self.ready_queue.as_mut().unwrap().pop_front() {
			None => if self.task_table[self.current_task.into()].status != TaskStatus::TaskRunning {
				// current task isn't able to run, no other task available => switch to the idle task
				self.current_task = TaskId::from(0);
			},
			Some(id) => self.current_task = id
		}

		if old_task != self.current_task {
				if self.task_table[self.current_task.into()].status != TaskStatus::TaskIdle {
					self.task_table[self.current_task.into()].status = TaskStatus::TaskRunning;
				}

				if self.task_table[old_task.into()].status == TaskStatus::TaskRunning {
					self.task_table[old_task.into()].status = TaskStatus::TaskReady;
					self.ready_queue.as_mut().unwrap().push_back(old_task);
				} else if self.task_table[old_task.into()].status == TaskStatus::TaskFinished {
					self.task_table[old_task.into()].status = TaskStatus::TaskInvalid;
				}

				debug!("switch task from {} to {}", old_task, self.current_task);
				unsafe {
					switch(&mut self.task_table[old_task.into()].last_stack_pointer,
						self.task_table[self.current_task.into()].last_stack_pointer);
				}
		}
	}

	#[inline(always)]
	pub fn reschedule(&mut self) {
		self.schedule();
	}
}
