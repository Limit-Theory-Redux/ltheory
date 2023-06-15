use std::{env::VarError, fs::File, io::Write, path::PathBuf};

use crate::{args::AttrArgs, impl_info::ImplInfo};

const LUAJIT_FFI_GEN_DIR_ENV: &str = "LUAJIT_FFI_GEN_DIR";
const LUAJIT_FFI_GEN_DIR: &str = "../phx/script/ffi";
const IDENT: &str = "    ";

/// Generate Lua FFI file
pub fn generate_ffi(attr_args: &AttrArgs, impl_info: &ImplInfo) {
    let module_name = attr_args.name().unwrap_or(impl_info.name.clone());
    let luajit_ffi_gen_dir = match std::env::var(LUAJIT_FFI_GEN_DIR_ENV) {
        Ok(var) => {
            if !var.is_empty() {
                var
            } else {
                LUAJIT_FFI_GEN_DIR.into()
            }
        }
        Err(VarError::NotPresent) => LUAJIT_FFI_GEN_DIR.into(),
        Err(err) => {
            println!("Cannot read '{LUAJIT_FFI_GEN_DIR_ENV}' environment variable. Use default value: {LUAJIT_FFI_GEN_DIR}. Error: {err}");

            LUAJIT_FFI_GEN_DIR.into()
        }
    };

    let cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let luajit_ffi_gen_dir_path = cargo_manifest_dir.join(&luajit_ffi_gen_dir);
    assert!(
        luajit_ffi_gen_dir_path.exists(),
        "FFI directory '{luajit_ffi_gen_dir_path:?}' doesn't exist"
    );

    let luajit_ffi_module_path = luajit_ffi_gen_dir_path.join(format!("{module_name}.lua"));
    let mut file = File::create(&luajit_ffi_module_path).expect(&format!(
        "Cannot create file: {luajit_ffi_module_path:?}\nCurrent folder: {:?}",
        std::env::current_dir()
    ));

    // Generate metatype section only if there is at least one method with `self` parameter
    // or managed parameter is set
    let gen_metatype = attr_args.is_managed()
        || impl_info
            .methods
            .iter()
            .any(|method| method.self_param.is_some());

    // Header
    writeln!(
        &mut file,
        "-- {module_name} {:-<1$}",
        "-",
        80 - 4 - module_name.len()
    )
    .unwrap();
    writeln!(&mut file, "local ffi = require('ffi')").unwrap();
    writeln!(&mut file, "local libphx = require('ffi.libphx').lib").unwrap();
    writeln!(&mut file, "local {module_name}\n").unwrap();

    // C Definitions
    writeln!(&mut file, "do -- C Definitions").unwrap();
    writeln!(&mut file, "{IDENT}ffi.cdef [[").unwrap();

    let max_method_name_len =
        write_c_defs(&mut file, &module_name, impl_info, attr_args.is_managed());

    writeln!(&mut file, "{IDENT}]]").unwrap();
    writeln!(&mut file, "end\n").unwrap();

    // Global Symbol Table
    writeln!(&mut file, "do -- Global Symbol Table").unwrap();
    writeln!(&mut file, "{IDENT}{module_name} = {{").unwrap();

    write_global_sym_table(
        &mut file,
        &module_name,
        impl_info,
        max_method_name_len,
        attr_args.is_managed(),
    );

    writeln!(&mut file, "{IDENT}}}\n").unwrap();

    if gen_metatype && attr_args.is_clone() {
        writeln!(&mut file, "{IDENT}local mt = {{").unwrap();
        writeln!(
            &mut file,
            "{IDENT}{IDENT}__call = function(t, ...) return {module_name}_t(...) end,"
        )
        .unwrap();
        writeln!(&mut file, "{IDENT}}}\n").unwrap();
    }

    writeln!(
        &mut file,
        "{IDENT}if onDef_{module_name} then onDef_{module_name}({module_name}, mt) end"
    )
    .unwrap();
    writeln!(
        &mut file,
        "{IDENT}{module_name} = setmetatable({module_name}, mt)"
    )
    .unwrap();
    writeln!(&mut file, "end\n").unwrap();

    // Metatype for class instances
    if gen_metatype {
        writeln!(&mut file, "do -- Metatype for class instances").unwrap();
        writeln!(&mut file, "{IDENT}local t  = ffi.typeof('{module_name}')").unwrap();
        writeln!(&mut file, "{IDENT}local mt = {{").unwrap();

        // Add tostring implementation if declared
        if let Some(method) = impl_info
            .methods
            .iter()
            .find(|method| method.bind_args.is_to_string())
        {
            writeln!(
                &mut file,
                "{IDENT}{IDENT}__tostring = function(self) return ffi.string(libphx.{module_name}_{}(self)) end,",
                method.as_ffi_name()
            )
            .unwrap();
        }

        writeln!(&mut file, "{IDENT}{IDENT}__index = {{").unwrap();

        write_metatype(
            &mut file,
            &module_name,
            impl_info,
            max_method_name_len,
            attr_args,
        );

        writeln!(&mut file, "{IDENT}{IDENT}}},").unwrap();
        writeln!(&mut file, "{IDENT}}}\n").unwrap();

        writeln!(
            &mut file,
            "{IDENT}if onDef_{module_name}_t then onDef_{module_name}_t(t, mt) end"
        )
        .unwrap();
        writeln!(&mut file, "{IDENT}{module_name}_t = ffi.metatype(t, mt)").unwrap();
        writeln!(&mut file, "end\n").unwrap();
    }

    writeln!(&mut file, "return {module_name}").unwrap();
}

fn write_c_defs(
    file: &mut File,
    module_name: &str,
    impl_info: &ImplInfo,
    is_managed: bool,
) -> usize {
    let mut max_method_name_len = if is_managed { "void".len() } else { 0 };
    let mut max_ret_len = if is_managed { "Free".len() } else { 0 };

    // Calculate max len of method return parameters and method names to use them in formatting
    impl_info.methods.iter().for_each(|method| {
        let len = method
            .ret
            .as_ref()
            .map(|ret| {
                if ret.is_self() {
                    format!("{module_name}*")
                } else {
                    ret.as_ffi_string()
                }
                .len()
            })
            .unwrap_or("void".len());

        max_ret_len = std::cmp::max(max_ret_len, len);
        max_method_name_len = std::cmp::max(max_method_name_len, method.as_ffi_name().len());
    });

    if is_managed {
        writeln!(
            file,
            "{IDENT}{IDENT}{:<2$} {module_name}_{:<3$} ({module_name}*);",
            "void", "Free", max_ret_len, max_method_name_len
        )
        .unwrap();
    }

    impl_info.methods.iter().for_each(|method| {
        let method_name = method.as_ffi_name();
        let ret_ty_str = method
            .ret
            .as_ref()
            .map(|ret| {
                if ret.is_self() {
                    format!("{module_name}*")
                } else {
                    ret.as_ffi_string()
                }
            })
            .unwrap_or("void".into());

        let params_str: Vec<_> = method
            .params
            .iter()
            .map(|param| format!("{} {}", param.ty.as_ffi_string(), param.as_ffi_name()))
            .collect();

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

        writeln!(
            file,
            "{IDENT}{IDENT}{ret_ty_str:<1$} {module_name}_{method_name:<2$} ({self_str}{});",
            params_str.join(", "),
            max_ret_len,
            max_method_name_len
        )
        .unwrap();
    });

    // Return max len of the method names to avoid recalculation in the next step
    max_method_name_len
}

fn write_global_sym_table(
    file: &mut File,
    module_name: &str,
    impl_info: &ImplInfo,
    max_method_name_len: usize,
    is_managed: bool,
) {
    if is_managed {
        writeln!(
            file,
            "{IDENT}{IDENT}{0:<1$} = libphx.{module_name}_{0},",
            "Free", max_method_name_len
        )
        .unwrap();
    }

    impl_info.methods.iter().for_each(|method| {
        writeln!(
            file,
            "{IDENT}{IDENT}{0:<1$} = libphx.{module_name}_{0},",
            method.as_ffi_name(),
            max_method_name_len
        )
        .unwrap();
    });
}

fn write_metatype(
    file: &mut File,
    module_name: &str,
    impl_info: &ImplInfo,
    max_method_name_len: usize,
    attr_args: &AttrArgs,
) {
    let max_method_name_len = if attr_args.is_managed() {
        std::cmp::max(max_method_name_len, "managed".len())
    } else if attr_args.is_clone() {
        std::cmp::max(max_method_name_len, "clone".len())
    } else {
        max_method_name_len
    };

    // Add clone method if requested
    if attr_args.is_clone() {
        writeln!(
            file,
            "{IDENT}{IDENT}{IDENT}{0:<1$} = function(x) return {module_name}_t(x) end,",
            "clone", max_method_name_len
        )
        .unwrap();
    }

    // Add managed method if requested
    if attr_args.is_managed() {
        writeln!(
            file,
            "{IDENT}{IDENT}{IDENT}{0:<1$} = function(self) return ffi.gc(self, libphx.{module_name}_Free) end,",
            "managed", max_method_name_len
        )
        .unwrap();

        writeln!(
            file,
            "{IDENT}{IDENT}{IDENT}{0:<1$} = libphx.{module_name}_Free,",
            "free", max_method_name_len
        )
        .unwrap();
    }

    impl_info
        .methods
        .iter()
        .filter(|method| method.self_param.is_some())
        .for_each(|method| {
            writeln!(
                file,
                "{IDENT}{IDENT}{IDENT}{:<2$} = libphx.{module_name}_{},",
                method.as_ffi_var(),
                method.as_ffi_name(),
                max_method_name_len
            )
            .unwrap();
        });
}
