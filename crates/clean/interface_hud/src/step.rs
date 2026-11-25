//! The HUD's steps: a frame of play, whole, and a step outside play. The main loop chooses which
//! one each frame is.

use crate::countdown::{CountdownAnswer, UnsupportedFrame};
use crate::events::HudEvent;
use crate::inputs::{FrameInputs, NewArea, OutOfPlayInputs};
use crate::pictures::HudPictures;
use crate::rules::Rules;
use crate::state::{HudState, Selection};

/// What a frame of play reports, beside the HUD's State as the frame ends.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameOutputs {
    /// Spec: Switching. The selection after the controls: the selected item and the one-shot
    /// item as this frame's item select and item cancel leave them, before any drop. It depends
    /// only on the State as the frame begins, the buttons, `can switch` and her stock as the
    /// frame begins, so nothing later in the frame changes it. The weapons and her look read it.
    pub selection_after_controls: Selection,
    /// The events, in the order raised: a `Switched` from the controls, then a `Click` from the
    /// status bar.
    pub events: Vec<HudEvent>,
    /// The escape countdown's answer for this frame: its expiry answer and its picture.
    pub countdown: CountdownAnswer,
}

impl HudState {
    /// Plays one frame of play, whole: the frame's inputs go in, and the State is left as the
    /// frame ends, with the frame's outputs returned. The frame's order (§5):
    ///
    /// 1. The escape countdown takes its turn, which it alone reads and changes; a frame the
    ///    spec leaves out (a start on a frame that only shows the countdown) is refused there,
    ///    before any effect.
    /// 2. The selection responds to the controls, by her stock as the frame begins; that is
    ///    the selection after the controls.
    /// 3. The mini-map reflects her position after her moves.
    /// 4. A drop before the status bar lands, so the highlight and the flash show it.
    /// 5. The status bar draws the values as the frame ends.
    /// 6. A drop after the status bar lands, leaving the bar as drawn for the frame.
    ///
    /// # Errors
    ///
    /// `UnsupportedFrame::StartWhileOnlyShown` when the countdown starts on a frame whose scene
    /// only shows it; the State is then unchanged.
    pub fn frame_of_play(
        &mut self,
        rules: &Rules,
        pictures: &HudPictures,
        inputs: &FrameInputs<'_>,
    ) -> Result<FrameOutputs, UnsupportedFrame> {
        let countdown = self.countdown.play(inputs.countdown)?;
        let switched = self
            .selection
            .respond_to_controls(inputs.controls, inputs.stock_at_start);
        let selection_after_controls = self.selection;
        self.mini_map_turn(rules, pictures, inputs);
        self.selection.apply_drop(inputs.dropped_before_status_bar);
        let click = self.draw_status_bar(rules, pictures, inputs);
        self.selection.apply_drop(inputs.dropped_after_status_bar);
        Ok(FrameOutputs {
            selection_after_controls,
            events: switched.into_iter().chain(click).collect(),
            countdown,
        })
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
