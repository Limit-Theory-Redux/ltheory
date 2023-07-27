use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::args::ImplAttrArgs;

use super::method_info::*;
use super::type_info::*;
use super::ImplInfo;

impl ImplInfo {
    /// Generate C API and Lua FFI.
    pub fn generate(&self, attr_args: ImplAttrArgs) -> TokenStream {
        // Original impl source code (with removed `bind` attributes)
        let source = &self.source;
        // C API wrapper functions
        let method_tokens: Vec<_> = self
            .methods
            .iter()
            .map(|method| self.wrap_methods(method))
            .collect();
        // Additional Free C API wrapper if requested
        let free_method_token = if attr_args.is_managed() {
            let module_name = attr_args.name().unwrap_or(self.name.clone());
            let free_method_ident = format_ident!("{module_name}_Free");
            let module_ident = format_ident!("{}", self.name);

            quote! {
                #[no_mangle]
                pub extern "C" fn #free_method_ident(_: Box<#module_ident>) {}
            }
        } else {
            quote! {}
        };

        if attr_args.gen_lua_ffi() {
            self.generate_ffi(&attr_args);
        }

        quote! {
            #source

            #free_method_token
            #(#method_tokens)*
        }
    }

    fn wrap_methods(&self, method: &MethodInfo) -> TokenStream {
        let method_name = method.as_ffi_name();
        let func_name = format!("{}_{}", self.name, method_name);
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

        let param_tokens: Vec<_> = method
            .params
            .iter()
            .map(|param| wrap_param(&self.name, param))
            .collect();

        let ret_token = if let Some(ty) = &method.ret {
            let ty_token = wrap_ret_type(&self.name, &ty);

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
}

fn wrap_param(self_name: &str, param: &ParamInfo) -> TokenStream {
    let param_name_ident = format_ident!("{}", param.name);
    let param_type_token = wrap_type(self_name, &param.ty);

    quote! { #param_name_ident: #param_type_token }
}

fn wrap_type(self_name: &str, ty: &TypeInfo) -> TokenStream {
    match &ty.variant {
        TypeVariant::Str | TypeVariant::String | TypeVariant::CString => {
            if ty.is_mutable {
                quote! { *mut libc::c_char }
            } else {
                quote! { *const libc::c_char }
            }
        }
        TypeVariant::Custom(ty_name) => {
            let ty_ident = if ty.is_self() {
                format_ident!("{self_name}")
            } else {
                format_ident!("{ty_name}")
            };

            if ty.is_option {
                if ty.is_mutable {
                    quote! { *mut #ty_ident }
                } else {
                    quote! { *const #ty_ident }
                }
            } else {
                if ty.is_mutable {
                    // Mutable is always with reference
                    quote! { &mut #ty_ident }
                } else if TypeInfo::is_copyable(&ty_name) {
                    // Ignore immutable reference of the copyable type
                    quote! { #ty_ident }
                } else if ty.is_reference {
                    quote! { &#ty_ident }
                } else {
                    quote! { Box<#ty_ident> }
                }
            }
        }
        _ => {
            let ty_ident = format_ident!("{}", ty.variant.as_string());

            if ty.is_option {
                // All options are sent by pointer
                if ty.is_mutable {
                    quote! { *mut #ty_ident }
                } else {
                    quote! { *const #ty_ident }
                }
            } else if ty.is_mutable {
                // Mutable is always with reference
                quote! { &mut #ty_ident }
            } else {
                // We don't care if there is reference on the numeric type - just accept it by value
                quote! { #ty_ident }
            }
        }
    }
}

fn wrap_ret_type(self_name: &str, ty: &TypeInfo) -> TokenStream {
    match &ty.variant {
        TypeVariant::Str | TypeVariant::String | TypeVariant::CString => {
            quote! { *const libc::c_char }
        }
        TypeVariant::Custom(ty_name) => {
            let ty_ident = if ty.is_self() {
                format_ident!("{self_name}")
            } else {
                format_ident!("{ty_name}")
            };

            if ty.is_option {
                quote! { *const #ty_ident }
            } else if TypeInfo::is_copyable(&ty_name) {
                quote! { #ty_ident }
            } else if ty.is_reference {
                // TODO: copyable reference
                if ty.is_mutable {
                    quote! { *#ty_ident }
                } else {
                    quote! { *const #ty_ident }
                }
            } else {
                quote! { Box<#ty_ident> }
            }
        }
        _ => {
            let ty_ident = format_ident!("{}", ty.variant.as_string());

            if ty.is_option {
                quote! { *const #ty_ident }
            } else {
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
            let name_accessor = if param.ty.is_option {
                quote! { (*#name_ident) }
            } else {
                quote! { #name_ident }
            };

            let param_item = match &param.ty.variant {
                TypeVariant::Str => quote! { #name_accessor.as_str() },
                TypeVariant::String => quote! { #name_accessor.as_string() },
                TypeVariant::CString => quote! { #name_accessor.as_cstring() },
                TypeVariant::Custom(custom_ty) => {
                    if TypeInfo::is_copyable(&custom_ty) {
                        if param.ty.is_reference && !param.ty.is_mutable {
                            quote! { &#name_accessor }
                        } else {
                            quote! { #name_accessor }
                        }
                    } else if param.ty.is_reference{
                        quote! { #name_accessor }
                    } else {
                        // FIXME: Boxed type. into_inner is nightly only. Alternative Option::unwrap
                        quote! { #name_accessor.into_inner() }
                    }
                },
                _ => {
                    if param.ty.is_mutable {
                        quote! { &mut #name_accessor }
                    } else if param.ty.is_reference {
                        quote! { &#name_accessor }
                    } else {
                        quote! { #name_accessor }
                    }
                }
            };

            if param.ty.is_option {
                quote! {if #name_ident != std::ptr::null_mut() { unsafe { Some(#param_item) } } else { None }}
            } else {
                param_item
            }
        })
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

        let method_call = if ty.is_option {
            quote! {
                #method_call
                let Some(__res__) = __res__ else { return std::ptr::null(); };
            }
        } else {
            method_call
        };

        let return_item = match &ty.variant {
            TypeVariant::Str | TypeVariant::String => quote! { static_string!(__res__) },
            TypeVariant::CString => quote! { static_cstring!(__res__) },
            TypeVariant::Custom(custom_ty) => {
                let type_ident = if ty.is_self() {
                    self_ident.clone()
                } else {
                    format_ident!("{custom_ty}")
                };

                if ty.is_option {
                    gen_buffered_ret(&type_ident)
                } else if ty.is_reference {
                    if ty.is_mutable {
                        quote! { __res__ as * #type_ident }
                    } else {
                        quote! { __res__ as *const #type_ident }
                    }
                } else if ty.is_self() || !TypeInfo::is_copyable(&custom_ty) {
                    // Do boxing
                    quote! { __res__.into() }
                } else {
                    quote! { __res__ }
                }
            }
            _ => {
                if ty.is_option {
                    let type_ident = format_ident!("{}", ty.variant.as_string());

                    gen_buffered_ret(&type_ident)
                } else {
                    quote! { __res__ }
                }
            }
        };

        quote! {
            #method_call
            #return_item
        }
    } else {
        quote! {
            #accessor_token(#(#param_tokens),*);
        }
    }
}

fn gen_buffered_ret(type_ident: &Ident) -> TokenStream {
    quote! {
        unsafe {
            static mut __BUFFER__: Option<#type_ident> = None;

            __BUFFER__ = Some(__res__);

            __BUFFER__.as_ref().unwrap()  as *const #type_ident
        }
    }
}
