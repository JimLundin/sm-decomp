//! One whole frame of Samus, in any pose, or of a passage.
//!
//! A frame of play steps her health, the room's enemies (the station's actors), her clip's look
//! ahead and her moves, her animation, the camera following her, the HUD, the room's quake, then
//! the doors to other rooms. While a passage is under way, the doors to other rooms step alone,
//! the station's actors run on the frames the passage runs the enemies, and the HUD takes its
//! step outside play.
//!
//! This file holds only that order. Each owner's State, its step and what it reads of the others
//! live in its own module: [`samus`] her moves and animation, `camera` the camera, `health`
//! and `hud` her energy and the HUD, [`passage`] her room, its doors to other rooms and the
//! passage through them, [`station`] the order of the Ceres station's actors and their State,
//! with `ridley` Ridley and the baby, `ceres_doors` the Ceres doors and `shaft` the shaft's pad
//! and lights, [`quake`] the room's quake, the shakes the doors start and the stops,
//! [`frame`] the counts of frames and the random number, and [`arrival`] her
//! arrival as a new game begins. The screen's colours are the doors model's [`Palette`], which
//! the passage, the lights and the baby each set in turn. The fixture every owner's tests build
//! State from is `tests`. Ahead of room play, [`front_end`] is which part of the game runs, from
//! the console's power-on. Ahead of the new game, with no room, [`flight`] is the gunship's
//! flight to Ceres, and [`intro`] the intro's last pages.
//!
//! Each model is stepped once through its own whole-frame entry. The frame commits every owner
//! together or none.
//!
//! A frame whose moves reach a door to a room plays out whole, and the passage starts on the
//! next. While it runs, her motion, her animation and the camera don't: the passage moves her
//! position, the view and the colours, and nothing else of theirs changes. As each of the
//! passage's calls is done, the room's loading loads its part of the next room; building it
//! replaces her room: what her moves meet, its doors to other rooms and the camera's room.
//!
//! Her movement's frame is two steps: her moves, then her reaction to the frame's hit. The
//! station's actors run before her moves, and Health between the two steps. Every hitter reads
//! her position, body and protection as the frame starts. Health and her reaction read the
//! same hit from the tail or the first touching vent. The Ceres doors read where she was at
//! the end of the last frame, and her motion meets the boxes of the
//! Ceres doors that were solid as the frame began. Her appearance reads the energy the frame
//! starts with against the energy the hit leaves, and whether her reaction started a push, and
//! sets her row of the screen's colours while her hurt colours pulse; the HUD plays after her
//! moves, reading her position after them and the energy Health leaves.
//! While the elevator's pad holds her, her moves and her animation don't run, and the pad sets
//! her height after her frame. Each frame of work begins by drawing the game's random number
//! once.
//!
//! While a new game's start is made ready no frame of play runs; on the frame it is ready she
//! arrives. Each frame of the fade in after is a frame of play, and the fade takes its step
//! after it.

pub mod arrival;
mod camera;
mod ceres_doors;
pub mod clock_picture;
pub mod departure;
pub mod discovery;
pub mod explosion;
pub mod flashback;
pub mod flight;
pub mod frame;
pub mod front_end;
mod health;
mod hud;
pub mod intro;
pub mod intro_missiles;
pub mod intro_rooms;
pub mod landing_site;
pub mod passage;
pub mod quake;
mod ridley;
pub mod samus;
mod shaft;
mod start;
pub mod station;
pub mod warning;
pub mod warning_drawing;
pub mod zebes;

pub use hud::HudFrame;
pub(super) use samus::animation_inputs;

use crate::production::CameraRules;
use crate::production::animation::AppearanceContent;
use crate::production::connected::inputs::Inputs;
use crate::production::content::MovementContent;
use crate::production::health_tail::HealthPin;
use crate::production::hud::HudPin;
use crate::production::room_loading::{self, LoadFault, Rooms};
use crate::production::shaft::LightsFault;
use crate::state::owned::CameraPin;
use boss_ceres_ridley::{Lights, RidleyContent, RidleyPlacing, RidleyRun};
use canonical::model::Model;
use interface_hud::{FrameOutputs, HerMap, HudPictures};
use passage::Passing;
use quake::QuakePin;
use samus_appearance::Figure;
use types::{Item, Pose};
use world_doors::{Activity, COLOURS, ColourGroup, Doors, Palette, RoomCall, Scene};

/// Imported pose bodies, cannon headings, clips, run paces and the HUD's colours.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    /// Imported timer sprite figures for room drawing.
    pub clock_figures: Option<data::assets::countdown::Figures>,
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
    /// The checked opening HUD of a fresh Ceres start, when this run supports one.
    pub opening_hud: Option<(interface_hud::OpeningFrame, interface_hud::OpeningPreview)>,
    /// Her map of the area she is in.
    pub map: HerMap,
    /// Her place across the new game's start room, x in pixels.
    pub start_x: types::Px,
    /// Ceres Ridley's eye's steps and his body's colour sets.
    pub ridley: RidleyContent,
    /// The room warning artwork, when admitted for this file.
    pub warning: Option<warning::Content>,
    /// The whole energy his tail's hit is worth, imported as his tail's own.
    pub tail_worth: samus_health::WholeEnergy,
}

/// The tuning each model reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Movement parameters.
    pub movement: samus_movement::Rules,
    /// Animation parameters.
    pub appearance: samus_appearance::Rules,
    /// Camera parameters.
    pub camera: CameraRules,
    /// Passage parameters.
    pub doors: world_doors::Rules,
    /// Health parameters.
    pub health: samus_health::Rules,
    /// HUD parameters.
    pub hud: interface_hud::Rules,
    /// The Ceres doors' and Ceres Ridley's parameters.
    pub ceres: boss_ceres_ridley::Rules,
    /// The arrival's parameters.
    pub arrival: screens_arrival::Rules,
}

/// Each owner's State, carried frame to frame.
///
/// The doors' State and the colours are the doors model's own types, which have no saved form
/// yet, so this State is carried but not checkpointed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// The timer sprites held in the prepared and published pictures.
    pub clock_picture: clock_picture::ClockPicture,
    /// Her movement, her appearance and the figure she is drawn in.
    pub samus: samus::State,
    /// The camera.
    pub camera: Model<CameraPin>,
    /// Her energy, the warning and her protection.
    pub health: Model<HealthPin>,
    /// The HUD.
    pub hud: Model<HudPin>,
    /// The run's inventory and persistent progress, carried across rooms.
    pub progress: run_progress::Progress,
    /// Her room, play or a passage under way, and the door she touched.
    pub passage: passage::State,
    /// The screen's colours and the colours they head for.
    pub palette: Palette,
    /// The Ceres station's doors, pad and cover, lights, Ridley and steam vents.
    pub station: station::State,
    /// The room's quake: the shake under way and the phase.
    pub quake: Model<QuakePin>,
    /// The counts of finished frames and the game's random number.
    pub frame: frame::State,
    /// Her arrival as a new game begins: the preparation, the fade in, or play.
    pub arrival: arrival::State,
}

/// What one frame published.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outputs {
    /// Movement's complete outcome; none while a passage runs.
    pub movement: Option<samus_movement::Outcome>,
    /// Her appearance's sounds, in the order asked; none while a passage runs.
    pub sounds: Vec<samus_appearance::SoundOffer>,
    /// What the doors report.
    pub doors: world_doors::Report,
    /// Health's frame; none while a passage runs.
    pub health: Option<samus_health::PlayedFrame>,
    /// The HUD's frame of play; none while a passage runs.
    pub hud: Option<FrameOutputs>,
    /// The fixed status-bar row published by new-file room setup.
    pub opening_hud: Option<interface_hud::UpperRow>,
    /// Each Ceres door's run, in order; none on a frame the enemies don't run.
    pub ceres_doors: Vec<boss_ceres_ridley::Run>,
    /// What the shaft's lights show; none on a frame its overlay doesn't run.
    pub lights: Option<Lights>,
    /// Ridley's run, with the baby's cry; none on a frame he doesn't run.
    pub ridley: Option<RidleyRun>,
    /// The Ceres warning requests for drawing, audio and progress.
    pub warning: Option<warning::Output>,
    /// The steam hit offered to her movement and health.
    pub steam_hit: Option<types::Hit>,
    /// His placing, with the music's stop and the shake's stop it asks for; none on a frame
    /// that doesn't place him.
    pub ridley_placed: Option<RidleyPlacing>,
    /// The room quake's update: the layers' offsets and the actors' request; none while a
    /// passage runs.
    pub quake: Option<world_quake::Effects>,
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
    /// The HUD refused the frame: the escape countdown starts on a frame whose scene only shows
    /// it.
    Hud,
    /// The new file's room setup has no opening HUD provider.
    HudStartup,
    /// The fresh file's Samus could not be constructed.
    NewGame,
    /// The station's escape in the shaft: a later piece owns the shaft's overlay.
    Escape,
    /// The room's quake can't take the frame: a door's shake of a strength it has no size for,
    /// or an update it refuses.
    Quake,
    /// The shaft's lights could not be shown.
    Lights(LightsFault),
    /// Ridley starts a run in an attack: a later part's.
    RidleyAttacks,
    /// Ridley begins or continues the fake retreat.
    RidleyGivesUp,
    /// The getaway pass needs Samus’s shove.
    RidleyGetawayPush,
    /// The next run belongs to the evacuation warning.
    RidleyWarning,
    /// A colour of the baby's content has more than 15 bits.
    BabyColours,
    /// Her suit's content is not sixteen colours of 15 bits.
    SuitColours,
}

impl State {
    /// Play one frame from `inputs`. The candidate commits only when every owner completed.
    /// Her room is State's own ([`passage::State::room`]); the geometry and camera room of
    /// `inputs` are not read.
    ///
    /// # Errors
    /// Every [`Fault`] leaves State unchanged: missing pose content, an unknown pose, malformed
    /// room inputs, the escape with the shaft's overlay in the room ([`Fault::Escape`]), a shake
    /// the room's quake can't take ([`Fault::Quake`]), Ridley at a later fight handoff,
    /// and a next room that doesn't load, such as one placing an
    /// actor no owner here places ([`LoadFault::OtherActors`]).
    pub fn step(
        &mut self,
        inputs: &Inputs,
        passing: &Passing,
        hud: &HudFrame,
        content: &Content,
        rules: &Rules,
    ) -> Result<Outputs, Fault> {
        if self.arrival.preparing() {
            return self.prepare(passing, content, rules);
        }
        if matches!(self.passage.doors, Doors::Passing(_)) {
            return self.pass(inputs, passing, content, rules);
        }
        let room = self.ordinary_room(inputs)?;
        let random = self.frame.drawn(passing);
        let mut palette = self.palette;
        // Every hitter reads her position, body and protection as the frame starts, as the
        // game's frame runs the enemies before her moves. Health reads their hit between
        // her moves and her reaction.
        let at_start = self.enemy_contact(content)?;
        let enemies = self.enemies(
            station::Running::Play(at_start.as_ref()),
            random,
            passing,
            content,
            rules,
            &mut palette,
        )?;
        let hit = enemies.contact();
        let between = samus::Between {
            played: (enemies, self.health_frame(hit, content, rules)),
            hit,
            energy: self.energy_the_hit_leaves(hit, content, rules),
        };
        let (her, (enemies, health)) = if self.station.pad_and_cover.holds_her() {
            // The pad holds her in the shaft, where no tail reaches her.
            (self.samus.held(), between.played)
        } else {
            self.her_frame(inputs, content, rules, |_| Ok(between))?
        };
        if let (Some(row), Some(colours)) =
            (room_loading::suit_row(&mut palette.screen), her.colours)
        {
            *row = colours.0;
        }
        let mut samus = her.samus;
        let pad_and_cover = self.station.pad(&mut samus, rules);
        let camera = self.followed(&room, &her, &samus, rules);
        let touched = self.passage.touched(&her)?;
        let hud = warning::countdown_input(hud, enemies.warning.as_ref())?;
        let (hud_state, hud_outputs) = self.hud_frame(
            &hud,
            &samus,
            her.after_moves(),
            health.state,
            content,
            rules,
        )?;
        let clock_picture = self
            .clock_picture
            .play(
                content.clock_figures.as_ref(),
                hud_outputs.countdown.picture,
                passing.publishes,
            )
            .ok_or(Fault::Hud)?;
        let mut quake = ridley::finish_getaway(&enemies, &mut samus, &mut palette);
        let shaken = quake::update(&mut quake)?;
        let doors = self.passage.in_play(
            touched,
            hud::countdown_begun(&hud_state),
            &passage::scene(&samus, &camera, &palette),
            passing,
            content,
            rules,
        );

        // Her room and where it sits on her map stay as they are through a frame of play.
        self.samus
            .played(samus, &her, samus::jump_button(inputs.held));
        self.station.pad_and_cover = pad_and_cover;
        self.camera = Model(camera);
        warning::record_progress(&mut self.progress, enemies.warning.as_ref());
        self.passage.doors = doors.doors;
        self.passage.doorway = touched;
        self.palette = doors.scene.palette;
        self.health = Model(health.state);
        self.hud = Model(hud_state);
        self.clock_picture = clock_picture;
        self.station.ran(&enemies);
        if let Some(vents) = enemies.vents {
            self.station.vents = vents;
        }
        self.quake = Model(quake);
        self.frame = self.frame.after(doors.report.finishing, random);
        self.arrival = self.arrival.after_play(rules);
        Ok(Outputs {
            movement: her.outcome,
            sounds: her.sounds,
            doors: doors.report,
            health: Some(health),
            hud: Some(hud_outputs),
            opening_hud: None,
            ceres_doors: enemies.doors,
            lights: enemies.lights,
            ridley: enemies.ridley,
            warning: enemies.warning,
            steam_hit: enemies.steam_hit,
            ridley_placed: None,
            quake: Some(shaken),
        })
    }

    fn ordinary_room(&self, inputs: &Inputs) -> Result<world_camera::Room, Fault> {
        self.passage
            .room
            .geometry
            .validate()
            .map_err(|_| Fault::Geometry)?;
        if inputs.items[Item::VariaSuit] || inputs.items[Item::GravitySuit] {
            return Err(Fault::Suit);
        }
        camera::room(&self.passage.room.camera)
    }

    /// A frame of a passage: the doors alone, moving her position, the view, where the camera
    /// last saw her and the colours, then the room's loading of what the doors' call names. The
    /// doors' runs on its frames of work start shakes, and building the next room stops the
    /// shake; the room's quake has no update.
    fn pass(
        &mut self,
        inputs: &Inputs,
        passing: &Passing,
        content: &Content,
        rules: &Rules,
    ) -> Result<Outputs, Fault> {
        let mut random = self.frame.drawn(passing);
        let mut before = self.palette;
        let early = if passage::brightening(self.passage.doors, passing) {
            Some(self.enemies(
                station::Running::Passage,
                random,
                passing,
                content,
                rules,
                &mut before,
            )?)
        } else {
            None
        };
        let exports = self.passage.doors.step(
            passage::scene(&self.samus.movement.0, &self.camera.0, &before),
            &passage::doors_inputs(
                false,
                hud::countdown_begun(&self.hud.0),
                self.passage.doorway,
                passing,
                content,
            ),
            &rules.doors,
        );
        let Scene {
            her,
            seen,
            camera: view,
            mut palette,
        } = exports.scene;
        let mut enemies = match early {
            Some(enemies) => enemies,
            None if exports.report.runs[Activity::Enemies] => self.enemies(
                station::Running::Passage,
                random,
                passing,
                content,
                rules,
                &mut palette,
            )?,
            None => station::Enemies::none(self.quake.0),
        };
        let mut moved = self.samus.movement.0;
        moved.position = her;
        let last_draw_jump = self.passage_jump(&mut moved, inputs, passing, &exports.report);
        let pad_and_cover = self.passage_pad(&mut moved, &exports.report, rules);
        let clock_picture = self
            .clock_picture
            .passage(
                content.clock_figures.as_ref(),
                self.hud.0.countdown,
                exports.report.runs[Activity::Timer],
                passing,
            )
            .ok_or(Fault::Hud)?;
        let loaded = self.load(exports.report.call, content, &mut palette, &mut random)?;
        let ridley_placed = loaded.actors.as_ref().and_then(|placed| placed.ridley);
        let mut quake = enemies.quake;
        // Room loading's effects set-up, inside the building, stops the shake.
        if exports.report.call == Some(RoomCall::BuildRoom) {
            quake::stop(&mut quake);
        }
        if let Some(placing) = ridley_placed {
            self.meet_the_boss(placing, content, rules);
            // His placing asks for the shake's stop.
            let boss_ceres_ridley::ShakeStop = placing.shake;
            quake::stop(&mut quake);
        }
        self.station
            .passed(&mut enemies, loaded.actors, pad_and_cover);
        self.quake = Model(quake);
        self.samus.movement.0 = moved;
        self.samus.last_draw_jump.0 = last_draw_jump;
        self.camera.0 = camera::passed(self.camera.0, view, seen);
        self.palette = palette;
        self.frame = self.frame.after(exports.report.finishing, random);
        self.clock_picture = clock_picture;
        self.hud.0.out_of_play(&hud::out_of_play(&exports.report));
        self.passage.passed(exports.doors, loaded.room);
        Ok(Outputs {
            movement: None,
            sounds: Vec::new(),
            doors: exports.report,
            health: None,
            hud: None,
            opening_hud: None,
            ceres_doors: enemies.doors,
            lights: enemies.lights,
            ridley: enemies.ridley,
            warning: enemies.warning,
            steam_hit: enemies.steam_hit,
            ridley_placed,
            quake: None,
        })
    }
    fn passage_jump(
        &self,
        moved: &mut samus_movement::Samus,
        inputs: &Inputs,
        passing: &Passing,
        report: &world_doors::Report,
    ) -> samus_movement::JumpButton {
        let mut last_draw_jump = self.samus.last_draw_jump.0;
        if passage::draws_jump(self.passage.doors, passing, report) {
            samus::passage_jump_draw(
                &mut moved.jump_held,
                &mut last_draw_jump,
                samus::jump_button(inputs.held),
            );
        }
        last_draw_jump
    }

    fn passage_pad(
        &self,
        moved: &mut samus_movement::Samus,
        report: &world_doors::Report,
        rules: &Rules,
    ) -> boss_ceres_ridley::PadAndCover {
        if report.runs[Activity::Projectiles] {
            self.station.pad(moved, rules)
        } else {
            self.station.pad_and_cover
        }
    }
}

#[cfg(test)]
mod tests;
