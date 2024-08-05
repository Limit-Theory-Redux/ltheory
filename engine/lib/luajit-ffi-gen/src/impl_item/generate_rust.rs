use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use super::method_info::*;
use super::type_info::*;
use super::ImplInfo;
use crate::args::ImplAttrArgs;

impl ImplInfo {
    pub fn gen_rust_ffi(&self, attr_args: &ImplAttrArgs) -> TokenStream {
        let module_name = attr_args.name().unwrap_or(self.name.as_ref());

        // extern "C" wrapper functions
        let method_tokens: Vec<_> = self
            .methods
            .iter()
            .map(|method| self.gen_wrapper_fn(module_name, method))
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
        let name_ident = format_ident!("{}", param.name);
        let param_type = param.ty.as_ffi(&self.name).rust;
        let param_type_tokens: TokenStream =
            param_type.parse().expect("Unable to parse Rust FFI type");

        let mut tokens = vec![quote! { #name_ident: #param_type_tokens }];

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
            .map(|param| {
                self.gen_wrapper_name_from_ffi(&param.name, &param.ty, &mut use_convert_into_string)
            })
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

            let return_item = match &ty.variant {
                TypeVariant::Str => {
                    let maybe_unwrap = if ty.wrapper == TypeWrapper::Option {
                        if ty.is_mutable {
                            quote! { let Some(__res__) = __res__ else { return std::ptr::null_mut(); }; }
                        } else {
                            quote! { let Some(__res__) = __res__ else { return std::ptr::null(); }; }
                        }
                    } else {
                        quote! {}
                    };

                    quote! {
                        #maybe_unwrap
                        internal::static_string!(__res__)
                    }
                }
                TypeVariant::String => {
                    let maybe_unwrap = if ty.wrapper == TypeWrapper::Option {
                        if ty.is_mutable {
                            quote! { let Some(__res__) = __res__ else { return std::ptr::null_mut(); }; }
                        } else {
                            quote! { let Some(__res__) = __res__ else { return std::ptr::null(); }; }
                        }
                    } else {
                        quote! {}
                    };

                    if ty.is_reference {
                        quote! {
                            #maybe_unwrap
                            internal::static_string!(__res__.as_str())
                        }
                    } else {
                        quote! {
                            #maybe_unwrap
                            internal::static_string!(__res__)
                        }
                    }
                }
                TypeVariant::CString => {
                    let maybe_unwrap = if ty.wrapper == TypeWrapper::Option {
                        if ty.is_mutable {
                            quote! { let Some(__res__) = __res__ else { return std::ptr::null_mut(); }; }
                        } else {
                            quote! { let Some(__res__) = __res__ else { return std::ptr::null(); }; }
                        }
                    } else {
                        quote! {}
                    };

                    quote! {
                        #maybe_unwrap
                        internal::static_cstring!(__res__)
                    }
                }
                TypeVariant::Custom(ty_name) => {
                    let ty_name = if ty.is_self() { &self.name } else { ty_name };
                    let ty_ident = format_ident!("{ty_name}");

                    if ty.wrapper == TypeWrapper::Option {
                        if ty.is_reference || ty.is_mutable {
                            quote! { __res__ }
                        } else {
                            self.gen_buffered_ret(&ty_ident)
                        }
                    } else if ty.is_copyable(&self.name) || method.bind_args.gen_out_param() {
                        if ty.is_reference {
                            quote! { *__res__ }
                        } else {
                            quote! { __res__ }
                        }
                    } else if ty.is_mutable || ty.is_reference {
                        quote! { __res__ }
                    } else if ty.wrapper != TypeWrapper::Box {
                        quote! { __res__.into() }
                    } else {
                        quote! { __res__ }
                    }
                }
                _ => {
                    if ty.wrapper == TypeWrapper::Option {
                        if ty.is_reference || ty.is_mutable {
                            quote! { __res__ }
                        } else {
                            let type_ident = format_ident!("{}", ty.variant.as_ffi().rust);

                            self.gen_buffered_ret(&type_ident)
                        }
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
                        quote! { Option<&mut #ty_ident> }
                    } else if ty.is_reference {
                        quote! { Option<&#ty_ident> }
                    } else {
                        // We pin an instance of Option<T> using gen_buffered_ret, so ensure the
                        // right lifetime is used here.
                        quote! { Option<&'static #ty_ident> }
                    }
                } else if (ty.is_copyable(&self.name) && ty.wrapper != TypeWrapper::Box)
                    || never_box
                {
                    quote! { #ty_ident }
                } else if ty.is_mutable {
                    quote! { &mut #ty_ident }
                } else if ty.is_reference {
                    quote! { &#ty_ident }
                } else {
                    quote! { Box<#ty_ident> }
                }
            }
            _ => {
                let ty_ident = format_ident!("{}", ty.variant.as_ffi().rust);

                if ty.wrapper == TypeWrapper::Option {
                    if ty.is_mutable {
                        quote! { Option<&mut #ty_ident> }
                    } else if ty.is_reference {
                        quote! { Option<&#ty_ident> }
                    } else {
                        // We pin an instance of Option<T> using gen_buffered_ret, so ensure the
                        // right lifetime is used here.
                        quote! { Option<&'static #ty_ident> }
                    }
                } else {
                    quote! { #ty_ident }
                }
            }
        }
    }

    fn gen_wrapper_name_from_ffi(
        &self,
        name: &String,
        ty: &TypeInfo,
        use_convert_into_string: &mut bool,
    ) -> TokenStream {
        let name_ident = format_ident!("{}", name);
        let name_accessor = quote! { #name_ident };

        match &ty.variant {
            TypeVariant::Function { args, ret } => {
                // Assign a name for each argument.
                let args: Vec<_> = args
                    .iter()
                    .enumerate()
                    .map(|(index, ty)| (format!("arg{}", index), ty))
                    .collect();

                // Generate the tokens for the argument list.
                let arg_tokens: Vec<_> = args
                    .iter()
                    .map(|(name, _)| format_ident!("{}", name))
                    .collect();

                // Convert the Rust expressions to FFI.
                let mut prelude = vec![];
                let param_tokens: Vec<_> = args
                    .iter()
                    .map(|(name, ty)| self.gen_wrapper_name_to_ffi(name, ty, &mut prelude))
                    .collect();

                if let Some(ty) = ret.as_ref() {
                    let ret_expr = self.gen_wrapper_name_from_ffi(
                        &"ret".to_string(),
                        ty,
                        use_convert_into_string,
                    );
                    quote! {
                        |#(#arg_tokens),*| {
                            #(#prelude);*
                            let ret = #name_accessor(#(#param_tokens),*);
                            #ret_expr
                        }
                    }
                } else {
                    quote! {
                        |#(#arg_tokens),*| {
                            #(#prelude);*
                            #name_accessor(#(#param_tokens),*)
                        }
                    }
                }
            }
            TypeVariant::Str => {
                *use_convert_into_string = true;
                let str_expr = quote! { #name_accessor.as_str() };
                if ty.wrapper == TypeWrapper::Option {
                    quote! { if #name_accessor.is_null() { None } else { Some(#str_expr)} }
                } else {
                    str_expr
                }
            }
            TypeVariant::String => {
                *use_convert_into_string = true;

                // It's clearer to have it split in this way, rather than collapsing the else block below.
                #[allow(clippy::collapsible_else_if)]
                if ty.is_reference {
                    if ty.wrapper == TypeWrapper::Option {
                        quote! { if #name_accessor.is_null() { None } else { Some(#name_accessor.as_string()) }.as_ref() }
                    } else {
                        quote! { &#name_accessor.as_string() }
                    }
                } else {
                    if ty.wrapper == TypeWrapper::Option {
                        quote! { if #name_accessor.is_null() { None } else { Some(#name_accessor.as_string()) } }
                    } else {
                        quote! { #name_accessor.as_string() }
                    }
                }
            }
            TypeVariant::CString => {
                *use_convert_into_string = true;
                let cstring_expr = quote! { #name_accessor.as_cstring() };
                if ty.wrapper == TypeWrapper::Option {
                    quote! { if #name_accessor.is_null() { None } else { Some(#cstring_expr)} }
                } else {
                    cstring_expr
                }
            }
            TypeVariant::Custom(_) => {
                if ty.wrapper == TypeWrapper::Slice {
                    let size_param_ident = format_ident!("{}_size", name);
                    if ty.is_mutable {
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
                } else if let TypeWrapper::Array(size) = ty.wrapper {
                    let size_param_ident = format_ident!("{}_size", name);
                    if ty.is_mutable {
                        quote! {{
                            assert!(!#name_accessor.is_null(), "array pointer is null");
                            assert_eq!(#size_param_ident, #size, "incorrect number of elements for array");
                            std::slice::from_raw_parts_mut(#name_accessor, #size).try_into().unwrap()
                        }}
                    } else if ty.is_reference {
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
                } else if ty.wrapper == TypeWrapper::Box {
                    quote! { #name_accessor }
                } else if ty.wrapper == TypeWrapper::Option {
                    if ty.is_reference || ty.is_mutable {
                        quote! { #name_accessor }
                    } else {
                        // We need to promote Option<&T> to Option<T> if ty is neither a reference or mutable.
                        quote! { #name_accessor.copied() }
                    }
                } else if ty.is_copyable(&self.name) || ty.is_reference {
                    quote! { #name_accessor }
                } else {
                    quote! { *#name_accessor }
                }
            }
            _ => {
                if ty.wrapper == TypeWrapper::Slice {
                    let size_param_ident = format_ident!("{}_size", name);
                    if ty.is_mutable {
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
                } else if let TypeWrapper::Array(size) = ty.wrapper {
                    let size_param_ident = format_ident!("{}_size", name);
                    if ty.is_mutable {
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
                } else if ty.wrapper == TypeWrapper::Option {
                    if ty.is_mutable || ty.is_reference {
                        quote! { #name_accessor }
                    } else {
                        // We need to promote Option<&T> to Option<T> if ty is neither a
                        // reference or mutable.
                        quote! { #name_accessor.copied() }
                    }
                } else if ty.is_reference {
                    // Primitives passed by reference are received in FFI by value, so convert them
                    // back into a reference.
                    quote! { &#name_accessor }
                } else {
                    quote! { #name_accessor }
                }
            }
        }
    }

    /// This generates the code to turn a Rust type into an FFI type.
    ///
    /// Note that unlike gen_wrapper_body's return value conversion code, this does not worry about
    /// ownership as it leaves the FFI boundary because gen_wrapper_name_to_ffi is only used by
    /// callbacks right now, and callbacks don't need to worry about ownership of references because
    /// the callback's scope matches the scope of the borrows.
    fn gen_wrapper_name_to_ffi(
        &self,
        name: &String,
        ty: &TypeInfo,
        prelude: &mut Vec<TokenStream>,
    ) -> TokenStream {
        let name_ident = format_ident!("{}", name);
        match &ty.variant {
            TypeVariant::Str | TypeVariant::String => {
                prelude.push(
                    quote! { let #name_ident = std::ffi::CString::new(#name_ident).unwrap(); },
                );
                quote! { #name_ident.as_ptr() }
            }
            TypeVariant::CString => quote! { #name_ident.as_ptr() },
            // structs and primitives are handled in the same way.
            _ => {
                if ty.wrapper == TypeWrapper::Slice {
                    if ty.is_mutable {
                        quote! { #name_ident.as_mut_ptr(), #name_ident.len() }
                    } else {
                        quote! { #name_ident.as_ptr(), #name_ident.len() }
                    }
                } else if let TypeWrapper::Array(_) = ty.wrapper {
                    if ty.is_mutable {
                        quote! { #name_ident.as_mut_ptr(), #name_ident.len() }
                    } else {
                        quote! { #name_ident.as_ptr(), #name_ident.len() }
                    }
                } else if ty.wrapper == TypeWrapper::Option {
                    if !ty.is_reference {
                        // If it's passed by value, then we need to convert it to a reference.
                        quote! { #name_ident.as_ref() }
                    } else {
                        // If it's a reference or pass by move, then pass it directly.
                        quote! { #name_ident }
                    }
                } else if ty.is_copyable(&self.name) {
                    if ty.is_reference && !ty.is_mutable {
                        // If it's a reference, we need to convert it to a value.
                        quote! { *#name_ident }
                    } else {
                        // If it's copyable, then pass it directly.
                        quote! { #name_ident }
                    }
                } else if (ty.is_mutable || ty.is_reference) && ty.wrapper == TypeWrapper::None {
                    // A reference type that may or may not be wrapped by Option can be passed directly.
                    quote! { #name_ident }
                } else if ty.wrapper != TypeWrapper::Box {
                    // If it's not a box type, and non-copyable, we need to convert it into a Box<T>.
                    quote! { #name_ident.into() }
                } else {
                    // Pass it through directly.
                    quote! { #name_ident }
                }
            }
        }
    }

    fn gen_buffered_ret(&self, type_ident: &Ident) -> TokenStream {
        quote! {
            unsafe {
                static mut __BUFFER__: Option<#type_ident> = None;
                __BUFFER__ = __res__;
                __BUFFER__.as_ref()
            }
        }
    }
}
