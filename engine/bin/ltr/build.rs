use std::env;
use std::path::Path;

use winres::WindowsResource;

fn main() {
    if env::var("TARGET")
        .expect("target is not set")
        .ends_with("windows-msvc")
    {
        // App Icon
        WindowsResource::new()
            .set_icon("../../../res/images/LTR-Icon.ico")
            .compile()
            .unwrap();

        let manifest = Path::new("src/ltr.exe.manifest").canonicalize().unwrap();
        println!("cargo::rustc-link-arg-bins=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-bins=/MANIFESTINPUT:{}",
            manifest.display()
        );
        println!("cargo::rerun-if-changed=src/ltr.exe.manifest");

        // Prefer dedicated graphics.
        println!("cargo::rustc-link-arg=/EXPORT:NvOptimusEnablement");
        println!("cargo::rustc-link-arg=/EXPORT:AmdPowerXpressRequestHighPerformance");
    }

    if cfg!(target_os = "linux") {
        println!("cargo::rustc-link-arg=-Wl,-rpath,$ORIGIN");
        println!("cargo::rustc-link-arg=-Wl,-rpath,$ORIGIN/deps");
    } else if cfg!(target_os = "macos") {
        println!("cargo::rustc-link-arg=-Wl,-rpath,@executable_path");
        println!("cargo::rustc-link-arg=-Wl,-rpath,@executable_path/deps");
    }

    // libphx is linked with the #[link] attribute in main.rs. The phx library is placed in the
    // `target` directory, so we need to add that to the linker search path.
    //
    // OUT_DIR is set to <src>/target/<cfg>/build/ltr-<hash>/out, so we need to move up 3
    // directories from there.
    let dir = env::var("OUT_DIR").unwrap();
    println!(
        "cargo::rustc-link-search={}",
        Path::new(&dir).join("../../..").display()
    );

    // If we don't specify rerun-if-changed, then Cargo will always rerun build.rs, causing phx to
    // always be rebuilt.
    //
    // Here, we just specify build.rs as the file that should be used to guide if phx's build
    // script needs to be re-run.
    println!("cargo::rerun-if-changed=build.rs");
}
