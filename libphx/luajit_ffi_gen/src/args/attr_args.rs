use syn::parse::{Error, Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{Lit, Token};

use super::arg::Arg;

#[derive(Default)]
pub struct AttrArgs {
    name: Option<String>,
    meta: bool,
    managed: bool,
    no_lua_ffi: bool,
}

impl AttrArgs {
    /// If exists returns the name of the module used in C Api and Lua FFI,
    /// otherwise Rust type name is used.
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    /// Generate metatype section in Lua FFI file
    pub fn with_meta(&self) -> bool {
        self.meta
    }

    /// If true then Lua will be responsible for cleaning object memory.
    /// <module-name>_Free C Api function and Lua FFI 'managed' binding will be generated.
    pub fn is_managed(&self) -> bool {
        self.managed
    }

    /// TEST ONLY!
    /// If true then Lua FFI file won't be generated
    pub fn is_no_lua_ffi(&self) -> bool {
        self.no_lua_ffi
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
                "managed" => {
                    if let Lit::Bool(val) = &param.value.lit {
                        res.managed = val.value();
                    } else {
                        return Err(Error::new(
                            param.value.span(),
                            "expected 'managed' attribute parameter as bool literal",
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
                _ => {
                    return Err(Error::new(
                        param.name.span(),
                        // NOTE: do not show no_lua_ffi since it is test only
                        format!("expected attribute parameter value: name, meta, managed"),
                    ));
                }
            }
        }

        Ok(res)
    }
}
