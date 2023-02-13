use ::libc;
use super::internal::Memory::*;
extern "C" {
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
}
pub type uint32_t = libc::c_uint;
pub type uint32 = uint32_t;
pub type Error = uint32;
#[no_mangle]
pub unsafe extern "C" fn Error_Print(mut e: Error) {
    printf(b"ERROR: \0" as *const u8 as *const libc::c_char);
    if e & 0x10000 as libc::c_int as libc::c_uint != 0 {
        printf(b"Stack \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x20000 as libc::c_int as libc::c_uint != 0 {
        printf(b"Heap \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x40000 as libc::c_int as libc::c_uint != 0 {
        printf(b"Buffer \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x80000 as libc::c_int as libc::c_uint != 0 {
        printf(b"Path \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x100000 as libc::c_int as libc::c_uint != 0 {
        printf(b"Index \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x200000 as libc::c_int as libc::c_uint != 0 {
        printf(b"Vertex \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x400000 as libc::c_int as libc::c_uint != 0 {
        printf(b"Vertex Position \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x800000 as libc::c_int as libc::c_uint != 0 {
        printf(b"Vertex Normal \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x1000000 as libc::c_int as libc::c_uint != 0 {
        printf(b"Vertex UV \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x100 as libc::c_int as libc::c_uint != 0 {
        printf(b"Input \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x200 as libc::c_int as libc::c_uint != 0 {
        printf(b"Intermediate Value \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x400 as libc::c_int as libc::c_uint != 0 {
        printf(b"Output \0" as *const u8 as *const libc::c_char);
    }
    if e & 0x1 as libc::c_int as libc::c_uint != 0 {
        printf(b"NULL\0" as *const u8 as *const libc::c_char);
    }
    if e & 0x2 as libc::c_int as libc::c_uint != 0 {
        printf(b"Invalid\0" as *const u8 as *const libc::c_char);
    }
    if e & 0x4 as libc::c_int as libc::c_uint != 0 {
        printf(b"Overflow\0" as *const u8 as *const libc::c_char);
    }
    if e & 0x8 as libc::c_int as libc::c_uint != 0 {
        printf(b"Underflow\0" as *const u8 as *const libc::c_char);
    }
    if e & 0x10 as libc::c_int as libc::c_uint != 0 {
        printf(b"Empty\0" as *const u8 as *const libc::c_char);
    }
    if e & 0x20 as libc::c_int as libc::c_uint != 0 {
        printf(b"NaN\0" as *const u8 as *const libc::c_char);
    }
    if e & 0x40 as libc::c_int as libc::c_uint != 0 {
        printf(b"Degenerate\0" as *const u8 as *const libc::c_char);
    }
    if e & 0x80 as libc::c_int as libc::c_uint != 0 {
        printf(b"Incorrect Count\0" as *const u8 as *const libc::c_char);
    }
    if e == 0 as libc::c_int as libc::c_uint {
        printf(b"None!\0" as *const u8 as *const libc::c_char);
    }
    printf(b"\n\0" as *const u8 as *const libc::c_char);
}
