use std::collections::HashMap;

use proc_macro2::{Ident, Span};
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{LitStr, Token};

pub struct Args {
    params: HashMap<String, String>,
    span: Span,
}

impl Args {
    pub fn empty(span: Span) -> Self {
        Self {
            params: HashMap::new(),
            span,
        }
    }

    pub fn get(&self, name: &str) -> Option<String> {
        self.params
            .iter()
            .find(|(param_name, _)| *param_name == name)
            .map(|(_, value)| value.clone())
    }

    pub fn validate(&self, expected_params: &[&str]) -> Result<()> {
        if self
            .params
            .iter()
            .any(|(name, _)| !expected_params.contains(&name.as_str()))
        {
            Err(Error::new(
                self.span,
                format!(
                    "expected a one of the supported parameters: {}",
                    expected_params.join(", ")
                ),
            ))
        } else {
            Ok(())
        }
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let params = input.parse_terminated(Arg::parse, Token![,])?;

        Ok(Self {
            params: params
                .into_iter()
                .map(|arg| (arg.name, arg.value))
                .collect(),
            span: input.span(),
        })
    }
}

pub struct Arg {
    pub name: String,
    pub value: String,
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek2(Token![=]) {
            let name = format!("{}", input.parse::<Ident>()?);

            input.parse::<Token![=]>()?;

            let value = input.parse::<LitStr>()?;

            Ok(Self {
                name,
                value: value.value(),
            })
        } else {
            Err(Error::new(input.span(), "expected a 'key = value' pairs"))
        }
    }
}
