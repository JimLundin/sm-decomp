//! Actual actor data, codec and access helpers in the detached composition.
//! Runtime/backend adapters are deliberately not registered in this target.
#![forbid(unsafe_code)]
pub(in crate::game) use sm_game::actors::access;
pub(in crate::game) use sm_game::actors::codec;
#[path = "actors_host.rs"]
mod host;

#[cfg(test)]
#[path = "../actor_catalog_checks.rs"]
mod catalog_checks;
#[cfg(test)]
#[path = "../actor_codec_checks.rs"]
mod codec_checks;

#[cfg(test)]
use sm_game::actors::{content, fixture, identity, media, scene, snapshot};
#[cfg(test)]
#[path = "../actor_palette_checks.rs"]
mod palette_checks;

pub(in crate::game) use overlap::axis_overlaps;
use sm_game::actors::overlap;
