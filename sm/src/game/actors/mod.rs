//! Common actor state, gameplay rules and explicit compatibility boundaries.
#![forbid(unsafe_code)]
pub(in crate::game) mod access;
pub(in crate::game) mod api;
pub(in crate::game) mod beetom;
pub(in crate::game) mod content;
pub(in crate::game) mod effects;
pub(crate) mod identity;
pub(in crate::game) mod kago;
pub(in crate::game) mod media;
#[cfg(sm_verify_dll)]
#[path = "../../verification/actor_movement_candidate.rs"]
pub(crate) mod movement_verification;
#[cfg(sm_verify_dll)]
#[path = "../../verification/actor_population_animation_entry.rs"]
pub(crate) mod reference_population_animation;
pub(in crate::game) mod scene;
pub(crate) mod snapshot;
mod state;
pub(in crate::game) use state::Actors;

pub(in crate::game) mod codec;
mod runtime;

mod content_runtime;

pub(in crate::game) mod animation;
pub(in crate::game) mod combat;
pub(in crate::game) mod contacts;
pub(in crate::game) mod drawing;
pub(in crate::game) mod media_runtime;
pub(in crate::game) mod movement;
pub(in crate::game) mod population;
pub(in crate::game) mod queries;
pub(in crate::game) mod scheduling;
pub(in crate::game) mod staging;

pub(crate) use staging::prepare_room_assets;

#[cfg(sm_verify_dll)]
#[path = "../../verification/actor_runtime_checks.rs"]
mod runtime_checks;
#[cfg(sm_verify_dll)]
pub(crate) use runtime_checks::run_checked as verify_runtime_ownership;

#[cfg(sm_verify_dll)]
#[path = "../../verification/actor_fixture.rs"]
mod fixture;
#[cfg(sm_verify_dll)]
#[path = "../../verification/actor_media_runtime_checks.rs"]
mod media_checks;

#[cfg(sm_verify_dll)]
#[path = "../../verification/actor_mutable_content_checks.rs"]
pub(crate) mod mutable_content_checks;

#[cfg(sm_verify_dll)]
#[path = "../../verification/actor_initializer_checks.rs"]
mod initializer_checks;

#[cfg(sm_verify_dll)]
#[path = "../../verification/actor_owner_checks.rs"]
mod owner_checks;

#[cfg(sm_verify_dll)]
#[path = "../../verification/actor_contact_checks.rs"]
mod contact_checks;

#[cfg(sm_verify_dll)]
#[path = "../../verification/actor_composed_fixture.rs"]
pub(crate) mod composed_fixture;

#[cfg(sm_verify_dll)]
#[path = "../../verification/player_collision_composed_actor.rs"]
pub(crate) mod collision_composed_actor;
#[cfg(sm_verify_dll)]
#[path = "../../verification/actor_composed_checks.rs"]
pub(crate) mod composed_checks;

mod overlap;
pub(in crate::game) use overlap::axis_overlaps;
pub(in crate::game) mod evir;
pub(in crate::game) mod fune;
pub(in crate::game) mod wrecked_ship_ghost;
pub(in crate::game) mod wrecked_ship_robots;
