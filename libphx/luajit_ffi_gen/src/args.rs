use std::collections::HashMap;

use proc_macro2::{Ident, Span};
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{LitStr, Token};

pub struct Args {
    pub params: HashMap<String, String>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let params = input.parse_terminated(Arg::parse, Token![,])?;

        Ok(Self {
            params: params
                .into_iter()
                .map(|arg| (arg.name, arg.value))
                .collect(),
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
            let param_name = format!("{}", input.parse::<Ident>()?);

            input.parse::<Token![=]>()?;

            match param_name.as_ref() {
                "name" => {
                    let name = input.parse::<LitStr>()?;
                    Ok(Self {
                        name: "name".into(),
                        value: name.value(),
                    })
                }
                "no_lua_ffi" => {
                    let name = input.parse::<LitStr>()?;
                    Ok(Self {
                        name: "no_lua_ffi".into(),
                        value: name.value(),
                    })
                }
                _ => Err(Error::new(
                    Span::call_site(),
                    "expected a one of the supported parameters: name, no_lua_ffi",
                )),
            }
        } else {
            Err(Error::new(
                Span::call_site(),
                "expected a 'key = value' pairs",
            ))
        }
    }
}
