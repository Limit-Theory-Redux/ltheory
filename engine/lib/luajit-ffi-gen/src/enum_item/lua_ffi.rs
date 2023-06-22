use std::io::Write;
use std::{env::VarError, fs::File, path::PathBuf};

use crate::{args::EnumAttrArgs, IDENT, LUAJIT_FFI_GEN_DIR, LUAJIT_FFI_GEN_DIR_ENV};

use super::EnumInfo;

impl EnumInfo {
    /// Generate Lua FFI file
    pub fn generate_ffi(&self, attr_args: &EnumAttrArgs) {
        let module_name = attr_args.name().unwrap_or(self.name.clone());
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
        let variant_names = self.variants.get_names();

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

        let max_ret_len = std::cmp::max("cstr".len(), module_name.len());

        variant_names.iter().for_each(|v| {
            writeln!(
                &mut file,
                "{IDENT}{IDENT}{module_name:<0$} {module_name}_{v};",
                max_ret_len
            )
            .unwrap();
        });
        writeln!(
            &mut file,
            "{IDENT}{IDENT}{0:<1$} {module_name}_ToString({module_name});",
            "cstr", max_ret_len
        )
        .unwrap();

        writeln!(&mut file, "{IDENT}]]").unwrap();
        writeln!(&mut file, "end\n").unwrap();

        // Global Symbol Table
        writeln!(&mut file, "do -- Global Symbol Table").unwrap();
        writeln!(&mut file, "{IDENT}{module_name} = {{").unwrap();

        let max_variant_len = variant_names
            .iter()
            .map(|name| name.len())
            .max()
            .unwrap_or(0);
        let max_variant_len = std::cmp::max(max_variant_len, "ToString".len());

        variant_names.iter().for_each(|v| {
            writeln!(
                &mut file,
                "{IDENT}{IDENT}{v:<0$} = libphx.{module_name}_{v},",
                max_variant_len
            )
            .unwrap();
        });
        writeln!(
            &mut file,
            "{IDENT}{IDENT}ToString = libphx.{module_name}_ToString,"
        )
        .unwrap();

        writeln!(&mut file, "{IDENT}}}\n").unwrap();

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
}
