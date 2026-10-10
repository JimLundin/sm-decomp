// Include generated test cases if `cargo run -p fuzz -- gen ...` has produced them.
fn main() {
    println!("cargo:rerun-if-env-changed=SM_TIMED_STEPS");
    let timed = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("timed_steps.rs");
    if let Some(source) = std::env::var_os("SM_TIMED_STEPS") {
        println!("cargo:rerun-if-changed={}", std::path::Path::new(&source).display());
        std::fs::copy(source, timed).unwrap();
    } else {
        std::fs::write(timed, "pub static BODIES: &[runtime::step::StepFn] = &[];\n").unwrap();
    }
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
