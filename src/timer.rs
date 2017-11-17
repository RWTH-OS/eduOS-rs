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
use scheduler::task::*;
use scheduler::*;
use synch::spinlock::*;
use arch::processor::udelay;
use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::binary_heap::BinaryHeap;
use core::marker::Sync;

lazy_static! {
	pub static ref TIMER: Timer = Timer::new();
}

pub struct Timer {
	ticks: AtomicUsize,
	waiting: SpinlockIrqSave<BinaryHeap<WaitingTask>>
}

unsafe impl Sync for Timer {}

impl Timer {
	pub fn new() -> Timer {
		Timer {
			ticks: AtomicUsize::new(0),
			waiting: SpinlockIrqSave::new(BinaryHeap::new())
		}
	}

	pub fn increment(&self) {
		let tick = self.ticks.fetch_add(1, Ordering::SeqCst) + 1;
		let mut guard = self.waiting.lock();

		loop {
			// do we have a task waiting?
			match guard.peek() {
				None => { return; }
				Some(waiting_task) => {
					// do we have to wake up the tasks?
					if waiting_task.wakeup_time > tick {
						return;
					} else {
						wakeup_task(waiting_task.task);
					}
				}
			}

			// if we reach this point, we already wakeup the tasks
			// => remove the task from the heap
			guard.pop();
		}
	}

	pub fn get_clock_tick(&self) -> usize {
		self.ticks.load(Ordering::SeqCst)
	}

	pub fn wait(&self, count: usize)
	{
		let eticks: usize = self.ticks.load(Ordering::SeqCst) + count;

		self.waiting.lock().push(WaitingTask::new(block_current_task(), eticks));

		// switch to the next task
		reschedule();
	}

	pub fn msleep(&self, ms: u32)
	{
		if (ms * TIMER_FREQ) / 1000 > 0 {
			// roundup timeout by adding 999
			self.wait(((ms * TIMER_FREQ + 999) / 1000) as usize);
		} else if ms > 0 {
			// time is too small => busy waiting
			udelay((ms as u64) * 1000u64);
		}
	}
}
