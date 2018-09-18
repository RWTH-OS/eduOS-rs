pub mod ehyve;
pub mod vcpu;

use libkvm::system::*;

lazy_static! {
    static ref KVM: KVMSystem = {
        KVMSystem::new().unwrap()
	};
}
