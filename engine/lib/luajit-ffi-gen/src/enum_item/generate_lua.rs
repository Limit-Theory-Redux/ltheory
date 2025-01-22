use super::EnumInfo;
use crate::args::EnumAttrArgs;
use crate::ffi_generator::FFIGenerator;
use crate::impl_item::TypeVariant;
use crate::IDENT;

impl EnumInfo {
    /// Generate Lua FFI file
    pub fn gen_lua_ffi(&self, attr_args: &EnumAttrArgs, repr_type: &str) {
        let module_name = attr_args.name().unwrap_or(self.name.as_str());
        let enum_repr_ty = TypeVariant::from_rust_ffi_str(repr_type).unwrap_or(TypeVariant::U32);
        let variants_info = self.variants.get_info(attr_args.start_index());

        let mut ffi_gen = FFIGenerator::new(module_name);

        ffi_gen.set_type_decl_struct(enum_repr_ty.as_ffi(module_name).1);

        gen_class_definitions(&mut ffi_gen, &self.doc, module_name, &variants_info);
        gen_c_definitions(&mut ffi_gen, module_name, &variants_info);
        gen_global_symbol_table(&mut ffi_gen, module_name, &variants_info);

        if attr_args.with_impl() {
            ffi_gen.save();

            return;
        }

        ffi_gen.generate(attr_args.gen_dir(), attr_args.meta_dir());
    }
}

fn gen_class_definitions(
    ffi_gen: &mut FFIGenerator,
    doc: &[String],
    module_name: &str,
    variants_info: &[(&[String], &str, u64)],
) {
    ffi_gen.add_class_definition("---@meta\n".to_string());

    doc.iter()
        .for_each(|d| ffi_gen.add_class_definition(format!("-- {d}")));
    // use @class instead of @enum to make it easier to work with enums on the Lua side using LSP
    ffi_gen.add_class_definition(format!("---@class {module_name}"));

    variants_info.iter().for_each(|(docs, name, _)| {
        ffi_gen.add_class_definition(format!("---@field {name} integer {}", docs.join(" ")));
    });

    ffi_gen.add_class_definition(format!("{module_name} = {{"));

    variants_info.iter().for_each(|(docs, name, index)| {
        docs.iter()
            .for_each(|doc| ffi_gen.add_class_definition(format!("{IDENT}-- {doc}")));
        ffi_gen.add_class_definition(format!("{IDENT}{name} = {index},"));
    });

    ffi_gen.add_class_definition("}\n");
}

fn gen_c_definitions(
    ffi_gen: &mut FFIGenerator,
    module_name: &str,
    variants_info: &[(&[String], &str, u64)],
) {
    let max_ret_len = std::cmp::max("cstr".len(), module_name.len());

    variants_info.iter().for_each(|(_, name, _)| {
        ffi_gen.add_c_definition(format!(
            "{IDENT}{IDENT}{IDENT}{module_name:<0$} {module_name}_{name};",
            max_ret_len
        ));
    });

    ffi_gen.add_c_definition("");

    ffi_gen.add_c_definition(format!(
        "{IDENT}{IDENT}{IDENT}{0:<1$} {module_name}_ToString({module_name});",
        "cstr", max_ret_len
    ));
}

fn gen_global_symbol_table(
    ffi_gen: &mut FFIGenerator,
    module_name: &str,
    variants_info: &[(&[String], &str, u64)],
) {
    let max_variant_len = variants_info
        .iter()
        .map(|(_, name, _)| name.len())
        .max()
        .unwrap_or(0);
    let max_variant_len = std::cmp::max(max_variant_len, "ToString".len());

    variants_info.iter().for_each(|(_, name, _)| {
        ffi_gen.add_global_symbol(format!(
            "{IDENT}{IDENT}{IDENT}{name:<0$} = libphx.{module_name}_{name},",
            max_variant_len
        ));
    });

    ffi_gen.add_global_symbol("");

    ffi_gen.add_global_symbol(format!(
        "{IDENT}{IDENT}{IDENT}{0:<1$} = libphx.{module_name}_ToString,",
        "ToString", max_variant_len
    ));
}
