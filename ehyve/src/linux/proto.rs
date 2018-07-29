use linux::error::*;
use linux::kvm::*;

pub const PORT_UART:		u16 = 0x3f8;
pub const PORT_QEMU_SHUTDOWN:	u16 = 0xf4;

#[derive(Debug)]
pub enum HardwareInterface {
    UART(u8),
	QEMU_SHUTDOW(u8),
    Other(*const kvm_run)
}

#[derive(Debug)]
pub enum Return {
    Continue,
    Interrupt,
    Exit(i32)
}

impl HardwareInterface {
    pub fn from_mem(mem: *const u8) -> Result<HardwareInterface> {
        unsafe {
            let ref run = *(mem as *const kvm_run);

            // debug!("Exit reason {}", run.exit_reason);

            // TODO: KVM_EXIT_MMIO
            if run.exit_reason != KVM_EXIT_IO {
                return Ok(HardwareInterface::Other(mem as *const kvm_run));
            }

            let offset = *(mem.offset(run.__bindgen_anon_1.io.data_offset as isize) as *const isize);

            //let ptr = guest_mem.offset(offset);
            Ok(match run.__bindgen_anon_1.io.port {
                PORT_UART       => { HardwareInterface::UART(offset as u8) },
				PORT_QEMU_SHUTDOWN	=> { HardwareInterface::QEMU_SHUTDOW(offset as u8) },
                _ => {
                    let err = format!("KVM: unhandled KVM_EXIT_IO at port {:#x}, direction {}",
                        run.__bindgen_anon_1.io.port, run.__bindgen_anon_1.io.direction);
                    return Err(Error::Protocol(err));
                }
            })
        }
    }

    pub unsafe fn run(&self) -> Result<Return> {
        match *self {
            HardwareInterface::UART(obj) => {
                use std::io::{self, Write};
                let buf = [obj];
                io::stderr().write(&buf).ok();
            },
			HardwareInterface::QEMU_SHUTDOW(obj) => {
				return Ok(Return::Exit(obj as i32));
			},
            HardwareInterface::Other(id) => {
                let err = match (*id).exit_reason {
                    KVM_EXIT_HLT => format!("Guest has halted the CPU, this is considered as a normal exit."),
                    KVM_EXIT_MMIO => format!("KVM: unhandled KVM_EXIT_MMIO at {:#x}", (*id).__bindgen_anon_1.mmio.phys_addr ),
                    KVM_EXIT_FAIL_ENTRY => format!("KVM: entry failure: hw_entry_failure_reason={:#x}", (*id).__bindgen_anon_1.fail_entry.hardware_entry_failure_reason),
                    KVM_EXIT_INTERNAL_ERROR => format!("KVM: internal error exit: suberror = {:#x}", (*id).__bindgen_anon_1.internal.suberror),
                    KVM_EXIT_SHUTDOWN => format!("KVM: receive shutdown command"),
                    KVM_EXIT_DEBUG => return Err(Error::KVMDebug),
                    _ => format!("KVM: unhandled exit: exit_reason = {:#x}", (*id).exit_reason)
                };

                return Err(Error::Protocol(err));
            }
        }

        return Ok(Return::Continue);
    }
}
