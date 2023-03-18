use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use libc;

pub type Error = u32;

#[no_mangle]
pub unsafe extern "C" fn Error_Print(mut e: Error) {
    libc::printf(b"ERROR: \0" as *const u8 as *const libc::c_char);
    if e & 0x10000_u32 != 0 {
        libc::printf(b"Stack \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x20000_u32 != 0 {
        libc::printf(b"Heap \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x40000_u32 != 0 {
        libc::printf(b"Buffer \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x80000_u32 != 0 {
        libc::printf(b"Path \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x100000_u32 != 0 {
        libc::printf(b"Index \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x200000_u32 != 0 {
        libc::printf(b"Vertex \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x400000_u32 != 0 {
        libc::printf(b"Vertex Position \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x800000_u32 != 0 {
        libc::printf(b"Vertex Normal \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x1000000_u32 != 0 {
        libc::printf(b"Vertex UV \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x100_u32 != 0 {
        libc::printf(b"Input \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x200_u32 != 0 {
        libc::printf(b"Intermediate Value \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x400_u32 != 0 {
        libc::printf(b"Output \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x1_u32 != 0 {
        libc::printf(b"NULL\0" as *const u8 as *const libc::c_char);
    }
    if e & 0x2_u32 != 0 {
        libc::printf(b"Invalid\0" as *const u8 as *const libc::c_char);
    }
    if e & 0x4_u32 != 0 {
        libc::printf(b"Overflow\0" as *const u8 as *const libc::c_char);
    }
    if e & 0x8_u32 != 0 {
        libc::printf(b"Underflow\0" as *const u8 as *const libc::c_char);
    }
    if e & 0x10_u32 != 0 {
        libc::printf(b"Empty\0" as *const u8 as *const libc::c_char);
    }
    if e & 0x20_u32 != 0 {
        libc::printf(b"NaN\0" as *const u8 as *const libc::c_char);
    }
    if e & 0x40_u32 != 0 {
        libc::printf(b"Degenerate\0" as *const u8 as *const libc::c_char);
    }
    if e & 0x80_u32 != 0 {
        libc::printf(b"Incorrect Count\0" as *const u8 as *const libc::c_char);
    }
    if e == 0_u32 {
        libc::printf(b"None!\0" as *const u8 as *const libc::c_char);
    }
    libc::printf(b"\n\0" as *const u8 as *const libc::c_char);
}
