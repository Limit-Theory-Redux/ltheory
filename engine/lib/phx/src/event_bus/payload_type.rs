#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventPayloadType {
    /// Lua object pointer to communicate inside scripts only
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

    Table,
}
