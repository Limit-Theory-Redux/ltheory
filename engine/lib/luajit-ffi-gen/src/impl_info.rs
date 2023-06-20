use proc_macro2::TokenStream;

use crate::method_info::MethodInfo;

/// Parsed data of the `impl` block
pub struct ImplInfo {
    /// Original `impl` source code (with removed `bind` attributes.
    pub source: TokenStream,
    /// `Impl` type name
    pub name: String,
    /// `Impl` methods information
    pub methods: Vec<MethodInfo>,
}
