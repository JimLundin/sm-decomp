//! Steam vents at a recorded cut, in enemy slot order.

use boss_ceres_ridley::{FirstPuffWait, Puff, PuffFigure, Vent, VentFacing, VentPhase};
use core::num::{NonZeroU8, Wrapping};
use types::{Hit, Pair, Side};

use crate::replay::{compare::Observation, data_failure, result::Outcome};
use crate::source::{self, SlotField};

const SPECIES: u16 = 0xe1ff;

/// Restore each vent from its facing, instruction, displayed figure and counters.
pub(super) fn decode(wram: &[u8]) -> Result<Vec<Vent>, Outcome> {
    let mut vents = Vec::new();
    let bad = || data_failure(None, "steam vent encoding");
    for slot in 0..32 {
        let read = |field: SlotField| {
            source::ENEMY_DATA
                .word(field, slot)
                .and_then(|field| field.read(wram))
                .map_err(|_| bad())
        };
        if read(source::ENEMY_SPECIES)? != SPECIES {
            continue;
        }
        let (facing, list, first_figure): (_, u16, u16) = match read(source::ENEMY_PARAMETER_1)? {
            0 => (VentFacing::Up, 0xf04d, 0xf142),
            1 => (VentFacing::Left, 0xf081, 0xf188),
            2 => (VentFacing::Down, 0xf0b5, 0xf1ce),
            3 => (VentFacing::Right, 0xf0e9, 0xf214),
            4 | 5 => {
                return Err(Outcome::Unsupported {
                    cut: None,
                    dependency: "steam vents on the turning shaft".into(),
                });
            }
            _ => return Err(bad()),
        };
        let instruction = read(source::ENEMY_INSTRUCTION)?;
        let figure = read(source::ENEMY_SPRITEMAP)?;
        let wait = || {
            let wait = u8::try_from(read(source::ENEMY_AI_VARIABLES[3])?).map_err(|_| bad())?;
            FirstPuffWait::new(wait).ok_or_else(bad)
        };
        let hold = || u8::try_from(read(source::ENEMY_INSTRUCTION_TIMER)?).map_err(|_| bad());
        let phase = if instruction == list && figure == 0x804f {
            VentPhase::Placed(wait()?)
        } else if instruction == list.wrapping_add(6) {
            VentPhase::AwaitingFirstPuff(wait()?)
        } else if instruction == list.wrapping_add(0x12) {
            VentPhase::Resting(NonZeroU8::new(hold()?).ok_or_else(bad)?)
        } else {
            let offset = figure.checked_sub(first_figure).ok_or_else(bad)?;
            if !offset.is_multiple_of(10) {
                return Err(bad());
            }
            let index = u8::try_from((offset / 10).saturating_add(1)).map_err(|_| bad())?;
            let figure = PuffFigure::new(index).ok_or_else(bad)?;
            VentPhase::Puffing(Puff::new(figure, hold()?).ok_or_else(bad)?)
        };
        let vent = Vent {
            facing,
            position: Pair {
                x: Wrapping(read(source::ENEMY_X_POS)?),
                y: Wrapping(read(source::ENEMY_Y_POS)?),
            },
            phase,
        };
        if (read(source::ENEMY_PROPERTIES)? & 0x0500 == 0)
            != (vent.shown() || matches!(phase, VentPhase::Placed(_)))
        {
            return Err(bad());
        }
        vents.push(vent);
    }
    Ok(vents)
}

/// Compare complete vent State and the derived drawing figure at every finished cut.
pub(super) fn state(vents: &[Vent]) -> Observation {
    let bytes = vents
        .iter()
        .flat_map(|vent| {
            let facing = match vent.facing {
                VentFacing::Up => 0,
                VentFacing::Left => 1,
                VentFacing::Down => 2,
                VentFacing::Right => 3,
            };
            let [x_lo, x_hi] = vent.position.x.0.to_le_bytes();
            let [y_lo, y_hi] = vent.position.y.0.to_le_bytes();
            let (phase, runs) = match vent.phase {
                VentPhase::Placed(wait) => (0, wait.runs()),
                VentPhase::AwaitingFirstPuff(wait) => (1, wait.runs()),
                VentPhase::Puffing(puff) => (2, puff.remaining_hold()),
                VentPhase::Resting(runs) => (3, runs.get()),
            };
            let drawn = vent.shown_puff_figure().map_or(0, PuffFigure::get);
            [facing, x_lo, x_hi, y_lo, y_hi, phase, runs, drawn]
        })
        .collect();
    Observation {
        field: "steam.vents".into(),
        bytes,
    }
}

/// A steam hit during ordinary play starts protection and spends one of the push's frames
/// before the cut. This reads the observed grant and push without using the vent model.
pub(super) fn expected_hit(previous: &[u8], current: &[u8]) -> Result<Observation, Outcome> {
    let protection = source::SAMUS_HIT_PROTECTION;
    let before = super::word(previous, protection)?;
    let after = super::word(current, protection)?;
    let receipt = if before == 0 && after == 95 {
        let side = super::word(current, source::SAMUS_KNOCKBACK_X_DIRECTION)?;
        let push_left = super::word(current, source::SAMUS_KNOCKBACK_TIMER)?;
        [1, i64::from(side), i64::from(push_left).saturating_add(1)]
    } else {
        [0; 3]
    };
    Ok(impact(receipt))
}

/// The emitted side and push. The health observations separately compare the energy and
/// protection the impact actually leaves.
pub(super) fn actual_hit(hit: Option<Hit>) -> Observation {
    impact(hit.map_or([0; 3], |hit| {
        [
            1,
            i64::from(hit.side == Side::Right),
            i64::from(hit.push.frames()),
        ]
    }))
}

fn impact(receipt: [i64; 3]) -> Observation {
    Observation {
        field: "steam.hit".into(),
        bytes: receipt.into_iter().flat_map(i64::to_le_bytes).collect(),
    }
}
