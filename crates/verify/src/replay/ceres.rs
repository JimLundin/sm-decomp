//! Independent door observations; framework scheduling and Samus are borrowed inputs.

use super::{compare::Observation, data_failure, result::Outcome};
use crate::source::{self, SlotField};
use boss_ceres_ridley::{
    CeresDoor, DoorKind, Figure, FigureLook, Inputs, Phase, Quarter, Run, Stage, Wait,
};
use core::num::Wrapping;
use types::{Pair, Side};

#[derive(Clone, Copy)]
pub(super) struct Slot(u16);

impl TryFrom<u16> for Slot {
    type Error = Outcome;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value < 32 {
            Ok(Self(value))
        } else {
            Err(data_failure(None, "Ceres actor slot"))
        }
    }
}

fn word(wram: &[u8], slot: Slot, field: SlotField) -> Result<u16, Outcome> {
    source::ENEMY_DATA
        .word(field, slot.0)
        .and_then(|field| field.read(wram))
        .map_err(|_| data_failure(None, "Ceres actor observation"))
}

fn unsupported(dependency: &str) -> Outcome {
    Outcome::Unsupported {
        cut: None,
        dependency: dependency.to_owned(),
    }
}

fn identity(wram: &[u8], slot: Slot) -> Result<(DoorKind, Side), Outcome> {
    if word(wram, slot, source::ENEMY_SPECIES)? != 0xE23F {
        return Err(unsupported("Ceres door actor identity"));
    }
    match word(wram, slot, source::ENEMY_PARAMETER_1)? {
        0 => Ok((DoorKind::Door, Side::Right)),
        1 => Ok((DoorKind::Door, Side::Left)),
        4 => Ok((DoorKind::ShaftDoor, Side::Left)),
        _ => Err(unsupported("Ceres actor kind")),
    }
}

fn phase(wram: &[u8], slot: Slot) -> Result<Phase, Outcome> {
    let wait = Wait::new(word(wram, slot, source::ENEMY_INSTRUCTION_TIMER)?)
        .ok_or_else(|| data_failure(None, "Ceres nonzero wait"))?;
    Ok(match word(wram, slot, source::ENEMY_INSTRUCTION)? {
        0xF56C | 0xF5BE | 0xF61A => Phase::Placed,
        0xF574 | 0xF5C6 => Phase::Starting { wait },
        0xF57C | 0xF5CE => Phase::Open { wait },
        0xF58C | 0xF5DE => Phase::Closing {
            quarter: Quarter::First,
            wait,
        },
        0xF590 | 0xF5E2 => Phase::Closing {
            quarter: Quarter::Second,
            wait,
        },
        0xF594 | 0xF5E6 => Phase::Closing {
            quarter: Quarter::Third,
            wait,
        },
        0xF598 | 0xF5EA => Phase::Closing {
            quarter: Quarter::Fourth,
            wait,
        },
        0xF5A0 | 0xF5F2 => Phase::Closed { wait },
        0xF5AA | 0xF5FC => Phase::Opening {
            quarter: Quarter::First,
            wait,
        },
        0xF5AE | 0xF600 => Phase::Opening {
            quarter: Quarter::Second,
            wait,
        },
        0xF5B2 | 0xF604 => Phase::Opening {
            quarter: Quarter::Third,
            wait,
        },
        0xF5B6 | 0xF608 => Phase::Opening {
            quarter: Quarter::Fourth,
            wait,
        },
        0xF626 => Phase::Wall,
        _ => return Err(unsupported("Ceres phase observation")),
    })
}

pub(super) fn seed(wram: &[u8], slot: Slot) -> Result<CeresDoor, Outcome> {
    let (kind, facing) = identity(wram, slot)?;
    Ok(CeresDoor {
        kind,
        facing,
        position: Pair {
            x: Wrapping(word(wram, slot, source::ENEMY_X_POS)?),
            y: Wrapping(word(wram, slot, source::ENEMY_Y_POS)?),
        },
        phase: phase(wram, slot)?,
    })
}

/// The observed framework supplied exactly one run without replacing this actor.
pub(super) fn validate_step(prior: &[u8], current: &[u8], slot: Slot) -> Result<(), Outcome> {
    if identity(prior, slot)? != identity(current, slot)? {
        return Err(unsupported("Ceres actor replacement"));
    }
    for field in [source::ENEMY_X_POS, source::ENEMY_Y_POS] {
        if word(prior, slot, field)? != word(current, slot, field)? {
            return Err(unsupported("Ceres actor relocation"));
        }
    }
    if word(prior, slot, source::ENEMY_FRAME_COUNTER)?.wrapping_add(1)
        != word(current, slot, source::ENEMY_FRAME_COUNTER)?
    {
        return Err(unsupported("Ceres one-run framework schedule"));
    }
    Ok(())
}

pub(super) fn input(wram: &[u8], slot: Slot) -> Result<Inputs, Outcome> {
    identity(wram, slot)?;
    let read = |field: source::WramWord| {
        field
            .read(wram)
            .map_err(|_| data_failure(None, "Ceres preceding-cut input"))
    };
    let stage = match read(source::CERES_STATUS)? {
        0 => Stage::Calm,
        1 => Stage::Getaway,
        _ => return Err(unsupported("Ceres escape random and shake producers")),
    };
    Ok(Inputs {
        her: Pair {
            x: Wrapping(read(source::SAMUS_X_POS)?),
            y: Wrapping(read(source::SAMUS_Y_POS)?),
        },
        stage,
        // Neither input is read outside Escape, which admission rejects.
        random: 0,
        shake_frames_left: 0,
    })
}

const fn quarter(value: Quarter) -> i64 {
    match value {
        Quarter::First => 1,
        Quarter::Second => 2,
        Quarter::Third => 3,
        Quarter::Fourth => 4,
    }
}

const fn phase_value(value: Phase) -> i64 {
    match value {
        Phase::Placed => 0,
        Phase::Starting { .. } => 1,
        Phase::Open { .. } => 2,
        Phase::Closing { quarter: part, .. } => 10_i64.saturating_add(quarter(part)),
        Phase::Closed { .. } => 3,
        Phase::Opening { quarter: part, .. } => 20_i64.saturating_add(quarter(part)),
        Phase::Wall => 4,
    }
}

const fn wait_value(value: Phase) -> u16 {
    match value {
        Phase::Placed | Phase::Wall => 1,
        Phase::Starting { wait }
        | Phase::Open { wait }
        | Phase::Closing { wait, .. }
        | Phase::Closed { wait }
        | Phase::Opening { wait, .. } => wait.frames(),
    }
}

const fn figure_value(figure: Option<Figure>) -> i64 {
    match figure {
        None => 0,
        Some(Figure { look, facing }) => {
            let side: i64 = match facing {
                Side::Left => 0,
                Side::Right => 10,
            };
            side.saturating_add(match look {
                FigureLook::Closed => 1,
                FigureLook::Ajar1 => 2,
                FigureLook::Ajar2 => 3,
                FigureLook::Ajar3 => 4,
                FigureLook::Open => 5,
            })
        }
    }
}

pub(super) fn invisible(wram: &[u8], slot: Slot) -> Result<bool, Outcome> {
    Ok(word(wram, slot, source::ENEMY_PROPERTIES)? & 0x0100 != 0)
}

pub(super) fn expected(wram: &[u8], slot: Slot) -> Result<Vec<Observation>, Outcome> {
    identity(wram, slot)?;
    let figure = match word(wram, slot, source::ENEMY_SPRITEMAP)? {
        0x804D => 0,
        0xF95F => 1,
        0xF989 => 2,
        0xF9B3 => 3,
        0xF9D3 => 4,
        0xF9F3 => 5,
        0xFA13 => 11,
        0xFA3D => 12,
        0xFA67 => 13,
        0xFA87 => 14,
        0xFAA7 => 15,
        _ => return Err(unsupported("Ceres figure observation")),
    };
    let properties = word(wram, slot, source::ENEMY_PROPERTIES)?;
    Ok(observations(
        phase_value(phase(wram, slot)?),
        word(wram, slot, source::ENEMY_INSTRUCTION_TIMER)?,
        figure,
        properties & 0x0100 == 0,
        properties & 0x0400 == 0,
    ))
}

fn observations(phase: i64, wait: u16, figure: i64, shown: bool, solid: bool) -> Vec<Observation> {
    [
        ("ceres.phase", phase),
        ("ceres.wait", i64::from(wait)),
        ("ceres.figure", figure),
        ("ceres.shown", i64::from(shown)),
        ("ceres.solid", i64::from(solid)),
    ]
    .into_iter()
    .map(|(field, value)| Observation::quantity(field, value))
    .collect()
}

pub(super) fn initial(state: &CeresDoor) -> Vec<Observation> {
    observations(
        phase_value(state.phase),
        wait_value(state.phase),
        figure_value(state.figure()),
        state.phase.shown(),
        state.phase.solid(),
    )
}

pub(super) fn actual(state: &CeresDoor, _outputs: &Run) -> Vec<Observation> {
    initial(state)
}

pub(super) fn context(wram: &[u8], slot: Slot) -> Result<game::production::CeresContext, Outcome> {
    identity(wram, slot)?;
    let stage = match source::CERES_STATUS
        .read(wram)
        .map_err(|_| data_failure(None, "Ceres station stage"))?
    {
        0 => Stage::Calm,
        1 => Stage::Getaway,
        _ => return Err(unsupported("Ceres escape random and shake producers")),
    };
    Ok(game::production::CeresContext {
        stage,
        random: 0,
        shake_frames_left: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::{Slot, expected, input, validate_step};
    use crate::replay::{compare::Observation, result::Outcome};

    fn write(wram: &mut [u8], at: usize, value: u16) {
        wram[at..at.saturating_add(2)].copy_from_slice(&value.to_le_bytes());
    }

    fn captured_door() -> Vec<u8> {
        let mut wram = vec![0; 0x20000];
        for (at, value) in [
            (0x0F78, 0xE23F),
            (0x0F92, 0xF57C),
            (0x0F94, 2),
            (0x0F8E, 0xFAA7),
            (0x0F86, 0x8500),
        ] {
            write(&mut wram, at, value);
        }
        wram
    }

    #[test]
    fn expected_flags_and_figure_are_observed_independently_of_phase() {
        let mut wram = captured_door();
        // Deliberately inconsistent with the phase: expected values must retain the
        // captured figure and flags, rather than asking the model to reconstruct them.
        write(&mut wram, 0x0F8E, 0xFA13);
        write(&mut wram, 0x0F86, 0x8000);
        let observed = expected(&wram, Slot::try_from(0).unwrap()).unwrap();
        assert!(observed.contains(&Observation::quantity("ceres.figure", 11)));
        assert!(observed.contains(&Observation::quantity("ceres.shown", 1)));
        assert!(observed.contains(&Observation::quantity("ceres.solid", 1)));
    }

    #[test]
    fn missing_schedule_and_active_escape_producers_are_unsupported() {
        let prior = captured_door();
        let slot = Slot::try_from(0).unwrap();
        assert!(matches!(
            validate_step(&prior, &prior, slot),
            Err(Outcome::Unsupported { .. })
        ));
        let mut current = prior.clone();
        write(&mut current, 0x0FA4, 1);
        assert!(validate_step(&prior, &current, slot).is_ok());
        write(&mut current, 0x093F, 2);
        assert!(matches!(
            input(&current, slot),
            Err(Outcome::Unsupported { .. })
        ));
        assert!(matches!(
            Slot::try_from(32),
            Err(Outcome::DataFailure { .. })
        ));
        assert!(matches!(
            expected(&[], slot),
            Err(Outcome::DataFailure { .. })
        ));
    }
}
