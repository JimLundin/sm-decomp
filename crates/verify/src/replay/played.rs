//! Play a room alone from its seed: the movie's pads in, whole frames out, compared at each cut.
//!
//! This is the rung measure (MODEL decision 18): production State is admitted once from the
//! seed record and then carried with nothing fed back. Each cut's only new input is the pad
//! the movie presented at the preceding vblank; the oracle's later records are read for
//! comparison alone. Room facts that no selected owner produces yet are taken once from the
//! seed and held for the whole run; [`SEED_HELD`] names them. A run stops at the first
//! differing field, the first unsupported family or the first lag cut, and the number of cuts
//! it matched before that is the frames it played alone.
//!
//! The comparison is at frame cuts (decision 1): the selected owners' State after each
//! requested NMI return. Intra-frame bus timing is not compared here.

use super::compare::{CompareFault, Observation, compare};
use super::index::ObservationId;
use super::readers::record;
use super::result::Outcome;
use super::{Sources, appearance, camera, connected, data_failure, movement};
use crate::cache::Cache;
use crate::lock::{Lock, RowId};
use crate::source;
use canonical::model::Model;
use game::production::connected::inputs::Inputs;
use game::production::played;
use oracle_record::{Cut, FrameRecord};
use types::Parity;

/// Room facts taken from the seed and held, because no selected owner produces them yet.
/// Each one is a missing owner; the list shrinks as owners join the played loop.
pub const SEED_HELD: &[&str] = &[
    "equipped items",
    "energy",
    "liquid (none)",
    "sound queue occupancy",
    "Contact guards",
    "cinematic and elevator status",
    "room geometry and actor boxes",
    "camera room settings and lead",
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

/// The frame counter's parity during the interval after `elapsed` NMIs past the seed cut.
const fn parity(seed_counter: u16, elapsed: u64) -> Parity {
    if (seed_counter & 1 == 0) == (elapsed & 1 == 0) {
        Parity::Even
    } else {
        Parity::Odd
    }
}

const fn polled(record: &FrameRecord) -> bool {
    matches!(record.cut, Cut::NmiReturn(nmi) if nmi.polled)
}

fn word(wram: &[u8], at: source::WramWord) -> Result<u16, Outcome> {
    at.read(wram).map_err(|_| data_failure(None, at.name))
}

/// Seed-held room facts and the seed's frame counter.
struct Room {
    inputs: Inputs,
    counter: u16,
}

fn room(seed: &[u8], content: &connected::Content) -> Result<Room, Outcome> {
    let borrowed = movement::borrowed(seed, seed, &content.movement)?;
    if borrowed.cinematic || borrowed.elevator {
        return Err(Outcome::Unsupported {
            cut: None,
            dependency: "played seed outside ordinary play".into(),
        });
    }
    Ok(Room {
        inputs: connected::seed_inputs(seed, borrowed.items, borrowed.world)?,
        counter: word(seed, source::NMI_FRAME_COUNTER_WORD)?,
    })
}

fn expected(
    wram: &[u8],
    content: &appearance::Content,
    row: &str,
    cut: u64,
) -> Result<Vec<Observation>, Outcome> {
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
    let mut run = match Run::admit(&seed, &content, row, start) {
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
    start: u64,
}

impl<'a> Run<'a> {
    fn admit(
        seed: &FrameRecord,
        content: &'a connected::Content,
        row: &'a str,
        start: u64,
    ) -> Result<Self, Outcome> {
        let room = room(&seed.wram, content)?;
        let (appearance, figure) = appearance::owner(&seed.wram, &content.appearance)?;
        Ok(Self {
            state: played::State {
                movement: Model(movement::seed(&seed.wram)?),
                appearance: Model(appearance),
                figure: Model(figure),
                camera: Model(camera::seed(&seed.wram, &camera::Content)?),
            },
            played: played::Content {
                movement: content.movement.movement.clone(),
                appearance: content.appearance.appearance.clone(),
            },
            rules: played::Rules {
                movement: samus_movement::Rules::default(),
                appearance: samus_appearance::Rules::VANILLA,
                camera: game::production::CameraRules::new(world_camera::Rules::REFERENCE),
            },
            room,
            content,
            row,
            start,
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
        if !polled(current) {
            return Err(Outcome::Unsupported {
                cut: None,
                dependency: "lag cut (T1 or a borrowed lag record)".into(),
            });
        }
        let mut inputs = self.room.inputs.clone();
        inputs.held = movement::pad(controller.held);
        inputs.pressed = movement::pad(controller.pressed);
        inputs.parity = parity(
            self.room.counter,
            cut.saturating_sub(self.start).saturating_sub(1),
        );
        self.state
            .step(&inputs, &self.played, &self.rules)
            .map_err(|fault| Outcome::Unsupported {
                cut: Some(cut),
                dependency: format!("played frame {fault:?}"),
            })?;
        let expected = expected(&current.wram, &self.content.appearance, self.row, cut)?;
        compare(&expected, &actual(&self.state, &self.content.appearance)).map_err(|fault| {
            match fault {
                CompareFault::Difference(field) => Outcome::Difference { cut, field },
                CompareFault::EmptyExpected
                | CompareFault::DuplicateField(_)
                | CompareFault::MissingActual(_) => {
                    data_failure(Some(cut), "played comparison schema")
                }
            }
        })
    }
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
        part("movement.run", format!("{:?}", samus.run)),
        part("movement.vertical", format!("{:?}", samus.vertical)),
        part("movement.seat", format!("{:?}", samus.seat)),
        part("movement.jump-held", format!("{:?}", samus.jump_held)),
        part("movement.position", format!("{:?}", samus.position)),
    ]
}

fn actual(state: &played::State, content: &appearance::Content) -> Vec<Observation> {
    let samus = &state.movement.0;
    let mut fields = parts(samus);
    fields.extend([
        Observation::quantity("movement.position.x", i64::from(samus.position.x.0)),
        Observation::quantity("movement.position.y", i64::from(samus.position.y.0)),
    ]);
    fields.extend(appearance::initial(state.appearance.0, content));
    fields.push(connected::playing_observation(state.figure.0));
    fields.extend(camera::initial(&state.camera.0));
    fields
}

#[cfg(test)]
mod tests {
    use super::{Controller, Parity, parity};

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
    fn parity_follows_the_seed_counter() {
        assert_eq!(parity(4, 0), Parity::Even);
        assert_eq!(parity(4, 1), Parity::Odd);
        assert_eq!(parity(7, 0), Parity::Odd);
        assert_eq!(parity(7, 3), Parity::Even);
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
                "PLAYED_ALONE row={} start={} held={} stop={:?} seed_held={:?}",
                report.row,
                report.start,
                report.held,
                report.stop,
                super::SEED_HELD
            ));
        }
    }
}
