use alloc::boxed::Box;
use core::{
    cell::UnsafeCell,
    hint::spin_loop,
    ptr::NonNull,
    sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering},
};

use arm_gic_driver::fdt_parse_irq_config;
use axplat::{console::ConsoleIf, mem::phys_to_virt};
use fdt_parser::Fdt;
use log::{info, warn};
use some_serial::{BIrqHandler, BReciever, BSender, BSerial, InterruptMask, ns16550, pl011};
use somehal::boot_info;
use spin::Mutex;

static TX: Mutex<Option<BSender>> = Mutex::new(None);
static RX: Mutex<Option<BReciever>> = Mutex::new(None);
static IRQ_NUM: AtomicU32 = AtomicU32::new(0);
static DEBUG_BASE: AtomicUsize = AtomicUsize::new(0);
static DEBUG_DEV_ID: AtomicU64 = AtomicU64::new(0);
static DEBUG_IRQ_HANDLER: DebugIrqHandler = DebugIrqHandler(UnsafeCell::new(None));

struct DebugIrqHandler(UnsafeCell<Option<BIrqHandler>>);
unsafe impl Sync for DebugIrqHandler {}
unsafe impl Send for DebugIrqHandler {}

pub(crate) fn setup_early() -> Option<()> {
    let ptr = boot_info().fdt?;
    let fdt = Fdt::from_ptr(ptr).ok()?;
    let choson = fdt.chosen()?;
    let node = choson.debugcon()?;

    let reg = node.reg()?.next()?;
    DEBUG_BASE.store(reg.address as usize, core::sync::atomic::Ordering::Release);
    if let Some(mut irq) = node.interrupts()
        && let Some(irq) = irq.next()
    {
        let mut raw = [0u32; 3];
        for (i, v) in irq.enumerate() {
            raw[i] = v;
        }
        let config = fdt_parse_irq_config(&raw).unwrap();
        IRQ_NUM.store(config.id.to_u32(), core::sync::atomic::Ordering::Release);
    }

    Some(())
}

pub(crate) fn init() -> Option<()> {
    if set_serial().is_some() {
        return Some(());
    }

    let fdt = boot_info().fdt?.as_ptr() as usize;
    let ptr = phys_to_virt(fdt.into()).as_mut_ptr();
    let fdt = Fdt::from_ptr(NonNull::new(ptr).unwrap()).ok()?;

    let choson = fdt.chosen()?;
    let node = choson.debugcon()?;

    let base_reg = node.reg()?.next()?;
    let mmio_base =
        NonNull::new(phys_to_virt((base_reg.address as usize).into()).as_mut_ptr()).unwrap();
    let mut serial: Option<BSerial> = None;
    for cmp in node.compatibles() {
        info!("debugcon compatible: {}", cmp);
        if cmp == "arm,pl011" {
            serial = Some(Box::new(pl011::Pl011::new(mmio_base, 0)));
            break;
        } else if cmp == "snps,dw-apb-uart" {
            serial = Some(Box::new(ns16550::Ns16550::new_mmio(mmio_base, 0)));
            break;
        }
    }

    if let Some(mut dev) = serial {
        info!("Debug Serial@{:#x} registered successfully", dev.base());

        // dev.enable_interrupts(InterruptMask::RX_AVAILABLE);
        dev.disable_interrupts(InterruptMask::RX_AVAILABLE | InterruptMask::TX_EMPTY);
        let tx = dev.take_tx()?;
        let rx = dev.take_rx()?;
        let handler = dev.irq_handler()?;
        handler.clean_interrupt_status();
        *TX.lock() = Some(tx);
        *RX.lock() = Some(rx);
        unsafe { *DEBUG_IRQ_HANDLER.0.get() = Some(handler) };
    }

    Some(())
}

fn set_serial() -> Option<()> {
    let base = phys_to_virt(DEBUG_BASE.load(Ordering::Acquire).into()).as_usize();
    for dev in rdrive::get_list::<BSerial>() {
        let mut dev = dev.lock().unwrap();
        if dev.base() == base {
            DEBUG_DEV_ID.store(dev.descriptor().device_id().into(), Ordering::Release);
            dev.disable_interrupts(InterruptMask::RX_AVAILABLE | InterruptMask::TX_EMPTY);

            // dev.enable_interrupts(InterruptMask::RX_AVAILABLE);
            let tx = dev.take_tx()?;
            let rx = dev.take_rx()?;
            let handler = dev.irq_handler()?;
            handler.clean_interrupt_status();
            *TX.lock() = Some(tx);
            *RX.lock() = Some(rx);
            unsafe { *DEBUG_IRQ_HANDLER.0.get() = Some(handler) };
            return Some(());
        }
    }
    None
}

#[unsafe(no_mangle)]
unsafe extern "C" fn handle_console_irq(irq: u32) {
    if irq == IRQ_NUM.load(Ordering::Acquire) {
        let handler = unsafe { &mut *DEBUG_IRQ_HANDLER.0.get() };
        if let Some(h) = handler {
            h.clean_interrupt_status();
        }
    }
}

struct ConsoleIfImpl;

#[impl_plat_interface]
impl ConsoleIf for ConsoleIfImpl {
    /// Writes given bytes to the console.
    fn write_bytes(bytes: &[u8]) {
        let mut g = TX.lock();
        if let Some(tx) = g.as_mut() {
            let mut bytes = bytes;
            while !bytes.is_empty() {
                match tx.send(bytes) {
                    Ok(written) => bytes = &bytes[written..],
                    Err(_) => break,
                }
            }
        } else {
            let _ = somehal::early_debug::write_bytes(bytes);
        }
    }

    /// Reads bytes from the console into the given mutable slice.
    ///
    /// Returns the number of bytes read.
    fn read_bytes(bytes: &mut [u8]) -> usize {
        if let Some(rx) = RX.lock().as_mut() {
            for _ in 0..10000 {
                spin_loop();
            }
            // warn!("Console read_bytes called, len={}", bytes.len());
            match rx.recive(bytes) {
                Ok(n) => {
                    // warn!("Console read {:?}", &bytes[..n]);
                    n
                }
                Err(e) => {
                    warn!("Console read error: {:?}", e);
                    0
                }
            }
        } else {
            0
        }
    }

    /// Returns the IRQ number for the console, if applicable.
    #[cfg(feature = "irq")]
    fn irq_number() -> Option<u32> {
        return None;
        // let irq = IRQ_NUM.load(core::sync::atomic::Ordering::Acquire);
        // if irq != 0 { Some(irq) } else { None }
    }
}

// fn getchar() -> Option<u8> {
// let mut g = RX.lock();
// if let Some(rx) = g.as_mut() {
//     rx.read().ok()
// } else {
//     None
// }
// }
