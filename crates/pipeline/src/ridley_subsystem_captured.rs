//! Unpatched whole-Ridley fixtures from an authenticated actual-C inventory.
use crate::{digest, ridley_subsystem_inventory};
use anyhow::{Context, Result, bail, ensure};
use offline_verification::fixtures::Case;
use offline_verification::fixtures::Manifest;
use offline_verification::fixtures::Routine;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

const ROM_SHA256: &str = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72";
const INPUTS_SHA256: &str = "cc643d751554ad6d30b5d7e3a41a4259525517ca7614cf73abec9fac30176c0e";
const CAPTURE_SUFFIXES: [&str; 8] = [
    "before.wram",
    "before.host",
    "before.ppu",
    "after.wram",
    "after.host",
    "after.ppu",
    "effects.log",
    "services.bin",
];

#[derive(Debug, Deserialize)]
struct Event {
    sequence: u64,
    boundary_id: u64,
    parent_id: u64,
    capture: u64,
    frame: u64,
    window: u8,
    kind: String,
    phase: String,
    depth: usize,
    address: u32,
    argument: u32,
    returned: u32,
    return_kind: u8,
    slot: u16,
    entry_current_slot: u16,
    entry_header: u16,
    entry_pc: u16,
    entry_state: u16,
    host: [u16; 8],
    ppu: [u8; 20],
}

/// The inventory stores native pointer kinds with bank addresses already
/// normalized: zero, a bank-relative native sentinel, or an actual ROM pointer.
fn native_return(event: &Event) -> Result<u64> {
    match event.return_kind {
        0 => ensure!(event.returned == 0, "native stop pointer is nonzero"),
        1 => ensure!(event.returned >> 16 == 0xa6, "native bank-offset pointer bank differs"),
        2 => ensure!(
            event.returned >> 16 == 0xa6 && event.returned as u16 >= 0x8000,
            "native ROM pointer bank or extent differs"
        ),
        kind => bail!("unsupported native pointer kind {kind}"),
    }
    Ok(u64::from(event.returned))
}

fn definitions(source: &str) -> Result<BTreeMap<u32, String>> {
    let mut result = BTreeMap::new();
    for line in source.lines() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.len() == 3 && fields[0] == "#define" && fields[1].starts_with("fn") {
            let Some(value) = fields[2].strip_prefix("0x") else { continue };
            let address = u32::from_str_radix(value, 16)?;
            result.insert(address, fields[1][2..].into());
        }
    }
    Ok(result)
}

fn symbol<'a>(event: &Event, definitions: &'a BTreeMap<u32, String>) -> Result<Option<&'a str>> {
    match event.kind.as_str() {
        // These translated internals have no declared direct fixture ABI. Their
        // behavior remains covered inside their containing lifecycle captures.
        "baby" | "baby-state" | "helper" => Ok(None),
        "state" => Ok(Some("CallRidleyFunc")),
        "script" => {
            ensure!(event.address == 0xa0c26a, "native script boundary address differs");
            Ok(Some("ProcessEnemyInstructions"))
        }
        "tail" => {
            ensure!(event.address == 0xa6caf5, "native tail boundary address differs");
            Ok(Some("Ridley_Func_70"))
        }
        "direct" if event.address == 0xa6aaaf => Ok(Some("HandleCeresRidleyGetawayCutscene")),
        "direct" if event.address == 0xa6df60 => Ok(Some("sub_A6DF60")),
        "opcode" if event.address == 0xa6812f => {
            ensure!(
                definitions.get(&event.address).map(String::as_str) == Some("EnemyInstr_Sleep_A6"),
                "native generic sleep definition differs"
            );
            Ok(Some("EnemyInstr_Sleep"))
        }
        "ai" | "touch" | "shot" | "direct" | "opcode" => Ok(Some(
            definitions
                .get(&event.address)
                .with_context(|| format!("undeclared native {} address {:06x}", event.kind, event.address))?,
        )),
        kind => bail!("unsupported native boundary kind {kind}"),
    }
}

fn same_invocation(before: &Event, after: &Event) -> bool {
    before.boundary_id == after.boundary_id
        && before.parent_id == after.parent_id
        && before.capture == after.capture
        && before.frame == after.frame
        && before.window == after.window
        && before.kind == after.kind
        && before.depth == after.depth
        && before.address == after.address
        && before.argument == after.argument
        && before.slot == after.slot
        && before.entry_current_slot == after.entry_current_slot
        && before.entry_header == after.entry_header
        && before.entry_pc == after.entry_pc
        && before.entry_state == after.entry_state
        && before.sequence < after.sequence
}

fn capture_path(inventory: &Path, capture: u64, suffix: &str) -> PathBuf {
    inventory.join(format!("fixtures/ridley-{capture:010}-{suffix}"))
}

fn host(bytes: &[u8]) -> Result<[u16; 8]> {
    ensure!(bytes.len() == 16, "captured native host extent differs");
    Ok(std::array::from_fn(|i| u16::from_le_bytes([bytes[i * 2], bytes[i * 2 + 1]])))
}

fn check_capture(
    inventory: &Path,
    before: &Event,
    after: &Event,
    artifacts: &toml::map::Map<String, toml::Value>,
    dependencies: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    for suffix in CAPTURE_SUFFIXES {
        let path = capture_path(inventory, before.capture, suffix);
        let relative = path.strip_prefix(inventory)?.to_str().context("native capture path is not UTF-8")?;
        ensure!(
            digest(&path)? == artifacts.get(relative).and_then(toml::Value::as_str).context("capture seal missing")?,
            "authenticated native capture changed: {relative}"
        );
        dependencies.insert(path);
    }
    for event in [before, after] {
        let ram = capture_path(inventory, event.capture, &format!("{}.wram", event.phase));
        ensure!(fs::metadata(ram)?.len() == 0x20000, "captured native WRAM extent differs");
        ensure!(
            host(&fs::read(capture_path(inventory, event.capture, &format!("{}.host", event.phase)))?)? == event.host
                && fs::read(capture_path(inventory, event.capture, &format!("{}.ppu", event.phase)))? == event.ppu,
            "native host or hardware event differs from its capture"
        );
    }
    Ok(())
}

/// Generate independent fixture pairs from the raw stream. Every selected pair
/// starts from its complete native state; no capture is patched or chained.
pub fn generate(checkout: &Path, inventory_verdict: &Path, output: &Path) -> Result<Manifest> {
    let checkout = fs::canonicalize(checkout)?;
    let inventory_verdict = fs::canonicalize(inventory_verdict)?;
    let inventory = inventory_verdict.parent().context("native inventory parent missing")?;
    let verdict_hash = digest(&inventory_verdict)?;
    ridley_subsystem_inventory::authenticate(&checkout, &inventory_verdict)?;
    let verdict: toml::Value = toml::from_str(&fs::read_to_string(&inventory_verdict)?)?;
    ensure!(
        verdict["status"].as_str() == Some("diagnostic-complete")
            && verdict["frames"].as_integer() == Some(129712)
            && verdict["rom_sha256"].as_str() == Some(ROM_SHA256)
            && verdict["inputs_sha256"].as_str() == Some(INPUTS_SHA256),
        "native supplement requires the authenticated complete pinned replay"
    );
    let artifacts = verdict["artifacts"].as_table().context("native artifact seal missing")?;
    let controlled_path = checkout.join("tools/port/ridley-subsystem.smf");
    let mut manifest = offline_verification::fixtures::read(&controlled_path)?;
    ensure!(
        manifest.callback_adapter.as_deref() == Some("ridley-subsystem")
            && manifest.routines.len() == 33
            && manifest.rom.as_ref().is_some_and(|rom| rom.sha256 == ROM_SHA256),
        "controlled native subsystem signature contract differs"
    );
    let rom = manifest.rom.as_mut().context("controlled ROM missing")?;
    rom.file = fs::canonicalize(if rom.file.is_absolute() { rom.file.clone() } else { checkout.join(&rom.file) })?;
    ensure!(digest(&rom.file)? == ROM_SHA256, "controlled input ROM changed");
    let state_addresses = manifest
        .routines
        .iter()
        .find(|routine| routine.symbol == "CallRidleyFunc")
        .context("controlled state-dispatch signature missing")?
        .cases
        .iter()
        .map(|fixture| u32::try_from(fixture.a).context("controlled state address extent differs"))
        .collect::<Result<BTreeSet<_>>>()?;
    for routine in &mut manifest.routines {
        routine.cases.clear();
        routine.live_eligible = Some(false);
        routine.live_limitation =
            Some("Natural captured parity is offline evidence; the independent live gate determines admission.".into());
    }
    let mut sleep: Routine = manifest
        .routines
        .iter()
        .find(|routine| routine.symbol == "Ridley_Instr_1")
        .context("controlled callback signature missing")?
        .clone();
    sleep.symbol = "EnemyInstr_Sleep".into();
    manifest.routines.push(sleep);
    let defs_path = inventory.join("original-source/funcs.h");
    let defs = definitions(&fs::read_to_string(&defs_path)?)?;
    // Native declarations without fn constants have pinned annotated definitions.
    let native_a6 = fs::read_to_string(inventory.join("original-source/sm_a6.c"))?;
    for declaration in [
        "void HandleCeresRidleyGetawayCutscene(void) {  // 0xA6AAAF",
        "void Ridley_Func_70(void) {  // 0xA6CAF5",
        "void sub_A6DF60(void) {  // 0xA6DF60",
    ] {
        ensure!(native_a6.matches(declaration).count() == 1, "native annotated entry declaration differs");
    }
    let stream = inventory.join("ridley-subsystem-events.jsonl");
    let mut selected = BTreeMap::<u64, (Option<Event>, Option<Event>)>::new();
    for line in BufReader::new(fs::File::open(&stream)?).lines() {
        let event: Event = serde_json::from_str(&line?)?;
        if event.capture == 0 {
            continue;
        }
        let pair = selected.entry(event.capture).or_default();
        let phase = match event.phase.as_str() {
            "before" => &mut pair.0,
            "after" => &mut pair.1,
            phase => bail!("unsupported native capture phase {phase}"),
        };
        ensure!(phase.replace(event).is_none(), "duplicate native capture phase");
    }
    ensure!(
        i64::try_from(selected.len())? == verdict["retained_captures"].as_integer().context("capture count missing")?,
        "native capture stream and verdict counts differ"
    );
    let mut dependencies = manifest.rust_dependencies.iter().cloned().collect::<BTreeSet<_>>();
    dependencies.extend([
        PathBuf::from("crates/pipeline/src/ridley_subsystem_captured.rs"),
        controlled_path.clone(),
        inventory_verdict.clone(),
        stream,
        defs_path,
        inventory.join("original-source/sm_a6.c"),
    ]);
    let mut counts = BTreeMap::<String, usize>::new();
    let mut excluded = BTreeMap::<String, usize>::new();
    let mut order = Vec::new();
    for (_, (before, after)) in selected {
        let before = before.context("native capture before event missing")?;
        let after = after.context("native capture after event missing")?;
        ensure!(same_invocation(&before, &after), "native capture invocation identity differs");
        let Some(symbol) = symbol(&before, &defs)? else {
            *excluded.entry(before.kind.clone()).or_default() += 1;
            continue;
        };
        if before.kind == "state" {
            ensure!(state_addresses.contains(&before.address), "native state lies outside controlled dispatch domain");
        }
        check_capture(inventory, &before, &after, artifacts, &mut dependencies)?;
        let routine = manifest
            .routines
            .iter_mut()
            .find(|routine| routine.symbol == symbol)
            .with_context(|| format!("native captured entry has no controlled signature: {symbol}"))?;
        let name = format!("ridley-{:010}", before.capture);
        let (a, b, returned) = match before.kind.as_str() {
            "opcode" => (i64::from(before.slot), i64::from(before.argument), native_return(&after)?),
            "state" => {
                ensure!(after.returned == 0 && after.return_kind == 0, "native void state return differs");
                (i64::from(before.address), 0, 0)
            }
            _ => {
                ensure!(after.returned == 0 && after.return_kind == 0, "native void entry return differs");
                (0, 0, 0)
            }
        };
        let before_file = capture_path(inventory, before.capture, "before.wram");
        let after_file = capture_path(inventory, before.capture, "after.wram");
        routine.cases.push(Case {
            name: name.clone(),
            a,
            b,
            host_words: Some(before.host.to_vec()),
            before_sha256: Some(digest(&before_file)?),
            before_file: Some(before_file),
            expected_after_sha256: Some(digest(&after_file)?),
            expected_after_file: Some(after_file),
            expected_return: Some(returned),
            ..Default::default()
        });
        order.push(name);
        *counts.entry(before.kind).or_default() += 1;
    }
    ensure!(!order.is_empty(), "no supported native captures retained");
    manifest.family = "ridley-subsystem-captured".into();
    manifest.routines.retain(|routine| !routine.cases.is_empty());
    manifest.case_order = Some(order);
    manifest.rust_dependencies = dependencies.into_iter().collect();
    manifest.source = Some(format!(
        "Authenticated native inventory {} SHA256 {}; controlled signature manifest SHA256 {}.",
        inventory_verdict.display(),
        verdict_hash,
        digest(&controlled_path)?
    ));
    manifest.scope = Some(format!(
        "Unpatched natural native captures {counts:?}; excluded direct internals {excluded:?}. Complete WRAM, host, hardware, ordered effects and measured actual-C services remain sealed. Offline parity grants no public live admission or original-ROM parity."
    ));
    ensure!(digest(&inventory_verdict)? == verdict_hash, "inventory verdict changed during capture generation");
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    offline_verification::fixtures::write(output, &manifest)?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(returned: u32, return_kind: u8) -> Event {
        serde_json::from_value(serde_json::json!({
            "sequence":1,"boundary_id":1,"parent_id":0,"capture":1,"frame":1,"window":0,
            "kind":"opcode","phase":"after","depth":1,"address":0xa6e517,"argument":0xa6e73c,
            "returned":returned,"return_kind":return_kind,"slot":0,"entry_current_slot":0,
            "entry_header":0xe17f,"entry_pc":0xe73a,"entry_state":0xa35b,
            "host":[0,0,0,0,0,0,0,0],"ppu":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
        }))
        .unwrap()
    }

    #[test]
    fn normalized_native_pointers_keep_stop_offset_and_rom_distinct() {
        assert_eq!(native_return(&event(0, 0)).unwrap(), 0);
        assert_eq!(native_return(&event(0xa6e73a, 1)).unwrap(), 0xa6e73a);
        assert_eq!(native_return(&event(0xa6e73a, 2)).unwrap(), 0xa6e73a);
        assert!(native_return(&event(0xa6e73a, 0)).is_err());
        assert!(native_return(&event(0xa50000, 1)).is_err());
        assert!(native_return(&event(0xa67fff, 2)).is_err());
        assert!(native_return(&event(0, 3)).is_err());
    }
}
