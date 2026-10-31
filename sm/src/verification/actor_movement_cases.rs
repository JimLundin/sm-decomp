//! Forty bounded actual-4c572a4a common-actor predecessor fixtures.
//! No copied movement/contact rules. Synthetic records contain no ROM assets.
use crate::ram_map::{word_arrays as a, words as w};
use crate::types::{EnemyData, EnemyDef, Point16U, Rect16U};
use crate::{game_state::Game, rom::RomImage};
use core::mem::{offset_of, size_of};
use sm_game::session::Vector32;
use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
};
pub(crate) const CASES: u32 = 40;
struct ObserverGuard;
impl Drop for ObserverGuard {
    fn drop(&mut self) {
        crate::actor_movement_observer::cancel();
    }
}
pub(crate) fn offset(address: u32) -> usize {
    crate::rtl::lorom_offset(address) as usize
}
pub(crate) fn put(g: &mut Game, at: usize, v: u16) {
    g.ram().copy_in(at, &v.to_le_bytes());
}
pub(crate) fn read(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes([g.ram().read_byte(at), g.ram().read_byte(at + 1)])
}
pub(crate) fn set_actor_word(g: &mut Game, k: u16, field: usize, value: u16) {
    put(g, 0xf78 + usize::from(k) + field, value);
}
pub(crate) fn actor_word(g: &Game, k: u16, field: usize) -> u16 {
    read(g, 0xf78 + usize::from(k) + field)
}
fn rw(image: &mut [u8], address: u32, value: u16) {
    let at = offset(address);
    image[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn actor(g: &mut Game, k: u16, x: u16, y: u16, radius: u16) {
    let at = 0xf78 + usize::from(k);
    assert!(at + size_of::<EnemyData>() <= g.ram().len());
    g.ram().fill(at..at + size_of::<EnemyData>(), 0);
    for (field, value) in [
        (offset_of!(EnemyData, x_pos), x),
        (offset_of!(EnemyData, y_pos), y),
        (offset_of!(EnemyData, x_subpos), 0xffff),
        (offset_of!(EnemyData, y_subpos), 0xffff),
        (offset_of!(EnemyData, x_width), radius),
        (offset_of!(EnemyData, y_height), radius),
        (offset_of!(EnemyData, enemy_ptr), 0x9000),
        (offset_of!(EnemyData, spritemap_pointer), 0x9000),
    ] {
        set_actor_word(g, k, field, value);
    }
}
// Same explicit32x32 terrain fixture shape as candidate_collision_cases::room.
// Helpers there are private to the eproj carrier; no production rule duplicated.
fn room(g: &mut Game, kind: u16, bts: u8) {
    put(g, w::ROOM_WIDTH_IN_BLOCKS, 32);
    for block in 0..1024 {
        put(g, 0x10002 + block * 2, kind << 12);
        g.ram().write_byte(0x16402 + block, bts);
    }
}
fn seed(g: &mut Game, loaded: &mut [u8], native: &mut [u8]) {
    for i in 0..g.ram().len() {
        g.ram()
            .write_byte(i, (i.wrapping_mul(29) ^ (i >> 7) ^ 0xa3) as u8);
    }
    g.VERIFY_ACTIVE = true;
    g.g_fail = false;
    g.eproj_spawn_rect = Rect16U {
        x: 0x1357,
        y: 0x2468,
        w: 0x3579,
        h: 0x468a,
    };
    g.eproj_spawn_r22 = 0x579b;
    g.eproj_spawn_pt = Point16U {
        x: 0x68ac,
        y: 0x79bd,
    };
    g.eproj_spawn_varE24 = 0x8ace;
    room(g, 0, 0);
    actor(g, 0, 136, 136, 3);
    actor(g, 64, 136, 136, 3);
    for (at, value) in [
        (w::CUR_ENEMY_INDEX, 0),
        (w::CUR_BLOCK_INDEX, 0),
        (w::SAMUS_X_POS, 136),
        (w::SAMUS_Y_POS, 136),
        (w::SAMUS_X_RADIUS, 3),
        (w::SAMUS_Y_RADIUS, 3),
        (w::SAMUS_X_SUBPOS, 0),
        (w::SAMUS_Y_SUBPOS, 0x2222),
        (w::SAMUS_CONTACT_DAMAGE_INDEX, 0),
        (w::SAMUS_INVINCIBILITY_TIMER, 0),
        (w::PROJECTILE_COUNTER, 0),
        (w::BOMB_COUNTER, 0),
    ] {
        put(g, at, value);
    }
    for i in 0..40 {
        put(g, a::PLM_HEADER_PTR + i * 2, 0);
    }
    // Distinct immutable contents make accidental native/loaded selection visible.
    let at = offset(0x948b2b);
    loaded[at..at + 512].fill(4);
    native[at..at + 512].fill(27);
    let at = offset(0xa0b143);
    for (i, (loaded_byte, native_byte)) in loaded[at..at + 128]
        .iter_mut()
        .zip(&mut native[at..at + 128])
        .enumerate()
    {
        *loaded_byte = i as u8;
        *native_byte = 127 - i as u8;
    }
    rw(loaded, 0xa0c49f + 22, 128);
    rw(native, 0xa0c49f + 22, 256);
    rw(loaded, 0xa0c2da, 0x9000);
    rw(loaded, 0x849000, 0xb3d0); // actual setup Return(0), no device effects
    rw(loaded, 0x849002, 0x8888);
    rw(native, 0x849000, 0xb3d2); // actual Return(1), would expose wrong source
    rw(native, 0x849002, 0x7777);
}
#[derive(Clone, Copy)]
pub(crate) enum Operation {
    Vector { phase: u16, value: Vector32 },
    ProbeX { k: u16, amount: i32 },
    ProbeY { k: u16, amount: i32 },
    ProbeOffset { k: u16, edge: u16, amount: i32 },
    Right { k: u16, amount: i32, policy: u8 },
    Down { k: u16, amount: i32 },
    Solid(i32),
    Touch(u16),
    Padding(u16),
    Distance(Point16U, Point16U),
    Body,
    Contacts,
}
/// Candidate integration can supply another dispatcher while preserving every
/// fixture, observer assertion and capture byte; this dispatcher is predecessor.
pub(crate) fn run(op: Operation, g: &mut Game) -> Option<u64> {
    crate::game::actors::movement_verification::run(op, g)
}

fn fixture(id: u32, g: &mut Game, loaded: &mut [u8]) -> Operation {
    match id {
        0..=2 => {
            let k = [0, 1, 0xffff][id as usize];
            let (x, xf, y, yf, phase, vx, vy) = match id {
                0 => (0x1234, 0xffff, 0xff, 0, 192, 1, 0x10001),
                1 => (1, 0, 0, 0, 64, 1, 0x10001),
                _ => (0xfffe, 0xffff, 0x8000, 0xffff, 192, 0x10002, 1),
            };
            actor(g, k, x, y, 3);
            set_actor_word(g, k, offset_of!(EnemyData, x_subpos), xf);
            set_actor_word(g, k, offset_of!(EnemyData, y_subpos), yf);
            put(g, w::CUR_ENEMY_INDEX, k);
            Operation::Vector {
                phase,
                value: Vector32 { x: vx, y: vy },
            }
        }
        3 => {
            room(g, 8, 0);
            Operation::ProbeX {
                k: 0,
                amount: 0x10001,
            }
        }
        4 => {
            room(g, 0, 0);
            Operation::ProbeY {
                k: 0,
                amount: 0x10001,
            }
        }
        5 | 6 => {
            // Y center136: subtracting positive16 gives edge117 (row7), adding
            // negative16 gives edge122(row7); the opposite sign heuristic hits9.
            actor(g, 0, 136, 136, 3);
            set_actor_word(g, 0, offset_of!(EnemyData, y_subpos), 0);
            for b in 224..256 {
                put(g, 0x10002 + b * 2, 0x8000);
            }
            Operation::ProbeOffset {
                k: 0,
                edge: if id == 5 { 0 } else { 1 },
                amount: if id == 5 { 0x100000 } else { -0x100000 },
            }
        }
        7 | 8 => {
            actor(g, 0, 16392, 2056, 1);
            put(g, w::ROOM_WIDTH_IN_BLOCKS, 255);
            // Horizontal wraps byte offset0x10700 to0700; vertical retains10700.
            put(g, 0x10702, 0x8000);
            if id == 7 {
                Operation::ProbeX { k: 0, amount: 0 }
            } else {
                Operation::ProbeY { k: 0, amount: 0 }
            }
        }
        9..=16 => {
            let direction = (id - 9) % 4;
            room(g, if id >= 13 { 8 } else { 0 }, 0);
            let amount = if direction & 1 == 0 {
                0x10001
            } else {
                -0x10001
            };
            if direction < 2 {
                Operation::Right {
                    k: 0,
                    amount,
                    policy: 0,
                }
            } else {
                Operation::Down { k: 0, amount }
            }
        }
        17 | 18 => {
            room(g, 1, 0);
            if id == 17 {
                Operation::Right {
                    k: 0,
                    amount: 0x10001,
                    policy: 2,
                }
            } else {
                Operation::Down {
                    k: 0,
                    amount: 0x10001,
                }
            }
        }
        19..=21 => {
            room(g, 1, 5);
            Operation::Right {
                k: 0,
                amount: 0x10001,
                policy: (id - 19) as u8,
            }
        }
        22 | 23 => {
            room(g, 1, if id == 22 { 5 } else { 0x85 });
            Operation::Down {
                k: 0,
                amount: if id == 22 { 0x10001 } else { -0x10001 },
            }
        }
        24 => {
            put(g, 0x10002 + 264 * 2, 0x5000);
            g.ram().write_byte(0x16402 + 264, 1);
            put(g, 0x10002 + 265 * 2, 0xd000);
            g.ram().write_byte(0x16402 + 265, 1);
            Operation::Right {
                k: 0,
                amount: 0x10001,
                policy: 0,
            }
        }
        25 => {
            put(g, 0x10002 + 264 * 2, 0xd000);
            g.ram().write_byte(0x16402 + 264, 0xff);
            put(g, 0x10002 + 232 * 2, 0x5000);
            g.ram().write_byte(0x16402 + 232, 0xff);
            put(g, 0x10002 + 231 * 2, 0x8000);
            Operation::Down {
                k: 0,
                amount: 0x10001,
            }
        }
        26..=28 => {
            room(g, 10, 0);
            if id == 27 {
                Operation::Down {
                    k: 0,
                    amount: 0x10001,
                }
            } else {
                Operation::Right {
                    k: 0,
                    amount: 0x10001,
                    policy: 0,
                }
            }
        }
        29..=32 | 39 => {
            let direction = if id == 39 { 0 } else { (id - 29) as u16 };
            actor(g, 0, 136, 136, 3);
            set_actor_word(g, 0, offset_of!(EnemyData, frozen_timer), 1);
            put(g, w::INTERACTIVE_ENEMY_INDEXES_WRITE_PTR, 2);
            put(g, a::INTERACTIVE_ENEMY_INDEXES, 0);
            put(g, a::INTERACTIVE_ENEMY_INDEXES + 2, 0xffff);
            put(g, w::SAMUS_COLLISION_DIRECTION, direction);
            let axis = if direction < 2 {
                w::SAMUS_X_POS
            } else {
                w::SAMUS_Y_POS
            };
            put(g, axis, if direction & 1 == 0 { 142 } else { 144 });
            if direction & 1 != 0 {
                set_actor_word(
                    g,
                    0,
                    if direction < 2 {
                        offset_of!(EnemyData, x_pos)
                    } else {
                        offset_of!(EnemyData, y_pos)
                    },
                    150,
                );
            }
            if id == 39 {
                set_actor_word(g, 0, offset_of!(EnemyData, x_pos), 0xfffe);
                set_actor_word(g, 0, offset_of!(EnemyData, x_width), 1);
                put(g, w::SAMUS_X_POS, 0);
                put(g, w::SAMUS_X_RADIUS, 1);
            }
            // This entry takes magnitude; its direction applies subtraction.
            Operation::Solid(0x10000)
        }
        33 => {
            put(g, w::SAMUS_Y_POS, 130);
            Operation::Touch(0)
        }
        34 => {
            put(g, w::SAMUS_X_POS, 145);
            Operation::Padding(0)
        }
        35 => Operation::Distance(Point16U { x: 136, y: 136 }, Point16U { x: 145, y: 143 }),
        36 => Operation::Distance(Point16U { x: 100, y: 100 }, Point16U { x: 356, y: 100 }),
        37 | 38 => {
            actor(g, 0xe3ea, 136, 136, 3);
            put(g, w::CUR_ENEMY_INDEX, 0xe3ea);
            put(g, w::ENEMY_PROCESSING_STAGE, 0); // physically Map of E3EA
            put(g, w::SAMUS_CONTACT_DAMAGE_INDEX, 1);
            put(g, w::SAMUS_INVINCIBILITY_TIMER, 77);
            rw(
                loaded,
                0xa09000 + offset_of!(EnemyDef, touch_ai) as u32,
                0x804c,
            );
            if id == 37 {
                Operation::Body
            } else {
                Operation::Contacts
            }
        }
        _ => unreachable!(),
    }
}
fn panic_text(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
        .expect("expected textual checked panic")
}
fn checks(id: u32, g: &Game, returned: Option<u64>) {
    assert!(!g.g_fail, "no native diagnostic or unrelated failure");
    if matches!(id, 3 | 5 | 6 | 7 | 13..=16 | 20 | 25) {
        assert_eq!(returned, Some(1), "directed blocked fixture");
    }
    if matches!(id, 4 | 9..=12 | 19 | 21 | 24 | 26 | 27) {
        assert_eq!(returned, Some(0), "directed clear fixture");
    }
    if id == 24 {
        assert_eq!(
            g.cur_block_index(),
            297,
            "two extension retries reach empty cell"
        );
    }
    if id == 25 {
        assert_eq!(
            g.cur_block_index(),
            231,
            "signed extension reaches solid cell"
        );
    }
    if id == 26 || id == 27 {
        let expected = if id == 26 {
            [201, 8, 300, 9]
        } else {
            [200, 7, 138, 0]
        };
        for (field, value) in [
            offset_of!(EnemyData, x_pos),
            offset_of!(EnemyData, x_subpos),
            offset_of!(EnemyData, y_pos),
            offset_of!(EnemyData, y_subpos),
        ]
        .into_iter()
        .zip(expected)
        {
            assert_eq!(
                actor_word(g, 0, field),
                value,
                "live horizontal versus captured vertical commit"
            );
        }
        assert_eq!(g.cur_enemy_index(), 64);
        assert_eq!(g.room_width_in_blocks(), 64);
        assert_eq!(read(g, 0x10002 + 264 * 2), 0);
    }
    if id == 28 {
        assert_eq!(
            read(g, a::PLM_HEADER_PTR + 78),
            0x9000,
            "header prefix before failing program read"
        );
        assert_eq!(
            read(g, a::PLM_BLOCK_INDICES + 78),
            528,
            "block prefix before failing program read"
        );
        assert_eq!(
            read(g, a::PLM_PRE_INSTRS + 78),
            0x853d,
            "preaction prefix before failing program read"
        );
        assert_eq!(
            actor_word(g, 0, offset_of!(EnemyData, x_pos)),
            136,
            "movement commit absent on callback unwind"
        );
    }
    if (29..=32).contains(&id) || id == 39 {
        assert_eq!(
            returned,
            Some(0xffff),
            "solid exact-zero displacement result"
        );
        assert_eq!(
            g.samus_y_subpos(),
            if id == 39 { 0x2222 } else { 0 },
            "widened equality controls exact-contact fraction clear"
        );
    }
    if id == 33 || id == 34 {
        assert_eq!(returned, Some(0xffff));
    }
    if id == 36 {
        assert_eq!(returned, Some(256));
    }
    if id == 37 || id == 38 {
        assert_eq!(
            read(g, w::SAMUS_INVINCIBILITY_TIMER),
            0,
            "stage-before-Map gate executes contact prefix"
        );
        assert_eq!(
            actor_word(g, 0xe3ea, offset_of!(EnemyData, spritemap_pointer)),
            9
        );
    }
}
pub(crate) fn capture_case(
    id: u32,
    dispatch: fn(Operation, &mut Game) -> Option<u64>,
) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    capture_prepared_case(id, dispatch)
}
#[path = "actor_movement_table_cases.rs"]
mod table_cases;
pub(crate) fn capture_table_case(
    id: u32,
    dispatch: fn(Operation, &mut Game) -> Option<u64>,
) -> Option<Vec<u8>> {
    if id >= table_cases::CASES {
        return None;
    }
    capture_prepared_case(CASES + id, dispatch)
}
fn capture_prepared_case(
    id: u32,
    dispatch: fn(Operation, &mut Game) -> Option<u64>,
) -> Option<Vec<u8>> {
    let mut loaded = vec![0; 0x200000];
    let mut native = vec![0; 0x200000];
    let mut g = Game::new();
    seed(&mut g, &mut loaded, &mut native);
    let op = if id < CASES {
        fixture(id, &mut g, &mut loaded)
    } else {
        table_cases::fixture(id - CASES, &mut g, &mut loaded)
    };
    // Every byte is authored before the images are registered.
    g.loaded_rom = Some(RomImage::from(loaded));
    let native = RomImage::from(native);
    g.register_native_rom(native.clone());
    let before = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<_>>();
    crate::actor_movement_observer::begin(id);
    let _observer_guard = ObserverGuard;
    let result = catch_unwind(AssertUnwindSafe(|| dispatch(op, &mut g)));
    let expectation = match id {
        // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
        8 => Some("index out of bounds: the len is 131072 but the index is 132866"),
        28 | 40..=43 => Some("ROM read outside loaded allocation"),
        _ => None,
    };
    let returned = match result {
        Ok(value) => {
            if expectation.is_some() {
                crate::actor_movement_observer::cancel();
                panic!("expected checked failure case{id}");
            }
            value
        }
        Err(payload) => {
            let text = panic_text(payload.as_ref());
            assert_eq!(
                Some(text.as_str()),
                expectation,
                "exact checked-failure boundary case{id}"
            );
            None
        }
    };
    if id == 8 {
        assert_eq!(
            &(0..g.ram().len())
                .map(|at| g.ram().read_byte(at))
                .collect::<Vec<_>>(),
            before.as_slice(),
            "read-only wide probe failure has no writes"
        );
    }
    checks(id, &g, returned);
    if id >= CASES {
        table_cases::checks(id - CASES, &g, &before, &native);
    }
    let trace = crate::actor_movement_observer::finish();
    let mut out = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<_>>();
    for v in [
        g.eproj_spawn_rect.x,
        g.eproj_spawn_rect.y,
        g.eproj_spawn_rect.w,
        g.eproj_spawn_rect.h,
        g.eproj_spawn_r22,
        g.eproj_spawn_pt.x,
        g.eproj_spawn_pt.y,
        g.eproj_spawn_varE24,
    ] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.push(u8::from(returned.is_some()));
    out.extend_from_slice(&returned.unwrap_or(0).to_le_bytes());
    out.push(u8::from(expectation.is_some()));
    out.push(u8::from(g.g_fail));
    assert_eq!(
        out.len(),
        131099,
        "unchanged existing carrier prefix layout"
    );
    // New carrier only: append exact failure text and ordered observer bytes.
    let failure = expectation.unwrap_or("").as_bytes();
    out.extend_from_slice(&(failure.len() as u32).to_le_bytes());
    out.extend_from_slice(failure);
    out.extend_from_slice(&(trace.len() as u32).to_le_bytes());
    out.extend(trace);
    Some(out)
}
