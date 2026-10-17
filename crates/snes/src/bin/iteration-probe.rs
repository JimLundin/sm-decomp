//! Retained late-occurrence candidate and matching controls, synthetic only.
use anyhow::{Result, ensure};
use serde::Serialize;
#[derive(Serialize)]
struct Expectation {
    source: String,
    reference_write: u8,
    candidate_write: u8,
}
#[derive(Serialize)]
struct Report {
    schema_version: u32,
    scope: String,
    status: String,
    complete: bool,
    accepted_count: u64,
    source_sha256: String,
    binary_sha256: String,
    rom_sha256: String,
    input_sha256: String,
    early: snes::focused::Report,
    candidate: snes::focused::Report,
    predecessor: snes::focused::Report,
    disabled: snes::focused::Report,
    cache: snes::paired::CacheReport,
    warm: snes::focused::Report,
    altered_key: (u32, u8),
    independent_expectation: Expectation,
}
use snes::{Reference, Subject, paired::sha256};
use std::path::Path;

mod emitted {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/feedback_steps.rs"));
}
fn wrong_late(m: &mut runtime::Machine) {
    let value = m.cpu.a as u8;
    m.write8(0x10, if value == 0 { 0x99 } else { value });
}
fn execute(output: &Path, source: &str) -> Result<()> {
    std::fs::create_dir(output)?;
    let mut rom = vec![0; 0x10000];
    let mut program = vec![0x18, 0xfb, 0xa9, 0x12, 0x8d, 0x10, 0];
    program.extend(std::iter::repeat_n(0xea, 80));
    program.extend([0xa9, 0, 0x82, 0xa8, 0xff]);
    rom[..program.len()].copy_from_slice(&program);
    rom[0x7fc0..0x7fd5].copy_from_slice(b"SM ITERATION FIXTURE ");
    rom[0x7fd5] = 0x20;
    rom[0x7fd9] = 1;
    rom[0x7fdc..0x7fe0].copy_from_slice(&[0xcb, 0xed, 0x34, 0x12]);
    rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
    let correct = emitted::STEP_TABLE.iter().copied().find(|row| row.0 == 0x8004 && row.1 == 3).unwrap();
    let candidate = [(correct.0, correct.1, correct.2, wrong_late as runtime::StepFn)];
    let pads = [0, 0];
    std::fs::write(output.join("fixture.sfc"), &rom)?;
    std::fs::write(output.join("fixture.inputs"), movie::write_inputs(&pads))?;
    std::fs::write(output.join("production-steps.rs"), include_str!(concat!(env!("OUT_DIR"), "/feedback_steps.rs")))?;
    let early = snes::focused::compare(
        &mut Reference::new(&rom, 0)?,
        &mut Subject::with_steps(&rom, 0, &candidate)?,
        &pads,
        0,
        65,
        &output.join("early"),
    )?;
    let late = snes::focused::compare_frame_from_boundary(
        &mut Reference::new(&rom, 0)?,
        &mut Subject::with_steps(&rom, 0, &candidate)?,
        &pads,
        0,
        &output.join("candidate"),
    )?;
    ensure!(
        early.status == "pass"
            && late.status == "divergence"
            && late.first_mismatch.as_ref().is_some_and(|trace| trace.instruction == 86),
        "late occurrence not localized"
    );
    let count = late.completed_instructions;
    let predecessor = snes::focused::compare(
        &mut Reference::new(&rom, 0)?,
        &mut Subject::with_steps(&rom, 0, &[correct])?,
        &pads,
        0,
        count,
        &output.join("predecessor"),
    )?;
    let disabled = snes::focused::compare(
        &mut Reference::new(&rom, 0)?,
        &mut Subject::with_steps(&rom, 0, &[])?,
        &pads,
        0,
        count,
        &output.join("disabled"),
    )?;
    ensure!(predecessor.status == "pass" && disabled.status == "pass", "matching controls fail");
    let ids = pipeline_model::CheckpointIdentities {
        rom_sha256: sha256(&rom),
        movie_sha256: sha256(movie::write_inputs(&pads).as_bytes()),
        input_sha256: sha256(movie::write_inputs(&pads).as_bytes()),
        source_sha256: source.into(),
        hardware_sha256: sha256(include_bytes!("../paired.rs")),
        profile: "lsnes-profile".into(),
        serializer: "paired-v2".into(),
        compiler: "iteration-probe".into(),
        features: vec!["lsnes-profile".into()],
        settings: pipeline_model::CheckpointSettings {
            seed: 0,
            mode: None,
            candidate_sha256: None,
            disabled_keys: vec![],
        },
        dispatch_sha256: None,
    };
    let (pair, cache) =
        snes::paired::prepare_cache(&rom, 0, &candidate, &pads, (0, 0), ids, &output.join("divergent-cache"))?;
    ensure!(
        cache.validation.complete
            && cache.validation.observed_rollovers == 2
            && cache.validation.candidate_reference_comparison == "divergence",
        "cache did not record actual next-frame divergence"
    );
    let mut warm_r = Reference::new(&rom, 0)?;
    let mut warm_s = Subject::with_steps(&rom, 0, &candidate)?;
    snes::paired::restore(&pair, &pair.identities, &mut warm_r, &mut warm_s)?;
    let warm = snes::focused::compare_restored(
        &mut warm_r,
        &mut warm_s,
        &pads,
        (0, 0, 0),
        count,
        &output.join("warm"),
        "warm",
    )?;
    ensure!(
        warm.status == "divergence" && warm.first_mismatch.as_ref().is_some_and(|trace| trace.instruction == 86),
        "warm cache hid candidate divergence"
    );
    let report = Report {
        schema_version: 3,
        scope: "synthetic late occurrence; acceptance not certified".into(),
        status: "pass".into(),
        complete: true,
        accepted_count: 0,
        source_sha256: source.into(),
        binary_sha256: sha256(&std::fs::read(std::env::current_exe()?)?),
        rom_sha256: sha256(&rom),
        input_sha256: sha256(movie::write_inputs(&pads).as_bytes()),
        early,
        candidate: late,
        predecessor,
        disabled,
        cache,
        warm,
        altered_key: (0x8004, 3),
        independent_expectation: Expectation {
            source: "literal cartridge instructions and production emitted correct store".into(),
            reference_write: 0,
            candidate_write: 0x99,
        },
    };
    pipeline_model::write(&output.join("report.toml"), &report)?;
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(args.len() == 2, "usage: iteration-probe NEW_OUTPUT SOURCE_SHA256");
    execute(Path::new(&args[0]), &args[1])
}
