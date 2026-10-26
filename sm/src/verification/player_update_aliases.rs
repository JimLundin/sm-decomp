//! Authored exceptional map aliases through the actual Runtime and coordinator.
//! These are physical-byte contracts, not comparisons with unaligned old Rust.
#![forbid(unsafe_code)]
use super::Runtime;
use crate::{
    game::{
        player_update::{self, MovementStep, minimap},
        session_encoding,
    },
    game_state::Game,
    ram_map::words as w,
    rom::RomImage,
};
use std::panic::{AssertUnwindSafe, catch_unwind};

pub(crate) const COUNT: usize = 2;

fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn author(image: &RomImage, address: u32, bytes: &[u8]) {
    let at = crate::rom::lorom_offset(address) as usize;
    for (index, &byte) in bytes.iter().enumerate() {
        image.write(at + index, byte);
    }
}
fn seed(game: &mut Game, map_x: u16, map_y: u16) {
    game.g_ram.fill(0);
    game.g_fail = false;
    let loaded = RomImage::zeroed(0x100000);
    let native = RomImage::zeroed(0x100000);
    // Authored distinct mask and tile sources, both containing only zeros.
    author(&loaded, 0x829717, &0xa000u16.to_ne_bytes());
    author(&native, 0x82964a, &[0x00, 0x80, 0x83, 0x00]);
    // The actual settlement palette restores an authored all-zero suit.
    author(&loaded, 0x91d727, &0x9000u16.to_ne_bytes());
    game.register_rom(loaded);
    game.register_native_rom(native);
    for (at, value) in [
        (w::room_x_coordinate_on_map, map_x),
        (w::room_y_coordinate_on_map, map_y),
        (w::room_width_in_blocks, 0xffff),
        (w::room_height_in_blocks, 0xffff),
        (w::samus_new_pose, 0xffff),
        (w::samus_new_pose_interrupted, 0xffff),
        (w::samus_new_pose_transitional, 0xffff),
        (w::timer_for_shine_timer, 10),
        (w::samus_visor_palette_timer_index, 1537),
        (w::samus_anim_frame_timer, 2),
        (w::samus_health, 99),
    ] {
        put(&mut game.g_ram, at, value);
    }
    game.g_ram[0x5b5] = 8;
}
fn hud(expected: &mut [u8], middle: [u16; 5]) {
    for row in 0..3 {
        for column in 0..5 {
            put(
                expected,
                0xc608 + 2 * (26 + column + 32 * row),
                if row == 1 { middle[column] } else { 0x2c1f },
            );
        }
    }
}

fn late_gamma(game: &mut Game) {
    // Physical exploration offset611 = 4*152+3 lands at gamma byte A5A.
    // Column30 marks mask02: E419 becomes actual Func9 selector E41B.
    // E419 is deliberately not dispatched; no invalid legacy path is executed.
    seed(game, 30, 151);
    for (at, value) in [
        (w::frame_handler_gamma, 0xe419),
        (w::samus_pose, 0x29),
        (w::samus_prev_pose, 0x29),
        (w::samus_y_speed, 4),
        (w::samus_y_subspeed, 0xf800),
        (w::samus_y_subaccel, 0x1000),
    ] {
        put(&mut game.g_ram, at, value);
    }
    let mut expected = game.g_ram.to_vec();
    // Authored neighboring bytes are zero except the selected gamma word.
    // Marked byte1B supplies columns28..31. Column32 crosses to the other
    // map half (exploration byte224, zero), so the five bits are10110.
    hud(&mut expected, [0x2800, 0x2c1f, 0x2800, 0x2800, 0x2c1f]);
    for (at, value) in [
        (w::frame_handler_gamma, 0xe41b),
        (w::samus_y_speed, 5),
        (w::samus_y_subspeed, 0x0800),
        (w::samus_anim_frame_timer, 15),
        (w::samus_anim_frame, 4),
    ] {
        put(&mut expected, at, value);
    }
    {
        let mut runtime = Runtime::new(game);
        assert_eq!(runtime.word(w::frame_handler_gamma), 0xe419);
        assert_eq!(
            player_update::advance(
                &mut runtime,
                crate::game::player_transitions::state::FrameBetaMode::Gameplay,
                MovementStep::AlreadyAdvanced
            ),
            Ok(())
        );
        assert_eq!(
            runtime.word(w::frame_handler_gamma),
            0xe41b,
            "map exploration changes gamma before selection"
        );
        assert_eq!(
            runtime.session().player.contact.y_speed.whole,
            5,
            "actual newly selected gamma executes and reimports its result"
        );
        assert_eq!(
            runtime.session().player.animation_clock.remaining,
            15,
            "settlement follows gamma frame initialization"
        );
    }
    assert_eq!(
        game.g_ram.as_slice(),
        expected.as_slice(),
        "actual map-to-gamma-to-settlement full physical publication"
    );
    assert!(!game.g_fail && !game.actor_identity_is_with_session());
}

fn owned_position(game: &mut Game) {
    // Offset767 = 4*191+3 targets AF6, the owned player's X low byte.
    // Column31 marks mask01. That same newly written physical byte is in
    // the middle exploration row, so the map must also render its center tile.
    seed(game, 31, 190);
    let before = game.g_ram.to_vec();
    let mut expected = before.clone();
    put(&mut expected, w::samus_x_pos, 1);
    hud(&mut expected, [0x2c1f, 0x2c1f, 0x2800, 0x2c1f, 0x2c1f]);
    {
        let mut runtime = Runtime::new(game);
        minimap::update(&mut runtime);
        assert_eq!(
            runtime.session().player.x,
            1,
            "exceptional exploration updates the sole owned player position"
        );
        assert_eq!(
            runtime.session().minimap.tiles[1][2],
            0x2800,
            "later exploration read observes the freshly changed owned alias"
        );
        let mut projected = before;
        session_encoding::encode(runtime.session(), &mut projected);
        assert_eq!(
            projected, expected,
            "live Session projection contains both alias and resulting HUD tile"
        );
    }
    assert_eq!(
        game.g_ram.as_slice(),
        expected.as_slice(),
        "owned position alias survives publication without a RAM shadow"
    );
    assert!(!game.g_fail && !game.actor_identity_is_with_session());
}

pub(crate) fn run(game: &mut Game) -> usize {
    assert!(!game.actor_identity_is_with_session());
    let ram = game.g_ram.to_vec();
    let fail = game.g_fail;
    let loaded = game.loaded_rom.clone();
    let native = game.capture_native_rom_binding();
    let result = catch_unwind(AssertUnwindSafe(|| {
        late_gamma(game);
        owned_position(game);
        COUNT
    }));
    game.g_ram.copy_from_slice(&ram);
    game.g_fail = fail;
    game.loaded_rom = loaded;
    game.restore_native_rom_binding(native);
    match result {
        Ok(count) => count,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}
