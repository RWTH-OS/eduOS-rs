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
use scheduler::*;
use arch::processor::udelay;
use core::sync::atomic::{AtomicUsize, Ordering, hint_core_should_pause};

pub static TIMER: Timer = Timer::new();

pub struct Timer {
	ticks: AtomicUsize
}

impl Timer {
	pub const fn new() -> Timer {
		Timer {
			ticks: AtomicUsize::new(0)
		}
	}

	pub fn increment(&self) {
		self.ticks.fetch_add(1, Ordering::SeqCst);
	}

	pub fn get_clock_tick(&self) -> usize {
		self.ticks.load(Ordering::SeqCst)
	}

	pub fn wait(&self, count: u64)
	{
		let eticks: u64 = self.ticks.load(Ordering::SeqCst) as u64 + count;

		/*
		 * This will continuously loop until the given time has
		 * been reached
		 */
		while (self.ticks.load(Ordering::SeqCst) as u64) < eticks {
			hint_core_should_pause();
			reschedule();
		}
	}

	pub fn msleep(&self, ms: u32)
	{
		if (ms * TIMER_FREQ) / 1000 > 0 {
			self.wait(((ms * TIMER_FREQ) / 1000) as u64);
		} else if ms > 0 {
			udelay((ms as u64) * 1000u64);
		}
	}
}
