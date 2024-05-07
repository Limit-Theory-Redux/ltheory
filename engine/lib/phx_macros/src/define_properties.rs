use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::*;

pub fn process(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Properties);

    generate(input)
}

fn generate(input: Properties) -> TokenStream {
    let mut type_variants = vec![];
    let mut value_variants = vec![];
    let mut variants_to_str = vec![];
    let mut variants_to_ty = vec![];
    let mut variants_from = vec![];
    let mut variants_from_ffi = vec![];

    for Property {
        ty_ident,
        reference,
    } in input.properties
    {
        let ty_name = format!("{ty_ident}");
        let from_fn_name = format_ident!("from_{}", ty_name.to_lowercase());
        let get_fn_name = format_ident!("get_{}", ty_name.to_lowercase());
        let enum_name = format_ident!("{}", ty_name[..1].to_uppercase() + &ty_name[1..]);
        let enum_name_str = format!("{enum_name}");

        type_variants.push(quote! {#enum_name});
        value_variants.push(quote! {#enum_name(#ty_ident)});
        variants_to_str.push(quote! {Self::#enum_name(_) => #enum_name_str});
        variants_to_ty.push(quote! {Self::#enum_name(_) => HmGuiPropertyType::#enum_name});
        variants_from.push(quote! {
            impl From<#ty_ident> for HmGuiPropertyValue {
                fn from(value: #ty_ident) -> Self {
                    Self::#enum_name(value)
                }
            }
        });

        if reference {
            variants_from_ffi.push(quote! {
                pub fn #from_fn_name(value: &#ty_ident) -> Self {
                    value.clone().into()
                }

                pub fn #get_fn_name(&self) -> &#ty_ident {
                    if let Self::#enum_name(value) = self {
                        value
                    } else {
                        panic!("Wrong property type. Requested {} but actual type is {}", #ty_name, self.name());
                    }
                }
            });
        } else {
            variants_from_ffi.push(quote! {
                pub fn #from_fn_name(value: #ty_ident) -> Self {
                    value.into()
                }

                pub fn #get_fn_name(&self) -> #ty_ident {
                    if let Self::#enum_name(value) = self {
                        value.clone()
                    } else {
                        panic!("Wrong property type. Requested {} but actual type is {}", #ty_name, self.name());
                    }
                }
            });
        }
    }

    quote! {
        #[luajit_ffi_gen::luajit_ffi(name = "GuiPropertyType")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum HmGuiPropertyType {
            #(#type_variants,)*
        }

        #[derive(Clone, Debug, PartialEq)]
        pub enum HmGuiPropertyValue {
            #(#value_variants,)*
        }

        impl HmGuiPropertyValue {
            pub fn name(&self) -> &'static str {
                match self {
                    #(#variants_to_str,)*
                }
            }
        }

        #(#variants_from)*

        #[luajit_ffi_gen::luajit_ffi(name = "GuiPropertyValue")]
        impl HmGuiPropertyValue {
            pub fn get_type(&self) -> HmGuiPropertyType {
                match self {
                    #(#variants_to_ty,)*
                }
            }

            #(#variants_from_ffi)*
        }
    }
    .into()
}

struct Properties {
    properties: Punctuated<Property, Token![,]>,
}

impl Parse for Properties {
    fn parse(input: ParseStream) -> Result<Self> {
        let properties = input.parse_terminated(Property::parse, Token![,])?;

        Ok(Self { properties })
    }
}

struct Property {
    ty_ident: Ident,
    reference: bool,
}

impl Parse for Property {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        let reference = if lookahead.peek(Token![&]) {
            let _amp: Token![&] = input.parse()?;
            true
        } else {
            false
        };
        let ty_ident = input.parse()?;

        Ok(Self {
            ty_ident,
            reference,
        })
    }
}
