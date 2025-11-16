use alloc::{format, string::String};

pub use arm_gic_driver::v2::Gic;
use arm_gic_driver::v2::*;
use lazyinit::LazyInit;
use log::*;
use spin::Mutex;

use super::IRQ_HANDLER_TABLE;
use crate::irq::{self, current_cpu};

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
        cpu.init_current_cpu();
        #[cfg(feature = "hv")]
        cpu.set_eoi_mode_ns(true);
    })
}

pub fn handle(_unused: usize) {
    let ack = TRAP.ack();
    let intid = match ack {
        Ack::SGI { intid, cpu_id: _ } => intid,
        Ack::Other(intid) => intid,
    };
    if intid.is_special() {
        return;
    }

    let irq_num = intid.to_u32();

    // if irq_num == 0x21 {
    //     info!("1");
    // }
    // info!("IRQ {}", irq_num);
    if !IRQ_HANDLER_TABLE.handle(irq_num as _) {
        warn!("Unhandled IRQ {irq_num}");
    }

    TRAP.eoi(ack);
    if TRAP.eoi_mode_ns() {
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
            let cpu = c.lock();
            cpu.set_irq_enable(id, enabled);

            if let Some(t) = trigger {
                cpu.set_cfg(id, t);
            }
        });
    } else {
        use_gicd(|gic| {
            debug!("IRQ({irq_raw:#x}) set enable done, set target cpu");
            gic.set_target_cpu(id, TargetList::new([current_cpu()].into_iter()));
            debug!("IRQ({irq_raw:#x}) set enable done, set cfg");
            if let Some(t) = trigger {
                gic.set_cfg(id, t);
            }
            debug!("set enable irq {irq_raw} on gicd");
            gic.set_irq_enable(id, enabled);
        });
    }
    debug!("IRQ({irq_raw:#x}) set enable done");
}

pub fn send_ipi(id: usize, target: axplat::irq::IpiTarget) {
    use_gicd(|gic| {
        gic.send_sgi(
            IntId::sgi(id as _),
            match target {
                axplat::irq::IpiTarget::Current { cpu_id: _ } => SGITarget::Current,
                axplat::irq::IpiTarget::Other { cpu_id } => {
                    SGITarget::TargetList(TargetList::new([cpu_id].into_iter()))
                }
                axplat::irq::IpiTarget::AllExceptCurrent {
                    cpu_id: _,
                    cpu_num: _,
                } => SGITarget::AllOther,
            },
        );
    });
}
