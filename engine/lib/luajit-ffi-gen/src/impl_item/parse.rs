use quote::quote;
use syn::parse::{Error, Parse, Result};
use syn::spanned::Spanned;
use syn::{Attribute, FnArg, ImplItem, ItemImpl, Pat, ReturnType, Type};

use super::*;
use crate::args::BindArgs;
use crate::util::{
    get_meta_name, get_path_last_name, get_path_last_name_with_generics, parse_doc_attrs,
};

impl ImplInfo {
    pub fn parse(mut item: ItemImpl, attrs: &[Attribute]) -> Result<Self> {
        let doc = parse_doc_attrs(attrs)?;
        let name = get_impl_self_name(&item.self_ty)?;
        let methods = parse_methods(&mut item.items)?;
        let source = quote! { #(#attrs)* #item };

        let impl_info = ImplInfo {
            doc,
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

fn parse_methods(items: &mut Vec<ImplItem>) -> Result<Vec<MethodInfo>> {
    let mut methods = vec![];

    for item in items {
        if let ImplItem::Fn(fn_item) = item {
            let (self_param, params) = parse_params(fn_item.sig.inputs.iter())?;
            let (doc, bind_args) = parse_method_attrs(&mut fn_item.attrs)?;

            methods.push(MethodInfo {
                doc,
                bind_args,
                name: format!("{}", fn_item.sig.ident),
                self_param,
                params,
                ret: parse_ret_ty(&fn_item.sig.output)?,
            });
        }
    }

    Ok(methods)
}

/// Parse the document and bind attributes.
///
/// Expected format:
/// ```ignore
/// /// My cool method
/// #[bind(name = "lua_function_name")]
/// fn my_cool_method(...) {...}
/// ```
fn parse_method_attrs(attrs: &mut Vec<Attribute>) -> Result<(Vec<String>, BindArgs)> {
    let mut res = None;
    let mut doc = vec![];

    for (i, attr) in attrs.iter().enumerate() {
        let attr_name = get_path_last_name(attr.meta.path())?;

        match attr_name.as_str() {
            "bind" => {
                if res.is_some() {
                    return Err(Error::new(
                        attr.span(),
                        "multiple 'bind' attributes are not supported",
                    ));
                }

                let meta_list = attr.meta.require_list()?;
                let args = meta_list.parse_args_with(BindArgs::parse)?;

                res = Some((i, args));
            }
            "doc" => {
                if let Some(doc_text) = get_meta_name(&attr.meta) {
                    doc.push(doc_text);
                }
            }
            _ => {}
        }
    }

    if let Some((i, args)) = res {
        // Remove #[bind] attribute so it won't break compilation
        attrs.remove(i);

        Ok((doc, args))
    } else {
        Ok((doc, BindArgs::default()))
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
                        "Result<T> as an input parameter is not supported",
                    ));
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
                        "nested Result is not supported".to_string(),
                    ));
                }

                type_info.is_result = true;

                return Ok(type_info);
            } else if type_name == "Option" || type_name == "Box" {
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

                if type_info.wrapper != TypeWrapper::None {
                    return Err(Error::new(
                        type_path.span(),
                        "an Option or Box can only contain a bare type",
                    ));
                }

                if type_name == "Option" {
                    type_info.wrapper = TypeWrapper::Option;
                } else if type_name == "Box" {
                    type_info.wrapper = TypeWrapper::Box;
                } else {
                    return Err(Error::new(
                        type_path.span(),
                        format!("unknown type wrapper {}", type_name),
                    ));
                }

                return Ok(type_info);
            }

            let variant = TypeVariant::from_str(&type_name);
            let res = if let Some(variant) = variant {
                TypeInfo {
                    wrapper: TypeWrapper::None,
                    is_reference: false,
                    is_mutable: false,
                    is_result: false,
                    variant,
                }
            } else {
                TypeInfo {
                    wrapper: TypeWrapper::None,
                    is_reference: false,
                    is_mutable: false,
                    is_result: false,
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
        Type::Slice(type_slice) => {
            let mut type_info = parse_type(&type_slice.elem)?;

            if type_info.wrapper != TypeWrapper::None {
                return Err(Error::new(
                    ty.span(),
                    "a slice can only contain a bare type",
                ));
            }

            type_info.wrapper = TypeWrapper::Slice;

            Ok(type_info)
        }
        _ => Err(Error::new(
            ty.span(),
            format!(
                "expected a type, reference to type or mutable reference to type, got {:?}",
                ty
            ),
        )),
    }
}

fn parse_ret_ty(ret_ty: &ReturnType) -> Result<Option<TypeInfo>> {
    match ret_ty {
        ReturnType::Default => Ok(None),
        ReturnType::Type(_, ty) => {
            let type_info = parse_type(ty)?;

            if type_info.wrapper == TypeWrapper::Slice {
                return Err(Error::new(ty.span(), "returning a slice is not supported"));
            }

            Ok(Some(type_info))
        }
    }
}
