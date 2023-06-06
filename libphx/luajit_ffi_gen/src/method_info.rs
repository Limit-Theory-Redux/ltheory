use crate::{args::Args, util::as_camel_case};

const RUST_TO_LUA_TYPE_MAP: [(&str, &str); 1] = [("IVec2", "Vec2i")];
const COPY_TYPES: [&str; 3] = ["IVec2", "WindowPos", "WindowMode"];

pub struct MethodInfo {
    pub bind_args: Args,
    pub name: String,
    pub self_param: Option<SelfType>,
    pub params: Vec<ParamInfo>,
    pub ret: Option<TypeInfo>,
}

impl MethodInfo {
    pub fn as_ffi_name(&self) -> String {
        self.bind_args
            .get("name")
            .unwrap_or_else(|| as_camel_case(&self.name))
    }
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

impl ParamInfo {
    pub fn as_ffi_name(&self) -> String {
        let res = as_camel_case(&self.name);

        if let Some(c) = res.get(..1) {
            // First character of the FFI variable should be lowercase
            format!("{}{}", c.to_lowercase(), res.get(1..).unwrap_or(""))
        } else {
            res
        }
    }
}

pub struct TypeInfo {
    pub is_reference: bool,
    pub is_mutable: bool,
    pub variant: TypeVariant,
}

impl TypeInfo {
    pub fn is_self(&self) -> bool {
        if let TypeVariant::Custom(ty) = &self.variant {
            if ty == "Self" {
                return true;
            }
        }

        false
    }

    pub fn is_copyable(ty: &str) -> bool {
        COPY_TYPES.contains(&ty)
    }

    pub fn as_ffi_string(&self) -> String {
        let ffi_ty = self.variant.as_ffi_string();

        let res = if self.variant.is_custom() {
            RUST_TO_LUA_TYPE_MAP
                .iter()
                .find(|(r_ty, _)| *r_ty == ffi_ty)
                .map(|(_, l_ty)| l_ty.to_string())
                .unwrap_or(ffi_ty)
        } else {
            ffi_ty
        };

        if self.is_reference && self.variant != TypeVariant::Str {
            format!("{}*", res)
        } else {
            res
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum TypeVariant {
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
    Str,
    String,
    Custom(String),
}

impl TypeVariant {
    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    pub fn from_str(type_name: &str) -> Option<Self> {
        let res = match type_name {
            "bool" => Self::Bool,
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
            Self::Bool => "bool",
            Self::I8 => "i8",
            Self::U8 => "u8",
            Self::I16 => "i16",
            Self::U16 => "u16",
            Self::I32 => "i32",
            Self::U32 => "u32",
            Self::I64 => "i64",
            Self::U64 => "u64",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::Str => "str",
            Self::String => "String",
            Self::Custom(val) => return val.clone(),
        }
        .into()
    }

    fn as_ffi_string(&self) -> String {
        match self {
            Self::Bool => "bool",
            Self::I8 => "int8",
            Self::U8 => "uint8",
            Self::I16 => "int16",
            Self::U16 => "uint16",
            Self::I32 => "int",
            Self::U32 => "uint32",
            Self::I64 => "int64",
            Self::U64 => "uint64",
            Self::F32 => "float",
            Self::F64 => "double",
            Self::Str => "cstr",
            Self::String => "cstr",
            Self::Custom(val) => return val.clone(),
        }
        .into()
    }
}
