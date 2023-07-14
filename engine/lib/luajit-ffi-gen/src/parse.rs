use syn::parse::{Parse, ParseStream, Result};
use syn::{Attribute, Error, Token};

use crate::enum_item::EnumInfo;
use crate::impl_item::ImplInfo;

/// Information about parsed target: impl block
pub enum Item {
    Impl(ImplInfo),
    Enum(EnumInfo),
}

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        if input.peek(Token![impl]) {
            Ok(Item::Impl(ImplInfo::parse(input.parse()?, &attrs)?))
        } else if input.peek(Token![enum]) || input.peek2(Token![enum]) {
            Ok(Item::Enum(EnumInfo::parse(input.parse()?, &attrs)?))
        } else {
            Err(Error::new(input.span(), "expected impl or enum"))
        }
    }
}
