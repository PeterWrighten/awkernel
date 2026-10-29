use super::console;
use crate::{config::BACKUP_HEAP_SIZE, kernel_info::KernelInfo};
use awkernel_lib::{
    console::{unsafe_print_hex_u64, unsafe_puts},
    cpu, heap,
};
use core::{
    arch::global_asm,
    fmt::Write,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};
use ns16550a::Uart;

const UART_BASE: usize = 0x1000_0000;

/// Initial per-hart boot stacks set up by boot.S.
///
/// Hart `i` uses `[BOOT_STACK_BASE + BOOT_STACK_SIZE * i, BOOT_STACK_BASE + BOOT_STACK_SIZE * (i + 1))`.
/// These constants must match boot.S.
const BOOT_STACK_BASE: usize = 0x8080_0000;
const BOOT_STACK_SIZE: usize = 0x0080_0000;

/// Size of the page table frame region: 10% of the available RAM, clamped to this range.
const MIN_PT_FRAMES_SIZE: usize = 4 * 1024 * 1024;
const MAX_PT_FRAMES_SIZE: usize = 256 * 1024 * 1024;

const PAGE_MASK: usize = 0xfff;

extern "C" {
    /// The device tree pointer passed in `a1` by the firmware. Saved by boot.S.
    static dtb_ptr: usize;
}

static PRIMARY_INITIALIZED: AtomicBool = AtomicBool::new(false);
static NUM_CPUS: AtomicUsize = AtomicUsize::new(0);

global_asm!(include_str!("boot.S"));

#[no_mangle]
pub unsafe extern "C" fn kernel_main() {
    unsafe { crate::config::init() };

    let hartid: usize = cpu::cpu_id();
    if hartid == 0 {
        primary_hart(hartid);
    } else {
        non_primary_hart(hartid);
    }
}

unsafe fn primary_hart(hartid: usize) {
    // setup interrupt; TODO;

    super::console::init_port(UART_BASE);

    // Detect the RAM size and the number of CPUs from the device tree.
    let dtb = dtb_ptr;
    let Some(memory_end) = awkernel_lib::arch::rv64::detect_memory_end(dtb) else {
        panic!("failed to detect the memory size from the device tree");
    };
    let Some(num_cpu) = awkernel_lib::arch::rv64::detect_cpu_count(dtb) else {
        panic!("failed to detect the number of CPUs from the device tree");
    };

    unsafe_puts("Memory end: 0x");
    unsafe_print_hex_u64(memory_end as u64);
    unsafe_puts("\r\nNumber of CPUs: 0x");
    unsafe_print_hex_u64(num_cpu as u64);
    unsafe_puts("\r\n");

    // Initialize the page allocator, the heap, and the kernel page table.
    init_memory(memory_end, num_cpu);

    // initialize serial device and dump booting logo
    let mut port = Uart::new(UART_BASE);
    port.init(
        ns16550a::WordLength::EIGHT,
        ns16550a::StopBits::ONE,
        ns16550a::ParityBit::DISABLE,
        ns16550a::ParitySelect::EVEN,
        ns16550a::StickParity::DISABLE,
        ns16550a::Break::DISABLE,
        ns16550a::DMAMode::MODE0,
        ns16550a::Divisor::BAUD115200,
    );

    let _ = port.write_str("\r\nAwkernel is booting\r\n\r\n");

    // initialize console driver to which log messages are dumped
    console::init(UART_BASE);

    // switch to S-Mode; TODO;
    // * currntly this impl. holds both kernel and userland
    // * in M-Mode, which is the highest priority.

    // wake up another harts
    NUM_CPUS.store(num_cpu, Ordering::Relaxed);
    PRIMARY_INITIALIZED.store(true, Ordering::SeqCst);

    let kernel_info = KernelInfo {
        info: (),
        cpu_id: hartid,
        num_cpu,
    };

    crate::main::<()>(kernel_info);
}

/// Initialize the page allocator, the heap, and the kernel page table.
///
/// Physical memory layout:
///
/// ```text
/// | kernel image | boot stacks | page table frames | backup heap | primary heap |
///                               ^ alloc_base        ^ heap_start                ^ memory_end
/// ```
///
/// # Safety
///
/// Must be called once by the primary hart during boot.
/// `num_cpu` must be the number of harts which have a boot stack.
unsafe fn init_memory(memory_end: usize, num_cpu: usize) {
    extern "C" {
        fn ekernel();
    }

    // The boot stacks are in use, so allocate memory above them and the kernel image.
    let kernel_end = ekernel as *const () as usize;
    let stack_end = BOOT_STACK_BASE + BOOT_STACK_SIZE * num_cpu;
    let alloc_base = (kernel_end.max(stack_end) + PAGE_MASK) & !PAGE_MASK;

    let available = memory_end.saturating_sub(alloc_base);
    let pt_frames_size =
        ((available / 10).clamp(MIN_PT_FRAMES_SIZE, MAX_PT_FRAMES_SIZE) + PAGE_MASK) & !PAGE_MASK;

    let pt_start = alloc_base;
    let heap_start = pt_start + pt_frames_size;

    if heap_start + BACKUP_HEAP_SIZE >= memory_end {
        panic!("insufficient memory for the page table frames and the heap");
    }

    // Page table frames are allocated from `[pt_start, heap_start)`.
    awkernel_lib::arch::rv_common::init_page_allocator(pt_start, heap_start);

    // The heap uses `[heap_start, memory_end)`.
    let backup_start = heap_start;
    let backup_size = BACKUP_HEAP_SIZE;
    let primary_start = heap_start + BACKUP_HEAP_SIZE;
    let primary_size = memory_end - primary_start;

    heap::init_primary(primary_start, primary_size);
    heap::init_backup(backup_start, backup_size);
    heap::TALLOC.use_primary_then_backup(); // use backup allocator

    // Build and activate the kernel page table. This uses the heap.
    awkernel_lib::arch::rv_common::init_kernel_space(pt_start, heap_start, memory_end);
    awkernel_lib::arch::rv_common::activate_kernel_space();
}

unsafe fn non_primary_hart(hartid: usize) {
    while !PRIMARY_INITIALIZED.load(Ordering::SeqCst) {
        core::hint::spin_loop();
    }

    heap::TALLOC.use_primary_then_backup(); // use backup allocator

    let kernel_info = KernelInfo {
        info: (),
        cpu_id: hartid,
        num_cpu: NUM_CPUS.load(Ordering::Relaxed),
    };

    crate::main::<()>(kernel_info);
}
