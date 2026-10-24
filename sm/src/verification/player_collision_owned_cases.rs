//! Additive candidate-owned Runtime invariants, globals97..99 only.
//! The room-world export supplies a real initialized SnesInit/audio/PPU host.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::{
    game::player_collision_owned_checks as checks, game_state::Game, ram_map::words as ram,
    rom::RomImage, types::EnemyData,
};
use core::mem::offset_of;

pub(crate) const COUNT: u32 = 3;
fn put(g: &mut Game, at: usize, value: u16) {
    g.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn snapshot(out: &mut Vec<u8>, g: &Game) {
    out.extend_from_slice(&g.g_ram);
    out.push(u8::from(g.g_fail));
}
fn seed(g: &mut Game, local: u32) {
    // Authored fixture bytes, as in movement/walked cases. Keep allocated
    // hardware and prepared-world binding; no ROM-derived bytes are retained.
    g.g_ram.fill(0);
    g.g_fail = false;
    g.VERIFY_ACTIVE = true;
    g.register_rom(RomImage::zeroed(0x200000));
    g.register_native_rom(RomImage::zeroed(0x200000));
    for (at, value) in [
        (ram::room_width_in_blocks, 16),
        (ram::room_height_in_blocks, 16),
        (ram::samus_x_pos, 100),
        (ram::samus_y_pos, 100),
        (ram::samus_x_radius, 2),
        (ram::samus_y_radius, 2),
        (ram::speed_echoes_index, 0xffff),
        (ram::equipped_items, 0x20),
    ] {
        put(g, at, value);
    }
    // All actual terrain cells remain empty. The interactive-list count is0;
    // every entry is a real terminator, including the no-actor PC9/A7 fixtures.
    for slot in 0..32 {
        put(g, 0x17ec + slot * 2, 0xffff);
    }
    if local == 1 {
        // Existing ordinary actor codec reads this actual native-layout record.
        // No AI/graphics dispatch is needed for the real padded-overlap query.
        for (field, value) in [
            (offset_of!(EnemyData, enemy_ptr), 0x9000),
            (offset_of!(EnemyData, x_pos), 120),
            (offset_of!(EnemyData, y_pos), 100),
            (offset_of!(EnemyData, x_width), 2),
            (offset_of!(EnemyData, y_height), 2),
            (offset_of!(EnemyData, properties), 0x8000),
        ] {
            put(g, 0xf78 + field, value);
        }
        put(g, 0x17ec, 0);
        put(g, 0x17a6, 2);
    }
}

/// Locals0/1/2 are globals97/98/99; all use the existing GameOwner constructor.
pub(crate) fn run(local: u32, g: &mut Game) -> Vec<u8> {
    assert!(local < COUNT);
    assert!(!g.g_snes.is_null(), "collision.owned.initialized_hardware");
    seed(g, local);
    let mut out = b"PCOWN1".to_vec();
    out.extend_from_slice(&local.to_le_bytes());
    snapshot(&mut out, g);
    match local {
        0 => checks::foreign_sequence_then_move(g),
        1 => checks::same_interval_radius_runtime(g),
        2 => checks::transition_a7_single_commit(g, |game| {
            let mut runtime = crate::game::session_runtime::Runtime::new(game);
            crate::game::player_transitions::verify_momentum_action(
                &mut runtime,
                crate::game::player_transitions::state::MomentumAction::from_raw(7),
            );
        }),
        _ => unreachable!(),
    }
    assert!(!g.g_fail, "collision.owned.final_no_diagnostic");
    snapshot(&mut out, g);
    out
}
