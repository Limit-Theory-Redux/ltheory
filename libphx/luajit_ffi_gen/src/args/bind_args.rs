use proc_macro2::Span;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{Lit, Token};

use super::arg::Arg;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BindMethodType {
    Constructor,
    ToString,
}

impl BindMethodType {
    fn try_from(value: String, span: Span) -> std::result::Result<Self, Error> {
        match value.as_str() {
            "constructor" => Ok(Self::Constructor),
            "to_string" => Ok(Self::ToString),
            _ => Err(Error::new(
                span,
                "expected 'type' bind attribute parameter value: constructor, to_string",
            )),
        }
    }
}

#[derive(Default)]
pub struct BindArgs {
    name: Option<String>,
    bind_method_type: Option<BindMethodType>,
}

impl BindArgs {
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    pub fn is_constructor(&self) -> bool {
        let Some(ty) = self.bind_method_type else { return false; };
        ty == BindMethodType::Constructor
    }

    pub fn is_to_string(&self) -> bool {
        let Some(ty) = self.bind_method_type else { return false; };
        ty == BindMethodType::ToString
    }
}

impl Parse for BindArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let params = input.parse_terminated(Arg::parse, Token![,])?;
        let mut res = Self::default();

        for param in params {
            match param.name.as_str() {
                "name" => {
                    if let Lit::Str(val) = &param.value.lit {
                        res.name = Some(val.value());
                    } else {
                        return Err(Error::new(
                            param.value.span(),
                            "expected 'name' bind attribute parameter as string literal",
                        ));
                    }
                }
                "type" => {
                    if let Lit::Str(val) = &param.value.lit {
                        let ty = BindMethodType::try_from(val.value(), param.value.span())?;
                        res.bind_method_type = Some(ty);
                    } else {
                        return Err(Error::new(
                            param.value.span(),
                            "expected 'name' bind attribute parameter as string literal",
                        ));
                    }
                }
                _ => {
                    return Err(Error::new(
                        param.value.span(),
                        format!("expected bind attribute parameter: name, type"),
                    ))
                }
            }
        }

        Ok(res)
    }
}
