use core::arch::naked_asm;

use somehal::BootInfo;

const BOOT_STACK_SIZE: usize = 0x40000; // 256KB

#[unsafe(link_section = ".bss.stack")]
static mut BOOT_STACK: [u8; BOOT_STACK_SIZE] = [0; BOOT_STACK_SIZE];

#[somehal::entry]
fn main(args: &BootInfo) -> ! {
    unsafe {
        switch_sp(args);
    }
}
#[unsafe(naked)]
unsafe extern "C" fn switch_sp(_args: &BootInfo) -> ! {
    naked_asm!(
        "
        adrp x8, {sp}
        add  x8, x8, :lo12:{sp}
        add  x8, x8, {size}
        mov  sp, x8
        bl   {next}
        ",
        sp = sym BOOT_STACK,
        size = const BOOT_STACK_SIZE,
        next = sym sp_reset,
    )
}

fn sp_reset(args: &BootInfo) -> ! {
    axplat::call_main(0, args.fdt.map(|p| p.as_ptr() as usize).unwrap_or_default());
}

#[cfg(feature = "smp")]
#[somehal::secondary_entry]
fn secondary(cpu_id: usize) {
    use aarch64_cpu_ext::cache::{CacheOp, dcache_all};
    dcache_all(CacheOp::Invalidate);
    let cpu_idx = crate::smp::cpu_id_to_idx(cpu_id);
    axplat::call_secondary_main(cpu_idx)
}
