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

const ERROR_NONE: Error = 0x00000000;

const ERROR_NULL: Error = 0x00000001;
const ERROR_INVALID: Error = 0x00000002;
const ERROR_OVERFLOW: Error = 0x00000004;
const ERROR_UNDERFLOW: Error = 0x00000008;
const ERROR_EMPTY: Error = 0x00000010;
const ERROR_NA_N: Error = 0x00000020;
const ERROR_DEGENERATE: Error = 0x00000040;
const ERROR_BAD_COUNT: Error = 0x00000080;

const ERROR_INPUT: Error = 0x00000100;
const ERROR_INTERMEDIATE: Error = 0x00000200;
const ERROR_OUTPUT: Error = 0x00000400;

const ERROR_STACK: Error = 0x00010000;
const ERROR_HEAP: Error = 0x00020000;
const ERROR_BUFFER: Error = 0x00040000;
const ERROR_PATH: Error = 0x00080000;
const ERROR_INDEX: Error = 0x00100000;
const ERROR_VERTEX: Error = 0x00200000;
const ERROR_VERT_POS: Error = 0x00400000;
const ERROR_VERT_NORM: Error = 0x00800000;
const ERROR_VERT_UV: Error = 0x01000000;

#[no_mangle]
pub extern "C" fn Error_Print(e: Error) {
    if e == ERROR_NONE {
        error!("None!");
        return;
    }

    let err_source = if e & ERROR_STACK != 0 {
        "Stack "
    } else if e & ERROR_HEAP != 0 {
        "Heap "
    } else if e & ERROR_BUFFER != 0 {
        "Buffer "
    } else if e & ERROR_PATH != 0 {
        "Path "
    } else if e & ERROR_INDEX != 0 {
        "Index "
    } else if e & ERROR_VERTEX != 0 {
        "Vertex "
    } else if e & ERROR_VERT_POS != 0 {
        "Vertex Position "
    } else if e & ERROR_VERT_NORM != 0 {
        "Vertex Normal "
    } else if e & ERROR_VERT_UV != 0 {
        "Vertex UV "
    } else {
        ""
    };

    let err_type = if e & ERROR_INPUT != 0 {
        "Input "
    } else if e & ERROR_INTERMEDIATE != 0 {
        "Intermediate Value "
    } else if e & ERROR_OUTPUT != 0 {
        "Output "
    } else {
        ""
    };

    let err_value = if e & ERROR_NULL != 0 {
        "NULL"
    } else if e & ERROR_INVALID != 0 {
        "Invalid"
    } else if e & ERROR_OVERFLOW != 0 {
        "Overflow"
    } else if e & ERROR_UNDERFLOW != 0 {
        "Underflow"
    } else if e & ERROR_EMPTY != 0 {
        "Empty"
    } else if e & ERROR_NA_N != 0 {
        "NaN"
    } else if e & ERROR_DEGENERATE != 0 {
        "Degenerate"
    } else if e & ERROR_BAD_COUNT != 0 {
        "Incorrect Count"
    } else {
        ""
    };

    error!("{err_source}{err_type}{err_value}");
}
