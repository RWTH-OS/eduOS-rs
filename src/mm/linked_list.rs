use core::marker::PhantomData;

#[derive(Copy, Clone, Debug)]
pub(crate) struct LinkedList {
	head: *mut usize,
}

unsafe impl Send for LinkedList {}

impl LinkedList {
	pub const fn new() -> Self {
		Self {
			head: core::ptr::null_mut(),
		}
	}

	/// check if the list is empty
	pub fn is_empty(&self) -> bool {
		self.head.is_null()
	}

	/// add address to the front of the list
	pub unsafe fn push(&mut self, item: *mut usize) {
		*item = self.head as usize;
		self.head = item;
	}

	/// Try to remove the first item in the list
	pub fn pop(&mut self) -> Option<*mut usize> {
		match self.is_empty() {
			true => None,
			false => {
				// Advance head pointer
				let item = self.head;
				self.head = unsafe { *item as *mut usize };
				Some(item)
			}
		}
	}

	/// Return an iterator over the items in the list
	pub fn iter(&self) -> Iter {
		Iter {
			curr: self.head,
			list: PhantomData,
		}
	}

	/// Return an mutable iterator over the items in the list
	pub fn iter_mut(&mut self) -> IterMut {
		IterMut {
			prev: &mut self.head as *mut *mut usize as *mut usize,
			curr: self.head,
			list: PhantomData,
		}
	}
}

/// A simple iterator for the linked list
pub(crate) struct Iter<'a> {
	curr: *mut usize,
	list: PhantomData<&'a LinkedList>,
}

impl<'a> Iterator for Iter<'a> {
	type Item = *mut usize;

	fn next(&mut self) -> Option<Self::Item> {
		if self.curr.is_null() {
			None
		} else {
			let item = self.curr;
			let next = unsafe { *item as *mut usize };
			self.curr = next;
			Some(item)
		}
	}
}

/// Represent a mutable node in `LinkedList`
pub(crate) struct ListNode {
	prev: *mut usize,
	curr: *mut usize,
}

impl ListNode {
	/// Remove the current node from the list
	pub fn remove(self) -> *mut usize {
		// Skip the current one
		unsafe {
			*(self.prev) = *(self.curr);
		}
		self.curr
	}

	/// Returns the pointed address
	pub fn value(&self) -> *mut usize {
		self.curr
	}
}

/// A mutable iterator over the linked list
pub(crate) struct IterMut<'a> {
	list: PhantomData<&'a mut LinkedList>,
	prev: *mut usize,
	curr: *mut usize,
}

impl<'a> Iterator for IterMut<'a> {
	type Item = ListNode;

	fn next(&mut self) -> Option<Self::Item> {
		if self.curr.is_null() {
			None
		} else {
			let res = ListNode {
				prev: self.prev,
				curr: self.curr,
			};
			self.prev = self.curr;
			self.curr = unsafe { *self.curr as *mut usize };
			Some(res)
		}
	}
}
