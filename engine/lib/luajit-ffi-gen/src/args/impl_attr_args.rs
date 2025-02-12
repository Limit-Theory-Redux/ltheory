use syn::parse::{Error, Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{Lit, Token};

use super::arg::Arg;

/// Arguments of the `luajit_ffi` attribute.
pub struct ImplAttrArgs {
    name: Option<String>,
    typedef: Option<String>,
    opaque: bool,
    clone: bool,
    lua_ffi: bool,
    gen_dir: Option<String>,
    meta_dir: Option<String>,
}

impl Default for ImplAttrArgs {
    fn default() -> Self {
        Self {
            name: None,
            typedef: None,
            opaque: true,
            clone: false,
            lua_ffi: true,
            gen_dir: None,
            meta_dir: None,
        }
    }
}

impl ImplAttrArgs {
    /// If exists returns the name of the module used in C Api and Lua FFI,
    /// otherwise Rust type name is used.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// If exists returns the type definition in the generated Lua FFI.
    pub fn typedef(&self) -> Option<&str> {
        self.typedef.as_deref()
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

    /// Overrides the Lua ffi_gen directory.
    pub fn gen_dir(&self) -> Option<&str> {
        self.gen_dir.as_deref()
    }

    /// Overrides the Lua ffi_meta directory.
    pub fn meta_dir(&self) -> Option<&str> {
        self.meta_dir.as_deref()
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
                "typedef" => {
                    if let Lit::Str(val) = &param.value.lit {
                        let typedef_str = val.value().trim().to_string();
                        if !typedef_str.is_empty() {
                            res.typedef = Some(typedef_str);
                        }
                    } else {
                        return Err(Error::new(
                            param.value.span(),
                            "expected 'typedef' attribute parameter as string literal",
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
                "gen_dir" => {
                    if let Lit::Str(val) = &param.value.lit {
                        res.gen_dir = Some(val.value());
                    } else {
                        return Err(Error::new(
                            param.value.span(),
                            "expected 'gen_dir' attribute parameter as string literal",
                        ));
                    }
                }
                "meta_dir" => {
                    if let Lit::Str(val) = &param.value.lit {
                        res.meta_dir = Some(val.value());
                    } else {
                        return Err(Error::new(
                            param.value.span(),
                            "expected 'meta_dir' attribute parameter as string literal",
                        ));
                    }
                }
                _ => {
                    return Err(Error::new(
                        param.name.span(),
                        "expected attribute parameter value: name, typedef, opaque, clone, lua_ffi, gen_dir, meta_dir"
                            .to_string(),
                    ));
                }
            }
        }

        Ok(res)
    }
}
