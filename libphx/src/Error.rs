use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use libc;

pub type Error = u32;

#[no_mangle]
pub unsafe extern "C" fn Error_Print(e: Error) {
    CPrintf!("ERROR: ");
    if e & 0x10000 != 0 {
        CPrintf!("Stack ");
    }
    if e & 0x20000 != 0 {
        CPrintf!("Heap ");
    }
    if e & 0x40000 != 0 {
        CPrintf!("Buffer ");
    }
    if e & 0x80000 != 0 {
        CPrintf!("Path ");
    }
    if e & 0x100000 != 0 {
        CPrintf!("Index ");
    }
    if e & 0x200000 != 0 {
        CPrintf!("Vertex ");
    }
    if e & 0x400000 != 0 {
        CPrintf!("Vertex Position ");
    }
    if e & 0x800000 != 0 {
        CPrintf!("Vertex Normal ");
    }
    if e & 0x1000000 != 0 {
        CPrintf!("Vertex UV ");
    }
    if e & 0x100 != 0 {
        CPrintf!("Input ");
    }
    if e & 0x200 != 0 {
        CPrintf!("Intermediate Value ");
    }
    if e & 0x400 != 0 {
        CPrintf!("Output ");
    }
    if e & 0x1 != 0 {
        CPrintf!("NULL");
    }
    if e & 0x2 != 0 {
        CPrintf!("Invalid");
    }
    if e & 0x4 != 0 {
        CPrintf!("Overflow");
    }
    if e & 0x8 != 0 {
        CPrintf!("Underflow");
    }
    if e & 0x10 != 0 {
        CPrintf!("Empty");
    }
    if e & 0x20 != 0 {
        CPrintf!("NaN");
    }
    if e & 0x40 != 0 {
        CPrintf!("Degenerate");
    }
    if e & 0x80 != 0 {
        CPrintf!("Incorrect Count");
    }
    if e == 0 {
        CPrintf!("None!");
    }
    CPrintf!("\n");
}
