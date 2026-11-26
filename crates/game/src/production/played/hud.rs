//! The HUD's frame of play, its step outside play, and its meeting with the station's boss.
//!
//! The HUD plays after her moves, reading her position after them and the energy Health leaves.

use super::{Content, Fault, Rules};
use crate::production::ridley;
use boss_ceres_ridley::RidleyPlacing;
use interface_hud::{
    Ammunition, CancelButton, Controls, Countdown, CountdownInputs, CountdownRuns, Dropped, Energy,
    FinishedFrames, FrameInputs, FrameOutputs, MiniMapInputs, Mute, NewArea, OutOfPlayInputs,
    Reserve, SelectButton, StartTime, Status, StockAtStart,
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
    /// The escape's scene starts the countdown this frame at this time, or not.
    pub countdown_starts: Option<StartTime>,
}

impl HudFrame {
    pub(super) const fn during_elevator(mut self) -> Self {
        self.countdown = CountdownRuns::PresentationOnly;
        self.buttons = (SelectButton::NotPressed, CancelButton::NotPressed);
        self
    }
}

/// Whether an escape countdown has begun in `hud`: the doors read it to draw the countdown
/// through a passage and keep its colours lit.
pub(super) const fn countdown_begun(hud: &interface_hud::HudState) -> bool {
    matches!(hud.countdown, Countdown::Begun(_))
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

    /// Retain the ordinary room HUD and its completed-frame count when the elevator locks her.
    #[must_use]
    pub fn locked_hud(
        &self,
        content: &Content,
        frame: &HudFrame,
    ) -> (
        crate::production::blackout::LockedHud,
        crate::production::blackout::HudContent,
    ) {
        let centre = self.samus.movement.0.position;
        crate::production::blackout::LockedHud::from_ordinary(
            &self.hud,
            self.frame.count.0,
            (content.hud, content.map.clone(), self.passage.map_room),
            Pair {
                x: centre.x.pixel().0,
                y: centre.y.pixel().0,
            },
            self.health.0.energy.whole.get(),
            frame,
        )
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
            | Action::Leaping
            | Action::Moonwalking
            | Action::Morphing
            | Action::Recoiling
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
        countdown: CountdownInputs {
            runs: hud.countdown,
            start: hud.countdown_starts,
            // The count as the last frame left it: this frame's isn't counted yet.
            finished: FinishedFrames(state.frame.finished_frames),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{content, hud_frame, inputs, rules, state};
    use super::super::{State, arrival, passage};
    use super::{HudFrame, countdown_begun};
    use interface_hud::{
        BegunCountdown, Countdown, CountdownAge, CountdownPlace, FrameOutputs, StartTime, TimeLeft,
    };
    use types::Pair;

    fn frame_of_play(state: &mut State, hud: &HudFrame) -> FrameOutputs {
        let passing = passage::Passing {
            actor_fetch: crate::production::played::passage::ActorFetch::Place,
            enemy_tiles: crate::production::played::passage::EnemyTileTransfer::Waiting,
            shaft_background: crate::production::played::room_tiles::ShaftBackground::Pending,
            shaft_raster: crate::production::played::passage::ShaftRaster::Pending,
            work_begun: true,
            work_done: true,
            publishes: true,
            audio: passage::PassageAudio {
                shaft_reply: None,
                sounds_sent: true,
                music_ready: true,
            },
            random: None,
            start: arrival::Readiness::NotYet,
        };
        let outputs = state
            .step(
                &inputs(types::Buttons::default()),
                &passing,
                hud,
                &content(),
                &rules(),
            )
            .unwrap();
        outputs.hud.unwrap()
    }

    #[test]
    fn the_scenes_start_begins_the_countdown_the_doors_then_read() {
        let mut played = state(Vec::new());
        assert!(!countdown_begun(&played.hud.0));
        let start = StartTime::new(1, 0);
        frame_of_play(
            &mut played,
            &HudFrame {
                countdown_starts: start,
                ..hud_frame()
            },
        );
        assert!(countdown_begun(&played.hud.0));
    }

    #[test]
    fn a_running_countdown_takes_the_rhythm_of_the_count_of_finished_frames_state_carries() {
        let begun = BegunCountdown {
            time_left: TimeLeft::new(1, 0, 0).unwrap(),
            age: CountdownAge::new(16).unwrap(),
            place: CountdownPlace::new(Pair {
                x: 128 * 256,
                y: 128 * 256,
            })
            .unwrap(),
        };
        // The rhythm's first cell takes one hundredth and its second two.
        for (finished, left) in [(0, 5999), (1, 5998)] {
            let mut played = state(Vec::new());
            played.hud.0.countdown = Countdown::Begun(begun);
            played.frame.finished_frames = finished;
            frame_of_play(&mut played, &hud_frame());
            let Countdown::Begun(after) = played.hud.0.countdown else {
                panic!("the countdown stays begun");
            };
            assert_eq!(after.time_left.hundredths(), left);
        }
    }
}
