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

        // extern "C-unwind" wrapper functions
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
                pub extern "C-unwind" fn #free_method_ident(_: Box<#module_ident>) {}
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

        #[cfg(feature = "log_ffi_calls")]
        let ffi_call_log = quote! {tracing::trace!("Calling: {}", #func_name);};
        #[cfg(not(feature = "log_ffi_calls"))]
        let ffi_call_log = quote! {};

        quote! {
            #[no_mangle]
            pub unsafe extern "C-unwind" fn #func_ident(#self_token #(#param_tokens),*) #ret_token {
                #ffi_call_log

                #func_body
            }
        }
    }

    // Note: We return a list of token streams here, because a single parameter can generate multiple parameters in the wrapper function.
    fn gen_wrapper_param(&self, param: &ParamInfo) -> Vec<TokenStream> {
        let name_ident = format_ident!("{}", param.name);
        let param_type = param.ty.as_ffi(&self.name).0;
        let param_type_tokens: TokenStream =
            param_type.parse().expect("Unable to parse Rust FFI type");

        let mut tokens = vec![quote! { #name_ident: #param_type_tokens }];

        // If this is a slice or array, we need to additionally generate a "size" parameter.
        match &param.ty {
            TypeInfo::Slice { .. } | TypeInfo::Array { .. } => {
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
        let mut prelude = vec![];
        let mut use_convert_into_string = false;
        let param_tokens: Vec<_> = method
            .params
            .iter()
            .map(|param| {
                self.gen_wrapper_name_from_ffi(
                    &param.name,
                    &param.ty,
                    &mut use_convert_into_string,
                    &mut prelude,
                )
            })
            .collect();

        // If we ended up using the ConvertIntoString trait, make sure to bring it into scope.
        let use_convert_to_string = if use_convert_into_string {
            quote! { use ::internal::ConvertIntoString; }
        } else {
            quote! {}
        };

        if let Some(ty) = &method.ret {
            let method_call = if let TypeInfo::Result { .. } = ty {
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

            let ty = if let TypeInfo::Result { inner } = ty {
                &**inner
            } else {
                ty
            };

            let return_item = match ty {
                TypeInfo::Plain { is_ref, ty } => match ty {
                    TypeVariant::Str => {
                        quote! {
                            internal::static_string!(__res__)
                        }
                    }
                    TypeVariant::String => {
                        if *is_ref != TypeRef::Value {
                            quote! {
                                internal::static_string!(__res__.as_str())
                            }
                        } else {
                            quote! {
                                internal::static_string!(__res__)
                            }
                        }
                    }
                    _ => {
                        // If it's a managed type by value, we need to box it when returning.
                        if !ty.is_copyable(&self.name) && *is_ref == TypeRef::Value {
                            quote! { __res__.into() }
                        } else {
                            quote! { __res__ }
                        }
                    }
                },
                TypeInfo::Option {
                    is_ref,
                    inner_ty: ty,
                } => match ty {
                    TypeVariant::Str => {
                        let unwrap = if *is_ref == TypeRef::MutableReference {
                            quote! { let Some(__res__) = __res__ else { return std::ptr::null_mut(); }; }
                        } else {
                            quote! { let Some(__res__) = __res__ else { return std::ptr::null(); }; }
                        };

                        quote! {
                            #unwrap
                            internal::static_string!(__res__)
                        }
                    }
                    TypeVariant::String => {
                        let unwrap = if *is_ref == TypeRef::MutableReference {
                            quote! { let Some(__res__) = __res__ else { return std::ptr::null_mut(); }; }
                        } else {
                            quote! { let Some(__res__) = __res__ else { return std::ptr::null(); }; }
                        };

                        if is_ref.is_reference() {
                            quote! {
                                #unwrap
                                internal::static_string!(__res__.as_str())
                            }
                        } else {
                            quote! {
                                #unwrap
                                internal::static_string!(__res__)
                            }
                        }
                    }
                    _ => {
                        if is_ref.is_reference() {
                            quote! { __res__ }
                        } else if ty.is_copyable(&self.name) {
                            let ty_ident = format_ident!("{}", ty.as_ffi(&self.name).0);

                            self.gen_buffered_ret(&ty_ident)
                        } else {
                            // Option<T> -> Option<Box<T>>.
                            quote! { __res__.map(Box::new) }
                        }
                    }
                },
                TypeInfo::Box { inner_ty: ty } => match ty {
                    TypeVariant::Str | TypeVariant::String => {
                        panic!("Boxed strings are not supported.")
                    }
                    _ => {
                        quote! { __res__ }
                    }
                },
                _ => {
                    panic!(
                        "Returning a slice, array or function is not supported. {:?}",
                        ty
                    )
                }
            };

            if method.bind_args.gen_out_param() {
                quote! {
                    #use_convert_to_string
                    #(#prelude);*
                    #method_call
                    *out = #return_item;
                }
            } else {
                quote! {
                    #use_convert_to_string
                    #(#prelude);*
                    #method_call
                    #return_item
                }
            }
        } else {
            quote! {
                #use_convert_to_string
                #(#prelude);*
                #accessor_token(#(#param_tokens),*);
            }
        }
    }

    fn gen_wrapper_return_type(&self, ty: &TypeInfo, never_box: bool) -> TokenStream {
        let ty = if let TypeInfo::Result { inner } = ty {
            &**inner
        } else {
            ty
        };

        match ty {
            TypeInfo::Plain { is_ref, ty } => match ty {
                TypeVariant::Str | TypeVariant::String => {
                    quote! { *const libc::c_char }
                }
                _ => {
                    let ty_ident = format_ident!("{}", ty.as_ffi(&self.name).0);

                    match is_ref {
                        TypeRef::MutableReference => quote! { &mut #ty_ident },
                        TypeRef::Reference => quote! { &#ty_ident },
                        TypeRef::Value if ty.is_copyable(&self.name) || never_box => {
                            quote! { #ty_ident }
                        }
                        TypeRef::Value => quote! { Box<#ty_ident> },
                    }
                }
            },
            TypeInfo::Option {
                is_ref,
                inner_ty: ty,
            } => {
                match ty {
                    TypeVariant::Str | TypeVariant::String => {
                        quote! { *const libc::c_char }
                    }
                    _ => {
                        let ty_ident = format_ident!("{}", ty.as_ffi(&self.name).0);

                        match is_ref {
                            TypeRef::MutableReference => quote! { Option<&mut #ty_ident> },
                            TypeRef::Reference => quote! { Option<&#ty_ident> },
                            TypeRef::Value if ty.is_copyable(&self.name) => {
                                // We pin a thread-local instance of Option<T> using
                                // gen_buffered_ret to encode None, so instead we
                                // return a pointer.
                                quote! { *const #ty_ident }
                            }
                            TypeRef::Value => {
                                quote! { Option<Box<#ty_ident>> }
                            }
                        }
                    }
                }
            }
            TypeInfo::Box { inner_ty: ty } => match ty {
                TypeVariant::Str | TypeVariant::String => {
                    panic!("Boxed strings are not supported.")
                }
                TypeVariant::Custom(ty_name) => {
                    let ty_name = if ty.is_self() { &self.name } else { ty_name };
                    let ty_ident = format_ident!("{ty_name}");

                    quote! { Box<#ty_ident> }
                }
                _ => {
                    let ty_ident = format_ident!("{}", ty.as_ffi(&self.name).0);

                    quote! { Box<#ty_ident> }
                }
            },
            _ => panic!(
                "Returning a slice, array or function is not supported. {:?}",
                ty
            ),
        }
    }

    /// This generates the code to turn a named object of an FFI type into a Rust type.
    fn gen_wrapper_name_from_ffi(
        &self,
        name: &String,
        ty: &TypeInfo,
        use_convert_into_string: &mut bool,
        prelude: &mut Vec<TokenStream>,
    ) -> TokenStream {
        let name_ident = format_ident!("{}", name);
        let name_accessor = quote! { #name_ident };

        match ty {
            TypeInfo::Function { args, ret_ty } => {
                // Assign a name for each argument.
                let args: Vec<_> = args
                    .iter()
                    .enumerate()
                    .map(|(index, ty)| (format!("arg{}", index + 1), ty))
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

                if let Some(ty) = ret_ty.as_ref() {
                    let mut ret_prelude = vec![];
                    let ret_expr = self.gen_wrapper_name_from_ffi(
                        &"ret".to_string(),
                        ty,
                        use_convert_into_string,
                        &mut ret_prelude,
                    );
                    quote! {
                        |#(#arg_tokens),*| {
                            #(#prelude);*
                            let ret = #name_accessor(#(#param_tokens),*);
                            #(#ret_prelude);*
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
            TypeInfo::Plain { is_ref, ty } => {
                match ty {
                    TypeVariant::Str => {
                        *use_convert_into_string = true;
                        quote! { #name_accessor.as_str() }
                    }
                    TypeVariant::String => {
                        *use_convert_into_string = true;
                        if is_ref.is_reference() {
                            quote! { &#name_accessor.as_string() }
                        } else {
                            quote! { #name_accessor.as_string() }
                        }
                    }
                    TypeVariant::Custom(_) => {
                        if ty.is_copyable(&self.name) || is_ref.is_reference() {
                            quote! { #name_accessor }
                        } else {
                            quote! { *#name_accessor }
                        }
                    }
                    _ => {
                        if is_ref.is_reference()
                            && !is_ref.is_mutable()
                            && ty.is_copyable(&self.name)
                        {
                            // Primitives passed by reference are received in FFI by value, so convert them
                            // back into a reference.
                            quote! { &#name_accessor }
                        } else {
                            quote! { #name_accessor }
                        }
                    }
                }
            }
            TypeInfo::Option {
                is_ref,
                inner_ty: ty,
            } => {
                match ty {
                    TypeVariant::Str => {
                        *use_convert_into_string = true;
                        quote! { if #name_accessor.is_null() { None } else { Some(#name_accessor.as_str())} }
                    }
                    TypeVariant::String => {
                        *use_convert_into_string = true;
                        if is_ref.is_reference() {
                            quote! { if #name_accessor.is_null() { None } else { Some(#name_accessor.as_string()) }.as_ref() }
                        } else {
                            quote! { if #name_accessor.is_null() { None } else { Some(#name_accessor.as_string()) } }
                        }
                    }
                    _ => {
                        if is_ref.is_reference() {
                            quote! { #name_accessor }
                        } else if ty.is_copyable(&self.name) {
                            // We need to promote Option<&T> to Option<T> if ty is copyable and it's passed by value
                            quote! { #name_accessor.cloned() }
                        } else {
                            // Option<Box<T>> -> Option<T>
                            quote! { #name_accessor.map(|inner| *inner) }
                        }
                    }
                }
            }
            TypeInfo::Box { .. } => {
                quote! { #name_accessor }
            }
            TypeInfo::Slice {
                is_ref,
                elem_ty: ty,
            } => {
                let size_param_ident = format_ident!("{}_size", name);

                prelude.push(quote! {
                    assert!(!#name_accessor.is_null(), "array pointer is null");
                    assert!(#size_param_ident > 0, "array length must be greater than 0");
                });

                match ty {
                    TypeVariant::Str | TypeVariant::String => {
                        assert!(*is_ref != TypeRef::MutableReference);
                        *use_convert_into_string = true;
                        let ptr_conversion = match ty {
                            TypeVariant::Str => quote! { ptr.as_str() },
                            TypeVariant::String => quote! { ptr.as_string() },
                            _ => panic!("Unhandled pointer conversion."),
                        };
                        prelude.push(quote! {
                            let #name_accessor: Vec<_> = std::slice::from_raw_parts(#name_accessor, #size_param_ident).iter().map(|ptr| #ptr_conversion).collect();
                        });
                        quote! { #name_accessor.as_slice() }
                    }
                    _ => {
                        if *is_ref == TypeRef::MutableReference {
                            quote! {
                                std::slice::from_raw_parts_mut(#name_accessor, #size_param_ident)
                            }
                        } else {
                            quote! {
                                std::slice::from_raw_parts(#name_accessor, #size_param_ident)
                            }
                        }
                    }
                }
            }
            TypeInfo::Array {
                is_ref,
                elem_ty: ty,
                length,
            } => {
                let size_param_ident = format_ident!("{}_size", name);

                prelude.push(quote! {
                    assert!(!#name_accessor.is_null(), "array pointer is null");
                    assert_eq!(#size_param_ident, #length, "incorrect number of elements for array");
                });

                match ty {
                    TypeVariant::Str | TypeVariant::String => {
                        *use_convert_into_string = true;
                        let ptr_conversion = match ty {
                            TypeVariant::Str => quote! { ptr.as_str() },
                            TypeVariant::String => quote! { ptr.as_string() },
                            _ => panic!("Unhandled pointer conversion."),
                        };
                        prelude.push(quote! {
                            let #name_accessor: Vec<_> = std::slice::from_raw_parts(#name_accessor, #size_param_ident).iter().map(|ptr| #ptr_conversion).collect();
                        });
                        match is_ref {
                            TypeRef::MutableReference => {
                                panic!("Mutable ref to an array of strings is not supported.")
                            }
                            TypeRef::Reference => {
                                quote! { #name_accessor.as_slice().try_into().unwrap() }
                            }
                            TypeRef::Value => quote! { #name_accessor.try_into().unwrap() },
                        }
                    }
                    _ => match is_ref {
                        TypeRef::MutableReference => quote! {
                            std::slice::from_raw_parts_mut(#name_accessor, #length).try_into().unwrap()
                        },
                        TypeRef::Reference => quote! {
                            std::slice::from_raw_parts(#name_accessor, #length).try_into().unwrap()
                        },
                        TypeRef::Value => quote! {
                            std::slice::from_raw_parts(#name_accessor, #length).to_owned().try_into().unwrap()
                        },
                    },
                }
            }
            TypeInfo::Result { .. } => {
                panic!("Result can only be used in the return position.")
            }
        }
    }

    /// This generates the code to turn a named object of a Rust type into an FFI type.
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
        match ty {
            TypeInfo::Plain { is_ref, ty } => {
                match ty {
                    TypeVariant::Str | TypeVariant::String => {
                        prelude.push(
                            quote! { let #name_ident = std::ffi::CString::new(#name_ident).unwrap(); },
                        );
                        quote! { #name_ident.as_ptr() }
                    }
                    _ => {
                        // If it's a managed type by value, we need to box it when returning.
                        if !ty.is_copyable(&self.name) && *is_ref == TypeRef::Value {
                            quote! { #name_ident.into() }
                        } else {
                            quote! { #name_ident }
                        }
                    }
                }
            }
            TypeInfo::Option {
                is_ref,
                inner_ty: ty,
            } => {
                match ty {
                    TypeVariant::Str | TypeVariant::String => {
                        prelude.push(
                            quote! { let #name_ident = std::ffi::CString::new(#name_ident).unwrap(); },
                        );
                        quote! { #name_ident.as_ptr() }
                    }
                    _ => {
                        if is_ref.is_reference() {
                            // If it's a reference, then pass it directly.
                            quote! { #name_ident }
                        } else if ty.is_copyable(&self.name) {
                            // If it's passed by value and is copyable, then we need to convert it
                            // to a reference.
                            quote! { #name_ident.as_ref() }
                        } else {
                            // We need to wrap it in a Box when transferring ownership to the callback.
                            quote! { #name_ident.map(Box::new) }
                        }
                    }
                }
            }
            TypeInfo::Box { .. } => {
                quote! { #name_ident }
            }
            TypeInfo::Slice {
                is_ref,
                elem_ty: ty,
            }
            | TypeInfo::Array {
                is_ref,
                elem_ty: ty,
                ..
            } => match ty {
                TypeVariant::Str | TypeVariant::String => {
                    panic!("Slice/Array of strings not implemented.")
                }
                _ => {
                    if *is_ref == TypeRef::MutableReference {
                        quote! { #name_ident.as_mut_ptr(), #name_ident.len() }
                    } else {
                        quote! { #name_ident.as_ptr(), #name_ident.len() }
                    }
                }
            },
            TypeInfo::Function { .. } => panic!("Cannot convert a function type to FFI"),
            TypeInfo::Result { .. } => panic!("Cannot convert a result type to FFI"),
        }
    }

    fn gen_buffered_ret(&self, type_ident: &Ident) -> TokenStream {
        quote! {
            unsafe {
                thread_local! { static __BUFFER__: std::cell::RefCell<Option<#type_ident>> = Default::default(); }
                __BUFFER__.replace(__res__);
                __BUFFER__.with_borrow(|buf| match buf.as_ref() {
                    Some(val) => val as *const _,
                    None => std::ptr::null(),
                })
            }
        }
    }
}
