//! What a Samus asset is keyed by: her pose as the player sees it (the samus-movement spec's §2
//! Pose, `types::Pose`), and what she wears.

pub use types::{Action, Aim, Body, Facing, Gun, LandedFrom, Legs, Pose, Turn};

/// The pose whose sprite shows `pose`. The air turns have one sprite for down-forward and
/// down, the down one; the crouched turns the same sprite, which is down-forward.
#[must_use]
pub fn shown_as(pose: Pose) -> Pose {
    let turning = matches!(pose.action, Action::Turning(_));
    let in_the_air = matches!(pose.action, Action::Turning(Turn::Jump | Turn::Fall));
    let aim = match pose.aim {
        Aim::DownForward if in_the_air => Aim::Down,
        Aim::Down if turning && pose.body == Body::Crouched => Aim::DownForward,
        Aim::Down | Aim::DownForward | Aim::Forward | Aim::Up | Aim::UpForward => pose.aim,
    };
    Pose { aim, ..pose }
}

/// How her spin is drawn: by her equipment, and always plain in a liquid without the gravity
/// suit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpinSprite {
    /// Neither the space jump nor the screw attack draws it.
    Plain,
    /// The space jump, without the screw attack.
    SpaceJump,
    /// The screw attack.
    ScrewAttack,
}

/// The suit she is drawn in facing the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Suit {
    /// Her first suit.
    First,
    /// Varia or gravity.
    VariaOrGravity,
}

/// What the sprite shows that her pose doesn't decide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Dress {
    /// How her spin is drawn.
    pub spin: SpinSprite,
    /// Her suit, facing the screen.
    pub suit: Suit,
}

/// When an asset is the one shown for its pose.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Worn {
    /// Whatever she wears.
    Always,
    /// With this spin's sprite.
    Spin(SpinSprite),
    /// In this suit.
    Suit(Suit),
}

impl Worn {
    /// Whether the asset shows in `dress`.
    #[must_use]
    pub fn holds(self, dress: Dress) -> bool {
        match self {
            Self::Always => true,
            Self::Spin(spin) => spin == dress.spin,
            Self::Suit(suit) => suit == dress.suit,
        }
    }
}
