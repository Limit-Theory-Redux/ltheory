fn main() {
    let lib = "phx";
    println!("cargo:rustc-link-lib=dylib={}", lib);
}
