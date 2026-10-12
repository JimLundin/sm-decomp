//! Deterministic actual-ROM direct callbacks and complete native attack-frame chains.
use crate::family_model::{Asset, Case, Manifest, Routine, Write};
use crate::{atomic_toml, digest};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

const ROM_SHA256: &str = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72";
fn write(offset: usize, value: u16) -> Write {
    Write { offset, data: value.to_le_bytes().to_vec() }
}
fn base(slot: u16, orientation: u16, health: u16, area: u16) -> Vec<Write> {
    let enemy = 0xf78 + usize::from(slot);
    vec![
        write(0xe54, slot),
        write(enemy, 0xe13f),
        write(enemy + 0x2e, 0xa6),
        write(enemy + 0x1a, 0xe73a),
        write(enemy + 0x1c, 1),
        write(0xf7a, 180),
        write(0xf7e, 111),
        write(0xaf6, 80),
        write(0xafa, 180),
        write(0x7820, orientation),
        write(0x9c2, health),
        write(0x79f, area),
    ]
}
fn host() -> Vec<u16> {
    vec![0x5aa5, 0xa55a, 11, 22, 33, 44, 55, 66]
}
fn direct(symbol: &str) -> Routine {
    Routine {
        symbol: symbol.into(),
        args: 2,
        arg_types: Some(vec!["uint16".into(), "rom_ptr".into()]),
        returns: "rom_ptr".into(),
        pointer_bank: Some(0xa6),
        live_eligible: Some(false),
        live_limitation: Some(
            "Native live ownership is claimed only at complete ProcessEnemyInstructions frames.".into(),
        ),
        ..Default::default()
    }
}
fn routine<'a>(routines: &'a mut [Routine], symbol: &str) -> &'a mut Routine {
    routines.iter_mut().find(|r| r.symbol == symbol).unwrap()
}
fn expected_file(root: &Path, name: &str, writes: &[Write], updates: &[Write]) -> Result<Asset> {
    let mut bytes = vec![0; 0x20000];
    for w in writes.iter().chain(updates) {
        bytes[w.offset..w.offset + w.data.len()].copy_from_slice(&w.data);
    }
    let file = root.join(format!("{name}.wram"));
    fs::write(&file, bytes)?;
    Ok(Asset { sha256: digest(&file)?, file })
}
fn angle_coordinates() -> [(u16, u16); 256] {
    let mut pairs = [None; 256];
    for dx in -255_i16..=255 {
        for dy in -255_i16..=255 {
            let x = dx.unsigned_abs();
            let y = dy.unsigned_abs();
            let horizontal = y < x;
            let (num, den) = if horizontal { (y << 8, x as u8) } else { (x << 8, y as u8) };
            let step = ((if den == 0 { 0xffff } else { num / u16::from(den) }) >> 3) as u8;
            let quadrant = usize::from(dx < 0) * 2 + usize::from(dy < 0);
            let angle = if horizontal {
                match quadrant {
                    0 => 64u8.wrapping_add(step),
                    1 => 64u8.wrapping_sub(step),
                    2 => 192u8.wrapping_sub(step),
                    _ => step.wrapping_sub(64),
                }
            } else {
                match quadrant {
                    0 => 128u8.wrapping_sub(step),
                    1 => step,
                    2 => 128u8.wrapping_add(step),
                    _ => 0u8.wrapping_sub(step),
                }
            };
            pairs[usize::from(angle.wrapping_add(128).wrapping_neg())].get_or_insert((dx as u16, dy as u16));
        }
    }
    pairs.map(Option::unwrap)
}
fn component(rom: &[u8], angle: u8) -> u16 {
    let offset = 0x1033c3 + (usize::from(angle) + 64) * 2;
    let sine = i16::from_le_bytes([rom[offset], rom[offset + 1]]);
    let value = ((u32::from(sine.unsigned_abs()) * 0x500) >> 8) as u16;
    if sine < 0 { value.wrapping_neg() } else { value }
}
fn callback_cases(routines: &mut [Routine], rom: &[u8], expected: &Path) -> Result<usize> {
    let mut cases = 0;
    let coordinates = angle_coordinates();
    for orientation in [0_u16, 1] {
        for (raw, (dx, dy)) in coordinates.into_iter().enumerate() {
            let name = format!("aim-o{orientation}-raw{raw:03}");
            let mut writes = base(64, orientation, 99, 0);
            let mouth_x = 180u16.wrapping_add(if orientation == 0 { (-25_i16) as u16 } else { 25 });
            writes.extend([
                write(0xaf6, mouth_x.wrapping_add(dx)),
                write(0xafa, 68_u16.wrapping_add(dy)),
                write(0xfba, 5000),
                write(0xfbe, 6000),
            ]);
            let raw = raw as u8;
            let angle = if orientation != 0 {
                if !(0x15..0xc0).contains(&raw) { 0x15 } else { raw.min(0x50) }
            } else if !(0x40..0xeb).contains(&raw) {
                0xeb
            } else {
                raw.max(0xb0)
            };
            let after = expected_file(
                expected,
                &name,
                &writes,
                &[write(0x7832, component(rom, angle)), write(0x7834, component(rom, angle.wrapping_add(64)))],
            )?;
            routine(routines, "Ridley_Instr_11").cases.push(Case {
                name,
                a: 64,
                b: 0xa6e755,
                writes,
                host_words: Some(host()),
                expected_return: Some(0xa6e755),
                expected_after_file: Some(after.file),
                expected_after_sha256: Some(after.sha256),
                ..Default::default()
            });
            cases += 1;
        }
    }
    let deltas = [0_u16, 1, 25, 43, 0xff, 0x100, 0x101, 0x7fff, 0x8000, 0xffff];
    for orientation in [0_u16, 2, 0xffff] {
        for (i, dx) in deltas.into_iter().enumerate() {
            let dy = deltas[(i + 3) % deltas.len()];
            for (x, y) in [(0_u16, 0_u16), (0xffff, 0xffff)] {
                let mut writes = base(64, orientation, 99, 0);
                let mouth_x = x.wrapping_add(if orientation == 0 { (-25_i16) as u16 } else { 25 });
                writes.extend([
                    write(0xf7a, x),
                    write(0xf7e, y),
                    write(0xaf6, mouth_x.wrapping_add(dx)),
                    write(0xafa, y.wrapping_sub(43).wrapping_add(dy)),
                ]);
                routine(routines, "Ridley_Instr_11").cases.push(Case {
                    name: format!("aim-wrap-o{orientation}-d{i}-x{x}"),
                    a: 64,
                    b: 0xa6e755,
                    writes,
                    host_words: Some(host()),
                    expected_return: Some(0xa6e755),
                    ..Default::default()
                });
                cases += 1;
            }
        }
    }
    for name in ["Ridley_Instr_12", "Ridley_Instr_13"] {
        for free in (0..18).map(|i| Some(i * 2)).chain(std::iter::once(None)) {
            for orientation in [0, 1, 2, 0xffff] {
                let mut writes = base(64, orientation, 99, 0);
                writes.extend([write(0x1995, 0xa5a5), write(0x7832, 0x2345), write(0x7834, 0xcdef)]);
                writes.extend((0..36).step_by(2).map(|slot| write(0x1997 + slot, 0x7777)));
                if let Some(slot) = free {
                    writes.push(write(0x1997 + slot, 0));
                }
                routine(routines, name).cases.push(Case {
                    name: format!(
                        "{name}-free{}-o{orientation}",
                        free.map_or_else(|| "full".into(), |i| i.to_string())
                    ),
                    a: 64,
                    b: 0xa6e755,
                    writes,
                    host_words: Some(host()),
                    expected_return: Some(0xa6e755),
                    ..Default::default()
                });
                cases += 1;
            }
        }
    }
    for orientation in [0, 1, 2, 0xffff] {
        for name in ["Ridley_Instr_1", "Ridley_Instr_10"] {
            for health in [0, 29, 30, 31, 0x7fff, 0x8000, 0x801d, 0x801e, 0xffff] {
                for area in [0, 2] {
                    let writes = base(64, orientation, health, area);
                    let branch = if name == "Ridley_Instr_1" {
                        orientation != 0
                    } else {
                        area != 2 && (health.wrapping_sub(30) as i16) < 0
                    };
                    let return_value = if branch { 0xa6e7b4 } else { 0xa6e73e };
                    routine(routines, name).cases.push(Case {
                        name: format!("{name}-o{orientation}-h{health}-a{area}"),
                        a: 64,
                        b: 0xa6e73c,
                        writes,
                        host_words: Some(host()),
                        expected_return: Some(return_value),
                        ..Default::default()
                    });
                    cases += 1;
                }
            }
        }
    }
    for (read, writepos) in [(0_u8, 0_u8), (0, 5), (0, 6), (15, 4), (15, 5), (15, 15), (0, 15)] {
        for (debug, state, powerbomb) in [(0_u16, 8_u16, 0_u16), (1, 8, 0), (0, 40, 0), (0, 8, 0x8000)] {
            let mut writes = base(64, 0, 99, 0);
            writes.extend([
                write(0x5f5, debug),
                write(0x998, state),
                write(0x592, powerbomb),
                Write { offset: 0x644, data: vec![read] },
                Write { offset: 0x647, data: vec![writepos] },
            ]);
            routine(routines, "Ridley_Instr_5").cases.push(Case {
                name: format!("sound-r{read}-w{writepos}-d{debug}-s{state}-p{powerbomb}"),
                a: 64,
                b: 0xa6e755,
                writes,
                host_words: Some(host()),
                expected_return: Some(0xa6e755),
                ..Default::default()
            });
            cases += 1;
        }
    }
    for pointer in [0xa6e755, 0xa6ffff] {
        routine(routines, "Ridley_Instr_6").cases.push(Case {
            name: format!("sound-clear-p{pointer}"),
            a: 64,
            b: pointer,
            writes: base(64, 0, 99, 0),
            host_words: Some(host()),
            expected_return: Some(pointer as u64),
            ..Default::default()
        });
        cases += 1;
    }
    Ok(cases)
}
fn frame_cases(routines: &mut [Routine], order: &mut Vec<String>) -> usize {
    let native = routine(routines, "ProcessEnemyInstructions");
    let mut cases = 0;
    for orientation in [0_u16, 1] {
        for health in [29_u16, 30, 31, 0x8000] {
            for area in [0_u16, 2] {
                let name = format!("entry-o{orientation}-h{health}-a{area}");
                native.cases.push(Case {
                    name: name.clone(),
                    writes: base(64, orientation, health, area),
                    host_words: Some(host()),
                    expected_return: Some(0),
                    ..Default::default()
                });
                order.push(name);
                cases += 1;
            }
        }
    }
    for timer in [0, 1, 0xffff] {
        for frozen in [false, true] {
            let name = format!("timer-{timer}-frozen{frozen}");
            let mut writes = base(64, 0, 99, 0);
            writes.extend([write(0xfd4, timer), write(0xfca, if frozen { 4 } else { 0 }), write(0xfc8, 0x8007)]);
            native.cases.push(Case {
                name: name.clone(),
                writes,
                host_words: Some(host()),
                expected_return: Some(0),
                ..Default::default()
            });
            order.push(name);
            cases += 1;
        }
    }
    // Complete natural ROM script, controlled starting state. Continuations do
    // not patch timers or replace host: both subjects inherit each full frame.
    for orientation in [0_u16, 1] {
        for saturation in [false, true] {
            let prefix = format!("chain-o{orientation}-full{saturation}");
            let mut previous = None;
            for step in 0..221 {
                let name = format!("{prefix}-frame{step:03}");
                let mut writes = if step == 0 { base(0, orientation, 99, 0) } else { Vec::new() };
                if step == 0 && saturation {
                    writes.extend((0..36).step_by(2).map(|slot| write(0x1997 + slot, 0x7777)));
                }
                native.cases.push(Case {
                    name: name.clone(),
                    writes,
                    host_words: (step == 0).then(host),
                    continue_from: previous.clone(),
                    expected_return: Some(0),
                    ..Default::default()
                });
                order.push(name.clone());
                previous = Some(name);
                cases += 1;
            }
        }
    }
    cases
}

fn manifest(rom_path: &Path) -> Result<Manifest> {
    Ok(Manifest {
        family:"observed-ridley-instructions".into(),rust_module:"crates/port/src/ridley_enemy_instructions.rs".into(),
        bank:"sm_a0.c".into(),c_support:vec!["sm_a6.c".into(),"sm_86.c".into(),"sm_80.c".into(),"sm_rtl.c".into()],
        rust_imports:vec!["audio".into(),"enemy".into(),"enemy_instructions".into(),"enemy_projectiles".into(),"observed_projectile_ownership".into()],
        rom:Some(Asset{file:fs::canonicalize(rom_path)?,sha256:ROM_SHA256.into()}),projectile_host:true,
        source:Some("snesrev/sm 578f90b3cc49557bb70060ad033bb90b8cf8ac50; actual native A0/A6 interpreter and bank86 allocator".into()),
        scope:Some("Complete pinned A6E73A..E828 aiming/spawning attack frames; direct callback and controlled chains are offline evidence. Native live ownership requires separately authenticated replay.".into()),
        routines:vec![direct("Ridley_Instr_1"),direct("Ridley_Instr_5"),direct("Ridley_Instr_6"),direct("Ridley_Instr_10"),direct("Ridley_Instr_11"),direct("Ridley_Instr_12"),direct("Ridley_Instr_13"),Routine{symbol:"ProcessEnemyInstructions".into(),returns:"void".into(),args:0,arg_types:Some(vec![]),live_eligible:Some(true),original_address:Some(0xa0c26a),..Default::default()}],
        ..Default::default()
    })
}

pub fn generate(checkout: &Path, rom_path: &Path, output: &Path, expected: &Path) -> Result<usize> {
    ensure!(digest(rom_path)? == ROM_SHA256, "Ridley fixture ROM identity differs");
    fs::create_dir_all(expected)?;
    let expected = fs::canonicalize(expected)?;
    let rom = fs::read(rom_path)?;
    let mut manifest = manifest(rom_path)?;
    let direct_count = callback_cases(&mut manifest.routines, &rom, &expected)?;
    let mut order = manifest.routines.iter().flat_map(|r| r.cases.iter().map(|c| c.name.clone())).collect::<Vec<_>>();
    let frame_count = frame_cases(&mut manifest.routines, &mut order);
    manifest.case_order = Some(order);
    ensure!(checkout.join(&manifest.rust_module).is_file(), "Ridley gameplay module missing");
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    atomic_toml(output, &manifest)?;
    Ok(direct_count + frame_count)
}

/// A sealed actual-C event; pointer kinds are 0=stop, 1=bank offset, 2=ROM.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NativeEvent {
    pub sequence: u64,
    pub frame_sequence: u64,
    pub frame: u32,
    pub window: u8,
    pub kind: String,
    pub phase: String,
    pub depth: u8,
    pub slot: u16,
    pub header: u16,
    pub bank: u8,
    pub entry_pc: u16,
    pub pc: u16,
    pub timer: u16,
    pub ai_bits: u16,
    pub extra: u16,
    pub spritemap: u16,
    pub opcode: u32,
    pub argument: u32,
    pub returned: u32,
    pub return_kind: u8,
    pub capture: u8,
    pub orientation: u16,
    pub f_source: u16,
    pub health: u16,
    pub area: u16,
    pub host: [u16; 8],
}
impl NativeEvent {
    fn attack(&self) -> bool {
        self.bank == 0xa6 && matches!(self.header, 0xe13f | 0xe17f) && (0xe73a..0xe828).contains(&self.entry_pc)
    }
    fn capture_stem(&self) -> String {
        let address = if self.kind == "frame" { self.entry_pc } else { self.opcode as u16 };
        format!("enemy-{}-{address:04x}-window{}-{:02}", self.kind, self.window, self.capture)
    }
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct WindowOwnership {
    pub frames: usize,
    pub first_frame: u32,
    pub last_frame: u32,
    pub timer_due: usize,
    pub frozen: usize,
    pub retained_frames: usize,
    pub retained_callbacks: usize,
    pub opcode_counts: BTreeMap<String, usize>,
}
#[derive(Clone, Debug, Serialize)]
pub struct NaturalCapture {
    pub name: String,
    pub symbol: String,
    pub before: NativeEvent,
    pub after: NativeEvent,
    pub before_wram: PathBuf,
    pub after_wram: PathBuf,
    pub before_host: PathBuf,
    pub after_host: PathBuf,
    pub opcodes: Vec<NativeEvent>,
}
#[derive(Debug, Serialize)]
pub struct CapturedGeneration {
    pub inventory: PathBuf,
    pub inventory_verdict_sha256: String,
    pub manifest: PathBuf,
    pub manifest_sha256: String,
    pub authenticated_artifacts: usize,
    pub windows: BTreeMap<u8, WindowOwnership>,
    pub captures: Vec<NaturalCapture>,
}
fn callback_symbol(opcode: u32) -> Result<&'static str> {
    Ok(match opcode {
        0xa6e517 => "Ridley_Instr_1",
        0xa6e4be => "Ridley_Instr_5",
        0xa6e4ca => "Ridley_Instr_6",
        0xa6e4d2 => "Ridley_Instr_10",
        0xa6e84d => "Ridley_Instr_11",
        0xa6e904 => "Ridley_Instr_12",
        0xa6e909 => "Ridley_Instr_13",
        0xa6812f => "EnemyInstr_Sleep",
        _ => anyhow::bail!("unexpected opcode in native Ridley attack closure: {opcode:06x}"),
    })
}
fn captured_host(path: &Path) -> Result<[u16; 8]> {
    let bytes = fs::read(path)?;
    ensure!(bytes.len() == 16, "native host capture must contain all eight words");
    Ok(std::array::from_fn(|i| u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]])))
}
fn authenticate_inventory(inventory: &Path, rom_path: &Path) -> Result<usize> {
    let verdict: toml::Value = toml::from_str(&fs::read_to_string(inventory.join("verdict.toml"))?)?;
    ensure!(
        verdict["status"].as_str() == Some("diagnostic-complete") && verdict["frames"].as_integer() == Some(129712),
        "native inventory must be a completed full replay"
    );
    ensure!(
        verdict["source"]["revision"].as_str() == Some("578f90b3cc49557bb70060ad033bb90b8cf8ac50"),
        "native source revision differs"
    );
    ensure!(
        verdict["rom_sha256"].as_str() == Some(ROM_SHA256) && digest(rom_path)? == ROM_SHA256,
        "native inventory ROM differs"
    );
    ensure!(
        verdict["inputs_sha256"].as_str() == Some("cc643d751554ad6d30b5d7e3a41a4259525517ca7614cf73abec9fac30176c0e"),
        "native input identity differs"
    );
    let artifacts = verdict["artifacts"].as_table().context("native inventory artifact seal missing")?;
    for (name, hash) in artifacts {
        ensure!(
            !Path::new(name).is_absolute()
                && Path::new(name).components().all(|c| matches!(c, std::path::Component::Normal(_))),
            "unsafe native artifact path"
        );
        let path = inventory.join(name);
        ensure!(
            !path.is_symlink() && digest(&path)? == hash.as_str().context("invalid native artifact digest")?,
            "native artifact identity differs: {name}"
        );
    }
    for (file, expected) in [
        ("sm_a0.c", "a2cee5c286304e7d606357ac4960413e5fe3dfe585d468d0f41a654aa093e8d3"),
        ("sm_a6.c", "36ce5f36896e6a28555fba833be68fc189bf1a35b17a64cab04cd3a4e34e6b96"),
        ("sm_86.c", "8268f45c3df230de54c99d833e166802d59bd942ea4ac58e75f4acf134e48cf1"),
    ] {
        ensure!(
            digest(&inventory.join("original-source").join(file))? == expected,
            "native gameplay source identity differs: {file}"
        );
    }
    Ok(artifacts.len())
}

/// Reuse one full native inventory to make a subsystem-sized natural capture batch.
/// Inputs are exact captured state; timers, WRAM and host words are never patched.
pub fn generate_captured(
    checkout: &Path,
    rom_path: &Path,
    inventory: &Path,
    output: &Path,
) -> Result<CapturedGeneration> {
    let inventory = fs::canonicalize(inventory)?;
    let authenticated_artifacts = authenticate_inventory(&inventory, rom_path)?;
    let mut manifest = manifest(rom_path)?;
    manifest.family = "observed-ridley-instructions-captured".into();
    manifest.routines.push(direct("EnemyInstr_Sleep"));
    manifest.scope = Some("Exact naturally observed attack-frame and callback captures from one completed pinned actual-C replay; full WRAM and eight host words. This is offline evidence, not live Rust ownership or original-ROM parity.".into());
    let mut windows = BTreeMap::<u8, WindowOwnership>::new();
    let mut frames = BTreeMap::<u64, Vec<NativeEvent>>::new();
    let mut selected = BTreeMap::<String, (Option<NativeEvent>, Option<NativeEvent>)>::new();
    let mut last_sequence = 0;
    for line in BufReader::new(fs::File::open(inventory.join("enemy-events.jsonl"))?).lines() {
        let event: NativeEvent = serde_json::from_str(&line?)?;
        ensure!(event.sequence == last_sequence + 1, "native ledger sequence differs");
        last_sequence = event.sequence;
        if !event.attack() {
            continue;
        }
        let counts = windows.entry(event.window).or_default();
        if event.kind == "frame" && event.phase == "before" {
            if counts.frames == 0 {
                counts.first_frame = event.frame;
            }
            counts.frames += 1;
            counts.last_frame = event.frame;
            counts.timer_due += usize::from(event.timer == 1);
            counts.frozen += usize::from(event.ai_bits & 4 != 0);
        }
        if event.kind == "opcode" {
            callback_symbol(event.opcode)?;
            if event.phase == "before" {
                *counts.opcode_counts.entry(format!("{:06x}", event.opcode)).or_default() += 1;
            } else {
                frames.entry(event.frame_sequence).or_default().push(event.clone());
            }
        }
        if event.capture > 0 {
            let pair = selected.entry(event.capture_stem()).or_default();
            let phase = if event.phase == "before" { &mut pair.0 } else { &mut pair.1 };
            ensure!(phase.replace(event).is_none(), "native capture phase duplicated");
        }
    }
    ensure!(
        windows.len() == 2 && windows.contains_key(&1) && windows.contains_key(&2),
        "native attack combat windows differ"
    );
    let mut captures = Vec::new();
    let mut order = Vec::new();
    for (name, (before, after)) in selected {
        let before = before.context("native capture before event missing")?;
        let after = after.context("native capture after event missing")?;
        ensure!(
            before.frame_sequence == after.frame_sequence
                && before.frame == after.frame
                && before.slot == after.slot
                && before.opcode == after.opcode
                && before.argument == after.argument,
            "native capture event pair differs"
        );
        let before_wram = inventory.join("fixtures").join(format!("{name}-before.wram"));
        let after_wram = inventory.join("fixtures").join(format!("{name}-after.wram"));
        let before_host = inventory.join("fixtures").join(format!("{name}-before.host"));
        let after_host = inventory.join("fixtures").join(format!("{name}-after.host"));
        ensure!(
            fs::metadata(&before_wram)?.len() == 0x20000 && fs::metadata(&after_wram)?.len() == 0x20000,
            "native capture WRAM extent differs"
        );
        ensure!(
            captured_host(&before_host)? == before.host && captured_host(&after_host)? == after.host,
            "native host sidecar differs from event"
        );
        let counts = windows.get_mut(&before.window).unwrap();
        let symbol = if before.kind == "frame" {
            counts.retained_frames += 1;
            "ProcessEnemyInstructions"
        } else {
            counts.retained_callbacks += 1;
            callback_symbol(before.opcode)?
        };
        routine(&mut manifest.routines, symbol).cases.push(Case {
            name: name.clone(),
            a: if before.kind == "opcode" { i64::from(before.slot) } else { 0 },
            b: if before.kind == "opcode" { i64::from(before.argument) } else { 0 },
            before_file: Some(before_wram.clone()),
            before_sha256: Some(digest(&before_wram)?),
            host_words: Some(before.host.to_vec()),
            expected_return: Some(if before.kind == "opcode" { u64::from(after.returned) } else { 0 }),
            expected_after_file: Some(after_wram.clone()),
            expected_after_sha256: Some(digest(&after_wram)?),
            ..Default::default()
        });
        let opcodes = frames.get(&before.frame_sequence).cloned().unwrap_or_default();
        order.push(name.clone());
        captures.push(NaturalCapture {
            name,
            symbol: symbol.into(),
            before,
            after,
            before_wram,
            after_wram,
            before_host,
            after_host,
            opcodes,
        });
    }
    // Native generic sleep is normally retained via its complete frame. Include
    // its signature only if the capture policy retained a direct attack call.
    manifest.routines.retain(|r| !r.cases.is_empty());
    manifest.case_order = Some(order);
    ensure!(checkout.join(&manifest.rust_module).is_file(), "Ridley gameplay module missing");
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    atomic_toml(output, &manifest)?;
    Ok(CapturedGeneration {
        inventory_verdict_sha256: digest(&inventory.join("verdict.toml"))?,
        inventory,
        manifest: fs::canonicalize(output)?,
        manifest_sha256: digest(output)?,
        authenticated_artifacts,
        windows,
        captures,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_raw_angle_fixtures_cover_every_value() {
        let pairs = angle_coordinates();
        assert_eq!(pairs.len(), 256);
        assert!(pairs.iter().all(|(x, y)| (*x as i16).unsigned_abs() <= 255 && (*y as i16).unsigned_abs() <= 255));
    }
    #[test]
    fn native_frame_chains_inherit_state_without_timer_or_host_replacement() {
        let mut routines = vec![Routine { symbol: "ProcessEnemyInstructions".into(), ..Default::default() }];
        let mut order = Vec::new();
        assert_eq!(frame_cases(&mut routines, &mut order), 906);
        assert_eq!(order.len(), 906);
        let chain = routines[0].cases.iter().filter(|c| c.continue_from.is_some()).collect::<Vec<_>>();
        assert_eq!(chain.len(), 880);
        assert!(chain.iter().all(|c| c.writes.is_empty() && c.host_words.is_none()));
    }
}
