//! Additional state for player collision; Session remains the only owner.
#![forbid(unsafe_code)]
use crate::geometry::Position;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CollisionState {
    pub grapple_ejection: [u16; 4],
    pub flags: u16,
    pub slope_adjusted: u16,
    pub solid_actor_flags: u16,
    pub pose_input: u16,
    pub horizontal_slope_flags: u16,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PoseClearance {
    pub actor_flags: u16,
    pub terrain_flags: u16,
    pub terrain_up: u16,
    pub radius_difference: u16,
    pub terrain_down: u16,
    pub actor_up: u16,
    pub actor_down: u16,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PoseRecord {
    pub pose: u16,
    // Low byte: facing; high byte: movement type. Native stores use the codec.
    pub movement_and_facing: u16,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PoseHistory {
    pub previous: PoseRecord,
    pub last_different: PoseRecord,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PreviousPosition {
    pub x: Position,
    pub y: Position,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimationClock {
    pub remaining: u16,
    pub frame: u16,
}

/// Shared by shinespark, suit effects, cinematics and retained sequence code.
/// Shinespark composes stage as the high word and value as the low word.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SequenceWorkspace {
    pub stage: u16,
    pub value: u16,
}

/// Observable multiply operands shared with retained compatibility routines.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MultiplyScratch {
    pub left: u16,
    pub right: u16,
}
