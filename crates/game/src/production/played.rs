//! One whole frame of Samus, in any pose: her clip's look ahead, her moves, her animation, the
//! camera following her, then the doors; or, while a passage is under way, the doors alone.
//!
//! Each model is stepped once through its own whole-frame entry. Movement reports what animation
//! reads "as her moves leave her, before the pose change", so stepping movement whole and then
//! animation gives the values the original reads between them; no pause inside either model is
//! needed. The frame commits every owner together or none.
//!
//! A frame whose moves reach a door to a room plays out whole, and the passage starts on the
//! next. While it runs, her motion, her animation and the camera don't: the passage moves her
//! position, the view and the colours, and nothing else of theirs changes. As each of the
//! passage's calls is done, the room's loading loads its part of the next room.

use super::animation::{self, AppearanceContent};
use super::camera_semantic;
use super::connected::inputs::{CameraInputs, Inputs, Lead, Screen};
use super::content::MovementContent;
use super::room_loading::{self, LoadFault, Rooms};
use crate::state::owned::{AppearancePin, CameraPin, MovementPin};
use canonical::model::{Model, Pinned};
use core::cell::Cell;
use core::num::Wrapping;
use samus_appearance::{Dash, Figure, MoveEvent};
use samus_movement::{Dashing, StandingAnimation, WallTest};
use types::{Pair, Parts, Pose, Set};
use world_camera::{RememberedPosition, ViewPosition};
use world_doors::{Arrival, COLOURS, ColourGroup, Doors, Palette, Scene};

/// The figure she is drawn in as the frame starts.
pub struct FigurePin;
impl Pinned for FigurePin {
    type State = Figure;
    const NAME: &'static str = "production-played-figure";
    const SHAPE: u64 = 0x5d1e_2c4a_93b0_7f16;
}

/// Imported pose bodies, cannon headings, clips, run paces and the HUD's colours.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    /// Pose bodies and cannon headings.
    pub movement: MovementContent,
    /// Clips by figure, and the run's paces.
    pub appearance: AppearanceContent,
    /// Which group each of the screen's colours is in, for the passage's fades.
    pub colour_groups: [ColourGroup; COLOURS],
    /// The rooms, tilesets and sprite colours the passages load.
    pub rooms: Rooms,
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
    /// Passage parameters.
    pub doors: world_doors::Rules,
}

/// A door to another room, from the room's content: the cell she touches, the doorway it
/// leads through, where she arrives, and whether the room she leaves or the next one keeps the
/// items dark.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Doorway {
    /// The door's cell.
    pub cell: types::Cell,
    /// The doorway she goes through, which names the next room.
    pub through: data::id::DoorwayId,
    /// Where the view and she arrive.
    pub arrival: Arrival,
    /// Either room keeps the items dark through the passage.
    pub items_dark: bool,
}

/// What a frame brings the doors from owners outside this loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Passing<'a> {
    /// The room's doors to other rooms.
    pub doorways: &'a [Doorway],
    /// The platform's: the passage's work in progress is done by this frame's end.
    pub work_done: bool,
    /// Audio's answers.
    pub audio: PassageAudio,
    /// The escape's: its timer is running.
    pub timer_running: bool,
    /// Her progress, which chooses the next room's version, her suit and her beams.
    pub progress: &'a run_progress::Progress,
    /// The Ceres station's stage, for the colours of the doors the next room places.
    pub ceres: boss_ceres_ridley::Stage,
}

/// What audio answers the passage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PassageAudio {
    /// Every sound request so far has been sent.
    pub sounds_sent: bool,
    /// No track waits in the queue of tracks.
    pub music_ready: bool,
}

/// Samus's movement, her animation, the figure she shows, the camera and the doors, carried
/// frame to frame.
///
/// The doors' State and the colours are the doors model's own types, which have no saved form
/// yet, so this State is carried but not checkpointed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// Her movement.
    pub movement: Model<MovementPin>,
    /// Her animation's playhead and boost.
    pub appearance: Model<AppearancePin>,
    /// The figure she is drawn in.
    pub figure: Model<FigurePin>,
    /// The camera.
    pub camera: Model<CameraPin>,
    /// Play, or a passage under way.
    pub doors: Doors,
    /// The door she touched, from the touch until play goes on.
    pub doorway: Option<Doorway>,
    /// The screen's colours and the colours they head for.
    pub palette: Palette,
}

/// What one frame published.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outputs {
    /// Movement's complete outcome; none while a passage runs.
    pub movement: Option<samus_movement::Outcome>,
    /// The sound her animation asked for, if any.
    pub sound: Option<types::AnimationSound>,
    /// What the doors report.
    pub doors: world_doors::Report,
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
    /// The next room did not load from the content.
    Load(LoadFault),
}

impl State {
    /// Play one frame from `inputs`. The candidate commits only when every owner completed.
    ///
    /// # Errors
    /// Missing pose content, an unknown pose or malformed room inputs leave State unchanged.
    pub fn step(
        &mut self,
        inputs: &Inputs,
        passing: &Passing<'_>,
        content: &Content,
        rules: &Rules,
    ) -> Result<Outputs, Fault> {
        if matches!(self.doors, Doors::Passing(_)) {
            return self.pass(passing, content, rules);
        }
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

        let touched = passing
            .doorways
            .iter()
            .find(|doorway| {
                outcome
                    .reached
                    .iter()
                    .any(|touch| touch.cell == doorway.cell)
            })
            .copied();
        let doors = self.doors.step(
            scene(&samus, &camera, &self.palette),
            &doors_inputs(touched.is_some(), touched, passing, content),
            &rules.doors,
        );

        *self = Self {
            movement: Model(samus),
            appearance: Model(appearance),
            figure: Model(after),
            camera: Model(camera),
            doors: doors.doors,
            doorway: touched,
            palette: doors.scene.palette,
        };
        Ok(Outputs {
            movement: Some(outcome),
            sound,
            doors: doors.report,
        })
    }

    /// A frame of a passage: the doors alone, moving her position, the view, where the camera
    /// last saw her and the colours, then the room's loading of what the doors' call names.
    fn pass(
        &mut self,
        passing: &Passing<'_>,
        content: &Content,
        rules: &Rules,
    ) -> Result<Outputs, Fault> {
        let camera = self.camera.0;
        let exports = self.doors.step(
            scene(&self.movement.0, &camera, &self.palette),
            &doors_inputs(false, self.doorway, passing, content),
            &rules.doors,
        );
        let Scene {
            her,
            seen,
            camera: view,
            mut palette,
        } = exports.scene;
        if let Some(doorway) = self.doorway {
            room_loading::load(
                exports.report.call,
                doorway.through,
                room_loading::Run {
                    progress: passing.progress,
                    ceres: passing.ceres,
                },
                &content.rooms,
                &mut palette,
            )
            .map_err(Fault::Load)?;
        }
        let kept = camera.seen();
        self.movement.0.position = her;
        self.camera.0 = camera
            .with_view(ViewPosition(view))
            .with_remembered(RememberedPosition(Pair {
                x: types::fixed(seen.x, kept.x.sub()),
                y: types::fixed(seen.y, kept.y.sub()),
            }));
        self.palette = palette;
        self.doors = exports.doors;
        if self.doors == Doors::Play {
            self.doorway = None;
        }
        Ok(Outputs {
            movement: None,
            sound: None,
            doors: exports.report,
        })
    }
}

/// What the passage moves, as the frame before left it.
fn scene(samus: &samus_movement::Samus, camera: &world_camera::Camera, palette: &Palette) -> Scene {
    Scene {
        her: samus.position,
        seen: Pair {
            x: camera.seen().x.pixel(),
            y: camera.seen().y.pixel(),
        },
        camera: camera.position(),
        palette: *palette,
    }
}

fn doors_inputs(
    touched: bool,
    doorway: Option<Doorway>,
    passing: &Passing<'_>,
    content: &Content,
) -> world_doors::Inputs {
    world_doors::Inputs {
        touched,
        work_done: passing.work_done,
        sounds_sent: passing.audio.sounds_sent,
        music_ready: passing.audio.music_ready,
        timer_running: passing.timer_running,
        items_dark: doorway.is_some_and(|doorway| doorway.items_dark),
        doorway: doorway.map(|doorway| doorway.arrival),
        groups: content.colour_groups,
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
