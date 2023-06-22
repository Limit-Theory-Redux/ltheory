use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::args::AttrArgs;
use crate::lua_ffi::generate_ffi;
use crate::method_info::*;
use crate::parse::*;
use crate::type_info::TypeInfo;
use crate::type_info::TypeVariant;

/// Generate C API and Lua FFI.
pub fn generate(item: Item, attr_args: AttrArgs) -> TokenStream {
    match item {
        Item::Impl(impl_info) => {
            // Original impl source code (with removed `bind` attributes)
            let source = &impl_info.source;
            // C API wrapper functions
            let method_tokens: Vec<_> = impl_info
                .methods
                .iter()
                .map(|method| wrap_methods(&attr_args, &impl_info.name, method))
                .collect();
            // Additional Free C API wrapper if requested
            let free_method_token = if attr_args.is_managed() {
                let module_name = attr_args.name().unwrap_or(impl_info.name.clone());
                let free_method_ident = format_ident!("{module_name}_Free");
                let module_ident = format_ident!("{}", impl_info.name);

                quote! {
                    #[no_mangle]
                    pub extern "C" fn #free_method_ident(_: Box<#module_ident>) {}
                }
            } else {
                quote! {}
            };

            if attr_args.gen_lua_ffi() {
                generate_ffi(&attr_args, &impl_info);
            }

            quote! {
                #source

                #free_method_token
                #(#method_tokens)*
            }
        }
    }
}

fn wrap_methods(attr_args: &AttrArgs, self_name: &str, method: &MethodInfo) -> TokenStream {
    let module_name = attr_args.name().unwrap_or(self_name.into());
    let method_name = method.as_ffi_name();
    let func_name = format!("{self_name}_{}", method_name);
    let func_ident = format_ident!("{func_name}");
    let self_ident = format_ident!("{self_name}");

    let self_token = if let Some(self_param) = &method.self_param {
        if self_param.is_mutable {
            quote! { this: &mut #self_ident, }
        } else {
            quote! { this: &#self_ident, }
        }
    } else {
        quote! {}
    };

    let param_tokens: Vec<_> = method
        .params
        .iter()
        .map(|param| wrap_param(&module_name, param))
        .collect();

    let ret_token = if let Some(ty) = &method.ret {
        let ty_token = wrap_type(&module_name, &ty, true);

        quote! { -> #ty_token }
    } else {
        quote! {}
    };

    let func_ident_str = format!("{func_ident}");
    let func_body = gen_func_body(&self_ident, method);

    quote! {
        #[no_mangle]
        pub extern "C" fn #func_ident(#self_token #(#param_tokens),*) #ret_token {
            tracing::trace!("Calling: {}", #func_ident_str);
            #func_body
        }
    }
}

fn wrap_param(module_name: &str, param: &ParamInfo) -> TokenStream {
    let param_name_ident = format_ident!("{}", param.name);
    let param_type_token = wrap_type(module_name, &param.ty, false);

    quote! { #param_name_ident: #param_type_token }
}

fn wrap_type(module_name: &str, ty: &TypeInfo, ret: bool) -> TokenStream {
    let opt_item = if ty.is_option {
        quote! { *mut }
    } else {
        quote! {}
    };

    match &ty.variant {
        TypeVariant::Str | TypeVariant::String | TypeVariant::CString => {
            quote! { #opt_item *const libc::c_char }
        }
        TypeVariant::Custom(ty_name) => {
            let ty_ident = format_ident!("{ty_name}");

            if ty.is_mutable {
                // Mutable is always with reference
                quote! { #opt_item &mut #ty_ident }
            } else if TypeInfo::is_copyable(&ty_name) && !ty.is_reference {
                quote! { #opt_item #ty_ident }
            } else if ret {
                // We always send unregistered return type boxed
                if ty.is_self() {
                    let ty_ident = format_ident!("{module_name}");

                    quote! { Box<#opt_item #ty_ident> }
                } else {
                    quote! { Box<#opt_item #ty_ident> }
                }
            } else {
                // We always send unregistered type by reference
                quote! { #opt_item &#ty_ident }
            }
        }
        _ => {
            let ty_ident = format_ident!("{}", ty.variant.as_string());

            if ty.is_mutable {
                // Mutable is always with reference
                quote! { #opt_item &mut #ty_ident }
            } else {
                // We don't care if there is reference on the numeric type - just accept it by value
                quote! { #opt_item #ty_ident }
            }
        }
    }
}

fn gen_func_body(self_ident: &Ident, method: &MethodInfo) -> TokenStream {
    let method_ident = format_ident!("{}", method.name);
    let accessor_token = if method.self_param.is_some() {
        quote! { this.#method_ident }
    } else {
        quote! { #self_ident::#method_ident }
    };

    let param_tokens: Vec<_> = method
        .params
        .iter()
        .map(|param| {
            let name_ident = format_ident!("{}", param.name);
            let name_accessor = if param.ty.is_option {
                quote! { (*#name_ident) }
            } else {
                quote! { #name_ident }
            };

            let param_item = match param.ty.variant {
                TypeVariant::Str => quote! { #name_accessor.as_str() },
                TypeVariant::String => quote! { #name_accessor.as_string() },
                TypeVariant::CString => quote! { #name_accessor.as_cstring() },
                TypeVariant::Custom(_) => quote! { #name_accessor },
                _ => {
                    if param.ty.is_mutable {
                        quote! { #name_accessor }
                    } else if param.ty.is_reference {
                        quote! { &#name_accessor }
                    } else {
                        quote! { #name_accessor }
                    }
                }
            };

            if param.ty.is_option {
                quote! {if #name_ident != std::ptr::null_mut() { unsafe { Some(#param_item) } } else { Option::None }}
            } else {
                param_item
            }
        })
        .collect();

    if let Some(ty) = &method.ret {
        let method_call = if ty.is_result {
            let method_call_str = format!("{}::{}", self_ident, method.name);

            quote! {
                match #accessor_token(#(#param_tokens),*) {
                    Ok(res) => res,
                    Err(err) => {
                        panic!("Error on calling method '{}': {}", #method_call_str, err);
                    }
                }
            }
        } else {
            quote! { #accessor_token(#(#param_tokens),*) }
        };

        match &ty.variant {
            TypeVariant::Str | TypeVariant::String => {
                quote! { let res = #method_call; static_string!(res) }
            }
            TypeVariant::CString => quote! { let res = #method_call; static_cstring!(res) },
            TypeVariant::Custom(custom_ty)
                if ty.is_self() || !TypeInfo::is_copyable(&custom_ty) =>
            {
                quote! { let res = #method_call; res.into() }
            }
            _ => quote! { #method_call },
        }
    } else {
        quote! {
            #accessor_token(#(#param_tokens),*);
        }
    }
}
