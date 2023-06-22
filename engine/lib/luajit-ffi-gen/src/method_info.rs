use crate::{args::BindArgs, type_info::TypeInfo, util::as_camel_case};

/// `Impl` method information
pub struct MethodInfo {
    /// `#[bind]` attribute arguments
    pub bind_args: BindArgs,
    /// Method name
    pub name: String,
    /// Information about `self` method parameter if present
    pub self_param: Option<SelfType>,
    /// Other method parameters info
    pub params: Vec<ParamInfo>,
    /// Return type info if present
    pub ret: Option<TypeInfo>,
}

impl MethodInfo {
    pub fn as_ffi_name(&self) -> String {
        self.bind_args
            .name()
            .unwrap_or_else(|| as_camel_case(&self.name, true))
    }

    pub fn as_ffi_var(&self) -> String {
        self.bind_args
            .name()
            .map(|name| {
                if let Some(c) = name.get(..1) {
                    // First character of the FFI variable should be lowercase
                    format!("{}{}", c.to_lowercase(), name.get(1..).unwrap_or(""))
                } else {
                    name
                }
            })
            .unwrap_or_else(|| as_camel_case(&self.name, false))
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
        as_camel_case(&self.name, false)
    }
}
