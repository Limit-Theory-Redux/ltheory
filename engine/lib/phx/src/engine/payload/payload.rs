use super::{PayloadTable, PayloadType};

#[derive(Debug, Clone, PartialEq)]
pub enum Payload {
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

    BoolArray(Vec<bool>),
    I8Array(Vec<i8>),
    U8Array(Vec<u8>),
    I16Array(Vec<i16>),
    U16Array(Vec<u16>),
    I32Array(Vec<i32>),
    U32Array(Vec<u32>),
    I64Array(Vec<i64>),
    U64Array(Vec<u64>),
    F32Array(Vec<f32>),
    F64Array(Vec<f64>),
    StringArray(Vec<String>),

    Table(Box<PayloadTable>),
}

#[luajit_ffi_gen::luajit_ffi]
impl Payload {
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

    pub fn from_bool_array(value: &[bool]) -> Self {
        Self::BoolArray(value.into())
    }

    pub fn for_each_bool(&self, f: impl Fn(bool)) {
        let Self::BoolArray(value) = self else {
            self.type_panic("BoolArray");
        };

        value.iter().for_each(|v| f(*v));
    }

    pub fn from_i8_array(value: &[i8]) -> Self {
        Self::I8Array(value.into())
    }

    pub fn for_each_i8(&self, f: impl Fn(i8)) {
        let Self::I8Array(value) = self else {
            self.type_panic("I8Array");
        };

        value.iter().for_each(|v| f(*v));
    }

    pub fn from_u8_array(value: &[u8]) -> Self {
        Self::U8Array(value.into())
    }

    pub fn for_each_u8(&self, f: impl Fn(u8)) {
        let Self::U8Array(value) = self else {
            self.type_panic("U8Array");
        };

        value.iter().for_each(|v| f(*v));
    }

    pub fn from_i16_array(value: &[i16]) -> Self {
        Self::I16Array(value.into())
    }

    pub fn for_each_i16(&self, f: impl Fn(i16)) {
        let Self::I16Array(value) = self else {
            self.type_panic("I16Array");
        };

        value.iter().for_each(|v| f(*v));
    }

    pub fn from_u16_array(value: &[u16]) -> Self {
        Self::U16Array(value.into())
    }

    pub fn for_each_u16(&self, f: impl Fn(u16)) {
        let Self::U16Array(value) = self else {
            self.type_panic("U16Array");
        };

        value.iter().for_each(|v| f(*v));
    }

    pub fn from_i32_array(value: &[i32]) -> Self {
        Self::I32Array(value.into())
    }

    pub fn for_each_i32(&self, f: impl Fn(i32)) {
        let Self::I32Array(value) = self else {
            self.type_panic("I32Array");
        };

        value.iter().for_each(|v| f(*v));
    }

    pub fn from_u32_array(value: &[u32]) -> Self {
        Self::U32Array(value.into())
    }

    pub fn for_each_u32(&self, f: impl Fn(u32)) {
        let Self::U32Array(value) = self else {
            self.type_panic("U32Array");
        };

        value.iter().for_each(|v| f(*v));
    }

    pub fn from_i64_array(value: &[i64]) -> Self {
        Self::I64Array(value.into())
    }

    pub fn for_each_i64(&self, f: impl Fn(i64)) {
        let Self::I64Array(value) = self else {
            self.type_panic("I64Array");
        };

        value.iter().for_each(|v| f(*v));
    }

    pub fn from_u64_array(value: &[u64]) -> Self {
        Self::U64Array(value.into())
    }

    pub fn for_each_u64(&self, f: impl Fn(u64)) {
        let Self::U64Array(value) = self else {
            self.type_panic("U64Array");
        };

        value.iter().for_each(|v| f(*v));
    }

    pub fn from_f32_array(value: &[f32]) -> Self {
        Self::F32Array(value.into())
    }

    pub fn for_each_f32(&self, f: impl Fn(f32)) {
        let Self::F32Array(value) = self else {
            self.type_panic("F32Array");
        };

        value.iter().for_each(|v| f(*v));
    }

    pub fn from_f64_array(value: &[f64]) -> Self {
        Self::F64Array(value.into())
    }

    pub fn for_each_f64(&self, f: impl Fn(f64)) {
        let Self::F64Array(value) = self else {
            self.type_panic("F64Array");
        };

        value.iter().for_each(|v| f(*v));
    }

    pub fn from_string_array(value: &[&str]) -> Self {
        Self::StringArray(value.iter().map(|v| v.to_string()).collect())
    }

    pub fn for_each_string(&self, f: impl Fn(&str)) {
        let Self::StringArray(value) = self else {
            self.type_panic("StringArray");
        };

        value.iter().for_each(|v| f(v));
    }

    pub fn from_table(value: PayloadTable) -> Self {
        Self::Table(Box::new(value))
    }

    pub fn get_table(&self) -> &PayloadTable {
        let Self::Table(value) = self else {
            self.type_panic("Table");
        };
        value.as_ref()
    }

    pub fn get_type(&self) -> PayloadType {
        match self {
            Self::Lua(_) => PayloadType::Lua,
            Self::Bool(_) => PayloadType::Bool,
            Self::I8(_) => PayloadType::I8,
            Self::U8(_) => PayloadType::U8,
            Self::I16(_) => PayloadType::I16,
            Self::U16(_) => PayloadType::U16,
            Self::I32(_) => PayloadType::I32,
            Self::U32(_) => PayloadType::U32,
            Self::I64(_) => PayloadType::I64,
            Self::U64(_) => PayloadType::U64,
            Self::F32(_) => PayloadType::F32,
            Self::F64(_) => PayloadType::F64,
            Self::String(_) => PayloadType::String,
            Self::BoolArray(_) => PayloadType::BoolArray,
            Self::I8Array(_) => PayloadType::I8Array,
            Self::U8Array(_) => PayloadType::U8Array,
            Self::I16Array(_) => PayloadType::I16Array,
            Self::U16Array(_) => PayloadType::U16Array,
            Self::I32Array(_) => PayloadType::I32Array,
            Self::U32Array(_) => PayloadType::U32Array,
            Self::I64Array(_) => PayloadType::I64Array,
            Self::U64Array(_) => PayloadType::U64Array,
            Self::F32Array(_) => PayloadType::F32Array,
            Self::F64Array(_) => PayloadType::F64Array,
            Self::StringArray(_) => PayloadType::StringArray,
            Self::Table(_) => PayloadType::Table,
        }
    }
}

impl Payload {
    #[inline]
    fn type_panic(&self, expected: &str) -> ! {
        panic!(
            "Cannot get {expected} type from payload of type {:?}",
            self.get_type()
        );
    }
}
