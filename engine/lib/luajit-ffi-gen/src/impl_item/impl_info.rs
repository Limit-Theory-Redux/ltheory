use proc_macro2::TokenStream;

use super::{MethodInfo, TypeInfo};

/// Parsed data of the `impl` block
pub struct ImplInfo {
    /// Impl documentation strings.
    pub doc: Vec<String>,
    /// Original `impl` source code (with removed `bind` attributes.
    pub source: TokenStream,
    /// `impl` type name
    pub name: String,
    /// `impl` methods information
    pub methods: Vec<MethodInfo>,
}

impl ImplInfo {
    // This type is managed if if has any methods which either have a `self` parameter, or any methods return an instance of this.
    pub fn is_managed(&self) -> bool {
        self.methods.iter().any(|method| {
            if method.bind_args.gen_lua_ffi() {
                method.self_param.is_some()
                    || method.ret.as_ref().is_some_and(|ret| match ret {
                        TypeInfo::Plain { ty, .. }
                        | TypeInfo::Option { inner_ty: ty, .. }
                        | TypeInfo::Box { inner_ty: ty, .. } => ty.is_self(),
                        _ => false,
                    })
            } else {
                false
            }
        })
    }
}
