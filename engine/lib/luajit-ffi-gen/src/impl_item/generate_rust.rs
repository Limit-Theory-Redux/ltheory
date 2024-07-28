use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use super::method_info::*;
use super::type_info::*;
use super::ImplInfo;
use crate::args::ImplAttrArgs;

impl ImplInfo {
    pub fn gen_rust_ffi(&self, attr_args: &ImplAttrArgs) -> TokenStream {
        let module_name = attr_args.name().unwrap_or(self.name.clone());

        // extern "C" wrapper functions
        let method_tokens: Vec<_> = self
            .methods
            .iter()
            .map(|method| self.gen_wrapper_fn(&module_name, method))
            .collect();

        // Free wrapper function, if the type is managed.
        let free_method_token = if self.is_managed() {
            let free_method_ident = format_ident!("{module_name}_Free");
            let module_ident = format_ident!("{}", self.name);

            quote! {
                #[no_mangle]
                pub extern "C" fn #free_method_ident(_: Box<#module_ident>) {}
            }
        } else {
            quote! {}
        };

        quote! {
            #free_method_token
            #(#method_tokens)*
        }
    }

    fn gen_wrapper_fn(&self, module_name: &str, method: &MethodInfo) -> TokenStream {
        let method_name = method.as_ffi_name();
        let func_name = format!("{module_name}_{}", method_name);
        let func_ident = format_ident!("{func_name}");
        let self_ident = format_ident!("{}", self.name);

        let self_token = if let Some(self_param) = &method.self_param {
            if self_param.is_mutable {
                quote! { this: &mut #self_ident, }
            } else {
                quote! { this: &#self_ident, }
            }
        } else {
            quote! {}
        };

        // Generate parameter list.
        let mut param_tokens: Vec<_> = method
            .params
            .iter()
            .flat_map(|param| self.gen_wrapper_param(param))
            .collect();

        if method.bind_args.gen_out_param() && method.ret.is_some() {
            let return_ty_token = self.gen_wrapper_return_type(method.ret.as_ref().unwrap(), true);
            param_tokens.push(quote! { out: &mut #return_ty_token })
        }

        let ret_token = if method.bind_args.gen_out_param() || method.ret.is_none() {
            quote! {}
        } else {
            let ret = method.ret.as_ref().unwrap();
            let ty_token = self.gen_wrapper_return_type(ret, false);

            quote! { -> #ty_token }
        };

        // Generate function body.
        let func_body = self.gen_wrapper_body(&self_ident, method);

        quote! {
            #[no_mangle]
            pub unsafe extern "C" fn #func_ident(#self_token #(#param_tokens),*) #ret_token {
                tracing::trace!("Calling: {}", #func_name);

                #func_body
            }
        }
    }

    // Note: We return a list of token streams here, because a single parameter can generate multiple parameters in the wrapper function.
    fn gen_wrapper_param(&self, param: &ParamInfo) -> Vec<TokenStream> {
        let param_name_ident = format_ident!("{}", param.name);
        let param_type_token = self.gen_wrapper_type(&param.ty);

        let mut tokens = vec![quote! { #param_name_ident: #param_type_token }];

        // If this is a slice, we need to additionally generate a "size" parameter.
        if param.ty.wrapper == TypeWrapper::Slice {
            let slice_size_param_ident = format_ident!("{}_size", param.name);
            tokens.push(quote! { #slice_size_param_ident: u32 });
        }

        tokens
    }

    fn gen_wrapper_body(&self, self_ident: &Ident, method: &MethodInfo) -> TokenStream {
        let method_ident = format_ident!("{}", method.name);
        let accessor_token = if method.self_param.is_some() {
            quote! { this.#method_ident }
        } else {
            quote! { #self_ident::#method_ident }
        };

        // Generate tokens to convert the parameters from the extern interface to the impl interface.
        let param_tokens: Vec<_> = method
            .params
            .iter()
            .map(|param| self.gen_wrapper_param_to_impl(param))
            .collect();

        if let Some(ty) = &method.ret {
            let method_call = if ty.is_result {
                let method_call_str = format!("{}::{}", self_ident, method.name);

                quote! {
                    let __res__ = match #accessor_token(#(#param_tokens),*) {
                        Ok(res) => res,
                        Err(err) => {
                            panic!("Error on calling method '{}': {}", #method_call_str, err);
                        }
                    };
                }
            } else {
                quote! { let __res__ = #accessor_token(#(#param_tokens),*); }
            };

            let method_call = if ty.wrapper == TypeWrapper::Option {
                if ty.is_mutable {
                    quote! {
                        #method_call
                        let Some(__res__) = __res__ else { return std::ptr::null_mut(); };
                    }
                } else {
                    quote! {
                        #method_call
                        let Some(__res__) = __res__ else { return std::ptr::null(); };
                    }
                }
            } else {
                method_call
            };

            let return_item = match &ty.variant {
                TypeVariant::Str => quote! { internal::static_string!(__res__) },
                TypeVariant::String => {
                    if ty.is_reference {
                        quote! { internal::static_string!(__res__.as_str()) }
                    } else {
                        quote! { internal::static_string!(__res__) }
                    }
                }
                TypeVariant::CString => quote! { static_cstring!(__res__) },
                TypeVariant::Custom(custom_ty) => {
                    let type_ident = if ty.is_self() {
                        self_ident.clone()
                    } else {
                        format_ident!("{custom_ty}")
                    };
                    let is_copyable = TypeInfo::is_copyable(custom_ty)
                        || TypeInfo::is_copyable(&self_ident.to_string());

                    if ty.wrapper == TypeWrapper::Option && !ty.is_reference {
                        if is_copyable {
                            quote! { &__res__ }
                        } else {
                            self.gen_buffered_ret(&type_ident)
                        }
                    } else if is_copyable || method.bind_args.gen_out_param() {
                        if ty.is_reference {
                            quote! { *__res__ }
                        } else {
                            quote! { __res__ }
                        }
                    } else if ty.is_mutable {
                        quote! { __res__ as *mut #type_ident }
                    } else if ty.is_reference {
                        quote! { __res__ as *const #type_ident }
                    } else if ty.wrapper != TypeWrapper::Box {
                        quote! { __res__.into() }
                    } else {
                        quote! { __res__ }
                    }
                }
                _ => {
                    if ty.wrapper == TypeWrapper::Option {
                        let type_ident = format_ident!("{}", ty.variant.as_string());

                        self.gen_buffered_ret(&type_ident)
                    } else {
                        quote! { __res__ }
                    }
                }
            };

            if method.bind_args.gen_out_param() {
                quote! {
                    #method_call
                    *out = #return_item;
                }
            } else {
                quote! {
                    #method_call
                    #return_item
                }
            }
        } else {
            quote! {
                #accessor_token(#(#param_tokens),*);
            }
        }
    }

    fn gen_wrapper_type(&self, ty: &TypeInfo) -> TokenStream {
        match &ty.variant {
            TypeVariant::Str | TypeVariant::String | TypeVariant::CString => {
                if ty.is_mutable {
                    quote! { *mut libc::c_char }
                } else {
                    quote! { *const libc::c_char }
                }
            }
            TypeVariant::Custom(ty_name) => {
                let ty_name = if ty.is_self() { &self.name } else { ty_name };
                let ty_ident = format_ident!("{ty_name}");

                match ty.wrapper {
                    TypeWrapper::Option => {
                        // Options are always pointers to the custom type.
                        if ty.is_mutable {
                            quote! { *mut #ty_ident }
                        } else {
                            quote! { *const #ty_ident }
                        }
                    }
                    TypeWrapper::Slice => {
                        // Slices are always pointers to the custom type.
                        if ty.is_mutable {
                            quote! { *mut #ty_ident }
                        } else {
                            quote! { *const #ty_ident }
                        }
                    }
                    _ => {
                        if ty.is_mutable {
                            // Mutable is always with reference
                            quote! { &mut #ty_ident }
                        } else if ty.is_reference {
                            quote! { &#ty_ident }
                        } else if TypeInfo::is_copyable(ty_name) {
                            quote! { #ty_ident }
                        } else {
                            quote! { Box<#ty_ident> }
                        }
                    }
                }
            }
            _ => {
                let ty_ident = format_ident!("{}", ty.variant.as_string());

                match ty.wrapper {
                    TypeWrapper::Option => {
                        // Options are always pointers to the primitive type.
                        if ty.is_mutable {
                            quote! { *mut #ty_ident }
                        } else {
                            quote! { *const #ty_ident }
                        }
                    }
                    TypeWrapper::Slice => {
                        // Slices are always pointers to the primitive type.
                        if ty.is_mutable {
                            quote! { *mut #ty_ident }
                        } else {
                            quote! { *const #ty_ident }
                        }
                    }
                    _ => {
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
        }
    }

    fn gen_wrapper_return_type(&self, ty: &TypeInfo, never_box: bool) -> TokenStream {
        match &ty.variant {
            TypeVariant::Str | TypeVariant::String | TypeVariant::CString => {
                quote! { *const libc::c_char }
            }
            TypeVariant::Custom(ty_name) => {
                let ty_name = if ty.is_self() { &self.name } else { ty_name };
                let ty_ident = format_ident!("{ty_name}");
                let is_copyable = TypeInfo::is_copyable(ty_name.as_str());

                if ty.wrapper == TypeWrapper::Option {
                    if ty.is_mutable {
                        quote! { *mut #ty_ident }
                    } else {
                        quote! { *const #ty_ident }
                    }
                } else if (is_copyable && ty.wrapper != TypeWrapper::Box) || never_box {
                    quote! { #ty_ident }
                } else if ty.is_mutable {
                    quote! { *mut #ty_ident }
                } else if ty.is_reference {
                    quote! { *const #ty_ident }
                } else {
                    quote! { Box<#ty_ident> }
                }
            }
            _ => {
                let ty_ident = format_ident!("{}", ty.variant.as_string());

                if ty.wrapper == TypeWrapper::Option {
                    quote! { *const #ty_ident }
                } else {
                    quote! { #ty_ident }
                }
            }
        }
    }

    fn gen_wrapper_param_to_impl(&self, param: &ParamInfo) -> TokenStream {
        let name_ident = format_ident!("{}", param.name);
        let name_accessor =
            if param.ty.wrapper == TypeWrapper::Option && !param.ty.variant.is_string() {
                quote! { (*#name_ident) }
            } else {
                quote! { #name_ident }
            };

        let param_item = match &param.ty.variant {
            TypeVariant::Str => quote! { #name_accessor.as_str() },
            TypeVariant::String => {
                if param.ty.is_reference {
                    quote! { &#name_accessor.as_string() }
                } else {
                    quote! { #name_accessor.as_string() }
                }
            }
            TypeVariant::CString => quote! { #name_accessor.as_cstring() },
            TypeVariant::Custom(custom_ty) => {
                if param.ty.wrapper == TypeWrapper::Slice {
                    let slice_size_param_ident = format_ident!("{}_size", param.name);
                    if param.ty.is_mutable {
                        quote! { std::slice::from_raw_parts_mut(#name_accessor, #slice_size_param_ident as usize) }
                    } else {
                        quote! { std::slice::from_raw_parts(#name_accessor, #slice_size_param_ident as usize) }
                    }
                } else if param.ty.wrapper == TypeWrapper::Box || TypeInfo::is_copyable(custom_ty) {
                    quote! { #name_accessor }
                } else if param.ty.is_reference {
                    if param.ty.wrapper == TypeWrapper::Option {
                        quote! { &#name_accessor }
                    } else {
                        quote! { #name_accessor }
                    }
                } else {
                    quote! { *#name_accessor }
                }
            }
            _ => {
                if param.ty.wrapper == TypeWrapper::Slice {
                    let slice_size_param_ident = format_ident!("{}_size", param.name);
                    if param.ty.is_mutable {
                        quote! { std::slice::from_raw_parts_mut(#name_accessor, #slice_size_param_ident as usize) }
                    } else {
                        quote! { std::slice::from_raw_parts(#name_accessor, #slice_size_param_ident as usize) }
                    }
                } else if param.ty.is_mutable {
                    quote! { &mut #name_accessor }
                } else if param.ty.is_reference {
                    quote! { &#name_accessor }
                } else {
                    quote! { #name_accessor }
                }
            }
        };

        if param.ty.wrapper == TypeWrapper::Option {
            quote! {if !#name_ident.is_null() { unsafe { Some(#param_item) } } else { None }}
        } else {
            param_item
        }
    }

    fn gen_buffered_ret(&self, type_ident: &Ident) -> TokenStream {
        quote! {
            unsafe {
                static mut __BUFFER__: Option<#type_ident> = None;
                __BUFFER__ = Some(__res__);
                __BUFFER__.as_ref().unwrap() as *const #type_ident
            }
        }
    }
}
