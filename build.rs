fn main() {
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    if std::env::var_os("CARGO_FEATURE_DIAGNOSTICS").is_some() {
        println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    }
}
