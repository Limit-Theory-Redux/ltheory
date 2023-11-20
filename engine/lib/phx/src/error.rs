use tracing::error;

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
    if e == Error_None {
        error!("None!");
        return;
    }

    let err_source = if e & Error_Stack != 0 {
        "Stack "
    } else if e & Error_Heap != 0 {
        "Heap "
    } else if e & Error_Buffer != 0 {
        "Buffer "
    } else if e & Error_Path != 0 {
        "Path "
    } else if e & Error_Index != 0 {
        "Index "
    } else if e & Error_Vertex != 0 {
        "Vertex "
    } else if e & Error_VertPos != 0 {
        "Vertex Position "
    } else if e & Error_VertNorm != 0 {
        "Vertex Normal "
    } else if e & Error_VertUV != 0 {
        "Vertex UV "
    } else {
        ""
    };

    let err_type = if e & Error_Input != 0 {
        "Input "
    } else if e & Error_Intermediate != 0 {
        "Intermediate Value "
    } else if e & Error_Output != 0 {
        "Output "
    } else {
        ""
    };

    let err_value = if e & Error_Null != 0 {
        "NULL"
    } else if e & Error_Invalid != 0 {
        "Invalid"
    } else if e & Error_Overflow != 0 {
        "Overflow"
    } else if e & Error_Underflow != 0 {
        "Underflow"
    } else if e & Error_Empty != 0 {
        "Empty"
    } else if e & Error_NaN != 0 {
        "NaN"
    } else if e & Error_Degenerate != 0 {
        "Degenerate"
    } else if e & Error_BadCount != 0 {
        "Incorrect Count"
    } else {
        ""
    };

    error!("{err_source}{err_type}{err_value}");
}
