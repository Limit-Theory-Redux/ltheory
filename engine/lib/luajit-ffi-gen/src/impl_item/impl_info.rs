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
