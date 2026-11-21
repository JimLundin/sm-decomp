//! Independent observations of the native Random owner.

use super::data_failure;
use crate::lock::Sha256;
use crate::replay::result::Outcome;
use serde::Deserialize;
use std::path::PathBuf;

pub(in crate::replay) struct Source {
    pub(in crate::replay) path: PathBuf,
    pub(in crate::replay) sha256: Sha256,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Capture {
    format: u16,
    draw: [Draw; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::replay) struct Draw {
    cut: u64,
    pub(in crate::replay) complete_tick: u64,
    pub(in crate::replay) result: u16,
    pub(in crate::replay) write_ticks: [u64; 4],
    #[cfg(test)]
    pub(in crate::replay) write_values: [u8; 4],
    #[cfg(not(test))]
    #[serde(rename = "write_values")]
    _write_values: [u8; 4],
    pub(in crate::replay) sample_ticks: [u64; 3],
    #[cfg(test)]
    pub(in crate::replay) sample_values: [u8; 3],
    #[cfg(not(test))]
    #[serde(rename = "sample_values")]
    _sample_values: [u8; 3],
}

impl Source {
    pub(super) fn read(&self, cut: u64) -> Result<Draw, Outcome> {
        let fault = || data_failure(Some(cut), "independent Random observations");
        let bytes = super::read_expected(&self.path, self.sha256, cut)?;
        let text = std::str::from_utf8(&bytes).map_err(|_| fault())?;
        let capture: Capture = toml::from_str(text).map_err(|_| fault())?;
        let [first, second] = &capture.draw;
        if capture.format != 1
            || [first.cut, second.cut] != [1, 2]
            || first.complete_tick >= second.complete_tick
            || capture.draw.iter().any(|draw| {
                !draw.write_ticks.is_sorted()
                    || !draw.sample_ticks.is_sorted()
                    || draw.write_ticks.last() >= Some(&draw.complete_tick)
                    || draw.sample_ticks.last() >= Some(&draw.complete_tick)
            })
        {
            return Err(fault());
        }
        capture
            .draw
            .into_iter()
            .find(|draw| draw.cut == cut)
            .ok_or_else(fault)
    }
}
