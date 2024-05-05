use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::*;

pub fn process(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Properties);

    generate(input)
}

fn generate(input: Properties) -> TokenStream {
    let mut variant_quotes = vec![];
    let mut reg_quotes = vec![];

    for Property {
        docs,
        id,
        name,
        def,
        map_ids,
        ..
    } in input.properties
    {
        let doc1 = format!("Config name: {name}");
        let def_quote = quote! {#def};
        let doc2 = if map_ids.is_empty() {
            format!("Default value: {}", def_quote.to_string())
        } else {
            let map_ids_str = quote! {#(#map_ids),*};

            format!(
                "Default value: {}. Maps to: {}",
                def_quote.to_string(),
                map_ids_str.to_string()
            )
        };
        let variant_ident = format_ident!("{id}");

        variant_quotes.push(quote! {
            #(#[doc = #docs])*
            #[doc = #doc1]
            #[doc = #doc2]
            #variant_ident
        });

        reg_quotes.push(quote! {
            reg(&mut r, #name, #def, HmGuiProperties::#variant_ident, &[#((HmGuiProperties::#map_ids as usize).into(),)*]);
        });
    }

    quote! {
        /// Core properties
        #[luajit_ffi_gen::luajit_ffi(name = "GuiProperties")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum HmGuiProperties {
            #(#variant_quotes),*
        }

        // NOTE: it's not possible to implement Deref because of recursive call
        impl HmGuiProperties {
            pub fn id(&self) -> usize {
                *self as _
            }
        }

        pub fn register_core_properties() -> IndexMap<String, HmGuiPropertyInfo> {
            let mut r = Default::default();

            #(#reg_quotes)*

            r
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
    docs: Vec<String>,
    id: Ident,
    name: String,
    def: Expr,
    map_ids: Vec<Ident>,
}

impl Parse for Property {
    fn parse(input: ParseStream) -> Result<Self> {
        let docs = input
            .call(Attribute::parse_outer)?
            .iter()
            .filter_map(|attr| {
                if attr.path().segments[0].ident.to_string() == "doc" {
                    get_meta_name(&attr.meta)
                } else {
                    None
                }
            })
            .collect();
        let content;
        let _paren_token = parenthesized!(content in input);
        let fields = content.parse_terminated(Expr::parse, Token![,])?;

        if fields.len() < 3 {
            return Err(Error::new(input.span(), "Expected at least 3 parameters"));
        }

        let id = parse_ident(&fields[0])?;
        let Expr::Lit(name_expr) = &fields[1] else {
            return Err(Error::new(fields[1].span(), "Expected string literal"));
        };
        let Lit::Str(name_lit) = &name_expr.lit else {
            return Err(Error::new(name_expr.span(), "Expected string literal"));
        };
        let mut map_ids = vec![];

        for id in fields.iter().skip(3) {
            map_ids.push(parse_ident(id)?);
        }

        Ok(Self {
            docs,
            id,
            name: name_lit.value(),
            def: fields[2].clone(),
            map_ids,
        })
    }
}

fn parse_ident(id: &Expr) -> Result<Ident> {
    let Expr::Path(path_expr) = id else {
        return Err(Error::new(id.span(), "Expected identifier"));
    };

    Ok(path_expr.path.segments[0].ident.clone())
}

fn get_meta_name(meta: &Meta) -> Option<String> {
    let Ok(doc_text) = meta.require_name_value() else {
        return None;
    };

    if let Expr::Lit(ExprLit { lit, .. }) = &doc_text.value {
        if let Lit::Str(lit_str) = lit {
            return Some(format!("{}", lit_str.value().trim()));
        }
    }

    None
}
