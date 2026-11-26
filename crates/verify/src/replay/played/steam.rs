//! Steam vents at a recorded cut, in enemy slot order.

use boss_ceres_ridley::{FirstPuffWait, Puff, PuffFigure, Vent, VentFacing, VentPhase};
use core::num::{NonZeroU8, Wrapping};
use types::{Hit, Pair};

use crate::replay::{compare::Observation, data_failure, result::Outcome};
use crate::source::{self, SlotField};

const SPECIES: u16 = 0xe1ff;

/// Active vents in population order.
pub(super) fn slots(wram: &[u8]) -> Result<Vec<u16>, Outcome> {
    let mut slots = Vec::new();
    for slot in 0..32 {
        if source::ENEMY_DATA
            .word(source::ENEMY_SPECIES, slot)
            .and_then(|field| field.read(wram))
            .map_err(|_| data_failure(None, "steam vent slot"))?
            == SPECIES
        {
            slots.push(slot);
        }
    }
    Ok(slots)
}

/// Restore each vent from its facing, instruction, displayed figure and counters.
pub(super) fn decode(wram: &[u8]) -> Result<Vec<Vent>, Outcome> {
    decode_vents(wram, false)
}

/// Restore the mounted variants with their ordinary leftward or rightward phase.
pub(super) fn decode_shaft(wram: &[u8]) -> Result<Vec<Vent>, Outcome> {
    decode_vents(wram, true)
}

fn decode_vents(wram: &[u8], mounted: bool) -> Result<Vec<Vent>, Outcome> {
    let mut vents = Vec::new();
    let bad = || data_failure(None, "steam vent encoding");
    for slot in slots(wram)? {
        let read = |field: SlotField| {
            source::ENEMY_DATA
                .word(field, slot)
                .and_then(|field| field.read(wram))
                .map_err(|_| bad())
        };
        let (facing, list, first_figure): (_, u16, u16) = match read(source::ENEMY_PARAMETER_1)? {
            0 => (VentFacing::Up, 0xf04d, 0xf142),
            1 => (VentFacing::Left, 0xf081, 0xf188),
            2 => (VentFacing::Down, 0xf0b5, 0xf1ce),
            3 => (VentFacing::Right, 0xf0e9, 0xf214),
            4 if mounted => (VentFacing::Left, 0xf081, 0xf188),
            5 if mounted => (VentFacing::Right, 0xf0e9, 0xf214),
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
pub(super) fn station(station: &game::production::played::station::State) -> Observation {
    state(
        station.vents.iter().chain(
            station
                .escape_shaft
                .iter()
                .flat_map(|shaft| shaft.vents.iter().map(|vent| &vent.vent)),
        ),
    )
}

pub(super) fn state<'a>(vents: impl Iterator<Item = &'a Vent>) -> Observation {
    let bytes = vents
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
    super::contact::expected("steam.hit", previous, current)
}

/// The emitted side and push. The health observations separately compare the energy and
/// protection the impact actually leaves.
pub(super) fn actual_hit(hit: Option<Hit>) -> Observation {
    super::contact::actual("steam.hit", hit)
}
