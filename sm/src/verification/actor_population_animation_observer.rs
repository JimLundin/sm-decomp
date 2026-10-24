//! Verification-only actual-boundary trace. Dormant outside this fixture carrier.
#![forbid(unsafe_code)]
use crate::actor_population_animation_cases::{get, offset, put, snapshot};
use crate::{game_state::Game, rom::RomImage, types::EnemyDef as D};
use core::mem::offset_of;
use std::cell::RefCell;
struct Capture {
    id: u32,
    events: Vec<u16>,
    bytes: Vec<u8>,
    diagnostics: u32,
    opcodes: u32,
    nested: bool,
}
thread_local! {static ACTIVE:RefCell<Option<Capture>>=const{RefCell::new(None)};}
pub(crate) fn begin(id: u32) {
    ACTIVE.with(|a| {
        assert!(a.borrow().is_none());
        *a.borrow_mut() = Some(Capture {
            id,
            events: Vec::new(),
            bytes: Vec::new(),
            diagnostics: 0,
            opcodes: 0,
            nested: false,
        });
    });
}
pub(crate) fn clear() {
    ACTIVE.with(|a| {
        a.borrow_mut().take();
    });
}
fn append(c: &mut Capture, event: u16, key: u16, address: u32, g: &Game) {
    c.events.push(event);
    c.bytes.extend_from_slice(&event.to_le_bytes());
    c.bytes.extend_from_slice(&key.to_le_bytes());
    c.bytes.extend_from_slice(&address.to_le_bytes());
    c.bytes
        .extend_from_slice(&(g.loaded_rom.as_ref().map_or(0, RomImage::len) as u32).to_le_bytes());
    snapshot(&mut c.bytes, g);
}
pub(crate) fn stage(event: u16, key: u16, g: &mut Game) -> bool {
    ACTIVE.with(|a| {
        let mut active = a.borrow_mut();
        let Some(c) = active.as_mut() else {
            return false;
        };
        append(c, event, key, 0, g);
        if event == 6 {
            assert!(
                matches!(get(g, 0xf78 + usize::from(key)), 0x9000 | 0x9100),
                "initializer sees published definition"
            );
            assert_eq!(
                get(g, crate::ram_map::words::cur_enemy_index),
                key,
                "initializer live selector"
            );
            // Species definition is already in the saved record for room/dynamic creation.
            if !matches!(c.id, 16..=21) {
                assert_eq!(
                    get(
                        g,
                        0x7000 + usize::from(key) + offset_of!(crate::types::EnemySpawnData, id)
                    ),
                    get(g, 0xf78 + usize::from(key)),
                    "record before initializer"
                );
            }
        }
        false
    })
}
pub(crate) fn dispatch_noop(g: &mut Game) -> bool {
    ACTIVE.with(|a| {
        let mut active = a.borrow_mut();
        let Some(c) = active.as_mut() else {
            return false;
        };
        if c.id != 13 {
            return false;
        }
        let key = get(g, crate::ram_map::words::cur_enemy_index);
        append(c, 20, key, 0xa2804c, g);
        if c.nested {
            false
        } else {
            c.nested = true;
            true
        }
    })
}
pub(crate) fn opcode(event: u16, address: u32, key: u16, g: &mut Game) {
    ACTIVE.with(|a| {
        let mut active = a.borrow_mut();
        let Some(c) = active.as_mut() else {
            return;
        };
        append(c, event, key, address, g);
        if event == 16 {
            c.opcodes += 1;
            if c.id == 32 && c.opcodes == 1 {
                // Explicit callback-boundary perturbation; actual opcode already completed.
                put(g, crate::ram_map::words::cur_enemy_index, 64);
                g.loaded_rom = Some(RomImage::zeroed(0x200000));
                append(c, 17, 64, address, g);
            }
        }
    });
}
pub(crate) fn diagnostic(address: u32, g: &mut Game) {
    ACTIVE.with(|a| {
        let mut active = a.borrow_mut();
        let Some(c) = active.as_mut() else {
            return;
        };
        assert!(matches!(c.id, 11 | 17 | 18 | 20), "unexpected diagnostic");
        assert_eq!(address, 0xa01000);
        c.diagnostics += 1;
        append(
            c,
            18,
            get(g, crate::ram_map::words::cur_enemy_index),
            address,
            g,
        );
        if c.id != 11 || c.diagnostics == 2 {
            // Actual diagnostic precedes this new RomRecord acquisition. Already captured
            // population/first definition retains its original full allocation unchanged.
            let end = offset(address)
                + match c.id {
                    11 | 17 => offset_of!(D, x_radius),
                    18 => offset_of!(D, y_radius),
                    20 => offset_of!(D, layer),
                    _ => unreachable!(),
                };
            let old = g.loaded_rom.as_ref().unwrap();
            let short = RomImage::zeroed(end);
            for i in 0..end {
                short.write(i, old.read(i));
            }
            g.loaded_rom = Some(short);
            append(
                c,
                19,
                get(g, crate::ram_map::words::cur_enemy_index),
                address,
                g,
            );
        }
    });
}
pub(crate) fn finish() -> Vec<u8> {
    ACTIVE.with(|a| {
        let c = a.borrow_mut().take().unwrap();
        let create = &[3, 4, 5, 8, 9, 6, 7][..];
        let room = &[1, 2, 3, 4, 5, 8, 9, 6, 7][..];
        let reads = &[10, 11, 12, 13, 14, 6, 7][..];
        let mut expected = Vec::new();
        match c.id {
            0 => {}
            1 | 3 => {
                expected.extend_from_slice(room);
                expected.extend_from_slice(room);
            }
            2 => expected.extend_from_slice(room),
            4..=6 | 10 => expected.extend_from_slice(create),
            7 | 8 | 12 => {
                expected.extend_from_slice(create);
                expected.extend_from_slice(create);
            }
            9 => {}
            11 => expected.extend_from_slice(&[18, 18, 19, 3]),
            13 => {
                expected.extend_from_slice(&create[..6]);
                expected.push(20);
                expected.extend_from_slice(&create[..6]);
                expected.extend_from_slice(&[20, 7, 7]);
            }
            14 | 15 => expected.extend_from_slice(&[8, 9]),
            16 | 19 | 21 => expected.extend_from_slice(reads),
            17 => expected.extend_from_slice(&[18, 19, 10]),
            18 => expected.extend_from_slice(&[18, 19, 10, 11]),
            20 => expected.extend_from_slice(&[18, 19, 10, 11, 12, 13]),
            22..=25 | 30 | 33 | 34 => {}
            26..=29 | 31 => expected.extend_from_slice(&[15, 16]),
            32 => expected.extend_from_slice(&[15, 16, 17, 15, 16]),
            _ => unreachable!(),
        }
        if c.id == 31 {
            expected.pop();
        } // Actual packed operand read fails before dispatch returns.
        assert_eq!(c.events, expected, "exact actual-boundary order");
        assert_eq!(
            c.diagnostics,
            match c.id {
                11 => 2,
                17 | 18 | 20 => 1,
                _ => 0,
            },
            "diagnostic count"
        );
        c.bytes
    })
}

/// Avoid any Runtime handoff outside this exact active capture carrier.
pub(crate) fn active() -> bool {
    ACTIVE.with(|state| state.borrow().is_some())
}
