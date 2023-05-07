fn main() {
    // let lib = if cfg!(target_os = "windows") {
    //     "phx.dll"
    // } else {
    //     "phx"
    // };
    // println!("cargo:rustc-link-lib=dylib={}", lib);

    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/deps");
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/deps");
    }
}
