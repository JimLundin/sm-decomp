//! Draygon, Maridia's crustacean boss: the slot views, functions and
//! instruction lists its routines share (bank $A5:8687-$A5:C5C6), and what it
//! needs beyond the shared boss effects.
//!
//! Draygon is four enemies: the body in slot 0, then the eye, tail and arms.
use crate::actors::boss_lease::BossLease;
use crate::actors::host::BossEffects;
pub use crate::actors::host::abs16;
use crate::actors::slot_view::{SlotState, slot_view};

/// A lease on a game with Draygon's effects.
pub trait Lease:
    BossLease<Game: Host>
    + crate::actors::boss_interaction::Access
    + SlotState
    + crate::progress::FlagHost
    + crate::sprite_effect_host::Host
    + crate::enemy_projectiles::host::Host
    + crate::player_movement::effects::Host
    + crate::player_projectiles::host::Host
    + crate::cartridge::NativeDiagnostics
{
}
impl<T> Lease for T where
    T: BossLease<Game: Host>
        + crate::actors::boss_interaction::Access
        + SlotState
        + crate::progress::FlagHost
        + crate::sprite_effect_host::Host
        + crate::enemy_projectiles::host::Host
        + crate::player_movement::effects::Host
        + crate::player_projectiles::host::Host
        + crate::cartridge::NativeDiagnostics
        + ?Sized
{
}

pub trait Host: BossEffects + BossLease<Game = Self> {
    fn draygon_spawn_trails(&mut self);
    fn draygon_trajectory(&mut self);
    /// Draygon grabs Samus, facing `direction`.
    fn grab_samus(&mut self, direction: u16);
}

slot_view! {
    /// A Draygon part's slot; the named words are the body's: its function and
    /// function timer (the swoop's Y position index), the swoop's duration
    /// (times 4) and X speed, the reset positions, the goop counter and Y
    /// angle, the grab's spiral, the tail whip timer, the health palette, the
    /// swoop's Y acceleration, the facing, the death drift's speeds (which
    /// nothing reads) and angle, the destroyed turrets and the graphics
    /// displacement.
    Draygon, DraygonEdit {
        record: [
            function = var_a, function_timer = var_b, swoop_duration = var_c, x_speed = var_d,
            x_subspeed = var_e,
        ],
        extended: [
            left_reset_x = var_00, reset_y = var_01, right_reset_x = var_02,
            goop_counter = var_03, spiral_x_radius = var_05, spiral_x = var_06,
            spiral_y = var_07, spiral_angle = var_08, spiral_x_subradius = var_09,
            spiral_y_subpixel = var_0a, spiral_angle_delta = var_0b, tail_whip_timer = var_0c,
            goop_y_angle = var_0d, health_palette = var_0e, swoop_y_acceleration = var_0f,
            facing_right = var_20, death_x_speed = var_28, death_x_subspeed = var_29,
            death_y_speed = var_2a, death_y_subspeed = var_2b, turret_2_destroyed = var_42,
            death_angle = var_43, turret_4_destroyed = var_44, turret_5_destroyed = var_45,
            graphics_x_displacement = var_5e, graphics_y_displacement = var_5f,
        ],
    }
}

impl Draygon<'_> {
    /// Whether wall turret `index` (its byte offset, 4 to 10) is destroyed:
    /// the body's word at `index` past `var_40`, so turret 3's is the death
    /// angle.
    pub fn turret_destroyed(&self, index: u16) -> u16 {
        match index {
            4 => self.turret_2_destroyed(),
            6 => self.death_angle(),
            8 => self.turret_4_destroyed(),
            10 => self.turret_5_destroyed(),
            _ => unreachable!("Draygon turret offset {index:#x}"),
        }
    }
}

/// The parts' slots.
pub const BODY: u16 = 0;
pub const EYE: u16 = 0x40;
pub const TAIL: u16 = 0x80;
pub const ARMS: u16 = 0xc0;

/// Bank $A5 Draygon body functions, stored in `function`.
pub mod function {
    pub const FIGHT_INTRO_INITIAL_DELAY: u16 = 0x871b;
    pub const FIGHT_INTRO_DANCE: u16 = 0x878b;
    pub const SWOOP_RIGHT_SETUP: u16 = 0x87f4;
    pub const SWOOP_RIGHT_DESCENDING: u16 = 0x88b1;
    pub const SWOOP_RIGHT_APEX: u16 = 0x8922;
    pub const SWOOP_RIGHT_ASCENDING: u16 = 0x8951;
    pub const SWOOP_LEFT_SETUP: u16 = 0x89b3;
    pub const SWOOP_LEFT_DESCENDING: u16 = 0x8a00;
    pub const SWOOP_LEFT_APEX: u16 = 0x8a50;
    pub const SWOOP_LEFT_ASCENDING: u16 = 0x8a90;
    pub const GOOP_RIGHT_SETUP: u16 = 0x8b0a;
    pub const GOOP_RIGHT_MOVE_UNTIL_IN_RANGE: u16 = 0x8b52;
    pub const GOOP_RIGHT_FIRING: u16 = 0x8bae;
    pub const GOOP_RIGHT_MOVE_UNTIL_OFF_SCREEN: u16 = 0x8c33;
    pub const GOOP_LEFT_SETUP: u16 = 0x8c8e;
    pub const GOOP_LEFT_MOVE_UNTIL_IN_RANGE: u16 = 0x8cd4;
    pub const GOOP_LEFT_FIRING: u16 = 0x8d30;
    pub const GOOP_LEFT_MOVE_UNTIL_OFF_SCREEN: u16 = 0x8db2;
    pub const CHASE_SAMUS: u16 = 0x8e19;
    pub const REPELLED_BY_GRAPPLE: u16 = 0x8f10;
    pub const GRABBED_MOVING_TO_TARGET: u16 = 0x8f1e;
    pub const GRABBED_RISING_SPIRAL: u16 = 0x8fd6;
    pub const GRABBED_TAIL_WHIP: u16 = 0x90d4;
    pub const GRABBED_FINAL_TAIL_WHIPS_START: u16 = 0x9105;
    pub const GRABBED_FINAL_TAIL_WHIPS_ONGOING: u16 = 0x9124;
    pub const FLAIL_TAIL_AND_FLY_STRAIGHT_UP: u16 = 0x9128;
    pub const FLY_STRAIGHT_UP: u16 = 0x9154;
    pub const DEATH_DRIFT_TO_DEATH_SPOT: u16 = 0x9185;
    pub const DEATH_WAIT_FOR_EVIRS: u16 = 0x9294;
    pub const DEATH_BURIED_BY_EVIRS: u16 = 0x92ab;
    /// The eye's functions while facing left ($A5:C48D) and right ($A5:C513).
    pub const EYE_FACING_LEFT: u16 = 0xc48d;
    pub const EYE_FACING_RIGHT: u16 = 0xc513;
    /// $A5:804B, an RTS: the eye's function while it does nothing.
    pub const NONE: u16 = 0x804b;
}

/// Bank $A5 instruction lists, by part and facing.
pub mod instruction_list {
    pub const SLEEP: u16 = 0x97b9;
    pub const DELETE: u16 = 0x98ed;
    pub const BODY_LEFT_RESET: u16 = 0x97bb;
    pub const BODY_RIGHT_RESET: u16 = 0x97d1;
    pub const BODY_LEFT_IDLE: u16 = 0x9889;
    pub const BODY_LEFT_FIRE_GOOP: u16 = 0x98fe;
    pub const BODY_RIGHT_FIRE_GOOP: u16 = 0x9c90;
    pub const BODY_LEFT_DYING: u16 = 0x9867;
    pub const BODY_RIGHT_DYING: u16 = 0x9c5a;
    pub const BODY_LEFT_ROAR: u16 = 0x9922;
    pub const BODY_RIGHT_ROAR: u16 = 0x9cb4;
    pub const EYE_LEFT_IDLE: u16 = 0x9944;
    pub const EYE_LEFT_DYING: u16 = 0x997a;
    pub const EYE_RIGHT_DYING: u16 = 0x9d1c;
    pub const EYE_LEFT_DEAD: u16 = 0x999c;
    pub const EYE_RIGHT_DEAD: u16 = 0x9d3e;
    pub const EYE_LEFT_LOOKING_LEFT: u16 = 0x99ae;
    pub const EYE_LEFT_LOOKING_RIGHT: u16 = 0x99b4;
    pub const EYE_LEFT_LOOKING_UP: u16 = 0x99ba;
    pub const EYE_LEFT_LOOKING_DOWN: u16 = 0x99c0;
    pub const EYE_RIGHT_LOOKING_RIGHT: u16 = 0x9d50;
    pub const EYE_RIGHT_LOOKING_LEFT: u16 = 0x9d56;
    pub const EYE_RIGHT_LOOKING_UP: u16 = 0x9d5c;
    pub const EYE_RIGHT_LOOKING_DOWN: u16 = 0x9d62;
    pub const TAIL_LEFT_FAKE_TAIL_WHIP: u16 = 0x99fc;
    pub const TAIL_LEFT_TAIL_WHIP: u16 = 0x9ae8;
    pub const TAIL_RIGHT_TAIL_WHIP: u16 = 0x9ea1;
    pub const TAIL_LEFT_FINAL_TAIL_WHIPS: u16 = 0x9a68;
    pub const TAIL_RIGHT_FINAL_TAIL_WHIPS: u16 = 0x9e21;
    pub const TAIL_LEFT_TAIL_FLAIL: u16 = 0x9b5a;
    pub const TAIL_RIGHT_TAIL_FLAIL: u16 = 0x9f15;
    pub const ARMS_LEFT_IDLE: u16 = 0x97e7;
    pub const ARMS_RIGHT_IDLE: u16 = 0x9bda;
    pub const ARMS_LEFT_NEAR_SWOOP_APEX: u16 = 0x9813;
    pub const ARMS_RIGHT_NEAR_SWOOP_APEX: u16 = 0x9c06;
    pub const ARMS_LEFT_GRAB: u16 = 0x9845;
    pub const ARMS_RIGHT_GRAB: u16 = 0x9c38;
}

/// Draygon's code and data are in bank $A5.
pub const BANK: u8 = 0xa5;
/// Grapple beam function $C8C5: dropped.
pub const GRAPPLE_DROPPED: u16 = 0xc8c5;
/// Properties bit 10: intangible (the decompiler's `K_ENEMY_PROPS_TANGIBLE`).
pub const INTANGIBLE: u16 = 0x400;
/// Properties bit 9: delete.
pub const DELETE: u16 = 0x200;
/// A sign test on the low 16 bits.
pub const SIGN: u16 = 0x8000;
