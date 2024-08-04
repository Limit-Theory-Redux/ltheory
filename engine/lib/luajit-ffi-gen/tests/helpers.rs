#[derive(Debug, Default, Clone, PartialEq)]
pub struct Data {
    pub val: u32,
}

impl Data {
    pub const fn new(val: u32) -> Data {
        Data { val }
    }
}

// This is a well known copyable type defined in type_info.rs
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct CopyableData {
    pub val: u32,
}

impl CopyableData {
    pub const fn new(val: u32) -> CopyableData {
        CopyableData { val }
    }
}
