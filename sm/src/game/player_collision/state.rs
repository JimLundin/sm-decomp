//! Additional state for player collision; Session remains the only owner.
#![forbid(unsafe_code)]
use crate::game::session::Position;

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::game) struct CollisionState {
    pub(in crate::game) grapple_ejection: [u16; 4],
    pub(in crate::game) flags: u16,
    pub(in crate::game) slope_adjusted: u16,
    pub(in crate::game) solid_actor_flags: u16,
    pub(in crate::game) pose_input: u16,
    pub(in crate::game) horizontal_slope_flags: u16,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::game) struct PoseClearance {
    pub(in crate::game) actor_flags: u16,
    pub(in crate::game) terrain_flags: u16,
    pub(in crate::game) terrain_up: u16,
    pub(in crate::game) radius_difference: u16,
    pub(in crate::game) terrain_down: u16,
    pub(in crate::game) actor_up: u16,
    pub(in crate::game) actor_down: u16,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::game) struct PoseRecord {
    pub(in crate::game) pose: u16,
    // Low byte: facing; high byte: movement type. Native stores use the codec.
    pub(in crate::game) movement_and_facing: u16,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::game) struct PoseHistory {
    pub(in crate::game) previous: PoseRecord,
    pub(in crate::game) last_different: PoseRecord,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::game) struct PreviousPosition {
    pub(in crate::game) x: Position,
    pub(in crate::game) y: Position,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::game) struct AnimationClock {
    pub(in crate::game) remaining: u16,
    pub(in crate::game) frame: u16,
}

/// Shared by shinespark, suit effects, cinematics and retained sequence code.
/// Shinespark composes stage as the high word and value as the low word.
#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::game) struct SequenceWorkspace {
    pub(in crate::game) stage: u16,
    pub(in crate::game) value: u16,
}

/// Observable multiply operands shared with retained compatibility routines.
#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::game) struct MultiplyScratch {
    pub(in crate::game) left: u16,
    pub(in crate::game) right: u16,
}
