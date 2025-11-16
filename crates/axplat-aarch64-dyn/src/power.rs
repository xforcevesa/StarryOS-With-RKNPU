use axplat::power::PowerIf;
use log::info;

struct PowerImpl;

#[impl_plat_interface]
impl PowerIf for PowerImpl {
    /// Bootstraps the given CPU core with the given initial stack (in physical
    /// address).
    ///
    /// Where `cpu_id` is the logical CPU ID (0, 1, ..., N-1, N is the number of
    /// CPU cores on the platform).
    #[cfg(feature = "smp")]
    fn cpu_boot(cpu_idx: usize, stack_top_paddr: usize) {
        use axcpu::asm::{disable_irqs, irqs_enabled};

        let irq = irqs_enabled();
        disable_irqs();
        let cpu_id = crate::smp::cpu_idx_to_id(cpu_idx);
        info!("booting CPU{cpu_idx} id {cpu_id:#x} with stack top {stack_top_paddr:#x}",);

        somehal::power::cpu_on(cpu_id as _, stack_top_paddr as _).unwrap();
        if irq {
            axcpu::asm::enable_irqs();
        }
    }

    /// Shutdown the whole system.
    fn system_off() -> ! {
        somehal::power::shutdown()
    }
}
