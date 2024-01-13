use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{args::EnumAttrArgs, util::camel_to_snake_case};

use super::EnumInfo;

impl EnumInfo {
    /// Generate C API and Lua FFI.
    pub fn generate(&self, attr_args: EnumAttrArgs) -> TokenStream {
        // Original enum source code
        let source = &self.source;

        let self_ident = format_ident!("{}", self.name);
        let start_index = attr_args.start_index();
        let repr_type = if let Some(repr_type) = attr_args.repr() {
            repr_type
        } else {
            let max_discriminant = self.variants.max_discriminant(start_index);

            if max_discriminant > u32::MAX as u64 {
                "u64"
            } else if max_discriminant > u16::MAX as u64 {
                "u32"
            } else if max_discriminant > u8::MAX as u64 {
                "u16"
            } else {
                "u8"
            }
            .into()
        };
        let repr_type_ident = format_ident!("{repr_type}");

        let variant_pairs = self.variants.get_pairs(start_index);
        let constant_items: Vec<_> = variant_pairs
            .iter()
            .map(|(variant_name, _)| {
                let mangle_ident = if let Some(enum_name) = attr_args.name() {
                    let export_name = format!("{enum_name}_{variant_name}");
                    quote!(#[export_name = #export_name])
                } else {
                    quote!(#[no_mangle])
                };
                let const_ident = format_ident!("{}_{variant_name}", self.name);
                let variant_ident = format_ident!("{variant_name}");

                quote! {
                    #mangle_ident
                    pub static #const_ident: #repr_type_ident = #self_ident::#variant_ident.value();
                }
            })
            .collect();
        let enum_size_ident = format_ident!("{}_COUNT", camel_to_snake_case(&self.name, true));
        let enum_size = variant_pairs.len();
        let value_items: Vec<_> = variant_pairs
            .iter()
            .map(|(name, d)| {
                let variant_ident = format_ident!("{name}");

                quote! {
                    Self::#variant_ident => #d as #repr_type_ident,
                }
            })
            .collect();

        let to_string_mangle = if let Some(enum_name) = attr_args.name() {
            let export_name = format!("{enum_name}_ToString");
            quote!(#[export_name = #export_name])
        } else {
            quote!(#[no_mangle])
        };
        let to_string_c_ident = format_ident!("{}_ToString", self.name);

        if attr_args.gen_lua_ffi() {
            self.generate_ffi(&attr_args, &repr_type);
        }

        // TODO: generate repr type binding for Lua
        quote! {
            #[repr(#repr_type_ident)]
            #source

            impl #self_ident {
                pub const SIZE: usize = #enum_size;

                pub const fn value(&self) -> #repr_type_ident {
                    match self {
                        #(#value_items)*
                    }
                }
            }

            impl ToString for #self_ident {
                fn to_string(&self) -> String {
                    format!("{:?}", self)
                }
            }

            #(#constant_items)*

            pub const #enum_size_ident: usize = #enum_size;

            #to_string_mangle
            pub extern "C" fn #to_string_c_ident(this: #self_ident) -> *const libc::c_char {
                let res = this.to_string();

                internal::static_string!(res)
            }
        }
    }
}
