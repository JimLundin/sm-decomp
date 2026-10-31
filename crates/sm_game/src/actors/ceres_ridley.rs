//! Ceres Ridley (bank $A6:A0F5-$A6:C537): the Ceres station fight's intro
//! fades, hovering, fireballs, lunges and swoops, the fake and real retreats,
//! the mode 7 getaway cutscene and the self destruct sequence it starts.
//!
//! Was `CeresRidley_*` and the self destruct `Ridley_*` functions in the
//! original decompilation. Ceres Ridley is enemy 0 with Norfair Ridley's slot
//! and RAM, so it shares the `Ridley` view and his tail, baby metroid,
//! movement and drawing in `ridley`; the intro (`initial` to
//! `roar_before_flying`) and `init` serve both fights.
#![forbid(unsafe_code)]
use crate::actors::boss_lease::{BossLease, BossState as _};
use crate::actors::host::BossEffects as _;
use crate::actors::host::{abs16, below};
use crate::actors::ridley::Host as _;
use crate::actors::ridley::{
    self, FACING_RIGHT, NORFAIR, RIDLEY, Rect, Ridley, function, instruction_list, tail,
};
use crate::actors::session_slots::session_slot;
use crate::constants::enemy_properties::{
    K_ENEMY_PROPS_BLOCK_PLASMA_BEAM, K_ENEMY_PROPS_DELETED, K_ENEMY_PROPS_INVISIBLE,
    K_ENEMY_PROPS_TANGIBLE,
};
use crate::menus::prelude::{owned, scroll};
use crate::palette_copies::{copy_rom_colors_to_palette, copy_rom_colors_to_target_palette};
use crate::projectile_geometry::SINE_COSINE;
use crate::ram::word;
use sm_data::cartridge::lorom_offset;

/// A lease on a game with Ceres Ridley's effects.
pub trait Lease: BossLease<Game: Host> {}
impl<T: BossLease<Game: Host> + ?Sized> Lease for T {}

/// What Ceres Ridley needs beyond Ridley's host: the boss flags, the mode 7
/// transfers, the getaway push and the escape sequence's tiles and text.
pub trait Host: ridley::Host {
    /// The boss bits for `area`.
    fn boss_byte(&mut self, area: u16) -> u8;
    /// Queue the bank `bank` mode 7 transfer list at `list`.
    fn queue_mode7_transfer(&mut self, bank: u8, list: u16);
    /// Start Samus's Ceres getaway push.
    fn begin_ceres_push(&mut self);
    /// Queue the next escape timer tile transfer; true once they are done.
    fn queue_escape_timer_tiles(&mut self) -> bool;
    /// Queue the Ceres getaway's tile transfer.
    fn queue_getaway_tiles(&mut self);
    /// Queue the Japanese "self destruct sequence" text tilemaps.
    fn queue_escape_tile_table(&mut self);
    /// Type the next escape text character at tiles `tiles`; true once done.
    fn type_escape_text(&mut self, tiles: u16) -> bool;
}

/// Bank $A6 Ceres Ridley functions, stored in the body's `function` word.
pub mod ceres_function {
    pub const TRANSITION_TO_FLYING: u16 = 0xa354;
    pub const INITIAL: u16 = 0xa35b;
    pub const WAIT_TO_FADE_IN: u16 = 0xa377;
    pub const FADE_IN_EYE: u16 = 0xa389;
    pub const FADE_IN_BODY: u16 = 0xa3df;
    pub const WAIT_TO_ROAR: u16 = 0xa455;
    pub const ROAR_BEFORE_FLYING: u16 = 0xa478;
    pub const LIFTOFF_DECELERATING: u16 = 0xa6c8;
    pub const HOVERING: u16 = 0xa6e8;
    pub const FIREBALLS_MOVE_TO_POSITION: u16 = 0xa782;
    pub const FIREBALLS_SHOOTING: u16 = 0xa7f9;
    pub const LUNGE_SETUP: u16 = 0xa83c;
    pub const LUNGE: u16 = 0xa84e;
    pub const SWOOP_SETUP: u16 = 0xa88d;
    pub const SWOOP_MOVE_TO_POSITION: u16 = 0xa8a4;
    pub const SWOOP_AIMING_DOWN: u16 = 0xa8d4;
    pub const SWOOP_AIMING_LEFT: u16 = 0xa8f8;
    pub const SWOOP_AIMING_UP: u16 = 0xa923;
    pub const SWOOP_AIMING_UP_FASTER: u16 = 0xa947;
    pub const WRITE_PALETTES_AND_SPAWN_WALLS: u16 = 0xa9a0;
    pub const SET_UP_GETAWAY: u16 = 0xaa11;
    /// The RTS while the getaway cutscene runs.
    pub const GETAWAY: u16 = 0xaa4f;
    pub const SELF_DESTRUCT_ACTIVATED: u16 = 0xaa50;
    pub const SELF_DESTRUCT_ACTIVATING: u16 = 0xc04e;
}

/// Bank $A6 instruction lists.
pub mod ceres_instruction_list {
    pub const INITIAL: u16 = 0xe538;
    pub const LUNGE: u16 = 0xe548;
    pub const FIGHT_INTRO_ROAR: u16 = 0xe690;
    pub const TRANSITION_TO_FLYING: u16 = 0xe91d;
    pub const BABY_METROID: u16 = 0xbf31;
}

/// Bank $A6 palettes and populations.
pub mod ceres_data {
    /// Initial additional palettes - Ceres Ridley.
    pub const CERES_PALETTES: u16 = 0xe16f;
    /// Initial additional palettes - Norfair Ridley.
    pub const NORFAIR_PALETTES: u16 = 0xe1cf;
    /// The real retreat's palettes.
    pub const RETREAT_SPRITE_PALETTE: u16 = 0xa9e3;
    pub const RETREAT_BACKGROUND_PALETTE: u16 = 0xaa01;
    /// The walls spawned during the getaway cutscene.
    pub const GETAWAY_WALLS: [u16; 2] = [0xaa2f, 0xaa3f];
}

/// Ridley eye fade-in index table ($A6:E269), byte per step, FFh-terminated.
const EYE_FADE_STEPS: u32 = 0xa6_e269;
/// The Ceres Ridley palettes by zoom level, 20h bytes each ($A6:B107).
const ZOOM_PALETTES: u16 = 0xb107;
/// Hardware registers' RAM mirrors: the fake BG mode and mode 7 settings.
const FAKE_BG_MODE: usize = 0x56;
const MODE7_SETTINGS: usize = 0x5f;
/// Mode 7 with overflowing tiles transparent and no screen flip.
const MODE7_TRANSPARENT_OVERFLOW: u8 = 0x80;
/// The Ceres escape typewriter's tiles.
const ESCAPE_TEXT_TILES: u16 = 0x3582;
/// Ceres status values.
const RIDLEY_ESCAPING: u16 = 1;
const ESCAPE_STARTED: u16 = 2;

/// Initialisation AI - enemy $E13F/$E17F (Ridley) ($A6:A0F5), for both fights;
/// a defeated Ridley is deleted. Was `CeresRidley_Init`.
pub fn init<L: Lease + ?Sized>(game: &mut L) {
    if (game.foreign(|game| game.boss_byte(game.area())) & 1) != 0 {
        session_slot!(game, Ridley[RIDLEY].base.properties, |old| old
            | K_ENEMY_PROPS_TANGIBLE as u16
            | K_ENEMY_PROPS_DELETED as u16
            | K_ENEMY_PROPS_INVISIBLE as u16);
        return;
    }
    crate::word_transfers!(game, descending, 2048; le(TILEMAP_STUFF, 2047) <- value(0));
    game.foreign(|game| game.disable_minimap_for_boss());
    session_slot!(game, Ridley[RIDLEY]; parameter_1 = 0, parameter_2 = 0);
    ridley::set_instruction_list(ceres_instruction_list::INITIAL, game);
    session_slot!(game, Ridley[RIDLEY].base.palette_index = 3584);
    session_slot!(game, Ridley[RIDLEY].palette = 3584);
    session_slot!(game, Ridley[RIDLEY].base.extra_properties, |old| old | 4);
    session_slot!(game, Ridley[RIDLEY]; fight_mode = 0, hit_counter = 0);
    word! { game, 0; EARTHQUAKE_TYPE, EARTHQUAKE_TIMER }
    ridley::initialise_tail(game);
    ridley::initialise_ribs(game);
    session_slot!(game, Ridley[RIDLEY].base.properties, |old| old
        | K_ENEMY_PROPS_BLOCK_PLASMA_BEAM as u16
        | K_ENEMY_PROPS_TANGIBLE as u16);
    if game.area() == NORFAIR {
        session_slot!(game, Ridley[RIDLEY].base; x_pos = 96, y_pos = 394);
        session_slot!(game, Ridley[RIDLEY].function = ceres_function::INITIAL);
        session_slot!(game, Ridley[RIDLEY]; x_velocity = 0, y_velocity = 0);
        session_slot!(game, Ridley[RIDLEY].base.layer = 5);
        session_slot!(game, Ridley[RIDLEY]; fight_mode = 0, movement_enabled = 1, min_y = 64, max_y = 416,
            min_x = 64, max_x = 224, facing = FACING_RIGHT, tail_damage = 120, );
        game.foreign(|game| {
            copy_rom_colors_to_target_palette(0xa6, 0x140, ceres_data::NORFAIR_PALETTES, 0x20, game)
        });
        crate::word_transfers!(game, ascending, 15;
            le(TARGET_PALETTES, 113) <- value(0),
            le(TARGET_PALETTES, 241) <- value(0),
        );
    } else {
        session_slot!(game, Ridley[RIDLEY].base; x_pos = 186, y_pos = 169);
        word!(game.CERES_STATUS = 0);
        session_slot!(game, Ridley[RIDLEY]; facing = ridley::FACING_LEFT, movement_enabled = 0);
        word!(game.TILEMAP_STUFF[tail::IDLE_WHIP] = 1);
        session_slot!(game, Ridley[RIDLEY]; tail_damage = 15, function = ceres_function::INITIAL);
        session_slot!(game, Ridley[RIDLEY]; x_velocity = 0, y_velocity = 0);
        session_slot!(game, Ridley[RIDLEY]; min_y = -32i32 as u16, max_y = 176, min_x = 40,
            max_x = 224);
        game.foreign(|game| {
            copy_rom_colors_to_target_palette(0xa6, 0x140, ceres_data::CERES_PALETTES, 0x20, game)
        });
        crate::word_transfers!(game, ascending, 15; le(TARGET_PALETTES, 241) <- value(0));
        session_slot!(
            game,
            Ridley[RIDLEY].baby_metroid_instruction = ceres_instruction_list::BABY_METROID
        );
        session_slot!(game, Ridley[RIDLEY].baby_metroid_instruction_timer = 1);
        session_slot!(
            game,
            Ridley[RIDLEY].baby_metroid_function = ridley::baby_metroid_function::HELD
        );
        session_slot!(game, Ridley[RIDLEY].wings_frame = 5);
        game.foreign(|game| game.queue_music_delayed(0));
    }
}

/// Main AI - enemy $E13F (Ceres Ridley) ($A6:A288): immortal; until the
/// getaway it moves, animates and draws as Norfair Ridley does. Was
/// `CeresRidley_Main`.
pub fn main<L: Lease + ?Sized>(game: &mut L) {
    // The state steps and the function run on the lease, the tail, drawing,
    // ribs and baby metroid in one trip to the game.
    session_slot!(game, Ridley[RIDLEY].base.health = 0x7fff);
    let function = session_slot!(game, Ridley[RIDLEY].function);
    run_function(game, function);
    if word!(game.CERES_STATUS) == 0 {
        let moving = session_slot!(game, Ridley[RIDLEY].movement_enabled) != 0;
        if moving {
            ridley::hurt_flash(game);
            ridley::move_ridley(game);
            ridley::update_wings(game);
            randomize_tail_curliness(game);
        }
        game.foreign(|game| {
            if moving {
                ridley::handle_tail(game);
                ridley::ceres_health_palette(game);
                ridley::handle_ribs(game);
                ridley::draw_tail(game);
                ridley::draw_wings(game);
            }
            ridley::handle_baby_metroid(game);
        });
    }
}

/// Run the bank $A6 Ridley function `function`, Ceres Ridley's or Norfair
/// Ridley's, on this lease. Was `CallRidleyFunc`.
pub fn run_function<L: Lease + ?Sized>(game: &mut L, function: u16) {
    if !run_ceres_function(game, function) && !ridley::run_ridley_function(game, function) {
        game.foreign(|game| game.unreachable());
    }
}

/// Run Ceres Ridley's bank $A6 function `function`; false when it is not one
/// of his.
fn run_ceres_function<L: Lease + ?Sized>(game: &mut L, function: u16) -> bool {
    use ceres_function as f;
    match function {
        f::TRANSITION_TO_FLYING => transition_to_flying(game),
        f::INITIAL => initial(game),
        f::WAIT_TO_FADE_IN => wait_to_fade_in(game),
        f::FADE_IN_EYE => fade_in_eye(game),
        f::FADE_IN_BODY => fade_in_body(game),
        f::WAIT_TO_ROAR => wait_to_roar(game),
        f::ROAR_BEFORE_FLYING => roar_before_flying(game),
        ridley::function::CERES_LIFTOFF_ACCELERATING => liftoff_accelerating(game),
        f::LIFTOFF_DECELERATING => liftoff_decelerating(game),
        f::HOVERING => hovering(game),
        f::FIREBALLS_MOVE_TO_POSITION => fireballs_move_to_position(game),
        f::FIREBALLS_SHOOTING => fireballs_shooting(game),
        f::LUNGE_SETUP => lunge_setup(game),
        f::LUNGE => lunge(game),
        f::SWOOP_SETUP => swoop_setup(game),
        f::SWOOP_MOVE_TO_POSITION => swoop_move_to_position(game),
        f::SWOOP_AIMING_DOWN => swoop_aiming_down(game),
        f::SWOOP_AIMING_LEFT => swoop_aiming_left(game),
        f::SWOOP_AIMING_UP => swoop_aiming_up(game),
        f::SWOOP_AIMING_UP_FASTER => swoop_aiming_up_faster(game),
        ridley::function::CERES_REAL_RETREAT_RISING => real_retreat_rising(game),
        f::WRITE_PALETTES_AND_SPAWN_WALLS => write_palettes_and_spawn_walls(game),
        f::SET_UP_GETAWAY => set_up_getaway(game),
        f::GETAWAY => {}
        f::SELF_DESTRUCT_ACTIVATED => self_destruct_activated(game),
        f::SELF_DESTRUCT_ACTIVATING => self_destruct_activating(game),
        _ => return false,
    }
    true
}

/// Randomly update Ridley tail curliness ($A6:A2BD): the ideal inter-segment
/// angle, 8 to 17h, one frame in 256. Was `CeresRidley_Func_1`.
pub fn randomize_tail_curliness<L: Lease + ?Sized>(game: &mut L) {
    if word!(game.RANDOM_NUMBER) >= 0xff00 {
        word!(
            game.TILEMAP_STUFF[tail::IDEAL_SEGMENT_ANGLE] = (word!(game.RANDOM_NUMBER) & 0xf) + 8
        );
    }
}

/// Hurt AI - enemy $E13F (Ceres Ridley) ($A6:A2D3): flash and draw. Was
/// `CeresRidley_Hurt`.
pub fn hurt<L: Lease + ?Sized>(game: &mut L) {
    ridley::hurt_flash(game);
    ridley::draw_tail(game);
    ridley::draw_wings(game);
    ridley::handle_baby_metroid(game);
}

/// Ridley function - transition to flying ($A6:A354). Was `CeresRidley_Func_2`.
pub fn transition_to_flying<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY]; x_velocity = 0, y_velocity = 0);
}

/// Ridley function - initial ($A6:A35B): after the door transition, wait to
/// fade in, shorter in Norfair. Was `CeresRidley_Func_3`.
pub fn initial<L: Lease + ?Sized>(game: &mut L) {
    if word!(game.DOOR_TRANSITION_FLAG_ENEMIES) == 0 {
        session_slot!(
            game,
            Ridley[RIDLEY].function = ceres_function::WAIT_TO_FADE_IN
        );
        let wait = if game.area() == NORFAIR { 170 } else { 512 };
        session_slot!(game, Ridley[RIDLEY].function_timer = wait);
        wait_to_fade_in(game);
    }
}

/// Count down the function timer; true once it is negative.
fn timer_expired<L: Lease + ?Sized>(game: &mut L) -> bool {
    session_slot!(game, Ridley[RIDLEY].function_timer, |function_timer| {
        function_timer.wrapping_sub(1)
    });
    (session_slot!(game, Ridley[RIDLEY].function_timer) & 0x8000) != 0
}

/// Ridley function - wait to fade in ($A6:A377). Was `CeresRidley_A377`.
pub fn wait_to_fade_in<L: Lease + ?Sized>(game: &mut L) {
    if timer_expired(game) {
        session_slot!(game, Ridley[RIDLEY].function = ceres_function::FADE_IN_EYE);
        session_slot!(game, Ridley[RIDLEY]; fade_index = 0, function_timer = 0);
    }
}

/// Ridley function - fade in eye ($A6:A389): every 256th frame the next step
/// of the eye colours, then the body. Was `CeresRidley_Func_4`.
pub fn fade_in_eye<L: Lease + ?Sized>(game: &mut L) {
    if (session_slot!(game, Ridley[RIDLEY].fade_index) & 0x8000) != 0 {
        return;
    }
    let index = session_slot!(game, Ridley[RIDLEY].fade_index);
    session_slot!(game, Ridley[RIDLEY].function_timer, |function_timer| {
        function_timer.wrapping_add(1)
    });
    if session_slot!(game, Ridley[RIDLEY].function_timer) == 0 {
        return;
    }
    session_slot!(game, Ridley[RIDLEY].function_timer = 0);
    let step = game
        .native_image()
        .read(lorom_offset(EYE_FADE_STEPS) as usize + usize::from(index));
    if step == 0xff {
        session_slot!(game, Ridley[RIDLEY]; fade_index = 0, function = ceres_function::FADE_IN_BODY,
            movement_enabled = 1);
    } else {
        session_slot!(game, Ridley[RIDLEY].fade_index = index.wrapping_add(1));
        let colour = 3 * isize::from(step);
        word! { game;
            PALETTE_BUFFER[252] = crate::ceres_ridley_tables::eye_fade(game, colour);
            PALETTE_BUFFER[253] = crate::ceres_ridley_tables::eye_fade(game, colour + 1);
            PALETTE_BUFFER[254] = crate::ceres_ridley_tables::eye_fade(game, colour + 2);
        }
    }
}

/// Ridley function - fade in body ($A6:A3DF): every other frame the next 11
/// body colours, into sprite palettes 1 and 7; then tangible and waiting to
/// roar. Was `CeresRidley_Func_5`.
pub fn fade_in_body<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY].function_timer, |function_timer| {
        function_timer.wrapping_add(1)
    });
    if session_slot!(game, Ridley[RIDLEY].function_timer) < 2 {
        return;
    }
    session_slot!(game, Ridley[RIDLEY].function_timer = 0);
    let mut index = session_slot!(game, Ridley[RIDLEY].fade_index);
    let (mut first, mut second) = (145, 241);
    for _ in 0..11 {
        let colour = crate::ceres_ridley_tables::body_fade(game, (index >> 1) as isize);
        word! {game; PALETTE_BUFFER[first] = colour; PALETTE_BUFFER[second] = colour; }
        index = index.wrapping_add(2);
        first += 1;
        second += 1;
    }
    if index >= 0x160 {
        if game.area() == NORFAIR {
            session_slot!(game, Ridley[RIDLEY].base.layer = 2);
        }
        session_slot!(game, Ridley[RIDLEY].base.properties, |old| old
            & !(K_ENEMY_PROPS_TANGIBLE as u16));
        session_slot!(game, Ridley[RIDLEY]; fade_index = 0, function = ceres_function::WAIT_TO_ROAR,
            function_timer = 4);
        game.foreign(|game| game.queue_music_delayed(5));
    } else {
        session_slot!(game, Ridley[RIDLEY].fade_index = index);
    }
}

/// Ridley function - wait to roar ($A6:A455): the fight intro roar, longer in
/// Ceres. Was `CeresRidley_Func_6`.
pub fn wait_to_roar<L: Lease + ?Sized>(game: &mut L) {
    if timer_expired(game) {
        ridley::set_instruction_list(ceres_instruction_list::FIGHT_INTRO_ROAR, game);
        session_slot!(game, Ridley[RIDLEY].function_timer = 0);
        session_slot!(
            game,
            Ridley[RIDLEY].function = ceres_function::ROAR_BEFORE_FLYING
        );
        if game.area() != NORFAIR {
            session_slot!(game, Ridley[RIDLEY].function_timer = 252);
        }
    }
}

/// Ridley function - roar before fly ($A6:A478): in Norfair the background
/// fades in and the lava rises first. Was `CeresRidley_Func_7`.
pub fn roar_before_flying<L: Lease + ?Sized>(game: &mut L) {
    if !timer_expired(game) {
        return;
    }
    if game.area() == NORFAIR {
        session_slot!(game, Ridley[RIDLEY].function_timer = 2);
        let step = session_slot!(game, Ridley[RIDLEY].fade_index).wrapping_add(1);
        session_slot!(game, Ridley[RIDLEY].fade_index = step);
        if !fade_in_background(step.wrapping_sub(1), game) {
            return;
        }
        game.foreign(|game| game.move_liquid(440, -96i32 as u16, 32));
    }
    session_slot!(game, Ridley[RIDLEY].fade_index = 0);
    ridley::set_instruction_list(ceres_instruction_list::TRANSITION_TO_FLYING, game);
    session_slot!(game, Ridley[RIDLEY]; wings_timer_delta = 8, wings_timer = 8);
    ridley::activate_tail_segments(game);
    word!(game.TILEMAP_STUFF[tail::FUNCTION] = 1);
    session_slot!(
        game,
        Ridley[RIDLEY].function = ceres_function::TRANSITION_TO_FLYING
    );
}

/// Fade in Ridley room background ($A6:A4D6) by step `step`; true once done.
/// Was `CeresRidley_Func_8`.
pub fn fade_in_background<L: Lease + ?Sized>(step: u16, game: &mut L) -> bool {
    let colours = crate::ceres_ridley_tables::background_fade(game, step as isize);
    if colours == 0 {
        return true;
    }
    game.foreign(|game| copy_rom_colors_to_palette(0xe2, 0xa6, colours, 0xe, game));
    false
}

/// Ridley function - liftoff - accelerating ($A6:A6AF). Was
/// `CeresRidley_Func_9`.
pub fn liftoff_accelerating<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY].y_velocity, |old| old.wrapping_sub(16));
    if below(session_slot!(game, Ridley[RIDLEY].base.y_pos), 112) {
        session_slot!(
            game,
            Ridley[RIDLEY].function = ceres_function::LIFTOFF_DECELERATING
        );
        liftoff_decelerating(game);
    }
}

/// Ridley function - liftoff - decelerating ($A6:A6C8). Was
/// `CeresRidley_Func_10`.
pub fn liftoff_decelerating<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY].y_velocity, |old| old.wrapping_add(20));
    if below(session_slot!(game, Ridley[RIDLEY].base.y_pos), 80) {
        session_slot!(game, Ridley[RIDLEY]; function = ceres_function::HOVERING, fight_mode = 1);
    }
}

/// Ridley function - hovering ($A6:A6E8): after 64h hits the fake retreat,
/// when Samus is below 30 health the real one, otherwise a random attack once
/// in position or after 7Ch frames. Was `CeresRidley_Func_11`.
pub fn hovering<L: Lease + ?Sized>(game: &mut L) {
    if session_slot!(game, Ridley[RIDLEY].hit_counter) >= 0x64 {
        session_slot!(game, Ridley[RIDLEY].fight_mode = 0);
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::FAKE_RETREAT_MOVE_TO_POSITION
        );
        ridley::fake_retreat_move_to_position(game);
    } else if below(word!(game.SAMUS_HEALTH), 30) {
        session_slot!(game, Ridley[RIDLEY].fight_mode = 0);
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::CERES_REAL_RETREAT_RISING
        );
        real_retreat_rising(game);
    } else if !hovering_movement(game) || {
        let timer = session_slot!(game, Ridley[RIDLEY].timer).wrapping_add(1);
        session_slot!(game, Ridley[RIDLEY].timer = timer);
        timer >= 0x7c
    } {
        let attack =
            crate::ceres_ridley_tables::attack(game, (word!(game.RANDOM_NUMBER) & 0xf) as isize);
        session_slot!(game, Ridley[RIDLEY]; function = attack, timer = 0);
    }
}

/// Ceres Ridley hovering movement ($A6:A763): towards (C0h, 64h); false once
/// there. Was `CeresRidley_Func_12`.
pub fn hovering_movement<L: Lease + ?Sized>(game: &mut L) -> bool {
    let rect = Rect {
        x: 192,
        y: 100,
        w: 8,
        h: 8,
    };
    ridley::ceres_accelerate(0, 0, rect.x, rect.y, game);
    ridley::separated_from_rect(0, rect, game)
}

/// Ridley function - shoot fireballs - move to position ($A6:A782): rise with
/// at least 80h speed, then shoot from the top. Was `CeresRidley_Func_13`.
pub fn fireballs_move_to_position<L: Lease + ?Sized>(game: &mut L) {
    let mut speed = abs16(session_slot!(game, Ridley[RIDLEY].y_velocity));
    if speed < 0x80 {
        speed = 128;
    }
    let y_velocity = if session_slot!(game, Ridley[RIDLEY].y_velocity) & 0x8000 != 0 {
        -(speed as i32)
    } else {
        speed as i32
    };
    session_slot!(game, Ridley[RIDLEY].y_velocity = y_velocity as u16);
    ridley::ceres_accelerate(
        0,
        0,
        session_slot!(game, Ridley[RIDLEY].base.x_pos),
        88,
        game,
    );
    if below(session_slot!(game, Ridley[RIDLEY].base.y_pos), 80) {
        let timer = session_slot!(game, Ridley[RIDLEY].timer).wrapping_add(1);
        session_slot!(game, Ridley[RIDLEY].timer = timer);
        if timer >= 0x30 {
            session_slot!(game, Ridley[RIDLEY].function = ceres_function::SWOOP_SETUP);
        }
    } else if below(session_slot!(game, Ridley[RIDLEY].base.y_pos), 128) {
        let copied = session_slot!(game, Ridley[RIDLEY].base.x_pos);
        session_slot!(game, Ridley[RIDLEY].target_x = copied);
        let copied = session_slot!(game, Ridley[RIDLEY].base.y_pos);
        session_slot!(game, Ridley[RIDLEY].target_y = copied);
        ridley::set_instruction_list(instruction_list::SHOOT_FIREBALLS, game);
        session_slot!(
            game,
            Ridley[RIDLEY].function = ceres_function::FIREBALLS_SHOOTING
        );
        session_slot!(game, Ridley[RIDLEY].timer = 224);
        fireballs_shooting(game);
    }
}

/// Ridley function - shoot fireballs - shooting ($A6:A7F9): jitter around the
/// shooting spot, then hover. Was `CeresRidley_A7F9`.
pub fn fireballs_shooting<L: Lease + ?Sized>(game: &mut L) {
    let jitter = if word!(game.RANDOM_NUMBER) & 0x8000 != 0 {
        -(word!(game.RANDOM_NUMBER) as i32 & 7)
    } else {
        word!(game.RANDOM_NUMBER) as i32 & 7
    } as u16;
    // The Y add's ADC carries in from the X add, so a negative jitter adds one more.
    let (x, carry) = crate::carry::adc(session_slot!(game, Ridley[RIDLEY].target_x), jitter, false);
    let y = crate::carry::adc(session_slot!(game, Ridley[RIDLEY].target_y), jitter, carry).0;
    ridley::ceres_accelerate(0, 0, x, y, game);
    let timer = (session_slot!(game, Ridley[RIDLEY].timer) as i16).wrapping_sub(1);
    session_slot!(game, Ridley[RIDLEY].timer = timer as u16);
    if timer < 0 {
        session_slot!(game, Ridley[RIDLEY]; timer = 0, function = ceres_function::HOVERING);
    }
}

/// Ridley function - lunge - setup ($A6:A83C). Was `CeresRidley_Func_14`.
pub fn lunge_setup<L: Lease + ?Sized>(game: &mut L) {
    ridley::set_instruction_list(ceres_instruction_list::LUNGE, game);
    session_slot!(game, Ridley[RIDLEY]; function = ceres_function::LUNGE, function_timer = 64);
    lunge(game);
}

/// Ridley function - lunge - main ($A6:A84E): at Samus, 44h px above her and
/// no higher than 40h, until there or the timer runs out. Was
/// `CeresRidley_Func_15`.
pub fn lunge<L: Lease + ?Sized>(game: &mut L) {
    let x = game.player_x();
    let mut y = game.player_y().wrapping_sub(68);
    if below(game.player_y(), 132) {
        y = 64;
    }
    ridley::ceres_accelerate(0, 0xd, x, y, game);
    let rect = Rect { x, y, w: 2, h: 2 };
    if !ridley::separated_from_rect(0, rect, game) || {
        let expired =
            (session_slot!(game, Ridley[RIDLEY].function_timer) as i16).wrapping_sub(1) < 0;
        session_slot!(game, Ridley[RIDLEY].function_timer, |function_timer| {
            function_timer.wrapping_sub(1)
        });
        expired
    } {
        session_slot!(game, Ridley[RIDLEY]; timer = 0, function = ceres_function::HOVERING);
    }
}

/// Ridley function - swoop - setup ($A6:A88D), without idle tail whips. Was
/// `CeresRidley_Func_16`.
pub fn swoop_setup<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(
        game,
        Ridley[RIDLEY].function = ceres_function::SWOOP_MOVE_TO_POSITION
    );
    session_slot!(game, Ridley[RIDLEY]; function_timer = 10, swoop_angle = 0);
    word!(game.TILEMAP_STUFF[tail::IDLE_WHIP] = 0);
    swoop_move_to_position(game);
}

/// Ridley function - swoop - move to position ($A6:A8A4). Was
/// `CeresRidley_Func_17`.
pub fn swoop_move_to_position<L: Lease + ?Sized>(game: &mut L) {
    ridley::ceres_accelerate(0, 1, 192, 80, game);
    if below(session_slot!(game, Ridley[RIDLEY].base.y_pos), 96) {
        session_slot!(
            game,
            Ridley[RIDLEY].function = ceres_function::SWOOP_AIMING_DOWN
        );
        session_slot!(game, Ridley[RIDLEY]; function_timer = 10, swoop_angle = 0);
    }
}

/// One swoop stage: steer, and on the timer running out take `next` for
/// `frames` frames.
fn swoop_stage<L: Lease + ?Sized>(
    game: &mut L,
    angle_step: i32,
    target_angle: i32,
    next: u16,
    frames: u16,
) -> bool {
    ridley::calculate_swoop_velocities(angle_step as u16, target_angle as u16, 768, game);
    if timer_expired(game) {
        session_slot!(game, Ridley[RIDLEY]; function = next, function_timer = frames);
        return true;
    }
    false
}

/// Ridley function - swoop - descending - aiming down ($A6:A8D4). Was
/// `CeresRidley_Func_18`.
pub fn swoop_aiming_down<L: Lease + ?Sized>(game: &mut L) {
    swoop_stage(game, -32, -1024, ceres_function::SWOOP_AIMING_LEFT, 36);
}

/// Ridley function - swoop - descending - aiming left ($A6:A8F8), with a tail
/// whip. Was `CeresRidley_Func_19`.
pub fn swoop_aiming_left<L: Lease + ?Sized>(game: &mut L) {
    if swoop_stage(game, -512, -16384, ceres_function::SWOOP_AIMING_UP, 28) {
        word!(game.TILEMAP_STUFF[tail::WHIP_REQUEST] = 1);
    }
}

/// Ridley function - swoop - ascending - aiming up ($A6:A923). Was
/// `CeresRidley_Func_20`.
pub fn swoop_aiming_up<L: Lease + ?Sized>(game: &mut L) {
    swoop_stage(
        game,
        -512,
        -30720,
        ceres_function::SWOOP_AIMING_UP_FASTER,
        1,
    );
}

/// Ridley function - swoop - ascending - aiming up faster ($A6:A947): back to
/// hovering, with idle tail whips again. Was `CeresRidley_Func_21`.
pub fn swoop_aiming_up_faster<L: Lease + ?Sized>(game: &mut L) {
    ridley::calculate_swoop_velocities(-768i32 as u16, -30720i32 as u16, 768, game);
    if timer_expired(game) {
        session_slot!(game, Ridley[RIDLEY]; function = ceres_function::HOVERING, timer = 0);
        word!(game.TILEMAP_STUFF[tail::IDLE_WHIP] = 1);
    }
}

/// Ridley function - real retreat - rising ($A6:A971): off the top with the
/// baby metroid. Was `CeresRidley_Func_22`.
pub fn real_retreat_rising<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY].min_y = -192i32 as u16);
    ridley::ceres_accelerate(0, 1, 192, -128i32 as u16, game);
    if (session_slot!(game, Ridley[RIDLEY].base.y_pos) as i32 + 128) & 0x8000 != 0 {
        real_retreat_risen(game);
    }
}

/// The end of the real retreat's rise ($A6:A994). Was `CeresRidley_A994`.
pub fn real_retreat_risen<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(
        game,
        Ridley[RIDLEY].function = ceres_function::WRITE_PALETTES_AND_SPAWN_WALLS
    );
    session_slot!(game, Ridley[RIDLEY].function_timer = 64);
    write_palettes_and_spawn_walls(game);
}

/// Ridley function - real retreat - write palettes and spawn walls ($A6:A9A0).
/// Was `CeresRidley_Func_23`.
pub fn write_palettes_and_spawn_walls<L: Lease + ?Sized>(game: &mut L) {
    if timer_expired(game) {
        spawn_getaway_walls(game);
        session_slot!(game, Ridley[RIDLEY]; x_velocity = 0, y_velocity = 0);
        word!(game.TILEMAP_STUFF[tail::FUNCTION] = 0);
        session_slot!(
            game,
            Ridley[RIDLEY].function = ceres_function::SET_UP_GETAWAY
        );
        game.foreign(|game| {
            copy_rom_colors_to_palette(0xa2, 0xa6, ceres_data::RETREAT_SPRITE_PALETTE, 0xf, game);
            copy_rom_colors_to_palette(0x42, 0xa6, ceres_data::RETREAT_BACKGROUND_PALETTE, 8, game);
            copy_rom_colors_to_palette(
                0x1e2,
                0xa6,
                ceres_data::RETREAT_BACKGROUND_PALETTE,
                8,
                game,
            );
        });
    }
}

/// Ridley function - real retreat - set up getaway cutscene ($A6:AA11). Was
/// `CeresRidley_Func_24`.
pub fn set_up_getaway<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY].function = ceres_function::GETAWAY);
    word!(game.CERES_STATUS = RIDLEY_ESCAPING);
    set_up_mode7(game);
}

/// Spawn walls during Ceres Ridley getaway cutscene ($A6:AA20). Was
/// `CeresRidley_Func_25`.
pub fn spawn_getaway_walls<L: Lease + ?Sized>(game: &mut L) {
    for population in ceres_data::GETAWAY_WALLS {
        game.foreign(|game| game.spawn_population(0xa6, population));
    }
}

/// Ridley function - real retreat - activated self destruct sequence
/// ($A6:AA50): the emergency text glows. Was `CeresRidley_Func_26`.
pub fn self_destruct_activated<L: Lease + ?Sized>(game: &mut L) {
    ridley::emergency_text_glow(game);
}

/// Set up Ceres Ridley mode 7 ($A6:AA54). Was `CeresRidley_Func_27`.
pub fn set_up_mode7<L: Lease + ?Sized>(game: &mut L) {
    game.set_slot_byte(FAKE_BG_MODE, 7);
    word!(game.IRQ_ENABLE_MODE7 = 1);
    game.set_slot_byte(MODE7_SETTINGS, MODE7_TRANSPARENT_OVERFLOW);
    word! { game, 256; REG_M7A, REG_M7B, REG_M7C, REG_M7D }
    word! { game, 64; REG_M7X, REG_M7Y }
    session_slot!(game, Ridley[RIDLEY]; getaway_finished = 0, getaway_index = 0);
    session_slot!(game, Ridley[RIDLEY]; swoop_angle = 1, mode7_x = -128i32 as u16);
    scroll!(game.Bg1H = -128i32 as u16);
    session_slot!(game, Ridley[RIDLEY].mode7_y = 32);
    scroll!(game.Bg1V = 32);
    session_slot!(game, Ridley[RIDLEY]; mode7_zoom = 2048, unread_8022 = 1024);
}

/// Handle Ceres Ridley getaway cutscene if active ($A6:AAAF). Was
/// `HandleCeresRidleyGetawayCutscene`.
pub fn handle_getaway_if_active<L: Lease + ?Sized>(game: &mut L) {
    if session_slot!(game, Ridley[RIDLEY].getaway_finished) == 0 {
        handle_getaway(game);
    }
}

/// Handle Ceres Ridley getaway cutscene ($A6:AABD): Ridley flies off in mode 7
/// along the zoom and velocity tables; the push and quake start at 208, and
/// the self destruct sequence when the table ends. Was `CeresRidley_Func_28`.
pub fn handle_getaway<L: Lease + ?Sized>(game: &mut L) {
    let index = session_slot!(game, Ridley[RIDLEY].getaway_index);
    session_slot!(game, Ridley[RIDLEY].getaway_index = index.wrapping_add(2));
    if index == 0 {
        game.queue_sound(1, 0x4e, 6);
    }
    if index == 208 {
        game.foreign(|game| game.begin_ceres_push());
        word! {game; EARTHQUAKE_TYPE = 35; EARTHQUAKE_TIMER = 64; }
    }
    let step = (index >> 1) as isize;
    if crate::ceres_ridley_tables::getaway_zoom(game, step) == 0xffff {
        session_slot!(game, Ridley[RIDLEY].getaway_finished, |getaway_finished| {
            getaway_finished.wrapping_add(1)
        });
        game.set_slot_byte(FAKE_BG_MODE, 9);
        game.set_slot_byte(MODE7_SETTINGS, 0);
        word! { game, 0;
            REG_M7A, REG_M7B, REG_M7C, REG_M7D, REG_M7X, REG_M7Y, REG_BG1HOFS, REG_BG1VOFS,
        }
        session_slot!(
            game,
            Ridley[RIDLEY].function = ceres_function::SELF_DESTRUCT_ACTIVATING
        );
        session_slot!(game, Ridley[RIDLEY].function_timer = 0);
    } else {
        session_slot!(
            game,
            Ridley[RIDLEY].mode7_zoom = crate::ceres_ridley_tables::getaway_zoom(game, step)
        );
        let velocity = crate::ceres_ridley_tables::getaway_velocity(game, step);
        let y = velocity
            .y
            .wrapping_add(session_slot!(game, Ridley[RIDLEY].mode7_y));
        session_slot!(game, Ridley[RIDLEY].mode7_y = y);
        scroll!(game.Bg1V = y);
        let x = session_slot!(game, Ridley[RIDLEY].mode7_x).wrapping_sub(velocity.x);
        session_slot!(game, Ridley[RIDLEY].mode7_x = x);
        scroll!(game.Bg1H = x);
        palette_by_zoom(session_slot!(game, Ridley[RIDLEY].mode7_zoom), game);
        session_slot!(game, Ridley[RIDLEY].swoop_angle, |swoop_angle| swoop_angle
            .wrapping_add(48));
        set_mode7_rotation(game);
        animate_mode7_baby_metroid(game);
        animate_mode7_wings(game);
    }
}

/// Set Ceres Ridley mode 7 rotation matrix ($A6:AB5F): rotation by the high
/// byte of the mode 7 angle, scaled by the zoom. Was `CeresRidley_Func_29`.
pub fn set_mode7_rotation<L: Lease + ?Sized>(game: &mut L) {
    let angle = session_slot!(byte game, Ridley[RIDLEY].swoop_angle[1]) as u16;
    word! { game;
        REG_M7A = signed_scale(cosine_256(angle), session_slot!(game, Ridley[RIDLEY].mode7_zoom));
        REG_M7B = signed_scale(sine_256(angle), session_slot!(game, Ridley[RIDLEY].mode7_zoom));
        REG_M7C = signed_scale(-(sine_256(angle) as i32) as u16, session_slot!(game, Ridley[RIDLEY].mode7_zoom));
        REG_M7D = signed_scale(cosine_256(angle), session_slot!(game, Ridley[RIDLEY].mode7_zoom));
    }
}

/// 100h times the cosine of `angle` (8-bit cosine multiplication, $A6:AC1E).
fn cosine_256(angle: u16) -> u16 {
    sine_multiply(angle.wrapping_add(64), 0x100) as u16
}

/// 100h times the sine of `angle` (8-bit sine multiplication, $A6:AC0E).
fn sine_256(angle: u16) -> u16 {
    sine_multiply(angle, 0x100) as u16
}

/// 8-bit sine multiplication ($A6:AC30): `amplitude` times the sine of
/// `angle`, over 100h. Was `CeresRidley_AC30`.
pub fn sine_multiply(angle: u16, amplitude: i16) -> i16 {
    ((SINE_COSINE[((angle as i32 & 0xff) + 64) as usize] as i32 * amplitude as i32) >> 8) as i16
}

/// Signed `a` times unsigned `b` over 100h (16-bit multiplication, $A6:AC58).
/// Was `CeresMult`.
fn signed_scale(a: u16, b: u16) -> u16 {
    let product = ((abs16(a) as i32 * b as i32) >> 8) as u16;
    (if a & 0x8000 != 0 {
        -(product as i32)
    } else {
        product as i32
    }) as u16
}

/// Animate Ceres Ridley mode 7 baby metroid ($A6:ACBC): four frames, one
/// every fourth frame. Was `CeresRidley_Func_33`.
pub fn animate_mode7_baby_metroid<L: Lease + ?Sized>(game: &mut L) {
    if (game.frame() & 3) == 0 {
        let frame = session_slot!(game, Ridley[RIDLEY].baby_metroid_frame).wrapping_add(1) & 3;
        session_slot!(game, Ridley[RIDLEY].baby_metroid_frame = frame);
        let list = crate::ceres_ridley_tables::baby_metroid_frame(game, frame as isize);
        game.foreign(|game| game.queue_mode7_transfer(0xa6, list));
    }
}

/// Animate Ceres Ridley mode 7 wings ($A6:AD27): two frames, one every
/// eighth frame. Was `CeresRidley_Func_34`.
pub fn animate_mode7_wings<L: Lease + ?Sized>(game: &mut L) {
    if (game.frame() & 7) == 0 {
        let frame = session_slot!(game, Ridley[RIDLEY].mode7_wings_frame).wrapping_add(1) & 1;
        session_slot!(game, Ridley[RIDLEY].mode7_wings_frame = frame);
        let list = crate::ceres_ridley_tables::wing_frame(game, frame as isize);
        game.foreign(|game| game.queue_mode7_transfer(0xa6, list));
    }
}

/// Set Ceres Ridley palette according to zoom level ($A6:B0EF): the palette
/// by the zoom's high byte. Was `CeresRidley_Func_35`.
pub fn palette_by_zoom<L: Lease + ?Sized>(zoom: u16, game: &mut L) {
    let colours = ZOOM_PALETTES.wrapping_add(32 * u16::from(zoom.to_ne_bytes()[1]));
    game.foreign(|game| copy_rom_colors_to_palette(0xa2, 0xa6, colours, 0xf, game));
}

/// Ridley function - real retreat - activating self destruct sequence
/// ($A6:C04E): the sequence step in the function timer. Was `Ridley_Func_54`.
pub fn self_destruct_activating<L: Lease + ?Sized>(game: &mut L) {
    match session_slot!(game, Ridley[RIDLEY].function_timer) {
        0 => load_text_colours(game),
        2 => load_escape_timer_tiles(game),
        4 => load_escape_background_tiles(game),
        6 => emergency_text_appears(game),
        8 => japanese_text_appears(game),
        10 => type_self_destruct_text(game),
        12 => start_escape(game),
        _ => game.foreign(|game| game.unreachable()),
    }
}

/// The next self destruct sequence step.
fn next_step<L: Lease + ?Sized>(game: &mut L) {
    for _ in 0..2 {
        session_slot!(game, Ridley[RIDLEY].function_timer, |function_timer| {
            function_timer.wrapping_add(1)
        });
    }
}

/// Self destruct sequence function index 0 - load text colours ($A6:C062).
/// Was `Ridley_Func_55`.
pub fn load_text_colours<L: Lease + ?Sized>(game: &mut L) {
    word!(game.PALETTE_BUFFER[97] = word!(game.PALETTE_BUFFER[1]));
    word!(game.PALETTE_BUFFER[99] = word!(game.PALETTE_BUFFER[3]));
    word!(game.PALETTE_BUFFER[81] = word!(game.PALETTE_BUFFER[17]));
    word!(game.PALETTE_BUFFER[83] = word!(game.PALETTE_BUFFER[19]));
    // The escape timer sprite tile transfer entries ($A6:C4CB).
    session_slot!(game, Ridley[RIDLEY].fade_index = 0xc4cb);
    next_step(game);
    load_escape_timer_tiles(game);
}

/// Self destruct sequence function index 2 - loading escape timer sprite
/// tiles ($A6:C08E). Was `Ridley_C08E`.
pub fn load_escape_timer_tiles<L: Lease + ?Sized>(game: &mut L) {
    if game.foreign(|game| game.queue_escape_timer_tiles()) {
        // The Ceres escape timer BG1/2 tile transfer entries ($A6:C4FE).
        session_slot!(game, Ridley[RIDLEY].fade_index = 0xc4fe);
        next_step(game);
        load_escape_background_tiles(game);
    }
}

/// Self destruct sequence function index 4 - loading Ceres escape timer BG1/2
/// tiles ($A6:C09F). Was `Ridley_C09F`.
pub fn load_escape_background_tiles<L: Lease + ?Sized>(game: &mut L) {
    if game.foreign(|game| game.queue_escape_timer_tiles()) {
        next_step(game);
        game.foreign(|game| game.queue_getaway_tiles());
        session_slot!(game, Ridley[RIDLEY].fade_index = 128);
        game.foreign(|game| game.queue_music_delayed(7));
    }
}

/// Self destruct sequence function index 6 - "emergency" text appears
/// ($A6:C0BB): then the Japanese text, or straight to typing. Was
/// `Ridley_Func_56`.
pub fn emergency_text_appears<L: Lease + ?Sized>(game: &mut L) {
    ridley::emergency_text_glow(game);
    let timer = session_slot!(game, Ridley[RIDLEY].fade_index);
    session_slot!(game, Ridley[RIDLEY].fade_index = timer.wrapping_sub(1));
    if timer == 1 {
        // Typewriter text - Ceres escape timer ($A6:C450).
        session_slot!(game, Ridley[RIDLEY].typewriter_list = 0xc450);
        session_slot!(game, Ridley[RIDLEY]; typewriter_tilemap = 0, typewriter_timer = 0,
            typewriter_timer_reset = 0, typewriter_stroke_timer = 0);
        session_slot!(game, Ridley[RIDLEY].fade_index = 32);
        if word!(game.JAPANESE_TEXT_FLAG) == 0 {
            next_step(game);
        }
        next_step(game);
    }
}

/// Self destruct sequence function index 8 - Japanese text appears ($A6:C0F5).
/// Was `Ridley_Func_57`.
pub fn japanese_text_appears<L: Lease + ?Sized>(game: &mut L) {
    let timer = session_slot!(game, Ridley[RIDLEY].fade_index);
    session_slot!(game, Ridley[RIDLEY].fade_index = timer.wrapping_sub(1));
    if timer == 1 {
        next_step(game);
        game.foreign(|game| game.queue_escape_tile_table());
    }
    type_self_destruct_text(game);
}

/// Self destruct sequence function index Ah - typing "self destruct sequence"
/// text ($A6:C104). Was `Ridley_C104`.
pub fn type_self_destruct_text<L: Lease + ?Sized>(game: &mut L) {
    ridley::emergency_text_glow(game);
    if game.foreign(|game| game.type_escape_text(ESCAPE_TEXT_TILES)) {
        next_step(game);
    }
}

/// Self destruct sequence function index Ch - start escape sequence
/// ($A6:C117): the timer starts and Ceres's boss is marked defeated. Was
/// `Ridley_Func_58`.
pub fn start_escape<L: Lease + ?Sized>(game: &mut L) {
    ridley::emergency_text_glow(game);
    transition_to_flying(game);
    session_slot!(game, Ridley[RIDLEY].function_timer = 0);
    session_slot!(
        game,
        Ridley[RIDLEY].function = ceres_function::SELF_DESTRUCT_ACTIVATED
    );
    word! {game; TIMER_STATUS = 1; CERES_STATUS = ESCAPE_STARTED; }
    game.foreign(|game| game.set_boss_bit(1));
}
