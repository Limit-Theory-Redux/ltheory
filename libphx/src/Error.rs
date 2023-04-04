use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use libc;

/* --- Error -------------------------------------------------------------------
 *
 *   A compact error status encoding.
 *
 *    high <---                                               ---> low
 *
 *           2 bytes               byte               byte
 *     [ operand location ]  [ operand type ]   [ failure type ]
 *
 *
 *   For example:
 *     0x00000101 -> received null pointer
 *     0x00080102 -> received invalid path
 *     0x00040404 -> output buffer overflow
 *     0x00000220 -> NaN detected in intermediate computation
 *     0x00200120 -> NaN detected in vertex normal of input
 *
 *   TODO : Do this in a better way, such that error information can still be
 *          concatenated/built-up from sub-functions and subjected to partial
 *          queries, but without the limitations of a bitfield.
 *
 * -------------------------------------------------------------------------- */

pub type Error = u32;

const Error_None: Error = 0x00000000;

const Error_Null: Error = 0x00000001;
const Error_Invalid: Error = 0x00000002;
const Error_Overflow: Error = 0x00000004;
const Error_Underflow: Error = 0x00000008;
const Error_Empty: Error = 0x00000010;
const Error_NaN: Error = 0x00000020;
const Error_Degenerate: Error = 0x00000040;
const Error_BadCount: Error = 0x00000080;

const Error_Input: Error = 0x00000100;
const Error_Intermediate: Error = 0x00000200;
const Error_Output: Error = 0x00000400;

const Error_Stack: Error = 0x00010000;
const Error_Heap: Error = 0x00020000;
const Error_Buffer: Error = 0x00040000;
const Error_Path: Error = 0x00080000;
const Error_Index: Error = 0x00100000;
const Error_Vertex: Error = 0x00200000;
const Error_VertPos: Error = 0x00400000;
const Error_VertNorm: Error = 0x00800000;
const Error_VertUV: Error = 0x01000000;

#[no_mangle]
pub extern "C" fn Error_Print(e: Error) {
    CPrintf!("ERROR: ");
    if e & Error_Stack != 0 {
        CPrintf!("Stack ");
    }
    if e & Error_Heap != 0 {
        CPrintf!("Heap ");
    }
    if e & Error_Buffer != 0 {
        CPrintf!("Buffer ");
    }
    if e & Error_Path != 0 {
        CPrintf!("Path ");
    }
    if e & Error_Index != 0 {
        CPrintf!("Index ");
    }
    if e & Error_Vertex != 0 {
        CPrintf!("Vertex ");
    }
    if e & Error_VertPos != 0 {
        CPrintf!("Vertex Position ");
    }
    if e & Error_VertNorm != 0 {
        CPrintf!("Vertex Normal ");
    }
    if e & Error_VertUV != 0 {
        CPrintf!("Vertex UV ");
    }

    if e & Error_Input != 0 {
        CPrintf!("Input ");
    }
    if e & Error_Intermediate != 0 {
        CPrintf!("Intermediate Value ");
    }
    if e & Error_Output != 0 {
        CPrintf!("Output ");
    }

    if e & Error_Null != 0 {
        CPrintf!("NULL");
    }
    if e & Error_Invalid != 0 {
        CPrintf!("Invalid");
    }
    if e & Error_Overflow != 0 {
        CPrintf!("Overflow");
    }
    if e & Error_Underflow != 0 {
        CPrintf!("Underflow");
    }
    if e & Error_Empty != 0 {
        CPrintf!("Empty");
    }
    if e & Error_NaN != 0 {
        CPrintf!("NaN");
    }
    if e & Error_Degenerate != 0 {
        CPrintf!("Degenerate");
    }
    if e & Error_BadCount != 0 {
        CPrintf!("Incorrect Count");
    }

    if e == Error_None {
        CPrintf!("None!");
    }
    CPrintf!("\n");
}
