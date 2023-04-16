#![allow(unused_imports, dead_code)]
use std::env;
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use std::path::PathBuf;

extern crate bindgen;
extern crate reqwest;
extern crate zip_extract;

fn download(url: &str) -> Vec<u8> {
    reqwest::blocking::get(url)
        .unwrap()
        .bytes()
        .unwrap()
        .to_vec()
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Download FMOD package from GitHub.
    let fmod_package = "fmod-2.02.08";
    let fmod_path = out_path.join(fmod_package);
    if !fmod_path.is_dir() {
        let bin_archive = download(
            format!(
                "https://github.com/Limit-Theory-Redux/ltheory/releases/download/v0.0.1-pre/{}.zip",
                fmod_package
            )
            .as_str(),
        );
        zip_extract::extract(Cursor::new(bin_archive), fmod_path.as_path(), true).unwrap();
    }

    // Link against it.
    println!(
        "cargo:rustc-link-search={}",
        fmod_path.join("lib").join("macos").display()
    );
    println!("cargo:rustc-link-lib=fmod");
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", fmod_path.join("include").display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
