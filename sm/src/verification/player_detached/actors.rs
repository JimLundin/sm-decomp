//! Actual actor data, codec and access helpers in the detached composition.
//! Runtime/backend adapters are deliberately not registered in this target.
#![forbid(unsafe_code)]
#[path = "../../game/actors/access.rs"]
pub(in crate::game) mod access;
#[path = "../../game/actors/botwoon_death.rs"]
pub(in crate::game) mod botwoon_death;
#[path = "../../game/actors/codec.rs"]
pub(in crate::game) mod codec;
#[path = "../../game/actors/content.rs"]
pub(in crate::game) mod content;
#[path = "../../game/actors/identity.rs"]
pub(in crate::game) mod identity;
#[path = "../../game/actors/media.rs"]
pub(in crate::game) mod media;
#[path = "../../game/actors/scene.rs"]
pub(in crate::game) mod scene;
#[path = "../../game/actors/snapshot.rs"]
pub(in crate::game) mod snapshot;
#[path = "../../game/actors/state.rs"]
mod state;
pub(in crate::game) use state::Actors;
#[path = "actors_host.rs"]
mod host;

#[cfg(test)]
#[path = "../actor_catalog_checks.rs"]
mod catalog_checks;
#[cfg(test)]
#[path = "../actor_codec_checks.rs"]
mod codec_checks;

#[cfg(test)]
#[path = "../actor_fixture.rs"]
mod fixture;
#[cfg(test)]
#[path = "../actor_palette_checks.rs"]
mod palette_checks;

use sm_game::actors::overlap;
pub(in crate::game) use overlap::axis_overlaps;
