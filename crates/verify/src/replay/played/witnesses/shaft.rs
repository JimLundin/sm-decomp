//! Independently attributed Ceres shaft and return observations, in row-local cuts.

use super::RowWindow;
use game::state::video::OAM_SPRITES;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::replay) struct Shaft {
    pub(in crate::replay) entry: RowWindow,
    pub(in crate::replay::played) background: Background,
    hall: RowWindow,
    pub(in crate::replay::played) elevator: RowWindow,
    following_vent: Vent,
    pub(in crate::replay::played) prepared_prefix: u8,
    pub(in crate::replay::played) locked_prefix: u8,
    sound: Vec<Sound>,
    pub(in crate::replay::played) return_fade: Fade,
}

/// Completed writes within the entry trace. An interrupted shadow fill cannot be compared whole.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::replay::played) struct Background {
    pub(in crate::replay::played) effects_from: u64,
    pub(in crate::replay::played) tiles_from: u64,
    pub(in crate::replay::played) interrupted_shadow: Vec<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Vent {
    cut: u64,
    slot: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::replay::played) struct Sound {
    window: RowWindow,
    /// Inclusive intervals where the CPU reads the acknowledged cue. Other reads are clear.
    cue: Vec<[u64; 2]>,
}

impl Sound {
    pub(in crate::replay::played) fn cue(&self, cut: u64) -> bool {
        self.cue
            .iter()
            .any(|&[first, last]| (first..=last).contains(&cut))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::replay::played) struct Fade {
    pub(in crate::replay::played) window: RowWindow,
    pub(in crate::replay::played) timer_draw: u64,
    pub(in crate::replay::played) first_palette: u64,
    pub(in crate::replay::played) interrupted: usize,
    pub(in crate::replay::played) prepared: usize,
}

impl Shaft {
    pub(super) fn valid(&self) -> bool {
        let fade = &self.return_fade;
        self.entry.valid()
            && self.entry.covers(
                &self.entry.row,
                self.background.effects_from,
                self.background.tiles_from,
            )
            && self
                .background
                .interrupted_shadow
                .iter()
                .all(|cut| (self.background.effects_from..self.background.tiles_from).contains(cut))
            && self
                .background
                .interrupted_shadow
                .windows(2)
                .all(|pair| matches!(pair, [a, b] if a < b))
            && self.hall.valid()
            && self.elevator.valid()
            && !self.hall.overlaps(&self.elevator)
            && self
                .elevator
                .contains((&self.elevator.row, self.following_vent.cut))
            && usize::from(self.following_vent.slot) < OAM_SPRITES
            && [self.prepared_prefix, self.locked_prefix]
                .into_iter()
                .all(|len| len > 0 && usize::from(len) <= OAM_SPRITES)
            && !self.sound.is_empty()
            && self.sound.iter().enumerate().all(|(index, trace)| {
                let w = &trace.window;
                w.valid()
                    && (self.hall.covers(&w.row, w.first, w.last)
                        || self.elevator.covers(&w.row, w.first, w.last))
                    && self
                        .sound
                        .iter()
                        .take(index)
                        .all(|prior| !prior.window.overlaps(w))
                    && trace
                        .cue
                        .iter()
                        .all(|&[first, last]| w.covers(&w.row, first, last))
                    && trace
                        .cue
                        .windows(2)
                        .all(|pair| matches!(pair, [[_, last], [next, _]] if last < next))
            })
            && fade.window.valid()
            && fade.window.first > 0
            && fade
                .window
                .covers(&fade.window.row, fade.timer_draw, fade.first_palette)
            && fade.timer_draw < fade.first_palette
            && fade.interrupted > 0
            && fade.prepared > 0
    }

    pub(in crate::replay::played) fn actors(&self, place: (&str, u64)) -> bool {
        self.hall.contains(place) || self.elevator.contains(place)
    }

    pub(in crate::replay::played) fn following_vent(&self, place: (&str, u64)) -> Option<usize> {
        (self.elevator.contains(place) && self.following_vent.cut == place.1)
            .then(|| usize::from(self.following_vent.slot))
    }

    pub(in crate::replay::played) fn sound(&self, place: (&str, u64)) -> Option<&Sound> {
        self.sound.iter().find(|s| s.window.contains(place))
    }
}
