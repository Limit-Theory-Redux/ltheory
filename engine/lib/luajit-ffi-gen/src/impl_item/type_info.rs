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
    ("BspNodeRel", "BSPNodeRel"),
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
    "PayloadType",
    "TexFormat",
    "TexFilter",
    "TexWrapMode",
    "PixelFormat",
    "DataFormat",
    "Worker",
    "Metric",
    "CubeFace",
    "BlendMode",
    "CullFace",
    "BspNodeRel",
];

#[derive(Debug, PartialEq)]
pub enum TypeRef {
    // A value T
    Value,
    // A reference such as &T
    Reference,
    // A mutable reference such as &mut T.
    MutableReference,
}

impl TypeRef {
    pub fn is_reference(&self) -> bool {
        *self != Self::Value
    }

    pub fn is_mutable(&self) -> bool {
        *self == Self::MutableReference
    }
}

#[derive(Debug, PartialEq)]
pub enum TypeInfo {
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
    // Box<T>
    Box {
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

impl TypeInfo {
    #[cfg(feature = "assert_ffi_input")]
    pub fn is_option(&self) -> bool {
        matches!(self, Self::Option { .. })
    }

    #[cfg(feature = "assert_ffi_input")]
    pub fn is_reference(&self) -> bool {
        match self {
            TypeInfo::Plain { is_ref, .. } => is_ref.is_reference(),
            TypeInfo::Option { is_ref, .. } => is_ref.is_reference(),
            TypeInfo::Box { .. } => false,
            TypeInfo::Slice { is_ref, .. } => is_ref.is_reference(),
            TypeInfo::Array { is_ref, .. } => is_ref.is_reference(),
            TypeInfo::Function { .. } => false,
            TypeInfo::Result { .. } => false,
        }
    }

    pub fn as_ffi(&self, self_name: &str) -> (String, String) {
        match self {
            Self::Plain { is_ref, ty } => match ty {
                TypeVariant::Str | TypeVariant::String => {
                    if *is_ref == TypeRef::MutableReference {
                        ("*mut libc::c_char".into(), "char*".into())
                    } else {
                        ("*const libc::c_char".into(), "cstr".into())
                    }
                }
                _ => {
                    let (rust_ty_name, c_ty_name) = ty.as_ffi(self_name);
                    match is_ref {
                        TypeRef::MutableReference => {
                            (format!("&mut {rust_ty_name}"), format!("{c_ty_name}*"))
                        }
                        TypeRef::Reference => {
                            (format!("&{rust_ty_name}"), format!("{c_ty_name} const*"))
                        }
                        TypeRef::Value if ty.is_copyable(self_name) => (rust_ty_name, c_ty_name),
                        TypeRef::Value => (format!("Box<{rust_ty_name}>"), format!("{c_ty_name}*")),
                    }
                }
            },
            Self::Option { is_ref, inner_ty } => {
                match inner_ty {
                    TypeVariant::Str | TypeVariant::String => {
                        if *is_ref == TypeRef::MutableReference {
                            ("*mut libc::c_char".into(), "char*".into())
                        } else {
                            ("*const libc::c_char".into(), "cstr".into())
                        }
                    }
                    _ => {
                        let (rust_ty_name, c_ty_name) = inner_ty.as_ffi(self_name);
                        // When we return an Option<T> from a Rust function where T is copyable,
                        // we pin the data to a static instance and return a pointer so we can
                        // encode None properly.
                        match is_ref {
                            TypeRef::MutableReference => (
                                format!("Option<&mut {rust_ty_name}>"),
                                format!("{c_ty_name}*"),
                            ),
                            TypeRef::Reference => (
                                format!("Option<&{rust_ty_name}>"),
                                format!("{c_ty_name} const*"),
                            ),
                            TypeRef::Value if inner_ty.is_copyable(self_name) => (
                                format!("Option<&{rust_ty_name}>"),
                                format!("{c_ty_name} const*"),
                            ),
                            TypeRef::Value => (
                                format!("Option<Box<{rust_ty_name}>>"),
                                format!("{c_ty_name}*"),
                            ),
                        }
                    }
                }
            }
            Self::Box { inner_ty } => {
                let (rust_ty_name, c_ty_name) = inner_ty.as_ffi(self_name);
                (format!("Box<{rust_ty_name}>"), format!("{c_ty_name}*"))
            }
            Self::Slice { is_ref, elem_ty }
            | Self::Array {
                is_ref, elem_ty, ..
            } => {
                // Slices and arrays are always pointers to the inner type.
                let (rust_ty_name, c_ty_name) = elem_ty.as_ffi(self_name);
                if *is_ref == TypeRef::MutableReference {
                    (format!("*mut {rust_ty_name}"), format!("{c_ty_name}*"))
                } else {
                    match elem_ty {
                        TypeVariant::Str | TypeVariant::String => {
                            ("*const *const libc::c_char".into(), "cstr*".into())
                        }
                        _ => (
                            format!("*const {rust_ty_name}"),
                            format!("{c_ty_name} const*"),
                        ),
                    }
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
                                args.push(TypeVariant::USize.as_ffi(self_name))
                            }
                            _ => {}
                        }

                        args
                    })
                    .reduce(|acc, next| {
                        (
                            format!("{}, {}", acc.0, next.0),
                            format!("{}, {}", acc.1, next.1),
                        )
                    })
                    .unwrap_or(("".into(), "".into()));

                let ret_ty = ret_ty
                    .as_ref()
                    .map_or(("()".into(), "void".into()), |ret| ret.as_ffi(self_name));

                (
                    format!("extern fn({}) -> {}", args.0, ret_ty.0),
                    format!("{} (*)({})", ret_ty.1, args.1),
                )
            }
            Self::Result { inner } => {
                // Result's just get unwrapped, so return the inner type.
                inner.as_ffi(self_name)
            }
        }
    }

    // This returns the annotation type supported by LLS as per
    // https://luals.github.io/wiki/annotations/#documenting-types
    pub fn get_luals_annotation(&self, self_name: &str) -> String {
        match self {
            Self::Function { args, ret_ty } => {
                let args = args
                    .iter()
                    .enumerate()
                    .map(|(idx, ty)| {
                        format!("arg{}: {}", idx + 1, ty.get_luals_annotation(self_name))
                    })
                    .collect::<Vec<_>>()
                    .join(", ");

                let ret_ty = ret_ty.as_ref().map_or("nil".to_string(), |ret_ty| {
                    ret_ty.get_luals_annotation(self_name)
                });

                format!("fun({args}): {ret_ty}")
            }
            Self::Plain { ty, .. } | Self::Box { inner_ty: ty, .. } => {
                ty.get_luals_annotation(self_name)
            }
            Self::Option { inner_ty: ty, .. } => format!("{}?", ty.get_luals_annotation(self_name)),
            Self::Slice { elem_ty: ty, .. } | Self::Array { elem_ty: ty, .. } => {
                format!("{}[]", ty.get_luals_annotation(self_name))
            }
            Self::Result { inner } => inner.get_luals_annotation(self_name),
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

    pub fn as_ffi(&self, self_name: &str) -> (String, String) {
        match self {
            Self::Bool => ("bool".into(), "bool".into()),
            Self::I8 => ("i8".into(), "int8".into()),
            Self::U8 => ("u8".into(), "uint8".into()),
            Self::I16 => ("i16".into(), "int16".into()),
            Self::U16 => ("u16".into(), "uint16".into()),
            Self::I32 => ("i32".into(), "int".into()),
            Self::U32 => ("u32".into(), "uint32".into()),
            Self::I64 => ("i64".into(), "int64".into()),
            Self::U64 => ("u64".into(), "uint64".into()),
            Self::ISize => ("isize".into(), "int64".into()),
            Self::USize => ("usize".into(), "uint64".into()),
            Self::F32 => ("f32".into(), "float".into()),
            Self::F64 => ("f64".into(), "double".into()),
            Self::Str => ("str".into(), "cstr".into()),
            Self::String => ("String".into(), "cstr".into()),
            Self::Custom(ty_name) => {
                let ty_name = if self.is_self() {
                    self_name
                } else {
                    ty_name.as_str()
                };
                let ffi_ty_name = RUST_TO_LUA_TYPE_MAP
                    .iter()
                    .find(|(r_ty, _)| *r_ty == ty_name)
                    .map(|(_, l_ty)| *l_ty)
                    .unwrap_or(ty_name);
                (ty_name.to_string(), ffi_ty_name.to_string())
            }
        }
    }

    // This returns the annotation type supported by LLS as per
    // https://luals.github.io/wiki/annotations/#documenting-types
    pub fn get_luals_annotation(&self, self_name: &str) -> String {
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
        }
        .into()
    }
}
