use syn::parse::{Error, Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{Lit, Token};

use super::arg::Arg;

#[derive(Default)]
pub struct AttrArgs {
    name: Option<String>,
    no_lua_ffi: bool,
    meta: bool,
}

impl AttrArgs {
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    pub fn no_lua_ffi(&self) -> bool {
        self.no_lua_ffi
    }

    pub fn meta(&self) -> bool {
        self.meta
    }
}

impl Parse for AttrArgs {
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
                            "expected 'name' attribute parameter as string literal",
                        ));
                    }
                }
                "no_lua_ffi" => {
                    if let Lit::Bool(val) = &param.value.lit {
                        res.no_lua_ffi = val.value();
                    } else {
                        return Err(Error::new(
                            param.value.span(),
                            "expected 'no_lua_ffi' attribute parameter as bool literal",
                        ));
                    }
                }
                "meta" => {
                    if let Lit::Bool(val) = &param.value.lit {
                        res.meta = val.value();
                    } else {
                        return Err(Error::new(
                            param.value.span(),
                            "expected 'meta' attribute parameter as bool literal",
                        ));
                    }
                }
                _ => {
                    return Err(Error::new(
                        param.name.span(),
                        // NOTE: do not show no_lua_ffi since it is test only
                        format!("expected attribute parameter value: name, meta"),
                    ));
                }
            }
        }

        Ok(res)
    }
}
