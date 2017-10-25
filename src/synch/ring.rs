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

#![allow(dead_code)]

use scheduler::task::TaskId;
use consts::*;

#[derive(Debug)]
pub enum RingBufferError {
    BufferFull
}

/// Static ring buffer to realize a queue of TaskId. This queue is used to avoid a memory allocation
/// on the heap. Consequently, this queue could be also used for static variables.
#[derive(Debug)]
pub struct TaskRingBuffer
{
	buffer: [TaskId; MAX_TASKS],
	counter: usize,
	rpos: usize,
	wpos: usize
}

impl TaskRingBuffer
{
	pub const fn new() -> TaskRingBuffer
    {
        TaskRingBuffer
        {
			buffer: [TaskId::from(0); MAX_TASKS],
			counter: 0,
			rpos: 0,
			wpos: 0
        }
    }

	/// Appends an element to the back of the TaskQueue.
	pub fn push_back(&mut self, id: TaskId) -> Result<(), RingBufferError>
	{
		if self.counter >= MAX_TASKS {
			return Err(RingBufferError::BufferFull);
		}

		self.buffer[self.wpos] = id;
		self.wpos = (self.wpos + 1) % MAX_TASKS;
		self.counter += 1;

		Ok(())
	}

	/// Removes the first element and returns it, or None if the TaskQueue is empty.
	pub fn pop_front(&mut self) -> Option<TaskId>
	{
		if self.counter == 0 {
			return None;
		}

		let id = self.buffer[self.rpos];
		self.rpos = (self.rpos + 1) % MAX_TASKS;
		self.counter -= 1;

		Some(id)
	}

	/// Returns the number of elements the TaskQueue can hold
	pub const fn capacity(&self) -> usize
	{
		MAX_TASKS
	}
}
