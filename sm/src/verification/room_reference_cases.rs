//! Frozen synthetic inputs for actual room predecessor execution. No ROM assets.
use crate::room_background_support::{ram_bytes, ram_bytes_parts};
use crate::{
    game_state::{Game, WorkRam},
    rom::RomImage,
};
use std::panic::{AssertUnwindSafe, catch_unwind};
pub(crate) const CASES: u32 = 36;
const DEFAULT: u32 = 9430502;
const EVENT: u32 = 9430546;
const BOSS: u32 = 9430569;
const BOSS1: u32 = 9430527;
const MORPH: u32 = 9430610;
const POWER: u32 = 9430633;
#[derive(Clone, Copy)]
pub(crate) enum Operation {
    Entry(u32, u16),
    Select(u16),
    Header,
    State,
    Door,
}
pub(crate) fn put(g: &mut Game, at: usize, v: u16) {
    g.ram().copy_in(at, &v.to_le_bytes());
}
fn exp(e: &mut [u8], at: usize, v: u16) {
    e[at..at + 2].copy_from_slice(&v.to_le_bytes());
}
fn offset(a: u32) -> usize {
    crate::rtl::lorom_offset(a) as usize
}
pub(crate) fn bytes(r: &RomImage, a: u32, v: &[u8]) {
    for (i, b) in v.iter().enumerate() {
        r.write(offset(a) + i, *b);
    }
}
fn rw(r: &RomImage, a: u32, v: u16) {
    bytes(r, a, &v.to_ne_bytes());
}
fn header(r: &RomImage, a: u32) {
    bytes(r, a, &[7, 1, 3, 4, 17, 18, 5, 6, 0, 0x34, 0x92]);
}
fn state(r: &RomImage) {
    bytes(
        r,
        0x8f9000,
        &[
            0x12, 0x34, 0x91, 0, 0x23, 0x45, 0x11, 0x81, 0x22, 0x82, 0x33, 0x83, 0x56, 0x78, 0, 0,
            0, 0, 0x44, 0x84,
        ],
    );
}
const ASSETS: [u8; 9] = [0x21, 0x43, 0xa1, 0x65, 0x87, 0xb2, 0xa9, 0xcb, 0xc3];
pub(crate) fn snapshot(out: &mut Vec<u8>, g: &Game) {
    snapshot_parts(out, g.ram(), &g.g_fail);
}
pub(crate) fn snapshot_parts(out: &mut Vec<u8>, ram: &WorkRam, fail: &bool) {
    out.extend_from_slice(&ram_bytes_parts(ram));
    out.push(u8::from(*fail));
}
fn expect_header(e: &mut [u8]) {
    for (a, v) in [
        (0x79d, 7),
        (0x79f, 1),
        (0x7a1, 3),
        (0x7a3, 4),
        (0x7a5, 272),
        (0x7a7, 288),
        (0x7a9, 17),
        (0x7ab, 18),
        (0x7ad, 5),
        (0x7af, 6),
        (0x7b5, 0x9234),
        (0x7b9, 1024),
    ] {
        exp(e, a, v);
    }
}
fn expect_state(e: &mut [u8]) {
    e[0x7c0..0x7c9].copy_from_slice(&ASSETS);
    e[0x7bd..0x7c0].copy_from_slice(&[0x12, 0x34, 0x91]);
    for (a, v) in [
        (0x7cb, 0x23),
        (0x7c9, 0x45),
        (0x7cd, 0x8111),
        (0x7cf, 0x8222),
        (0x7d1, 0x8333),
        (0x91b, 0x7856),
        (0x7df, 0x8444),
    ] {
        exp(e, a, v);
    }
}
pub(crate) fn capture_case(
    id: u32,
    run: fn(Operation, &mut Game) -> Option<u16>,
) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let length = match id {
            13 | 14 => offset(0x8f9000) + 1,
            23 => offset(0x8f9000) + 2,
            25 => offset(0x8fffff) + 3,
            26 => offset(0x8fe7a7) + 1,
            31 => offset(0x839000) + 3,
            34 => offset(0x8f9010),
            35 => offset(0x8f9012),
            _ => 0x100000,
        };
        let r = RomImage::zeroed(length);
        let mut g = Game::new();
        for i in 0..g.ram().len() {
            g.ram()
                .write_byte(i, (i.wrapping_mul(37) ^ (i >> 8) ^ 0x5a) as u8);
        }
        g.loaded_rom = Some(r.clone());
        g.g_fail = false;
        g.VERIFY_ACTIVE = true;
        put(&mut g, 0x5e7, 0xa534);
        put(&mut g, 0x79f, 0);
        put(&mut g, 0x7bb, 0xbeef);
        put(&mut g, 0x79b, 0x9000);
        put(&mut g, 0x78d, 0x9000);
        put(&mut g, 0x9a4, 0);
        put(&mut g, 0x9c8, 0);
        put(&mut g, 0x9d0, 0);
        g.ram().write_byte(0xd820, 0);
        g.ram().write_byte(0xd828, 0);
        g.ram().write_byte(0xd829, 0);
        let mut op = Operation::Entry(DEFAULT, 0x9000);
        let mut return_value = Some(0);
        let mut panics = false;
        let mut selected = None;
        let mut scratch = None;
        match id {
            0 => {
                g.loaded_rom = None;
                selected = Some(0x9000);
            }
            1 => {
                g.loaded_rom = None;
                op = Operation::Entry(DEFAULT, 0);
                selected = Some(0);
            }
            2..=7 => {
                let code = if id <= 3 {
                    EVENT
                } else if id <= 5 {
                    BOSS
                } else {
                    BOSS1
                };
                let taken = id % 2 == 0;
                op = Operation::Entry(code, 0x9000);
                if code == BOSS1 {
                    rw(&r, 0x8f9000, 0xabcd);
                    scratch = Some(0xa501);
                    if taken {
                        g.ram().write_byte(0xd828, 1);
                    }
                } else {
                    bytes(&r, 0x8f9000, &[if code == EVENT { 6 } else { 4 }]);
                    rw(&r, 0x8f9001, 0xabcd);
                    scratch = Some(if code == EVENT { 0x40 } else { 0xa504 });
                    if taken {
                        g.ram().write_byte(
                            if code == EVENT { 0xd820 } else { 0xd828 },
                            if code == EVENT { 0x40 } else { 4 },
                        );
                    }
                }
                if taken {
                    selected = Some(0xabcd);
                } else {
                    return_value = Some(if code == BOSS1 { 0x9002 } else { 0x9003 });
                }
            }
            8..=10 => {
                op = Operation::Entry(MORPH, 0x9000);
                put(&mut g, 0x9a4, if id == 8 { 0 } else { 4 });
                put(&mut g, 0x9c8, if id == 9 { 0 } else { 5 });
                if id == 10 {
                    rw(&r, 0x8f9000, 0xabcd);
                    selected = Some(0xabcd);
                } else {
                    g.loaded_rom = None;
                    return_value = Some(0x9002);
                }
            }
            11 | 12 => {
                op = Operation::Entry(POWER, 0x9000);
                if id == 12 {
                    put(&mut g, 0x9d0, 7);
                    rw(&r, 0x8f9000, 0xabcd);
                    selected = Some(0xabcd);
                } else {
                    g.loaded_rom = None;
                    return_value = Some(0x9002);
                }
            }
            13 | 14 => {
                op = Operation::Entry(EVENT, 0x9000);
                bytes(&r, 0x8f9000, &[6]);
                scratch = Some(0x40);
                if id == 14 {
                    g.ram().write_byte(0xd820, 0x40);
                    panics = true;
                } else {
                    return_value = Some(0x9003);
                }
            }
            15 | 17 => {
                op = Operation::Entry(BOSS1, if id == 15 { 0x9000 } else { 0xfffe });
                g.loaded_rom = None;
                scratch = Some(0xa501);
                return_value = Some(if id == 15 { 0x9002 } else { 0 });
            }
            16 | 18 => {
                let cursor = if id == 16 { 0xfffd } else { 0xffff };
                op = Operation::Entry(EVENT, cursor);
                bytes(&r, 0x8f0000 | u32::from(cursor), &[6]);
                scratch = Some(0x40);
                return_value = Some(if id == 16 { 0 } else { 2 });
            }
            19 => {
                op = Operation::Select(0x9000);
                return_value = None;
                let mut a = 0x8f900b;
                for code in [EVENT, BOSS, BOSS1, MORPH, POWER] {
                    rw(&r, a, code as u16);
                    a += 2;
                    if code == EVENT || code == BOSS {
                        bytes(&r, a, &[6]);
                        rw(&r, a + 1, 0xabcd);
                        a += 3;
                    } else {
                        rw(&r, a, 0xabcd);
                        a += 2;
                    }
                }
                rw(&r, a, DEFAULT as u16);
                selected = Some((a + 2) as u16);
                scratch = Some(1);
            }
            20 => {
                op = Operation::Select(0xfff3);
                return_value = None;
                rw(&r, 0x8ffffe, DEFAULT as u16);
                selected = Some(0);
            }
            21 | 22 => {
                op = Operation::Header;
                return_value = None;
                header(&r, 0x8f9000);
                rw(&r, 0x8f900b, BOSS1 as u16);
                rw(&r, 0x8f900d, 0xabcd);
                rw(&r, 0x8f900f, DEFAULT as u16);
                g.ram().write_byte(0xd828, if id == 21 { 1 } else { 0 });
                g.ram().write_byte(0xd829, if id == 22 { 1 } else { 0 });
                scratch = Some(0xa501);
                selected = Some(if id == 21 { 0x9011 } else { 0xabcd });
            }
            23 => {
                op = Operation::Header;
                return_value = None;
                bytes(&r, 0x8f9000, &[7, 1]);
                panics = true;
            }
            24..=26 | 32 => {
                op = Operation::State;
                return_value = None;
                put(&mut g, 0x7bb, 0x9000);
                state(&r);
                if id != 26 {
                    rw(
                        &r,
                        0x8fe7a7,
                        if id == 25 {
                            0xffff
                        } else if id == 32 {
                            0x10
                        } else {
                            0xa000
                        },
                    );
                }
                if id == 24 {
                    bytes(&r, 0x8fa000, &ASSETS);
                }
                if id == 25 {
                    bytes(&r, 0x8fffff, &ASSETS[..3]);
                }
                panics = id == 25 || id == 26;
            }
            27..=31 => {
                op = Operation::Door;
                return_value = None;
                if id == 31 {
                    bytes(&r, 0x839000, &[0x34, 0x92, 0x80]);
                    panics = true;
                } else {
                    let dir = if id == 28 { 2 } else { 0 };
                    bytes(&r, 0x839000, &[0x34, 0x92, 0x80, dir, 0x55, 0x66, 7, 9]);
                    rw(
                        &r,
                        0x839008,
                        if id <= 28 {
                            0xffff
                        } else if id == 29 {
                            0x123
                        } else {
                            0
                        },
                    );
                }
            }
            33 => {
                op = Operation::Header;
                return_value = None;
                put(&mut g, 0x79b, 0x10);
                selected = Some(0xa00d);
            }
            34 => {
                op = Operation::Select(0x9000);
                return_value = None;
                rw(&r, 0x8f900b, EVENT as u16);
                bytes(&r, 0x8f900d, &[6]);
                rw(&r, 0x8f900e, 0xabcd);
                g.ram().write_byte(0xd820, 0x40);
                scratch = Some(0x40);
                selected = Some(0xabcd);
            }
            35 => {
                op = Operation::Select(0x9000);
                return_value = None;
                rw(&r, 0x8f900b, EVENT as u16);
                bytes(&r, 0x8f900d, &[6]);
                rw(&r, 0x8f900e, 0xabcd);
                rw(&r, 0x8f9010, BOSS as u16);
                scratch = Some(0x40);
                panics = true;
            }
            _ => unreachable!(),
        }
        let mut expected = ram_bytes(&g);
        if let Some(v) = selected {
            exp(&mut expected, 0x7bb, v);
        }
        if let Some(v) = scratch {
            exp(&mut expected, 0x5e7, v);
        }
        match id {
            21 | 22 | 33 => expect_header(&mut expected),
            23 => {
                exp(&mut expected, 0x79d, 7);
                exp(&mut expected, 0x79f, 1);
            }
            24 | 32 => expect_state(&mut expected),
            25 => expected[0x7c0..0x7c3].copy_from_slice(&ASSETS[..3]),
            27..=31 => {
                exp(&mut expected, 0x79b, 0x9234);
                if id != 31 {
                    let dir = if id == 28 { 2 } else { 0 };
                    for (a, v) in [
                        (0x793, 0x80 | (dir << 8)),
                        (0xe16, 0x80),
                        (0x791, dir),
                        (0x927, 0x700),
                        (0x929, 0x900),
                    ] {
                        exp(&mut expected, a, v);
                    }
                    let distance = match id {
                        27 => 200u32,
                        28 => 384,
                        29 => 0x123,
                        _ => 0,
                    };
                    exp(&mut expected, 0x92b, (distance << 8) as u16);
                    exp(&mut expected, 0x92d, (distance >> 8) as u16);
                }
            }
            _ => (),
        }
        if id == 32 {
            exp(&mut expected, 0x7bb, 0xb000);
        }
        if id == 33 {
            exp(&mut expected, 0x79b, 0xa000);
        }
        let mut out = b"ROOM1".to_vec();
        out.extend_from_slice(&id.to_le_bytes());
        snapshot(&mut out, &g);
        crate::room_reference_observer::begin(id);
        let actual = catch_unwind(AssertUnwindSafe(|| run(op, &mut g)));
        let observations = crate::room_reference_observer::finish();
        match actual {
            Ok(value) => {
                assert!(!panics, "expected checked failure");
                assert_eq!(value, return_value);
                match value {
                    None => out.push(0),
                    Some(v) => {
                        out.push(1);
                        out.extend_from_slice(&v.to_le_bytes());
                    }
                }
            }
            Err(payload) => {
                assert!(panics, "unexpected panic");
                out.push(2);
                let (kind, message) = if let Some(s) = payload.downcast_ref::<String>() {
                    (1, s.as_str())
                } else if let Some(s) = payload.downcast_ref::<&str>() {
                    (2, *s)
                } else {
                    (3, "non-string panic")
                };
                let expected_message = if id == 26 {
                    "ROM read outside loaded allocation".to_owned()
                } else {
                    // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
                    format!("index out of bounds: the len is {length} but the index is {length}")
                };
                assert_eq!(
                    kind,
                    if id == 26 { 2 } else { 1 },
                    "checked panic payload type"
                );
                assert_eq!(message, expected_message, "checked panic payload text");
                out.push(kind);
                out.extend_from_slice(&(message.len() as u32).to_le_bytes());
                out.extend_from_slice(message.as_bytes());
            }
        }
        assert_eq!(
            &ram_bytes(&g),
            &expected[..],
            "room {id} complete expected RAM"
        );
        assert_eq!(g.g_fail, id == 32 || id == 33);
        snapshot(&mut out, &g);
        out.extend_from_slice(&(observations.len() as u32).to_le_bytes());
        out.extend(observations);
        out
    }));
    match result {
        Ok(v) => Some(v),
        Err(_) => {
            eprintln!("room reference case {id} failed assertions");
            None
        }
    }
}
pub(crate) fn run(op: Operation, g: &mut Game) -> Option<u16> {
    use sm_game::room_entry::content::{SelectorCursor, SelectorKind};
    use sm_game::room_entry::{installation, selection};
    let mut h = crate::game::session_runtime::Runtime::new(g);
    match op {
        Operation::Entry(a, c) => {
            let kind = SelectorKind::from_legacy(a);
            match selection::select_entry(&mut h, kind, SelectorCursor::Legacy(c)) {
                Some(SelectorCursor::Legacy(next)) => Some(next),
                None => Some(0),
                Some(SelectorCursor::Prepared(_)) => {
                    unreachable!("legacy selector returned authored cursor")
                }
            }
        }
        Operation::Select(r) => {
            let room = h.session().world.room_identity(r);
            selection::select(&mut h, room);
            None
        }
        Operation::Header => {
            installation::load_header(&mut h);
            None
        }
        Operation::State => {
            installation::load_state(&mut h);
            None
        }
        Operation::Door => {
            installation::load_door(&mut h);
            None
        }
    }
}
#[cfg(sm_verify_dll)]
pub(crate) fn replacement(id: u32) -> RomImage {
    let r = RomImage::zeroed(0x100000);
    if id == 32 {
        bytes(&r, 0x8f0010, &ASSETS);
        bytes(&r, 0x8f9000, &[0xee; 20]);
    } else {
        header(&r, 0x8f0010);
        rw(&r, 0x8f001b, DEFAULT as u16);
        rw(&r, 0x8fa00b, DEFAULT as u16);
    }
    r
}
