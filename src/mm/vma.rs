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

use alloc::{fmt,BinaryHeap};
use core::cmp::Ordering;
use synch::spinlock::SpinlockIrqSave;
use logging::*;

bitflags! {
	pub struct VmaType: u8 {
		/// This VMA is not accessable
		const NO_ACCESS	= 0b00000000;
		/// Read access to this VMA is allowed
        const READ		= 0b00000001;
		/// Write access to this VMA is allowed
        const WRITE		= 0b00000010;
		/// Instructions fetches in this VMA are allowed
        const EXECUTE	= 0b00000100;
		/// This VMA is cacheable
        const CACHEABLE	= 0b00001000;
		/// This VMA should be part of the userspace
		const USER		= 0b00010000;
		/// A collection of flags used for the kernel heap
		const VMA_HEAP	= Self::READ.bits | Self::WRITE.bits | Self::CACHEABLE.bits;
    }
}

impl fmt::Display for VmaType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.contains(VmaType::CACHEABLE) == true {
        	write!(f, "c")?;
		} else {
			write!(f, "-")?;
		}
		
		if self.contains(VmaType::READ) == true {
        	write!(f, "r")?;
		} else {
			write!(f, "-")?;
		}

		if self.contains(VmaType::WRITE) == true {
        	write!(f, "w")?;
		} else {
			write!(f, "-")?;
		}

		if self.contains(VmaType::EXECUTE) == true {
        	write!(f, "x")?;
		} else {
			write!(f, "-")?;
		}

		Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VirtualMemoryArea {
	start: usize,
	length: usize,
	vma_type: VmaType
}

impl VirtualMemoryArea {
	pub const fn new(s: usize, l: usize, vt: VmaType) -> VirtualMemoryArea {
		VirtualMemoryArea {
			start: s,
			length: l,
			vma_type: vt
		}
	}
}

impl Eq for VirtualMemoryArea {}

impl PartialOrd for VirtualMemoryArea {
    fn partial_cmp(&self, other: &VirtualMemoryArea) -> Option<Ordering> {
        Some(self.start.cmp(&other.start).reverse())
    }
}

impl Ord for VirtualMemoryArea {
    fn cmp(&self, other: &VirtualMemoryArea) -> Ordering {
        self.start.cmp(&other.start).reverse()
    }
}

impl PartialEq for VirtualMemoryArea {
    fn eq(&self, other: &VirtualMemoryArea) -> bool {
        self.start == other.start
    }
}

impl fmt::Display for VirtualMemoryArea {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:08x} -- 0x{:08x} ({})", self.start, self.start+self.length, self.vma_type)
    }
}

pub struct VmaManager {
	vmheap: BinaryHeap<VirtualMemoryArea>
}

impl VmaManager {
	pub fn new() -> VmaManager
	{
		VmaManager {
			vmheap: BinaryHeap::new()
		}
	}
}

lazy_static! {
	pub static ref VMA_MANAGER: SpinlockIrqSave<VmaManager> = {
		SpinlockIrqSave::new(VmaManager::new())
	};
}

pub fn vma_add(size: usize, len: usize, vt: VmaType)
{
	let vm = VirtualMemoryArea::new(size, len, vt);

	VMA_MANAGER.lock().vmheap.push(vm);
}

pub fn vma_dump()
{
	info!("Snapshot of the current virtual memory areas:");
	for x in VMA_MANAGER.lock().vmheap.iter() {
		info!("VMA: {}", x);
	}
}
