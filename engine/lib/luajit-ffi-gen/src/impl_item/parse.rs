use quote::quote;
use syn::parse::{Error, Parse, Result};
use syn::spanned::Spanned;
use syn::{
    Attribute, FnArg, GenericArgument, ImplItem, ItemImpl, Pat, Path, PathArguments, ReturnType,
    Type,
};

use crate::args::BindArgs;

use super::*;

impl ImplInfo {
    pub fn parse(mut item: ItemImpl, attrs: &[Attribute]) -> Result<Self> {
        let name = get_impl_self_name(&item.self_ty)?;
        let methods = parse_methods(&mut item.items)?;
        let source = quote! { #(#attrs)* #item };

        let impl_info = ImplInfo {
            source,
            name,
            methods,
        };

        Ok(impl_info)
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
    let (name, generics) = get_path_last_name_with_generics(path)?;

    if !generics.is_empty() {
        Err(Error::new(
            path.span(),
            "expected a type name without generic arguments",
        ))
    } else {
        Ok(name)
    }
}

fn get_path_last_name_with_generics(path: &Path) -> Result<(String, Vec<Type>)> {
    let Some(last_seg) = path.segments.last() else {
        return Err(Error::new(
            path.span(),
            "expected a type identifier",
        ));
    };

    let generic_types = if let PathArguments::AngleBracketed(generic_args) = &last_seg.arguments {
        generic_args
            .args
            .iter()
            .filter_map(|arg| {
                if let GenericArgument::Type(ty) = arg {
                    Some(ty.clone())
                } else {
                    None
                }
            })
            .collect()
    } else {
        vec![]
    };

    Ok((format!("{}", last_seg.ident), generic_types))
}

fn parse_methods(items: &mut Vec<ImplItem>) -> Result<Vec<MethodInfo>> {
    let mut methods = vec![];

    for item in items {
        if let ImplItem::Fn(fn_item) = item {
            let (self_param, params) = parse_params(fn_item.sig.inputs.iter())?;

            methods.push(MethodInfo {
                bind_args: get_bind_args(&mut fn_item.attrs)?,
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
/// ```ignore
/// #[bind(name = "lua_function_name")]
/// fn my_cool_method(...) {...}
/// ```
fn get_bind_args(attrs: &mut Vec<Attribute>) -> Result<BindArgs> {
    let mut res = None;

    for (i, attr) in attrs.iter().enumerate() {
        let attr_name = get_path_last_name(attr.meta.path())?;

        if attr_name != "bind" {
            continue;
        }

        let meta_list = attr.meta.require_list()?;
        let args = meta_list.parse_args_with(BindArgs::parse)?;

        res = Some((i, args));

        break;
    }

    if let Some((i, args)) = res {
        // Remove #[bind] attribute so it won't break compilation
        attrs.remove(i);

        Ok(args)
    } else {
        Ok(BindArgs::default())
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
                    return Err(Error::new(
                        receiver.span(),
                        "expected only &self or &mut self",
                    ));
                }

                self_param_info = Some(SelfType {
                    is_mutable: receiver.mutability.is_some(),
                });
            }
            FnArg::Typed(pat_type) => {
                let name = get_arg_name(&pat_type.pat)?;
                let ty = parse_type(&pat_type.ty)?;

                if ty.is_result {
                    return Err(Error::new(
                        pat_type.ty.span(),
                        "result as input parameter is not supported",
                    ));
                }

                if let TypeVariant::Custom(ty_name) = &ty.variant {
                    if !ty.is_reference && !TypeInfo::is_copyable(&ty_name) {
                        return Err(Error::new(
                            pat_type.ty.span(),
                            "by value non-copyable parameters are not supported",
                        ));
                    }
                }

                let param_info = ParamInfo { name, ty };

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
            let (type_name, generics) = get_path_last_name_with_generics(&type_path.path)?;

            if type_name == "Result" {
                if generics.len() != 1 && generics.len() != 2 {
                    return Err(Error::new(
                        type_path.span(),
                        format!(
                            "expected an Result with 1 or 2 generic arguments but was {}",
                            generics.len()
                        ),
                    ));
                }

                let mut type_info = parse_type(&generics[0])?;

                if type_info.is_result {
                    return Err(Error::new(
                        type_path.span(),
                        format!("nested result is not supported"),
                    ));
                }

                type_info.is_result = true;

                return Ok(type_info);
            } else if type_name == "Option" {
                if generics.len() != 1 {
                    return Err(Error::new(
                        type_path.span(),
                        format!(
                            "expected an Option with 1 generic argument but was {}",
                            generics.len()
                        ),
                    ));
                }

                let mut type_info = parse_type(&generics[0])?;

                if type_info.is_option {
                    return Err(Error::new(
                        type_path.span(),
                        format!("nested option is not supported"),
                    ));
                }

                if type_info.is_result {
                    return Err(Error::new(
                        type_path.span(),
                        format!("result nested in option is not supported"),
                    ));
                }

                type_info.is_option = true;

                return Ok(type_info);
            }

            let variant = TypeVariant::from_str(&type_name);
            let res = if let Some(variant) = variant {
                TypeInfo {
                    is_result: false,
                    is_option: false,
                    is_reference: false,
                    is_mutable: false,
                    variant,
                }
            } else {
                TypeInfo {
                    is_result: false,
                    is_option: false,
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

fn parse_ret_ty(ret_ty: &ReturnType) -> Result<Option<TypeInfo>> {
    match ret_ty {
        ReturnType::Default => Ok(None),
        ReturnType::Type(_, ty) => {
            let type_info = parse_type(&ty)?;

            Ok(Some(type_info))
        }
    }
}
