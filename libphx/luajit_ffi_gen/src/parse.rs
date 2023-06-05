use proc_macro2::Span;
use quote::quote;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{Attribute, FnArg, ImplItem, ItemImpl, Pat, Path, ReturnType, Token, Type};

use crate::args::Args;
use crate::impl_info::ImplInfo;
use crate::method_info::{MethodInfo, ParamInfo, SelfType, TypeInfo, TypeVariant};

pub enum Item {
    Impl(ImplInfo),
    // TODO: implement for structs and enums
}

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![impl]) {
            let mut item: ItemImpl = input.parse()?;

            let name = get_impl_self_name(&item.self_ty)?;
            let methods = parse_methods(&mut item.items)?;
            let source = quote! { #(#attrs)* #item };

            let impl_info = ImplInfo {
                source,
                name,
                methods,
            };

            Ok(Item::Impl(impl_info))
        } else {
            Err(lookahead.error())
        }
    }
}

fn get_impl_self_name(ty: &Type) -> Result<String> {
    match ty {
        Type::Path(ty_path) => get_path_last_name(&ty_path.path),
        // TODO: do we really have to support a reference? Example: impl MyTrait for &MyStruct {...}
        Type::Reference(ty_ref) => get_impl_self_name(&ty_ref.elem),
        _ => Err(Error::new(
            ty.span(),
            "expected an impl for type or type reference",
        )),
    }
}

fn get_path_last_name(path: &Path) -> Result<String> {
    let Some(last_seg) = path.segments.last() else {
        return Err(Error::new(
            path.span(),
            "expected a type identifier",
        ));
    };

    Ok(format!("{}", last_seg.ident))
}

fn parse_methods(items: &mut Vec<ImplItem>) -> Result<Vec<MethodInfo>> {
    let mut methods = vec![];

    for item in items {
        if let ImplItem::Fn(fn_item) = item {
            let (self_param, params) = parse_params(fn_item.sig.inputs.iter())?;

            methods.push(MethodInfo {
                bind_args: get_bind_args(fn_item.span(), &mut fn_item.attrs)?,
                name: format!("{}", fn_item.sig.ident),
                self_param,
                params,
                ret: parse_ret_ty(&fn_item.sig.output)?,
            });
        }
    }

    Ok(methods)
}

/// Look for the bind attribute an extract its parameters.
///
/// Expected format:
/// ```
/// #[bind(name = "lua_function_name")]
/// fm my_cool_method(...) {...}
/// ```
fn get_bind_args(span: Span, attrs: &mut Vec<Attribute>) -> Result<Args> {
    let mut res = None;

    for (i, attr) in attrs.iter().enumerate() {
        let attr_name = get_path_last_name(attr.meta.path())?;

        if attr_name != "bind" {
            continue;
        }

        let meta_list = attr.meta.require_list()?;
        let args = meta_list.parse_args_with(Args::parse)?;

        args.validate(&["name"])?;

        res = Some((i, args));

        break;
    }

    if let Some((i, args)) = res {
        // Remove #[bind] attribute so it won't break compilation
        attrs.remove(i);

        Ok(args)
    } else {
        Ok(Args::empty(span))
    }
}

fn parse_params<'a>(
    params: impl Iterator<Item = &'a FnArg>,
) -> Result<(Option<SelfType>, Vec<ParamInfo>)> {
    let mut self_param_info = None;
    let mut params_info = vec![];

    for param in params {
        match param {
            FnArg::Receiver(receiver) => {
                if receiver.reference.is_none() {
                    return Err(Error::new(param.span(), "expected only &self or &mut self"));
                }

                self_param_info = Some(SelfType {
                    is_mutable: receiver.mutability.is_some(),
                });
            }
            FnArg::Typed(pat_type) => {
                let param_info = ParamInfo {
                    name: get_arg_name(&pat_type.pat)?,
                    ty: parse_type(&pat_type.ty)?,
                };

                params_info.push(param_info);
            }
        }
    }

    Ok((self_param_info, params_info))
}

fn get_arg_name(pat: &Pat) -> Result<String> {
    if let Pat::Ident(pat_ident) = pat {
        return Ok(format!("{}", pat_ident.ident));
    }

    Err(Error::new(pat.span(), "expected a method argument name"))
}

fn parse_type(ty: &Type) -> Result<TypeInfo> {
    match ty {
        Type::Path(type_path) => {
            let type_name = get_path_last_name(&type_path.path)?;
            let variant = TypeVariant::from_str(&type_name);
            let res = if let Some(variant) = variant {
                TypeInfo {
                    is_reference: false,
                    is_mutable: false,
                    variant,
                }
            } else {
                TypeInfo {
                    is_reference: false,
                    is_mutable: false,
                    // TODO: are we going to support full path to type? I.e. std::path::PathBuf
                    variant: TypeVariant::Custom(type_name),
                }
            };

            Ok(res)
        }
        Type::Reference(type_ref) => {
            let mut type_info = parse_type(&type_ref.elem)?;

            type_info.is_reference = true;
            type_info.is_mutable = type_ref.mutability.is_some();

            Ok(type_info)
        }
        _ => Err(Error::new(
            ty.span(),
            "expected a type, reference to type or mutable reference to type",
        )),
    }
}

fn parse_ret_ty(ty: &ReturnType) -> Result<Option<TypeInfo>> {
    match ty {
        ReturnType::Default => Ok(None),
        ReturnType::Type(_, ty) => {
            let type_info = parse_type(&ty)?;

            Ok(Some(type_info))
        }
    }
}
