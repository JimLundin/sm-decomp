//! Play a room alone from its seed: the movie's pads in, whole frames out, compared at each cut.
//!
//! This is the rung measure (MODEL decision 18): production State is admitted once from the
//! seed record and then carried with nothing fed back. Each cut's only new input is the pad
//! the movie presented at the preceding vblank; the oracle's later records are read for
//! comparison alone. Room facts that no selected owner produces yet are taken once from the
//! seed, or held at a constant, for the whole run; [`SEED_HELD`] names them. Whether each cut's frame of work
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
//! `quake`, `health`, `hud`, `frame` and `arrival`, with `station` the station's State; the
//! camera's are
//! the camera replay's. `platform` holds the borrowed lag, pad and audio answers, `seed` the
//! facts held from the seed, and `rows` the chain of rows a run plays through. A row from the
//! console's power-on plays the front end alone, from its own power-on State with no seed, on
//! into the intro: `power_on`, with the screens' fields from the title on in `screens`. A row
//! seeded at the flight to Ceres's first frame plays the flight into the new game's preparation:
//! `flight`. A row seeded as the intro begins or inside its last pages plays the intro alone,
//! from its seed: `intro`.

mod arrival;
mod ceres_doors;
mod colours;
mod flight;
mod frame;
mod health;
mod hud;
pub(super) mod intro;
mod passage;
pub(super) mod platform;
mod power_on;
mod quake;
mod ridley;
pub(super) mod rows;
mod samus;
mod screens;
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
    sounds_sent,
};
use std::path::Path;
use world_doors::Doors;

/// Room facts taken from the seed and held, or held at a constant, because no selected owner
/// produces them yet. Each one is a missing owner; the list shrinks as owners join the played
/// loop.
pub const SEED_HELD: &[&str] = &[
    "equipped items",
    "time running",
    "no periodic energy loss",
    "Ridley's hits taken, held at 0: her shots against him have no owner here (the game's 11 is under the 50 and 100 his rules read)",
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
    "progress, which chooses the next room's version, her suit and her beams, and whether the station's boss is beaten",
    "Ceres station stage, but where Ridley's placing sets it",
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
    "audio: whether every sound request has been sent as the frame of work begins",
    "the beginning of the prepared room's final music service (R2 and C1's loading, T1's lag)",
    "the start ready: the frame a new game's start is made ready (R2 and C1's loading, T1's lag)",
    "the escape countdown's start: the timer set between two records, at the time the second holds, until the station's boss raises it",
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
    /// The streams this kind of run doesn't compare, because no owner here produces them.
    pub masked: &'static [&'static str],
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
        fields.extend(hud::countdown(super::hud::countdown_origin(wram)?));
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
        fields.extend(hud::countdown(state.hud.0.countdown));
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

fn read_lock(sources: &Sources) -> Result<crate::lock::Lock, Outcome> {
    let text = std::fs::read_to_string(&sources.lock)
        .map_err(|_| data_failure(None, "record lock file"))?;
    crate::lock::Lock::parse(&text).map_err(|_| data_failure(None, "record lock schema"))
}

fn open_rows<'a>(
    sources: &'a Sources,
    lock: &'a crate::lock::Lock,
    row: &str,
) -> Result<rows::Rows<'a>, Outcome> {
    // The lock is the repository's `scenarios/locks/<oracle>.toml`, under the tables of seeds.
    let root = sources
        .lock
        .ancestors()
        .nth(3)
        .ok_or_else(|| data_failure(None, "played lock path"))?;
    rows::Rows::open(lock, root, &sources.cache, row)
}

/// Play `row` from its cut `start` through `end` inclusive with the movie's pads alone. Cuts
/// past the row's last go on through the rows that continue it, without a new seed.
///
/// A row from the console's power-on plays from its cut 0 with no seed.
///
/// A row seeded at the flight to Ceres's first frame plays the flight into the preparation.
///
/// A row seeded as the intro begins or inside its last pages plays the intro from its seed.
///
/// # Errors
/// Missing or mismatched lock, cache or content refuse the run before any cut is played, and a
/// cut past every row that continues the run, or a row that doesn't follow on, stops it.
pub fn play(sources: &Sources, row: &str, start: u64, end: u64) -> Result<Report, Outcome> {
    let lock = read_lock(sources)?;
    let mut rows = open_rows(sources, &lock, row)?;
    let id = crate::lock::RowId(row.to_owned());
    if lock
        .row(&id)
        .is_some_and(|listed| listed.start == crate::lock::Start::PowerOn)
    {
        return power_on::play(&mut rows, &sources.rom, row, start, end);
    }
    let seed = rows.record(start)?;
    if flight::starts(&seed.wram).map_err(|outcome| at(start, outcome))? {
        return flight::play(&mut rows, &sources.rom, &seed, row, start, end);
    }
    if let Some(part) = intro::starts(&seed.wram).map_err(|outcome| at(start, outcome))? {
        return intro::play(&mut rows, &sources.rom, seed, part, row, start, end);
    }
    let content = connected::Content {
        movement: movement::Content::load(&sources.rom)?,
        appearance: appearance::Content::load(&sources.rom)?,
    };
    let report = |held, stop| Report {
        row: row.to_owned(),
        start,
        held,
        stop,
        seed_held: SEED_HELD,
        borrowed: BORROWED,
        masked: &[],
    };
    let mut run = match Run::admit(&seed, &content, &sources.rom, row) {
        Ok(run) => run,
        Err(outcome) => return Ok(report(0, Stop::At(at(start, outcome)))),
    };
    let controller = match Controller::seeded(&seed.wram) {
        Ok(controller) => controller,
        Err(outcome) => return Ok(report(0, Stop::At(at(start, outcome)))),
    };
    // A seed that read the pad read the movie's: a stale or early pad is a harness fault.
    if polled(&seed) && seed.pad.0 != controller.held {
        let misaligned = data_failure(None, "played pad alignment");
        return Ok(report(0, Stop::At(at(start, misaligned))));
    }
    let (held, stop) = run.through(&mut rows, seed, controller, start.saturating_add(1)..=end)?;
    Ok(report(held, stop))
}

// Comparison starts at the first published cut and includes the first ordinary updates.
const HUD_SETUP_CUT: u64 = 184;
fn opening_witness((row, cut): (&str, u64)) -> bool {
    row == "any-ceres-arrival" && (HUD_SETUP_CUT..=222).contains(&cut)
}

/// The interval a cut ends: the records at its ends, and where its cut lies, as the row that
/// holds it and the cut there.
#[derive(Clone, Copy)]
struct Frame<'a> {
    previous: &'a FrameRecord,
    current: &'a FrameRecord,
    place: (&'a str, u64),
    cut: u64,
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
    upper_row: Option<interface_hud::UpperRow>,
}

impl<'a> Run<'a> {
    fn admit(
        seed: &FrameRecord,
        content: &'a connected::Content,
        path: &Path,
        row: &'a str,
    ) -> Result<Self, Outcome> {
        let arrival = arrival::seed(&seed.wram)?;
        let fresh = if arrival.preparing() {
            Some((arrival, frame::seed(&seed.wram)?, quake::seed(&seed.wram)?))
        } else {
            None
        };
        Self::load(seed, content, path, row, fresh)
    }

    fn after_flight(
        seed: &FrameRecord,
        content: &'a connected::Content,
        path: &Path,
        row: &'a str,
        arrival: played::arrival::State,
        frame: played::frame::State,
    ) -> Result<Self, Outcome> {
        let quake = quake::seed(&seed.wram)?;
        Self::load(seed, content, path, row, Some((arrival, frame, quake)))
    }

    fn load(
        seed: &FrameRecord,
        content: &'a connected::Content,
        path: &Path,
        row: &'a str,
        fresh: Option<(
            played::arrival::State,
            played::frame::State,
            world_quake::State,
        )>,
    ) -> Result<Self, Outcome> {
        let bad = || data_failure(None, "played ROM image");
        let image = Rom::new(std::fs::read(path).map_err(|_| bad())?).map_err(|_| bad())?;
        let start = import::start::import(image.lorom())
            .map_err(|_| data_failure(None, "played new game's start"))?;
        let rooms = seed::rooms(&image)?;
        let ridley = ridley::content(&rooms.actors.ceres_ridley)?;
        let tail_worth = samus_health::WholeEnergy::new(rooms.actors.ceres_tail_worth)
            .ok_or_else(|| data_failure(None, "Ceres Ridley's tail's worth"))?;
        // A seed in a new game's preparation holds the intro's RAM: State comes from the start.
        let (room, state) = if let Some((arrival, frame, quake)) = fresh {
            start::admit(&seed.wram, &start, &rooms, arrival, frame, quake)?
        } else {
            Self::seeded(seed, content, &rooms)?
        };
        let hud = super::hud::Content::load(path)?;
        let (_, area) = room.map;
        let opening_hud = if fresh.is_some() {
            match import::start::hud(image.lorom(), &start) {
                Ok(asset) => Some(hud::opening(&asset)?),
                Err(import::start::HudError::Room) => None,
                Err(_) => return Err(data_failure(None, "played opening HUD import")),
            }
        } else {
            None
        };
        Ok(Self {
            state,
            played: played::Content {
                movement: content.movement.movement.clone(),
                appearance: content.appearance.appearance.clone(),
                colour_groups: colours::groups(),
                rooms,
                hud: hud.pictures,
                opening_hud,
                map: hud.map_of(area)?.clone(),
                start_x: core::num::Wrapping(start.her.0),
                ridley,
                tail_worth,
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
            upper_row: None,
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
            quake: Model(quake::seed(&seed.wram)?),
            arrival: arrival::seed(&seed.wram)?,
        };
        Ok((room, state))
    }

    /// Continue room play from the active owners at the cut before this range.
    fn through(
        &mut self,
        rows: &mut rows::Rows<'_>,
        mut previous: FrameRecord,
        mut controller: Controller,
        cuts: core::ops::RangeInclusive<u64>,
    ) -> Result<(u64, Stop), Outcome> {
        let mut held: u64 = 0;
        for cut in cuts {
            let current = rows.record(cut)?;
            if current.video_frame != previous.video_frame.wrapping_add(1) {
                return Err(data_failure(Some(cut), "played rows join"));
            }
            let (row, local) = rows.place(cut)?;
            if let Err(outcome) = self.step(controller, &previous, &current, cut, (&row, local)) {
                return Ok((held, Stop::At(at(cut, outcome))));
            }
            held = held.saturating_add(1);
            controller = controller.read(current.pad.0, polled(&current));
            previous = current;
        }
        Ok((held, Stop::End))
    }

    /// Play the interval ending at `cut` and compare its owners with the record at `cut`.
    fn step(
        &mut self,
        controller: Controller,
        previous: &FrameRecord,
        current: &FrameRecord,
        cut: u64,
        place: (&str, u64),
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
                sounds_sent: sounds_sent(&previous.wram)?,
                music_ready: music_ready(&previous.wram)?,
            },
            progress: &self.room.progress,
            // The borrowed random number, only through a passage; in play the loop carries it.
            random: match doors {
                Doors::Play => None,
                Doors::Passing(_) => Some(frame::random(&previous.wram)?),
            },
            start: arrival::start(previous, current)?,
        };
        let start = super::hud::countdown_start(&previous.wram, &current.wram)?;
        let hud_frame = hud::frame(&self.room.hud, controller, start);
        let outputs = self
            .state
            .step(&inputs, &passing, &hud_frame, &self.played, &self.rules)
            .map_err(|fault| Outcome::Unsupported {
                cut: Some(cut),
                dependency: format!("played frame {fault:?}"),
            })?;
        // A frame of play gives the escape's scene an expiry answer; a passage's gives none.
        let expiry = outputs.hud.map(|hud| hud.countdown.expiry);
        let published = outputs.opening_hud.is_some();
        if let Some(row) = outputs.opening_hud {
            self.upper_row = Some(row);
        }
        let (mut expected, mut actual) = if self.state.arrival.preparing() {
            // Before the fade the loading path is still preparing the room. Compare the
            // arrival, the carried frame clock and the HUD once setup publishes it.
            let mut expected = arrival::expected(&current.wram, false)?;
            expected.push(frame::random_observation(frame::random(&current.wram)?));
            let mut actual = arrival::actual(self.state.arrival);
            actual.push(frame::random_observation(self.state.frame.random));
            expected.extend(frame::expected(&current.wram)?);
            actual.extend(frame::actual(self.state.frame));
            if opening_witness(place) {
                expected.extend(hud::parts(super::hud::expected(&current.wram, &self.hud)?));
                expected.extend(hud::countdown(super::hud::countdown_origin(&current.wram)?));
                actual.extend(hud::parts(super::hud::initial(
                    &self.state.hud.0,
                    &self.hud,
                )?));
                actual.extend(hud::countdown(self.state.hud.0.countdown));
            }
            (expected, actual)
        } else {
            let frame = Frame {
                previous,
                current,
                place,
                cut,
            };
            self.compared(&frame, hud_settled, expiry, outputs.quake)?
        };
        if preparing && !self.state.arrival.preparing() {
            let fresh = super::progress::decode(&current.wram)?;
            let same = |value| Observation::quantity("progress.fresh-file", i64::from(value));
            expected.push(same(true));
            actual.push(same(fresh == self.room.progress));
        }
        if place.0 == "any-ceres-arrival" {
            let publication = |value| Observation::quantity("hud.setup-publish", i64::from(value));
            expected.insert(0, publication(place.1 == HUD_SETUP_CUT));
            actual.push(publication(published));
        }
        if opening_witness(place) {
            expected.extend(hud::video_expected(current)?);
            actual.extend(hud::video_actual(
                self.upper_row.as_ref(),
                &self.state.hud.0,
            )?);
        }
        compare(&expected, &actual).map_err(|fault| match fault {
            CompareFault::Difference(field) => Outcome::Difference { cut, field },
            CompareFault::EmptyExpected
            | CompareFault::DuplicateField(_)
            | CompareFault::MissingActual(_) => data_failure(Some(cut), "played comparison schema"),
        })
    }

    /// Every owner's fields at the frame's cut, as the record and as the played State hold
    /// them, with the room quake's `update`, if the frame had one.
    fn compared(
        &self,
        frame: &Frame<'_>,
        hud_settled: bool,
        expiry: Option<interface_hud::CountdownExpiry>,
        update: Option<world_quake::Effects>,
    ) -> Result<(Vec<Observation>, Vec<Observation>), Outcome> {
        let Frame {
            previous,
            current,
            place,
            cut,
        } = *frame;
        let mut expected = expected(
            current,
            &self.content.appearance,
            &self.hud,
            hud_settled,
            arrival::counted(self.state.arrival),
            self.row,
            cut,
        )?;
        let mut actual = actual(
            &self.state,
            &self.content.appearance,
            &self.hud,
            finished(current),
            hud_settled,
        )?;
        // A frame of play answers the escape's scene whether its time has run out.
        if let Some(expiry) = expiry {
            expected.push(hud::expiry_expected(&current.wram)?);
            actual.push(hud::expiry_actual(expiry));
        }
        if finished(current) {
            expected.extend(quake::expected(previous, current, place)?);
            actual.extend(quake::actual(&self.state, update));
        }
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

    /// Every reviewed run of `scenarios/replay/played.toml`, or those `SM_PLAYED` names, played
    /// in this one process (`tools/lane/played.sh`), each held to the file's `held` and `stop`.
    /// A run is named `<row>/<start>-<end>`, and a bare row names all its runs. The file names
    /// a failed stop by its field, or by the missing piece that ends the dependency's name.
    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn reviewed_runs_hold() {
        use super::{Outcome, Stop};
        #[derive(serde::Deserialize)]
        struct Listed {
            run: Vec<Listing>,
        }
        #[derive(serde::Deserialize)]
        struct Listing {
            row: String,
            start: u64,
            end: u64,
            held: u64,
            stop: toml::Value,
        }
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../scenarios/replay/played.toml"
        );
        let listed: Listed = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        assert!(!listed.run.is_empty(), "{path} lists no run");
        let name = |run: &Listing| format!("{}/{}-{}", run.row, run.start, run.end);
        let asked = std::env::var("SM_PLAYED").unwrap_or_default();
        let asked: Vec<&str> = asked.split_whitespace().collect();
        let selects = |asked: &str, run: &Listing| asked == run.row || asked == name(run);
        let unknown: Vec<_> = asked
            .iter()
            .filter(|asked| !listed.run.iter().any(|run| selects(asked, run)))
            .collect();
        assert!(
            unknown.is_empty(),
            "SM_PLAYED names no run of {path}: {unknown:?}"
        );
        let sources = sources();
        let mut wrong = Vec::new();
        for run in &listed.run {
            if !asked.is_empty() && !asked.iter().any(|asked| selects(asked, run)) {
                continue;
            }
            let failed = run.stop.get("failed").and_then(toml::Value::as_str);
            let after = run.start.saturating_add(run.held).saturating_add(1);
            let stated = |identity: &str| {
                failed.is_some_and(|failed| {
                    identity == failed || identity.ends_with(&format!(" {failed}"))
                })
            };
            let played = super::play(&sources, &run.row, run.start, run.end);
            let holds = played.as_ref().is_ok_and(|report| {
                report.held == run.held
                    && match &report.stop {
                        Stop::End => run.stop.as_str() == Some("end"),
                        Stop::At(Outcome::Difference { cut, field }) => {
                            *cut == after && stated(field)
                        }
                        Stop::At(
                            Outcome::Unsupported {
                                cut: Some(cut),
                                dependency: identity,
                            }
                            | Outcome::DataFailure {
                                cut: Some(cut),
                                source: identity,
                            },
                        ) => *cut == after && stated(identity),
                        Stop::At(
                            Outcome::Pass
                            | Outcome::Unsupported { cut: None, .. }
                            | Outcome::DataFailure { cut: None, .. }
                            | Outcome::CheckpointFailure { .. },
                        ) => false,
                    }
            });
            let seen = match &played {
                Ok(report) => format!("held={} stop={:?}", report.held, report.stop),
                Err(outcome) => format!("refused={outcome:?}"),
            };
            let name = name(run);
            diagnostic(format_args!(
                "PLAYED {name} {seen} reviewed held={} stop={} {}",
                run.held,
                run.stop,
                if holds { "HOLDS" } else { "DIFFERS" }
            ));
            if !holds {
                wrong.push(name);
            }
        }
        assert!(
            wrong.is_empty(),
            "played runs that differ from {path}: {wrong:?}"
        );
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
