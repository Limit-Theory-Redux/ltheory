#![allow(unused_imports, dead_code)]
use std::path::PathBuf;
use std::path::Path;
use std::env;
use std::io::Cursor;
use http_req::request;
use std::fs;

// extern crate flate2;
// extern crate http_req;
// extern crate tar;

// fn get_install_dir() -> String {
//     env::var("OUT_DIR").unwrap() + "/installed"
// }

// fn download(url: &str) -> Cursor<Vec<u8>> {
//     let response = request::get(url).map_err(|error| error.to_string()).unwrap();
//     if response.status_code() < 200 || response.status_code() >= 300 {
//         panic!("Download error: HTTP {}", response.status_code());
//     }
//     let resp_body = response.body();
//     let buffer = resp_body.to_vec();
//     Cursor::new(buffer)
// }

// fn get_tarball(url: &str, dir: &str) {
//     use flate2::read::GzDecoder;
//     use tar::Archive;

//     // Download gz tarball
//     let install_dir = get_install_dir();
//     let lib_install_dir = Path::new(&install_dir).join("lib");
//     fs::create_dir_all(&lib_install_dir);
//     let compressed_file = download(url);

//     // Unpack the tarball
//     let gz_decoder = GzDecoder::new(compressed_file);
//     let mut archive = Archive::new(gz_decoder);

//     archive.unpack(dir);

//     // Extract just the appropriate version of libsodium.a and headers to the install path
//     let arch_path = if cfg!(target_pointer_width = "32") {
//         Path::new("libsodium-win32")
//     } else if cfg!(target_pointer_width = "64") {
//         Path::new("libsodium-win64")
//     } else {
//         panic!("target_pointer_width not 32 or 64")
//     };

//     let unpacked_include = arch_path.join("include");
//     let unpacked_lib = arch_path.join("lib\\libsodium.a");
//     let entries = unwrap!(archive.entries());
//     for entry_result in entries {
//         let mut entry = unwrap!(entry_result);
//         let entry_path = unwrap!(entry.path()).to_path_buf();
//         let full_install_path = if entry_path.starts_with(&unpacked_include) {
//             let include_file = unwrap!(entry_path.strip_prefix(arch_path));
//             Path::new(&install_dir).join(include_file)
//         } else if entry_path == unpacked_lib {
//             lib_install_dir.join("libsodium.a")
//         } else {
//             continue;
//         };
//         unwrap!(entry.unpack(full_install_path));
//     }
// }

fn link_lib_from_cmake(lib: &str, root: &PathBuf, path_segments: &[&str])
{
    let mut path = root.clone();
    path.extend(path_segments);
    println!("cargo:rustc-link-search=native={}", path.display());
    println!("cargo:rustc-link-lib={}", lib);
}

fn main() {
    println!("cargo:rustc-env=PHX_VERSION=0.0.1");

    use std::str::FromStr;

    // Download dependencies.
    let cmake_root = cmake::Config::new(".")
        .profile("Release")
        .build_target("libphx-external")
        .build();
    let deps_root = cmake_root.join("build").join("_deps");

    // Build C++ files which haven't been ported yet.
    cc::Build::new()
        .cpp(true)
        .file("src/cpp/CollisionShape.cpp")
        .file("src/cpp/Physics.cpp")
        .file("src/cpp/RigidBody.cpp")
        .file("src/cpp/Trigger.cpp")
        .flag("-std=c++11")
        .warnings(false)
        .link_lib_modifier("+whole-archive,-bundle")
        .include("src/cpp/include")
        .include(deps_root.join("bullet-src").join("src"))
        .compile("phx-cc");

    // Link dependencies.
    link_lib_from_cmake("luajit", &deps_root, &["luajit-src", "src"]);
    link_lib_from_cmake("freetype", &deps_root, &["freetype-build"]);
    link_lib_from_cmake("GLEW", &cmake_root, &["build", "lib"]);
    link_lib_from_cmake("BulletDynamics", &deps_root, &["bullet-build", "src", "BulletDynamics"]);
    link_lib_from_cmake("BulletCollision", &deps_root, &["bullet-build", "src", "BulletCollision"]);
    link_lib_from_cmake("LinearMath", &deps_root, &["bullet-build", "src", "LinearMath"]);
    // if cfg!(target_os = "macos") {
    //     link_lib_from_cmake("fmod", &deps_root, &["fmod-src", "lib", "macos"]);
    // } else if cfg!(target_os = "linux") {
    //     link_lib_from_cmake("fmod", &deps_root, &["fmod-src", "lib", "linux", "x86_64"]);
    // }
    // println!("cargo:rustc-link-lib={}", "fmod");

    println!("cargo:rustc-link-lib={}", "z");
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=framework={}", "CoreHaptics");
        println!("cargo:rustc-link-lib=framework={}", "OpenGL");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib={}", "GLX");
        println!("cargo:rustc-link-lib={}", "OpenGL");
    }

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-Wl,-keep_dwarf_unwind");
        println!("cargo:rustc-link-arg=-Wl,-no_compact_unwind");
    }
    // panic!();
}
