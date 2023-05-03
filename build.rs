#![allow(unused_imports, dead_code)]
use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};
use std::env;
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use std::path::PathBuf;

extern crate cbindgen;
extern crate gl_generator;

fn link_lib_from_cmake(lib: &str, root: &PathBuf, path_segments: &[&str]) {
    let mut path = root.clone();
    path.extend(path_segments);
    println!("cargo:rustc-link-search=native={}", path.display());
    println!("cargo:rustc-link-lib={}", lib);
}

fn gen_bindings() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .generate()
        .expect("Error generating bindings.")
        .write_to_file("src/cpp/include/bindings.h");
}

fn main() {
    println!("cargo:rustc-env=PHX_VERSION=0.0.1");

    use std::str::FromStr;

    // Generate GL bindings.
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(&Path::new(&dest).join("bindings.rs")).unwrap();
    Registry::new(
        Api::Gl,
        (2, 1),
        Profile::Compatibility,
        Fallbacks::All,
        [
            "GL_ARB_seamless_cubemap_per_texture",
            "GL_ARB_texture_rg",
            "GL_EXT_texture_filter_anisotropic",
            "GL_ARB_framebuffer_object",
            "GL_ARB_texture_mirror_clamp_to_edge",
        ],
    )
    .write_bindings(GlobalGenerator, &mut file)
    .unwrap();

    // Download dependencies.
    let cmake_root = cmake::Config::new("libphx")
        .profile("Release")
        .build_target("libphx-external")
        .build();
    let deps_root = cmake_root.join("build").join("_deps");

    gen_bindings();

    // Build C++ files which haven't been ported yet.
    cc::Build::new()
        .cpp(true)
        .file("libphx/src/cpp/CollisionShape.cpp")
        .file("libphx/src/cpp/Physics.cpp")
        .file("libphx/src/cpp/RigidBody.cpp")
        .file("libphx/src/cpp/Trigger.cpp")
        .flag("-std=c++11")
        .warnings(false)
        .link_lib_modifier("+whole-archive,-bundle")
        .include("libphx/src/cpp/include")
        .include(deps_root.join("bullet-src").join("src"))
        .compile("phx-cc");

    // Link dependencies.
    link_lib_from_cmake("freetype", &deps_root, &["freetype-build"]);
    link_lib_from_cmake(
        "BulletDynamics",
        &deps_root,
        &["bullet-build", "src", "BulletDynamics"],
    );
    link_lib_from_cmake(
        "BulletCollision",
        &deps_root,
        &["bullet-build", "src", "BulletCollision"],
    );
    link_lib_from_cmake(
        "LinearMath",
        &deps_root,
        &["bullet-build", "src", "LinearMath"],
    );

    println!("cargo:rustc-link-lib={}", "z");
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=framework={}", "CoreHaptics");
    }

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-Wl,-keep_dwarf_unwind");
        println!("cargo:rustc-link-arg=-Wl,-no_compact_unwind");
        println!("cargo:rustc-link-arg=-Wl,-install_name,@rpath/libphx.dylib");
    }
    
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/deps");
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/deps");
    }
}
