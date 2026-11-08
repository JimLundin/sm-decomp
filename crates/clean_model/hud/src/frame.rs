//! One frame of the HUD, and a boss's event outside a frame (§5: the order that matters).

use clean_types::MapCell;

use crate::events::Event;
use crate::inputs::FrameInputs;
use crate::mini_map::{boss_turns_it_off, update, updates};
use crate::rules::{Boss, Rules};
use crate::state::HudState;
use crate::status_bar::draw_status_bar;

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
    /// then the status bar is drawn from the selection as it then is.
    ///
    /// Spec: SwitchClick. When the highlight moves and nothing mutes it (a spin jump, a wall
    /// jump, the grapple beam out, time stopped), the frame requests a `Click`.
    #[must_use]
    pub fn frame(mut self, rules: &Rules, inputs: &FrameInputs<'_>) -> FrameOutcome {
        let mut events = Vec::new();
        let (selection, switched) = self
            .selection
            .after_buttons(&inputs.controls, &inputs.status);
        events.extend(switched);
        self.selection = selection.after_shot(&inputs.controls);

        let mini_map = &inputs.mini_map;
        if let Some(boss) = inputs.boss {
            boss_turns_it_off(
                rules,
                boss,
                mini_map.room.cell,
                &mut self.grid,
                &mut self.explored,
            );
        }
        // Spec: BossBlanksTheMap. Once a boss turns it off, the mini-map is off from then (spec:
        // MiniMapStill).
        if inputs.boss.is_none() && updates(mini_map) {
            update(
                rules,
                mini_map,
                inputs.frame,
                &mut self.grid,
                &mut self.explored,
            );
        }

        let moved = draw_status_bar(
            rules,
            inputs,
            self.selection,
            &mut self.grid,
            &mut self.shown,
        );
        if moved && inputs.controls.muted_by.is_none() {
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
        boss_turns_it_off(rules, boss, room, &mut self.grid, &mut self.explored);
        self
    }
}
