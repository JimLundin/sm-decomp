//! Independent door observations; framework scheduling and Samus are borrowed inputs.

use super::{compare::Observation, data_failure, result::Outcome};
use crate::source::{self, SlotField};
use boss_ceres_ridley::{CeresDoor, DoorKind, Figure, FigureLook, Phase, Quarter, Wait};
use core::num::Wrapping;
use types::{Pair, Side};

#[derive(Clone, Copy)]
pub(super) struct Slot(pub(super) u16);

impl Slot {
    /// Its place among the enemy slots.
    pub(super) const fn index(self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for Slot {
    type Error = Outcome;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value < SLOTS {
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

/// The species word a Ceres door's slot holds.
const SPECIES: u16 = 0xE23F;
/// The enemy slots.
const SLOTS: u16 = 32;

/// The first parameter that makes an actor of the doors' species the shaft's overlay.
const SHAFT_OVERLAY: u16 = 2;
/// The properties' bits for a hidden and an intangible actor.
const HIDDEN: u16 = 0x0100;
const INTANGIBLE: u16 = 0x0400;

/// The slots that hold a Ceres door, in the room's order of its enemies: the actors of the
/// doors' species but the shaft's overlay.
pub(super) fn doors(wram: &[u8]) -> Result<Vec<Slot>, Outcome> {
    let mut doors = Vec::new();
    for slot in (0..SLOTS).map(Slot) {
        if word(wram, slot, source::ENEMY_SPECIES)? == SPECIES
            && word(wram, slot, source::ENEMY_PARAMETER_1)? != SHAFT_OVERLAY
        {
            doors.push(slot);
        }
    }
    Ok(doors)
}

/// Whether a slot holds the shaft's overlay, which shows the shaft's lights.
pub(super) fn shaft_overlay(wram: &[u8]) -> Result<bool, Outcome> {
    for slot in (0..SLOTS).map(Slot) {
        if word(wram, slot, source::ENEMY_SPECIES)? == SPECIES
            && word(wram, slot, source::ENEMY_PARAMETER_1)? == SHAFT_OVERLAY
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn identity(wram: &[u8], slot: Slot) -> Result<(DoorKind, Side), Outcome> {
    if word(wram, slot, source::ENEMY_SPECIES)? != SPECIES {
        return Err(unsupported("Ceres door actor identity"));
    }
    match word(wram, slot, source::ENEMY_PARAMETER_1)? {
        0 => Ok((DoorKind::Door, Side::Right)),
        1 => Ok((DoorKind::Door, Side::Left)),
        3 => Ok((DoorKind::RidleyDoor, Side::Right)),
        4 => Ok((DoorKind::ShaftDoor, Side::Left)),
        5 => Ok((DoorKind::GetawayWall, Side::Left)),
        6 => Ok((DoorKind::GetawayWall, Side::Right)),
        _ => Err(unsupported("Ceres actor kind")),
    }
}

fn phase(wram: &[u8], slot: Slot) -> Result<Phase, Outcome> {
    let wait = Wait::new(word(wram, slot, source::ENEMY_INSTRUCTION_TIMER)?)
        .ok_or_else(|| data_failure(None, "Ceres nonzero wait"))?;
    let slam = |quarter| Phase::Slamming { quarter, wait };
    Ok(match word(wram, slot, source::ENEMY_INSTRUCTION)? {
        0xF56C | 0xF5BE | 0xF61A | 0xF53A | 0xF62A | 0xF634 => Phase::Placed,
        0xF542 => Phase::Lingering { wait },
        0xF54A => slam(Quarter::First),
        0xF54E => slam(Quarter::Second),
        0xF552 => slam(Quarter::Third),
        0xF556 => slam(Quarter::Fourth),
        0xF55C => Phase::Slammed,
        0xF562 => Phase::Locked { wait },
        // A getaway wall shows by its properties alone, whichever of its two steps it is at.
        0xF630 | 0xF63A if word(wram, slot, source::ENEMY_PROPERTIES)? & HIDDEN == 0 => {
            Phase::Standing
        }
        0xF630 | 0xF63A => Phase::Unseen,
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
        Phase::Lingering { .. } => 5,
        Phase::Locked { .. } => 6,
        Phase::Slammed => 7,
        Phase::Slamming { quarter: part, .. } => 30_i64.saturating_add(quarter(part)),
        Phase::Standing => 8,
        Phase::Unseen => 9,
    }
}

const fn wait_value(value: Phase) -> u16 {
    match value {
        Phase::Placed | Phase::Wall | Phase::Slammed | Phase::Standing | Phase::Unseen => 1,
        Phase::Starting { wait }
        | Phase::Open { wait }
        | Phase::Closing { wait, .. }
        | Phase::Closed { wait }
        | Phase::Opening { wait, .. }
        | Phase::Lingering { wait }
        | Phase::Locked { wait }
        | Phase::Slamming { wait, .. } => wait.frames(),
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
                FigureLook::Side => 6,
            })
        }
    }
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
        0xFACE => 6,
        0xFB2F => 16,
        _ => return Err(unsupported("Ceres figure observation")),
    };
    let properties = word(wram, slot, source::ENEMY_PROPERTIES)?;
    Ok(observations(
        phase_value(phase(wram, slot)?),
        word(wram, slot, source::ENEMY_INSTRUCTION_TIMER)?,
        figure,
        properties & HIDDEN == 0,
        properties & INTANGIBLE == 0,
    ))
}

/// Whether the door is drawn with Ridley, as the record holds it: his drawing reads the door's
/// own flag.
pub(super) fn drawn_with_ridley(wram: &[u8], slot: Slot) -> Result<Observation, Outcome> {
    let [_, flag, ..] = source::ENEMY_AI_VARIABLES;
    Ok(drawn_with_ridley_observation(word(wram, slot, flag)? != 0))
}

/// Whether the door is drawn with Ridley, as the model leaves it (`DrawnWithRidley`).
pub(super) const fn drawn_with_ridley_of(state: &CeresDoor) -> bool {
    state.phase.drawn_with_ridley()
}

pub(super) fn drawn_with_ridley_observation(drawn: bool) -> Observation {
    Observation::quantity("ceres.drawn-with-ridley", i64::from(drawn))
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

#[cfg(test)]
mod tests {
    use super::{Slot, expected};
    use crate::replay::compare::Observation;

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
}
