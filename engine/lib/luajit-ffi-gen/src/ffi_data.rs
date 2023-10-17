use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FfiData {
    pub has_typedef: bool,
    pub c_definitions: Vec<String>,
    pub global_symbol_table: Vec<String>,
}

impl FfiData {
    pub fn load(module_name: &str) -> Option<Self> {
        let target_ffi_file = Self::ffi_file(module_name);

        if !target_ffi_file.exists() {
            return None;
        }

        let data = std::fs::read_to_string(&target_ffi_file)
            .expect(&format!("Cannot load {target_ffi_file:?} FFI data file"));

        let res = serde_json::from_str(&data)
            .expect(&format!("Cannot parse {target_ffi_file:?} FFI data file"));

        std::fs::remove_file(&target_ffi_file)
            .expect(&format!("Cannot remove {target_ffi_file:?} FFI data file"));

        Some(res)
    }

    pub fn save(&self, module_name: &str) {
        let data =
            serde_json::to_string(self).expect(&format!("Cannot serialize {module_name} data"));

        let target_ffi_dir = Self::ffi_dir();

        std::fs::create_dir_all(&target_ffi_dir)
            .expect(&format!("Cannot create {target_ffi_dir:?} folder"));

        let target_ffi_file = Self::ffi_file(module_name);

        std::fs::write(&target_ffi_file, data)
            .expect(&format!("Cannot save {target_ffi_file:?} FFI data file"));
    }

    fn ffi_dir() -> PathBuf {
        // TODO: env!("OUT_DIR") doesn't work
        PathBuf::new().join("target").join("ffi")
    }

    fn ffi_file(module_name: &str) -> PathBuf {
        Self::ffi_dir().join(format!("{module_name}.json"))
    }
}
