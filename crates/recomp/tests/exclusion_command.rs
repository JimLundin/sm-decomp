//! A deferred key is absent from the actual generated table, not a coverage waiver.
use std::{fs, process::Command};

#[test]
fn generation_excludes_one_deferred_width_and_keeps_other_widths_and_legacy_keys() {
    let root = std::env::temp_dir().join(format!("recomp-exclusion-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let mut rom = vec![0xff; 3 * 1024 * 1024];
    rom[..3].copy_from_slice(&[0xea, 0xca, 0x60]);
    fs::write(root.join("rom.sfc"), rom).unwrap();
    fs::write(root.join("functions.toml"),"[[function]]\naddr = \"808000\"\nvariants = [\"m0x0\",\"m1x1\"]\n\n[[excluded_step]]\naddr = \"808001\"\nvariant = \"m1x1\"\nreason = \"unexecuted proposed key\"\n").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_recomp"))
        .args([
            "--rom",
            root.join("rom.sfc").to_str().unwrap(),
            "--functions",
            root.join("functions.toml").to_str().unwrap(),
            "--out",
            root.join("game.rs").to_str().unwrap(),
            "--steps-out",
            root.join("steps.rs").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let table = fs::read_to_string(root.join("steps.rs")).unwrap();
    assert!(!table.contains("fn step_808001_3"), "deferred body remains compiled");
    assert!(table.contains("fn step_808001_0"), "other width was dropped");
    assert!(table.contains("fn step_808000_3"), "covered neighbor was dropped");
    assert!(table.contains("fn step_808000_0"), "legacy baseline was dropped");
    fs::remove_dir_all(root).unwrap();
}
