//! Frozen synthetic actual contact/start predecessor calls; no ROM-derived assets.
use crate::room_background_support::{ram_bytes, ram_bytes_parts};
use crate::{
    game_state::{Game, WorkRam},
    rom::RomImage,
};
use std::panic::{AssertUnwindSafe, catch_unwind};
pub(crate) const CASES: u32 = 20;
pub(crate) fn offset(address: u32) -> usize {
    crate::rtl::lorom_offset(address) as usize
}
fn bytes(r: &mut [u8], address: u32, value: &[u8]) {
    // Byte by byte, so a short image fails at its first missing byte.
    for (i, &b) in value.iter().enumerate() {
        r[offset(address) + i] = b;
    }
}
fn rw(r: &mut [u8], address: u32, value: u16) {
    bytes(r, address, &value.to_ne_bytes());
}
fn put(g: &mut Game, at: usize, value: u16) {
    g.ram().copy_in(at, &value.to_le_bytes());
}
fn exp(e: &mut [u8], at: usize, value: u16) {
    e[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
pub(crate) fn snapshot(out: &mut Vec<u8>, g: &Game) {
    snapshot_parts(out, g.ram(), &g.g_fail);
}
pub(crate) fn snapshot_parts(out: &mut Vec<u8>, ram: &WorkRam, fail: &bool) {
    out.extend_from_slice(&ram_bytes_parts(ram));
    out.push(u8::from(*fail));
}
#[derive(Clone, Copy)]
pub(crate) enum Operation {
    Right(i32),
    Down(i32),
    Station,
}
pub(crate) fn run(operation: Operation, g: &mut Game) -> Option<i32> {
    {
        match operation {
            Operation::Right(n) => Some(
                crate::game::player_collision::operations::displace(
                    &mut crate::game::session_runtime::Runtime::new(g),
                    crate::game::player_collision::operations::DisplacementRequest::TerrainOnly {
                        axis: crate::game::player_collision::operations::Axis::X,
                        amount: n,
                    },
                )
                .displacement,
            ),
            Operation::Down(n) => Some(
                crate::game::player_collision::operations::displace(
                    &mut crate::game::session_runtime::Runtime::new(g),
                    crate::game::player_collision::operations::DisplacementRequest::TerrainOnly {
                        axis: crate::game::player_collision::operations::Axis::Y,
                        amount: n,
                    },
                )
                .displacement,
            ),
            Operation::Station => {
                sm_game::room_entry::selection::begin(
                    &mut crate::game::session_runtime::Runtime::new(g),
                    crate::game::room_entry::ArrivalRequest::Start(
                        crate::game::room_entry::StartSite::CurrentLoadStation,
                    ),
                );
                None
            }
        }
    }
}
fn station(r: &mut [u8], cursor: u16, room: u16, wrap: bool) {
    let values = [
        room,
        0xa246,
        0x55aa,
        if wrap { 0xfff0 } else { 0x1234 },
        if wrap { 0xfffe } else { 0x2345 },
        if wrap { 5 } else { 0x5678 },
        if wrap { 0x90 } else { 0x6789 },
    ];
    for (i, v) in values.into_iter().enumerate() {
        rw(r, 0x800000 | u32::from(cursor) + i as u32 * 2, v);
    }
}
pub(crate) fn capture_case(
    id: u32,
    run: fn(Operation, &mut Game) -> Option<i32>,
) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        // Native pointer-producing paths always retain a full distinct allocation.
        let mut native = vec![0; 0x100000];
        let mut loaded = vec![
            0;
            if id == 9 {
                offset(0x8f9000) + 1
            } else if id == 17 {
                offset(0x80c4b5) + 1
            } else {
                0x100000
            }
        ];
        let mut g = Game::new();
        for i in 0..g.ram().len() {
            g.ram()
                .write_byte(i, (i.wrapping_mul(29) ^ (i >> 7) ^ 0xa3) as u8);
        }
        // Both images are installed once their bytes are complete.
        let mut loaded_present = true;
        g.g_fail = false;
        g.VERIFY_ACTIVE = true;
        put(&mut g, 0x79f, 0);
        put(&mut g, 0x78b, 0);
        put(&mut g, 0x5f7, 0x5634);
        put(&mut g, 0x99c, 0x7777);
        put(&mut g, 0x998, 8);
        put(&mut g, 0x78d, 0xbeef);
        put(&mut g, 0x1e75, 0x2468);
        let mut operation = Operation::Station;
        let mut returned = None;
        let panics = matches!(id, 8 | 9 | 10 | 16 | 17);
        let diagnostic = matches!(id, 6 | 7 | 10 | 15 | 19);
        let mut expected_door = 0xa000;
        let mut expected_area = 7u8;
        if id <= 10 {
            operation = if id == 3 {
                Operation::Down(65536)
            } else {
                Operation::Right(if id == 4 { -65536 } else { 65536 })
            };
            returned = Some(if matches!(id, 1 | 2 | 4) { 0 } else { 65536 });
            for (at, v) in [
                (0xaf6, 8),
                (0xaf8, 0),
                (0xafa, 8),
                (0xafc, 0),
                (0xafe, 1),
                (0xb00, 1),
                (0x7a5, 16),
                (0x5b6, 0),
                (0xa1c, if id == 2 { 9 } else { 8 }),
                (0xe16, 0x5a5a),
                (0x7b5, if id == 6 { 0xfffe } else { 0x9000 }),
            ] {
                put(&mut g, at, v);
            }
            put(&mut g, 0x10002, 0x9000);
            let bts = if id == 5 {
                0x82
            } else if id == 6 {
                1
            } else {
                0
            };
            g.ram().write_byte(0x16402, bts);
            let list_address = if id == 6 {
                0x8f8000
            } else if id == 5 {
                0x8f9004
            } else {
                0x8f9000
            };
            expected_door = if id == 7 || id == 10 { 0x10 } else { 0xa000 };
            if id == 9 {
                bytes(&mut loaded, list_address, &[0]);
            } else {
                rw(&mut loaded, list_address, expected_door);
                rw(
                    &mut loaded,
                    0x838000 | u32::from(expected_door),
                    if matches!(id, 1 | 2 | 4) { 0 } else { 0x9234 },
                );
                if id == 5 {
                    rw(&mut loaded, 0x8f9000, 0xb000);
                    rw(&mut loaded, 0x83b000, 0);
                }
            }
            if id == 8 {
                loaded_present = false;
            }
        } else {
            if id == 13 {
                put(&mut g, 0x79f, 0x0101);
            }
            let area = if id == 13 { 0x0101 } else { 0 };
            let index = if id == 14 {
                3
            } else if id == 15 {
                2
            } else {
                0
            };
            put(&mut g, 0x78b, index);
            let table = if id == 15 {
                0xfffe
            } else if id == 18 {
                0x9100
            } else {
                0x9000
            };
            let cursor = if id == 14 {
                0x902a
            } else if id == 15 {
                0x001a
            } else if id == 18 {
                0x9100
            } else {
                0x9000
            };
            let room = if id == 19 { 0x0010 } else { 0x9234 };
            if id != 17 {
                rw(&mut loaded, 0x80c4b5 + 2 * area, table);
            } else {
                bytes(&mut loaded, 0x80c4b5, &[0]);
            }
            station(&mut native, cursor, room, id == 12);
            // Full packed header allocated; only its source area byte is material here.
            bytes(
                &mut native,
                0x8f0000 | u32::from(room),
                &[0x66, 7, 0x55, 0x44, 1, 2, 3, 4, 5, 6, 7],
            );
            if id == 14 {
                station(&mut native, 0x9000, 0xabcd, false);
            }
            if id == 18 {
                // If table is wrongly read from native, it selects a decoy station.
                rw(&mut native, 0x80c4b5, 0xa000);
                station(&mut native, 0xa000, 0xabcd, false);
                // If station/header is wrongly read from loaded, all result fields differ.
                station(&mut loaded, 0x9100, 0xb123, true);
                bytes(&mut loaded, 0x8fb123, &[0, 0x88, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
                bytes(&mut loaded, 0x8f9234, &[0, 0x99, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            }
            if id == 16 {
                loaded_present = false;
            }
            expected_area = 7;
        }
        g.register_native_rom(RomImage::from(native));
        g.loaded_rom = loaded_present.then(|| RomImage::from(loaded));
        let initial = ram_bytes(&g);
        let mut expected = initial.clone();
        if id <= 10 {
            exp(&mut expected, 0x1e71, 0);
            exp(&mut expected, 0xdc4, 0);
            exp(&mut expected, 0x99c, 0xe17d);
            if id == 3 {
                exp(&mut expected, 0xdba, 0);
            }
            if !panics {
                if matches!(id, 1 | 2 | 4) {
                    if id != 2 {
                        exp(&mut expected, 0xe16, 1);
                    }
                    exp(&mut expected, 0xdd0, 1);
                    exp(&mut expected, 0xaf8, if id == 4 { 0 } else { 0xffff });
                } else {
                    exp(&mut expected, 0x78d, expected_door);
                    exp(&mut expected, 0x998, 9);
                    exp(&mut expected, 0xdd0, 0);
                    exp(&mut expected, if id == 3 { 0xafa } else { 0xaf6 }, 9);
                }
            }
        } else {
            exp(&mut expected, 0x1e75, 1);
            if !panics {
                let room = if id == 19 { 0x0010 } else { 0x9234 };
                let (cx, cy, x, y) = if id == 12 {
                    (0xfff0, 0xfffe, 0x100, 3)
                } else {
                    (0x1234, 0x2345, 0x7a3d, 0x79bd)
                };
                for (at, v) in [
                    (0x79b, room),
                    (0x78d, 0xa246),
                    (0x911, cx),
                    (0x91d, cx),
                    (0x915, cy),
                    (0x91f, cy),
                    (0xafa, y),
                    (0xb14, y),
                    (0xaf6, x),
                    (0xb10, x),
                    (0xb1, 0),
                    (0xb3, 0),
                ] {
                    exp(&mut expected, at, v);
                }
                expected[0x79f] = expected_area;
                expected[0x5f7] = 0;
            }
        }
        let diagnostic_expected = if diagnostic {
            let mut e = if id <= 10 {
                let mut e = initial.clone();
                exp(&mut e, 0x1e71, 0);
                exp(&mut e, 0xdc4, 0);
                exp(&mut e, 0x99c, 0xe17d);
                e
            } else if id == 15 {
                let mut e = initial.clone();
                exp(&mut e, 0x1e75, 1);
                e
            } else {
                let mut e = expected.clone();
                e[0x79f] = initial[0x79f];
                e[0x5f7] = initial[0x5f7];
                e
            };
            e.push(1);
            Some(e)
        } else {
            None
        };
        let mut out = b"RMCS1".to_vec();
        out.extend_from_slice(&id.to_le_bytes());
        snapshot(&mut out, &g);
        crate::room_contact_start_observer::begin(id, diagnostic_expected);
        let actual = catch_unwind(AssertUnwindSafe(|| run(operation, &mut g)));
        let observations = crate::room_contact_start_observer::finish();
        match actual {
            Ok(value) => {
                assert!(!panics, "expected checked panic");
                assert_eq!(value, returned, "actual source return");
                match value {
                    None => out.push(0),
                    Some(v) => {
                        out.push(1);
                        out.extend_from_slice(&v.to_le_bytes());
                    }
                }
            }
            Err(payload) => {
                assert!(panics, "unexpected source panic");
                let (kind, message) = if let Some(s) = payload.downcast_ref::<String>() {
                    (1, s.as_str())
                } else if let Some(s) = payload.downcast_ref::<&str>() {
                    (2, *s)
                } else {
                    (3, "non-string panic")
                };
                let expected_message = match id {
                    8 | 16 => "ROM must be loaded".to_owned(),
                    9 | 10 => "ROM read outside loaded allocation".to_owned(),
                    17 => {
                        let len = offset(0x80c4b5) + 1;
                        // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
                        format!("index out of bounds: the len is {len} but the index is {len}")
                    }
                    _ => unreachable!(),
                };
                assert_eq!(
                    kind,
                    if matches!(id, 8 | 16 | 17) { 1 } else { 2 },
                    "checked panic type"
                );
                assert_eq!(message, expected_message, "checked panic text");
                out.push(2);
                out.push(kind);
                out.extend_from_slice(&(message.len() as u32).to_le_bytes());
                out.extend_from_slice(message.as_bytes());
            }
        }
        assert_eq!(
            &ram_bytes(&g),
            &expected[..],
            "room contact/start {id} full expected RAM"
        );
        assert_eq!(g.g_fail, diagnostic);
        snapshot(&mut out, &g);
        out.extend_from_slice(&(observations.len() as u32).to_le_bytes());
        out.extend(observations);
        out
    }));
    match result {
        Ok(v) => Some(v),
        Err(_) => {
            eprintln!("room contact/start case {id} assertion failed");
            None
        }
    }
}
