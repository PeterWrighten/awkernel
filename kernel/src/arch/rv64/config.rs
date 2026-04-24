pub const PREEMPT_IRQ: u16 = 0;
pub const WAKEUP_IRQ: u16 = 1;

/// Heap starts after the kernel image and per-hart stacks (4 harts × 8 MiB each).
pub const HEAP_START: usize = 0x8800_0000;
