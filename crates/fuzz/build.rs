// Include generated test cases if `cargo run -p fuzz -- gen ...` has produced them.
fn main() {
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("cases.rs");
    let real =
        std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../out/generated/fuzz.rs");
    println!("cargo:rerun-if-changed={}", real.display());
    if real.exists() {
        std::fs::copy(&real, &out).unwrap();
    } else {
        std::fs::write(&out, "// no cases generated yet\n").unwrap();
    }
}
