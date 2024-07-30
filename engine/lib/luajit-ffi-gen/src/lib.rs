mod args;
mod enum_item;
mod ffi_generator;
mod impl_item;
mod parse;
mod util;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use crate::args::{EnumAttrArgs, ImplAttrArgs};
use crate::parse::Item;

pub(crate) const IDENT: &str = "    ";

/// Proc macro attribute for generating Lua FFI bindings.
///
/// Arguments for `impl` block:
/// - **name** \[string]: optional object name. If not specified then name is taken from the `impl` definition.
/// - **clone** \[bool]: if true then adds `__call` method to Global Symbol Table section and `clone` method to metatype section. Default: false.
/// - **lua_ffi** \[bool]: specify if Lua FFI file should be generated or only C API. Default: true.
///
/// Arguments for `enum` block:
/// - **name** \[string]: optional object name. If not specified then name is taken from the `impl` definition.
/// - **repr** \[string]: specify what type will be used in `#[repr(...)]` attribute that will be added to the enum definition. If not set then type will be deducted from the maximal discriminant: u8, u16, u32 or u64.
/// - **start_index** \[int]: set starting index for discriminant values. Ignored if enum already has discriminants. Default: 0.
/// - **lua_ffi** \[bool]: specify if Lua FFI file should be generated or only C API. Default: true.
/// - **with_impl** \[bool]: specify if enum has connected implementation block. Default: false.
#[proc_macro_attribute]
pub fn luajit_ffi(attr_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);

    let item = match item {
        Item::Impl(item_info) => {
            let args = parse_macro_input!(attr_args as ImplAttrArgs);

            item_info.generate(&args)
        }
        Item::Enum(enum_info) => {
            let args = parse_macro_input!(attr_args as EnumAttrArgs);

            enum_info.generate(&args)
        }
    };

    quote!(#item).into()
}
