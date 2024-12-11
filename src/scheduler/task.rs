#![allow(dead_code)]

use crate::arch;
use crate::arch::mm::PhysAddr;
use crate::arch::mm::VirtAddr;
use crate::arch::processor::msb;
use crate::arch::{BasePageSize, PageSize};
use crate::consts::*;
use crate::fd::stdio::{GenericStderr, GenericStdin, GenericStdout};
use crate::fd::{FileDescriptor, IoInterface, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use crate::logging::*;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::rc::Rc;
use alloc::sync::Arc;
use core::cell::RefCell;
use core::fmt;

/// The status of the task - used for scheduling
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum TaskStatus {
	Invalid,
	Ready,
	Running,
	Blocked,
	Finished,
	Idle,
}

/// Unique identifier for a task (i.e. `pid`).
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct TaskId(u32);

impl TaskId {
	pub const fn into(self) -> u32 {
		self.0
	}

	pub const fn from(x: u32) -> Self {
		TaskId(x)
	}
}

impl alloc::fmt::Display for TaskId {
	fn fmt(&self, f: &mut fmt::Formatter) -> alloc::fmt::Result {
		write!(f, "{}", self.0)
	}
}

/// Priority of a task
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct TaskPriority(u8);

impl TaskPriority {
	pub const fn into(self) -> u8 {
		self.0
	}

	pub const fn from(x: u8) -> Self {
		TaskPriority(x)
	}
}

impl alloc::fmt::Display for TaskPriority {
	fn fmt(&self, f: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
		write!(f, "{}", self.0)
	}
}

pub const REALTIME_PRIORITY: TaskPriority = TaskPriority::from(NO_PRIORITIES as u8 - 1);
pub const HIGH_PRIORITY: TaskPriority = TaskPriority::from(24);
pub const NORMAL_PRIORITY: TaskPriority = TaskPriority::from(16);
pub const LOW_PRIORITY: TaskPriority = TaskPriority::from(0);

/// Realize a priority queue for tasks
pub(crate) struct PriorityTaskQueue {
	queues: [VecDeque<Rc<RefCell<Task>>>; NO_PRIORITIES],
	prio_bitmap: usize,
}

impl PriorityTaskQueue {
	/// Creates an empty priority queue for tasks
	pub const fn new() -> PriorityTaskQueue {
		const VALUE: VecDeque<Rc<RefCell<Task>>> = VecDeque::new();

		PriorityTaskQueue {
			queues: [VALUE; NO_PRIORITIES],
			prio_bitmap: 0,
		}
	}

	/// Add a task by its priority to the queue
	pub fn push(&mut self, task: Rc<RefCell<Task>>) {
		let i: usize = task.borrow().prio.into().into();
		//assert!(i < NO_PRIORITIES, "Priority {} is too high", i);

		self.prio_bitmap |= 1 << i;
		self.queues[i].push_back(task.clone());
	}

	fn pop_from_queue(&mut self, queue_index: usize) -> Option<Rc<RefCell<Task>>> {
		let task = self.queues[queue_index].pop_front();
		if self.queues[queue_index].is_empty() {
			self.prio_bitmap &= !(1 << queue_index);
		}

		task
	}

	/// Pop the task with the highest priority from the queue
	pub fn pop(&mut self) -> Option<Rc<RefCell<Task>>> {
		if let Some(i) = msb(self.prio_bitmap) {
			return self.pop_from_queue(i);
		}

		None
	}

	/// Pop the next task, which has a higher or the same priority as `prio`
	pub fn pop_with_prio(&mut self, prio: TaskPriority) -> Option<Rc<RefCell<Task>>> {
		if let Some(i) = msb(self.prio_bitmap) {
			if i >= prio.into().into() {
				return self.pop_from_queue(i);
			}
		}

		None
	}
}

#[allow(dead_code)]
pub(crate) trait Stack {
	fn top(&self) -> VirtAddr;
	fn bottom(&self) -> VirtAddr;
	fn interrupt_top(&self) -> VirtAddr;
	fn interrupt_bottom(&self) -> VirtAddr;
}

#[derive(Copy, Clone)]
#[repr(C, align(64))]
pub(crate) struct TaskStack {
	buffer: [u8; STACK_SIZE],
	ist_buffer: [u8; INTERRUPT_STACK_SIZE],
}

impl Default for TaskStack {
	fn default() -> Self {
		Self::new()
	}
}

impl TaskStack {
	pub const fn new() -> TaskStack {
		TaskStack {
			buffer: [0; STACK_SIZE],
			ist_buffer: [0; INTERRUPT_STACK_SIZE],
		}
	}
}

impl Stack for TaskStack {
	fn top(&self) -> VirtAddr {
		VirtAddr::from(self.buffer.as_ptr() as usize + STACK_SIZE - 16)
	}

	fn bottom(&self) -> VirtAddr {
		VirtAddr::from(self.buffer.as_ptr() as usize)
	}

	fn interrupt_top(&self) -> VirtAddr {
		VirtAddr::from(self.ist_buffer.as_ptr() as usize + INTERRUPT_STACK_SIZE - 16)
	}

	fn interrupt_bottom(&self) -> VirtAddr {
		VirtAddr::from(self.ist_buffer.as_ptr() as usize)
	}
}

/// A task control block, which identifies either a process or a thread
#[repr(align(64))]
pub(crate) struct Task {
	/// The ID of this context
	pub id: TaskId,
	/// Task Priority
	pub prio: TaskPriority,
	/// Status of a task, e.g. if the task is ready or blocked
	pub status: TaskStatus,
	/// Last stack pointer before a context switch to another task
	pub last_stack_pointer: VirtAddr,
	/// Stack of the task
	pub stack: Box<dyn Stack>,
	/// Physical address of the 1st level page table
	pub root_page_table: PhysAddr,
	/// Mapping between file descriptor and the referenced IO interface
	pub fd_map: BTreeMap<FileDescriptor, Arc<dyn IoInterface>>,
}

impl Task {
	pub fn new_idle(id: TaskId) -> Task {
		Task {
			id,
			prio: LOW_PRIORITY,
			status: TaskStatus::Idle,
			last_stack_pointer: VirtAddr::zero(),
			stack: Box::new(crate::arch::mm::get_boot_stack()),
			root_page_table: arch::get_kernel_root_page_table(),
			fd_map: BTreeMap::new(),
		}
	}

	pub fn new(id: TaskId, status: TaskStatus, prio: TaskPriority) -> Task {
		let mut fd_map: BTreeMap<FileDescriptor, Arc<dyn IoInterface>> = BTreeMap::new();
		fd_map
			.try_insert(STDIN_FILENO, Arc::new(GenericStdin::new()))
			.unwrap();
		fd_map
			.try_insert(STDOUT_FILENO, Arc::new(GenericStdout::new()))
			.unwrap();
		fd_map
			.try_insert(STDERR_FILENO, Arc::new(GenericStderr::new()))
			.unwrap();

		Task {
			id,
			prio,
			status,
			last_stack_pointer: VirtAddr::zero(),
			stack: Box::new(TaskStack::new()),
			root_page_table: arch::get_kernel_root_page_table(),
			fd_map,
		}
	}
}

pub(crate) trait TaskFrame {
	/// Create the initial stack frame for a new task
	fn create_stack_frame(&mut self, func: extern "C" fn());
}

impl Drop for Task {
	fn drop(&mut self) {
		if self.root_page_table != arch::get_kernel_root_page_table() {
			debug!(
				"Deallocate page table 0x{:x} of task {}",
				self.root_page_table, self.id
			);
			arch::mm::physicalmem::deallocate(self.root_page_table, BasePageSize::SIZE);
		}
	}
}
