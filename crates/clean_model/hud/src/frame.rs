//! One frame of the HUD, and a boss met outside a frame (§5: the order that matters).

use crate::events::Event;
use crate::grid::Point;
use crate::inputs::FrameInputs;
use crate::pictures::HudPictures;
use crate::rules::{Boss, RoomBoss, Rules};
use crate::state::{HudState, MiniMapPower};

/// The HUD after a frame, and what it reports.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FrameOutcome {
    /// The state after the frame; its grid is the picture the HUD draws.
    pub state: HudState,
    /// What it reports, in the order raised: `Switched`, then `Click`.
    pub events: Vec<Event>,
}

impl HudState {
    /// One frame of play, in the order §5 gives: the selection changes (Select, item cancel),
    /// then a shot may cancel it; the mini-map is turned on, then Mother Brain may fall, then the
    /// mini-map updates; then the status bar is drawn from the selection as it then is: the
    /// energy, the counts and the sign, a pickup's icon, the highlight, and the one-shot item's
    /// flash last.
    ///
    /// Spec: SwitchClick. When the item shown highlighted changes, from an item to none as much
    /// as to another item, and nothing mutes it (a spin jump, a wall jump, the grapple beam out,
    /// time stopped), the frame requests a `Click`.
    #[must_use]
    pub fn frame(
        mut self,
        rules: &Rules,
        pictures: &HudPictures,
        inputs: &FrameInputs<'_>,
    ) -> FrameOutcome {
        let mut events = Vec::new();
        events.extend(self.press_buttons(inputs.controls, &inputs.status));
        self.shot_cancels(inputs.controls);
        // Spec: MiniMapStill. Turned on, it isn't off, before this frame's update.
        if inputs.mini_map.turned_on {
            self.mini_map = MiniMapPower::On;
        }
        // Spec: BossBlanksTheMap. Her fall leaves the mini-map off, so this frame's update
        // doesn't happen.
        if inputs.mother_brain_falls {
            self.blank_mini_map(
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
        let highlight_moved = self.highlight(pictures);
        self.one_shot_flash(rules, pictures, inputs.frame);
        if highlight_moved && inputs.controls.muted_by.is_none() {
            events.push(Event::Click);
        }
        FrameOutcome {
            state: self,
            events,
        }
    }

    /// Spec: BossBlanksTheMap. A boss met on entering its room while it lives, a step of its own
    /// outside a frame of play: the window goes blank, the screens it names, from her room's
    /// top-left cell `room`, join the explored cells, and the mini-map is off. Nothing else
    /// changes.
    #[must_use]
    pub fn boss_encountered(
        mut self,
        rules: &Rules,
        pictures: &HudPictures,
        boss: RoomBoss,
        room: Point,
    ) -> Self {
        self.blank_mini_map(rules, pictures, Boss::InItsRoom(boss), room);
        self
    }
}
