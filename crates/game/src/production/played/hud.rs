//! The HUD's frame of play, its step outside play, and its meeting with the station's boss.
//!
//! The HUD plays after her moves, reading her position after them and the energy Health leaves.

use super::{Content, Fault, Rules};
use crate::production::ridley;
use boss_ceres_ridley::RidleyPlacing;
use interface_hud::{
    Ammunition, CancelButton, Controls, CountdownRuns, Dropped, Energy, FrameInputs, FrameOutputs,
    MiniMapInputs, Mute, NewArea, OutOfPlayInputs, Reserve, SelectButton, Status, StockAtStart,
};
use types::{Action, Facing, Pair, Parts};

/// What a frame of play brings the HUD from owners outside this loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HudFrame {
    /// Item select and item cancel this frame.
    pub buttons: (SelectButton, CancelButton),
    /// Her stock as the frame begins.
    pub stock_at_start: StockAtStart,
    /// Her energy capacity as the frame ends.
    pub capacity: u16,
    /// Her reserve tanks as the frame ends.
    pub reserve: Reserve,
    /// Her ammunition as the frame ends.
    pub ammunition: Ammunition,
    /// She has collected her map.
    pub map_collected: bool,
    /// Whether the scene runs the escape countdown this frame.
    pub countdown: CountdownRuns,
}

impl super::State {
    /// The HUD's frame of play, from `hud`, `samus` as the frame leaves her, where her moves
    /// left her and the `health` the frame leaves; State's HUD is not changed.
    pub(super) fn hud_frame(
        &self,
        hud: &HudFrame,
        samus: &samus_movement::Samus,
        after_moves: types::Position,
        health: samus_health::State,
        content: &Content,
        rules: &Rules,
    ) -> Result<(interface_hud::HudState, FrameOutputs), Fault> {
        let mut hud_state = self.hud.0.clone();
        let outputs = hud_state
            .frame_of_play(
                &rules.hud,
                &content.hud,
                &hud_inputs(hud, self, samus, after_moves, health, content),
            )
            .map_err(|_| Fault::Hud)?;
        Ok((hud_state, outputs))
    }

    /// The HUD meets the boss as his placing hands it on, outside a frame of play, in the room
    /// the door she touched leads to.
    pub(super) fn meet_the_boss(
        &mut self,
        placing: RidleyPlacing,
        content: &Content,
        rules: &Rules,
    ) {
        let room = self.passage.doorway.and_then(|doorway| doorway.map_room);
        self.hud.0.boss_blanks_the_map(
            &rules.hud,
            &content.hud,
            ridley::encountered(placing.boss),
            room.map_or(self.passage.map_room.cell, |room| room.cell),
        );
    }
}

/// What a passage's frame brings the HUD's step outside play.
pub(super) const fn out_of_play(report: &world_doors::Report) -> OutOfPlayInputs<'_> {
    OutOfPlayInputs {
        mini_map_turned_on: report.minimap_on,
        new_area: NewArea::SameArea,
        dropped: Dropped::Nothing,
    }
}

/// What the HUD reads this frame: her moves, the energy Health leaves, and its neighbours.
fn hud_inputs<'a>(
    hud: &HudFrame,
    state: &super::State,
    samus: &samus_movement::Samus,
    after_moves: types::Position,
    health: samus_health::State,
    content: &'a Content,
) -> FrameInputs<'a> {
    let (select, cancel) = hud.buttons;
    let pose = samus.pose;
    FrameInputs {
        frame: state.frame.count,
        stock_at_start: hud.stock_at_start,
        controls: Controls {
            select,
            cancel,
            // The player controls her in this loop, so only facing the screen stops a switch.
            can_switch: pose.facing != Facing::Screen,
        },
        mini_map: MiniMapInputs {
            runs: true,
            turned_on: false,
            position: Pair {
                x: after_moves.x.pixel().0,
                y: after_moves.y.pixel().0,
            },
            room: state.passage.map_room,
            map: &content.map,
            map_collected: hud.map_collected,
        },
        mother_brain_falls: false,
        status: Status {
            energy: Energy {
                now: health.energy.whole.get(),
                capacity: hud.capacity,
            },
            reserve: hud.reserve,
            ammunition: hud.ammunition,
        },
        collected: None,
        // Neither the grapple beam nor stopped time is in this loop.
        muted_by: match pose.action {
            Action::Spinning => Some(Mute::SpinJump),
            Action::WallJumping => Some(Mute::WallJump),
            Action::AgainstWall
            | Action::Crouching
            | Action::Falling
            | Action::Jumping
            | Action::Landing(_)
            | Action::Moonwalking
            | Action::Morphing
            | Action::Rolling
            | Action::Running
            | Action::StandingUp
            | Action::Still
            | Action::TakingOff
            | Action::Turning(_)
            | Action::Unmorphing => None,
        },
        dropped_before_status_bar: Dropped::Nothing,
        dropped_after_status_bar: Dropped::Nothing,
        countdown: hud.countdown,
    }
}
