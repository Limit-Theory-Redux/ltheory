use proc_macro2::TokenStream;

use super::MethodInfo;

/// Parsed data of the `impl` block
pub struct ImplInfo {
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
                method.self_param.is_some() || method.ret.as_ref().is_some_and(|ret| ret.is_self())
            } else {
                false
            }
        })
    }
}
