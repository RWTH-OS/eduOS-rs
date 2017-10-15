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
use core::cell::UnsafeCell;
use core::marker::Sync;
use core::fmt;
use core::ops::{Drop, Deref, DerefMut};

use scheduler::task::TaskId;
use scheduler::get_current_taskid;
use consts::*;
use arch;

/// This type provides a lock based on busy waiting to realize mutual exclusion of tasks.
///
/// # Description
///
/// This structure behaves a lot like a normal Mutex. There are some differences:
///
/// - By using busy waiting, it can be used outside the runtime.
/// - A task can repeatedly lock the same object, either via multiple calls to Spinlock via nested
///   lock statements. The object is then unlocked when a corresponding number of Spinlocks unlock
///   statements have executed.
/// - It is a so called ticket lock ([https://en.wikipedia.org/wiki/Ticket_lock](https://en.wikipedia.org/wiki/Ticket_lock))
///   and completly fair.
///
/// The interface is derived from [https://mvdnes.github.io/rust-docs/spin-rs/spin/index.html](https://mvdnes.github.io/rust-docs/spin-rs/spin/index.html).
///
/// # Simple examples
///
/// ```
/// let spinlock = synch::Spinlock::new(0);
///
/// // Modify the data
/// {
///     let mut data = spinlock.lock();
///     *data = 2;
/// }
///
/// // Read the data
/// let answer =
/// {
///     let data = spinlock.lock();
///     *data
/// };
///
/// assert_eq!(answer, 2);
/// ```
pub struct Spinlock<T: ?Sized>
{
    queue: AtomicUsize,
	dequeue: AtomicUsize,
	owner: TaskId,
	counter: usize,
    data: UnsafeCell<T>,
}

/// A guard to which the protected data can be accessed
///
/// When the guard falls out of scope it will release the lock.
pub struct SpinlockGuard<'a, T: ?Sized + 'a>
{
	//queue: &'a AtomicUsize,
	dequeue: &'a mut AtomicUsize,
	owner: &'a mut TaskId,
	counter: &'a mut usize,
    data: &'a mut T,
}

// Same unsafe impls as `std::sync::Mutex`
unsafe impl<T: ?Sized + Send> Sync for Spinlock<T> {}
unsafe impl<T: ?Sized + Send> Send for Spinlock<T> {}

impl<T> Spinlock<T>
{
	pub const fn new(user_data: T) -> Spinlock<T>
    {
        Spinlock
        {
			queue: AtomicUsize::new(0),
			dequeue: AtomicUsize::new(1),
			owner: TaskId::from(MAX_TASKS),
            counter: 0,
            data: UnsafeCell::new(user_data),
        }
    }

	/// Consumes this mutex, returning the underlying data.
    pub fn into_inner(self) -> T {
        // We know statically that there are no outstanding references to
        // `self` so there's no need to lock.
        let Spinlock { data, .. } = self;
        unsafe { data.into_inner() }
    }
}

impl<T: ?Sized> Spinlock<T>
{
	fn obtain_lock(&mut self) {
		let tid = get_current_taskid();

		if self.owner == tid {
			self.counter = self.counter + 1;
			return;
		}

		let ticket = self.queue.fetch_add(1, Ordering::SeqCst) + 1;
		while self.dequeue.load(Ordering::SeqCst) != ticket {
			arch::processor::pause();
		}

		self.owner = tid;
		self.counter = 1;
	}

	pub fn lock(&mut self) -> SpinlockGuard<T>
    {
        self.obtain_lock();
        SpinlockGuard
        {
			//queue: &self.queue,
			dequeue: &mut self.dequeue,
			owner: &mut self.owner,
            counter: &mut self.counter,
            data: unsafe { &mut *self.data.get() },
        }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Spinlock<T>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
		write!(f, "owner: {} ", self.owner)?;
		write!(f, "counter: {} ", self.counter)?;
		write!(f, "queue: {} ", self.queue.load(Ordering::SeqCst))?;
		write!(f, "dequeue: {}", self.dequeue.load(Ordering::SeqCst))
    }
}

impl<T: ?Sized + Default> Default for Spinlock<T> {
    fn default() -> Spinlock<T> {
        Spinlock::new(Default::default())
    }
}

impl<'a, T: ?Sized> Deref for SpinlockGuard<'a, T>
{
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T { &*self.data }
}

impl<'a, T: ?Sized> DerefMut for SpinlockGuard<'a, T>
{
    fn deref_mut<'b>(&'b mut self) -> &'b mut T { &mut *self.data }
}

impl<'a, T: ?Sized> Drop for SpinlockGuard<'a, T>
{
    /// The dropping of the SpinlockGuard will release the lock it was created from.
    fn drop(&mut self)
    {
		*self.counter -= 1;
		if *self.counter == 0 {
			*self.owner = TaskId::from(MAX_TASKS);
			self.dequeue.fetch_add(1, Ordering::SeqCst);
		}
    }
}
