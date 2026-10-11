//! Cold instruction comparison with bounded first-mismatch context.
use crate::diagnostic::{BusEvent, CpuBoundary};
use crate::{Reference, Subject, diagnostic::InstructionTrace};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::VecDeque, path::Path};

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Summary {
    pub before: CpuBoundary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<CpuBoundary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_key: Option<(u32, u8)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opcode: Option<u8>,
    pub mx: u8,
    pub events: Vec<BusEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardware_before_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardware_after_sha256: Option<String>,
}
impl Summary {
    fn boundary(before: CpuBoundary) -> Self {
        Self {
            mx: ((u8::from(before.p & 0x20 != 0)) << 1) | u8::from(before.p & 0x10 != 0),
            before,
            after: None,
            generated_key: None,
            opcode: None,
            events: vec![],
            hardware_before_sha256: None,
            hardware_after_sha256: None,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PairedTrace {
    pub instruction: usize,
    pub input_index: usize,
    pub pad: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    pub reference: Summary,
    pub subject: Summary,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MatchingBoundary {
    pub reference: CpuBoundary,
    pub subject: CpuBoundary,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ByteDifference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_bytes: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_bytes: Option<usize>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BusDifference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<BusEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<BusEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_events: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_events: Option<usize>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Identities {
    pub rom_sha256: String,
    pub movie_sha256: String,
    pub input_sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Key {
    pub address: u32,
    pub mx: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Report {
    pub schema_version: u32,
    pub status: String,
    pub complete: bool,
    pub start_input_index: usize,
    pub start_instruction: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_instructions: Option<usize>,
    pub completed_instructions: usize,
    pub replay: String,
    pub observer: String,
    pub history_limit: usize,
    pub scope: String,
    pub instruction_budget: usize,
    pub observed_rollovers: usize,
    pub history: Vec<PairedTrace>,
    pub executed_generated_keys: Vec<(u32, u8)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_mismatch: Option<PairedTrace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preceding_matching_boundary: Option<MatchingBoundary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardware_difference: Option<ByteDifference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_cpu_difference: Option<ByteDifference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_bus_difference: Option<BusDifference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warm_replay_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<crate::paired::CacheReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_pair_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_history_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identities: Option<Identities>,
    pub disabled_keys: Vec<Key>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_sha256: Option<String>,
    pub rerun: Vec<String>,
}
fn summary(trace: &InstructionTrace) -> Summary {
    Summary {
        before: trace.before.cpu.clone(),
        after: Some(trace.after.cpu.clone()),
        generated_key: trace.generated_key,
        opcode: trace.events.iter().find(|e| e.kind == "fetch").and_then(|e| e.value),
        mx: ((u8::from(trace.before.cpu.p & 0x20 != 0)) << 1) | u8::from(trace.before.cpu.p & 0x10 != 0),
        events: trace.events.clone(),
        hardware_before_sha256: Some(hash(&trace.before.hardware)),
        hardware_after_sha256: Some(hash(&trace.after.hardware)),
    }
}
fn first_difference(left: &[u8], right: &[u8]) -> Option<ByteDifference> {
    left.iter()
        .zip(right)
        .position(|(a, b)| a != b)
        .map(|index| ByteDifference {
            offset: Some(index),
            reference: Some(left[index]),
            subject: Some(right[index]),
            reference_bytes: None,
            subject_bytes: None,
        })
        .or_else(|| {
            (left.len() != right.len()).then_some(ByteDifference {
                offset: None,
                reference: None,
                subject: None,
                reference_bytes: Some(left.len()),
                subject_bytes: Some(right.len()),
            })
        })
}
fn save(output: &Path, report: &Report) -> Result<()> {
    pipeline_model::write(&output.join("instruction.toml"), report)
}
pub fn compare(
    reference: &mut Reference,
    subject: &mut Subject,
    pads: &[u16],
    start: usize,
    instructions: usize,
    output: &Path,
) -> Result<Report> {
    compare_from_boundary(reference, subject, pads, start, 0, instructions, output)
}

/// Focus after an exact cold prefix and a bounded number of unobserved instructions.
pub fn compare_from_boundary(
    reference: &mut Reference,
    subject: &mut Subject,
    pads: &[u16],
    start: usize,
    skip: usize,
    instructions: usize,
    output: &Path,
) -> Result<Report> {
    compare_cold(reference, subject, pads, (start, skip), Some(instructions), output)
}

/// Observe every instruction in one recorded failed input frame. This cold
/// path needs no paired cache whose validation might mask the actual mismatch.
pub fn compare_frame_from_boundary(
    reference: &mut Reference,
    subject: &mut Subject,
    pads: &[u16],
    start: usize,
    output: &Path,
) -> Result<Report> {
    compare_cold(reference, subject, pads, (start, 0), None, output)
}

fn compare_cold(
    reference: &mut Reference,
    subject: &mut Subject,
    pads: &[u16],
    boundary: (usize, usize),
    instructions: Option<usize>,
    output: &Path,
) -> Result<Report> {
    let (start, skip) = boundary;
    ensure!(
        start < pads.len() && instructions.is_none_or(|count| count > 0),
        "focused scope requires existing input and positive instruction count"
    );
    std::fs::create_dir(output)?;
    for pad in &pads[..start] {
        reference.run_frame(*pad)?;
        subject.run_frame(*pad)?;
        ensure!(
            reference.state_bytes() == subject.state_bytes()
                && reference.wram() == subject.wram()
                && reference.framebuffer_bytes() == subject.framebuffer_bytes(),
            "Cold prefix diverged before selected focus; rerun from an earlier input frame"
        );
    }
    reference.set_input(pads[start]);
    subject.set_input(pads[start]);
    let mut input = start;
    for _ in 0..skip {
        let frame = reference.instruction_snapshot().cpu.frame;
        reference.step_instruction()?;
        subject.step_instruction()?;
        if reference.instruction_snapshot().cpu.frame != frame {
            input += 1;
            ensure!(input < pads.len(), "focused prefix exhausted input");
            reference.set_input(pads[input]);
            subject.set_input(pads[input]);
        }
    }
    compare_scope(reference, subject, pads, (input, start, skip), instructions, output, "cold")
}

/// Compare from an already restored exact paired boundary without replaying its prefix.
pub fn compare_restored(
    reference: &mut Reference,
    subject: &mut Subject,
    pads: &[u16],
    boundary: (usize, usize, usize),
    instructions: usize,
    output: &Path,
    replay: &str,
) -> Result<Report> {
    compare_scope(reference, subject, pads, boundary, Some(instructions), output, replay)
}

fn compare_scope(
    reference: &mut Reference,
    subject: &mut Subject,
    pads: &[u16],
    boundary: (usize, usize, usize),
    instructions: Option<usize>,
    output: &Path,
    replay: &str,
) -> Result<Report> {
    let (mut input, start, skip) = boundary;
    ensure!(input < pads.len() && instructions.is_none_or(|count| count > 0), "invalid warm focus scope");
    let limit = instructions.unwrap_or(2_000_000);
    if !output.exists() {
        std::fs::create_dir(output)?;
    }
    let mut history = VecDeque::new();
    let mut preceding = None;
    let mut executed_generated_keys = std::collections::BTreeSet::new();
    let mut report = Report {
        schema_version: 2,
        status: "incomplete".into(),
        complete: false,
        start_input_index: start,
        start_instruction: skip,
        expected_instructions: instructions,
        completed_instructions: 0,
        replay: replay.into(),
        observer: "passive actual effects".into(),
        history_limit: 64,
        scope: if instructions.is_none() { "until-frame-end" } else { "instruction-count" }.into(),
        instruction_budget: limit,
        observed_rollovers: 0,
        history: vec![],
        executed_generated_keys: vec![],
        first_mismatch: None,
        preceding_matching_boundary: None,
        hardware_difference: None,
        pending_cpu_difference: None,
        first_bus_difference: None,
        reason: None,
        warm_replay_seconds: None,
        cache: None,
        cache_pair_sha256: None,
        cache_history_sha256: None,
        identities: None,
        disabled_keys: vec![],
        binary_sha256: None,
        rerun: vec![],
    };
    save(output, &report)?;
    for index in 0..limit {
        let before_reference = reference.instruction_snapshot();
        let before_subject = subject.instruction_snapshot();
        if before_reference != before_subject {
            for (name, snapshot) in [("reference-before", &before_reference), ("subject-before", &before_subject)] {
                std::fs::write(output.join(format!("{name}.hardware.bin")), &snapshot.hardware)?;
                std::fs::write(output.join(format!("{name}.cpu.bin")), &snapshot.pending_cpu)?;
            }
            report.status = "divergence".into();
            report.complete = true;
            report.first_mismatch = Some(PairedTrace {
                instruction: index,
                phase: Some("before execution".into()),
                input_index: input,
                pad: pads[input],
                reference: Summary::boundary(before_reference.cpu),
                subject: Summary::boundary(before_subject.cpu),
            });
            report.preceding_matching_boundary = preceding;
            report.history = history.into_iter().collect();
            report.hardware_difference = first_difference(&before_reference.hardware, &before_subject.hardware);
            report.pending_cpu_difference =
                first_difference(&before_reference.pending_cpu, &before_subject.pending_cpu);
            save(output, &report)?;
            return Ok(report);
        }
        let a = reference.step_instruction_observed()?;
        let b = subject.step_instruction_observed()?;
        if let Some(key) = b.generated_key {
            executed_generated_keys.insert(key);
        }
        let mismatch = a.before != b.before || a.after != b.after || a.events != b.events;
        let paired = PairedTrace {
            instruction: index,
            input_index: input,
            pad: pads[input],
            phase: None,
            reference: summary(&a),
            subject: summary(&b),
        };
        if mismatch {
            for (name, snapshot) in [
                ("reference-before", &a.before),
                ("subject-before", &b.before),
                ("reference-after", &a.after),
                ("subject-after", &b.after),
            ] {
                std::fs::write(output.join(format!("{name}.hardware.bin")), &snapshot.hardware)?;
                std::fs::write(output.join(format!("{name}.cpu.bin")), &snapshot.pending_cpu)?;
            }
            report.status = "divergence".into();
            report.complete = true;
            report.completed_instructions = index + 1;
            report.first_mismatch = Some(paired);
            report.preceding_matching_boundary = preceding;
            report.hardware_difference = first_difference(&a.after.hardware, &b.after.hardware);
            report.pending_cpu_difference = first_difference(&a.after.pending_cpu, &b.after.pending_cpu);
            report.history = history.into_iter().collect();
            report.first_bus_difference = a
                .events
                .iter()
                .zip(&b.events)
                .position(|(a, b)| a != b)
                .map(|index| BusDifference {
                    index: Some(index),
                    reference: Some(a.events[index].clone()),
                    subject: Some(b.events[index].clone()),
                    reference_events: None,
                    subject_events: None,
                })
                .or_else(|| {
                    (a.events.len() != b.events.len()).then_some(BusDifference {
                        index: None,
                        reference: None,
                        subject: None,
                        reference_events: Some(a.events.len()),
                        subject_events: Some(b.events.len()),
                    })
                });
            save(output, &report)?;
            return Ok(report);
        }
        preceding = Some(MatchingBoundary { reference: a.after.cpu.clone(), subject: b.after.cpu.clone() });
        history.push_back(paired);
        if history.len() > 64 {
            history.pop_front();
        }
        if a.after.cpu.frame != a.before.cpu.frame {
            report.observed_rollovers += 1;
            if instructions.is_none() {
                report.completed_instructions = index + 1;
                break;
            }
        }
        if a.after.cpu.frame != a.before.cpu.frame && index + 1 < limit {
            input += 1;
            ensure!(input < pads.len(), "focused replay exhausted input");
            reference.set_input(pads[input]);
            subject.set_input(pads[input]);
        }
        report.completed_instructions = index + 1;
    }
    report.executed_generated_keys = executed_generated_keys.into_iter().collect();
    if instructions.is_none() && report.observed_rollovers != 1 {
        report.reason = Some("Instruction budget exhausted before failed frame rollover".into());
        save(output, &report)?;
        return Ok(report);
    }
    report.status = "pass".into();
    report.complete = true;
    report.history = history.into_iter().collect();
    save(output, &report)?;
    Ok(report)
}
