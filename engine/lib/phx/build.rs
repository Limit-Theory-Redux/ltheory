use std::env;

fn main() {
    let phx_version = env::var("PHX_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
    println!("cargo::rustc-env=PHX_VERSION={phx_version}");

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
