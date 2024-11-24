use crate::mm::linked_list;
use crate::synch::spinlock::Spinlock;
use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use core::cmp::{max, min};
use core::fmt;
use core::ptr::NonNull;

/// Minimal size of an allocated memory block
const MIN_ALLOC_SIZE: usize = 128;

#[derive(Debug)]
pub(crate) enum AllocatorError {
	OutOfMemory,
	TooBig,
}

#[repr(align(64))]
pub(crate) struct BuddySystem<const ORDER: usize> {
	free_list: [linked_list::LinkedList; ORDER],
}

impl<const ORDER: usize> BuddySystem<ORDER> {
	/// Constructs an empty buddy system.
	pub const fn new() -> Self {
		Self {
			free_list: [linked_list::LinkedList::new(); ORDER],
		}
	}

	/// Initialize buddy system. `start` specifies the
	/// start address of the heap, while `len` specifies#
	/// the heap size.
	pub unsafe fn init(&mut self, start: *mut u8, len: usize) {
		assert!((len & (len - 1)) == 0, "Heap size isn't a power of two");
		let order: usize = len.trailing_zeros().try_into().unwrap();
		assert!(order <= ORDER, "ORDER isn't large enough");

		unsafe {
			self.free_list[order].push(start as *mut usize);
		}
	}

	/// Allocates memory as described by the given `layout`.
	///
	/// Returns as result a pointer to newly-allocated memory,
	/// or an error, which describes the reason of the error.
	pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocatorError> {
		let size = max(
			layout.size().next_power_of_two(),
			max(layout.align(), MIN_ALLOC_SIZE),
		);
		let order: usize = size.trailing_zeros().try_into().unwrap();

		if order >= ORDER {
			// size of the memory allocation is too large
			return Err(AllocatorError::TooBig);
		}

		for i in order..self.free_list.len() {
			// Find the first non-empty list, which handles a block of
			// memory with a equal or a larger size as the requested size
			if !self.free_list[i].is_empty() {
				// Split larger blocks in two buddies
				for j in (order + 1..i + 1).rev() {
					if let Some(block) = self.free_list[j].pop() {
						unsafe {
							self.free_list[j - 1]
								.push((block as usize + (1 << (j - 1))) as *mut usize);
							self.free_list[j - 1].push(block);
						}
					} else {
						return Err(AllocatorError::OutOfMemory);
					}
				}

				if let Some(addr) = self.free_list[order].pop() {
					return Ok(NonNull::new(addr as *mut u8).unwrap());
				} else {
					return Err(AllocatorError::OutOfMemory);
				}
			}
		}

		Err(AllocatorError::OutOfMemory)
	}

	/// Deallocates the block of memory at the given `ptr` pointer with the given layout.
	pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
		let size = max(
			layout.size().next_power_of_two(),
			max(layout.align(), MIN_ALLOC_SIZE),
		);
		let order: usize = size.trailing_zeros().try_into().unwrap();

		unsafe {
			// add block to free list
			self.free_list[order].push(ptr.as_ptr() as *mut usize);

			// Try to merge two buddies to one buddy
			let mut current_ptr = ptr.as_ptr() as usize;
			let mut current_order = order;

			'outer: while current_order < self.free_list.len() - 1 {
				let block_size = 1 << current_order;

				// the list is unordered => check all nodes to find a buddy
				for block in self.free_list[current_order].iter_mut() {
					let buddy = block.value() as usize;
					if buddy == current_ptr + block_size || buddy == current_ptr - block_size {
						// remove current block from the list
						block.remove();
						// the first node of the list includes `ptr`
						self.free_list[current_order].pop().unwrap();
						// merge buddies
						current_ptr = min(current_ptr, buddy);
						current_order += 1;
						self.free_list[current_order].push(current_ptr as *mut usize);
						continue 'outer;
					}
				}

				// no buddy merged => leave while loop
				break;
			}
		}
	}
}

impl<const ORDER: usize> fmt::Debug for BuddySystem<ORDER> {
	/// Print all elements of the lists
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for order in (0..self.free_list.len()).rev() {
			if !self.free_list[order].is_empty() {
				write!(f, "Block size {:>8}: ", 1 << order)?;

				for block in self.free_list[order].iter() {
					write!(f, "0x{:x} ", block as usize)?;
				}

				writeln!(f)?;
			}
		}

		Ok(())
	}
}

/// A memory allocator that can be registered as default allocator through
/// the #[global_allocator] attribute.
pub(crate) struct LockedHeap<const ORDER: usize>(Spinlock<BuddySystem<ORDER>>);

impl<const ORDER: usize> LockedHeap<ORDER> {
	/// Constructs an empty buddy system
	pub const fn new() -> Self {
		LockedHeap(Spinlock::new(BuddySystem::<ORDER>::new()))
	}

	pub unsafe fn init(&self, start: *mut u8, len: usize) {
		unsafe {
			self.0.lock().init(start, len);
		}
	}
}

impl<const ORDER: usize> fmt::Debug for LockedHeap<ORDER> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.lock().fmt(f)
	}
}

unsafe impl<const ORDER: usize> GlobalAlloc for LockedHeap<ORDER> {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		self.0
			.lock()
			.alloc(layout)
			.ok()
			.map_or(core::ptr::null_mut(), |allocation| allocation.as_ptr())
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		self.0.lock().dealloc(NonNull::new_unchecked(ptr), layout)
	}
}
