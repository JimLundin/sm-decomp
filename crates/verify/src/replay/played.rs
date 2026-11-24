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
use super::{
    Sources, appearance, camera, ceres, connected, data_failure, health_tail, hud, movement,
};
use crate::cache::Cache;
use crate::lock::{Lock, Row, RowId, Start};
use crate::source;
use canonical::model::Model;
use data::{assets::world, rom::Rom, room::RoomKey};
use game::production::connected::inputs::Inputs;
use game::production::{played, room_loading};
use oracle_record::{Cut, FrameRecord, NmiKind};
use std::path::Path;
use types::Parity;
use world_doors::{ColourGroup, Doors, Fetching, Palette, Passage, Phase};

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
    "passage sounds sent and music ready",
    "progress, which chooses the next room's version, her suit and her beams",
    "Ceres station stage",
];

/// Inputs read from the record at each cut, because their owner is outside the rungs
/// (decision 35). The list shrinks as the platform's costs are modelled.
pub const BORROWED: &[&str] = &[
    "lag: whether each cut's frame of work finished",
    "lag: the contents' fetch done on its first frame, within the slide",
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
}

/// The controller as the game sees it: the pad read by the last polling NMI and the
/// buttons that went down at that read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Controller {
    held: u16,
    pressed: u16,
}

impl Controller {
    /// The controller as the seed leaves it: the game's own last read and the buttons that
    /// went down at it. A button held through the seed went down before it, not on cut 1.
    fn seeded(wram: &[u8]) -> Result<Self, Outcome> {
        Ok(Self {
            held: word(wram, source::JOYPAD1_LASTKEYS)?,
            pressed: word(wram, source::JOYPAD1_NEWKEYS)?,
        })
    }

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

/// The borrowed lag of the contents' fetch: done on its first frame. The fetch's work waits on
/// its own uploads, so every NMI through it is a lag NMI and none says when its colours were
/// set; the record shows them set within the slide, and the visor lit as the slide ends, which
/// only a fetch done before the slide's end gives. Its first frame is the earliest such.
const fn fetching_contents(doors: Doors) -> bool {
    matches!(
        doors,
        Doors::Passing(Passage {
            phase: Phase::Arriving {
                fetching: Fetching::Contents,
                ..
            },
            ..
        })
    )
}

fn word(wram: &[u8], at: source::WramWord) -> Result<u16, Outcome> {
    at.read(wram).map_err(|_| data_failure(None, at.name))
}

/// Seed-held room facts and the seed's frame counter.
struct Room {
    inputs: Inputs,
    timer_running: bool,
    /// Audio is outside the loop: the seed's queue holds no sound, and none is taken to wait.
    sounds_sent: bool,
    progress: run_progress::Progress,
    ceres: boss_ceres_ridley::Stage,
    counter: u16,
    /// The HUD's neighbours outside the loop.
    hud: HudHeld,
    /// Where her room sits on her map, and the area it is in.
    map: (interface_hud::Room, data::room::Area),
}

/// The HUD's inputs that no owner in the loop produces.
struct HudHeld {
    bindings: hud::Bindings,
    stock_at_start: interface_hud::StockAtStart,
    status: interface_hud::Status,
    map_collected: bool,
}

/// The facts held from the seed, and what play reads of the seed's room: its geometry and
/// camera room as the seed holds them, and its doors by the seed's door cells.
fn room(
    seed: &[u8],
    content: &connected::Content,
    world: &world::World,
) -> Result<(Room, played::Room), Outcome> {
    if let Some(refused) = locked(seed)? {
        return Err(refused);
    }
    let borrowed = movement::borrowed(seed, seed, &content.movement)?;
    if borrowed.cinematic || borrowed.elevator || hud::time_stopped(seed)? {
        return Err(Outcome::Unsupported {
            cut: None,
            dependency: "played seed outside ordinary play".into(),
        });
    }
    let inputs = connected::seed_inputs(seed, borrowed.items, borrowed.world)?;
    let (doorways, map) = doorways(seed, world)?;
    let played = played::Room {
        geometry: inputs.geometry.clone(),
        doorways,
        camera: inputs.camera.clone(),
    };
    let held = Room {
        sounds_sent: inputs.sounds_waiting == samus_appearance::SoundsWaiting::NONE,
        inputs,
        map,
        hud: HudHeld {
            bindings: hud::bindings(seed)?,
            stock_at_start: hud::stock_at_start(seed)?,
            status: hud::status(seed)?,
            map_collected: hud::map_collected(seed)?,
        },
        timer_running: word(seed, source::ESCAPE_TIMER_RUNNING)? != 0,
        progress: super::progress::seed(seed)?,
        ceres: match word(seed, source::CERES_STATUS)? {
            0 => boss_ceres_ridley::Stage::Calm,
            1 => boss_ceres_ridley::Stage::Getaway,
            _ => {
                return Err(Outcome::Unsupported {
                    cut: None,
                    dependency: "played seed in the Ceres escape".into(),
                });
            }
        },
        counter: word(seed, source::NMI_FRAME_COUNTER_WORD)?,
    };
    Ok((held, played))
}

/// Samus's new-state handler, a bank $90 routine pointer: `$E8DC` while the game holds her
/// locked, as the station's elevator pad lowers her into room 1 on arrival.
const SAMUS_NEW_STATE_HANDLER: source::WramWord = source::WramWord {
    name: "SAMUS_NEW_STATE_HANDLER",
    offset: 0x0A44,
};
const LOCKED_HANDLER: u16 = 0xe8dc;

/// While she is locked, her moves, her clip and her pose input don't run; the lock and the
/// pad that carries her belong to the pad and arrival owners, which aren't in the loop.
fn locked(wram: &[u8]) -> Result<Option<Outcome>, Outcome> {
    Ok(
        (word(wram, SAMUS_NEW_STATE_HANDLER)? == LOCKED_HANDLER).then(|| Outcome::Unsupported {
            cut: None,
            dependency: "her frame locked at the arrival (the pad and arrival owners, B8/F4)"
                .into(),
        }),
    )
}

type Doorways = (
    Vec<played::Doorway>,
    (interface_hud::Room, data::room::Area),
);

/// The seed room's doors to other rooms, by the seed's door cells, and where the room sits on
/// her map, from the imported world.
fn doorways(seed: &[u8], world: &world::World) -> Result<Doorways, Outcome> {
    let bad = || data_failure(None, "played room doors");
    let id = import::world::id(RoomKey::from_room_pointer(word(seed, source::ROOM_PTR)?))
        .ok_or_else(bad)?;
    let room = world.room(id).ok_or_else(bad)?;
    let doorways =
        room_loading::doorways(world, room, movement::door_cells(seed)?).map_err(|_| bad())?;
    Ok((doorways, (room_loading::map_room(room), room.place.area)))
}

/// The rooms the engine loads, their tilesets, the sprites' colours and the slopes, imported
/// from the player's ROM.
fn rooms(image: &Rom) -> Result<room_loading::Rooms, Outcome> {
    let bad = || data_failure(None, "played rooms import");
    let world = import::world::import(image.lorom(), &import::world::LOADED).map_err(|_| bad())?;
    let tilesets = import::tileset::import(image.lorom(), &world).map_err(|_| bad())?;
    Ok(room_loading::Rooms {
        world,
        tilesets,
        actors: import::actors::import(image.lorom()).map_err(|_| bad())?,
        samus: import::samus::palettes::import(image.lorom()).map_err(|_| bad())?,
        slopes: data::level::slope::Slopes::load(image.lorom()).map_err(|_| bad())?,
    })
}

fn expected(
    record: &FrameRecord,
    content: &appearance::Content,
    hud: &hud::Content,
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
    let (state, playing) = appearance::animation_owner(wram, content)?;
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
    if finished(record) {
        fields.extend(door_observations(
            ceres::doors(wram)?
                .into_iter()
                .map(|slot| ceres::expected(wram, slot))
                .collect::<Result<_, _>>()?,
        ));
    }
    fields.extend(health_parts(&health_tail::decode(wram, None)?));
    fields.extend(hud_parts(hud::expected(wram, hud)?));
    fields.push(frame_count_observation(hud::frame_count(wram)?));
    Ok(fields)
}

/// Health's State by part, so a difference names the part that differs.
fn health_parts(health: &samus_health::State) -> [Observation; 6] {
    [
        ("health.energy.whole", health.energy.whole.get()),
        ("health.energy.fraction", health.energy.fraction),
        ("health.pending.whole", health.pending_loss.whole.get()),
        ("health.pending.fraction", health.pending_loss.fraction),
        (
            "health.warning",
            u16::from(health.warning == samus_health::Warning::On),
        ),
        ("health.protection", u16::from(health.protection.get())),
    ]
    .map(|(field, value)| Observation::quantity(field, i64::from(value)))
}

/// The room's Ceres doors by part, each named by its place in the room's order of its enemies,
/// after how many there are, so a difference names the door and the part that differ.
fn door_observations(doors: Vec<Vec<Observation>>) -> Vec<Observation> {
    let count = i64::try_from(doors.len()).unwrap_or(i64::MAX);
    let mut fields = vec![Observation::quantity("ceres.doors", count)];
    for (place, parts) in doors.into_iter().enumerate() {
        fields.extend(parts.into_iter().map(|part| Observation {
            field: part.field.replacen("ceres.", &format!("ceres.{place}."), 1),
            bytes: part.bytes,
        }));
    }
    fields
}

/// The seed room's Ceres doors, decoded once from the seed record. Her motion meets each one
/// as the loop carries it, so none may stand among the actor boxes held from the seed too.
///
/// Her motion meets the boxes in the original's slot order, and the first that blocks her
/// decides the move. Production puts the doors' boxes after the held ones, since the held
/// geometry keeps no slots, so a held box in a slot after a door's would be met out of order.
fn seed_doors(seed: &[u8]) -> Result<Vec<boss_ceres_ridley::CeresDoor>, Outcome> {
    let slots = ceres::doors(seed)?;
    let roster = movement::roster_slots(seed)?;
    let refuse = |dependency: &str| Outcome::Unsupported {
        cut: None,
        dependency: dependency.into(),
    };
    if slots.iter().any(|slot| roster.contains(&slot.index())) {
        return Err(refuse(
            "played seed holds a Ceres door among the held actor boxes",
        ));
    }
    let first_door = slots.iter().map(|slot| slot.index()).min();
    if first_door.is_some_and(|door| roster.iter().any(|&held| held > door)) {
        return Err(refuse(
            "played seed holds an actor box in a slot after a Ceres door's",
        ));
    }
    slots
        .into_iter()
        .map(|slot| ceres::seed(seed, slot))
        .collect()
}

/// The mini-map's window: rows 0 to 2, columns 26 to 30 of the HUD's 32 columns.
const MAP_WINDOW: core::ops::RangeInclusive<usize> = 26..=30;
const HUD_COLUMNS: usize = 32;

/// The HUD's observations with its cells split into the mini-map's window and the status bar,
/// so a difference names the one that differs.
fn hud_parts(fields: Vec<Observation>) -> Vec<Observation> {
    let mut parts = Vec::with_capacity(fields.len().saturating_add(1));
    for field in fields {
        if field.field != "hud.grid" {
            parts.push(field);
            continue;
        }
        let (window, bar): (Vec<_>, Vec<_>) = field
            .bytes
            .as_chunks::<2>()
            .0
            .iter()
            .enumerate()
            .partition(|(cell, _)| MAP_WINDOW.contains(&(cell % HUD_COLUMNS)));
        let bytes =
            |cells: Vec<(usize, &[u8; 2])>| cells.into_iter().flat_map(|(_, cell)| *cell).collect();
        parts.push(Observation {
            field: "hud.map-window".into(),
            bytes: bytes(window),
        });
        parts.push(Observation {
            field: "hud.status-bar".into(),
            bytes: bytes(bar),
        });
    }
    parts
}

/// The game's states for play and the three stretches of a door passage.
const PLAY: u16 = 8;
const TOUCHED: u16 = 9;
const LOADING: u16 = 10;
const ARRIVING: u16 = 11;
/// The screen's colours as they stand, and the colours a fade heads for.
const SCREEN_COLOURS: usize = 0xc000;
const GOAL_COLOURS: usize = 0xc200;

/// The game's count of finished frames, which the HUD reads as the next frame begins.
fn frame_count_observation(frame: interface_hud::FrameCount) -> Observation {
    Observation::quantity("frame-count", i64::from(frame.0))
}

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

/// The lock rows a run plays through: its own, then each row that continues it, reached as the
/// run reaches it. A row continues another when both start from one capture's seeds, its own
/// the next, so its cut 0 is the video frame after the other's last cut; the run's cuts count
/// on through it.
struct Chain<'a> {
    lock: &'a Lock,
    /// Each row reached so far, with the run's cut its own cut 0 is.
    reached: Vec<(u64, &'a Row)>,
}

impl<'a> Chain<'a> {
    fn new(lock: &'a Lock, row: &str) -> Result<Self, Outcome> {
        let row = lock
            .row(&RowId(row.to_owned()))
            .ok_or_else(|| data_failure(None, "played lock row"))?;
        Ok(Self {
            lock,
            reached: vec![(0, row)],
        })
    }

    /// Which reached row holds the run's `cut`, by its place in the chain, and its own cut
    /// there, reaching the rows that continue the run up to it.
    fn place(&mut self, cut: u64) -> Result<(usize, u64), Outcome> {
        loop {
            let place = self.reached.len().saturating_sub(1);
            let &(first, row) = self
                .reached
                .last()
                .ok_or_else(|| data_failure(Some(cut), "played rows"))?;
            let next = first.saturating_add(row.frames.0);
            if cut < next {
                let own = cut
                    .checked_sub(first)
                    .ok_or_else(|| data_failure(Some(cut), "played rows"))?;
                return Ok((place, own));
            }
            let following = self
                .following(row)
                .ok_or_else(|| data_failure(Some(cut), "played row continuing the last row"))?;
            self.reached.push((next, following));
        }
    }

    /// The row whose seed is the next of `row`'s capture.
    fn following(&self, row: &Row) -> Option<&'a Row> {
        let Start::Seed { capture, index } = &row.start else {
            return None;
        };
        let next = index.checked_add(1)?;
        self.lock.rows.iter().find(|candidate| {
            matches!(&candidate.start, Start::Seed { capture: own, index: at } if own == capture && *at == next)
        })
    }
}

/// The records of a run's chain of rows, each row's cache opened as the run reaches it.
struct Rows<'a> {
    chain: Chain<'a>,
    cache: &'a Path,
    /// The caches of the rows reached, in the chain's order.
    opened: Vec<Cache>,
}

impl<'a> Rows<'a> {
    fn open(lock: &'a Lock, cache: &'a Path, row: &str) -> Result<Self, Outcome> {
        Ok(Self {
            chain: Chain::new(lock, row)?,
            cache,
            opened: Vec::new(),
        })
    }

    /// The record at the run's `cut`.
    fn record(&mut self, cut: u64) -> Result<FrameRecord, Outcome> {
        let (place, own) = self.chain.place(cut)?;
        while self.opened.len() <= place {
            let (_, row) = self
                .chain
                .reached
                .get(self.opened.len())
                .ok_or_else(|| data_failure(Some(cut), "played rows"))?;
            self.opened.push(
                Cache::open(self.cache, row)
                    .map_err(|_| data_failure(None, "record cache pins"))?,
            );
        }
        let cache = self
            .opened
            .get(place)
            .ok_or_else(|| data_failure(Some(cut), "played rows"))?;
        record(cache, own)
    }
}

/// Play `row` from its cut `start` through `end` inclusive with the movie's pads alone. Cuts
/// past the row's last go on through the rows that continue it, without a new seed.
///
/// # Errors
/// Missing or mismatched lock, cache or content refuse the run before any cut is played, and a
/// cut past every row that continues the run, or a row that doesn't follow on, stops it.
pub fn play(sources: &Sources, row: &str, start: u64, end: u64) -> Result<Report, Outcome> {
    let text = std::fs::read_to_string(&sources.lock)
        .map_err(|_| data_failure(None, "record lock file"))?;
    let lock = Lock::parse(&text).map_err(|_| data_failure(None, "record lock schema"))?;
    let mut rows = Rows::open(&lock, &sources.cache, row)?;
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
    room: Room,
    content: &'a connected::Content,
    hud: hud::Content,
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
        let rooms = rooms(&image)?;
        let (room, seeded) = room(&seed.wram, content, &rooms.world)?;
        let (appearance, figure) = appearance::owner(&seed.wram, &content.appearance)?;
        let hud = hud::Content::load(path)?;
        let (map_room, area) = room.map;
        let ceres_doors = seed_doors(&seed.wram)?;
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
                health: Model(health_tail::decode(&seed.wram, None)?),
                hud: Model(hud::decode_state(&seed.wram)?),
                map_room,
                room: seeded,
                ceres_doors,
                frame: hud::frame_count(&seed.wram)?,
            },
            played: played::Content {
                movement: content.movement.movement.clone(),
                appearance: content.appearance.appearance.clone(),
                colour_groups: groups(),
                rooms,
                hud: hud.pictures,
                map: hud.map_of(area)?.clone(),
            },
            rules: played::Rules {
                movement: samus_movement::Rules::default(),
                appearance: samus_appearance::Rules::VANILLA,
                camera: game::production::CameraRules::new(world_camera::Rules::REFERENCE),
                doors: world_doors::Rules::default(),
                health: samus_health::Rules::REFERENCE,
                hud: interface_hud::Rules::default(),
                ceres: boss_ceres_ridley::Rules::default(),
            },
            room,
            content,
            hud,
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
            work_done: finished(current) || fetching_contents(self.state.doors),
            audio: played::PassageAudio {
                sounds_sent: self.room.sounds_sent,
                music_ready: true,
            },
            timer_running: self.room.timer_running,
            progress: &self.room.progress,
            ceres: self.room.ceres,
        };
        let held = &self.room.hud;
        let status = held.status;
        let hud_frame = played::HudFrame {
            buttons: held.bindings.buttons(controller.pressed, controller.held),
            stock_at_start: held.stock_at_start,
            capacity: status.energy.capacity,
            reserve: status.reserve,
            ammunition: status.ammunition,
            map_collected: held.map_collected,
            // The HUD's frame of play is a frame of main gameplay.
            countdown: hud::MAIN_GAMEPLAY_COUNTDOWN,
        };
        self.state
            .step(&inputs, &passing, &hud_frame, &self.played, &self.rules)
            .map_err(|fault| Outcome::Unsupported {
                cut: Some(cut),
                dependency: format!("played frame {fault:?}"),
            })?;
        if polled(current) {
            self.room.counter = self.room.counter.wrapping_add(1);
        }
        let expected = expected(current, &self.content.appearance, &self.hud, self.row, cut)?;
        let actual = actual(
            &self.state,
            &self.content.appearance,
            &self.hud,
            finished(current),
        )?;
        compare(&expected, &actual).map_err(|fault| match fault {
            CompareFault::Difference(field) => Outcome::Difference { cut, field },
            CompareFault::EmptyExpected
            | CompareFault::DuplicateField(_)
            | CompareFault::MissingActual(_) => data_failure(Some(cut), "played comparison schema"),
        })
    }
}

/// The doors' colour groups, by colour index: which colours the passage lights, which belong
/// to the HUD's icons, the escape timer and the visor, and the rest.
fn groups() -> [ColourGroup; world_doors::COLOURS] {
    std::array::from_fn(|index| match index {
        9 | 10 | 13 | 14 | 17..=19 | 29 => ColourGroup::Lit,
        20..=23 | 28 => ColourGroup::Icon,
        209 | 210 | 212 | 221 => ColourGroup::Timer,
        196 => ColourGroup::Visor,
        _ => ColourGroup::Other,
    })
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
    hud: &hud::Content,
    finished: bool,
) -> Result<Vec<Observation>, Outcome> {
    let samus = &state.movement.0;
    let mut fields = parts(samus);
    fields.extend([
        Observation::quantity("movement.position.x", i64::from(samus.position.x.0)),
        Observation::quantity("movement.position.y", i64::from(samus.position.y.0)),
    ]);
    fields.extend(appearance::actual(&state.appearance.0, content));
    fields.push(connected::playing_observation(state.figure.0));
    fields.extend(camera::initial(&state.camera.0));
    fields.push(passing_observation(matches!(
        state.doors,
        Doors::Passing(_)
    )));
    if finished {
        fields.extend(colour_observations(&state.palette));
    }
    if finished {
        fields.extend(door_observations(
            state.ceres_doors.iter().map(ceres::initial).collect(),
        ));
    }
    fields.extend(health_parts(&state.health.0));
    fields.extend(hud_parts(hud::initial(&state.hud.0, hud)?));
    fields.push(frame_count_observation(state.frame));
    Ok(fields)
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
    fn a_button_held_through_the_seed_is_not_pressed_on_the_first_cut() {
        let mut wram = vec![0; 0x100];
        wram[0x8b..0x8d].copy_from_slice(&0x0180_u16.to_le_bytes());
        wram[0x8f..0x91].copy_from_slice(&0x0100_u16.to_le_bytes());
        let seeded = Controller::seeded(&wram).unwrap();
        assert_eq!((seeded.held, seeded.pressed), (0x0180, 0x0100));
        assert_eq!(seeded.read(0x0180, true).pressed, 0);
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

    #[test]
    fn a_run_reaches_the_rows_that_continue_it_and_counts_its_cuts_on_through_them() {
        use crate::lock::{Lock, RowId};
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../scenarios/locks/lsnes.toml"
        );
        let lock = Lock::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
        let frames = |id: &str| lock.row(&RowId(id.into())).unwrap().frames.0;
        let first = frames("chain-any-0003-DFD7");
        let second = frames("chain-any-0004-E021");
        let mut chain = super::Chain::new(&lock, "chain-any-0003-DFD7").unwrap();
        assert_eq!(chain.place(0).unwrap(), (0, 0));
        assert_eq!(chain.place(first - 1).unwrap(), (0, first - 1));
        assert_eq!(
            chain.reached.len(),
            1,
            "a row is reached only when the run reaches it"
        );
        assert_eq!(chain.place(first).unwrap(), (1, 0));
        assert_eq!(chain.place(first + second + 2).unwrap(), (2, 2));
        let ids: Vec<_> = chain
            .reached
            .iter()
            .map(|(at, row)| (*at, row.id.0.as_str()))
            .collect();
        assert_eq!(
            ids,
            [
                (0, "chain-any-0003-DFD7"),
                (first, "chain-any-0004-E021"),
                (first + second, "chain-any-0005-E06B"),
            ]
        );
        assert!(
            chain.place(u64::MAX - 1).is_err(),
            "no row continues the capture's last"
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

    /// The Ceres rows played back to back from cut 0, each on through the rooms she enters.
    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn ceres_rows_played_back_to_back() {
        let sources = sources();
        for row in [
            "chain-any-0002-DF8D",
            "chain-any-0003-DFD7",
            "chain-any-0004-E021",
        ] {
            let report = super::play(&sources, row, 0, 3000).unwrap();
            diagnostic(format_args!(
                "PLAYED_ALONE row={} start={} held={} stop={:?}",
                report.row, report.start, report.held, report.stop
            ));
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
