use alloc::{format, string::String};

pub use arm_gic_driver::v3::Gic;
use arm_gic_driver::v3::*;
use lazyinit::LazyInit;
use log::*;
use spin::Mutex;

use super::IRQ_HANDLER_TABLE;
use crate::irq;

#[percpu::def_percpu]
pub static CPU_IF: LazyInit<Mutex<CpuInterface>> = LazyInit::new();
pub static TRAP: LazyInit<TrapOp> = LazyInit::new();

fn use_gicd(f: impl FnOnce(&mut Gic)) {
    let mut gic = irq::get_gicd().lock().unwrap();
    f(gic.typed_mut::<Gic>().expect("GICD is not initialized"));
}

pub fn init_current_cpu() {
    CPU_IF.with_current(|c| {
        let mut cpu = c.lock();
        cpu.init_current_cpu().unwrap();
        #[cfg(feature = "hv")]
        cpu.set_eoi_mode(true);
    });
}

pub fn handle(_unused: usize) {
    let ack = TRAP.ack1();
    let irq_num = ack.to_u32();
    if ack.is_special() {
        return;
    }

    // let cpu_id = crate::irq::current_cpu();
    // warn!("[{cpu_id}] IRQ {}", irq_num);
    if !IRQ_HANDLER_TABLE.handle(irq_num as _) {
        warn!("Unhandled IRQ {irq_num}");
    }

    TRAP.eoi1(ack);
    if TRAP.eoi_mode() {
        TRAP.dir(ack);
    }
}

pub(crate) fn set_enable(irq_raw: usize, trigger: Option<Trigger>, enabled: bool) {
    debug!(
        "IRQ({:#x}) set enable: {}, {}",
        irq_raw,
        enabled,
        match trigger {
            Some(t) => format!("trigger: {t:?}"),
            None => String::new(),
        }
    );
    let id = unsafe { IntId::raw(irq_raw as _) };
    if id.is_private() {
        CPU_IF.with_current(|c| {
            let c = c.lock();

            if let Some(t) = trigger {
                c.set_cfg(id, t);
            }
            c.set_irq_enable(id, enabled);
        });
    } else {
        use_gicd(|gic| {
            gic.set_target_cpu(id, Some(Affinity::current()));
            if let Some(t) = trigger {
                gic.set_cfg(id, t);
            }
            gic.set_irq_enable(id, enabled);
        });
    }
    debug!("IRQ({irq_raw:#x}) set enable done");
}

pub fn send_ipi(id: usize, target: axplat::irq::IpiTarget) {
    arm_gic_driver::v3::send_sgi(
        IntId::sgi(id as _),
        match target {
            axplat::irq::IpiTarget::Current { cpu_id: _ } => {
                SGITarget::List(TargetList::new([Affinity::current()]))
            }
            axplat::irq::IpiTarget::Other { cpu_id } => {
                #[cfg(feature = "smp")]
                {
                    let hw_id = crate::smp::cpu_idx_to_id(cpu_id);
                    SGITarget::List(TargetList::new([Affinity::from_mpidr(hw_id as _)]))
                }
                #[cfg(not(feature = "smp"))]
                {
                    return;
                }
            }
            axplat::irq::IpiTarget::AllExceptCurrent { cpu_id, cpu_num } => {
                #[cfg(feature = "smp")]
                {
                    let mut list = alloc::vec::Vec::new();
                    for i in 0..cpu_num {
                        if i != cpu_id {
                            let hw_id = crate::smp::cpu_idx_to_id(i);
                            list.push(Affinity::from_mpidr(hw_id as _));
                        }
                    }
                    SGITarget::List(TargetList::new(&list))
                }
                #[cfg(not(feature = "smp"))]
                {
                    return;
                }
            }
        },
    );
}
