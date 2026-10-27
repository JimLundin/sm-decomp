//! Frozen inputs/expected effects for actual background and installation entries.
use crate::room_background_support::ram_bytes;
use crate::{game_state::Game, rom::RomImage, room_background_support::*};
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
            use crate::game::{
                room_entry::{background, content::LoadMode},
                session_runtime::Runtime,
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
        Operation::Level(true) => {
            crate::game::room_state::load_level::<true>(g);
            None
        }
        Operation::Level(false) => {
            crate::game::room_state::load_level::<false>(g);
            None
        }
        Operation::Tiles => {
            crate::game::room_state::load_tiles_and_palette(g);
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
fn transfer(r: &RomImage, at: u16, opcode: u16) {
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
fn state(r: &RomImage, scroll: u16) {
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
        let a = RomImage::zeroed(len);
        let b = RomImage::zeroed(0x400000);
        g.loaded_rom = Some(a.clone());
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
        state(&a, 0);
        state(&b, 0);
        if id < 24 {
            // Selected-state background defaults to0; command cases override it.
            if !matches!(id, 0 | 1 | 8 | 9 | 20 | 21) {
                rw(
                    &a,
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
                    transfer(&a, 0xa000, if id == 16 { 9 } else { 3 });
                    rw(&a, 0x8fa009, if id == 4 || id == 5 { 16 } else { 0 });
                    rw(&b, 0x8fa009, 0);
                }
                6 | 7 => {
                    rw(&a, 0x8feffe, 2);
                    rw(&a, 0x8f9016, 0xeffe);
                    bytes(&a, 0x8ff000, &[0, 0x60, 0x7e, 0x23, 0x31]);
                    panic_text = Some("ROM read outside loaded allocation".into());
                }
                8 | 9 => {
                    put(&mut g, 0x7bb, 0x10);
                    bytes(&a, 0x8f0010, &[0; 26]);
                    if id == 8 {
                        diagnostics.push(0x8f0010);
                    } else {
                        panic_text = Some("ROM address must select a LoROM window".into());
                    }
                }
                10 | 11 => transfer(&a, 0xfff7, 2),
                12 => rw(&a, 0x8fa000, 1),
                13 => {
                    rw(&a, 0x8ffff5, 14);
                    rw(&a, 0x8ffff7, 0x8123);
                }
                14 => {
                    rw(&a, 0x8ffffe, 14);
                    rw(&a, 0x8f0000, 0x8000);
                    diagnostics.push(0x8f0000);
                    panic_text = Some("ROM address must select a LoROM window".into());
                }
                15 => {
                    rw(&a, 0x8fa000, 5);
                    bytes(&a, 0x8fa002, &[0, 0xa0, 0x80, 0, 0x60]);
                    literal(&a, 0x80a000, &[0xe1, 0xe2]);
                    rw(&a, 0x8fa007, 0);
                    expected_source = 0x80a004;
                }
                17 | 18 => {
                    rw(&a, 0x8fa000, 12);
                    rw(&a, 0x8fa002, 0);
                }
                19 => {
                    rw(&a, 0x8fa000, 16);
                    panic_kind = 1;
                    panic_text = Some("invalid room background command: 0x10".into());
                }
                20 | 21 => put(&mut g, 0x1964, 0x9000),
                22 | 23 => {
                    transfer(&a, 0xfff9, 2);
                    rw(&a, 0x8f0002, 0);
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
            literal(&a, 0x808000, &[2, 0, 0xa1, 0xa2, 0xb1, 0xc1, 0xc2]);
            literal(&a, 0x809000, &[0xd1, 0xd2]);
            expected_source = 0x809004;
            if matches!(id, 26 | 27 | 29) {
                rw(&a, 0x8f900e, 0xfffe);
                rw(&a, 0x8ffffe, 0x1100);
                for i in 0..24 {
                    rw(&a, 0x8f0000 + 2 * i, 0x1101 + i as u16);
                    rw(&b, 0x8f0000 + 2 * i, 0x2201 + i as u16);
                    diagnostics.push(0x8f0000 + 2 * i);
                }
                if id == 29 {
                    rw(&a, 0x8f9014, 0x40);
                    rw(&b, 0x8f0040, 0);
                    diagnostics.push(0x8f0040);
                    bytes(&b, 0x8f0080, &[0; 26]);
                    diagnostics.push(0x8f0080);
                }
            }
            if id == 28 {
                rw(&a, 0x8f900e, 0xfff0);
                for i in 0..3 {
                    rw(&a, 0x8ffff0 + 2 * i, 0x3300 + i as u16);
                }
                bytes(&a, 0x8ffff6, &[0x44]);
                panic_kind = 1;
                panic_text = Some(format!(
                    "index out of bounds: the len is {len} but the index is {len}"
                ));
            }
            if id >= 30 {
                put(&mut g, 0x79f, 0);
                put(&mut g, 0x7b3, if id == 31 { 2 } else { 0 });
                literal(&a, 0xb9a09d, &[0xc3, 0xc4]);
            }
        } else {
            asset(&mut g, 0x7c3, 0x808000);
            asset(&mut g, 0x7c6, 0x809000);
            literal(&a, 0xb98000, &[0x91, 0x92]);
            literal(&a, 0x808000, &[0xa1, 0xa2]);
            literal(&a, 0x809000, &[0xb1, 0xb2]);
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
                    direct(
                        &mut writes,
                        0x3123,
                        if id == 4 { 0x6200 } else { 0x6000 },
                        if id == 4 { 0x777e } else { 0x237e },
                        if id == 6 {
                            None
                        } else {
                            Some(if id == 4 { 8 } else { 4 })
                        },
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
                    let start = if id == 4 { 0x6200 } else { 0x6000 };
                    let n = if id == 4 { 8 } else { 4 };
                    vram_bytes(&mut vram, 0x3123, &expected[start..start + n]);
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
        crate::room_background_observer::begin(id, writes, diagnostics, a, b, mutations);
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
