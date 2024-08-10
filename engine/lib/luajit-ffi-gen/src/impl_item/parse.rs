use std::collections::HashMap;

use quote::quote;
use syn::parse::{Error, Parse, Result};
use syn::spanned::Spanned;
use syn::{
    Attribute, Expr, FnArg, ImplItem, ItemImpl, Lit, Pat, PathArguments, ReturnType, Type,
    TypeParam, TypeParamBound,
};

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
            let (self_param, params) = parse_params(
                fn_item.sig.inputs.iter(),
                fn_item.sig.generics.type_params(),
            )?;
            let (doc, bind_args) = parse_method_attrs(&mut fn_item.attrs)?;

            methods.push(MethodInfo {
                doc,
                bind_args,
                name: format!("{}", fn_item.sig.ident),
                self_param,
                params,
                ret: parse_ret_type(&fn_item.sig.output)?,
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
    generics: impl Iterator<Item = &'a TypeParam>,
) -> Result<(Option<SelfType>, Vec<ParamInfo>)> {
    let mut self_param_info = None;
    let mut params_info = vec![];

    let mut generic_types: HashMap<String, Vec<TypeParamBound>> = HashMap::new();
    for param in generics {
        generic_types.insert(
            param.ident.to_string(),
            param.bounds.iter().cloned().collect(),
        );
    }

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
                let ty = parse_type(&pat_type.ty, &generic_types)?;

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

fn parse_type(ty: &Type, generic_types: &HashMap<String, Vec<TypeParamBound>>) -> Result<TypeInfo> {
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

                let mut type_info = parse_type(&generics[0], generic_types)?;

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

                let mut type_info = parse_type(&generics[0], generic_types)?;

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
            } else if let Some(type_params) = generic_types.get(&type_name) {
                if type_params.len() == 1 {
                    if let TypeParamBound::Trait(param_trait) = &type_params[0] {
                        if let Some(last_segment) = param_trait.path.segments.last() {
                            if last_segment.ident == "Fn"
                                || last_segment.ident == "FnOnce"
                                || last_segment.ident == "FnMut"
                            {
                                if let PathArguments::Parenthesized(p) = &last_segment.arguments {
                                    let mut args = vec![];
                                    for input in &p.inputs {
                                        args.push(parse_type(input, generic_types)?);
                                    }

                                    let ret = parse_ret_type(&p.output)?.map(Box::new);

                                    return Ok(TypeInfo {
                                        is_reference: false,
                                        is_mutable: false,
                                        is_result: false,
                                        wrapper: TypeWrapper::None,
                                        variant: TypeVariant::Function { args, ret },
                                    });
                                }
                            }
                        }
                    }
                }
            }

            let variant = TypeVariant::from_rust_ffi_str(&type_name);
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
            let mut type_info = parse_type(&type_ref.elem, generic_types)?;

            type_info.is_reference = true;
            type_info.is_mutable = type_ref.mutability.is_some();

            Ok(type_info)
        }
        Type::Slice(type_slice) => {
            let mut type_info = parse_type(&type_slice.elem, generic_types)?;

            if type_info.wrapper != TypeWrapper::None {
                return Err(Error::new(
                    ty.span(),
                    "a slice can only contain a bare type",
                ));
            }

            type_info.wrapper = TypeWrapper::Slice;

            Ok(type_info)
        }
        Type::Array(type_array) => {
            let mut type_info = parse_type(&type_array.elem, generic_types)?;

            if type_info.wrapper != TypeWrapper::None {
                return Err(Error::new(
                    ty.span(),
                    "an array can only contain a bare type",
                ));
            }

            type_info.wrapper = if let Expr::Lit(lit) = &type_array.len {
                if let Lit::Int(value) = &lit.lit {
                    TypeWrapper::Array(value.base10_parse::<usize>()?)
                } else {
                    return Err(Error::new(
                        ty.span(),
                        "an array length can only be a literal integer",
                    ));
                }
            } else {
                return Err(Error::new(
                    ty.span(),
                    "an array length can only be a literal",
                ));
            };

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

fn parse_ret_type(ret_ty: &ReturnType) -> Result<Option<TypeInfo>> {
    match ret_ty {
        ReturnType::Default => Ok(None),
        ReturnType::Type(_, ty) => {
            // If `ty` is a Type::Tuple { ..., elems: [] }, then this is returning ()
            if let Type::Tuple(tuple) = &**ty {
                if tuple.elems.is_empty() {
                    return Ok(None);
                }
            }

            let type_info = parse_type(ty, &HashMap::new())?;

            if type_info.wrapper == TypeWrapper::Slice {
                return Err(Error::new(ty.span(), "returning a slice is not supported"));
            }

            Ok(Some(type_info))
        }
    }
}
