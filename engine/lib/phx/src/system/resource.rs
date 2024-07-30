use std::path::PathBuf;

use super::*;

const RESOURCE_EXTENSION_INFO: &[(&str, &[&str])] = &[
    ("font", &["ttf", "otf"]),                // ResourceType::Font
    ("mesh", &["bin", "obj"]),                // ResourceType::Mesh
    ("script", &["lua"]),                     // ResourceType::Script
    ("shader", &["glsl"]),                    // ResourceType::Shader
    ("sound", &["mp3", "ogg", "ogx", "wav"]), // ResourceType::Sound
    ("tex1d", &["bin"]),                      // ResourceType::Tex1D
    ("tex2d", &["jpg", "png"]),               // ResourceType::Tex2D
    ("tex3d", &["bin"]),                      // ResourceType::Tex3D
    ("texcube", &[]),                         // ResourceType::TexCube
    ("theme", &["yaml"]),                     // ResourceType::Theme
    ("", &[]),                                // ResourceType::Other
];
const _: () = assert!(
    RESOURCE_EXTENSION_INFO.len() == ResourceType::SIZE,
    "Size of RESOURCE_EXTENSION_INFO is not equal to number of ResourceType variants"
);

const RESOURCE_FOLDERS: &[&str] = &["../shared/res", "./res"];

pub struct Resource;

#[luajit_ffi_gen::luajit_ffi]
impl Resource {
    pub fn exists(ty: ResourceType, name: &str) -> bool {
        resolve_opt(ty, name).is_some()
    }

    pub fn get_path(ty: ResourceType, name: &str) -> String {
        resolve(ty, name)
    }

    pub fn load_bytes(ty: ResourceType, name: &str) -> Bytes {
        let path = resolve(ty, name);

        match std::fs::read(&path) {
            Ok(bytes) => unsafe { *Bytes_FromVec(bytes) },
            Err(err) => panic!("Cannot read file: {path}. Error: {err}"),
        }
    }

    pub fn load_string(ty: ResourceType, name: &str) -> String {
        let path = resolve(ty, name);

        match std::fs::read_to_string(&path) {
            Ok(data) => data,
            Err(err) => panic!("Cannot read file: {path}. Error: {err}"),
        }
    }
}

impl Resource {
    pub fn get_folders(ty: ResourceType) -> Vec<PathBuf> {
        let (ext_folder, _) = &RESOURCE_EXTENSION_INFO[ty.value() as usize];

        RESOURCE_FOLDERS
            .iter()
            .map(|f| PathBuf::from(f).join(ext_folder))
            .collect()
    }
}

fn resolve_opt(ty: ResourceType, name: &str) -> Option<String> {
    if PathBuf::from(name).exists() {
        return Some(name.into());
    }

    let (resource_dir, extensions) = RESOURCE_EXTENSION_INFO.get(ty as usize)?;
    let mut folders: Vec<_> = RESOURCE_FOLDERS
        .iter()
        .map(|f| PathBuf::from(f).join(resource_dir))
        .collect();
    folders.push("".into());

    if extensions.is_empty() {
        for folder in &folders {
            let path = folder.join(name);

            if path.exists() {
                return Some(path.display().to_string());
            }
        }
    } else {
        for folder in &folders {
            for extension in *extensions {
                let path = folder.join(format!("{name}.{extension}"));

                if path.exists() {
                    return Some(path.display().to_string());
                }
            }
        }
    }

    None
}

fn resolve(ty: ResourceType, name: &str) -> String {
    resolve_opt(ty, name).unwrap_or_else(|| {
        panic!(
            "Resource resolve: Failed to find {ty:?} <{name}>. Current directory: {:?}",
            std::env::current_dir(),
        )
    })
}
