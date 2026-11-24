//! The two looks the pad and the platform each take in turn.

/// One of two looks shown in turn, the pad's and the platform's alike.
///
/// The pad's look is which of its two figures it shows (`PadFlickers`); the platform's look is
/// which of its two pictures shows (`PlatformTurns`). Which sprites or picture each is belongs
/// to the shaft's content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlternatingLook {
    /// The first look.
    First,
    /// The second look.
    Second,
}

impl AlternatingLook {
    /// The look the turn moves to: the other one.
    #[must_use]
    pub const fn other(self) -> Self {
        match self {
            Self::First => Self::Second,
            Self::Second => Self::First,
        }
    }
}
