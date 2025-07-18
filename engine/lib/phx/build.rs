#![allow(unused_imports, dead_code)]
use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::{env, fs};

use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};

fn main() {
    let phx_version = env::var("PHX_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
    println!("cargo::rustc-env=PHX_VERSION={phx_version}");

    use std::str::FromStr;

    // Generate GL bindings.
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(Path::new(&dest).join("bindings.rs")).unwrap();
    Registry::new(
        Api::Gl,
        (3, 3),
        Profile::Core,
        Fallbacks::All,
        [
            "GL_EXT_texture_filter_anisotropic",
            "GL_ARB_texture_mirror_clamp_to_edge",
        ],
    )
    .write_bindings(GlobalGenerator, &mut file)
    .unwrap();

    // Link dependencies.
    if !cfg!(target_os = "windows") {
        println!("cargo::rustc-link-lib=z");
    }
    if cfg!(target_os = "macos") {
        println!("cargo::rustc-link-lib=framework=CoreHaptics");
    }

    if cfg!(target_os = "macos") {
        println!("cargo::rustc-link-arg=-Wl,-keep_dwarf_unwind");
        println!("cargo::rustc-link-arg=-Wl,-no_compact_unwind");
        println!("cargo::rustc-link-arg=-Wl,-install_name,@rpath/libphx.dylib");
    }

    // Set rpath correctly for libphx.
    if cfg!(target_os = "linux") {
        println!("cargo::rustc-link-arg=-Wl,-rpath,$ORIGIN");
        println!("cargo::rustc-link-arg=-Wl,-rpath,$ORIGIN/deps");
    } else if cfg!(target_os = "macos") {
        println!("cargo::rustc-link-arg=-Wl,-rpath,@executable_path");
        println!("cargo::rustc-link-arg=-Wl,-rpath,@executable_path/deps");
    }

    // If we don't specify rerun-if-changed, then Cargo will always rerun build.rs, causing phx to always be rebuild.
    //
    // Here, we just specify build.rs as the file that should be used to guide if phx's build script needs to be re-run.
    println!("cargo::rerun-if-changed=build.rs");
}
