//! Literal restore boundaries and actual legacy caller observations.
//! Expected values are examples, not a second implementation of restoration.
use crate::{game_state::Game, ram_map::words as w};
use std::cell::RefCell;

// Each row: amount, current, maximum, reserve, reserve capacity, reserve mode;
// then expected current, reserve, reserve mode.
type Boundary = ([u16; 6], [u16; 3]);
const HEALTH: [Boundary; 13] = [
    ([9, 40, 100, 7, 50, 0], [49, 7, 0]),
    ([10, 90, 100, 7, 50, 0], [100, 7, 1]), // equality activates reserves
    ([10, 90, 100, 0, 50, 0], [100, 0, 0]),
    ([20, 90, 100, 7, 50, 0], [100, 17, 1]),
    ([20, 90, 100, 40, 50, 0], [100, 50, 1]),
    ([40, 90, 100, 40, 50, 0], [100, 50, 1]),
    ([20, 90, 100, 7, 50, 2], [100, 17, 2]),
    ([0x30, 0xfff0, 100, 7, 50, 0], [0x20, 7, 0]),
    ([0, 0x8000, 0, 7, 50, 0], [0x8000, 7, 0]),
    ([0, 0xffff, 100, 7, 50, 0], [0xffff, 7, 0]),
    ([1, 100, 100, 0xffff, 50, 0], [100, 0, 0]),
    ([100, 100, 100, 0x8000, 50, 0], [100, 0x8064, 1]),
    ([0, 100, 0x9000, 7, 50, 0], [0x9000, 50, 1]),
];
const MISSILES: [Boundary; 17] = [
    ([1, 4, 10, 3, 0, 2], [5, 3, 2]),
    ([1, 9, 10, 3, 0, 2], [10, 3, 2]),
    ([6, 9, 10, 3, 0, 2], [10, 8, 2]),
    ([6, 9, 10, 8, 0, 2], [10, 10, 2]),
    ([4, 97, 98, 97, 0, 2], [98, 98, 2]),
    ([3, 98, 99, 98, 0, 2], [99, 99, 2]),
    ([3, 99, 100, 98, 0, 2], [100, 99, 2]),
    ([20, 199, 200, 90, 0, 2], [200, 99, 2]),
    ([0, 10, 10, 20, 0, 2], [10, 10, 2]), // equality still clamps reserve
    ([0, 100, 100, 120, 0, 2], [100, 99, 2]),
    ([0x30, 0xfff0, 100, 3, 0, 2], [0x20, 3, 2]),
    ([0, 0x8000, 0, 3, 0, 2], [0x8000, 3, 2]),
    ([0, 0x8063, 0x8063, 100, 0, 2], [0x8063, 100, 2]),
    ([0, 0x8062, 0x8062, 100, 0, 2], [0x8062, 99, 2]),
    ([1, 10, 10, 0xffff, 0, 2], [10, 0, 2]),
    ([1, 10, 10, 0x800a, 0, 2], [10, 0x800b, 2]),
    ([1, 100, 100, 0x8063, 0, 2], [100, 0x8064, 2]),
];
// Both resources execute their own maintained routine against these literals.
const CLAMPED: [Boundary; 8] = [
    ([1, 4, 10, 7, 50, 2], [5, 7, 2]),
    ([1, 9, 10, 7, 50, 2], [10, 7, 2]),
    ([3, 9, 10, 7, 50, 2], [10, 7, 2]),
    ([0x30, 0xfff0, 100, 7, 50, 2], [0x20, 7, 2]),
    ([0, 0x8000, 0, 7, 50, 2], [0x8000, 7, 2]),
    ([0, 100, 0x9000, 7, 50, 2], [0x9000, 7, 2]),
    ([0, 0xffff, 10, 7, 50, 2], [0xffff, 7, 2]),
    ([0, 0, 0xffff, 7, 50, 2], [0xffff, 7, 2]),
];
const BOUNDARIES: u32 = 46;
pub(crate) const CASES: u32 = BOUNDARIES + 12;
const AMMO: [usize; 3] = [
    w::samus_missiles,
    w::samus_super_missiles,
    w::samus_power_bombs,
];
const SHIP_STATE: usize = 0xf78 + core::mem::offset_of!(crate::types::EnemyData, ai_preinstr);
const OBSERVED: [usize; 16] = [
    w::samus_health,
    w::samus_max_health,
    w::samus_reserve_health,
    w::samus_max_reserve_health,
    w::reserve_health_mode,
    w::samus_missiles,
    w::samus_max_missiles,
    w::samus_reserve_missiles,
    w::samus_super_missiles,
    w::samus_max_super_missiles,
    w::samus_power_bombs,
    w::samus_max_power_bombs,
    w::substate,
    w::which_item_to_pickup,
    SHIP_STATE,
    w::bitmask,
];
thread_local! {
    static TRACE: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) };
}
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn record(bytes: &[u8], resource: u8, phase: u8, amount: u16, trace: &mut Vec<u8>) {
    trace.extend_from_slice(&[resource, phase]);
    trace.extend_from_slice(&amount.to_le_bytes());
    for at in OBSERVED {
        trace.extend_from_slice(&bytes[at..at + 2]);
    }
}
/// Observer only; never changes the behavior or supplies a restoration result.
/// The old-reference overlay calls this at each actual restore entry and return.
pub(crate) fn observe(resource: u8, phase: u8, amount: u16, game: &Game) {
    TRACE.with(|trace| {
        if let Some(trace) = trace.borrow_mut().as_mut() {
            record(&game.g_ram, resource, phase, amount, trace);
        }
    });
}
fn begin_trace() {
    TRACE.with(|trace| {
        assert!(trace.borrow().is_none());
        *trace.borrow_mut() = Some(Vec::new());
    });
}
fn finish_trace() -> Vec<u8> {
    TRACE.with(|trace| {
        trace
            .borrow_mut()
            .take()
            .expect("restore caller trace armed")
    })
}
// This is the only dispatch to retarget when Inventory owns these algorithms.
// The cases and literal expectations stay unchanged.
fn restore(resource: u8, amount: u16, game: &mut Game) {
    use crate::game::item_pickups::Resource;
    let resource = match resource {
        0 => Resource::Health,
        1 => Resource::Missiles,
        2 => Resource::SuperMissiles,
        3 => Resource::PowerBombs,
        _ => unreachable!(),
    };
    crate::game::item_pickups_runtime::restore(resource, amount, game);
}
fn boundary(id: u32, game: &mut Game) {
    let (resource, row) = match id {
        0..=12 => (0, HEALTH[id as usize]),
        13..=29 => (1, MISSILES[id as usize - 13]),
        30..=37 => (2, CLAMPED[id as usize - 30]),
        _ => (3, CLAMPED[id as usize - 38]),
    };
    let (current_at, maximum_at) = match resource {
        0 => (w::samus_health, w::samus_max_health),
        1 => (w::samus_missiles, w::samus_max_missiles),
        2 => (w::samus_super_missiles, w::samus_max_super_missiles),
        _ => (w::samus_power_bombs, w::samus_max_power_bombs),
    };
    let reserve_at = if resource == 1 {
        w::samus_reserve_missiles
    } else {
        w::samus_reserve_health
    };
    let ([amount, current, maximum, reserve, capacity, mode], expected) = row;
    for (at, value) in [
        (current_at, current),
        (maximum_at, maximum),
        (reserve_at, reserve),
        (w::samus_max_reserve_health, capacity),
        (w::reserve_health_mode, mode),
    ] {
        put(&mut game.g_ram, at, value);
    }
    let mut expected_ram = game.g_ram.to_vec();
    put(&mut expected_ram, current_at, expected[0]);
    put(&mut expected_ram, reserve_at, expected[1]);
    put(&mut expected_ram, w::reserve_health_mode, expected[2]);
    restore(resource, amount, game);
    let resource_name = ["health", "missile", "super-missile", "power-bomb"][resource as usize];
    let actual = [current_at, reserve_at, w::reserve_health_mode]
        .map(|at| u16::from_le_bytes([game.g_ram[at], game.g_ram[at + 1]]));
    assert_eq!(actual, expected, "{resource_name} restore boundary {id}");
    assert_eq!(&game.g_ram[..], &expected_ram[..], "restore boundary {id}");
}

unsafe fn crystal_flash(id: u32, game: &mut Game) -> Vec<u8> {
    let lane = id as usize / 3;
    let variant = id % 3;
    for (at, value) in [
        (w::samus_health, 90),
        (w::samus_max_health, 100),
        (w::samus_reserve_health, 10),
        (w::samus_max_reserve_health, 50),
        (w::reserve_health_mode, 0),
        (w::samus_missiles, 10),
        (w::samus_super_missiles, 10),
        (w::samus_power_bombs, 10),
        (w::substate, if variant == 0 { 2 } else { 1 }),
        (w::which_item_to_pickup, lane as u16),
        (w::nmi_frame_counter_word, if variant == 2 { 1 } else { 8 }),
    ] {
        put(&mut game.g_ram, at, value);
    }
    let mut expected_ram = game.g_ram.to_vec();
    if variant != 2 {
        put(&mut expected_ram, AMMO[lane], 9);
        for (at, value) in [
            (w::samus_health, 100),
            (w::samus_reserve_health, 50),
            (w::reserve_health_mode, 1),
        ] {
            put(&mut expected_ram, at, value);
        }
        if variant == 0 {
            put(&mut expected_ram, w::substate, 1);
        } else if lane < 2 {
            put(&mut expected_ram, w::substate, 10);
            put(&mut expected_ram, w::which_item_to_pickup, lane as u16 + 1);
        } else {
            for (at, value) in [
                (w::substate, 0),
                (w::samus_movement_handler, 0xd75b),
                (w::samus_draw_handler, 0xeb52),
                (w::samus_anim_frame_timer, 3),
                (w::samus_anim_frame, 12),
            ] {
                put(&mut expected_ram, at, value);
            }
        }
    }
    unsafe {
        crate::game::session_runtime::advance_special_movement(0x90d6ce, game);
    }
    put(&mut expected_ram, w::samus_invincibility_timer, 0);
    put(&mut expected_ram, w::samus_knockback_timer, 0);
    assert_eq!(
        &game.g_ram[..],
        &expected_ram[..],
        "crystal flash final state {id}"
    );
    Vec::new()
}

unsafe fn gunship(id: u32, game: &mut Game) -> Vec<u8> {
    for (at, value) in [
        (w::samus_health, if id == 0 { 96 } else { 100 }),
        (w::samus_max_health, 100),
        (
            w::samus_reserve_health,
            if id == 1 {
                8
            } else if id == 2 {
                6
            } else {
                0
            },
        ),
        (w::samus_max_reserve_health, 10),
        (w::reserve_health_mode, 0),
        (w::samus_missiles, 3),
        (w::samus_max_missiles, 5),
        (w::samus_reserve_missiles, 0),
        (w::samus_super_missiles, if id == 0 { 1 } else { 3 }),
        (w::samus_max_super_missiles, 5),
        (w::samus_power_bombs, if id == 0 { 1 } else { 3 }),
        (w::samus_max_power_bombs, 5),
        (SHIP_STATE, 0xcafe),
    ] {
        put(&mut game.g_ram, at, value);
    }
    game.g_ram[0xd821] = 0; // Tourian event14 is clear: ordinary refill branch.
    let mut expected_ram = game.g_ram.to_vec();
    put(&mut expected_ram, w::bitmask, 0x40); // Actual CheckEventHappened precedes refill.
    let mut expected_trace = Vec::new();
    for resource in 0..4 {
        record(&expected_ram, resource, 0, 2, &mut expected_trace);
        match resource {
            0 if id == 0 => put(&mut expected_ram, w::samus_health, 98),
            0 => {
                put(
                    &mut expected_ram,
                    w::samus_reserve_health,
                    if id == 1 { 10 } else { 8 },
                );
                put(&mut expected_ram, w::reserve_health_mode, 1);
            }
            1 => put(&mut expected_ram, w::samus_missiles, 5),
            2 => put(
                &mut expected_ram,
                w::samus_super_missiles,
                if id == 0 { 3 } else { 5 },
            ),
            _ => put(
                &mut expected_ram,
                w::samus_power_bombs,
                if id == 0 { 3 } else { 5 },
            ),
        }
        record(&expected_ram, resource, 1, 2, &mut expected_trace);
    }
    if id == 1 {
        put(&mut expected_ram, SHIP_STATE, 0xab1f);
    }
    begin_trace();
    crate::game::gunship::phase(0, 0xaaa2, game);
    let trace = finish_trace();
    assert_eq!(trace, expected_trace, "gunship restore ordering {id}");
    assert_eq!(
        &game.g_ram[..],
        &expected_ram[..],
        "gunship completion after refill {id}"
    );
    trace
}

fn capture_checked(id: u32) -> Vec<u8> {
    let mut game = Game::new();
    // Nonzero unchanged surroundings catch collateral writes and owner defaults.
    game.g_ram.fill(0x5a);
    let trace = if id < BOUNDARIES {
        boundary(id, &mut game);
        Vec::new()
    } else if id < BOUNDARIES + 9 {
        unsafe { crystal_flash(id - BOUNDARIES, &mut game) }
    } else {
        unsafe { gunship(id - BOUNDARIES - 9, &mut game) }
    };
    assert!(!game.g_fail, "restore caller diagnostic {id}");
    let mut output = game.g_ram.to_vec();
    for value in [
        game.eproj_spawn_rect.x,
        game.eproj_spawn_rect.y,
        game.eproj_spawn_rect.w,
        game.eproj_spawn_rect.h,
        game.eproj_spawn_r22,
        game.eproj_spawn_pt.x,
        game.eproj_spawn_pt.y,
        game.eproj_spawn_varE24,
    ] {
        output.extend_from_slice(&value.to_le_bytes());
    }
    output.extend_from_slice(&[0; 11]); // Existing carrier: no return/panic/diagnostic.
    output.extend_from_slice(&(trace.len() as u32).to_le_bytes());
    output.extend_from_slice(&trace);
    output
}

pub(crate) fn capture_case(id: u32) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| capture_checked(id)));
    assert_eq!(outcome.is_ok(), true, "restore case {} failed", 1382 + id);
    outcome.ok()
}
