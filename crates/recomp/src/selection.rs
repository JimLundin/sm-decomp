//! Measured routine/width selection through the same CFG and timed emitter as production.
use crate::{cfg, config, decode, emit, rom::Rom, step};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
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
) -> Result<Value> {
    let profile: Value = serde_json::from_slice(&std::fs::read(profile)?)?;
    let inventory: Value = serde_json::from_slice(&std::fs::read(inventory)?)?;
    let mut counts = BTreeMap::new();
    for row in profile["interpreted"].as_array().context("interpreted instruction counts required")? {
        if row["emulation"] == false {
            let key = (
                row["address"].as_u64().context("address")? as u32,
                row["mx"].as_u64().context("width")? as u8,
                row["opcode"].as_u64().context("opcode")? as u8,
            );
            *counts.entry(key).or_insert(0u64) += row["count"].as_u64().context("count")?;
        }
    }
    let existing: BTreeSet<_> = inventory["keys"]
        .as_array()
        .context("generated keys required")?
        .iter()
        .map(|row| {
            Ok((row["address"].as_u64().context("key address")? as u32, row["mx"].as_u64().context("key width")? as u8))
        })
        .collect::<Result<_>>()?;
    let mut candidates = BTreeSet::new();
    for line in std::fs::read_to_string(discoveries)?.lines().skip(1).filter(|line| !line.trim().is_empty()) {
        let columns: Vec<_> = line.split(',').collect();
        ensure!(columns.len() == 3, "invalid call-target discovery row");
        let address = config::parse_addr(columns[0])?;
        let (m8, x8) = config::parse_variant(columns[1])?;
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
        if configured.contains(&(address, mx)) || overrides.contains(&address) {
            continue;
        }
        // Calls discover a root; their frequency never contributes to estimated gain.
        let graph = match cfg::build(&read, address, mx & 2 != 0, mx & 1 != 0, &boundaries) {
            Ok(graph) => graph,
            Err(error) => {
                rejected.push(json!({"address":address,"mx":mx,"reason":error.to_string()}));
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
            rejected.push(
                json!({"address":address,"mx":mx,"reason":"routine exceeds bounded slice (128 blocks/1024 keys)"}),
            );
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
            let key = json!({"address":key_address,"mx":key_mx,"opcode":instruction.op,"bytes":bytes,
                             "pattern":format!("{}/{:?}",instruction.mnem,instruction.mode),"count":count});
            if existing.contains(&(key_address, key_mx)) {
                overlaps.push(key);
            } else if count > 0 {
                gain += count;
                executed.push(key);
            } else {
                deferred.push(key);
            }
        }
        if gain == 0 {
            rejected.push(json!({"address":address,"mx":mx,"reason":"no newly compilable executed keys"}));
            continue;
        }
        ranked.push(json!({"address":address,"mx":mx,"variant":format!("m{}x{}",mx >> 1,mx & 1),
                           "estimated_compiled_step_gain":gain,"complexity":{"blocks":graph.blocks.len(),"instruction_keys":unique.len()},
                           "new_executed_keys":executed,"deferred_keys":deferred,"overlapping_keys":overlaps}));
    }
    ranked.sort_by_key(|row| {
        (
            std::cmp::Reverse(row["estimated_compiled_step_gain"].as_u64().unwrap()),
            row["complexity"]["instruction_keys"].as_u64().unwrap(),
            row["complexity"]["blocks"].as_u64().unwrap(),
            row["address"].as_u64().unwrap(),
            row["mx"].as_u64().unwrap(),
        )
    });
    let mut unsupported = BTreeMap::<String, u64>::new();
    for (&(address, mx, opcode), &count) in &counts {
        if let Some(instruction) = decode::decode(rom, address, mx & 2 != 0, mx & 1 != 0)
            && instruction.op == opcode
            && !step::supported(&instruction)
        {
            *unsupported.entry(format!("{}/{:?}", instruction.mnem, instruction.mode)).or_default() += count;
        }
    }
    let mut patterns: Vec<_> =
        unsupported.into_iter().map(|(pattern, count)| json!({"pattern":pattern,"count":count})).collect();
    patterns.sort_by_key(|row| {
        (std::cmp::Reverse(row["count"].as_u64().unwrap()), row["pattern"].as_str().unwrap().to_string())
    });
    Ok(
        json!({"schema_version":1,"outcome":if !ranked.is_empty(){"supported-routine"}else if !patterns.is_empty(){"needs-timed-pattern"}else{"no-covered-candidate"},
              "chosen":ranked.first(),"ranked":ranked,"rejected":rejected,"observed_unsupported_patterns":patterns,
              "ranking_policy":"descending newly compiled instruction count; exact equal gains prefer fewer CFG keys, fewer blocks, address, MX",
              "scope_policy":"ROM-backed routine CFG; at most 128 blocks/1024 keys; execution outside accepted prefix requires independent domain validation"}),
    )
}
