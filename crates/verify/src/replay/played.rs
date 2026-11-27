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
mod contact;
pub(super) mod explosion;
mod fight_picture;
mod flight;
pub(super) mod flyby_scope;
mod frame;
pub(super) mod getaway;
mod health;
pub(super) mod hud;
pub(super) mod indexed;
pub(super) mod intro;
mod passage;
pub(super) mod platform;
mod play_time;
mod power_on;
mod quake;
#[cfg(test)]
mod return_fade;
mod return_timing;
mod ridley;
#[cfg(test)]
mod ridley_body_picture;
#[cfg(test)]
mod ridley_door_picture;
pub(super) mod ridley_ribs;
#[cfg(test)]
mod room_picture;
mod room_video;
pub(super) mod rows;
pub(super) mod samus;
pub(super) mod samus_picture;
mod screens;
pub(super) mod seed;
mod shaft;
pub(super) mod shaft_dust;
pub(super) mod shaft_native;
mod shaft_restore;
pub(super) mod shaft_sound;
mod start;
mod station;
mod steam;
mod warning;
mod zebes;

use super::compare::{CompareFault, Observation, compare};
use super::result::Outcome;
use super::{
    Sources, appearance, camera, connected, data_failure, health_tail, movement, read_lock,
};
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

fn reference_rules() -> played::Rules {
    played::Rules {
        movement: samus_movement::Rules::default(),
        appearance: samus_appearance::Rules::VANILLA,
        camera: game::production::CameraRules::new(world_camera::Rules::REFERENCE),
        doors: world_doors::Rules::default(),
        health: samus_health::Rules::REFERENCE,
        hud: interface_hud::Rules::default(),
        ceres: boss_ceres_ridley::Rules::default(),
        arrival: screens_arrival::Rules::REFERENCE,
    }
}

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
    "progress at room entry, carried by production and updated when the warning reports defeat",
    "Ceres station stage, but where Ridley's placing sets it",
    "the full cartridge battery for an ordinary room seed, with no save request",
    "from a new game's seed: the platform's counts of finished frames and the random number, which run on through the preparation",
];

/// Inputs read from the record at each cut, because their owner is outside the rungs
/// (decision 35). The list shrinks as the platform's costs are modelled.
pub const BORROWED: &[&str] = &[
    "lag: whether each cut's frame of work finished",
    "lag: the return doorway darkening turn's completed-work interrupt site before frame publication",
    "lag: the interrupted escape-shaft population clear and its slot cursor",
    "lag: completed shaft background work during the interrupted contents fetch",
    "lag: the contents' fetch done on its first frame, within the slide",
    "lag: whether a frame of work begins, the last one's done, when the random number is drawn",
    "the random number through a passage, except the draws held from placing the room's vents to the end of that frame",
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
    frame: &Frame<'_>,
    content: &appearance::Content,
    hud: &super::hud::Content,
    hud_settled: bool,
    counted: bool,
    retiring_return: bool,
    row: &str,
) -> Result<Vec<Observation>, Outcome> {
    let wram = &frame.current.wram;
    let finished = finished(frame.current);
    let mut fields = samus::expected(&frame.previous.wram, wram, content, row, frame.cut)?;
    fields.extend(camera::expected(wram)?);
    let shaft_elevator = shaft_native::in_elevator(frame)?;
    let passing = !shaft_elevator && passage::passing(wram)?;
    fields.push(passage::observation(passing));
    // In play every draw is the loop's own, so a frame's draws are compared where it finished.
    if (finished && !passing) || shaft_native::compared(frame) {
        fields.push(frame::random_observation(frame::random(wram)?));
    }
    if finished {
        fields.extend(colours::observations(&colours::palette(wram)?));
        fields.extend(ceres_doors::expected(wram)?);
        if !retiring_return {
            fields.extend(ridley::expected(wram)?);
        }
        fields.push(steam::state(steam::decode_shaft(wram)?.iter()));
    }
    fields.extend(shaft::expected(wram, shaft_elevator)?);
    fields.extend(health::parts(&health_tail::decode(wram, None)?));
    if hud_settled {
        fields.extend(hud::parts(super::hud::expected(wram, hud)?));
        fields.extend(hud::countdown(super::hud::countdown_origin(wram)?));
    }
    fields.extend(frame::expected(wram)?);
    if !shaft_elevator {
        fields.extend(arrival::expected(wram, counted)?);
    }
    Ok(fields)
}

/// The owners' fields as the played State holds them.
fn actual(
    state: &played::State,
    content: &appearance::Content,
    hud: &super::hud::Content,
    finished: bool,
    hud_settled: bool,
    retiring_return: bool,
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
        if !retiring_return {
            fields.extend(ridley::actual(&state.station));
        }
        fields.push(steam::station(&state.station));
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
    if super::zebes::starts(&seed).map_err(|outcome| at(start, outcome))? {
        return zebes::play(&mut rows, &sources.rom, seed, row, start, end);
    }
    if explosion::shaft_starts(&seed)? {
        return explosion::play(&mut rows, &sources.rom, seed, row, start, end);
    }
    if let Some(before) = start.checked_sub(1)
        && super::explosion::pending(&seed)?
        && super::explosion::enters_from(&rows.record(before)?)?
    {
        return explosion::play(&mut rows, &sources.rom, seed, row, start, end);
    }
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
    let controller = match seed_controller(&seed) {
        Ok(controller) => controller,
        Err(outcome) => return Ok(report(0, Stop::At(at(start, outcome)))),
    };
    let mut battery = match (!run.state.arrival.preparing())
        .then(|| seed::Battery::load(&seed, &sources.rom))
        .transpose()
    {
        Ok(battery) => battery,
        Err(outcome) => return Ok(report(0, Stop::At(at(start, outcome)))),
    };
    let (held, stop) = run.through_observing(
        &mut rows,
        seed,
        controller,
        start.saturating_add(1)..=end,
        |record, cut| {
            battery
                .as_mut()
                .map_or(Ok(()), |battery| battery.held(record, cut))
        },
    )?;
    Ok(report(held, stop))
}

/// A seed that read the pad read the movie's.
fn seed_controller(seed: &FrameRecord) -> Result<Controller, Outcome> {
    let controller = Controller::seeded(&seed.wram)?;
    if polled(seed) && seed.pad.0 != controller.held {
        return Err(data_failure(None, "played pad alignment"));
    }
    Ok(controller)
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
    cut: u64,
    doors_before: Doors,
    publishes: bool,
    place: (&'a str, u64),
}

/// One played run's carried owners and the facts held from its seed.
#[derive(Clone)]
struct Run<'a> {
    state: played::State,
    played: played::Content,
    rules: played::Rules,
    room: seed::Room,
    content: &'a connected::Content,
    rom: &'a Path,
    hud: super::hud::Content,
    row: &'a str,
    drawing: ridley::flyby::Drawing,
    warning_picture: warning::Picture,
    picture_scope: PictureScope,
    return_timing: return_timing::ReturnTiming,
    cartridge: Option<power_on::LiveCartridge>,
    blackout: Option<game::production::blackout::LiveRoom>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PictureScope {
    Room,
    Fight,
    ShaftActors,
}

#[derive(Clone, Copy)]
enum Start<'a> {
    Room(&'a FrameRecord),
    Preparing(played::arrival::State, played::frame::State, start::Carried),
}

impl<'a> Run<'a> {
    fn admit(
        seed: &FrameRecord,
        content: &'a connected::Content,
        path: &'a Path,
        row: &'a str,
    ) -> Result<Self, Outcome> {
        let arrival = arrival::seed(&seed.wram)?;
        let origin = if arrival.preparing() {
            Start::Preparing(
                arrival,
                frame::seed(&seed.wram)?,
                start::Carried::seed(&seed.wram)?,
            )
        } else {
            Start::Room(seed)
        };
        Self::load(origin, content, path, row)
    }

    fn load(
        origin: Start<'_>,
        content: &'a connected::Content,
        path: &'a Path,
        row: &'a str,
    ) -> Result<Self, Outcome> {
        let bad = || data_failure(None, "played ROM image");
        let image = Rom::new(std::fs::read(path).map_err(|_| bad())?).map_err(|_| bad())?;
        let start = import::start::import(image.lorom())
            .map_err(|_| data_failure(None, "played new game's start"))?;
        let rooms = seed::rooms(&image)?;
        let flyby = import::ridley_flyby::import(image.lorom())
            .map_err(|_| data_failure(None, "Ridley flyby assets"))?;
        let ridley = ridley::content(&rooms.actors.ceres_ridley, &flyby)?;
        let tail_worth = samus_health::WholeEnergy::new(rooms.actors.ceres_tail_worth)
            .ok_or_else(|| data_failure(None, "Ceres Ridley's tail's worth"))?;
        // A seed in a new game's preparation holds the intro's RAM: State comes from the start.
        let (room, mut state) = match origin {
            Start::Preparing(arrival, frame, carried) => {
                start::admit(carried, &start, &rooms, arrival, frame)?
            }
            Start::Room(seed) => Self::seeded(seed, content, &rooms)?,
        };
        let hud = super::hud::Content::load(path)?;
        let language = match origin {
            Start::Preparing(_, _, carried) => carried.language,
            Start::Room(seed) => seed::language(&seed.wram)?,
        };
        let warning_art = import::ridley_warning::import(image.lorom()).ok();
        let warning = warning_art
            .clone()
            .and_then(|art| played::warning::Content::new(art, language));
        let drawing = match origin {
            Start::Preparing(..) => ridley::flyby::Drawing::unpublished(flyby),
            Start::Room(seed) => ridley::flyby::Drawing::new(flyby, seed),
        };
        let timer = import::countdown::import(image.lorom())
            .map_err(|_| data_failure(None, "warning countdown content"))?;
        let warning_seed = match origin {
            Start::Room(seed) if super::ridley_warning_native::is_warning_room(seed) => Some(seed),
            Start::Room(_) | Start::Preparing(..) => None,
        };
        if let Some(seed) = warning_seed {
            if seed.vram.len() != game::state::video::VRAM_WORDS {
                return Err(data_failure(None, "warning admitted drawing"));
            }
            state.room_video.video.memory.vram = seed.vram.to_vec();
        }
        let warning_picture =
            warning::Picture::new(timer.clone(), warning_art.as_ref(), warning_seed.is_some());
        let shaft = import::shaft::import(image.lorom()).ok();
        let art = room_video::ArrivalArt::load(image.lorom())?;
        Ok(Self {
            state,
            played: played::Content {
                clock_figures: Some(timer),
                standing: game::samus_appearance_bridge::elevator::Art::load(image.lorom()).ok(),
                shaft_track: shaft.as_ref().map(played::shaft::track),
                shaft_overlay_art: shaft,
                wall_jump_dust_art: Some(samus::dust_art(image.lorom())?),
                shaft_vent_art: import::shaft::vent::import(image.lorom()).ok(),
                shaft_burst_art: Some(shaft::burst_art(image.lorom())?),
                shaft_effects: import::countdown::shaft_effects(image.lorom())
                    .map_err(|_| data_failure(None, "shaft effects tilemap"))?,
                arrival_beam_tiles: Some(art.beam),
                arrival_overlay: Some(art.overlay),
                arrival_graphics: Some(art.graphics),
                shaft_raster: Some(
                    import::countdown::shaft_raster(image.lorom())
                        .map_err(|_| data_failure(None, "shaft raster content"))?,
                ),
                ridley_main_screen: import::countdown::ridley_main_screen(image.lorom()).ok(),
                movement: content.movement.movement.clone(),
                appearance: content.appearance.appearance.clone(),
                samus_body_art: Some(samus::body_art(image.lorom())?),
                samus_tiles: Some(
                    import::samus::tile_uploads(image.lorom())
                        .map_err(|_| data_failure(None, "ordinary Samus tile uploads"))?,
                ),
                ridley_picture: Some(
                    import::ridley_picture::import(image.lorom())
                        .map_err(|_| data_failure(None, "Ridley ordinary sprite art"))?,
                ),
                colour_groups: colours::groups(),
                rooms,
                hud: hud.pictures,
                opening_hud: Self::opening_hud(origin, image.lorom(), &start)?,
                map: hud.map_of(room.map.1)?.clone(),
                start_x: core::num::Wrapping(start.her.0),
                ridley,
                ridley_ribs_art: import::ridley_ribs::import(image.lorom()).ok(),
                warning,
                tail_worth,
            },
            rules: reference_rules(),
            room,
            content,
            rom: path,
            hud,
            row,
            drawing,
            warning_picture,
            picture_scope: PictureScope::Room,
            return_timing: return_timing::ReturnTiming::default(),
            cartridge: None,
            blackout: None,
        })
    }

    fn opening_hud(
        origin: Start<'_>,
        rom: data::rom::Lorom<'_>,
        start: &data::assets::start::NewGame,
    ) -> Result<Option<(interface_hud::OpeningFrame, interface_hud::OpeningPreview)>, Outcome> {
        if !matches!(origin, Start::Preparing(..)) {
            return Ok(None);
        }
        match import::start::hud(rom, start) {
            Ok(asset) => hud::opening(&asset).map(Some),
            Err(import::start::HudError::Room) => Ok(None),
            Err(_) => Err(data_failure(None, "played opening HUD import")),
        }
    }

    /// The held facts and State as a seed in play holds them.
    fn seeded(
        seed: &FrameRecord,
        content: &connected::Content,
        rooms: &game::production::room_loading::Rooms,
    ) -> Result<(seed::Room, played::State), Outcome> {
        let (mut room, mut seeded, progress) = seed::room(seed, content, &rooms.world)?;
        let (map_room, _) = room.map;
        let pending = seed::return_fetch(seed, &rooms.world)?;
        let awaiting_actors = pending.is_some_and(|seed| seed.awaiting_actors);
        let (ceres_doors, owns_roster) = if awaiting_actors {
            (Vec::new(), false)
        } else {
            ceres_doors::seed_with_roster(
                &seed.wram,
                types::World::solid_enemies(&seeded.geometry),
                pending.is_some(),
            )?
        };
        if owns_roster {
            let geometry = movement::level_world(&seed.wram, &content.movement, Vec::new())?;
            room.inputs.geometry = geometry.clone();
            seeded.geometry = geometry;
        }
        let mut room_video = played::room_video::RoomVideo::default();
        let station = station::seed(&seed.wram, ceres_doors, room.ceres)?;
        if station.ridley.is_some() {
            ridley_ribs::admit(&mut room_video.video, seed)?;
        }
        room_video.bg1_offset = seed::bg1_offset(&seed.wram)?;
        let state = played::State {
            room_video,
            shaft_sound: shaft_sound::seed(&seed.wram)?,
            play_time: play_time::read(&seed.wram)?,
            clock_picture: played::clock_picture::ClockPicture::default(),
            room_tiles: played::room_tiles::ShaftBackground::Pending,
            samus: samus::seed(&seed.wram, &content.appearance)?,
            camera: Model(camera::seed(&seed.wram, &camera::Content)?),
            progress,
            passage: played::passage::State {
                doors: pending.map_or(Doors::Play, |seed| Doors::Passing(seed.passage)),
                doorway: pending.map(|seed| seed.doorway),
                awaiting_actors,
                room: seeded,
                station: seed::station_selector(&seed.wram)?,
                map_room,
            },
            palette: colours::palette(&seed.wram)?,
            health: Model(health_tail::decode(&seed.wram, None)?),
            hud: Model(super::hud::decode_state(&seed.wram)?),
            frame: frame::seed(&seed.wram)?,
            station,
            quake: Model(quake::seed(&seed.wram)?),
            arrival: arrival::seed(&seed.wram)?,
        };
        Ok((room, state))
    }

    /// Continue room play from the active owners at the cut before this range.
    fn through(
        &mut self,
        rows: &mut rows::Rows<'_>,
        previous: FrameRecord,
        controller: Controller,
        cuts: core::ops::RangeInclusive<u64>,
    ) -> Result<(u64, Stop), Outcome> {
        self.witness_through(rows, previous, controller, cuts, |_, _| Ok(()))
    }

    fn through_observing(
        &mut self,
        rows: &mut rows::Rows<'_>,
        previous: FrameRecord,
        controller: Controller,
        cuts: core::ops::RangeInclusive<u64>,
        observe: impl FnMut(&FrameRecord, u64) -> Result<(), Outcome>,
    ) -> Result<(u64, Stop), Outcome> {
        self.through_with(rows, previous, controller, cuts, observe, |_, _| Ok(()))
    }

    fn witness_through(
        &mut self,
        rows: &mut rows::Rows<'_>,
        previous: FrameRecord,
        controller: Controller,
        cuts: core::ops::RangeInclusive<u64>,
        matched: impl FnMut(u64, Vec<Observation>) -> Result<(), Outcome>,
    ) -> Result<(u64, Stop), Outcome> {
        self.through_with(rows, previous, controller, cuts, |_, _| Ok(()), matched)
    }

    fn through_with(
        &mut self,
        rows: &mut rows::Rows<'_>,
        mut previous: FrameRecord,
        mut controller: Controller,
        cuts: core::ops::RangeInclusive<u64>,
        mut observe: impl FnMut(&FrameRecord, u64) -> Result<(), Outcome>,
        mut matched: impl FnMut(u64, Vec<Observation>) -> Result<(), Outcome>,
    ) -> Result<(u64, Stop), Outcome> {
        let mut held: u64 = 0;
        let end = *cuts.end();
        for cut in cuts {
            let current = rows.record(cut)?;
            if current.video_frame != previous.video_frame.wrapping_add(1) {
                return Err(data_failure(Some(cut), "played rows join"));
            }
            let (row, local) = rows.place(cut)?;
            match self.step(controller, &previous, &current, cut, (&row, local)) {
                Ok(fields) => {
                    #[cfg(test)]
                    if fields.iter().any(|field| field.field == play_time::FIELD) {
                        rows.clock_cuts.push(cut);
                    }
                    if let Err(outcome) = self
                        .compare_departure_room(&current, cut)
                        .and_then(|()| {
                            self.cartridge.as_mut().map_or(Ok(()), |cartridge| {
                                cartridge.observe(&current, cut, false)?;
                                power_on::compare_oam_dma(
                                    self.state.room_video.video.dma[0],
                                    &current,
                                    cut,
                                )
                            })
                        })
                        .and_then(|()| observe(&current, cut))
                    {
                        return Ok((held, Stop::At(at(cut, outcome))));
                    }
                    matched(cut, fields)?;
                }
                Err(outcome) => return Ok((held, Stop::At(at(cut, outcome)))),
            }
            held = held.saturating_add(1);
            if cut < end
                && self
                    .blackout
                    .as_ref()
                    .is_some_and(game::production::blackout::LiveRoom::awaits_blank)
            {
                let next = cut.saturating_add(1);
                match self.take_departure(controller, next) {
                    Ok((shaft, assets)) => {
                        let (more, stop) =
                            explosion::through_shaft(shaft, assets, rows, current, next..=end)?;
                        return Ok((held.saturating_add(more), stop));
                    }
                    Err(outcome) => return Ok((held, Stop::At(outcome))),
                }
            }
            controller = controller.read(current.pad.0, polled(&current));
            previous = current;
        }
        Ok((held, Stop::End))
    }

    fn compare_departure_room(&self, record: &FrameRecord, cut: u64) -> Result<(), Outcome> {
        let Some(started) = self
            .cartridge
            .as_ref()
            .and_then(|live| live.cartridge.started())
        else {
            return Ok(());
        };
        if self.blackout.is_none() {
            return Ok(());
        }
        explosion::carry::check_placement_and_maps(&self.state, started, record, cut)
    }

    fn take_departure(
        &mut self,
        controller: Controller,
        next: u64,
    ) -> Result<
        (
            game::production::escape_departure::Shaft,
            game::production::escape_departure::Content,
        ),
        Outcome,
    > {
        use game::production::escape_departure::Shaft;
        let unsupported = |dependency| Outcome::Unsupported {
            cut: Some(next),
            dependency,
        };
        let live = self.cartridge.as_ref().ok_or_else(|| {
            unsupported("live shaft departure needs the selected cartridge".into())
        })?;
        let (room, area, maps) = live
            .cartridge
            .started()
            .and_then(|selected| self.state.departure_room(selected))
            .filter(|(_, area, _)| *area == played::landing_site::Area::Ceres)
            .ok_or_else(|| {
                unsupported("live shaft departure needs Ceres placement and maps".into())
            })?;
        let shadows = self.state.departure_shadows().ok_or_else(|| {
            unsupported("live shaft departure needs the retained BG1 and BG2 shadows".into())
        })?;
        let landing = played::landing_site::State::new(area, maps, shadows);
        let location = (types::Area::Ceres, self.state.passage.station);
        let (locked_hud, hud_content) = self
            .state
            .locked_hud(&self.played, &hud::frame(&self.room.hud, controller, None));
        let assets =
            explosion::live_content(self.rom, &self.played, live.format.clone(), hud_content)?;
        let scene = self.blackout.take().ok_or_else(|| {
            data_failure(Some(next), "live shaft departure lost the pending blackout")
        })?;
        let live = self.cartridge.take().ok_or_else(|| {
            data_failure(
                Some(next),
                "live shaft departure lost the selected cartridge",
            )
        })?;
        let platform = self.state.room_video.pending_platform();
        let video = std::mem::take(&mut self.state.room_video.video);
        let blackout = scene
            .into_blackout(
                &self.state,
                video,
                live.cartridge,
                location,
                platform,
                locked_hud,
            )
            .map_err(|fault| unsupported(format!("live shaft departure {fault:?}")))?;
        Ok((
            Shaft {
                blackout,
                room,
                landing,
                progress: self.state.departure_progress(),
            },
            assets,
        ))
    }

    fn passage_timing(&mut self, current: &FrameRecord) -> return_timing::CutTiming {
        self.return_timing.cut(
            self.state.passage.doorway.map(|door| door.through),
            self.state.passage.doors,
            current,
        )
    }

    fn room_work(
        &mut self,
        previous: &FrameRecord,
        current: &FrameRecord,
        place: (&str, u64),
        owns_vent_draws: bool,
    ) -> Result<played::passage::Passing, Outcome> {
        let doors = self.state.passage.doors;
        let platform = self.passage_timing(current);
        Ok(played::passage::Passing {
            // The borrowed lag: the last NMI found the frame's work finished, so the next began.
            work_begun: finished(previous)
                && !shaft_native::fetch_retires(&self.state.station, doors),
            work_done: platform.ready || fetching_contents(doors),
            publishes: platform.publishes,
            actor_fetch: platform::actor_fetch(current),
            enemy_tiles: platform::door_enemy_tiles(previous, current),
            shaft_background: platform::shaft_background(current, doors),
            shaft_raster: platform::shaft_raster(previous, current)?,
            room_hdma: platform::room_hdma(previous, current)?,
            audio: played::passage::PassageAudio {
                shaft_reply: shaft_sound::reply(place, finished(current)),
                sounds_sent: sounds_sent(&previous.wram)?,
                music_ready: music_ready(&previous.wram)?,
            },
            // Borrow random draws during passages outside the vent placement work frame.
            random: (matches!(doors, Doors::Passing(_))
                && !owns_vent_draws
                && self.state.station.escape_shaft.is_none())
            .then(|| frame::random(&previous.wram))
            .transpose()?,
            start: arrival::start(previous, current)?,
        })
    }

    /// Play the interval ending at `cut` and compare its owners with the record at `cut`.
    fn step(
        &mut self,
        controller: Controller,
        previous: &FrameRecord,
        current: &FrameRecord,
        cut: u64,
        place: (&str, u64),
    ) -> Result<Vec<Observation>, Outcome> {
        self.step_with_outputs(controller, previous, current, cut, place)
            .map(|(fields, _)| fields)
    }

    fn step_with_outputs(
        &mut self,
        controller: Controller,
        previous: &FrameRecord,
        current: &FrameRecord,
        cut: u64,
        place: (&str, u64),
    ) -> Result<(Vec<Observation>, played::Outputs), Outcome> {
        // While a new game's start is made ready no frame of play reads the pad, and the
        // preparation's own waits for the frame's end are its lag.
        let preparing = self.state.arrival.preparing();
        if !preparing && polled(previous) {
            controller.check_poll(&previous.wram, cut)?;
        }
        let doors = self.state.passage.doors;
        let final_room_work = self
            .blackout
            .as_ref()
            .is_some_and(game::production::blackout::LiveRoom::needs_final_room_work)
            && super::blackout::final_room_overrun(previous, current);
        // A shaft pass can be complete while its publication waits for vblank end and the save.
        if !preparing && doors == Doors::Play && !finished(current) && !final_room_work {
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
        // The fetch can span lag cuts. Its vent draws are ours until that frame finishes.
        let owns_vent_draws = fetch_under_way(doors)
            && !fetching_contents(doors)
            && (!self.state.station.vents.is_empty() || self.state.station.escape_shaft.is_some())
            && !finished(previous);
        let passing = self.room_work(previous, current, place, owns_vent_draws)?;
        let start = if warning::selected(&self.state.station) {
            None
        } else {
            super::hud::countdown_start(&previous.wram, &current.wram)?
        };
        let hud_frame = hud::frame(&self.room.hud, controller, start);
        let standing_view = self.state.passage.room.entry.view.transform;
        let before = types::ceres_blackout::FinishedPasses(self.state.frame.finished_frames);
        let outputs = self
            .state
            .step(&inputs, &passing, &hud_frame, &self.played, &self.rules)
            .map_err(|fault| Outcome::Unsupported {
                cut: Some(cut),
                dependency: format!("played frame {fault:?}"),
            })?;
        self.blackout_turn(
            outputs.elevator_reach,
            before,
            passing.work_begun,
            standing_view,
            passing.publishes,
            cut,
        )?;
        let frame = Frame {
            previous,
            current,
            cut,
            doors_before: doors,
            publishes: passing.publishes,
            place,
        };
        let (mut expected, mut actual) = if self.state.arrival.preparing() {
            self.preparing_fields(current, place)?
        } else {
            self.compared(&frame, hud_settled, owns_vent_draws, &outputs)?
        };
        if preparing && !self.state.arrival.preparing() {
            let fresh = super::progress::decode(&current.wram)?;
            let same = |value| Observation::quantity("progress.fresh-file", i64::from(value));
            expected.push(same(true));
            actual.push(same(fresh == self.state.progress));
        }
        self.compare_room_publications(&outputs, &frame, &passing, (&mut expected, &mut actual))?;
        Ok((expected, outputs))
    }

    fn compare_room_publications(
        &self,
        outputs: &played::Outputs,
        frame: &Frame<'_>,
        passing: &played::passage::Passing,
        (expected, actual): (&mut Vec<Observation>, &mut Vec<Observation>),
    ) -> Result<(), Outcome> {
        let Frame {
            current,
            place,
            cut,
            ..
        } = *frame;
        if place.0 == "any-ceres-arrival" {
            let publication = |value| Observation::quantity("hud.setup-publish", i64::from(value));
            expected.insert(0, publication(place.1 == HUD_SETUP_CUT));
            actual.push(publication(outputs.opening_hud.is_some()));
        }
        if opening_witness(place) {
            expected.extend(hud::video_observations(&current.vram)?);
            actual.extend(hud::video_observations(
                &self.state.room_video.video.memory.vram,
            )?);
        }
        if self.picture_scope == PictureScope::Fight {
            fight_picture::append_observations(
                *frame,
                &self.state.room_video.video,
                outputs.attached_ridley_door.as_ref(),
                (expected, actual),
            )?;
        }
        room_video::compare(
            &self.state.room_video,
            outputs.shaft_overlay_placement.is_some(),
            passing,
            frame,
            expected,
            actual,
        )?;
        if let Some(scene) = &self.blackout {
            super::blackout::observe::live_scene(
                current,
                scene.scene(),
                &self.state.room_video.video,
                (expected, actual),
            )?;
        }
        shaft_dust::observations(self, frame, (expected, actual))?;
        ridley_ribs::compare(
            *frame,
            &self.state,
            &outputs.serviced_vram,
            (expected, actual),
        )?;
        expected.push(play_time::observation(play_time::read(&current.wram)?));
        actual.push(play_time::observation(self.state.play_time));
        compare(expected, actual).map_err(|fault| comparison_failure(cut, fault))?;
        // A wrong pose or hit can change the body picture on the same cut.
        samus_picture::compare(
            frame,
            &self.state.room_video.video,
            outputs.samus_body.clone(),
            expected,
        )?;
        Ok(())
    }

    fn blackout_turn(
        &mut self,
        reach: Option<boss_ceres_ridley::ElevatorLock>,
        before: types::ceres_blackout::FinishedPasses,
        worked: bool,
        view: game::state::video::Mode7,
        publishes: bool,
        cut: u64,
    ) -> Result<(), Outcome> {
        use game::production::blackout::LiveRoom;
        let unsupported = |fault| Outcome::Unsupported {
            cut: None,
            dependency: format!("live shaft blackout {fault:?}"),
        };
        match (&mut self.blackout, reach) {
            (None, Some(reach)) => {
                self.blackout = Some(
                    LiveRoom::reached(&mut self.state, reach, before, publishes)
                        .map_err(unsupported)?,
                );
            }
            (Some(_), Some(_)) => {
                return Err(Outcome::Difference {
                    cut,
                    field: "shaft.elevator-reach".into(),
                });
            }
            (Some(scene), None) if worked => {
                let standing = game::production::blackout::StandingFrame::from_locked(
                    &self.state.samus,
                    self.state.camera.0.position(),
                    view,
                );
                scene
                    .worked(&mut self.state, standing, publishes)
                    .map_err(unsupported)?;
            }
            (Some(_) | None, None) => {}
        }
        Ok(())
    }

    fn preparing_fields(
        &self,
        current: &FrameRecord,
        place: (&str, u64),
    ) -> Result<(Vec<Observation>, Vec<Observation>), Outcome> {
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
        Ok((expected, actual))
    }

    /// Every owner's fields at the frame's cut, as the record and as the played State hold
    /// them, with the room quake's `update`, if the frame had one.
    fn compared(
        &mut self,
        frame: &Frame<'_>,
        hud_settled: bool,
        owns_vent_draws: bool,
        outputs: &played::Outputs,
    ) -> Result<(Vec<Observation>, Vec<Observation>), Outcome> {
        let Frame {
            previous, current, ..
        } = *frame;
        let retiring_return =
            self.state.passage.doorway.is_some_and(|door| {
                door.through == data::assets::world::doorways::RIDLEY_TO_ESCAPE_58
            }) && matches!(
                self.state.passage.doors,
                Doors::Passing(world_doors::Passage {
                    phase: world_doors::Phase::Arriving { .. },
                    ..
                })
            );
        let mut expected = expected(
            frame,
            &self.content.appearance,
            &self.hud,
            hud_settled,
            arrival::counted(self.state.arrival),
            retiring_return,
            self.row,
        )?;
        let mut actual = actual(
            &self.state,
            &self.content.appearance,
            &self.hud,
            finished(current),
            hud_settled,
            retiring_return,
        )?;
        self.population_fields(frame, retiring_return, &mut expected, &mut actual)?;
        if shaft_native::compared(frame) {
            expected.extend(shaft_native::expected(frame)?);
            actual.extend(shaft_native::actual(&self.state, outputs));
        }
        // A frame of play answers the escape's scene whether its time has run out.
        if let Some(hud) = &outputs.hud {
            expected.push(hud::expiry_expected(&current.wram)?);
            actual.push(hud::expiry_actual(hud.countdown.expiry));
            // These rooms have no other contact hitter.
            if !self.state.station.vents.is_empty() || self.state.station.escape_shaft.is_some() {
                expected.insert(0, steam::expected_hit(&previous.wram, &current.wram)?);
                actual.push(steam::actual_hit(outputs.steam_hit));
            }
        }
        if finished(current) {
            if expected
                .iter()
                .any(|field| field.field == "ridley.tail.pace")
            {
                expected.push(contact::expected(
                    "ridley.tail.hit",
                    &previous.wram,
                    &current.wram,
                )?);
                expected.push(ridley::expected_swish(previous, current)?);
            }
            if !retiring_return {
                expected.extend(ridley::retreat::expected(current)?);
                expected.extend(ridley::flyby::expected(previous, current)?);
            }
            if self.state.station.warning.is_some() {
                // The old tail stays in State while the warning owns the room turn.
                actual.push(contact::actual("ridley.tail.hit", None));
                actual.push(ridley::swish(false));
            }
            if let Some(run) = outputs.ridley {
                actual.extend(ridley::retreat::actual(&self.state, Some(&run)));
                if let Some(effects) = run.getaway {
                    actual.extend(self.drawing.actual(&self.state, effects)?);
                }
                actual.push(contact::actual("ridley.tail.hit", run.hit));
                actual.push(ridley::swish(run.swish.is_some()));
                if let Some(places) = &run.tail_places {
                    actual.extend(ridley::places(places));
                }
            }
            expected.extend(quake::expected(
                previous,
                current,
                outputs.ridley.is_some_and(|run| run.getaway.is_some()),
            )?);
            actual.extend(quake::actual(&self.state, outputs.quake));
            if owns_vent_draws && !shaft_native::compared(frame) && passage::passing(&current.wram)?
            {
                expected.push(frame::random_observation(frame::random(&current.wram)?));
            }
        }
        self.warning_fields(frame, outputs, &mut expected, &mut actual)?;
        Ok((expected, actual))
    }

    fn population_fields(
        &self,
        frame: &Frame<'_>,
        retiring_return: bool,
        expected: &mut Vec<Observation>,
        actual: &mut Vec<Observation>,
    ) -> Result<(), Outcome> {
        let current = frame.current;
        if retiring_return {
            if !finished(current) {
                expected.extend(ceres_doors::expected(&current.wram)?);
                actual.extend(ceres_doors::actual(&self.state.station));
            }
            expected.push(ridley::placement(&current.wram)?);
            actual.push(ridley::placed(self.state.station.ridley.is_some()));
        }
        if frame.place.0 == super::shaft_entry::ROW && (222..=278).contains(&frame.place.1) {
            expected.extend(super::shaft_entry::expected(current)?);
            actual.extend(super::shaft_entry::fields(&self.state.passage.room.entry));
            platform::compare_shaft_background(
                self.state.room_tiles,
                &self.state.room_video.video,
                current,
                frame.place.1,
            )?;

            if !finished(current) {
                let observed = colours::observations(&colours::palette(&current.wram)?);
                let played = colours::observations(&self.state.palette);
                if shaft_native::partial_palette(frame, self.rules.doors.fade_steps) {
                    // The T1 cut crosses an in-progress interpolation of the screen row.
                    // The unchanged target palette and all actor state can already be compared.
                    let [_, observed_goal] = observed;
                    let [_, played_goal] = played;
                    expected.push(observed_goal);
                    actual.push(played_goal);
                } else {
                    expected.extend(observed);
                    actual.extend(played);
                }
                expected.extend(ceres_doors::expected(&current.wram)?);
                actual.extend(ceres_doors::actual(&self.state.station));
                expected.push(steam::state(steam::decode_shaft(&current.wram)?.iter()));
                actual.push(steam::station(&self.state.station));
            }
        }
        Ok(())
    }

    fn warning_fields(
        &mut self,
        frame: &Frame<'_>,
        outputs: &played::Outputs,
        expected: &mut Vec<Observation>,
        actual: &mut Vec<Observation>,
    ) -> Result<(), Outcome> {
        let warning_turn = matches!(
            frame.doors_before,
            Doors::Play
                | Doors::Passing(world_doors::Passage {
                    phase: world_doors::Phase::Quieting { .. },
                    ..
                })
        );
        if self.state.station.warning.is_some() && finished(frame.current) && warning_turn {
            let (warning_expected, warning_actual) =
                self.warning_picture.checked(frame, &self.state, outputs)?;
            expected.extend(warning_expected);
            actual.extend(warning_actual);
        }
        if self.state.station.warning.is_some() {
            let (clock_expected, clock_actual) = self.warning_picture.clock(frame, &self.state)?;
            expected.extend(clock_expected);
            actual.extend(clock_actual);
        }
        Ok(())
    }
}

fn comparison_failure(cut: u64, fault: CompareFault) -> Outcome {
    match fault {
        CompareFault::Difference(field) => Outcome::Difference { cut, field },
        CompareFault::EmptyExpected => data_failure(Some(cut), "played comparison schema"),
        CompareFault::DuplicateField(field) => {
            data_failure(Some(cut), &format!("duplicate played field {field}"))
        }
        CompareFault::MissingActual(field) => {
            data_failure(Some(cut), &format!("missing played field {field}"))
        }
    }
}

#[cfg(test)]
pub(super) mod tests {
    pub(super) fn diagnostic(args: std::fmt::Arguments<'_>) {
        use std::io::Write as _;
        writeln!(std::io::stdout().lock(), "{args}").unwrap();
    }

    pub(in crate::replay) fn sources() -> super::Sources {
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

    /// The alternate controls carry both airborne hits from the room seed, through the fall's
    /// Down and Up releases.
    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn alternate_room_from_seed_plays_through_both_airborne_tail_hits() {
        let report = super::play(&sources(), "samus-ceres-push", 0, 993).unwrap();
        assert_eq!(report.stop, super::Stop::End);
        assert_eq!(report.held, 993);
    }

    /// A checkpoint record admits the whole tail, including a chain partway through a whip.
    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn airborne_tail_continues_from_observed_checkpoints() {
        let sources = sources();
        for row in ["chain-any-0006-E0B5", "samus-ceres-push"] {
            for start in [930, 940, 992] {
                let report = super::play(&sources, row, start, 993).unwrap();
                assert_eq!(report.held, 993_u64.saturating_sub(start));
                assert_eq!(report.stop, super::Stop::End);
            }
        }
    }

    /// Native retreat and flyby checkpoints carry through the room return.
    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn retreat_continues_from_observed_checkpoints() {
        use super::Stop;
        let sources = sources();
        for row in ["chain-any-0006-E0B5", "samus-ceres-push"] {
            let continuous = super::play(&sources, row, 0, 1271).unwrap();
            assert_eq!(continuous.held, 1271);
            assert_eq!(continuous.stop, Stop::End);
            for start in [926, 993, 1004, 1093, 1115, 1156, 1158, 1170, 1261] {
                let report = super::play(&sources, row, start, 1271).unwrap();
                diagnostic(format_args!(
                    "RETREAT row={row} origin={start} held={} stop={:?}",
                    report.held, report.stop
                ));
                assert_eq!(report.held, 1271_u64.saturating_sub(start));
                assert_eq!(report.stop, Stop::End);
            }
            restore_stopped_station(&sources, row);
        }
    }

    /// Carry the separate room owners to the stop, then restore the entire station from its
    /// record. Appearance's native kept movement type can name multiple actions at this cut.
    fn restore_stopped_station(sources: &super::Sources, row: &str) {
        use super::{Run, Stop, appearance, ceres_doors, connected, movement, station};

        let lock = super::read_lock(sources).unwrap();
        let mut rows = super::open_rows(sources, &lock, row).unwrap();
        let coast = rows.record(1156).unwrap();
        let stopped = rows.record(1157).unwrap();
        let content = connected::Content {
            movement: movement::Content::load(&sources.rom).unwrap(),
            appearance: appearance::Content::load(&sources.rom).unwrap(),
        };
        let mut run = Run::admit(&coast, &content, &sources.rom, row).unwrap();
        let controller = super::seed_controller(&coast).unwrap();
        assert_eq!(
            run.through(&mut rows, coast, controller, 1157..=1157)
                .unwrap(),
            (1, Stop::End)
        );
        let carried = run.state.clone();
        let boxes = movement::enemy_roster(&stopped.wram).unwrap();
        let (doors, _) = ceres_doors::seed_with_roster(&stopped.wram, &boxes, false).unwrap();
        run.state.station = station::seed(&stopped.wram, doors, carried.station.stage).unwrap();
        assert_eq!(run.state.station, carried.station);
        let controller = super::seed_controller(&stopped).unwrap();
        assert_eq!(
            run.through(&mut rows, stopped, controller, 1158..=1261)
                .unwrap(),
            (104, Stop::End)
        );
        let flying = rows.record(1261).unwrap();
        let controller = super::seed_controller(&flying).unwrap();
        let (held, stop) = run
            .through(&mut rows, flying, controller, 1262..=1270)
            .unwrap();
        assert_eq!((held, stop), (9, Stop::End));
        diagnostic(format_args!(
            "RETREAT row={row} restored station at 1157 through its handoff"
        ));
    }

    /// The `MainBoss` report reaches production progress once and survives restoration.
    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn warning_defeat_is_carried_by_production() {
        use super::{Run, Stop, appearance, connected, movement};
        use types::{Area, BossKind};

        let sources = sources();
        let lock = super::read_lock(&sources).unwrap();
        let content = connected::Content {
            movement: movement::Content::load(&sources.rom).unwrap(),
            appearance: appearance::Content::load(&sources.rom).unwrap(),
        };
        for row in ["chain-any-0006-E0B5", "samus-ceres-push"] {
            let mut rows = super::open_rows(&sources, &lock, row).unwrap();
            let origin = rows.record(1270).unwrap();
            let mut run = Run::admit(&origin, &content, &sources.rom, row).unwrap();
            let controller = super::seed_controller(&origin).unwrap();
            assert_eq!(
                run.through(&mut rows, origin, controller, 1271..=1585)
                    .unwrap(),
                (315, Stop::End),
            );
            let ready = rows.record(1585).unwrap();
            let active = rows.record(1586).unwrap();
            let next = rows.record(1587).unwrap();
            let before = run.state.progress.clone();
            let native_before = crate::replay::progress::decode(&ready.wram).unwrap();
            let native_after = crate::replay::progress::decode(&active.wram).unwrap();
            assert!(!native_before.defeated[Area::Ceres][BossKind::MainBoss]);
            assert!(native_after.defeated[Area::Ceres][BossKind::MainBoss]);
            assert!(!before.defeated[Area::Ceres][BossKind::MainBoss]);
            let mut expected = before;
            expected.defeated[Area::Ceres][BossKind::MainBoss] = true;

            let mut restored = run.clone();
            let controller = super::seed_controller(&ready).unwrap();
            for played in [&mut run, &mut restored] {
                assert_eq!(
                    played
                        .through(&mut rows, ready.clone(), controller, 1586..=1586)
                        .unwrap(),
                    (1, Stop::End),
                );
                assert!(
                    played.state.progress == expected,
                    "{row} 1586 carried MainBoss"
                );
            }
            assert!(run.state == restored.state, "{row} 1586 restored state");
            let controller = super::seed_controller(&active).unwrap();
            for played in [&mut run, &mut restored] {
                assert_eq!(
                    played
                        .through(&mut rows, active.clone(), controller, 1587..=1587)
                        .unwrap(),
                    (1, Stop::End),
                );
                assert!(
                    played.state.progress == expected,
                    "{row} 1587 held progress"
                );
            }
            assert!(run.state == restored.state, "{row} 1587 restored state");
            assert!(
                crate::replay::progress::decode(&next.wram)
                    .unwrap()
                    .defeated
                    == native_after.defeated,
                "{row} 1587 native defeat held",
            );
        }
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
