use std::env;
use std::path::Path;
use winres::WindowsResource;

fn main() {
    if env::var("TARGET")
        .expect("target")
        .ends_with("windows-msvc")
    {
        // App Icon
        WindowsResource::new()
        .set_icon("res/images/LTR-Icon.ico")
        .compile().unwrap();

        let manifest = Path::new("src/ltr.exe.manifest").canonicalize().unwrap();
        println!("cargo:rustc-link-arg-bins=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-bins=/MANIFESTINPUT:{}",
            manifest.display()
        );
        println!("cargo:rerun-if-changed=src/ltr.exe.manifest");
    }

    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/deps");
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/deps");
    }
}
