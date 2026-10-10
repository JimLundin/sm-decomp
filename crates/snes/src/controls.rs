//! Find real executed compiled keys and compare candidate/control intervals.
use crate::{Reference, Subject};
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

/// Discover the first actual generated dispatch within the exact input prefix.
pub fn covered_intervals(
    reference: &mut Reference,
    subject: &mut Subject,
    pads: &[u16],
    frames: usize,
    keys: &[(u32, u8)],
) -> Result<Value> {
    ensure!(frames <= pads.len() && frames > 0, "discovery scope exceeds available input");
    ensure!(keys.iter().collect::<std::collections::BTreeSet<_>>().len() == keys.len(), "duplicate discovery keys");
    if keys.is_empty() {
        return Ok(json!({"schema_version":1,"status":"uncovered","complete":true,
        "scope":"discover covered prefix","expected_frames":frames,"completed_frames":0,"intervals":[],"unexecuted":[]}));
    }
    let mut found = BTreeMap::new();
    let mut completed = 0;
    let mut boundary_divergence = None;
    for (input, pad) in pads[..frames].iter().enumerate() {
        reference.set_input(*pad);
        subject.set_input(*pad);
        let frame = reference.instruction_frame();
        for instruction in 0usize..2_000_000 {
            reference.step_instruction()?;
            let key = subject.step_instruction()?;
            if let Some(key) = key
                && keys.contains(&key)
            {
                found.entry(key).or_insert((input, instruction));
            }
            let reference_frame = reference.instruction_frame();
            let subject_frame = subject.instruction_frame();
            if reference_frame != subject_frame {
                boundary_divergence = Some(
                    json!({"input_index":input,"instruction":instruction,"reference_frame":reference_frame,"subject_frame":subject_frame}),
                );
                break;
            }
            if reference_frame != frame {
                completed = input + 1;
                break;
            }
            if found.len() == keys.len() {
                break;
            }
        }
        if found.len() == keys.len() || boundary_divergence.is_some() {
            break;
        }
    }
    let intervals:Vec<_>=found.iter().map(|(&(address,mx),&(input,instruction))|
        json!({"key":{"address":address,"mx":mx},"start_input":input,"start_instruction":instruction.saturating_sub(64),
               "instructions":65,"first_execution_instruction":instruction})).collect();
    let missing: Vec<_> = keys
        .iter()
        .filter(|key| !found.contains_key(key))
        .map(|&(address, mx)| json!({"address":address,"mx":mx}))
        .collect();
    Ok(
        json!({"schema_version":1,"status":if boundary_divergence.is_some(){"divergence"}else if missing.is_empty(){"pass"}else{"uncovered"},"complete":true,
              "scope":"discover covered prefix","expected_frames":frames,"completed_frames":completed,"boundary_divergence":boundary_divergence,"intervals":intervals,"unexecuted":missing}),
    )
}

pub fn compare_interval(
    reference: &mut Reference,
    subject: &mut Subject,
    pads: &[u16],
    input: usize,
    skip: usize,
    instructions: usize,
    output: &Path,
) -> Result<Value> {
    crate::focused::compare_from_boundary(reference, subject, pads, input, skip, instructions, output)
}
