use proc_macro2::Span;
use quote::quote;
use syn::parse::Result;
use syn::spanned::Spanned;
use syn::{Attribute, Error, Expr, ExprLit, Fields, ItemEnum, Lit, Variant};

use crate::util::parse_doc_attrs;

use super::variants_info::VariantsInfo;
use super::*;

impl EnumInfo {
    pub fn parse(item: ItemEnum, attrs: &[Attribute]) -> Result<Self> {
        let doc = parse_doc_attrs(attrs)?;
        let name = item.ident.to_string();
        let variants = parse_variants(item.variants.iter(), item.span())?;
        let source = quote! { #(#attrs)* #item };

        let enum_info = EnumInfo {
            doc,
            source,
            name,
            variants,
        };

        Ok(enum_info)
    }
}

fn parse_variants(
    variants: syn::punctuated::Iter<'_, Variant>,
    enum_span: Span,
) -> Result<VariantsInfo> {
    let mut res = vec![];

    for variant in variants {
        if variant.fields != Fields::Unit {
            return Err(Error::new(
                variant.span(),
                "expected unit enumeration variant",
            ));
        }

        let docs = parse_doc_attrs(&variant.attrs)?;

        let discriminant = if let Some((_, expr)) = &variant.discriminant {
            if let Expr::Lit(ExprLit { lit, .. }) = expr {
                match lit {
                    Lit::Int(i) => Some(i.base10_parse::<u64>()?),
                    _ => return Err(Error::new(lit.span(), "expected integer discriminant")),
                }
            } else {
                return Err(Error::new(expr.span(), "expected literal discriminant"));
            }
        } else {
            None
        };

        let name = variant.ident.to_string();

        res.push((docs, name, discriminant));
    }

    let discriminants: Vec<_> = res.iter().filter_map(|(_, _, index)| *index).collect();

    if discriminants.is_empty() {
        Ok(VariantsInfo::Simple(
            res.into_iter().map(|(doc, name, _)| (doc, name)).collect(),
        ))
    } else if discriminants.len() != res.len() {
        Err(Error::new(
            enum_span,
            "expected enum to has either all variants with the discriminant or all without",
        ))
    } else {
        let values = res
            .into_iter()
            .zip(discriminants)
            .map(|((doc, name, _), d)| (doc, name, d))
            .collect();

        Ok(VariantsInfo::Value(values))
    }
}
