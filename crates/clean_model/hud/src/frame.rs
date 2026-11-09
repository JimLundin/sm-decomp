//! One frame of the HUD, and a boss's event outside a frame (§5: the order that matters).

use crate::events::Event;
use crate::inputs::FrameInputs;
use crate::map::MapCell;
use crate::rules::{Boss, Rules};
use crate::state::HudState;

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
    /// then a shot may cancel it and a boss may turn the mini-map off, then the mini-map updates,
    /// then the status bar is drawn from the selection as it then is: the energy, the counts and
    /// the sign, a pickup's icon, the highlight, and the one-shot item's flash last.
    ///
    /// Spec: SwitchClick. When the item shown highlighted changes, from an item to none as much
    /// as to another item, and nothing mutes it (a spin jump, a wall jump, the grapple beam out,
    /// time stopped), the frame requests a `Click`.
    #[must_use]
    pub fn frame(mut self, rules: &Rules, inputs: &FrameInputs<'_>) -> FrameOutcome {
        let mut events = Vec::new();
        events.extend(self.press_buttons(inputs.controls, &inputs.status));
        self.shot_cancels(inputs.controls);
        if let Some(boss) = inputs.boss {
            self.blank_mini_map(rules, boss, inputs.mini_map.room.cell);
        }
        if inputs.mini_map.updates() {
            self.update_mini_map(rules, &inputs.mini_map, inputs.frame);
        }
        self.energy_gauge(inputs.status.energy);
        self.ammo_counts(&inputs.status.ammunition);
        self.reserve_sign(inputs.status.reserve);
        if let Some(item) = inputs.collected {
            self.icon_appears(item);
        }
        let highlight_changed = self.highlight();
        self.one_shot_flash(rules, inputs.frame);
        if highlight_changed && inputs.controls.muted_by.is_none() {
            events.push(Event::Click);
        }
        FrameOutcome {
            state: self,
            events,
        }
    }

    /// Spec: BossBlanksTheMap. A boss turns the mini-map off outside a frame of play, on entering
    /// its room while it lives: the window goes blank and the screens it names, from her room's
    /// top-left cell `room`, join the explored cells. Nothing else changes; the mini-map is off
    /// from then, which the frames that follow read as their input.
    #[must_use]
    pub fn boss_turns_off_mini_map(mut self, rules: &Rules, boss: Boss, room: MapCell) -> Self {
        self.blank_mini_map(rules, boss, room);
        self
    }
}
