//! Representation-only payload. Not registered, compiled, or executed.
//! No secondary Session and no permanent mutable RAM mirror.
#![forbid(unsafe_code)]

// Codec construction preserves all 16 bits. Classification is an ephemeral view:
// raw & 0x8000 == 0 => Pending(PoseId(raw)), otherwise Inactive(raw).
// Do not canonicalize 0x8000..=0xffff to a single None or 0xffff.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoseRequestWord(u16);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoseId(u16);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestView {
    Pending(PoseId),
    Inactive { encoded: u16 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestChannel {
    Ordinary,
    Interrupted,
    Transitional,
}

// Selector wrappers preserve invalid values until their original dispatch point.
// They are different types: no accidental momentum/special/hurt substitution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MomentumAction(u16);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InterruptionAction(u16);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HurtAction(u16);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationSkip(u16);

// Replace the SAME Player.movement_handler field, not a new transitions copy.
// Only compatibility codec maps known modes to legacy selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovementMode {
    Normal,
    DrainedCrouching,
    Knockback,
    BombJumpStart,
    Xray,
    ShinesparkWindup,
    ShinesparkHorizontal,
    ShinesparkVertical,
    ShinesparkDiagonal,
    ShinesparkProjectileResponse,
    Unknown(u16),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Demo,
    AutoJump,
    Xray,
    Disabled,
    Unknown(u16),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameAlphaMode {
    Empty,
    Unknown(u16),
}

// One word each; low and high byte operations are computed views.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BombJump(u16);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LandingBounce(u16);

#[derive(Clone, Default, Debug, PartialEq)]
pub struct PlayerTransitions {
    // Rehomed 14 existing encoded bytes; remove from Preparation.
    pub ordinary: PoseRequestWord,
    pub interrupted: PoseRequestWord,
    pub transitional: PoseRequestWord,
    pub momentum: MomentumAction,
    pub interruption: InterruptionAction,
    pub hurt: HurtAction,
    pub animation_skip: AnimationSkip,
    // 16 newly owned words = 32 encoded bytes.
    pub landing_compatibility: u16,
    pub frame_alpha: FrameAlphaMode,
    pub frame_beta: u16,
    pub draw_handler: u16,
    pub knockback: u16,
    pub bomb_jump: BombJump,
    pub input_mode: InputMode,
    pub xray_angle: u16,
    pub animation_buffer: u16,
    pub grapple_walljump_timer: u16,
    pub elevator_fade_timer: u16,
    pub shinespark_start_stop_timer: u16,
    pub liquid_physics: u16,
    pub landing_motion_compatibility: u16,
    pub bounce: LandingBounce,
    pub falling: u16,
    pub vertical_motion_compatibility: u16,
    pub vertical_auxiliary_compatibility: u16,
}

// Session presentation/effects owns these, separate from 32-slot SpriteEffects.
// One slot's fields are noncontiguous in compatibility encoding.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct AtmosphericSlot {
    pub remaining: u16,
    pub x: u16,
    pub y: u16,
    pub frame_and_kind: u16,
}
#[derive(Clone, Default, Debug, PartialEq)]
pub struct AtmosphericEffects {
    pub slots: [AtmosphericSlot; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationStep {
    Gameplay,
    Death,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JumpKind {
    Normal,
    Wall,
    Knockback,
    Bomb,
}

// Read-only refreshed input additions, never authoritative owned copies.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct TransitionObservations {
    pub frame_gamma: u16,
    pub debug_invincibility: u16,
    pub moonwalk_enabled: u16,
    pub turn_request: u16,
    pub game_state: u16,
    pub game_time_frames: u16,
    pub fx_kind: u16,
}

// Lossless compatibility conversions are representation, not gameplay dispatch.
macro_rules! raw_word {
    ($($name:ident),+ $(,)?) => { $(
        impl $name {
            pub const fn from_raw(raw: u16) -> Self { Self(raw) }
            pub const fn to_raw(self) -> u16 { self.0 }
        }
        impl Default for $name {
            fn default() -> Self { Self::from_raw(0) }
        }
    )+};
}
raw_word!(
    PoseRequestWord,
    PoseId,
    MomentumAction,
    InterruptionAction,
    HurtAction,
    AnimationSkip,
    BombJump,
    LandingBounce
);
impl PoseRequestWord {
    pub const fn view(self) -> RequestView {
        if self.0 & 0x8000 == 0 {
            RequestView::Pending(PoseId::from_raw(self.0))
        } else {
            RequestView::Inactive { encoded: self.0 }
        }
    }
}
macro_rules! mode_word {
    ($name:ident { $($variant:ident = $raw:literal),+ $(,)? }) => {
        impl $name {
            pub const fn from_raw(raw: u16) -> Self {
                match raw { $($raw => Self::$variant,)+ other => Self::Unknown(other) }
            }
            pub const fn to_raw(self) -> u16 {
                match self { $(Self::$variant => $raw,)+ Self::Unknown(raw) => raw }
            }
        }
        impl Default for $name {
            fn default() -> Self { Self::Unknown(0) }
        }
    };
}
mode_word!(MovementMode {
    Normal = 0xa337,
    DrainedCrouching = 0x94cb,
    Knockback = 0xdf38,
    BombJumpStart = 0xe025,
    Xray = 0xe94f,
    ShinesparkWindup = 0xd068,
    ShinesparkHorizontal = 0xd106,
    ShinesparkVertical = 0xd0ab,
    ShinesparkDiagonal = 0xd0d7,
    ShinesparkProjectileResponse = 0xf04b,
});
mode_word!(InputMode {
    Normal = 0xe913, Demo = 0xe91d, AutoJump = 0xe926,
    Xray = 0xe918, Disabled = 0xe90e,
});
mode_word!(FrameAlphaMode { Empty = 0xe8cd });
