//! One whole frame of Samus, in any pose, or of a passage.
//!
//! A frame of play steps her health, the room's Ceres doors, her clip's look ahead, her moves,
//! her animation, the camera following her, the HUD, then the doors to other rooms. While a
//! passage is under way, the doors to other rooms step alone, the Ceres doors run on the frames
//! the passage runs the enemies, and the HUD takes its step outside play.
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
//!
//! Health plays first. Her animation reads the energy the frame's tail hit leaves, before the
//! frame's loss; no Ridley is in this loop, so that is the energy the frame starts with. The
//! HUD plays after her moves, reading her position after them and the energy Health leaves.
//!
//! The Ceres doors run before she moves, reading where she was at the end of the last frame;
//! her motion meets the boxes of those that were solid as the frame began. Fetching the next
//! room's contents lets the last room's doors go and places its own.

use super::animation::{self, AppearanceContent};
use super::camera_semantic;
use super::connected::inputs::{CameraInputs, Inputs, Lead, Screen};
use super::content::MovementContent;
use super::health_tail::HealthPin;
use super::hud::HudPin;
use super::room_loading::{self, LoadFault, Rooms};
use crate::state::owned::{AppearancePin, CameraPin, MovementPin};
use boss_ceres_ridley::{CeresDoor, Stage};
use canonical::model::{Model, Pinned};
use core::cell::Cell;
use core::num::Wrapping;
use interface_hud::{
    Ammunition, CancelButton, Controls, CountdownRuns, Dropped, Energy, FrameCount, FrameInputs,
    FrameOutputs, HerMap, HudPictures, MiniMapInputs, Mute, NewArea, OutOfPlayInputs, Reserve,
    SelectButton, Status, StockAtStart,
};
use samus_appearance::{Dash, Figure, MoveEvent};
use samus_movement::{Dashing, StandingAnimation, WallTest};
use types::{Action, Facing, Hitbox, Item, Pair, Parts, Pose, Set};
use world_camera::{RememberedPosition, ViewPosition};
use world_doors::{
    Activity, Arrival, COLOURS, ColourGroup, Doors, Finishing, Palette, RoomCall, Scene,
};

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
    /// The HUD's pictures.
    pub hud: HudPictures,
    /// Her map of the area she is in.
    pub map: HerMap,
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
    /// Health parameters.
    pub health: samus_health::Rules,
    /// HUD parameters.
    pub hud: interface_hud::Rules,
    /// The Ceres doors' parameters.
    pub ceres: boss_ceres_ridley::Rules,
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
    /// Where the next room sits on her map, or `None` when it is in another area, whose map
    /// this loop doesn't hold.
    pub map_room: Option<interface_hud::Room>,
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
    /// Her whole appearance State.
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
    /// Her energy, the warning and her protection.
    pub health: Model<HealthPin>,
    /// The HUD.
    pub hud: Model<HudPin>,
    /// Where her room sits on her map.
    pub map_room: interface_hud::Room,
    /// The room's Ceres doors, in its order of its enemies.
    pub ceres_doors: Vec<CeresDoor>,
    /// The game's count of finished frames, in eight bits, which the HUD reads as the frame
    /// begins.
    pub frame: FrameCount,
}

/// What one frame published.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outputs {
    /// Movement's complete outcome; none while a passage runs.
    pub movement: Option<samus_movement::Outcome>,
    /// Her appearance's sounds, in the order asked; none while a passage runs.
    pub sounds: Vec<samus_appearance::SoundRequest>,
    /// What the doors report.
    pub doors: world_doors::Report,
    /// Health's frame; none while a passage runs.
    pub health: Option<samus_health::PlayedFrame>,
    /// The HUD's frame of play; none while a passage runs.
    pub hud: Option<FrameOutputs>,
    /// Each Ceres door's run, in order; none on a frame the enemies don't run.
    pub ceres_doors: Vec<boss_ceres_ridley::Run>,
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
    /// She wears a suit, whose frames Health doesn't play.
    Suit,
    /// The door she touched leads into another area.
    NewArea,
    /// The HUD refused the frame: its scene only shows the countdown.
    Hud,
    /// The station's escape: the doors' shakes read the random numbers and the screen's
    /// shaking, which aren't in this loop.
    Escape,
}

impl State {
    /// Play one frame from `inputs`. The candidate commits only when every owner completed.
    ///
    /// # Errors
    /// Every [`Fault`] leaves State unchanged: missing pose content, an unknown pose, malformed
    /// room inputs, the escape with Ceres doors in the room ([`Fault::Escape`]), and a next
    /// room that doesn't load, such as one placing a Ceres actor of a later piece
    /// ([`LoadFault::LaterCeresActor`]).
    pub fn step(
        &mut self,
        inputs: &Inputs,
        passing: &Passing<'_>,
        hud: &HudFrame,
        content: &Content,
        rules: &Rules,
    ) -> Result<Outputs, Fault> {
        if matches!(self.doors, Doors::Passing(_)) {
            return self.pass(passing, content, rules);
        }
        inputs.geometry.validate().map_err(|_| Fault::Geometry)?;
        if inputs.items[Item::VariaSuit] || inputs.items[Item::GravitySuit] {
            return Err(Fault::Suit);
        }
        let health = self
            .health
            .0
            .play_frame(samus_health::TailImpact::Absent, &rules.health);
        let room = camera_room(&inputs.camera)?;
        let start = self.figure.0;
        // The doors run before she moves, but her motion meets them as they block when the
        // frame began.
        let ceres = run_doors(
            &self.ceres_doors,
            &self.movement.0,
            passing.ceres,
            &rules.ceres,
        )?;
        let world = WithDoors::new(&inputs.geometry, &self.ceres_doors);
        let (samus, outcome) = self.moves(inputs, &world, content, rules)?;
        let after = Figure {
            pose: samus.pose,
            sprite: outcome.sprite,
        };
        let mut appearance = self.appearance.0;
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
        if touched.is_some_and(|doorway| doorway.map_room.is_none()) {
            return Err(Fault::NewArea);
        }
        let mut hud_state = self.hud.0.clone();
        let hud_outputs = hud_state
            .frame_of_play(
                &rules.hud,
                &content.hud,
                &hud_inputs(hud, self, &samus, &outcome, health.state, content),
            )
            .map_err(|_| Fault::Hud)?;
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
            health: Model(health.state),
            hud: Model(hud_state),
            map_room: self.map_room,
            ceres_doors: ceres.iter().map(|run| run.door).collect(),
            frame: counted(self.frame, doors.report.finishing),
        };
        Ok(Outputs {
            movement: Some(outcome),
            sounds: shown.sounds,
            doors: doors.report,
            health: Some(health),
            hud: Some(hud_outputs),
            ceres_doors: ceres,
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
        let ceres = if exports.report.runs[Activity::Enemies] {
            run_doors(
                &self.ceres_doors,
                &self.movement.0,
                passing.ceres,
                &rules.ceres,
            )?
        } else {
            Vec::new()
        };
        let mut ceres_doors = if ceres.is_empty() {
            self.ceres_doors.clone()
        } else {
            ceres.iter().map(|run| run.door).collect()
        };
        if let Some(doorway) = self.doorway {
            let run = room_loading::Run {
                progress: passing.progress,
                ceres: passing.ceres,
            };
            room_loading::load(
                exports.report.call,
                doorway.through,
                run,
                &content.rooms,
                &mut palette,
            )
            .map_err(Fault::Load)?;
            // The next room's contents let the last room's doors go and place its own.
            if exports.report.call == Some(RoomCall::FetchContents) {
                ceres_doors = room_loading::ceres_doors(doorway.through, run, &content.rooms)
                    .map_err(Fault::Load)?;
            }
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
        self.ceres_doors = ceres_doors;
        self.frame = counted(self.frame, exports.report.finishing);
        self.hud.0.out_of_play(&OutOfPlayInputs {
            mini_map_turned_on: exports.report.minimap_on,
            new_area: NewArea::SameArea,
            dropped: Dropped::Nothing,
        });
        if self.doors == Doors::Play {
            if let Some(room) = self.doorway.and_then(|doorway| doorway.map_room) {
                self.map_room = room;
            }
            self.doorway = None;
        }
        Ok(Outputs {
            movement: None,
            sounds: Vec::new(),
            doors: exports.report,
            health: None,
            hud: None,
            ceres_doors: ceres,
        })
    }
}

/// What the HUD reads this frame: her moves, the energy Health leaves, and its neighbours.
fn hud_inputs<'a>(
    hud: &HudFrame,
    state: &State,
    samus: &samus_movement::Samus,
    outcome: &samus_movement::Outcome,
    health: samus_health::State,
    content: &'a Content,
) -> FrameInputs<'a> {
    let (select, cancel) = hud.buttons;
    let pose = samus.pose;
    FrameInputs {
        frame: state.frame,
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
                x: outcome.position_after_moves.x.pixel().0,
                y: outcome.position_after_moves.y.pixel().0,
            },
            room: state.map_room,
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

/// The frame count after a frame: a finished frame counts, and a frame the doors restart the
/// count on counts from 0 (`FinishedFrames`).
const fn counted(frame: FrameCount, finishing: Finishing) -> FrameCount {
    match finishing {
        Finishing::Overrun => frame,
        Finishing::Finished => FrameCount(frame.0.wrapping_add(1)),
        Finishing::Restarts(_) => FrameCount(1),
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
pub(super) fn animation_inputs(
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
        activities: inputs.activities,
        shoot_held: inputs.held.any(types::button::SHOOT),
        charge: inputs.charge,
    }
}

/// One run of each of the room's Ceres doors, in its order of its enemies, reading where she
/// was at the end of the last frame.
fn run_doors(
    doors: &[CeresDoor],
    her: &samus_movement::Samus,
    stage: Stage,
    rules: &boss_ceres_ridley::Rules,
) -> Result<Vec<boss_ceres_ridley::Run>, Fault> {
    if stage == Stage::Escape && !doors.is_empty() {
        return Err(Fault::Escape);
    }
    let inputs = boss_ceres_ridley::Inputs {
        her: Pair {
            x: her.position.x.pixel(),
            y: her.position.y.pixel(),
        },
        stage,
        // Outside the escape no door reads the random number or the screen's shaking.
        random: 0,
        shake_frames_left: 0,
    };
    Ok(doors.iter().map(|door| door.run(&inputs, rules)).collect())
}

/// The room's geometry, with the Ceres doors that block her after its other solid actors.
///
/// The original meets the boxes in slot order, and the first that blocks her decides the move.
/// The geometry's boxes keep no slots, so the doors go last: the admission accepts only rooms
/// whose other solid actors sit in slots before the doors'.
struct WithDoors<'a> {
    geometry: &'a super::world::MovementWorld,
    solid: Vec<Hitbox>,
}

impl<'a> WithDoors<'a> {
    fn new(geometry: &'a super::world::MovementWorld, doors: &[CeresDoor]) -> Self {
        let blocking = doors.iter().filter(|door| door.phase.solid());
        Self {
            geometry,
            solid: types::World::solid_enemies(geometry)
                .iter()
                .cloned()
                .chain(blocking.map(|door| door.hitbox()))
                .collect(),
        }
    }
}

impl types::World for WithDoors<'_> {
    fn block(&self, cell: types::Cell) -> types::Block {
        types::World::block(self.geometry, cell)
    }

    fn lets_through(&self, cell: types::Cell, way: types::Way) -> bool {
        types::World::lets_through(self.geometry, cell, way)
    }

    fn solid_enemies(&self) -> &[Hitbox] {
        &self.solid
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

#[cfg(test)]
mod tests;
