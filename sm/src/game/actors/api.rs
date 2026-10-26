//! Semantic operation signatures. Bodies are intentionally declaration-only.
#![forbid(unsafe_code)]
use super::{
    content::{ActorDefinitionKey, ActorTemplateId},
    media::AppearanceId,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game) enum Axis {
    X,
    Y,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game) enum SlopePolicy {
    Ignore,
    Solid,
    Follow,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game) enum VerticalProbeEdge {
    Positive,
    Negative,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game) enum MovementResult {
    Clear,
    Blocked,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game) enum SolidContact {
    Clear { amount: i32 },
    Blocked { amount: i32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game) enum PlayerDirection {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game) struct DistanceDirection {
    pub distance: u16,
    pub direction: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game) enum DeathHandling {
    Finalize,
    LeavePending,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game) enum ShotHandling {
    ReactionOnly,
    Flash,
    Finalize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game) enum DeathKind {
    Normal { animation: u16 },
    Rinka { animation: u16 },
    Grapple,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game) enum GrappleReaction {
    None,
    Attach,
    Ignore,
    Cancel,
    AttachAlternate,
    AttachAlternate2,
    DamagePlayer,
    Unrecognized(u16),
}
pub(in crate::game) struct GrappleContact {
    pub(super) reaction: GrappleReaction,
    pub(super) definition: ActorDefinitionKey,
}
impl GrappleContact {
    pub(in crate::game) fn has_reaction(&self) -> bool {
        !matches!(self.reaction, GrappleReaction::None)
    }
}
pub(super) struct MoveScan {
    pub displacement: i32,
    pub proposed_coordinate: u16,
    pub leading_edge: u16,
    pub remaining_extent: u16,
    pub original_extent: u16,
    pub slopes: SlopePolicy,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ActorSpawnRequest {
    pub template: ActorTemplateId,
    pub appearance: Option<AppearanceId>,
}
pub(in crate::game) use super::combat::contact_reaction;
pub(in crate::game) use super::combat::create_dud;
pub(in crate::game) use super::combat::latch;
pub(in crate::game) use super::combat::power_bomb_reaction;
pub(in crate::game) use super::combat::resume_ai;
pub(in crate::game) use super::combat::shot_reaction;
pub(in crate::game) use super::movement::apply_vector;
pub(in crate::game) use super::movement::move_horizontal;
pub(in crate::game) use super::movement::move_vertical;
pub(in crate::game) use super::movement::probe_horizontal;
pub(in crate::game) use super::movement::probe_vertical;
pub(in crate::game) use super::movement::probe_vertical_offset;
pub(in crate::game) use super::movement::select_block_at;
pub(in crate::game) use super::movement::snap_to_slopes;
pub(in crate::game) use super::population::respawn;
pub(in crate::game) use super::queries::angle_between;
pub(in crate::game) use super::queries::angle_from_delta;
pub(in crate::game) use super::queries::angle_to_player;
pub(in crate::game) use super::queries::current_outside_padded_screen;
pub(in crate::game) use super::queries::current_outside_screen;
pub(in crate::game) use super::queries::direction_to_player;
pub(in crate::game) use super::queries::distance_and_direction;
pub(in crate::game) use super::queries::leaving_screen;
pub(in crate::game) use super::queries::overlaps_player_with_padding;
pub(in crate::game) use super::queries::player_delta;
pub(in crate::game) use super::queries::player_within;
pub(in crate::game) use super::queries::touches_player_from_above;
pub(in crate::game) use super::scheduling::advance_selected;
pub(in crate::game) use super::staging::prepare_room_assets;
pub(in crate::game) use super::staging::transfer_actor_tiles_and_initialize;
