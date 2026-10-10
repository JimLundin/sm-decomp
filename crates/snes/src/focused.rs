//! Cold instruction comparison with bounded first-mismatch context.
use crate::{Reference, Subject, diagnostic::InstructionTrace};
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::VecDeque, path::Path};

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn summary(trace: &InstructionTrace) -> Value {
    json!({"before":trace.before.cpu,"after":trace.after.cpu,"generated_key":trace.generated_key,
        "opcode":trace.events.iter().find(|e|e.kind=="fetch").and_then(|e|e.value),
        "mx":((u8::from(trace.before.cpu.p&0x20!=0))<<1)|u8::from(trace.before.cpu.p&0x10!=0),
        "events":trace.events,"hardware_before_sha256":hash(&trace.before.hardware),"hardware_after_sha256":hash(&trace.after.hardware)})
}
fn first_difference(left: &[u8], right: &[u8]) -> Option<Value> {
    left.iter()
        .zip(right)
        .position(|(a, b)| a != b)
        .map(|index| json!({"offset":index,"reference":left[index],"subject":right[index]}))
        .or_else(|| {
            (left.len() != right.len()).then(|| json!({"reference_bytes":left.len(),"subject_bytes":right.len()}))
        })
}
pub fn compare(
    reference: &mut Reference,
    subject: &mut Subject,
    pads: &[u16],
    start: usize,
    instructions: usize,
    output: &Path,
) -> Result<Value> {
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
) -> Result<Value> {
    ensure!(
        start < pads.len() && instructions > 0,
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
    compare_restored(reference, subject, pads, (input, start, skip), instructions, output, "cold")
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
) -> Result<Value> {
    let (mut input, start, skip) = boundary;
    ensure!(input < pads.len() && instructions > 0, "invalid warm focus scope");
    if !output.exists() {
        std::fs::create_dir(output)?;
    }
    let mut history = VecDeque::new();
    let mut preceding = None;
    let mut report = json!({"schema_version":1,"status":"incomplete","complete":false,"start_input_index":start,
        "start_instruction":skip,"expected_instructions":instructions,"completed_instructions":0,"replay":replay,"observer":"passive actual effects", "history_limit":64});
    std::fs::write(output.join("instruction.json"), serde_json::to_vec_pretty(&report)?)?;
    for index in 0..instructions {
        let before_reference = reference.instruction_snapshot();
        let before_subject = subject.instruction_snapshot();
        if before_reference != before_subject {
            for (name, snapshot) in [("reference-before", &before_reference), ("subject-before", &before_subject)] {
                std::fs::write(output.join(format!("{name}.hardware.bin")), &snapshot.hardware)?;
                std::fs::write(output.join(format!("{name}.cpu.bin")), &snapshot.pending_cpu)?;
            }
            report["status"] = "divergence".into();
            report["complete"] = true.into();
            report["first_mismatch"] = json!({"instruction":index,"phase":"before execution","input_index":input,"pad":pads[input],
                "reference":{"before":before_reference.cpu},"subject":{"before":before_subject.cpu}});
            report["preceding_matching_boundary"] = preceding.unwrap_or(Value::Null);
            report["history"] = serde_json::to_value(&history)?;
            report["hardware_difference"] =
                first_difference(&before_reference.hardware, &before_subject.hardware).unwrap_or(Value::Null);
            report["pending_cpu_difference"] =
                first_difference(&before_reference.pending_cpu, &before_subject.pending_cpu).unwrap_or(Value::Null);
            std::fs::write(output.join("instruction.json"), serde_json::to_vec_pretty(&report)?)?;
            return Ok(report);
        }
        let a = reference.step_instruction_observed()?;
        let b = subject.step_instruction_observed()?;
        let mismatch = a.before != b.before || a.after != b.after || a.events != b.events;
        let paired = json!({"instruction":index,"input_index":input,"pad":pads[input],"reference":summary(&a),"subject":summary(&b)});
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
            report["status"] = "divergence".into();
            report["complete"] = true.into();
            report["completed_instructions"] = (index + 1).into();
            report["first_mismatch"] = paired;
            report["preceding_matching_boundary"] = preceding.unwrap_or(Value::Null);
            report["hardware_difference"] =
                first_difference(&a.after.hardware, &b.after.hardware).unwrap_or(Value::Null);
            report["pending_cpu_difference"] =
                first_difference(&a.after.pending_cpu, &b.after.pending_cpu).unwrap_or(Value::Null);
            report["history"] = serde_json::to_value(&history)?;
            report["first_bus_difference"] = a
                .events
                .iter()
                .zip(&b.events)
                .position(|(a, b)| a != b)
                .map(|index| json!({"index":index,"reference":a.events[index],"subject":b.events[index]}))
                .unwrap_or_else(|| {
                    if a.events.len() != b.events.len() {
                        json!({"reference_events":a.events.len(),"subject_events":b.events.len()})
                    } else {
                        Value::Null
                    }
                });
            std::fs::write(output.join("instruction.json"), serde_json::to_vec_pretty(&report)?)?;
            return Ok(report);
        }
        preceding = Some(json!({"reference":a.after.cpu,"subject":b.after.cpu}));
        history.push_back(paired);
        if history.len() > 64 {
            history.pop_front();
        }
        if a.after.cpu.frame != a.before.cpu.frame && index + 1 < instructions {
            input += 1;
            ensure!(input < pads.len(), "focused replay exhausted input");
            reference.set_input(pads[input]);
            subject.set_input(pads[input]);
        }
        report["completed_instructions"] = (index + 1).into();
    }
    report["status"] = "pass".into();
    report["complete"] = true.into();
    report["history"] = serde_json::to_value(&history)?;
    std::fs::write(output.join("instruction.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(report)
}
