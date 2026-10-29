pub(super) mod address;
pub(super) mod dvfs;

use fdt::Fdt;

pub struct RV64;

impl super::Arch for RV64 {}

/// Return the end address of the RAM described by the device tree blob at `dtb_addr`.
///
/// The memory region containing the kernel image (i.e., `_start`) is used.
///
/// # Safety
///
/// `dtb_addr` must be the device tree pointer passed by the firmware in `a1`.
pub unsafe fn detect_memory_end(dtb_addr: usize) -> Option<usize> {
    extern "C" {
        fn _start();
    }
    let kernel_start = _start as *const () as usize;

    let fdt = unsafe { Fdt::from_ptr(dtb_addr as *const u8).ok()? };
    let memory_end = fdt.memory().regions().find_map(|region| {
        let start = region.starting_address as usize;
        let end = start.checked_add(region.size?)?;
        (start..end).contains(&kernel_start).then_some(end)
    });
    memory_end
}

/// Return the number of CPUs described by the device tree blob at `dtb_addr`.
///
/// # Safety
///
/// `dtb_addr` must be the device tree pointer passed by the firmware in `a1`.
pub unsafe fn detect_cpu_count(dtb_addr: usize) -> Option<usize> {
    let fdt = unsafe { Fdt::from_ptr(dtb_addr as *const u8).ok()? };
    let num_cpu = fdt.cpus().count();
    (num_cpu > 0).then_some(num_cpu)
}
