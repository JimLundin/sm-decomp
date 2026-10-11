//! Projectile allocation, motion and scheduling domains, independent of candidates.
use super::*;
use anyhow::Context;
const SLOTS: [usize; 4] = [0, 2, 16, 34];
fn pool() -> Vec<(usize, i64)> {
    (0..36).step_by(2).map(|slot| (0x1997 + slot, 1)).collect()
}
fn scheduling(manifest: &mut Manifest, spawn: Vec<Case>, pre: Vec<Case>, frames: Vec<Case>) {
    manifest.routines = vec![
        control(),
        routine("SpawnEprojWithRoomGfx", 2, "void", spawn),
        routine("EprojPreinstruction", 2, "void", pre),
        routine("EprojRunAll", 0, "void", frames),
    ];
}
fn state_pair(
    pre: &mut Vec<Case>,
    frames: &mut Vec<Case>,
    label: &str,
    k: usize,
    address: i64,
    values: Vec<(usize, i64)>,
) {
    pre.push(Case {
        name: (format!("pre-{label}")).to_owned(),
        a: (k) as i64,
        b: address,
        writes: words(values.clone()),
        ..Default::default()
    });
    frames.push(Case {
        name: (format!("frame-{label}")).to_owned(),
        writes: words(values.into_iter().chain([(0x198d, 0x8000)])),
        ..Default::default()
    });
}
fn particles(checkout: &Path, name: &str) -> Result<Manifest> {
    let mut m = load(checkout, "tourian-statue-effects")?;
    let falling = name == "enemy-falling-particles";
    let mut spawn = Vec::new();
    let mut pre = Vec::new();
    let mut frames = Vec::new();
    for definition in if falling { vec![0x8aaf] } else { vec![0x8bc2, 0x8bd0, 0x8bde, 0x8bec] } {
        for i in 0..48 {
            let k = SLOTS[i % 4];
            let enemy = (i % 32) * 64;
            let mut v = pool();
            v.extend([
                (0x1997 + k, 0),
                (0xe54, enemy as i64),
                (0xf7a + enemy, (i * 1777) as i64),
                (0xf7e + enemy, (i * 733) as i64),
            ]);
            if falling {
                v.extend([(0xfae + enemy, (i * 257) as i64), (0x5e5, (i * 997) as i64)]);
            }
            spawn.push(Case {
                name: (format!("spawn-{definition:x}-{i}")).to_owned(),
                a: (definition) as i64,
                b: (i) as i64,
                writes: words(v),
                ..Default::default()
            });
        }
    }
    for address in if falling { vec![0x8a7d] } else { vec![0x8b5d, 0xe4fe] } {
        for i in 0..128 {
            let k = SLOTS[i % 4];
            let camera = [0, 0xff80, 0x7fff][i % 3];
            let mut v = vec![
                (0x1997 + k, 1),
                (0x1a03 + k, address),
                (0x1b47 + k, 0x9754),
                (0x1b8f + k, 2),
                (0x911, camera),
                (0x915, camera),
                (0x1a4b + k, camera + [-1, 0, 1, 128, 255, 256, 257, 0x8000][i % 8]),
                (0x1a93 + k, camera + [0, 255, 256, 257][i % 4]),
                (0x1a27 + k, 0xffa5),
                (0x1a6f + k, 0xff5a),
                (0x1ab7 + k, [0, 1, 0x100, 0xffff, 0xff00, 0x7fff, 0x8000, 320][(i / 4) % 8]),
                (0x1adb + k, [0, 1, 0xff00, 0xfbff][(i / 32) % 4]),
            ];
            if falling {
                v.push((0x19df + k, [0, 255, 256, 0xffff][i % 4]));
            }
            state_pair(&mut pre, &mut frames, &format!("{address:x}-{i}"), k, address, v);
        }
    }
    scheduling(&mut m, spawn, pre, frames);
    Ok(m)
}
fn horizontal(checkout: &Path, name: &str) -> Result<Manifest> {
    let mut m = load(checkout, "tourian-statue-effects")?;
    let lava = name == "lavaman-thrown-lava";
    let mut spawn = Vec::new();
    let mut pre = Vec::new();
    let mut frames = Vec::new();
    for definition in if lava { vec![0xe0e0] } else { vec![0xdbf2, 0xdfbc] } {
        for i in 0..128 {
            let k = SLOTS[i % 4];
            let enemy = (i % 32) * 64;
            let mut v = pool();
            v.extend([(0x1997 + k, 0), (0xe54, enemy as i64)]);
            v.extend([2, 4, 6, 8].map(|d| (0xf78 + enemy + d, (i * 997 + d * 73) as i64)));
            v.extend([(0xfb4 + enemy, [0, 1, 15, 16][i % 4]), (0xfb6 + enemy, (0xff00 | (i % 8)) as i64)]);
            spawn.push(Case {
                name: (format!("spawn-{definition:x}-{i}")).to_owned(),
                a: (definition) as i64,
                b: ([0, 1, 0xffff][i % 3]) as i64,
                writes: words(v),
                ..Default::default()
            });
        }
    }
    for (address, states) in if lava {
        vec![(0xe049, [0xe050, 0xe07a])]
    } else {
        vec![(0xdb5b, [0xdb62, 0xdb8c]), (0xdf39, [0xdf40, 0xdf6a])]
    } {
        for i in 0..128 {
            let k = SLOTS[i % 4];
            let camera = [0, 0xff80, 0x7fff][i % 3];
            let v = vec![
                (0x1997 + k, 1),
                (0x1a03 + k, address),
                (0x1b47 + k, 0x9754),
                (0x1b8f + k, 2),
                (0x1aff + k, states[(i / 8) % 2]),
                (0x911, camera),
                (0x915, camera),
                (0x1a4b + k, camera + [-1, 0, 1, 128, 255, 256, 257, 0x8000][i % 8]),
                (0x1a93 + k, camera + [0, 256, 257][i % 3]),
                (0x1a27 + k, 0xffa5),
                (0x1a6f + k, 0x1234),
                (0x1ab7 + k, [0, 1, 255, 256, 0xffff, 0xff00, 0x7fff, 0x8000][(i / 16) % 8]),
                (0x1adb + k, 0xff00),
            ];
            state_pair(&mut pre, &mut frames, &format!("{address:x}-{i}"), k, address, v);
        }
    }
    scheduling(&mut m, spawn, pre, frames);
    Ok(m)
}
fn spores(checkout: &Path) -> Result<Manifest> {
    let mut m = load(checkout, "tourian-statue-effects")?;
    let mut spawn = Vec::new();
    let mut pre = Vec::new();
    let mut frames = Vec::new();
    for (definition, count) in [(0xde6c, 5), (0xde88, 4)] {
        for i in 0..80 {
            let k = SLOTS[i % 4];
            let mut v = pool();
            v.extend([(0x1997 + k, 0), (0xf7a, (i * 997) as i64), (0xf7e, (i * 733) as i64)]);
            spawn.push(Case {
                name: (format!("spawn-{definition:x}-{i}")).to_owned(),
                a: (definition) as i64,
                b: (i % count) as i64,
                writes: words(v),
                ..Default::default()
            });
        }
    }
    for address in [0xdcee, 0xdd46] {
        for i in 0..256 {
            let k = SLOTS[i % 4];
            let v = vec![
                (0x1997 + k, 0xde7a),
                (0x1a03 + k, address),
                (0x1b47 + k, 0x9754),
                (0x1b8f + k, 2),
                (0x1aff + k, (0xa500 | i) as i64),
                (0x1b23 + k, [0, 1, 2, 0x80, 0xffff][i % 5]),
                (0x1a4b + k, (i * 997) as i64),
                (0x1a93 + k, [767, 768, 769, 0xffff, 0x8000, 520][i % 6]),
                (0x9000, (i % 2) as i64),
                (0x5e5, (i * 257) as i64),
            ];
            state_pair(&mut pre, &mut frames, &format!("{address:x}-{i}"), k, address, v);
        }
    }
    scheduling(&mut m, spawn, pre, frames);
    Ok(m)
}
fn maw(checkout: &Path) -> Result<Manifest> {
    let mut m = load(checkout, "botwoon-body-allocation")?;
    let mut cases = Vec::new();
    for i in 0..384 {
        let enemy = (i % 32) * 64;
        let index = [0, 1, 2, 3, 0x7fff, 0x8000, 0xfffe, 0xffff][(i / 5) % 8];
        let target = (enemy + 2 * index) & 0xffff;
        let mut v = pool();
        if let Some(free) = [Some(0), Some(2), Some(16), Some(34), None][i % 5] {
            v.push((0x1997 + free, 0));
        }
        v.extend([
            (0xe54, enemy as i64),
            (0xf7a + enemy, (i * 997) as i64),
            (0xf7e + enemy, (i * 733) as i64),
            (0xfb6 + enemy, [0, 1, 0x8000, 0xffff][(i / 32) % 4]),
            (0x8800 + target, 0xdead),
            (0x8808 + enemy, index as i64),
            (0x1c27, 0xffff),
        ]);
        cases.push(Case {
            name: (format!("body-{i}")).to_owned(),
            a: 0xec95_i64,
            b: (i) as i64,
            writes: words(v),
            ..Default::default()
        });
    }
    m.routines[1].cases = cases;
    Ok(m)
}
fn rom_pointer(rom: &[u8], bytes: &[u8], start: usize, end: usize) -> Result<i64> {
    let offset = rom
        .get(start..end)
        .context("ROM search outside image")?
        .windows(bytes.len())
        .position(|word| word == bytes)
        .context("required ROM word absent")?
        + start;
    Ok((offset - 0x30000 + 0x8000) as i64)
}
fn lavaquake(checkout: &Path) -> Result<Manifest> {
    let mut m = load(checkout, "tourian-statue-effects")?;
    let mut spawn = Vec::new();
    let mut pre = Vec::new();
    let mut frames = Vec::new();
    for state in [0xbc16, 0xbc8f] {
        for speed in [0, 1, 2, 3, 4, 16, 32, 62, 63, 64, 65, 90, 0xffff] {
            for (i, edge) in [-1, 0, 1, 128, 255, 256, 257].into_iter().enumerate() {
                let k = SLOTS[i % 4];
                let camera = [0, 0xff80, 0x7fff][i % 3];
                let v = vec![
                    (0x1997 + k, 0xbbc7),
                    (0x1a03 + k, 0xbc0f),
                    (0x1b47 + k, 0x9754),
                    (0x1b8f + k, 2),
                    (0x1aff + k, state),
                    (0x1adb + k, speed),
                    (0x1ab7 + k, [0, 1, 0xffff, 0x100, 0xff00, 0x8000, 0x7fff][i]),
                    (0x911, camera),
                    (0x915, camera),
                    (0x1a4b + k, camera + edge),
                    (0x1a93 + k, camera + edge),
                    (0x1a27 + k, 0xff5a),
                    (0x1a6f + k, 0xffa5),
                    (0x1b23 + k, 0xdead),
                ];
                pre.push(Case {
                    name: (format!("arc-{state:x}-{speed}-{i}")).to_owned(),
                    a: (k) as i64,
                    b: 0xbc0f_i64,
                    writes: words(v.clone()),
                    ..Default::default()
                });
                frames.push(Case {
                    name: (format!("frame-{state:x}-{speed}-{i}")).to_owned(),
                    writes: words(v.into_iter().chain([(0x198d, 0x8000)])),
                    ..Default::default()
                });
            }
        }
    }
    let rom = super::rom(checkout)?;
    let definition = rom_pointer(&rom, &[0xdb, 0xbb, 0x0f, 0xbc], 0x30000, 0x38000)?;
    for i in 0..72 {
        let k = SLOTS[i % 4];
        let enemy = (i % 32) * 64;
        let mut v = pool();
        v.extend([(0x1997 + k, 0), (0xe54, enemy as i64), (0x1995, ((i * 1377) & 65535) as i64)]);
        v.extend([2, 4, 6, 8].map(|d| (0xf78 + enemy + d, ((i * 1777 + d * 93) & 65535) as i64)));
        spawn.push(Case {
            name: (format!("spawn-{i}")).to_owned(),
            a: (definition) as i64,
            b: ([0, 1, 2, 32, 64, 0xffff][i % 6]) as i64,
            writes: words(v),
            ..Default::default()
        });
    }
    m.routines = vec![
        control(),
        routine("EprojPreinstruction", 2, "void", pre),
        routine("EprojRunAll", 0, "void", frames),
        routine("SpawnEprojWithRoomGfx", 2, "void", spawn),
    ];
    Ok(m)
}
fn gates(checkout: &Path) -> Result<Manifest> {
    let mut m = load(checkout, "tourian-statue-effects")?;
    let rom = super::rom(checkout)?;
    let mut spawn = Vec::new();
    let mut pre = Vec::new();
    let mut frames = Vec::new();
    for init in [0xe5d0u16, 0xe5d5, 0xe5da, 0xe6ad] {
        let p = rom_pointer(&rom, &init.to_le_bytes(), 0x36500, 0x36700)?;
        for i in 0..64 {
            let k = SLOTS[i % 4];
            let plm = (i % 40) * 2;
            let mut v = pool();
            v.extend([
                (0x1997 + k, 0),
                (0x1c27, plm as i64),
                (0x7a5, [1, 16, 32, 64][i % 4]),
                (0x1c87 + plm, (i * 14) as i64),
                (0x1c29, 0xdead),
                (0x1c2b, 0xbeef),
            ]);
            spawn.push(Case {
                name: (format!("spawn-{init:x}-{i}")).to_owned(),
                a: (p) as i64,
                b: 0_i64,
                writes: words(v),
                ..Default::default()
            });
        }
    }
    for i in 0..128 {
        let k = SLOTS[i % 4];
        let v = vec![
            (0x1997 + k, 1),
            (0x1a03 + k, 0xe605),
            (0x1b47 + k, 0x9752),
            (0x1b8f + k, 2),
            (0x19df + k, [0, 0xfff, 0x1000, 0xffff][i % 4]),
            (0x1adb + k, [0, 1, 255, 256, 0xffff, 0xff00, 0x8000, 0x7fff][(i / 4) % 8]),
            (0x1a93 + k, (i * 997) as i64),
            (0x1a6f + k, 0xffa5),
        ];
        pre.push(Case {
            name: (format!("move-{i}")).to_owned(),
            a: (k) as i64,
            b: 0xe605_i64,
            writes: words(v.clone()),
            ..Default::default()
        });
        frames.push(Case {
            name: (format!("frame-{i}")).to_owned(),
            writes: words(v.into_iter().chain([(0x198d, 0x8000)])),
            ..Default::default()
        });
    }
    scheduling(&mut m, spawn, pre, frames);
    Ok(m)
}
fn shaktool(checkout: &Path) -> Result<Manifest> {
    let mut m = load(checkout, "tourian-statue-effects")?;
    let mut spawn = Vec::new();
    let mut pre = Vec::new();
    let mut frames = Vec::new();
    for definition in [0xbe25, 0xbe33, 0xbe41] {
        for angle in 0..256 {
            let k = SLOTS[angle % 4];
            let enemy = (angle % 32) * 64;
            let mut v = pool();
            v.extend([
                (0x1997 + k, 0),
                (0xe54, enemy as i64),
                (0xfae + enemy, (0xa500 | angle) as i64),
                (0xf7a + enemy, ((angle * 997) & 65535) as i64),
                (0xf7e + enemy, ((angle * 733) & 65535) as i64),
            ]);
            spawn.push(Case {
                name: (format!("spawn-{definition:x}-{angle}")).to_owned(),
                a: (definition) as i64,
                b: ([0, 3, 16, 35][angle % 4]) as i64,
                writes: words(v),
                ..Default::default()
            });
        }
    }
    for address in [0xbe03, 0xbe12] {
        for i in 0..96 {
            let k = SLOTS[i % 4];
            let linked = [0usize, 3, 16, 35][(i / 4) % 4];
            let mut v = vec![
                (0x7a5, 16),
                (0x1991, k as i64),
                (0x1997 + (linked & !1), (i % 2) as i64),
                (0x1997 + k, 0xbe25),
                (0x1aff + k, linked as i64),
                (0x1a03 + k, address),
                (0x1b47 + k, 0x9754),
                (0x1b8f + k, 2),
                (0x1bb3 + k, 0x404),
                (0x1a4b + k, 29),
                (0x1a93 + k, 29),
                (0x1a27 + k, 0xffa5),
                (0x1a6f + k, 0xff5a),
                (0x1ab7 + k, [0, 0x100, 0xff00, 0x400, 0xfc00, 0x7f, 0xff81, 0x200][i % 8]),
                (0x1adb + k, [0x200, 0xfe00, 0x100, 0xff00][(i / 8) % 4]),
            ];
            if (i / 16) % 3 != 0 {
                v.extend((0..16).map(|row| (0x10002 + 2 * (row * 16 + 2), 0x8000)));
            }
            if (i / 16) % 3 == 2 {
                v.extend((0..16).map(|col| (0x10002 + 2 * (2 * 16 + col), 0x8000)));
            }
            pre.push(Case {
                name: (format!("pre-{address:x}-{i}")).to_owned(),
                a: (k) as i64,
                b: address,
                writes: words(v.clone()),
                ..Default::default()
            });
            let other = linked & !1;
            if other != k {
                v.extend([(0x1a03 + other, 0x8170), (0x1b47 + other, 0x9754), (0x1b8f + other, 2)]);
            }
            frames.push(Case {
                name: (format!("frame-{address:x}-{i}")).to_owned(),
                writes: words(v.into_iter().chain([(0x198d, 0x8000)])),
                ..Default::default()
            });
        }
    }
    for script in [0xbd68, 0xbd78, 0xbd8c] {
        for timer in [1, 2] {
            frames.push(Case {
                name: (format!("real-script-{script:x}-{timer}")).to_owned(),
                writes: words([
                    (0x7a5, 16),
                    (0x198d, 0x8000),
                    (0x1997, 0xbe33),
                    (0x1a03, 0x84fb),
                    (0x1b47, script),
                    (0x1b8f, timer),
                    (0x1a4b, 64),
                    (0x1a93, 64),
                    (0x1bb3, 0x404),
                ]),
                ..Default::default()
            });
        }
    }
    m.routines = vec![
        control(),
        routine("EprojPreinstruction", 2, "void", pre),
        routine("EprojRunAll", 0, "void", frames),
        routine("SpawnEprojWithRoomGfx", 2, "void", spawn),
    ];
    Ok(m)
}
fn draygon(checkout: &Path, name: &str) -> Result<Manifest> {
    let mut m = super::motion(checkout, "projectile-paired-motion")?;
    let mut pre = Vec::new();
    let mut frames = Vec::new();
    let mut bombs = Vec::new();
    for i in 0..192 {
        let k = SLOTS[i % 4];
        let address = [0x8dff, 0x8e0f][i % 2];
        let v = vec![
            (0x1997 + k, 1),
            (0x1a03 + k, address),
            (0x1b47 + k, 0x9754),
            (0x1b8f + k, 2),
            (0x97dc + k, i as i64),
            (0x1a4b + k, [100, 511, 512, 0xffff][i % 4]),
            (0x1a93 + k, 100),
            (0x1ab7 + k, 1),
            (0x1aff + k, 0x8000),
            (0x1adb + k, 0),
            (0x1b23 + k, 0x4000),
            (0xaf6, (100 + i % 20) as i64),
            (0xafa, (100 + i % 24) as i64),
            (0xa66, 3),
            (0xcea, [0, 0x100, 0x200, 0xff00][i % 4]),
            (0xce2, (100 + i % 4) as i64),
            (0xce4, (100 + i % 4) as i64),
        ];
        pre.push(Case {
            name: (format!("pre-{i}")).to_owned(),
            a: (k) as i64,
            b: address,
            writes: words(v.clone()),
            ..Default::default()
        });
        frames.push(Case {
            name: (format!("frame-{i}")).to_owned(),
            writes: words(v.clone().into_iter().chain([(0x198d, 0x8000)])),
            ..Default::default()
        });
        bombs.push(Case {
            name: (format!("bomb-{i}")).to_owned(),
            a: (k) as i64,
            writes: words(v),
            ..Default::default()
        });
    }
    if name == "draygon-attached-gunk" {
        let rom = super::rom(checkout)?;
        let pc = rom_pointer(&rom, &[0x99, 0x8d], 0x30c00, 0x30d00)?;
        let mut ops =
            vec![Case { name: ("sleep-control").to_owned(), a: 0_i64, b: 0x869754_i64, ..Default::default() }];
        for i in 0..128 {
            let k = SLOTS[i % 4];
            let v = vec![
                (0x1997 + k, 1),
                (0x1a03 + k, 0x8dca),
                (0x1b47 + k, 0x9754),
                (0x1b8f + k, 2),
                (0x1aff + k, [0, 1, 2, 256][i % 4]),
                (0x1b23 + k, (i * 997) as i64),
                (0xaf6, (i * 733) as i64),
                (0xafa, (i * 1777) as i64),
                (0xa6e, ((i / 4) % 2) as i64),
                (0xa66, [0, 1, 4, 5, 6, 0x7fff, 0x8000, 0xffff][(i / 8) % 8]),
                (0x18a8, 123),
                (0x18aa, 456),
                (0x1bd7 + k, 0xffff),
                (0xcea, if i % 3 == 0 { 0xff00 } else { 0 }),
                (0xce2, 0),
                (0xce4, 0),
            ];
            pre.push(Case {
                name: (format!("attached-{i}")).to_owned(),
                a: (k) as i64,
                b: 0x8dca_i64,
                writes: words(v.clone()),
                ..Default::default()
            });
            frames.push(Case {
                name: (format!("attached-frame-{i}")).to_owned(),
                writes: words(v.clone().into_iter().chain([(0x198d, 0x8000)])),
                ..Default::default()
            });
            ops.push(Case {
                name: (format!("attach-op-{i}")).to_owned(),
                a: (k) as i64,
                b: (0x860000 + pc) as i64,
                writes: words(v),
                ..Default::default()
            });
        }
        m.routines[0].cases = ops;
    }
    let routines = &mut m.routines;
    routines.truncate(3);
    routines.extend([
        routine("EprojPreinstruction", 2, "void", pre),
        routine("EprojRunAll", 0, "void", frames),
        routine("EprojPowerBombCollision", 1, "void", bombs),
    ]);
    Ok(m)
}
fn tourian(checkout: &Path) -> Result<Manifest> {
    let mut m = load(checkout, "tourian-statue-effects")?;
    let mut pre = Vec::new();
    let mut frames = Vec::new();
    let mut spawn = Vec::new();
    let mut ops = vec![Case { name: ("sleep-control").to_owned(), a: 0_i64, b: 0x869754_i64, ..Default::default() }];
    for address in [0xb977, 0xb982, 0xb9fd, 0xba37, 0xba42] {
        for i in 0..72 {
            let k = SLOTS[i % 4];
            let y = [0, 1, 255, 256, 511, 512, 767, 768, 0xffff][i % 9];
            let velocity = [0, 1, 255, 256, 0xff00, 0xffff, 0x7fff, 0x8000][i / 9];
            let v = vec![
                (0x1997 + k, 0xba78),
                (0x1a03 + k, address),
                (0x1b47 + k, 0x9754),
                (0x1b8f + k, 2),
                (0x1a4b + k, 0xffff - i as i64),
                (0x1a27 + k, 0xffa5),
                (0x1a93 + k, y),
                (0x1a6f + k, 0xff5a),
                (0x1ab7 + k, velocity ^ 0xffff),
                (0x1adb + k, velocity),
                (0x195e, y + (i % 3) as i64 - 1),
                (0x1aff + k, (i * 997) as i64),
                (0x1b23 + k, (i * 733) as i64),
                (0x915, 0xffff - i as i64),
                (0x9e00, (i * 1231) as i64),
                (0x1e6d, 0x1234),
                (0x1e6f, (i % 2) as i64),
            ];
            pre.push(Case {
                name: (format!("pre-{address:04x}-{i}")).to_owned(),
                a: (k) as i64,
                b: address,
                writes: words(v.clone()),
                ..Default::default()
            });
            if i < 18 {
                frames.push(Case {
                    name: (format!("frame-{address:04x}-{i}")).to_owned(),
                    writes: words(v.into_iter().chain([(0x198d, 0x8000)])),
                    ..Default::default()
                });
            }
        }
    }
    for target in [Some(0), Some(16), Some(34), None] {
        for is_frame in [false, true] {
            if is_frame && target == Some(16) {
                continue;
            }
            let mut v = Vec::new();
            for s in (0..36).step_by(2) {
                v.extend([(0x1997 + s, 1), (0x1a03 + s, 0x8170), (0x1b47 + s, 0x9754), (0x1b8f + s, 2)]);
            }
            if let Some(t) = target {
                v.push((0x1997 + t, 0));
            }
            v.extend([(0x1a03 + 16, 0xb982), (0x1a93 + 16, 99), (0x1adb + 16, 0x200), (0x195e, 100), (0x198d, 0x8000)]);
            let mut case = Case {
                name: (format!(
                    "cross-{}-{}",
                    target.map(|t| t.to_string()).unwrap_or("None".into()),
                    if is_frame { "True" } else { "False" }
                ))
                .to_owned(),
                writes: words(v),
                ..Default::default()
            };
            if is_frame {
                frames.push(case);
            } else {
                case.a = 16;
                case.b = 0xb982;
                pre.push(case);
            }
        }
    }
    let rom = super::rom(checkout)?;
    for definition in (0xba5cusize..0xbb1e).step_by(14) {
        let p = 0x30000 + (definition & 0x7fff);
        let init = u16::from_le_bytes(rom[p..p + 2].try_into()?);
        if ![0xb87a, 0xb93e, 0xb951, 0xb964].contains(&init) {
            continue;
        }
        for i in 0..18 {
            let k = i * 2;
            spawn.push(Case {
                name: (format!("spawn-{definition:04x}-{i}")).to_owned(),
                a: (definition) as i64,
                b: (k) as i64,
                writes: words([(0x1a4b + k, (i * 1777) as i64), (0x195e, (i * 4093) as i64)]),
                ..Default::default()
            });
        }
    }
    for opcode in [0xb7eau16, 0xb818] {
        let p = rom_pointer(&rom, &opcode.to_le_bytes(), 0x33700, 0x33900)?;
        for k in SLOTS {
            ops.push(Case {
                name: (format!("op-{opcode:04x}-{k}")).to_owned(),
                a: (k) as i64,
                b: (0x860000 + p) as i64,
                writes: words([(0x1a4b + k, 100), (0x1a93 + k, 90), (0x5e5, 0xface)]),
                ..Default::default()
            });
        }
    }
    let mut op = control();
    op.cases = ops;
    m.routines = vec![
        op,
        routine("EprojPreinstruction", 2, "void", pre),
        routine("EprojRunAll", 0, "void", frames),
        routine("SpawnEprojWithRoomGfx", 2, "void", spawn),
    ];
    Ok(m)
}
fn turrets(checkout: &Path) -> Result<Manifest> {
    let mut m = load(checkout, "tourian-statue-effects")?;
    let mut spawn = Vec::new();
    let mut pre = Vec::new();
    let mut frames = Vec::new();
    for i in 0..144 {
        let k = SLOTS[i % 4];
        let mut v = pool();
        v.extend([(0x1997 + k, 0), (0x5e5, (i * 997) as i64)]);
        spawn.push(Case {
            name: (format!("turret-{i}")).to_owned(),
            a: 0xc17e_i64,
            b: (i % 12) as i64,
            writes: words(v),
            ..Default::default()
        });
    }
    for i in 0..128 {
        let k = SLOTS[i % 4];
        let source = SLOTS[(i / 4) % 4];
        let mut v = pool();
        v.extend([
            (0x1997 + k, 0),
            (0x1a6f + source, (0xff00 | (i % 8)) as i64),
            (0x1a4b + source, (i * 997) as i64),
            (0x1a93 + source, (i * 733) as i64),
        ]);
        spawn.push(Case {
            name: (format!("bullet-{i}")).to_owned(),
            a: 0xc18c_i64,
            b: (source + i % 2) as i64,
            writes: words(v),
            ..Default::default()
        });
    }
    for i in 0..192 {
        let k = SLOTS[i % 4];
        let active = SLOTS[(i / 4) % 4];
        let mut v = vec![
            (0x7a5, 16),
            (0x1991, active as i64),
            (0x1997 + k, 0xc18c),
            (0x1997 + active, 0xc18c),
            (0x1a03 + k, 0xc0e0),
            (0x1b47 + k, 0x9754),
            (0x1b8f + k, 2),
            (0x1a4b + k, 512),
            (0x1a93 + k, 512),
            (0x1a27 + k, 0xffa5),
            (0x1a6f + k, 0xff5a),
            (0x1ab7 + k, [0, 0x100, 0xff00, 0x2c0, 0xfd40, 0x7fff, 0x8000, 0xffff][i % 8]),
            (0x1adb + k, [0, 0x100, 0xff00, 0x2c0][(i / 8) % 4]),
            (0x1bd7 + k, (i * 127) as i64),
        ];
        if i % 3 != 0 {
            for row in 23..42 {
                for col in 23..42 {
                    v.push((0x10002 + 2 * (row * 16 + col), (((i % 15) + 1) << 12) as i64));
                }
            }
        }
        pre.push(Case {
            name: (format!("move-{i}")).to_owned(),
            a: (k) as i64,
            b: 0xc0e0_i64,
            writes: words(v.clone()),
            ..Default::default()
        });
        if active != k {
            v.extend([(0x1a03 + active, 0x8170), (0x1b47 + active, 0x9754), (0x1b8f + active, 2)]);
        }
        frames.push(Case {
            name: (format!("frame-{i}")).to_owned(),
            writes: words(v.into_iter().chain([(0x198d, 0x8000)])),
            ..Default::default()
        });
    }
    let rom = super::rom(checkout)?;
    let mut rotation = Vec::new();
    let mut rx = Vec::new();
    let mut ry = Vec::new();
    for i in 0..96 {
        let p = 0x33ec9 + (i % 12) * 2;
        let ptr = u16::from_le_bytes(rom[p..p + 2].try_into()?);
        let k = SLOTS[i % 4];
        rotation.push(Case {
            name: (format!("rotate-{i}")).to_owned(),
            a: (k) as i64,
            writes: words([
                (0x1a27 + k, ptr as i64),
                (0x1a6f + k, ((if i % 2 != 0 { 0x100 } else { 0xff00 }) | (i % 8)) as i64),
            ]),
            ..Default::default()
        });
    }
    for i in 0..256 {
        let k = SLOTS[i % 4];
        let w = words([(0x5e5, (i * 257) as i64)]);
        rx.push(Case {
            name: (format!("random-x-{i}")).to_owned(),
            a: (k) as i64,
            writes: w.clone(),
            ..Default::default()
        });
        ry.push(Case {
            name: (format!("random-y-{i}")).to_owned(),
            a: (k) as i64,
            writes: w.clone(),
            ..Default::default()
        });
    }
    let mut ops = vec![Case { name: ("sleep-control").to_owned(), a: 0_i64, b: 0x869754_i64, ..Default::default() }];
    for opcode in [0xc173u16, 0xc1b4] {
        let p = rom_pointer(&rom, &opcode.to_le_bytes(), 0x33f00, 0x34200)?;
        for i in 0..8 {
            ops.push(Case {
                name: (format!("op-{opcode:x}-{i}")).to_owned(),
                a: 0_i64,
                b: (0x860000 + p) as i64,
                writes: words([(0x1aff, (i * 2) as i64), (0x19bb, 1024)]),
                ..Default::default()
            });
        }
    }
    let mut op = control();
    op.cases = ops;
    m.routines = vec![
        op,
        routine("SpawnEprojWithRoomGfx", 2, "void", spawn),
        routine("EprojPreinstruction", 2, "void", pre),
        routine("EprojRunAll", 0, "void", frames),
        routine("Eproj_MotherBrainRoomTurretBullets_Func2", 1, "void", rotation),
        routine("Eproj_SetXvelRandom", 1, "void", rx),
        routine("Eproj_SetYvelRandom", 1, "void", ry),
    ];
    Ok(m)
}
fn pickups(checkout: &Path) -> Result<Manifest> {
    let mut m = load(checkout, "enemy-drop-selection")?;
    let mut pre = Vec::new();
    let mut frames = Vec::new();
    for kind in 1..6 {
        for i in 0..256 {
            let k = SLOTS[i % 4];
            let dx = [0, 15, 16, 19, 20, 21, 0x8000, 0xffff][(i / 8) % 8];
            let dy = [0, 15, 16, 21][(i / 32) % 4];
            let value = [0, 98, 99, 100, 0x7fff, 0x8000, 0xfffe, 0xffff][i % 8];
            let v = vec![
                (0x1aff + k, kind * 2),
                (0x1b23 + k, [1, 0, 384, 385, 400, 2, 0x8000, 0xffff][i % 8]),
                (0x1a03 + k, 0xefe0),
                (0x1b47 + k, 0x9754),
                (0x1b8f + k, 2),
                (0x1997 + k, 1),
                (0x1a4b + k, 100),
                (0x1a93 + k, 100),
                (0x1bb3 + k, 0x0404),
                (0xaf6, 100 + dx),
                (0xafa, 100 + dy),
                (0xafe, 16),
                (0xb00, 16),
                (0xd08, 100 + dx),
                (0xd0c, 100 + dy),
                (0xd32, if i & 1 != 0 { 0xc4f0 } else { 0 }),
                (0x9c2, value),
                (0x9c4, 99),
                (0x9d6, value),
                (0x9d4, 100),
                (0x9c6, value),
                (0x9c8, [10, 99, 100, 0x8000][i % 4]),
                (0x9d8, value),
                (0x9ca, value),
                (0x9cc, 10),
                (0x9ce, value),
                (0x9d0, 10),
                (0x9c0, [0, 1, 2, 0xffff][i % 4]),
                (0x998, if i % 16 == 0 { 40 } else { 8 }),
                (0x5f5, i64::from(i % 16 == 1)),
                (0x592, if i % 16 == 2 { 0x8000 } else { 0 }),
            ];
            let mut w = words(v);
            w.extend([
                Write { offset: 0x647_usize, data: vec![i % 16].into_iter().map(|v| v as u8).collect() },
                Write { offset: 0x644_usize, data: vec![(i / 16) % 16].into_iter().map(|v| v as u8).collect() },
                Write { offset: 0x654_usize, data: vec![0xa5].into_iter().map(|v| v as u8).collect() },
            ]);
            pre.push(Case {
                name: (format!("pickup-{kind}-{i}")).to_owned(),
                a: (k) as i64,
                b: 0xefe0_i64,
                writes: w.clone(),
                ..Default::default()
            });
            frames.push(Case {
                name: (format!("frame-{kind}-{i}")).to_owned(),
                writes: w
                    .into_iter()
                    .chain([Write { offset: 0x198d_usize, data: vec![0, 128].into_iter().map(|v| v as u8).collect() }])
                    .collect::<Vec<_>>(),
                ..Default::default()
            });
        }
    }
    let base = pre[0].writes.clone();
    let targeted = |name: String, changes: Vec<(usize, i64)>| -> Case {
        let mut writes = base.clone();
        for (p, v) in changes {
            let value = vec![(v & 255) as u8, ((v >> 8) & 255) as u8];
            if let Some(w) = writes.iter_mut().find(|w| w.offset == p) {
                w.data = value;
            } else {
                writes.push(Write { offset: p, data: value });
            }
        }
        Case { name: (name).to_owned(), a: 0_i64, b: 0xefe0_i64, writes, ..Default::default() }
    };
    for life in [385, 384] {
        for axis in [0, 1] {
            for distance in [15, 16, 0x8000] {
                pre.push(targeted(
                    format!("grapple-{life}-{axis}-{distance}"),
                    vec![
                        (0x1b23, life),
                        (0xd32, 0),
                        (0xaf6, 1000),
                        (0xafa, 1000),
                        (0xd08, 100 + if axis == 0 { distance } else { 0 }),
                        (0xd0c, 100 + if axis == 1 { distance } else { 0 }),
                    ],
                ));
            }
        }
    }
    for axis in [0, 1] {
        for distance in [19, 20, 0x8000] {
            pre.push(targeted(
                format!("collision-{axis}-{distance}"),
                vec![
                    (0x1b23, 400),
                    (0xd32, 0xc4f0),
                    (0xaf6, 100 + if axis == 0 { distance } else { 0 }),
                    (0xafa, 100 + if axis == 1 { distance } else { 0 }),
                ],
            ));
        }
    }
    for kind in 1..6 {
        for maximum in [98, 99, 100] {
            for delta in [-1, 0, 1] {
                for gate in 0..5 {
                    let mut case = targeted(
                        format!("restore-{kind}-{maximum}-{delta}-{gate}"),
                        vec![
                            (0x1aff, kind * 2),
                            (0x1b23, 400),
                            (0xaf6, 100),
                            (0xafa, 100),
                            (0x9c2, maximum + delta),
                            (0x9c4, maximum),
                            (0x9c6, maximum + delta),
                            (0x9c8, maximum),
                            (0x9ca, maximum + delta),
                            (0x9cc, maximum),
                            (0x9ce, maximum + delta),
                            (0x9d0, maximum),
                            (0x998, if gate == 1 { 40 } else { 8 }),
                            (0x5f5, i64::from(gate == 2)),
                            (0x592, if gate == 3 { 0x8000 } else { 0 }),
                        ],
                    );
                    case.writes.extend([
                        Write {
                            offset: 0x647_usize,
                            data: vec![if gate == 4 { 1 } else { 0 }].into_iter().map(|v| v as u8).collect(),
                        },
                        Write { offset: 0x644_usize, data: vec![0].into_iter().map(|v| v as u8).collect() },
                    ]);
                    pre.push(case);
                }
            }
        }
    }
    let mut dependencies = m.rust_dependencies.clone();
    dependencies.push("crates/port/src/audio.rs".into());
    dependencies.sort();
    dependencies.dedup();
    m.rust_dependencies = dependencies;
    m.routines = vec![
        m.routines[0].clone(),
        routine("EprojPreinstruction", 2, "void", pre),
        routine("EprojRunAll", 0, "void", frames),
    ];
    Ok(m)
}
pub(super) fn generate(checkout: &Path, name: &str) -> Result<Manifest> {
    let mut m = match name {
        "skree-particles" | "enemy-falling-particles" => particles(checkout, name)?,
        "nami-fune-horizontal-projectiles" | "lavaman-thrown-lava" => horizontal(checkout, name)?,
        "spore-projectile-motion" => spores(checkout)?,
        "yapping-maw-body" => maw(checkout)?,
        "lavaquake-rocks" => lavaquake(checkout)?,
        "shot-gates-electricity" => gates(checkout)?,
        "shaktool-attack-circles" => shaktool(checkout)?,
        "tourian-statue-scheduling" => tourian(checkout)?,
        "mother-brain-turret-projectiles" => turrets(checkout)?,
        "projectile-pickup-collection" => pickups(checkout)?,
        "draygon-projectile-scheduling" | "draygon-attached-gunk" => draygon(checkout, name)?,
        _ => anyhow::bail!("unknown projectile domain: {name}"),
    };
    m.family = name.into();
    Ok(m)
}
