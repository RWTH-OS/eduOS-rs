use crate::synch::spinlock::Spinlock;
use core::fmt;
use x86::io::*;

const CRT_CONTROLLER_ADDRESS_PORT: u16 = 0x3D4;
const CRT_CONTROLLER_DATA_PORT: u16 = 0x3D5;
const CURSOR_START_REGISTER: u8 = 0x0A;
const CURSOR_DISABLE: u8 = 0x20;

const ATTRIBUTE_BLACK: u8 = 0x00;
const ATTRIBUTE_LIGHTGREY: u8 = 0x07;
const COLS: usize = 80;
const ROWS: usize = 25;
const VGA_BUFFER_ADDRESS: u64 = 0xB8000;

pub(crate) static VGA_SCREEN: Spinlock<VgaScreen> = Spinlock::new(VgaScreen::new());

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub(crate) struct VgaCharacter {
	character: u8,
	attribute: u8,
}

impl VgaCharacter {
	const fn new(character: u8, attribute: u8) -> Self {
		Self {
			character,
			attribute,
		}
	}
}

pub(crate) struct VgaScreen {
	buffer: *mut [[VgaCharacter; COLS]; ROWS],
	current_col: usize,
	current_row: usize,
	is_initialized: bool,
}

impl VgaScreen {
	const fn new() -> Self {
		Self {
			buffer: VGA_BUFFER_ADDRESS as *mut _,
			current_col: 0,
			current_row: 0,
			is_initialized: false,
		}
	}

	fn init(&mut self) {
		// Disable the cursor.
		unsafe {
			outb(CRT_CONTROLLER_ADDRESS_PORT, CURSOR_START_REGISTER);
			outb(CRT_CONTROLLER_DATA_PORT, CURSOR_DISABLE);
		}

		// Clear the screen.
		for r in 0..ROWS {
			self.clear_row(r);
		}

		// Initialization done!
		self.is_initialized = true;
	}

	#[inline]
	fn clear_row(&mut self, row: usize) {
		// Overwrite this row by a bogus character in black.
		for c in 0..COLS {
			unsafe {
				(*self.buffer)[row][c] = VgaCharacter::new(0, ATTRIBUTE_BLACK);
			}
		}
	}

	fn write_byte(&mut self, byte: u8) {
		if !self.is_initialized {
			return;
		}

		// Move to the next row if we have a newline character or hit the end of a column.
		if byte == b'\n' || self.current_col == COLS {
			self.current_col = 0;
			self.current_row += 1;
		}

		// Check if we have hit the end of the screen rows.
		if self.current_row == ROWS {
			// Shift all rows up by one line, removing the oldest visible screen row.
			for r in 1..ROWS {
				for c in 0..COLS {
					unsafe {
						(*self.buffer)[r - 1][c] = (*self.buffer)[r][c];
					}
				}
			}

			// Clear the last screen row and write to it next time.
			self.clear_row(ROWS - 1);
			self.current_row = ROWS - 1;
		}

		if byte != b'\n' {
			// Put our character into the VGA screen buffer and advance the column counter.
			unsafe {
				(*self.buffer)[self.current_row][self.current_col] =
					VgaCharacter::new(byte, ATTRIBUTE_LIGHTGREY);
			}
			self.current_col += 1;
		}
	}
}

unsafe impl Send for VgaScreen {}

impl fmt::Write for VgaScreen {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		for &b in s.as_bytes() {
			self.write_byte(b);
		}

		Ok(())
	}
}

pub(crate) fn init() {
	VGA_SCREEN.lock().init();
}
