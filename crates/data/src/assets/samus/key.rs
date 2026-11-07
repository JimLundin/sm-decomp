//! What a Samus asset is keyed by: her pose as the player sees it, and what she wears.
//!
//! This mirrors the samus-motion spec's §2 Pose; it gives way to the clean room's
//! `clean_types::Pose` when that carries every part.

/// Her body's shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Body {
    /// Standing height.
    Upright,
    /// Crouched.
    Crouched,
    /// The morph ball.
    Ball,
    /// The spring ball.
    SpringBall,
}

/// Which way she faces: a turn's new facing, a moonwalk's the way she walks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Facing {
    /// Facing left.
    Left,
    /// Facing right.
    Right,
    /// Facing the player.
    Screen,
}

/// The arm's direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Aim {
    /// Aimed down.
    Down,
    /// Aimed down and forward.
    DownForward,
    /// Unaimed.
    Forward,
    /// Aimed up.
    Up,
    /// Aimed up and forward.
    UpForward,
}

/// What she is doing, with the part only that action has.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LandedFrom {
    /// A straight jump or a fall, upright.
    Jump,
    /// A spin or a wall jump.
    Spin,
}

/// The turn she is in: what she was doing when she turned.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Gun {
    /// Not held out.
    #[default]
    Stowed,
    /// Held out in front.
    Drawn,
}

/// Her legs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Legs {
    /// Together.
    #[default]
    Together,
    /// Drawn moving forward.
    Striding,
}

/// Her pose as the player sees it: what a Samus asset is keyed by.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PoseKey {
    /// Her body's shape.
    pub body: Body,
    /// What she is doing.
    pub action: Action,
    /// Which way she faces.
    pub facing: Facing,
    /// The arm's direction.
    pub aim: Aim,
    /// Her gun.
    pub gun: Gun,
    /// Her legs.
    pub legs: Legs,
}

impl PoseKey {
    /// The pose whose sprite shows this one. The air turns have one sprite for down-forward
    /// and down, the down one; the crouched turns the same sprite, which is down-forward.
    #[must_use]
    pub fn shown_as(self) -> Self {
        let turning = matches!(self.action, Action::Turning(_));
        let in_the_air = matches!(
            self.action,
            Action::Turning(TurnKind::Jump | TurnKind::Fall)
        );
        let aim = match self.aim {
            Aim::DownForward if in_the_air => Aim::Down,
            Aim::Down if turning && self.body == Body::Crouched => Aim::DownForward,
            Aim::Down | Aim::DownForward | Aim::Forward | Aim::Up | Aim::UpForward => self.aim,
        };
        Self { aim, ..self }
    }
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
