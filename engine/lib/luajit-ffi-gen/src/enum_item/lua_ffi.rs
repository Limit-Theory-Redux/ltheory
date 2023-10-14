use crate::ffi_generator::FfiGenerator;
use crate::impl_item::TypeVariant;
use crate::{args::EnumAttrArgs, IDENT};

use super::EnumInfo;

impl EnumInfo {
    /// Generate Lua FFI file
    pub fn generate_ffi(&self, attr_args: &EnumAttrArgs, repr_type: &str) {
        let module_name = attr_args.name().unwrap_or(self.name.clone());
        let enum_repr_ty = TypeVariant::from_str(repr_type).unwrap_or(TypeVariant::U32);
        let variant_names = self.variants.get_names();

        let mut ffi_gen = FfiGenerator::new(&module_name);

        ffi_gen.set_type_decl_struct(enum_repr_ty.as_ffi_string());

        gen_c_definitions(&mut ffi_gen, &module_name, &variant_names);
        gen_global_symbol_table(&mut ffi_gen, &module_name, &variant_names);

        if attr_args.with_impl() {
            ffi_gen.save(&module_name);

            return;
        }

        ffi_gen.generate();
    }
}

fn gen_c_definitions(ffi_gen: &mut FfiGenerator, module_name: &str, variant_names: &[&str]) {
    let max_ret_len = std::cmp::max("cstr".len(), module_name.len());

    variant_names.iter().for_each(|v| {
        ffi_gen.add_c_definition(format!(
            "{IDENT}{IDENT}{IDENT}{module_name:<0$} {module_name}_{v};",
            max_ret_len
        ));
    });

    ffi_gen.add_c_definition("");

    ffi_gen.add_c_definition(format!(
        "{IDENT}{IDENT}{IDENT}{0:<1$} {module_name}_ToString({module_name});",
        "cstr", max_ret_len
    ));
}

fn gen_global_symbol_table(ffi_gen: &mut FfiGenerator, module_name: &str, variant_names: &[&str]) {
    let max_variant_len = variant_names
        .iter()
        .map(|name| name.len())
        .max()
        .unwrap_or(0);
    let max_variant_len = std::cmp::max(max_variant_len, "ToString".len());

    variant_names.iter().for_each(|v| {
        ffi_gen.add_global_symbol(format!(
            "{IDENT}{IDENT}{IDENT}{v:<0$} = libphx.{module_name}_{v},",
            max_variant_len
        ));
    });

    ffi_gen.add_global_symbol("");

    ffi_gen.add_global_symbol(format!(
        "{IDENT}{IDENT}{IDENT}{0:<1$} = libphx.{module_name}_ToString,",
        "ToString", max_variant_len
    ));
}
