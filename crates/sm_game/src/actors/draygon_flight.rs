//! Draygon's fight intro, wall turrets, setup, goop passes and the
//! instructions its body scripts call (bank $A5:8687-$A5:9E0A).
//! Extended motion words stay native compatibility state until the whole boss migrates.
use crate::actors::boss_lease::BossState;
use crate::actors::draygon_host::Lease;
use crate::actors::draygon_host::{BODY, Draygon};
use crate::actors::session_slots::session_slot;
use crate::enemy_script::EnemyScript;

/// Sprite object 3Dh: Draygon foaming at the mouth, 20h px ahead of and 10h
/// px above the body.
const FOAM: u16 = 0x3d;
const FOAM_X: u16 = 0x20;
const FOAM_Y: u16 = 0x10;
/// Sprite object 18h: a bubble.
const BUBBLE: u16 = 0x18;
const BUBBLE_OFFSET: u16 = 0x10;
const TURRET_PROJECTILE: u16 = 0x8e5e;
const TURRET_SPEED: u16 = 3;
const TURRET_FRAMES: u16 = 0x3f;
/// The fight intro waits 100h frames, then dances for 4D0h.
const INTRO_DELAY: u16 = 0x100;
const INTRO_DANCE: u16 = 0x4d0;
/// The right side reset X is 2A0h right of the left.
const RIGHT_RESET_OFFSET: u16 = 0x2a0;
const INITIAL_SWOOP_Y_ACCELERATION: u16 = 0x18;
/// $A5:A217: the target sprite palettes 1 and 2.
const TARGET_PALETTES: u32 = 0xa5_a217;
const PALETTE_7: u16 = 0xe00;
const BG2_TILEMAP_SIZE: u16 = 0x400;
/// The BG2 scroll follows the body, 1C2h left and C0h up.
const BG2_X_OFFSET: i32 = 0x1c2;
const BG2_Y_OFFSET: i32 = 0xc0;
/// Goop passes fly at Y 180h, oscillating 20h, from X FFB0h, firing within
/// D0h of Samus 10h goops at 1 in 16 frames, and end past X 2A0h or -50h.
const GOOP_Y: u16 = 0x180;
const GOOP_Y_AMPLITUDE: u16 = 0x20;
const GOOP_START_X: u16 = 0xffb0;
const GOOP_RANGE: u16 = 0xd0;
const GOOPS: u16 = 0x10;
const GOOP_CHANCE: u16 = 0xf;
const OFF_SCREEN_RIGHT: u16 = 0x2a0;
const OFF_SCREEN_LEFT: u16 = 0x50;
const RIGHT_RESET_X: u16 = 0x250;
const RESET_Y: u16 = 0xffb0;

/// Start `part`'s left or right instruction list, by `facing_key`'s facing.
pub fn facing_animation<S: crate::actors::session_slots::SlotAccess>(
    facing_key: i32,
    part_key: i32,
    instructions: [u16; 2],
    game: &mut S,
) {
    owned::facing_animation(facing_key, part_key, instructions, game)
}
/// On frames matching `frame_mask`, Draygon foams at the mouth. Was
/// `shed_particle`.
pub fn foam_at_mouth<L: Lease>(k: u16, frame_mask: u16, game: &mut L) {
    game.foreign(|game| {
        owned::foam_at_mouth(
            k,
            frame_mask,
            &mut crate::runtime_access::RuntimeAccess::runtime(game),
        )
    })
}
/// Handle firing wall turret ($A5:87AA): every 40h frames, a random turret of
/// the last four fires unless destroyed. Returns the turret index it then
/// leaves in X, which the breath bubbles read after it. Was `turret`.
pub fn fire_wall_turret<S: owned::Host>(game: &mut S) -> Option<u16> {
    owned::fire_wall_turret(game)
}
/// Draygon body function - fight intro: the initial delay ($A5:871B, with
/// `first`) or the dance ($A5:878B). Was `opening`.
pub fn fight_intro<L: Lease + owned::Host>(first: bool, game: &mut L) {
    game.foreign(|game| {
        entry::fight_intro(
            first,
            &mut crate::runtime_access::RuntimeAccess::runtime(game),
        )
    });
}
/// Initialisation AI - Draygon body ($A5:8687). Actor preparation and
/// completion surround the existing synchronous HDMA install seam.
pub fn initialize<L: Lease>(install_layers: impl FnOnce(&mut L), game: &mut L) {
    game.foreign(|game| {
        entry::prepare_initialization(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    });
    install_layers(game);
    game.foreign(|game| {
        entry::complete_initialization(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    });
}
/// Initialisation AI - Draygon tail ($A5:C599) or, with `arms`, arms ($A5:C5AD).
/// Was `initialize_part`.
pub fn initialize_tail_or_arms<S: owned::Host>(arms: bool, game: &mut S) {
    owned::initialize_tail_or_arms(arms, game)
}

/// Main AI - Draygon body ($A5:86FC, with `body`) or eye ($A5:C486): run the
/// part's function, then the body carries the other parts with it. Was `update`.
pub fn main_ai<L: Lease + owned::Host + crate::actors::draygon_sweep::owned::Host>(
    body: bool,
    _dispatch: impl FnOnce(u32, u16, &mut L::Game),
    _lease_dispatch: impl FnOnce(u32, u16, &mut L),
    game: &mut L,
) {
    game.foreign(|game| {
        entry::main_ai(
            body,
            &mut crate::runtime_access::RuntimeAccess::runtime(game),
        )
    });
}

/// Enemy graphics drawn hook - Draygon - set BG2 X/Y scroll ($A5:9342). Was
/// `scroll_layer`.
pub fn set_bg2_scroll<S: entry::Host>(game: &mut S) {
    entry::set_bg2_scroll(game);
}
/// Move the body right, or left, by its X speed. Was `displace`.
pub fn move_horizontally<S: crate::actors::session_slots::SlotAccess>(right: bool, game: &mut S) {
    owned::move_horizontally(right, game)
}

/// Draygon body function - goop right ($A5:8B0A) or left ($A5:8C8E) - setup.
/// Was `begin`.
pub fn goop_setup<S: owned::Host>(right: bool, game: &mut S) {
    owned::goop_setup(right, game)
}
/// Every 128 frames of the current part, a bubble (sprite object 18h) 10h px
/// left of and above the body. The original reads the frame counter at X, so
/// after a turret pick it reads the body's words at that index ($A5:931C).
pub fn body_bubble<L: Lease>(turret: Option<u16>, game: &mut L) {
    game.foreign(|game| {
        owned::body_bubble(
            turret,
            &mut crate::runtime_access::RuntimeAccess::runtime(game),
        )
    })
}
/// Draygon body function - goop right ($A5:8B52) or left ($A5:8CD4) - move
/// until Samus in range. Was `approach`.
pub fn goop_move_until_in_range<L: Lease>(right: bool, game: &mut L) {
    game.foreign(|game| {
        owned::goop_move_until_in_range(
            right,
            &mut crate::runtime_access::RuntimeAccess::runtime(game),
        )
    })
}
/// Draygon body function - goop right ($A5:8BAE) or left ($A5:8D30) - firing
/// goops. Was `fire`.
pub fn goop_firing<S: owned::Host>(right: bool, game: &mut S) {
    owned::goop_firing(right, game)
}
/// Draygon body function - goop right ($A5:8C33) or left ($A5:8DB2) - move
/// until off-screen. Was `exit`.
pub fn goop_move_until_off_screen<L: Lease>(right: bool, game: &mut L) {
    game.foreign(|game| {
        owned::goop_move_until_off_screen(
            right,
            &mut crate::runtime_access::RuntimeAccess::runtime(game),
        )
    })
}

/// Instruction - Draygon instructions lists = [[Y]], [[Y] + 2], [[Y] + 4],
/// [[Y] + 6] ($A5:94DD). Was `pose`.
pub fn set_instruction_lists<S: crate::actors::session_slots::SlotAccess>(
    script: EnemyScript,
    game: &mut S,
) -> EnemyScript {
    owned::set_instruction_lists(script, game)
}
/// Instruction - displace Draygon body graphics [[Y]] px left, [[Y] + 2] px
/// down ($A5:9E0A). Was `layer_offsets`.
pub fn displace_graphics<S: BossState>(script: EnemyScript, game: &mut S) -> EnemyScript {
    session_slot!(game, Draygon[BODY]; graphics_x_displacement = script.word(0),
        graphics_y_displacement = script.word(2));
    script.offset(2)
}
/// Instruction - paralyse Draygon tail and arms ($A5:98D3). Was `hide_parts`.
pub fn paralyse_tail_and_arms<S: crate::actors::session_slots::SlotAccess>(
    script: EnemyScript,
    game: &mut S,
) -> EnemyScript {
    owned::paralyse_tail_and_arms(script, game)
}
/// Instruction - set Draygon body as intangible ($A5:98EF). Was `tangible`.
pub fn set_intangible<S: crate::actors::session_slots::SlotAccess>(
    script: EnemyScript,
    game: &mut S,
) -> EnemyScript {
    owned::set_intangible(script, game)
}

#[path = "draygon_flight_owned.rs"]
pub mod owned;

#[path = "draygon_entry_owned.rs"]
pub mod entry;

pub fn install_initial_palette<
    H: crate::session_access::SessionAccess + crate::word_transfers::LoadedRom,
>(
    game: &mut H,
) {
    let image = game.loaded_rom_image();
    let base = usize::try_from(crate::word_transfers::lorom_offset(TARGET_PALETTES)).unwrap();
    for index in (0..25).rev() {
        let at = base + (index * 2);
        let value = u16::from_ne_bytes([image.read(at), image.read(at + 1)]);
        game.session_mut().presentation.visual_mut().targets[144 + index] = value;
    }
}
