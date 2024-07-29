use proc_macro2::TokenStream;
use quote::quote;

use super::ImplInfo;
use crate::args::ImplAttrArgs;

impl ImplInfo {
    // Entry point for `impl` blocks.
    pub fn generate(&self, attr_args: &ImplAttrArgs) -> TokenStream {
        // Original impl source code (with removed `bind` attributes)
        let source = &self.source;

        // Generate Rust FFI code.
        let ffi_source = self.gen_rust_ffi(attr_args);

        // Generate Lua FFI files if enabled.
        if attr_args.gen_lua_ffi() {
            self.gen_lua_ffi(attr_args);
        }

        quote! {
            #source
            #ffi_source
        }
    }
}
