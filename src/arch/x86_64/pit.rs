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
use logging::*;
use arch::processor::*;
use cpuio::outb;
use x86::time::rdtsc;

const CLOCK_TICK_RATE: u32 = 1193182u32; /* 8254 chip's internal oscillator frequency */

unsafe fn wait_some_time() {
 	let start = rdtsc();

	mb();
	while rdtsc() - start < 1000000 {
		mb();
	}
}

// initialize the Programmable Interrupt controller
pub fn init()
{
	debug!("initialize timer");

	let latch = ((CLOCK_TICK_RATE + TIMER_FREQ/2) / TIMER_FREQ) as u16;

	unsafe {
		/*
		 * Port 0x43 is for initializing the PIT:
		 *
		 * 0x34 means the following:
		 * 0b...     (step-by-step binary representation)
		 * ...  00  - channel 0
		 * ...  11  - write two values to counter register:
		 *            first low-, then high-byte
		 * ... 010  - mode number 2: "rate generator" / frequency divider
		 * ...   0  - binary counter (the alternative is BCD)
		 */
		outb(0x34, 0x43);

		wait_some_time();

		/* Port 0x40 is for the counter register of channel 0 */

		outb((latch & 0xFF) as u8, 0x40);   /* low byte  */

		wait_some_time();

		outb((latch >> 8) as u8, 0x40);     /* high byte */
	}
}
