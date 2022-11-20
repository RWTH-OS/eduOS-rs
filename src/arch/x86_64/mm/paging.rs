// Copyright (c) 2017 Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(dead_code)]

use crate::arch::x86_64::kernel::irq;
use crate::arch::x86_64::kernel::processor;
use crate::arch::x86_64::kernel::BOOT_INFO;
use crate::arch::x86_64::mm::{physicalmem, virtualmem};
use crate::consts::*;
use crate::logging::*;
use crate::scheduler;
use core::arch::asm;
use core::convert::TryInto;
use core::marker::PhantomData;
use core::mem::size_of;
use core::ptr::write_bytes;
use num_traits::CheckedShr;
use x86::controlregs;
use x86::irq::*;

/// Pointer to the root page table (PML4)
const PML4_ADDRESS: *mut PageTable<PML4> = 0xFFFF_FFFF_FFFF_F000 as *mut PageTable<PML4>;

/// Number of Offset bits of a virtual address for a 4 KiB page, which are shifted away to get its Page Frame Number (PFN).
const PAGE_BITS: usize = 12;

/// Number of bits of the index in each table (PML4, PDPT, PD, PT).
const PAGE_MAP_BITS: usize = 9;

/// A mask where PAGE_MAP_BITS are set to calculate a table index.
const PAGE_MAP_MASK: usize = 0x1FF;

bitflags! {
	/// Possible flags for an entry in either table (PML4, PDPT, PD, PT)
	///
	/// See Intel Vol. 3A, Tables 4-14 through 4-19
	pub struct PageTableEntryFlags: usize {
		/// Set if this entry is valid and points to a page or table.
		const PRESENT = 1 << 0;

		/// Set if memory referenced by this entry shall be writable.
		const WRITABLE = 1 << 1;

		/// Set if memory referenced by this entry shall be accessible from user-mode (Ring 3).
		const USER_ACCESSIBLE = 1 << 2;

		/// Set if Write-Through caching shall be enabled for memory referenced by this entry.
		/// Otherwise, Write-Back caching is used.
		const WRITE_THROUGH = 1 << 3;

		/// Set if caching shall be disabled for memory referenced by this entry.
		const CACHE_DISABLE = 1 << 4;

		/// Set if software has accessed this entry (for memory access or address translation).
		const ACCESSED = 1 << 5;

		/// Only for page entries: Set if software has written to the memory referenced by this entry.
		const DIRTY = 1 << 6;

		/// Only for page entries in PDPT or PDT: Set if this entry references a 1 GiB (PDPT) or 2 MiB (PDT) page.
		const HUGE_PAGE = 1 << 7;

		/// Only for page entries: Set if this address translation is global for all tasks and does not need to
		/// be flushed from the TLB when CR3 is reset.
		const GLOBAL = 1 << 8;

		/// Set if code execution shall be disabled for memory referenced by this entry.
		const EXECUTE_DISABLE = 1 << 63;
	}
}

impl PageTableEntryFlags {
	/// An empty set of flags for unused/zeroed table entries.
	/// Needed as long as empty() is no const function.
	const BLANK: PageTableEntryFlags = PageTableEntryFlags { bits: 0 };

	pub fn device(&mut self) -> &mut Self {
		self.insert(PageTableEntryFlags::CACHE_DISABLE);
		self
	}

	pub fn normal(&mut self) -> &mut Self {
		self.remove(PageTableEntryFlags::CACHE_DISABLE);
		self
	}

	pub fn read_only(&mut self) -> &mut Self {
		self.remove(PageTableEntryFlags::WRITABLE);
		self
	}

	pub fn writable(&mut self) -> &mut Self {
		self.insert(PageTableEntryFlags::WRITABLE);
		self
	}

	pub fn execute_disable(&mut self) -> &mut Self {
		self.insert(PageTableEntryFlags::EXECUTE_DISABLE);
		self
	}
}

/// An entry in either table (PML4, PDPT, PD, PT)
#[derive(Clone, Copy)]
pub struct PageTableEntry {
	/// Physical memory address this entry refers, combined with flags from PageTableEntryFlags.
	physical_address_and_flags: usize,
}

impl PageTableEntry {
	/// Return the stored physical address.
	pub fn address(&self) -> usize {
		self.physical_address_and_flags
			& !(BasePageSize::SIZE - 1)
			& !(PageTableEntryFlags::EXECUTE_DISABLE).bits()
	}

	/// Returns whether this entry is valid (present).
	fn is_present(&self) -> bool {
		(self.physical_address_and_flags & PageTableEntryFlags::PRESENT.bits()) != 0
	}

	fn is_huge(&self) -> bool {
		(self.physical_address_and_flags & PageTableEntryFlags::HUGE_PAGE.bits()) != 0
	}

	fn is_user(&self) -> bool {
		(self.physical_address_and_flags & PageTableEntryFlags::USER_ACCESSIBLE.bits()) != 0
	}

	/// Mark this as a valid (present) entry and set address translation and flags.
	///
	/// # Arguments
	///
	/// * `physical_address` - The physical memory address this entry shall translate to
	/// * `flags` - Flags from PageTableEntryFlags (note that the PRESENT and ACCESSED flags are set automatically)
	fn set(&mut self, physical_address: usize, flags: PageTableEntryFlags) {
		if flags.contains(PageTableEntryFlags::HUGE_PAGE) {
			// HUGE_PAGE may indicate a 2 MiB or 1 GiB page.
			// We don't know this here, so we can only verify that at least the offset bits for a 2 MiB page are zero.
			assert!(
				(physical_address % LargePageSize::SIZE) == 0,
				"Physical address is not on a 2 MiB page boundary (physical_address = {:#X})",
				physical_address
			);
		} else {
			// Verify that the offset bits for a 4 KiB page are zero.
			assert!(
				(physical_address % BasePageSize::SIZE) == 0,
				"Physical address is not on a 4 KiB page boundary (physical_address = {:#X})",
				physical_address
			);
		}

		// Verify that the physical address does not exceed the CPU's physical address width.
		assert!(
			CheckedShr::checked_shr(
				&physical_address,
				processor::get_physical_address_bits() as u32
			) == Some(0),
			"Physical address exceeds CPU's physical address width (physical_address = {:#X})",
			physical_address
		);

		let mut flags_to_set = flags;
		flags_to_set.insert(PageTableEntryFlags::PRESENT);
		flags_to_set.insert(PageTableEntryFlags::ACCESSED);
		self.physical_address_and_flags = physical_address | flags_to_set.bits();
	}
}

/// A generic interface to support all possible page sizes.
///
/// This is defined as a subtrait of Copy to enable #[derive(Clone, Copy)] for Page.
/// Currently, deriving implementations for these traits only works if all dependent types implement it as well.
pub trait PageSize: Copy {
	/// The page size in bytes.
	const SIZE: usize;

	/// The page table level at which a page of this size is mapped (from 0 for PT through 3 for PML4).
	/// Implemented as a numeric value to enable numeric comparisons.
	const MAP_LEVEL: usize;

	/// Any extra flag that needs to be set to map a page of this size.
	/// For example: PageTableEntryFlags::HUGE_PAGE
	const MAP_EXTRA_FLAG: PageTableEntryFlags;
}

/// A 4 KiB page mapped in the PT.
#[derive(Clone, Copy)]
pub enum BasePageSize {}
impl PageSize for BasePageSize {
	const SIZE: usize = 0x1000;
	const MAP_LEVEL: usize = 0;
	const MAP_EXTRA_FLAG: PageTableEntryFlags = PageTableEntryFlags::BLANK;
}

/// A 2 MiB page mapped in the PD.
#[derive(Clone, Copy)]
pub enum LargePageSize {}
impl PageSize for LargePageSize {
	const SIZE: usize = 0x200000;
	const MAP_LEVEL: usize = 1;
	const MAP_EXTRA_FLAG: PageTableEntryFlags = PageTableEntryFlags::HUGE_PAGE;
}

/// A 1 GiB page mapped in the PDPT.
#[derive(Clone, Copy)]
pub enum HugePageSize {}
impl PageSize for HugePageSize {
	const SIZE: usize = 0x40000000;
	const MAP_LEVEL: usize = 2;
	const MAP_EXTRA_FLAG: PageTableEntryFlags = PageTableEntryFlags::HUGE_PAGE;
}

/// A memory page of the size given by S.
#[derive(Clone, Copy)]
struct Page<S: PageSize> {
	/// Virtual memory address of this page.
	/// This is rounded to a page size boundary on creation.
	virtual_address: usize,

	/// Required by Rust to support the S parameter.
	size: PhantomData<S>,
}

impl<S: PageSize> Page<S> {
	/// Return the stored virtual address.
	fn address(&self) -> usize {
		self.virtual_address
	}

	/// Flushes this page from the TLB of this CPU.
	#[inline(always)]
	fn flush_from_tlb(&self) {
		unsafe {
			asm!("invlpg [{}]", in(reg) self.virtual_address, options(preserves_flags, nostack));
		}
	}

	/// Returns whether the given virtual address is a valid one in the x86-64 memory model.
	///
	/// Current x86-64 supports only 48-bit for virtual memory addresses.
	/// This is enforced by requiring bits 63 through 48 to replicate bit 47 (cf. Intel Vol. 1, 3.3.7.1).
	/// As a consequence, the address space is divided into the two valid regions 0x8000_0000_0000
	/// and 0xFFFF_8000_0000_0000.
	///
	/// Although we could make this check depend on the actual linear address width from the CPU,
	/// any extension above 48-bit would require a new page table level, which we don't implement.
	fn is_valid_address(virtual_address: usize) -> bool {
		virtual_address < 0x8000_0000_0000 || virtual_address >= 0xFFFF_8000_0000_0000
	}

	/// Returns a Page including the given virtual address.
	/// That means, the address is rounded down to a page size boundary.
	fn including_address(virtual_address: usize) -> Self {
		assert!(
			Self::is_valid_address(virtual_address),
			"Virtual address {:#X} is invalid",
			virtual_address
		);

		if S::SIZE == 1024 * 1024 * 1024 {
			assert!(processor::supports_1gib_pages());
		}

		Self {
			virtual_address: align_down!(virtual_address, S::SIZE),
			size: PhantomData,
		}
	}

	/// Returns a PageIter to iterate from the given first Page to the given last Page (inclusive).
	fn range(first: Self, last: Self) -> PageIter<S> {
		assert!(first.virtual_address <= last.virtual_address);
		PageIter {
			current: first,
			last: last,
		}
	}

	/// Returns the index of this page in the table given by L.
	fn table_index<L: PageTableLevel>(&self) -> usize {
		assert!(L::LEVEL >= S::MAP_LEVEL);
		self.virtual_address >> PAGE_BITS >> L::LEVEL * PAGE_MAP_BITS & PAGE_MAP_MASK
	}
}

/// An iterator to walk through a range of pages of size S.
struct PageIter<S: PageSize> {
	current: Page<S>,
	last: Page<S>,
}

impl<S: PageSize> Iterator for PageIter<S> {
	type Item = Page<S>;

	fn next(&mut self) -> Option<Page<S>> {
		if self.current.virtual_address <= self.last.virtual_address {
			let p = self.current;
			self.current.virtual_address += S::SIZE;
			Some(p)
		} else {
			None
		}
	}
}

/// An interface to allow for a generic implementation of struct PageTable for all 4 page tables.
/// Must be implemented by all page tables.
trait PageTableLevel {
	/// Numeric page table level (from 0 for PT through 3 for PML4) to enable numeric comparisons.
	const LEVEL: usize;
}

/// An interface for page tables with sub page tables (all except PT).
/// Having both PageTableLevel and PageTableLevelWithSubtables leverages Rust's typing system to provide
/// a subtable method only for those that have sub page tables.
///
/// Kudos to Philipp Oppermann for the trick!
trait PageTableLevelWithSubtables: PageTableLevel {
	type SubtableLevel;
}

/// The Page Map Level 4 (PML4) table, with numeric level 3 and PDPT subtables.
enum PML4 {}
impl PageTableLevel for PML4 {
	const LEVEL: usize = 3;
}

impl PageTableLevelWithSubtables for PML4 {
	type SubtableLevel = PDPT;
}

/// A Page Directory Pointer Table (PDPT), with numeric level 2 and PDT subtables.
enum PDPT {}
impl PageTableLevel for PDPT {
	const LEVEL: usize = 2;
}

impl PageTableLevelWithSubtables for PDPT {
	type SubtableLevel = PD;
}

/// A Page Directory (PD), with numeric level 1 and PT subtables.
enum PD {}
impl PageTableLevel for PD {
	const LEVEL: usize = 1;
}

impl PageTableLevelWithSubtables for PD {
	type SubtableLevel = PT;
}

/// A Page Table (PT), with numeric level 0 and no subtables.
enum PT {}
impl PageTableLevel for PT {
	const LEVEL: usize = 0;
}

/// Representation of any page table (PML4, PDPT, PD, PT) in memory.
/// Parameter L supplies information for Rust's typing system to distinguish between the different tables.
struct PageTable<L> {
	/// Each page table has 512 entries (can be calculated using PAGE_MAP_BITS).
	entries: [PageTableEntry; 1 << PAGE_MAP_BITS],

	/// Required by Rust to support the L parameter.
	level: PhantomData<L>,
}

/// A trait defining methods every page table has to implement.
/// This additional trait is necessary to make use of Rust's specialization feature and provide a default
/// implementation of some methods.
trait PageTableMethods {
	fn get_page_table_entry<S: PageSize>(&self, page: Page<S>) -> Option<PageTableEntry>;
	fn map_page_in_this_table<S: PageSize>(
		&mut self,
		page: Page<S>,
		physical_address: usize,
		flags: PageTableEntryFlags,
	) -> bool;
	fn map_page<S: PageSize>(
		&mut self,
		page: Page<S>,
		physical_address: usize,
		flags: PageTableEntryFlags,
	) -> bool;
	fn drop_user_space(&mut self);
}

impl<L: PageTableLevel> PageTableMethods for PageTable<L> {
	/// Maps a single page in this table to the given physical address.
	/// Returns whether an existing entry was updated. You can use this return value to flush TLBs.
	///
	/// Must only be called if a page of this size is mapped at this page table level!
	fn map_page_in_this_table<S: PageSize>(
		&mut self,
		page: Page<S>,
		physical_address: usize,
		flags: PageTableEntryFlags,
	) -> bool {
		assert!(L::LEVEL == S::MAP_LEVEL);
		let index = page.table_index::<L>();
		let flush = self.entries[index].is_present();

		self.entries[index].set(
			physical_address,
			PageTableEntryFlags::DIRTY | S::MAP_EXTRA_FLAG | flags,
		);

		if flush {
			page.flush_from_tlb();
		}

		flush
	}

	/// Returns the PageTableEntry for the given page if it is present, otherwise returns None.
	///
	/// This is the default implementation called only for PT.
	/// It is overridden by a specialized implementation for all tables with sub tables (all except PT).
	default fn get_page_table_entry<S: PageSize>(&self, page: Page<S>) -> Option<PageTableEntry> {
		assert!(L::LEVEL == S::MAP_LEVEL);
		let index = page.table_index::<L>();

		if self.entries[index].is_present() {
			Some(self.entries[index])
		} else {
			None
		}
	}

	default fn drop_user_space(&mut self) {
		let last = 1 << PAGE_MAP_BITS;

		for index in 0..last {
			if self.entries[index].is_present() && self.entries[index].is_user() {
				let physical_address = self.entries[index].address();

				debug!("Free page frame at 0x{:x}", physical_address);
				physicalmem::deallocate(physical_address, BasePageSize::SIZE);
			}
		}
	}

	/// Maps a single page to the given physical address.
	/// Returns whether an existing entry was updated. You can use this return value to flush TLBs.
	///
	/// This is the default implementation that just calls the map_page_in_this_table method.
	/// It is overridden by a specialized implementation for all tables with sub tables (all except PT).
	default fn map_page<S: PageSize>(
		&mut self,
		page: Page<S>,
		physical_address: usize,
		flags: PageTableEntryFlags,
	) -> bool {
		self.map_page_in_this_table::<S>(page, physical_address, flags)
	}
}

impl<L: PageTableLevelWithSubtables> PageTableMethods for PageTable<L>
where
	L::SubtableLevel: PageTableLevel,
{
	/// Returns the PageTableEntry for the given page if it is present, otherwise returns None.
	///
	/// This is the implementation for all tables with subtables (PML4, PDPT, PDT).
	/// It overrides the default implementation above.
	fn get_page_table_entry<S: PageSize>(&self, page: Page<S>) -> Option<PageTableEntry> {
		assert!(L::LEVEL >= S::MAP_LEVEL);
		let index = page.table_index::<L>();

		if self.entries[index].is_present() {
			if L::LEVEL > S::MAP_LEVEL {
				let subtable = self.subtable::<S>(page);
				subtable.get_page_table_entry::<S>(page)
			} else {
				Some(self.entries[index])
			}
		} else {
			None
		}
	}

	fn drop_user_space(&mut self) {
		let last = 1 << PAGE_MAP_BITS;
		let table_address = self as *const PageTable<L> as usize;

		for index in 0..last {
			if self.entries[index].is_present() && self.entries[index].is_user() {
				// currently, the user space uses only 4KB pages
				if L::LEVEL > BasePageSize::MAP_LEVEL {
					// Calculate the address of the subtable.
					let subtable_address = (table_address << PAGE_MAP_BITS) | (index << PAGE_BITS);
					let subtable =
						unsafe { &mut *(subtable_address as *mut PageTable<L::SubtableLevel>) };

					subtable.drop_user_space();

					//let physical_address = self.entries[index].address();
					//debug!("Free page table at 0x{:x}", physical_address);
					//physicalmem::deallocate(physical_address, BasePageSize::SIZE);
				}
			}
		}
	}

	/// Maps a single page to the given physical address.
	/// Returns whether an existing entry was updated. You can use this return value to flush TLBs.
	///
	/// This is the implementation for all tables with subtables (PML4, PDPT, PDT).
	/// It overrides the default implementation above.
	fn map_page<S: PageSize>(
		&mut self,
		page: Page<S>,
		physical_address: usize,
		flags: PageTableEntryFlags,
	) -> bool {
		assert!(L::LEVEL >= S::MAP_LEVEL);

		if L::LEVEL > S::MAP_LEVEL {
			let index = page.table_index::<L>();

			// Does the table exist yet?
			if !self.entries[index].is_present() {
				// Allocate a single 4 KiB page for the new entry and mark it as a valid, writable subtable.
				let pt_addr = physicalmem::allocate(BasePageSize::SIZE);
				if flags.contains(PageTableEntryFlags::USER_ACCESSIBLE) {
					self.entries[index].set(
						pt_addr,
						PageTableEntryFlags::WRITABLE | PageTableEntryFlags::USER_ACCESSIBLE,
					);
				} else {
					self.entries[index].set(pt_addr, PageTableEntryFlags::WRITABLE);
				}

				// Mark all entries as unused in the newly created table.
				let subtable = self.subtable::<S>(page);
				for entry in subtable.entries.iter_mut() {
					entry.physical_address_and_flags = 0;
				}

				subtable.map_page::<S>(page, physical_address, flags)
			} else {
				let subtable = self.subtable::<S>(page);
				subtable.map_page::<S>(page, physical_address, flags)
			}
		} else {
			// Calling the default implementation from a specialized one is not supported (yet),
			// so we have to resort to an extra function.
			self.map_page_in_this_table::<S>(page, physical_address, flags)
		}
	}
}

impl<L: PageTableLevelWithSubtables> PageTable<L>
where
	L::SubtableLevel: PageTableLevel,
{
	/// Returns the next subtable for the given page in the page table hierarchy.
	///
	/// Must only be called if a page of this size is mapped in a subtable!
	fn subtable<S: PageSize>(&self, page: Page<S>) -> &mut PageTable<L::SubtableLevel> {
		assert!(L::LEVEL > S::MAP_LEVEL);

		// Calculate the address of the subtable.
		let index = page.table_index::<L>();
		let table_address = self as *const PageTable<L> as usize;
		let subtable_address = (table_address << PAGE_MAP_BITS) | (index << PAGE_BITS);
		unsafe { &mut *(subtable_address as *mut PageTable<L::SubtableLevel>) }
	}

	/// Maps a continuous range of pages.
	///
	/// # Arguments
	///
	/// * `range` - The range of pages of size S
	/// * `physical_address` - First physical address to map these pages to
	/// * `flags` - Flags from PageTableEntryFlags to set for the page table entry (e.g. WRITABLE or EXECUTE_DISABLE).
	///             The PRESENT, ACCESSED, and DIRTY flags are already set automatically.
	fn map_pages<S: PageSize>(
		&mut self,
		range: PageIter<S>,
		physical_address: usize,
		flags: PageTableEntryFlags,
	) {
		let mut current_physical_address = physical_address;

		for page in range {
			self.map_page(page, current_physical_address, flags);
			current_physical_address += S::SIZE;
		}
	}

	fn drop_user_space(&mut self) {
		assert!(L::LEVEL == PML4::LEVEL);

		// the last entry is required to get access to the page tables
		let last = (1 << PAGE_MAP_BITS) - 1;
		let table_address = self as *const PageTable<L> as usize;

		for index in 0..last {
			if self.entries[index].is_present() && self.entries[index].is_user() {
				// Calculate the address of the subtable.
				let subtable_address = (table_address << PAGE_MAP_BITS) | (index << PAGE_BITS);
				let subtable =
					unsafe { &mut *(subtable_address as *mut PageTable<L::SubtableLevel>) };

				subtable.drop_user_space();

				let physical_address = self.entries[index].address();
				debug!("Free page table at 0x{:x}", physical_address);
				physicalmem::deallocate(physical_address, BasePageSize::SIZE);
			}
		}
	}
}

pub extern "x86-interrupt" fn page_fault_handler(
	stack_frame: irq::ExceptionStackFrame,
	error_code: u64,
) {
	let mut virtual_address = unsafe { controlregs::cr2() };

	// do we have to create the user-space stack?
	if virtual_address > USER_SPACE_START {
		virtual_address = align_down!(virtual_address, BasePageSize::SIZE);

		// Ok, user space want to have memory (for the stack / heap)
		let physical_address =
			physicalmem::allocate_aligned(BasePageSize::SIZE, BasePageSize::SIZE);

		debug!(
			"Map 0x{:x} into the user space at 0x{:x}",
			physical_address, virtual_address
		);

		map::<BasePageSize>(
			virtual_address,
			physical_address,
			1,
			PageTableEntryFlags::WRITABLE
				| PageTableEntryFlags::USER_ACCESSIBLE
				| PageTableEntryFlags::EXECUTE_DISABLE,
		);

		unsafe {
			// clear new page
			write_bytes(virtual_address as *mut u8, 0x00, BasePageSize::SIZE);

			// clear cr2 to signalize that the pagefault is solved by the pagefault handler
			controlregs::cr2_write(0);
		}
	} else {
		// Anything else is an error!
		let pferror = PageFaultError::from_bits_truncate(error_code as u32);

		error!("Page Fault (#PF) Exception: {:#?}", stack_frame);
		error!(
			"virtual_address = {:#X}, page fault error = {}",
			virtual_address, pferror
		);

		// clear cr2 to signalize that the pagefault is solved by the pagefault handler
		unsafe {
			controlregs::cr2_write(0);
		}

		scheduler::abort();
	}
}

fn get_page_range<S: PageSize>(virtual_address: usize, count: usize) -> PageIter<S> {
	let first_page = Page::<S>::including_address(virtual_address);
	let last_page = Page::<S>::including_address(virtual_address + (count - 1) * S::SIZE);
	Page::range(first_page, last_page)
}

pub fn get_page_table_entry<S: PageSize>(virtual_address: usize) -> Option<PageTableEntry> {
	debug!("Looking up Page Table Entry for {:#X}", virtual_address);

	let page = Page::<S>::including_address(virtual_address);
	let root_pagetable = unsafe { &mut *PML4_ADDRESS };
	root_pagetable.get_page_table_entry(page)
}

pub fn get_physical_address<S: PageSize>(virtual_address: usize) -> usize {
	debug!("Getting physical address for {:#X}", virtual_address);

	let page = Page::<S>::including_address(virtual_address);
	let root_pagetable = unsafe { &mut *PML4_ADDRESS };
	let address = root_pagetable
		.get_page_table_entry(page)
		.expect("Entry not present")
		.address();
	let offset = virtual_address & (S::SIZE - 1);
	address | offset
}

/// Translate a virtual memory address to a physical one.
/// Just like get_physical_address, but automatically uses the correct page size for the respective memory address.
pub fn virtual_to_physical(virtual_address: usize) -> usize {
	get_physical_address::<BasePageSize>(virtual_address)
}

pub fn unmap<S: PageSize>(virtual_address: usize, count: usize) {
	debug!(
		"Unmapping virtual address {:#X} ({} pages)",
		virtual_address, count
	);

	let range = get_page_range::<S>(virtual_address, count);
	let root_pagetable = unsafe { &mut *PML4_ADDRESS };
	root_pagetable.map_pages(range, 0, PageTableEntryFlags::BLANK);
}

pub fn map<S: PageSize>(
	virtual_address: usize,
	physical_address: usize,
	count: usize,
	flags: PageTableEntryFlags,
) {
	debug!(
		"Mapping virtual address {:#X} to physical address {:#X} ({} pages)",
		virtual_address, physical_address, count
	);

	let range = get_page_range::<S>(virtual_address, count);
	let root_pagetable = unsafe { &mut *PML4_ADDRESS };
	root_pagetable.map_pages(range, physical_address, flags);
}

static mut ROOT_PAGE_TABLE: usize = 0;

#[inline(always)]
pub fn get_kernel_root_page_table() -> usize {
	unsafe { ROOT_PAGE_TABLE }
}

pub fn drop_user_space() {
	let root_pagetable = unsafe { &mut *PML4_ADDRESS };

	root_pagetable.drop_user_space();
}

// just an workaround to explaine the difference between
// kernel and user space
pub fn create_usr_pgd() -> usize {
	debug!("Create 1st level page table for the user-level task");

	unsafe {
		let physical_address =
			physicalmem::allocate_aligned(BasePageSize::SIZE, BasePageSize::SIZE);
		let user_page_table: usize =
			virtualmem::allocate_aligned(BasePageSize::SIZE, BasePageSize::SIZE);

		debug!(
			"Map page frame 0x{:x} at virtual address 0x{:x}",
			physical_address, user_page_table
		);

		map::<BasePageSize>(
			user_page_table,
			physical_address,
			1,
			PageTableEntryFlags::WRITABLE | PageTableEntryFlags::EXECUTE_DISABLE,
		);

		write_bytes(user_page_table as *mut u8, 0x00, BasePageSize::SIZE);

		let recursive_pgt = BOOT_INFO.unwrap().recursive_page_table_addr as *const u64;
		let recursive_pgt_idx = BOOT_INFO.unwrap().recursive_index();
		let pml4 = user_page_table as *mut u64;
		for i in 0..recursive_pgt_idx + 2 {
			*pml4.offset(i.try_into().unwrap()) = *recursive_pgt.offset(i.try_into().unwrap());
		}

		let pml4 =
			(user_page_table + BasePageSize::SIZE - size_of::<usize>()) as *mut PageTableEntry;
		(*pml4).set(physical_address, PageTableEntryFlags::WRITABLE);

		// unmap page table
		unmap::<BasePageSize>(user_page_table, 1);
		virtualmem::deallocate(user_page_table, BasePageSize::SIZE);

		scheduler::set_root_page_table(physical_address);

		physical_address
	}
}

pub fn init() {
	let recursive_pgt = unsafe { BOOT_INFO.unwrap().recursive_page_table_addr } as *mut u64;
	let recursive_pgt_idx = unsafe { BOOT_INFO.unwrap().recursive_index() };

	debug!(
		"Found recursive_page_table_addr at 0x{:x}",
		recursive_pgt as u64
	);
	debug!("Recursive index: {}", recursive_pgt_idx);

	unsafe {
		ROOT_PAGE_TABLE = *recursive_pgt.offset(recursive_pgt_idx.try_into().unwrap()) as usize
			& !(BasePageSize::SIZE - 1);
		*recursive_pgt.offset(511) = *recursive_pgt.offset(recursive_pgt_idx.try_into().unwrap());

		for i in recursive_pgt_idx + 2..511 {
			*recursive_pgt.offset(i.try_into().unwrap()) = 0;
		}

		//flush TLB
		controlregs::cr3_write(controlregs::cr3());
	}
}
