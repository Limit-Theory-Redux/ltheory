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
    "CopyableData", // used in luajit-ffi-gen tests.
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
enum TypeRef {
    // A value T
    Value,
    // A reference such as &T
    Reference,
    // A mutable reference such as &mut T.
    MutableReference,
}

#[derive(Debug, PartialEq)]
enum TypeInfo {
    // T, &T, &mut T
    Plain {
        is_ref: TypeRef,
        ty: TypeVariant,
    },
    // Option<T>, Option<&T>, Option<&mut T>
    Option {
        is_ref: TypeRef,
        inner_ty: TypeVariant,
    },
    // Box<T>, &Box<T>, &mut Box<T> - TODO: Do we want to support refs with boxes?
    Box {
        is_ref: TypeRef,
        inner_ty: TypeVariant,
    },
    // &[T], &mut [T]
    Slice {
        is_ref: TypeRef,
        elem_ty: TypeVariant,
    },
    // [T; N], &[T; N], &mut [T; N]
    Array {
        is_ref: TypeRef,
        elem_ty: TypeVariant,
        length: usize,
    },
    // Fn/FnMut/FnOnce(args) -> R
    Function {
        args: Vec<TypeInfo>,
        ret_ty: Option<Box<TypeInfo>>,
    },
    // Result<T>, only valid in the return position
    Result {
        inner: Box<TypeInfo>,
    },
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

    pub fn to_tuple(self) -> (String, String) {
        (self.rust, self.c)
    }
}

impl TypeInfo {
    pub fn as_ffi(&self, self_name: &str) -> FFIType {
        match self {
            Self::Plain { is_ref, ty } => match ty {
                TypeVariant::Str | TypeVariant::String => {
                    if *is_ref == TypeRef::MutableReference {
                        FFIType::new("*mut libc::c_char", "char*")
                    } else {
                        FFIType::new("*const libc::c_char", "cstr")
                    }
                }
                _ => {
                    let (rust_ty_name, c_ty_name) = ty.as_ffi().to_tuple();
                    match is_ref {
                        TypeRef::MutableReference => {
                            FFIType::new(format!("&mut {rust_ty_name}"), format!("{c_ty_name}*"))
                        }
                        TypeRef::Reference => {
                            FFIType::new(format!("&{rust_ty_name}"), format!("{c_ty_name} const*"))
                        }
                        TypeRef::Value if ty.is_copyable(self_name) => {
                            FFIType::new(rust_ty_name, c_ty_name)
                        }
                        TypeRef::Value => {
                            FFIType::new(format!("Box<{rust_ty_name}>"), format!("{c_ty_name}*"))
                        }
                    }
                }
            },
            Self::Option { is_ref, inner_ty } => {
                match inner_ty {
                    TypeVariant::Str | TypeVariant::String => {
                        if *is_ref == TypeRef::MutableReference {
                            FFIType::new("*mut libc::c_char", "char*")
                        } else {
                            FFIType::new("*const libc::c_char", "cstr")
                        }
                    }
                    _ => {
                        let (rust_ty_name, c_ty_name) = inner_ty.as_ffi().to_tuple();
                        // Both Option<T> and Option<&T> is passed by reference as Option<&T>
                        // which gets coerced to T const* by the Rust compiler.
                        //
                        // When we return an Option<T> from a Rust function, we pin the data
                        // to a static instance.
                        match is_ref {
                            TypeRef::MutableReference => FFIType::new(
                                format!("Option<&mut {rust_ty_name}>"),
                                format!("{c_ty_name}*"),
                            ),
                            TypeRef::Reference | TypeRef::Value => FFIType::new(
                                format!("Option<&{rust_ty_name}>"),
                                format!("{c_ty_name} const*"),
                            ),
                        }
                    }
                }
            }
            Self::Box { is_ref, inner_ty } => {
                // TODO: Old code didn't seem to handle references to boxes. Is that even supported?
                let (rust_ty_name, c_ty_name) = inner_ty.as_ffi().to_tuple();
                FFIType::new(format!("Box<{rust_ty_name}>"), format!("{c_ty_name}*"))
            }
            Self::Slice { is_ref, elem_ty }
            | Self::Array {
                is_ref, elem_ty, ..
            } => {
                // Slices and arrays are always pointers to the inner type.
                let (rust_ty_name, c_ty_name) = elem_ty.as_ffi().to_tuple();
                if *is_ref == TypeRef::MutableReference {
                    FFIType::new(format!("*mut {rust_ty_name}"), format!("{c_ty_name}*"))
                } else {
                    FFIType::new(
                        format!("*const {rust_ty_name}"),
                        format!("{c_ty_name} const*"),
                    )
                }
            }
            Self::Function { args, ret_ty } => {
                let self_name = "Self";

                let args = args
                    .iter()
                    .flat_map(|arg| {
                        let mut args = vec![arg.as_ffi(self_name)];

                        match arg {
                            Self::Slice { .. } | Self::Array { .. } => {
                                args.push(TypeVariant::USize.as_ffi())
                            }
                            _ => {}
                        }

                        args
                    })
                    .reduce(|acc, next| {
                        FFIType::new(
                            format!("{}, {}", acc.rust, next.rust),
                            format!("{}, {}", acc.c, next.c),
                        )
                    })
                    .unwrap_or(FFIType::new("", ""));

                let ret_ty = ret_ty
                    .as_ref()
                    .map_or(FFIType::new("()", "void"), |ret| ret.as_ffi(self_name));

                FFIType::new(
                    format!("extern fn({}) -> {}", args.rust, ret_ty.rust),
                    format!("{} (*)({})", ret_ty.c, args.c),
                )
            }
            Self::Result { inner } => {
                // TODO.
                FFIType::new("", "")
            }
        }
    }

    pub fn as_lua_ffi_string(&self, self_name: &str) -> String {
        match self {
            Self::Plain { ty, .. }
            | Self::Option { inner_ty: ty, .. }
            | Self::Box { inner_ty: ty, .. }
            | Self::Slice { elem_ty: ty, .. }
            | Self::Array { elem_ty: ty, .. } => ty.as_lua_ffi_string(self_name),
            Self::Function { .. } => "function".to_string(),
            Self::Result { inner } => inner.as_lua_ffi_string(self_name),
        }
    }
}

#[derive(Debug, PartialEq)]
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
    Custom(String),
}

impl TypeVariant {
    pub fn is_self(&self) -> bool {
        if let TypeVariant::Custom(ty) = self {
            if ty == "Self" {
                return true;
            }
        }

        false
    }

    pub fn is_copyable(&self, self_name: &str) -> bool {
        match self {
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

    pub fn is_string(&self) -> bool {
        matches!(self, Self::Str | Self::String)
    }

    pub fn get_managed_type(&self) -> Option<&str> {
        match self {
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
            Self::Custom(val) => FFIType::new(val.clone(), val),
        }
    }

    pub fn as_lua_ffi_string(&self, self_name: &str) -> String {
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
            Self::Str | Self::String => "string",
            Self::Custom(ty_name) => {
                let ty_name = if self.is_self() { self_name } else { ty_name };
                RUST_TO_LUA_TYPE_MAP
                    .iter()
                    .find(|(r_ty, _)| *r_ty == ty_name)
                    .map(|(_, l_ty)| *l_ty)
                    .unwrap_or(ty_name)
            }
            // Self::Function { .. } => "function",
        }
        .into()
    }
}
