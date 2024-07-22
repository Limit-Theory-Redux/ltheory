use syn::parse::{Error, Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{Lit, Token};

use super::arg::Arg;

/// Arguments of the `luajit_ffi` attribute.
pub struct ImplAttrArgs {
    name: Option<String>,
    opaque: bool,
    clone: bool,
    lua_ffi: bool,
}

impl Default for ImplAttrArgs {
    fn default() -> Self {
        Self {
            name: None,
            opaque: true,
            clone: false,
            lua_ffi: true,
        }
    }
}

impl ImplAttrArgs {
    /// If exists returns the name of the module used in C Api and Lua FFI,
    /// otherwise Rust type name is used.
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    /// If true then typedef is generated for the module.
    pub fn is_opaque(&self) -> bool {
        self.opaque
    }

    /// If true then adds `__call` method to Global Symbol Table section and `clone` method to metatype section.
    pub fn is_clone(&self) -> bool {
        self.clone
    }

    /// Specify if Lua FFI file should be generated or only C API.
    pub fn gen_lua_ffi(&self) -> bool {
        self.lua_ffi
    }
}

impl Parse for ImplAttrArgs {
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
                "opaque" => {
                    if let Lit::Bool(val) = &param.value.lit {
                        res.opaque = val.value();
                    } else {
                        return Err(Error::new(
                            param.value.span(),
                            "expected 'opaque' attribute parameter as bool literal",
                        ));
                    }
                }
                "clone" => {
                    if let Lit::Bool(val) = &param.value.lit {
                        res.clone = val.value();
                    } else {
                        return Err(Error::new(
                            param.value.span(),
                            "expected 'clone' attribute parameter as bool literal",
                        ));
                    }
                }
                "lua_ffi" => {
                    if let Lit::Bool(val) = &param.value.lit {
                        res.lua_ffi = val.value();
                    } else {
                        return Err(Error::new(
                            param.value.span(),
                            "expected 'lua_ffi' attribute parameter as bool literal",
                        ));
                    }
                }
                _ => {
                    return Err(Error::new(
                        param.name.span(),
                        "expected attribute parameter value: name, opaque, clone, lua_ffi"
                            .to_string(),
                    ));
                }
            }
        }

        Ok(res)
    }
}
