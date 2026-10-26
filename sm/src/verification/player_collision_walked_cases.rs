//! Actual walked-obstacle predecessor fixtures; no collision implementation.
use crate::{game_state::Game, ram_map::words as w, rom::RomImage, types::EnemyData};
use core::mem::offset_of;

pub(crate) const COUNT: u32 = 9;
fn put(g: &mut Game, at: usize, value: u16) {
    g.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn read(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes([g.g_ram[at], g.g_ram[at + 1]])
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
    out.extend_from_slice(&g.g_ram);
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
    g.g_ram.fill(0);
    g.register_rom(loaded);
    g.register_native_rom(native);
    g.g_fail = false;
    g.VERIFY_ACTIVE = true;
    let left = local >= 6;
    let initial_x = if left { 132 } else { 140 };
    for (at, value) in [
        (w::samus_pose, 2),
        (w::samus_new_pose, if local == 4 { 0xffff } else { 3 }),
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
            w::samus_collides_with_solid_enemy,
            if local == 0 { 1 } else { 0x7777 },
        ),
        (w::samus_x_pos, initial_x),
        (w::samus_x_subpos, 0),
        (w::samus_y_pos, 136),
        (w::samus_y_subpos, 0x2222),
        (w::samus_x_radius, 4),
        (w::samus_y_radius, 6),
        (w::room_width_in_blocks, 32),
        (w::samus_collision_direction, 0x9999),
        (w::samus_collision_flag, 0x1234),
        (w::collision_detection_index, 0x5555),
        (w::input_to_pose_calc, 0x6666),
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
    unsafe { crate::game::player_collision::verification::walked_obstacle(g) };
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
    check(g, w::samus_pose, 2, "player.walked.current_unchanged");
    check(
        g,
        w::samus_new_pose,
        expected_pose,
        "player.walked.current_pending_and_sources",
    );
    check(
        g,
        w::samus_collides_with_solid_enemy,
        0,
        "player.walked.marker_clear",
    );
    check(
        g,
        w::samus_collision_direction,
        if early { 0x9999 } else { u16::from(!left) },
        "player.walked.direction",
    );
    check(
        g,
        w::samus_collision_flag,
        if early || blocked_actor {
            0x1234
        } else {
            u16::from(blocked_terrain)
        },
        "player.walked.collision_publication",
    );
    check(
        g,
        w::collision_detection_index,
        if blocked_actor { 0 } else { 0x5555 },
        "player.walked.actor_selection",
    );
    check(
        g,
        w::samus_x_pos,
        if clear {
            if left { 131 } else { 141 }
        } else {
            initial_x
        },
        "player.walked.coordinate",
    );
    check(
        g,
        w::samus_x_subpos,
        if local == 2 { 0xffff } else { 0 },
        "player.walked.fraction",
    );
    check(g, w::samus_y_pos, 136, "player.walked.y_unchanged");
    check(
        g,
        w::samus_y_subpos,
        if blocked_actor { 0 } else { 0x2222 },
        "player.walked.exact_actor_fraction",
    );
    check(
        g,
        w::input_to_pose_calc,
        0x6666,
        "player.walked.input_unchanged",
    );
    assert!(!g.g_fail, "player.walked.no_diagnostic");
    snapshot(&mut out, g);
    out
}
