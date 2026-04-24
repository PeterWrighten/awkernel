use alloc::boxed::Box;
use awkernel_lib::interrupt::InterruptController;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::Ordering;

pub struct RiscvPlic {

}
