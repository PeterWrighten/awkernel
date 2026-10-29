use crate::sync::mcs::MCSNode;

#[cfg(feature = "rv32")]
use super::rv32::RV32 as ArchImpl;
#[cfg(feature = "rv64")]
use super::rv64::RV64 as ArchImpl;

pub(super) mod address;
pub mod barrier;
pub mod clint;
pub(super) mod cpu;
pub(super) mod delay;
pub(super) mod frame_allocator;
pub(super) mod interrupt;
pub mod interrupt_controller;
pub(super) mod page_table;
pub(super) mod paging;
pub mod plic;
pub(super) mod power;
pub(super) mod vm;

/// Initialize the frame allocator over `[start, end)`.
/// The range is used only for page table frames and must not overlap the heap.
pub fn init_page_allocator(start: usize, end: usize) {
    frame_allocator::init_page_allocator(start, end);
}

/// Build the kernel page table and identity-map the page-table-frame region
/// `[pt_start, heap_start)` and the heap `[heap_start, memory_end)`.
/// Must be called after `init_page_allocator` and the heap are initialized.
pub fn init_kernel_space(pt_start: usize, heap_start: usize, memory_end: usize) {
    vm::init_kernel_space(pt_start, heap_start, memory_end);
}

pub fn activate_kernel_space() {
    vm::activate_kernel_space();
}

pub fn kernel_token() -> usize {
    vm::kernel_token()
}

pub fn translate_kernel_address(vpn: address::VirtPageNum) -> Option<page_table::PageTableEntry> {
    let mut node = MCSNode::new();
    let mut kernel_space = vm::KERNEL_SPACE.lock(&mut node);
    if let Some(ref mut space) = *kernel_space {
        space.translate(vpn)
    } else {
        None
    }
}
