use core::ptr::NonNull;

use rdrive::{Platform, init, probe_pre_kernel};
use somehal::{boot_info, mem::phys_to_virt};

pub fn setup() {
    let paddr = boot_info().fdt.expect("FDT must be present");
    let fdt = phys_to_virt(paddr.as_ptr() as usize);
    init(Platform::Fdt {
        addr: unsafe { NonNull::new_unchecked(fdt) },
    })
    .unwrap();

    probe_pre_kernel().unwrap();
}
