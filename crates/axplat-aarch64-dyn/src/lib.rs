#![cfg(target_arch = "aarch64")]
#![no_std]
#![feature(used_with_arg)]

#[macro_use]
extern crate axplat;
extern crate alloc;

use core::ptr::NonNull;

use axplat::mem::phys_to_virt;
use fdt_parser::Fdt;

mod boot;
mod console;
mod driver;
mod fdt;
mod init;
#[cfg(feature = "irq")]
mod irq;
mod mem;
mod power;
#[cfg(feature = "smp")]
mod smp;
mod time;

pub mod config {
    axconfig_macros::include_configs!(path_env = "AX_CONFIG_PATH", fallback = "axconfig.toml");
}

fn fdt() -> Fdt<'static> {
    let paddr = somehal::boot_info()
        .fdt
        .expect("FDT is not available, please check the bootloader configuration");
    let addr = phys_to_virt((paddr.as_ptr() as usize).into());

    Fdt::from_ptr(NonNull::new(addr.as_mut_ptr()).unwrap()).expect("Failed to parse FDT")
}
