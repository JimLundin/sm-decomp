//! Deterministic PLM scripts and valid canonical-WRAM fixture domains.
use crate::family_model::{Asset, Case, Manifest, PlmGeneration, Routine, Write};
use anyhow::{Context, Result, ensure};
use std::{fs, path::Path};

pub(crate) const BASE_ROM_SHA256: &str = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72";

fn le(word: usize) -> Vec<u8> {
    (word as u16).to_le_bytes().to_vec()
}
fn words(values: &[usize]) -> Vec<u8> {
    values.iter().flat_map(|v| le(*v)).collect()
}
fn word(offset: usize, value: usize) -> Write {
    Write { offset, data: le(value) }
}
fn write(rom: &mut [u8], bank: usize, pc: usize, data: &[u8]) {
    let offset = (bank & 0x7f) * 0x8000 + (pc & 0x7fff);
    rom[offset..offset + data.len()].copy_from_slice(data);
}
fn slot(seed: usize) -> usize {
    [0, 2, 38, 78][seed % 4]
}
fn fixture(name: String, a: usize, b: usize, writes: Vec<Write>) -> Case {
    Case { name: (name).to_owned(), a: (a) as i64, b: (b) as i64, writes: writes.into(), ..Default::default() }
}
fn returning(name: String, a: Option<usize>, writes: Vec<Write>) -> Case {
    let mut v = Case { name: (name).to_owned(), writes: writes.into(), expected_return: Some(0), ..Default::default() };
    if let Some(a) = a {
        v.a = a as i64;
    }
    v
}
fn tiles(base: usize) -> Vec<u8> {
    (0..8).flat_map(|tile| (0..4).flat_map(move |quadrant| le(base + tile * 16 + quadrant))).collect()
}

pub fn generate(checkout: &Path, output: &Path, manifest_path: Option<&Path>) -> Result<PlmGeneration> {
    fs::create_dir_all(output)?;
    let output = fs::canonicalize(output)?;
    let path = output.join("plm-instructions-synthetic.sfc");
    let (manifest, rom) = generated_contents(checkout, &path)?;
    fs::write(&path, rom)?;
    let destination = manifest_path.map(Path::to_path_buf).unwrap_or(output.join("plm-instructions.smf"));
    fs::create_dir_all(destination.parent().context("manifest parent missing")?)?;
    crate::fixture_format::write(&destination, &manifest)?;
    Ok(PlmGeneration { manifest: fs::canonicalize(destination)?, rom: path, rom_sha256: manifest.rom.unwrap().sha256 })
}

/// Reconstruct the complete deterministic declaration without changing a frozen
/// verification checkout or accepting caller-selected generated fixture rows.
pub(crate) fn generated_contents(checkout: &Path, path: &Path) -> Result<(Manifest, Vec<u8>)> {
    let original = if let Some(view) = crate::input_view::current() {
        view.read(&checkout.join("rom/sm.sfc"), Some(BASE_ROM_SHA256))?
    } else {
        let (bytes, hash) = tool_execution::bounded::read_with_digest(checkout.join("rom/sm.sfc"))?;
        ensure!(hash == BASE_ROM_SHA256, "PLM fixture ROM identity differs");
        bytes
    };
    let mut rom = vec![0; 0x300000];
    let mut opcodes = vec![
        0x86b4, 0x86bc, 0x86c1, 0x86ca, 0x8724, 0x873f, 0x874e, 0x87e5, 0x880e, 0x882d, 0x883e, 0x8848, 0x8865, 0x887c,
        0x8899, 0x8a24, 0x8a2e, 0x8a3a, 0x8a72, 0x8a91, 0x8acd, 0x8af1, 0x8b55, 0x8b93, 0x8bd1, 0x8bdd, 0x8d41,
    ];
    opcodes.extend((0x8c07..0x8c86).step_by(9));
    let mut cases = Vec::new();
    for (oi, op) in opcodes.into_iter().enumerate() {
        let pc = 0x9000 + oi * 32;
        for seed in 0..8 {
            let k = slot(seed);
            let argument = [0, 1, 7, 8, 63, 0x8001, 0xffff, 3][seed];
            let operand = match op {
                0x874e => vec![9],
                0x87e5 => [words(&[4, 0x2000]), vec![0x7e], le(0x3000)].concat(),
                0x880e => [vec![2], le(0x9800)].concat(),
                0x882d | 0x883e => words(&[7, 0x9800]),
                0x8a91 | 0x8acd => [vec![4], le(0x9800)].concat(),
                0x8af1 | 0x8bd1 | 0x8bdd | 0x8c07..=0x8c85 => vec![6],
                0x8d41 => [vec![3, 2], le(0x9800)].concat(),
                _ => le(0x9800),
            };
            write(&mut rom, 0x84, pc, &[le(op), operand].concat());
            let mut writes = vec![
                word(0x1c37 + k, 0xb703),
                word(0x1c87 + k, 0x200),
                word(0x1cd7 + k, 0x853d),
                word(0x1d27 + k, pc),
                word(0x1d77 + k, [1, 2, 0, 0x1201][seed % 4]),
                word(0x1dc7 + k, argument),
                word(0xdebc + k, 0x9800),
                word(0xdf0c + k, [0, 2, 3, 0xff, 0xffff, 4, 5, 0x100][seed]),
                word(0x7a5, 16),
                word(0xaf6, 0x20),
                word(0xafa, 0x100),
                word(0x5e7, 0x55aa),
                word(0x330, seed * 7),
                word(0x998, 8),
                word(0x79f, seed % 8),
                Write { offset: 0xd828 + seed % 8, data: vec![if seed % 2 == 1 { 2 } else { 0 }] },
                Write { offset: 0xd820, data: vec![if seed % 2 == 1 { 0x80 } else { 0 }] },
                Write { offset: 0xd830, data: vec![if seed % 2 == 1 { 0xff } else { 0 }] },
                Write { offset: 0xd870, data: vec![if seed % 2 == 1 { 0xff } else { 0 }] },
                Write { offset: 0xd8b0, data: vec![if seed % 2 == 1 { 0xff } else { 0 }] },
                word(0x10202, 0x8123),
            ];
            if matches!(op, 0x8b55 | 0x8b93) {
                writes.push(word(0x1dc7 + k, 0x8800));
                write(&mut rom, 0x8f, 0x8800, &[0, 1, 3, 2, 0x80]);
            }
            cases.push(fixture(format!("op-{op:04x}-{seed}"), k, 0x840000 | pc, writes));
        }
    }
    for seed in 0..16 {
        let k = slot(seed);
        let pc = 0x9700 + seed * 8;
        write(&mut rom, 0x84, pc, &words(&[0x8b17, 0x86bc]));
        let writes = vec![
            word(0x1c27, k),
            word(0x1c87 + k, 66),
            word(0x1e17 + k, 1 | ((seed % 4) << 10)),
            word(0x7a5, [0, 32, 256, 288][seed % 4]),
            word(0x330, 0),
            word(0x1c25, 0),
        ];
        let mut case = fixture(format!("draw-block-{seed}"), k, 0x840000 | pc, writes);
        case.expected_return = 0.into();
        cases.push(case);
    }
    for (start, size) in [(0xa7ab, 32), (0xce41, 32), (0xe42c, 60), (0xe47e, 40)]
        .into_iter()
        .chain([0xb4b1, 0xa993, 0xb743, 0xb751, 0xcefc, 0xd904, 0xe517].into_iter().map(|start| (start, 14)))
    {
        let offset = 0x30000 + (start & 0x7fff);
        write(&mut rom, 0x86, start, original.get(offset..offset + size).context("ROM table missing")?);
    }
    let mut effect_index = 0;
    for op in [0xbbe1, 0xd357, 0xd52c, 0xd77a, 0xd790, 0xd79f, 0xd7b6, 0xd30b] {
        for seed in 0..8 {
            let k = slot(seed);
            let pc = 0x8000 + effect_index * 16;
            effect_index += 1;
            let operand =
                if op == 0xd30b { words(&[0, 2, 4, 0]) } else { le(if op == 0xbbe1 { 0xb4b1 } else { seed % 3 * 2 }) };
            write(&mut rom, 0x84, pc, &[le(op), operand].concat());
            let mut writes = vec![
                Write { offset: 0x1997, data: vec![0; 36] },
                word(0x1c27, k),
                word(0x1c87 + k, 0x208),
                word(0x7a5, 32),
                word(0x1dc7 + k, 0x1234),
                word(0x5e5, seed * 193),
            ];
            if seed % 2 == 1 {
                writes.push(Write { offset: 0x1997, data: [le(0xb4b1).repeat(17), le(0)].concat() });
            }
            cases.push(fixture(format!("spawn-op-{op:04x}-{seed}"), k, 0x840000 | pc, writes));
        }
    }
    let mut frames = Vec::new();
    for seed in 0..12 {
        let pc = 0xa000 + seed * 64;
        let k = slot(seed);
        write(
            &mut rom,
            0x84,
            pc,
            &[
                le(0x874e),
                vec![3],
                words(&[0x873f, pc + 3, 0x8a2e, pc + 13, if seed % 2 == 0 { 0x86bc } else { 0x86b4 }, 0x8a3a]),
            ]
            .concat(),
        );
        frames.push(returning(
            format!("frame-{seed}"),
            None,
            vec![
                word(0x1c23, 0x8000),
                word(0x1c37 + k, 0xb703),
                word(0x1cd7 + k, 0x853d),
                word(0x1d27 + k, pc),
                word(0xde1c + k, [1, 2, 0][seed % 3]),
            ],
        ));
    }
    let mut preinstructions = vec![
        0xbd0f, 0xbd26, 0xbd50, 0xbd88, 0xbdb2, 0xbe1c, 0xbdd4, 0xbde3, 0xbdf2, 0xbe01, 0xbe1f, 0xbe30, 0xb7dd, 0x8469,
        0x84e6, 0x853d, 0x86d0, 0x8aa6, 0x8ae0, 0xbafa, 0xbb6a, 0xbba3, 0xd356,
    ];
    preinstructions.extend(0xdad5..0xdade);
    preinstructions.push(0xdfa8);
    preinstructions.extend([
        0xac89, 0xb7ef, 0xb82a, 0xb846, 0xb8b0, 0xb948, 0xbb52, 0xbb6b, 0xbba4, 0xd15c, 0xd1e6, 0xd33b, 0xd4bf, 0xd753,
        0xdade, 0xdaee, 0xdafe, 0xdb0e, 0xdb64, 0xdf89, 0xdfe6,
    ]);
    for pre in preinstructions {
        for seed in 0..12 {
            let k = slot(seed);
            let pc = 0xa800;
            let link = 0xa810;
            write(&mut rom, 0x84, pc, &[words(&[0x86bc, 0x874e]), vec![5], le(0x86bc)].concat());
            write(&mut rom, 0x84, link, &[le(0x874e), vec![7], le(0x86bc)].concat());
            let mut writes = vec![
                word(0x1c23, 0x8000),
                word(0x1c37 + k, 0xb703),
                word(0x1cd7 + k, pre),
                word(0x1d27 + k, pc),
                word(0xdebc + k, link),
                word(0xde1c + k, [0, 1, 2][seed % 3]),
                word(0x1d77 + k, [0, 1, 0x100, 0x200, 0x300, 0x400][seed % 6]),
                word(0x79f, seed % 8),
                Write { offset: 0xd828 + seed % 8, data: vec![if seed % 2 == 1 { 7 } else { 0 }] },
                word(0xe50, seed),
                word(0xe52, 6),
                word(0x1e6d, if seed % 2 == 1 { 0x8000 } else { 0 }),
                Write { offset: 0xd821, data: vec![if seed % 2 == 1 { 0x80 } else { 0 }] },
                word(0x5f5, 1),
                word(0x7a5, 32),
                word(0x1c87 + k, 2 * (32 * 8 + 4)),
                word(0x1dc7 + k, seed % 8),
                word(0x9a4, [0, 0x1000, 0x2000, 0x3000][seed % 4]),
                word(0xaf6, [0, 240, 241, 0x348, 0x349, 0xae0][seed % 6]),
                word(0xafa, if seed % 2 == 1 { 1345 } else { 128 }),
                word(0x197a, if seed % 2 == 1 { 0x8000 } else { 0x200 }),
                word(0x1978, 0x400),
                word(0x592, seed % 2),
                word(0x8f, if seed % 2 == 1 { 0x8000 } else { 0 }),
                Write { offset: 0xd8b0, data: vec![if seed % 2 == 1 { 0xff } else { 0 }] },
                word(0x10202 + 2 * (32 * 8 + 4), if seed % 2 == 1 { 255 } else { 254 }),
            ];
            if pre == 0xb846 {
                writes.push(word(0x1d77 + k, [0, 6, 12, 18][seed % 4]));
            }
            frames.push(returning(format!("frame-pre-{pre:04x}-{seed}"), None, writes));
        }
    }
    let mut draw = Vec::new();
    for seed in 0..12 {
        let pc = 0xb000 + seed * 32;
        let k = slot(seed);
        let data = words(&[3, 0x1111, 0x2222, 0x3333, 0x01ff, 0x8002, 0x4444, 0x5555, 0]);
        write(&mut rom, 0x84, pc, &data);
        let mut writes = vec![word(0x1c87 + k, 0x400), word(0xde6c + k, pc), word(0x7a5, 16)];
        if seed % 2 == 1 {
            writes.push(Write { offset: 0x1000, data });
            writes[1] = word(0xde6c + k, 0x1000);
        }
        draw.push(returning(format!("draw-{seed}"), Some(k), writes));
    }
    let mut render = Vec::new();
    for vertical in [false, true] {
        for count in [1, 2, 4, 16] {
            for seed in 0..32 {
                let mut data = le(count | if vertical { 0x8000 } else { 0 });
                for i in 0..count {
                    data.extend(le(((i + seed) % 8) | ((seed % 4) << 10)));
                }
                data.extend(words(&[0x0101, 1, 3 | ((seed % 4) << 10), 0]));
                let pc = 0xb800 + render.len() * 64;
                let k = slot(seed);
                write(&mut rom, 0x84, pc, &data);
                let mut writes = vec![
                    word(0xde6c + k, pc),
                    word(0x1c29, [0, 1, 15, 16, 17, 31, 32, 48][seed % 8]),
                    word(0x1c2b, [0, 1, 14, 15, 16, 17, 20, 32][(seed / 4) % 8]),
                    word(0x911, 16 * (seed % 3)),
                    word(0x915, 16 * (seed % 5)),
                    word(0x91d, if seed % 2 == 1 { 0x100 } else { 0 }),
                    word(0x1c25, [0, 8, 480, 504][seed % 4]),
                    word(0x330, [0, 14, 224, 240, 476, 480][seed % 6]),
                    Write { offset: 0xa000, data: tiles(0x1000) },
                ];
                if seed % 2 == 1 {
                    writes.extend([Write { offset: 0x1000, data }, word(0xde6c + k, 0x1000)]);
                }
                render.push(returning(format!("render-{}-{count}-{seed}", usize::from(vertical)), Some(k), writes));
            }
        }
    }
    let mut drawn = Vec::new();
    for seed in 0..32 {
        let k = slot(seed);
        let pc = 0xa900 + seed * 16;
        let data_pc = 0xab00 + seed * 32;
        write(&mut rom, 0x84, pc, &words(&[3, data_pc, 0x86bc]));
        write(
            &mut rom,
            0x84,
            data_pc,
            &words(&[2 | if seed % 2 == 1 { 0x8000 } else { 0 }, 1 | ((seed % 4) << 10), 2, 0]),
        );
        drawn.push(returning(
            format!("drawn-frame-{seed}"),
            None,
            vec![
                word(0x1c23, 0x8000),
                word(0x1c37 + k, 0xb703),
                word(0x1cd7 + k, 0x8469),
                word(0x1d27 + k, pc),
                word(0xde1c + k, 1),
                word(0x1c87 + k, 2 * (32 + seed)),
                word(0x7a5, 32),
                Write { offset: 0xa000, data: tiles(0x2000) },
                word(0x911, seed % 3 * 16),
                word(0x915, 0),
                word(0x91d, if seed % 2 == 1 { 0x100 } else { 0 }),
            ],
        ));
    }
    for seed in 0..8 {
        let mut writes = vec![word(0x1c23, 0x8000), word(0x7a5, 32), Write { offset: 0xa000, data: tiles(0x2000) }];
        for k in [0, 2, 38, 78] {
            let pc = 0xae00 + k * 8;
            let data_pc = 0xb100 + k * 8;
            write(&mut rom, 0x84, pc, &words(&[2, data_pc]));
            write(&mut rom, 0x84, data_pc, &words(&[1, 1 + k % 4, 0]));
            writes.extend([
                word(0x1c37 + k, 0xb703),
                word(0x1cd7 + k, 0x8469),
                word(0x1d27 + k, pc),
                word(0xde1c + k, 1),
                word(0x1c87 + k, 2 * (32 + 1 + seed)),
            ]);
        }
        drawn.push(returning(format!("drawn-multi-frame-{seed}"), None, writes));
    }
    for seed in 0..12 {
        let k = slot(seed);
        let pc = 0x8400 + seed * 8;
        write(&mut rom, 0x84, pc, &words(&[0x86bc, 0x86bc]));
        drawn.push(returning(
            format!("escape-spawn-frame-{seed}"),
            None,
            vec![
                word(0x1c23, 0x8000),
                word(0x1c37 + k, 0xb703),
                word(0x1cd7 + k, 0xb927),
                word(0x1d27 + k, pc),
                word(0xde1c + k, 2),
                Write { offset: 0x1997, data: vec![0; 36] },
                word(0xaf6, if seed % 2 == 1 { 241 } else { 240 }),
                word(0xafa, if seed % 3 != 0 { 2081 } else { 2080 }),
            ],
        ));
    }
    let mut coordinates = Vec::new();
    for width in [0, 1, 16, 31, 255, 256, 272, 65535] {
        for index in [0, 2, 30, 32, 1022, 65534] {
            coordinates.push(returning(
                format!("coords-{width}-{index}"),
                Some(78),
                vec![word(0x7a5, width), word(0x1c87 + 78, index)],
            ));
        }
    }
    let manifest = Manifest {
        family: ("plm-instructions").to_owned(),
        bank: ("sm_84.c").to_owned(),
        c_support: (vec!["sm_80.c", "sm_rtl.c"]).into_iter().map(str::to_owned).collect(),
        rust_module: ("crates/port/src/plm_instructions.rs").to_owned(),
        rust_imports: (vec!["audio"]).into_iter().map(str::to_owned).collect(),
        rom: Some(Asset { file: path.into(), sha256: translation_execution::model::sha256(&rom) }),
        routines: vec![
            Routine {
                symbol: ("PlmInstruction").to_owned(),
                arg_types: Some((vec!["uint16", "rom_ptr"]).into_iter().map(str::to_owned).collect()),
                returns: ("rom_ptr").to_owned(),
                cases,
                ..Default::default()
            },
            Routine {
                symbol: ("PlmControlFrame").to_owned(),
                args: 0,
                returns: ("uint32").to_owned(),
                cases: frames,
                ..Default::default()
            },
            Routine {
                symbol: ("PlmFrame").to_owned(),
                args: 0,
                returns: ("uint32").to_owned(),
                cases: drawn,
                ..Default::default()
            },
            Routine {
                symbol: ("CalculatePlmBlockCoords").to_owned(),
                args: 1,
                returns: ("void").to_owned(),
                cases: coordinates,
                ..Default::default()
            },
            Routine {
                symbol: ("DrawPlm").to_owned(),
                args: 1,
                returns: ("void").to_owned(),
                cases: render,
                ..Default::default()
            },
            Routine {
                symbol: ("ProcessPlmDrawInstruction").to_owned(),
                args: 1,
                returns: ("void").to_owned(),
                cases: draw,
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    Ok((manifest, rom))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    #[test]
    fn pinned_rom_and_all_fixture_domains_match_the_migration_reference() {
        let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
        if !checkout.join("rom/sm.sfc").exists() {
            return;
        }
        let temporary = std::env::temp_dir()
            .join(format!("sm-plm-fixtures-test-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        let generated = generate(checkout, &temporary, None).unwrap();
        assert_eq!(generated.rom_sha256, "66891cd1701badac52f5f1ac65a5e71c9b6c33c5c0497fa8b166111f8e69c457");
        let manifest = crate::fixture_format::read(&generated.manifest).unwrap();
        assert_eq!(manifest.routines.iter().map(|r| r.cases.len()).sum::<usize>(), 1444);
        assert_eq!(manifest.routines.len(), 6);
        for routine in &manifest.routines {
            for case in &routine.cases {
                assert!(case.writes.iter().all(|w| w.offset + w.data.len() <= 0x20000));
            }
        }
        fs::remove_dir_all(temporary).unwrap();
    }
}
