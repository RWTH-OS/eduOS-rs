use crate::arch;
use core::sync::atomic::{AtomicUsize, Ordering};
use lock_api::{GuardSend, RawMutex, RawMutexFair};

/// A [fair] [ticket lock].
///
/// [fair]: https://en.wikipedia.org/wiki/Unbounded_nondeterminism
/// [ticket lock]: https://en.wikipedia.org/wiki/Ticket_lock
pub struct RawSpinlock {
	queue: AtomicUsize,
	dequeue: AtomicUsize,
}

unsafe impl RawMutex for RawSpinlock {
	#[allow(clippy::declare_interior_mutable_const)]
	const INIT: Self = Self {
		queue: AtomicUsize::new(0),
		dequeue: AtomicUsize::new(0),
	};

	type GuardMarker = GuardSend;

	#[inline]
	fn lock(&self) {
		let ticket = self.queue.fetch_add(1, Ordering::Relaxed);
		while self.dequeue.load(Ordering::Acquire) != ticket {
			arch::processor::pause();
		}
	}

	#[inline]
	fn try_lock(&self) -> bool {
		let ticket = self
			.queue
			.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |ticket| {
				if self.dequeue.load(Ordering::Acquire) == ticket {
					Some(ticket + 1)
				} else {
					None
				}
			});

		ticket.is_ok()
	}

	#[inline]
	unsafe fn unlock(&self) {
		self.dequeue.fetch_add(1, Ordering::Release);
	}

	#[inline]
	fn is_locked(&self) -> bool {
		let ticket = self.dequeue.load(Ordering::Relaxed);
		self.dequeue.load(Ordering::Relaxed) != ticket
	}
}

unsafe impl RawMutexFair for RawSpinlock {
	#[inline]
	unsafe fn unlock_fair(&self) {
		unsafe { self.unlock() }
	}

	#[inline]
	unsafe fn bump(&self) {
		let ticket = self.queue.load(Ordering::Relaxed);
		let serving = self.dequeue.load(Ordering::Relaxed);
		if serving + 1 != ticket {
			unsafe {
				self.unlock_fair();
				self.lock();
			}
		}
	}
}

/// A [`lock_api::Mutex`] based on [`RawSpinlockMutex`].
pub type Spinlock<T> = lock_api::Mutex<RawSpinlock, T>;

/// A [`lock_api::MutexGuard`] based on [`RawSpinlockMutex`].
pub type SpinlockGuard<'a, T> = lock_api::MutexGuard<'a, RawSpinlock, T>;
