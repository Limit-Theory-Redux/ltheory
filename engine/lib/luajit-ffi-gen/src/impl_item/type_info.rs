const RUST_TO_LUA_TYPE_MAP: &[(&str, &str)] = &[
    ("IVec2", "Vec2i"),
    ("IVec3", "Vec3i"),
    ("IVec4", "Vec4i"),
    ("UVec2", "Vec2u"),
    ("UVec3", "Vec3u"),
    ("UVec4", "Vec4u"),
    ("DVec2", "Vec2d"),
    ("DVec3", "Vec3d"),
    ("DVec4", "Vec4d"),
    ("Vec2", "Vec2f"),
    ("Vec3", "Vec3f"),
    ("Vec4", "Vec4f"),
    ("Box3", "Box3f"),
    ("HmGuiPropertyType", "GuiPropertyType"),
    ("HmGuiPropertyValue", "GuiPropertyValue"),
    ("LayoutType", "GuiLayoutType"),
];

// TODO: find out different way to mark types as copyable
const COPY_TYPES: &[&str] = &[
    "IVec2",
    "UVec2",
    "DVec2",
    "Vec2",
    "IVec3",
    "UVec3",
    "DVec3",
    "Vec3",
    "IVec4",
    "UVec4",
    "DVec4",
    "Vec4",
    "Box3",
    "Position",
    "WindowPos",
    "WindowMode",
    "MouseControl",
    "KeyboardButton",
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
    "PresentMode",
    "CursorIcon",
    "CursorGrabMode",
    "FocusType",
    "AlignHorizontal",
    "AlignVertical",
    "ResourceType",
    "ScrollDirection",
    "HmGuiPropertyType",
    "LayoutType",
    "HmGuiStyleId",
    "TextAlignment",
    "FrameStage",
    "EventPayloadType",
];

#[derive(Debug, PartialEq)]
pub enum TypeWrapper {
    /// Base type T, no wrapper.
    None,
    /// Option type: Option<T>, Option<&T>, Option<&mut T>
    Option,
    /// Boxed type: Box<T>
    Box,
    /// Slice type: &[T], &mut [T]
    Slice,
    /// Array type: [T; N], &[T; N], &mut [T; N]
    Array(usize),
}

#[derive(Debug)]
pub struct TypeInfo {
    /// Reference type: &T
    pub is_reference: bool,
    /// Mutable reference type: &mut T
    pub is_mutable: bool,
    /// Result type. Can be used only in the return position
    pub is_result: bool,

    pub wrapper: TypeWrapper,
    pub variant: TypeVariant,
}

#[derive(Debug)]
pub struct FFIType {
    /// Rust FFI type i.e. &mut T
    pub rust: String,
    /// C FFI type i.e. T*
    pub c: String,
}

impl FFIType {
    pub fn new<R: Into<String>, C: Into<String>>(rust: R, c: C) -> Self {
        Self {
            rust: rust.into(),
            c: c.into(),
        }
    }
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

    pub fn is_copyable(&self, self_name: &str) -> bool {
        match &self.variant {
            TypeVariant::Custom(ty_name) => {
                let ty_name = if ty_name == "Self" {
                    self_name
                } else {
                    ty_name
                };
                COPY_TYPES.contains(&ty_name)
            }
            _ => true,
        }
    }

    pub fn get_managed_type(&self) -> Option<&str> {
        match &self.variant {
            TypeVariant::Custom(ty_name) => {
                if !COPY_TYPES.contains(&ty_name.as_str()) {
                    Some(ty_name.as_str())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn as_ffi(&self, self_name: &str) -> FFIType {
        match &self.variant {
            TypeVariant::Str | TypeVariant::String | TypeVariant::CString => {
                if self.is_mutable {
                    FFIType::new("*mut libc::c_char", "char*")
                } else {
                    FFIType::new("*const libc::c_char", "cstr")
                }
            }
            TypeVariant::Custom(ty_name) => {
                let ty_name = if self.is_self() { self_name } else { ty_name };

                let ffi_ty_name = RUST_TO_LUA_TYPE_MAP
                    .iter()
                    .find(|(r_ty, _)| *r_ty == ty_name)
                    .map(|(_, l_ty)| l_ty.to_string())
                    .unwrap_or(ty_name.to_string());

                match self.wrapper {
                    TypeWrapper::Slice | TypeWrapper::Array(_) => {
                        // Options, slices and arrays are always pointers to the struct.
                        if self.is_mutable {
                            FFIType::new(format!("*mut {ty_name}"), format!("{ffi_ty_name}*"))
                        } else {
                            FFIType::new(
                                format!("*const {ty_name}"),
                                format!("{ffi_ty_name} const*"),
                            )
                        }
                    }
                    TypeWrapper::Option => {
                        if self.is_mutable {
                            FFIType::new(
                                format!("Option<&mut {ty_name}>"),
                                format!("{ffi_ty_name}*"),
                            )
                        } else {
                            // Both Option<T> and Option<&T> is passed by reference as Option<&T>
                            // which gets coerced to T const* by the Rust compiler.
                            //
                            // When we return an Option<T> from a Rust function, we pin the data
                            // to a static instance.
                            FFIType::new(
                                format!("Option<&{ty_name}>"),
                                format!("{ffi_ty_name} const*"),
                            )
                        }
                    }
                    _ => {
                        if self.is_mutable {
                            // Mutable is always with reference
                            FFIType::new(format!("&mut {ty_name}"), format!("{ffi_ty_name}*"))
                        } else if self.is_reference {
                            FFIType::new(format!("&{ty_name}"), format!("{ffi_ty_name} const*"))
                        } else if self.is_copyable(self_name) {
                            FFIType::new(ty_name, ffi_ty_name)
                        } else {
                            FFIType::new(format!("Box<{ty_name}>"), format!("{ffi_ty_name}*"))
                        }
                    }
                }
            }
            _ => {
                let ffy_ty_name = self.variant.as_ffi();
                let rust_ty_name = ffy_ty_name.rust;
                let c_ty_name = ffy_ty_name.c;

                match self.wrapper {
                    TypeWrapper::Slice | TypeWrapper::Array(_) => {
                        // Options and slices are always pointers to the primitive type.
                        if self.is_mutable {
                            FFIType::new(format!("*mut {rust_ty_name}"), format!("{c_ty_name}*"))
                        } else {
                            FFIType::new(
                                format!("*const {rust_ty_name}"),
                                format!("{c_ty_name} const*"),
                            )
                        }
                    }
                    TypeWrapper::Option => {
                        if self.is_mutable {
                            FFIType::new(
                                format!("Option<&mut {rust_ty_name}>"),
                                format!("{c_ty_name}*"),
                            )
                        } else {
                            // Both Option<T> and Option<&T> is passed by reference as Option<&T>
                            // which gets coerced to T const* by the Rust compiler.
                            //
                            // When we return an Option<T> from a Rust function, we pin the data
                            // to a static instance.
                            FFIType::new(
                                format!("Option<&{rust_ty_name}>"),
                                format!("{c_ty_name} const*"),
                            )
                        }
                    }
                    _ => {
                        if self.is_mutable {
                            // Mutable is always with reference
                            FFIType::new(format!("&mut {rust_ty_name}"), format!("{c_ty_name}*"))
                        } else {
                            // We don't care if there is reference on the numeric type - just accept it by value
                            FFIType::new(rust_ty_name, c_ty_name)
                        }
                    }
                }
            }
        }
    }

    pub fn as_lua_ffi_string(&self, self_name: &str) -> String {
        if let TypeVariant::Custom(ty_name) = &self.variant {
            let ty_ident = if self.is_self() { self_name } else { ty_name };

            RUST_TO_LUA_TYPE_MAP
                .iter()
                .find(|(r_ty, _)| *r_ty == ty_name)
                .map(|(_, l_ty)| l_ty.to_string())
                .unwrap_or(ty_ident.to_string())
        } else {
            self.variant.as_lua_ffi_string()
        }
    }
}

#[derive(Debug)]
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
    pub fn is_string(&self) -> bool {
        matches!(self, Self::Str | Self::String | Self::CString)
    }

    pub fn from_rust_ffi_str(type_name: &str) -> Option<Self> {
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

    pub fn as_ffi(&self) -> FFIType {
        match self {
            Self::Bool => FFIType::new("bool", "bool"),
            Self::I8 => FFIType::new("i8", "int8"),
            Self::U8 => FFIType::new("u8", "uint8"),
            Self::I16 => FFIType::new("i16", "int16"),
            Self::U16 => FFIType::new("u16", "uint16"),
            Self::I32 => FFIType::new("i32", "int"),
            Self::U32 => FFIType::new("u32", "uint32"),
            Self::I64 => FFIType::new("i64", "int64"),
            Self::U64 => FFIType::new("u64", "uint64"),
            Self::ISize => FFIType::new("isize", "int64"),
            Self::USize => FFIType::new("usize", "uint64"),
            Self::F32 => FFIType::new("f32", "float"),
            Self::F64 => FFIType::new("f64", "double"),
            Self::Str => FFIType::new("str", "cstr"),
            Self::String => FFIType::new("String", "cstr"),
            Self::CString => FFIType::new("CString", "cstr"),
            Self::Custom(val) => FFIType::new(val.clone(), val),
        }
    }

    pub fn as_lua_ffi_string(&self) -> String {
        match self {
            Self::Bool => "boolean",
            Self::I8
            | Self::U8
            | Self::I16
            | Self::U16
            | Self::I32
            | Self::U32
            | Self::I64
            | Self::U64
            | Self::ISize
            | Self::USize => "integer",
            Self::F32 | Self::F64 => "number",
            Self::Str | Self::String | Self::CString => "string",
            Self::Custom(val) => return val.clone(),
        }
        .into()
    }
}
