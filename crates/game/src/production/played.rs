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
pub mod play_time;
pub mod quake;
mod ridley;
mod room_bg1;
mod room_palette;
pub mod room_tiles;
pub mod room_video;
pub mod samus;
mod samus_picture;
mod samus_tiles;
pub mod save_snapshot;
pub mod shaft;
pub mod shaft_sound;
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
    /// Imported body maps and graphics for the locked elevator pose.
    pub standing: Option<crate::samus_appearance_bridge::elevator::Art>,
    /// Ordered tilt samples admitted with the escape shaft assets.
    pub shaft_track: Option<shaft::Track>,
    /// Sprite graphics written when the escape shaft places its overlay.
    pub shaft_overlay_art: Option<data::assets::shaft::Shaft>,
    /// The escape shaft effects tilemap selected by its room FX.
    pub shaft_effects: Vec<u16>,
    /// The haze, mode split and HUD scroll selected in the escape shaft.
    pub shaft_raster: Option<[data::assets::countdown::IndirectHdmaProgram; 3]>,
    /// Standard background and object graphics for the Ceres start.
    pub arrival_graphics: Option<data::assets::landing_site::Graphics>,
    /// The starting beam tiles and their cartridge source.
    pub arrival_beam_tiles: Option<(data::dma::BusAddress, Vec<u16>)>,
    /// The starting Ceres door's overlay art, if the room uses that actor.
    pub arrival_overlay: Option<Box<[u8; data::assets::shaft::OVERLAY_BYTES]>>,
    /// Pose bodies and cannon headings.
    pub movement: MovementContent,
    /// Clips by figure, and the run's paces.
    pub appearance: AppearanceContent,
    /// Imported body maps of the ordinary drawing.
    pub samus_body_art: Option<data::assets::samus::SamusAssets>,
    /// The pose tiles and graphics read by the room NMI.
    pub samus_tiles: Option<(data::pose::tiles::SamusTiles, data::pose::tiles::Graphics)>,
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
    /// Physical room graphics with a qualified writer.
    pub room_video: room_video::RoomVideo,
    /// Elapsed time carried from the selected file.
    pub play_time: types::PlayTime,
    /// The timer sprites held in the prepared and published pictures.
    pub clock_picture: clock_picture::ClockPicture,
    /// The completed room background writes.
    pub room_tiles: room_tiles::ShaftBackground,
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
    /// The shaft burst's CPU sender and passage sound permission.
    pub shaft_sound: shaft_sound::State,
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
    /// OAM slots appended by the ordinary body, empty when it was hidden.
    pub samus_body: Option<core::ops::Range<usize>>,
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
    /// Overlay tiles requested on successful actor placement.
    pub shaft_overlay_placement: Option<boss_ceres_ridley::OverlaySpriteUpload>,
    /// The escape overlay's drawing, lights and burst requests from a fresh actor turn.
    pub shaft_actors: Option<boss_ceres_ridley::OverlayTurn>,
    /// The offer carried from a fresh shaft burst to its CPU queue.
    pub shaft_sound_offer: Option<audio_transport::Offer>,
    /// The elevator scene and standing lock requested once, after ordinary motion.
    pub elevator_reach: Option<boss_ceres_ridley::ElevatorLock>,
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
    /// The return shaft has no imported raster programs.
    ShaftRaster,
    /// The arrival graphics or its direct transfer could not be published.
    ArrivalGraphics,
    /// The doorway’s selected enemy tile uploads could not be represented.
    DoorEnemyGraphics,
    /// The fresh file's Samus could not be constructed.
    NewGame,
    /// The station's escape in the shaft: a later piece owns the shaft's overlay.
    Escape,
    /// The shaft burst requested an unsupported sound queue limit.
    ShaftSoundAllowance,
    /// The locked standing picture could not be prepared.
    StandingPicture(super::blackout::Fault),
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
    /// The ordinary body figure or step has no imported art.
    SamusBody(Figure),
    /// The ordinary figure or animation step has no imported strip definition.
    SamusTiles(Figure),
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
        let output = if self.arrival.preparing() {
            self.prepare(passing, content, rules)
        } else if matches!(self.passage.doors, Doors::Passing(_)) {
            self.pass(inputs, passing, content, rules)
        } else {
            self.play(inputs, passing, hud, content, rules)
        }?;
        if passing.publishes {
            self.room_video.publish_ordinary_sprites();
            if let Some((tiles, graphics)) = &content.samus_tiles {
                self.room_video.video.transfer_samus_tiles(tiles, graphics);
                self.room_video.video.batch.samus_tiles =
                    crate::state::video::SamusTileUploads::default();
            }
            self.room_video.publish(content.standing.as_ref());
        }
        Ok(output)
    }

    fn play(
        &mut self,
        inputs: &Inputs,
        passing: &Passing,
        hud: &HudFrame,
        content: &Content,
        rules: &Rules,
    ) -> Result<Outputs, Fault> {
        let room = self.ordinary_room(inputs)?;
        let mut random = self.frame.drawn(passing);
        let mut palette = self.palette;
        let locked = self.elevator_locked();
        let (mut her, (mut enemies, health)) =
            self.play_actors_and_samus(inputs, passing, content, rules, &mut random, &mut palette)?;
        let uploads = content
            .samus_tiles
            .as_ref()
            .filter(|_| her.outcome.is_some())
            .map(|(tiles, _)| her.tiles(tiles))
            .transpose()?;
        if let (Some(row), Some(colours)) =
            (room_loading::suit_row(&mut palette.screen), her.colours)
        {
            *row = colours.0;
        }
        let mut samus = her.samus;
        let pad_and_cover = self.station.pad(&mut samus, rules);
        let camera = self.followed(&room, &her, &samus, rules);
        let touched = self.passage.touched(&her)?;
        let draw_at = passing.work_begun.then_some(inputs.parity);
        let body = self.ordinary_fight_body(&her, health, camera.position(), draw_at, content)?;
        let hud = warning::countdown_input(hud, enemies.warning.as_ref())?;
        let hud = if locked { hud.during_elevator() } else { hud };
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
        let elevator_reach = self.late_shaft_turn(&mut enemies, &mut her, &mut samus, content)?;
        let shaken = quake::update(&mut quake)?;
        let doors = self.passage.in_play(
            touched,
            hud::countdown_begun(&hud_state),
            &passage::scene(&samus, &camera, &palette),
            passing,
            content,
            rules,
        );

        let samus_body = body.map(|body| self.write_ordinary_fight_body(&body));
        self.prepare_room_picture(&hud_state, &hud_outputs, camera.position(), content)?;
        let shaft_sound_offer =
            self.shaft_outputs(enemies.shaft_actors, &doors.report, passing, content)?;

        if let Some(uploads) = uploads {
            self.room_video.video.batch.samus_tiles = uploads;
        }
        self.prepare_bg1(content, camera.position());

        // Her room and where it sits on her map stay as they are through a frame of play.
        self.samus
            .played(samus, &her, samus::jump_button(inputs.held));
        self.station.pad_and_cover = pad_and_cover;
        self.camera = Model(camera);
        warning::record_progress(&mut self.progress, enemies.warning.as_ref());
        self.passage.doors = doors.doors;
        self.passage.doorway = touched;
        self.palette = doors.scene.palette;
        self.prepare_room_palette(passing.publishes);
        self.health = Model(health.state);
        self.hud = Model(hud_state);
        self.clock_picture = clock_picture;
        self.finish_room_actors(&mut enemies, passing.publishes);
        self.quake = Model(quake);
        let finishing = if self.station.escape_shaft.is_some() && !passing.publishes {
            world_doors::Finishing::Overrun
        } else {
            doors.report.finishing
        };
        self.frame = self.frame.after(finishing, random);
        self.play_time = play_time::after_work(self.play_time, health.state, passing.work_begun);
        self.arrival = self.arrival.after_play(rules);
        Ok(Outputs {
            movement: her.outcome,
            samus_body,
            sounds: her.sounds,
            doors: doors.report,
            health: Some(health),
            hud: Some(hud_outputs),
            opening_hud: None,
            ceres_doors: enemies.doors,
            lights: enemies.lights,
            shaft_overlay_placement: None,
            shaft_actors: enemies.shaft_actors,
            shaft_sound_offer,
            elevator_reach,
            ridley: enemies.ridley,
            warning: enemies.warning,
            steam_hit: enemies.steam_hit,
            ridley_placed: None,
            quake: Some(shaken),
        })
    }

    fn finish_room_actors(&mut self, enemies: &mut station::Enemies, publishes: bool) {
        self.station.ran(enemies);
        if let Some(shaft) = enemies.shaft.take() {
            shaft.update_entry(&mut self.passage.room.entry, publishes);
            self.station.escape_shaft = Some(shaft);
        }
        if let Some(vents) = enemies.vents.take() {
            self.station.vents = vents;
        }
    }

    fn prepare_room_picture(
        &mut self,
        hud: &interface_hud::HudState,
        output: &FrameOutputs,
        camera: types::Position,
        content: &Content,
    ) -> Result<(), Fault> {
        let standing = self.elevator_locked().then(|| {
            super::blackout::StandingFrame::from_locked(
                &self.samus,
                camera,
                self.passage.room.entry.view.transform,
            )
        });
        self.room_video
            .prepare_play_picture(hud, standing, output.countdown.picture, content)
    }

    fn elevator_locked(&self) -> bool {
        self.station
            .escape_shaft
            .as_ref()
            .is_some_and(|shaft| shaft.locked)
    }

    fn play_actors_and_samus(
        &self,
        inputs: &Inputs,
        passing: &Passing,
        content: &Content,
        rules: &Rules,
        random: &mut types::RandomSequence,
        palette: &mut Palette,
    ) -> Result<(samus::Her, (station::Enemies, samus_health::PlayedFrame)), Fault> {
        let locked = self.elevator_locked();
        // Contact reads the room as it stood before movement.
        let at_start = if locked {
            None
        } else {
            self.enemy_contact(content)?
        };
        let enemies = self.enemies(
            station::Running::Play(at_start.as_ref()),
            random,
            passing,
            content,
            rules,
            palette,
        )?;
        let hit = enemies.contact();
        let between = samus::Between {
            played: (enemies, self.health_frame(hit, content, rules)),
            hit,
            energy: self.energy_the_hit_leaves(hit, content, rules),
        };
        if locked || self.station.pad_and_cover.holds_her() {
            Ok((self.samus.held(), between.played))
        } else {
            self.her_frame(inputs, content, rules, |_| Ok(between))
        }
    }

    fn late_shaft_turn(
        &self,
        enemies: &mut station::Enemies,
        her: &mut samus::Her,
        samus: &mut samus_movement::Samus,
        content: &Content,
    ) -> Result<Option<boss_ceres_ridley::ElevatorLock>, Fault> {
        let Some(shaft) = &mut enemies.shaft else {
            return Ok(None);
        };
        let track = content.shaft_track.as_ref().ok_or(Fault::Escape)?;
        let start = enemies.shaft_actors.and_then(|turn| turn.start_rotation);
        let reach = shaft.late_turn(start, track, samus, self.samus.movement.0.choices.latest)?;
        if let Some(reach) = reach {
            her.stand_at_elevator(samus, reach, &content.appearance)?;
        }
        Ok(reach)
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
        let shaft_brightening =
            self.station.escape_shaft.is_some() && passage::is_brightening(self.passage.doors);
        let early = self.brightening_actors(
            shaft_brightening,
            passing,
            content,
            rules,
            &mut random,
            &mut before,
        )?;
        let exports = self.passage_export(&before, passing, content, rules);
        let Scene {
            her,
            seen,
            camera: view,
            mut palette,
        } = exports.scene;
        let mut enemies = match early {
            Some(enemies) => enemies,
            None if !shaft_brightening && exports.report.runs[Activity::Enemies] => self.enemies(
                station::Running::Passage,
                &mut random,
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
        let mut loaded = self.load_passage_contents(
            &exports.report,
            passing,
            content,
            &mut palette,
            &mut random,
        )?;
        let shaft_sound_offer =
            self.shaft_outputs(enemies.shaft_actors, &exports.report, passing, content)?;
        let ridley_placed = loaded.actors.as_ref().and_then(|placed| placed.ridley);
        let shaft_overlay_placement = loaded.shaft_placement.map(|placement| placement.upload);
        if loaded.actors.is_some() {
            self.station.escape_shaft = loaded.shaft.take();
        }
        let quake = self.passage_quake(
            enemies.quake,
            exports.report.call,
            ridley_placed,
            content,
            rules,
        );
        self.station
            .passed(&mut enemies, loaded.actors.take(), pad_and_cover);
        self.quake = Model(quake);
        self.samus.movement.0 = moved;
        self.samus.last_draw_jump.0 = last_draw_jump;
        self.apply_passage_view(content, view, seen, exports.report.layer_2);
        self.palette = palette;
        self.prepare_room_palette(passing.publishes);
        self.frame = self.frame.after(exports.report.finishing, random);
        self.clock_picture = clock_picture;
        self.hud.0.out_of_play(&hud::out_of_play(&exports.report));
        self.finish_passage(&exports, loaded, passing);
        Ok(Outputs {
            movement: None,
            samus_body: None,
            sounds: Vec::new(),
            doors: exports.report,
            health: None,
            hud: None,
            opening_hud: None,
            ceres_doors: enemies.doors,
            lights: enemies.lights,
            shaft_overlay_placement,
            shaft_actors: enemies.shaft_actors,
            shaft_sound_offer,
            elevator_reach: None,
            ridley: enemies.ridley,
            warning: enemies.warning,
            steam_hit: enemies.steam_hit,
            ridley_placed,
            quake: None,
        })
    }
    fn shaft_outputs(
        &mut self,
        turn: Option<boss_ceres_ridley::OverlayTurn>,
        doors: &world_doors::Report,
        passing: &Passing,
        content: &Content,
    ) -> Result<Option<audio_transport::Offer>, Fault> {
        let (sound, offer) = self
            .shaft_sound
            .next(turn, doors, passing.audio.shaft_reply)?;
        self.room_video.shaft_picture(
            turn,
            self.vent_camera(),
            passing.publishes,
            content.shaft_overlay_art.as_ref(),
        )?;
        self.shaft_sound = sound;
        Ok(offer)
    }

    fn passage_export(
        &self,
        before: &Palette,
        passing: &Passing,
        content: &Content,
        rules: &Rules,
    ) -> world_doors::Exports {
        self.passage.doors.step(
            passage::scene(&self.samus.movement.0, &self.camera.0, before),
            &passage::doors_inputs(
                false,
                hud::countdown_begun(&self.hud.0),
                self.passage.doorway,
                passing,
                content,
            ),
            &rules.doors,
        )
    }

    fn passage_quake(
        &mut self,
        mut quake: world_quake::State,
        call: Option<RoomCall>,
        ridley: Option<RidleyPlacing>,
        content: &Content,
        rules: &Rules,
    ) -> world_quake::State {
        if call == Some(RoomCall::BuildRoom) {
            quake::stop(&mut quake);
        }
        if let Some(placing) = ridley {
            self.meet_the_boss(placing, content, rules);
            let boss_ceres_ridley::ShakeStop = placing.shake;
            quake::stop(&mut quake);
        }
        quake
    }

    fn brightening_actors(
        &self,
        shaft_brightening: bool,
        passing: &Passing,
        content: &Content,
        rules: &Rules,
        random: &mut types::RandomSequence,
        palette: &mut Palette,
    ) -> Result<Option<station::Enemies>, Fault> {
        let runs = if shaft_brightening {
            passing.work_begun
        } else {
            passage::brightening(self.passage.doors, passing)
        };
        runs.then(|| {
            self.enemies(
                station::Running::Passage,
                random,
                passing,
                content,
                rules,
                palette,
            )
        })
        .transpose()
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
