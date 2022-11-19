// Copyright (c) 2017 Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::arch::mm::{PhysAddr, VirtAddr};
use crate::logging::*;
use alloc::collections::linked_list::LinkedList;
use core::cmp::Ordering;

pub struct FreeListEntry<
	T: core::marker::Copy
		+ core::cmp::PartialEq
		+ core::cmp::PartialOrd
		+ core::fmt::UpperHex
		+ core::ops::Sub
		+ core::ops::BitAnd<u64>
		+ core::ops::Add<usize>
		+ core::ops::AddAssign<u64>
		+ core::ops::Add<usize, Output = T>,
> {
	pub start: T,
	pub end: T,
}

impl<
		T: core::marker::Copy
			+ core::cmp::PartialEq
			+ core::cmp::PartialOrd
			+ core::fmt::UpperHex
			+ core::ops::Sub
			+ core::ops::BitAnd<u64>
			+ core::ops::Add<usize>
			+ core::ops::AddAssign<u64>
			+ core::ops::Add<usize, Output = T>,
	> FreeListEntry<T>
{
	pub const fn new(start: T, end: T) -> Self {
		FreeListEntry { start, end }
	}
}

pub struct FreeList<
	T: core::marker::Copy
		+ core::cmp::PartialEq
		+ core::cmp::PartialOrd
		+ core::fmt::UpperHex
		+ core::ops::Sub
		+ core::ops::BitAnd<u64>
		+ core::ops::Add<usize>
		+ core::ops::AddAssign<u64>
		+ core::ops::Add<usize, Output = T>,
> {
	pub list: LinkedList<FreeListEntry<T>>,
}

impl<
		T: core::marker::Copy
			+ core::cmp::PartialEq
			+ core::cmp::PartialOrd
			+ core::fmt::UpperHex
			+ core::ops::Sub
			+ core::ops::BitAnd<u64>
			+ core::ops::Add<usize>
			+ core::ops::AddAssign<u64>
			+ core::ops::Add<usize, Output = T>,
	> FreeList<T>
{
	pub const fn new() -> Self {
		Self {
			list: LinkedList::new(),
		}
	}

	pub fn deallocate(&mut self, address: T, size: usize) {
		debug!(
			"Deallocating {} bytes at {:#X} from Free List {:#X}",
			size, address, self as *const Self as usize
		);

		let end = address + size;
		let mut cursor = self.list.cursor_front_mut();

		while let Some(node) = cursor.current() {
			let (region_start, region_end) = (node.start, node.end);

			if region_start == end {
				// The deallocated memory extends this free memory region to the left.
				node.start = address;

				// Check if it can even reunite with the previous region.
				if let Some(prev_node) = cursor.peek_prev() {
					let prev_region_end = prev_node.end;

					if prev_region_end == address {
						// It can reunite, so let the current region span over the reunited region and move the duplicate node
						// into the pool for deletion or reuse.
						prev_node.end = region_end;
						cursor.remove_current();
					}
				}

				return;
			} else if region_end == address {
				node.end = end;

				// Check if it can even reunite with the next region.
				if let Some(next_node) = cursor.peek_next() {
					let next_region_start = next_node.start;

					if next_region_start == end {
						// It can reunite, so let the current region span over the reunited region and move the duplicate node
						// into the pool for deletion or reuse.
						next_node.start = region_start;
						cursor.remove_current();
					}
				}

				return;
			} else if end < region_start {
				// The deallocated memory does not extend any memory region and needs an own entry in the Free List.
				// Get that entry from the node pool.
				// We search the list from low to high addresses and insert us before the first entry that has a
				// higher address than us.
				let new_entry = FreeListEntry::new(address, end);
				cursor.insert_before(new_entry);
				return;
			}

			cursor.move_next();
		}

		// We could not find an entry with a higher address than us.
		// So we become the new last entry in the list. Get that entry from the node pool.
		let new_element = FreeListEntry::new(address, end);
		self.list.push_back(new_element);
	}
}

#[cfg(not(target_os = "none"))]
#[test]
fn add_element() {
	let mut freelist = FreeList::new();
	let entry = FreeListEntry::new(0x10000, 0x100000);

	freelist.list.push_back(entry);

	let mut cursor = freelist.list.cursor_front_mut();

	while let Some(node) = cursor.peek_next() {
		assert!(node.start != 0x1000);
		assert!(node.end != 0x10000);

		cursor.move_next();
	}
}

#[cfg(not(target_os = "none"))]
#[test]
fn allocate() {
	let mut freelist = FreeList::new();
	let entry = FreeListEntry::new(0x10000, 0x100000);

	freelist.list.push_back(entry);
	let addr = freelist.allocate(0x1000, None);

	assert_eq!(addr.unwrap(), 0x10000);

	let mut cursor = freelist.list.cursor_front_mut();
	while let Some(node) = cursor.current() {
		assert_eq!(node.start, 0x11000);
		assert_eq!(node.end, 0x100000);

		cursor.move_next();
	}

	let addr = freelist.allocate(0x1000, Some(0x2000));
	let mut cursor = freelist.list.cursor_front_mut();
	assert!(cursor.current().is_some());
	if let Some(node) = cursor.current() {
		assert_eq!(node.start, 0x11000);
	}

	cursor.move_next();
	assert!(cursor.current().is_some());
	if let Some(node) = cursor.current() {
		assert_eq!(node.start, 0x13000);
	}
}

#[cfg(not(target_os = "none"))]
#[test]
fn deallocate() {
	let mut freelist = FreeList::new();
	let entry = FreeListEntry::new(0x10000, 0x100000);

	freelist.list.push_back(entry);
	let addr = freelist.allocate(0x1000, None);
	freelist.deallocate(addr.unwrap(), 0x1000);

	let mut cursor = freelist.list.cursor_front_mut();
	while let Some(node) = cursor.current() {
		assert_eq!(node.start, 0x10000);
		assert_eq!(node.end, 0x100000);

		cursor.move_next();
	}
}

impl FreeList<PhysAddr> {
	pub fn allocate(&mut self, size: usize, alignment: Option<usize>) -> Result<PhysAddr, ()> {
		debug!(
			"Allocating {} bytes from Free List {:#X}",
			size, self as *const Self as usize
		);

		let new_size = if let Some(align) = alignment {
			size + align
		} else {
			size
		};

		// Find a region in the Free List that has at least the requested size.
		let mut cursor = self.list.cursor_front_mut();
		while let Some(node) = cursor.current() {
			let (region_start, region_size) = (node.start, node.end - node.start);

			match region_size.as_usize().cmp(&new_size) {
				Ordering::Greater => {
					// We have found a region that is larger than the requested size.
					// Return the address to the beginning of that region and shrink the region by that size.
					if let Some(align) = alignment {
						let new_addr = PhysAddr::from(align_up!(region_start.as_usize(), align));
						node.start += (new_addr - region_start) + size as u64;
						if new_addr != region_start {
							let new_entry = FreeListEntry::new(region_start, new_addr);
							cursor.insert_before(new_entry);
						}
						return Ok(new_addr);
					} else {
						node.start += size as u64;
						return Ok(region_start);
					}
				}
				Ordering::Equal => {
					// We have found a region that has exactly the requested size.
					// Return the address to the beginning of that region and move the node into the pool for deletion or reuse.
					if let Some(align) = alignment {
						let new_addr = PhysAddr::from(align_up!(region_start.as_usize(), align));
						if new_addr != region_start {
							node.end = new_addr;
						}
						return Ok(new_addr);
					} else {
						cursor.remove_current();
						return Ok(region_start);
					}
				}
				Ordering::Less => {}
			}

			cursor.move_next();
		}

		Err(())
	}
}

impl FreeList<VirtAddr> {
	pub fn allocate(&mut self, size: usize, alignment: Option<usize>) -> Result<VirtAddr, ()> {
		debug!(
			"Allocating {} bytes from Free List {:#X}",
			size, self as *const Self as usize
		);

		let new_size = if let Some(align) = alignment {
			size + align
		} else {
			size
		};

		// Find a region in the Free List that has at least the requested size.
		let mut cursor = self.list.cursor_front_mut();
		while let Some(node) = cursor.current() {
			let (region_start, region_size) = (node.start, node.end - node.start);

			match region_size.as_usize().cmp(&new_size) {
				Ordering::Greater => {
					// We have found a region that is larger than the requested size.
					// Return the address to the beginning of that region and shrink the region by that size.
					if let Some(align) = alignment {
						let new_addr = VirtAddr::from(align_up!(region_start.as_usize(), align));
						node.start += (new_addr - region_start) + size as u64;
						if new_addr != region_start {
							let new_entry = FreeListEntry::new(region_start, new_addr);
							cursor.insert_before(new_entry);
						}
						return Ok(new_addr);
					} else {
						node.start += size as u64;
						return Ok(region_start);
					}
				}
				Ordering::Equal => {
					// We have found a region that has exactly the requested size.
					// Return the address to the beginning of that region and move the node into the pool for deletion or reuse.
					if let Some(align) = alignment {
						let new_addr = VirtAddr::from(align_up!(region_start.as_usize(), align));
						if new_addr != region_start {
							node.end = new_addr;
						}
						return Ok(new_addr);
					} else {
						cursor.remove_current();
						return Ok(region_start);
					}
				}
				Ordering::Less => {}
			}

			cursor.move_next();
		}

		Err(())
	}
}
