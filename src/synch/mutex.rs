use crate::scheduler::task::*;
use crate::scheduler::{block_current_task, reschedule, wakeup_task};
use crate::synch::spinlock::*;
use core::cell::UnsafeCell;
use core::marker::Sync;
use core::ops::{Deref, DerefMut, Drop};

/// A mutual exclusion primitive useful for protecting shared data
///
/// This mutex will block threads waiting for the lock to become available. The
/// mutex can also be statically initialized or created via a `new`
/// constructor. Each mutex has a type parameter which represents the data that
/// it is protecting. The data can only be accessed through the RAII guards
/// returned from `lock` and `try_lock`, which guarantees that the data is only
/// ever accessed when the mutex is locked.
///
/// # Simple examples
///
/// ```
/// let mutex = synch::Mutex::new(0);
///
/// // Modify the data
/// {
///     let mut data = mutex.lock();
///     *data = 2;
/// }
///
/// // Read the data
/// let answer =
/// {
///     let data = mutex.lock();
///     *data
/// };
///
/// assert_eq!(answer, 2);
/// ```
pub struct Mutex<T: ?Sized> {
	/// in principle a binary semaphore
	value: Spinlock<bool>,
	/// Priority queue of waiting tasks
	queue: Spinlock<PriorityTaskQueue>,
	/// protected data
	data: UnsafeCell<T>,
}

/// A guard to which the protected data can be accessed
///
/// When the guard falls out of scope it will release the lock.
pub struct MutexGuard<'a, T: ?Sized + 'a> {
	value: &'a Spinlock<bool>,
	queue: &'a Spinlock<PriorityTaskQueue>,
	data: &'a mut T,
}

// Same unsafe impls as `std::sync::Mutex`
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
	/// Creates a new semaphore with the initial count specified.
	///
	/// The count specified can be thought of as a number of resources, and a
	/// call to `acquire` or `access` will block until at least one resource is
	/// available. It is valid to initialize a semaphore with a negative count.
	pub const fn new(user_data: T) -> Mutex<T> {
		Mutex {
			value: Spinlock::new(true),
			queue: Spinlock::new(PriorityTaskQueue::new()),
			data: UnsafeCell::new(user_data),
		}
	}

	/// Consumes this mutex, returning the underlying data.
	pub fn into_inner(self) -> T {
		// We know statically that there are no outstanding references to
		// `self` so there's no need to lock.
		let Mutex { data, .. } = self;
		data.into_inner()
	}
}

impl<T: ?Sized> Mutex<T> {
	fn obtain_lock(&self) {
		loop {
			let mut count = self.value.lock();

			if *count {
				*count = false;
				return;
			} else {
				self.queue.lock().push(block_current_task());
				// release lock
				drop(count);
				// switch to the next task
				reschedule();
			}
		}
	}

	pub fn lock(&self) -> MutexGuard<T> {
		self.obtain_lock();
		MutexGuard {
			value: &self.value,
			queue: &self.queue,
			data: unsafe { &mut *self.data.get() },
		}
	}
}

impl<T: Default> Default for Mutex<T> {
	fn default() -> Mutex<T> {
		Mutex::new(Default::default())
	}
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
	type Target = T;
	fn deref(&self) -> &T {
		&*self.data
	}
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		&mut *self.data
	}
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
	/// The dropping of the MutexGuard will release the lock it was created from.
	fn drop(&mut self) {
		let mut count = self.value.lock();
		*count = true;

		// try to wakeup next task
		if let Some(task) = self.queue.lock().pop() {
			wakeup_task(task);
		}
	}
}
