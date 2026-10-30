//! Draygon, Maridia's crustacean boss: the slot views, functions and
//! instruction lists its routines share (bank $A5:8687-$A5:C5C6), and what it
//! needs beyond the shared boss effects.
//!
//! Draygon is four enemies: the body in slot 0, then the eye, tail and arms.
#![forbid(unsafe_code)]
use crate::actors::boss_lease::{BossLease, LeaseBoss};
use crate::actors::host::BossEffects;
pub use crate::actors::host::abs16;
use crate::actors::layouts::part_view;

pub trait Host: BossEffects + BossLease<Game = Self> + LeaseBoss {
    fn draygon_spawn_trails(&mut self);
    fn draygon_trajectory(&mut self);
    /// Draygon grabs Samus, facing `direction`.
    fn grab_samus(&mut self, direction: u16);
    fn release_samus(&mut self);
    fn draygon_spawn_ring(&mut self);
    fn draygon_death_bubble(&mut self);
    /// Draygon's sprite motion with `horizontal_bias`.
    fn draygon_motion(&mut self, horizontal_bias: u16);
    /// Draygon's death drop for `area`.
    fn scatter_draygon_drop(&mut self, area: u16);
}

part_view! {
    /// A Draygon part's view of its slot; the named words are the body's.
    Draygon: EnemyLayout104 {
        words: [
            function,
            // The function timer, which the swoop uses as its Y position index.
            function_timer,
            // The swoop's duration, times 4.
            swoop_duration,
            x_speed, x_subspeed, var_F, parameter_1, parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            left_reset_x, reset_y, right_reset_x, goop_counter, var_04, spiral_x_radius, spiral_x,
            spiral_y, spiral_angle, spiral_x_subradius, spiral_y_subpixel, spiral_angle_delta,
            tail_whip_timer, goop_y_angle, health_palette, swoop_y_acceleration, var_10, var_11,
            var_12, var_13, var_14, var_15, var_16, var_17, var_18, var_19, var_1A, var_1B, var_1C,
            var_1D, var_1E, var_1F,
        ],
        pad2: [u8; 1984],
        words: [
            facing_right, var_21, var_22, var_23, var_24, var_25, var_26, var_27,
            // The death drift's speeds, which nothing reads.
            death_x_speed, death_x_subspeed, death_y_speed, death_y_subspeed,
            var_2C, var_2D, var_2E, var_2F, var_30, var_31, var_32, var_33, var_34, var_35, var_36,
            var_37, var_38, var_39, var_3A, var_3B, var_3C, var_3D, var_3E, var_3F,
        ],
        pad3: [u8; 1984],
        words: [
            // Indexed by the turret's byte offset, as a slot.
            turret_destroyed,
            var_41, var_42, death_angle, var_44, turret_5_destroyed, var_46, var_47, var_48, var_49,
            var_4A, var_4B, var_4C, var_4D, var_4E, var_4F, var_50, var_51, var_52, var_53, var_54,
            var_55, var_56, var_57, var_58, var_59, var_5A, var_5B, var_5C, var_5D,
            graphics_x_displacement, graphics_y_displacement,
        ],
    }
}

/// The parts' slots.
pub const BODY: i32 = 0;
pub const EYE: i32 = 0x40;
pub const TAIL: i32 = 0x80;
pub const ARMS: i32 = 0xc0;

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
/// Properties bit 10: intangible (the decompiler's `kEnemyProps_Tangible`).
pub const INTANGIBLE: u16 = 0x400;
/// Properties bit 9: delete.
pub const DELETE: u16 = 0x200;
/// A sign test on the low 16 bits.
pub const SIGN: u16 = 0x8000;
