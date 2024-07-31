use super::{EventPayloadTable, EventPayloadType};

#[derive(Debug, Clone, PartialEq)]
pub enum EventPayload {
    /// Lua object pointer/index to communicate inside scripts only
    Lua(u64),

    Bool(bool),
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    String(String),

    Table(Box<EventPayloadTable>),
}

#[luajit_ffi_gen::luajit_ffi]
impl EventPayload {
    pub fn from_lua(value: u64) -> Self {
        Self::Lua(value)
    }

    pub fn get_lua(&self) -> u64 {
        let Self::Lua(value) = self else {
            self.type_panic("Lua");
        };
        *value
    }

    pub fn from_bool(value: bool) -> Self {
        Self::Bool(value)
    }

    pub fn get_bool(&self) -> bool {
        let Self::Bool(value) = self else {
            self.type_panic("Bool");
        };
        *value
    }

    pub fn from_i8(value: i8) -> Self {
        Self::I8(value)
    }

    pub fn get_i8(&self) -> i8 {
        let Self::I8(value) = self else {
            self.type_panic("I8");
        };
        *value
    }

    pub fn from_u8(value: u8) -> Self {
        Self::U8(value)
    }

    pub fn get_u8(&self) -> u8 {
        let Self::U8(value) = self else {
            self.type_panic("U8");
        };
        *value
    }

    pub fn from_i16(value: i16) -> Self {
        Self::I16(value)
    }

    pub fn get_i16(&self) -> i16 {
        let Self::I16(value) = self else {
            self.type_panic("I16");
        };
        *value
    }

    pub fn from_u16(value: u16) -> Self {
        Self::U16(value)
    }

    pub fn get_u16(&self) -> u16 {
        let Self::U16(value) = self else {
            self.type_panic("U16");
        };
        *value
    }

    pub fn from_i32(value: i32) -> Self {
        Self::I32(value)
    }

    pub fn get_i32(&self) -> i32 {
        let Self::I32(value) = self else {
            self.type_panic("I32");
        };
        *value
    }

    pub fn from_u32(value: u32) -> Self {
        Self::U32(value)
    }

    pub fn get_u32(&self) -> u32 {
        let Self::U32(value) = self else {
            self.type_panic("U32");
        };
        *value
    }

    pub fn from_i64(value: i64) -> Self {
        Self::I64(value)
    }

    pub fn get_i64(&self) -> i64 {
        let Self::I64(value) = self else {
            self.type_panic("I64");
        };
        *value
    }

    pub fn from_u64(value: u64) -> Self {
        Self::U64(value)
    }

    pub fn get_u64(&self) -> u64 {
        let Self::U64(value) = self else {
            self.type_panic("U64");
        };
        *value
    }

    pub fn from_f32(value: f32) -> Self {
        Self::F32(value)
    }

    pub fn get_f32(&self) -> f32 {
        let Self::F32(value) = self else {
            self.type_panic("F32");
        };
        *value
    }

    pub fn from_f64(value: f64) -> Self {
        Self::F64(value)
    }

    pub fn get_f64(&self) -> f64 {
        let Self::F64(value) = self else {
            self.type_panic("F64");
        };
        *value
    }

    pub fn from_string(value: &str) -> Self {
        Self::String(value.into())
    }

    pub fn get_string(&self) -> &str {
        let Self::String(value) = self else {
            self.type_panic("String");
        };
        value.as_str()
    }

    pub fn from_table(value: EventPayloadTable) -> Self {
        Self::Table(Box::new(value))
    }

    pub fn get_table(&self) -> &EventPayloadTable {
        let Self::Table(value) = self else {
            self.type_panic("Table");
        };
        value.as_ref()
    }

    pub fn get_type(&self) -> EventPayloadType {
        match self {
            EventPayload::Lua(_) => EventPayloadType::Lua,
            EventPayload::Bool(_) => EventPayloadType::Bool,
            EventPayload::I8(_) => EventPayloadType::I8,
            EventPayload::U8(_) => EventPayloadType::U8,
            EventPayload::I16(_) => EventPayloadType::I16,
            EventPayload::U16(_) => EventPayloadType::U16,
            EventPayload::I32(_) => EventPayloadType::I32,
            EventPayload::U32(_) => EventPayloadType::U32,
            EventPayload::I64(_) => EventPayloadType::I64,
            EventPayload::U64(_) => EventPayloadType::U64,
            EventPayload::F32(_) => EventPayloadType::F32,
            EventPayload::F64(_) => EventPayloadType::F64,
            EventPayload::String(_) => EventPayloadType::String,
            EventPayload::Table(_) => EventPayloadType::Table,
        }
    }
}

impl EventPayload {
    #[inline]
    fn type_panic(&self, expected: &str) -> ! {
        panic!(
            "Cannot get {expected} type from payload of type {:?}",
            self.get_type()
        );
    }
}
