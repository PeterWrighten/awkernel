//! The RV64GC context switch.
//!
//! This kernel runs in M-mode on RV64GC, so the M-mode ABI applies:
//! the integer callee-saved registers (`ra`, `sp`, `s0`-`s11`) and the
//! floating-point callee-saved registers (`fs0`-`fs11`, i.e. `f8`, `f9`
//! and `f18`-`f27` of the D extension) must be preserved across a
//! switch.
//!
//! Floating-point state is only reachable because each hart enables
//! `mstatus.FS` during initialization (`awkernel_lib::arch::rv64::init_fpu`);
//! with the reset value `FS = Off`, any FP instruction (including the
//! `fsd`/`fld` below) traps as an illegal instruction.
//!
//! Task stacks are allocated dynamically from the heap (`TALLOC`), so
//! this code must stay position-independent and must not assume any
//! static stack placement.
use core::arch::global_asm;

/// Callee-saved register frame.
///
/// The field order and the offsets asserted below must match the
/// `context_switch` assembly.
#[derive(Debug, Copy, Clone, Default)]
#[repr(C)]
pub struct Context {
    /// Return address (`x1`). Points to `entry_point` for a fresh context.
    pub ra: usize, // offset 0
    /// Stack pointer (`x2`).
    pub sp: usize, // offset 8
    /// `s0` (`x8`). Holds the entry argument for a fresh context.
    pub s0: usize, // offset 16
    /// `s1` (`x9`). Holds the entry function for a fresh context.
    pub s1: usize, // offset 24
    pub s2: usize,  // offset 32
    pub s3: usize,  // offset 40
    pub s4: usize,  // offset 48
    pub s5: usize,  // offset 56
    pub s6: usize,  // offset 64
    pub s7: usize,  // offset 72
    pub s8: usize,  // offset 80
    pub s9: usize,  // offset 88
    pub s10: usize, // offset 96
    pub s11: usize, // offset 104
    /// `fs0` (`f8`).
    pub fs0: usize, // offset 112
    pub fs1: usize, // offset 120
    pub fs2: usize, // offset 128
    pub fs3: usize, // offset 136
    pub fs4: usize, // offset 144
    pub fs5: usize, // offset 152
    pub fs6: usize, // offset 160
    pub fs7: usize, // offset 168
    pub fs8: usize, // offset 176
    pub fs9: usize, // offset 184
    pub fs10: usize, // offset 192
    pub fs11: usize, // offset 200
}

const _: () = {
    assert!(core::mem::size_of::<Context>() == 26 * 8);

    assert!(core::mem::offset_of!(Context, ra) == 0);
    assert!(core::mem::offset_of!(Context, sp) == 8);
    assert!(core::mem::offset_of!(Context, s0) == 16);
    assert!(core::mem::offset_of!(Context, s1) == 24);
    assert!(core::mem::offset_of!(Context, s2) == 32);
    assert!(core::mem::offset_of!(Context, s3) == 40);
    assert!(core::mem::offset_of!(Context, s4) == 48);
    assert!(core::mem::offset_of!(Context, s5) == 56);
    assert!(core::mem::offset_of!(Context, s6) == 64);
    assert!(core::mem::offset_of!(Context, s7) == 72);
    assert!(core::mem::offset_of!(Context, s8) == 80);
    assert!(core::mem::offset_of!(Context, s9) == 88);
    assert!(core::mem::offset_of!(Context, s10) == 96);
    assert!(core::mem::offset_of!(Context, s11) == 104);
    assert!(core::mem::offset_of!(Context, fs0) == 112);
    assert!(core::mem::offset_of!(Context, fs11) == 200);
};

global_asm!(
    "
.attribute arch, \"rv64gc\"
.global context_switch
context_switch:
    // a0 = current, a1 = next

    // Store floating-point callee-saved registers.
    fsd fs0,  112(a0)
    fsd fs1,  120(a0)
    fsd fs2,  128(a0)
    fsd fs3,  136(a0)
    fsd fs4,  144(a0)
    fsd fs5,  152(a0)
    fsd fs6,  160(a0)
    fsd fs7,  168(a0)
    fsd fs8,  176(a0)
    fsd fs9,  184(a0)
    fsd fs10, 192(a0)
    fsd fs11, 200(a0)

    // Store general purpose callee-saved registers.
    sd ra,   0(a0)
    sd sp,   8(a0)
    sd s0,  16(a0)
    sd s1,  24(a0)
    sd s2,  32(a0)
    sd s3,  40(a0)
    sd s4,  48(a0)
    sd s5,  56(a0)
    sd s6,  64(a0)
    sd s7,  72(a0)
    sd s8,  80(a0)
    sd s9,  88(a0)
    sd s10, 96(a0)
    sd s11, 104(a0)

    // Load floating-point callee-saved registers.
    fld fs0,  112(a1)
    fld fs1,  120(a1)
    fld fs2,  128(a1)
    fld fs3,  136(a1)
    fld fs4,  144(a1)
    fld fs5,  152(a1)
    fld fs6,  160(a1)
    fld fs7,  168(a1)
    fld fs8,  176(a1)
    fld fs9,  184(a1)
    fld fs10, 192(a1)
    fld fs11, 200(a1)

    // Load general purpose callee-saved registers.
    ld ra,   0(a1)
    ld sp,   8(a1)
    ld s0,  16(a1)
    ld s1,  24(a1)
    ld s2,  32(a1)
    ld s3,  40(a1)
    ld s4,  48(a1)
    ld s5,  56(a1)
    ld s6,  64(a1)
    ld s7,  72(a1)
    ld s8,  80(a1)
    ld s9,  88(a1)
    ld s10, 96(a1)
    ld s11, 104(a1)

    ret
"
);

impl crate::context::Context for Context {
    unsafe fn set_stack_pointer(&mut self, sp: usize) {
        self.sp = sp;
    }

    unsafe fn set_entry_point(&mut self, entry: extern "C" fn(usize) -> !, arg: usize) {
        self.s0 = arg;
        self.s1 = entry as usize;
        self.ra = entry_point as *const () as usize;
    }

    unsafe fn set_argument(&mut self, arg: usize) {
        self.s0 = arg;
    }
}

extern "C" {
    fn entry_point() -> !;
}

global_asm!(
    "
.attribute arch, \"rv64gc\"
.global entry_point
entry_point:
    mv a0, s0     // a0 = arg
    jalr s1       // jump to entry(arg); it never returns
1:
    wfi
    j 1b
"
);
