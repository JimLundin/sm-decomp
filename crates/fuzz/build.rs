// Include generated test cases if `cargo run -p fuzz -- gen ...` has produced them.
fn main() {
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("cases.rs");
    let real = std::path::Path::new("generated/cases.rs");
    println!("cargo:rerun-if-changed=generated");
    if real.exists() {
        std::fs::copy(real, &out).unwrap();
    } else {
        std::fs::write(&out, "// no cases generated yet\n").unwrap();
    }
}
