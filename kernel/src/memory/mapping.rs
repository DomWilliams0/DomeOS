use core::{mem, panic};

use log::*;

use kernel_utils::memory::address::{PhysicalAddress, VirtualAddress};
use kernel_utils::memory::page_table::hierarchy::{Frame, P1, P2, P3, P4};
use kernel_utils::memory::page_table::{
    CommonEntry, Executable, Overwrite, PageTable, Writeable, PAGE_TABLE_ENTRY_COUNT,
};
use kernel_utils::memory::{kilobytes, megabytes, terabytes};

use crate::idt;
use crate::memory::page_table::{pml4, set_pml4};
use crate::multiboot::{multiboot_info, MemoryRegion, MemoryRegions};

/// Kernel virtual addresses: 128TiB -> 192TiB
/// Physical mapping:         192TiB -> 256TiB

/// Start of kernel code mapping at 128TiB
const KERNEL_START_ADDR: u64 = terabytes(128);

const KERNEL_MAPPING_LENGTH: u64 = terabytes(64);

/// Start of direct physical mapping at 128TiB
const PHYSICAL_MAPPING_OFFSET: u64 = terabytes(192);

/// Size of direct physical mapping
const PHYSICAL_MAPPING_LENGTH: u64 = terabytes(64);

extern "C" {
    static KERNEL_START: u64;
    static KERNEL_END: u64;
}

fn kernel_range() -> (u64, u64) {
    unsafe {
        (
            &KERNEL_START as *const _ as u64,
            &KERNEL_END as *const _ as u64,
        )
    }
}

struct DumbAllocator {
    next_free: PhysicalAddress,
}

const fn round_up(val: u64, multiple: u64) -> u64 {
    (val + (multiple - 1)) & !(multiple - 1)
}

const fn div_round_up(a: u64, b: u64) -> u64 {
    (a + b - 1) / b
}

impl DumbAllocator {
    fn new(start_addr: PhysicalAddress) -> Self {
        Self {
            next_free: start_addr,
        }
    }

    fn alloc<T: Sized>(&mut self) -> &'static mut T {
        // calculate aligned size
        let sz = round_up(core::mem::size_of::<T>() as u64, 8);
        let allocated_ref: &mut T = unsafe {
            let ptr = self.next_free.cast_mut();
            core::ptr::write_bytes(ptr, 0, 1);
            ptr
        };

        self.next_free += sz as u64;

        allocated_ref
    }
}

struct KernelStack {
    stack: [u8; 32768],
}

fn remap_kernel(reserved_region: MemoryRegion) {
    let (old_start, old_end) = kernel_range();
    let kernel_size = old_end - old_start;
    debug!(
        "kernel code ranges from {:#x?} to {:#x?}",
        old_start, old_end
    );

    let mut dumb_alloc = {
        // offset into reserved region if this overlaps with kernel code
        let start = if reserved_region.range().contains(&old_start) {
            PhysicalAddress(round_up(old_end, 4096))
        } else {
            reserved_region.base_addr
        };
        DumbAllocator::new(start)
    };

    let entries_per_table = PAGE_TABLE_ENTRY_COUNT as u64;

    // calculate page count
    let (n_pages, n_pagetables) = {
        let page_count = div_round_up(kernel_size, kilobytes(4)); // TODO huge 2MiB
        let pagetable_count = (page_count / entries_per_table) + 1;
        assert!(
            pagetable_count < entries_per_table,
            "kernel size is ridiculously big"
        );
        (page_count, pagetable_count)
    };

    let new_start = VirtualAddress::new(KERNEL_START_ADDR);

    debug!(
        "mapping {} pages across {} page tables starting at {:?}",
        n_pages, n_pagetables, new_start
    );

    assert_eq!(core::mem::size_of::<CommonEntry>(), 8);
    assert_eq!(core::mem::size_of::<PageTable<P3>>(), 4096);

    debug!("page tables start at {:?}", dumb_alloc.next_free);

    // allocate 1 of each table
    let p4_table = {
        let table: &mut PageTable<P3> = dumb_alloc.alloc();

        // inherit current
        let current = pml4();
        current.copy_to(table);

        table
    };

    let p3_table: &mut PageTable<P2> = dumb_alloc.alloc();
    let p2_table: &mut PageTable<P1> = dumb_alloc.alloc();

    p4_table[new_start.pml4t_offset()].init(
        PhysicalAddress(p3_table as *mut _ as u64),
        Writeable::Read,
        Executable::Executable,
        Overwrite::OverwriteExisting,
    );
    p3_table[new_start.pdp_offset()].init(
        PhysicalAddress(p2_table as *mut _ as u64),
        Writeable::Read,
        Executable::Executable,
        Overwrite::OverwriteExisting,
    );

    // TODO loop multiple p1s
    assert_eq!(n_pagetables, 1);

    let p1_table: &mut PageTable<Frame> = dumb_alloc.alloc();
    p2_table[new_start.pd_offset()].init(
        PhysicalAddress(p1_table as *mut _ as u64),
        Writeable::Read,
        Executable::Executable,
        Overwrite::OverwriteExisting,
    );

    // TODO map_region function
    let mut addr_phys = old_start;
    let mut addr_virt = new_start;
    for _ in 0..n_pages.min(entries_per_table) as u16 {
        let entry = &mut p1_table[addr_virt.pt_offset()];
        entry.init(
            PhysicalAddress(addr_phys),
            Writeable::Write,
            Executable::Executable,
            Overwrite::OverwriteExisting,
        );

        addr_phys += kilobytes(4);
        addr_virt += kilobytes(4);
    }

    // allocate new kernel stack too
    let kernel_stack = {
        let stack: &KernelStack = dumb_alloc.alloc();
        let stack_virt = VirtualAddress(round_up(new_start.0 + kernel_size, kilobytes(4)));
        let stack_size = mem::size_of::<KernelStack>() as u64;
        trace!(
            "mapping kernel stack of size {} at {:?}|{:?}",
            stack_size,
            PhysicalAddress::from(stack),
            stack_virt
        );
        map_region(
            p4_table,
            stack.into(),
            stack_size,
            stack_virt,
            Writeable::Write,
            Executable::Executable,
        );

        stack_virt + stack_size // start at the bottom, stack grows upwards
    };

    //    testy();
    let jmp = VirtualAddress::new(KERNEL_START_ADDR + testy as *const () as u64 - old_start);
    debug!("set rsp to {:?} and jmp to {:?}", kernel_stack, jmp);
    set_pml4(P4::PML4T(p4_table));
    idt::remap(KERNEL_START_ADDR);
    unsafe {
        llvm_asm!("mov $0, %rsp\n\
              jmp *$1" :: "r" (kernel_stack.0), "r" (jmp));
    }
}

fn map_region<'p>(
    p4: &mut PageTable<'p, P3<'p>>,
    start_phys: PhysicalAddress,
    len: u64,
    start_virt: VirtualAddress,
    w: Writeable,
    x: Executable,
) {
    let mut pages = round_up(len / kilobytes(4), 1); // TODO huge 2MiB

    let p1s = div_round_up(pages, PAGE_TABLE_ENTRY_COUNT as u64);

    let mut addr = start_virt;
    let mut phys = start_phys;
    let p3: &mut PageTable<P2> = unsafe { &mut p4[addr.pml4t_offset()].address().cast_mut() };
    let p2: &mut PageTable<P1> = unsafe { &mut p3[addr.pdp_offset()].address().cast_mut() };
    (0..p1s).for_each(|p1| {
        let p1: &mut PageTable<Frame> = unsafe { &mut p2[addr.pd_offset()].address().cast_mut() };
        let pages_this_round = pages.min(PAGE_TABLE_ENTRY_COUNT as u64);

        for _ in 0..pages_this_round as u16 {
            let offset = addr.pt_offset();
            let entry = &mut p1[offset];
            trace!("  mapping p1[{}] => {:#x}", offset, phys.0);
            entry.init(phys, w, x, Overwrite::MustNotExist);
            phys += kilobytes(4);
            addr.0 += kilobytes(4);
        }

        pages -= pages_this_round;
    })
}

fn testy() {
    let x = 5;
    debug!("hello from {:?}", &x as *const _);
}

pub fn init(multiboot: &multiboot_info) {
    const RESERVE_LENGTH: u64 = megabytes(4);
    for x in MemoryRegions::new(multiboot).available() {
        info!("can use {:?}", x);
    }
}
