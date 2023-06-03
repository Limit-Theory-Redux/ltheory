pub struct MethodInfo {
    pub bind_args: Option<BindArgs>,
    pub name: String,
    pub self_param: Option<SelfType>,
    pub params: Vec<ParamInfo>,
    pub ret: Option<TypeInfo>,
}

pub struct BindArgs {
    pub name: String,
}

/// Type of the method receiver.
/// Expected only ```&self``` or ```&mut self```
pub struct SelfType {
    pub is_mutable: bool,
}

pub struct ParamInfo {
    pub name: String,
    pub ty: TypeInfo,
}

pub struct TypeInfo {
    pub is_reference: bool,
    pub is_mutable: bool,
    pub variant: TypeVariant,
}

pub enum TypeVariant {
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
    Str,
    String,
    Custom(String),
}

impl TypeVariant {
    pub fn from_str(type_name: &str) -> Option<Self> {
        let res = match type_name {
            "i8" => Self::I8,
            "u8" => Self::U8,
            "i16" => Self::I16,
            "u16" => Self::U16,
            "i32" => Self::I32,
            "u32" => Self::U32,
            "i64" => Self::I64,
            "u64" => Self::U64,
            "f32" => Self::F32,
            "f64" => Self::F64,
            "str" => Self::Str,
            "String" => Self::String,
            _ => return None,
        };

        Some(res)
    }

    pub fn as_string(&self) -> String {
        match self {
            TypeVariant::I8 => "i8",
            TypeVariant::U8 => "u8",
            TypeVariant::I16 => "i16",
            TypeVariant::U16 => "u16",
            TypeVariant::I32 => "i32",
            TypeVariant::U32 => "u32",
            TypeVariant::I64 => "i64",
            TypeVariant::U64 => "u64",
            TypeVariant::F32 => "f32",
            TypeVariant::F64 => "f64",
            TypeVariant::Str => "str",
            TypeVariant::String => "String",
            TypeVariant::Custom(val) => return val.clone(),
        }
        .into()
    }
}
