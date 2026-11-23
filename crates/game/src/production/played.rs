//! One whole frame of Samus, in any pose: her clip's look ahead, her moves, her animation, then
//! the camera following her.
//!
//! Each model is stepped once through its own whole-frame entry. Movement reports what animation
//! reads "as her moves leave her, before the pose change", so stepping movement whole and then
//! animation gives the values the original reads between them; no pause inside either model is
//! needed. The frame commits all three owners together or none.

use super::animation::{self, AppearanceContent};
use super::camera_semantic;
use super::connected::inputs::{CameraInputs, Inputs, Lead, Screen};
use super::content::MovementContent;
use crate::state::owned::{AppearancePin, CameraPin, MovementPin};
use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use core::cell::Cell;
use core::num::Wrapping;
use samus_appearance::{Dash, Figure, MoveEvent};
use samus_movement::{Dashing, StandingAnimation, WallTest};
use types::{Pose, Set};

/// The figure she is drawn in as the frame starts.
pub struct FigurePin;
impl Pinned for FigurePin {
    type State = Figure;
    const NAME: &'static str = "production-played-figure";
    const SHAPE: u64 = 0x5d1e_2c4a_93b0_7f16;
}

/// Imported pose bodies, cannon headings, clips and run paces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    /// Pose bodies and cannon headings.
    pub movement: MovementContent,
    /// Clips by figure, and the run's paces.
    pub appearance: AppearanceContent,
}

/// The tuning each model reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Movement parameters.
    pub movement: samus_movement::Rules,
    /// Animation parameters.
    pub appearance: samus_appearance::Rules,
    /// Camera parameters.
    pub camera: super::CameraRules,
}

/// Samus's movement, her animation, the figure she shows and the camera, carried frame to frame.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// Her movement.
    pub movement: Model<MovementPin>,
    /// Her animation's playhead and boost.
    pub appearance: Model<AppearancePin>,
    /// The figure she is drawn in.
    pub figure: Model<FigurePin>,
    /// The camera.
    pub camera: Model<CameraPin>,
}

/// What one frame published.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outputs {
    /// Movement's complete outcome.
    pub movement: samus_movement::Outcome,
    /// The sound her animation asked for, if any.
    pub sound: Option<types::AnimationSound>,
}

/// Why a frame was not played. State is unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Movement has no situation for her pose.
    Pose(Pose),
    /// A pose she takes has no imported body.
    Body(Pose),
    /// A figure she shows has no imported clip.
    Clip(Figure),
    /// The room's camera settings are malformed.
    CameraRoom,
    /// The room's geometry is malformed.
    Geometry,
}

impl State {
    /// Play one frame from `inputs`. The candidate commits only when every owner completed.
    ///
    /// # Errors
    /// Missing pose content, an unknown pose or malformed room inputs leave State unchanged.
    pub fn step(
        &mut self,
        inputs: &Inputs,
        content: &Content,
        rules: &Rules,
    ) -> Result<Outputs, Fault> {
        inputs.geometry.validate().map_err(|_| Fault::Geometry)?;
        let room = camera_room(&inputs.camera)?;
        let start = self.figure.0;
        let clip = content.appearance.get(start).ok_or(Fault::Clip(start))?;
        let ahead = super::appearance::ahead(&self.appearance.0, clip);

        let mut samus = self.movement.0;
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
            cinematic_active: matches!(
                inputs.cinematic,
                samus_appearance::CinematicActivity::Active
            ),
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
            .step(&frame, &inputs.geometry, &rules.movement)
            .map_err(|samus_movement::UnknownPose(pose)| Fault::Pose(pose))?;
        if let Some(pose) = missing_body.get() {
            return Err(Fault::Body(pose));
        }

        let after = Figure {
            pose: samus.pose,
            sprite: outcome.sprite,
        };
        let mut appearance = self.appearance.0;
        let sound = animation::advance(
            &mut appearance,
            &content.appearance,
            &rules.appearance,
            &animation_inputs(start, after, &outcome, inputs),
        )
        .map_err(|fault| match fault {
            animation::ClipFault::Missing(figure) | animation::ClipFault::Conflicting(figure) => {
                Fault::Clip(figure)
            }
        })?;

        let mut camera = self.camera.0;
        camera_semantic::step(
            &mut camera,
            rules.camera.get(),
            &world_camera::Frame {
                samus: camera_semantic::completed_samus(&samus, &outcome),
                room: &room,
                lead: lead(inputs.camera.lead),
                time_stopped: false,
            },
        );

        *self = Self {
            movement: Model(samus),
            appearance: Model(appearance),
            figure: Model(after),
            camera: Model(camera),
        };
        Ok(Outputs {
            movement: outcome,
            sound,
        })
    }
}

/// What animation reads of this frame's moves.
fn animation_inputs(
    start: Figure,
    after: Figure,
    outcome: &samus_movement::Outcome,
    inputs: &Inputs,
) -> samus_appearance::Inputs {
    let mut moves = Set::default();
    moves[MoveEvent::StandingRestart] = outcome.standing_animation == StandingAnimation::Restarts;
    moves[MoveEvent::WallJumpReadied] = outcome.wall_test == WallTest::Readied;
    moves[MoveEvent::HandedOver] = outcome.handed_over;
    moves[MoveEvent::DashLost] = outcome.dash_lost;
    samus_appearance::Inputs {
        figure: start,
        figure_after: after,
        moves,
        dash: match (outcome.dash_began, outcome.dashing_after_moves) {
            (true, _) => Dash::Began,
            (false, Dashing::On) => Dash::On,
            (false, Dashing::Off) => Dash::Off,
        },
        run_held: inputs.held.any(types::button::RUN),
        heading_down: outcome.heading_down,
        vertical_speed: outcome.vertical_speed_after_moves,
        energy: inputs.energy,
        items: inputs.items,
        liquid: inputs.liquid,
        sounds_waiting: inputs.sounds_waiting,
    }
}

fn camera_room(input: &CameraInputs) -> Result<world_camera::Room, Fault> {
    let [width, height] = input.size;
    let [normal, rising] = input.heights;
    world_camera::Room::new(
        input
            .screens
            .iter()
            .map(|screen| match screen {
                Screen::Open => types::ScreenKind::Open,
                Screen::Closed => types::ScreenKind::Closed,
                Screen::Short => types::ScreenKind::Short,
            })
            .collect(),
        types::Pair {
            x: usize::from(width),
            y: usize::from(height),
        },
        world_camera::Heights {
            while_not_rising: Wrapping(normal),
            while_rising: Wrapping(rising),
        },
    )
    .map_err(|_| Fault::CameraRoom)
}

const fn lead(lead: Lead) -> world_camera::LeadSetting {
    match lead {
        Lead::Ahead => world_camera::LeadSetting::Ahead,
        Lead::Left => world_camera::LeadSetting::KeepLeft,
        Lead::Right => world_camera::LeadSetting::KeepRight,
        Lead::Near => world_camera::LeadSetting::Near,
    }
}
