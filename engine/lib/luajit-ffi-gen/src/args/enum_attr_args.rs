use syn::parse::{Error, Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{Lit, Token};

use super::arg::Arg;

/// Arguments of the `luajit_ffi` attribute.
pub struct EnumAttrArgs {
    name: Option<String>,
    repr: Option<String>, // TODO: extract this value from #[repr(...)] instead
    start_index: Option<u64>,
    lua_ffi: bool,
    with_impl: bool,
}

impl Default for EnumAttrArgs {
    fn default() -> Self {
        Self {
            name: None,
            repr: None,
            start_index: None,
            lua_ffi: true,
            with_impl: false,
        }
    }
}

impl EnumAttrArgs {
    /// If exists returns the name of the module used in C Api and Lua FFI,
    /// otherwise Rust type name is used.
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    /// Specify what type will be used in `#[repr(...)]` attribute that will be added to the enum definition.
    /// If not set then type will be deducted from the maximal discriminant: u8, u16, u32 or u64.
    pub fn repr(&self) -> Option<String> {
        self.repr.clone()
    }

    /// Set starting index for discriminant values. Ignored if enum already has discriminants. Default: 0.
    pub fn start_index(&self) -> u64 {
        self.start_index.unwrap_or(0)
    }

    /// Specify if Lua FFI file should be generated or only C API.
    pub fn gen_lua_ffi(&self) -> bool {
        self.lua_ffi
    }

    /// Specify if enum has connected implementation block.
    /// In this case Lua FFI file won't be generated. Instead config file with all enum data needed for FFI
    /// generation will be stored in the target folder. Impl attribute will read it, combine it with it's own data
    /// and then generate complete FFI file.
    pub fn with_impl(&self) -> bool {
        self.with_impl
    }
}

impl Parse for EnumAttrArgs {
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
                "repr" => {
                    if let Lit::Str(val) = &param.value.lit {
                        res.repr = Some(val.value());
                    } else {
                        return Err(Error::new(
                            param.value.span(),
                            "expected 'repr' attribute parameter as string literal",
                        ));
                    }
                }
                "start_index" => {
                    if let Lit::Int(val) = &param.value.lit {
                        res.start_index = Some(val.base10_parse()?);
                    } else {
                        return Err(Error::new(
                            param.value.span(),
                            "expected 'start_index' attribute parameter as integer literal",
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
                "with_impl" => {
                    if let Lit::Bool(val) = &param.value.lit {
                        res.with_impl = val.value();
                    } else {
                        return Err(Error::new(
                            param.value.span(),
                            "expected 'with_impl' attribute parameter as bool literal",
                        ));
                    }
                }
                _ => {
                    return Err(Error::new(
                        param.name.span(),
                        format!(
                            "expected attribute parameter value: name, repr, start_index, lua_ffi, with_impl"
                        ),
                    ));
                }
            }
        }

        Ok(res)
    }
}
