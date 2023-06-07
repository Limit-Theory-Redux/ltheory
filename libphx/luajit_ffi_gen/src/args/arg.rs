use proc_macro2::Ident;
use syn::{
    parse::{Parse, ParseStream},
    Error, ExprLit, Result, Token,
};

/// Parse `key = value` pairs of the attribute arguments.
pub struct Arg {
    pub name: String,
    pub value: ExprLit,
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek2(Token![=]) {
            let name = format!("{}", input.parse::<Ident>()?);

            input.parse::<Token![=]>()?;

            let value = input.parse::<ExprLit>()?;

            Ok(Self { name, value })
        } else {
            Err(Error::new(input.span(), "expected a 'key = value' pair"))
        }
    }
}
