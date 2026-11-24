//! Independent observations of Ceres Ridley as she enters his room, and the baby he holds:
//! decoded from his enemy slot and the baby's words, compared by part.

use super::{compare::Observation, data_failure, result::Outcome};
use crate::source::{self, SlotField, WramWord};
use boss_ceres_ridley::{
    Baby, BabyFigure, BabyRoutine, CeresRidley, FidgetFigure, GlowFigure, RidleyLook, RidleyPhase,
    RidleyWait, ShownFigure,
};
use core::num::Wrapping;
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

/// The baby's list before its first run, its fidget's eight entries and its glow's first.
const BABY_NEW: u16 = 0xBF31;
const FIDGET: [u16; 8] = [
    0xBF35, 0xBF39, 0xBF3D, 0xBF41, 0xBF49, 0xBF4D, 0xBF51, 0xBF55,
];
const GLOW: u16 = 0xBF61;
const GLOW_STRIDE: u16 = 8;
const GLOW_FIGURES: usize = 12;

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

/// Ceres Ridley as the record holds him, or none outside his room.
pub(super) fn seed(wram: &[u8]) -> Result<Option<CeresRidley>, Outcome> {
    let Some(slot) = slot(wram)? else {
        return Ok(None);
    };
    let [function, .., wait] = source::ENEMY_AI_VARIABLES;
    let phase = match slot_word(wram, slot, function)? {
        0xA35B => RidleyPhase::Holding,
        0xA377 => RidleyPhase::Waiting {
            wait: RidleyWait::new(slot_word(wram, slot, wait)?)
                .ok_or_else(|| unsupported("Ceres Ridley wait observation"))?,
        },
        0xA389 => RidleyPhase::Awake,
        _ => return Err(unsupported("Ceres Ridley phase observation")),
    };
    let look = match slot_word(wram, slot, source::ENEMY_SPRITEMAP)? {
        0x804F => None,
        0xE983 => Some(RidleyLook::Rest),
        _ => return Err(unsupported("Ceres Ridley look observation")),
    };
    Ok(Some(CeresRidley {
        phase,
        look,
        position: Pair {
            x: Wrapping(slot_word(wram, slot, source::ENEMY_X_POS)?),
            y: Wrapping(slot_word(wram, slot, source::ENEMY_Y_POS)?),
        },
        baby: baby(wram)?,
    }))
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

/// Ridley's State by part, so a difference names the part that differs; whether he is in the
/// room first.
pub(super) fn observations(ridley: Option<&CeresRidley>) -> Vec<Observation> {
    let mut fields = vec![Observation::quantity(
        "ridley.placed",
        i64::from(ridley.is_some()),
    )];
    let Some(ridley) = ridley else {
        return fields;
    };
    let (phase, wait) = match ridley.phase {
        RidleyPhase::Holding => (0, 0),
        RidleyPhase::Waiting { wait } => (1, wait.runs()),
        RidleyPhase::Awake => (2, 0),
    };
    let (routine, runs) = match ridley.baby.routine {
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
            ("ridley.wait", i64::from(wait)),
            ("ridley.look", i64::from(ridley.look.is_some())),
            ("ridley.position", place(Some(ridley.position))),
            (
                "ridley.baby.routine",
                i64::try_from(routine).unwrap_or(i64::MAX),
            ),
            ("ridley.baby.runs", i64::from(runs)),
            ("ridley.baby.position", place(ridley.baby.position)),
        ]
        .map(|(field, value)| Observation::quantity(field, value)),
    );
    fields
}

#[cfg(test)]
mod tests {
    use super::{baby, observations, seed};
    use boss_ceres_ridley::{BabyRoutine, RidleyPhase};

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
        let ridley = seed(&wram).unwrap().unwrap();
        assert!(matches!(ridley.phase, RidleyPhase::Waiting { wait } if wait.runs() == 7));
        assert!(matches!(ridley.baby.routine, BabyRoutine::Showing(shown) if shown.runs() == 3));
        assert_eq!(observations(Some(&ridley)).len(), 8);
        // A figure shown past its length is not a state the baby holds.
        write(&mut wram, 0x7808, 5);
        assert!(baby(&wram).is_err());
        write(&mut wram, slot, 0);
        assert_eq!(seed(&wram).unwrap(), None);
    }
}
