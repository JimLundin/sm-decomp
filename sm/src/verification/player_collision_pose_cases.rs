//! Source-only actual predecessor fixtures at 50cec3e0. No collision implementation.
use crate::{game_state::Game, ram_map::words as w, rom::RomImage, types::EnemyData};
use core::mem::offset_of;

pub(crate) const COUNT: u32 = 24;
const X: usize = 0xaf6;
const Y: usize = 0xafa;
const RADIUS: usize = 0xb00;
const DIRECTION: usize = 0xb02;
const INPUT: usize = 0xdc6;
const COLLISION: usize = 0xdd0;
const SOLID: usize = 0xa34;
const BLOCK: usize = 0xa36;
const UP_BLOCK: usize = 0xa38;
const DIFFERENCE: usize = 0xa3a;
const DOWN_BLOCK: usize = 0xa3c;
const UP_ACTOR: usize = 0xa3e;
const DOWN_ACTOR: usize = 0xa40;

fn put(g: &mut Game, at: usize, value: u16) {
    g.ram().copy_in(at, &value.to_le_bytes());
}
fn read(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes([g.ram().read_byte(at), g.ram().read_byte(at + 1)])
}
fn check(g: &Game, expected: &[(usize, u16)], label: &str) {
    for &(at, value) in expected {
        assert_eq!(read(g, at), value, "{label}: word {at:04x}");
    }
}
fn pose_radius(image: &RomImage, pose: u16, radius: u8) {
    let at = crate::rtl::lorom_offset(0x91b629) as usize + usize::from(pose) * 8 + 6;
    image.write(at, radius);
}
fn actor(g: &mut Game, slot: usize, x: u16, y: u16) {
    let key = slot * 64;
    for (field, value) in [
        (offset_of!(EnemyData, enemy_ptr), 0x9000),
        (offset_of!(EnemyData, x_pos), x),
        (offset_of!(EnemyData, y_pos), y),
        (offset_of!(EnemyData, x_width), 4),
        (offset_of!(EnemyData, y_height), 6),
        (offset_of!(EnemyData, frozen_timer), 1),
    ] {
        put(g, 0xf78 + key + field, value);
    }
    put(g, 0x17ec + slot * 2, key as u16);
    put(g, 0x17ec + (slot + 1) * 2, 0xffff);
    put(g, 0x17a6, (slot as u16 + 1) * 2);
}
fn solid_row(g: &mut Game, row: usize) {
    for column in 0..32 {
        put(g, 0x10002 + (row * 32 + column) * 2, 0x8000);
    }
}
fn solid_column(g: &mut Game, column: usize) {
    for row in 0..32 {
        put(g, 0x10002 + (row * 32 + column) * 2, 0x8000);
    }
}
fn seed(id: u32, g: &mut Game, loaded: &RomImage, native: &RomImage) {
    // Full initialized allocations: never manufacture an out-of-bounds native pointer.
    g.ram().fill(0..g.ram().len(), 0);
    g.register_rom(loaded.clone());
    g.register_native_rom(native.clone());
    g.g_fail = false;
    g.VERIFY_ACTIVE = true;
    for (at, value) in [
        (X, 136),
        (Y, 136),
        (X + 2, 0),
        (Y + 2, 0),
        (0xafe, 4),
        (RADIUS, 6),
        (0x7a5, 32),
        (0xa1c, 2),
        (0xa1e, 8),
        (0xa20, 1),
        (0xa24, 3),
        (0xa26, 0x0308),
        (0xa94, 0xbeef),
        (0xa96, 11),
        (0xb14, 0x3333),
        (DIRECTION, 0x9999),
        (INPUT, 0x4444),
        (COLLISION, 0x1234),
        (SOLID, 0x6666),
        (BLOCK, 0x7777),
        (DIFFERENCE, 0x8888),
        (w::enemy_index_to_shake, 0x5555),
        (w::button_config_jump_a, 0x0080),
        (w::joypad1_newkeys, 0x0080),
        (w::joypad1_lastkeys, 0x0300),
    ] {
        put(g, at, value);
    }
    for (pose, radius) in [(1, 6), (2, 10), (3, 14), (0x27, 6), (0x28, 6)] {
        pose_radius(native, pose, radius);
        // Wrong source reverses the old/new-radius comparison and must fail case13.
        pose_radius(loaded, pose, if pose == 1 { 20 } else { 2 });
    }
    if id <= 10 || matches!(id, 20 | 21) {
        put(g, Y + 2, 0x2222);
    }
    match id {
        0 => put(g, 0xa26, 0x0108),
        1 => put(g, w::joypad1_lastkeys, 0),
        2..=7 => {
            actor(
                g,
                0,
                match id {
                    5 => 151,
                    6 => 152,
                    _ => 144,
                },
                136,
            );
            if id == 2 {
                put(g, 0xa96, 10);
            }
            if id == 3 {
                put(g, 0xa1c, 0x81);
                put(g, 0xa96, 26);
            }
            if id == 7 {
                put(g, w::joypad1_newkeys, 0);
            }
        }
        8 | 9 => {
            put(g, 0xa1e, 4);
            actor(g, 0, 128, 136);
            if id == 9 {
                put(g, w::joypad1_newkeys, 0);
            }
        }
        10 => put(g, 0xa1e, 0),
        11 => put(g, 0xa1c, 0),
        12 => {
            pose_radius(native, 1, 10);
            pose_radius(native, 2, 6);
            put(g, RADIUS, 10);
        }
        13 => {}
        14 => actor(g, 0, 136, 122),
        15 => actor(g, 0, 136, 150),
        16 => {
            actor(g, 0, 136, 122);
            actor(g, 1, 136, 150);
        }
        17 => solid_row(g, 7),
        18 => solid_row(g, 9),
        19 => {
            pose_radius(native, 1, 8);
            pose_radius(native, 2, 12);
            put(g, RADIUS, 8);
            solid_row(g, 7);
            solid_row(g, 9);
        }
        20 => solid_column(g, 9),
        21 => {
            put(g, 0xa1e, 4);
            solid_column(g, 7);
        }
        // Only the expanded-radius opposite probe hits the second actor.
        22 => {
            actor(g, 0, 136, 122);
            actor(g, 1, 136, 153);
        }
        23 => {
            actor(g, 0, 136, 150);
            actor(g, 1, 136, 119);
        }
        _ => unreachable!(),
    }
}

fn assert_jump(id: u32, returned: u8, g: &Game) {
    let expected_return = u8::from(matches!(id, 4 | 5 | 8 | 20 | 21));
    assert_eq!(
        returned, expected_return,
        "player.pose.jump_admission case{id}"
    );
    let shake = if matches!(id, 0 | 2 | 3) {
        0x5555
    } else if matches!(id, 4 | 5 | 8) {
        0
    } else {
        0xffff
    };
    let direction = if matches!(id, 0 | 1 | 10) {
        0x9999
    } else if matches!(id, 8 | 9) {
        0
    } else if matches!(id, 20 | 21) {
        15
    } else {
        1
    };
    check(
        g,
        &[
            (w::enemy_index_to_shake, shake),
            (DIRECTION, direction),
            (INPUT, if matches!(id, 4 | 5 | 20) { 5 } else { 0x4444 }),
            (X, 136),
            (Y, 136),
            (RADIUS, 6),
            (COLLISION, if matches!(id, 20 | 21) { 1 } else { 0x1234 }),
            (0xa94, if matches!(id, 2 | 3) { 1 } else { 0xbeef }),
            (
                0xa96,
                match id {
                    2 => 10,
                    3 => 26,
                    _ => 11,
                },
            ),
            (
                Y + 2,
                if matches!(id, 2 | 3 | 4 | 7 | 8 | 9) {
                    0
                } else {
                    0x2222
                },
            ),
            (X + 2, if id == 20 { 0xffff } else { 0 }),
        ],
        "player.pose.jump_state",
    );
}

fn assert_pose(id: u32, g: &Game) {
    check(
        g,
        &[(X, 136), (INPUT, 0x4444), (0xa20, 1), (0xa24, 3)],
        "player.pose.distinct_history",
    );
    if id == 11 {
        check(
            g,
            &[
                (SOLID, 0x6666),
                (BLOCK, 0x7777),
                (DIFFERENCE, 0x8888),
                (RADIUS, 6),
                (0xa1c, 0),
            ],
            "player.pose.front_pose_skip",
        );
        return;
    }
    if id == 12 {
        check(
            g,
            &[
                (SOLID, 0),
                (BLOCK, 0),
                (DIFFERENCE, 0x8888),
                (RADIUS, 10),
                (Y, 136),
                (0xa1c, 2),
            ],
            "player.pose.shrink_no_probe",
        );
        return;
    }
    let solid = match id {
        14 | 22 => 1,
        15 | 23 => 2,
        16 => 3,
        _ => 0,
    };
    let block = match id {
        17 => 1,
        18 => 2,
        19 => 3,
        _ => 0,
    };
    let y = match id {
        14 | 17 => 138,
        15 | 18 => 134,
        _ => 136,
    };
    let pose = if matches!(id, 16 | 22 | 23) {
        1
    } else if id == 19 {
        0x27
    } else {
        2
    };
    check(
        g,
        &[
            (SOLID, solid),
            (BLOCK, block),
            (DIFFERENCE, 4),
            (Y, y),
            (0xa1c, pose),
            (RADIUS, if id == 19 { 8 } else { 6 }),
            (
                0xb14,
                if matches!(id, 14 | 15 | 17 | 18) {
                    y
                } else {
                    0x3333
                },
            ),
            (
                COLLISION,
                if matches!(id, 22 | 23) {
                    0xffff
                } else if id == 19 {
                    1
                } else {
                    0
                },
            ),
            (
                DIRECTION,
                if id == 22 {
                    3
                } else if id == 23 {
                    2
                } else {
                    15
                },
            ),
        ],
        "player.pose.radius_restored_and_opposite_probe",
    );
    check(
        g,
        &[
            (UP_ACTOR, if matches!(id, 14 | 16 | 22) { 2 } else { 4 }),
            (DOWN_ACTOR, if matches!(id, 15 | 16 | 23) { 2 } else { 4 }),
        ],
        "player.pose.actor_clearance",
    );
    if matches!(id, 22 | 23) {
        check(
            g,
            &[(UP_BLOCK, 0), (DOWN_BLOCK, 0)],
            "player.pose.no_terrain_after_rejection",
        );
    } else {
        check(
            g,
            &[
                (
                    UP_BLOCK,
                    if id == 17 {
                        2
                    } else if id == 19 {
                        0
                    } else {
                        4
                    },
                ),
                (
                    DOWN_BLOCK,
                    if id == 18 {
                        2
                    } else if id == 19 {
                        0
                    } else {
                        4
                    },
                ),
            ],
            "player.pose.terrain_clearance",
        );
    }
}

fn snapshot(out: &mut Vec<u8>, g: &Game) {
    out.extend((0..g.ram().len()).map(|at| g.ram().read_byte(at)));
    out.push(u8::from(g.g_fail));
}

/// Root maps local0..23 to room-world52..75; existing case IDs are unchanged.
/// This must be linked only to the real predecessor source, never declaration stubs.
pub(crate) unsafe fn run(local_id: u32, g: &mut Game) -> Vec<u8> {
    assert!(local_id < COUNT);
    let loaded = RomImage::zeroed(0x200000);
    let native = RomImage::zeroed(0x200000);
    seed(local_id, g, &loaded, &native);
    let mut out = b"PCPOSE1".to_vec();
    out.extend_from_slice(&local_id.to_le_bytes());
    snapshot(&mut out, g);
    let returned = {
        match local_id {
            0..=7 | 20 => crate::game::player_collision::verification::wall_jump(9 << 16, g),
            8..=10 => crate::game::player_collision::verification::grapple_jump(9 << 16, g),
            21 => crate::game::player_collision::verification::grapple_jump(-(9 << 16), g),
            _ => {
                crate::game::player_collision::verification::fit_pose(g);
                0
            }
        }
    };
    assert!(!g.g_fail, "player.pose.no_diagnostic case{local_id}");
    if local_id <= 10 || matches!(local_id, 20 | 21) {
        assert_jump(local_id, returned, g);
    } else {
        assert_pose(local_id, g);
    }
    out.push(returned);
    snapshot(&mut out, g);
    if local_id == 3 {
        // Same actual screw-contact route on both sides of readiness26/27.
        put(g, 0xa96, 27);
        out.extend_from_slice(&27u16.to_le_bytes());
        let ready = { crate::game::player_collision::verification::wall_jump(9 << 16, g) };
        assert_eq!(ready, 1, "player.pose.screw_ready_at27");
        check(
            g,
            &[
                (0xa96, 27),
                (0xa94, 1),
                (INPUT, 5),
                (w::enemy_index_to_shake, 0),
                (DIRECTION, 1),
            ],
            "player.pose.screw_ready_state",
        );
        out.push(ready);
        snapshot(&mut out, g);
    }
    out
}
