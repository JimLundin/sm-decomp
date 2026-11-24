//! Independent observations of Ceres Ridley and the baby he holds, from her entering his room
//! to his take-off: decoded from his enemy slot, his words and the baby's, compared by part.

use super::{compare::Observation, data_failure, result::Outcome};
use crate::source::{self, SlotField, WramWord};
use boss_ceres_ridley::{
    Baby, BabyFigure, BabyRoutine, CeresRidley, FidgetFigure, FigureLength as _, GlowFigure,
    ReadyWait, RidleyLook, RidleyPhase, RidleyWait, RoarFigure, RoarWait, ShownFigure,
    TimedRoarFigure,
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
            tangible: phase.tangible(),
            moving: phase.moving(),
            roaring: phase.roaring(),
        }
    }
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
    // From his take-off his figure is his next part's.
    let look = if phase == RidleyPhase::Flying {
        None
    } else {
        match slot_word(wram, slot, source::ENEMY_SPRITEMAP)? {
            0x804F => None,
            0xE983 => Some(RidleyLook::Rest),
            0xE9C7 => Some(RidleyLook::Open),
            0xE9E9 => Some(RidleyLook::Wide),
            _ => return Err(unsupported("Ceres Ridley look observation")),
        }
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
        baby: baby(wram)?,
    };
    Ok(Some((ridley, seen)))
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
        0xA354 => RidleyPhase::Flying,
        _ => return Err(unsupported("Ceres Ridley phase observation")),
    })
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

/// Ridley's State by part, so a difference names the part that differs, with what his phase
/// shows; whether he is in the room first.
pub(super) fn observations(ridley: Option<(&CeresRidley, Shows)>) -> Vec<Observation> {
    let mut fields = vec![Observation::quantity(
        "ridley.placed",
        i64::from(ridley.is_some()),
    )];
    let Some((ridley, seen)) = ridley else {
        return fields;
    };
    // Each phase's code, with its count, its runs and its wait where it has them.
    let (phase, count, runs, wait) = match ridley.phase {
        RidleyPhase::Holding => (0, 0, 0, 0),
        RidleyPhase::Waiting { wait } => (1, 0, 0, wait.runs()),
        RidleyPhase::Awake => (2, 0, 0, 0),
        RidleyPhase::EyeFading { steps } => (3, steps.get(), 0, 0),
        RidleyPhase::BodyFading { sets, runs } => (4, sets, runs, 0),
        RidleyPhase::ReadyToRoar { wait } => (5, 0, 0, u16::from(wait.runs())),
        RidleyPhase::Roaring { wait, .. } => (6, 0, 0, u16::from(wait.runs())),
        RidleyPhase::Flying => (7, 0, 0, 0),
    };
    let (figure, figure_runs) = match ridley.phase {
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
        | RidleyPhase::Flying => (0, 0),
    };
    let look = match ridley.look {
        None => 0,
        Some(RidleyLook::Rest) => 1,
        Some(RidleyLook::Open) => 2,
        Some(RidleyLook::Wide) => 3,
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
                "ridley.baby.routine",
                i64::try_from(routine).unwrap_or(i64::MAX),
            ),
            ("ridley.baby.runs", i64::from(baby_runs)),
            ("ridley.baby.position", place(ridley.baby.position)),
        ]
        .map(|(field, value)| Observation::quantity(field, value)),
    );
    fields
}

#[cfg(test)]
mod tests {
    use super::{Shows, baby, observations, observed, seed};
    use boss_ceres_ridley::{BabyRoutine, RidleyLook, RidleyPhase, RoarFigure};

    fn write(wram: &mut [u8], at: usize, value: u16) {
        wram[at..at.saturating_add(2)].copy_from_slice(&value.to_le_bytes());
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
        ] {
            write(&mut wram, at, value);
        }
        let (ridley, seen) = observed(&wram).unwrap().unwrap();
        assert!(matches!(ridley.phase, RidleyPhase::Waiting { wait } if wait.runs() == 7));
        assert!(matches!(ridley.baby.routine, BabyRoutine::Showing(shown) if shown.runs() == 3));
        assert_eq!(seen, Shows::of(ridley.phase));
        assert_eq!(observations(Some((&ridley, seen))).len(), 15);
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
