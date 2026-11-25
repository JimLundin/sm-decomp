//! What the encounter reads from the other owners on a beat: the intro's return dissolve, and
//! what room play supplies once Samus has moved and weapons have stepped.

use types::Px;

/// Spec: `ReturnToPage`. Where the intro's return dissolve stands on this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnDissolve {
    /// It hasn't reached zero on this frame.
    Continuing,
    /// It reaches zero on this frame.
    ReachesZero,
}

/// A shot's identity, as weapons names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShotId(pub u8);

/// The kind of a flying shot. Only ordinary missiles meet Mother Brain; the others are passed
/// and ignored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShotKind {
    /// An ordinary missile.
    Missile,
    /// A super missile.
    SuperMissile,
    /// A beam.
    Beam,
}

/// A flying shot weapons offers: its identity, its kind and the whole room X of its centre.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShotCandidate {
    /// Its identity.
    pub id: ShotId,
    /// Its kind.
    pub kind: ShotKind,
    /// Its centre's whole room X.
    pub x: Px,
}

/// What room play supplies on a beat.
///
/// Samus's whole X after movement, and the flying shots after weapons step, in the weapons
/// owner's precedence order, highest first. Once Samus is hidden, her last whole X is still
/// supplied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomPlay {
    /// Samus's whole room X.
    pub samus_x: Px,
    /// The flying shots, highest precedence first.
    pub shots: Vec<ShotCandidate>,
}

impl RoomPlay {
    /// Spec: `BrainEndures`. The first ordinary missile in precedence order, the only one that
    /// can hit this beat.
    #[must_use]
    pub fn first_missile(&self) -> Option<&ShotCandidate> {
        self.shots
            .iter()
            .find(|shot| shot.kind == ShotKind::Missile)
    }
}
