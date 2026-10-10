//! Selection's public command/evidence seam uses a synthetic LoROM and measured counts.
use serde_json::{Value, json};
use std::{fs, process::Command};

#[test]
fn observed_deferred_supported_key_in_configured_routine_precedes_new_pattern() {
    let root = std::env::temp_dir().join(format!("recomp-selection-deferred-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let mut rom = vec![0xff; 3 * 1024 * 1024];
    rom[..6].copy_from_slice(&[0xea, 0xca, 0xad, 0x00, 0x20, 0x60]);
    fs::write(root.join("rom.sfc"), rom).unwrap();
    fs::write(root.join("functions.toml"),
        "[[function]]\naddr = \"808000\"\nvariants = [\"m1x1\"]\n\n[[excluded_step]]\naddr = \"808001\"\nvariant = \"m1x1\"\nreason = \"deferred unexecuted proposed key\"\n").unwrap();
    fs::write(root.join("entries.csv"), "addr,mx,count\n808000,m1x1,1\n").unwrap();
    fs::write(root.join("inventory.json"), json!({"keys":[{"address":0x808000,"mx":3}]}).to_string()).unwrap();
    fs::write(
        root.join("profile.json"),
        json!({"interpreted":[
            {"address":0x808001,"mx":3,"opcode":0xca,"emulation":false,"count":20},
            {"address":0x808002,"mx":3,"opcode":0xad,"emulation":false,"count":1000}
        ]})
        .to_string(),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_recomp"))
        .args([
            "--rom",
            root.join("rom.sfc").to_str().unwrap(),
            "--functions",
            root.join("functions.toml").to_str().unwrap(),
            "--selection-profile",
            root.join("profile.json").to_str().unwrap(),
            "--selection-entries",
            root.join("entries.csv").to_str().unwrap(),
            "--selection-inventory",
            root.join("inventory.json").to_str().unwrap(),
            "--selection-out",
            root.join("selection.json").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&fs::read(root.join("selection.json")).unwrap()).unwrap();
    assert_eq!(report["outcome"], "supported-routine");
    assert_eq!(report["chosen"]["action"], "enable-deferred-keys");
    assert_eq!(report["chosen"]["estimated_compiled_step_gain"], 20);
    assert_eq!(report["chosen"]["new_executed_keys"][0]["address"], 0x808001);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn measured_selection_excludes_overlapping_work_and_prefers_simpler_equal_gain() {
    let root = std::env::temp_dir().join(format!("recomp-selection-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let mut rom = vec![0xff; 3 * 1024 * 1024];
    // Routine 808000: LDA; STA; unsupported RTS. 808010: NOP; RTS.
    rom[..6].copy_from_slice(&[0xa9, 0x01, 0x8d, 0x00, 0x20, 0x60]);
    rom[16..18].copy_from_slice(&[0xea, 0x60]);
    fs::write(root.join("rom.sfc"), rom).unwrap();
    fs::write(root.join("functions.toml"), "").unwrap();
    fs::write(root.join("entries.csv"), "addr,mx,count\n808000,m1x1,999999\n808010,m1x1,1\n").unwrap();
    fs::write(root.join("inventory.json"), json!({"keys":[{"address":0x808000,"mx":3}]}).to_string()).unwrap();
    fs::write(
        root.join("profile.json"),
        json!({"interpreted":[
            {"address":0x808000,"mx":3,"opcode":0xa9,"emulation":false,"count":100},
            {"address":0x808002,"mx":3,"opcode":0x8d,"emulation":false,"count":20},
            {"address":0x808010,"mx":3,"opcode":0xea,"emulation":false,"count":20}
        ]})
        .to_string(),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_recomp"))
        .args([
            "--rom",
            root.join("rom.sfc").to_str().unwrap(),
            "--functions",
            root.join("functions.toml").to_str().unwrap(),
            "--selection-profile",
            root.join("profile.json").to_str().unwrap(),
            "--selection-entries",
            root.join("entries.csv").to_str().unwrap(),
            "--selection-inventory",
            root.join("inventory.json").to_str().unwrap(),
            "--selection-out",
            root.join("selection.json").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&fs::read(root.join("selection.json")).unwrap()).unwrap();
    assert_eq!(report["chosen"]["address"], 0x808010);
    assert_eq!(report["chosen"]["estimated_compiled_step_gain"], 20);
    assert_eq!(report["ranked"][1]["overlapping_keys"].as_array().unwrap().len(), 1);
    assert_eq!(report["ranked"][1]["new_executed_keys"][0]["address"], 0x808002);
    assert_eq!(report["ranked"][1]["new_executed_keys"][0]["bytes"], json!([0x8d, 0x00, 0x20]));
    assert!(!root.join("out/generated/game.rs").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn selection_distinguishes_uncovered_work_from_observed_unsupported_pattern() {
    let root = std::env::temp_dir().join(format!("recomp-selection-pattern-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let mut rom = vec![0xff; 3 * 1024 * 1024];
    rom[..2].copy_from_slice(&[0x18, 0x60]); // CLC; RTS, both still interpreted.
    fs::write(root.join("rom.sfc"), rom).unwrap();
    fs::write(root.join("functions.toml"), "").unwrap();
    fs::write(root.join("entries.csv"), "addr,mx,count\n808000,m1x1,1000000\n").unwrap();
    fs::write(root.join("inventory.json"), json!({"keys":[]}).to_string()).unwrap();
    for (rows, outcome) in [
        (json!([]), "no-covered-candidate"),
        (json!([{"address":0x808000,"mx":3,"opcode":0x18,"emulation":false,"count":40}]), "needs-timed-pattern"),
    ] {
        fs::write(root.join("profile.json"), json!({"interpreted":rows}).to_string()).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_recomp"))
            .args([
                "--rom",
                root.join("rom.sfc").to_str().unwrap(),
                "--functions",
                root.join("functions.toml").to_str().unwrap(),
                "--selection-profile",
                root.join("profile.json").to_str().unwrap(),
                "--selection-entries",
                root.join("entries.csv").to_str().unwrap(),
                "--selection-inventory",
                root.join("inventory.json").to_str().unwrap(),
                "--selection-out",
                root.join("selection.json").to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let report: Value = serde_json::from_slice(&fs::read(root.join("selection.json")).unwrap()).unwrap();
        assert_eq!(report["outcome"], outcome);
        assert!(report["chosen"].is_null());
        if outcome == "needs-timed-pattern" {
            assert_eq!(report["observed_unsupported_patterns"][0], json!({"pattern":"CLC/Imp","count":40}));
        }
    }
    fs::remove_dir_all(root).unwrap();
}
