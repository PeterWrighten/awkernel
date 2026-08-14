//! Floating-point unit management.

/// `mstatus.FS` = Initial (`0b01 << 13`).
///
/// `Initial` permits FP instructions; the hardware transitions it to
/// `Dirty` on the first FP register write, so no further software
/// action is required.
const MSTATUS_FS_INITIAL: usize = 0b01 << 13;

/// Enable the floating-point unit for the current hart.
///
/// On reset `mstatus.FS` is `Off` and any floating-point instruction —
/// including the `fsd`/`fld` in [`crate::context`]'s `context_switch` —
/// traps as an illegal instruction. Each hart must call this once
/// during initialization, before the first context switch.
///
/// This kernel runs in M-mode, so the M-mode status register is used.
///
/// # Safety
///
/// Must be called once per hart, before any floating-point instruction
/// is executed on that hart.
pub unsafe fn init_fpu() {
    unsafe {
        core::arch::asm!(
            "csrs mstatus, {bits}",
            bits = in(reg) MSTATUS_FS_INITIAL,
        );
    }
}
