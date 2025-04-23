use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::EnumInfo;
use crate::args::EnumAttrArgs;

impl EnumInfo {
    /// Entry point for `enum` blocks.
    pub fn generate(&self, attr_args: &EnumAttrArgs) -> TokenStream {
        // Original enum source code
        let source = &self.source;

        // Decide repr type.
        let repr_type = if let Some(repr_type) = attr_args.repr() {
            repr_type
        } else if let Some(max_discriminant) =
            self.variants.max_discriminant(attr_args.start_index())
        {
            if max_discriminant > u32::MAX as u64 {
                "u64"
            } else if max_discriminant > u16::MAX as u64 {
                "u32"
            } else if max_discriminant > u8::MAX as u64 {
                "u16"
            } else {
                "u8"
            }
        } else {
            panic!("If non-numeric variant values are used then type representation should be specified explicitly. Example: #[luajit_ffi(repr = \"u16\")]");
        };
        let repr_type_ident = format_ident!("{repr_type}");

        // Generate Rust FFI code.
        let ffi_source = self.gen_rust_ffi(&repr_type_ident, attr_args);

        // Generate Lua FFI files if enabled.
        if attr_args.gen_lua_ffi() {
            self.gen_lua_ffi(attr_args, repr_type);
        }

        // TODO: generate repr type binding for Lua
        quote! {
            #[repr(#repr_type_ident)]
            #source

            #ffi_source
        }
    }
}
