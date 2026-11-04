//! Verification-only actual-boundary trace. Dormant outside this fixture carrier.
#![forbid(unsafe_code)]
use crate::actor_population_animation_cases::{get_parts, offset, snapshot_parts};
use crate::{
    game::session_runtime::{NativeSpawn, Runtime},
    game_state::{Game, WorkRam},
    rom::RomImage,
    types::{EnemyDef as D, Point16U, Rect16U},
};
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
/// Record one event over the lease's projected RAM `ram` and its spawn words.
fn append(
    c: &mut Capture,
    event: u16,
    key: u16,
    address: u32,
    (g, ram, spawn): (&Game, &WorkRam, &NativeSpawn),
) {
    append_parts(
        c,
        event,
        key,
        address,
        ram,
        &g.loaded_rom,
        &g.g_fail,
        &spawn.0,
        &spawn.1,
        &spawn.2,
        &spawn.3,
    );
}
fn append_parts(
    c: &mut Capture,
    event: u16,
    key: u16,
    address: u32,
    ram: &WorkRam,
    loaded_rom: &Option<RomImage>,
    fail: &bool,
    spawn_rect: &Rect16U,
    spawn_r22: &u16,
    spawn_pt: &Point16U,
    spawn_var_e24: &u16,
) {
    c.events.push(event);
    c.bytes.extend_from_slice(&event.to_le_bytes());
    c.bytes.extend_from_slice(&key.to_le_bytes());
    c.bytes.extend_from_slice(&address.to_le_bytes());
    c.bytes
        .extend_from_slice(&(loaded_rom.as_ref().map_or(0, RomImage::len) as u32).to_le_bytes());
    snapshot_parts(
        &mut c.bytes,
        ram,
        fail,
        spawn_rect,
        spawn_r22,
        spawn_pt,
        spawn_var_e24,
    );
}
fn stage(event: u16, key: u16, view: (&Game, &WorkRam, &NativeSpawn)) {
    let ram = view.1;
    ACTIVE.with(|a| {
        let mut active = a.borrow_mut();
        let Some(c) = active.as_mut() else {
            return;
        };
        append(c, event, key, 0, view);
        if event == 6 {
            assert!(
                matches!(get_parts(ram, 0xf78 + usize::from(key)), 0x9000 | 0x9100),
                "initializer sees published definition"
            );
            assert_eq!(
                get_parts(ram, crate::ram_map::words::CUR_ENEMY_INDEX),
                key,
                "initializer live selector"
            );
            // Species definition is already in the saved record for room/dynamic creation.
            if !matches!(c.id, 16..=21) {
                assert_eq!(
                    get_parts(
                        ram,
                        0x7000 + usize::from(key) + offset_of!(crate::types::EnemySpawnData, id)
                    ),
                    get_parts(ram, 0xf78 + usize::from(key)),
                    "record before initializer"
                );
            }
        }
    });
}
/// Whether the population case that observes the $A2:804C dispatch is capturing.
pub(crate) fn noop_armed() -> bool {
    ACTIVE.with(|a| a.borrow().as_ref().is_some_and(|c| c.id == 13))
}
/// The $A2:804C dispatch on the lease: whether the case's one nested spawn runs now.
pub(crate) fn dispatch_noop(runtime: &mut Runtime<'_>) -> bool {
    runtime.observe_hook(|g, ram, spawn| dispatch_noop_observed((g, ram, spawn)))
}
fn dispatch_noop_observed(view: (&Game, &WorkRam, &NativeSpawn)) -> bool {
    ACTIVE.with(|a| {
        let mut active = a.borrow_mut();
        let Some(c) = active.as_mut() else {
            return false;
        };
        if c.id != 13 {
            return false;
        }
        let key = get_parts(view.1, crate::ram_map::words::CUR_ENEMY_INDEX);
        append(c, 20, key, 0xa2804c, view);
        if c.nested {
            false
        } else {
            c.nested = true;
            true
        }
    })
}
fn opcode(event: u16, address: u32, g: &mut Game, ram: &WorkRam, spawn: &NativeSpawn) {
    let key = get_parts(ram, crate::ram_map::words::CUR_ENEMY_INDEX);
    ACTIVE.with(|a| {
        let mut active = a.borrow_mut();
        let Some(c) = active.as_mut() else {
            return;
        };
        append(c, event, key, address, (g, ram, spawn));
        if event == 16 {
            c.opcodes += 1;
            if c.id == 32 && c.opcodes == 1 {
                // Explicit callback-boundary perturbation; actual opcode already completed.
                // The lease takes the selector store back through its byte path.
                ram.copy_in(crate::ram_map::words::CUR_ENEMY_INDEX, &64u16.to_ne_bytes());
                g.loaded_rom = Some(RomImage::zeroed(0x200000));
                append(c, 17, 64, address, (g, ram, spawn));
            }
        }
    });
}
pub(crate) fn diagnostic_parts(
    address: u32,
    ram: &WorkRam,
    loaded_rom: &mut Option<RomImage>,
    fail: &bool,
    spawn_rect: &Rect16U,
    spawn_r22: &u16,
    spawn_pt: &Point16U,
    spawn_var_e24: &u16,
) {
    ACTIVE.with(|a| {
        let mut active = a.borrow_mut();
        let Some(c) = active.as_mut() else {
            return;
        };
        assert!(matches!(c.id, 11 | 17 | 18 | 20), "unexpected diagnostic");
        assert_eq!(address, 0xa01000);
        c.diagnostics += 1;
        append_parts(
            c,
            18,
            get_parts(ram, crate::ram_map::words::CUR_ENEMY_INDEX),
            address,
            ram,
            loaded_rom,
            fail,
            spawn_rect,
            spawn_r22,
            spawn_pt,
            spawn_var_e24,
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
            let old = loaded_rom.as_ref().unwrap();
            let short = (0..end).map(|i| old.read(i)).collect::<Vec<_>>();
            *loaded_rom = Some(RomImage::from(short));
            append_parts(
                c,
                19,
                get_parts(ram, crate::ram_map::words::CUR_ENEMY_INDEX),
                address,
                ram,
                loaded_rom,
                fail,
                spawn_rect,
                spawn_r22,
                spawn_pt,
                spawn_var_e24,
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
/// The population case's stage hook on the scheduler's lease: while a case is
/// capturing, its observer snapshots the lease's projected RAM.
pub(crate) fn stage_on_lease(runtime: &mut Runtime<'_>, event: u16, key: u16) {
    if active() {
        runtime.observe_hook(|game, ram, spawn| stage(event, key, (game, ram, spawn)));
    }
}
/// The population case's opcode hook on the scheduler's lease, as
/// [`stage_on_lease`] runs its stage hook; the selected slot is the lease's.
pub(crate) fn opcode_on_lease(runtime: &mut Runtime<'_>, event: u16, address: u32) {
    if active() {
        runtime.observe_hook(|game, ram, spawn| opcode(event, address, game, ram, spawn));
    }
}
