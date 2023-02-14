#[cfg(all(unix, not(target_os = "macos")))]
fn main() {
    // add unix dependencies below
    // println!("cargo:rustc-flags=-l readline");
}

#[cfg(target_os = "macos")]
fn main() {
    let dst = cmake::Config::new(".")
        .profile("Release")
        .build_target("libphx-external")
        .build();
    let package_path = dst.join("build").join("_deps");
    println!("cargo:rustc-link-search=native={}", package_path.join("luajit-src").join("src").display());
    println!("cargo:rustc-link-lib={}", "luajit");
    println!("cargo:rustc-link-search=native={}", package_path.join("fmod-src").join("lib").join("macos").display());
    println!("cargo:rustc-link-lib={}", "fmod");
    println!("cargo:rustc-link-search=native={}", dst.join("build").join("lib").display());
    println!("cargo:rustc-link-lib={}", "GLEW");
    println!("cargo:rustc-link-search=native={}", package_path.join("freetype-build").display());
    println!("cargo:rustc-link-lib={}", "freetype");
    println!("cargo:rustc-link-search=native={}", package_path.join("lz4-build").display());
    println!("cargo:rustc-link-lib={}", "lz4");
    println!("cargo:rustc-link-lib={}", "z");
    // println!("cargo:rustc-link-search=native={}", package_path.join("sdl-build").display());
    // println!("cargo:rustc-link-lib={}", "SDL2");
    println!("cargo:rustc-link-lib=framework={}", "CoreHaptics");
    // panic!();
}
