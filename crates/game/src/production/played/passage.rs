//! Her room, its doors to other rooms, and the passage through them.
//!
//! A frame whose moves reach a door to a room plays out whole, and the passage starts on the
//! next. As each of the passage's calls is done, the room's loading loads its part of the next
//! room; building it replaces her room: what her moves meet, its doors to other rooms and the
//! camera's room. Fetching the next room's contents lets the last room's Ceres actors go and
//! places its own.

use super::samus::Her;
use super::{Content, Fault, Rules};
use crate::production::connected::inputs::CameraInputs;
use crate::production::room_loading::{self, CeresActors};
use crate::production::shaft_entry;
use types::{Pair, Parts};
use world_doors::{Arrival, Doors, Exports, Palette, Passage, Phase, RoomCall, Scene};

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

/// What play reads of the room she is in: what her moves meet, its doors to other rooms, and
/// the camera's room. A passage's building of the next room replaces it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Room {
    /// The built level and the room setup carried through the passage.
    pub entry: shaft_entry::State,
    /// The room's blocks and the boxes of its solid actors other than the Ceres doors.
    pub geometry: crate::production::world::MovementWorld,
    /// Its doors to other rooms, by cell.
    pub doorways: Vec<Doorway>,
    /// Its screens, size, scrollers and the camera's lead.
    pub camera: CameraInputs,
}

/// What a frame brings the doors from owners outside this loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Passing {
    /// The platform's: a frame of work begins in this frame, the last one's work done.
    pub work_begun: bool,
    /// The platform's: the passage's work in progress is done by this frame's end.
    pub work_done: bool,
    /// The platform publishes the current prepared picture and counts its frame.
    pub publishes: bool,
    /// Actor loading reached at the interrupted cut of an escape-shaft fetch.
    pub actor_fetch: ActorFetch,
    /// Audio's answers.
    pub audio: PassageAudio,
    /// The random numbers' owner's, where this loop doesn't own every draw: the game's random
    /// number as the last frame left it, which replaces the one State carries. None where
    /// State's own is the game's. The frame draws it once as its work begins.
    pub random: Option<types::RandomSequence>,
    /// The start's loading's, with the platform's lag: whether a new game's start is made ready
    /// this frame.
    pub start: super::arrival::Readiness,
}

/// How much actor loading the platform permitted before the cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorFetch {
    /// First slot whose old actor has not yet been retired.
    Clearing(usize),
    /// All old slots are clear and the species graphics have been prepared.
    Graphics,
    /// Cleared the old actors; destination placement can run.
    Place,
}

/// What audio answers the passage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PassageAudio {
    /// Every sound request so far has been sent.
    pub sounds_sent: bool,
    /// No track waits in the queue of tracks.
    pub music_ready: bool,
}

/// Her room, play or a passage under way, and the door she touched, carried frame to frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// Play, or a passage under way.
    pub doors: Doors,
    /// The door she touched, from the touch until play goes on.
    pub doorway: Option<Doorway>,
    /// Static setup has run while the loader is retiring the previous population.
    pub awaiting_actors: bool,
    /// What play reads of her room.
    pub room: Room,
    /// Where her room sits on her map.
    pub map_room: interface_hud::Room,
}

impl State {
    /// The door to another room `her` moves reached, if any; none while the pad holds her.
    ///
    /// # Errors
    /// [`Fault::NewArea`] when it leads into another area.
    pub(super) fn touched(&self, her: &Her) -> Result<Option<Doorway>, Fault> {
        let touched = her.outcome.as_ref().and_then(|outcome| {
            self.room
                .doorways
                .iter()
                .find(|doorway| {
                    outcome
                        .reached
                        .iter()
                        .any(|touch| touch.cell == doorway.cell)
                })
                .copied()
        });
        if touched.is_some_and(|doorway| doorway.map_room.is_none()) {
            return Err(Fault::NewArea);
        }
        Ok(touched)
    }

    /// The doors' step at the end of a frame of play, `touched` the door her moves reached and
    /// `countdown_begun` whether the escape countdown has begun as the HUD's frame leaves it.
    pub(super) fn in_play(
        &self,
        touched: Option<Doorway>,
        countdown_begun: bool,
        scene: &Scene,
        passing: &Passing,
        content: &Content,
        rules: &Rules,
    ) -> Exports {
        self.doors.step(
            *scene,
            &doors_inputs(
                touched.is_some(),
                countdown_begun,
                touched,
                passing,
                content,
            ),
            &rules.doors,
        )
    }

    /// After a passage's frame leaves the doors at `doors`, with the room its building gave, if
    /// any: once play goes on, her room is the one the door she touched leads to.
    pub(super) fn passed(&mut self, doors: Doors, room: Option<Room>) {
        if let Some(room) = room {
            self.room = room;
        }
        self.doors = doors;
        if self.doors == Doors::Play {
            if let Some(room) = self.doorway.and_then(|doorway| doorway.map_room) {
                self.map_room = room;
            }
            self.doorway = None;
        }
    }
}

impl super::State {
    /// Commit the built room, partial population and block-script turn together.
    pub(super) fn finish_passage(
        &mut self,
        exports: &Exports,
        loaded: Loaded,
        actor_fetch: ActorFetch,
    ) {
        self.passage.passed(exports.doors, loaded.room);
        self.passage.awaiting_actors = loaded.awaiting_actors;
        if loaded.awaiting_actors {
            match actor_fetch {
                ActorFetch::Clearing(first_live) => self.station.retire_before(first_live),
                ActorFetch::Graphics => self.station.retire_before(usize::MAX),
                ActorFetch::Place => {}
            }
        }
        if exports.report.runs[world_doors::Activity::Blocks] {
            self.passage.room.entry.run_blocks();
        }
    }

    /// What the passage's call `call` loads of the next room, as its work is done: the colours
    /// it sets, the room it builds and the Ceres actors it places.
    pub(super) fn load(
        &self,
        call: Option<RoomCall>,
        content: &Content,
        palette: &mut Palette,
        random: &mut types::RandomSequence,
    ) -> Result<Loaded, Fault> {
        let mut loaded = Loaded {
            room: None,
            actors: None,
            awaiting_actors: self.passage.awaiting_actors,
        };
        let Some(doorway) = self.passage.doorway else {
            return Ok(loaded);
        };
        let run = room_loading::Run {
            progress: &self.progress,
            ceres: self.station.stage,
        };
        if call != Some(RoomCall::FetchContents) {
            room_loading::load(call, doorway.through, run, &content.rooms, palette)
                .map_err(Fault::Load)?;
        }
        match call {
            // Building the next room replaces what play reads of hers.
            Some(RoomCall::BuildRoom) => {
                let mut room = room_loading::room(
                    doorway.through,
                    run,
                    &content.rooms,
                    &self.passage.room.camera,
                )
                .map_err(Fault::Load)?;
                room.entry.view = self.passage.room.entry.view;
                room.entry.displayed = self.passage.room.entry.displayed;
                loaded.room = Some(room);
            }
            // The next room's contents let the last room's Ceres actors go and place its own.
            Some(RoomCall::FetchContents) => {
                let setup = room_loading::setup(&content.rooms, doorway.through, run.progress)
                    .map_err(Fault::Load)?;
                let mut room = self.passage.room.clone();
                room.entry
                    .advance(setup, shaft_entry::Opportunity::Fetch)
                    .map_err(Fault::Load)?;
                if let Some([x, y]) = room.entry.placement {
                    for row in y..y.saturating_add(4) {
                        let at = types::Cell { x, y: row };
                        room.geometry
                            .close(at)
                            .map_err(|_| Fault::Load(room_loading::LoadFault::Geometry))?;
                        room.doorways.retain(|door| door.cell != at);
                    }
                }
                if setup.door == data::assets::world::DoorRule::ToCeresElevatorShaft
                    && setup.room.closing_anchor().is_some()
                {
                    loaded.awaiting_actors = true;
                } else {
                    room_loading::load(call, doorway.through, run, &content.rooms, palette)
                        .map_err(Fault::Load)?;
                    loaded.actors = Some(
                        room_loading::ceres_actors(doorway.through, run, &content.rooms, random)
                            .map_err(Fault::Load)?,
                    );
                }
                loaded.room = Some(room);
            }
            None
            | Some(
                RoomCall::LeaveRoom
                | RoomCall::DrawTopRow
                | RoomCall::FetchTiles
                | RoomCall::AnimateTiles
                | RoomCall::PlayMusic
                | RoomCall::ReadLiquid,
            ) => {}
        }
        if loaded.awaiting_actors {
            room_loading::fetch_palette(
                doorway.through,
                run,
                &content.rooms,
                palette,
                passing.actor_fetch,
            )
            .map_err(Fault::Load)?;
        }
        if loaded.awaiting_actors && passing.actor_fetch == ActorFetch::Place {
            loaded.actors = Some(
                room_loading::ceres_actors(doorway.through, run, &content.rooms, random)
                    .map_err(Fault::Load)?,
            );
            loaded.awaiting_actors = false;
        }
        Ok(loaded)
    }
}

/// Darkening draws report with completed work. Brightening reaches its history-writing draw
/// when work begins, which can precede the cut that advances the fade.
pub(super) fn draws_jump(doors: Doors, passing: &Passing, report: &world_doors::Report) -> bool {
    if matches!(
        doors,
        Doors::Passing(Passage {
            phase: Phase::Brightening(_),
            ..
        })
    ) {
        passing.work_begun
    } else {
        report.runs[world_doors::Activity::Drawn]
    }
}

/// What the passage loaded from the next room.
pub(super) struct Loaded {
    pub(super) awaiting_actors: bool,
    /// The room building it gives, which replaces hers.
    pub(super) room: Option<Room>,
    /// The Ceres actors fetching its contents places, which replace the last room's.
    pub(super) actors: Option<CeresActors>,
}

/// As the colours come up, the enemies run before the fade takes its step, so the beacon's
/// colours go in first; that frame of work runs them whenever its work is done.
pub(super) const fn brightening(doors: Doors, passing: &Passing) -> bool {
    passing.work_done
        && matches!(
            doors,
            Doors::Passing(Passage {
                phase: Phase::Brightening(_),
                ..
            })
        )
}

/// What the passage moves, as the frame before left it.
pub(super) fn scene(
    samus: &samus_movement::Samus,
    camera: &world_camera::Camera,
    palette: &Palette,
) -> Scene {
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

/// What the doors read this frame: whether and which door she touched, whether the HUD's escape
/// countdown has begun, and their neighbours' answers.
pub(super) fn doors_inputs(
    touched: bool,
    countdown_begun: bool,
    doorway: Option<Doorway>,
    passing: &Passing,
    content: &Content,
) -> world_doors::Inputs {
    world_doors::Inputs {
        touched,
        turn_ready: passing.work_done,
        publishes: passing.publishes,
        sounds_sent: passing.audio.sounds_sent,
        music_ready: passing.audio.music_ready,
        timer_running: countdown_begun,
        items_dark: doorway.is_some_and(|doorway| doorway.items_dark),
        doorway: doorway.map(|doorway| doorway.arrival),
        groups: content.colour_groups,
    }
}
