use super::{ImplInfo, ParamInfo, TypeInfo, TypeRef, TypeVariant};
use crate::args::ImplAttrArgs;
use crate::ffi_generator::FFIGenerator;
use crate::IDENT;

impl ImplInfo {
    /// Generate Lua FFI file
    pub fn gen_lua_ffi(&self, attr_args: &ImplAttrArgs) {
        let module_name = attr_args.name().unwrap_or(self.name.as_ref());

        let mut ffi_gen = FFIGenerator::load(module_name);
        let is_managed = self.is_managed();

        // Generate metatype section only if there is at least one method with `self` parameter,
        // or clone parameter is set
        let gen_metatype = attr_args.is_clone() || is_managed;

        // Type declaration
        let is_opaque = !ffi_gen.has_type_decl() && attr_args.is_opaque() && gen_metatype;

        if is_opaque {
            ffi_gen.set_type_decl_opaque();
        }

        // Class definition
        self.write_class_defs(&mut ffi_gen, module_name);

        // C Definitions
        let (max_method_name_len, max_self_method_name_len) =
            self.write_c_defs(&mut ffi_gen, module_name, is_managed);

        // Global Symbol Table
        self.write_global_sym_table(&mut ffi_gen, module_name, max_method_name_len);

        if gen_metatype && attr_args.is_clone() {
            ffi_gen.set_mt_clone();
        }

        // Metatype for class instances
        if gen_metatype {
            // Add tostring implementation if declared
            if let Some(method) = self
                .methods
                .iter()
                .find(|method| method.bind_args.is_to_string())
            {
                ffi_gen.set_to_string_method(&method.as_ffi_name());
            }

            self.write_metatype(
                &mut ffi_gen,
                module_name,
                max_self_method_name_len,
                attr_args,
            );
        }

        ffi_gen.generate(attr_args.gen_dir(), attr_args.meta_dir());
    }

    fn write_class_defs(&self, ffi_gen: &mut FFIGenerator, module_name: &str) {
        if !ffi_gen.has_class_definitions() {
            ffi_gen.add_class_definition("---@meta\n".to_string());

            self.doc
                .iter()
                .for_each(|d| ffi_gen.add_class_definition(format!("-- {d}")));
            ffi_gen.add_class_definition(format!("---@class {module_name}"));
            ffi_gen.add_class_definition(format!("{module_name} = {{}}\n"));
        }

        self.methods
            .iter()
            .filter(|method| method.bind_args.gen_lua_ffi())
            .for_each(|method| {
                let (directives, docs): (_, Vec<_>) = method.doc.iter().partition(|d| {
                    if d.len() > 1 {
                        let mut chars = d.chars();

                        chars.next().unwrap() == '@' && chars.next().unwrap().is_alphabetic()
                    } else {
                        false
                    }
                });

                // Add user defined method documentation
                docs.into_iter()
                    .for_each(|d| ffi_gen.add_class_definition(format!("-- {d}")));

                let has_overload = directives.iter().any(|d| d.contains("@overload"));
                let mut params: Vec<_> = method
                    .params
                    .iter()
                    .flat_map(|param| {
                        let ffi_name = param.as_ffi_name();
                        let mut params = vec![ffi_name.clone()];

                        // If this is a slice or array, we need to additionally generate a "size" parameter.
                        match &param.ty {
                            TypeInfo::Slice { .. } | TypeInfo::Array { .. } => {
                                params.push(format!("{}_size", ffi_name))
                            }
                            _ => {}
                        }
                        params
                    })
                    .collect();

                if !has_overload {
                    // Add method signature documentation
                    method.params.iter().for_each(|param| {
                        ffi_gen.add_class_definition(format!(
                            "---@param {} {}",
                            param.as_ffi_name(),
                            param.ty.get_luals_annotation(module_name)
                        ));

                        // If this is a slice or array, we need to additionally generate a "size" parameter.
                        match &param.ty {
                            TypeInfo::Slice { .. } | TypeInfo::Array { .. } => {
                                ffi_gen.add_class_definition(format!(
                                    "---@param {}_size {}",
                                    param.as_ffi_name(),
                                    TypeVariant::USize.get_luals_annotation(module_name)
                                ));
                            }
                            _ => {}
                        }
                    });

                    if let Some(ret) = &method.ret {
                        if method.bind_args.gen_out_param() {
                            ffi_gen.add_class_definition(format!(
                                "---@param result {} [out]",
                                ret.get_luals_annotation(module_name)
                            ));

                            params.push("result".into());
                        } else {
                            ffi_gen.add_class_definition(format!(
                                "---@return {}",
                                ret.get_luals_annotation(module_name)
                            ));
                        }
                    }
                }

                // Add user defined Lua LSP directives
                directives
                    .into_iter()
                    .for_each(|d| ffi_gen.add_class_definition(format!("---{d}")));

                if method.self_param.is_some() {
                    ffi_gen.add_class_definition(format!(
                        "function {module_name}:{}({}) end\n",
                        method.as_ffi_var(),
                        params.join(", ")
                    ));
                } else {
                    ffi_gen.add_class_definition(format!(
                        "function {module_name}.{}({}) end\n",
                        method.as_ffi_name(),
                        params.join(", ")
                    ));
                }
            });
    }

    fn write_c_defs(
        &self,
        ffi_gen: &mut FFIGenerator,
        module_name: &str,
        is_managed: bool,
    ) -> (usize, usize) {
        if ffi_gen.has_c_definitions() {
            ffi_gen.add_c_definition("");
        }

        // For managed types, we add 'void Free' method
        let mut max_method_name_len = if is_managed { "void".len() } else { 0 };
        let mut max_self_method_name_len = max_method_name_len;
        let mut max_ret_len = if is_managed { "Free".len() } else { 0 };

        // Calculate max len of method return parameters and method names to use them in formatting
        self.methods
            .iter()
            .filter(|method| method.bind_args.gen_lua_ffi())
            .for_each(|method| {
                let len = if method.bind_args.gen_out_param() || method.ret.is_none() {
                    "void".len()
                } else {
                    let ret = method.ret.as_ref().unwrap();
                    ret.as_ffi(module_name).1.len()
                };

                max_ret_len = std::cmp::max(max_ret_len, len);
                max_method_name_len =
                    std::cmp::max(max_method_name_len, method.as_ffi_name().len());

                if method.self_param.is_some() {
                    max_self_method_name_len =
                        std::cmp::max(max_self_method_name_len, method.as_ffi_name().len());
                }
            });

        if is_managed {
            ffi_gen.add_c_definition(format!(
                "{IDENT}{IDENT}{IDENT}{:<2$} {module_name}_{:<3$} ({module_name}*);",
                "void", "Free", max_ret_len, max_method_name_len
            ));
        }

        self
            .methods
            .iter()
            .filter(|method| method.bind_args.gen_lua_ffi())
            .for_each(|method| {
                let method_name = method.as_ffi_name();

                let ret_ty_str =  if method.bind_args.gen_out_param() || method.ret.is_none() {
                    "void".into()
                } else {
                    let ret = method.ret.as_ref().unwrap();
                    ret.as_ffi(module_name).1
                };

                let mut params_str: Vec<_> = method
                    .params
                    .iter()
                    .flat_map(|param| self.get_c_ffi_param(module_name, param))
                    .collect();

                if method.bind_args.gen_out_param() && method.ret.is_some() {
                    let ret = method.ret.as_ref().unwrap();
                    let ret_ffi = ret.as_ffi(module_name).1;
                    let ret_param = match &ret {
                        TypeInfo::Plain { is_ref, ty } => {
                            match ty {
                                TypeVariant::Custom(_) => {
                                    if !ty.is_copyable(&self.name) && *is_ref == TypeRef::Value {
                                        // If we have a non-copyable type that's not boxed, optional or a ref,
                                        // we don't need to return it as a pointer as it's already a pointer.
                                        format!("{} out", ret_ffi)
                                    } else {
                                        format!("{}* out", ret_ffi)
                                    }
                                },
                                _ => {
                                    format!("{}* out", ret_ffi)
                                }
                            }
                        }
                        _ => {
                            format!("{}* out", ret_ffi)
                        }
                    };
                    params_str.push(ret_param);
                }

                let self_str = if let Some(self_type) = &method.self_param {
                    let const_str = if !self_type.is_mutable { " const" } else { "" };

                    if params_str.is_empty() {
                        format!("{module_name}{const_str}*")
                    } else {
                        format!("{module_name}{const_str}*, ")
                    }
                } else {
                    "".into()
                };

                ffi_gen.add_c_definition(format!(
                    "{IDENT}{IDENT}{IDENT}{ret_ty_str:<1$} {module_name}_{method_name:<2$} ({self_str}{});",
                    params_str.join(", "),
                    max_ret_len,
                    max_method_name_len
                )
                );
            });

        // Return max len of the method names (all and self only) to avoid recalculation in the next step
        (max_method_name_len, max_self_method_name_len)
    }

    fn get_c_ffi_param(&self, module_name: &str, param: &ParamInfo) -> Vec<String> {
        if let TypeInfo::Function { .. } = &param.ty {
            // Function pointers in C have the param name inside the type signature, just get
            // the type without the name.
            return vec![param.ty.as_ffi(module_name).1];
        }

        let mut params = vec![format!(
            "{} {}",
            param.ty.as_ffi(module_name).1,
            param.as_ffi_name()
        )];

        // If this is a slice or array, we need to additionally generate a "size" parameter.
        match &param.ty {
            TypeInfo::Slice { .. } | TypeInfo::Array { .. } => params.push(format!(
                "{} {}_size",
                TypeVariant::USize.as_ffi(module_name).1,
                param.as_ffi_name()
            )),
            _ => {}
        }

        params
    }

    fn write_global_sym_table(
        &self,
        ffi_gen: &mut FFIGenerator,
        module_name: &str,
        max_method_name_len: usize,
    ) {
        if ffi_gen.has_global_symbols() {
            ffi_gen.add_global_symbol("");
        }

        self.methods
            .iter()
            .filter(|method| method.bind_args.gen_lua_ffi() && method.self_param.is_none())
            .for_each(|method| {
                write_method_map(
                    &format!("{IDENT}{IDENT}{IDENT}"),
                    &format!("{:<1$}", method.as_ffi_name(), max_method_name_len),
                    method,
                    module_name,
                    |value| ffi_gen.add_global_symbol(value),
                );
            });
    }

    fn write_metatype(
        &self,
        ffi_gen: &mut FFIGenerator,
        module_name: &str,
        max_self_method_name_len: usize,
        attr_args: &ImplAttrArgs,
    ) {
        let max_method_name_len = if attr_args.is_clone() {
            std::cmp::max(max_self_method_name_len, "clone".len())
        } else {
            max_self_method_name_len
        };

        // Add clone method if requested
        if attr_args.is_clone() {
            ffi_gen.add_metatype(format!(
                "{IDENT}{IDENT}{IDENT}{IDENT}{0:<1$} = function(x) return {module_name}_t(x) end,",
                "clone", max_method_name_len
            ));
        }

        self.methods
            .iter()
            .filter(|method| method.bind_args.gen_lua_ffi() && method.self_param.is_some())
            .for_each(|method| {
                write_method_map(
                    &format!("{IDENT}{IDENT}{IDENT}{IDENT}"),
                    &format!("{:<1$}", method.as_ffi_var(), max_method_name_len),
                    method,
                    module_name,
                    |value| ffi_gen.add_metatype(value),
                );
            });
    }
}

fn write_method_map<F: FnMut(String)>(
    ident: &str,
    mapped_method: &str,
    method: &super::MethodInfo,
    module_name: &str,
    mut writer: F,
) {
    let mut args = vec![];
    // list of the non-copyable arguments sent by value, to be removed from Lua GC
    let mut value_args = vec![];

    if method.self_param.is_some() {
        args.push("self".to_string());
    }

    method.params.iter().for_each(|param_info| {
        args.push(param_info.as_ffi_name());
        if let TypeInfo::Plain { is_ref, ty } = &param_info.ty {
            if *is_ref == TypeRef::Value && !ty.is_copyable(module_name) {
                value_args.push(param_info.as_ffi_name());
            }
        };
    });

    let args_str = args.join(", ");

    // TODO: refactor these nested ifs
    // Here, we want to package the return type in ffi.gc if we're returning a managed type by value.
    let gc_type = if !method.bind_args.gen_out_param() {
        if let Some(ret) = &method.ret {
            let ret_custom = match ret {
                TypeInfo::Plain { is_ref, ty } => {
                    if *is_ref == TypeRef::Value {
                        Some(ty)
                    } else {
                        None
                    }
                }
                TypeInfo::Option { is_ref, inner_ty } => {
                    if *is_ref == TypeRef::Value {
                        Some(inner_ty)
                    } else {
                        None
                    }
                }
                TypeInfo::Box { inner_ty } => Some(inner_ty),
                _ => None,
            };
            ret_custom
                .and_then(|ty| ty.get_managed_type())
                .map(|gc_type| {
                    if gc_type == "Self" {
                        module_name
                    } else {
                        gc_type
                    }
                })
        } else {
            None
        }
    } else {
        None
    };

    if let Some(gc_type) = gc_type {
        writer(format!("{ident}{mapped_method} = function({args_str})"));

        // remove non-copyable arguments sent by value from Lua GC
        for arg in value_args {
            writer(format!("{ident}{IDENT}local {arg} = ffi.gc({arg}, nil)"));
        }

        writer(format!(
            "{ident}{IDENT}local _instance = libphx.{module_name}_{}({args_str})",
            method.as_ffi_name(),
        ));
        writer(format!(
            "{ident}{IDENT}return Core.ManagedObject(_instance, libphx.{gc_type}_Free)"
        ));
        writer(format!("{ident}end,"));
    } else if !value_args.is_empty() {
        writer(format!("{ident}{mapped_method} = function({args_str})"));

        // remove non-copyable arguments sent by value from Lua GC
        for arg in value_args {
            writer(format!("{ident}{IDENT}local {arg} = ffi.gc({arg}, nil)"));
        }

        if method.ret.is_some() {
            writer(format!(
                "{ident}{IDENT}return libphx.{module_name}_{}({args_str})",
                method.as_ffi_name(),
            ));
        } else {
            writer(format!(
                "{ident}{IDENT}libphx.{module_name}_{}({args_str})",
                method.as_ffi_name(),
            ));
        }

        writer(format!("{ident}end,"));
    } else {
        writer(format!(
            "{ident}{mapped_method} = libphx.{module_name}_{},",
            method.as_ffi_name(),
        ));
    }
}
