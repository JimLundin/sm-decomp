//! Independent observations at the held adjacent Animation return.

use super::{Datum, Expected, read_expected, word};
use crate::lock::Sha256;
use crate::replay::{compare::Observation, compared, data_failure, result::Outcome};
use data::{pose::animation::AnimationFrame, timing::MasterTicks};
use game::{
    production::{Production, frame, timing::Event},
    samus_appearance_bridge::Clips,
    state::owned::OwnedState,
};
use std::path::{Path, PathBuf};

pub(in crate::replay) struct Source {
    path: PathBuf,
    clips: Clips,
}

#[derive(serde::Deserialize)]
pub(in crate::replay) struct Observed {
    #[cfg(test)]
    pub start_ticks: u64,
    pub return_ticks: u64,
    #[cfg(test)]
    pub intrinsic_ticks: u64,
    #[cfg(test)]
    pub exposures: u64,
    #[cfg(test)]
    pub refresh_ticks: u64,
    #[cfg(test)]
    pub descriptor_only_hdma_ticks: u64,
    ordered_callbacks_through_return: usize,
    current_step: u16,
    entry_hold: u16,
    returned_step: u16,
    returned_hold: u16,
    emitted_sounds: u16,
    emitted_graphics: u16,
}

impl Source {
    pub fn new(root: &Path, clips: Clips) -> Self {
        Self {
            path: root.join("adjacent-animation-v1/observations.toml"),
            clips,
        }
    }

    pub fn read(&self, cut: u64) -> Result<Observed, Outcome> {
        let fault = || data_failure(Some(cut), "adjacent Animation endpoint");
        let pin =
            Sha256::try_from("d57256b266b6e08e940bf7a2129c88077d9baa1d725c8f03136a2fde8339fcea")
                .map_err(|_| fault())?;
        let bytes = read_expected(&self.path, pin, cut)?;
        toml::from_str(std::str::from_utf8(&bytes).map_err(|_| fault())?).map_err(|_| fault())
    }

    pub fn compare(
        &self,
        expected: &Expected,
        production: &Production,
        cut: u64,
        events: &[Event],
    ) -> Result<(), Outcome> {
        let fault = || data_failure(Some(cut), "adjacent Animation endpoint");
        let observed = self.read(cut)?;
        let preceding = cut.checked_sub(1).ok_or_else(fault)?;
        let prior = expected
            .returns
            .get(&preceding)
            .ok_or_else(fault)?
            .read(preceding, true)?;
        let prior_time: MasterTicks = canonical::from_bytes(
            &prior
                .fields
                .iter()
                .find(|value| value.field == "frame.time")
                .ok_or_else(fault)?
                .bytes,
        )
        .map_err(|_| fault())?;
        let returned = MasterTicks(
            prior_time
                .0
                .checked_add(observed.return_ticks)
                .ok_or_else(fault)?,
        );
        let next = expected
            .returns
            .get(&cut)
            .ok_or_else(fault)?
            .read(cut, true)?;
        let mut callbacks: Vec<Datum> = canonical::from_bytes(
            &next
                .fields
                .iter()
                .find(|value| value.field == "frame.outputs")
                .ok_or_else(fault)?
                .bytes,
        )
        .map_err(|_| fault())?;
        callbacks.retain(|datum| datum.at <= returned);
        if callbacks.len() != observed.ordered_callbacks_through_return {
            return Err(fault());
        }
        let OwnedState::Frame(state) = production.state() else {
            return Err(fault());
        };
        compared(
            cut,
            &[
                observation("frame.outputs", &callbacks),
                observation("frame.time", &returned),
            ],
            &[
                observation(
                    "frame.outputs",
                    &super::project(events).map_err(|_| fault())?,
                ),
                observation("frame.time", &state.timing.hardware.now),
            ],
        )?;
        self.compare_appearance(cut, state, &observed)?;
        let expected_health: Vec<_> = prior
            .fields
            .into_iter()
            .filter(|value| value.field.starts_with("frame.health."))
            .collect();
        let actual_health =
            super::FinalHealth::from(state.health.as_ref().ok_or_else(fault)?.state()).fields();
        compared(cut, &expected_health, &actual_health)
    }

    fn compare_appearance(
        &self,
        cut: u64,
        state: &frame::State,
        observed: &Observed,
    ) -> Result<(), Outcome> {
        let fault = || data_failure(Some(cut), "adjacent Animation playhead");
        let frame::animation_caller::Stage::Complete(done) = &state.animation else {
            return Err(Outcome::Difference {
                cut,
                field: "frame.animation.completion".to_owned(),
            });
        };
        let receipt = &done.receipt;
        let clip = self
            .clips
            .poses()
            .iter()
            .find(|entry| entry.figure == receipt.playing.0)
            .ok_or_else(fault)?;
        let step = |native| {
            clip.step_of(AnimationFrame(native))
                .map(|step| u16::from(step.0))
                .ok_or_else(fault)
        };
        let before = &receipt.before.model.0.state;
        let result = &state.appearance.model.0.state;
        let before_work = &receipt.before.model.0;
        let after_work = &state.appearance.model.0;
        let delta = |before: usize, after: usize| {
            u16::try_from(after.checked_sub(before).ok_or_else(fault)?).map_err(|_| fault())
        };
        let expected_fields = [
            word("frame.animation.before.step", step(observed.current_step)?),
            word("frame.animation.before.hold", observed.entry_hold),
            word("frame.animation.step", step(observed.returned_step)?),
            word("frame.animation.hold", observed.returned_hold),
            observation("frame.animation.stage", &true),
            word("frame.animation.laps", laps(before.laps_left)),
            observation("frame.animation.playing", &receipt.playing),
            word("frame.animation.sounds", observed.emitted_sounds),
            word("frame.animation.graphics", observed.emitted_graphics),
        ];
        let actual_fields = [
            word(
                "frame.animation.before.step",
                u16::from(before.playhead.step.0),
            ),
            word(
                "frame.animation.before.hold",
                u16::from(before.playhead.time_left.0),
            ),
            word("frame.animation.step", u16::from(result.playhead.step.0)),
            word(
                "frame.animation.hold",
                u16::from(result.playhead.time_left.0),
            ),
            observation("frame.animation.stage", &(result.stage == before.stage)),
            word("frame.animation.laps", laps(result.laps_left)),
            observation("frame.animation.playing", &state.playing_figure),
            word(
                "frame.animation.sounds",
                delta(before_work.sounds.len(), after_work.sounds.len())?,
            ),
            word(
                "frame.animation.graphics",
                delta(before_work.graphics.len(), after_work.graphics.len())?,
            ),
        ];
        compared(cut, &expected_fields, &actual_fields)
    }
}

fn laps(value: samus_appearance::LapsLeft) -> u16 {
    match value {
        samus_appearance::LapsLeft::WithoutBooster => 0,
        samus_appearance::LapsLeft::WithBooster(laps) => u16::from(laps.count()),
    }
}

fn observation(field: &str, value: &impl canonical::Canonical) -> Observation {
    Observation {
        field: field.to_owned(),
        bytes: canonical::to_bytes(value),
    }
}
