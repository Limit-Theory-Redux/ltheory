use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::args::AttrArgs;
use crate::lua_ffi::generate_ffi;
use crate::method_info::*;
use crate::parse::*;

pub fn generate(item: Item, attr_args: AttrArgs) -> TokenStream {
    match item {
        Item::Impl(impl_info) => {
            let source = &impl_info.source;
            let method_tokens: Vec<_> = impl_info
                .methods
                .iter()
                .map(|method| wrap_methods(&attr_args, &impl_info.name, method))
                .collect();
            let free_method_token = if attr_args.is_managed() {
                let module_name = attr_args.name().unwrap_or(impl_info.name.clone());
                let free_method_ident = format_ident!("{module_name}_Free");

                quote! {
                    #[no_mangle]
                    pub extern "C" fn #free_method_ident(_: Box<Window>) {}
                }
            } else {
                quote! {}
            };

            if !attr_args.is_no_lua_ffi() {
                generate_ffi(&attr_args, &impl_info);
            }

            // let methods_str = format!("{:#?}", method_tokens);
            // std::fs::write("dump.txt", methods_str.as_bytes()).unwrap();

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

    let func_body = gen_func_body(&self_ident, method);

    quote! {
        #[no_mangle]
        pub extern "C" fn #func_ident(#self_token #(#param_tokens),*) #ret_token {
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
    match &ty.variant {
        TypeVariant::Str | TypeVariant::String => quote! { *const libc::c_char },
        TypeVariant::Custom(ty_name) => {
            let ty_ident = format_ident!("{ty_name}");

            if ty.is_mutable {
                // Mutable is always with reference
                quote! { &mut #ty_ident }
            } else if TypeInfo::is_copyable(&ty_name) && !ty.is_reference {
                quote! { #ty_ident }
            } else if ret {
                // We always send unregistered return type boxed
                if ty.is_self() {
                    let ty_ident = format_ident!("{module_name}");

                    quote! { Box<#ty_ident> }
                } else {
                    quote! { Box<#ty_ident> }
                }
            } else {
                // We always send unregistered type by reference
                quote! { &#ty_ident }
            }
        }
        _ => {
            let ty_ident = format_ident!("{}", ty.variant.as_string());

            if ty.is_mutable {
                // Mutable is always with reference
                quote! { &mut #ty_ident }
            } else {
                // We don't care if there is reference on the numeric type - just accept it by value
                quote! { #ty_ident }
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

            match param.ty.variant {
                TypeVariant::Str => quote! { #name_ident.as_str() },
                // TODO: do we accept &String?
                TypeVariant::String => quote! { #name_ident.as_string() },
                TypeVariant::Custom(_) => quote! { #name_ident },
                _ => {
                    if param.ty.is_mutable {
                        quote! { #name_ident }
                    } else if param.ty.is_reference {
                        quote! { &#name_ident }
                    } else {
                        quote! { #name_ident }
                    }
                }
            }
        })
        .collect();

    if let Some(ty) = &method.ret {
        match &ty.variant {
            TypeVariant::Str | TypeVariant::String => quote! {
                static_string!(#accessor_token(#(#param_tokens),*))
            },
            TypeVariant::Custom(custom_ty)
                if ty.is_self() || !TypeInfo::is_copyable(&custom_ty) =>
            {
                quote! {
                    #accessor_token(#(#param_tokens),*).into()
                }
            }
            _ => quote! {
                #accessor_token(#(#param_tokens),*)
            },
        }
    } else {
        quote! {
            #accessor_token(#(#param_tokens),*);
        }
    }
}
