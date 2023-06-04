use std::{env::VarError, fs::File, io::Write, path::PathBuf};

use crate::impl_info::ImplInfo;

const LUAJIT_FFI_GEN_DIR_ENV: &str = "LUAJIT_FFI_GEN_DIR";
const LUAJIT_FFI_GEN_DIR: &str = "../script/ffi";
// TODO: change to 4 spaces after Lua code refactoring
const IDENT: &str = "  ";

pub fn generate_ffi(module_name: &str, impl_info: &ImplInfo) {
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

    let max_method_name_len = write_c_defs(&mut file, module_name, impl_info);

    writeln!(&mut file, "{IDENT}]]").unwrap();
    writeln!(&mut file, "end\n").unwrap();

    // Global Symbol Table
    writeln!(&mut file, "do -- Global Symbol Table").unwrap();
    writeln!(&mut file, "{IDENT}{module_name} = {{").unwrap();

    write_global_sym_table(&mut file, module_name, impl_info, max_method_name_len);

    writeln!(&mut file, "{IDENT}}}\n").unwrap();

    // Footer
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
    writeln!(&mut file, "return {module_name}").unwrap();
}

fn write_c_defs(file: &mut File, module_name: &str, impl_info: &ImplInfo) -> usize {
    let mut max_method_name_len = 0;
    let mut max_ret_len = 0;

    impl_info.methods.iter().for_each(|method| {
        let len = method
            .ret
            .as_ref()
            .map(|ret| ret.variant.as_ffi_string().len())
            .unwrap_or("void".len());

        max_ret_len = std::cmp::max(max_ret_len, len);
        max_method_name_len = std::cmp::max(max_method_name_len, method.as_ffi_name().len());
    });

    impl_info.methods.iter().for_each(|method| {
        let ret_ty_str = method
            .ret
            .as_ref()
            .map(|ret| ret.variant.as_ffi_string())
            .unwrap_or("void".into());

        let params_str: Vec<_> = method
            .params
            .iter()
            .map(|param| {
                format!(
                    "{} {}",
                    param.ty.variant.as_ffi_string(),
                    param.as_ffi_name()
                )
            })
            .collect();

        writeln!(
            file,
            "{IDENT}{IDENT}{ret_ty_str:<2$} {module_name}_{:<3$} ({});",
            method.as_ffi_name(),
            params_str.join(", "),
            max_ret_len,
            max_method_name_len
        )
        .unwrap();
    });

    max_method_name_len
}

fn write_global_sym_table(
    file: &mut File,
    module_name: &str,
    impl_info: &ImplInfo,
    max_method_name_len: usize,
) {
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
