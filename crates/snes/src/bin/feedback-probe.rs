//! Isolated synthetic execution evidence for the mandatory feedback gate.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use snes::{Reference, Subject, paired::sha256};
use std::path::Path;

mod emitted {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/feedback_steps.rs"));
}

fn rom() -> Vec<u8> {
    let mut rom = vec![0; 0x10000];
    rom[..14].copy_from_slice(&[0x18, 0xfb, 0xa9, 0x12, 0x8d, 0x10, 0, 0xa9, 0, 0x8d, 0x10, 0, 0x80, 0xfe]);
    rom[0x100] = 0xea;
    rom[0x7fc0..0x7fd5].copy_from_slice(b"SM FEEDBACK FIXTURE  ");
    rom[0x7fd5] = 0x20;
    rom[0x7fd9] = 1;
    rom[0x7fdc..0x7fe0].copy_from_slice(&[0xcb, 0xed, 0x34, 0x12]);
    rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
    rom
}

fn wrong_store(m: &mut runtime::Machine) {
    m.write8(0x10, 0x99);
}

fn write(path: &Path, value: &Value) -> Result<()> {
    std::fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn execute(case: &str, output: &Path, source: &str, hardware: &str) -> Result<Value> {
    std::fs::create_dir(output)?;
    let mut rom = rom();
    if case == "paired-cache" {
        // Repeated 8192-byte DMA covers substantial frame time with few CPU
        // boundaries while exercising pending DMA state in the paired cache.
        let program = [
            0x18, 0xfb, 0xa9, 0x40, 0x8d, 1, 0x43, 0xa9, 0x7e, 0x8d, 4, 0x43, 0xa9, 0, 0x8d, 2, 0x43, 0x8d, 3, 0x43,
            0x8d, 5, 0x43, 0xa9, 0x20, 0x8d, 6, 0x43, 0xa9, 1, 0x8d, 0x0b, 0x42, 0xea, 0x80, 0xe8,
        ];
        rom[..program.len()].copy_from_slice(&program);
    }
    let pads = [0, 0, 0, 0];
    std::fs::write(output.join("fixture.sfc"), &rom)?;
    std::fs::write(output.join("fixture.inputs"), movie::write_inputs(&pads))?;
    std::fs::write(output.join("steps.rs"), include_str!(concat!(env!("OUT_DIR"), "/feedback_steps.rs")))?;
    let identities = json!({"rom_sha256":sha256(&rom),"movie_sha256":sha256(movie::write_inputs(&pads).as_bytes()),
        "input_sha256":sha256(movie::write_inputs(&pads).as_bytes()),"source_sha256":source,"hardware_sha256":hardware,
        "profile":"lsnes-profile","serializer":"paired-v1","compiler":"feedback-probe","features":["lsnes-profile"],"settings":{"seed":0}});
    let mut table = emitted::STEP_TABLE.to_vec();
    if case == "paired-cache" {
        table.clear();
    }
    if matches!(case, "wrong-instruction" | "transient-effect") {
        let entry = table.iter_mut().find(|entry| entry.0 == 0x8004 && entry.1 == 3).context("fixture store")?;
        entry.3 = wrong_store;
    }
    if case == "empty-compilation" {
        table.clear();
    }
    let mut r = Reference::new(&rom, 0)?;
    let mut s = Subject::with_steps(&rom, 0, &table)?;
    let mut report = match case {
        "correct-execution" | "wrong-instruction" | "transient-effect" => {
            let mut report = snes::focused::compare(&mut r, &mut s, &pads, 0, 8, &output.join("focus"))?;
            let mut frame_r = Reference::new(&rom, 0)?;
            let mut frame_s = Subject::with_steps(&rom, 0, &table)?;
            frame_r.run_frame(0)?;
            frame_s.run_frame(0)?;
            report["frame_end_equal"] = json!(
                frame_r.state_bytes() == frame_s.state_bytes()
                    && frame_r.wram() == frame_s.wram()
                    && frame_r.framebuffer_bytes() == frame_s.framebuffer_bytes()
            );
            report["frame_end_wram_sha256"] =
                json!({"reference":sha256(frame_r.wram()),"subject":sha256(frame_s.wram())});
            report["altered_key"] = if case == "correct-execution" { Value::Null } else { json!([0x8004, 3]) };
            report
        }
        "uncovered-key" | "empty-compilation" => {
            for pad in &pads[..2] {
                r.run_frame(*pad)?;
                s.run_frame(*pad)?;
                ensure!(
                    r.state_bytes() == s.state_bytes()
                        && r.wram() == s.wram()
                        && r.framebuffer_bytes() == s.framebuffer_bytes(),
                    "fixture frame parity"
                );
            }
            let profile = serde_json::to_value(s.coverage())?;
            write(&output.join("profile.json"), &profile)?;
            json!({"status":if table.is_empty(){"uncovered"}else{"pass"},"complete":true,
                "profile":profile,"matched_frames":2,"added_key":[0x8100,3]})
        }
        "paired-cache" => {
            let cache_path = output.join("cache");
            let (pair, created) =
                snes::paired::prepare_cache(&rom, 0, &table, &pads, (0, 3), identities.clone(), &cache_path)?;
            snes::paired::restore(&pair, &pair.identities, &mut r, &mut s)?;
            let warm =
                snes::focused::compare_restored(&mut r, &mut s, &pads, (0, 0, 3), 5, &output.join("warm"), "warm")?;
            let mut cold_r = Reference::new(&rom, 0)?;
            let mut cold_s = Subject::with_steps(&rom, 0, &table)?;
            let cold =
                snes::focused::compare_from_boundary(&mut cold_r, &mut cold_s, &pads, 0, 3, 5, &output.join("cold"))?;
            ensure!(
                warm["status"] == cold["status"] && warm["history"] == cold["history"],
                "warm/cold outcome and trace"
            );
            let (_, reused) =
                snes::paired::prepare_cache(&rom, 0, &table, &pads, (0, 3), identities.clone(), &cache_path)?;
            std::fs::write(cache_path.join("pair.json"), b"truncated")?;
            let (_, corrupt) =
                snes::paired::prepare_cache(&rom, 0, &table, &pads, (0, 3), identities.clone(), &cache_path)?;
            let mut changed = identities.clone();
            changed["source_sha256"] = "f".repeat(64).into();
            let (_, incompatible) = snes::paired::prepare_cache(&rom, 0, &table, &pads, (0, 3), changed, &cache_path)?;
            json!({"status":"pass","complete":true,"created":created,"reused":reused,
                "corrupt":corrupt,"incompatible":incompatible,"warm":warm,"cold":cold})
        }
        _ => anyhow::bail!("unknown feedback case"),
    };
    report["identities"] = identities;
    report["binary_sha256"] = sha256(&std::fs::read(std::env::current_exe()?)?).into();
    write(&output.join("report.json"), &report)?;
    Ok(report)
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(args.len() == 4, "usage: feedback-probe CASE NEW_OUTPUT SOURCE_SHA256 HARDWARE_SHA256");
    let report = execute(&args[0], Path::new(&args[1]), &args[2], &args[3])?;
    if matches!(report["status"].as_str(), Some("divergence" | "uncovered")) {
        std::process::exit(1);
    }
    Ok(())
}
