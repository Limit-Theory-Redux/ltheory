// This represents a managed custom type that is managed by ffi.gc in LuaJIT.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ManagedData {
    pub val: u32,
}

impl ManagedData {
    pub const fn new(val: u32) -> ManagedData {
        ManagedData { val }
    }
}
// This represents a well known copyable custom type that is not managed by
// ffi.gc in LuaJIT, listed in COPY_TYPES in type_info.rs
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct CopyableData {
    pub val: u32,
}

impl CopyableData {
    pub const fn new(val: u32) -> CopyableData {
        CopyableData { val }
    }
}
