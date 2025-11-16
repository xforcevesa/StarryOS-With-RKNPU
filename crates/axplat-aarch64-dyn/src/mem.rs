use core::ops::Range;

use axplat::mem::{MemIf, PhysAddr, RawRange, VirtAddr};
use heapless::Vec;
use log::trace;
use memory_addr::MemoryAddr;
use somehal::{KIMAGE_VADDR, KIMAGE_VSIZE, KLINER_OFFSET, MemoryRegionKind, boot_info};
use spin::Once;

struct MemIfImpl;

static RAM_LIST: Once<Vec<RawRange, 32>> = Once::new();
static RESERVED_LIST: Once<Vec<RawRange, 32>> = Once::new();
static MMIO: Once<Vec<RawRange, 32>> = Once::new();
static mut VA_OFFSET: usize = 0;

fn va_offset() -> usize {
    unsafe { VA_OFFSET }
}

pub fn setup() {
    unsafe {
        VA_OFFSET = boot_info().kimage_start_vma as usize - boot_info().kimage_start_lma as usize
    };

    RAM_LIST.call_once(|| {
        let mut ram_list = Vec::new();
        for region in boot_info()
            .memory_regions
            .iter()
            .filter(|one| matches!(one.kind, MemoryRegionKind::Ram))
            .map(|one| (one.start, one.end - one.start))
        {
            let _ = ram_list.push(region);
        }
        ram_list
    });

    RESERVED_LIST.call_once(|| {
        let mut rsv_list = Vec::new();

        unsafe extern "C" {
            fn _skernel();
        }
        let head_start = boot_info().kimage_start_lma as usize;
        let head_section = (head_start, (_skernel as usize) - va_offset() - head_start);

        rsv_list.push(head_section).unwrap();

        for region in boot_info()
            .memory_regions
            .iter()
            .filter(|one| {
                matches!(
                    one.kind,
                    MemoryRegionKind::Reserved | MemoryRegionKind::Bootloader
                )
            })
            .map(|one| {
                (
                    one.start.align_down_4k(),
                    one.end.align_up_4k() - one.start.align_down_4k(),
                )
            })
        {
            let _ = rsv_list.push(region);
        }

        rsv_list
    });

    MMIO.call_once(|| {
        let mut mmio_list = Vec::new();
        if let Some(debug) = &boot_info().debug_console {
            let start = debug.base_phys.align_down_4k();
            let _ = mmio_list.push((start, 0x1000));
        }

        mmio_list
    });
}

#[impl_plat_interface]
impl MemIf for MemIfImpl {
    /// Returns all physical memory (RAM) ranges on the platform.
    ///
    /// All memory ranges except reserved ranges (including the kernel loaded
    /// range) are free for allocation.
    fn phys_ram_ranges() -> &'static [RawRange] {
        let ls = RAM_LIST.wait();
        for range in ls {
            trace!("RAM range: {:#x?}", range);
        }

        ls
    }

    /// Returns all reserved physical memory ranges on the platform.
    ///
    /// Reserved memory can be contained in [`phys_ram_ranges`], they are not
    /// allocatable but should be mapped to kernel's address space.
    ///
    /// Note that the ranges returned should not include the range where the
    /// kernel is loaded.
    fn reserved_phys_ram_ranges() -> &'static [RawRange] {
        RESERVED_LIST.wait()
    }

    /// Returns all device memory (MMIO) ranges on the platform.
    fn mmio_ranges() -> &'static [RawRange] {
        MMIO.wait()
    }

    fn phys_to_virt(p: PhysAddr) -> VirtAddr {
        if kimage_range_phys().contains(&p) {
            VirtAddr::from_usize(p.as_usize() + va_offset())
        } else {
            // MMIO or other reserved regions
            VirtAddr::from_usize(p.as_usize() + KLINER_OFFSET)
        }
    }

    fn virt_to_phys(p: VirtAddr) -> PhysAddr {
        if (KIMAGE_VADDR..KIMAGE_VADDR + KIMAGE_VSIZE).contains(&p.as_usize()) {
            PhysAddr::from_usize(p.as_usize() - va_offset())
        } else {
            PhysAddr::from_usize(p.as_usize() - KLINER_OFFSET)
        }
    }
}

fn kimage_range_phys() -> Range<PhysAddr> {
    unsafe extern "C" {
        fn _ekernel();
    }

    let start = PhysAddr::from_usize(boot_info().kimage_start_lma as usize);
    let end = PhysAddr::from_usize(_ekernel as usize - va_offset());
    start..end
}
