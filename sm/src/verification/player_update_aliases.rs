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
    game.ram().fill(0..game.ram().len(), 0);
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
        (w::ROOM_X_COORDINATE_ON_MAP, map_x),
        (w::ROOM_Y_COORDINATE_ON_MAP, map_y),
        (w::ROOM_WIDTH_IN_BLOCKS, 0xffff),
        (w::ROOM_HEIGHT_IN_BLOCKS, 0xffff),
        (w::SAMUS_NEW_POSE, 0xffff),
        (w::SAMUS_NEW_POSE_INTERRUPTED, 0xffff),
        (w::SAMUS_NEW_POSE_TRANSITIONAL, 0xffff),
        (w::TIMER_FOR_SHINE_TIMER, 10),
        (w::SAMUS_VISOR_PALETTE_TIMER_INDEX, 1537),
        (w::SAMUS_ANIM_FRAME_TIMER, 2),
        (w::SAMUS_HEALTH, 99),
    ] {
        game.ram().copy_in(at, &(value as u16).to_le_bytes());
    }
    game.ram().write_byte(0x5b5, 8);
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
        (w::FRAME_HANDLER_GAMMA, 0xe419),
        (w::SAMUS_POSE, 0x29),
        (w::SAMUS_PREV_POSE, 0x29),
        (w::SAMUS_Y_SPEED, 4),
        (w::SAMUS_Y_SUBSPEED, 0xf800),
        (w::SAMUS_Y_SUBACCEL, 0x1000),
    ] {
        game.ram().copy_in(at, &(value as u16).to_le_bytes());
    }
    let mut expected = (0..game.ram().len())
        .map(|at| game.ram().read_byte(at))
        .collect::<Vec<u8>>();
    // Authored neighboring bytes are zero except the selected gamma word.
    // Marked byte1B supplies columns28..31. Column32 crosses to the other
    // map half (exploration byte224, zero), so the five bits are10110.
    hud(&mut expected, [0x2800, 0x2c1f, 0x2800, 0x2800, 0x2c1f]);
    for (at, value) in [
        (w::FRAME_HANDLER_GAMMA, 0xe41b),
        (w::SAMUS_Y_SPEED, 5),
        (w::SAMUS_Y_SUBSPEED, 0x0800),
        (w::SAMUS_ANIM_FRAME_TIMER, 15),
        (w::SAMUS_ANIM_FRAME, 4),
    ] {
        put(&mut expected, at, value);
    }
    {
        let mut runtime = Runtime::new(game);
        assert_eq!(runtime.word(w::FRAME_HANDLER_GAMMA), 0xe419);
        assert_eq!(
            player_update::advance(
                &mut runtime,
                crate::game::player_transitions::state::FrameBetaMode::Gameplay,
                MovementStep::AlreadyAdvanced
            ),
            Ok(())
        );
        assert_eq!(
            runtime.word(w::FRAME_HANDLER_GAMMA),
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
        &(0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<u8>>()[..],
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
    let before = (0..game.ram().len())
        .map(|at| game.ram().read_byte(at))
        .collect::<Vec<u8>>();
    let mut expected = before.clone();
    put(&mut expected, w::SAMUS_X_POS, 1);
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
        session_encoding::encode(runtime.session(), |at, span| {
            projected[at..at + span.len()].copy_from_slice(span)
        });
        assert_eq!(
            projected, expected,
            "live Session projection contains both alias and resulting HUD tile"
        );
    }
    assert_eq!(
        &(0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<u8>>()[..],
        expected.as_slice(),
        "owned position alias survives publication without a RAM shadow"
    );
    assert!(!game.g_fail && !game.actor_identity_is_with_session());
}

pub(crate) fn run(game: &mut Game) -> usize {
    assert!(!game.actor_identity_is_with_session());
    let ram = (0..game.ram().len())
        .map(|at| game.ram().read_byte(at))
        .collect::<Vec<u8>>();
    let fail = game.g_fail;
    let loaded = game.loaded_rom.clone();
    let native = game.capture_native_rom_binding();
    let result = catch_unwind(AssertUnwindSafe(|| {
        late_gamma(game);
        owned_position(game);
        COUNT
    }));
    game.ram().copy_in(0, &ram);
    game.g_fail = fail;
    game.loaded_rom = loaded;
    game.restore_native_rom_binding(native);
    match result {
        Ok(count) => count,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}
