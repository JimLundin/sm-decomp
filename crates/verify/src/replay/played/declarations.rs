//! Reviewed played runs, shared by fixture replay and progress reporting.

use std::path::Path;

use serde::Deserialize;

use super::Outcome;

/// The reviewed played runs, from the repository root.
pub const PATH: &str = "scenarios/replay/played.toml";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Listed {
    pub(crate) movie: String,
    pub(crate) run: Vec<Listing>,
}

impl Listed {
    pub(crate) fn read(root: &Path) -> Result<Self, String> {
        let path = root.join(PATH);
        let text =
            std::fs::read_to_string(&path).map_err(|err| format!("{}: {err}", path.display()))?;
        Self::parse(&text)
    }

    pub(crate) fn parse(text: &str) -> Result<Self, String> {
        let listed: Self = toml::from_str(text).map_err(|err| format!("{PATH}: {err}"))?;
        for run in &listed.run {
            run.validate()
                .map_err(|why| format!("{PATH}: {}: {why}", run.name()))?;
        }
        Ok(listed)
    }
}

/// A run in cuts of its lock row and in movie frames.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Listing {
    pub(crate) row: String,
    pub(crate) start: u64,
    pub(crate) end: u64,
    pub(crate) held: u64,
    pub(crate) outcome: Outcome,
    pub(crate) seed_frame: u64,
    pub(crate) frames: Option<[u64; 2]>,
    pub(crate) power_on: bool,
}

impl Listing {
    pub(crate) fn name(&self) -> String {
        format!("{}/{}-{}", self.row, self.start, self.end)
    }

    fn validate(&self) -> Result<(), &'static str> {
        let Some(asked) = self.end.checked_sub(self.start).filter(|asked| *asked > 0) else {
            return Err("the end must follow the seed cut");
        };
        let next = self
            .start
            .checked_add(self.held)
            .and_then(|last| last.checked_add(1))
            .filter(|cut| *cut <= self.end);
        match &self.outcome {
            Outcome::Pass if self.held == asked => Ok(()),
            Outcome::Difference { cut, field } if Some(*cut) == next && !field.is_empty() => Ok(()),
            Outcome::Unsupported { cut, dependency }
                if cut.is_some() && *cut == next && !dependency.is_empty() =>
            {
                Ok(())
            }
            Outcome::Pass | Outcome::Difference { .. } | Outcome::Unsupported { .. } => {
                Err("the outcome contradicts the held cuts or omits its field or dependency")
            }
            Outcome::DataFailure { .. } | Outcome::CheckpointFailure { .. } => Err(
                "the outcome must describe a completed run, a difference or an unsupported dependency",
            ),
        }
    }
}
