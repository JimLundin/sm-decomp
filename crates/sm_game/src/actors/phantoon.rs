//! Phantoon, the Wrecked Ship's ghost: its fight as a state machine over the
//! body's function word, the swoop, the palette fades and the death sequence.
//!
//! Was `Phantoon_*` in the original decompilation (bank $A7:CDF3-$A7:DD9B).
//! Phantoon is four enemies: the body in slot 0, then the eye, the tentacles
//! and the mouth. Each part's view names its slot words over the same RAM
//! bytes as `EnemyLayout8`; the figure-8 flight and the flames stay in
//! `phantoon_motion` and `phantoon_actions`.
use crate::actors::api::PlayerDirection;
use crate::actors::boss_lease::{BossLease, BossState};
use crate::actors::deaths::Deaths as _;
use crate::actors::host::BossEffects as _;
use crate::actors::host::{Death, Shot, below};
use crate::actors::layouts::{EnemyBase, same_words};
use crate::actors::session_slots::session_slot;
use crate::actors::{phantoon_actions, phantoon_motion};
use crate::constants::enemy_properties::{
    K_ENEMY_PROPS_DELETED, K_ENEMY_PROPS_DISABLE_SAMUS_COLL, K_ENEMY_PROPS_INVISIBLE,
    K_ENEMY_PROPS_TANGIBLE,
};
use crate::display::hdma_spawn::HdmaObjectDefinition;
use crate::enemy_projectiles::drops::DropSource;
use crate::menus::prelude::{owned, scroll};
use crate::phantoon_tables::{self, Timer};
use crate::ram::RawRam as _;
use crate::ram::{slot, word};

/// A lease on a game with Phantoon's effects.
pub trait Lease: BossLease<Game: Host> {}
impl<T: BossLease<Game: Host> + ?Sized> Lease for T {}

/// What Phantoon needs beyond the shared boss effects, deaths and flames.
pub trait Host:
    crate::actors::host::BossEffects
    + crate::actors::deaths::Deaths
    + phantoon_actions::Host
    + crate::records::Records
    + crate::cartridge::NativeImage
    + BossLease<Game = Self>
{
    /// Run the bank $A7 function at `address` for the current enemy.
    fn run_function(&mut self, address: u32);
    /// Which way Samus is from the current enemy.
    fn direction_to_player(&mut self) -> PlayerDirection;
    /// Spawn an HDMA object from bank `bank`.
    fn spawn_hdma(&mut self, bank: u8, args: &HdmaObjectDefinition);
    /// Start the wavy BG2 effect moving in `direction` with phase `speed`.
    fn start_wave(&mut self, direction: u16, speed: u16);
}

/// The body's view of its slot. `movement_index` also counts the opening
/// flames and the death explosion rounds.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Body {
    pub base: EnemyBase,
    pub movement_index: u16,
    pub speed_subpixel: u16,
    pub speed: u16,
    pub movement_stage: u16,
    pub timer: u16,
    pub function: u16,
    pad_0fb4: [u16; 1],
    pub flame_rain_next: u16,
}

same_words!(Body: EnemyLayout8 {
    movement_index = var_a, speed_subpixel = var_b, speed = var_c, movement_stage = var_d,
    timer = var_e, function = var_f, flame_rain_next = parameter_2,
});

/// The eye's view of its slot. In the death sequence `movement_reversed`
/// counts fades and then holds the fade brightness; while enraged
/// `fade_complete` counts the flame volleys.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Eye {
    pub base: EnemyBase,
    pub timer: u16,
    pub unread_shot_value: u16,
    pub movement_reversed: u16,
    pub fade_steps: u16,
    pub fade_step: u16,
    pub fade_complete: u16,
    pub background_held: u16,
    pub parameter_2: u16,
}

same_words!(Eye: EnemyLayout8 {
    timer = var_a, unread_shot_value = var_b, movement_reversed = var_c, fade_steps = var_d,
    fade_step = var_e, fade_complete = var_f, background_held = parameter_1,
    parameter_2 = parameter_2,
});

/// The tentacles' view of their slot: the swoop and the hurt flash.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Tentacles {
    pub base: EnemyBase,
    pub swooping: u16,
    pub damage_taken: u16,
    pub swoop_x_speed: u16,
    pub swoop_y_speed: u16,
    pub swoop_target_x: u16,
    pub explosion: u16,
    pub wave_direction: u16,
    pub hurt_flash: u16,
}

same_words!(Tentacles: EnemyLayout8 {
    swooping = var_a, damage_taken = var_b, swoop_x_speed = var_c, swoop_y_speed = var_d,
    swoop_target_x = var_e, explosion = var_f, wave_direction = parameter_1,
    hurt_flash = parameter_2,
});

/// The mouth's view of its slot: casual flames and the wavy effect.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Mouth {
    pub base: EnemyBase,
    pub casual_flame_pattern: u16,
    pub casual_flame_timer: u16,
    pub casual_flame_pattern_index: u16,
    pub wave_amplitude: u16,
    pub wave_shrinking: u16,
    pad_0fb2: [u16; 1],
    pub semi_transparency: u16,
    pub shot_roll: u16,
}

same_words!(Mouth: EnemyLayout8 {
    casual_flame_pattern = var_a, casual_flame_timer = var_b, casual_flame_pattern_index = var_c,
    wave_amplitude = var_d, wave_shrinking = var_e, semi_transparency = parameter_1,
    shot_roll = parameter_2,
});

const BODY: i32 = 0;
const EYE: i32 = 0x40;
const TENTACLES: i32 = 0x80;
const MOUTH: i32 = 0xc0;

/// Bank $A7 functions, stored in the body's `function` word.
pub mod function {
    pub const NOTHING: u16 = 0xd4a8;
    pub const SPAWN_STARTING_FLAMES: u16 = 0xd4a9;
    pub const WAIT_BEFORE_SPINNING_FLAMES: u16 = 0xd4ee;
    pub const SPAWN_FLAMES_BEFORE_FIGHT: u16 = 0xd508;
    pub const WAVY_FADE_IN: u16 = 0xd54a;
    pub const PICK_ROUND_1_PATTERN: u16 = 0xd596;
    pub const FIGURE_EIGHT: u16 = 0xd5e7;
    pub const EYE_FOLLOWS_SAMUS: u16 = 0xd60d;
    pub const BECOME_SOLID_AND_SWOOP: u16 = 0xd65c;
    pub const SWOOPING: u16 = 0xd678;
    pub const FADE_OUT_WITH_SWOOP: u16 = 0xd6b9;
    pub const WAIT_AFTER_FADE_OUT: u16 = 0xd6d4;
    pub const REAPPEAR: u16 = 0xd6e2;
    pub const FADE_IN_BEFORE_FIGURE_EIGHT: u16 = 0xd72d;
    pub const BECOME_SOLID_AFTER_FLAME_RAIN: u16 = 0xd73f;
    pub const FADE_IN_DURING_FLAME_RAIN: u16 = 0xd767;
    pub const FOLLOW_SAMUS_DURING_FLAME_RAIN: u16 = 0xd788;
    pub const FADE_OUT_DURING_FLAME_RAIN: u16 = 0xd7d5;
    pub const SPAWN_FLAME_RAIN: u16 = 0xd7f7;
    pub const FADE_OUT_BEFORE_FIRST_FLAME_RAIN: u16 = 0xd82a;
    pub const FADE_OUT_BEFORE_ENRAGE: u16 = 0xd85c;
    pub const MOVE_TO_TOP_CENTRE: u16 = 0xd874;
    pub const FADE_IN_ENRAGED: u16 = 0xd891;
    pub const ENRAGED: u16 = 0xd8ac;
    pub const FADE_OUT_AFTER_ENRAGE: u16 = 0xd916;
    pub const FINISH_SWOOP_AFTER_FATAL_SHOT: u16 = 0xd92e;
    pub const DYING_FADE_IN_OUT: u16 = 0xd948;
    pub const DYING_EXPLOSIONS: u16 = 0xd98b;
    pub const WAVY_DYING_CRY: u16 = 0xda51;
    pub const DYING_FADE_OUT: u16 = 0xda86;
    pub const ALMOST_DEAD: u16 = 0xdad7;
    pub const DEAD: u16 = 0xdb3d;
}

/// Bank $A7 instruction lists.
pub mod animation {
    pub const BODY_INVULNERABLE: u16 = 0xcc41;
    pub const BODY_FULL_HITBOX: u16 = 0xcc47;
    pub const BODY_EYE_HITBOX: u16 = 0xcc4d;
    pub const EYE_OPEN: u16 = 0xcc53;
    pub const EYE_CLOSE_AND_PICK_PATTERN: u16 = 0xcc81;
    pub const EYE_CLOSE: u16 = 0xcc91;
    pub const EYEBALL_CENTRED: u16 = 0xcc9d;
    /// The empty spritemap the other parts start with.
    pub const NOTHING_SPRITEMAP: u16 = 0x804d;
}

/// Bank $A7 tables.
/// Four bytes per death explosion: X and Y offsets, dust cloud kind, delay.
const DEATH_EXPLOSIONS: u32 = 0xa7_da1d;
/// Values written to the eye's unread word when shot ("values for $0FEA").
const SHOT_VALUES: [u8; 8] = [6, 6, 8, 8, 6, 8, 6, 8];

/// The intro wave: maximum amplitude C00h, amplitude delta 40h.
const INTRO_WAVE: i32 = (0xc00 << 16) | 0x40;
/// The dying wave: maximum amplitude F000h, amplitude delta 100h.
const DYING_WAVE: i32 = ((0xf000u32 << 16) | 0x100) as i32;
const INTRO_MAX_AMPLITUDE: u16 = 0xc00;
const WAVE_PHASE_DELTA: u16 = 8;

/// Bank $86 enemy projectiles.
const STARTING_FLAMES: u16 = 0x9c37;
const DUST_CLOUD: u16 = 0xe509;
/// The tiles and HDMA objects of Phantoon's BG2.
const BLANK_BG2_TILE: u16 = 824;
const BG2_TILEMAP_SIZE: u16 = 864;
const SEMI_TRANSPARENCY_HDMA: HdmaObjectDefinition = HdmaObjectDefinition::new(0x1, 0x14, 0xce96);
/// Health lost in one swoop or volley that makes Phantoon flee or enrage.
const ENRAGE_DAMAGE: u16 = 300;
/// The super missile projectile type bits.
const SUPER_MISSILE: u16 = 512;
/// Library 2 and 3 sounds.
const HIT_SOUND: u8 = 0x73;
const STARTING_FLAME_SOUND: u8 = 0x1d;
const ENRAGED_FLAMES_SOUND: u8 = 0x29;
const DYING_CRY: u8 = 0x7e;
const BIG_EXPLOSION_SOUND: u8 = 0x24;
const SMALL_EXPLOSION_SOUND: u8 = 0x2b;
/// Bit 14 of `phantom_related_layer_flag` hides BG2.
const HIDE_BG2: u16 = 0x4000;
/// Steps of a normal fade in or out.
const FADE_STEPS: u16 = 0xc;
/// RAM mirrors of the mosaic and main screen layer registers.
const MOSAIC: usize = 0x57;
const MAIN_SCREEN_LAYERS: usize = 0x69;
const BG2_ENABLED: u8 = 2;
/// Native RAM: the BG2 scroll HDMA table, Phantoon's palette 7 source
/// colours, and the ends of the regions the fight clears.
const BG2_SCROLL_TABLE: isize = 0x9100;
const SOURCE_PALETTES: isize = 0xc000;
const BG2_HDMA_DATA_END: u16 = 0x97fe;
const SPRITE_PALETTE_7_END: u16 = 0xc2fe;
const BODY_PALETTE_END: u16 = 0xc0fe;
const WHITE: u16 = 0x7fff;
/// The bank $84 PLMs spawned when the fight starts and when Phantoon dies.
const FIGHT_START_PLM: u16 = 0xb781;
const DEFEAT_PLM: u16 = 0xb78b;

fn current<S: BossState + ?Sized>(game: &S) -> i32 {
    game.current_enemy() as i32
}

fn native_word<S: BossState + ?Sized>(game: &S, at: isize) -> u16 {
    let at = usize::try_from(at).expect("native word outside RAM");
    u16::from_ne_bytes([game.slot_byte(at), game.slot_byte(at + 1)])
}

fn set_native_word<S: BossState + ?Sized>(game: &mut S, at: isize, value: u16) {
    let at = usize::try_from(at).expect("native word outside RAM");
    let bytes = u16::to_ne_bytes(value);
    game.set_slot_byte(at, bytes[0]);
    game.set_slot_byte(at + 1, bytes[1]);
}

/// Palette RAM word `index` of the colours Phantoon fades from.
fn palette_source(index: i32) -> isize {
    SOURCE_PALETTES + (index as isize) * 2
}

/// Counts down the function timer of part `k`; true when it runs out.
fn timer_expired<S: BossState + ?Sized>(game: &mut S, k: u16) -> bool {
    let me = k as i32;
    let last = session_slot!(game, Body[me].timer) == 1;
    session_slot!(game, Body[me].timer, |old| old.wrapping_sub(1));
    let negative = (session_slot!(game, Body[me].timer) & 0x8000) != 0;
    last || negative
}

/// Counts down the eye's timer; true when it runs out.
fn eye_timer_expired<S: BossState + ?Sized>(game: &mut S) -> bool {
    let last = session_slot!(game, Eye[EYE].timer) == 1;
    session_slot!(game, Eye[EYE].timer, |old| old.wrapping_sub(1));
    let negative = (session_slot!(game, Eye[EYE].timer) & 0x8000) != 0;
    last || negative
}

/// Closes the eye, makes the body invulnerable and hides BG2's background.
fn close_eye<S: BossState + ?Sized>(game: &mut S, eye_list: u16) {
    session_slot!(game, Body[BODY].base.instruction_timer = 1);
    session_slot!(game, Eye[EYE].base.instruction_timer = 1);
    session_slot!(
        game,
        Body[BODY].base.current_instruction = animation::BODY_INVULNERABLE
    );
    session_slot!(game, Eye[EYE].base.current_instruction = eye_list);
    session_slot!(
        game,
        Body[BODY].base.properties =
            (session_slot!(game, Body[BODY].base.properties)) | (K_ENEMY_PROPS_TANGIBLE as u16)
    );
}

/// Instruction: the materialisation sound, cycling through three. Was
/// `Phantoon_Func_1`.
pub fn materialisation_sound<S: BossState + ?Sized>(game: &mut S) {
    let sound = crate::boss_lists::phantoon_materialisation_sound(
        game,
        word!(game.PHANTOON_MATERIALISATION_SOUND_INDEX) as isize,
    );
    game.queue_sound(1, sound as u8, 6);
    let mut next: u16 = (word!(game.PHANTOON_MATERIALISATION_SOUND_INDEX)).wrapping_add(1);
    if !below(word!(game.PHANTOON_MATERIALISATION_SOUND_INDEX), 2) {
        next = 0;
    }
    word!(game.PHANTOON_MATERIALISATION_SOUND_INDEX = next);
}

/// Instruction: opens the eye for the figure-8's vulnerable window. Was
/// `Phantoon_StartTrackingSamusAndInitEyeTimer`.
pub fn open_eye<S: BossState + ?Sized>(game: &mut S) {
    session_slot!(game, Tentacles[TENTACLES].swooping = 0);
    session_slot!(game, Body[BODY].base; instruction_timer = 1, current_instruction = animation::BODY_EYE_HITBOX);
    session_slot!(
        game,
        Body[BODY].base.properties = (session_slot!(game, Body[BODY].base.properties) as i32
            & !(K_ENEMY_PROPS_TANGIBLE as i32)) as u16
    );
    let roll = (game.roll_random() as i32 & 7) as usize;
    session_slot!(
        game,
        Body[BODY].timer = phantoon_tables::timer(game, Timer::FigureEightVulnerable, roll)
    );
    session_slot!(game, Body[BODY].function = function::EYE_FOLLOWS_SAMUS);
    session_slot!(game, Eye[EYE].base; instruction_timer = 1, current_instruction = animation::EYEBALL_CENTRED);
}

/// Instruction: picks the next figure-8 direction and whether flame rain
/// comes first. Was `Phantoon_PickPatternForRound2`.
pub fn pick_pattern<S: BossState + ?Sized>(game: &mut S) {
    session_slot!(game, Body[BODY].timer = 60);
    let roll = (game.roll_random() as i32 & 7) as usize;
    let closed: u16 = phantoon_tables::timer(game, Timer::EyeClosed, roll);
    session_slot!(game, Eye[EYE].timer = closed);
    if (game.frame() & 1) != 0 {
        if session_slot!(game, Eye[EYE].movement_reversed) == 0 {
            let index: i16 =
                ((session_slot!(game, Body[BODY].movement_index)) as i16).wrapping_sub(1);
            session_slot!(game, Body[BODY].movement_index = index as u16);
            if index < 0 {
                session_slot!(game, Body[BODY].movement_index = 533);
            }
        }
        session_slot!(game, Body[BODY]; speed = 0, speed_subpixel = 0, movement_stage = 0);
        session_slot!(game, Eye[EYE].movement_reversed = 1);
    } else {
        if session_slot!(game, Eye[EYE].movement_reversed) != 0 {
            let index: u16 = (session_slot!(game, Body[BODY].movement_index)).wrapping_add(1);
            session_slot!(game, Body[BODY].movement_index = index);
            if !below(index, 534) {
                session_slot!(game, Body[BODY].movement_index = 0);
            }
        }
        session_slot!(game, Body[BODY].speed = 1);
        session_slot!(game, Body[BODY]; speed_subpixel = 0, movement_stage = 0);
        session_slot!(game, Eye[EYE].movement_reversed = 0);
    }
    if session_slot!(game, Body[BODY].flame_rain_next) != 0 {
        session_slot!(game, Eye[EYE].fade_complete = 0);
        session_slot!(
            game,
            Body[BODY].function = function::FADE_OUT_BEFORE_FIRST_FLAME_RAIN
        );
    } else {
        session_slot!(game, Body[BODY].function = function::FIGURE_EIGHT);
    }
}

/// Swoops toward Samus: X oscillates across the room, Y follows Samus (or
/// the floor once fatally shot). Was `Phantoon_MoveInSwoopingPattern`.
fn swoop<S: BossState + ?Sized>(game: &mut S, k: u16) {
    let mut target_x: u16;
    let previous: i16 = session_slot!(game, Tentacles[TENTACLES].swoop_target_x) as i16;
    if previous < 0 {
        let next: u16 = (previous as u16).wrapping_sub(2);
        session_slot!(game, Tentacles[TENTACLES].swoop_target_x = next);
        target_x = next & 0x7fff;
        if target_x == 0 {
            target_x = 0;
            session_slot!(game, Tentacles[TENTACLES].swoop_target_x = 0);
        }
    } else {
        target_x = (previous as u16).wrapping_add(2);
        session_slot!(game, Tentacles[TENTACLES].swoop_target_x = target_x);
        if !below(target_x, 256) {
            let next: u16 = target_x | 0x8000;
            session_slot!(game, Tentacles[TENTACLES].swoop_target_x = next);
            target_x = next & 0x7fff;
        }
    }
    if (target_x as i16).wrapping_sub((session_slot!(game, Body[BODY].base.x_pos)) as i16) < 0 {
        if (session_slot!(game, Tentacles[TENTACLES].swoop_x_speed) as i32 + 2047) & 0x8000 == 0 {
            session_slot!(
                game,
                Tentacles[TENTACLES].swoop_x_speed =
                    (session_slot!(game, Tentacles[TENTACLES].swoop_x_speed)).wrapping_sub(32)
            );
        }
    } else if below(
        session_slot!(game, Tentacles[TENTACLES].swoop_x_speed),
        2048,
    ) {
        session_slot!(
            game,
            Tentacles[TENTACLES].swoop_x_speed =
                (session_slot!(game, Tentacles[TENTACLES].swoop_x_speed)).wrapping_add(32)
        );
    }
    session_slot!(
        game,
        Body[BODY].base(x_pos, x_subpos) +=
            ((session_slot!(game, Tentacles[TENTACLES].swoop_x_speed) as i16 as i32) << 8) as u32
    );
    if (session_slot!(game, Body[BODY].base.x_pos) as i32 + 64) & 0x8000 != 0 {
        session_slot!(game, Body[BODY].base.x_pos = -64i32 as u16);
    } else if !below(session_slot!(game, Body[BODY].base.x_pos), 448) {
        session_slot!(game, Body[BODY].base.x_pos = 448);
    }
    let target_y: i16 = if session_slot!(game, Body[k as i32].function)
        == function::FINISH_SWOOP_AFTER_FATAL_SHOT
    {
        112
    } else {
        ((game.player_y()) as i16).wrapping_sub(48)
    };
    if target_y.wrapping_sub((session_slot!(game, Body[BODY].base.y_pos)) as i16) < 0 {
        if (session_slot!(game, Tentacles[TENTACLES].swoop_y_speed) as i32 + 1535) & 0x8000 == 0 {
            session_slot!(
                game,
                Tentacles[TENTACLES].swoop_y_speed =
                    (session_slot!(game, Tentacles[TENTACLES].swoop_y_speed)).wrapping_sub(64)
            );
        }
    } else if below(
        session_slot!(game, Tentacles[TENTACLES].swoop_y_speed),
        1536,
    ) {
        session_slot!(
            game,
            Tentacles[TENTACLES].swoop_y_speed =
                (session_slot!(game, Tentacles[TENTACLES].swoop_y_speed)).wrapping_add(64)
        );
    }
    session_slot!(
        game,
        Body[BODY].base(y_pos, y_subpos) +=
            ((session_slot!(game, Tentacles[TENTACLES].swoop_y_speed) as i16 as i32) << 8) as u32
    );
    if below(session_slot!(game, Body[BODY].base.y_pos), 64) {
        session_slot!(game, Body[BODY].base.y_pos = 64);
    } else if !below(session_slot!(game, Body[BODY].base.y_pos), 216) {
        session_slot!(game, Body[BODY].base.y_pos = 216);
    }
}

/// Was `Phantoon_BeginSwoopingPattern`.
fn begin_swoop<S: BossState + ?Sized>(game: &mut S, k: u16) {
    session_slot!(game, Tentacles[TENTACLES]; swoop_x_speed = 1024, swoop_y_speed = 1024);
    session_slot!(game, Tentacles[TENTACLES].swoop_target_x = 0);
    session_slot!(game, Body[k as i32]; function = function::SWOOPING, timer = 360);
}

/// Points the eyeball at Samus. Was `Phantoon_ChangeEyeSpriteBasedOnSamusDist`.
fn look_at_samus<S: BossState + ?Sized>(game: &mut S) {
    let direction: u16 = match game.player_direction() {
        PlayerDirection::Up => 0,
        PlayerDirection::UpRight => 1,
        PlayerDirection::Right => 2,
        PlayerDirection::DownRight => 3,
        PlayerDirection::Down => 4,
        PlayerDirection::DownLeft => 6,
        PlayerDirection::Left => 7,
        PlayerDirection::UpLeft => 8,
    };
    let row: u16 = 2u16.wrapping_mul(direction);
    session_slot!(game, Eye[EYE].base; instruction_timer = 1, current_instruction = crate::boss_lists::phantoon_instruction_list(game, (row >> 1) as isize));
}

/// Starts dying: finishing the swoop first if swooping. Was
/// `Phantoon_StartDeathSequence`.
fn start_dying<S: BossState + ?Sized>(game: &mut S, k: u16) {
    let me = k as i32;
    if session_slot!(game, Body[me].function) == function::SWOOPING
        || session_slot!(game, Body[me].function) == function::FADE_OUT_WITH_SWOOP
    {
        session_slot!(
            game,
            Body[me].function = function::FINISH_SWOOP_AFTER_FATAL_SHOT
        );
    } else {
        session_slot!(game, Body[me].function = function::DYING_FADE_IN_OUT);
    }
    session_slot!(game, Eye[EYE]; movement_reversed = 0, fade_complete = 0);
    look_at_samus(game);
    let scroll: u16 = scroll!(game.Bg2H);
    for offset in (0i16..=510).rev().step_by(2) {
        set_native_word(
            game,
            BG2_SCROLL_TABLE + (offset as u16 as i32 as isize),
            scroll,
        );
    }
    word!(game.PHANTOM_RELATED_LAYER_FLAG |= HIDE_BG2);
    session_slot!(game, Mouth[MOUTH].shot_roll = 1);
}

/// Fades the body palette one step toward black, every other frame, until
/// `steps` steps. Was `Phantoon_FadeOut`.
fn fade_out<S: BossState + ?Sized>(game: &mut S, steps: u16) {
    if (game.frame() & 1) == 0 && session_slot!(game, Eye[EYE].fade_complete) == 0 {
        session_slot!(game, Eye[EYE].fade_steps = steps);
        if fade_step_toward_target(game) & 1 != 0 {
            session_slot!(game, Eye[EYE].fade_complete = 1);
        }
    }
}

/// Fades the body palette one step toward its health colours, every other
/// frame, until `steps` steps. Was `Phantoon_FadeIn`.
fn fade_in<S: BossState + ?Sized>(game: &mut S, steps: u16) {
    if (game.frame() & 1) == 0 && session_slot!(game, Eye[EYE].fade_complete) == 0 {
        session_slot!(game, Eye[EYE].fade_steps = steps);
        if fade_step_toward_health(game) & 1 != 0 {
            session_slot!(game, Eye[EYE].fade_complete = 1);
        }
    }
}

/// One step of palette 7 toward the fade-out target; 1 once done. Was
/// `Phantoon_Func_8`.
fn fade_step_toward_target<S: BossState + ?Sized>(game: &mut S) -> u8 {
    if (session_slot!(game, Eye[EYE].fade_steps)).wrapping_add(1)
        >= session_slot!(game, Eye[EYE].fade_step)
    {
        for i in (0i32..32).step_by(2) {
            word!(
                game.PALETTE_BUFFER[((i >> 1) + 112)] = transition_color(
                    session_slot!(game, Eye[EYE].fade_step),
                    native_word(game, palette_source((i >> 1) + 112)),
                    crate::boss_lists::fade_out_target_colour(game, (i >> 1) as isize),
                    game,
                )
            );
        }
        session_slot!(game, Eye[EYE].fade_step, |old| old.wrapping_add(1));
        0
    } else {
        session_slot!(game, Eye[EYE].fade_step = 0);
        1
    }
}

/// One step of palette 7 toward the health palette; 1 once done. Was
/// `Phantoon_SetColorBasedOnHp`.
fn fade_step_toward_health<S: BossState + ?Sized>(game: &mut S) -> u8 {
    if (session_slot!(game, Eye[EYE].fade_steps)).wrapping_add(1)
        >= session_slot!(game, Eye[EYE].fade_step)
    {
        for base in (0u16..32).step_by(2) {
            let (index, colour) = phantoon_actions::health_palette_colour(base, game);
            let from = native_word(game, palette_source((index as i32 >> 1) + 112));
            let value =
                transition_color(session_slot!(game, Eye[EYE].fade_step), from, colour, game);
            set_native_word(game, palette_source((base as i32 >> 1) + 112), value);
        }
        session_slot!(game, Eye[EYE].fade_step, |old| old.wrapping_add(1));
        0
    } else {
        session_slot!(game, Eye[EYE].fade_step = 0);
        1
    }
}

/// One step of BG palettes 0-6 toward the Wrecked Ship's powered-on colours;
/// 1 once done. Was `Phantoon_Func_9`.
fn fade_step_toward_powered_on<S: BossState + ?Sized>(game: &mut S) -> u8 {
    if (session_slot!(game, Eye[EYE].fade_steps)).wrapping_add(1)
        >= session_slot!(game, Eye[EYE].fade_step)
    {
        for i in (0i32..224).step_by(2) {
            word!(
                game.PALETTE_BUFFER[(i >> 1)] = transition_color(
                    session_slot!(game, Eye[EYE].fade_step),
                    word!(game.PALETTE_BUFFER[(i >> 1)]),
                    crate::boss_lists::wrecked_ship_powered_on_colour(game, (i >> 1) as isize),
                    game,
                )
            );
        }
        session_slot!(game, Eye[EYE].fade_step, |old| old.wrapping_add(1));
        0
    } else {
        session_slot!(game, Eye[EYE].fade_step = 0);
        1
    }
}

/// Colour `step` of the fade from BGR555 `from` to `to`. Was
/// `Phantoon_Func_10_CalculateNthTransitionColorFromXtoY`.
fn transition_color<S: BossState + ?Sized>(step: u16, from: u16, to: u16, game: &S) -> u16 {
    (transition_component(step, from & 0x1f, to & 0x1f, game) as i32
        + ((transition_component(
            step,
            (from as i32 >> 5 & 0x1f) as u16,
            (to as i32 >> 5 & 0x1f) as u16,
            game,
        ) as i32)
            << 5)
        + ((transition_component(
            step,
            (from as i32 >> 10 & 0x1f) as u16,
            (to as i32 >> 10 & 0x1f) as u16,
            game,
        ) as i32)
            << 10)) as u16
}

/// One 5-bit component of [`transition_color`]. Was
/// `Phantoon_CalculateNthTransitionColorComponentFromXtoY`.
fn transition_component<S: BossState + ?Sized>(step: u16, from: u16, to: u16, game: &S) -> u16 {
    if step == 0 {
        return from;
    }
    let previous: u16 = step.wrapping_sub(1);
    if previous == session_slot!(game, Eye[EYE].fade_steps) {
        return to;
    }
    let done: u16 = previous.wrapping_add(1);
    let mut delta: u16 = to.wrapping_sub(from);
    let magnitude: u8 = crate::actors::host::abs16(delta) as u8;
    let divisor = (session_slot!(byte game, Eye[EYE].fade_steps[0]) as i32 - done as i32 + 1) as u8;
    let quotient: u16 = divide((magnitude as i32) << 8, divisor);
    delta = (if delta & 0x8000 != 0 {
        -(quotient as i32)
    } else {
        quotient as i32
    }) as u16;
    (delta.wrapping_add(u16::swap_bytes(from))) >> 8
}

/// The SNES hardware divide: FFFFh for a zero divisor.
fn divide(dividend: i32, divisor: u8) -> u16 {
    let dividend = dividend as u16;
    (if divisor as i32 == 0 {
        0xffff
    } else {
        dividend as i32 / divisor as i32
    }) as u16
}

/// Sets up the fight: blank BG2, the semi-transparency HDMA and the parts.
/// Was `Phantoon_Init`.
pub fn initialize<H: Host + ?Sized>(game: &mut H) {
    crate::word_transfers!(game, descending, 2048; le(TILEMAP_STUFF, 2047) <- value(BLANK_BG2_TILE));
    crate::word_transfers!(game, descending, 1024; native(BG2_HDMA_DATA_END) <- value(0));
    crate::word_transfers!(game, descending, 16; native(SPRITE_PALETTE_7_END) <- value(0));
    word!(game.ENEMY_BG2_TILEMAP_SIZE = BG2_TILEMAP_SIZE);
    game.disable_minimap_for_boss();
    let me = current(game);
    slot!(game, Body[me].timer = 120);
    slot!(game, Body[me]; movement_index = 0, speed_subpixel = 0);
    word!(game.PHANTOON_MATERIALISATION_SOUND_INDEX = 0);
    slot!(game, Eye[EYE]; background_held = 0, parameter_2 = 0);
    slot!(
        game,
        Body[BODY].base.properties =
            (slot!(game, Body[BODY].base.properties)) | (K_ENEMY_PROPS_TANGIBLE as u16)
    );
    game.spawn_hdma(0xa7, &SEMI_TRANSPARENCY_HDMA);
    initialize_part(game);
}

/// Sets up one part with the body's graphics and its instruction list. Was
/// `Phantoon2_Init`.
pub fn initialize_part<S: BossState + ?Sized>(game: &mut S) {
    let me = current(game);
    session_slot!(
        game,
        Body[me].base.spritemap_pointer = animation::NOTHING_SPRITEMAP
    );
    session_slot!(game, Body[me].base; instruction_timer = 1, timer = 0);
    let copied = session_slot!(game, Body[BODY].base.palette_index);
    session_slot!(game, Body[me].base.palette_index = copied);
    let copied = session_slot!(game, Body[BODY].base.vram_tiles_index);
    session_slot!(game, Body[me].base.vram_tiles_index = copied);
    session_slot!(
        game,
        Body[me].base.current_instruction = crate::boss_lists::phantoon_flame_instruction_list(
            game,
            session_slot!(game, Body[me].flame_rain_next) as isize
        )
    );
    session_slot!(game, Body[me].function = function::SPAWN_STARTING_FLAMES);
    session_slot!(game, Mouth[MOUTH]; semi_transparency = 0,
        casual_flame_pattern_index = -1i32 as u16);
}

/// Runs the current function; the body carries the other parts and BG2.
/// Was `Phantoon_Main`.
pub fn main_ai<L: Lease + ?Sized>(game: &mut L) {
    // The bank $A7 function runs on the lease, the effects that need the
    // game in grouped foreign calls.
    phantoon_actions::broken_nothingness(game.current_enemy(), game);
    let me = current(game);
    let function = session_slot!(game, Body[me].function);
    run_function_on(game, function);
    if game.current_enemy() == 0 {
        let x: u16 = session_slot!(game, Body[BODY].base.x_pos);
        session_slot!(game, Eye[EYE].base.x_pos = x);
        session_slot!(game, Tentacles[TENTACLES].base.x_pos = x);
        session_slot!(game, Mouth[MOUTH].base.x_pos = x);
        let y: u16 = session_slot!(game, Body[BODY].base.y_pos);
        session_slot!(game, Eye[EYE].base.y_pos = y);
        session_slot!(game, Tentacles[TENTACLES].base.y_pos = y);
        session_slot!(game, Mouth[MOUTH].base.y_pos = y);
        if session_slot!(game, Eye[EYE].background_held) == 0 {
            scroll!(
                game.Bg2H = (game.camera().layer1[0] as i32
                    - session_slot!(game, Body[BODY].base.x_pos) as i32
                    + 40) as u16
            );
            scroll!(
                game.Bg2V = (game.camera().layer1[1] as i32
                    - session_slot!(game, Body[BODY].base.y_pos) as i32
                    + 40) as u16
            );
        }
    }
}

/// Run the bank $A7 function `function` for the current part, as the bank's
/// pre-instruction dispatch does: Phantoon's on this lease, any other on the
/// game.
fn run_function_on<L: Lease + ?Sized>(game: &mut L, function: u16) {
    let k = game.current_enemy();
    match function {
        function::WAVY_FADE_IN => wavy_fade_in(game, k),
        function::FIGURE_EIGHT => figure_eight(game, k),
        function::EYE_FOLLOWS_SAMUS => eye_follows_samus(game, k),
        function::SWOOPING => swooping(game, k),
        function::FADE_OUT_WITH_SWOOP => fade_out_with_swoop(game, k),
        function::REAPPEAR => reappear(game, k),
        function::FADE_IN_BEFORE_FIGURE_EIGHT => fade_in_before_figure_eight(game),
        function::BECOME_SOLID_AFTER_FLAME_RAIN => become_solid_after_flame_rain(game, k),
        function::FADE_IN_DURING_FLAME_RAIN => fade_in_during_flame_rain(game, k),
        function::FADE_OUT_DURING_FLAME_RAIN => fade_out_during_flame_rain(game, k),
        function::FADE_OUT_BEFORE_FIRST_FLAME_RAIN => fade_out_before_first_flame_rain(game, k),
        function::FADE_OUT_BEFORE_ENRAGE => fade_out_before_enrage(game, k),
        function::FADE_IN_ENRAGED => fade_in_enraged(game, k),
        function::ENRAGED => enraged(game, k),
        function::FADE_OUT_AFTER_ENRAGE => fade_out_after_enrage(game, k),
        function::FINISH_SWOOP_AFTER_FATAL_SHOT => finish_swoop_after_fatal_shot(game, k),
        function::DYING_FADE_IN_OUT => dying_fade_in_out(game, k),
        function::WAVY_DYING_CRY => wavy_dying_cry(game, k),
        function::DYING_FADE_OUT => dying_fade_out(game, k),
        function::ALMOST_DEAD => almost_dead(game, k),
        function::DEAD => dead(game, k),
        function::BECOME_SOLID_AND_SWOOP => become_solid_and_swoop(game, k),
        function::SPAWN_STARTING_FLAMES
        | function::WAIT_BEFORE_SPINNING_FLAMES
        | function::SPAWN_FLAMES_BEFORE_FIGHT
        | function::PICK_ROUND_1_PATTERN
        | function::WAIT_AFTER_FADE_OUT
        | function::FOLLOW_SAMUS_DURING_FLAME_RAIN
        | function::SPAWN_FLAME_RAIN
        | function::MOVE_TO_TOP_CENTRE
        | function::DYING_EXPLOSIONS => countdown(game, k, function),
        _ => game.foreign(|game| game.run_function(u32::from(function) | 0xa7_0000)),
    }
}

/// Flashes white when hurt, then restores the health palette. Was
/// `Phantoon_Hurt`.
pub fn hurt<S: BossState + ?Sized>(game: &mut S) {
    if session_slot!(game, Body[BODY].base.flash_timer) != 8 {
        if (session_slot!(game, Body[BODY].base.frame_counter) & 2) == 0 {
            if session_slot!(byte game, Tentacles[TENTACLES].hurt_flash[1]) == 0 {
                return;
            }
        } else {
            if session_slot!(byte game, Tentacles[TENTACLES].hurt_flash[1]) == 0 {
                crate::word_transfers!(game, descending, 16; native(BODY_PALETTE_END) <- value(WHITE));
                session_slot!(
                    game,
                    Tentacles[TENTACLES].hurt_flash =
                        (session_slot!(game, Tentacles[TENTACLES].hurt_flash)) | 0x100
                );
            }
            return;
        }
    }
    let mut base: u16 = 30;
    loop {
        let (index, colour) = phantoon_actions::health_palette_colour(base, game);
        set_native_word(game, palette_source((index as i32 >> 1) + 112), colour);
        base = index.wrapping_sub(2);
        if (index as i16).wrapping_sub(2) < 0 {
            break;
        }
    }
    session_slot!(
        game,
        Tentacles[TENTACLES].hurt_flash =
            session_slot!(byte game, Tentacles[TENTACLES].hurt_flash[0]) as u16
    );
}

/// Touched by Samus. Was `Phantoon_Touch`.
pub fn touch<S: BossState + ?Sized>(game: &mut S) {
    game.react_to_contact(Death::LeavePending);
}

/// Hit by a shot: dies, or after enough damage flees into a swoop, a flame
/// rain or (to a super missile) enrages. Was `Phantoon_Shot`.
pub fn shot<S: BossState + ?Sized>(game: &mut S) {
    // Once dying, shots do nothing.
    if (session_slot!(game, Body[BODY].function) as i32 + 0x26b8) & 0x8000 == 0 {
        return;
    }
    let k: u16 = game.current_enemy();
    let me = current(game);
    let mut health: u16 = session_slot!(game, Body[me].base.health);
    game.react_to_shot(Shot::ReactionOnly);
    if session_slot!(game, Body[me].base.health) == 0 {
        game.queue_sound(1, HIT_SOUND, 6);
        session_slot!(game, Tentacles[TENTACLES].hurt_flash = 1);
        session_slot!(
            game,
            Body[BODY].base.properties =
                (session_slot!(game, Body[BODY].base.properties)) | (K_ENEMY_PROPS_TANGIBLE as u16)
        );
        start_dying(game, k);
        return;
    }
    if (session_slot!(game, Body[me].base.ai_handler_bits) & 2) != 0 {
        game.queue_sound(1, HIT_SOUND, 6);
        let function: u16 = session_slot!(game, Body[me].function);
        'react_to_hit: {
            let initial_hit = function == function::EYE_FOLLOWS_SAMUS
                || function == function::FOLLOW_SAMUS_DURING_FLAME_RAIN;
            if !initial_hit && function != function::SWOOPING {
                break 'react_to_hit;
            }
            health = health.wrapping_sub(session_slot!(game, Body[me].base.health));
            if below(health, ENRAGE_DAMAGE)
                || (word!(game.PROJECTILE_TYPE[word!(game.COLLISION_DETECTION_INDEX)]) & 0xf00)
                    != SUPER_MISSILE
            {
                let tentacles = k as i32 + TENTACLES;
                let damage =
                    health.wrapping_add(session_slot!(game, Tentacles[tentacles].damage_taken));
                session_slot!(game, Tentacles[tentacles].damage_taken = damage);
                if !initial_hit {
                    if !below(damage, ENRAGE_DAMAGE) {
                        session_slot!(game, Body[me].timer = 1);
                    }
                    break 'react_to_hit;
                }
                if !below(damage, ENRAGE_DAMAGE) {
                    session_slot!(game, Body[me].function = function::FADE_OUT_WITH_SWOOP);
                } else {
                    let roll: u16 = game.roll_random() & 7;
                    session_slot!(
                        game,
                        Eye[EYE].unread_shot_value = SHOT_VALUES[roll as i32 as usize] as u16
                    );
                    session_slot!(game, Mouth[MOUTH].shot_roll = roll);
                    session_slot!(game, Tentacles[TENTACLES].hurt_flash = 1);
                    if session_slot!(game, Tentacles[TENTACLES].swooping) == 0 {
                        session_slot!(game, Tentacles[TENTACLES].swooping = 1);
                        if !below(session_slot!(game, Body[me].timer), 16) {
                            session_slot!(game, Body[me].timer = 16);
                        }
                    }
                    return;
                }
            } else {
                session_slot!(game, Body[me].function = function::FADE_OUT_BEFORE_ENRAGE);
            }
            session_slot!(game, Body[me].timer = 0);
            session_slot!(game, Tentacles[TENTACLES].swooping = 0);
            session_slot!(game, Tentacles[k as i32 + TENTACLES].damage_taken = 0);
            word!(game.PHANTOM_RELATED_LAYER_FLAG |= HIDE_BG2);
            session_slot!(game, Body[BODY].base.instruction_timer = 1);
            session_slot!(game, Eye[EYE].base.instruction_timer = 1);
            session_slot!(
                game,
                Body[BODY].base.current_instruction = animation::BODY_INVULNERABLE
            );
            session_slot!(
                game,
                Eye[EYE].base.current_instruction = animation::EYE_CLOSE
            );
            session_slot!(
                game,
                Body[BODY].base.properties = (session_slot!(game, Body[BODY].base.properties))
                    | (K_ENEMY_PROPS_TANGIBLE as u16)
            );
            session_slot!(game, Eye[EYE].fade_complete = 0);
        }
        session_slot!(game, Tentacles[TENTACLES].hurt_flash = 2);
    }
}

/// Fades in while the wave grows, then picks the first pattern. Was
/// `Phantoon_WavyFadeIn`.
pub fn wavy_fade_in<S: BossState + ?Sized>(game: &mut S, k: u16) {
    fade_in(game, FADE_STEPS);
    if phantoon_actions::grow_or_shrink_wave_amplitude(INTRO_WAVE, game) != 0 {
        session_slot!(
            game,
            Body[k as i32].function = function::PICK_ROUND_1_PATTERN
        );
        session_slot!(game, Mouth[MOUTH].semi_transparency = 1);
        session_slot!(game, Body[k as i32].timer = 30);
    } else {
        let me = k as i32;
        let last: bool = session_slot!(game, Body[me].timer) == 1;
        session_slot!(game, Body[me].timer, |old| old.wrapping_sub(1));
        let negative: bool = (session_slot!(game, Body[me].timer) & 0x8000) != 0;
        if last || negative {
            session_slot!(game, Mouth[MOUTH].wave_shrinking = 1);
        }
    }
}

/// Flies the figure-8 with the eye shut, then opens it with a flame spiral.
/// Was `Phantoon_MovePhantoonInFigure8ThenOpenEye`.
pub fn figure_eight<S: BossState + ?Sized>(game: &mut S, k: u16) {
    phantoon_motion::fly_figure_eight(k, game);
    if eye_timer_expired(game) {
        session_slot!(game, Body[k as i32].function = function::NOTHING);
        session_slot!(game, Eye[EYE].base; instruction_timer = 1, current_instruction = animation::EYE_OPEN);
        session_slot!(game, Body[BODY].flame_rain_next = 0);
        phantoon_actions::spawn_flame_spiral(k, game);
    }
}

/// The vulnerable window: the eye tracks Samus until the timer runs out, then
/// Phantoon swoops if struck or closes its eye. Was
/// `Phantoon_EyeFollowsSamusUntilTimerRunsOut`.
pub fn eye_follows_samus<S: BossState + ?Sized>(game: &mut S, k: u16) {
    if timer_expired(game, k) {
        session_slot!(game, Tentacles[k as i32 + TENTACLES].damage_taken = 0);
        if session_slot!(game, Tentacles[TENTACLES].swooping) == 0 {
            session_slot!(game, Body[k as i32].function = function::NOTHING);
            session_slot!(game, Body[BODY].base.instruction_timer = 1);
            session_slot!(game, Eye[EYE].base.instruction_timer = 1);
            session_slot!(
                game,
                Body[BODY].base.current_instruction = animation::BODY_INVULNERABLE
            );
            session_slot!(
                game,
                Eye[EYE].base.current_instruction = animation::EYE_CLOSE_AND_PICK_PATTERN
            );
            session_slot!(
                game,
                Body[BODY].base.properties = (session_slot!(game, Body[BODY].base.properties))
                    | (K_ENEMY_PROPS_TANGIBLE as u16)
            );
            session_slot!(game, Body[BODY].flame_rain_next = 1);
            return;
        }
        session_slot!(game, Tentacles[TENTACLES].swooping = 0);
        session_slot!(game, Body[k as i32].timer = 60);
        session_slot!(
            game,
            Body[k as i32].function = function::BECOME_SOLID_AND_SWOOP
        );
    }
    look_at_samus(game);
}

/// Swoops until the timer runs out, then fades out. Was `Phantoon_IsSwooping`.
pub fn swooping<S: BossState + ?Sized>(game: &mut S, k: u16) {
    look_at_samus(game);
    swoop(game, k);
    if timer_expired(game, k) {
        session_slot!(
            game,
            Body[k as i32].function = function::FADE_OUT_WITH_SWOOP
        );
        word!(game.PHANTOM_RELATED_LAYER_FLAG |= HIDE_BG2);
        close_eye(game, animation::EYE_CLOSE);
        session_slot!(game, Eye[EYE].fade_complete = 0);
        session_slot!(game, Tentacles[k as i32 + TENTACLES].damage_taken = 0);
    }
}

/// Was `Phantoon_FadeoutWithSwoop`.
pub fn fade_out_with_swoop<S: BossState + ?Sized>(game: &mut S, k: u16) {
    swoop(game, k);
    fade_out(game, FADE_STEPS);
    if session_slot!(game, Eye[EYE].fade_complete) != 0 {
        session_slot!(
            game,
            Body[k as i32].function = function::WAIT_AFTER_FADE_OUT
        );
        session_slot!(game, Body[k as i32].timer = 120);
    }
}

/// Reappears on a random side for a new figure-8. Was
/// `Phantoon_MoveLeftOrRightAndPickEyeOpenPatt`.
pub fn reappear<S: BossState + ?Sized>(game: &mut S, k: u16) {
    if (game.roll_random() & 1) != 0 {
        session_slot!(game, Body[BODY].movement_index = 136);
        session_slot!(game, Body[BODY].base.x_pos = 208);
    } else {
        session_slot!(game, Body[BODY].movement_index = 399);
        session_slot!(game, Body[BODY].base.x_pos = 48);
    }
    session_slot!(game, Body[BODY].base.y_pos = 96);
    session_slot!(game, Eye[EYE].movement_reversed = 0);
    session_slot!(game, Body[BODY].speed = 1);
    session_slot!(game, Body[BODY]; speed_subpixel = 0, flame_rain_next = 0);
    pick_pattern(game);
    session_slot!(
        game,
        Body[k as i32].function = function::FADE_IN_BEFORE_FIGURE_EIGHT
    );
    session_slot!(game, Eye[EYE].fade_complete = 0);
}

/// Was `Phantoon_FadeInBeforeFigure8`.
pub fn fade_in_before_figure_eight<S: BossState + ?Sized>(game: &mut S) {
    fade_in(game, FADE_STEPS);
    if session_slot!(game, Eye[EYE].fade_complete) != 0 {
        session_slot!(game, Body[BODY].function = function::FIGURE_EIGHT);
    }
}

/// Was `Phantoon_BecomeSolidAfterRainingFireballs`.
pub fn become_solid_after_flame_rain<S: BossState + ?Sized>(game: &mut S, k: u16) {
    session_slot!(game, Eye[EYE].fade_complete = 0);
    word!(game.PHANTOM_RELATED_LAYER_FLAG &= !HIDE_BG2);
    session_slot!(game, Body[BODY].base.instruction_timer = 1);
    session_slot!(game, Eye[EYE].base.instruction_timer = 1);
    session_slot!(
        game,
        Body[BODY].base.current_instruction = animation::BODY_FULL_HITBOX
    );
    session_slot!(
        game,
        Eye[EYE].base.current_instruction = animation::EYEBALL_CENTRED
    );
    session_slot!(
        game,
        Body[k as i32].function = function::FADE_IN_DURING_FLAME_RAIN
    );
}

/// Was `Phantoon_FadeInDuringFireballRain`.
pub fn fade_in_during_flame_rain<S: BossState + ?Sized>(game: &mut S, k: u16) {
    fade_in(game, 1);
    if session_slot!(game, Eye[EYE].fade_complete) != 0 {
        session_slot!(
            game,
            Body[BODY].base.properties = (session_slot!(game, Body[BODY].base.properties) as i32
                & !(K_ENEMY_PROPS_TANGIBLE as i32)) as u16
        );
        session_slot!(
            game,
            Body[k as i32].function = function::FOLLOW_SAMUS_DURING_FLAME_RAIN
        );
        session_slot!(game, Body[BODY].timer = 90);
    }
}

/// Was `Phantoon_FadeOutDuringFireballRain`.
pub fn fade_out_during_flame_rain<S: BossState + ?Sized>(game: &mut S, k: u16) {
    fade_out(game, FADE_STEPS);
    if session_slot!(game, Eye[EYE].fade_complete) != 0 {
        session_slot!(game, Body[k as i32].function = function::SPAWN_FLAME_RAIN);
        let roll = (game.roll_random() as i32 & 7) as usize;
        session_slot!(
            game,
            Body[k as i32].timer = phantoon_tables::timer(game, Timer::FlameRainHiding, roll)
        );
    }
}

/// Fades out on the figure-8, then rains flames on Samus's side. Was
/// `Phantoon_FadeOutBeforeFirstFireballRain`.
pub fn fade_out_before_first_flame_rain<S: BossState + ?Sized>(game: &mut S, k: u16) {
    fade_out(game, FADE_STEPS);
    phantoon_motion::fly_figure_eight(k, game);
    if eye_timer_expired(game) {
        session_slot!(game, Tentacles[TENTACLES].swooping = 0);
        session_slot!(
            game,
            Body[k as i32].function = function::BECOME_SOLID_AFTER_FLAME_RAIN
        );
        if below(session_slot!(game, Body[BODY].base.x_pos), 128) {
            phantoon_actions::spawn_flame_rain(k, 0, game);
        } else {
            phantoon_actions::spawn_flame_rain(k, 2, game);
        }
    }
}

/// Was `Phantoon_FadeOutBeforeEnrage`.
pub fn fade_out_before_enrage<S: BossState + ?Sized>(game: &mut S, k: u16) {
    fade_out(game, FADE_STEPS);
    if session_slot!(game, Eye[EYE].fade_complete) != 0 {
        session_slot!(game, Body[k as i32]; function = function::MOVE_TO_TOP_CENTRE, timer = 120);
    }
}

/// Was `Phantoon_FadeInEnragedPhantoon`.
pub fn fade_in_enraged<S: BossState + ?Sized>(game: &mut S, k: u16) {
    fade_in(game, FADE_STEPS);
    if session_slot!(game, Eye[EYE].fade_complete) != 0 {
        session_slot!(game, Body[BODY].function = function::ENRAGED);
        session_slot!(game, Body[k as i32].timer = 4);
        session_slot!(game, Eye[EYE].fade_complete = 0);
    }
}

/// Fires eight volleys of flames, then closes its eye. Was `Phantoon_Enraged`.
pub fn enraged<S: BossState + ?Sized>(game: &mut S, k: u16) {
    if timer_expired(game, k) {
        if (session_slot!(game, Eye[EYE].fade_complete) & 1) != 0 {
            for flame in (8u16..=15).rev() {
                game.spawn_actor_eproj(flame | 0x200, k, phantoon_actions::DESTROYABLE_FLAMES);
            }
        } else {
            for flame in (0u16..=6).rev() {
                game.spawn_actor_eproj(flame | 0x200, k, phantoon_actions::DESTROYABLE_FLAMES);
            }
        }
        game.queue_sound(2, ENRAGED_FLAMES_SOUND, 6);
        let volleys: u16 = (session_slot!(game, Eye[EYE].fade_complete)).wrapping_add(1);
        session_slot!(game, Eye[EYE].fade_complete = volleys);
        if below(volleys, 8) {
            session_slot!(game, Body[k as i32].timer = 128);
        } else {
            session_slot!(game, Eye[EYE].base; instruction_timer = 1, current_instruction = animation::EYE_CLOSE);
            session_slot!(game, Eye[EYE].fade_complete = 0);
            session_slot!(
                game,
                Body[k as i32].function = function::FADE_OUT_AFTER_ENRAGE
            );
        }
    }
}

/// Was `Phantoon_FadeoutAfterEnrage`.
pub fn fade_out_after_enrage<S: BossState + ?Sized>(game: &mut S, k: u16) {
    fade_out(game, FADE_STEPS);
    if session_slot!(game, Eye[EYE].fade_complete) != 0 {
        session_slot!(
            game,
            Body[k as i32].function = function::WAIT_AFTER_FADE_OUT
        );
        session_slot!(game, Body[k as i32].timer = 120);
    }
}

/// Swoops to the middle of the room once fatally shot mid-swoop. Was
/// `Phantoon_CompleteSwoopAfterFatalShot`.
pub fn finish_swoop_after_fatal_shot<S: BossState + ?Sized>(game: &mut S, k: u16) {
    look_at_samus(game);
    swoop(game, k);
    let x: u16 = session_slot!(game, Body[BODY].base.x_pos);
    if !below(x, 96) && below(x, 160) {
        session_slot!(game, Body[k as i32].function = function::DYING_FADE_IN_OUT);
    }
}

/// Flickers in and out ten times, then starts the explosions. Was
/// `Phantoon_DyingPhantoonFadeInOut`.
pub fn dying_fade_in_out<S: BossState + ?Sized>(game: &mut S, k: u16) {
    if (session_slot!(game, Eye[EYE].movement_reversed) & 1) != 0 {
        fade_in(game, FADE_STEPS);
        if session_slot!(game, Eye[EYE].fade_complete) == 0 {
            return;
        }
    } else {
        fade_out(game, FADE_STEPS);
        if session_slot!(game, Eye[EYE].fade_complete) == 0 {
            return;
        }
    }
    session_slot!(game, Eye[EYE].fade_complete = 0);
    let fades: u16 = (session_slot!(game, Eye[EYE].movement_reversed)).wrapping_add(1);
    session_slot!(game, Eye[EYE].movement_reversed = fades);
    if !below(fades, 10) {
        session_slot!(game, Body[k as i32]; function = function::DYING_EXPLOSIONS, timer = 15);
        session_slot!(game, Tentacles[TENTACLES].explosion = 0);
        session_slot!(game, Body[BODY].movement_index = 0);
    }
}

/// Starts the dying wave and cry. Was `Phantoon_WavyDyingPhantoonAndCry`.
pub fn wavy_dying_cry<L: Lease + ?Sized>(game: &mut L, k: u16) {
    game.foreign(|game| game.start_wave(1, WAVE_PHASE_DELTA));
    session_slot!(game, Mouth[MOUTH].wave_amplitude = 0);
    session_slot!(game, Body[k as i32].function = function::DYING_FADE_OUT);
    session_slot!(game, Eye[EYE].movement_reversed = 2);
    let hidden: u16 = (session_slot!(game, Body[BODY].base.properties) as i32
        & !(K_ENEMY_PROPS_DISABLE_SAMUS_COLL as i32
            | K_ENEMY_PROPS_TANGIBLE as i32
            | K_ENEMY_PROPS_INVISIBLE as i32)
        | K_ENEMY_PROPS_TANGIBLE as i32
        | K_ENEMY_PROPS_INVISIBLE as i32) as u16;
    session_slot!(game, Eye[EYE].base.properties = hidden);
    session_slot!(game, Tentacles[TENTACLES].base.properties = hidden);
    session_slot!(game, Mouth[MOUTH].base.properties = hidden);
    game.queue_sound(1, DYING_CRY, 6);
}

/// Brightens the screen in steps while the wave grows, then fades out. Was
/// `Phantoon_DyingFadeOut`.
pub fn dying_fade_out<S: BossState + ?Sized>(game: &mut S, k: u16) {
    phantoon_actions::grow_or_shrink_wave_amplitude(DYING_WAVE, game);
    if session_slot!(game, Eye[EYE].movement_reversed) == 0xffff {
        fade_out(game, FADE_STEPS);
        if session_slot!(game, Eye[EYE].fade_complete) != 0 {
            session_slot!(game, Body[k as i32].function = function::ALMOST_DEAD);
        }
    } else if (game.frame() & 0xf) == 0 {
        if session_slot!(byte game, Eye[EYE].movement_reversed[0]) == 0xf2 {
            session_slot!(game, Eye[EYE]; movement_reversed = -1i32 as u16, fade_complete = 0);
        } else {
            let brightness: u8 =
                (session_slot!(byte game, Eye[EYE].movement_reversed[0])).wrapping_add(16);
            session_slot!(byte game, Eye[EYE].movement_reversed[0] = brightness);
            game.set_slot_byte(MOSAIC, brightness);
        }
    }
}

/// Clears BG2 and queues its blank tilemap. Was `Phantoon_AlmostDead`.
pub fn almost_dead<S: BossState + ?Sized>(game: &mut S, k: u16) {
    game.set_slot_byte(MOSAIC, 0);
    session_slot!(game, Eye[EYE].background_held = 0);
    word!(game.PHANTOM_RELATED_LAYER_FLAG &= !HIDE_BG2);
    session_slot!(game, Mouth[MOUTH].semi_transparency = -1i32 as u16);
    session_slot!(game, Body[k as i32]; function = function::DEAD, timer = 60);
    session_slot!(game, Eye[EYE].fade_complete = 0);
    session_slot!(game, Body[BODY].base; x_pos = 384, y_pos = 128);
    crate::word_transfers!(game, descending, 512; le(TILEMAP_STUFF, 511) <- value(BLANK_BG2_TILE));
    // A VRAM write queue entry: 400h bytes from $7E:2000 to VRAM $4800.
    let queue = 0xd0;
    let mut tail: u16 = word!(game.VRAM_WRITE_QUEUE_TAIL);
    set_native_word(
        game,
        queue + word!(game.VRAM_WRITE_QUEUE_TAIL) as isize,
        1024,
    );
    tail = tail.wrapping_add(2);
    set_native_word(game, queue + tail as isize, 0x2000);
    tail = tail.wrapping_add(2);
    let bank_at = tail;
    tail = tail.wrapping_add(1);
    let at = usize::try_from(queue + bank_at as isize).expect("VRAM entry outside RAM");
    game.set_slot_byte(at, 126);
    set_native_word(game, queue + tail as isize, 18432);
    word!(game.VRAM_WRITE_QUEUE_TAIL = tail.wrapping_add(2));
}

/// Fades the room to its powered-on colours, then dies: the drop, the boss
/// flag, the door PLM and the music. Was `Phantoon_Dead`.
pub fn dead<L: Lease + ?Sized>(game: &mut L, k: u16) {
    let me = k as i32;
    if session_slot!(game, Body[me].timer) != 0 {
        session_slot!(game, Body[me].timer, |old| old.wrapping_sub(1));
    } else if (game.frame() & 3) == 0 {
        session_slot!(game, Eye[EYE].fade_steps = 12);
        if fade_step_toward_powered_on(game) & 1 != 0 {
            let value = game.slot_byte(MAIN_SCREEN_LAYERS) | BG2_ENABLED;
            game.set_slot_byte(MAIN_SCREEN_LAYERS, value);
            game.foreign(|game| game.scatter_enemy_drop(DropSource::Phantoon, k));
            let deleted: u16 =
                (session_slot!(game, Body[BODY].base.properties)) | (K_ENEMY_PROPS_DELETED as u16);
            session_slot!(game, Body[BODY].base.properties = deleted);
            session_slot!(game, Eye[EYE].base.properties = deleted);
            session_slot!(game, Tentacles[TENTACLES].base.properties = deleted);
            session_slot!(game, Mouth[MOUTH].base.properties = deleted);
            game.foreign(|game| {
                let area = game.area();
                let bits = game.boss_word(area);
                game.set_boss_word(area, bits | 1);
                game.spawn_plm(0, 0x6, DEFEAT_PLM);
                game.queue_music_delayed(3);
            });
        }
    }
}

/// Looks at Samus and starts the swoop. Was `Phantoon_BecomesSolidAndBodyVuln`.
pub fn become_solid_and_swoop<S: BossState + ?Sized>(game: &mut S, k: u16) {
    look_at_samus(game);
    word!(game.PHANTOM_RELATED_LAYER_FLAG &= !HIDE_BG2);
    begin_swoop(game, k);
    session_slot!(game, Body[BODY].base; instruction_timer = 1, current_instruction = animation::BODY_FULL_HITBOX);
}

/// The countdown functions: each runs its step when the body's timer runs
/// out. Was the `@countdown` group of `DispatchEnemyPreInstr`.
pub fn countdown<L: Lease + ?Sized>(game: &mut L, k: u16, function: u16) {
    if !timer_expired(game, k) {
        return;
    }
    match function {
        function::SPAWN_STARTING_FLAMES => spawn_starting_flame(game, k),
        function::WAIT_BEFORE_SPINNING_FLAMES => {
            session_slot!(game, Body[k as i32]; timer = 240, speed_subpixel = 1);
            session_slot!(
                game,
                Body[k as i32].function = function::SPAWN_FLAMES_BEFORE_FIGHT
            );
        }
        function::SPAWN_FLAMES_BEFORE_FIGHT => start_fight(game, k),
        function::PICK_ROUND_1_PATTERN => pick_first_pattern(game, k),
        function::WAIT_AFTER_FADE_OUT => {
            session_slot!(game, Body[k as i32].function = function::REAPPEAR);
        }
        function::FOLLOW_SAMUS_DURING_FLAME_RAIN => end_flame_rain_window(game, k),
        function::SPAWN_FLAME_RAIN => spawn_flame_rain(game, k),
        function::MOVE_TO_TOP_CENTRE => {
            session_slot!(game, Body[k as i32].function = function::FADE_IN_ENRAGED);
            session_slot!(game, Body[BODY].base; x_pos = 128, y_pos = 32);
            session_slot!(game, Eye[EYE].fade_complete = 0);
        }
        function::DYING_EXPLOSIONS => death_explosion(game, k),
        _ => game.foreign(|game| game.unreachable()),
    }
}

/// One of the eight opening flames. Was `Phantoon_Spawn8FireballsInCircleAtStart`.
fn spawn_starting_flame<L: Lease + ?Sized>(game: &mut L, k: u16) {
    game.spawn_actor_eproj(
        session_slot!(game, Body[BODY].movement_index),
        k,
        STARTING_FLAMES,
    );
    game.queue_sound(2, STARTING_FLAME_SOUND, 6);
    session_slot!(game, Body[k as i32].timer = 30);
    let flames: u16 = (session_slot!(game, Body[BODY].movement_index)).wrapping_add(1);
    session_slot!(game, Body[BODY].movement_index = flames);
    if !below(flames, 8) {
        session_slot!(game, Body[BODY].movement_index = 0);
        session_slot!(game, Tentacles[k as i32 + TENTACLES].damage_taken = 0);
        session_slot!(
            game,
            Body[k as i32].function = function::WAIT_BEFORE_SPINNING_FLAMES
        );
        session_slot!(game, Body[k as i32].timer = 30);
        game.foreign(|game| game.spawn_plm(0, 0x6, FIGHT_START_PLM));
    }
}

/// The opening flames fly off and the wavy fade-in starts the fight. Was
/// `Phantoon_SpawnFireballsBeforeFight`.
fn start_fight<L: Lease + ?Sized>(game: &mut L, k: u16) {
    session_slot!(game, Body[k as i32].speed_subpixel = 0);
    word!(game.PHANTOM_RELATED_LAYER_FLAG |= HIDE_BG2);
    session_slot!(game, Body[k as i32].function = function::WAVY_FADE_IN);
    session_slot!(game, Mouth[MOUTH].semi_transparency = -32767i32 as u16);
    session_slot!(game, Body[k as i32].timer = 120);
    game.foreign(|game| game.start_wave(2, WAVE_PHASE_DELTA));
    session_slot!(game, Mouth[MOUTH].wave_amplitude = INTRO_MAX_AMPLITUDE);
    session_slot!(game, Eye[EYE].fade_complete = 0);
    game.foreign(|game| game.queue_music_delayed(5));
}

/// Picks the first figure-8 direction. Was `Phantoon_PickPatternForRound1`.
fn pick_first_pattern<S: BossState + ?Sized>(game: &mut S, k: u16) {
    session_slot!(game, Eye[EYE].background_held = 0);
    session_slot!(game, Body[k as i32].function = function::FIGURE_EIGHT);
    session_slot!(
        game,
        Eye[EYE].timer = phantoon_tables::timer(
            game,
            Timer::EyeClosed,
            (game.frame() as i32 >> 1 & 3) as usize
        )
    );
    if (game.roll_random() & 1) != 0 {
        session_slot!(game, Body[BODY]; speed = 0, speed_subpixel = 0, movement_stage = 0);
        session_slot!(game, Eye[EYE].movement_reversed = 1);
        session_slot!(game, Body[BODY].movement_index = 533);
    } else {
        session_slot!(game, Body[BODY].speed = 1);
        session_slot!(game, Body[BODY]; speed_subpixel = 0, movement_stage = 0);
        session_slot!(game, Eye[EYE].movement_reversed = 0);
        session_slot!(game, Body[BODY].movement_index = 0);
    }
}

/// The flame rain's vulnerable window ends: a swoop if struck, else hide.
/// Was `Phantoon_FollowSamusWithEyeDuringFireballRain`.
fn end_flame_rain_window<S: BossState + ?Sized>(game: &mut S, k: u16) {
    session_slot!(game, Tentacles[k as i32 + TENTACLES].damage_taken = 0);
    if session_slot!(game, Tentacles[TENTACLES].swooping) != 0 {
        session_slot!(game, Tentacles[TENTACLES].swooping = 0);
        session_slot!(game, Body[BODY].flame_rain_next = 1);
        begin_swoop(game, k);
    } else {
        session_slot!(
            game,
            Body[k as i32].function = function::FADE_OUT_DURING_FLAME_RAIN
        );
        session_slot!(game, Eye[EYE].fade_complete = 0);
        close_eye(game, animation::EYE_CLOSE);
        word!(game.PHANTOM_RELATED_LAYER_FLAG |= HIDE_BG2);
    }
}

/// Reappears at a random flame rain position and rains flames. Was
/// `Phantoon_SpawnRainingFireballs`.
fn spawn_flame_rain<S: BossState + ?Sized>(game: &mut S, k: u16) {
    let pattern: u16 = game.roll_random() & 7;
    let row: i32 = (8 * pattern as i32) >> 1;
    let position = phantoon_tables::flame_rain(game, (row >> 2) as usize);
    session_slot!(
        game,
        Body[BODY].movement_index = position.figure_eight_index
    );
    session_slot!(game, Body[BODY].base; x_pos = position.x_position, y_pos = position.y_position);
    session_slot!(game, Eye[EYE].movement_reversed = 0);
    session_slot!(
        game,
        Body[k as i32].function = function::BECOME_SOLID_AFTER_FLAME_RAIN
    );
    phantoon_actions::spawn_flame_rain(k, pattern, game);
}

/// One death explosion; three rounds of the last eight, then the cry. Was
/// `Phantoon_DyingPhantoonExplosions`.
fn death_explosion<S: BossState + ?Sized>(game: &mut S, k: u16) {
    let offset: u16 = 4u16.wrapping_mul(session_slot!(game, Tentacles[TENTACLES].explosion));
    let explosion = |game: &mut S, at: u16| {
        game.native_image().read(
            sm_data::cartridge::lorom_offset(DEATH_EXPLOSIONS) as usize
                + usize::from(offset)
                + usize::from(at),
        )
    };
    let x = ((explosion(game, 0) as i8) as u16)
        .wrapping_add(session_slot!(game, Body[BODY].base.x_pos));
    let y = ((explosion(game, 1) as i8) as u16)
        .wrapping_add(session_slot!(game, Body[BODY].base.y_pos));
    game.place_projectile_spawn(x, y);
    let kind: u16 = explosion(game, 2) as u16;
    game.spawn_room_eproj(DUST_CLOUD, kind);
    if kind == 29 {
        game.queue_sound(1, BIG_EXPLOSION_SOUND, 6);
    } else {
        game.queue_sound(1, SMALL_EXPLOSION_SOUND, 6);
    }
    session_slot!(game, Body[k as i32].timer = explosion(game, 3) as u16);
    let next: u16 = (session_slot!(game, Tentacles[TENTACLES].explosion)).wrapping_add(1);
    session_slot!(game, Tentacles[TENTACLES].explosion = next);
    if !below(next, 13) {
        session_slot!(game, Tentacles[TENTACLES].explosion = 5);
        let rounds: u16 = (session_slot!(game, Body[BODY].movement_index)).wrapping_add(1);
        session_slot!(game, Body[BODY].movement_index = rounds);
        if !below(rounds, 3) {
            session_slot!(game, Body[k as i32].function = function::WAVY_DYING_CRY);
        }
    }
}
