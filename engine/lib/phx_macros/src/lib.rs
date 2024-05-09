mod core_properties;

use proc_macro::TokenStream;

#[proc_macro]
pub fn core_properties(tokens: TokenStream) -> TokenStream {
    core_properties::process(tokens)
}
