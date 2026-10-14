//! Fixture domains with explicit independent projectile host words.
use super::*;
use anyhow::Context;
const SLOTS: [usize; 4] = [0, 2, 16, 34];
fn pool() -> Vec<(usize, i64)> {
    (0..36).step_by(2).map(|s| (0x1997 + s, 1)).collect()
}
fn host_case(name: String, a: usize, b: usize, writes: Writes, h: &[i64]) -> Case {
    Case {
        name: (name).to_owned(),
        a: (a) as i64,
        b: (b) as i64,
        writes,
        host_words: Some(h.iter().map(|v| *v as u16).collect()),
        ..Default::default()
    }
}
fn frame(name: String, writes: Writes, h: &[i64]) -> Case {
    Case {
        name: (name).to_owned(),
        writes,
        host_words: Some((h).iter().map(|v| *v as u16).collect()),
        ..Default::default()
    }
}
fn initializer(checkout: &Path) -> Result<Manifest> {
    let mut m = load(checkout, "enemy-drop-selection")?;
    let rom = super::rom(checkout)?;
    let mut cases = Vec::new();
    m.rust_module = "crates/port/src/enemy_projectiles.rs".into();
    m.projectile_host = true;
    for init in [0x947fu16, 0xe468, 0xef29] {
        let definition = rom[0x30000..0x38000]
            .windows(2)
            .position(|v| v == init.to_le_bytes())
            .context("projectile initializer absent")?
            + 0x8000;
        for i in 0..256 {
            let k = SLOTS[i % 4];
            let mut w = words(pool());
            if i % 16 != 15 {
                w.extend(words([(0x1997 + k, 0)]));
            }
            w.extend([
                Write { offset: 0x5e5_usize, data: vec![i, i].into_iter().map(|v| v as u8).collect() },
                Write { offset: 0x9c2_usize, data: vec![i, 0].into_iter().map(|v| v as u8).collect() },
                Write { offset: 0x9c4_usize, data: vec![99, 0].into_iter().map(|v| v as u8).collect() },
            ]);
            let h = [
                [0, 1, 0x7fff, 0x8000, 0xffff, ((i * 997) & 65535) as i64][i % 6],
                [0, 1, 0x7fff, 0x8000, 0xffff, ((i * 733) & 65535) as i64][(i / 6) % 6],
                11,
                22,
                33,
                44,
                0xa55a,
                [0, 0xd37f, 0xd3bf, 0xf593][i % 4],
            ];
            cases.push(host_case(
                format!("host-{init:04x}-{i}"),
                definition,
                if init == 0xe468 { i % 30 } else { i * 257 },
                w,
                &h,
            ));
        }
    }
    m.routines = vec![
        Routine {
            symbol: ("EprojInstruction").to_owned(),
            arg_types: Some((["uint16", "rom_ptr"]).into_iter().map(str::to_owned).collect()),
            returns: ("rom_ptr").to_owned(),
            cases: vec![Case {
                name: "control".into(),
                a: 0,
                b: 0x869754,
                host_words: Some(vec![0; 8]),
                ..Default::default()
            }],
            ..Default::default()
        },
        routine("SpawnEprojWithRoomGfx", 2, "void", cases),
    ];
    Ok(m)
}
fn pirate_callers(checkout: &Path) -> Result<Manifest> {
    let mut m = load(checkout, "pirate-host-context")?;
    let rom = super::rom(checkout)?;
    let mut routines = vec![m.routines[0].clone()];
    for (name, opcode, pointers) in [
        ("SpacePirates_Instr_FireLaserL", 0xef2au16, vec![0xecce]),
        ("SpacePirates_Instr_FireLaserR", 0xef5d, vec![0xed8e]),
        ("SpacePirates_Instr_15", 0xf564, vec![0xf17a, 0xf1aa, 0xf1e2, 0xf368, 0xf398, 0xf3d0]),
    ] {
        for &pc in &pointers {
            let p = 0x190000 + (pc & 0x7fff);
            ensure!(rom[p - 2..p] == opcode.to_le_bytes(), "pirate caller script differs");
            if opcode == 0xf564 {
                ensure!(u16::from_le_bytes(rom[p..p + 2].try_into()?) <= 1, "pirate parameter differs");
            }
        }
        let mut cases = Vec::new();
        for i in 0..192 {
            let enemy = i % 32 * 64;
            let k = SLOTS[i % 4];
            let mut v = (0..36).step_by(2).map(|s| (0x1997 + s, 0xaaaa)).collect::<Vec<_>>();
            if i % 6 != 0 {
                v.push((0x1997 + k, 0));
            }
            v.extend([
                (0xe54, enemy as i64),
                (0xf78 + enemy, 0xd37f),
                (0xf7a + enemy, [0, 23, 24, 0x7fff, 0x8000, 0xffff][i % 6]),
                (0xf7e + enemy, [0, 15, 16, 0xffff][i % 4]),
                (0xf96 + enemy, [0, 0x8000, 0x200][i % 3]),
                (0xf98 + enemy, [0, 0x1ff, 0xffff][i % 3]),
                (0xfb4 + enemy, [0, 0x8000, 0xffff][i % 3]),
                (0x1993, 0xface),
                (0x998, if i % 16 == 0 { 40 } else { 8 }),
                (0x5f5, i64::from(i % 16 == 1)),
            ]);
            let mut w = words(v);
            w.extend([
                Write { offset: 0x647_usize, data: vec![([0, 6, 15][i % 3])].into_iter().map(|v| v as u8).collect() },
                Write {
                    offset: 0x644_usize,
                    data: vec![usize::from(i % 4 == 0)].into_iter().map(|v| v as u8).collect(),
                },
                Write { offset: 0x654_usize, data: vec![0xa5].into_iter().map(|v| v as u8).collect() },
            ]);
            cases.push(host_case(
                format!("{opcode:04x}-{i}"),
                (enemy + 64) & 0x7c0,
                0xb20000 + pointers[i % pointers.len()],
                w,
                &[0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0xf6, 0xffff, 0x1234],
            ));
        }
        routines.push(Routine {
            symbol: (name).to_owned(),
            arg_types: Some((["uint16", "rom_ptr"]).into_iter().map(str::to_owned).collect()),
            pointer_bank: Some(0xb2),
            returns: ("rom_ptr").to_owned(),
            cases,
            ..Default::default()
        });
    }
    m.routines = routines;
    Ok(m)
}
fn pirates(checkout: &Path) -> Result<Manifest> {
    let mut m = load(checkout, "projectile-host-context")?;
    let rom = super::rom(checkout)?;
    ensure!(rom[0x3217b..0x3217d] == [9, 0xa0] && rom[0x32189..0x3218b] == [0x98, 0xa0], "pirate initializers differ");
    let mut spawn = Vec::new();
    let mut frames = Vec::new();
    for (init, definition) in [(0xa009, 0xa17b), (0xa098, 0xa189)] {
        for i in 0..128 {
            let k = SLOTS[i % 4];
            let enemy = (i % 32) * 64;
            let parameter = [0, 1, 0xffff][i % 3];
            let x = [0, 127, 128, 255, 0x7fff, 0x8000, 0xffff, 128][i % 8];
            let y = [0, 16, 239, 255][(i / 8) % 4];
            let h = [
                x,
                y,
                x,
                y,
                [0, 1, 0xffff, 0x8000][i % 4],
                [0, 1, 0xffff, 0x8000][(i / 4) % 4],
                [0, 1, 0xffff][i % 3],
                77,
            ];
            let mut v = pool();
            v.extend([
                (0x1997 + k, 0),
                (0xe54, enemy as i64),
                (0xf78 + enemy, 0xd37f),
                (0xfb4 + enemy, [0, 0x8000, 0xffff][i % 3]),
                (0x998, if i % 16 == 0 { 40 } else { 8 }),
                (0x5f5, i64::from(i % 16 == 1)),
            ]);
            spawn.push(host_case(format!("spawn-{init:04x}-{i}"), definition, parameter as usize, words(v), &h));
            let script = if init == 0xa009 {
                if h[6] != 0 { 0x9f7d } else { 0x9f41 }
            } else if parameter != 0 {
                0x9fe1
            } else {
                0x9fb9
            };
            let xx = if init == 0xa009 { x } else { (h[2] + h[4]) & 65535 };
            let yy = if init == 0xa009 { y } else { (h[3] + h[5]) & 65535 };
            frames.push(frame(
                format!("frame-{init:04x}-{i}"),
                words([
                    (0x1997 + k, definition as i64),
                    (0x1a03 + k, 0xa05b),
                    (0x1b47 + k, script),
                    (0x1b8f + k, 1),
                    (0x1a4b + k, xx),
                    (0x1a93 + k, yy),
                    (0x1aff + k, if init == 0xa009 { 0x8000 } else { 2048 }),
                    (0x1b23 + k, if init == 0xa009 { 0 } else { 1 }),
                    (0x198d, 0x8000),
                ]),
                &h,
            ));
        }
    }
    for i in 0..32 {
        let k = [0, 34][i % 2];
        let pre = [0xa05c, 0xa07a, 0xa0d1, 0xa124][i % 4];
        frames.push(frame(
            format!("flight-{i}"),
            words([
                (0x1997 + k, if pre >= 0xa0d1 { 0xa189 } else { 0xa17b }),
                (0x1a03 + k, pre),
                (0x1b47 + k, 0x9754),
                (0x1b8f + k, 2),
                (0x1a4b + k, [0, 128, 255, 0xffff][i % 4]),
                (0x1a93 + k, 32),
                (0x1aff + k, [0, 32, 2048, 0x8000][i % 4]),
                (0x1b23 + k, (i % 2) as i64),
                (0x198d, 0x8000),
            ]),
            &[1, 2, 3, 4, 5, 6, 7, 8],
        ));
    }
    for (i, (write, read)) in [(0, 0), (6, 0), (15, 0), (0, 1)].into_iter().enumerate() {
        let mut w = words([(0xe54, 0), (0xf78, 0xd37f), (0x998, 8)]);
        w.extend([
            Write { offset: 0x647_usize, data: vec![write].into_iter().map(|v| v as u8).collect() },
            Write { offset: 0x644_usize, data: vec![read].into_iter().map(|v| v as u8).collect() },
            Write { offset: 0x654_usize, data: vec![0xa5].into_iter().map(|v| v as u8).collect() },
            Write { offset: (0x666 + write) as usize, data: vec![0xff].into_iter().map(|v| v as u8).collect() },
        ]);
        spawn.push(host_case(format!("quota-{i}"), 0xa17b, 0, w, &[100, 32, 3, 4, 5, 6, 0, 8]));
    }
    m.routines = vec![
        m.routines[0].clone(),
        routine("SpawnEprojWithRoomGfx", 2, "void", spawn),
        routine("EprojRunAll", 0, "void", frames),
    ];
    Ok(m)
}
fn spores(checkout: &Path) -> Result<Manifest> {
    let mut m = load(checkout, "projectile-host-context")?;
    let rom = super::rom(checkout)?;
    ensure!(rom[0x35c0e..0x35c10] == [0x77, 0xdc], "spore opcode differs");
    let mut direct = Vec::new();
    let mut frames = Vec::new();
    let mut pre = Vec::new();
    let h = [1, 2, 3, 4, 5, 6, 7, 8];
    for i in 0..256 {
        let k = SLOTS[i % 4];
        let x = [0, 127, 128, 255, 0x7fff, 0x8000, 0xffff, ((i * 997) & 65535) as i64][i % 8];
        let y = [0, 766, 767, 768, 0x7fff, 0x8000, 0xffff, 520][(i / 8) % 8];
        let mut v = vec![
            (0x1997 + k, 0xde88),
            (0x1a03 + k, 0xdd46),
            (0x1b23 + k, 5),
            (0x1b47 + k, 0xdc0e),
            (0x1b8f + k, 1),
            (0x1a4b + k, x),
            (0x1a93 + k, y),
            (0x5e5, (i * 257) as i64),
        ];
        if i & 4 != 0 {
            for s in (0..36).step_by(2).filter(|s| *s != k) {
                v.extend([(0x1997 + s, 1), (0x1a03 + s, 0x8170), (0x1b47 + s, 0x9754), (0x1b8f + s, 2)]);
            }
            if i & 8 != 0 {
                v.push((0x1997 + if k != 0 { 0 } else { 2 }, 0));
            }
        }
        direct.push(host_case(format!("spawn-{i}"), k, 0x86dc0e, words(v.clone()), &h));
        frames.push(frame(format!("frame-{i}"), words(v.into_iter().chain([(0x198d, 0x8000)])), &h));
        pre.push(host_case(
            format!("move-{i}"),
            k,
            0xdcee,
            words([
                (0x1997 + k, 0xde7a),
                (0x1a03 + k, 0xdcee),
                (0x1aff + k, i as i64),
                (0x1b23 + k, x),
                (0x1a4b + k, x),
                (0x1a93 + k, y),
            ]),
            &h,
        ));
    }
    m.routines = vec![
        Routine {
            symbol: ("EprojInstruction").to_owned(),
            arg_types: Some((["uint16", "rom_ptr"]).into_iter().map(str::to_owned).collect()),
            returns: ("rom_ptr").to_owned(),
            cases: direct,
            ..Default::default()
        },
        routine("EprojRunAll", 0, "void", frames),
        routine("EprojPreinstruction", 2, "void", pre),
    ];
    Ok(m)
}
fn turret(checkout: &Path) -> Result<Manifest> {
    let mut m = load(checkout, "projectile-host-context")?;
    let rom = super::rom(checkout)?;
    ensure!(
        rom[0x36509..0x3650b] == [0x68, 0xe4] && rom[0x3418c..0x3418e] == [0x59, 0xbf],
        "turret definitions differ"
    );
    let mut pre = Vec::new();
    let mut frames = Vec::new();
    let h = [1, 2, 3, 4, 5, 6, 7, 8];
    for i in 0..256 {
        let k = SLOTS[i % 4];
        let camera = [0, 0xff80, 0x7fff][i % 3];
        let x = camera + [-5, -4, 0, 259, 260, 261, 0x8000, 128][i % 8];
        let y = camera + [-17, -16, 0, 239, 240, 241, 0x8000, 32][(i / 8) % 8];
        let mut v = vec![
            (0x1997 + k, 0xc17e),
            (0x1a03 + k, 0xbfdf),
            (0x1b47 + k, 0x9754),
            (0x1b8f + k, 2),
            (0x1a4b + k, x & 65535),
            (0x1a93 + k, y & 65535),
            (0x911, camera),
            (0x915, camera),
            (0x783a, (i % 2) as i64),
            (0x1a6f + k, (0x100 | (i % 8)) as i64),
            (0x1a27 + k, 0xbec1),
            (0x1ab7 + k, [0, 1, 2, 0xffff][i % 4]),
            (0x1adb + k, [1, 2, 0, 0xffff][(i / 4) % 4]),
            (0x5e5, (i * 257) as i64),
            (0x7a5, 16),
        ];
        if i & 16 != 0 {
            for s in (0..36).step_by(2).filter(|s| *s != k) {
                v.extend([(0x1997 + s, 1), (0x1a03 + s, 0x8170), (0x1b47 + s, 0x9754), (0x1b8f + s, 2)]);
            }
        }
        pre.push(host_case(format!("turret-{i}"), k, 0xbfdf, words(v.clone()), &h));
        frames.push(frame(format!("frame-{i}"), words(v.into_iter().chain([(0x198d, 0x8000)])), &h));
    }
    for i in 0..32 {
        let k = [0, 34][i % 2];
        let v = vec![
            (0x1997 + k, 0xc17e),
            (0x1a03 + k, 0xbfdf),
            (0x1b47 + k, 0x9754),
            (0x1b8f + k, 2),
            (0x1a4b + k, 100),
            (0x1a93 + k, 32),
            (0x783a, ((i / 2) % 2) as i64),
            (0x1a6f + k, (0x100 | (i % 8)) as i64),
            (0x1a27 + k, 0xbec1),
            (0x1ab7 + k, 1),
            (0x1adb + k, 1),
            (0x7a5, 16),
        ];
        pre.push(host_case(format!("active-{i}"), k, 0xbfdf, words(v.clone()), &h));
        frames.push(frame(format!("frame-active-{i}"), words(v.into_iter().chain([(0x198d, 0x8000)])), &h));
    }
    m.routines = vec![
        m.routines[0].clone(),
        routine("EprojPreinstruction", 2, "void", pre),
        routine("EprojRunAll", 0, "void", frames),
    ];
    Ok(m)
}
fn ridley(checkout: &Path) -> Result<Manifest> {
    let mut m = load(checkout, "projectile-host-context")?;
    let mut pre = Vec::new();
    let mut frames = Vec::new();
    let h = [0xa55a, 0x5aa5, 11, 22, 33, 44, 55, 66];
    for i in 0..256 {
        let k = SLOTS[i % 4];
        let axis = i % 3;
        let mut v = vec![
            (0x1997 + k, 0x9642),
            (0x1a03 + k, 0x940e),
            (0x1b23 + k, if i % 8 == 7 { 1 } else { 0 }),
            (0x1a4b + k, 29),
            (0x1a93 + k, 29),
            (0x1a27 + k, 0xffa5),
            (0x1a6f + k, 0xff5a),
            (0x1bb3 + k, 0x0202),
            (0x1ab7 + k, 0x300),
            (0x1adb + k, 0x300),
            (0x7a5, 16),
            (0x1b47 + k, 0x9754),
            (0x1b8f + k, 2),
            (0x198d, 0x8000),
            (0x998, if i % 16 == 0 { 40 } else { 8 }),
            (0x5f5, i64::from(i % 16 == 1)),
        ];
        if axis == 0 {
            v.push((0x10002 + 18 * 2, 0x8000));
        } else if axis == 1 {
            v.push((0x10002 + 33 * 2, 0x8000));
        }
        if i & 4 != 0 {
            for s in (0..36).step_by(2).filter(|s| *s != k) {
                v.extend([(0x1997 + s, 1), (0x1a03 + s, 0x8170), (0x1b47 + s, 0x9754), (0x1b8f + s, 2)]);
            }
        }
        pre.push(host_case(format!("impact-{i}"), k, 0x940e, words(v.clone()), &h));
        frames.push(frame(format!("frame-{i}"), words(v), &h));
    }
    // Keep targeted controls distinct from the broad matrix.
    let mut append = |name: String, k: usize, w: Writes| {
        pre.push(host_case(name.clone(), k, 0x940e, w.clone(), &h));
        frames.push(frame(format!("frame-{name}"), w, &h));
    };
    for axis in [0, 1, 2] {
        for k in [0, 34] {
            for gate in 0..5 {
                let mut v = vec![
                    (0x1997 + k, 0x9642),
                    (0x1a03 + k, 0x940e),
                    (0x1b23 + k, 0),
                    (0x1a4b + k, 29),
                    (0x1a93 + k, 29),
                    (0x1bb3 + k, 0x0202),
                    (0x1ab7 + k, 0x300),
                    (0x1adb + k, 0x300),
                    (0x7a5, 16),
                    (0x1b47 + k, 0x9754),
                    (0x1b8f + k, 2),
                    (0x198d, 0x8000),
                    (0x998, if gate == 1 { 40 } else { 8 }),
                    (0x5f5, i64::from(gate == 2)),
                    (0x592, if gate == 3 { 0x8000 } else { 0 }),
                ];
                if axis == 0 || axis == 2 {
                    v.push((0x10002 + 18 * 2, 0x8000));
                }
                if axis == 1 || axis == 2 {
                    v.push((0x10002 + 33 * 2, 0x8000));
                }
                let mut w = words(v);
                w.extend([
                    Write {
                        offset: 0x647_usize,
                        data: vec![if gate == 4 { 6 } else { 0 }].into_iter().map(|v| v as u8).collect(),
                    },
                    Write { offset: 0x644_usize, data: vec![0].into_iter().map(|v| v as u8).collect() },
                ]);
                append(format!("priority-{axis}-{k}-{gate}"), k, w);
            }
        }
    }
    for k in [0, 34] {
        for (vx, vy) in [(0xfd00, 0), (0, 0xfd00), (0xfd00, 0xfd00)] {
            append(
                format!("negative-{k}-{vx}-{vy}"),
                k,
                words([
                    (0x1997 + k, 0x9642),
                    (0x1a03 + k, 0x940e),
                    (0x1a4b + k, 17),
                    (0x1a93 + k, 17),
                    (0x1bb3 + k, 0x0202),
                    (0x1ab7 + k, vx),
                    (0x1adb + k, vy),
                    (0x7a5, 16),
                    (0x10002, 0x8000),
                    (0x1b47 + k, 0x9754),
                    (0x1b8f + k, 2),
                    (0x198d, 0x8000),
                ]),
            );
        }
    }
    m.routines = vec![
        m.routines[0].clone(),
        routine("EprojPreinstruction", 2, "void", pre),
        routine("EprojRunAll", 0, "void", frames),
    ];
    Ok(m)
}
fn drops(checkout: &Path) -> Result<Manifest> {
    let mut m = load(checkout, "projectile-host-context")?;
    let rom = super::rom(checkout)?;
    let mut direct = Vec::new();
    let mut frames = Vec::new();
    let h = [1, 2, 3, 4, 5, 6, 7, 8];
    for (opcode, pointer) in [(0xab8au16, 0xab82), (0x9270, 0x901b), (0x8c68, 0x8c60), (0xdc61, 0xdc54)] {
        let p = 0x30000 + (pointer & 0x7fff);
        ensure!(rom[p..p + 2] == opcode.to_le_bytes(), "drop script word differs");
        for i in 0..64 {
            let k = SLOTS[i % 4];
            let mut v = vec![
                (0x1997 + k, 1),
                (0x1a03 + k, 0x8170),
                (0x1b47 + k, pointer as i64),
                (0x1b8f + k, 1),
                (0x1a4b + k, ((i * 997) & 65535) as i64),
                (0x1a93 + k, ((i * 733) & 65535) as i64),
                (0x79f, (i % 6) as i64),
                (0x5e5, (i * 257) as i64),
                (0x9c2, [0, 29, 30, 49, 50, 99][i % 6]),
                (0x9c4, 99),
                (0xf96 + k, (i * 113) as i64),
                (0xf98 + k, (i * 137) as i64),
            ];
            if i & 4 != 0 {
                v.extend(pool());
                if i & 8 != 0 {
                    v.push((0x1997 + if k != 0 { 0 } else { 2 }, 0));
                }
                v.extend((0..36).step_by(2).filter(|s| *s != k).map(|s| (0x1a03 + s, 0x8170)));
                v.extend((0..36).step_by(2).filter(|s| *s != k).map(|s| (0x1b47 + s, 0x9754)));
                v.extend((0..36).step_by(2).filter(|s| *s != k).map(|s| (0x1b8f + s, 2)));
            }
            let w = words(v);
            direct.push(host_case(format!("drop-{opcode:04x}-{i}"), k, 0x860000 | pointer, w.clone(), &h));
            frames.push(frame(
                format!("frame-{opcode:04x}-{i}"),
                w.into_iter()
                    .chain([Write { offset: 0x198d_usize, data: vec![0, 128].into_iter().map(|v| v as u8).collect() }])
                    .collect(),
                &h,
            ));
        }
    }
    let mut append = |name: String, k: usize, pointer: usize, w: Writes| {
        direct.push(host_case(name.clone(), k, 0x860000 | pointer, w.clone(), &h));
        frames.push(frame(
            format!("frame-{name}"),
            w.into_iter()
                .chain([Write { offset: 0x198d_usize, data: vec![0, 128].into_iter().map(|v| v as u8).collect() }])
                .collect(),
            &h,
        ));
    };
    for i in 0..12 {
        let k = [0, 2, 34][i % 3];
        let pointer = 0xab82;
        append(
            format!("edge-{i}"),
            k,
            pointer,
            words([
                (0x1997 + k, 1),
                (0x1a03 + k, 0x8170),
                (0x1b47 + k, pointer as i64),
                (0x1b8f + k, 1),
                (0x1a4b + k, [0x7fff, 0x8000, 0xffff][i % 3]),
                (0x1a93 + k, 0xffff),
                (0x5e5, 99),
                (0x79f, (i % 2) as i64),
                (0x9c2, 99),
                (0x9c4, 99),
                (0x9d6, 100),
                (0x9d4, 100),
                (0x9c6, 10),
                (0x9c8, 10),
                (0x9ca, 10),
                (0x9cc, 10),
                (0x9ce, 10),
                (0x9d0, 10),
            ]),
        );
    }
    m.routines = vec![
        Routine {
            symbol: ("EprojInstruction").to_owned(),
            arg_types: Some((["uint16", "rom_ptr"]).into_iter().map(str::to_owned).collect()),
            returns: ("rom_ptr").to_owned(),
            cases: direct,
            ..Default::default()
        },
        routine("EprojRunAll", 0, "void", frames),
    ];
    Ok(m)
}
pub(super) fn generate(checkout: &Path, name: &str) -> Result<Manifest> {
    let mut m = match name {
        "projectile-host-context" => initializer(checkout)?,
        "pirate-host-callers" => pirate_callers(checkout)?,
        "pirate-host-context" => pirates(checkout)?,
        "host-spore-spawning" => spores(checkout)?,
        "mother-brain-turret-host-lifecycle" => turret(checkout)?,
        "ridley-fireball-host-impact" => ridley(checkout)?,
        "projectile-host-drop-instructions" => drops(checkout)?,
        _ => anyhow::bail!("unknown host domain: {name}"),
    };
    m.family = name.into();
    Ok(m)
}
