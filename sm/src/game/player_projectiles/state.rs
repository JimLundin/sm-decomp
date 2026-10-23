//! Concrete owned projectile, trail, charge and launch-history state.
#![forbid(unsafe_code)]
use crate::game::projectile_playback as playback;
#[derive(Clone, Copy, Default)]
pub struct Cursor(pub u16);
#[derive(Clone, Copy, Default)]
pub struct Sprite(pub u16);
#[derive(Clone, Copy, Default)]
pub struct MotionCode(pub u16);
#[derive(Clone, Copy, Default)]
pub struct Kind(pub u16);
#[derive(Default)]
pub struct Shot {
    pub x: u16,
    pub y: u16,
    pub x_fraction: u16,
    pub y_fraction: u16,
    pub x_speed: u16,
    pub y_speed: u16,
    pub direction: u16,
    pub kind: Kind,
    pub damage: u16,
    pub motion: MotionCode,
    pub variable: u16,
    pub trail_time: u16,
    pub auxiliary: u16,
    pub animation: playback::Animation<Cursor, Sprite>,
}
#[derive(Default)]
pub struct PlayerProjectiles {
    pub trails: crate::game::projectile_trail_emission::Pool<Cursor>,
    pub launch: crate::game::launch_motion::LaunchMotion,
    pub controls: Controls,
    pub shots: [Shot; 10],
    pub selected: u16,
    pub count: u16,
    pub bomb_count: u16,
}

#[derive(Default)]
pub struct Controls {
    pub cooldown: u16,
    pub spread_charge: u16,
    pub orbit_step: u16,
    pub charge: u16,
    pub previous_charge: u16,
    pub glow: u16,
    pub charge_palette: u16,
    pub resume_sound: u16,
    pub echo_sound: u16,
    pub pending_aim: u16,
    pub protection: u16,
    pub selected_weapon: u16,
    pub automatic_weapon: u16,
    pub weapon_changed: u16,
    pub flare_frames: [u16; 3],
    pub flare_timers: [u16; 3],
}
