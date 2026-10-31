//! Directed actual-50cec3e0 player movement fixtures. Synthetic content only.
//! Requires the coordinator's fully initialized SnesInit/audio/PPU host.
use crate::{
    game_state::Game,
    ram_map::{word_arrays as a, words as w},
    rom::RomImage,
    types::EnemyData,
};
use core::mem::{offset_of, size_of};
pub(crate) const COUNT: u32 = 36;
thread_local! {
    static CANCEL_ARMED: core::cell::Cell<bool> = const { core::cell::Cell::new(false) };
    static CANCEL_VALUES: std::cell::RefCell<Vec<[u16;3]>> = const { std::cell::RefCell::new(Vec::new()) };
}
struct CancelGuard;
impl Drop for CancelGuard {
    fn drop(&mut self) {
        CANCEL_ARMED.with(|v| v.set(false));
    }
}
fn checkpoint(out: &mut Vec<u8>, g: &Game, tag: u8, returned: Option<u32>) {
    out.push(tag);
    out.extend((0..g.ram().len()).map(|at| g.ram().read_byte(at)));
    out.push(u8::from(g.g_fail));
    out.push(u8::from(returned.is_some()));
    out.extend_from_slice(&returned.unwrap_or(0).to_le_bytes());
}
fn put(g: &mut Game, at: usize, value: u16) {
    g.ram().copy_in(at, &value.to_le_bytes());
}
fn get(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes([g.ram().read_byte(at), g.ram().read_byte(at + 1)])
}
fn expect(g: &Game, at: usize, value: u16, why: &str) {
    assert_eq!(get(g, at), value, "{why} at {at:04x}");
}
fn rom_word(r: &RomImage, address: u32, value: u16) {
    let at = crate::rtl::lorom_offset(address) as usize;
    for (i, b) in value.to_le_bytes().into_iter().enumerate() {
        r.write(at + i, b);
    }
}
fn block(g: &mut Game, at: usize, kind: u16, bts: u8) {
    put(g, 0x10002 + 2 * at, kind << 12);
    g.ram().write_byte(0x16402 + at, bts);
}
fn actor(g: &mut Game, key: u16, x: u16, y: u16) {
    let at = 0xf78 + usize::from(key);
    g.ram().fill(at..at + size_of::<EnemyData>(), 0);
    for (field, v) in [
        (offset_of!(EnemyData, x_pos), x),
        (offset_of!(EnemyData, y_pos), y),
        (offset_of!(EnemyData, x_width), 3),
        (offset_of!(EnemyData, y_height), 3),
        (offset_of!(EnemyData, frozen_timer), 1),
    ] {
        put(g, at + field, v);
    }
}
fn seed(g: &mut Game) -> (RomImage, RomImage) {
    // Host hardware remains initialized; every byte of both replacement ROM
    // allocations is initialized, including raw/native table consumers.
    let loaded = RomImage::zeroed(0x200000);
    let native = RomImage::zeroed(0x200000);
    g.loaded_rom = Some(loaded.clone());
    g.register_native_rom(native.clone());
    g.ram().fill(0..g.ram().len(), 0);
    g.g_fail = false;
    g.VERIFY_ACTIVE = true;
    for (at, v) in [
        (w::ROOM_WIDTH_IN_BLOCKS, 32),
        (w::SAMUS_X_POS, 136),
        (w::SAMUS_Y_POS, 136),
        (w::SAMUS_X_RADIUS, 3),
        (w::SAMUS_Y_RADIUS, 3),
        (w::SAMUS_Y_SUBPOS, 0x2222),
        (w::SAMUS_PREV_X_POS, 136),
        (w::SAMUS_PREV_Y_POS, 136),
        (w::EQUIPPED_ITEMS, 0x20),
        (w::SPEED_ECHOES_INDEX, 0xffff),
        (w::SAMUS_X_SPEED_TABLE_POINTER, 0xe000),
        (w::BUTTON_CONFIG_JUMP_A, 0x80),
    ] {
        put(g, at, v);
    }
    g.ram().write_byte(0xa1e, 8);
    for at in (0xdaa..0xdba).step_by(2) {
        put(g, at, 0x7777);
    }
    for r in [&loaded, &native] {
        rom_word(r, 0x949139, 0x9000);
        rom_word(r, 0x849000, 0xb4c4);
        rom_word(r, 0x849002, 0x8888);
        // Actual SamusSpeedTableEntry: accel/sub,max/sub,decel/sub.
        for address in (0x90e000..0x90e12c).step_by(12).chain([0x909f31]) {
            for (i, v) in [0, 0, 8, 0, 2, 0].into_iter().enumerate() {
                rom_word(r, address + (i * 2) as u32, v);
            }
        }
    }
    for i in 0..16 {
        rom_word(&native, 0x9b9800 + 2 * i, 0x4100 + i as u16);
        rom_word(&loaded, 0x9b9800 + 2 * i, 0x6200 + i as u16);
    }
    (loaded, native)
}
unsafe fn motion(direction: u32, amount: i32, g: &mut Game) {
    put(g, w::SAMUS_COLLISION_DIRECTION, direction as u16);
    {
        match direction {
            0 => crate::game::player_collision_adapters::move_left(-amount, g),
            1 => crate::game::player_collision_adapters::move_right(amount, g),
            2 => crate::game::player_collision_adapters::move_up(-amount, g),
            3 => crate::game::player_collision_adapters::move_down(amount, g),
            _ => unreachable!(),
        }
    }
}
fn publication(g: &Game, direction: u32, amount: i32) {
    for d in 0..4 {
        let at = 0xdaa + 4 * d;
        if d == direction as usize {
            expect(g, at, (amount >> 16) as u16, "published whole");
            expect(g, at + 2, amount as u16, "published fraction");
        } else {
            expect(g, at, 0x7777, "other whole retained");
            expect(g, at + 2, 0x7777, "other fraction retained");
        }
    }
}
/// Runs actual predecessor bank entries; candidate redirects only these entries.
/// Return bytes: full WRAM, optional return marker/u32, g_fail, cancellation trace.
pub(crate) unsafe fn run(id: u32, g: &mut Game) -> Vec<u8> {
    assert!(id < COUNT);
    let (_loaded, _native) = seed(g);
    let mut returned = None::<u32>;
    let mut checkpoints = Vec::new();
    CANCEL_VALUES.with_borrow_mut(Vec::clear);
    CANCEL_ARMED.with(|v| v.set(id == 14));
    let _cancel_guard = CancelGuard;
    unsafe {
        match id {
            0..=3 => {
                motion(id, 0x10001, g);
                let (axis, fraction, whole, frac) = match id {
                    0 => (w::SAMUS_X_POS, w::SAMUS_X_SUBPOS, 134, 0xffff),
                    1 => (w::SAMUS_X_POS, w::SAMUS_X_SUBPOS, 137, 1),
                    2 => (w::SAMUS_Y_POS, w::SAMUS_Y_SUBPOS, 135, 0x2221),
                    _ => (w::SAMUS_Y_POS, w::SAMUS_Y_SUBPOS, 137, 0x2223),
                };
                expect(g, axis, whole, "clear directional whole");
                expect(g, fraction, frac, "clear directional fraction");
                expect(g, w::SAMUS_COLLISION_FLAG, 0, "clear");
                publication(g, id, if id & 1 == 0 { -0x10001 } else { 0x10001 });
            }
            4..=7 => {
                let d = id - 4;
                let axis = if d < 2 {
                    w::SAMUS_X_POS
                } else {
                    w::SAMUS_Y_POS
                };
                put(g, axis, if d & 1 == 0 { 0 } else { 0xffff });
                put(g, axis + 2, if d & 1 == 0 { 0 } else { 0xffff });
                match d {
                    0 => crate::game::player_collision_adapters::unchecked_left(1, g),
                    1 => crate::game::player_collision_adapters::unchecked_right(1, g),
                    2 => crate::game::player_collision_adapters::unchecked_up(1, g),
                    _ => crate::game::player_collision_adapters::unchecked_down(1, g),
                }
                expect(
                    g,
                    axis,
                    if d & 1 == 0 { 0xffff } else { 0 },
                    "unchecked wrapping whole",
                );
                expect(
                    g,
                    axis + 2,
                    if d & 1 == 0 { 0xffff } else { 0 },
                    "unchecked wrapping fraction",
                );
                publication(g, d, if d & 1 == 0 { -1 } else { 1 });
            }
            8..=11 => {
                let d = id - 8;
                actor(
                    g,
                    0,
                    if d == 0 {
                        130
                    } else if d == 1 {
                        142
                    } else {
                        136
                    },
                    if d == 2 {
                        130
                    } else if d == 3 {
                        142
                    } else {
                        136
                    },
                );
                put(g, w::INTERACTIVE_ENEMY_INDEXES_WRITE_PTR, 2);
                put(g, a::INTERACTIVE_ENEMY_INDEXES, 0);
                put(g, a::INTERACTIVE_ENEMY_INDEXES + 2, 0xffff);
                motion(d, 0x10000, g);
                expect(g, w::SAMUS_X_POS, 136, "exact actor keeps X");
                expect(g, w::SAMUS_Y_POS, 136, "exact actor keeps Y");
                expect(
                    g,
                    w::SAMUS_Y_SUBPOS,
                    0,
                    "exact contact clears Y fraction even for X",
                );
                expect(
                    g,
                    w::SAMUS_COLLISION_FLAG,
                    0xffff,
                    "actor collision representation",
                );
                publication(g, d, 0);
                if id == 8 {
                    checkpoint(&mut checkpoints, g, 0, None);
                    actor(g, 0, 128, 136);
                    put(g, w::SAMUS_Y_SUBPOS, 0x2222);
                    motion(0, 0x10001, g);
                    expect(
                        g,
                        w::SAMUS_X_POS,
                        134,
                        "negative actor probe fractional borrow",
                    );
                    expect(
                        g,
                        w::SAMUS_X_SUBPOS,
                        0,
                        "blocked actor commits integer clearance",
                    );
                    expect(
                        g,
                        w::SAMUS_COLLISION_FLAG,
                        0xffff,
                        "fractional borrow enters strict actor edge",
                    );
                }
            }
            12 | 13 => {
                actor(g, 0, 146, 136);
                actor(g, 64, 144, 136);
                put(g, w::INTERACTIVE_ENEMY_INDEXES_WRITE_PTR, 4);
                put(
                    g,
                    a::INTERACTIVE_ENEMY_INDEXES,
                    if id == 12 { 0 } else { 64 },
                );
                put(
                    g,
                    a::INTERACTIVE_ENEMY_INDEXES + 2,
                    if id == 12 { 64 } else { 0 },
                );
                put(g, a::INTERACTIVE_ENEMY_INDEXES + 4, 0xffff);
                motion(1, 0x50000, g);
                expect(
                    g,
                    w::SAMUS_X_POS,
                    if id == 12 { 140 } else { 138 },
                    "first encountered actor, not nearest",
                );
                expect(g, w::SAMUS_COLLISION_FLAG, 0xffff, "actor order collision");
                expect(
                    g,
                    a::ENEMY_INDEX_COLLIDING_DIRS + 2,
                    if id == 12 { 0 } else { 64 },
                    "encounter identity",
                );
            }
            14 => {
                actor(g, 0, 142, 136);
                put(g, w::INTERACTIVE_ENEMY_INDEXES_WRITE_PTR, 2);
                put(g, a::INTERACTIVE_ENEMY_INDEXES + 2, 0xffff);
                block(g, 264, 9, 0);
                put(g, 0x7b5, 0x9000);
                put(g, w::SAMUS_POSE, 8);
                put(g, 0x998, 8);
                rom_word(&_loaded, 0x8f9000, 0xa000);
                rom_word(&_loaded, 0x83a000, 0x9234);
                put(g, w::CUR_BLOCK_INDEX, 0xbeef);
                put(g, w::SAMUS_HAS_MOMENTUM_FLAG, 1);
                put(g, w::SPEED_BOOST_COUNTER, 0x401);
                put(g, w::SAMUS_X_BASE_SPEED, 7);
                put(g, w::SAMUS_X_EXTRA_RUN_SPEED, 5);
                put(g, w::SPEED_ECHOES_INDEX, 0);
                put(g, w::SAMUS_X_ACCEL_MODE, 2);
                motion(1, 0x10000, g);
                expect(
                    g,
                    w::CUR_BLOCK_INDEX,
                    0xbeef,
                    "actor precedes effectful door terrain",
                );
                CANCEL_VALUES.with_borrow(|v| {
                    assert_eq!(
                        v.as_slice(),
                        &[[7, 5, 2]],
                        "PC10 cancellation observes speed before clears"
                    )
                });
                expect(g, w::SAMUS_HAS_MOMENTUM_FLAG, 0, "boost cancelled");
                expect(g, w::SAMUS_X_BASE_SPEED, 0, "base reset");
                expect(g, w::SAMUS_X_EXTRA_RUN_SPEED, 0, "extra reset");
                expect(g, w::SPEED_ECHOES_INDEX, 0xffff, "echo cancellation");
                for i in 0..16 {
                    expect(
                        g,
                        0xc000 + 2 * (192 + i),
                        0x4100 + i as u16,
                        "actual cancellation loaded native suit colors",
                    );
                }
            }
            15 => {
                block(g, 264, 1, 4);
                motion(3, 0x10001, g);
                expect(g, w::SAMUS_COLLISION_FLAG, 1, "square full quadrant");
                expect(g, w::SAMUS_Y_SUBPOS, 0xffff, "square floor fraction");
                expect(g, 0xdba, 1, "square floor adjustment flag");
            }
            16 => {
                block(g, 264, 1, 7);
                motion(3, 0x10001, g);
                expect(g, w::SAMUS_COLLISION_FLAG, 1, "nonsquare floor");
                expect(g, w::SAMUS_Y_POS, 136, "nonsquare integer-only correction");
                expect(g, w::SAMUS_Y_SUBPOS, 0x2222, "nonsquare retains fraction");
            }
            17 => {
                put(g, w::SAMUS_Y_POS, 139);
                put(g, 0xa46, 2);
                block(g, 264, 1, 0x87);
                crate::game::player_collision_adapters::align_slopes(g);
                expect(g, w::SAMUS_Y_POS, 139, "top equality does not displace");
                expect(g, 0xdba, 1, "top equality still marks slope adjusted");
            }
            18 | 19 => {
                put(g, w::SAMUS_X_POS, 144);
                put(g, w::SAMUS_X_RADIUS, 9);
                put(g, 0x5b6, (id - 18) as u16);
                block(g, 264, 8, 0);
                block(g, 265, 8, 0);
                motion(3, 0x10000, g);
                expect(
                    g,
                    w::CUR_BLOCK_INDEX,
                    if id == 18 { 264 } else { 265 },
                    "frame parity selects first terrain column",
                );
                expect(g, w::SAMUS_COLLISION_FLAG, 1, "solid floor");
            }
            20 | 21 => {
                block(g, 264, 3, 0);
                if id == 21 {
                    put(g, w::SAMUS_CONTACT_DAMAGE_INDEX, 1);
                }
                motion(3, 0x10001, g);
                expect(g, w::SAMUS_Y_POS, 136, "surface whole");
                expect(
                    g,
                    w::SAMUS_Y_SUBPOS,
                    if id == 20 { 0x5223 } else { 0x2222 },
                    "real CollisionSurface displacement",
                );
                expect(
                    g,
                    0x1e71,
                    if id == 20 { 1 } else { 0 },
                    "actual surface sand visits",
                );
                expect(
                    g,
                    w::SAMUS_COLLISION_FLAG,
                    if id == 20 { 1 } else { 0 },
                    "special-air callback low bit ignored for local BTS",
                );
                crate::game::session_runtime::collision_mechanism_checks::run(g, id);
            }
            22 | 23 => {
                put(g, w::SAMUS_X_BASE_SPEED, 5);
                put(g, w::SAMUS_X_BASE_SUBSPEED, 7);
                put(g, w::SAMUS_COLLISION_FLAG, 0xffff);
                put(g, w::EXTRA_SAMUS_X_DISPLACEMENT, 1);
                {
                    use crate::game::{
                        player_collision::operations as collision, player_movement::speed,
                        session_runtime::Runtime,
                    };
                    let mut runtime = Runtime::new(g);
                    let request = if id == 23 {
                        speed::horizontal_air::<true>(&runtime, 0x50007)
                    } else {
                        speed::horizontal_air::<false>(&runtime, 0x50007)
                    };
                    collision::horizontal_control(&mut runtime, request);
                }
                expect(g, w::SAMUS_X_BASE_SPEED, 0, "at-rest base reset");
                expect(g, w::SAMUS_X_BASE_SUBSPEED, 0, "at-rest fraction reset");
                expect(
                    g,
                    w::SAMUS_X_POS,
                    if id == 22 { 136 } else { 137 },
                    "MOVE_AT_REST controls actual external displacement",
                );
                expect(g, w::SAMUS_COLLISION_FLAG, 0, "reset collision");
            }
            24 => {
                put(g, w::SAMUS_X_EXTRA_RUN_SUBSPEED, 0xffff);
                put(g, w::SAMUS_X_SPEED_DIVISOR, 5);
                crate::game::player_collision_adapters::move_x(0x10001, g);
                expect(
                    g,
                    w::SAMUS_X_SUBPOS,
                    0x2000,
                    "actual MoveX divisor capped at4 after carry",
                );
                expect(g, w::SAMUS_TOTAL_X_SUBSPEED, 0x2000, "total published");
                checkpoint(&mut checkpoints, g, 0, None);
                put(g, w::SAMUS_X_EXTRA_RUN_SUBSPEED, 0);
                put(g, w::SAMUS_X_SPEED_DIVISOR, 0);
                put(g, w::SAMUS_X_SUBPOS, 0);
                crate::game::player_collision_adapters::move_x(0x200123, g);
                expect(
                    g,
                    w::SAMUS_X_POS,
                    151,
                    "actual MoveX positive integer clamp",
                );
                expect(g, w::SAMUS_X_SUBPOS, 0x123, "positive clamp fraction");
                checkpoint(&mut checkpoints, g, 1, None);
                g.ram().write_byte(0xa1e, 4);
                put(g, w::SAMUS_X_SUBPOS, 0);
                crate::game::player_collision_adapters::move_x(0x200123, g);
                expect(
                    g,
                    w::SAMUS_X_POS,
                    136,
                    "actual MoveX negative integer clamp",
                );
                expect(g, w::SAMUS_X_SUBPOS, 0xfedd, "negative clamp fraction");
            }
            25 | 26 | 31 => {
                put(g, w::SAMUS_Y_POS, 133);
                put(g, w::SAMUS_Y_SUBPOS, 0);
                let amount = if id == 25 {
                    7
                } else if id == 26 {
                    8
                } else {
                    9
                };
                block(g, 264, if id == 25 { 0 } else { 8 }, 0);
                block(g, 296, if id == 25 { 8 } else { 0 }, 0);
                let mut value =
                    crate::game::player_collision_adapters::pose_clearance(amount << 16, g);
                assert_eq!(
                    value,
                    if id == 25 { 7 << 16 } else { 0 },
                    "abs7/8/9 snapped first probe"
                );
                expect(g, w::CUR_BLOCK_INDEX, 264, "first probe block");
                if id == 31 {
                    checkpoint(&mut checkpoints, g, 0, Some(value as u32));
                    block(g, 264, 0, 0);
                    block(g, 296, 8, 0);
                    let second = crate::game::player_collision_adapters::pose_clearance(9 << 16, g);
                    assert_eq!(
                        second,
                        8 << 16,
                        "clear snapped probe proceeds to full9 contact"
                    );
                    expect(g, w::CUR_BLOCK_INDEX, 296, "second probe reaches next row");
                    value = second;
                }
                expect(
                    g,
                    w::SAMUS_Y_POS,
                    133,
                    "pose probe does not commit position",
                );
                returned = Some(value as u32);
            }
            27 => {
                g.ram().write_byte(0xa1e, 4);
                put(g, w::EXTRA_SAMUS_X_DISPLACEMENT, 3);
                crate::game::player_collision_adapters::move_x(0x10000, g);
                expect(
                    g,
                    w::SAMUS_X_POS,
                    138,
                    "left intention, positive final displacement",
                );
                expect(
                    g,
                    w::SAMUS_COLLISION_DIRECTION,
                    0,
                    "requested direction retained",
                );
                publication(g, 1, 0x20000);
            }
            28 => {
                put(g, w::SAMUS_X_EXTRA_RUN_SPEED, 14);
                put(g, w::SAMUS_X_EXTRA_RUN_SUBSPEED, 0xffff);
                put(g, w::SAMUS_Y_SUBACCEL, 1);
                crate::game::player_collision_adapters::shinespark_x(g);
                expect(g, w::SAMUS_X_POS, 151, "shinespark clamps at15");
                expect(
                    g,
                    w::SAMUS_X_EXTRA_RUN_SUBSPEED,
                    0,
                    "run clamp clears fraction",
                );
                expect(g, w::SAMUS_SHINE_TIMER, 15, "shine timer");
                checkpoint(&mut checkpoints, g, 0, None);
                actor(g, 0, 169, 136);
                put(g, w::INTERACTIVE_ENEMY_INDEXES_WRITE_PTR, 2);
                put(g, a::INTERACTIVE_ENEMY_INDEXES + 2, 0xffff);
                crate::game::player_collision_adapters::shinespark_x(g);
                expect(
                    g,
                    w::SAMUS_X_POS,
                    151,
                    "shinespark actor block does not commit clearance",
                );
                expect(
                    g,
                    w::SAMUS_COLLISION_FLAG,
                    0xffff,
                    "shinespark X actor block",
                );
            }
            29 => {
                put(g, w::SAMUS_Y_SPEED, 13);
                put(g, w::SAMUS_Y_SUBSPEED, 0xffff);
                put(g, w::SAMUS_Y_SUBACCEL, 1);
                put(g, 0xdec, 0);
                put(g, 0xdee, 0xffff);
                crate::game::player_collision_adapters::shinespark_y(g);
                expect(g, 0xdee, 0, "shared sequence fraction carry");
                expect(g, 0xdec, 1, "shared sequence whole carry");
                expect(
                    g,
                    w::SAMUS_Y_POS,
                    121,
                    "14.ffff upward displacement borrows",
                );
                expect(g, w::SAMUS_Y_SUBPOS, 0x2223, "shinespark fraction");
                expect(g, w::SAMUS_PREV_Y_POS, 135, "previous Y bounded to14");
                checkpoint(&mut checkpoints, g, 0, None);
                actor(g, 0, 136, 105);
                put(g, w::INTERACTIVE_ENEMY_INDEXES_WRITE_PTR, 2);
                put(g, a::INTERACTIVE_ENEMY_INDEXES + 2, 0xffff);
                crate::game::player_collision_adapters::shinespark_y(g);
                expect(
                    g,
                    w::SAMUS_Y_POS,
                    121,
                    "shinespark Y actor block does not commit clearance",
                );
                expect(
                    g,
                    w::SAMUS_COLLISION_FLAG,
                    0xffff,
                    "shinespark Y actor block",
                );
            }
            30 => {
                put(g, 0x1e71, 9);
                put(g, 0xdba, 7);
                put(g, w::SAMUS_COLLISION_FLAG, 0xffff);
                motion(1, 0, g);
                expect(g, w::SAMUS_COLLISION_FLAG, 0, "zero X clears collision");
                expect(g, 0x1e71, 9, "zero X retains quicksand");
                expect(g, 0xdba, 7, "zero X retains slope marker");
                checkpoint(&mut checkpoints, g, 0, None);
                motion(3, 0, g);
                expect(
                    g,
                    w::SAMUS_COLLISION_FLAG,
                    1,
                    "zero Y publishes retained sand",
                );
                expect(g, 0x1e71, 9, "zero Y retains quicksand");
                expect(g, 0xdba, 7, "zero Y retains slope marker");
            }
            32..=35 => {
                put(g, w::SAMUS_X_BASE_SPEED, 1);
                put(g, w::SAMUS_X_BASE_SUBSPEED, 0x1234);
                put(g, w::EXTRA_SAMUS_X_DISPLACEMENT, 2);
                put(g, w::SAMUS_Y_DIR, 2);
                g.ram().write_byte(0xa1f, if id == 33 { 20 } else { 3 });
                match id {
                    32 | 33 => crate::game::player_movement::speed::jumping(
                        &mut crate::game::session_runtime::Runtime::new(g),
                    ),
                    34 => {
                        put(g, w::JOYPAD1_LASTKEYS, 0x200);
                        crate::game::player_movement::speed::spin_jumping(
                            &mut crate::game::session_runtime::Runtime::new(g),
                        );
                    }
                    _ => crate::game::session_runtime::advance_special_movement(0x90946e, g),
                }
                expect(
                    g,
                    w::SAMUS_X_POS,
                    match id {
                        33 => 139,
                        34 => 138,
                        _ => 136,
                    },
                    "actual controller reset versus forced movement",
                );
                expect(
                    g,
                    w::SAMUS_X_BASE_SPEED,
                    if id == 33 { 1 } else { 0 },
                    "controller base speed",
                );
                expect(
                    g,
                    w::SAMUS_X_ACCEL_MODE,
                    if id == 33 { 2 } else { 0 },
                    "controller acceleration result",
                );
            }
            _ => unreachable!(),
        }
    }
    assert!(!g.g_fail, "movement case{id} native failure");
    let mut out = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    out.push(u8::from(returned.is_some()));
    out.extend_from_slice(&returned.unwrap_or(0).to_le_bytes());
    out.push(u8::from(g.g_fail));
    CANCEL_VALUES.with_borrow(|values| {
        out.extend_from_slice(&(values.len() as u32).to_le_bytes());
        for row in values {
            for value in row {
                out.extend_from_slice(&value.to_le_bytes());
            }
        }
    });
    out.extend_from_slice(&(checkpoints.len() as u32).to_le_bytes());
    out.extend(checkpoints);
    out
}

/// Same preserved PC10 triplet observed before the actual owned cancellation.
pub(crate) fn observe_owned_cancel(values: [u16; 3]) {
    if CANCEL_ARMED.with(|v| v.get()) {
        CANCEL_VALUES.with_borrow_mut(|v| v.push(values));
    }
}
