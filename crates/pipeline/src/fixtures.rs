//! Deterministic whole-WRAM fixture domains for actual C/Rust comparison.

use anyhow::{Result, ensure};
use offline_verification::fixtures::Case;
use offline_verification::fixtures::Generation;
use offline_verification::fixtures::Manifest;
use offline_verification::fixtures::Routine;
use offline_verification::fixtures::Write;
use offline_verification::fixtures::Writes;
use std::{fs, path::Path};
#[path = "fixtures/host.rs"]
mod host;
#[path = "fixtures/projectiles.rs"]
mod projectiles;
#[path = "fixtures/sequences.rs"]
mod sequences;
pub use sequences::{SequenceRequest, generate_sequence};
fn load(checkout: &Path, name: &str) -> Result<Manifest> {
    offline_verification::fixtures::read(&checkout.join(format!("tools/port/{name}.smf")))
}

// These bounded domains are maintained directly as typed case matrices. Keeping
// one declaration preserves their explicit chains and avoids a second generator
// implementation drifting from the checked-in actual-ROM cases.
const DECLARED_DOMAINS: &[&str] = &[
    "mother-brain-blue-ring-allocation",
    "mother-brain-blue-ring-lifecycle",
    "mother-brain-blue-ring-frames",
    "mother-brain-bomb-allocation",
    "mother-brain-bomb-movement",
    "mother-brain-bomb-collision",
    "mother-brain-bomb-update",
    "mother-brain-bomb-frames",
    "enemy-projectile-hit-samus",
    "enemy-projectile-detect-samus",
    "enemy-projectile-hit-projectile",
    "enemy-projectile-detect-projectiles",
    "enemy-projectile-rendering",
    "enemy-projectile-rendering-direct",
    "mother-brain-glass-allocation",
    "mother-brain-glass-motion",
    "mother-brain-glass-frames",
    "mother-brain-glass-callers",
    "mother-brain-glass-sequence",
    "observed-projectile-initializers",
    "observed-projectile-ownership",
    "observed-projectile-frame-boundaries",
    "sparks-allocation",
    "sparks-update",
    "sparks-frames",
    "sparks-sequence",
    "n00b-tube",
    "n00b-tube-callers",
    "n00b-tube-sequence",
];

fn declared(checkout: &Path, name: &str) -> Result<Manifest> {
    let manifest = load(checkout, name)?;
    ensure!(manifest.family == name && !manifest.routines.is_empty(), "fixture domain identity differs");
    if let Some(asset) = &manifest.rom {
        let path = crate::documents::resolve(checkout, &asset.file.to_string_lossy());
        ensure!(crate::digest(&path)? == asset.sha256, "fixture ROM identity differs");
    }
    let mut names = std::collections::BTreeSet::new();
    for routine in &manifest.routines {
        ensure!(routine.has_valid_symbol(), "unsafe fixture routine");
        routine.argument_types()?;
        for case in &routine.cases {
            ensure!(
                !case.name.is_empty()
                    && case.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                    && names.insert(case.name.clone()),
                "unsafe or duplicate fixture case"
            );
            ensure!(case.host_words.as_ref().is_none_or(|words| words.len() == 8), "fixture host extent differs");
            for write in &case.writes {
                ensure!(
                    write.offset.checked_add(write.data.len()).is_some_and(|end| end <= 0x20000),
                    "fixture write outside canonical WRAM"
                );
            }
        }
    }
    if let Some(order) = &manifest.case_order {
        let ordered: std::collections::BTreeSet<_> = order.iter().cloned().collect();
        ensure!(ordered == names && ordered.len() == order.len(), "fixture case order differs");
    }
    for case in manifest.routines.iter().flat_map(|routine| &routine.cases) {
        if let Some(previous) = &case.continue_from {
            ensure!(names.contains(previous) && previous != &case.name, "fixture predecessor missing");
        }
    }
    Ok(manifest)
}
fn words(values: impl IntoIterator<Item = (usize, i64)>) -> Writes {
    values
        .into_iter()
        .map(|(offset, value)| Write {
            offset,
            data: vec![(value & 255), ((value >> 8) & 255)].into_iter().map(|v| v as u8).collect(),
        })
        .collect()
}
fn routine(symbol: &str, args: usize, result: &str, cases: Vec<Case>) -> Routine {
    Routine { symbol: (symbol).to_owned(), args, returns: (result).to_owned(), cases, ..Default::default() }
}
fn control() -> Routine {
    Routine {
        symbol: ("EprojInstruction").to_owned(),
        arg_types: Some((["uint16", "rom_ptr"]).into_iter().map(str::to_owned).collect()),
        returns: ("rom_ptr").to_owned(),
        cases: vec![Case { name: "sleep-control".into(), a: 0, b: 0x869754, ..Default::default() }],
        ..Default::default()
    }
}

fn sprites(checkout: &Path, name: &str) -> Result<Manifest> {
    let mut manifest = load(checkout, if name == "sprite-objects" { "ceres-platform" } else { "sprite-objects" })?;
    let mut routines = Vec::new();
    if name == "sprite-objects" {
        manifest.bank = "sm_b4.c".into();
        manifest.rust_module = "crates/port/src/sprite_objects.rs".into();
        for symbol in [
            "SpriteObject_Instr_RepeatLast",
            "SpriteObject_Instr_Terminate",
            "SpriteObject_Instr_Goto",
            "HandleSpriteObjects",
            "ClearSpriteObjects",
        ] {
            let mut cases = Vec::new();
            if symbol == "ClearSpriteObjects" {
                cases.extend([0, 165, 255].map(|fill| Case {
                    name: (format!("clear-{fill}")).to_owned(),
                    fill: Some(fill),
                    ..Default::default()
                }));
            }
            for slot in (0..64).step_by(2) {
                if symbol.starts_with("SpriteObject_Instr_") {
                    let pointers = if symbol.ends_with("Goto") {
                        vec![0xbf70, 0xc1a8, 0xc300]
                    } else {
                        vec![0, 2, 0xbe5a, 0xfffe]
                    };
                    for pointer in pointers {
                        cases.push(Case {
                            name: (format!("{symbol}-{slot}-{pointer}")).to_owned(),
                            fill: Some(165),
                            writes: words([(0x1844, slot as i64), (0xef78 + slot, pointer)]),
                            ..Default::default()
                        });
                    }
                } else if symbol == "HandleSpriteObjects" {
                    for (pointer, timer) in [
                        (0xbe5a, 0),
                        (0xbe5a, 1),
                        (0xbe5a, 2),
                        (0xbf0c, 1),
                        (0xbe54, 1),
                        (0xbf6c, 1),
                        (0xbe58, 0xbcf0),
                        (0xbf10, 0xbd07),
                        (0xbf70, 0xbd12),
                    ] {
                        for disabled in [0, 1, 2] {
                            cases.push(Case {
                                name: (format!("frame-{slot}-{pointer}-{timer}-{disabled}")).to_owned(),
                                writes: words([
                                    (0xef78 + slot, pointer),
                                    (0xeff8 + slot, timer),
                                    (0xf2f8 + slot, disabled),
                                ]),
                                ..Default::default()
                            });
                        }
                    }
                }
            }
            if symbol == "HandleSpriteObjects" {
                cases.extend([0x185e, 0xa78].map(|address| Case {
                    name: (format!("frozen-{address}")).to_owned(),
                    fill: Some(165),
                    writes: words([(address, 1)]),
                    ..Default::default()
                }));
            }
            routines.push(routine(symbol, 0, "void", cases));
        }
    } else if name == "sprite-allocation" {
        let mut cases = Vec::new();
        for free in -1i64..32 {
            for list in [0u32, 1, 3, 7, 8, 9, 15, 21, 24, 29, 43, 44, 45, 46, 61] {
                for (x, y, palette) in [(0u32, 0u32, 0u32), (0xffff, 0x8000, 0xffff), (123, 456, 0xe05)] {
                    cases.push(Case {
                        name: (format!("alloc-{free}-{list}-{x}")).to_owned(),
                        a: (x | (y << 16)) as i64,
                        b: (list | (palette << 16)) as i64,
                        fill: Some(165),
                        writes: words((0..32).map(|i| (0xef78 + i * 2, if i as i64 == free { 0 } else { 0xbe5a }))),
                        ..Default::default()
                    });
                }
            }
        }
        routines.push(Routine {
            symbol: ("CreateSpriteAtPos").to_owned(),
            arg_types: Some((["uint32", "uint32"]).into_iter().map(str::to_owned).collect()),
            packed_u16_args: Some(4),
            returns: ("uint16").to_owned(),
            cases,
            ..Default::default()
        });
    } else {
        let mut cases = Vec::new();
        for slot in [0, 30, 62] {
            for x in [0xffef, 0xfff0, 0xffff, 0, 255, 271, 272, 0x7fff, 0x8000] {
                for y in [0xffff, 0, 255, 271, 272, 0x8000] {
                    for cursor in [0, 28, 508] {
                        cases.push(Case {
                            name: (format!("draw-{slot}-{x}-{y}-{cursor}")).to_owned(),
                            writes: words([
                                (0xef78 + slot, 0xbe6c),
                                (0xf0f8 + slot, x),
                                (0xf1f8 + slot, y),
                                (0xf078 + slot, 0xffff),
                                (0x590, cursor),
                                (0x570, 0x5555),
                            ]),
                            ..Default::default()
                        });
                    }
                }
            }
        }
        if !manifest.c_support.contains(&"sm_81.c".to_owned()) {
            manifest.c_support.push("sm_81.c".into());
        }
        routines.push(routine("DrawSpriteObjects", 0, "void", cases));
    }
    manifest.family = name.into();
    manifest.routines = routines;
    Ok(manifest)
}
fn motion(checkout: &Path, name: &str) -> Result<Manifest> {
    if name == "projectile-paired-live-leaves" {
        let mut manifest = motion(checkout, "projectile-paired-motion")?;
        manifest.family = name.into();
        for routine in &mut manifest.routines {
            routine.live_eligible = Some(routine.symbol != "EprojInstruction");
        }
        return Ok(manifest);
    }
    let mut manifest = load(checkout, "tourian-statue-effects")?;
    let mut motion = Vec::new();
    let mut extent = Vec::new();
    let mut pre = Vec::new();
    let mut frames = Vec::new();
    for i in 0..256 {
        let k = [0, 2, 16, 34][i % 4];
        motion.push(Case {
            name: (format!("motion-{i}")).to_owned(),
            a: (k) as i64,
            writes: words([
                (0x97dc + k, (0xa500 | i) as i64),
                (0x1a4b + k, (i * 997) as i64),
                (0x1a27 + k, 0xffa5),
                (0x1a93 + k, (i * 733) as i64),
                (0x1a6f + k, 0xff5a),
                (0x1ab7 + k, (i * 127) as i64),
                (0x1aff + k, (i * 1777) as i64),
                (0x1adb + k, (i * 233) as i64),
                (0x1b23 + k, (i * 3331) as i64),
            ]),
            ..Default::default()
        });
        if name == "botwoon-spit-scheduling" {
            let camera = [0, 0xff80, 0x7fff][i % 3];
            let values = vec![
                (0x1997 + k, 1),
                (0x1a03 + k, 0xec05),
                (0x1b47 + k, 0x9754),
                (0x1b8f + k, 2),
                (0x97dc + k, i as i64),
                (0x911, camera),
                (0x915, camera),
                (0x1a4b + k, camera + [-1, 0, 1, 255, 256, 257, 128, 0x8000][i % 8]),
                (0x1a93 + k, camera + [0, 255, 256, 257][i % 4]),
                (0x1a27 + k, 0xffa5),
                (0x1a6f + k, 0xff5a),
                (0x1ab7 + k, 1),
                (0x1aff + k, 0x8000),
                (0x1adb + k, 2),
                (0x1b23 + k, 0x4000),
            ];
            pre.push(Case {
                name: (format!("pre-{i}")).to_owned(),
                a: (k) as i64,
                b: 0xec05_i64,
                writes: words(values.clone()),
                ..Default::default()
            });
            frames.push(Case {
                name: (format!("frame-{i}")).to_owned(),
                writes: words(values.into_iter().chain([(0x198d, 0x8000)])),
                ..Default::default()
            });
        }
    }
    for (i, x) in [0, 1, 511, 512, 513, 0x7fff, 0x8000, 0xffff].into_iter().enumerate() {
        for (j, y) in [0, 1, 511, 512, 513, 0x7fff, 0x8000, 0xffff].into_iter().enumerate() {
            let k = [0, 2, 16, 34][i % 4];
            extent.push(Case {
                name: (format!("extent-{i}-{j}")).to_owned(),
                a: (k) as i64,
                writes: words([(0x1a4b + k, x), (0x1a93 + k, y)]),
                ..Default::default()
            });
        }
    }
    let mut routines = vec![
        control(),
        routine("Eproj_FuncE73E_MoveXY", 1, "void", motion),
        routine("Eproj_FuncE722", 1, "uint16", extent),
    ];
    if name == "botwoon-spit-scheduling" {
        routines.push(routine("EprojPreinstruction", 2, "void", pre));
        routines.push(routine("EprojRunAll", 0, "void", frames));
    }
    manifest.family = name.into();
    manifest.routines = routines;
    Ok(manifest)
}
fn script_leaves(checkout: &Path) -> Result<Manifest> {
    let mut manifest = load(checkout, "ceres-platform")?;
    let mut routines = Vec::new();
    for symbol in ["EprojInstr_Delete", "EprojInstr_SetPreInstr_", "EprojInstr_ClearPreInstr", "EprojInstr_Goto"] {
        let mut cases = Vec::new();
        for slot in (0..36).step_by(2) {
            for pointer in [0x868000, 0x868161, 0x869754, 0x86a28b, 0x86fffd] {
                if symbol == "EprojInstr_Goto" && pointer == 0x868161 {
                    continue;
                }
                cases.push(Case {
                    name: (format!("{symbol}-{slot}-{pointer:x}")).to_owned(),
                    a: (slot) as i64,
                    b: (pointer) as i64,
                    writes: words([
                        (0x1997 + slot, 0xbeef),
                        (0x1a03 + slot, 0xabcd),
                        (0x1997 + (slot + 2) % 36, 0xface),
                        (0x1a03 + (slot + 2) % 36, 0x1234),
                    ]),
                    ..Default::default()
                });
            }
        }
        routines.push(Routine {
            symbol: (symbol).to_owned(),
            arg_types: Some((["uint16", "rom_ptr"]).into_iter().map(str::to_owned).collect()),
            returns: ("rom_ptr").to_owned(),
            pointer_bank: Some(0x86),
            cases,
            live_eligible: Some(true),
            ..Default::default()
        });
    }
    manifest.family = "projectile-script-leaves".into();
    manifest.routines = routines;
    Ok(manifest)
}
fn drops(checkout: &Path, name: &str) -> Result<Manifest> {
    let mut manifest = load(checkout, "botwoon-spit-scheduling")?;
    let mut cases = Vec::new();
    for header in [0, 0xd37f, 0xd3bf, 0xd3ff, 0xe0ff, 0xf593] {
        for i in 0..256 {
            let k = [0, 2, 16, 34][i % 4];
            let health = [0, 29, 30, 49, 50, 99, 0xffff, 0xffe0][i % 8];
            let mut values = vec![
                (0xf3c8 + k, header),
                (0x5e5, (i * 257) as i64),
                (0xe1a, [0, 1, 0x100, 0x101][i % 4]),
                (0x9c2, health),
                (0x9d6, [0, 1, 30, 100][(i / 8) % 4]),
                (0x9c4, 99),
                (0x9d4, 100),
            ];
            for (j, (current, maximum)) in [(0x9c6, 0x9c8), (0x9ca, 0x9cc), (0x9ce, 0x9d0)].into_iter().enumerate() {
                values.extend([(current, if i & (1 << (j + 2)) != 0 { 10 } else { 9 }), (maximum, 10)]);
            }
            cases.push(Case {
                name: (format!("drop-{header:04x}-{i}")).to_owned(),
                a: (k) as i64,
                writes: words(values),
                ..Default::default()
            });
        }
    }
    for seed in 0i64..65536 {
        let low = (seed & 255) * 5;
        let carry = (low >> 8) + (((seed >> 8) * 5) & 255) + 1;
        let next = (((low & 255) | ((carry & 255) << 8)) + (carry >> 8) + 17) & 65535;
        if next & 255 == 0 {
            cases.push(Case {
                name: (format!("zero-retry-{seed}")).to_owned(),
                a: 0_i64,
                writes: words([(0xf3c8, 0xd37f), (0x5e5, seed)]),
                ..Default::default()
            });
        }
    }
    if name == "enemy-drop-selection" {
        manifest.routines = vec![manifest.routines[0].clone(), routine("RandomDropRoutine", 1, "uint16", cases)];
    } else {
        let mut direct = Vec::new();
        let mut frames = Vec::new();
        for (i, case) in cases.into_iter().enumerate() {
            let k = case.a as usize;
            let mut writes = case.writes.clone();
            writes.extend(words([
                (0x1997 + k, 1),
                (0x1a03 + k, 0x8170),
                (0x1b47 + k, 0xecc1),
                (0x1b8f + k, 1),
                (0x1bd7 + k, [0xffff, 0x4000, 0x1234, 0][i % 4]),
                (0x1aff + k, 0xa55a),
                (0x1b23 + k, 0x5aa5),
            ]));
            direct.push(Case {
                name: (case.name.clone()).to_owned(),
                a: (k) as i64,
                b: 0x86ecc1_i64,
                writes: writes.clone(),
                ..Default::default()
            });
            if i % 4 == 0 || i < 256 {
                writes.extend(words([(0x198d, 0x8000)]));
                frames.push(Case { name: (format!("frame-{}", case.name)).to_owned(), writes, ..Default::default() });
            }
        }
        manifest.routines = vec![
            Routine {
                symbol: ("EprojInstruction").to_owned(),
                arg_types: Some((["uint16", "rom_ptr"]).into_iter().map(str::to_owned).collect()),
                returns: ("rom_ptr").to_owned(),
                cases: direct,
                ..Default::default()
            },
            routine("EprojRunAll", 0, "void", frames),
        ];
    }
    manifest.family = name.into();
    Ok(manifest)
}
fn linked(checkout: &Path, name: &str) -> Result<Manifest> {
    let mut manifest = load(
        checkout,
        if name == "enemy-death-explosion" { "botwoon-body-allocation" } else { "tourian-statue-effects" },
    )?;
    let mut cases = Vec::new();
    for definition in if name == "enemy-death-explosion" { vec![0, 1, 2, 3, 4] } else { vec![0xbb50, 0xbbc7] } {
        for i in 0..192 {
            let enemy = (i % if name == "enemy-death-explosion" { 32 } else { 16 }) * 64;
            let target = enemy + ((i / 16) % 8) * 64;
            let mut values = (0..36).step_by(2).map(|s| (0x1997 + s, 1)).collect::<Vec<_>>();
            if let Some(free) = [Some(0), Some(2), Some(16), Some(34), None][i % 5] {
                values.push((0x1997 + free, 0));
            }
            if name == "enemy-death-explosion" {
                values.extend([
                    (0xe54, enemy as i64),
                    (0xf7a + enemy, (i * 997) as i64),
                    (0xf7e + enemy, (i * 733) as i64),
                    (0xf78 + enemy, (0x8000 + i * 17) as i64),
                    (0xf86 + enemy, [0, 0x4000, 0xffff, 0xbfff][(i / 32) % 4]),
                    (0x1c27, 0xffff),
                ]);
                values.extend((0..36).step_by(2).flat_map(|s| [0xf3c8, 0xf410, 0x19bb].map(|base| (base + s, 0xa55a))));
                cases.push(Case {
                    name: (format!("animation-{definition}-{i}")).to_owned(),
                    a: 0xf345_i64,
                    b: (definition) as i64,
                    writes: words(values),
                    ..Default::default()
                });
            } else {
                values.extend([
                    (0xe54, enemy as i64),
                    (0x8014 + enemy, (target - enemy) as i64),
                    (0x8022 + enemy, (i * 997) as i64),
                    (0x8024 + enemy, (i * 733) as i64),
                    (0x7800 + target, 0xdead),
                ]);
                values.extend([2, 4, 6, 8].map(|d| (0xf78 + enemy + d, (i * 1777 + d * 93) as i64)));
                values.extend((0..36).step_by(2).map(|s| (0xf380 + s, (0x1234 + s) as i64)));
                cases.push(Case {
                    name: (format!("spawn-{definition:x}-{i}")).to_owned(),
                    a: (definition) as i64,
                    b: (i % 3) as i64,
                    writes: words(values),
                    ..Default::default()
                });
            }
        }
    }
    if name == "enemy-death-explosion" {
        manifest.routines[1].cases = cases;
    } else {
        manifest.routines = vec![control(), routine("SpawnEprojWithRoomGfx", 2, "void", cases)];
    }
    manifest.family = name.into();
    Ok(manifest)
}
fn rom(checkout: &Path) -> Result<Vec<u8>> {
    let path = checkout.join("rom/sm.sfc");
    ensure!(
        crate::digest(&path)? == "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72",
        "fixture ROM identity differs"
    );
    Ok(fs::read(path)?)
}
fn bank_units(checkout: &Path, excluded: &str) -> Result<Vec<String>> {
    Ok(decomp_source::c_units(&checkout.join("deps/snesrev-sm"))?
        .into_iter()
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            (name.starts_with("sm_") && name != excluded).then(|| name.to_owned())
        })
        .collect::<Vec<_>>())
}
fn palette(checkout: &Path) -> Result<Manifest> {
    let mut manifest = load(checkout, "palette-effect-allocation")?;
    let rom = rom(checkout)?;
    let mut routines = Vec::new();
    for symbol in ["PalFx_ProcessOne", "PaletteFxHandler"] {
        let mut cases = Vec::new();
        for slot in (0..16).step_by(2) {
            for header in [0xe194, 0xe1a0, 0xe1a4, 0xe1bc, 0xf761, 0xf779] {
                let offset = 0x68000 + (header & 0x7fff) + 2;
                let pointer = u16::from_le_bytes([rom[offset], rom[offset + 1]]);
                for timer in [0, 1, 2, 0xffff] {
                    let mut case = Case {
                        name: (format!("{symbol}-{slot}-{header}-{timer}")).to_owned(),
                        writes: words([
                            (0x1e7b, slot as i64),
                            (0x1e79, 0x8000),
                            (0x1e7d + slot, header as i64),
                            (0x1ead + slot, 0xc526),
                            (0x1ebd + slot, pointer as i64),
                            (0x1ecd + slot, timer),
                            (0x1e8d + slot, 0),
                            (0x79f, 0),
                            (0xd828, 0),
                        ]),
                        ..Default::default()
                    };
                    if symbol == "PalFx_ProcessOne" {
                        case.a = slot as i64;
                    }
                    cases.push(case);
                }
            }
        }
        routines.push(routine(symbol, usize::from(symbol == "PalFx_ProcessOne"), "void", cases));
    }
    for symbol in ["EnablePaletteFx", "DisablePaletteFx", "ClearPaletteFXObjects"] {
        routines.push(routine(
            symbol,
            0,
            "void",
            [0, 1, 0x8000, 0xffff]
                .map(|flag| Case {
                    name: (format!("{symbol}-{flag}")).to_owned(),
                    writes: words([(0x1e79, flag)]),
                    ..Default::default()
                })
                .to_vec(),
        ));
    }
    manifest.family = "palette-effect-frames".into();
    manifest.c_support = bank_units(checkout, "sm_8d.c")?;
    manifest.rust_imports = vec![
        "audio",
        "system",
        "enemy_projectiles",
        "enemy_projectile_instructions",
        "enemy_projectile_collision",
        "enemy",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    manifest.routines = routines;
    Ok(manifest)
}
fn projectile_sprites(checkout: &Path) -> Result<Manifest> {
    let mut manifest = load(checkout, "projectile-script-leaves")?;
    let rom = rom(checkout)?;
    let mut routines = Vec::new();
    for (symbol, opcode) in [("EprojInstr_ECE3", 0xece3), ("EprojInstr_ED17", 0xed17)] {
        let operands = (0x30000..0x37ffc)
            .filter(|offset| {
                u16::from_le_bytes([rom[*offset], rom[*offset + 1]]) == opcode
                    && u16::from_le_bytes([rom[*offset + 2], rom[*offset + 3]]) <= 61
            })
            .map(|offset| 0x868000 + offset - 0x30000 + 2)
            .collect::<Vec<_>>();
        ensure!(!operands.is_empty(), "no native operands for {symbol}");
        let mut cases = Vec::new();
        for pointer in operands {
            for slot in [0, 16, 34] {
                for seed in [0, 1, 0x1234, 0xffff] {
                    for free in [-1i64, 0, 31] {
                        let mut values = vec![(0x5e5, seed), (0x1a4b + slot, 0), (0x1a93 + slot, 0xffff)];
                        values.extend((0..32).map(|i| (0xef78 + i * 2, if i as i64 == free { 0 } else { 0xbe5a })));
                        cases.push(Case {
                            name: (format!("{symbol}-{pointer}-{slot}-{seed}-{free}")).to_owned(),
                            a: (slot) as i64,
                            b: (pointer) as i64,
                            writes: words(values),
                            ..Default::default()
                        });
                    }
                }
            }
        }
        routines.push(Routine {
            symbol: (symbol).to_owned(),
            arg_types: Some((["uint16", "rom_ptr"]).into_iter().map(str::to_owned).collect()),
            returns: ("rom_ptr").to_owned(),
            pointer_bank: Some(134),
            cases,
            ..Default::default()
        });
    }
    manifest.family = "projectile-sprite-spawns".into();
    manifest.c_support = bank_units(checkout, "sm_86.c")?;
    manifest.routines = routines;
    Ok(manifest)
}
fn room_explosions(checkout: &Path) -> Result<Manifest> {
    let mut manifest = load(checkout, "room")?;
    let mut routines = Vec::new();
    for symbol in [
        "RoomCode_GenRandomExplodes",
        "RoomCode_GenRandomExplodes_Nonblank",
        "RoomCode_GenRandomExplodes_4th",
        "RoomCode_ExplodeShakes",
    ] {
        let mut cases = Vec::new();
        for seed in 0..16 {
            for (frozen, frame) in [(0, 0), (0, 1), (0, 2), (1, 0)] {
                for blank in [0, 255] {
                    for full in [false, true] {
                        let mut values = vec![
                            (0x5e5, seed),
                            (0xa78, frozen),
                            (0x5b6, frame),
                            (0x911, 0),
                            (0x915, 0),
                            (0x7a5, 16),
                            (0x1840, 3),
                        ];
                        if full {
                            values.extend((0..32).map(|i| (0xef78 + i * 2, 0xbe5a)));
                        }
                        values.extend((0..16).map(|i| (0x10002 + i * 2, blank)));
                        let mut case = Case {
                            name: (format!(
                                "{symbol}-{seed}-{frozen}-{frame}-{blank}-{}",
                                if full { "True" } else { "False" }
                            ))
                            .to_owned(),
                            writes: words(values),
                            ..Default::default()
                        };
                        if symbol == "RoomCode_GenRandomExplodes" {
                            case.a = 0xffff;
                            case.b = 0x8000;
                        }
                        cases.push(case);
                    }
                }
            }
        }
        routines.push(routine(symbol, if symbol == "RoomCode_GenRandomExplodes" { 2 } else { 0 }, "void", cases));
    }
    manifest.family = "room-random-explosions".into();
    manifest.bank = "sm_8f.c".into();
    manifest.c_support = bank_units(checkout, "sm_8f.c")?;
    manifest.routines = routines;
    Ok(manifest)
}

fn animated(checkout: &Path, name: &str) -> Result<Manifest> {
    if name == "animated-tiles" {
        let mut manifest = load(checkout, "sprite-objects")?;
        manifest.family = name.into();
        manifest.bank = "sm_87.c".into();
        manifest.rust_module = "crates/port/src/animated_tiles.rs".into();
        let mut routines = Vec::new();
        for symbol in ["EnableAnimtiles", "DisableAnimtiles", "ClearAnimtiles"] {
            routines.push(routine(
                symbol,
                0,
                "void",
                [0, 1, 0x7fff, 0x8000, 0xffff]
                    .into_iter()
                    .map(|flag| Case {
                        name: (format!("{symbol}-{flag}")).to_owned(),
                        fill: Some(165),
                        writes: words([(0x1ef1, flag)]),
                        ..Default::default()
                    })
                    .collect(),
            ));
        }
        let mut cases = Vec::new();
        for free in -1i64..6 {
            for header in [0x82ab, 0x82c9, 0x82e7, 0x82fd] {
                cases.push(Case {
                    name: (format!("spawn-{free}-{header}")).to_owned(),
                    a: (header) as i64,
                    fill: Some(165),
                    writes: words((0..6).map(|i| (0x1ef5 + i * 2, if i as i64 == free { 0 } else { 0x82ab }))),
                    ..Default::default()
                });
            }
        }
        routines.push(routine("SpawnAnimtiles", 1, "void", cases));
        manifest.routines = routines;
        return Ok(manifest);
    }
    let mut manifest = load(checkout, "animated-tile-events")?;
    manifest.family = name.into();
    manifest.c_support = vec!["sm_rtl.c", "sm_80.c", "sm_8d.c", "sm_86.c"].into_iter().map(str::to_owned).collect();
    manifest.rust_imports = vec![
        "audio",
        "system",
        "enemy_projectiles",
        "enemy_projectile_instructions",
        "enemy_projectile_collision",
        "enemy",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let mut routines = Vec::new();
    for symbol in ["ProcessAnimtilesObject", "AnimtilesHandler"] {
        let mut cases = Vec::new();
        for slot in (0..12).step_by(2) {
            for (label, pointer) in [(0x82b1, 0x8293), (0x82cf, 0x82b1), (0x82ed, 0x82cf), (0x8303, 0x82ed)] {
                for timer in [0, 1, 2, 0xffff] {
                    cases.push(Case {
                        name: (format!("{symbol}-{slot}-{label}-{timer}")).to_owned(),
                        writes: words([
                            (0x1ef3, slot as i64),
                            (0x1ef1, 0x8000),
                            (0x1ef5 + slot, 0x82ab),
                            (0x1f0d + slot, pointer),
                            (0x1f19 + slot, timer),
                        ]),
                        ..Default::default()
                    });
                }
            }
        }
        routines.push(routine(symbol, 0, "void", cases));
    }
    manifest.routines = routines;
    if name == "animated-tile-frame-live" {
        manifest.c_support = decomp_source::c_units(&checkout.join("deps/snesrev-sm"))?
            .into_iter()
            .filter_map(|path| {
                let name = path.file_name()?.to_str()?;
                (name.starts_with("sm_") && name != "sm_87.c").then(|| name.to_owned())
            })
            .collect::<Vec<_>>();
    }
    Ok(manifest)
}
fn ceres(checkout: &Path, name: &str) -> Result<Manifest> {
    let pad = name == "ceres-pad";
    let mut manifest = load(checkout, if pad { "ceres-platform" } else { "projectile-paired-live-leaves" })?;
    let mut cases = Vec::new();
    for slot in (0..36).step_by(2) {
        let timers = if pad { vec![0, 1, 2, 60, 0x7fff, 0x8000, 0x8001, 0xffff] } else { vec![0, 1, 0xffff] };
        let heights = if pad {
            vec![0, 71, 72, 73, 0x7fff, 0x8000, 0x8047, 0x8048, 0xffff]
        } else {
            vec![0, 71, 72, 73, 0x7fff, 0x8000, 0xffff]
        };
        for height in heights {
            for timer in &timers {
                let values = if pad {
                    vec![
                        (0xafa, height),
                        (0x1aff + slot, *timer),
                        (0x1a93 + slot, 0xface),
                        (0x1b47 + slot, 0xabcd),
                        (0x1b8f + slot, 0xbeef),
                        (0xa42, 0x1234),
                        (0xa44, 0x5678),
                    ]
                } else {
                    vec![
                        (0xafa, height),
                        (0x1b8f + slot, *timer),
                        (0x1b47 + slot, 0xabcd),
                        (0x1b47 + (slot + 2) % 36, 0xface),
                        (0x1b8f + (slot + 2) % 36, 0xbeef),
                    ]
                };
                cases.push(Case {
                    name: (if pad {
                        format!("pad-{slot}-{timer}-{height}")
                    } else {
                        format!("platform-{slot}-{height}-{timer}")
                    })
                    .to_owned(),
                    a: (slot) as i64,
                    writes: words(values),
                    ..Default::default()
                });
            }
        }
    }
    let mut entry = routine(
        if pad { "EprojPreInstr_CeresElevatorPad" } else { "EprojPreInstr_CeresElevatorPlatform" },
        1,
        "void",
        cases,
    );
    entry.live_eligible = Some(true);
    manifest.family = name.into();
    manifest.routines = vec![entry];
    Ok(manifest)
}
fn botwoon(checkout: &Path, name: &str) -> Result<Manifest> {
    let mut manifest = load(checkout, "tourian-statue-effects")?;
    let spit = name == "botwoon-spit-allocation";
    let definition = if spit { 0xec48 } else { 0xeba0 };
    let mut cases = Vec::new();
    for i in 0..if spit { 256 } else { 192 } {
        let enemy = (i % 16) * 64;
        let target = enemy + ((i / 16) % 8) * 64;
        let mut values = (0..36).step_by(2).map(|slot| (0x1997 + slot, 1)).collect::<Vec<_>>();
        if let Some(free) = [Some(0), Some(2), Some(16), Some(34), None][i % 5] {
            values.push((0x1997 + free, 0));
        }
        values.extend([
            (0xe54, enemy as i64),
            (0x8014 + enemy, (target - enemy) as i64),
            (0x8022 + enemy, (i * 997) as i64),
            (0x8024 + enemy, (i * 733) as i64),
            (0x7800 + target, 0xdead),
            (0xfa8, target as i64),
        ]);
        if spit {
            values.push((0x60b, (0xa500 | i) as i64));
        }
        values.extend([(0xf7a, (i * 997) as i64), (0xf7e, (i * 733) as i64), (0x7820 + target, 0xbeef)]);
        values.extend([2, 4, 6, 8].map(|d| (0xf78 + enemy + d, (i * 1777 + d * 93) as i64)));
        values.extend((0..36).step_by(2).map(|slot| (0xf380 + slot, (0x1234 + slot) as i64)));
        cases.push(Case {
            name: (format!("spawn-{definition:x}-{i}")).to_owned(),
            a: (definition) as i64,
            b: (if spit { [0, 1, 255, 256, 0x7fff, 0x8000, 0xffff][i % 7] } else { i % 3 }) as i64,
            writes: words(values),
            ..Default::default()
        });
    }
    let mut routines = vec![control(), routine("SpawnEprojWithRoomGfx", 2, "void", cases)];
    if name == "botwoon-body-lifecycle" {
        let mut pre = Vec::new();
        let mut frames = Vec::new();
        for state in [0xea98, 0xeaf4, 0xeb04, 0xeb1f, 0xeb93] {
            for i in 0..64 {
                let k = [0, 10, 16, 34][i % 4];
                let values = vec![
                    (0x1997 + k, 1),
                    (0x1a03 + k, 0xea80),
                    (0x1ab7 + k, state),
                    (0x1aff + k, if state == 0xea98 { [0, 16, 48, 63][i % 4] } else { [0, 254, 255, 256][i % 4] }),
                    (0x1b23 + k, 0xe8f3),
                    (0x1b47 + k, 0x9754),
                    (0x1b8f + k, 2),
                    (0x1a93 + k, [0, 199, 200, 201][i % 4]),
                    (0x1a6f + k, 0xffa5),
                    (0x1adb + k, ((i % 64) * 256) as i64),
                    (0x19bb + k, 0x100),
                    (0x8020, ((i / 4) % 2) as i64),
                    (0xf9c, ((i / 8) % 2) as i64),
                    (0xe44, ((i / 16) % 4) as i64),
                ];
                pre.push(Case {
                    name: (format!("pre-{state:x}-{i}")).to_owned(),
                    a: (k) as i64,
                    b: 0xea80_i64,
                    writes: words(values.clone()),
                    ..Default::default()
                });
                frames.push(Case {
                    name: (format!("frame-{state:x}-{i}")).to_owned(),
                    writes: words(values.into_iter().chain([(0x198d, 0x8000)])),
                    ..Default::default()
                });
            }
        }
        routines.push(routine("EprojPreinstruction", 2, "void", pre));
        routines.push(routine("EprojRunAll", 0, "void", frames));
    }
    manifest.family = name.into();
    manifest.routines = routines;
    Ok(manifest)
}

fn crawler(checkout: &Path) -> Result<Manifest> {
    let mut routines = Vec::new();
    for name in [
        "StoneZoomer_Init",
        "FireZoomer_Init",
        "BigEyeBugs_Init",
        "WreckedShipOrangeZoomer_Init",
        "StoneZoomer_E67A",
        "StoneZoomer_Main",
        "WreckedShipOrangeZoomer_Main",
        "FireZoomer_Func_1",
        "FireZoomer_Func_2",
        "FireZoomer_Func_3",
        "FireZoomer_E8A5",
        "WreckedShipOrangeZoomer_Func_2",
        "sub_A3E168",
        "EnemyFunc_C8AD",
    ] {
        let mut cases = Vec::new();
        for n in 0..56 {
            let k = [0, 64, 128, 1984][n % 4];
            let e = 0xf78 + k;
            let x = 0x7800 + k;
            let state =
                if name == "WreckedShipOrangeZoomer_Main" { 0xe091 } else { [0xe6c8, 0xe785, 0xe7f2, 0xe6c1][n % 4] };
            let mut w = words([
                (0xe54, k as i64),
                (0x7a5, 16),
                (e + 2, 128),
                (e + 4, 0x9000),
                (e + 6, 128),
                (e + 8, 0x7000),
                (e + 10, 8),
                (e + 12, 8),
                (e + 0xe, (n % 4) as i64),
                (e + 0x1a, (n % 4) as i64),
                (e + 0x30, [128, 256, 0xff80, 0xff00][n % 4]),
                (e + 0x32, [128, 0xff80, 256, 0xff00][n % 4]),
                (e + 0x3c, (n % 8) as i64),
                (e + 0x3e, ((n % 6) * 2) as i64),
                (x + 2, [0, 0xc000, 0x8000, 0xffff][n % 4]),
                (x + 4, [0, 2, 4, 0xffff][n % 4]),
                (x + 6, 0xe7f2),
                (x + 8, (n % 5) as i64),
                (e + 0x3a, state),
                (0xaf6, [96, 160][n % 2]),
                (0xafa, [160, 96][n % 2]),
                (0x1840, if n % 3 == 0 { 30 } else { 0 }),
                (0x183e, 20),
            ]);
            if n % 8 == 7 && name != "WreckedShipOrangeZoomer_Init" {
                w.extend(words([(e + 0x3c, 255)]));
            }
            let terrain = n / 8;
            for row in 6..10 {
                for col in 6..10 {
                    let index = row * 16 + col;
                    match terrain {
                        6 => {
                            w.extend(words([(0x10002 + 2 * index, 0xa000)]));
                            w.push(Write {
                                offset: 0x16402 + index,
                                data: vec![15].into_iter().map(|v| v as u8).collect(),
                            });
                        }
                        1 | 3 => w.extend(words([(0x10002 + 2 * index, 0x8000)])),
                        2 | 4 | 5 => {
                            w.extend(words([(0x10002 + 2 * index, 0x1000)]));
                            w.push(Write {
                                offset: 0x16402 + index,
                                data: vec![(5 | if terrain >= 3 { [0, 64, 128][terrain - 3] } else { 0 })]
                                    .into_iter()
                                    .map(|v| v as u8)
                                    .collect(),
                            });
                        }
                        _ => (),
                    }
                }
            }
            let mut case = Case {
                name: (format!("{name}-{n}")).to_owned(),
                a: (k) as i64,
                writes: w.clone(),
                ..Default::default()
            };
            if terrain == 6
                && !(name == "FireZoomer_Func_2" && n % 4 == 0)
                && [
                    "FireZoomer_Func_1",
                    "FireZoomer_Func_2",
                    "FireZoomer_Func_3",
                    "WreckedShipOrangeZoomer_Func_2",
                    "sub_A3E168",
                ]
                .contains(&name)
            {
                case.minimum_callbacks = 1;
            }
            cases.push(case);
        }
        routines.push(Routine { symbol: (name).to_owned(), arg_types: Some((if name.ends_with("_Init")||name.ends_with("_Main") {Vec::<&str>::new()}else{vec!["uint16"]}).into_iter().map(str::to_owned).collect()), returns: (if name=="FireZoomer_E8A5" {"int32"}else if name=="EnemyFunc_C8AD" {"uint8"}else{"void"}).to_owned(), cases, domain: Some(("aligned enemy offset<0x800; speed index<32 or Stone/Fire sentinel255; animation index<6; bounded terrain; native shared collision and explicit PLM host").to_owned()), ..Default::default() });
    }
    for name in ["Zoomer_Instr_SetPreinstr", "WreckedShipOrangeZoomer_Func_1"] {
        let cases = (0..8)
            .map(|i| Case {
                name: (format!("{name}-{i}")).to_owned(),
                a: (i * 64) as i64,
                b: (0xa3e600 + 2 * i) as i64,
                writes: vec![Write { offset: 0xf78 + i * 64 + 0x2e, data: vec![0xa3, 0] }].into(),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        routines.push(Routine {
            symbol: (name).to_owned(),
            arg_types: Some((["uint16", "rom_ptr"]).into_iter().map(str::to_owned).collect()),
            returns: ("rom_ptr").to_owned(),
            domain: Some(("aligned record; readable bank A3 script word, next pointer within bank").to_owned()),
            cases,
            ..Default::default()
        });
    }
    let reference = load(checkout, "enemy-crawler")?;
    Ok(Manifest {
        family: ("enemy-crawler").to_owned(),
        rust_module: ("crates/port/src/enemy_crawler.rs").to_owned(),
        rom: reference.rom.clone(),
        routines,
        ..Default::default()
    })
}
pub fn generate(checkout: &Path, name: &str, output: &Path) -> Result<Generation> {
    let manifest = match name {
        "animated-tiles" | "animated-tile-frames" | "animated-tile-frame-live" => animated(checkout, name)?,
        "ceres-platform" | "ceres-pad" => ceres(checkout, name)?,
        "botwoon-body-allocation" | "botwoon-body-lifecycle" | "botwoon-spit-allocation" => botwoon(checkout, name)?,
        "sprite-objects" | "sprite-allocation" | "sprite-rendering" => sprites(checkout, name)?,
        "projectile-paired-motion" | "projectile-paired-live-leaves" | "botwoon-spit-scheduling" => {
            motion(checkout, name)?
        }
        "projectile-script-leaves" => script_leaves(checkout)?,
        "enemy-drop-selection" | "enemy-drop-transition" => drops(checkout, name)?,
        "enemy-linked-projectile-allocation" | "enemy-death-explosion" => linked(checkout, name)?,
        "palette-effect-frames" => palette(checkout)?,
        "projectile-sprite-spawns" => projectile_sprites(checkout)?,
        "room-random-explosions" => room_explosions(checkout)?,
        "skree-particles"
        | "enemy-falling-particles"
        | "nami-fune-horizontal-projectiles"
        | "lavaman-thrown-lava"
        | "spore-projectile-motion"
        | "yapping-maw-body"
        | "lavaquake-rocks"
        | "shot-gates-electricity"
        | "shaktool-attack-circles"
        | "draygon-projectile-scheduling"
        | "draygon-attached-gunk" => projectiles::generate(checkout, name)?,
        "tourian-statue-scheduling" | "mother-brain-turret-projectiles" | "projectile-pickup-collection" => {
            projectiles::generate(checkout, name)?
        }
        "pirate-host-context"
        | "projectile-host-context"
        | "pirate-host-callers"
        | "host-spore-spawning"
        | "mother-brain-turret-host-lifecycle"
        | "ridley-fireball-host-impact"
        | "projectile-host-drop-instructions" => host::generate(checkout, name)?,
        "enemy-crawler" => crawler(checkout)?,
        name if DECLARED_DOMAINS.contains(&name) => declared(checkout, name)?,
        _ => anyhow::bail!("unknown native fixture generator: {name}"),
    };
    ensure!(manifest.family == name, "fixture family identity differs");
    offline_verification::fixtures::write(output, &manifest)?;
    Ok(Generation {
        manifest: output.into(),
        family: (name).to_owned(),
        cases: manifest.routines.iter().map(|routine| routine.cases.len()).sum::<usize>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checked_in_family_configs_use_typed_contracts() {
        let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
        let mut count = 0;
        for entry in fs::read_dir(checkout.join("tools/port")).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|s| s.to_str()) != Some("smf") {
                continue;
            }
            let manifest =
                offline_verification::fixtures::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            assert!(!manifest.family.is_empty(), "{}: missing family", path.display());
            assert!(!manifest.routines.is_empty() || !manifest.cases.is_empty(), "{}: empty cases", path.display());
            for routine in &manifest.routines {
                assert!(
                    routine.return_type.as_ref().is_none_or(|kind| kind == &routine.returns),
                    "{}: conflicting return declarations",
                    path.display()
                );
            }
            count += 1;
        }
        assert!(count > 100, "incomplete migrated family inventory");
    }
    #[test]
    fn native_domains_match_checked_in_fixture_matrices() {
        let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
        let directory = std::env::temp_dir().join(format!("sm-native-fixtures-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        for name in [
            "animated-tiles",
            "animated-tile-frames",
            "animated-tile-frame-live",
            "ceres-platform",
            "ceres-pad",
            "botwoon-body-allocation",
            "botwoon-body-lifecycle",
            "botwoon-spit-allocation",
            "sprite-objects",
            "sprite-allocation",
            "sprite-rendering",
            "projectile-paired-motion",
            "projectile-paired-live-leaves",
            "botwoon-spit-scheduling",
            "projectile-script-leaves",
            "enemy-drop-selection",
            "enemy-drop-transition",
            "enemy-linked-projectile-allocation",
            "enemy-death-explosion",
            "palette-effect-frames",
            "projectile-sprite-spawns",
            "room-random-explosions",
            "skree-particles",
            "enemy-falling-particles",
            "nami-fune-horizontal-projectiles",
            "lavaman-thrown-lava",
            "spore-projectile-motion",
            "yapping-maw-body",
            "lavaquake-rocks",
            "shot-gates-electricity",
            "shaktool-attack-circles",
            "draygon-projectile-scheduling",
            "draygon-attached-gunk",
            "projectile-host-context",
            "pirate-host-callers",
            "host-spore-spawning",
            "mother-brain-turret-host-lifecycle",
            "ridley-fireball-host-impact",
            "projectile-host-drop-instructions",
            "pirate-host-context",
            "tourian-statue-scheduling",
            "mother-brain-turret-projectiles",
            "projectile-pickup-collection",
            "enemy-crawler",
        ] {
            let output = directory.join(format!("{name}.smf"));
            generate(checkout, name, &output).unwrap();
            let actual = offline_verification::fixtures::read(&output).unwrap();
            let mut expected = load(checkout, name).unwrap();
            if name == "ceres-pad" {
                // Compare case declarations by name; the legacy pad loop ordered timer first.
                expected.routines[0].cases.sort_by_key(|case| case.name.to_string());
                let mut actual = actual;
                actual.routines[0].cases.sort_by_key(|case| case.name.to_string());
                assert!(actual == expected, "{name}: generated fixture domain differs");
            } else {
                assert!(actual == expected, "{name}: generated fixture domain differs");
            }
        }
        fs::remove_dir_all(directory).unwrap();
    }
}
