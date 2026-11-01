//! Phantoon, the Wrecked Ship's ghost: its fight as a state machine over the
//! body's function word, the swoop, the palette fades and the death sequence.
//!
//! Was `Phantoon_*` in the original decompilation (bank $A7:CDF3-$A7:DD9B).
//! Phantoon is four enemies: the body in slot 0, then the eye, the tentacles
//! and the mouth. Canonical parts use Actor fields; exceptional identities
//! retain the original queried alias. The figure-8 flight and flames stay in
//! `phantoon_motion` and `phantoon_actions`.
use crate::actors::api::PlayerDirection;
use crate::actors::boss_lease::{BossLease, BossState};
use crate::actors::deaths::Deaths as _;
use crate::actors::host::BossEffects as _;
use crate::actors::host::{Death, Shot, below};

use crate::actors::{phantoon_actions, phantoon_motion};
use crate::constants::enemy_properties::{
    K_ENEMY_PROPS_DELETED, K_ENEMY_PROPS_DISABLE_SAMUS_COLL, K_ENEMY_PROPS_INVISIBLE,
    K_ENEMY_PROPS_TANGIBLE,
};
use crate::display::hdma_spawn::HdmaObjectDefinition;
use crate::display::transfer::{Source, TilemapUpload, queue_tilemap};
use crate::enemy_projectiles::drops::DropSource;

use crate::phantoon_tables::{self, Timer};

/// Phantoon's already-owned state, plus the retained effect capabilities.
pub trait State: BossState + crate::session_access::SessionAccess {
    /// The next owned random value, admitted at the original query seam.
    fn phantoon_random(&mut self) -> u16;
}
pub trait Lease:
    State
    + crate::progress::FlagHost
    + BossLease<Game: crate::actors::host::BossEffects + crate::actors::deaths::Deaths>
{
    fn run_foreign_function(&mut self, address: u32);
    fn spawn_hdma(&mut self, bank: u8, args: &HdmaObjectDefinition);
    fn start_wave(&mut self, direction: u16, speed: u16);
    fn disable_minimap(&mut self);
    fn owned<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R;
}
/// Compatibility dispatch only; state transitions themselves require a Session.
pub trait Dispatch {
    fn phantoon(&mut self, callback: Callback);
}
#[derive(Clone, Copy)]
pub enum Callback {
    Initialize,
    InitializePart,
    Main,
    Hurt,
    Touch,
    Shot,
    PreInstruction { function: u16, actor: u16 },
    Instruction { address: u32, actor: u16 },
}
pub fn callback<L: Lease + ?Sized>(game: &mut L, callback: Callback) {
    match callback {
        Callback::Initialize => initialize(game),
        Callback::InitializePart => initialize_part(game),
        Callback::Main => main_ai(game),
        Callback::Hurt => hurt(game),
        Callback::Touch => touch(game),
        Callback::Shot => shot(game),
        Callback::PreInstruction { function, actor } => run_function_at(game, function, actor),
        Callback::Instruction { address, actor } => match address {
            10997485 => materialisation_sound(game),
            10997598 => phantoon_actions::spawn_casual_flame(actor, game),
            10997823 => open_eye(game),
            10997878 => pick_pattern(game),
            _ => game.foreign(|game| game.unreachable()),
        },
    }
}

/// Canonical parts use Actor fields. Exceptional identities retain a single
/// queried word/byte through the compatibility projection, without predecoding.
macro_rules! fields {
    ($($name:ident => $at:literal, $head:ident $(.$tail:ident)* $([$index:literal])?;)+) => {
        #[derive(Clone, Copy)]
        pub(crate) enum Field { $($name,)+ }
        impl Field {
            fn offset(self) -> isize { match self { $(Self::$name => $at,)+ } }
            fn get(self, actor: &crate::actors::state::Actor) -> u16 {
                match self { $(Self::$name => actor.$head $(.$tail)* $([$index])?,)+ }
            }
            fn set(self, actor: &mut crate::actors::state::Actor, value: u16) {
                match self { $(Self::$name => actor.$head $(.$tail)* $([$index])? = value,)+ }
            }
        }
    };
}
fields! {
    BaseXPos => 2, x.whole;
    BaseXSubpos => 4, x.fraction;
    BaseYPos => 6, y.whole;
    BaseYSubpos => 8, y.fraction;
    BaseProperties => 14, properties;
    BaseAiHandlerBits => 18, ai_handlers;
    BaseHealth => 20, health;
    BaseSpritemapPointer => 22, spritemap;
    BaseTimer => 24, timer;
    BaseCurrentInstruction => 26, instruction;
    BaseInstructionTimer => 28, instruction_delay;
    BasePaletteIndex => 30, palette;
    BaseVramTilesIndex => 32, tile;
    BaseFlashTimer => 36, flash;
    BaseFrameCounter => 44, frame;
    BodyMovementIndex => 48, variables[0];
    BodySpeedSubpixel => 50, variables[1];
    BodySpeed => 52, variables[2];
    BodyMovementStage => 54, variables[3];
    BodyTimer => 56, variables[4];
    BodyFunction => 58, pre_instruction;
    BodyFlameRainNext => 62, parameters[1];
    EyeTimer => 48, variables[0];
    EyeUnreadShotValue => 50, variables[1];
    EyeMovementReversed => 52, variables[2];
    EyeFadeSteps => 54, variables[3];
    EyeFadeStep => 56, variables[4];
    EyeFadeComplete => 58, pre_instruction;
    EyeBackgroundHeld => 60, parameters[0];
    EyeParameter2 => 62, parameters[1];
    TentaclesSwooping => 48, variables[0];
    TentaclesDamageTaken => 50, variables[1];
    TentaclesSwoopXSpeed => 52, variables[2];
    TentaclesSwoopYSpeed => 54, variables[3];
    TentaclesSwoopTargetX => 56, variables[4];
    TentaclesExplosion => 58, pre_instruction;
    TentaclesHurtFlash => 62, parameters[1];
    MouthCasualFlamePattern => 48, variables[0];
    MouthCasualFlameTimer => 50, variables[1];
    MouthCasualFlamePatternIndex => 52, variables[2];
    MouthWaveAmplitude => 54, variables[3];
    MouthWaveShrinking => 56, variables[4];
    MouthSemiTransparency => 60, parameters[0];
    MouthShotRoll => 62, parameters[1];
}
fn part_index(slot: i32) -> Option<usize> {
    let slot = usize::try_from(slot).ok()?;
    (slot < 2048 && slot.is_multiple_of(64) && cfg!(target_endian = "little")).then_some(slot / 64)
}
fn part_address(slot: i32, field: Field) -> usize {
    usize::try_from(slot as isize + 0xf78 + field.offset()).expect("enemy field outside RAM")
}
pub(crate) fn part_get<S: State + ?Sized>(game: &S, slot: i32, field: Field) -> u16 {
    match part_index(slot) {
        Some(index) => field.get(&game.session().actors.slots[index]),
        None => game.slot_word(part_address(slot, field)),
    }
}
pub(crate) fn part_set<S: State + ?Sized>(game: &mut S, slot: i32, field: Field, value: u16) {
    match part_index(slot) {
        Some(index) => field.set(&mut game.session_mut().actors.slots[index], value),
        None => game.set_slot_word(part_address(slot, field), value),
    }
}
pub(crate) fn part_byte<S: State + ?Sized>(game: &S, slot: i32, field: Field, lane: usize) -> u8 {
    match part_index(slot).filter(|_| lane < 2) {
        Some(index) => field.get(&game.session().actors.slots[index]).to_le_bytes()[lane],
        None => game.slot_byte(part_address(slot, field) + lane),
    }
}
pub(crate) fn set_part_byte<S: State + ?Sized>(
    game: &mut S,
    slot: i32,
    field: Field,
    lane: usize,
    value: u8,
) {
    match part_index(slot).filter(|_| lane < 2) {
        Some(index) => {
            let actor = &mut game.session_mut().actors.slots[index];
            let mut bytes = field.get(actor).to_le_bytes();
            bytes[lane] = value;
            field.set(actor, u16::from_le_bytes(bytes));
        }
        None => game.set_slot_byte(part_address(slot, field) + lane, value),
    }
}
macro_rules! part {
    (byte $g:ident, $layout:ident[$slot:expr] $(.$field:ident)+ [$lane:expr] = $value:expr) => {{
        let value: u8 = $value;
        $crate::actors::phantoon::set_part_byte($g, $slot, part!(@field $layout, $($field).+), $lane as usize, value)
    }};
    (byte $g:ident, $layout:ident[$slot:expr] $(.$field:ident)+ [$lane:expr]) => {
        $crate::actors::phantoon::part_byte(&*$g, $slot, part!(@field $layout, $($field).+), $lane as usize)
    };
    ($g:ident, $layout:ident[$slot:expr] $(.$prefix:ident)* ($hi:ident, $lo:ident) += $value:expr) => {{
        let value: u32 = $value;
        let low = u32::from(part!($g, $layout[$slot] $(.$prefix)* .$lo));
        let high = u32::from(part!($g, $layout[$slot] $(.$prefix)* .$hi));
        let sum = (low | (high << 16)).wrapping_add(value);
        part!($g, $layout[$slot] $(.$prefix)* .$lo = sum as u16);
        part!($g, $layout[$slot] $(.$prefix)* .$hi = (sum >> 16) as u16);
    }};
    ($g:ident, $layout:ident[$slot:expr] $(.$prefix:ident)*; $($field:ident = $value:expr),+ $(,)?) => {
        $crate::actors::phantoon::part!(@set_fields $g, $layout[$slot] [$(.$prefix)*]; $($field = $value),+);
    };
    (@set_fields $g:ident, $layout:ident[$slot:expr] [$($prefix:tt)*]; $field:ident = $value:expr, $($rest:tt)+) => {
        $crate::actors::phantoon::part!($g, $layout[$slot] $($prefix)* .$field = $value);
        $crate::actors::phantoon::part!(@set_fields $g, $layout[$slot] [$($prefix)*]; $($rest)+);
    };
    (@set_fields $g:ident, $layout:ident[$slot:expr] [$($prefix:tt)*]; $field:ident = $value:expr) => {
        $crate::actors::phantoon::part!($g, $layout[$slot] $($prefix)* .$field = $value);
    };
    ($g:ident, $layout:ident[$slot:expr] $(.$field:ident)+, |$old:ident| $value:expr) => {{
        let slot: i32 = $slot;
        let field = part!(@field $layout, $($field).+);
        let $old = $crate::actors::phantoon::part_get(&*$g, slot, field);
        let value: u16 = $value;
        $crate::actors::phantoon::part_set($g, slot, field, value)
    }};
    ($g:ident, $layout:ident[$slot:expr] $(.$field:ident)+ = $value:expr) => {{
        let value: u16 = $value;
        $crate::actors::phantoon::part_set($g, $slot, part!(@field $layout, $($field).+), value)
    }};
    ($g:ident, $layout:ident[$slot:expr] $(.$field:ident)+) => {
        $crate::actors::phantoon::part_get(&*$g, $slot, part!(@field $layout, $($field).+))
    };
    (@field $($tokens:tt)*) => { $crate::actors::phantoon::part_field!($($tokens)*) };
}
macro_rules! part_field {
    ($layout:ident, base.x_pos) => {
        $crate::actors::phantoon::Field::BaseXPos
    };
    ($layout:ident, base.x_subpos) => {
        $crate::actors::phantoon::Field::BaseXSubpos
    };
    ($layout:ident, base.y_pos) => {
        $crate::actors::phantoon::Field::BaseYPos
    };
    ($layout:ident, base.y_subpos) => {
        $crate::actors::phantoon::Field::BaseYSubpos
    };
    ($layout:ident, base.properties) => {
        $crate::actors::phantoon::Field::BaseProperties
    };
    ($layout:ident, base.ai_handler_bits) => {
        $crate::actors::phantoon::Field::BaseAiHandlerBits
    };
    ($layout:ident, base.health) => {
        $crate::actors::phantoon::Field::BaseHealth
    };
    ($layout:ident, base.spritemap_pointer) => {
        $crate::actors::phantoon::Field::BaseSpritemapPointer
    };
    ($layout:ident, base.timer) => {
        $crate::actors::phantoon::Field::BaseTimer
    };
    ($layout:ident, base.current_instruction) => {
        $crate::actors::phantoon::Field::BaseCurrentInstruction
    };
    ($layout:ident, base.instruction_timer) => {
        $crate::actors::phantoon::Field::BaseInstructionTimer
    };
    ($layout:ident, base.palette_index) => {
        $crate::actors::phantoon::Field::BasePaletteIndex
    };
    ($layout:ident, base.vram_tiles_index) => {
        $crate::actors::phantoon::Field::BaseVramTilesIndex
    };
    ($layout:ident, base.flash_timer) => {
        $crate::actors::phantoon::Field::BaseFlashTimer
    };
    ($layout:ident, base.frame_counter) => {
        $crate::actors::phantoon::Field::BaseFrameCounter
    };
    (Body, movement_index) => {
        $crate::actors::phantoon::Field::BodyMovementIndex
    };
    (Body, speed_subpixel) => {
        $crate::actors::phantoon::Field::BodySpeedSubpixel
    };
    (Body, speed) => {
        $crate::actors::phantoon::Field::BodySpeed
    };
    (Body, movement_stage) => {
        $crate::actors::phantoon::Field::BodyMovementStage
    };
    (Body, timer) => {
        $crate::actors::phantoon::Field::BodyTimer
    };
    (Body, function) => {
        $crate::actors::phantoon::Field::BodyFunction
    };
    (Body, flame_rain_next) => {
        $crate::actors::phantoon::Field::BodyFlameRainNext
    };
    (Eye, timer) => {
        $crate::actors::phantoon::Field::EyeTimer
    };
    (Eye, unread_shot_value) => {
        $crate::actors::phantoon::Field::EyeUnreadShotValue
    };
    (Eye, movement_reversed) => {
        $crate::actors::phantoon::Field::EyeMovementReversed
    };
    (Eye, fade_steps) => {
        $crate::actors::phantoon::Field::EyeFadeSteps
    };
    (Eye, fade_step) => {
        $crate::actors::phantoon::Field::EyeFadeStep
    };
    (Eye, fade_complete) => {
        $crate::actors::phantoon::Field::EyeFadeComplete
    };
    (Eye, background_held) => {
        $crate::actors::phantoon::Field::EyeBackgroundHeld
    };
    (Eye, parameter_2) => {
        $crate::actors::phantoon::Field::EyeParameter2
    };
    (Tentacles, swooping) => {
        $crate::actors::phantoon::Field::TentaclesSwooping
    };
    (Tentacles, damage_taken) => {
        $crate::actors::phantoon::Field::TentaclesDamageTaken
    };
    (Tentacles, swoop_x_speed) => {
        $crate::actors::phantoon::Field::TentaclesSwoopXSpeed
    };
    (Tentacles, swoop_y_speed) => {
        $crate::actors::phantoon::Field::TentaclesSwoopYSpeed
    };
    (Tentacles, swoop_target_x) => {
        $crate::actors::phantoon::Field::TentaclesSwoopTargetX
    };
    (Tentacles, explosion) => {
        $crate::actors::phantoon::Field::TentaclesExplosion
    };
    (Tentacles, hurt_flash) => {
        $crate::actors::phantoon::Field::TentaclesHurtFlash
    };
    (Mouth, casual_flame_pattern) => {
        $crate::actors::phantoon::Field::MouthCasualFlamePattern
    };
    (Mouth, casual_flame_timer) => {
        $crate::actors::phantoon::Field::MouthCasualFlameTimer
    };
    (Mouth, casual_flame_pattern_index) => {
        $crate::actors::phantoon::Field::MouthCasualFlamePatternIndex
    };
    (Mouth, wave_amplitude) => {
        $crate::actors::phantoon::Field::MouthWaveAmplitude
    };
    (Mouth, wave_shrinking) => {
        $crate::actors::phantoon::Field::MouthWaveShrinking
    };
    (Mouth, semi_transparency) => {
        $crate::actors::phantoon::Field::MouthSemiTransparency
    };
    (Mouth, shot_roll) => {
        $crate::actors::phantoon::Field::MouthShotRoll
    };
}
pub(crate) use part;
pub(crate) use part_field;

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
const WHITE: u16 = 0x7fff;
/// The bank $84 PLMs spawned when the fight starts and when Phantoon dies.
const FIGHT_START_PLM: u16 = 0xb781;
const DEFEAT_PLM: u16 = 0xb78b;

fn projectile_kind<S: State + ?Sized>(game: &S, index: u16) -> u16 {
    match game.session().projectiles.shots.get(usize::from(index)) {
        Some(shot) => shot.kind,
        None => {
            let at = 0xc18 + 2 * usize::from(index);
            u16::from_le_bytes([game.slot_byte(at), game.slot_byte(at + 1)])
        }
    }
}
fn bg2_scroll<S: State + ?Sized>(game: &S, axis: usize) -> u16 {
    game.scroll_word(crate::display_scroll::WORDS[2 + axis])
}
fn set_bg2_scroll<S: State + ?Sized>(game: &mut S, axis: usize, value: u16) {
    game.set_scroll_word(crate::display_scroll::WORDS[2 + axis], value);
}
fn set_mosaic<S: State + ?Sized>(game: &mut S, value: u8) {
    match game.session_mut().presentation.owner_mut() {
        Ok(display) => display.state.controls.mosaic = value,
        Err(_) => game.set_slot_byte(MOSAIC, value),
    }
}
fn enable_bg2<S: State + ?Sized>(game: &mut S) {
    match game.session_mut().presentation.owner_mut() {
        Ok(display) => display.state.controls.main_layers |= BG2_ENABLED,
        Err(_) => {
            let value = game.slot_byte(MAIN_SCREEN_LAYERS) | BG2_ENABLED;
            game.set_slot_byte(MAIN_SCREEN_LAYERS, value);
        }
    }
}

pub(crate) fn frame<S: State + ?Sized>(game: &S) -> u16 {
    match game.session().presentation.owner() {
        Ok(display) => display.state.nmi.frame_word,
        Err(_) => u16::from_le_bytes([game.slot_byte(0x5b6), game.slot_byte(0x5b7)]),
    }
}

fn current<S: State + ?Sized>(game: &S) -> i32 {
    game.current_enemy() as i32
}

fn native_word<S: State + ?Sized>(game: &S, at: isize) -> u16 {
    let at = usize::try_from(at).expect("native word outside RAM");
    if (0xc000..0xc200).contains(&at) && at.is_multiple_of(2) {
        return u16::from_ne_bytes(
            game.session().presentation.visual().colors[(at - 0xc000) / 2].to_le_bytes(),
        );
    }
    u16::from_ne_bytes([game.slot_byte(at), game.slot_byte(at + 1)])
}

fn set_native_word<S: State + ?Sized>(game: &mut S, at: isize, value: u16) {
    let at = usize::try_from(at).expect("native word outside RAM");
    if (0xc000..0xc200).contains(&at) && at.is_multiple_of(2) {
        game.session_mut().presentation.visual_mut().colors[(at - 0xc000) / 2] =
            u16::from_le_bytes(value.to_ne_bytes());
        return;
    }
    if (0xc200..0xc400).contains(&at) && at.is_multiple_of(2) {
        game.session_mut().presentation.visual_mut().targets[(at - 0xc200) / 2] =
            u16::from_le_bytes(value.to_ne_bytes());
        return;
    }
    if (0x97dc..0x9800).contains(&at) && at.is_multiple_of(2) {
        game.session_mut().enemy_projectiles.slots[(at - 0x97dc) / 2].angle =
            u16::from_le_bytes(value.to_ne_bytes());
        return;
    }
    if (0x9000..0x97dc).contains(&at) && at.is_multiple_of(2) {
        game.session_mut().actors.extended.blocks[3][(at - 0x9000) / 2] =
            u16::from_le_bytes(value.to_ne_bytes());
        return;
    }
    let bytes = u16::to_ne_bytes(value);
    game.set_slot_byte(at, bytes[0]);
    game.set_slot_byte(at + 1, bytes[1]);
}

/// Palette RAM word `index` of the colours Phantoon fades from.
fn palette_source(index: i32) -> isize {
    SOURCE_PALETTES + (index as isize) * 2
}

/// Counts down the function timer of part `k`; true when it runs out.
fn timer_expired<S: State + ?Sized>(game: &mut S, k: u16) -> bool {
    let me = k as i32;
    let last = part!(game, Body[me].timer) == 1;
    part!(game, Body[me].timer, |old| old.wrapping_sub(1));
    let negative = (part!(game, Body[me].timer) & 0x8000) != 0;
    last || negative
}

/// Counts down the eye's timer; true when it runs out.
fn eye_timer_expired<S: State + ?Sized>(game: &mut S) -> bool {
    let last = part!(game, Eye[EYE].timer) == 1;
    part!(game, Eye[EYE].timer, |old| old.wrapping_sub(1));
    let negative = (part!(game, Eye[EYE].timer) & 0x8000) != 0;
    last || negative
}

/// Closes the eye, makes the body invulnerable and hides BG2's background.
fn close_eye<S: State + ?Sized>(game: &mut S, eye_list: u16) {
    part!(game, Body[BODY].base.instruction_timer = 1);
    part!(game, Eye[EYE].base.instruction_timer = 1);
    part!(
        game,
        Body[BODY].base.current_instruction = animation::BODY_INVULNERABLE
    );
    part!(game, Eye[EYE].base.current_instruction = eye_list);
    part!(game, Body[BODY].base.properties, |old| old
        | (K_ENEMY_PROPS_TANGIBLE as u16));
}

/// Instruction: the materialisation sound, cycling through three. Was
/// `Phantoon_Func_1`.
pub fn materialisation_sound<S: State + ?Sized>(game: &mut S) {
    let sound = crate::boss_lists::phantoon_materialisation_sound(
        game,
        game.session().actors.extended.blocks[3][25] as isize,
    );
    game.queue_sound(1, sound as u8, 6);
    let mut next: u16 = (game.session().actors.extended.blocks[3][25]).wrapping_add(1);
    if !below(game.session().actors.extended.blocks[3][25], 2) {
        next = 0;
    }
    game.session_mut().actors.extended.blocks[3][25] = next;
}

/// Instruction: opens the eye for the figure-8's vulnerable window. Was
/// `Phantoon_StartTrackingSamusAndInitEyeTimer`.
pub fn open_eye<S: State + ?Sized>(game: &mut S) {
    part!(game, Tentacles[TENTACLES].swooping = 0);
    part!(game, Body[BODY].base; instruction_timer = 1, current_instruction = animation::BODY_EYE_HITBOX);
    part!(
        game,
        Body[BODY].base.properties,
        |old| (old as i32 & !(K_ENEMY_PROPS_TANGIBLE as i32)) as u16
    );
    let roll = (game.phantoon_random() as i32 & 7) as usize;
    part!(
        game,
        Body[BODY].timer = phantoon_tables::timer(game, Timer::FigureEightVulnerable, roll)
    );
    part!(game, Body[BODY].function = function::EYE_FOLLOWS_SAMUS);
    part!(game, Eye[EYE].base; instruction_timer = 1, current_instruction = animation::EYEBALL_CENTRED);
}

/// Instruction: picks the next figure-8 direction and whether flame rain
/// comes first. Was `Phantoon_PickPatternForRound2`.
pub fn pick_pattern<S: State + ?Sized>(game: &mut S) {
    crate::probe::hit("phantoon.pick_pattern");
    part!(game, Body[BODY].timer = 60);
    let roll = (game.phantoon_random() as i32 & 7) as usize;
    let closed: u16 = phantoon_tables::timer(game, Timer::EyeClosed, roll);
    part!(game, Eye[EYE].timer = closed);
    if (frame(game) & 1) != 0 {
        if part!(game, Eye[EYE].movement_reversed) == 0 {
            let index: i16 = ((part!(game, Body[BODY].movement_index)) as i16).wrapping_sub(1);
            part!(game, Body[BODY].movement_index = index as u16);
            if index < 0 {
                part!(game, Body[BODY].movement_index = 533);
            }
        }
        part!(game, Body[BODY]; speed = 0, speed_subpixel = 0, movement_stage = 0);
        part!(game, Eye[EYE].movement_reversed = 1);
    } else {
        if part!(game, Eye[EYE].movement_reversed) != 0 {
            let index: u16 = (part!(game, Body[BODY].movement_index)).wrapping_add(1);
            part!(game, Body[BODY].movement_index = index);
            if !below(index, 534) {
                part!(game, Body[BODY].movement_index = 0);
            }
        }
        part!(game, Body[BODY].speed = 1);
        part!(game, Body[BODY]; speed_subpixel = 0, movement_stage = 0);
        part!(game, Eye[EYE].movement_reversed = 0);
    }
    if part!(game, Body[BODY].flame_rain_next) != 0 {
        part!(game, Eye[EYE].fade_complete = 0);
        part!(
            game,
            Body[BODY].function = function::FADE_OUT_BEFORE_FIRST_FLAME_RAIN
        );
    } else {
        part!(game, Body[BODY].function = function::FIGURE_EIGHT);
    }
}

/// Swoops toward Samus: X oscillates across the room, Y follows Samus (or
/// the floor once fatally shot). Was `Phantoon_MoveInSwoopingPattern`.
fn swoop<S: State + ?Sized>(game: &mut S, k: u16) {
    let mut target_x: u16;
    let previous: i16 = part!(game, Tentacles[TENTACLES].swoop_target_x) as i16;
    if previous < 0 {
        let next: u16 = (previous as u16).wrapping_sub(2);
        part!(game, Tentacles[TENTACLES].swoop_target_x = next);
        target_x = next & 0x7fff;
        if target_x == 0 {
            target_x = 0;
            part!(game, Tentacles[TENTACLES].swoop_target_x = 0);
        }
    } else {
        target_x = (previous as u16).wrapping_add(2);
        part!(game, Tentacles[TENTACLES].swoop_target_x = target_x);
        if !below(target_x, 256) {
            let next: u16 = target_x | 0x8000;
            part!(game, Tentacles[TENTACLES].swoop_target_x = next);
            target_x = next & 0x7fff;
        }
    }
    if (target_x as i16).wrapping_sub((part!(game, Body[BODY].base.x_pos)) as i16) < 0 {
        if (part!(game, Tentacles[TENTACLES].swoop_x_speed) as i32 + 2047) & 0x8000 == 0 {
            part!(game, Tentacles[TENTACLES].swoop_x_speed, |old| old
                .wrapping_sub(32));
        }
    } else if below(part!(game, Tentacles[TENTACLES].swoop_x_speed), 2048) {
        part!(game, Tentacles[TENTACLES].swoop_x_speed, |old| old
            .wrapping_add(32));
    }
    part!(
        game,
        Body[BODY].base(x_pos, x_subpos) +=
            ((part!(game, Tentacles[TENTACLES].swoop_x_speed) as i16 as i32) << 8) as u32
    );
    if (part!(game, Body[BODY].base.x_pos) as i32 + 64) & 0x8000 != 0 {
        part!(game, Body[BODY].base.x_pos = -64i32 as u16);
    } else if !below(part!(game, Body[BODY].base.x_pos), 448) {
        part!(game, Body[BODY].base.x_pos = 448);
    }
    let target_y: i16 =
        if part!(game, Body[k as i32].function) == function::FINISH_SWOOP_AFTER_FATAL_SHOT {
            112
        } else {
            ((game.session().player.y) as i16).wrapping_sub(48)
        };
    if target_y.wrapping_sub((part!(game, Body[BODY].base.y_pos)) as i16) < 0 {
        if (part!(game, Tentacles[TENTACLES].swoop_y_speed) as i32 + 1535) & 0x8000 == 0 {
            part!(game, Tentacles[TENTACLES].swoop_y_speed, |old| old
                .wrapping_sub(64));
        }
    } else if below(part!(game, Tentacles[TENTACLES].swoop_y_speed), 1536) {
        part!(game, Tentacles[TENTACLES].swoop_y_speed, |old| old
            .wrapping_add(64));
    }
    part!(
        game,
        Body[BODY].base(y_pos, y_subpos) +=
            ((part!(game, Tentacles[TENTACLES].swoop_y_speed) as i16 as i32) << 8) as u32
    );
    if below(part!(game, Body[BODY].base.y_pos), 64) {
        part!(game, Body[BODY].base.y_pos = 64);
    } else if !below(part!(game, Body[BODY].base.y_pos), 216) {
        part!(game, Body[BODY].base.y_pos = 216);
    }
}

/// Was `Phantoon_BeginSwoopingPattern`.
fn begin_swoop<S: State + ?Sized>(game: &mut S, k: u16) {
    part!(game, Tentacles[TENTACLES]; swoop_x_speed = 1024, swoop_y_speed = 1024);
    part!(game, Tentacles[TENTACLES].swoop_target_x = 0);
    part!(game, Body[k as i32]; function = function::SWOOPING, timer = 360);
}

/// Points the eyeball at Samus. Was `Phantoon_ChangeEyeSpriteBasedOnSamusDist`.
fn look_at_samus<S: State + ?Sized>(game: &mut S) {
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
    part!(game, Eye[EYE].base; instruction_timer = 1, current_instruction = crate::boss_lists::phantoon_instruction_list(game, (row >> 1) as isize));
}

/// Starts dying: finishing the swoop first if swooping. Was
/// `Phantoon_StartDeathSequence`.
fn start_dying<S: State + ?Sized>(game: &mut S, k: u16) {
    let me = k as i32;
    if part!(game, Body[me].function) == function::SWOOPING
        || part!(game, Body[me].function) == function::FADE_OUT_WITH_SWOOP
    {
        part!(
            game,
            Body[me].function = function::FINISH_SWOOP_AFTER_FATAL_SHOT
        );
    } else {
        part!(game, Body[me].function = function::DYING_FADE_IN_OUT);
    }
    part!(game, Eye[EYE]; movement_reversed = 0, fade_complete = 0);
    look_at_samus(game);
    let scroll: u16 = bg2_scroll(game, 0);
    for offset in (0i16..=510).rev().step_by(2) {
        set_native_word(
            game,
            BG2_SCROLL_TABLE + (offset as u16 as i32 as isize),
            scroll,
        );
    }
    game.session_mut().room.fx_layer_flags |= HIDE_BG2;
    part!(game, Mouth[MOUTH].shot_roll = 1);
}

/// Fades the body palette one step toward black, every other frame, until
/// `steps` steps. Was `Phantoon_FadeOut`.
fn fade_out<S: State + ?Sized>(game: &mut S, steps: u16) {
    if (frame(game) & 1) == 0 && part!(game, Eye[EYE].fade_complete) == 0 {
        part!(game, Eye[EYE].fade_steps = steps);
        if fade_step_toward_target(game) & 1 != 0 {
            part!(game, Eye[EYE].fade_complete = 1);
        }
    }
}

/// Fades the body palette one step toward its health colours, every other
/// frame, until `steps` steps. Was `Phantoon_FadeIn`.
fn fade_in<S: State + ?Sized>(game: &mut S, steps: u16) {
    if (frame(game) & 1) == 0 && part!(game, Eye[EYE].fade_complete) == 0 {
        part!(game, Eye[EYE].fade_steps = steps);
        if fade_step_toward_health(game) & 1 != 0 {
            part!(game, Eye[EYE].fade_complete = 1);
        }
    }
}

/// One step of palette 7 toward the fade-out target; 1 once done. Was
/// `Phantoon_Func_8`.
fn fade_step_toward_target<S: State + ?Sized>(game: &mut S) -> u8 {
    if (part!(game, Eye[EYE].fade_steps)).wrapping_add(1) >= part!(game, Eye[EYE].fade_step) {
        for i in (0i32..32).step_by(2) {
            {
                let value = transition_color(
                    part!(game, Eye[EYE].fade_step),
                    native_word(game, palette_source((i >> 1) + 112)),
                    crate::boss_lists::fade_out_target_colour(game, (i >> 1) as isize),
                    game,
                );
                game.session_mut().presentation.visual_mut().colors[((i >> 1) + 112) as usize] =
                    value;
            }
        }
        part!(game, Eye[EYE].fade_step, |old| old.wrapping_add(1));
        0
    } else {
        part!(game, Eye[EYE].fade_step = 0);
        1
    }
}

/// One step of palette 7 toward the health palette; 1 once done. Was
/// `Phantoon_SetColorBasedOnHp`.
fn fade_step_toward_health<S: State + ?Sized>(game: &mut S) -> u8 {
    if (part!(game, Eye[EYE].fade_steps)).wrapping_add(1) >= part!(game, Eye[EYE].fade_step) {
        for base in (0u16..32).step_by(2) {
            let (index, colour) = phantoon_actions::health_palette_colour(base, game);
            let from = native_word(game, palette_source((index as i32 >> 1) + 112));
            let value = transition_color(part!(game, Eye[EYE].fade_step), from, colour, game);
            set_native_word(game, palette_source((base as i32 >> 1) + 112), value);
        }
        part!(game, Eye[EYE].fade_step, |old| old.wrapping_add(1));
        0
    } else {
        part!(game, Eye[EYE].fade_step = 0);
        1
    }
}

/// One step of BG palettes 0-6 toward the Wrecked Ship's powered-on colours;
/// 1 once done. Was `Phantoon_Func_9`.
fn fade_step_toward_powered_on<S: State + ?Sized>(game: &mut S) -> u8 {
    if (part!(game, Eye[EYE].fade_steps)).wrapping_add(1) >= part!(game, Eye[EYE].fade_step) {
        for i in (0i32..224).step_by(2) {
            {
                let value = transition_color(
                    part!(game, Eye[EYE].fade_step),
                    game.session().presentation.visual().colors[(i >> 1) as usize],
                    crate::boss_lists::wrecked_ship_powered_on_colour(game, (i >> 1) as isize),
                    game,
                );
                game.session_mut().presentation.visual_mut().colors[(i >> 1) as usize] = value;
            }
        }
        part!(game, Eye[EYE].fade_step, |old| old.wrapping_add(1));
        0
    } else {
        part!(game, Eye[EYE].fade_step = 0);
        1
    }
}

/// Colour `step` of the fade from BGR555 `from` to `to`. Was
/// `Phantoon_Func_10_CalculateNthTransitionColorFromXtoY`.
fn transition_color<S: State + ?Sized>(step: u16, from: u16, to: u16, game: &S) -> u16 {
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
fn transition_component<S: State + ?Sized>(step: u16, from: u16, to: u16, game: &S) -> u16 {
    if step == 0 {
        return from;
    }
    let previous: u16 = step.wrapping_sub(1);
    if previous == part!(game, Eye[EYE].fade_steps) {
        return to;
    }
    let done: u16 = previous.wrapping_add(1);
    let mut delta: u16 = to.wrapping_sub(from);
    let magnitude: u8 = crate::actors::host::abs16(delta) as u8;
    let divisor = (part!(byte game, Eye[EYE].fade_steps[0]) as i32 - done as i32 + 1) as u8;
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
pub fn initialize<H: Lease + ?Sized>(game: &mut H) {
    for index in (0..=2047).rev() {
        game.session_mut().actors.extended.blocks[4 + index / 1024][index % 1024] = BLANK_BG2_TILE;
    }
    for index in (0..1024).rev() {
        set_native_word(game, 0x9000 + 2 * index, 0);
    }
    for index in (0..16).rev() {
        set_native_word(game, 0xc2e0 + 2 * index, 0);
    }
    game.session_mut().actors.drawing.tilemap_size = BG2_TILEMAP_SIZE;
    game.disable_minimap();
    let me = current(game);
    part!(game, Body[me].timer = 120);
    part!(game, Body[me]; movement_index = 0, speed_subpixel = 0);
    game.session_mut().actors.extended.blocks[3][25] = 0;
    part!(game, Eye[EYE]; background_held = 0, parameter_2 = 0);
    part!(
        game,
        Body[BODY].base.properties =
            (part!(game, Body[BODY].base.properties)) | (K_ENEMY_PROPS_TANGIBLE as u16)
    );
    game.spawn_hdma(0xa7, &SEMI_TRANSPARENCY_HDMA);
    initialize_part(game);
}

/// Sets up one part with the body's graphics and its instruction list. Was
/// `Phantoon2_Init`.
pub fn initialize_part<S: State + ?Sized>(game: &mut S) {
    let me = current(game);
    part!(
        game,
        Body[me].base.spritemap_pointer = animation::NOTHING_SPRITEMAP
    );
    part!(game, Body[me].base; instruction_timer = 1, timer = 0);
    let copied = part!(game, Body[BODY].base.palette_index);
    part!(game, Body[me].base.palette_index = copied);
    let copied = part!(game, Body[BODY].base.vram_tiles_index);
    part!(game, Body[me].base.vram_tiles_index = copied);
    part!(
        game,
        Body[me].base.current_instruction = crate::boss_lists::phantoon_flame_instruction_list(
            game,
            part!(game, Body[me].flame_rain_next) as isize
        )
    );
    part!(game, Body[me].function = function::SPAWN_STARTING_FLAMES);
    part!(game, Mouth[MOUTH]; semi_transparency = 0,
        casual_flame_pattern_index = -1i32 as u16);
}

/// Runs the current function; the body carries the other parts and BG2.
/// Was `Phantoon_Main`.
pub fn main_ai<L: Lease + ?Sized>(game: &mut L) {
    // The bank $A7 function runs on the lease, the effects that need the
    // game in grouped foreign calls. The main AI's debug toggle ($A7:CF0C)
    // stores to ROM, which the cartridge ignores, so it has no effect.
    let me = current(game);
    let function = part!(game, Body[me].function);
    run_function_on(game, function);
    // The main AI carries the parts only when the function returns X = 0
    // ($A7:CEB2). "Almost dead" returns the VRAM queue's new tail in X, so
    // that frame the parts and BG2 keep their old positions.
    if game.current_enemy() == 0 && function != function::ALMOST_DEAD {
        let x: u16 = part!(game, Body[BODY].base.x_pos);
        part!(game, Eye[EYE].base.x_pos = x);
        part!(game, Tentacles[TENTACLES].base.x_pos = x);
        part!(game, Mouth[MOUTH].base.x_pos = x);
        let y: u16 = part!(game, Body[BODY].base.y_pos);
        part!(game, Eye[EYE].base.y_pos = y);
        part!(game, Tentacles[TENTACLES].base.y_pos = y);
        part!(game, Mouth[MOUTH].base.y_pos = y);
        if part!(game, Eye[EYE].background_held) == 0 {
            let x_scroll = (game.camera().layer1[0] as i32
                - part!(game, Body[BODY].base.x_pos) as i32
                + 40) as u16;
            set_bg2_scroll(game, 0, x_scroll);
            let y_scroll = (game.camera().layer1[1] as i32
                - part!(game, Body[BODY].base.y_pos) as i32
                + 40) as u16;
            set_bg2_scroll(game, 1, y_scroll);
        }
    }
}

/// Run the bank $A7 function `function` for the current part, as the bank's
/// pre-instruction dispatch does: Phantoon's on this lease, any other on the
/// game.
fn run_function_on<L: Lease + ?Sized>(game: &mut L, function: u16) {
    let k = game.current_enemy();
    run_function_at(game, function, k);
}
fn run_function_at<L: Lease + ?Sized>(game: &mut L, function: u16, k: u16) {
    match function {
        function::NOTHING => game.owned(|_| ()),
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
        _ => game.run_foreign_function(u32::from(function) | 0xa7_0000),
    }
}

/// Flashes white when hurt, then restores the health palette. Was
/// `Phantoon_Hurt`.
pub fn hurt<S: State + ?Sized>(game: &mut S) {
    if part!(game, Body[BODY].base.flash_timer) != 8 {
        if (part!(game, Body[BODY].base.frame_counter) & 2) == 0 {
            if part!(byte game, Tentacles[TENTACLES].hurt_flash[1]) == 0 {
                return;
            }
        } else {
            if part!(byte game, Tentacles[TENTACLES].hurt_flash[1]) == 0 {
                for index in (112..128).rev() {
                    set_native_word(game, palette_source(index), WHITE);
                }
                part!(game, Tentacles[TENTACLES].hurt_flash, |old| old | 0x100);
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
    part!(
        game,
        Tentacles[TENTACLES].hurt_flash =
            part!(byte game, Tentacles[TENTACLES].hurt_flash[0]) as u16
    );
}

/// Touched by Samus. Was `Phantoon_Touch`.
pub fn touch<S: State + ?Sized>(game: &mut S) {
    game.react_to_contact(Death::LeavePending);
}

/// Hit by a shot: dies, or after enough damage flees into a swoop, a flame
/// rain or (to a super missile) enrages. Was `Phantoon_Shot`.
pub fn shot<S: State + ?Sized>(game: &mut S) {
    // Once dying, shots do nothing.
    if (part!(game, Body[BODY].function) as i32 + 0x26b8) & 0x8000 == 0 {
        return;
    }
    let k: u16 = game.current_enemy();
    let me = current(game);
    let mut health: u16 = part!(game, Body[me].base.health);
    game.react_to_shot(Shot::ReactionOnly);
    if part!(game, Body[me].base.health) == 0 {
        game.queue_sound(1, HIT_SOUND, 6);
        part!(game, Tentacles[TENTACLES].hurt_flash = 1);
        part!(game, Body[BODY].base.properties, |old| old
            | (K_ENEMY_PROPS_TANGIBLE as u16));
        start_dying(game, k);
        return;
    }
    if (part!(game, Body[me].base.ai_handler_bits) & 2) != 0 {
        game.queue_sound(1, HIT_SOUND, 6);
        let function: u16 = part!(game, Body[me].function);
        'react_to_hit: {
            let initial_hit = function == function::EYE_FOLLOWS_SAMUS
                || function == function::FOLLOW_SAMUS_DURING_FLAME_RAIN;
            if !initial_hit && function != function::SWOOPING {
                break 'react_to_hit;
            }
            health = health.wrapping_sub(part!(game, Body[me].base.health));
            if below(health, ENRAGE_DAMAGE)
                || (projectile_kind(game, game.session().room.collision_detection_index) & 0xf00)
                    != SUPER_MISSILE
            {
                let tentacles = k as i32 + TENTACLES;
                let damage = health.wrapping_add(part!(game, Tentacles[tentacles].damage_taken));
                part!(game, Tentacles[tentacles].damage_taken = damage);
                if !initial_hit {
                    if !below(damage, ENRAGE_DAMAGE) {
                        part!(game, Body[me].timer = 1);
                    }
                    break 'react_to_hit;
                }
                if !below(damage, ENRAGE_DAMAGE) {
                    part!(game, Body[me].function = function::FADE_OUT_WITH_SWOOP);
                } else {
                    let roll: u16 = game.phantoon_random() & 7;
                    part!(
                        game,
                        Eye[EYE].unread_shot_value = SHOT_VALUES[roll as i32 as usize] as u16
                    );
                    part!(game, Mouth[MOUTH].shot_roll = roll);
                    part!(game, Tentacles[TENTACLES].hurt_flash = 1);
                    if part!(game, Tentacles[TENTACLES].swooping) == 0 {
                        part!(game, Tentacles[TENTACLES].swooping = 1);
                        if !below(part!(game, Body[me].timer), 16) {
                            part!(game, Body[me].timer = 16);
                        }
                    }
                    return;
                }
            } else {
                part!(game, Body[me].function = function::FADE_OUT_BEFORE_ENRAGE);
            }
            part!(game, Body[me].timer = 0);
            part!(game, Tentacles[TENTACLES].swooping = 0);
            part!(game, Tentacles[k as i32 + TENTACLES].damage_taken = 0);
            game.session_mut().room.fx_layer_flags |= HIDE_BG2;
            part!(game, Body[BODY].base.instruction_timer = 1);
            part!(game, Eye[EYE].base.instruction_timer = 1);
            part!(
                game,
                Body[BODY].base.current_instruction = animation::BODY_INVULNERABLE
            );
            part!(
                game,
                Eye[EYE].base.current_instruction = animation::EYE_CLOSE
            );
            part!(game, Body[BODY].base.properties, |old| old
                | (K_ENEMY_PROPS_TANGIBLE as u16));
            part!(game, Eye[EYE].fade_complete = 0);
        }
        part!(game, Tentacles[TENTACLES].hurt_flash = 2);
    }
}

/// Fades in while the wave grows, then picks the first pattern. Was
/// `Phantoon_WavyFadeIn`.
pub fn wavy_fade_in<S: State + ?Sized>(game: &mut S, k: u16) {
    fade_in(game, FADE_STEPS);
    if phantoon_actions::grow_or_shrink_wave_amplitude(INTRO_WAVE, game) != 0 {
        part!(
            game,
            Body[k as i32].function = function::PICK_ROUND_1_PATTERN
        );
        part!(game, Mouth[MOUTH].semi_transparency = 1);
        part!(game, Body[k as i32].timer = 30);
    } else {
        let me = k as i32;
        let last: bool = part!(game, Body[me].timer) == 1;
        part!(game, Body[me].timer, |old| old.wrapping_sub(1));
        let negative: bool = (part!(game, Body[me].timer) & 0x8000) != 0;
        if last || negative {
            part!(game, Mouth[MOUTH].wave_shrinking = 1);
        }
    }
}

/// Flies the figure-8 with the eye shut, then opens it with a flame spiral.
/// Was `Phantoon_MovePhantoonInFigure8ThenOpenEye`.
pub fn figure_eight<S: State + ?Sized>(game: &mut S, k: u16) {
    phantoon_motion::fly_figure_eight(k, game);
    if eye_timer_expired(game) {
        part!(game, Body[k as i32].function = function::NOTHING);
        part!(game, Eye[EYE].base; instruction_timer = 1, current_instruction = animation::EYE_OPEN);
        part!(game, Body[BODY].flame_rain_next = 0);
        phantoon_actions::spawn_flame_spiral(k, game);
    }
}

/// The vulnerable window: the eye tracks Samus until the timer runs out, then
/// Phantoon swoops if struck or closes its eye. Was
/// `Phantoon_EyeFollowsSamusUntilTimerRunsOut`.
pub fn eye_follows_samus<S: State + ?Sized>(game: &mut S, k: u16) {
    if timer_expired(game, k) {
        part!(game, Tentacles[k as i32 + TENTACLES].damage_taken = 0);
        if part!(game, Tentacles[TENTACLES].swooping) == 0 {
            part!(game, Body[k as i32].function = function::NOTHING);
            part!(game, Body[BODY].base.instruction_timer = 1);
            part!(game, Eye[EYE].base.instruction_timer = 1);
            part!(
                game,
                Body[BODY].base.current_instruction = animation::BODY_INVULNERABLE
            );
            part!(
                game,
                Eye[EYE].base.current_instruction = animation::EYE_CLOSE_AND_PICK_PATTERN
            );
            part!(game, Body[BODY].base.properties, |old| old
                | (K_ENEMY_PROPS_TANGIBLE as u16));
            part!(game, Body[BODY].flame_rain_next = 1);
            return;
        }
        part!(game, Tentacles[TENTACLES].swooping = 0);
        part!(game, Body[k as i32].timer = 60);
        part!(
            game,
            Body[k as i32].function = function::BECOME_SOLID_AND_SWOOP
        );
    }
    look_at_samus(game);
}

/// Swoops until the timer runs out, then fades out. Was `Phantoon_IsSwooping`.
pub fn swooping<S: State + ?Sized>(game: &mut S, k: u16) {
    look_at_samus(game);
    swoop(game, k);
    if timer_expired(game, k) {
        part!(
            game,
            Body[k as i32].function = function::FADE_OUT_WITH_SWOOP
        );
        game.session_mut().room.fx_layer_flags |= HIDE_BG2;
        close_eye(game, animation::EYE_CLOSE);
        part!(game, Eye[EYE].fade_complete = 0);
        part!(game, Tentacles[k as i32 + TENTACLES].damage_taken = 0);
    }
}

/// Was `Phantoon_FadeoutWithSwoop`.
pub fn fade_out_with_swoop<S: State + ?Sized>(game: &mut S, k: u16) {
    swoop(game, k);
    fade_out(game, FADE_STEPS);
    if part!(game, Eye[EYE].fade_complete) != 0 {
        part!(
            game,
            Body[k as i32].function = function::WAIT_AFTER_FADE_OUT
        );
        part!(game, Body[k as i32].timer = 120);
    }
}

/// Reappears on a random side for a new figure-8. Was
/// `Phantoon_MoveLeftOrRightAndPickEyeOpenPatt`.
pub fn reappear<S: State + ?Sized>(game: &mut S, k: u16) {
    if (game.phantoon_random() & 1) != 0 {
        part!(game, Body[BODY].movement_index = 136);
        part!(game, Body[BODY].base.x_pos = 208);
    } else {
        part!(game, Body[BODY].movement_index = 399);
        part!(game, Body[BODY].base.x_pos = 48);
    }
    part!(game, Body[BODY].base.y_pos = 96);
    part!(game, Eye[EYE].movement_reversed = 0);
    part!(game, Body[BODY].speed = 1);
    part!(game, Body[BODY]; speed_subpixel = 0, flame_rain_next = 0);
    pick_pattern(game);
    part!(
        game,
        Body[k as i32].function = function::FADE_IN_BEFORE_FIGURE_EIGHT
    );
    part!(game, Eye[EYE].fade_complete = 0);
}

/// Was `Phantoon_FadeInBeforeFigure8`.
pub fn fade_in_before_figure_eight<S: State + ?Sized>(game: &mut S) {
    fade_in(game, FADE_STEPS);
    if part!(game, Eye[EYE].fade_complete) != 0 {
        part!(game, Body[BODY].function = function::FIGURE_EIGHT);
    }
}

/// Was `Phantoon_BecomeSolidAfterRainingFireballs`.
pub fn become_solid_after_flame_rain<S: State + ?Sized>(game: &mut S, k: u16) {
    part!(game, Eye[EYE].fade_complete = 0);
    game.session_mut().room.fx_layer_flags &= !HIDE_BG2;
    part!(game, Body[BODY].base.instruction_timer = 1);
    part!(game, Eye[EYE].base.instruction_timer = 1);
    part!(
        game,
        Body[BODY].base.current_instruction = animation::BODY_FULL_HITBOX
    );
    part!(
        game,
        Eye[EYE].base.current_instruction = animation::EYEBALL_CENTRED
    );
    part!(
        game,
        Body[k as i32].function = function::FADE_IN_DURING_FLAME_RAIN
    );
}

/// Was `Phantoon_FadeInDuringFireballRain`.
pub fn fade_in_during_flame_rain<S: State + ?Sized>(game: &mut S, k: u16) {
    fade_in(game, 1);
    if part!(game, Eye[EYE].fade_complete) != 0 {
        part!(game, Body[BODY].base.properties, |old| (old as i32
            & !(K_ENEMY_PROPS_TANGIBLE as i32))
            as u16);
        part!(
            game,
            Body[k as i32].function = function::FOLLOW_SAMUS_DURING_FLAME_RAIN
        );
        part!(game, Body[BODY].timer = 90);
    }
}

/// Was `Phantoon_FadeOutDuringFireballRain`.
pub fn fade_out_during_flame_rain<S: State + ?Sized>(game: &mut S, k: u16) {
    fade_out(game, FADE_STEPS);
    if part!(game, Eye[EYE].fade_complete) != 0 {
        part!(game, Body[k as i32].function = function::SPAWN_FLAME_RAIN);
        let roll = (game.phantoon_random() as i32 & 7) as usize;
        part!(
            game,
            Body[k as i32].timer = phantoon_tables::timer(game, Timer::FlameRainHiding, roll)
        );
    }
}

/// Fades out on the figure-8, then rains flames on Samus's side. Was
/// `Phantoon_FadeOutBeforeFirstFireballRain`.
pub fn fade_out_before_first_flame_rain<S: State + ?Sized>(game: &mut S, k: u16) {
    fade_out(game, FADE_STEPS);
    phantoon_motion::fly_figure_eight(k, game);
    if eye_timer_expired(game) {
        part!(game, Tentacles[TENTACLES].swooping = 0);
        part!(
            game,
            Body[k as i32].function = function::BECOME_SOLID_AFTER_FLAME_RAIN
        );
        if below(part!(game, Body[BODY].base.x_pos), 128) {
            phantoon_actions::spawn_flame_rain(k, 0, game);
        } else {
            phantoon_actions::spawn_flame_rain(k, 2, game);
        }
    }
}

/// Was `Phantoon_FadeOutBeforeEnrage`.
pub fn fade_out_before_enrage<S: State + ?Sized>(game: &mut S, k: u16) {
    fade_out(game, FADE_STEPS);
    if part!(game, Eye[EYE].fade_complete) != 0 {
        part!(game, Body[k as i32]; function = function::MOVE_TO_TOP_CENTRE, timer = 120);
    }
}

/// Was `Phantoon_FadeInEnragedPhantoon`.
pub fn fade_in_enraged<S: State + ?Sized>(game: &mut S, k: u16) {
    fade_in(game, FADE_STEPS);
    if part!(game, Eye[EYE].fade_complete) != 0 {
        part!(game, Body[BODY].function = function::ENRAGED);
        part!(game, Body[k as i32].timer = 4);
        part!(game, Eye[EYE].fade_complete = 0);
    }
}

/// Fires eight volleys of flames, then closes its eye. Was `Phantoon_Enraged`.
pub fn enraged<S: State + ?Sized>(game: &mut S, k: u16) {
    if timer_expired(game, k) {
        if (part!(game, Eye[EYE].fade_complete) & 1) != 0 {
            for flame in (8u16..=15).rev() {
                game.spawn_actor_eproj(flame | 0x200, k, phantoon_actions::DESTROYABLE_FLAMES);
            }
        } else {
            for flame in (0u16..=6).rev() {
                game.spawn_actor_eproj(flame | 0x200, k, phantoon_actions::DESTROYABLE_FLAMES);
            }
        }
        game.queue_sound(2, ENRAGED_FLAMES_SOUND, 6);
        let volleys: u16 = (part!(game, Eye[EYE].fade_complete)).wrapping_add(1);
        part!(game, Eye[EYE].fade_complete = volleys);
        if below(volleys, 8) {
            part!(game, Body[k as i32].timer = 128);
        } else {
            part!(game, Eye[EYE].base; instruction_timer = 1, current_instruction = animation::EYE_CLOSE);
            part!(game, Eye[EYE].fade_complete = 0);
            part!(
                game,
                Body[k as i32].function = function::FADE_OUT_AFTER_ENRAGE
            );
        }
    }
}

/// Was `Phantoon_FadeoutAfterEnrage`.
pub fn fade_out_after_enrage<S: State + ?Sized>(game: &mut S, k: u16) {
    fade_out(game, FADE_STEPS);
    if part!(game, Eye[EYE].fade_complete) != 0 {
        part!(
            game,
            Body[k as i32].function = function::WAIT_AFTER_FADE_OUT
        );
        part!(game, Body[k as i32].timer = 120);
    }
}

/// Swoops to the middle of the room once fatally shot mid-swoop. Was
/// `Phantoon_CompleteSwoopAfterFatalShot`.
pub fn finish_swoop_after_fatal_shot<S: State + ?Sized>(game: &mut S, k: u16) {
    look_at_samus(game);
    swoop(game, k);
    let x: u16 = part!(game, Body[BODY].base.x_pos);
    if !below(x, 96) && below(x, 160) {
        part!(game, Body[k as i32].function = function::DYING_FADE_IN_OUT);
    }
}

/// Flickers in and out ten times, then starts the explosions. Was
/// `Phantoon_DyingPhantoonFadeInOut`.
pub fn dying_fade_in_out<S: State + ?Sized>(game: &mut S, k: u16) {
    if (part!(game, Eye[EYE].movement_reversed) & 1) != 0 {
        fade_in(game, FADE_STEPS);
        if part!(game, Eye[EYE].fade_complete) == 0 {
            return;
        }
    } else {
        fade_out(game, FADE_STEPS);
        if part!(game, Eye[EYE].fade_complete) == 0 {
            return;
        }
    }
    part!(game, Eye[EYE].fade_complete = 0);
    let fades: u16 = (part!(game, Eye[EYE].movement_reversed)).wrapping_add(1);
    part!(game, Eye[EYE].movement_reversed = fades);
    if !below(fades, 10) {
        part!(game, Body[k as i32]; function = function::DYING_EXPLOSIONS, timer = 15);
        part!(game, Tentacles[TENTACLES].explosion = 0);
        part!(game, Body[BODY].movement_index = 0);
    }
}

/// Starts the dying wave and cry. Was `Phantoon_WavyDyingPhantoonAndCry`.
pub fn wavy_dying_cry<L: Lease + ?Sized>(game: &mut L, k: u16) {
    game.start_wave(1, WAVE_PHASE_DELTA);
    part!(game, Mouth[MOUTH].wave_amplitude = 0);
    part!(game, Body[k as i32].function = function::DYING_FADE_OUT);
    part!(game, Eye[EYE].movement_reversed = 2);
    let hidden: u16 = (part!(game, Body[BODY].base.properties) as i32
        & !(K_ENEMY_PROPS_DISABLE_SAMUS_COLL as i32
            | K_ENEMY_PROPS_TANGIBLE as i32
            | K_ENEMY_PROPS_INVISIBLE as i32)
        | K_ENEMY_PROPS_TANGIBLE as i32
        | K_ENEMY_PROPS_INVISIBLE as i32) as u16;
    part!(game, Eye[EYE].base.properties = hidden);
    part!(game, Tentacles[TENTACLES].base.properties = hidden);
    part!(game, Mouth[MOUTH].base.properties = hidden);
    game.queue_sound(1, DYING_CRY, 6);
}

/// Brightens the screen in steps while the wave grows, then fades out. Was
/// `Phantoon_DyingFadeOut`.
pub fn dying_fade_out<S: State + ?Sized>(game: &mut S, k: u16) {
    phantoon_actions::grow_or_shrink_wave_amplitude(DYING_WAVE, game);
    if part!(game, Eye[EYE].movement_reversed) == 0xffff {
        fade_out(game, FADE_STEPS);
        if part!(game, Eye[EYE].fade_complete) != 0 {
            part!(game, Body[k as i32].function = function::ALMOST_DEAD);
        }
    } else if (frame(game) & 0xf) == 0 {
        if part!(byte game, Eye[EYE].movement_reversed[0]) == 0xf2 {
            part!(game, Eye[EYE]; movement_reversed = -1i32 as u16, fade_complete = 0);
        } else {
            let brightness: u8 = (part!(byte game, Eye[EYE].movement_reversed[0])).wrapping_add(16);
            part!(byte game, Eye[EYE].movement_reversed[0] = brightness);
            set_mosaic(game, brightness);
        }
    }
}

/// Clears BG2 and queues its blank tilemap. Was `Phantoon_AlmostDead`.
pub fn almost_dead<S: State + ?Sized>(game: &mut S, k: u16) {
    set_mosaic(game, 0);
    part!(game, Eye[EYE].background_held = 0);
    game.session_mut().room.fx_layer_flags &= !HIDE_BG2;
    part!(game, Mouth[MOUTH].semi_transparency = -1i32 as u16);
    part!(game, Body[k as i32]; function = function::DEAD, timer = 60);
    part!(game, Eye[EYE].fade_complete = 0);
    part!(game, Body[BODY].base; x_pos = 384, y_pos = 128);
    for index in (0..=511).rev() {
        game.session_mut().actors.extended.blocks[4][index] = BLANK_BG2_TILE;
    }
    // A VRAM write queue entry: 400h bytes from $7E:2000 to VRAM $4800.
    let source = Source {
        address: 0x2000,
        bank: 0x7e,
    };
    queue_tilemap(
        game,
        TilemapUpload {
            bytes: 1024,
            source,
            column: false,
            vram_address: 18432,
        },
    );
}

/// Fades the room to its powered-on colours, then dies: the drop, the boss
/// flag, the door PLM and the music. Was `Phantoon_Dead`.
pub fn dead<L: Lease + ?Sized>(game: &mut L, k: u16) {
    let me = k as i32;
    if part!(game, Body[me].timer) != 0 {
        part!(game, Body[me].timer, |old| old.wrapping_sub(1));
    } else if (frame(game) & 3) == 0 {
        part!(game, Eye[EYE].fade_steps = 12);
        if fade_step_toward_powered_on(game) & 1 != 0 {
            enable_bg2(game);
            game.foreign(|game| game.scatter_enemy_drop(DropSource::Phantoon, k));
            let deleted: u16 =
                (part!(game, Body[BODY].base.properties)) | (K_ENEMY_PROPS_DELETED as u16);
            part!(game, Body[BODY].base.properties = deleted);
            part!(game, Eye[EYE].base.properties = deleted);
            part!(game, Tentacles[TENTACLES].base.properties = deleted);
            part!(game, Mouth[MOUTH].base.properties = deleted);
            game.owned(|game| {
                let area = game.session().room.area;
                let bits = crate::progress::boss_word_at(
                    game,
                    area,
                    crate::gameplay_flag_host::BossWordOrder::Little,
                );
                crate::progress::set_boss_word_le(game, area, bits | 1);
            });
            game.foreign(|game| {
                game.spawn_plm(0, 0x6, DEFEAT_PLM);
                game.queue_music_delayed(3);
            });
        }
    }
}

/// Looks at Samus and starts the swoop. Was `Phantoon_BecomesSolidAndBodyVuln`.
pub fn become_solid_and_swoop<S: State + ?Sized>(game: &mut S, k: u16) {
    look_at_samus(game);
    game.session_mut().room.fx_layer_flags &= !HIDE_BG2;
    begin_swoop(game, k);
    part!(game, Body[BODY].base; instruction_timer = 1, current_instruction = animation::BODY_FULL_HITBOX);
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
            part!(game, Body[k as i32]; timer = 240, speed_subpixel = 1);
            part!(
                game,
                Body[k as i32].function = function::SPAWN_FLAMES_BEFORE_FIGHT
            );
        }
        function::SPAWN_FLAMES_BEFORE_FIGHT => start_fight(game, k),
        function::PICK_ROUND_1_PATTERN => pick_first_pattern(game, k),
        function::WAIT_AFTER_FADE_OUT => {
            part!(game, Body[k as i32].function = function::REAPPEAR);
        }
        function::FOLLOW_SAMUS_DURING_FLAME_RAIN => end_flame_rain_window(game, k),
        function::SPAWN_FLAME_RAIN => spawn_flame_rain(game, k),
        function::MOVE_TO_TOP_CENTRE => {
            part!(game, Body[k as i32].function = function::FADE_IN_ENRAGED);
            part!(game, Body[BODY].base; x_pos = 128, y_pos = 32);
            part!(game, Eye[EYE].fade_complete = 0);
        }
        function::DYING_EXPLOSIONS => death_explosion(game, k),
        _ => game.foreign(|game| game.unreachable()),
    }
}

/// One of the eight opening flames. Was `Phantoon_Spawn8FireballsInCircleAtStart`.
fn spawn_starting_flame<L: Lease + ?Sized>(game: &mut L, k: u16) {
    game.spawn_actor_eproj(part!(game, Body[BODY].movement_index), k, STARTING_FLAMES);
    game.queue_sound(2, STARTING_FLAME_SOUND, 6);
    part!(game, Body[k as i32].timer = 30);
    let flames: u16 = (part!(game, Body[BODY].movement_index)).wrapping_add(1);
    part!(game, Body[BODY].movement_index = flames);
    if !below(flames, 8) {
        part!(game, Body[BODY].movement_index = 0);
        part!(game, Tentacles[k as i32 + TENTACLES].damage_taken = 0);
        part!(
            game,
            Body[k as i32].function = function::WAIT_BEFORE_SPINNING_FLAMES
        );
        part!(game, Body[k as i32].timer = 30);
        game.foreign(|game| game.spawn_plm(0, 0x6, FIGHT_START_PLM));
    }
}

/// The opening flames fly off and the wavy fade-in starts the fight. Was
/// `Phantoon_SpawnFireballsBeforeFight`.
fn start_fight<L: Lease + ?Sized>(game: &mut L, k: u16) {
    part!(game, Body[k as i32].speed_subpixel = 0);
    game.session_mut().room.fx_layer_flags |= HIDE_BG2;
    part!(game, Body[k as i32].function = function::WAVY_FADE_IN);
    part!(game, Mouth[MOUTH].semi_transparency = -32767i32 as u16);
    part!(game, Body[k as i32].timer = 120);
    game.start_wave(2, WAVE_PHASE_DELTA);
    part!(game, Mouth[MOUTH].wave_amplitude = INTRO_MAX_AMPLITUDE);
    part!(game, Eye[EYE].fade_complete = 0);
    game.foreign(|game| game.queue_music_delayed(5));
}

/// Picks the first figure-8 direction. Was `Phantoon_PickPatternForRound1`.
fn pick_first_pattern<S: State + ?Sized>(game: &mut S, k: u16) {
    part!(game, Eye[EYE].background_held = 0);
    part!(game, Body[k as i32].function = function::FIGURE_EIGHT);
    part!(
        game,
        Eye[EYE].timer = phantoon_tables::timer(
            game,
            Timer::EyeClosed,
            (frame(game) as i32 >> 1 & 3) as usize
        )
    );
    if (game.phantoon_random() & 1) != 0 {
        part!(game, Body[BODY]; speed = 0, speed_subpixel = 0, movement_stage = 0);
        part!(game, Eye[EYE].movement_reversed = 1);
        part!(game, Body[BODY].movement_index = 533);
    } else {
        part!(game, Body[BODY].speed = 1);
        part!(game, Body[BODY]; speed_subpixel = 0, movement_stage = 0);
        part!(game, Eye[EYE].movement_reversed = 0);
        part!(game, Body[BODY].movement_index = 0);
    }
}

/// The flame rain's vulnerable window ends: a swoop if struck, else hide.
/// Was `Phantoon_FollowSamusWithEyeDuringFireballRain`.
fn end_flame_rain_window<S: State + ?Sized>(game: &mut S, k: u16) {
    part!(game, Tentacles[k as i32 + TENTACLES].damage_taken = 0);
    if part!(game, Tentacles[TENTACLES].swooping) != 0 {
        part!(game, Tentacles[TENTACLES].swooping = 0);
        part!(game, Body[BODY].flame_rain_next = 1);
        begin_swoop(game, k);
    } else {
        part!(
            game,
            Body[k as i32].function = function::FADE_OUT_DURING_FLAME_RAIN
        );
        part!(game, Eye[EYE].fade_complete = 0);
        close_eye(game, animation::EYE_CLOSE);
        game.session_mut().room.fx_layer_flags |= HIDE_BG2;
    }
}

/// Reappears at a random flame rain position and rains flames. Was
/// `Phantoon_SpawnRainingFireballs`.
fn spawn_flame_rain<S: State + ?Sized>(game: &mut S, k: u16) {
    let pattern: u16 = game.phantoon_random() & 7;
    let row: i32 = (8 * pattern as i32) >> 1;
    let position = phantoon_tables::flame_rain(game, (row >> 2) as usize);
    part!(
        game,
        Body[BODY].movement_index = position.figure_eight_index
    );
    part!(game, Body[BODY].base; x_pos = position.x_position, y_pos = position.y_position);
    part!(game, Eye[EYE].movement_reversed = 0);
    part!(
        game,
        Body[k as i32].function = function::BECOME_SOLID_AFTER_FLAME_RAIN
    );
    phantoon_actions::spawn_flame_rain(k, pattern, game);
}

/// One death explosion; three rounds of the last eight, then the cry. Was
/// `Phantoon_DyingPhantoonExplosions`.
fn death_explosion<S: State + ?Sized>(game: &mut S, k: u16) {
    let offset: u16 = 4u16.wrapping_mul(part!(game, Tentacles[TENTACLES].explosion));
    let explosion = |game: &mut S, at: u16| {
        game.native_image().read(
            sm_data::cartridge::lorom_offset(DEATH_EXPLOSIONS) as usize
                + usize::from(offset)
                + usize::from(at),
        )
    };
    let x = ((explosion(game, 0) as i8) as u16).wrapping_add(part!(game, Body[BODY].base.x_pos));
    let y = ((explosion(game, 1) as i8) as u16).wrapping_add(part!(game, Body[BODY].base.y_pos));
    game.place_projectile_spawn(x, y);
    let kind: u16 = explosion(game, 2) as u16;
    game.spawn_room_eproj(DUST_CLOUD, kind);
    if kind == 29 {
        game.queue_sound(1, BIG_EXPLOSION_SOUND, 6);
    } else {
        game.queue_sound(1, SMALL_EXPLOSION_SOUND, 6);
    }
    part!(game, Body[k as i32].timer = explosion(game, 3) as u16);
    let next: u16 = (part!(game, Tentacles[TENTACLES].explosion)).wrapping_add(1);
    part!(game, Tentacles[TENTACLES].explosion = next);
    if !below(next, 13) {
        part!(game, Tentacles[TENTACLES].explosion = 5);
        let rounds: u16 = (part!(game, Body[BODY].movement_index)).wrapping_add(1);
        part!(game, Body[BODY].movement_index = rounds);
        if !below(rounds, 3) {
            part!(game, Body[k as i32].function = function::WAVY_DYING_CRY);
        }
    }
}

// The named bank $A7 functions are the ones `DispatchEnemyPreInstr` dispatches on.
const _: () = {
    use crate::actors::phantoon::function as f;
    const fn at(word: u16) -> u32 {
        0xa7_0000 | word as u32
    }
    assert!(
        at(f::SPAWN_STARTING_FLAMES) == 10998953 && at(f::WAIT_BEFORE_SPINNING_FLAMES) == 10999022
    );
    assert!(at(f::SPAWN_FLAMES_BEFORE_FIGHT) == 10999048 && at(f::WAVY_FADE_IN) == 10999114);
    assert!(at(f::PICK_ROUND_1_PATTERN) == 10999190 && at(f::FIGURE_EIGHT) == 10999271);
    assert!(at(f::EYE_FOLLOWS_SAMUS) == 10999309 && at(f::BECOME_SOLID_AND_SWOOP) == 10999388);
    assert!(at(f::SWOOPING) == 10999416 && at(f::FADE_OUT_WITH_SWOOP) == 10999481);
    assert!(at(f::WAIT_AFTER_FADE_OUT) == 10999508 && at(f::REAPPEAR) == 10999522);
    assert!(
        at(f::FADE_IN_BEFORE_FIGURE_EIGHT) == 10999597
            && at(f::BECOME_SOLID_AFTER_FLAME_RAIN) == 10999615
    );
    assert!(
        at(f::FADE_IN_DURING_FLAME_RAIN) == 10999655
            && at(f::FOLLOW_SAMUS_DURING_FLAME_RAIN) == 10999688
    );
    assert!(at(f::FADE_OUT_DURING_FLAME_RAIN) == 10999765 && at(f::SPAWN_FLAME_RAIN) == 10999799);
    assert!(
        at(f::FADE_OUT_BEFORE_FIRST_FLAME_RAIN) == 10999850
            && at(f::FADE_OUT_BEFORE_ENRAGE) == 10999900
    );
    assert!(at(f::MOVE_TO_TOP_CENTRE) == 10999924 && at(f::FADE_IN_ENRAGED) == 10999953);
    assert!(at(f::ENRAGED) == 10999980 && at(f::FADE_OUT_AFTER_ENRAGE) == 11000086);
    assert!(
        at(f::FINISH_SWOOP_AFTER_FATAL_SHOT) == 11000110 && at(f::DYING_FADE_IN_OUT) == 11000136
    );
    assert!(at(f::DYING_EXPLOSIONS) == 11000203 && at(f::WAVY_DYING_CRY) == 11000401);
    assert!(at(f::DYING_FADE_OUT) == 11000454 && at(f::ALMOST_DEAD) == 11000535);
    assert!(
        at(f::DEAD) == 11000637 && f::NOTHING == crate::constants::bank_a7::FNNULLSUB_237 as u16
    );
};
