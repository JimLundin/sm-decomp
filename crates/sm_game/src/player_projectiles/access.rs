//! Direct ordinary shot access, with a typed exceptional-selector edge.
#![forbid(unsafe_code)]
#[derive(Clone, Copy)]
pub enum ShotLane {
    X,
    Y,
    XFraction,
    YFraction,
    XRadius,
    YRadius,
    XSpeed,
    YSpeed,
    Direction,
    Kind,
    Damage,
    Cursor,
    Remaining,
    Motion,
    Variable,
    TrailTime,
    Auxiliary,
    Sprite,
}
#[derive(Clone, Copy)]
pub enum Store {
    PrefixLe,
    SliceLe,
    SliceNative,
}
#[macro_export]
macro_rules! shot_lane {
    (x) => {
        $crate::player_projectiles::access::ShotLane::X
    };
    (y) => {
        $crate::player_projectiles::access::ShotLane::Y
    };
    (x_fraction) => {
        $crate::player_projectiles::access::ShotLane::XFraction
    };
    (y_fraction) => {
        $crate::player_projectiles::access::ShotLane::YFraction
    };
    (x_radius) => {
        $crate::player_projectiles::access::ShotLane::XRadius
    };
    (y_radius) => {
        $crate::player_projectiles::access::ShotLane::YRadius
    };
    (x_speed) => {
        $crate::player_projectiles::access::ShotLane::XSpeed
    };
    (y_speed) => {
        $crate::player_projectiles::access::ShotLane::YSpeed
    };
    (direction) => {
        $crate::player_projectiles::access::ShotLane::Direction
    };
    (kind) => {
        $crate::player_projectiles::access::ShotLane::Kind
    };
    (damage) => {
        $crate::player_projectiles::access::ShotLane::Damage
    };
    (cursor) => {
        $crate::player_projectiles::access::ShotLane::Cursor
    };
    (remaining) => {
        $crate::player_projectiles::access::ShotLane::Remaining
    };
    (motion) => {
        $crate::player_projectiles::access::ShotLane::Motion
    };
    (variable) => {
        $crate::player_projectiles::access::ShotLane::Variable
    };
    (trail_time) => {
        $crate::player_projectiles::access::ShotLane::TrailTime
    };
    (auxiliary) => {
        $crate::player_projectiles::access::ShotLane::Auxiliary
    };
    (sprite) => {
        $crate::player_projectiles::access::ShotLane::Sprite
    };
}
#[macro_export]
macro_rules! shot {
    ($h:ident, $raw:expr, $field:ident) => {{
        let raw: u16 = $raw;
        if let Some(shot) = $h.session().projectiles.shots.get(usize::from(raw >> 1)) {
            shot.$field
        } else {
            $h.alias_shot_read($crate::shot_lane!($field), raw)
        }
    }};
    ($h:ident, $raw:expr, $field:ident = $value:expr) => {
        $crate::shot!($h, $raw, $field = $value; PrefixLe)
    };
    ($h:ident, $raw:expr, $field:ident = $value:expr; $store:ident) => {{
        let value: u16 = $value;
        let raw: u16 = $raw;
        if let Some(shot) = $h.session_mut().projectiles.shots.get_mut(usize::from(raw >> 1)) {
            shot.$field = match $crate::player_projectiles::access::Store::$store {
                $crate::player_projectiles::access::Store::SliceNative => u16::from_le_bytes(value.to_ne_bytes()),
                _ => value,
            };
        } else {
            $h.alias_shot_write($crate::shot_lane!($field), raw, value, $crate::player_projectiles::access::Store::$store);
        }
    }};
}
#[macro_export]
macro_rules! shot_native {
    ($h:ident, $raw:expr, $field:ident) => {
        u16::from_ne_bytes($crate::shot!($h, $raw, $field).to_le_bytes())
    };
}

pub use crate::{shot, shot_lane};

pub fn room_level<H: super::host::Host>(h: &H, cell: u16) -> u16 {
    h.session()
        .room
        .terrain
        .cells()
        .get(usize::from(cell))
        .map_or_else(|| h.room_level(cell), |cell| cell.level)
}
pub fn room_bts<H: super::host::Host>(h: &H, cell: u16) -> u8 {
    h.session()
        .room
        .terrain
        .cells()
        .get(usize::from(cell))
        .map_or_else(|| h.room_bts(cell), |cell| cell.behavior)
}
