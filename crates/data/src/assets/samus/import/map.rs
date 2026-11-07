//! The pose map: which of the original's poses shows each pose as the player sees it.
//!
//! Nothing in the ROM names a pose by its parts, so the map is held here, once (the
//! samus-motion decode notes, "The pose map"). Of the original's 253 poses, 164 are one of
//! hers; the rest are another domain's or unused.

use crate::assets::samus::key::{
    Action, Aim, Body, Dress, Facing, Gun, LandedFrom, Legs, Pose, SpinSprite, Suit, Turn, Worn,
    shown_as,
};
use crate::pose::PoseId;

/// One pose of the original's and the pose it shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The original's pose.
    pub id: PoseId,
    /// The pose it shows.
    pub key: Pose,
    /// When it is the one shown.
    pub worn: Worn,
}

impl Entry {
    const fn drawn(mut self) -> Self {
        self.key.gun = Gun::Drawn;
        self
    }

    const fn striding(mut self) -> Self {
        self.key.legs = Legs::Striding;
        self
    }

    const fn worn(mut self, worn: Worn) -> Self {
        self.worn = worn;
        self
    }
}

/// A pose with its gun stowed and its legs together, shown whatever she wears.
const fn row(id: u16, body: Body, action: Action, facing: Facing, aim: Aim) -> Entry {
    Entry {
        id: PoseId(id),
        key: Pose {
            body,
            action,
            facing,
            aim,
            gun: Gun::Stowed,
            legs: Legs::Together,
        },
        worn: Worn::Always,
    }
}

/// Every pose of the original's that is one of hers, in its order.
#[rustfmt::skip]
pub const MAP: [Entry; 164] = {
    use {Action as A, Aim as M, Body as B, Facing as F};
    [
        row(0x00, B::Upright, A::Still, F::Screen, M::Forward).worn(Worn::Suit(Suit::First)),
        row(0x01, B::Upright, A::Still, F::Right, M::Forward),
        row(0x02, B::Upright, A::Still, F::Left, M::Forward),
        row(0x03, B::Upright, A::Still, F::Right, M::Up),
        row(0x04, B::Upright, A::Still, F::Left, M::Up),
        row(0x05, B::Upright, A::Still, F::Right, M::UpForward),
        row(0x06, B::Upright, A::Still, F::Left, M::UpForward),
        row(0x07, B::Upright, A::Still, F::Right, M::DownForward),
        row(0x08, B::Upright, A::Still, F::Left, M::DownForward),
        row(0x09, B::Upright, A::Running, F::Right, M::Forward),
        row(0x0A, B::Upright, A::Running, F::Left, M::Forward),
        row(0x0B, B::Upright, A::Running, F::Right, M::Forward).drawn(),
        row(0x0C, B::Upright, A::Running, F::Left, M::Forward).drawn(),
        row(0x0F, B::Upright, A::Running, F::Right, M::UpForward),
        row(0x10, B::Upright, A::Running, F::Left, M::UpForward),
        row(0x11, B::Upright, A::Running, F::Right, M::DownForward),
        row(0x12, B::Upright, A::Running, F::Left, M::DownForward),
        row(0x13, B::Upright, A::Jumping, F::Right, M::Forward).drawn(),
        row(0x14, B::Upright, A::Jumping, F::Left, M::Forward).drawn(),
        row(0x15, B::Upright, A::Jumping, F::Right, M::Up),
        row(0x16, B::Upright, A::Jumping, F::Left, M::Up),
        row(0x17, B::Upright, A::Jumping, F::Right, M::Down),
        row(0x18, B::Upright, A::Jumping, F::Left, M::Down),
        row(0x19, B::Upright, A::Spinning, F::Right, M::Forward).worn(Worn::Spin(SpinSprite::Plain)),
        row(0x1A, B::Upright, A::Spinning, F::Left, M::Forward).worn(Worn::Spin(SpinSprite::Plain)),
        row(0x1B, B::Upright, A::Spinning, F::Right, M::Forward).worn(Worn::Spin(SpinSprite::SpaceJump)),
        row(0x1C, B::Upright, A::Spinning, F::Left, M::Forward).worn(Worn::Spin(SpinSprite::SpaceJump)),
        row(0x1D, B::Ball, A::Still, F::Right, M::Forward),
        row(0x1E, B::Ball, A::Rolling, F::Right, M::Forward),
        row(0x1F, B::Ball, A::Rolling, F::Left, M::Forward),
        row(0x25, B::Upright, A::Turning(Turn::Ground), F::Left, M::Forward),
        row(0x26, B::Upright, A::Turning(Turn::Ground), F::Right, M::Forward),
        row(0x27, B::Crouched, A::Still, F::Right, M::Forward),
        row(0x28, B::Crouched, A::Still, F::Left, M::Forward),
        row(0x29, B::Upright, A::Falling, F::Right, M::Forward),
        row(0x2A, B::Upright, A::Falling, F::Left, M::Forward),
        row(0x2B, B::Upright, A::Falling, F::Right, M::Up),
        row(0x2C, B::Upright, A::Falling, F::Left, M::Up),
        row(0x2D, B::Upright, A::Falling, F::Right, M::Down),
        row(0x2E, B::Upright, A::Falling, F::Left, M::Down),
        row(0x2F, B::Upright, A::Turning(Turn::Jump), F::Left, M::Forward),
        row(0x30, B::Upright, A::Turning(Turn::Jump), F::Right, M::Forward),
        row(0x31, B::Ball, A::Falling, F::Right, M::Forward),
        row(0x32, B::Ball, A::Falling, F::Left, M::Forward),
        row(0x35, B::Crouched, A::Crouching, F::Right, M::Forward),
        row(0x36, B::Crouched, A::Crouching, F::Left, M::Forward),
        row(0x37, B::Ball, A::Morphing, F::Right, M::Forward),
        row(0x38, B::Ball, A::Morphing, F::Left, M::Forward),
        row(0x3B, B::Upright, A::StandingUp, F::Right, M::Forward),
        row(0x3C, B::Upright, A::StandingUp, F::Left, M::Forward),
        row(0x3D, B::Crouched, A::Unmorphing, F::Right, M::Forward),
        row(0x3E, B::Crouched, A::Unmorphing, F::Left, M::Forward),
        row(0x41, B::Ball, A::Still, F::Left, M::Forward),
        row(0x43, B::Crouched, A::Turning(Turn::Ground), F::Left, M::Forward),
        row(0x44, B::Crouched, A::Turning(Turn::Ground), F::Right, M::Forward),
        row(0x49, B::Upright, A::Moonwalking, F::Right, M::Forward),
        row(0x4A, B::Upright, A::Moonwalking, F::Left, M::Forward),
        row(0x4B, B::Upright, A::TakingOff, F::Right, M::Forward),
        row(0x4C, B::Upright, A::TakingOff, F::Left, M::Forward),
        row(0x4D, B::Upright, A::Jumping, F::Right, M::Forward),
        row(0x4E, B::Upright, A::Jumping, F::Left, M::Forward),
        row(0x51, B::Upright, A::Jumping, F::Right, M::Forward).striding(),
        row(0x52, B::Upright, A::Jumping, F::Left, M::Forward).striding(),
        row(0x55, B::Upright, A::TakingOff, F::Right, M::Up),
        row(0x56, B::Upright, A::TakingOff, F::Left, M::Up),
        row(0x57, B::Upright, A::TakingOff, F::Right, M::UpForward),
        row(0x58, B::Upright, A::TakingOff, F::Left, M::UpForward),
        row(0x59, B::Upright, A::TakingOff, F::Right, M::DownForward),
        row(0x5A, B::Upright, A::TakingOff, F::Left, M::DownForward),
        row(0x67, B::Upright, A::Falling, F::Right, M::Forward).drawn(),
        row(0x68, B::Upright, A::Falling, F::Left, M::Forward).drawn(),
        row(0x69, B::Upright, A::Jumping, F::Right, M::UpForward),
        row(0x6A, B::Upright, A::Jumping, F::Left, M::UpForward),
        row(0x6B, B::Upright, A::Jumping, F::Right, M::DownForward),
        row(0x6C, B::Upright, A::Jumping, F::Left, M::DownForward),
        row(0x6D, B::Upright, A::Falling, F::Right, M::UpForward),
        row(0x6E, B::Upright, A::Falling, F::Left, M::UpForward),
        row(0x6F, B::Upright, A::Falling, F::Right, M::DownForward),
        row(0x70, B::Upright, A::Falling, F::Left, M::DownForward),
        row(0x71, B::Crouched, A::Still, F::Right, M::UpForward),
        row(0x72, B::Crouched, A::Still, F::Left, M::UpForward),
        row(0x73, B::Crouched, A::Still, F::Right, M::DownForward),
        row(0x74, B::Crouched, A::Still, F::Left, M::DownForward),
        row(0x75, B::Upright, A::Moonwalking, F::Right, M::UpForward),
        row(0x76, B::Upright, A::Moonwalking, F::Left, M::UpForward),
        row(0x77, B::Upright, A::Moonwalking, F::Right, M::DownForward),
        row(0x78, B::Upright, A::Moonwalking, F::Left, M::DownForward),
        row(0x79, B::SpringBall, A::Still, F::Right, M::Forward),
        row(0x7A, B::SpringBall, A::Still, F::Left, M::Forward),
        row(0x7B, B::SpringBall, A::Rolling, F::Right, M::Forward),
        row(0x7C, B::SpringBall, A::Rolling, F::Left, M::Forward),
        row(0x7D, B::SpringBall, A::Falling, F::Right, M::Forward),
        row(0x7E, B::SpringBall, A::Falling, F::Left, M::Forward),
        row(0x7F, B::SpringBall, A::Jumping, F::Right, M::Forward),
        row(0x80, B::SpringBall, A::Jumping, F::Left, M::Forward),
        row(0x81, B::Upright, A::Spinning, F::Right, M::Forward).worn(Worn::Spin(SpinSprite::ScrewAttack)),
        row(0x82, B::Upright, A::Spinning, F::Left, M::Forward).worn(Worn::Spin(SpinSprite::ScrewAttack)),
        row(0x83, B::Upright, A::WallJumping, F::Right, M::Forward),
        row(0x84, B::Upright, A::WallJumping, F::Left, M::Forward),
        row(0x85, B::Crouched, A::Still, F::Right, M::Up),
        row(0x86, B::Crouched, A::Still, F::Left, M::Up),
        row(0x87, B::Upright, A::Turning(Turn::Fall), F::Left, M::Forward),
        row(0x88, B::Upright, A::Turning(Turn::Fall), F::Right, M::Forward),
        row(0x89, B::Upright, A::AgainstWall, F::Right, M::Forward),
        row(0x8A, B::Upright, A::AgainstWall, F::Left, M::Forward),
        row(0x8B, B::Upright, A::Turning(Turn::Ground), F::Left, M::Up),
        row(0x8C, B::Upright, A::Turning(Turn::Ground), F::Right, M::Up),
        row(0x8D, B::Upright, A::Turning(Turn::Ground), F::Left, M::DownForward),
        row(0x8E, B::Upright, A::Turning(Turn::Ground), F::Right, M::DownForward),
        row(0x8F, B::Upright, A::Turning(Turn::Jump), F::Left, M::Up),
        row(0x90, B::Upright, A::Turning(Turn::Jump), F::Right, M::Up),
        row(0x91, B::Upright, A::Turning(Turn::Jump), F::Left, M::Down),
        row(0x92, B::Upright, A::Turning(Turn::Jump), F::Right, M::Down),
        row(0x93, B::Upright, A::Turning(Turn::Fall), F::Left, M::Up),
        row(0x94, B::Upright, A::Turning(Turn::Fall), F::Right, M::Up),
        row(0x95, B::Upright, A::Turning(Turn::Fall), F::Left, M::Down),
        row(0x96, B::Upright, A::Turning(Turn::Fall), F::Right, M::Down),
        row(0x97, B::Crouched, A::Turning(Turn::Ground), F::Left, M::Up),
        row(0x98, B::Crouched, A::Turning(Turn::Ground), F::Right, M::Up),
        row(0x99, B::Crouched, A::Turning(Turn::Ground), F::Left, M::DownForward),
        row(0x9A, B::Crouched, A::Turning(Turn::Ground), F::Right, M::DownForward),
        row(0x9B, B::Upright, A::Still, F::Screen, M::Forward).worn(Worn::Suit(Suit::VariaOrGravity)),
        row(0x9C, B::Upright, A::Turning(Turn::Ground), F::Left, M::UpForward),
        row(0x9D, B::Upright, A::Turning(Turn::Ground), F::Right, M::UpForward),
        row(0x9E, B::Upright, A::Turning(Turn::Jump), F::Left, M::UpForward),
        row(0x9F, B::Upright, A::Turning(Turn::Jump), F::Right, M::UpForward),
        row(0xA0, B::Upright, A::Turning(Turn::Fall), F::Left, M::UpForward),
        row(0xA1, B::Upright, A::Turning(Turn::Fall), F::Right, M::UpForward),
        row(0xA2, B::Crouched, A::Turning(Turn::Ground), F::Left, M::UpForward),
        row(0xA3, B::Crouched, A::Turning(Turn::Ground), F::Right, M::UpForward),
        row(0xA4, B::Upright, A::Landing(LandedFrom::Jump), F::Right, M::Forward),
        row(0xA5, B::Upright, A::Landing(LandedFrom::Jump), F::Left, M::Forward),
        row(0xA6, B::Upright, A::Landing(LandedFrom::Spin), F::Right, M::Forward),
        row(0xA7, B::Upright, A::Landing(LandedFrom::Spin), F::Left, M::Forward),
        row(0xBF, B::Upright, A::Turning(Turn::MoonwalkJump), F::Left, M::Forward),
        row(0xC0, B::Upright, A::Turning(Turn::MoonwalkJump), F::Right, M::Forward),
        row(0xC1, B::Upright, A::Turning(Turn::MoonwalkJump), F::Left, M::UpForward),
        row(0xC2, B::Upright, A::Turning(Turn::MoonwalkJump), F::Right, M::UpForward),
        row(0xC3, B::Upright, A::Turning(Turn::MoonwalkJump), F::Left, M::DownForward),
        row(0xC4, B::Upright, A::Turning(Turn::MoonwalkJump), F::Right, M::DownForward),
        row(0xCF, B::Upright, A::AgainstWall, F::Right, M::UpForward),
        row(0xD0, B::Upright, A::AgainstWall, F::Left, M::UpForward),
        row(0xD1, B::Upright, A::AgainstWall, F::Right, M::DownForward),
        row(0xD2, B::Upright, A::AgainstWall, F::Left, M::DownForward),
        row(0xE0, B::Upright, A::Landing(LandedFrom::Jump), F::Right, M::Up),
        row(0xE1, B::Upright, A::Landing(LandedFrom::Jump), F::Left, M::Up),
        row(0xE2, B::Upright, A::Landing(LandedFrom::Jump), F::Right, M::UpForward),
        row(0xE3, B::Upright, A::Landing(LandedFrom::Jump), F::Left, M::UpForward),
        row(0xE4, B::Upright, A::Landing(LandedFrom::Jump), F::Right, M::DownForward),
        row(0xE5, B::Upright, A::Landing(LandedFrom::Jump), F::Left, M::DownForward),
        row(0xE6, B::Upright, A::Landing(LandedFrom::Jump), F::Right, M::Forward).drawn(),
        row(0xE7, B::Upright, A::Landing(LandedFrom::Jump), F::Left, M::Forward).drawn(),
        row(0xF1, B::Crouched, A::Crouching, F::Right, M::Up),
        row(0xF2, B::Crouched, A::Crouching, F::Left, M::Up),
        row(0xF3, B::Crouched, A::Crouching, F::Right, M::UpForward),
        row(0xF4, B::Crouched, A::Crouching, F::Left, M::UpForward),
        row(0xF5, B::Crouched, A::Crouching, F::Right, M::DownForward),
        row(0xF6, B::Crouched, A::Crouching, F::Left, M::DownForward),
        row(0xF7, B::Upright, A::StandingUp, F::Right, M::Up),
        row(0xF8, B::Upright, A::StandingUp, F::Left, M::Up),
        row(0xF9, B::Upright, A::StandingUp, F::Right, M::UpForward),
        row(0xFA, B::Upright, A::StandingUp, F::Left, M::UpForward),
        row(0xFB, B::Upright, A::StandingUp, F::Right, M::DownForward),
        row(0xFC, B::Upright, A::StandingUp, F::Left, M::DownForward),
    ]
};

/// The pose the original's `id` shows; `None` for the poses outside her domain.
#[must_use]
pub fn key_of(id: PoseId) -> Option<Pose> {
    MAP.iter()
        .find(|entry| entry.id == id)
        .map(|entry| entry.key)
}

/// The original's pose that shows `key` in `dress`, by its nearest sprite where it has none
/// of its own; `None` for a pose no sprite shows.
#[must_use]
pub fn id_of(key: Pose, dress: Dress) -> Option<PoseId> {
    let shown = shown_as(key);
    MAP.iter()
        .find(|entry| entry.key == shown && entry.worn.holds(dress))
        .map(|entry| entry.id)
}
