//! Common actor state, gameplay rules and explicit compatibility boundaries.
#![forbid(unsafe_code)]
pub(in crate::game) mod access;
pub(in crate::game) mod api;
pub(in crate::game) mod bang;
pub(in crate::game) mod beetom;
pub(in crate::game) mod botwoon;
pub(in crate::game) mod botwoon_death;
pub(in crate::game) mod content;
pub(in crate::game) mod diving_enemies;
pub(in crate::game) mod effects;
pub(in crate::game) mod elevator;
pub(in crate::game) mod escape_animals;
pub(in crate::game) mod falling_platforms;
pub(in crate::game) mod fireflea;
pub(crate) mod identity;
pub(in crate::game) mod kago;
pub(in crate::game) mod maridia_fish;
pub(in crate::game) mod maridia_snail;
pub(in crate::game) mod media;
pub(crate) mod metroid;
pub(in crate::game) mod mochtroid;
#[cfg(sm_verify_dll)]
#[path = "../../verification/actor_movement_candidate.rs"]
pub(crate) mod movement_verification;
#[cfg(sm_verify_dll)]
#[path = "../../verification/actor_population_animation_entry.rs"]
pub(crate) mod reference_population_animation;
pub(in crate::game) mod refill_candy;
pub(in crate::game) mod reflec;
pub(in crate::game) mod roach;
pub(in crate::game) mod scene;
pub(in crate::game) mod sidehopper;
pub(crate) mod snapshot;
mod state;
pub(in crate::game) mod wall_crawlers;
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
pub(in crate::game) mod phantoon_motion;
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
pub(in crate::game) mod blue_face_block;
pub(in crate::game) mod brinstar_pipe_bug;
pub(in crate::game) mod crocomire_attack;
pub(in crate::game) mod crocomire_reactions;
pub(in crate::game) mod draygon_death;
pub(in crate::game) mod draygon_eye;
pub(in crate::game) mod draygon_flight;
pub(in crate::game) mod draygon_grab;
pub(in crate::game) mod draygon_sweep;
pub(in crate::game) mod evir;
pub(in crate::game) mod fune;
pub(in crate::game) mod ki_hunter;
pub(crate) mod lava_seahorse;
pub(in crate::game) mod maridia_floater;
pub(in crate::game) mod maridia_puffer;
pub(in crate::game) mod morph_ball_eye;
pub(in crate::game) mod norfair_lava_man;
pub(in crate::game) mod norfair_pipe_bug;
pub(crate) mod spore_spawn;
pub(in crate::game) mod wrecked_ship_ghost;
pub(in crate::game) mod wrecked_ship_orbs;
pub(in crate::game) mod wrecked_ship_robots;
pub(in crate::game) mod wrecked_ship_spark;
pub(crate) mod yapping_maw;
pub(in crate::game) mod yellow_pipe_bug;
