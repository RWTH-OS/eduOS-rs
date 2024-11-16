use crate::arch::mm::VirtAddr;
use crate::consts::*;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::fmt;

/// The status of the task - used for scheduling
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum TaskStatus {
	Invalid,
	Ready,
	Running,
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

#[allow(dead_code)]
pub(crate) trait Stack {
	fn top(&self) -> VirtAddr;
	fn bottom(&self) -> VirtAddr;
}

#[derive(Copy, Clone)]
#[repr(C, align(64))]
pub(crate) struct TaskStack {
	buffer: [u8; STACK_SIZE],
}

impl TaskStack {
	pub const fn new() -> Self {
		Self {
			buffer: [0; STACK_SIZE],
		}
	}
}

impl Stack for TaskStack {
	fn top(&self) -> VirtAddr {
		VirtAddr::from((&(self.buffer[STACK_SIZE - 16]) as *const _) as usize)
	}

	fn bottom(&self) -> VirtAddr {
		VirtAddr::from((&(self.buffer[0]) as *const _) as usize)
	}
}

impl Default for TaskStack {
	fn default() -> Self {
		Self::new()
	}
}

pub(crate) struct TaskQueue {
	queue: VecDeque<Rc<RefCell<Task>>>,
}

impl TaskQueue {
	pub fn new() -> Self {
		Self {
			queue: Default::default(),
		}
	}

	/// Add a task to the queue
	pub fn push(&mut self, task: Rc<RefCell<Task>>) {
		self.queue.push_back(task);
	}

	/// Pop the task from the queue
	pub fn pop(&mut self) -> Option<Rc<RefCell<Task>>> {
		self.queue.pop_front()
	}
}

impl Default for TaskQueue {
	fn default() -> Self {
		Self::new()
	}
}

/// A task control block, which identifies either a process or a thread
#[repr(align(64))]
pub(crate) struct Task {
	/// The ID of this context
	pub id: TaskId,
	/// Status of a task, e.g. if the task is ready or blocked
	pub status: TaskStatus,
	/// Last stack pointer before a context switch to another task
	pub last_stack_pointer: usize,
	/// Stack of the task
	pub stack: Box<dyn Stack>,
}

impl Task {
	pub fn new_idle(id: TaskId) -> Task {
		Task {
			id,
			status: TaskStatus::Idle,
			last_stack_pointer: 0,
			stack: Box::new(crate::arch::mm::get_boot_stack()),
		}
	}

	pub fn new(id: TaskId, status: TaskStatus) -> Task {
		Task {
			id,
			status,
			last_stack_pointer: 0,
			stack: Box::new(TaskStack::new()),
		}
	}
}

pub(crate) trait TaskFrame {
	/// Create the initial stack frame for a new task
	fn create_stack_frame(&mut self, func: extern "C" fn());
}
