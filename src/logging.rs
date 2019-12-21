#![allow(unused_macros)]

/// An enum representing the available verbosity levels of the logger.
#[derive(Copy, Clone)]
pub enum LogLevel {
	/// Disable all our put messages
	///
	/// Designates without information
	DISABLED = 0,
	/// The "error" level.
	///
	/// Designates very serious errors.
	ERROR,
	/// The "warn" level.
	///
	/// Designates hazardous situations.
	WARNING,
	/// The "info" level.
	///
	/// Designates useful information.
	INFO,
	// The "debug" level.
	///
	/// Designates lower priority information.
	DEBUG,
}

/// Data structures to filter kernel messages
pub struct KernelLogger {
	pub log_level: LogLevel,
}

/// default logger to handle kernel messages
pub const LOGGER: KernelLogger = KernelLogger {
	log_level: LogLevel::INFO,
};

/// Print formatted info text to our console, followed by a newline.
#[macro_export]
macro_rules! info {
	($fmt:expr) => ({
		let current_level = LOGGER.log_level as u8;
		let cmp_level = LogLevel::INFO as u8;

		if current_level >= cmp_level {
			println!(concat!("[INFO] ", $fmt));
		}
	});
	($fmt:expr, $($arg:tt)*) => ({
		let current_level = LOGGER.log_level as u8;
		let cmp_level = LogLevel::INFO as u8;

		if current_level >= cmp_level {
			println!(concat!("[INFO] ", $fmt), $($arg)*);
		}
	});
}

/// Print formatted warnings to our console, followed by a newline.
#[macro_export]
macro_rules! warn {
	($fmt:expr) => ({
		let current_level = LOGGER.log_level as u8;
		let cmp_level = LogLevel::WARNING as u8;

		if current_level >= cmp_level {
			println!(concat!("[WARNING] ", $fmt));
		}
	});
	($fmt:expr, $($arg:tt)*) => ({
		let current_level = LOGGER.log_level as u8;
		let cmp_level = LogLevel::WARNING  as u8;

		if current_level >= cmp_level {
			println!(concat!("[WARNING] ", $fmt), $($arg)*);
		}
	});
}

/// Print formatted warnings to our console, followed by a newline.
#[macro_export]
macro_rules! error {
	($fmt:expr) => ({
		let current_level = LOGGER.log_level as u8;
		let cmp_level = LogLevel::ERROR as u8;

		if current_level >= cmp_level {
			println!(concat!("[ERROR] ", $fmt));
		}
	});
	($fmt:expr, $($arg:tt)*) => ({
		let current_level = LOGGER.log_level as u8;
		let cmp_level = LogLevel::ERROR  as u8;

		if current_level >= cmp_level {
			println!(concat!("[ERROR] ", $fmt), $($arg)*);
		}
	});
}

/// Print formatted debug messages to our console, followed by a newline.
#[macro_export]
macro_rules! debug {
	($fmt:expr) => ({
		let current_level = LOGGER.log_level as u8;
		let cmp_level = LogLevel::DEBUG as u8;

		if current_level >= cmp_level {
			println!(concat!("[DEBUG] ", $fmt));
		}
	});
	($fmt:expr, $($arg:tt)*) => ({
		let current_level = LOGGER.log_level as u8;
		let cmp_level = LogLevel::DEBUG  as u8;

		if current_level >= cmp_level {
			println!(concat!("[DEBUG] ", $fmt), $($arg)*);
		}
	});
}
