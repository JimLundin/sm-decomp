//! Player collision requests and complete owned operations.
#![forbid(unsafe_code)]

use crate::actors::access::ActorKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
}

/// These select the controller's source direction, before speed calculation.
/// The final signed displacement can select a different movement direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HorizontalDirection {
    Left,
    Right,
    /// Read current facing and acceleration mode at the original entry point.
    FromFacingAndAcceleration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HorizontalRequest {
    /// Base displacement already produced by the retained speed-table selector.
    /// This is not a snapshot of player speed, position or external displacement.
    pub base: i32,
    pub direction: HorizontalDirection,
}

/// Selected by the original controller predicates, including spin's pair.flag,
/// facing and opposite-key branches. This is not inferred from held == 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HorizontalTransition {
    Move(HorizontalRequest),
    /// Clear base speed/fraction and collision; perform no X movement.
    ClearAndSkip,
    /// Same clears, then execute the full horizontal operation with base zero.
    /// External displacement can still move the player.
    ClearAndMoveZero,
    /// Spin's distinct branch also clears acceleration mode before MoveX(0).
    ClearAccelerationAndMoveZero,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalDirection {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// Distinct admission and publication policies; no mode silently adds actor
/// checks to terrain-only or unchecked callers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplacementRequest {
    /// Corresponds to the old MoveUp/MoveDown entry and retains its signed
    /// source argument. Direction is explicit, not re-inferred after callbacks.
    Vertical {
        direction: VerticalDirection,
        amount: i32,
    },
    /// Already signed terrain displacement, including standing/room scripts.
    TerrainOnly { axis: Axis, amount: i32 },
    /// Retained pose-transition clearance probe; preserves scratch effects but
    /// commits neither position nor LaunchMotion. Caller consumes source result.
    PoseClearanceProbe { amount: i32 },
    /// Old direction-specific NoColl argument convention; commit and record
    /// LaunchMotion without testing terrain or publishing a fabricated contact.
    Unchecked { direction: Direction, amount: i32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JumpKind {
    Wall,
    Grapple,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JumpRequest {
    pub kind: JumpKind,
    pub amount: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Obstruction {
    Clear,
    SolidActor(ActorKey),
    Terrain,
    /// No contact test was requested; existing flags are not necessarily clear.
    Unchecked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MotionResult {
    /// Source-defined permitted displacement; never reconstructed by subtracting
    /// wrapped coordinates. Slope/pose alignment is a separate state publication.
    pub displacement: i32,
    pub obstruction: Obstruction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlResult {
    Skipped,
    Moved(MotionResult),
}

/// 1. Complete speed -> actor -> terrain -> commit/launch -> slope operation.
/// Read mutable player fields through the single Session owner. Capture the
/// caller's base/direction before borrowing its Runtime. Release every ActorRead
/// and Player borrow before an effect; reacquire at the source's observation point.
/// Preserve boost cancellation before subsequent speed clears and flag encodings.
pub use super::motion::horizontal;

/// 2. Apply one already-selected controller transition. Skip, zero-base motion
/// and acceleration-clear-plus-zero-base motion are observably different.
pub use super::control::horizontal_control;

/// 3. Execute the requested vertical/terrain/unchecked policy. Preserve fraction
/// store order, original argument polarity, parity-dependent scan direction,
/// and permitted displacement across a synchronous mechanism call.
pub use super::motion::displace;

/// 4. Actual wall/grapple admission, including the not-ready animation writes,
/// first solid-actor identity and input-to-pose/shake publication. Read refreshed
/// jump binding and pressed keys AFTER effects; reread pose before animation hold.
pub use super::jump::jump_admission;

/// 5. Resolve pending walking pose against actor/terrain obstruction and publish
/// the next pose/solid-contact marker. Pending/current pose remain live Session data.
pub use super::pose::walked_obstacle;

/// 6. Fit the current pose using the true previous pose, both actor/terrain sides,
/// temporary radius and the second probe. Publish rejection/adjustment before the
/// retained caller refreshes pose and executes jump-transition logic.
pub use super::pose::fit_pose;

/// 7. One complete shinespark axis, including acceleration, the shared sequence
/// words, speed clamps and previous-position correction. Diagonal callers invoke
/// X then Y; each call publishes its state before the next one begins.
pub use super::motion::shinespark;

/// Resolve terrain penetration after a grapple release.
pub use super::grapple::settle as settle_after_grapple;

/// Apply terrain effects at the player's bottom, center and top.
pub use super::inside::sample as sample_inside;
