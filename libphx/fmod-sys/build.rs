#![allow(unused_imports, dead_code)]
use std::env;
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use std::path::PathBuf;

extern crate bindgen;
extern crate ureq;
extern crate zip_extract;

fn download(url: &str) -> Vec<u8> {
    let response = ureq::get(url).call().unwrap();
    assert!(response.has("Content-Length"));
    let len: usize = response.header("Content-Length").unwrap().parse().unwrap();
    let mut bytes: Vec<u8> = Vec::with_capacity(len);
    response.into_reader().read_to_end(&mut bytes).unwrap();
    bytes
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Download FMOD package from GitHub.
    let fmod_package = "fmod-2.02.08";
    let fmod_dir = out_dir.join(fmod_package);
    if !fmod_dir.is_dir() {
        let bin_archive = download(
            format!(
                "https://github.com/Limit-Theory-Redux/ltheory/releases/download/v0.0.1-pre/{}.zip",
                fmod_package
            )
            .as_str(),
        );
        zip_extract::extract(Cursor::new(bin_archive), fmod_dir.as_path(), true).unwrap();
    }

    // Copy the binary to target/<cfg>/deps.
    //
    // out_dir will be in the form <ltheory>/target/<cfg>/build/fmod-sys-1ac366e3920adf45/out
    // Move 3 directories out, then copy the fmod library there.
    let deps_dir = out_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("deps");

    if cfg!(target_os = "windows") {
        let src = fmod_dir
            .join("lib")
            .join("win")
            .join("x86_64")
            .join("fmod.dll");
        let dest = deps_dir.join("fmod.dll");
        fs::copy(src, dest).unwrap();
    } else if cfg!(target_os = "macos") {
        let src = fmod_dir.join("lib").join("macos").join("libfmod.dylib");
        let dest = deps_dir.join("libfmod.dylib");
        fs::copy(src, dest).unwrap();
    } else if cfg!(target_os = "linux") {
        let src = fmod_dir
            .join("lib")
            .join("linux")
            .join("x86_64")
            .join("libfmod.so.13.8");
        let dest = deps_dir.join("libfmod.so.13.8");
        fs::copy(src, dest).unwrap();
    }
    println!("cargo:rustc-link-lib={}", "fmod");

    // Link against it.
    println!(
        "cargo:rustc-link-search={}",
        fmod_dir.join("lib").join("macos").display()
    );
    println!("cargo:rustc-link-lib=fmod");
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", fmod_dir.join("include").display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .generate()
        .expect("Unable to generate bindings");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
