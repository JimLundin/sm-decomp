//! Picks the code to compile in:
//! - `out/generated/game.rs` if you've run `cargo run -p recomp` against your ROM;
//! - otherwise a tiny built-in demo program, recompiled right here, so the
//!   workspace builds and the end-to-end test runs without a ROM.

use std::collections::HashSet;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let out = PathBuf::from(std::env::var("OUT_DIR")?).join("generated.rs");
    let real = generated_input("SM_RECOMP_GENERATED_GAME", "game.rs")?;
    println!("cargo:rerun-if-changed={}", real.display());
    println!("cargo:rerun-if-changed=build.rs");

    if real.exists() {
        std::fs::copy(&real, &out)?;
        println!("cargo:rustc-cfg=sm_generated");
    } else {
        std::fs::write(&out, demo()?)?;
    }
    println!("cargo:rustc-check-cfg=cfg(sm_generated)");
    let steps = generated_input("SM_RECOMP_GENERATED_STEPS", "steps.rs")?;
    println!("cargo:rerun-if-changed={}", steps.display());
    let steps_out = PathBuf::from(std::env::var("OUT_DIR")?).join("steps.rs");
    if steps.exists() {
        std::fs::copy(steps, steps_out)?;
    } else {
        std::fs::write(
            steps_out,
            "pub static STEP_TABLE: &[runtime::StepEntry] = &[];\npub const ROM_SHA256: Option<&str> = None;\n",
        )?;
    }
    Ok(())
}

/// Explicit run-local inputs must exist; never substitute shared output or demo.
fn generated_input(variable: &str, filename: &str) -> anyhow::Result<PathBuf> {
    println!("cargo:rerun-if-env-changed={variable}");
    if let Some(path) = std::env::var_os(variable) {
        let path = PathBuf::from(path);
        anyhow::ensure!(path.is_file(), "{variable} names a missing generated input: {}", path.display());
        return Ok(path);
    }
    Ok(PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?).join("../../out/generated").join(filename))
}

/// Demo "ROM" at $80:8000 exercising widths, a loop, a call and a store.
/// Mirrors what the real pipeline does, minus the real ROM.
fn demo() -> anyhow::Result<String> {
    use recomp::{cfg, emit, rom::Rom};
    #[rustfmt::skip]
    let main_fn: &[u8] = &[
        0xC2, 0x30,             // REP #$30        ; 16-bit A, X, Y
        0xA9, 0x34, 0x12,       // LDA #$1234
        0x8D, 0x10, 0x00,       // STA $0010
        0xA2, 0x03, 0x00,       // LDX #$0003
        0xCA,                   // loop: DEX
        0xD0, 0xFD,             // BNE loop
        0x20, 0x20, 0x80,       // JSR $8020
        0xE2, 0x20,             // SEP #$20        ; 8-bit A
        0xA9, 0xAB,             // LDA #$AB
        0x8D, 0x12, 0x00,       // STA $0012
        0x60,                   // RTS
    ];
    #[rustfmt::skip]
    let callee: &[u8] = &[
        0xEE, 0x10, 0x00,       // INC $0010       ; 16-bit → $1235
        0x60,                   // RTS
    ];
    let mut data = vec![0u8; 0x8000];
    data[..main_fn.len()].copy_from_slice(main_fn);
    data[0x20..0x20 + callee.len()].copy_from_slice(callee);
    let rom = Rom::from_bytes(data);
    let read = |a: u32| rom.byte(a);
    let entries: HashSet<u32> = [0x80_8000, 0x80_8020].into();
    let funcs = vec![
        (cfg::build(&read, 0x80_8000, true, true, &entries)?, Some("demo_main".into())),
        (cfg::build(&read, 0x80_8020, false, false, &entries)?, Some("demo_callee".into())),
    ];
    emit::emit_crate(&funcs)
}
