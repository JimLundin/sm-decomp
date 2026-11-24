//! Her own frame: her clip's look ahead, her moves, then her animation from what they report.
//!
//! Movement reports what animation reads "as her moves leave her, before the pose change", so
//! stepping movement whole and then animation gives the values the original reads between
//! them; no pause inside either model is needed.

use super::super::animation;
use super::super::connected::inputs::Inputs;
use super::{Content, Fault, Rules};
use crate::state::owned::{AppearancePin, MovementPin};
use canonical::model::{Model, Pinned};
use core::cell::Cell;
use core::num::Wrapping;
use samus_appearance::{Ahead, Dash, Figure, MissingClip, MoveEvent};
use samus_movement::{Dashing, StandingAnimation, WallTest};
use types::{Pose, Set};

/// The figure she is drawn in as the frame starts.
pub struct FigurePin;
impl Pinned for FigurePin {
    type State = Figure;
    const NAME: &'static str = "production-played-figure";
    const SHAPE: u64 = 0x5d1e_2c4a_93b0_7f16;
}

/// Her movement, her animation and the figure she shows, carried frame to frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// Her movement.
    pub movement: Model<MovementPin>,
    /// Her whole appearance State.
    pub appearance: Model<AppearancePin>,
    /// The figure she is drawn in.
    pub figure: Model<FigurePin>,
}

impl State {
    /// While the pad holds her, her frame changes nothing of her.
    pub(super) const fn held(&self) -> Her {
        Her {
            samus: self.movement.0,
            outcome: None,
            appearance: self.appearance.0,
            figure: self.figure.0,
            sounds: Vec::new(),
        }
    }

    /// Her State after a frame of play: `samus` as the frame leaves her, and what `her` frame
    /// shows.
    pub(super) const fn played(&mut self, samus: samus_movement::Samus, her: &Her) {
        self.movement = Model(samus);
        self.appearance = Model(her.appearance);
        self.figure = Model(her.figure);
    }
}

impl super::State {
    /// Her own frame: her moves, then her animation from what they report.
    pub(super) fn her_frame(
        &self,
        inputs: &Inputs,
        content: &Content,
        rules: &Rules,
    ) -> Result<Her, Fault> {
        let start = self.samus.figure.0;
        let world = self.station.world(&self.passage.room.geometry);
        let (samus, outcome) = self.moves(inputs, &world, content, rules)?;
        let after = Figure {
            pose: samus.pose,
            sprite: outcome.sprite,
        };
        let mut appearance = self.samus.appearance.0;
        let mut seen = animation_inputs(start, after, &outcome, inputs);
        // The energy the tail hit leaves, before the frame's loss: with no tail, as it starts.
        seen.energy = self.health.0.energy.whole.get();
        let shown = animation::play(
            &mut appearance,
            &content.appearance,
            &rules.appearance,
            &animation::frame_of_play(seen),
        )
        .map_err(|fault| match fault {
            animation::ClipFault::Missing(figure) | animation::ClipFault::Conflicting(figure) => {
                Fault::Clip(figure)
            }
        })?;
        Ok(Her {
            samus,
            outcome: Some(outcome),
            appearance,
            figure: after,
            sounds: shown.sounds,
        })
    }

    /// Her moves, after her clip's look ahead from the figure she starts the frame in.
    fn moves(
        &self,
        inputs: &Inputs,
        world: &dyn types::World,
        content: &Content,
        rules: &Rules,
    ) -> Result<(samus_movement::Samus, samus_movement::Outcome), Fault> {
        let start = self.samus.figure.0;
        let ahead = Ahead::of(&self.samus.appearance.0, start, &content.appearance)
            .map_err(|MissingClip(figure)| Fault::Clip(figure))?;
        let mut samus = self.samus.movement.0;
        let missing_body = Cell::new(None);
        let body = |pose: Pose| {
            content.movement.radius(pose).unwrap_or_else(|| {
                missing_body.set(Some(pose));
                types::Pair {
                    x: Wrapping(0),
                    y: Wrapping(0),
                }
            })
        };
        let heading = |pose| content.movement.cannon_heading(pose);
        let frame = samus_movement::Frame {
            sprite_before_movement: start.sprite,
            cinematic_active: inputs.activities[samus_appearance::Activity::Cinematic],
            pad: inputs.held,
            pressed: inputs.pressed,
            items: inputs.items,
            spin_turned_far: ahead.turned_far,
            elevator_moving: false,
            parity: inputs.parity,
            animation_ends: ahead.end,
            body: &body,
            cannon_heading: &heading,
        };
        let outcome = samus
            .step(&frame, world, &rules.movement)
            .map_err(|samus_movement::UnknownPose(pose)| Fault::Pose(pose))?;
        missing_body
            .get()
            .map_or(Ok((samus, outcome)), |pose| Err(Fault::Body(pose)))
    }
}

/// Her own frame's results: her movement, its outcome, her appearance, the figure she shows and
/// the sounds it asks for.
pub(super) struct Her {
    pub(super) samus: samus_movement::Samus,
    /// None while the pad holds her.
    pub(super) outcome: Option<samus_movement::Outcome>,
    appearance: samus_appearance::AppearanceState,
    figure: Figure,
    pub(super) sounds: Vec<samus_appearance::SoundOffer>,
}

impl Her {
    /// Where her moves leave her, which the mini-map reads.
    pub(super) fn after_moves(&self) -> types::Position {
        self.outcome
            .as_ref()
            .map_or(self.samus.position, |outcome| outcome.position_after_moves)
    }
}

/// What animation reads of this frame's moves.
pub(in crate::production) fn animation_inputs(
    start: Figure,
    after: Figure,
    outcome: &samus_movement::Outcome,
    inputs: &Inputs,
) -> samus_appearance::Inputs {
    let mut moves = Set::default();
    moves[MoveEvent::StandingRestart] = outcome.standing_animation == StandingAnimation::Restarts;
    moves[MoveEvent::WallJumpReadied] = outcome.wall_test == WallTest::Readied;
    moves[MoveEvent::HandedOver] = outcome.handed_over;
    samus_appearance::Inputs {
        figure: start,
        figure_after: after,
        moves,
        dash: dash(
            outcome.dashing_after_moves,
            outcome.dash_began,
            outcome.dash_lost,
        ),
        run_held: inputs.held.any(types::button::RUN),
        heading_down: outcome.heading_down,
        vertical_speed: outcome.vertical_speed_after_moves,
        energy: inputs.energy,
        items: inputs.items,
        liquid: inputs.liquid,
        sounds_waiting: inputs.sounds_waiting,
        activities: inputs.activities,
        shoot_held: inputs.held.any(types::button::SHOOT),
        charge: inputs.charge,
    }
}

/// The dash as the clip moves on, from movement's report of her moves.
///
/// Whether she is dashing after her moves decides first: movement reports a dash begun into a
/// wall as begun and lost with her not dashing after her moves, and the original cancels it before
/// the clip moves on, so it is stopped.
pub(super) const fn dash(dashing_after_moves: Dashing, began: bool, lost: bool) -> Dash {
    match (dashing_after_moves, began, lost) {
        (Dashing::Off, _, true) => Dash::Stopped,
        (Dashing::Off, _, false) => Dash::Off,
        (Dashing::On, true, true) => Dash::BeganAndLost,
        (Dashing::On, true, false) => Dash::Began,
        (Dashing::On, false, true) => Dash::Lost,
        (Dashing::On, false, false) => Dash::On,
    }
}
