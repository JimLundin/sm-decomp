//! Frozen inputs/expected effects for actual background and installation entries.
use crate::room_background_support::ram_bytes;
use crate::{
    game_state::Game, rom::RomImage, room_background_fixture::*, room_background_support::*,
};
use std::panic::{AssertUnwindSafe, catch_unwind};
pub(crate) const CASES: u32 = 37;
#[derive(Clone, Copy)]
pub(crate) enum Operation {
    Background(bool),
    Level(bool),
    Tiles,
    Place,
}
pub(crate) fn run(op: Operation, g: &mut Game) -> Option<u8> {
    match op {
        Operation::Background(initial) => {
            use {
                crate::game::session_runtime::Runtime,
                sm_game::room_entry::{background, content::LoadMode},
            };
            background::load(
                &mut Runtime::new(g),
                if initial {
                    LoadMode::Initial
                } else {
                    LoadMode::Door
                },
            );
            None
        }
        Operation::Level(initial) => {
            use sm_game::room_entry::{content::LoadMode, installation};
            let mode = if initial {
                LoadMode::Initial
            } else {
                LoadMode::Door
            };
            installation::load_level(&mut crate::game::session_runtime::Runtime::new(g), mode);
            None
        }
        Operation::Tiles => {
            sm_game::room_entry::installation::load_tiles_and_palette(
                &mut crate::game::session_runtime::Runtime::new(g),
            );
            None
        }
        Operation::Place => {
            let poll = crate::game::room_entry::protocol::dispatch(
                &mut crate::game::session_runtime::Runtime::new(g),
                0x82e3c0,
            );
            Some(poll.returned_byte())
        }
    }
}
fn transfer(r: &mut [u8], at: u16, opcode: u16) {
    rw(r, 0x8f0000 | u32::from(at), opcode);
    let p = 0x8f0000 | u32::from(at) + 2;
    bytes(r, p, &[0, 0x60, 0x7e, 0x23, 0x31, 4, 0]);
}
fn asset(g: &mut Game, at: usize, address: u32) {
    g.ram().copy_in(at, &address.to_le_bytes()[..3]);
}
fn fill_words(e: &mut [u8], start: usize, end: usize, value: u16) {
    for a in (start..end).step_by(2) {
        exp(e, a, value);
    }
}
fn state(r: &mut [u8], scroll: u16) {
    for a in [0x8f9000, 0x8f9100] {
        bytes(r, a, &[0; 26]);
    }
    rw(r, 0x8f900e, scroll);
    bytes(r, 0x838000, &[0; 12]);
}
pub(crate) fn capture_case(
    id: u32,
    run: fn(Operation, &mut Game) -> Option<u8>,
) -> Option<Vec<u8>> {
    if id >= 38 {
        return raster_capture_case(id);
    }
    if id >= CASES {
        return None;
    }
    let result = catch_unwind(AssertUnwindSafe(|| unsafe {
        let mut g = initialized();
        let len = match id {
            6 | 7 => offset(0x8ff000) + 5,
            12 => offset(0x8fa000) + 2,
            13 => offset(0x8ffff7) + 2,
            28 => offset(0x8ffff0) + 7,
            _ => 0x400000,
        };
        let mut a = vec![0; len];
        let mut b = vec![0; 0x400000];
        put(&mut g, 0x7bb, 0x9000);
        put(&mut g, 0x78d, 0x8000);
        put(&mut g, 0x1964, 0);
        put(&mut g, 0x79f, 6);
        put(&mut g, 0x7b3, 0);
        put(&mut g, 0x7a9, 2);
        put(&mut g, 0x7ab, 2);
        put(&mut g, 0xe16, 1);
        put(&mut g, 0xe18, 0xcccc);
        let initial_mode = matches!(
            id,
            0 | 2 | 4 | 6 | 8 | 10 | 12 | 15 | 17 | 19 | 20 | 22 | 24 | 26 | 29 | 32
        );
        let op = if id < 24 {
            Operation::Background(initial_mode)
        } else if id < 33 {
            Operation::Level(initial_mode)
        } else if id == 33 {
            Operation::Tiles
        } else {
            Operation::Place
        };
        let mut writes = Vec::new();
        let mut diagnostics = Vec::new();
        let mut mutations = Vec::new();
        let mut expected_source = 0xabcdef;
        let mut panic_text: Option<String> = None;
        let mut panic_kind = 2;
        state(&mut a, 0);
        state(&mut b, 0);
        if id < 24 {
            // Selected-state background defaults to0; command cases override it.
            if !matches!(id, 0 | 1 | 8 | 9 | 20 | 21) {
                rw(
                    &mut a,
                    0x8f9016,
                    if matches!(id, 10 | 11) {
                        0xfff7
                    } else if id == 13 {
                        0xfff5
                    } else if id == 14 {
                        0xfffe
                    } else if matches!(id, 22 | 23) {
                        0xfff9
                    } else {
                        0xa000
                    },
                );
            }
            match id {
                2..=5 | 16 => {
                    transfer(&mut a, 0xa000, if id == 16 { 9 } else { 3 });
                    rw(&mut a, 0x8fa009, if id == 4 || id == 5 { 16 } else { 0 });
                    rw(&mut b, 0x8fa009, 0);
                }
                6 | 7 => {
                    rw(&mut a, 0x8feffe, 2);
                    rw(&mut a, 0x8f9016, 0xeffe);
                    bytes(&mut a, 0x8ff000, &[0, 0x60, 0x7e, 0x23, 0x31]);
                    panic_text = Some("ROM read outside loaded allocation".into());
                }
                8 | 9 => {
                    put(&mut g, 0x7bb, 0x10);
                    bytes(&mut a, 0x8f0010, &[0; 26]);
                    if id == 8 {
                        diagnostics.push(0x8f0010);
                    } else {
                        panic_text = Some("ROM address must select a LoROM window".into());
                    }
                }
                10 | 11 => transfer(&mut a, 0xfff7, 2),
                12 => rw(&mut a, 0x8fa000, 1),
                13 => {
                    rw(&mut a, 0x8ffff5, 14);
                    rw(&mut a, 0x8ffff7, 0x8123);
                }
                14 => {
                    rw(&mut a, 0x8ffffe, 14);
                    rw(&mut a, 0x8f0000, 0x8000);
                    diagnostics.push(0x8f0000);
                    panic_text = Some("ROM address must select a LoROM window".into());
                }
                15 => {
                    rw(&mut a, 0x8fa000, 5);
                    bytes(&mut a, 0x8fa002, &[0, 0xa0, 0x80, 0, 0x60]);
                    literal(&mut a, 0x80a000, &[0xe1, 0xe2]);
                    rw(&mut a, 0x8fa007, 0);
                    expected_source = 0x80a004;
                }
                17 | 18 => {
                    rw(&mut a, 0x8fa000, 12);
                    rw(&mut a, 0x8fa002, 0);
                }
                19 => {
                    rw(&mut a, 0x8fa000, 16);
                    panic_kind = 1;
                    panic_text = Some("invalid room background command: 0x10".into());
                }
                20 | 21 => put(&mut g, 0x1964, 0x9000),
                22 | 23 => {
                    transfer(&mut a, 0xfff9, 2);
                    rw(&mut a, 0x8f0002, 0);
                    if id == 22 {
                        diagnostics.push(0x8f0002);
                    } else {
                        panic_text = Some("ROM address must select a LoROM window".into());
                    }
                }
                _ => (),
            }
        } else if id < 33 {
            asset(&mut g, 0x7bd, 0x808000);
            asset(&mut g, 0x7c0, 0x809000);
            literal(&mut a, 0x808000, &[2, 0, 0xa1, 0xa2, 0xb1, 0xc1, 0xc2]);
            literal(&mut a, 0x809000, &[0xd1, 0xd2]);
            expected_source = 0x809004;
            if matches!(id, 26 | 27 | 29) {
                rw(&mut a, 0x8f900e, 0xfffe);
                rw(&mut a, 0x8ffffe, 0x1100);
                for i in 0..24 {
                    rw(&mut a, 0x8f0000 + 2 * i, 0x1101 + i as u16);
                    rw(&mut b, 0x8f0000 + 2 * i, 0x2201 + i as u16);
                    diagnostics.push(0x8f0000 + 2 * i);
                }
                if id == 29 {
                    rw(&mut a, 0x8f9014, 0x40);
                    rw(&mut b, 0x8f0040, 0);
                    diagnostics.push(0x8f0040);
                    bytes(&mut b, 0x8f0080, &[0; 26]);
                    diagnostics.push(0x8f0080);
                }
            }
            if id == 28 {
                rw(&mut a, 0x8f900e, 0xfff0);
                for i in 0..3 {
                    rw(&mut a, 0x8ffff0 + 2 * i, 0x3300 + i as u16);
                }
                bytes(&mut a, 0x8ffff6, &[0x44]);
                panic_kind = 1;
                panic_text = Some(format!(
                    // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
                    "index out of bounds: the len is {len} but the index is {len}"
                ));
            }
            if id >= 30 {
                put(&mut g, 0x79f, 0);
                put(&mut g, 0x7b3, if id == 31 { 2 } else { 0 });
                literal(&mut a, 0xb9a09d, &[0xc3, 0xc4]);
            }
        } else {
            asset(&mut g, 0x7c3, 0x808000);
            asset(&mut g, 0x7c6, 0x809000);
            literal(&mut a, 0xb98000, &[0x91, 0x92]);
            literal(&mut a, 0x808000, &[0xa1, 0xa2]);
            literal(&mut a, 0x809000, &[0xb1, 0xb2]);
            expected_source = 0x809004;
            if id >= 34 {
                put(&mut g, 0x78d, if id == 34 { 0x947a } else { 0x9000 });
                put(&mut g, 0x7b3, if id == 36 { 4 } else { 2 });
                put(
                    &mut g,
                    0x791,
                    if id == 34 {
                        3
                    } else if id == 35 {
                        2
                    } else {
                        0
                    },
                );
                put(&mut g, 0x911, 0x1200);
                put(&mut g, 0x915, 0x2300);
                put(&mut g, 0xaf6, 0x34);
                put(&mut g, 0xafa, 0x56);
                put(&mut g, 0x84, if id == 35 { 0 } else { 0x30 });
                put(&mut g, 0x931, 0xcccc);
            }
        }
        let mut expected = ram_bytes(&g);
        let mut vram = (&(*(*g.g_snes).ppu).vram).to_vec();
        if id < 24 {
            if initial_mode {
                clear_initial(&mut writes, 0x5880, 0xa2f7, 0xa2f8, 0x780);
                vram[0x5880..0x6000].fill(0x184e);
            } else {
                fill_words(&mut expected, 0x4000, 0x4f00, 0x184e);
                copy(&mut writes, 0x5880, 0x7e4000, 0xf00);
                vram[0x5880..0x6000].fill(0x184e);
            }
            if matches!(id, 20 | 21) {
                if id == 20 {
                    word(&mut writes, 0x4312, 0x9000);
                    word(&mut writes, 0x2116, 0x5be0);
                    word(&mut writes, 0x4310, 0x1801);
                    word(&mut writes, 0x4314, 0x8a);
                    word(&mut writes, 0x4315, 0x840);
                    writes.extend([(0x2115, 0x80), (0x420b, 2)]);
                } else {
                    copy(&mut writes, 0x5be0, 0x8a9000, 0x840);
                }
                vram[0x5be0..0x6000].fill(0); // synthetic cartridge bytes
            }
            if matches!(id, 2 | 3 | 4 | 5 | 6 | 7 | 10 | 11 | 16 | 22 | 23) {
                let base = writes.len();
                if initial_mode {
                    // The source moves to another image mid-command; the
                    // command's fields still come from the image it started in.
                    direct(
                        &mut writes,
                        0x3123,
                        0x6000,
                        0x237e,
                        if id == 6 { None } else { Some(4) },
                    );
                    if id == 4 {
                        mutations.extend([base + 1, base + 3]);
                    }
                } else if id != 7 {
                    copy(&mut writes, 0x3123, 0x7e6000, 4);
                    if id == 5 {
                        mutations.push(base);
                    }
                }
                if id != 6 && id != 7 {
                    vram_bytes(&mut vram, 0x3123, &expected[0x6000..0x6004]);
                }
                if id == 16 {
                    expected[0x5e] = 2;
                }
            }
            if id == 15 {
                expected[0x6000..0x6002].copy_from_slice(&[0xe1, 0xe2]);
            }
            if id == 17 {
                clear_initial(&mut writes, 0x4800, 0xa29a, 0xa29a, 0x800);
                vram[0x4800..0x5000].fill(0x3838);
            }
            if id == 18 {
                fill_words(&mut expected, 0x4000, 0x5000, 0x338);
                copy(&mut writes, 0x4000, 0x7e4000, 0x1000);
                copy(&mut writes, 0x4800, 0x7e4000, 0x1000);
                vram[0x4000..0x5000].fill(0x338);
            }
        } else if id < 33 {
            if initial_mode {
                fill_words(&mut expected, 0x10002, 0x16402, 0x8000);
            } else {
                fill_words(&mut expected, 0x10002, 0x11902, 0x8000);
                fill_words(&mut expected, 0x11902, 0x13202, 0x8000);
                fill_words(&mut expected, 0x13202, 0x14b02, 0x8000);
                fill_words(&mut expected, 0x14b02, 0x16402, 0x8000);
            }
            expected[0x10000..0x10007].copy_from_slice(&[2, 0, 0xa1, 0xa2, 0xb1, 0xc1, 0xc2]);
            expected[0x16402] = 0xb1;
            expected[0x19602..0x19604].copy_from_slice(&[0xc1, 0xc2]);
            if id < 30 {
                expected[0xa000..0xa002].copy_from_slice(&[0xd1, 0xd2]);
            } else {
                if id != 30 {
                    expected[0xa000..0xa002].copy_from_slice(&[0xc3, 0xc4]);
                }
                expected[0xa800..0xa802].copy_from_slice(&[0xd1, 0xd2]);
            }
            if matches!(id, 26 | 27 | 29) {
                exp(&mut expected, 0xcd20, 0x1100);
                for i in 0..24 {
                    exp(
                        &mut expected,
                        0xcd22 + 2 * i,
                        if id == 29 {
                            0x2201 + i as u16
                        } else {
                            0x1101 + i as u16
                        },
                    );
                }
            } else if id == 28 {
                for i in 0..3 {
                    exp(&mut expected, 0xcd20 + 2 * i, 0x3300 + i as u16);
                }
            } else {
                expected[0xcd20..0xcd24].copy_from_slice(&[2, 2, 1, 1]);
            }
            if initial_mode {
                exp(&mut expected, 0xe18, 2);
            }
            if id == 29 {
                exp(&mut expected, 0x7bb, 0x80);
            }
        } else if id == 33 {
            exp(&mut expected, 0xe16, 0);
            word(&mut writes, 0x2115, 0x80);
            word(&mut writes, 0x2116, 0x2800);
            writes.extend([(0x2118, 0x91), (0x2119, 0x92)]);
            word(&mut writes, 0x2116, 0);
            writes.extend([(0x2118, 0xa1), (0x2119, 0xa2)]);
            vram[0x2800] = 0x9291;
            vram[0] = 0xa2a1;
            expected[0xc200..0xc202].copy_from_slice(&[0xb1, 0xb2]);
        } else {
            expected[0x782] = 0;
            expected[0x612] = 0;
            for (at, value) in [
                (0xaf6, 0x1234),
                (0xb10, 0x1234),
                (0xafa, 0x2356),
                (0xb14, 0x2356),
                (0x931, 0),
                (0xa7, if id == 36 { 22 } else { 16 }),
                (0x99c, 0xe4a9),
            ] {
                exp(&mut expected, at, value);
            }
            if id == 35 {
                word(&mut writes, 0x4209, 0);
                word(&mut writes, 0x4207, 152);
                writes.push((0x4200, 0x30));
                exp(&mut expected, 0x84, 0x30);
                expected[0x7000..0x7002].copy_from_slice(&[0x91, 0x92]);
            }
            expected[0x2000..0x2002].copy_from_slice(&[0xa1, 0xa2]);
            expected[0xc200..0xc202].copy_from_slice(&[0xb1, 0xb2]);
            for (dst, src, n) in [
                (0, 0x2000, 0x2000),
                (0x1000, 0x4000, 0x2000),
                (0x2000, 0x6000, 0x1000),
            ] {
                copy(&mut writes, dst, 0x7e0000 | src, n);
                vram_bytes(
                    &mut vram,
                    dst,
                    &expected[src as usize..(src + n as u32) as usize],
                );
            }
            if id != 34 {
                for (dst, src, n) in [(0x2800, 0x7000, 0x1000), (0x3000, 0x8000, 0x2000)] {
                    copy(&mut writes, dst, 0x7e0000 | src, n);
                    vram_bytes(
                        &mut vram,
                        dst,
                        &expected[src as usize..(src + n as u32) as usize],
                    );
                }
                copy(&mut writes, 0x4000, 0x9ab200, 0x1000);
                let data = (0..0x1000usize)
                    .map(|i| (i.wrapping_mul(7) ^ 0xd3) as u8)
                    .collect::<Vec<_>>();
                vram_bytes(&mut vram, 0x4000, &data);
            }
        }
        let expected_fail = !diagnostics.is_empty();
        let expected_return = if id >= 34 { Some(0) } else { None };
        let mut out = b"RMBG1".to_vec();
        out.extend_from_slice(&id.to_le_bytes());
        record(&mut out, 0, &snapshot(&mut g));
        g.loaded_rom = Some(RomImage::from(a));
        crate::room_background_observer::begin(
            id,
            writes,
            diagnostics,
            RomImage::from(b),
            mutations,
        );
        let actual = catch_unwind(AssertUnwindSafe(|| run(op, &mut g)));
        let trace = crate::room_background_observer::finish();
        match actual {
            Ok(value) => {
                assert!(panic_text.is_none(), "expected checked failure");
                assert_eq!(value, expected_return);
                record(
                    &mut out,
                    7,
                    &match value {
                        Some(v) => vec![1, v],
                        None => vec![0],
                    },
                );
            }
            Err(payload) => {
                let (kind, text) = if let Some(s) = payload.downcast_ref::<String>() {
                    (1, s.as_str())
                } else if let Some(s) = payload.downcast_ref::<&str>() {
                    (2, *s)
                } else {
                    (3, "non-string panic")
                };
                assert_eq!(Some(text), panic_text.as_deref(), "exact expected panic");
                assert_eq!(kind, panic_kind, "panic payload type");
                let mut p = vec![2, kind];
                p.extend_from_slice(text.as_bytes());
                record(&mut out, 7, &p);
            }
        }
        assert_eq!(&ram_bytes(&g), &expected[..], "background {} full RAM", id);
        assert_eq!(
            &(&(*(*g.g_snes).ppu).vram)[..],
            &vram[..],
            "background {} full VRAM",
            id
        );
        assert_eq!(g.g_fail, expected_fail);
        assert_eq!(g.decompress_src, expected_source);
        record(&mut out, 8, &snapshot(&mut g));
        record(&mut out, 9, &trace);
        out
    }));
    match result {
        Ok(v) => Some(v),
        Err(_) => {
            eprintln!("background case {id} assertion failed");
            None
        }
    }
}

// No movie has hand-authored planes and palette; both actual consumers must meet literal BGRA.
fn raster_pixels(id: u32) {
    let mut p = raster_fixture();
    p.screenEnabled[0] = 7;
    p.layer
        .iter_mut()
        .take(3)
        .for_each(|l| l.mainScreenEnabled = true);
    for i in 0..3 {
        p.bgLayer[i].tilemapAdr = 0x1000 + i as u16 * 0x400;
        p.bgLayer[i].tileAdr = i as u16 * 0x100;
    }
    p.cgram[1] = 31;
    p.cgram[2] = 31 << 5;
    p.cgram[3] = 31 << 10;
    if id == 38 {
        for base in [0x1000, 0x1400, 0x1800] {
            p.vram[base..base + 0x400].fill(2);
            p.vram[base] = 0;
        }
        p.vram[1] = 0x80;
        p.vram[0x101] = 0x8000;
        p.vram[0x201] = 0x8080;
        p.vram[0x1800] = 0x2000;
    } else {
        p.mode = 7;
        p.screenEnabled[0] = 1;
        p.m7matrix = [256, 0, 0, 256, 0x1fff, 0x1fff, 0, 0];
        p.vram.fill(0);
        p.vram[0] = 1;
        p.vram[72] = 0x0100;
        p.vram[73] = 0x0200;
        p.vram[74] = 0x0300;
    }
    let mut out = [0x5a; 1024];
    p.renderBuffer = out.as_mut_ptr();
    let expected = if id == 38 {
        [[255, 0, 0, 0], [0; 4], [0; 4]]
    } else {
        [[0, 0, 255, 0], [0, 255, 0, 0], [255, 0, 0, 0]]
    };
    let prepared = raster_prepared(&p);
    unsafe {
        crate::snes::ppu::ppu_run_line(&mut *p, 1, &true);
    }
    for (x, pixel) in expected.iter().enumerate() {
        assert_eq!(&out[x * 4..x * 4 + 4], pixel);
        assert_eq!(&prepared.pixels[x], pixel);
    }
    assert_eq!(&out[12..], &[0; 1012]);
    assert!(prepared.pixels[3..].iter().all(|p| *p == [0; 4]));
    if id == 39 {
        p.m7largeField = true;
        p.m7matrix[6] = 0x1000;
        let clipped = raster_prepared(&p);
        assert_eq!(clipped.pixels[0], [0; 4]);
        p.m7charFill = true;
        p.vram[8] = 0x0200;
        assert_eq!(raster_prepared(&p).pixels[0], [0, 255, 0, 0]);
    }
}
// No movie reaches exact 32/33-object and 34/35-tile boundaries or signed workspace aliases.
fn raster_sprites() {
    for (size, count, range, time) in [
        (0, 32, false, false),
        (0, 33, true, false),
        (3, 17, false, false),
        (3, 18, false, true),
    ] {
        let mut p = raster_fixture();
        p.objSize = size;
        p.vram[0] = 0xff;
        for i in 0..count {
            p.oam[i * 2] = 0;
        }
        let mut out = [0x5a; 1024];
        p.renderBuffer = out.as_mut_ptr();
        let prepared = raster_prepared(&p);
        unsafe {
            crate::snes::ppu::ppu_run_line(&mut *p, 1, &true);
        }
        assert_eq!(
            (p.rangeOver, p.timeOver, p.lineHasSprites),
            (range, time, true)
        );
        assert_eq!(p.objBuffer.data[..8], [0x2681; 8]);
        assert_eq!(prepared.sprites.pixels[..8], [0x2681; 8]);
        assert_eq!(
            (
                prepared.sprites.overflow.range,
                prepared.sprites.overflow.time
            ),
            (range, time)
        );
    }
    let mut p = raster_fixture();
    p.oam[0] = 0;
    p.vram[0] = 0xff;
    p.screenEnabled[0] = 16;
    p.extraLeftCur = 1;
    p.extraRightCur = 2;
    p.extraLeftRight = 1;
    p.bgBuffers[1].data[255] = 0x2681;
    p.renderPitch = 0x2682;
    p.cgram[129] = 31;
    p.cgram[130] = 31 << 10;
    p.cgram[131] = 31 << 5;
    // x=256 aliases initialized alignment padding; x=257 aliases renderPitch.
    let padding = core::mem::offset_of!(crate::types::Ppu, objBuffer)
        + core::mem::size_of::<crate::types::PpuPixelPrioBufs>();
    assert_eq!(
        core::mem::offset_of!(crate::types::Ppu, renderPitch),
        padding + 2
    );
    unsafe {
        (&raw mut *p)
            .cast::<u8>()
            .add(padding)
            .cast::<u16>()
            .write_unaligned(0x2683);
    }
    let mut out = [0x5a; 1036];
    p.renderBuffer = out.as_mut_ptr();
    unsafe {
        crate::snes::ppu::ppu_run_line(&mut *p, 1, &true);
    }
    assert_eq!(&out[..4], &[0, 0, 255, 0]);
    assert_eq!(&out[1028..1032], &[0, 255, 0, 0]);
    assert_eq!(&out[1032..], &[255, 0, 0, 0]);
    p.screenEnabled[0] = 0;
    unsafe {
        crate::snes::ppu::ppu_run_line(&mut *p, 1, &true);
    }
    assert_eq!(&out[..4], &[0; 4]);
    assert_eq!(&out[1028..1032], &[0; 4]);
    assert_eq!(&out[1032..], &[0; 4]);
}
// No movie forces a lazy media failure/unwind after first span or reads the last native VRAM tail word.
fn raster_failures() {
    use crate::rendering::*;
    let mut p = raster_fixture();
    p.oam[0] = 0;
    p.oam[2] = 8;
    let input = SpriteInput {
        words: &p.oam,
        extension: &p.highOam,
        tile_words: [0; 2],
        size: 0,
        rotation: None,
        interlace: false,
        even_frame: false,
    };
    for panic in [false, true] {
        let source = RasterReads {
            reads: Default::default(),
            stop: 3,
            panic,
        };
        let mut pixels = [BACKDROP; 256];
        let result = catch_unwind(AssertUnwindSafe(|| {
            evaluate_sprites(
                &source,
                &input,
                0,
                &mut pixels,
                Overflow {
                    range: true,
                    time: false,
                },
            )
        }));
        if panic {
            assert!(result.is_err());
        } else {
            assert_eq!(result.unwrap(), Err(RasterFault::Word(0)));
        }
        assert_eq!(&*source.reads.borrow(), &[0, 8, 0]);
        assert_eq!(pixels[..8], [0x2681; 8]);
        assert_eq!(pixels[8..], [BACKDROP; 248]);
    }
    let mut p = raster_fixture();
    p.screenEnabled[0] = 1;
    p.bgLayer[0].tilemapAdr = 0x7fff;
    p.bgLayer[0].hScroll = 248;
    p.bgLayer[0].tileAdr = 0x100;
    p.cgram[25] = 1;
    p.cgram[1] = 31;
    p.vram[0x111] = 0xff;
    let mut out = [0x5a; 1024];
    p.renderBuffer = out.as_mut_ptr();
    unsafe {
        crate::snes::ppu::ppu_run_line(&mut *p, 1, &true);
    }
    assert_eq!(&out[..32], &[0, 0, 255, 0].repeat(8)[..]);
    p.objSize = 8;
    p.lineHasSprites = true;
    p.rangeOver = true;
    out.fill(0x5a);
    let result = catch_unwind(AssertUnwindSafe(|| unsafe {
        crate::snes::ppu::ppu_run_line(&mut *p, 1, &true)
    }));
    let payload = result.unwrap_err();
    assert_eq!(
        payload.downcast_ref::<String>().map(String::as_str),
        Some("PPU raster fault: SpriteSize(8)")
    );
    assert!(p.lineHasSprites && p.rangeOver && !p.timeOver);
    assert_eq!(p.objBuffer.data, [BACKDROP; 256]);
    assert_eq!(out, [0x5a; 1024]);
}
// No movie isolates EXTBG/mosaic dispatch, duplicate logic or native packed fixed-color overflow.
fn raster_fallback() {
    let mut p = raster_fixture();
    p.cgram[0] = 1;
    p.mathEnabled[5] = true;
    p.fixedColorR = 32;
    p.fixedColorG = 1;
    let mut out = [0x5a; 1024];
    p.renderBuffer = out.as_mut_ptr();
    for (native, expected) in [(true, [0, 8, 8, 0]), (false, [0, 8, 255, 0])] {
        unsafe {
            crate::snes::ppu::ppu_run_line(&mut *p, 1, &native);
        }
        assert_eq!(&out[..4], &expected);
    }
    p.fixedColorR = 0;
    p.fixedColorG = 0;
    p.cgram[0] = 31;
    p.clipMode = 2;
    p.windowsel = 10 << 20;
    p.window1left = 0;
    p.window1right = 1;
    p.window2left = 1;
    p.window2right = 2;
    p.windowLayer[5].window1enabled = true;
    p.windowLayer[5].window2enabled = true;
    p.windowLayer[5].maskLogic = 1;
    for (native, expected) in [(true, [0; 4]), (false, [0, 0, 255, 0])] {
        unsafe {
            crate::snes::ppu::ppu_run_line(&mut *p, 1, &native);
        }
        assert_eq!(&out[..4], &expected);
        assert_eq!(&out[4..8], &[0; 4]);
    }
    // BG1's high-priority 0x83 texel selects red3 ahead of blue131 in EXTBG.
    p.mode = 7;
    p.screenEnabled[0] = 3;
    p.layer[0].mainScreenEnabled = true;
    p.layer[1].mainScreenEnabled = true;
    p.m7matrix = [256, 0, 0, 256, 0, 0, 0, 0];
    p.vram[0] = 1;
    p.vram[72] = 0x8300;
    p.cgram[0] = 0;
    p.cgram[3] = 31;
    p.cgram[0x83] = 31 << 10;
    for (extbg, expected) in [(false, [0; 4]), (true, [0, 0, 255, 0])] {
        p.m7extBg = extbg;
        unsafe {
            crate::snes::ppu::ppu_run_line(&mut *p, 1, &true);
        }
        assert_eq!(&out[..4], &expected);
        assert_eq!(&out[4..8], &[0; 4]);
    }
    // Adjacent texels 1,2,3,0 become 1,1,3,3 only through enabled size2 mosaic.
    p.mode = 1;
    p.screenEnabled[0] = 1;
    p.layer[1].mainScreenEnabled = false;
    p.bgLayer[0].tilemapAdr = 0x1000;
    p.mosaicSize = 2;
    p.clipMode = 0;
    p.vram.fill(0);
    p.vram[1] = 0x60a0;
    p.cgram[1] = 31;
    p.cgram[2] = 31 << 5;
    p.cgram[3] = 31 << 10;
    let red: [u8; 4] = [0, 0, 255, 0];
    let blue = [255, 0, 0, 0];
    p.bgLayer[0].mosaicEnabled = true;
    p.mosaicEnabled = 1;
    unsafe {
        crate::snes::ppu::ppu_run_line(&mut *p, 1, &true);
    }
    for (pixel, expected) in out.chunks_exact(4).zip([red, red, blue, blue]) {
        assert_eq!(pixel, &expected);
    }
    p.bgLayer[0].mosaicEnabled = false;
    p.mosaicEnabled = 0;
    p.forcedBlank = true;
    unsafe {
        crate::snes::ppu::ppu_run_line(&mut *p, 1, &true);
    }
    assert_eq!(out, [0; 1024]);
    assert!(!p.lineHasSprites);
    p.rangeOver = true;
    p.timeOver = true;
    let even = p.evenFrame;
    unsafe {
        crate::snes::ppu::ppu_run_line(&mut *p, 0, &true);
    }
    assert_eq!(
        (p.rangeOver, p.timeOver, p.evenFrame, p.mosaicStartLine),
        (false, false, !even, 1)
    );
}
// No movie supplies i32::MAX scanline; preserve the active profile's signed-overflow behavior.
fn raster_panic_prefix() {
    let overflow = catch_unwind(|| std::hint::black_box(i32::MAX) + std::hint::black_box(1i32));
    if let Ok(value) = &overflow {
        assert_eq!(*value, i32::MIN);
    }
    let mut p = raster_fixture();
    p.screenEnabled[0] = 1;
    p.bgLayer[0].vScroll = 1;
    p.bgLayer[0].tilemapAdr = 0x1000;
    p.vram[0] = 0xff;
    p.cgram[1] = 31;
    p.lineHasSprites = true;
    p.rangeOver = true;
    p.objBuffer.data.fill(0x1234);
    let mut out = [0x5a; 1024];
    p.renderBuffer = out.as_mut_ptr();
    p.renderPitch = 0;
    let result = catch_unwind(AssertUnwindSafe(|| unsafe {
        crate::snes::ppu::ppu_run_line(&mut *p, i32::MAX, &true)
    }));
    if overflow.is_err() {
        assert!(result.is_err());
        assert_eq!(out, [0x5a; 1024]);
    } else {
        assert!(result.is_ok());
        // Wrapped y is i32::MIN: row zero of tile zero is opaque red at every x.
        assert!(out.chunks_exact(4).all(|pixel| pixel == [0, 0, 255, 0]));
    }
    assert_eq!(p.lastBrightnessMult, 15);
    assert_eq!(p.brightnessMult[31], 255);
    assert_eq!(p.objBuffer.data, [0x0500; 256]);
    assert!(!p.lineHasSprites);
    assert!(p.rangeOver);
    assert!(!p.timeOver);
}
pub(crate) fn raster_capture_case(id: u32) -> Option<Vec<u8>> {
    catch_unwind(AssertUnwindSafe(|| {
        match id {
            38 | 39 => raster_pixels(id),
            40 => raster_sprites(),
            41 => raster_failures(),
            42 => raster_fallback(),
            43 => raster_panic_prefix(),
            _ => return None,
        }
        let mut out = b"RMPX1".to_vec();
        out.extend_from_slice(&id.to_le_bytes());
        Some(out)
    }))
    .unwrap_or(None)
}
