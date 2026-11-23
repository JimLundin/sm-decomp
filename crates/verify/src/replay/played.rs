//! Play a room alone from its seed: the movie's pads in, whole frames out, compared at each cut.
//!
//! This is the rung measure (MODEL decision 18): production State is admitted once from the
//! seed record and then carried with nothing fed back. Each cut's only new input is the pad
//! the movie presented at the preceding vblank; the oracle's later records are read for
//! comparison alone. Room facts that no selected owner produces yet are taken once from the
//! seed and held for the whole run; [`SEED_HELD`] names them. Whether each cut's frame of work
//! finished is the platform's, read from the record until it is modelled (decision 35);
//! [`BORROWED`] names it. A run stops at the first differing field or the first unsupported
//! family, and the number of cuts it matched before that is the frames it played alone.
//!
//! The comparison is at frame cuts (decision 1): the selected owners' State at each NMI
//! return. Intra-frame bus timing is not compared here.

use super::compare::{CompareFault, Observation, compare};
use super::index::ObservationId;
use super::readers::record;
use super::result::Outcome;
use super::{Sources, appearance, camera, connected, data_failure, movement};
use crate::cache::Cache;
use crate::lock::{Lock, RowId};
use crate::source;
use canonical::model::Model;
use data::{rom::Rom, room::RoomKey};
use game::production::connected::inputs::Inputs;
use game::production::played;
use oracle_record::{Cut, FrameRecord, NmiKind};
use std::path::Path;
use types::{Pair, Parity};
use world_doors::{Doors, Palette};

/// Room facts taken from the seed and held, because no selected owner produces them yet.
/// Each one is a missing owner; the list shrinks as owners join the played loop.
pub const SEED_HELD: &[&str] = &[
    "equipped items",
    "energy",
    "liquid (none)",
    "sound queue occupancy",
    "Contact guards",
    "cinematic and elevator status",
    "room geometry, doors and actor boxes",
    "camera room settings and lead",
    "escape timer",
    "passage sounds sent and music ready",
    "progress, which chooses the next room's version",
];

/// Inputs read from the record at each cut, because their owner is outside the rungs
/// (decision 35). The list shrinks as the platform's costs are modelled.
pub const BORROWED: &[&str] = &["lag: whether each cut's frame of work finished"];

/// Where a played run stopped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stop {
    /// Every cut through the requested end matched.
    End,
    /// Why the next cut was not played: a difference, a missing owner or bad data.
    At(Outcome),
}

/// The frames a room played alone from its seed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// Lock row played.
    pub row: String,
    /// Seed cut.
    pub start: u64,
    /// Consecutive cuts after the seed whose compared fields all matched.
    pub held: u64,
    /// Where the run stopped.
    pub stop: Stop,
}

/// The controller as the game sees it: the pad read by the last polling NMI and the
/// buttons that went down at that read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Controller {
    held: u16,
    pressed: u16,
}

impl Controller {
    /// A polling NMI latches the new pad; a lag NMI leaves the previous read in place.
    const fn read(self, pad: u16, polled: bool) -> Self {
        if polled {
            Self {
                held: pad,
                pressed: pad & !self.held,
            }
        } else {
            self
        }
    }
}

/// The frame counter's parity: the game counts each NMI that reads the pad.
const fn parity(counter: u16) -> Parity {
    if counter & 1 == 0 {
        Parity::Even
    } else {
        Parity::Odd
    }
}

const fn polled(record: &FrameRecord) -> bool {
    matches!(record.cut, Cut::NmiReturn(nmi) if nmi.polled)
}

/// The borrowed lag: the NMI found the frame's work finished, waiting for it. A lag NMI finds
/// the work part-done, whether or not it reads the pad.
const fn finished(record: &FrameRecord) -> bool {
    matches!(record.cut, Cut::NmiReturn(nmi) if matches!(nmi.kind, NmiKind::Requested))
}

fn word(wram: &[u8], at: source::WramWord) -> Result<u16, Outcome> {
    at.read(wram).map_err(|_| data_failure(None, at.name))
}

/// Seed-held room facts and the seed's frame counter.
struct Room {
    inputs: Inputs,
    doorways: Vec<played::Doorway>,
    timer_running: bool,
    /// Audio is outside the loop: the seed's queue holds no sound, and none is taken to wait.
    sounds_sent: bool,
    progress: run_progress::Progress,
    counter: u16,
}

fn room(seed: &[u8], content: &connected::Content, image: &Rom) -> Result<Room, Outcome> {
    let borrowed = movement::borrowed(seed, seed, &content.movement)?;
    if borrowed.cinematic || borrowed.elevator {
        return Err(Outcome::Unsupported {
            cut: None,
            dependency: "played seed outside ordinary play".into(),
        });
    }
    let inputs = connected::seed_inputs(seed, borrowed.items, borrowed.world)?;
    Ok(Room {
        sounds_sent: inputs.sounds_waiting == samus_appearance::SoundsWaiting::NONE,
        inputs,
        doorways: doorways(seed, image)?,
        timer_running: word(seed, source::ESCAPE_TIMER_RUNNING)? != 0,
        progress: super::progress::seed(seed)?,
        counter: word(seed, source::NMI_FRAME_COUNTER_WORD)?,
    })
}

/// The room's doors to other rooms, by cell, from the imported world.
fn doorways(seed: &[u8], image: &Rom) -> Result<Vec<played::Doorway>, Outcome> {
    use data::assets::world::{Door, ItemLighting};
    let bad = || data_failure(None, "played room doors");
    let id = import::world::id(RoomKey::from_room_pointer(word(seed, source::ROOM_PTR)?))
        .ok_or_else(bad)?;
    let world = import::world::import(image.lorom(), &[id]).map_err(|_| bad())?;
    let room = world.room(id).ok_or_else(bad)?;
    let mut doorways = Vec::new();
    for (cell, place) in movement::door_cells(seed)? {
        let Some(Door::To(to)) = room.doors.get(place) else {
            continue;
        };
        let doorway = world.doorway(*to).ok_or_else(bad)?;
        let next = import::world::import(image.lorom(), &[doorway.room]).map_err(|_| bad())?;
        let next = next.room(doorway.room).ok_or_else(bad)?;
        doorways.push(played::Doorway {
            cell,
            through: *to,
            arrival: world_doors::Arrival {
                way: doorway.arrival.way,
                view: Pair {
                    x: doorway.arrival.view.x.pixels(),
                    y: doorway.arrival.view.y.pixels(),
                },
                speed: doorway.arrival.speed,
            },
            items_dark: [room, next]
                .iter()
                .any(|room| room.item_lighting == ItemLighting::Dark),
        });
    }
    Ok(doorways)
}

/// The rooms the engine loads and their tilesets, imported from the player's ROM.
fn rooms(image: &Rom) -> Result<game::production::room_loading::Rooms, Outcome> {
    let bad = || data_failure(None, "played rooms import");
    let world = import::world::import(image.lorom(), &import::world::LOADED).map_err(|_| bad())?;
    let tilesets = import::tileset::import(image.lorom(), &world).map_err(|_| bad())?;
    Ok(game::production::room_loading::Rooms { world, tilesets })
}

fn expected(
    record: &FrameRecord,
    content: &appearance::Content,
    row: &str,
    cut: u64,
) -> Result<Vec<Observation>, Outcome> {
    let wram = &record.wram;
    let seeded = movement::seed(wram)?;
    let mut fields = parts(&seeded);
    fields.extend(movement::expected(
        wram,
        &[ObservationId::PositionX, ObservationId::PositionY],
        row,
        cut,
    )?);
    let (state, playing) = appearance::owner(wram, content)?;
    fields.extend(appearance::initial(state, content));
    fields.push(connected::playing_observation(playing));
    fields.extend(camera::expected(wram)?);
    let passing = match word(wram, source::GAME_STATE)? {
        PLAY => false,
        TOUCHED | LOADING | ARRIVING => true,
        _ => {
            return Err(Outcome::Unsupported {
                cut: None,
                dependency: "played game state outside play and door passages".into(),
            });
        }
    };
    fields.push(passing_observation(passing));
    if finished(record) {
        fields.extend(colour_observations(&Palette {
            screen: colours(wram, SCREEN_COLOURS)?,
            goal: colours(wram, GOAL_COLOURS)?,
        }));
    }
    Ok(fields)
}

/// The game's states for play and the three stretches of a door passage.
const PLAY: u16 = 8;
const TOUCHED: u16 = 9;
const LOADING: u16 = 10;
const ARRIVING: u16 = 11;
/// The screen's colours as they stand, and the colours a fade heads for.
const SCREEN_COLOURS: usize = 0xc000;
const GOAL_COLOURS: usize = 0xc200;

fn passing_observation(passing: bool) -> Observation {
    Observation {
        field: "doors.passing".into(),
        bytes: vec![u8::from(passing)],
    }
}

/// The colours, compared only at a cut whose frame of work finished: a lag cut finds a fade's
/// step part-done.
fn colour_observations(palette: &Palette) -> [Observation; 2] {
    let bytes = |colours: &[types::Colour]| {
        colours
            .iter()
            .flat_map(|colour| colour.bits().to_le_bytes())
            .collect()
    };
    [
        Observation {
            field: "doors.colours.screen".into(),
            bytes: bytes(&palette.screen),
        },
        Observation {
            field: "doors.colours.goal".into(),
            bytes: bytes(&palette.goal),
        },
    ]
}

fn at(cut: u64, outcome: Outcome) -> Outcome {
    match outcome {
        Outcome::Unsupported {
            cut: None,
            dependency,
        } => Outcome::Unsupported {
            cut: Some(cut),
            dependency,
        },
        Outcome::DataFailure { cut: None, source } => Outcome::DataFailure {
            cut: Some(cut),
            source,
        },
        other @ (Outcome::Pass
        | Outcome::Difference { .. }
        | Outcome::Unsupported { .. }
        | Outcome::DataFailure { .. }
        | Outcome::CheckpointFailure { .. }) => other,
    }
}

/// Play `row` from its cut `start` through `end` inclusive with the movie's pads alone.
///
/// # Errors
/// Missing or mismatched lock, cache or content refuse the run before any cut is played.
pub fn play(sources: &Sources, row: &str, start: u64, end: u64) -> Result<Report, Outcome> {
    let text = std::fs::read_to_string(&sources.lock)
        .map_err(|_| data_failure(None, "record lock file"))?;
    let lock = Lock::parse(&text).map_err(|_| data_failure(None, "record lock schema"))?;
    let locked = lock
        .row(&RowId(row.to_owned()))
        .ok_or_else(|| data_failure(None, "played lock row"))?;
    let cache =
        Cache::open(&sources.cache, locked).map_err(|_| data_failure(None, "record cache pins"))?;
    let content = connected::Content {
        movement: movement::Content::load(&sources.rom)?,
        appearance: appearance::Content::load(&sources.rom)?,
    };
    let seed = record(&cache, start)?;
    let report = |held, stop| Report {
        row: row.to_owned(),
        start,
        held,
        stop,
    };
    let mut run = match Run::admit(&seed, &content, &sources.rom, row) {
        Ok(run) => run,
        Err(outcome) => return Ok(report(0, Stop::At(at(start, outcome)))),
    };
    let mut controller = Controller::default().read(seed.pad.0, polled(&seed));
    let mut held = 0;
    let mut previous = seed;
    for cut in start.saturating_add(1)..=end {
        let current = record(&cache, cut)?;
        if let Err(outcome) = run.step(controller, &previous, &current, cut) {
            return Ok(report(held, Stop::At(at(cut, outcome))));
        }
        held = held.saturating_add(1);
        controller = controller.read(current.pad.0, polled(&current));
        previous = current;
    }
    Ok(report(held, Stop::End))
}

/// One played run's carried owners and the facts held from its seed.
struct Run<'a> {
    state: played::State,
    played: played::Content,
    rules: played::Rules,
    room: Room,
    content: &'a connected::Content,
    row: &'a str,
}

impl<'a> Run<'a> {
    fn admit(
        seed: &FrameRecord,
        content: &'a connected::Content,
        path: &Path,
        row: &'a str,
    ) -> Result<Self, Outcome> {
        let bad = || data_failure(None, "played ROM image");
        let image = Rom::new(std::fs::read(path).map_err(|_| bad())?).map_err(|_| bad())?;
        let room = room(&seed.wram, content, &image)?;
        let (appearance, figure) = appearance::owner(&seed.wram, &content.appearance)?;
        Ok(Self {
            state: played::State {
                movement: Model(movement::seed(&seed.wram)?),
                appearance: Model(appearance),
                figure: Model(figure),
                camera: Model(camera::seed(&seed.wram, &camera::Content)?),
                doors: Doors::Play,
                doorway: None,
                palette: Palette {
                    screen: colours(&seed.wram, SCREEN_COLOURS)?,
                    goal: colours(&seed.wram, GOAL_COLOURS)?,
                },
            },
            played: played::Content {
                movement: content.movement.movement.clone(),
                appearance: content.appearance.appearance.clone(),
                colour_groups: super::passage_audio::groups(),
                rooms: rooms(&image)?,
            },
            rules: played::Rules {
                movement: samus_movement::Rules::default(),
                appearance: samus_appearance::Rules::VANILLA,
                camera: game::production::CameraRules::new(world_camera::Rules::REFERENCE),
                doors: world_doors::Rules::default(),
            },
            room,
            content,
            row,
        })
    }

    /// Play the interval ending at `cut` and compare its owners with the record at `cut`.
    fn step(
        &mut self,
        controller: Controller,
        previous: &FrameRecord,
        current: &FrameRecord,
        cut: u64,
    ) -> Result<(), Outcome> {
        // The movie's pad is the input; the game's own latch only checks its alignment, so a
        // stale or early pad is a harness fault rather than a model difference.
        if polled(previous) && word(&previous.wram, source::JOYPAD1_LASTKEYS)? != controller.held {
            return Err(data_failure(None, "played pad alignment"));
        }
        // In play, a lag NMI finds her frame part-played; only a passage plays on through one.
        if self.state.doors == Doors::Play && !finished(current) {
            return Err(Outcome::Unsupported {
                cut: None,
                dependency: "lag cut in play (T1)".into(),
            });
        }
        let mut inputs = self.room.inputs.clone();
        inputs.held = movement::pad(controller.held);
        inputs.pressed = movement::pad(controller.pressed);
        inputs.parity = parity(self.room.counter);
        let passing = played::Passing {
            doorways: &self.room.doorways,
            work_done: finished(current),
            audio: played::PassageAudio {
                sounds_sent: self.room.sounds_sent,
                music_ready: true,
            },
            timer_running: self.room.timer_running,
            progress: &self.room.progress,
        };
        self.state
            .step(&inputs, &passing, &self.played, &self.rules)
            .map_err(|fault| Outcome::Unsupported {
                cut: Some(cut),
                dependency: format!("played frame {fault:?}"),
            })?;
        if polled(current) {
            self.room.counter = self.room.counter.wrapping_add(1);
        }
        let expected = expected(current, &self.content.appearance, self.row, cut)?;
        let actual = actual(&self.state, &self.content.appearance, finished(current));
        compare(&expected, &actual).map_err(|fault| match fault {
            CompareFault::Difference(field) => Outcome::Difference { cut, field },
            CompareFault::EmptyExpected
            | CompareFault::DuplicateField(_)
            | CompareFault::MissingActual(_) => data_failure(Some(cut), "played comparison schema"),
        })
    }
}

/// The 256 colours at `at`, as the record holds them.
fn colours(wram: &[u8], at: usize) -> Result<[types::Colour; world_doors::COLOURS], Outcome> {
    let bad = || data_failure(None, "played colours observation");
    let words = wram
        .get(at..)
        .and_then(|bank| bank.as_chunks::<2>().0.get(..world_doors::COLOURS))
        .ok_or_else(bad)?;
    let mut colours = [types::Colour::BLACK; world_doors::COLOURS];
    for (colour, word) in colours.iter_mut().zip(words) {
        *colour = types::Colour::new(u16::from_le_bytes(*word)).ok_or_else(bad)?;
    }
    Ok(colours)
}

/// Her movement State by part, so a difference names the part that differs; the encoding is
/// only compared, never logged. Her spin's frame count is the model's own history: the record
/// keeps it as the last different pose's type and cannot tell its first two frames apart, so
/// it is not compared here. Its effects (a reversal's skid, a wall jump readied) reach the
/// compared pose, run and vertical motion in later cuts.
fn parts(samus: &samus_movement::Samus) -> Vec<Observation> {
    let part = |field: &str, value: String| Observation {
        field: field.into(),
        bytes: value.into_bytes(),
    };
    vec![
        part("movement.pose", format!("{:?}", samus.pose)),
        part("movement.run", run(&samus.run)),
        part("movement.vertical", format!("{:?}", samus.vertical)),
        part("movement.seat", format!("{:?}", samus.seat)),
        part("movement.jump-held", format!("{:?}", samus.jump_held)),
        part("movement.position", format!("{:?}", samus.position)),
    ]
}

/// Her run as the record states it. While she follows the pad the game keeps no way of its
/// own, so the way her speed points is compared only while she carries on; a way that differs
/// while she follows the pad reaches the compared position if it ever moves her.
fn run(run: &samus_movement::Run) -> String {
    let samus_movement::Run {
        speed,
        way,
        carry,
        dash,
        dashing,
    } = run;
    let way = (*carry == samus_movement::Carry::CarriesOn).then_some(way);
    format!("{speed:?} {way:?} {carry:?} {dash:?} {dashing:?}")
}

fn actual(
    state: &played::State,
    content: &appearance::Content,
    finished: bool,
) -> Vec<Observation> {
    let samus = &state.movement.0;
    let mut fields = parts(samus);
    fields.extend([
        Observation::quantity("movement.position.x", i64::from(samus.position.x.0)),
        Observation::quantity("movement.position.y", i64::from(samus.position.y.0)),
    ]);
    fields.extend(appearance::initial(state.appearance.0, content));
    fields.push(connected::playing_observation(state.figure.0));
    fields.extend(camera::initial(&state.camera.0));
    fields.push(passing_observation(matches!(
        state.doors,
        Doors::Passing(_)
    )));
    if finished {
        fields.extend(colour_observations(&state.palette));
    }
    fields
}

#[cfg(test)]
mod tests {
    use super::Controller;

    #[test]
    fn a_polling_read_reports_only_buttons_that_went_down() {
        let first = Controller::default().read(0x0180, true);
        assert_eq!((first.held, first.pressed), (0x0180, 0x0180));
        let second = first.read(0x8180, true);
        assert_eq!((second.held, second.pressed), (0x8180, 0x8000));
        let released = second.read(0x0100, true);
        assert_eq!((released.held, released.pressed), (0x0100, 0));
    }

    #[test]
    fn a_lag_nmi_keeps_the_previous_read() {
        let read = Controller::default().read(0x0080, true);
        assert_eq!(read.read(0x0100, false), read);
    }

    #[test]
    fn her_way_is_compared_only_while_she_carries_on() {
        use samus_movement::{Carry, Dashing, Run};
        use types::{Side, ZERO};
        let run = |way, carry| {
            super::run(&Run {
                speed: ZERO,
                way,
                carry,
                dash: ZERO,
                dashing: Dashing::Off,
            })
        };
        assert_eq!(
            run(Side::Left, Carry::FollowsPad),
            run(Side::Right, Carry::FollowsPad)
        );
        assert_ne!(
            run(Side::Left, Carry::CarriesOn),
            run(Side::Right, Carry::CarriesOn)
        );
    }

    fn diagnostic(args: std::fmt::Arguments<'_>) {
        use std::io::Write as _;
        writeln!(std::io::stdout().lock(), "{args}").unwrap();
    }

    fn sources() -> super::Sources {
        let path = |name| std::env::var_os(name).unwrap().into();
        super::Sources {
            cache: path("SM_CACHE"),
            rom: path("SM_ROM"),
            lock: path("SM_LOCK"),
            ..Default::default()
        }
    }

    /// The M0 rung: Ceres's third room from its seed to the door, movie pads only.
    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn m0_room_three_played_alone() {
        let sources = sources();
        for start in [0, 6] {
            let report = super::play(&sources, "chain-any-0003-DFD7", start, 344).unwrap();
            diagnostic(format_args!(
                "PLAYED_ALONE row={} start={} held={} stop={:?} seed_held={:?} borrowed={:?}",
                report.row,
                report.start,
                report.held,
                report.stop,
                super::SEED_HELD,
                super::BORROWED
            ));
        }
    }
}
