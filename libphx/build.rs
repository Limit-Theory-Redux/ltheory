use std::path::PathBuf;
use std::env;

fn link(lib : &str, root : &PathBuf, path_segments: &[&str])
{
    let mut path = root.clone();
    path.extend(path_segments);
    println!("cargo:rustc-link-search={}", path.display());
    println!("cargo:rustc-link-lib={}", lib);
}
fn link_kind(kind : &str, lib :&str, root : &PathBuf, path_segments: &[&str])
{
    let mut path = root.clone();
    path.extend(path_segments);
    println!("cargo:rustc-link-search={}={}", kind, path.display());
    println!("cargo:rustc-link-lib={}", lib);
}

// #[cfg(target_os = "macos")]
fn main() {
    use std::str::FromStr;

    let out_dir = PathBuf::from_str(env::var_os("OUT_DIR").unwrap().to_str().unwrap()).unwrap();
    let package_path = out_dir.join("build").join("_deps");
    
    // Download dependencies.
    let dst = cmake::Config::new(".")
        .profile("Release")
        .build_target("libphx-external")
        .build();
    // let package_path = dst.join("build").join("_deps");
    // println!("DESTINATION: {}", dst.display());

    link_kind("native", "luajit", &package_path, &["luajit-src", "src"]);
    // println!("cargo:rustc-link-search=native={}", package_path.join("luajit-src").join("src").display());
    // println!("cargo:rustc-link-lib={}", "luajit");
    // if cfg!(target_os = "macos") {
    //     // println!("cargo:rustc-link-search=native={}", package_path.join("fmod-src").join("lib").join("macos").display());
    //     link_kind("native", "fmod", &package_path, &["fmod-src", "lib", "macos"]);
    // }
    // else if cfg!(target_os = "linux")
    // {
    //     // println!("cargo:rustc-link-search=native={}", package_path.join("fmod-src").join("lib").join("linux").join("x86_64").display());
    //     link_kind("native", "fmod", &package_path, &["fmod-src", "lib", "linux", "x86_64"]);
    // }
    // println!("cargo:rustc-link-lib={}", "fmod");

    println!("cargo:rustc-link-search=native={}", package_path.join("freetype-build").display());
    println!("cargo:rustc-link-lib={}", "freetype");
    println!("cargo:rustc-link-search=native={}", package_path.join("lz4-build").display());
    println!("cargo:rustc-link-lib={}", "lz4");
    println!("cargo:rustc-link-lib={}", "z");
    // println!("cargo:rustc-link-search=native={}", package_path.join("sdl-build").display());
    // println!("cargo:rustc-link-lib={}", "SDL2");
    println!("cargo:rustc-link-search=native={}", dst.join("build").join("lib").display());
    println!("cargo:rustc-link-lib={}", "GLEW");
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=framework={}", "CoreHaptics");
        println!("cargo:rustc-link-lib=framework={}", "OpenGL");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib={}", "GLX");
        println!("cargo:rustc-link-lib={}", "OpenGL");
    }

    // Build C++ files which haven't been ported over.
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
        .include(package_path.join("bullet-src").join("src"))
        .compile("phx-cc");

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-Wl,-keep_dwarf_unwind");
        println!("cargo:rustc-link-arg=-Wl,-no_compact_unwind");
    }

    println!("cargo:rustc-link-search=native={}", package_path.join("bullet-build").join("src").join("BulletDynamics").display());
    println!("cargo:rustc-link-lib={}", "BulletDynamics");
    println!("cargo:rustc-link-search=native={}", package_path.join("bullet-build").join("src").join("BulletCollision").display());
    println!("cargo:rustc-link-lib={}", "BulletCollision");
    println!("cargo:rustc-link-search=native={}", package_path.join("bullet-build").join("src").join("LinearMath").display());
    println!("cargo:rustc-link-lib={}", "LinearMath");
    // panic!();
}
