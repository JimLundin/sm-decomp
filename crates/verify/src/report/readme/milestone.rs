//! The headline milestone the README counts frames over, from the whole-game plan.

use std::path::Path;

use serde::Deserialize;

use crate::report::read;

/// The whole-game plan as data, from the repository root.
pub(super) const PLAN: &str = "docs/architecture/domains.toml";

/// The part of the plan the README reads.
#[derive(Deserialize)]
struct Plan {
    milestone: Milestone,
}

/// The headline milestone: a stretch of a movie, from power-on, that the engine is to replay
/// alone.
#[derive(Deserialize)]
pub(super) struct Milestone {
    /// What the README calls it.
    pub(super) name: String,
    /// The movie: the key of a movie of `movies/manifest.toml`, its file name's word after
    /// `super-metroid-`.
    pub(super) movie: String,
    /// The seed whose movie frame ends the stretch, `CAPTURE:INDEX`: the first room past it.
    ends: String,
}

impl Milestone {
    /// The plan's milestone under `root`.
    ///
    /// # Errors
    ///
    /// Why the plan can't be read, or names no milestone.
    pub(super) fn read(root: &Path) -> Result<Self, String> {
        toml::from_str::<Plan>(&read(&root.join(PLAN))?)
            .map(|plan| plan.milestone)
            .map_err(|e| format!("{PLAN}: {e}"))
    }

    /// The seed that ends the stretch: its capture and index.
    pub(super) fn end(&self) -> Result<(&str, u32), String> {
        seed(&self.ends)
    }
}

/// A seed named `CAPTURE:INDEX`: its capture and index.
///
/// # Errors
///
/// The plan and the name when it isn't `CAPTURE:INDEX`.
pub(super) fn seed(name: &str) -> Result<(&str, u32), String> {
    name.split_once(':')
        .and_then(|(capture, index)| Some((capture, index.parse().ok()?)))
        .ok_or_else(|| format!("{PLAN}: `{name}` is not a seed, CAPTURE:INDEX"))
}
