//! Basic PS/2 keyboard driver.  This needs to be split into a
//! PS/2-specific driver, and a high-level portable driver.
//!
//! Scancode table available at http://wiki.osdev.org/Keyboard#Scan_Code_Set_1

use spin::Mutex;
use cpuio;

/// A pair of keys which appear on both the left and right sides of the
/// keyboard, such as "left shift" and "right shift".
#[derive(Debug)]
struct KeyPair {
    left: bool,
    right: bool,
}

impl KeyPair {
    /// Create a new key pair.  Normally, we'd use `#[derive(Default)]` and
    /// `Default::default()` for this, but if we use those, we can't make
    /// them `const`, which means we can't use them to initialize static
    /// variables at compile time.  So let's reinvent this wheel.
    const fn new() -> Self {
        KeyPair { left: false, right: false }
    }

    /// Is either of the keys in this pair currently pressed?
    fn is_pressed(&self) -> bool {
        self.left || self.right
    }
}

/// All of our supported keyboard modifiers.
#[derive(Debug)]
struct Modifiers {
    shift: KeyPair,
    control: KeyPair,
    alt: KeyPair,
    caps_lock: bool,
}

impl Modifiers {
    /// Create a new set of modifiers.  See notes about `Default` above.
    const fn new() -> Self {
        Modifiers {
            shift: KeyPair::new(),
            control: KeyPair::new(),
            alt: KeyPair::new(),
            caps_lock: false,
        }
    }

    /// Given the current modifier state, should we convert letters to
    /// uppercase?
    fn use_uppercase_letters(&self) -> bool {
        self.shift.is_pressed() ^ self.caps_lock
    }

    /// Apply all of our modifiers to an ASCII character, and return a new
    /// ASCII character.
    fn apply_to(&self, ascii: u8) -> u8 {
        if b'a' <= ascii && ascii <= b'z' {
            if self.use_uppercase_letters() {
                return ascii - b'a' + b'A';
            }
        }
        ascii
    }

    /// Given a keyboard scancode, update our current modifier state.
    fn update(&mut self, scancode: u8) {
        match scancode {
            0x1D => self.control.left = true,
            0x2A => self.shift.left = true,
            0x36 => self.shift.right = true,
            0x38 => self.alt.left = true,
            /// Caps lock toggles on leading edge, instead of paying
            /// attention to key up/down events.
            0x3A => self.caps_lock = !self.caps_lock,
            0x9D => self.control.left = false,
            0xAA => self.shift.left = false,
            0xB6 => self.shift.right = false,
            0xB8 => self.alt.left = false,

            _ => {},
        }
    }
}

/// Our keyboard state, including our I/O port, our currently pressed
/// modifiers, etc.
struct State {
    /// The PS/2 serial IO port for the keyboard.  There's a huge amount of
    /// emulation going on at the hardware level to allow us to pretend to
    /// be an early-80s IBM PC.
    ///
    /// We could read the standard keyboard port directly using
    /// `inb(0x60)`, but it's nicer if we wrap it up in an `cpuio::Port`
    /// object.
    port: cpuio::Port<u8>,

    /// We also need to keep track of which modifier keys have been pressed
    /// and released.
    modifiers: Modifiers,
}

/// Our global keyboard state, protected by a mutex.
static STATE: Mutex<State> = Mutex::new(State {
    port: unsafe { cpuio::Port::new(0x60) },
    modifiers: Modifiers::new(),
});

/// Try to convert a scancode to an ASCII character.  If we don't recognize
/// it, just return `None`.
fn find_ascii(scancode: u8) -> Option<u8> {
    let idx = scancode as usize;
    match scancode {
        0x01 ... 0x0E => Some(b"\x1B1234567890-=\0x02"[idx-0x01]),
        0x0F ... 0x1C => Some(b"\tqwertyuiop[]\r"[idx-0x0F]),
        0x1E ... 0x28 => Some(b"asdfghjkl;'"[idx-0x1E]),
        0x2C ... 0x35 => Some(b"zxcvbnm,./"[idx-0x2C]),
        0x39 => Some(b' '),
        _ => None,
    }
}

/// Try to read a single input character
pub fn read_char() -> Option<char> {
    let mut state = STATE.lock();

    // Read a single scancode off our keyboard port.
    let scancode = state.port.read();

    // Give our modifiers first crack at this.
    state.modifiers.update(scancode);

    // Look up the ASCII keycode.
    if let Some(ascii) = find_ascii(scancode) {
        // The `as char` converts our ASCII data to Unicode, which is
        // correct as long as we're only using 7-bit ASCII.
        Some(state.modifiers.apply_to(ascii) as char)
    } else {
        // Either this was a modifier key, or it some key we don't know how
        // to handle yet, or it's part of a multibyte scancode.  Just look
        // innocent and pretend nothing happened.
        None
    }
}
