//! Validate observed completion and invocation evidence, never requested length.
use crate::{Result, Table, movie};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

const HEADER: &str = "frame,event,kind,address,value,call,game_state,room,x,y,held,pressed,projectile_count,slot,type,instruction,interrupts";

#[derive(Debug, PartialEq, Eq)]
struct CallContext {
    entry_s: i64,
    return_s: i64,
    continuation: i64,
    saved_status: i64,
}
struct ActiveCall {
    id: u64,
    kind: String,
    context: CallContext,
    exit: Option<u64>,
    interrupts: i64,
}
fn allowed_exit(kind: &str, address: u64) -> bool {
    match kind {
        "fire" => [0x90b89d, 0x90b8a9, 0x90b96d].contains(&address),
        "handle" => address == 0x90aef2,
        "draw-player" => address == 0x9382fc,
        "draw-bomb" => address == 0x9383be,
        _ => false,
    }
}

fn completion(path: &Path) -> Result<BTreeMap<String, u64>> {
    let text = fs::read_to_string(path).map_err(|e| format!("missing observed frame completion: {e}"))?;
    let mut result = BTreeMap::new();
    for line in text.lines() {
        let (key, value) = line.split_once('=').ok_or("invalid completion field")?;
        let value = value.parse().map_err(|_| "invalid completion number")?;
        if result.insert(key.to_owned(), value).is_some() {
            return Err("duplicate completion field".into());
        }
    }
    for (key, expected) in [
        ("completed_frames", movie::FRAMES as u64),
        ("last_frame", movie::FRAMES as u64),
        ("active_calls", 0),
        ("shot_created", 1),
        ("shot_impact", 1),
        ("shot_removed", 1),
        ("failed", 0),
    ] {
        if result.get(key) != Some(&expected) {
            return Err(format!("observed completion requirement failed: {key}"));
        }
    }
    Ok(result)
}
fn cpu(path: &Path, event: u64, address: u64, call: u64, phase: &str) -> Result<(CallContext, i64, i64)> {
    let stem = path.join(format!("states/state-{event}"));
    let ram = stem.with_extension("wram");
    if fs::symlink_metadata(&ram).map_err(|e| e.to_string())?.file_type().is_symlink()
        || fs::metadata(&ram).map_err(|e| e.to_string())?.len() != 131072
    {
        return Err(format!("invalid WRAM snapshot for event {event}"));
    }
    let text = fs::read_to_string(stem.with_extension("cpu.toml")).map_err(|e| e.to_string())?;
    let value: toml::Value = toml::from_str(&text).map_err(|e| e.to_string())?;
    let number =
        |key: &str| value.get(key).and_then(toml::Value::as_integer).ok_or_else(|| format!("missing CPU field {key}"));
    if value.get("schema").and_then(toml::Value::as_str) != Some("original-cpu-context-v1")
        || value.get("phase").and_then(toml::Value::as_str) != Some(phase)
        || number("boundary_pc")? != address as i64
        || number("call")? != call as i64
        || number("pc")? != ((address + 1) & 0xffff) as i64
        || number("pb")? != (address >> 16) as i64
        || number("e")? != 0
    {
        return Err(format!("CPU call context differs at event {event}"));
    }
    for key in [
        "a",
        "x",
        "y",
        "s",
        "d",
        "db",
        "p",
        "entry_s",
        "expected_return_s",
        "continuation_pc",
        "exit_pc",
        "interrupts",
        "saved_status",
    ] {
        number(key)?;
    }
    if phase == "entry" && number("s")? != number("entry_s")? {
        return Err("entry stack context differs".into());
    }
    if phase == "return"
        && (number("s")? != number("expected_return_s")?
            || number("exit_pc")? == 0
            || number("continuation_pc")? != address as i64)
    {
        return Err("return stack or exit context differs".into());
    }
    Ok((
        CallContext {
            entry_s: number("entry_s")?,
            return_s: number("expected_return_s")?,
            continuation: number("continuation_pc")?,
            saved_status: number("saved_status")?,
        },
        number("exit_pc")?,
        number("interrupts")?,
    ))
}

pub fn capture(out: &Path) -> Result<Table> {
    ledger(out, false)
}

/// Validate only reset through the first completed Fire invocation. A later
/// observer failure and unfinished later calls do not become lifecycle proof.
pub fn first_fire_prefix(out: &Path) -> Result<Table> {
    ledger(out, true)
}

fn ledger(out: &Path, first_fire_only: bool) -> Result<Table> {
    if !first_fire_only && out.join("failure.txt").exists() {
        return Err(format!(
            "original observer failed: {}",
            fs::read_to_string(out.join("failure.txt")).map_err(|e| e.to_string())?.trim()
        ));
    }
    let completed = if first_fire_only { None } else { Some(completion(&out.join("completion.txt"))?) };
    let mut lines = BufReader::new(File::open(out.join("events.csv")).map_err(|e| e.to_string())?).lines();
    if lines.next().transpose().map_err(|e| e.to_string())?.as_deref() != Some(HEADER) {
        return Err("original ledger header differs".into());
    }
    let mut counts = BTreeMap::<String, u64>::new();
    let mut active = Vec::<ActiveCall>::new();
    let mut expected_states = BTreeSet::new();
    let (mut event, mut last_frame, mut frames, mut snapshots) = (0, 0, 0, 0);
    let (mut poll, mut pad_low, mut pad_high) = (false, None, None);
    let (mut neutral, mut consumed_x, mut released, mut impact, mut removed) = (false, false, false, false, false);
    let mut consumed_right = false;
    let mut last_pad = None;
    let mut shot_slot = None;
    let mut first_fire_entry = None;
    let mut first_fire_return = None;
    let (mut dispatch_entry, mut dispatch_complete, mut controller_return) = (0, 0, 0);
    for line in lines {
        let line = line.map_err(|e| e.to_string())?;
        let parts: Vec<_> = line.split(',').collect();
        if parts.len() != 17 {
            return Err("original ledger row extent differs".into());
        }
        event += 1;
        let num = |index: usize| {
            parts[index].parse::<u64>().map_err(|_| format!("invalid ledger number at event {event} field {index}"))
        };
        let frame = num(0)?;
        if num(1)? != event || frame < last_frame || frame > movie::FRAMES as u64 || event > 500000 {
            return Err("original event/frame ordering differs".into());
        }
        last_frame = frame;
        let kind = parts[2];
        let address = num(3)?;
        let value = num(4)?;
        let call = num(5)?;
        if first_fire_only && event == 1 && (kind != "reset" || frame != 1 || address != 0x00841c) {
            return Err("first-Fire prefix does not begin at the original reset vector".into());
        }
        match kind {
            "dispatch-entry" => dispatch_entry = event,
            "dispatch-complete" => dispatch_complete = event,
            "controller-return" => controller_return = event,
            _ => {}
        }
        *counts.entry(kind.to_owned()).or_default() += 1;
        if matches!(kind, "nmi-entry" | "irq-entry") {
            for invocation in &mut active {
                invocation.interrupts += 1;
            }
        }
        if num(14)? == 0x8700 && num(15)? > 0 {
            impact = true;
        }
        if kind == "controller-entry" {
            if poll {
                return Err("nested controller entry".into());
            }
            poll = true;
            pad_low = None;
            pad_high = None;
        } else if kind == "controller-read" {
            if value > 255 {
                return Err("controller callback did not return an actual byte".into());
            }
            if poll {
                match address & 0xffff {
                    0x4218 if pad_low.is_none() => pad_low = Some(value),
                    0x4219 if pad_high.is_none() => pad_high = Some(value),
                    0x4218 | 0x4219 => return Err("duplicate actual controller read in one poll".into()),
                    _ => return Err("unexpected controller register".into()),
                }
            }
        } else if kind == "controller-return" {
            if !poll {
                return Err("controller return without entry".into());
            }
            let pad = pad_low.ok_or("controller return missing actual 4218 read")?
                | pad_high.ok_or("controller return missing actual 4219 read")? << 8;
            last_pad = Some(pad);
            poll = false;
            if (movie::TURN as u64..movie::FIRE as u64).contains(&frame) && pad == 0x100 && num(10)? == 0x100 {
                consumed_right = true;
            }
            if consumed_right && frame < movie::FIRE as u64 && pad == 0 && num(10)? == 0 && num(11)? == 0 {
                neutral = true;
            }
            if frame >= 8660 && pad == 64 && num(10)? == 64 && num(11)? & 64 != 0 {
                consumed_x = true;
            }
            if consumed_x && pad == 0 && num(10)? == 0 {
                released = true;
            }
        } else if kind == "frame-complete" {
            frames += 1;
            if frame != frames {
                return Err("observed frame completion is not contiguous from reset".into());
            }
        } else if ["fire-entry", "handle-entry", "draw-player-entry", "draw-bomb-entry"].contains(&kind) {
            if call == 0 || active.iter().any(|v| v.id == call) {
                return Err("invalid repeated call identity".into());
            }
            if kind == "fire-entry"
                && (!neutral
                    || !consumed_x
                    || last_pad != Some(64)
                    || num(6)? != 8
                    || num(7)? != 0xdf45
                    || num(8)? != 130
                    || num(9)? != 75)
            {
                return Err(
                    "Fire context lacks consumed Right, neutral settling and consumed X in standing context".into()
                );
            }
            if kind == "fire-entry" {
                if first_fire_entry.replace(event).is_some() {
                    return Err("more than one Fire entry before the first return".into());
                }
                if first_fire_only && (dispatch_entry <= dispatch_complete || controller_return >= dispatch_entry) {
                    return Err("first Fire lacks a preceding completed controller poll and active dispatch".into());
                }
            }
            let (context, exit, interrupts) = cpu(out, event, address, call, "entry")?;
            if exit != 0 || interrupts != 0 {
                return Err("entry context contains an earlier exit or interrupt".into());
            }
            active.push(ActiveCall {
                id: call,
                kind: kind.trim_end_matches("-entry").to_owned(),
                context,
                exit: None,
                interrupts: 0,
            });
            snapshots += 1;
            expected_states.insert(format!("state-{event}.wram"));
            expected_states.insert(format!("state-{event}.cpu.toml"));
        } else if ["fire-exit", "handle-exit", "draw-player-exit", "draw-bomb-exit"].contains(&kind) {
            let invocation = active.last_mut().ok_or("exit without entry")?;
            if invocation.id != call
                || invocation.kind != kind.trim_end_matches("-exit")
                || invocation.exit.is_some()
                || !allowed_exit(&invocation.kind, address)
            {
                return Err("exit does not match active call".into());
            }
            invocation.exit = Some(address);
        } else if ["fire-return", "handle-return", "draw-player-return", "draw-bomb-return"].contains(&kind) {
            let invocation = active.pop().ok_or("return without entry")?;
            if invocation.id != call || invocation.kind != kind.trim_end_matches("-return") || invocation.exit.is_none()
            {
                return Err("return does not match witnessed call".into());
            }
            let (context, exit, interrupts) = cpu(out, event, address, call, "return")?;
            if context != invocation.context
                || Some(exit as u64) != invocation.exit
                || interrupts != invocation.interrupts
                || interrupts != num(16)? as i64
            {
                return Err("return CPU context does not match its entry and witnessed events".into());
            }
            snapshots += 1;
            expected_states.insert(format!("state-{event}.wram"));
            expected_states.insert(format!("state-{event}.cpu.toml"));
            if kind == "fire-return" {
                if num(14)? != 0x8000 || num(15)? == 0 || num(16)? != 0 {
                    return Err("first Fire did not return one uninterrupted ordinary beam".into());
                }
                shot_slot = Some(parts[13].to_owned());
                first_fire_return = Some(event);
                if first_fire_only {
                    break;
                }
            }
        } else if kind == "shot-removed" {
            if !impact || num(12)? != 0 || num(15)? != 0 || shot_slot.as_deref() != Some(parts[13]) {
                return Err("same-slot shot removal was not observed after impact".into());
            }
            removed = true;
        }
    }
    if first_fire_only {
        if first_fire_entry.is_none()
            || first_fire_return != Some(event)
            || !active.is_empty()
            || poll
            || !consumed_right
            || !neutral
            || !consumed_x
            || snapshots != 2
            || last_frame != frames + 1
        {
            return Err("original first-Fire prefix is incomplete".into());
        }
        for kind in [
            "reset",
            "boot-clear",
            "controller-entry",
            "controller-return",
            "dispatch-entry",
            "dispatch-complete",
            "nmi-entry",
            "fire-entry",
            "fire-exit",
            "fire-return",
        ] {
            if counts.get(kind).copied().unwrap_or(0) == 0 {
                return Err(format!("required first-Fire prefix hook not observed: {kind}"));
            }
        }
        let mut result = Table::new();
        result.insert("status".into(), "validated".into());
        result.insert("scope".into(), "reset through first Fire return only; no later lifecycle claim".into());
        result.insert("through_event".into(), (event as i64).into());
        result.insert("through_frame".into(), (last_frame as i64).into());
        result.insert("completed_frames".into(), (frames as i64).into());
        result.insert("entry_event".into(), (first_fire_entry.unwrap() as i64).into());
        result.insert("snapshots".into(), (snapshots as i64).into());
        result.insert("consumed_right".into(), true.into());
        result.insert("consumed_neutral_after_right".into(), true.into());
        result.insert("consumed_x".into(), true.into());
        result.insert(
            "hook_counts".into(),
            toml::Value::Table(counts.into_iter().map(|(k, v)| (k, (v as i64).into())).collect()),
        );
        return Ok(result);
    }
    let completed = completed.expect("full capture loaded its completion record");
    if !active.is_empty()
        || poll
        || frames != movie::FRAMES as u64
        || completed.get("events") != Some(&event)
        || completed.get("snapshots") != Some(&snapshots)
        || snapshots > 512
        || !neutral
        || !consumed_x
        || !released
        || !impact
        || !removed
    {
        return Err("original capture is incomplete or missing the observed shot lifecycle".into());
    }
    for kind in [
        "reset",
        "boot-clear",
        "controller-entry",
        "controller-return",
        "dispatch-entry",
        "dispatch-complete",
        "nmi-entry",
        "fire-entry",
        "fire-return",
        "handle-entry",
        "handle-return",
        "draw-player-entry",
        "draw-player-return",
        "draw-bomb-entry",
        "draw-bomb-return",
        "hardware-write",
        "shot-removed",
    ] {
        if counts.get(kind).copied().unwrap_or(0) == 0 {
            return Err(format!("required original hook not observed: {kind}"));
        }
    }
    if counts.get("fire-entry") != Some(&1) || counts.get("shot-removed") != Some(&1) {
        return Err("capture did not contain exactly one shot".into());
    }
    let actual_states: BTreeSet<_> = fs::read_dir(out.join("states"))
        .map_err(|e| e.to_string())?
        .map(|e| e.map(|e| e.file_name().to_string_lossy().into_owned()).map_err(|e| e.to_string()))
        .collect::<Result<_>>()?;
    if actual_states != expected_states {
        return Err("snapshot inventory does not match observed entry/return events".into());
    }
    let mut result = Table::new();
    result.insert("completed_frames".into(), (frames as i64).into());
    result.insert("events".into(), (event as i64).into());
    result.insert("snapshots".into(), (snapshots as i64).into());
    result.insert("consumed_x".into(), true.into());
    result.insert("consumed_right".into(), consumed_right.into());
    result.insert("consumed_release".into(), true.into());
    result.insert("shot_impact_and_removal".into(), true.into());
    result.insert(
        "hook_counts".into(),
        toml::Value::Table(counts.into_iter().map(|(k, v)| (k, (v as i64).into())).collect()),
    );
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn declared_length_without_observed_frames_cannot_pass() {
        let root = std::env::temp_dir().join(format!("sm-original-missing-observation-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("completion.txt"), "completed_frames=8850\nlast_frame=8850\nactive_calls=0\nshot_created=1\nshot_impact=1\nshot_removed=1\nfailed=0\nevents=0\nsnapshots=0\n").unwrap();
        fs::write(root.join("events.csv"), format!("{HEADER}\n")).unwrap();
        let result = capture(&root);
        fs::remove_dir_all(root).unwrap();
        assert!(result.unwrap_err().contains("incomplete"));
    }
}
