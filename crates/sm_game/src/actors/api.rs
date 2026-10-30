//! Semantic operation signatures. Bodies are intentionally declaration-only.
#![forbid(unsafe_code)]
use crate::actors::{
    content::{ActorDefinitionKey, ActorTemplateId},
    media::AppearanceId,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlopePolicy {
    Ignore,
    Solid,
    Follow,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalProbeEdge {
    Positive,
    Negative,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovementResult {
    Clear,
    Blocked,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolidContact {
    Clear { amount: i32 },
    Blocked { amount: i32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerDirection {
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
pub struct DistanceDirection {
    pub distance: u16,
    pub direction: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeathHandling {
    Finalize,
    LeavePending,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShotHandling {
    ReactionOnly,
    Flash,
    Finalize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeathKind {
    Normal { animation: u16 },
    Rinka { animation: u16 },
    Grapple,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrappleReaction {
    None,
    Attach,
    Ignore,
    Cancel,
    AttachAlternate,
    AttachAlternate2,
    DamagePlayer,
    Unrecognized(u16),
}
pub struct GrappleContact {
    pub reaction: GrappleReaction,
    pub definition: ActorDefinitionKey,
}
impl GrappleContact {
    pub fn has_reaction(&self) -> bool {
        !matches!(self.reaction, GrappleReaction::None)
    }
}
pub struct MoveScan {
    pub displacement: i32,
    pub proposed_coordinate: u16,
    pub leading_edge: u16,
    pub remaining_extent: u16,
    pub original_extent: u16,
    pub slopes: SlopePolicy,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ActorSpawnRequest {
    pub template: ActorTemplateId,
    pub appearance: Option<AppearanceId>,
}
pub use crate::actors::combat::contact_reaction;
pub use crate::actors::combat::create_dud;
pub use crate::actors::combat::latch;
pub use crate::actors::combat::power_bomb_reaction;
pub use crate::actors::combat::resume_ai;
pub use crate::actors::combat::shot_reaction;
pub use crate::actors::movement::apply_vector;
pub use crate::actors::movement::move_horizontal;
pub use crate::actors::movement::move_vertical;
pub use crate::actors::movement::probe_horizontal;
pub use crate::actors::movement::probe_vertical;
pub use crate::actors::movement::probe_vertical_offset;
pub use crate::actors::movement::select_block_at;
pub use crate::actors::movement::snap_to_slopes;
#[cfg(sm_verify_dll)]
pub use crate::actors::population::respawn;
pub use crate::actors::queries::angle_between;
pub use crate::actors::queries::angle_from_delta;
pub use crate::actors::queries::angle_to_player;
pub use crate::actors::queries::current_outside_padded_screen;
pub use crate::actors::queries::current_outside_screen;
pub use crate::actors::queries::direction_to_player;
pub use crate::actors::queries::distance_and_direction;
pub use crate::actors::queries::leaving_screen;
pub use crate::actors::queries::overlaps_player_with_padding;
pub use crate::actors::queries::player_delta;
pub use crate::actors::queries::player_within;
pub use crate::actors::queries::touches_player_from_above;
#[cfg(sm_verify_dll)]
pub use crate::actors::scheduling::advance_selected;
#[cfg(sm_verify_dll)]
pub use crate::actors::staging::prepare_room_assets;
#[cfg(sm_verify_dll)]
pub use crate::actors::staging::transfer_actor_tiles_and_initialize;
