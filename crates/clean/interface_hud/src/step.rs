//! The steps of the HUD, each its own entry on its state (§1.3; §5: the order that matters).
//!
//! The main loop chooses which step each frame of the running game is, once the HUD is drawn:
//! a frame of play (`HudState::frame_of_play`), a step outside play (`HudState::out_of_play`),
//! or a boss met on entering its room (`HudState::boss_blanks_the_map`). The load flow's
//! start-up of the HUD and the pause screen's frames are no step of this model: those domains
//! set the state themselves.

use crate::events::HudEvent;
use crate::inputs::{FrameInputs, NewArea, OutOfPlayInputs};
use crate::pictures::HudPictures;
use crate::rules::{Boss, Rules};
use crate::state::{HudState, MiniMapPower};

impl HudState {
    /// One frame of play: a frame the game plays her and the enemies, in a room, and also while
    /// the picture fades in after a load, fades out to the pause menu and in again after it,
    /// while the reserve tanks refill, in her death's first frames, while the time runs out, on
    /// Ceres's last frames after she reaches its elevator, as she escapes Zebes, and in the
    /// demos. The frame on which she enters a door is one. It returns what it reports, in the
    /// order raised: `Switched`, then `Click`.
    ///
    /// In the order §5 gives: the selection changes (Select, item cancel), then the drop before
    /// the status bar lands; the mini-map is turned on, then Mother Brain may fall, then the
    /// mini-map updates; then the status bar is drawn from the selection as it then is: the
    /// energy, the counts and the sign, a pickup's icon, the highlight, and the one-shot item's
    /// flash last. The drop after the status bar comes last.
    ///
    /// Spec: SwitchClick. When the item shown highlighted after the frame isn't the one shown
    /// before it, from an item to none as much as to another item, and nothing mutes it (a spin
    /// jump, a wall jump, the grapple beam out, time stopped), the frame requests a `Click`.
    pub fn frame_of_play(
        &mut self,
        rules: &Rules,
        pictures: &HudPictures,
        inputs: &FrameInputs<'_>,
    ) -> Vec<HudEvent> {
        let highlighted_before = self.shown.highlighted;
        let mut events = Vec::new();
        events.extend(self.press_buttons(inputs.controls, &inputs.status));
        self.selection.apply_drop(inputs.dropped_before_status_bar);
        self.mini_map_turned_on(inputs.mini_map.turned_on);
        // Spec: BossBlanksTheMap. Her fall leaves the mini-map off, so this frame's update
        // doesn't happen.
        if inputs.mother_brain_falls {
            self.boss_blanks_the_map(
                rules,
                pictures,
                Boss::MotherBrain,
                inputs.mini_map.room.cell,
            );
        }
        if self.mini_map_updates(&inputs.mini_map) {
            self.explore_and_draw_mini_map(rules, pictures, &inputs.mini_map, inputs.frame);
        }
        self.energy_gauge(pictures, inputs.status.energy);
        self.ammo_counts(pictures, &inputs.status.ammunition);
        self.reserve_sign(pictures, inputs.status.reserve);
        if let Some(item) = inputs.collected {
            self.icon_appears(pictures, item);
        }
        self.highlight(pictures);
        self.one_shot_flash(rules, pictures, inputs.frame);
        self.selection.apply_drop(inputs.dropped_after_status_bar);
        let highlight_moved = self.shown.highlighted != highlighted_before;
        if highlight_moved && inputs.controls.muted_by.is_none() {
            events.push(HudEvent::Click);
        }
        events
    }

    /// Spec: OutOfPlay. A step outside play: a door's passage, a load's other frames, the
    /// cinematics (the opening's scenes among them). Every cell and the values last shown stay
    /// as they are, and so do the selection, the explored cells and whether the mini-map is off,
    /// but that the mini-map may be turned on, a new area brings the explored cells Progress
    /// keeps for it, and another domain may drop the selection.
    pub const fn out_of_play(&mut self, inputs: &OutOfPlayInputs<'_>) {
        self.mini_map_turned_on(inputs.mini_map_turned_on);
        if let NewArea::Entered(explored) = inputs.new_area {
            self.explored = *explored;
        }
        self.selection.apply_drop(inputs.dropped);
    }

    /// Spec: MiniMapStill, OutOfPlay. The mini-map turned on in this step: it isn't off, in a
    /// frame of play before the mini-map's turn.
    const fn mini_map_turned_on(&mut self, turned_on: bool) {
        if turned_on {
            self.mini_map = MiniMapPower::On;
        }
    }
}
