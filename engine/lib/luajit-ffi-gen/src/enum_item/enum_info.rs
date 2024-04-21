use proc_macro2::TokenStream;

use super::variants_info::VariantsInfo;

/// Parsed data of the `impl` block
pub struct EnumInfo {
    /// Enum type documentation strings
    pub doc: Vec<String>,
    /// Original `impl` source code (with removed `bind` attributes.
    pub source: TokenStream,
    /// `Impl` type name
    pub name: String,
    /// `enum` variants information
    pub variants: VariantsInfo,
}
