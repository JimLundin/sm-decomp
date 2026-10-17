//! Selection's public command/evidence seam uses a synthetic LoROM and measured counts.
use pipeline_model::{
    GeneratedKey, InstructionCount, InstructionProfile, KeyInventory, SelectionAction, SelectionOutcome,
    SelectionReport,
};
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
    inventory(&root, &[(0x808000, 3)]);
    profile(&root, &[(0x808001, 0xca, 20), (0x808002, 0xad, 1000)]);
    let output = Command::new(env!("CARGO_BIN_EXE_recomp"))
        .args([
            "--rom",
            root.join("rom.sfc").to_str().unwrap(),
            "--functions",
            root.join("functions.toml").to_str().unwrap(),
            "--selection-profile",
            root.join("profile.toml").to_str().unwrap(),
            "--selection-entries",
            root.join("entries.csv").to_str().unwrap(),
            "--selection-inventory",
            root.join("inventory.toml").to_str().unwrap(),
            "--selection-out",
            root.join("selection.toml").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: SelectionReport = pipeline_model::read(&root.join("selection.toml")).unwrap();
    assert_eq!(report.outcome, SelectionOutcome::SupportedRoutine);
    assert_eq!(report.chosen.as_ref().unwrap().action, SelectionAction::EnableDeferredKeys);
    assert_eq!(report.chosen.as_ref().unwrap().estimated_compiled_step_gain, 20);
    assert_eq!(report.chosen.as_ref().unwrap().new_executed_keys[0].address, 0x808001);
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
    inventory(&root, &[(0x808000, 3)]);
    profile(&root, &[(0x808000, 0xa9, 100), (0x808002, 0x8d, 20), (0x808010, 0xea, 20)]);
    let output = Command::new(env!("CARGO_BIN_EXE_recomp"))
        .args([
            "--rom",
            root.join("rom.sfc").to_str().unwrap(),
            "--functions",
            root.join("functions.toml").to_str().unwrap(),
            "--selection-profile",
            root.join("profile.toml").to_str().unwrap(),
            "--selection-entries",
            root.join("entries.csv").to_str().unwrap(),
            "--selection-inventory",
            root.join("inventory.toml").to_str().unwrap(),
            "--selection-out",
            root.join("selection.toml").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: SelectionReport = pipeline_model::read(&root.join("selection.toml")).unwrap();
    assert_eq!(report.chosen.as_ref().unwrap().address, 0x808010);
    assert_eq!(report.chosen.as_ref().unwrap().estimated_compiled_step_gain, 20);
    assert_eq!(report.ranked[1].overlapping_keys.len(), 1);
    assert_eq!(report.ranked[1].new_executed_keys[0].address, 0x808002);
    assert_eq!(report.ranked[1].new_executed_keys[0].bytes, vec![0x8d, 0x00, 0x20]);
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
    inventory(&root, &[]);
    for (rows, outcome) in [
        (vec![], SelectionOutcome::NoCoveredCandidate),
        (vec![(0x808000, 0x18, 40)], SelectionOutcome::NeedsTimedPattern),
    ] {
        profile(&root, &rows);
        let result = Command::new(env!("CARGO_BIN_EXE_recomp"))
            .args([
                "--rom",
                root.join("rom.sfc").to_str().unwrap(),
                "--functions",
                root.join("functions.toml").to_str().unwrap(),
                "--selection-profile",
                root.join("profile.toml").to_str().unwrap(),
                "--selection-entries",
                root.join("entries.csv").to_str().unwrap(),
                "--selection-inventory",
                root.join("inventory.toml").to_str().unwrap(),
                "--selection-out",
                root.join("selection.toml").to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let report: SelectionReport = pipeline_model::read(&root.join("selection.toml")).unwrap();
        assert_eq!(report.outcome, outcome);
        assert!(report.chosen.is_none());
        if outcome == SelectionOutcome::NeedsTimedPattern {
            assert_eq!(report.observed_unsupported_patterns[0].pattern, "CLC/Imp");
            assert_eq!(report.observed_unsupported_patterns[0].count, 40);
        }
    }
    fs::remove_dir_all(root).unwrap();
}

fn inventory(root: &std::path::Path, keys: &[(u32, u8)]) {
    pipeline_model::write(
        &root.join("inventory.toml"),
        &KeyInventory {
            schema_version: 1,
            keys: keys
                .iter()
                .map(|&(address, mx)| GeneratedKey {
                    address,
                    mx,
                    opcode: fs::read(root.join("rom.sfc")).unwrap()
                        [(address as usize & 0x7fff) | ((address as usize >> 16 & 0x7f) << 15)],
                    bytes: vec![
                        fs::read(root.join("rom.sfc")).unwrap()
                            [(address as usize & 0x7fff) | ((address as usize >> 16 & 0x7f) << 15)],
                    ],
                    body_sha256: "0".repeat(64),
                })
                .collect(),
        },
    )
    .unwrap();
}
fn profile(root: &std::path::Path, rows: &[(u32, u8, u64)]) {
    let total = rows.iter().map(|r| r.2).sum();
    pipeline_model::write(
        &root.join("profile.toml"),
        &InstructionProfile {
            schema_version: 1,
            compiled: vec![],
            interpreted: rows
                .iter()
                .map(|&(address, opcode, count)| InstructionCount {
                    address,
                    mx: 3,
                    opcode,
                    count,
                    emulation: false,
                    pattern: "fixture".into(),
                    supported: false,
                })
                .collect(),
            generated_instructions: 0,
            interpreted_instructions: total,
            generated_dispatches: 0,
            interpreted_dispatches: total,
            noninstruction_dispatches: 0,
        },
    )
    .unwrap();
}
