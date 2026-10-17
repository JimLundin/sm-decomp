//! Find real executed compiled keys and compare candidate/control intervals.
use crate::focused::{Identities, Key, Report};
use crate::{Reference, Subject};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BoundaryDivergence {
    pub input_index: usize,
    pub instruction: usize,
    pub reference_frame: u64,
    pub subject_frame: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Interval {
    pub key: Key,
    pub start_input: usize,
    pub start_instruction: usize,
    pub instructions: usize,
    pub first_execution_instruction: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Discovery {
    pub schema_version: u32,
    pub status: String,
    pub complete: bool,
    pub scope: String,
    pub expected_frames: usize,
    pub completed_frames: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boundary_divergence: Option<BoundaryDivergence>,
    pub intervals: Vec<Interval>,
    pub unexecuted: Vec<Key>,
    pub keys: Vec<Key>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identities: Option<Identities>,
}
use std::{collections::BTreeMap, path::Path};

/// Discover the first actual generated dispatch within the exact input prefix.
pub fn covered_intervals(
    reference: &mut Reference,
    subject: &mut Subject,
    pads: &[u16],
    frames: usize,
    keys: &[(u32, u8)],
) -> Result<Discovery> {
    ensure!(frames <= pads.len() && frames > 0, "discovery scope exceeds available input");
    ensure!(keys.iter().collect::<std::collections::BTreeSet<_>>().len() == keys.len(), "duplicate discovery keys");
    if keys.is_empty() {
        return Ok(Discovery {
            schema_version: 2,
            status: "uncovered".into(),
            complete: true,
            scope: "discover covered prefix".into(),
            expected_frames: frames,
            completed_frames: 0,
            boundary_divergence: None,
            intervals: vec![],
            unexecuted: vec![],
            keys: vec![],
            binary_sha256: None,
            identities: None,
        });
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
                boundary_divergence =
                    Some(BoundaryDivergence { input_index: input, instruction, reference_frame, subject_frame });
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
    let intervals = found
        .iter()
        .map(|(&(address, mx), &(input, instruction))| Interval {
            key: Key { address, mx },
            start_input: input,
            start_instruction: instruction.saturating_sub(64),
            instructions: 65,
            first_execution_instruction: instruction,
        })
        .collect();
    let missing: Vec<_> =
        keys.iter().filter(|key| !found.contains_key(key)).map(|&(address, mx)| Key { address, mx }).collect();
    Ok(Discovery {
        schema_version: 2,
        status: if boundary_divergence.is_some() {
            "divergence"
        } else if missing.is_empty() {
            "pass"
        } else {
            "uncovered"
        }
        .into(),
        complete: true,
        scope: "discover covered prefix".into(),
        expected_frames: frames,
        completed_frames: completed,
        boundary_divergence,
        intervals,
        unexecuted: missing,
        keys: vec![],
        binary_sha256: None,
        identities: None,
    })
}

pub fn compare_interval(
    reference: &mut Reference,
    subject: &mut Subject,
    pads: &[u16],
    input: usize,
    skip: usize,
    instructions: usize,
    output: &Path,
) -> Result<Report> {
    crate::focused::compare_from_boundary(reference, subject, pads, input, skip, instructions, output)
}
