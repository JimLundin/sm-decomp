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
//!
//! This file holds the run and the order in which the owners' fields are compared, which is
//! the order a difference is reported in. Each owner's seed, record reading and compared fields
//! live in its own module: `samus`, `passage`, `colours`, `ceres_doors`, `ridley`, `shaft`,
//! `health`, `hud`, `frame` and `arrival`, with `station` the station's State; the camera's are
//! the camera replay's. `platform` holds the borrowed lag, pad and audio answers, `seed` the
//! facts held from the seed, and `rows` the chain of rows a run plays through. A row from the
//! console's power-on plays the front end alone, from its own power-on State with no seed:
//! `power_on`.

mod arrival;
mod ceres_doors;
mod colours;
mod frame;
mod health;
mod hud;
mod passage;
mod platform;
mod power_on;
mod ridley;
mod rows;
mod samus;
mod seed;
mod shaft;
mod start;
mod station;

use super::compare::{CompareFault, Observation, compare};
use super::result::Outcome;
use super::{Sources, appearance, camera, connected, data_failure, health_tail, movement};
use crate::source;
use canonical::model::Model;
use data::rom::Rom;
use game::production::played;
use oracle_record::FrameRecord;
use platform::{
    Controller, fetch_under_way, fetching_contents, finished, music_ready, parity, polled,
};
use std::path::Path;
use world_doors::Doors;

/// Room facts taken from the seed and held, because no selected owner produces them yet.
/// Each one is a missing owner; the list shrinks as owners join the played loop.
pub const SEED_HELD: &[&str] = &[
    "equipped items",
    "time running",
    "no tail impact and no periodic energy loss",
    "ammunition, reserve tanks and energy capacity",
    "item select and cancel bindings",
    "her map collected",
    "liquid (none)",
    "sound queue occupancy",
    "the owners active at her stride: a cinematic, a boss, special colour time",
    "her shot's charge, which no frame of play reads",
    "cinematic and elevator status",
    "the seed room's geometry and the boxes of its actors other than the Ceres doors",
    "the seed room's camera screens, size, scrollers and lead",
    "escape timer",
    "passage sounds sent",
    "progress, which chooses the next room's version, her suit and her beams, and whether the station's boss is beaten",
    "Ceres station stage, but where Ridley's placing sets it",
    "the HUD as the load flow puts it up for a new game, held as she arrives and not compared then, but the mini-map's power, which her arrival shows",
    "from a new game's seed: the platform's counts of finished frames and the random number, which run on through the preparation",
];

/// Inputs read from the record at each cut, because their owner is outside the rungs
/// (decision 35). The list shrinks as the platform's costs are modelled.
pub const BORROWED: &[&str] = &[
    "lag: whether each cut's frame of work finished",
    "lag: the contents' fetch done on its first frame, within the slide",
    "lag: whether a frame of work begins, the last one's done, when the random number is drawn",
    "the random number as a passage's frame begins: no owner here draws it through a passage",
    "audio: whether a track waits in the queue of tracks as the frame of work begins",
    "the start ready: the frame a new game's start is made ready (R2 and C1's loading, T1's lag)",
];

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
    /// The facts this kind of run holds from its seed.
    pub seed_held: &'static [&'static str],
    /// The inputs this kind of run borrows from the record.
    pub borrowed: &'static [&'static str],
}

fn word(wram: &[u8], at: source::WramWord) -> Result<u16, Outcome> {
    at.read(wram).map_err(|_| data_failure(None, at.name))
}

/// The owners' fields as the record at `cut` holds them, in the order they are compared.
fn expected(
    record: &FrameRecord,
    content: &appearance::Content,
    hud: &super::hud::Content,
    hud_settled: bool,
    counted: bool,
    row: &str,
    cut: u64,
) -> Result<Vec<Observation>, Outcome> {
    let wram = &record.wram;
    let finished = finished(record);
    let mut fields = samus::expected(wram, content, row, cut)?;
    fields.extend(camera::expected(wram)?);
    let passing = passage::passing(wram)?;
    fields.push(passage::observation(passing));
    // In play every draw is the loop's own, so a frame's draws are compared where it finished.
    if finished && !passing {
        fields.push(frame::random_observation(frame::random(wram)?));
    }
    if finished {
        fields.extend(colours::observations(&colours::palette(wram)?));
        fields.extend(ceres_doors::expected(wram)?);
        fields.extend(ridley::expected(wram)?);
    }
    fields.extend(shaft::expected(wram)?);
    fields.extend(health::parts(&health_tail::decode(wram, None)?));
    if hud_settled {
        fields.extend(hud::parts(super::hud::expected(wram, hud)?));
    }
    fields.extend(frame::expected(wram)?);
    fields.extend(arrival::expected(wram, counted)?);
    Ok(fields)
}

/// The owners' fields as the played State holds them.
fn actual(
    state: &played::State,
    content: &appearance::Content,
    hud: &super::hud::Content,
    finished: bool,
    hud_settled: bool,
) -> Result<Vec<Observation>, Outcome> {
    let mut fields = samus::actual(&state.samus, content);
    fields.extend(camera::initial(&state.camera.0));
    fields.push(passage::observation(matches!(
        state.passage.doors,
        Doors::Passing(_)
    )));
    fields.push(frame::random_observation(state.frame.random));
    if finished {
        fields.extend(colours::observations(&state.palette));
        fields.extend(ceres_doors::actual(&state.station));
        fields.extend(ridley::actual(&state.station));
    }
    fields.extend(shaft::actual(&state.station));
    fields.extend(health::parts(&state.health.0));
    if hud_settled {
        fields.extend(hud::parts(super::hud::initial(&state.hud.0, hud)?));
    }
    fields.extend(frame::actual(state.frame));
    fields.extend(arrival::actual(state.arrival));
    Ok(fields)
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

/// Play `row` from its cut `start` through `end` inclusive with the movie's pads alone. Cuts
/// past the row's last go on through the rows that continue it, without a new seed.
///
/// A row from the console's power-on plays from its cut 0 with no seed.
///
/// # Errors
/// Missing or mismatched lock, cache or content refuse the run before any cut is played, and a
/// cut past every row that continues the run, or a row that doesn't follow on, stops it.
pub fn play(sources: &Sources, row: &str, start: u64, end: u64) -> Result<Report, Outcome> {
    let text = std::fs::read_to_string(&sources.lock)
        .map_err(|_| data_failure(None, "record lock file"))?;
    let lock =
        crate::lock::Lock::parse(&text).map_err(|_| data_failure(None, "record lock schema"))?;
    // The lock is the repository's `scenarios/locks/<oracle>.toml`, under the tables of seeds.
    let root = sources
        .lock
        .ancestors()
        .nth(3)
        .ok_or_else(|| data_failure(None, "played lock path"))?;
    let mut rows = rows::Rows::open(&lock, root, &sources.cache, row)?;
    let id = crate::lock::RowId(row.to_owned());
    if lock
        .row(&id)
        .is_some_and(|listed| listed.start == crate::lock::Start::PowerOn)
    {
        return power_on::play(&mut rows, row, start, end);
    }
    let content = connected::Content {
        movement: movement::Content::load(&sources.rom)?,
        appearance: appearance::Content::load(&sources.rom)?,
    };
    let seed = rows.record(start)?;
    let report = |held, stop| Report {
        row: row.to_owned(),
        start,
        held,
        stop,
        seed_held: SEED_HELD,
        borrowed: BORROWED,
    };
    let mut run = match Run::admit(&seed, &content, &sources.rom, row) {
        Ok(run) => run,
        Err(outcome) => return Ok(report(0, Stop::At(at(start, outcome)))),
    };
    let mut controller = match Controller::seeded(&seed.wram) {
        Ok(controller) => controller,
        Err(outcome) => return Ok(report(0, Stop::At(at(start, outcome)))),
    };
    // A seed that read the pad read the movie's: a stale or early pad is a harness fault.
    if polled(&seed) && seed.pad.0 != controller.held {
        let misaligned = data_failure(None, "played pad alignment");
        return Ok(report(0, Stop::At(at(start, misaligned))));
    }
    let mut held = 0;
    let mut previous = seed;
    for cut in start.saturating_add(1)..=end {
        let current = rows.record(cut)?;
        if current.video_frame != previous.video_frame.wrapping_add(1) {
            return Err(data_failure(Some(cut), "played rows join"));
        }
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
    room: seed::Room,
    content: &'a connected::Content,
    hud: super::hud::Content,
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
        let start = import::start::import(image.lorom())
            .map_err(|_| data_failure(None, "played new game's start"))?;
        let rooms = seed::rooms(&image)?;
        // A seed in a new game's preparation holds the intro's RAM: State comes from the start.
        let (room, state) = if arrival::seed(&seed.wram)?.preparing() {
            start::admit(&seed.wram, &start, &rooms)?
        } else {
            Self::seeded(seed, content, &rooms)?
        };
        let hud = super::hud::Content::load(path)?;
        let (_, area) = room.map;
        Ok(Self {
            state,
            played: played::Content {
                movement: content.movement.movement.clone(),
                appearance: content.appearance.appearance.clone(),
                colour_groups: colours::groups(),
                rooms,
                hud: hud.pictures,
                map: hud.map_of(area)?.clone(),
                start_x: core::num::Wrapping(start.her.0),
            },
            rules: played::Rules {
                movement: samus_movement::Rules::default(),
                appearance: samus_appearance::Rules::VANILLA,
                camera: game::production::CameraRules::new(world_camera::Rules::REFERENCE),
                doors: world_doors::Rules::default(),
                health: samus_health::Rules::REFERENCE,
                hud: interface_hud::Rules::default(),
                ceres: boss_ceres_ridley::Rules::default(),
                arrival: screens_arrival::Rules::REFERENCE,
            },
            room,
            content,
            hud,
            row,
        })
    }

    /// The held facts and State as a seed in play holds them.
    fn seeded(
        seed: &FrameRecord,
        content: &connected::Content,
        rooms: &game::production::room_loading::Rooms,
    ) -> Result<(seed::Room, played::State), Outcome> {
        let (room, seeded) = seed::room(&seed.wram, content, &rooms.world)?;
        let (map_room, _) = room.map;
        let ceres_doors = ceres_doors::seed(&seed.wram)?;
        let state = played::State {
            samus: samus::seed(&seed.wram, &content.appearance)?,
            camera: Model(camera::seed(&seed.wram, &camera::Content)?),
            passage: played::passage::State {
                doors: Doors::Play,
                doorway: None,
                room: seeded,
                map_room,
            },
            palette: colours::palette(&seed.wram)?,
            health: Model(health_tail::decode(&seed.wram, None)?),
            hud: Model(super::hud::decode_state(&seed.wram)?),
            frame: frame::seed(&seed.wram)?,
            station: station::seed(&seed.wram, ceres_doors, room.ceres)?,
            arrival: arrival::seed(&seed.wram)?,
        };
        Ok((room, state))
    }

    /// Play the interval ending at `cut` and compare its owners with the record at `cut`.
    fn step(
        &mut self,
        controller: Controller,
        previous: &FrameRecord,
        current: &FrameRecord,
        cut: u64,
    ) -> Result<(), Outcome> {
        // While a new game's start is made ready no frame of play reads the pad, and the
        // preparation's own waits for the frame's end are its lag.
        let preparing = self.state.arrival.preparing();
        // The movie's pad is the input; the game's own latch only checks its alignment, so a
        // stale or early pad is a harness fault rather than a model difference.
        if !preparing
            && polled(previous)
            && word(&previous.wram, source::JOYPAD1_LASTKEYS)? != controller.held
        {
            return Err(data_failure(None, "played pad alignment"));
        }
        let doors = self.state.passage.doors;
        // In play, a lag NMI finds her frame part-played; only a passage plays on through one.
        if !preparing && doors == Doors::Play && !finished(current) {
            return Err(Outcome::Unsupported {
                cut: None,
                dependency: "lag cut in play (T1)".into(),
            });
        }
        // A lag cut through the contents' fetch finds it part-done, and the HUD with it: Ridley's
        // placing blanks the mini-map within the fetch, which the run takes as done on its first
        // frame. At every other cut the HUD is as its frame left it.
        let hud_settled = finished(current) || !fetch_under_way(doors);
        let mut inputs = self.room.inputs.clone();
        inputs.held = movement::pad(controller.held);
        inputs.pressed = movement::pad(controller.pressed);
        inputs.parity = parity(self.state.frame.finished_frames);
        let passing = played::passage::Passing {
            // The borrowed lag: the last NMI found the frame's work finished, so the next began.
            work_begun: finished(previous),
            work_done: finished(current) || fetching_contents(doors),
            audio: played::passage::PassageAudio {
                sounds_sent: self.room.sounds_sent,
                music_ready: music_ready(&previous.wram)?,
            },
            timer_running: self.room.timer_running,
            progress: &self.room.progress,
            // The borrowed random number, only through a passage; in play the loop carries it.
            random: match doors {
                Doors::Play => None,
                Doors::Passing(_) => Some(frame::random(&previous.wram)?),
            },
            start: arrival::start(&previous.wram, &current.wram)?,
        };
        let hud_frame = hud::frame(&self.room.hud, controller);
        self.state
            .step(&inputs, &passing, &hud_frame, &self.played, &self.rules)
            .map_err(|fault| Outcome::Unsupported {
                cut: Some(cut),
                dependency: format!("played frame {fault:?}"),
            })?;
        let (expected, actual) = if self.state.arrival.preparing() {
            // The preparation's working values are its own: only the arrival and the random
            // number are the run's.
            let mut expected = arrival::expected(&current.wram, false)?;
            expected.push(frame::random_observation(frame::random(&current.wram)?));
            let mut actual = arrival::actual(self.state.arrival);
            actual.push(frame::random_observation(self.state.frame.random));
            (expected, actual)
        } else {
            if preparing {
                // She arrives: the load flow puts the HUD up, which no owner here does, so it
                // is held from the record and not compared on this cut; the mini-map's power is
                // her arrival's.
                self.state.hud =
                    Model(start::hud_put_up(&current.wram, self.state.hud.0.mini_map)?);
            }
            self.compared(current, hud_settled && !preparing, cut)?
        };
        compare(&expected, &actual).map_err(|fault| match fault {
            CompareFault::Difference(field) => Outcome::Difference { cut, field },
            CompareFault::EmptyExpected
            | CompareFault::DuplicateField(_)
            | CompareFault::MissingActual(_) => data_failure(Some(cut), "played comparison schema"),
        })
    }

    /// Every owner's fields at `current`, as the record and as the played State hold them.
    fn compared(
        &self,
        current: &FrameRecord,
        hud_settled: bool,
        cut: u64,
    ) -> Result<(Vec<Observation>, Vec<Observation>), Outcome> {
        let expected = expected(
            current,
            &self.content.appearance,
            &self.hud,
            hud_settled,
            arrival::counted(self.state.arrival),
            self.row,
            cut,
        )?;
        let actual = actual(
            &self.state,
            &self.content.appearance,
            &self.hud,
            finished(current),
            hud_settled,
        )?;
        Ok((expected, actual))
    }
}

#[cfg(test)]
mod tests {
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

    /// The Ceres rows played back to back from cut 0, each on through the rooms she enters.
    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn ceres_rows_played_back_to_back() {
        let sources = sources();
        for row in [
            "chain-any-0002-DF8D",
            "chain-any-0003-DFD7",
            "chain-any-0004-E021",
            "chain-any-0005-E06B",
        ] {
            let report = super::play(&sources, row, 0, 3000).unwrap();
            diagnostic(format_args!(
                "PLAYED_ALONE row={} start={} held={} stop={:?}",
                report.row, report.start, report.held, report.stop
            ));
        }
    }

    /// A new game's arrival at Ceres from the intro's last frame: alone to its last cut, then on
    /// into Ceres's first room and the rooms she enters. `SM_PLAYED_END` asks for one end.
    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn ceres_arrival_played_alone_and_on() {
        let sources = sources();
        let ends = std::env::var("SM_PLAYED_END")
            .ok()
            .and_then(|end| end.parse().ok())
            .map_or_else(|| vec![222, 3223], |end| vec![end]);
        for end in ends {
            let report = super::play(&sources, "any-ceres-arrival", 0, end).unwrap();
            diagnostic(format_args!(
                "PLAYED_ALONE row={} start={} end={end} held={} stop={:?}",
                report.row, report.start, report.held, report.stop
            ));
        }
    }

    /// Ceres's first room from her arrival: alone to its last cut, then on through the rooms
    /// she enters.
    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn ceres_room_one_played_alone_and_on() {
        let sources = sources();
        for end in [411, 3000] {
            let report = super::play(&sources, "chain-any-0001-DF45", 0, end).unwrap();
            diagnostic(format_args!(
                "PLAYED_ALONE row={} start={} end={end} held={} stop={:?}",
                report.row, report.start, report.held, report.stop
            ));
        }
    }

    /// Ridley's room in Ceres from its seed, with Ridley and the baby waiting in the dark.
    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn ceres_ridleys_room_played_alone() {
        let sources = sources();
        let report = super::play(&sources, "chain-any-0006-E0B5", 0, 1785).unwrap();
        diagnostic(format_args!(
            "PLAYED_ALONE row={} start={} held={} stop={:?}",
            report.row, report.start, report.held, report.stop
        ));
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
                report.seed_held,
                report.borrowed
            ));
        }
    }
}
