use std::io::Write;
use std::{env::VarError, fs::File, path::PathBuf};

use crate::ffi_data::FfiData;
use crate::impl_item::TypeVariant;
use crate::{args::EnumAttrArgs, IDENT, LUAJIT_FFI_GEN_DIR, LUAJIT_FFI_GEN_DIR_ENV};

use super::EnumInfo;

impl EnumInfo {
    /// Generate Lua FFI file
    pub fn generate_ffi(&self, attr_args: &EnumAttrArgs, repr_type: &str) {
        let module_name = attr_args.name().unwrap_or(self.name.clone());
        let enum_repr_ty = TypeVariant::from_str(repr_type).unwrap_or(TypeVariant::U32);
        let variant_names = self.variants.get_names();
        let c_definitions = gen_c_definitions(&module_name, &enum_repr_ty, &variant_names);
        let global_symbol_table = gen_global_symbol_table(&module_name, &variant_names);

        if attr_args.with_impl() {
            let ffi_data = FfiData {
                has_typedef: true,
                c_definitions,
                global_symbol_table,
            };

            ffi_data.save(&module_name);

            return;
        }

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

        c_definitions
            .iter()
            .for_each(|def| writeln!(&mut file, "{def}").unwrap());

        writeln!(&mut file, "{IDENT}]]").unwrap();
        writeln!(&mut file, "end\n").unwrap();

        // Global Symbol Table
        writeln!(&mut file, "do -- Global Symbol Table").unwrap();
        writeln!(&mut file, "{IDENT}{module_name} = {{").unwrap();

        global_symbol_table
            .iter()
            .for_each(|def| writeln!(&mut file, "{def}").unwrap());

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

fn gen_c_definitions(
    module_name: &str,
    _enum_repr_ty: &TypeVariant,
    variant_names: &[&str],
) -> Vec<String> {
    let mut res = vec![];

    // TODO: refactor the way generated FFI is processed so typedef is registered before all other FFI parts to prevent a problem with unknown types
    // res.push(format!(
    //     "{IDENT}{IDENT}typedef {} {module_name};\n",
    //     enum_repr_ty.as_ffi_string()
    // ));

    let max_ret_len = std::cmp::max("cstr".len(), module_name.len());

    variant_names.iter().for_each(|v| {
        res.push(format!(
            "{IDENT}{IDENT}{module_name:<0$} {module_name}_{v};",
            max_ret_len
        ));
    });

    res.push("".into());

    res.push(format!(
        "{IDENT}{IDENT}{0:<1$} {module_name}_ToString({module_name});",
        "cstr", max_ret_len
    ));

    res
}

fn gen_global_symbol_table(module_name: &str, variant_names: &[&str]) -> Vec<String> {
    let mut res = vec![];

    let max_variant_len = variant_names
        .iter()
        .map(|name| name.len())
        .max()
        .unwrap_or(0);
    let max_variant_len = std::cmp::max(max_variant_len, "ToString".len());

    variant_names.iter().for_each(|v| {
        res.push(format!(
            "{IDENT}{IDENT}{v:<0$} = libphx.{module_name}_{v},",
            max_variant_len
        ));
    });

    res.push("".into());

    res.push(format!(
        "{IDENT}{IDENT}{0:<1$} = libphx.{module_name}_ToString,",
        "ToString", max_variant_len
    ));

    res
}
