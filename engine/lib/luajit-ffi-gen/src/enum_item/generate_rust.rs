use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use super::EnumInfo;
use crate::args::EnumAttrArgs;
use crate::util::camel_to_snake_case;

impl EnumInfo {
    pub fn gen_rust_ffi(&self, repr_type_ident: &Ident, attr_args: &EnumAttrArgs) -> TokenStream {
        let self_ident = format_ident!("{}", self.name);

        let variant_pairs = self.variants.get_info(attr_args.start_index());
        let enum_size_ident = format_ident!("{}_COUNT", camel_to_snake_case(&self.name, true));
        let enum_size = variant_pairs.len();
        let value_items: Vec<_> = variant_pairs
            .iter()
            .map(|(_, name, d)| {
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

        quote! {
            impl #self_ident {
                pub const SIZE: usize = #enum_size;

                pub const fn value(&self) -> #repr_type_ident {
                    match self {
                        #(#value_items)*
                    }
                }
            }

            impl std::fmt::Display for #self_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                  write!(f, "{:?}", self)
                }
            }

            pub const #enum_size_ident: usize = #enum_size;

            #to_string_mangle
            pub extern "C-unwind" fn #to_string_c_ident(this: #self_ident) -> *const libc::c_char {
                let res = this.to_string();

                internal::static_string!(res)
            }
        }
    }
}
