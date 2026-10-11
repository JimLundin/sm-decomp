//! Measured routine/width selection through the same CFG and timed emitter as production.
use crate::{cfg, config, decode, emit, rom::Rom, step};
use anyhow::{Context, Result, ensure};
use pipeline_model::{
    Candidate, Complexity, InstructionProfile, KeyInventory, RejectedCandidate, SelectedKey, SelectionAction,
    SelectionOutcome, SelectionReport, UnsupportedPattern,
};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    path::Path,
};

pub fn select(
    rom: &Rom,
    functions: &config::Functions,
    profile: &Path,
    discoveries: &Path,
    inventory: &Path,
) -> Result<SelectionReport> {
    let profile: InstructionProfile = pipeline_model::read(profile)?;
    profile.validate()?;
    let inventory: KeyInventory = pipeline_model::read(inventory)?;
    inventory.validate()?;
    for key in &inventory.keys {
        ensure!(
            key.bytes.iter().enumerate().all(|(offset, &byte)| {
                let address = (key.address & 0xff0000) | ((key.address + offset as u32) & 0xffff);
                rom.byte(address) == Some(byte)
            }),
            "baseline inventory bytes differ from ROM"
        );
    }
    let mut counts = BTreeMap::new();
    for row in &profile.interpreted {
        if !row.emulation {
            ensure!(rom.byte(row.address) == Some(row.opcode), "observed opcode differs from ROM");
            let key = (row.address, row.mx, row.opcode);
            let count = counts.entry(key).or_insert(0u64);
            *count = count.checked_add(row.count).context("selection count overflow")?;
        }
    }
    let existing: BTreeSet<_> = inventory.keys.iter().map(|row| (row.address, row.mx)).collect();
    let mut candidates = BTreeSet::new();
    for line in std::fs::read_to_string(discoveries)?.lines().skip(1).filter(|line| !line.trim().is_empty()) {
        let columns: Vec<_> = line.split(',').collect();
        ensure!(columns.len() == 3, "invalid call-target discovery row");
        let address = config::parse_addr(columns[0])?;
        let (m8, x8) = config::parse_variant(columns[1])?;
        columns[2].parse::<u64>().context("invalid call-target discovery count")?;
        candidates.insert((address, emit::mx_key(m8, x8)));
    }
    // Declared routines remain useful discovery roots even when no direct call was logged.
    let mut configured = BTreeSet::new();
    let mut boundaries = HashSet::new();
    let mut overrides = HashSet::new();
    for function in &functions.functions {
        let address = config::parse_addr(&function.addr)?;
        boundaries.insert(address);
        if function.override_ {
            overrides.insert(address);
        }
        for variant in &function.variants {
            let (m8, x8) = config::parse_variant(variant)?;
            let key = (address, emit::mx_key(m8, x8));
            configured.insert(key);
            candidates.insert(key);
        }
    }
    let read = |address| rom.byte(address);
    let mut ranked = Vec::new();
    let mut rejected = Vec::new();
    for (address, mx) in candidates {
        if overrides.contains(&address) {
            continue;
        }
        // Calls discover a root; their frequency never contributes to estimated gain.
        let graph = match cfg::build(&read, address, mx & 2 != 0, mx & 1 != 0, &boundaries) {
            Ok(graph) => graph,
            Err(error) => {
                rejected.push(RejectedCandidate { address, mx, reason: error.to_string() });
                continue;
            }
        };
        let unique: BTreeMap<_, _> = graph
            .blocks
            .iter()
            .flat_map(|block| &block.insns)
            .map(|instruction| ((instruction.addr, emit::mx_key(instruction.m8, instruction.x8)), instruction))
            .collect();
        if graph.blocks.len() > 128 || unique.len() > 1024 {
            rejected.push(RejectedCandidate {
                address,
                mx,
                reason: "routine exceeds bounded slice (128 blocks/1024 keys)".into(),
            });
            continue;
        }
        let mut executed = Vec::new();
        let mut deferred = Vec::new();
        let mut overlaps = Vec::new();
        let mut gain = 0u64;
        for (&(key_address, key_mx), instruction) in &unique {
            if !step::supported(instruction) {
                continue;
            }
            let bytes: Vec<_> = (0..instruction.len)
                .map(|offset| read((key_address & 0xff0000) | ((key_address + offset) & 0xffff)).unwrap())
                .collect();
            let count = counts.get(&(key_address, key_mx, instruction.op)).copied().unwrap_or(0);
            let key = SelectedKey {
                address: key_address,
                mx: key_mx,
                opcode: instruction.op,
                bytes,
                pattern: format!("{}/{:?}", instruction.mnem, instruction.mode),
                count,
            };
            if existing.contains(&(key_address, key_mx)) {
                overlaps.push(key);
            } else if count > 0 {
                gain = gain.checked_add(count).context("selection gain overflow")?;
                executed.push(key);
            } else {
                deferred.push(key);
            }
        }
        if gain == 0 {
            rejected.push(RejectedCandidate { address, mx, reason: "no newly compilable executed keys".into() });
            continue;
        }
        ranked.push(Candidate {
            address,
            mx,
            variant: format!("m{}x{}", mx >> 1, mx & 1),
            action: if configured.contains(&(address, mx)) {
                SelectionAction::EnableDeferredKeys
            } else {
                SelectionAction::AddRoutineWidth
            },
            estimated_compiled_step_gain: gain,
            complexity: Complexity { blocks: graph.blocks.len(), instruction_keys: unique.len() },
            new_executed_keys: executed,
            deferred_keys: deferred,
            overlapping_keys: overlaps,
        });
    }
    ranked.sort_by_key(Candidate::order);
    let mut unsupported = BTreeMap::<String, u64>::new();
    for (&(address, mx, opcode), &count) in &counts {
        if let Some(instruction) = decode::decode(rom, address, mx & 2 != 0, mx & 1 != 0)
            && instruction.op == opcode
            && !step::supported(&instruction)
        {
            let total = unsupported.entry(format!("{}/{:?}", instruction.mnem, instruction.mode)).or_default();
            *total = total.checked_add(count).context("unsupported count overflow")?;
        }
    }
    let mut patterns: Vec<_> =
        unsupported.into_iter().map(|(pattern, count)| UnsupportedPattern { pattern, count }).collect();
    patterns.sort_by_key(|row| (std::cmp::Reverse(row.count), row.pattern.clone()));
    let report=SelectionReport{schema_version:1,outcome:if !ranked.is_empty(){SelectionOutcome::SupportedRoutine}else if !patterns.is_empty(){SelectionOutcome::NeedsTimedPattern}else{SelectionOutcome::NoCoveredCandidate},chosen:ranked.first().cloned(),ranked,rejected,observed_unsupported_patterns:patterns,ranking_policy:"descending newly compiled instruction count; exact equal gains prefer fewer CFG keys, fewer blocks, address, MX".into(),scope_policy:"ROM-backed routine CFG; at most 128 blocks/1024 keys; execution outside accepted prefix requires independent domain validation".into()};
    report.validate()?;
    Ok(report)
}
