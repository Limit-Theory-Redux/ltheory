use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use libc;

pub type Error = u32;

#[no_mangle]
pub unsafe extern "C" fn Error_Print(e: Error) {
    libc::printf(c_str!("ERROR: "));
    if e & 0x10000 != 0 {
        libc::printf(c_str!("Stack "));
    }
    if e & 0x20000 != 0 {
        libc::printf(c_str!("Heap "));
    }
    if e & 0x40000 != 0 {
        libc::printf(c_str!("Buffer "));
    }
    if e & 0x80000 != 0 {
        libc::printf(c_str!("Path "));
    }
    if e & 0x100000 != 0 {
        libc::printf(c_str!("Index "));
    }
    if e & 0x200000 != 0 {
        libc::printf(c_str!("Vertex "));
    }
    if e & 0x400000 != 0 {
        libc::printf(c_str!("Vertex Position "));
    }
    if e & 0x800000 != 0 {
        libc::printf(c_str!("Vertex Normal "));
    }
    if e & 0x1000000 != 0 {
        libc::printf(c_str!("Vertex UV "));
    }
    if e & 0x100 != 0 {
        libc::printf(c_str!("Input "));
    }
    if e & 0x200 != 0 {
        libc::printf(c_str!("Intermediate Value "));
    }
    if e & 0x400 != 0 {
        libc::printf(c_str!("Output "));
    }
    if e & 0x1 != 0 {
        libc::printf(c_str!("NULL"));
    }
    if e & 0x2 != 0 {
        libc::printf(c_str!("Invalid"));
    }
    if e & 0x4 != 0 {
        libc::printf(c_str!("Overflow"));
    }
    if e & 0x8 != 0 {
        libc::printf(c_str!("Underflow"));
    }
    if e & 0x10 != 0 {
        libc::printf(c_str!("Empty"));
    }
    if e & 0x20 != 0 {
        libc::printf(c_str!("NaN"));
    }
    if e & 0x40 != 0 {
        libc::printf(c_str!("Degenerate"));
    }
    if e & 0x80 != 0 {
        libc::printf(c_str!("Incorrect Count"));
    }
    if e == 0 {
        libc::printf(c_str!("None!"));
    }
    libc::printf(c_str!("\n"));
}
