//! Whole-frame and outside-play entries. The main loop chooses which one to run.

use crate::events::HudEvent;
use crate::inputs::{FrameInputs, ItemControlInputs, NewArea, OutOfPlayInputs, StatusInputs};
use crate::pictures::HudPictures;
use crate::rules::Rules;
use crate::state::HudState;

impl HudState {
    /// Spec: ItemControls, MiniMapUpdate, StatusRefresh. Run a complete frame of play: controls
    /// first, then the map at the supplied movement position, then late status with the supplied
    /// status quantities. The frame-start count drives blink and flash. Late status compares its
    /// highlight with the one shown at frame start. Returns `Switched` then `Click` if raised.
    /// Use `HudWork` when the observations become available at different times.
    pub fn frame_of_play(
        &mut self,
        rules: &Rules,
        pictures: &HudPictures,
        inputs: &FrameInputs<'_>,
    ) -> Vec<HudEvent> {
        let initial_highlight = self.shown.highlighted;
        let switched = self.item_controls(&ItemControlInputs::from_frame(inputs));
        self.movement_map(
            rules,
            pictures,
            &inputs.mini_map,
            inputs.mother_brain_falls,
            inputs.frame,
        );
        let click = self.refresh_status(
            rules,
            pictures,
            &StatusInputs::from_frame(inputs),
            inputs.frame,
            initial_highlight,
        );
        switched.into_iter().chain(click).collect()
    }

    /// Spec: OutOfPlay. Between frames of play, every cell and shown value stays as it was.
    /// Only map turn-on, another area's explored cells, and a selection drop may change State.
    pub const fn out_of_play(&mut self, inputs: &OutOfPlayInputs<'_>) {
        self.mini_map_turned_on(inputs.mini_map_turned_on);
        if let NewArea::Entered(explored) = inputs.new_area {
            self.explored = *explored;
        }
        self.selection.apply_drop(inputs.dropped);
    }
}
