#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PayloadType {
    /// Lua object pointer/index to communicate inside scripts only
    Lua,

    Bool,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    String,

    BoolArray,
    I8Array,
    U8Array,
    I16Array,
    U16Array,
    I32Array,
    U32Array,
    I64Array,
    U64Array,
    F32Array,
    F64Array,
    StringArray,

    Table,
}
