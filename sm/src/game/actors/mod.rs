//! Common actor state, gameplay rules and explicit compatibility boundaries.
#![forbid(unsafe_code)]
pub(in crate::game) use sm_game::actors::access;
pub(in crate::game) use sm_game::actors::api;
pub(in crate::game) use sm_game::actors::bang;
pub(in crate::game) use sm_game::actors::morph_ball_eye;
pub(in crate::game) use sm_game::actors::reflec;
pub(in crate::game) mod boss_lease;
pub(crate) mod botwoon;
pub(in crate::game) use sm_game::actors::botwoon_death;
pub(crate) mod baby_metroid;
pub(crate) mod escape_critters;
pub(in crate::game) mod fireflea;
pub(crate) use sm_game::actors::identity;
pub(in crate::game) mod maridia_snail;
#[cfg(sm_verify_dll)]
#[path = "../../verification/actor_movement_candidate.rs"]
pub(crate) mod movement_verification;
#[cfg(sm_verify_dll)]
#[path = "../../verification/actor_population_animation_entry.rs"]
pub(crate) mod reference_population_animation;
pub(crate) mod shaktool;
pub(in crate::game) mod skultera;
pub(crate) mod torizo;

mod runtime;
pub(crate) use runtime::restart_from_native_cache;

mod content_runtime;

#[cfg(sm_verify_dll)]
pub(in crate::game) use sm_game::actors::animation;
pub(in crate::game) use sm_game::actors::combat;
#[cfg(sm_verify_dll)]
pub(in crate::game) use sm_game::actors::contacts;
pub(in crate::game) use sm_game::actors::drawing;
pub(in crate::game) mod kraid_fingernails;
pub(in crate::game) mod kraid_lint;
pub(in crate::game) mod media_runtime;
pub(in crate::game) mod mother_brain_effects;
#[cfg(sm_verify_dll)]
pub(in crate::game) use sm_game::actors::movement;
pub(in crate::game) mod part_hitboxes;
pub(in crate::game) mod phantoon_actions;
pub(in crate::game) use sm_game::actors::population;
pub(in crate::game) use sm_game::actors::queries;
pub(in crate::game) use sm_game::actors::scheduling;
pub(in crate::game) use sm_game::actors::staging;
mod runtime_host;
mod species_host;

pub(crate) fn prepare_room_assets(h: &mut crate::game::session_runtime::Runtime<'_>) {
    staging::prepare_room_assets(h);
}

#[cfg(sm_verify_dll)]
#[path = "../../verification/actor_runtime_checks.rs"]
mod runtime_checks;
#[cfg(sm_verify_dll)]
pub(crate) use runtime_checks::run_checked as verify_runtime_ownership;

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

pub(in crate::game) mod atomic;
pub(crate) mod crocomire_reactions;
pub(in crate::game) mod draygon_eye;
pub(in crate::game) mod draygon_flight;
pub(in crate::game) mod fune;
pub(crate) mod magdollite;
pub(in crate::game) mod maridia_floater;
pub(crate) mod spore_spawn;
pub(crate) mod yapping_maw;
