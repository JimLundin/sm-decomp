//! Shared state proposal: ordinary fields, one live owner, no RAM storage.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PrepareEntry {
    Normal,
    Demo,
    ProjectilesOnly,
    Inactive,
}
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Shot {
    pub x: u16,
    pub y: u16,
    pub x_fraction: u16,
    pub y_fraction: u16,
    pub x_radius: u16,
    pub y_radius: u16,
    pub x_speed: u16,
    pub y_speed: u16,
    pub direction: u16,
    pub kind: u16,
    pub damage: u16,
    pub cursor: u16,
    pub remaining: u16,
    pub motion: u16,
    pub variable: u16,
    pub trail_time: u16,
    pub auxiliary: u16,
    pub sprite: u16,
}
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Trail {
    pub remaining: u16,
    pub cursor: u16,
    pub tile: u16,
    pub x: u16,
    pub y: u16,
}
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct TrailPair {
    pub left: Trail,
    pub right: Trail,
}
#[derive(Clone, Default, Debug, PartialEq)]
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
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Preparation {
    pub next_pose: u16,
    pub interrupted_pose: u16,
    pub transitional_pose: u16,
    pub momentum_routine: u16,
    pub special_graphics: u16,
    pub hurt_switch: u16,
    pub demo_held: u16,
    pub demo_pressed: u16,
    pub demo_filtered_held: u16,
    pub demo_filtered_pressed: u16,
    pub animation_skip: u16,
    pub aim_history: u16,
    pub previous_movement_and_facing: u16,
}
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Player {
    pub x: u16,
    pub y: u16,
    pub missiles: u16,
    pub super_missiles: u16,
    pub power_bombs: u16,
    pub grapple: u16,
    pub movement_handler: u16,
    pub shine_timer: u16,
    pub shine_tick: u16,
    pub palette_frame: u16,
    pub special_palette_flags: u16,
}
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Echoes {
    pub index: u16,
    pub x: [u16; 4],
    pub y: [u16; 4],
    pub x_speed: [u16; 4],
}
#[derive(Clone, Default, Debug, PartialEq)]
pub struct PowerBomb {
    pub flag: u16,
    pub x: u16,
    pub y: u16,
}
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Session {
    pub shots: [Shot; 10],
    pub trails: [TrailPair; 18],
    pub controls: Controls,
    pub selected: u16,
    pub count: u16,
    pub bomb_count: u16,
    pub player: Player,
    pub preparation: Preparation,
    pub echoes: Echoes,
    pub power_bomb: PowerBomb,
    pub room_cursor: u16,
    pub earthquake_kind: u16,
    pub earthquake_timer: u16,
    // Replace with existing LaunchMotion after adding its ordinary Clone/Debug
    // derives if necessary. Its displacements encode high-before-low.
    pub launch: crate::game::launch_motion::LaunchMotion,
}

// Observations are refreshed after every external effect; they are never
// encoded back. Captured ROM records are separate values held by each caller.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Inputs {
    pub pose: u16,
    pub previous_pose: u16,
    pub movement_and_facing: u16,
    pub equipped_items: u16,
    pub equipped_beams: u16,
    pub shoot_button: u16,
    pub cancel_button: u16,
    pub switch_button: u16,
    pub run_button: u16,
    pub held: u16,
    pub pressed: u16,
    pub filtered_held: u16,
    pub filtered_pressed: u16,
    pub hyper: u16,
    pub frozen: u16,
    pub cinematic: u16,
    pub suit_palette: u16,
    pub contact_attack: u16,
    pub hud_auto_cancel: u16,
    pub speed_boost: u16,
    pub camera_x: u16,
    pub camera_y: u16,
    pub frame: u16,
    pub ceres: u16,
    pub m7_x: u16,
    pub m7_y: u16,
    pub m7_a: u16,
    pub m7_b: u16,
    pub m7_c: u16,
    pub grapple_x: u16,
    pub grapple_y: u16,
    pub y_acceleration: u16,
    pub y_fractional_acceleration: u16,
    pub absolute_y_fraction: u16,
    pub area: u16,
    pub room_width: u16,
    pub room_height: u16,
    pub room_scroll_width: u16,
    pub room_scroll_height: u16,
    pub room_bytes: u16,
    pub power_bomb_radius: u16,
}
