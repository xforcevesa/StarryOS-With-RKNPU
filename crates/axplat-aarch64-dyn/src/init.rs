use axplat::init::InitIf;
use log::debug;

use crate::{console, driver};

struct InitIfImpl;

#[impl_plat_interface]
impl InitIf for InitIfImpl {
    /// Initializes the platform at the early stage for the primary core.
    ///
    /// This function should be called immediately after the kernel has booted,
    /// and performed earliest platform configuration and initialization (e.g.,
    /// early console, clocking).
    ///
    /// # Arguments
    ///
    /// * `cpu_id` is the logical CPU ID (0, 1, ..., N-1, N is the number of CPU
    /// cores on the platform).
    /// * `arg` is passed from the bootloader (typically the device tree blob
    /// address).
    ///
    /// # Before calling this function
    ///
    /// * CPU is booted in the kernel mode.
    /// * Early page table is set up, virtual memory is enabled.
    /// * CPU-local data is initialized.
    ///
    /// # After calling this function
    ///
    /// * Exception & interrupt handlers are set up.
    /// * Early console is initialized.
    /// * Current monotonic time and wall time can be obtained.
    fn init_early(_cpu_id: usize, _arg: usize) {
        console::setup_early();
        axcpu::init::init_trap();
        crate::mem::setup();
    }

    /// Initializes the platform at the early stage for secondary cores.
    ///
    /// See [`init_early`] for details.
    #[cfg(feature = "smp")]
    fn init_early_secondary(_cpu_id: usize) {
        axcpu::init::init_trap();
    }

    /// Initializes the platform at the later stage for the primary core.
    ///
    /// This function should be called after the kernel has done part of its
    /// initialization (e.g, logging, memory management), and finalized the rest
    /// of platform configuration and initialization.
    ///
    /// # Arguments
    ///
    /// * `cpu_id` is the logical CPU ID (0, 1, ..., N-1, N is the number of CPU
    /// cores on the platform).
    /// * `arg` is passed from the bootloader (typically the device tree blob
    /// address).
    ///
    /// # Before calling this function
    ///
    /// * Kernel logging is initialized.
    /// * Fine-grained kernel page table is set up (if applicable).
    /// * Physical memory allocation is initialized (if applicable).
    ///
    /// # After calling this function
    ///
    /// * Interrupt controller is initialized (if applicable).
    /// * Timer interrupts are enabled (if applicable).
    /// * Other platform devices are initialized.
    fn init_later(_cpu_id: usize, _arg: usize) {
        somehal::mem::flush_tlb(None);
        #[cfg(feature = "smp")]
        crate::smp::init();

        unsafe extern "C" {
            fn _percpu_start();
        }
        crate::time::enable();
        debug!("drivers setup...");
        driver::setup();
        #[cfg(feature = "irq")]
        {
            crate::irq::init();
            crate::irq::init_current_cpu();
            crate::time::enable_irqs();
        }
        crate::console::init();
    }

    /// Initializes the platform at the later stage for secondary cores.
    ///
    /// See [`init_later`] for details.
    #[cfg(feature = "smp")]
    fn init_later_secondary(_cpu_id: usize) {
        somehal::mem::flush_tlb(None);

        crate::time::enable();
        #[cfg(feature = "irq")]
        {
            crate::irq::init_current_cpu();
            crate::time::enable_irqs();
        }
    }
}
