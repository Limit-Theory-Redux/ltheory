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
        let param_type = param.ty.as_ffi(&self.name).rust;
        let param_type_tokens: TokenStream =
            param_type.parse().expect("Unable to parse Rust FFI type");

        let mut tokens = vec![quote! { #param_name_ident: #param_type_tokens }];

        // If this is a slice or array, we need to additionally generate a "size" parameter.
        match &param.ty.wrapper {
            TypeWrapper::Slice | TypeWrapper::Array(_) => {
                let size_param_ident = format_ident!("{}_size", param.name);
                tokens.push(quote! { #size_param_ident: usize });
            }
            _ => {}
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
        let mut use_convert_into_string = false;
        let param_tokens: Vec<_> = method
            .params
            .iter()
            .map(|param| self.gen_wrapper_param_to_impl(param, &mut use_convert_into_string))
            .collect();

        // If we ended up using the ConvertIntoString trait, make sure to bring it into scope.
        let prelude = if use_convert_into_string {
            quote! { use ::internal::ConvertIntoString; }
        } else {
            quote! {}
        };

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
                TypeVariant::Custom(ty_name) => {
                    let ty_name = if ty.is_self() { &self.name } else { ty_name };
                    let ty_ident = format_ident!("{ty_name}");

                    if ty.wrapper == TypeWrapper::Option && !ty.is_reference {
                        if ty.is_copyable(&self.name) {
                            quote! { &__res__ }
                        } else {
                            self.gen_buffered_ret(&ty_ident)
                        }
                    } else if ty.is_copyable(&self.name) || method.bind_args.gen_out_param() {
                        if ty.is_reference {
                            quote! { *__res__ }
                        } else {
                            quote! { __res__ }
                        }
                    } else if ty.is_mutable {
                        quote! { __res__ as *mut #ty_ident }
                    } else if ty.is_reference {
                        quote! { __res__ as *const #ty_ident }
                    } else if ty.wrapper != TypeWrapper::Box {
                        quote! { __res__.into() }
                    } else {
                        quote! { __res__ }
                    }
                }
                _ => {
                    if ty.wrapper == TypeWrapper::Option {
                        let type_ident = format_ident!("{}", ty.variant.as_ffi().rust);

                        self.gen_buffered_ret(&type_ident)
                    } else {
                        quote! { __res__ }
                    }
                }
            };

            if method.bind_args.gen_out_param() {
                quote! {
                    #prelude
                    #method_call
                    *out = #return_item;
                }
            } else {
                quote! {
                    #prelude
                    #method_call
                    #return_item
                }
            }
        } else {
            quote! {
                #prelude
                #accessor_token(#(#param_tokens),*);
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

                if ty.wrapper == TypeWrapper::Option {
                    if ty.is_mutable {
                        quote! { *mut #ty_ident }
                    } else {
                        quote! { *const #ty_ident }
                    }
                } else if (ty.is_copyable(&self.name) && ty.wrapper != TypeWrapper::Box)
                    || never_box
                {
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
                let ty_ident = format_ident!("{}", ty.variant.as_ffi().rust);

                if ty.wrapper == TypeWrapper::Option {
                    quote! { *const #ty_ident }
                } else {
                    quote! { #ty_ident }
                }
            }
        }
    }

    fn gen_wrapper_param_to_impl(
        &self,
        param: &ParamInfo,
        use_convert_into_string: &mut bool,
    ) -> TokenStream {
        let name_ident = format_ident!("{}", param.name);
        let name_accessor =
            if param.ty.wrapper == TypeWrapper::Option && !param.ty.variant.is_string() {
                quote! { (*#name_ident) }
            } else {
                quote! { #name_ident }
            };

        let param_item = match &param.ty.variant {
            TypeVariant::Str => {
                *use_convert_into_string = true;
                quote! { #name_accessor.as_str() }
            }
            TypeVariant::String => {
                *use_convert_into_string = true;
                if param.ty.is_reference {
                    quote! { &#name_accessor.as_string() }
                } else {
                    quote! { #name_accessor.as_string() }
                }
            }
            TypeVariant::CString => {
                *use_convert_into_string = true;
                quote! { #name_accessor.as_cstring() }
            }
            TypeVariant::Custom(_) => {
                if param.ty.wrapper == TypeWrapper::Slice {
                    let size_param_ident = format_ident!("{}_size", param.name);
                    if param.ty.is_mutable {
                        quote! {{
                            assert!(!#name_accessor.is_null(), "array pointer is null");
                            assert!(#size_param_ident > 0, "array length must be greater than 0");
                            std::slice::from_raw_parts_mut(#name_accessor, #size_param_ident)
                        }}
                    } else {
                        quote! {{
                            assert!(!#name_accessor.is_null(), "array pointer is null");
                            assert!(#size_param_ident > 0, "array length must be greater than 0");
                            std::slice::from_raw_parts(#name_accessor, #size_param_ident)
                        }}
                    }
                } else if let TypeWrapper::Array(size) = param.ty.wrapper {
                    let size_param_ident = format_ident!("{}_size", param.name);
                    if param.ty.is_mutable {
                        quote! {{
                            assert!(!#name_accessor.is_null(), "array pointer is null");
                            assert_eq!(#size_param_ident, #size, "incorrect number of elements for array");
                            std::slice::from_raw_parts_mut(#name_accessor, #size).try_into().unwrap()
                        }}
                    } else if param.ty.is_reference {
                        quote! {{
                            assert!(!#name_accessor.is_null(), "array pointer is null");
                            assert_eq!(#size_param_ident, #size, "incorrect number of elements for array");
                            std::slice::from_raw_parts(#name_accessor, #size).try_into().unwrap()
                        }}
                    } else {
                        quote! {{
                            assert!(!#name_accessor.is_null(), "array pointer is null");
                            assert_eq!(#size_param_ident, #size, "incorrect number of elements for array");
                            std::slice::from_raw_parts(#name_accessor, #size).to_owned().try_into().unwrap()
                        }}
                    }
                } else if param.ty.wrapper == TypeWrapper::Box || param.ty.is_copyable(&self.name) {
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
                    let size_param_ident = format_ident!("{}_size", param.name);
                    if param.ty.is_mutable {
                        quote! {{
                            assert!(!#name_accessor.is_null(), "array pointer is null");
                            assert!(#size_param_ident > 0, "array length must be greater than 0");
                            std::slice::from_raw_parts_mut(#name_accessor, #size_param_ident)
                        }}
                    } else {
                        quote! {{
                            assert!(!#name_accessor.is_null(), "array pointer is null");
                            assert!(#size_param_ident > 0, "array length must be greater than 0");
                            std::slice::from_raw_parts(#name_accessor, #size_param_ident)
                        }}
                    }
                } else if let TypeWrapper::Array(size) = param.ty.wrapper {
                    let size_param_ident = format_ident!("{}_size", param.name);
                    if param.ty.is_mutable {
                        quote! {{
                            assert!(!#name_accessor.is_null(), "array pointer is null");
                            assert_eq!(#size_param_ident, #size, "incorrect number of elements for array");
                            std::slice::from_raw_parts_mut(#name_accessor, #size).try_into().unwrap()
                        }}
                    } else {
                        quote! {{
                            assert!(!#name_accessor.is_null(), "array pointer is null");
                            assert_eq!(#size_param_ident, #size, "incorrect number of elements for array");
                            std::slice::from_raw_parts(#name_accessor, #size).try_into().unwrap()
                        }}
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
            quote! { if !#name_ident.is_null() { unsafe { Some(#param_item) } } else { None } }
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
