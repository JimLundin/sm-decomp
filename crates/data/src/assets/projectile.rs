//! Power-beam animation and cannon origins in the engine's own asset format.

use super::samus::key::Pose;
use super::samus::{FrameNumber, SpriteId, SpriteSheet};

/// A shot's heading, preserving facing for vertical aim.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Heading {
    /// Straight up while facing right.
    UpFacingRight,
    /// Up and right.
    UpRight,
    /// Right.
    Right,
    /// Down and right.
    DownRight,
    /// Straight down while facing right.
    DownFacingRight,
    /// Straight down while facing left.
    DownFacingLeft,
    /// Down and left.
    DownLeft,
    /// Left.
    Left,
    /// Up and left.
    UpLeft,
    /// Straight up while facing left.
    UpFacingLeft,
}

/// A complete heading map; its shape requires all ten choices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Headings<T> {
    /// Straight up, facing right.
    pub up_facing_right: T,
    /// Up and right.
    pub up_right: T,
    /// Right.
    pub right: T,
    /// Down and right.
    pub down_right: T,
    /// Straight down, facing right.
    pub down_facing_right: T,
    /// Straight down, facing left.
    pub down_facing_left: T,
    /// Down and left.
    pub down_left: T,
    /// Left.
    pub left: T,
    /// Up and left.
    pub up_left: T,
    /// Straight up, facing left.
    pub up_facing_left: T,
}

impl<T> Headings<T> {
    /// The record for this heading.
    #[must_use]
    pub const fn get(&self, heading: Heading) -> &T {
        match heading {
            Heading::UpFacingRight => &self.up_facing_right,
            Heading::UpRight => &self.up_right,
            Heading::Right => &self.right,
            Heading::DownRight => &self.down_right,
            Heading::DownFacingRight => &self.down_facing_right,
            Heading::DownFacingLeft => &self.down_facing_left,
            Heading::DownLeft => &self.down_left,
            Heading::Left => &self.left,
            Heading::UpLeft => &self.up_left,
            Heading::UpFacingLeft => &self.up_facing_left,
        }
    }
}

/// Nonnegative damage a power shot deals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Damage(u16);

impl TryFrom<u16> for Damage {
    type Error = u16;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value <= 32767 {
            Ok(Self(value))
        } else {
            Err(value)
        }
    }
}

impl Damage {
    /// Damage points, including zero.
    #[must_use]
    pub const fn points(self) -> u16 {
        self.0
    }
}

/// How many updates a picture holds, including zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hold(u16);

impl TryFrom<u16> for Hold {
    type Error = u16;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value <= 32767 {
            Ok(Self(value))
        } else {
            Err(value)
        }
    }
}

impl Hold {
    /// Updates the picture holds.
    #[must_use]
    pub const fn updates(self) -> u16 {
        self.0
    }
}

/// A picture's collision radii in pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Radius {
    /// Horizontal half-width.
    pub x: u8,
    /// Vertical half-height.
    pub y: u8,
}

/// A trail appearance in the power beam's pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TrailAppearanceId(pub u16);

/// One displayed picture of a beam clip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    /// How long the picture holds.
    pub hold: Hold,
    /// The picture in the sprite sheet.
    pub picture: SpriteId,
    /// Collision radii while this picture shows.
    pub radius: Radius,
    /// Appearance of a trail left while this picture shows.
    pub trail_appearance: TrailAppearanceId,
}

/// What follows a clip's last picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipEnd {
    /// Remove the shot.
    Remove,
    /// Continue at an existing frame.
    RepeatFrom(FrameNumber),
}

/// A nonempty animation sequence, checked by the importer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clip {
    /// Pictures in display order.
    pub frames: Vec<Frame>,
    /// What follows the last picture.
    pub end: ClipEnd,
}

/// A clip selected by named purpose.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClipId {
    /// The uncharged shot flying in this heading.
    Flight(Heading),
    /// The common block impact.
    Impact,
}

/// A signed pixel offset from Samus's centre.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Offset {
    /// Horizontal pixels, positive rightward.
    pub x: i16,
    /// Vertical pixels, positive downward.
    pub y: i16,
}

/// Where a pose aims its cannon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MuzzleAim {
    /// The pose forbids a shot origin.
    Forbidden,
    /// Resolve the previous pose's aimed heading.
    PreviousAim,
    /// The pose supplies this heading.
    Aimed(Heading),
}

/// A pose's cannon origin descriptor; selecting and resolving it belongs to the model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Muzzle {
    /// The pose as the player sees it.
    pub pose: Pose,
    /// The pose's aim decision.
    pub aim: MuzzleAim,
    /// Pixels above her centre before applying the heading offset.
    pub lift: u8,
    /// Offsets while stationary.
    pub stationary: Headings<Offset>,
    /// Offsets while moving.
    pub moving: Headings<Offset>,
}

/// The two trail origins relative to the shot centre.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrailOffsets {
    /// Left trail origin.
    pub left: Offset,
    /// Right trail origin.
    pub right: Offset,
}

/// A trail's appearance, with its original selector resolved into owned content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrailAppearance {
    /// An uncharged power-beam trail at these relative origins.
    PowerBeam(TrailOffsets),
    /// Impact pictures never leave a flying trail.
    Inactive,
}

/// All content required for uncharged power shots.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PowerBeam {
    /// The picture pool, using the shared sprite-sheet shape.
    pub sheet: SpriteSheet,
    /// Every flight heading's clip.
    pub flight: Headings<Clip>,
    /// The common finite impact clip.
    pub impact: Clip,
    /// Damage while flying.
    pub damage: Damage,
    /// The trail appearances frames name.
    pub trails: Vec<TrailAppearance>,
}

impl PowerBeam {
    /// The clip selected by purpose.
    #[must_use]
    pub const fn clip(&self, id: ClipId) -> &Clip {
        match id {
            ClipId::Flight(heading) => self.flight.get(heading),
            ClipId::Impact => &self.impact,
        }
    }
}

/// Power shots and the cannon origins of every named Samus pose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    /// Uncharged power-beam content.
    pub power: PowerBeam,
    /// Cannon origins keyed by pose, independent of dress.
    pub muzzles: Vec<Muzzle>,
}

impl Content {
    /// The origin descriptor of a named pose.
    #[must_use]
    pub fn muzzle(&self, pose: Pose) -> Option<&Muzzle> {
        self.muzzles.iter().find(|muzzle| muzzle.pose == pose)
    }
}

#[cfg(test)]
mod tests {
    use super::{Damage, Hold};

    #[test]
    fn damage_and_holds_accept_zero_and_the_signed_nonnegative_limit() {
        assert!(
            Damage::try_from(0).unwrap().points() == 0,
            "invented field equality"
        );
        assert!(
            Damage::try_from(32767).unwrap().points() == 32767,
            "invented field equality"
        );
        assert!(Damage::try_from(32768).is_err());
        assert!(
            Hold::try_from(0).unwrap().updates() == 0,
            "invented field equality"
        );
        assert!(
            Hold::try_from(32767).unwrap().updates() == 32767,
            "invented field equality"
        );
        assert!(Hold::try_from(32768).is_err());
    }
}
