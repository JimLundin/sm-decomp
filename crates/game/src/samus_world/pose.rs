//! The engine's pose id and the samus-motion spec's §2 Pose, both ways: the decode notes' pose
//! map (`docs/behaviours/samus-motion.decode.md`, "The pose map"), as data.
//!
//! The Pose here is §2's, with the parts the clean model's `clean_types::Pose` doesn't have
//! yet (the landing, the turn's kind, the gun and the legs); it is this module's until the
//! model's carries them. Of the 253 ids, 164 are a Pose; the rest are another domain's or
//! unused, and have none.

use clean_types::{Aim, Body, Facing};
use data::pose::PoseId;

/// What she is doing (§2), with the part only that action has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Pressed to the wall she ran into.
    AgainstWall,
    /// Crouching down.
    Crouching,
    /// Falling.
    Falling,
    /// A straight jump, or the spring ball's hop.
    Jumping,
    /// Touched down, from what.
    Landing(LandedFrom),
    /// Moonwalking.
    Moonwalking,
    /// Curling into the ball.
    Morphing,
    /// Rolling as a ball.
    Rolling,
    /// Running.
    Running,
    /// A spin jump.
    Spinning,
    /// Standing up from a crouch.
    StandingUp,
    /// At rest.
    Still,
    /// The first poses of a straight jump.
    TakingOff,
    /// Turning round, in this turn.
    Turning(TurnKind),
    /// Uncurling from the ball.
    Unmorphing,
    /// A wall jump.
    WallJumping,
}

/// What a landing touched down from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LandedFrom {
    /// A straight jump or a fall, upright.
    Jump,
    /// A spin or a wall jump.
    Spin,
}

/// The turn she is in: what she was doing when she turned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnKind {
    /// On the ground, upright or crouched.
    Ground,
    /// In a take-off or a straight jump.
    Jump,
    /// In a fall.
    Fall,
    /// The moonwalk's jump, still on the ground.
    MoonwalkJump,
}

/// Her gun.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Gun {
    /// Not held out.
    #[default]
    Stowed,
    /// Held out in front.
    Drawn,
}

/// Her legs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Legs {
    /// Together.
    #[default]
    Together,
    /// Drawn moving forward.
    Striding,
}

/// Her pose as the player sees it (§2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pose {
    /// Her body's shape.
    pub body: Body,
    /// What she is doing.
    pub action: Action,
    /// Which way she faces: a turn's new facing, a moonwalk's the way she walks.
    pub facing: Facing,
    /// The arm's direction.
    pub aim: Aim,
    /// Her gun.
    pub gun: Gun,
    /// Her legs.
    pub legs: Legs,
}

/// How her spin is drawn: by her equipment, and always plain in a liquid without the gravity
/// suit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpinSprite {
    /// Neither the space jump nor the screw attack draws it.
    Plain,
    /// The space jump, without the screw attack.
    SpaceJump,
    /// The screw attack.
    ScrewAttack,
}

/// The suit she is drawn in facing the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suit {
    /// Her first suit.
    First,
    /// Varia or gravity.
    VariaOrGravity,
}

/// What the sprite shows that her Pose doesn't decide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dress {
    /// How her spin is drawn.
    pub spin: SpinSprite,
    /// Her suit, facing the screen.
    pub suit: Suit,
}

/// When an id shows its Pose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shown {
    /// Whatever she wears.
    Always,
    /// With this spin's sprite.
    Spin(SpinSprite),
    /// In this suit.
    Suit(Suit),
}

impl Shown {
    fn holds(self, dress: Dress) -> bool {
        match self {
            Self::Always => true,
            Self::Spin(spin) => spin == dress.spin,
            Self::Suit(suit) => suit == dress.suit,
        }
    }
}

/// One id and the Pose it shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Entry {
    id: PoseId,
    pose: Pose,
    shown: Shown,
}

impl Entry {
    const fn drawn(mut self) -> Self {
        self.pose.gun = Gun::Drawn;
        self
    }

    const fn striding(mut self) -> Self {
        self.pose.legs = Legs::Striding;
        self
    }

    const fn shown(mut self, shown: Shown) -> Self {
        self.shown = shown;
        self
    }
}

/// An id whose Pose has its gun stowed and its legs together, shown whatever she wears.
const fn row(id: u16, body: Body, action: Action, facing: Facing, aim: Aim) -> Entry {
    Entry {
        id: PoseId(id),
        pose: Pose {
            body,
            action,
            facing,
            aim,
            gun: Gun::Stowed,
            legs: Legs::Together,
        },
        shown: Shown::Always,
    }
}

/// Every id that is a Pose, in id order.
#[rustfmt::skip]
const MAP: [Entry; 164] = {
    use {Action as A, Aim as M, Body as B, Facing as F};
    [
        row(0x00, B::Upright, A::Still, F::Screen, M::Forward).shown(Shown::Suit(Suit::First)),
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
        row(0x19, B::Upright, A::Spinning, F::Right, M::Forward).shown(Shown::Spin(SpinSprite::Plain)),
        row(0x1A, B::Upright, A::Spinning, F::Left, M::Forward).shown(Shown::Spin(SpinSprite::Plain)),
        row(0x1B, B::Upright, A::Spinning, F::Right, M::Forward).shown(Shown::Spin(SpinSprite::SpaceJump)),
        row(0x1C, B::Upright, A::Spinning, F::Left, M::Forward).shown(Shown::Spin(SpinSprite::SpaceJump)),
        row(0x1D, B::Ball, A::Still, F::Right, M::Forward),
        row(0x1E, B::Ball, A::Rolling, F::Right, M::Forward),
        row(0x1F, B::Ball, A::Rolling, F::Left, M::Forward),
        row(0x25, B::Upright, A::Turning(TurnKind::Ground), F::Left, M::Forward),
        row(0x26, B::Upright, A::Turning(TurnKind::Ground), F::Right, M::Forward),
        row(0x27, B::Crouched, A::Still, F::Right, M::Forward),
        row(0x28, B::Crouched, A::Still, F::Left, M::Forward),
        row(0x29, B::Upright, A::Falling, F::Right, M::Forward),
        row(0x2A, B::Upright, A::Falling, F::Left, M::Forward),
        row(0x2B, B::Upright, A::Falling, F::Right, M::Up),
        row(0x2C, B::Upright, A::Falling, F::Left, M::Up),
        row(0x2D, B::Upright, A::Falling, F::Right, M::Down),
        row(0x2E, B::Upright, A::Falling, F::Left, M::Down),
        row(0x2F, B::Upright, A::Turning(TurnKind::Jump), F::Left, M::Forward),
        row(0x30, B::Upright, A::Turning(TurnKind::Jump), F::Right, M::Forward),
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
        row(0x43, B::Crouched, A::Turning(TurnKind::Ground), F::Left, M::Forward),
        row(0x44, B::Crouched, A::Turning(TurnKind::Ground), F::Right, M::Forward),
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
        row(0x81, B::Upright, A::Spinning, F::Right, M::Forward).shown(Shown::Spin(SpinSprite::ScrewAttack)),
        row(0x82, B::Upright, A::Spinning, F::Left, M::Forward).shown(Shown::Spin(SpinSprite::ScrewAttack)),
        row(0x83, B::Upright, A::WallJumping, F::Right, M::Forward),
        row(0x84, B::Upright, A::WallJumping, F::Left, M::Forward),
        row(0x85, B::Crouched, A::Still, F::Right, M::Up),
        row(0x86, B::Crouched, A::Still, F::Left, M::Up),
        row(0x87, B::Upright, A::Turning(TurnKind::Fall), F::Left, M::Forward),
        row(0x88, B::Upright, A::Turning(TurnKind::Fall), F::Right, M::Forward),
        row(0x89, B::Upright, A::AgainstWall, F::Right, M::Forward),
        row(0x8A, B::Upright, A::AgainstWall, F::Left, M::Forward),
        row(0x8B, B::Upright, A::Turning(TurnKind::Ground), F::Left, M::Up),
        row(0x8C, B::Upright, A::Turning(TurnKind::Ground), F::Right, M::Up),
        row(0x8D, B::Upright, A::Turning(TurnKind::Ground), F::Left, M::DownForward),
        row(0x8E, B::Upright, A::Turning(TurnKind::Ground), F::Right, M::DownForward),
        row(0x8F, B::Upright, A::Turning(TurnKind::Jump), F::Left, M::Up),
        row(0x90, B::Upright, A::Turning(TurnKind::Jump), F::Right, M::Up),
        row(0x91, B::Upright, A::Turning(TurnKind::Jump), F::Left, M::Down),
        row(0x92, B::Upright, A::Turning(TurnKind::Jump), F::Right, M::Down),
        row(0x93, B::Upright, A::Turning(TurnKind::Fall), F::Left, M::Up),
        row(0x94, B::Upright, A::Turning(TurnKind::Fall), F::Right, M::Up),
        row(0x95, B::Upright, A::Turning(TurnKind::Fall), F::Left, M::Down),
        row(0x96, B::Upright, A::Turning(TurnKind::Fall), F::Right, M::Down),
        row(0x97, B::Crouched, A::Turning(TurnKind::Ground), F::Left, M::Up),
        row(0x98, B::Crouched, A::Turning(TurnKind::Ground), F::Right, M::Up),
        row(0x99, B::Crouched, A::Turning(TurnKind::Ground), F::Left, M::DownForward),
        row(0x9A, B::Crouched, A::Turning(TurnKind::Ground), F::Right, M::DownForward),
        row(0x9B, B::Upright, A::Still, F::Screen, M::Forward).shown(Shown::Suit(Suit::VariaOrGravity)),
        row(0x9C, B::Upright, A::Turning(TurnKind::Ground), F::Left, M::UpForward),
        row(0x9D, B::Upright, A::Turning(TurnKind::Ground), F::Right, M::UpForward),
        row(0x9E, B::Upright, A::Turning(TurnKind::Jump), F::Left, M::UpForward),
        row(0x9F, B::Upright, A::Turning(TurnKind::Jump), F::Right, M::UpForward),
        row(0xA0, B::Upright, A::Turning(TurnKind::Fall), F::Left, M::UpForward),
        row(0xA1, B::Upright, A::Turning(TurnKind::Fall), F::Right, M::UpForward),
        row(0xA2, B::Crouched, A::Turning(TurnKind::Ground), F::Left, M::UpForward),
        row(0xA3, B::Crouched, A::Turning(TurnKind::Ground), F::Right, M::UpForward),
        row(0xA4, B::Upright, A::Landing(LandedFrom::Jump), F::Right, M::Forward),
        row(0xA5, B::Upright, A::Landing(LandedFrom::Jump), F::Left, M::Forward),
        row(0xA6, B::Upright, A::Landing(LandedFrom::Spin), F::Right, M::Forward),
        row(0xA7, B::Upright, A::Landing(LandedFrom::Spin), F::Left, M::Forward),
        row(0xBF, B::Upright, A::Turning(TurnKind::MoonwalkJump), F::Left, M::Forward),
        row(0xC0, B::Upright, A::Turning(TurnKind::MoonwalkJump), F::Right, M::Forward),
        row(0xC1, B::Upright, A::Turning(TurnKind::MoonwalkJump), F::Left, M::UpForward),
        row(0xC2, B::Upright, A::Turning(TurnKind::MoonwalkJump), F::Right, M::UpForward),
        row(0xC3, B::Upright, A::Turning(TurnKind::MoonwalkJump), F::Left, M::DownForward),
        row(0xC4, B::Upright, A::Turning(TurnKind::MoonwalkJump), F::Right, M::DownForward),
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

/// The Pose an id shows; `None` for the ids outside the domain.
#[must_use]
pub fn pose_of(id: PoseId) -> Option<Pose> {
    MAP.iter()
        .find(|entry| entry.id == id)
        .map(|entry| entry.pose)
}

/// The id that shows `pose` in `dress`.
///
/// A Pose the spec never produces is shown by its nearest
/// sprite: an air turn aimed down-forward as aimed down, a crouched turn aimed down as aimed
/// down-forward. `None` for a Pose no sprite shows.
#[must_use]
pub fn id_of(pose: Pose, dress: Dress) -> Option<PoseId> {
    let nearest = Pose {
        aim: nearest_aim(pose),
        ..pose
    };
    MAP.iter()
        .find(|entry| entry.pose == nearest && entry.shown.holds(dress))
        .map(|entry| entry.id)
}

/// The turns' shared sprites: in the air one sprite for down-forward and down, which is down;
/// crouched, the same sprite, which is down-forward.
fn nearest_aim(pose: Pose) -> Aim {
    let in_the_air = matches!(
        pose.action,
        Action::Turning(TurnKind::Jump | TurnKind::Fall)
    );
    let crouched = pose.body == Body::Crouched && matches!(pose.action, Action::Turning(_));
    if in_the_air && pose.aim == Aim::DownForward {
        Aim::Down
    } else if crouched && pose.aim == Aim::Down {
        Aim::DownForward
    } else {
        pose.aim
    }
}

#[cfg(test)]
mod tests;
