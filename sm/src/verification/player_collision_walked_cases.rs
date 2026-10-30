//! Actual walked-obstacle predecessor fixtures; no collision implementation.
use crate::{game_state::Game, ram_map::words as w, rom::RomImage, types::EnemyData};
use core::mem::offset_of;

pub(crate) const COUNT: u32 = 9;
fn put(g: &mut Game, at: usize, value: u16) {
    g.ram().copy_in(at, &value.to_le_bytes());
}
fn read(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes([g.ram().read_byte(at), g.ram().read_byte(at + 1)])
}
fn check(g: &Game, at: usize, value: u16, label: &str) {
    assert_eq!(read(g, at), value, "{label}: word {at:04x}");
}
fn pose(image: &RomImage, key: u16, movement: u8, shots: u8) {
    let at = crate::rtl::lorom_offset(0x91b629) as usize + usize::from(key) * 8;
    image.write(at + 1, movement);
    image.write(at + 3, shots);
}
fn transition(image: &RomImage, index: usize, value: u16) {
    let at = crate::rtl::lorom_offset(0x91eb74) as usize + index * 2;
    for (i, byte) in value.to_ne_bytes().into_iter().enumerate() {
        image.write(at + i, byte);
    }
}
fn actor(g: &mut Game, x: u16) {
    for (field, value) in [
        (offset_of!(EnemyData, enemy_ptr), 0x9000),
        (offset_of!(EnemyData, x_pos), x),
        (offset_of!(EnemyData, y_pos), 136),
        (offset_of!(EnemyData, x_width), 4),
        (offset_of!(EnemyData, y_height), 6),
        (offset_of!(EnemyData, frozen_timer), 1),
    ] {
        put(g, 0xf78 + field, value);
    }
    put(g, 0x17ec, 0);
    put(g, 0x17ee, 0xffff);
    put(g, 0x17a6, 2);
}
fn solid_column(g: &mut Game, column: usize) {
    for row in 0..32 {
        put(g, 0x10002 + (row * 32 + column) * 2, 0x8000);
    }
}
fn snapshot(out: &mut Vec<u8>, g: &Game) {
    out.extend((0..g.ram().len()).map(|at| g.ram().read_byte(at)));
    out.push(u8::from(g.g_fail));
}

/// Locals0..8 append globals88..96; root retains its initialized hardware host.
pub(crate) unsafe fn run(local: u32, g: &mut Game) -> Vec<u8> {
    assert!(local < COUNT);
    let loaded = RomImage::zeroed(0x200000);
    let native = RomImage::zeroed(0x200000);
    // Both lookup sites use loaded EB74, but index through native B629.
    // Early site uses CURRENT2/index4; late site uses PENDING3/index6.
    pose(&native, 2, 0, 4);
    pose(&native, 3, if local == 5 { 0 } else { 1 }, 6);
    pose(&loaded, 2, 0, 9);
    pose(&loaded, 3, 0, 9);
    transition(&loaded, 4, 0x0123);
    transition(&loaded, 6, 0x0234);
    transition(&loaded, 9, 0x0345);
    transition(&native, 4, 0x0456);
    transition(&native, 6, 0x0567);
    g.ram().fill(0..g.ram().len(), 0);
    g.register_rom(loaded);
    g.register_native_rom(native);
    g.g_fail = false;
    g.VERIFY_ACTIVE = true;
    let left = local >= 6;
    let initial_x = if left { 132 } else { 140 };
    for (at, value) in [
        (w::SAMUS_POSE, 2),
        (w::SAMUS_NEW_POSE, if local == 4 { 0xffff } else { 3 }),
        (
            0xa1e,
            if local == 0 {
                0x0108
            } else if left {
                4
            } else {
                8
            },
        ),
        (
            w::SAMUS_COLLIDES_WITH_SOLID_ENEMY,
            if local == 0 { 1 } else { 0x7777 },
        ),
        (w::SAMUS_X_POS, initial_x),
        (w::SAMUS_X_SUBPOS, 0),
        (w::SAMUS_Y_POS, 136),
        (w::SAMUS_Y_SUBPOS, 0x2222),
        (w::SAMUS_X_RADIUS, 4),
        (w::SAMUS_Y_RADIUS, 6),
        (w::ROOM_WIDTH_IN_BLOCKS, 32),
        (w::SAMUS_COLLISION_DIRECTION, 0x9999),
        (w::SAMUS_COLLISION_FLAG, 0x1234),
        (w::COLLISION_DETECTION_INDEX, 0x5555),
        (w::INPUT_TO_POSE_CALC, 0x6666),
    ] {
        put(g, at, value);
    }
    if matches!(local, 1 | 6) {
        actor(g, if left { 124 } else { 148 });
    }
    if matches!(local, 2 | 8) {
        solid_column(g, if left { 7 } else { 9 });
    }
    let mut out = b"PCWALK1".to_vec();
    out.extend_from_slice(&local.to_le_bytes());
    snapshot(&mut out, g);
    {
        crate::game::player_collision_adapters::walked_obstacle(g)
    };
    let early = matches!(local, 0 | 4 | 5);
    let blocked_actor = matches!(local, 1 | 6);
    let blocked_terrain = matches!(local, 2 | 8);
    let clear = matches!(local, 3 | 7);
    let expected_pose = match local {
        0 => 0x0123,
        1 | 2 | 6 | 8 => 0x0234,
        4 => 0xffff,
        _ => 3,
    };
    check(g, w::SAMUS_POSE, 2, "player.walked.current_unchanged");
    check(
        g,
        w::SAMUS_NEW_POSE,
        expected_pose,
        "player.walked.current_pending_and_sources",
    );
    check(
        g,
        w::SAMUS_COLLIDES_WITH_SOLID_ENEMY,
        0,
        "player.walked.marker_clear",
    );
    check(
        g,
        w::SAMUS_COLLISION_DIRECTION,
        if early { 0x9999 } else { u16::from(!left) },
        "player.walked.direction",
    );
    check(
        g,
        w::SAMUS_COLLISION_FLAG,
        if early || blocked_actor {
            0x1234
        } else {
            u16::from(blocked_terrain)
        },
        "player.walked.collision_publication",
    );
    check(
        g,
        w::COLLISION_DETECTION_INDEX,
        if blocked_actor { 0 } else { 0x5555 },
        "player.walked.actor_selection",
    );
    check(
        g,
        w::SAMUS_X_POS,
        if clear {
            if left { 131 } else { 141 }
        } else {
            initial_x
        },
        "player.walked.coordinate",
    );
    check(
        g,
        w::SAMUS_X_SUBPOS,
        if local == 2 { 0xffff } else { 0 },
        "player.walked.fraction",
    );
    check(g, w::SAMUS_Y_POS, 136, "player.walked.y_unchanged");
    check(
        g,
        w::SAMUS_Y_SUBPOS,
        if blocked_actor { 0 } else { 0x2222 },
        "player.walked.exact_actor_fraction",
    );
    check(
        g,
        w::INPUT_TO_POSE_CALC,
        0x6666,
        "player.walked.input_unchanged",
    );
    assert!(!g.g_fail, "player.walked.no_diagnostic");
    snapshot(&mut out, g);
    out
}
