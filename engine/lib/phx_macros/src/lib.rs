mod core_properties;
mod define_properties;

use proc_macro::TokenStream;

#[proc_macro]
pub fn core_properties(tokens: TokenStream) -> TokenStream {
    core_properties::process(tokens)
}

#[proc_macro]
pub fn define_properties(tokens: TokenStream) -> TokenStream {
    define_properties::process(tokens)
}
