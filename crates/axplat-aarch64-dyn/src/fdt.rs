use alloc::vec::Vec;

use arm_gic_driver::{IntId, fdt_parse_irq_config, v3::Trigger};

use crate::fdt;

pub fn find_trigger(irq_raw: usize) -> Option<Trigger> {
    let id = unsafe { IntId::raw(irq_raw as _) };

    let mut trigger = None;
    let fdt = fdt();
    for node in fdt.all_nodes() {
        if let Some(irqs) = node.interrupts() {
            for irq in irqs {
                let one = irq.collect::<Vec<_>>();

                if one.is_empty() {
                    continue;
                }

                if let Ok(c) = fdt_parse_irq_config(&one)
                    && c.id == id
                {
                    trigger = Some(c.trigger);
                    break;
                }
            }
        }
    }

    trigger
}
