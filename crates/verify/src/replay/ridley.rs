//! Independent observations of Ceres Ridley and the baby he holds, from her entering his room
//! to his take-off: decoded from his enemy slot, his words, his tail's and the baby's, compared
//! by part. His tail's hit on her is compared where it lands, on her energy, her protection and
//! her push.

use super::{compare::Observation, data_failure, result::Outcome};
use crate::source::{self, SlotField, WramWord};
use boss_ceres_ridley::{
    Attack, Baby, BabyFigure, BabyRoutine, CeresRidley, CoastRuns, FidgetFigure, FigureLength as _,
    FlightPhase, FlightTail, GlowFigure, HoverRuns, LagCount, ReadyWait, RidleyLook, RidleyPhase,
    RidleyWait, RoarFigure, RoarWait, ShownFigure, SwingLag, SwingPace, SwingWay, SwishRuns,
    TailAngle, TailCurl, TailLength, TailMark, TailSegment, TakeOffFigure, TimedRoarFigure,
    TimedTakeOffFigure,
};
use core::num::{NonZeroU16, Wrapping};
use enum_map::Enum as _;
use types::{Pair, Px};

/// Ceres Ridley's species word.
const SPECIES: u16 = 0xE13F;
/// The enemy slots.
const SLOTS: u16 = 32;

/// The baby's list pointer, the runs its figure has shown, and its position.
const BABY_LIST: WramWord = WramWord {
    name: "CERES_RIDLEY_BABY_LIST",
    offset: 0x7806,
};
const BABY_RUNS: WramWord = WramWord {
    name: "CERES_RIDLEY_BABY_RUNS",
    offset: 0x7808,
};
const BABY_X: WramWord = WramWord {
    name: "CERES_RIDLEY_BABY_X",
    offset: 0x8804,
};
const BABY_Y: WramWord = WramWord {
    name: "CERES_RIDLEY_BABY_Y",
    offset: 0x8808,
};
/// Whether his tail and wings move: 1 once they do.
const MOVING: WramWord = WramWord {
    name: "CERES_RIDLEY_MOVING",
    offset: 0x7804,
};
/// His roar's word: not 0 while he roars.
const ROARING: WramWord = WramWord {
    name: "CERES_RIDLEY_ROARING",
    offset: 0x781E,
};

/// His tail's curl, the ideal angle between its segments.
const TAIL_CURL: WramWord = WramWord {
    name: "CERES_RIDLEY_TAIL_CURL",
    offset: 0x201E,
};
/// His tail's segments: the first's x, each next one's 14h bytes on, its y 2 bytes after its x;
/// the seventh is the tip.
const TAIL_SEGMENT_X: u16 = 0x202C;
const TAIL_SEGMENT_STRIDE: u16 = 0x14;
const TAIL_SEGMENTS: u16 = 7;
/// Maximum segment lengths, in subpixels, in observation order.
const FLIGHT_LENGTH_CAPS: [u16; 7] = [512, 6144, 6144, 5632, 5632, 4608, 1280];

/// The properties bit that keeps her touch and her shots from meeting him.
const INTANGIBLE: u16 = 0x0400;
/// The bytes of one of the body's colour sets: 11 colours.
const BODY_SET_BYTES: u16 = 22;

/// The baby's list before its first run, its fidget's eight entries and its glow's first.
const BABY_NEW: u16 = 0xBF31;
const FIDGET: [u16; 8] = [
    0xBF35, 0xBF39, 0xBF3D, 0xBF41, 0xBF49, 0xBF4D, 0xBF51, 0xBF55,
];
const GLOW: u16 = 0xBF61;
const GLOW_STRIDE: u16 = 8;
const GLOW_FIGURES: usize = 12;

/// His roar's list entries for its first four figures, and for its fifth.
const ROAR_FIGURES: [(u16, TimedRoarFigure); 4] = [
    (0xE698, TimedRoarFigure::First),
    (0xE69E, TimedRoarFigure::Second),
    (0xE6A2, TimedRoarFigure::Third),
    (0xE6A6, TimedRoarFigure::Fourth),
];
const ROAR_FIFTH: u16 = 0xE6AC;

/// His take-off's list entries for its first four figures, as the step leaves them, and for its
/// fifth.
const TAKEOFF_FIGURES: [(u16, TimedTakeOffFigure); 4] = [
    (0xE925, TimedTakeOffFigure::First),
    (0xE92F, TimedTakeOffFigure::Second),
    (0xE939, TimedTakeOffFigure::Third),
    (0xE93F, TimedTakeOffFigure::Fourth),
];
const TAKEOFF_FIFTH: u16 = 0xE943;

/// His hover's runs without reaching his point.
const HOVER_RUNS: WramWord = WramWord {
    name: "CERES_RIDLEY_HOVER_RUNS",
    offset: 0x7800,
};

fn unsupported(dependency: &str) -> Outcome {
    Outcome::Unsupported {
        cut: None,
        dependency: dependency.to_owned(),
    }
}

fn read(wram: &[u8], at: WramWord) -> Result<u16, Outcome> {
    at.read(wram)
        .map_err(|_| data_failure(None, "Ceres Ridley observation"))
}

fn slot_word(wram: &[u8], slot: u16, field: SlotField) -> Result<u16, Outcome> {
    source::ENEMY_DATA
        .word(field, slot)
        .and_then(|field| field.read(wram))
        .map_err(|_| data_failure(None, "Ceres Ridley observation"))
}

/// The slot that holds Ceres Ridley, if any.
fn slot(wram: &[u8]) -> Result<Option<u16>, Outcome> {
    for slot in 0..SLOTS {
        if slot_word(wram, slot, source::ENEMY_SPECIES)? == SPECIES {
            return Ok(Some(slot));
        }
    }
    Ok(None)
}

/// What his phase shows of him beyond his State: whether her touch meets him, whether his tail
/// and wings move, and whether he roars.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Shows {
    tangible: bool,
    moving: bool,
    roaring: bool,
}

impl Shows {
    /// What the model's phase shows.
    pub(super) const fn of(phase: RidleyPhase) -> Self {
        Self {
            tangible: phase.tangible() || flyby_holds_body(phase),
            moving: phase.moving() || flyby_holds_body(phase),
            roaring: phase.roaring(),
        }
    }
}

/// The flyby takes over drawing without clearing the dormant slot flags.
const fn flyby_holds_body(phase: RidleyPhase) -> bool {
    matches!(
        phase,
        RidleyPhase::Getaway(
            types::ceres_getaway::GetawayState::FlyingPast(_)
                | types::ceres_getaway::GetawayState::WarningReady
        )
    )
}

/// Ceres Ridley as the record holds him, or none outside his room.
pub(super) fn seed(wram: &[u8]) -> Result<Option<CeresRidley>, Outcome> {
    Ok(observed(wram)?.map(|(ridley, _)| ridley))
}

/// Ceres Ridley as the record holds him with what his phase shows, or none outside his room.
pub(super) fn observed(wram: &[u8]) -> Result<Option<(CeresRidley, Shows)>, Outcome> {
    let Some(slot) = slot(wram)? else {
        return Ok(None);
    };
    let phase = phase(wram, slot)?;
    let look = match slot_word(wram, slot, source::ENEMY_SPRITEMAP)? {
        0x804F => None,
        0xE983 => Some(RidleyLook::Rest),
        0xE9C7 => Some(RidleyLook::Open),
        0xE9E9 => Some(RidleyLook::Wide),
        0xEA4F => Some(RidleyLook::TakeOff1),
        0xEA71 => Some(RidleyLook::TakeOff2),
        _ => return Err(unsupported("Ceres Ridley look observation")),
    };
    let [_, speed_x, speed_y, ..] = source::ENEMY_AI_VARIABLES;
    // His fractions are his subpositions' high bytes; their low bytes stay 0, and a record
    // with either low byte set holds a fraction he doesn't keep.
    let fraction = |field| match slot_word(wram, slot, field)?.to_be_bytes() {
        [high, 0] => Ok(high),
        [_, _] => Err(unsupported("Ceres Ridley fraction observation")),
    };
    let flown = !matches!(
        phase,
        RidleyPhase::Holding
            | RidleyPhase::Waiting { .. }
            | RidleyPhase::Awake
            | RidleyPhase::EyeFading { .. }
            | RidleyPhase::BodyFading { .. }
            | RidleyPhase::ReadyToRoar { .. }
            | RidleyPhase::Roaring { .. }
    );
    let takeoff = if flown {
        Some(takeoff_figure(wram, slot)?)
    } else {
        None
    };
    let seen = Shows {
        tangible: slot_word(wram, slot, source::ENEMY_PROPERTIES)? & INTANGIBLE == 0,
        moving: read(wram, MOVING)? == 1,
        roaring: read(wram, ROARING)? != 0,
    };
    let ridley = CeresRidley {
        phase,
        look,
        position: Pair {
            x: Wrapping(slot_word(wram, slot, source::ENEMY_X_POS)?),
            y: Wrapping(slot_word(wram, slot, source::ENEMY_Y_POS)?),
        },
        fraction: Pair {
            x: fraction(source::ENEMY_X_SUBPOS)?,
            y: fraction(source::ENEMY_Y_SUBPOS)?,
        },
        speed: Pair {
            x: Wrapping(slot_word(wram, slot, speed_x)?.cast_signed()),
            y: Wrapping(slot_word(wram, slot, speed_y)?.cast_signed()),
        },
        takeoff,
        hover_runs: u8::try_from(read(wram, HOVER_RUNS)?)
            .ok()
            .and_then(HoverRuns::new)
            .ok_or_else(|| unsupported("Ceres Ridley hover observation"))?,
        baby: baby(wram)?,
        flight_tail: flown.then(|| flight_tail(wram)).transpose()?,
        curl: u8::try_from(read(wram, TAIL_CURL)?)
            .ok()
            .and_then(TailCurl::new)
            .ok_or_else(|| unsupported("Ceres Ridley tail curl observation"))?,
    };
    Ok(Some((ridley, seen)))
}

/// Where his seven tail segments are, tip last, as the record holds them once he moves.
pub(super) fn tail_segments(wram: &[u8], phase: RidleyPhase) -> Result<Vec<Tail>, Outcome> {
    (0..TAIL_SEGMENTS)
        .map(|segment| {
            if !phase.tail_at_rest() && !phase.moving() && !flyby_holds_body(phase) {
                return Ok(None);
            }
            let x = TAIL_SEGMENT_X.wrapping_add(segment.wrapping_mul(TAIL_SEGMENT_STRIDE));
            let word = |offset| {
                read(
                    wram,
                    WramWord {
                        name: "CERES_RIDLEY_TAIL_SEGMENT",
                        offset,
                    },
                )
            };
            Ok(Some(Pair {
                x: Wrapping(word(x)?),
                y: Wrapping(word(x.wrapping_add(2))?),
            }))
        })
        .collect()
}

/// The observed place of a tail segment once he moves.
pub(super) type Tail = Option<Pair<Px>>;

/// The places each link carries in flight, or their common resting place.
pub(super) fn modelled_tail_segments(ridley: &CeresRidley) -> Vec<Tail> {
    if let Some(tail) = ridley.flight_tail {
        return tail.places().map(Some).into();
    }
    (0..TAIL_SEGMENTS).map(|_| ridley.tail()).collect()
}

/// Decode the carried shape independently of the engine's flight step.
fn flight_tail(wram: &[u8]) -> Result<FlightTail, Outcome> {
    let bad = || unsupported("Ceres Ridley flight tail observation");
    let word = |offset| {
        read(
            wram,
            WramWord {
                name: "CERES_RIDLEY_FLIGHT_TAIL",
                offset,
            },
        )
    };
    let pace = match word(0x2014)? {
        1 => SwingPace::Slow,
        2 => SwingPace::Quick,
        8 => SwingPace::Whip,
        _ => return Err(bad()),
    };
    let mark = word(0x201A)?;
    let mark = if mark.cast_signed() < 0 {
        None
    } else {
        Some(TailMark::new(TailAngle(mark)).ok_or_else(bad)?)
    };
    if word(0x201C)?.cast_signed() >= 0 {
        return Err(bad());
    }
    let swish = u8::try_from(word(0x781C)?)
        .ok()
        .and_then(SwishRuns::new)
        .ok_or_else(bad)?;
    let segments = (0..TAIL_SEGMENTS)
        .zip(FLIGHT_LENGTH_CAPS)
        .map(|(index, cap)| {
            let base = 0x2020_u16.wrapping_add(index.wrapping_mul(TAIL_SEGMENT_STRIDE));
            let get = |offset| word(base.wrapping_add(offset));
            let lag = match get(2)? {
                u16::MAX => SwingLag::Passed,
                count => SwingLag::Counting(
                    u8::try_from(count)
                        .ok()
                        .and_then(LagCount::new)
                        .ok_or_else(bad)?,
                ),
            };
            let length = get(6)?;
            if length > cap {
                return Err(bad());
            }
            let stretching = match get(8)? {
                0 => false,
                3072 => true,
                _ => return Err(bad()),
            };
            Ok(TailSegment {
                swinging: get(0)?.cast_signed() < 0,
                lag,
                way: if get(4)?.cast_signed() < 0 {
                    SwingWay::Ahead
                } else {
                    SwingWay::Back
                },
                length: TailLength::new(length).ok_or_else(bad)?,
                stretching,
                angle: TailAngle(get(10)?),
                place: Pair {
                    x: Wrapping(get(12)?),
                    y: Wrapping(get(14)?),
                },
                offset: Pair {
                    x: get(16)?.cast_signed(),
                    y: get(18)?.cast_signed(),
                },
            })
        })
        .collect::<Result<Vec<_>, Outcome>>()?
        .try_into()
        .map_err(|_| bad())?;
    Ok(FlightTail {
        pace,
        mark,
        swish,
        segments,
    })
}

/// His phase from his slot's function, with its count and wait words and, roaring, his list.
fn phase(wram: &[u8], slot: u16) -> Result<RidleyPhase, Outcome> {
    let [function, .., count, wait] = source::ENEMY_AI_VARIABLES;
    let count = slot_word(wram, slot, count)?;
    let wait = slot_word(wram, slot, wait)?;
    Ok(match slot_word(wram, slot, function)? {
        0xA35B => RidleyPhase::Holding,
        0xA377 => RidleyPhase::Waiting {
            wait: RidleyWait::new(wait).ok_or_else(wait_unsupported)?,
        },
        // A negative count would end the eye's fade at once, which the game never does.
        0xA389 => match NonZeroU16::new(count) {
            None => RidleyPhase::Awake,
            Some(steps) if count < 0x8000 => RidleyPhase::EyeFading { steps },
            Some(_) => return Err(unsupported("Ceres Ridley eye observation")),
        },
        0xA3DF if count % BODY_SET_BYTES == 0 => RidleyPhase::BodyFading {
            sets: count / BODY_SET_BYTES,
            runs: runs(wait)?,
        },
        0xA3DF => return Err(unsupported("Ceres Ridley body observation")),
        0xA455 => RidleyPhase::ReadyToRoar {
            wait: ReadyWait::new(runs(wait)?).ok_or_else(wait_unsupported)?,
        },
        0xA478 => RidleyPhase::Roaring {
            wait: RoarWait::new(runs(wait)?).ok_or_else(wait_unsupported)?,
            figure: roar_figure(wram, slot)?,
        },
        0xA354 => RidleyPhase::InFlight(FlightPhase::TakingOff),
        0xA6AF => RidleyPhase::InFlight(FlightPhase::Rising),
        0xA6C8 => RidleyPhase::InFlight(FlightPhase::Slowing),
        0xA6E8 => RidleyPhase::InFlight(FlightPhase::Hovering),
        0xA782 => RidleyPhase::Attacking(Attack::Fireballs),
        0xA83C => RidleyPhase::Attacking(Attack::Lunge),
        0xA88D => RidleyPhase::Attacking(Attack::Swoop),
        0xA971 => RidleyPhase::Retreating,
        0xA9A0 => {
            RidleyPhase::CoastingAbove(CoastRuns::new(runs(wait)?).ok_or_else(wait_unsupported)?)
        }
        code @ (0xAA11 | 0xAA4F | 0xC04E | 0xAA50) => return getaway(wram, code),
        0xBD9A => RidleyPhase::FakeRetreat,
        _ => return Err(unsupported("Ceres Ridley phase observation")),
    })
}

fn getaway(wram: &[u8], code: u16) -> Result<RidleyPhase, Outcome> {
    use types::ceres_getaway::{
        FlybyBabyBeat, FlybyBeats, FlybyProgress, FlybyTurn, FlybyWingBeat, GetawayState,
    };
    let get = |offset| {
        read(
            wram,
            WramWord {
                name: "Ceres Ridley flyby state",
                offset,
            },
        )
    };
    let bad = || unsupported("Ceres Ridley flyby observation");
    if matches!(code, 0xc04e | 0xaa50) {
        return Ok(RidleyPhase::Getaway(GetawayState::WarningReady));
    }
    let baby = match get(0x802c)? {
        0 => FlybyBabyBeat::First,
        1 => FlybyBabyBeat::ToThird,
        2 => FlybyBabyBeat::Third,
        3 => FlybyBabyBeat::ToFirst,
        _ => return Err(bad()),
    };
    let wings = match get(0x802e)? {
        0 => FlybyWingBeat::First,
        1 => FlybyWingBeat::Second,
        _ => return Err(bad()),
    };
    let beats = FlybyBeats { baby, wings };
    let state = if code == 0xaa11 {
        GetawayState::GetawayReady(beats)
    } else {
        let count = get(0x8026)?;
        if !count.is_multiple_of(2) {
            return Err(bad());
        }
        GetawayState::FlyingPast(FlybyProgress {
            shown: core::num::NonZeroUsize::new(usize::from(count / 2)).ok_or_else(bad)?,
            scroll: Pair {
                x: get(0x8028)?.cast_signed(),
                y: get(0x802a)?.cast_signed(),
            },
            turn: FlybyTurn(get(0x7814)?),
            beats,
        })
    };
    Ok(RidleyPhase::Getaway(state))
}

/// A wait or a count of runs that is not one he holds.
fn wait_unsupported() -> Outcome {
    unsupported("Ceres Ridley wait observation")
}

/// The runs his wait word holds, in the byte his waits from his waking on fit in.
fn runs(wait: u16) -> Result<u8, Outcome> {
    u8::try_from(wait).map_err(|_| wait_unsupported())
}

/// The roar's figure from his list and its timer: a figure of length `n` with its timer at `t`
/// has shown `n − t + 1` runs.
fn roar_figure(wram: &[u8], slot: u16) -> Result<RoarFigure, Outcome> {
    let list = slot_word(wram, slot, source::ENEMY_INSTRUCTION)?;
    if list == ROAR_FIFTH {
        return Ok(RoarFigure::Fifth);
    }
    let bad = || unsupported("Ceres Ridley roar observation");
    let (_, figure) = ROAR_FIGURES
        .into_iter()
        .find(|&(entry, _)| entry == list)
        .ok_or_else(bad)?;
    let timer =
        u8::try_from(slot_word(wram, slot, source::ENEMY_INSTRUCTION_TIMER)?).map_err(|_| bad())?;
    figure
        .length()
        .checked_add(1)
        .and_then(|after| after.checked_sub(timer))
        .and_then(|runs| ShownFigure::new(figure, runs))
        .map(RoarFigure::Timed)
        .ok_or_else(bad)
}

/// The take-off's figure from his list and its timer, as for the roar's.
fn takeoff_figure(wram: &[u8], slot: u16) -> Result<TakeOffFigure, Outcome> {
    let list = slot_word(wram, slot, source::ENEMY_INSTRUCTION)?;
    if list == TAKEOFF_FIFTH {
        return Ok(TakeOffFigure::Fifth);
    }
    let bad = || unsupported("Ceres Ridley take-off observation");
    let (_, figure) = TAKEOFF_FIGURES
        .into_iter()
        .find(|&(entry, _)| entry == list)
        .ok_or_else(bad)?;
    let timer =
        u8::try_from(slot_word(wram, slot, source::ENEMY_INSTRUCTION_TIMER)?).map_err(|_| bad())?;
    figure
        .length()
        .checked_add(1)
        .and_then(|after| after.checked_sub(timer))
        .and_then(|runs| ShownFigure::new(figure, runs))
        .map(TakeOffFigure::Timed)
        .ok_or_else(bad)
}

/// The baby as the record holds it: its routine from its list, and its position once it has run.
fn baby(wram: &[u8]) -> Result<Baby, Outcome> {
    let list = read(wram, BABY_LIST)?;
    if list == BABY_NEW {
        return Ok(Baby::NEW);
    }
    let figure = FIDGET
        .iter()
        .position(|&entry| entry == list)
        .map(|place| BabyFigure::Fidget(FidgetFigure::from_usize(place)))
        .or_else(|| {
            let offset = list.checked_sub(GLOW)?;
            let place = usize::from(offset / GLOW_STRIDE);
            (offset % GLOW_STRIDE == 0 && place < GLOW_FIGURES)
                .then(|| BabyFigure::Glow(GlowFigure::from_usize(place)))
        })
        .ok_or_else(|| unsupported("Ceres baby routine observation"))?;
    let runs = u8::try_from(read(wram, BABY_RUNS)?)
        .ok()
        .and_then(|runs| ShownFigure::new(figure, runs))
        .ok_or_else(|| unsupported("Ceres baby runs observation"))?;
    Ok(Baby {
        routine: BabyRoutine::Showing(runs),
        position: Some(Pair {
            x: Wrapping(read(wram, BABY_X)?),
            y: Wrapping(read(wram, BABY_Y)?),
        }),
    })
}

/// Each phase's code, with its count, its runs and its wait where it has them.
fn phase_code(phase: RidleyPhase) -> (i64, u16, u8, u16) {
    match phase {
        RidleyPhase::Holding => (0, 0, 0, 0),
        RidleyPhase::Waiting { wait } => (1, 0, 0, wait.runs()),
        RidleyPhase::Awake => (2, 0, 0, 0),
        RidleyPhase::EyeFading { steps } => (3, steps.get(), 0, 0),
        RidleyPhase::BodyFading { sets, runs } => (4, sets, runs, 0),
        RidleyPhase::ReadyToRoar { wait } => (5, 0, 0, u16::from(wait.runs())),
        RidleyPhase::Roaring { wait, .. } => (6, 0, 0, u16::from(wait.runs())),
        RidleyPhase::InFlight(FlightPhase::TakingOff) => (7, 0, 0, 0),
        RidleyPhase::InFlight(FlightPhase::Rising) => (8, 0, 0, 0),
        RidleyPhase::InFlight(FlightPhase::Slowing) => (9, 0, 0, 0),
        RidleyPhase::InFlight(FlightPhase::Hovering) => (10, 0, 0, 0),
        RidleyPhase::Attacking(Attack::Fireballs) => (11, 0, 0, 0),
        RidleyPhase::Attacking(Attack::Lunge) => (12, 0, 0, 0),
        RidleyPhase::Attacking(Attack::Swoop) => (13, 0, 0, 0),
        RidleyPhase::Retreating => (14, 0, 0, 0),
        RidleyPhase::FakeRetreat => (15, 0, 0, 0),
        RidleyPhase::CoastingAbove(runs) => (16, 0, runs.runs(), 0),
        RidleyPhase::Getaway(types::ceres_getaway::GetawayState::GetawayReady(_)) => (17, 0, 0, 0),
        RidleyPhase::Getaway(types::ceres_getaway::GetawayState::FlyingPast(_)) => (18, 0, 0, 0),
        RidleyPhase::Getaway(types::ceres_getaway::GetawayState::WarningReady) => (19, 0, 0, 0),
    }
}

/// The take-off's figure as a code, 1 to 5 (0 before he takes off), with its runs.
const fn takeoff_code(takeoff: Option<TakeOffFigure>) -> (i64, u8) {
    match takeoff {
        None => (0, 0),
        Some(TakeOffFigure::Timed(shown)) => (
            match shown.figure() {
                TimedTakeOffFigure::First => 1,
                TimedTakeOffFigure::Second => 2,
                TimedTakeOffFigure::Third => 3,
                TimedTakeOffFigure::Fourth => 4,
            },
            shown.runs(),
        ),
        Some(TakeOffFigure::Fifth) => (5, 0),
    }
}

/// The held roaring figure and its number of elapsed runs.
const fn roar_figure_code(phase: RidleyPhase) -> (i64, u8) {
    match phase {
        RidleyPhase::Roaring {
            figure: RoarFigure::Timed(shown),
            ..
        } => (
            match shown.figure() {
                TimedRoarFigure::First => 1,
                TimedRoarFigure::Second => 2,
                TimedRoarFigure::Third => 3,
                TimedRoarFigure::Fourth => 4,
            },
            shown.runs(),
        ),
        RidleyPhase::Roaring {
            figure: RoarFigure::Fifth,
            ..
        } => (5, 0),
        RidleyPhase::Holding
        | RidleyPhase::Waiting { .. }
        | RidleyPhase::Awake
        | RidleyPhase::EyeFading { .. }
        | RidleyPhase::BodyFading { .. }
        | RidleyPhase::ReadyToRoar { .. }
        | RidleyPhase::InFlight(_)
        | RidleyPhase::Attacking(_)
        | RidleyPhase::Retreating
        | RidleyPhase::CoastingAbove(_)
        | RidleyPhase::Getaway(_)
        | RidleyPhase::FakeRetreat => (0, 0),
    }
}

/// Ridley's State by part, so a difference names the part that differs, with what his phase
/// shows and his tail's segments; whether he is in the room first.
pub(super) fn observations(ridley: Option<(&CeresRidley, Shows, &[Tail])>) -> Vec<Observation> {
    let mut fields = vec![Observation::quantity(
        "ridley.placed",
        i64::from(ridley.is_some()),
    )];
    let Some((ridley, seen, tail)) = ridley else {
        return fields;
    };
    let (phase, count, runs, wait) = phase_code(ridley.phase);
    let (figure, figure_runs) = roar_figure_code(ridley.phase);
    let (takeoff, takeoff_runs) = takeoff_code(ridley.takeoff);
    let look = match ridley.look {
        None => 0,
        Some(RidleyLook::Rest) => 1,
        Some(RidleyLook::Open) => 2,
        Some(RidleyLook::Wide) => 3,
        Some(RidleyLook::TakeOff1) => 4,
        Some(RidleyLook::TakeOff2) => 5,
    };
    let (routine, baby_runs) = match ridley.baby.routine {
        BabyRoutine::New => (0, 0),
        BabyRoutine::Showing(shown) => (
            match shown.figure() {
                BabyFigure::Fidget(figure) => figure.into_usize().saturating_add(10),
                BabyFigure::Glow(figure) => figure.into_usize().saturating_add(20),
            },
            shown.runs(),
        ),
    };
    let place = |position: Option<Pair<Px>>| {
        position.map_or(-1, |at| (i64::from(at.x.0) << 16) | i64::from(at.y.0))
    };
    fields.extend(
        [
            ("ridley.phase", phase),
            ("ridley.count", i64::from(count)),
            ("ridley.runs", i64::from(runs)),
            ("ridley.wait", i64::from(wait)),
            ("ridley.figure", figure),
            ("ridley.figure.runs", i64::from(figure_runs)),
            ("ridley.look", look),
            ("ridley.tangible", i64::from(seen.tangible)),
            ("ridley.moving", i64::from(seen.moving)),
            ("ridley.roaring", i64::from(seen.roaring)),
            ("ridley.position", place(Some(ridley.position))),
            (
                "ridley.fraction",
                (i64::from(ridley.fraction.x) << 8) | i64::from(ridley.fraction.y),
            ),
            ("ridley.speed.x", i64::from(ridley.speed.x.0)),
            ("ridley.speed.y", i64::from(ridley.speed.y.0)),
            ("ridley.takeoff.figure", takeoff),
            ("ridley.takeoff.runs", i64::from(takeoff_runs)),
            ("ridley.hover.runs", i64::from(ridley.hover_runs.get())),
            (
                "ridley.baby.routine",
                i64::try_from(routine).unwrap_or(i64::MAX),
            ),
            ("ridley.baby.runs", i64::from(baby_runs)),
            ("ridley.baby.position", place(ridley.baby.position)),
            ("ridley.tail.curl", i64::from(ridley.curl.get())),
        ]
        .map(|(field, value)| Observation::quantity(field, value)),
    );
    fields.extend(
        tail.iter().enumerate().map(|(segment, &at)| {
            Observation::quantity(format!("ridley.tail.{segment}"), place(at))
        }),
    );
    if let Some(tail) = &ridley.flight_tail {
        observe_flight_tail(&mut fields, tail);
    }
    fields
}

fn observe_flight_tail(fields: &mut Vec<Observation>, tail: &FlightTail) {
    let swing_pace = match tail.pace {
        SwingPace::Slow => 1,
        SwingPace::Quick => 2,
        SwingPace::Whip => 8,
    };
    fields.extend([
        Observation::quantity("ridley.tail.pace", swing_pace),
        Observation::quantity(
            "ridley.tail.mark",
            tail.mark.map_or(-1, |mark| i64::from(mark.angle().0)),
        ),
        Observation::quantity("ridley.tail.swish", i64::from(tail.swish.get())),
    ]);
    for (index, segment) in tail.segments.iter().enumerate() {
        let lag = match segment.lag {
            SwingLag::Passed => -1,
            SwingLag::Counting(count) => i64::from(count.get()),
        };
        fields.extend(
            [
                ("swinging", i64::from(segment.swinging)),
                ("lag", lag),
                ("ahead", i64::from(segment.way == SwingWay::Ahead)),
                ("length", i64::from(segment.length.subpixels())),
                ("stretching", i64::from(segment.stretching)),
                ("angle", i64::from(segment.angle.0)),
                ("offset.x", i64::from(segment.offset.x)),
                ("offset.y", i64::from(segment.offset.y)),
            ]
            .map(|(field, value)| {
                Observation::quantity(format!("ridley.tail.{index}.{field}"), value)
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{Shows, baby, observations, observed, seed, tail_segments};
    use boss_ceres_ridley::{
        BabyRoutine, FlightPhase, RidleyLook, RidleyPhase, RoarFigure, TakeOffFigure,
        TimedTakeOffFigure,
    };
    use core::num::Wrapping;
    use types::Pair;

    fn write(wram: &mut [u8], at: usize, value: u16) {
        wram[at..at.saturating_add(2)].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn his_take_off_decodes_from_his_slot_and_his_hovers_word() {
        let mut wram = vec![0; 0x10000];
        let slot = 0x0F78 + 2 * 0x40;
        for (at, value) in [
            (slot, 0xE13F),
            (slot + 0x0E, 0x0400),
            (0x201E, 16),
            (0x7806, 0xBF31),
        ] {
            write(&mut wram, at, value);
        }
        // Taking off: his second figure's second run of 4, crouched; his speed and fractions from
        // his slot, his hover's runs from their word.
        write(&mut wram, slot + 0x30, 0xA354);
        write(&mut wram, slot + 0x1A, 0xE92F);
        write(&mut wram, slot + 0x1C, 3);
        write(&mut wram, slot + 0x16, 0xEA4F);
        write(&mut wram, slot + 0x04, 0x4000);
        write(&mut wram, slot + 0x34, 0xFEA0);
        write(&mut wram, 0x7800, 5);
        write(&mut wram, 0x2014, 1);
        write(&mut wram, 0x201A, u16::MAX);
        write(&mut wram, 0x201C, u16::MAX);
        let flying = seed(&wram).unwrap().unwrap();
        assert_eq!(flying.phase, RidleyPhase::InFlight(FlightPhase::TakingOff));
        assert_eq!(flying.look, Some(RidleyLook::TakeOff1));
        assert!(matches!(
            flying.takeoff,
            Some(TakeOffFigure::Timed(shown))
                if shown.figure() == TimedTakeOffFigure::Second && shown.runs() == 2
        ));
        assert_eq!(flying.fraction, Pair { x: 0x40, y: 0 });
        assert_eq!(flying.speed.y, Wrapping(-352));
        assert_eq!(flying.hover_runs.get(), 5);
        // A fraction's low byte set is not a fraction he keeps, on either axis.
        for subposition in [slot + 0x04, slot + 0x08] {
            write(&mut wram, subposition, 0x4001);
            assert!(seed(&wram).is_err());
            write(&mut wram, subposition, 0x4000);
        }
        assert!(seed(&wram).is_ok());
        // A timer past the figure's length is not one he holds.
        write(&mut wram, slot + 0x1C, 5);
        assert!(seed(&wram).is_err());
    }

    #[test]
    fn ridley_and_the_baby_decode_from_his_slot_and_its_words() {
        let mut wram = vec![0; 0x10000];
        // Invented: Ridley in slot 2, waiting 7 runs, at rest; the baby in its glow's third figure.
        let slot = 0x0F78 + 2 * 0x40;
        for (at, value) in [
            (slot, 0xE13F),
            (slot + 0x02, 0x0100),
            (slot + 0x06, 0x0080),
            (slot + 0x0E, 0x0400),
            (slot + 0x16, 0xE983),
            (slot + 0x30, 0xA377),
            (slot + 0x3A, 7),
            (0x7806, 0xBF61 + 16),
            (0x7808, 3),
            (0x8804, 0x00F0),
            (0x8808, 0x0096),
            (0x201E, 16),
        ] {
            write(&mut wram, at, value);
        }
        let (ridley, seen) = observed(&wram).unwrap().unwrap();
        assert!(matches!(ridley.phase, RidleyPhase::Waiting { wait } if wait.runs() == 7));
        assert!(matches!(ridley.baby.routine, BabyRoutine::Showing(shown) if shown.runs() == 3));
        assert_eq!(seen, Shows::of(ridley.phase));
        assert_eq!(ridley.curl.get(), 16);
        // Waiting, his tail has no resting place.
        let tail = tail_segments(&wram, ridley.phase).unwrap();
        assert_eq!(tail, vec![None; 7]);
        assert_eq!(observations(Some((&ridley, seen, &tail))).len(), 29);
        // A curl outside 8 to 23 is not one his tail holds.
        write(&mut wram, 0x201E, 24);
        assert!(seed(&wram).is_err());
        write(&mut wram, 0x201E, 16);
        // A figure shown past its length is not a state the baby holds.
        write(&mut wram, 0x7808, 5);
        assert!(baby(&wram).is_err());
        write(&mut wram, 0x7808, 3);

        // His body's third set shown, a run since; a count between sets is not his.
        write(&mut wram, slot + 0x30, 0xA3DF);
        write(&mut wram, slot + 0x38, 3 * 22);
        write(&mut wram, slot + 0x3A, 1);
        let phase = seed(&wram).unwrap().unwrap().phase;
        assert_eq!(phase, RidleyPhase::BodyFading { sets: 3, runs: 1 });
        // His tail rests: each segment's place from its x and y, the tip seventh.
        write(&mut wram, 0x202C + 6 * 0x14, 0x0120);
        write(&mut wram, 0x202E + 6 * 0x14, 0x0090);
        let tail = tail_segments(&wram, phase).unwrap();
        assert_eq!(
            tail[6],
            Some(Pair {
                x: Wrapping(0x0120),
                y: Wrapping(0x0090),
            })
        );
        assert_eq!(
            tail[0],
            Some(Pair {
                x: Wrapping(0),
                y: Wrapping(0)
            })
        );
        write(&mut wram, slot + 0x38, 3 * 22 + 1);
        assert!(seed(&wram).is_err());

        // Roaring, tangible, his jaws wide: the third figure, of 96 runs, its timer at 90, has
        // shown 7; roaring and moving show from his words.
        for (at, value) in [
            (slot + 0x0E, 0),
            (slot + 0x16, 0xE9E9),
            (slot + 0x1A, 0xE6A2),
            (slot + 0x1C, 90),
            (slot + 0x30, 0xA478),
            (slot + 0x3A, 200),
            (0x7804, 1),
            (0x781E, 0x59),
        ] {
            write(&mut wram, at, value);
        }
        let (ridley, seen) = observed(&wram).unwrap().unwrap();
        assert_eq!(ridley.look, Some(RidleyLook::Wide));
        let RidleyPhase::Roaring {
            wait,
            figure: RoarFigure::Timed(shown),
        } = ridley.phase
        else {
            panic!("roaring in his third figure");
        };
        assert_eq!((wait.runs(), shown.runs()), (200, 7));
        assert_eq!(seen, Shows::of(ridley.phase));
        // A timer past the figure's length is not a run of it.
        write(&mut wram, slot + 0x1C, 97);
        assert!(seed(&wram).is_err());

        write(&mut wram, slot, 0);
        assert_eq!(seed(&wram).unwrap(), None);
    }
}
