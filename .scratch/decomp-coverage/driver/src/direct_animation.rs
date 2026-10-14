//! Actual-C three-argument fixture pairs captured directly into shared evidence.
#![allow(dead_code)]
use anyhow::{Context, Result, ensure};
use evidence::{Entry, EntryKind, Store};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path, process::Command};

#[path = "/tmp/sm-decomp-parallel-pause-equipment-render-offline-1/crates/port/src/memory.rs"]
mod memory;
pub use memory::Wram;
#[path = "/tmp/sm-decomp-parallel-pause-equipment-render-offline-1/crates/port/src/pause_equipment_render.rs"]
mod pause_equipment_render;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    schema: String,
    routine: String,
    animation_ids: Vec<u16>,
    timers: Vec<u16>,
    frames: Vec<u16>,
    coordinates: Vec<[u16; 2]>,
    oam_cursors: Vec<u16>,
    limitations: Vec<String>,
}
#[derive(Deserialize, Serialize)]
struct Pair {
    name: String,
    animation: u16,
    timer: u16,
    frame: u16,
    x: u16,
    y: u16,
    oam: u16,
    before: String,
    c_after: String,
    rust_after: String,
    c_return: u64,
    rust_return: u64,
}
#[derive(Deserialize, Serialize)]
struct Verdict {
    schema: String,
    status: String,
    routine: String,
    source_commit: String,
    verified_cases: usize,
    evidence_run: String,
    inputs: BTreeMap<String, String>,
    pairs: Vec<Pair>,
    limitations: Vec<String>,
}

fn rom_offset(address: usize) -> usize { (((address >> 16) << 15) | (address & 0x7fff)) & 0x3fffff }
fn rom_word(rom: &[u8], address: usize) -> u16 {
    let offset = rom_offset(address);
    u16::from_le_bytes(rom[offset..offset + 2].try_into().unwrap())
}
fn frames(rom: &[u8], script: u16) -> Result<usize> {
    for frame in 0..64 {
        if rom[rom_offset(0x820000 | script as usize) + frame * 3] == 255 { return Ok(frame) }
    }
    anyhow::bail!("animation sentinel missing")
}
fn file_entry(store: &mut Store, entries: &mut Vec<Entry>, name: &str, path: &Path) -> Result<()> {
    let id = evidence::put_file(store, &mut fs::File::open(path)?)?;
    entries.push(Entry { name: name.into(), kind: EntryKind::File, id });
    Ok(())
}
fn image_entry(store: &mut Store, entries: &mut Vec<Entry>, name: String, bytes: &[u8]) -> Result<String> {
    let id = evidence::put_image(store, bytes)?;
    entries.push(Entry { name, kind: EntryKind::Image, id });
    Ok(evidence::hex(&id))
}
fn state(rom: &[u8], animation: u16, timer: u16, frame: u16, oam: u16) -> Result<Vec<u8>> {
    let mut bytes = vec![0xa5; memory::WRAM_SIZE];
    let mut wram = memory::Wram::new(&mut bytes).unwrap();
    let index = usize::from(animation - 1) * 2;
    for (table, value) in [(0x82c0b2, timer), (0x82c0c4, frame), (0x82c0d6, 0)] {
        let offset = usize::from(rom_word(rom, table + index));
        ensure!(offset < 0x2000 - 1, "animation state pointer outside direct-page RAM");
        wram.set_word(offset, value);
    }
    wram.set_word(0x590, oam);
    Ok(bytes)
}

fn run(plan_path: &Path, output: &Path, oracle: &Path, candidate: &Path) -> Result<()> {
    ensure!(!output.exists(), "direct-animation verdict already exists");
    let root = Path::new("/home/jim/sm-decomp");
    let worktree = Path::new("/tmp/sm-decomp-parallel-pause-equipment-render-offline-1");
    let plan: Plan = toml::from_str(&fs::read_to_string(plan_path)?)?;
    ensure!(plan.schema == "decomp-direct-animation-cases-v1" && plan.routine == "DrawPauseScreenSpriteAnim", "wrong direct routine plan");
    ensure!(!plan.animation_ids.is_empty() && plan.animation_ids.iter().all(|id| matches!(id, 2 | 3 | 6..=9)), "invalid animation ids");
    let rom_path = root.join("rom/sm.sfc");
    let rom = fs::read(&rom_path)?;
    let mut store = Store::create(&root.join("out/evidence"))?;
    let mut entries = vec![];
    let mut paths = BTreeMap::new();
    for (name, path) in [
        ("plan.toml", plan_path.to_owned()),
        ("candidate.rs", candidate.to_owned()),
        ("canonical-memory.rs", worktree.join("crates/port/src/memory.rs")),
        ("c-oracle", oracle.to_owned()),
        ("rust-verifier", std::env::current_exe()?),
        ("verifier.rs", root.join(".scratch/decomp-coverage/driver/src/direct_animation.rs")),
        ("oracle.c", root.join(".scratch/decomp-coverage/renderer/direct-animation-oracle.c")),
        ("pinned-sm81.c", root.join("deps/snesrev-sm/src/sm_81.c")),
        ("pinned-sm82.c", root.join("deps/snesrev-sm/src/sm_82.c")),
        ("pinned-smrtl.c", root.join("deps/snesrev-sm/src/sm_rtl.c")),
        ("dependency-pins.toml", root.join("tools/dependencies.toml")),
        ("source-inventory.toml", root.join("tools/port/source-inventory.toml")),
        ("rom.sfc", rom_path.clone()),
    ] {
        file_entry(&mut store, &mut entries, name, &path)?;
        paths.insert(name.to_owned(), path.display().to_string());
    }
    for entry in fs::read_dir(root.join("deps/snesrev-sm/src"))? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "h") {
            let name = format!("header-{}", path.file_name().unwrap().to_string_lossy());
            file_entry(&mut store, &mut entries, &name, &path)?;
            paths.insert(name, path.display().to_string());
        }
    }
    let source = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(worktree).output()?;
    ensure!(source.status.success(), "source commit unavailable");
    let source_commit = String::from_utf8(source.stdout)?.trim().to_owned();
    let inputs_tree = evidence::put_tree(&mut store, entries.clone())?;
    let inputs = evidence::tree_inventory(&store, &inputs_tree)?.into_iter().map(|(key, member)| (paths[&key].clone(), evidence::hex(&member.sha256))).collect();
    let scratch = output.parent().context("output parent missing")?;
    fs::create_dir_all(scratch)?;
    let before_path = scratch.join("scratch-before.wram");
    let after_path = scratch.join("scratch-c-after.wram");
    let mut pairs = vec![];
    for &animation in &plan.animation_ids {
        let script = rom_word(&rom, 0x82c0e8 + usize::from(animation - 1) * 2);
        let count = frames(&rom, script)?;
        ensure!(count > 0, "empty animation sequence");
        for &timer in &plan.timers {
            for &requested_frame in &plan.frames {
                let frame = if requested_frame == 255 { (count - 1) as u16 } else { requested_frame };
                ensure!(usize::from(frame) < count, "requested frame outside animation");
                for &[x, y] in &plan.coordinates {
                    for &oam in &plan.oam_cursors {
                        let before = state(&rom, animation, timer, frame, oam)?;
                        fs::write(&before_path, &before)?;
                        let actual = Command::new(oracle).args([animation.to_string(), x.to_string(), y.to_string(), before_path.display().to_string(), after_path.display().to_string(), rom_path.display().to_string()]).output()?;
                        ensure!(actual.status.success(), "actual C animation failed: {}", String::from_utf8_lossy(&actual.stderr));
                        let c = fs::read(&after_path)?;
                        let c_return: u64 = String::from_utf8(actual.stdout)?.trim().parse()?;
                        let mut rust = before.clone();
                        let a = (u32::from(animation) | (u32::from(x) << 16)) as i32;
                        let rust_return = pause_equipment_render::dispatch_rom(&mut memory::Wram::new(&mut rust).unwrap(), &rom, "DrawPauseScreenSpriteAnim", a, y as i32).map_err(anyhow::Error::msg)? as u64;
                        ensure!(c == rust && c_return == rust_return, "direct animation mismatch: animation={animation} timer={timer} frame={frame} x={x} y={y} oam={oam}; first={:?}", c.iter().zip(&rust).position(|(a,b)| a != b));
                        let name = format!("pair-{}", pairs.len());
                        let before_id = image_entry(&mut store, &mut entries, format!("{name}-before.wram"), &before)?;
                        let c_id = image_entry(&mut store, &mut entries, format!("{name}-c-after.wram"), &c)?;
                        let rust_id = image_entry(&mut store, &mut entries, format!("{name}-rust-after.wram"), &rust)?;
                        pairs.push(Pair { name, animation, timer, frame, x, y, oam, before: before_id, c_after: c_id, rust_after: rust_id, c_return, rust_return });
                    }
                }
            }
        }
    }
    let pair_bytes = toml::to_string_pretty(&BTreeMap::from([("pairs", &pairs)]))?;
    let id = evidence::put_file(&mut store, &mut pair_bytes.as_bytes())?;
    entries.push(Entry { name: "pairs.toml".into(), kind: EntryKind::File, id });
    let tree = evidence::put_tree(&mut store, entries)?;
    let run = evidence::put_run(&mut store, tree, &evidence::Provenance { subject: Some("DrawPauseScreenSpriteAnim exact three-word actual-C fixtures".into()), ..Default::default() })?;
    store.commit()?;
    let verdict = Verdict { schema: "decomp-direct-animation-verdict-v1".into(), status: "offline-verified".into(), routine: plan.routine, source_commit, verified_cases: pairs.len(), evidence_run: evidence::hex(&run), inputs, pairs, limitations: plan.limitations };
    fs::write(output, toml::to_string_pretty(&verdict)?)?;
    fs::remove_file(before_path)?;
    fs::remove_file(after_path)?;
    println!("verified_cases={} evidence_run={}", verdict.verified_cases, verdict.evidence_run);
    Ok(())
}

fn authenticate(path: &Path) -> Result<()> {
    let verdict: Verdict = toml::from_str(&fs::read_to_string(path)?)?;
    ensure!(verdict.schema == "decomp-direct-animation-verdict-v1" && verdict.status == "offline-verified" && verdict.verified_cases == verdict.pairs.len() && !verdict.pairs.is_empty(), "invalid direct verdict");
    let store = Store::open(Path::new("/home/jim/sm-decomp/out/evidence"))?;
    let run_id = evidence::parse_id(&verdict.evidence_run)?;
    let (run, _) = evidence::verified_inventory(&store, &run_id)?;
    let entries = evidence::read_tree(&store, &run.observations)?;
    let mut pair_bytes = vec![];
    let entry = entries.iter().find(|entry| entry.name == "pairs.toml").context("pair declaration missing")?;
    evidence::write_entry(&store, entry.kind, &entry.id, &mut pair_bytes)?;
    let stored: toml::Value = toml::from_str(std::str::from_utf8(&pair_bytes)?)?;
    ensure!(stored["pairs"] == toml::Value::try_from(&verdict.pairs)?, "pair records differ from trusted store run");
    for pair in &verdict.pairs {
        let find = |suffix: &str, id: &str| -> Result<Vec<u8>> {
            let name = format!("{}-{suffix}.wram", pair.name);
            let entry = entries.iter().find(|e| e.name == name).context("pair image missing")?;
            ensure!(entry.kind == EntryKind::Image && evidence::hex(&entry.id) == id, "pair image id changed");
            let mut bytes = vec![];
            evidence::write_image(&store, &entry.id, &mut bytes)?;
            ensure!(bytes.len() == memory::WRAM_SIZE, "incomplete WRAM");
            Ok(bytes)
        };
        let _before = find("before", &pair.before)?;
        ensure!(find("c-after", &pair.c_after)? == find("rust-after", &pair.rust_after)? && pair.c_return == pair.rust_return, "stored pair mismatch");
    }
    let mut transient = Store::create(Path::new("/home/jim/sm-decomp/out/evidence"))?;
    for (path, expected) in &verdict.inputs {
        let id = evidence::put_file(&mut transient, &mut fs::File::open(path)?)?;
        let tree = evidence::put_tree(&mut transient, vec![Entry { name: "input".into(), kind: EntryKind::File, id }])?;
        ensure!(evidence::hex(&evidence::tree_inventory(&transient, &tree)?["input"].sha256) == *expected, "source/binary input changed: {path}");
    }
    transient.commit()?;
    println!("status = \"authenticated\"\nverified_cases = {}\nevidence_run = \"{}\"", verdict.verified_cases, verdict.evidence_run);
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("verify") => run(Path::new(&args[2]), Path::new(&args[3]), Path::new(&args[4]), Path::new(&args[5])),
        Some("authenticate") => authenticate(Path::new(&args[2])),
        _ => anyhow::bail!("use verify PLAN VERDICT ORACLE CANDIDATE or authenticate VERDICT"),
    }
}
