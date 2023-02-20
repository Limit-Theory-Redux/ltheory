fn main() {
    let lib = "phx";
    println!("cargo:rustc-link-lib=dylib={}", lib);

    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/deps");
    }
}
