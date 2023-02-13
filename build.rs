#[cfg(all(unix, not(target_os = "macos")))]
fn main() {
    // add unix dependencies below
    // println!("cargo:rustc-flags=-l readline");
}

#[cfg(target_os = "macos")]
fn main() {
    // add macos dependencies below
    // println!("cargo:rustc-flags=-l edit");
    {
        let dst = cmake::Config::new(".").build_target("BulletCollision").build();
        println!("cargo:rustc-link-search=native={}", dst.display());
    }
    // {
    //     let dst = cmake::Config::new(".").build_target("BulletDynamics").build();
    //     println!("cargo:rustc-link-search=native={}", dst.display());
    // }
}
