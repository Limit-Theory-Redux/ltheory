const RUST_TO_LUA_TYPE_MAP: [(&str, &str); 5] = [
    ("IVec2", "Vec2i"),
    ("UVec2", "Vec2u"),
    ("DVec2", "Vec2d"),
    ("Vec2", "Vec2f"),
    ("Vec3", "Vec3f"),
];

// TODO: find out different way to mark types as copyable
const COPY_TYPES: [&str; 19] = [
    "IVec2",
    "UVec2",
    "DVec2",
    "Vec2",
    "WindowPos",
    "WindowMode",
    "MouseControl",
    "KeyboardButton",
    "CursorControl",
    "TouchpadAxis",
    "GamepadId",
    "GamepadButton2",
    "GamepadAxis2",
    "Button",
    "Button2",
    "DeviceType",
    "GamepadButton",
    "GamepadAxis",
    "InputDeviceType",
];

pub struct TypeInfo {
    /// Result type. Can be used only in the return position
    pub is_result: bool,
    /// Option type: Option<T>, Option<&T>, Option<&mut T>
    /// Option<Option<T>> is not supported
    pub is_option: bool,
    /// Reference type: &T
    pub is_reference: bool,
    /// Mutable reference type: &mut T
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
        let opt = if self.is_option && !self.is_reference && !self.variant.is_string() {
            " const*"
        } else {
            ""
        };

        if self.is_reference && self.variant != TypeVariant::Str {
            if self.is_mutable {
                format!("{res}*{opt}")
            } else {
                format!("{res} const*{opt}")
            }
        } else {
            format!("{res}{opt}")
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
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
    ISize,
    USize,
    F32,
    F64,
    Str,
    String,
    CString,
    Custom(String),
}

impl TypeVariant {
    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    #[allow(dead_code)]
    pub fn is_str(&self) -> bool {
        matches!(self, Self::Str)
    }

    pub fn is_string(&self) -> bool {
        match self {
            Self::Str | Self::String | Self::CString => true,
            _ => false,
        }
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
            "isize" => Self::ISize,
            "usize" => Self::USize,
            "f32" => Self::F32,
            "f64" => Self::F64,
            "str" => Self::Str,
            "String" => Self::String,
            "CString" => Self::CString,
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
            Self::ISize => "isize",
            Self::USize => "usize",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::Str => "str",
            Self::String => "String",
            Self::CString => "CString",
            Self::Custom(val) => return val.clone(),
        }
        .into()
    }

    pub fn as_ffi_string(&self) -> String {
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
            Self::ISize => "int64",
            Self::USize => "uint64",
            Self::F32 => "float",
            Self::F64 => "double",
            Self::Str | Self::String | Self::CString => "cstr",
            Self::Custom(val) => return val.clone(),
        }
        .into()
    }
}
