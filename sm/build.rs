fn main() {
    // The platform layer still calls SDL2 (and libm on Unix) directly through
    // FFI. Replacing that with a Rust crate such as `sdl2` is phase 2 work.
    println!("cargo:rustc-link-lib=SDL2");
    if std::env::var("CARGO_CFG_UNIX").is_ok() {
        println!("cargo:rustc-link-lib=m");
    }
}
