//! Independent first-cut appearance quantities and checked current playing figure.

use data::pose::PoseId;
use data::pose::animation::AnimationFrame;
use game::samus_appearance_bridge::{Clips, figure_of};
use samus_appearance::{Figure, Frames, Laps, LapsLeft, Playhead, State};
use types::BoostStage;

/// Actual current appearance owner, seeded without selecting a future figure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seed {
    /// Complete current clean appearance quantities.
    pub state: State,
    /// Complete independent current cannon readiness from the same initial observation.
    pub cannon: samus_appearance::CannonDelay,
    /// Figure whose imported clip the actual current playhead belongs to.
    pub playing: Figure,
}

/// Invalid independent seed or absent checked content relation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeedFault {
    /// A required observation word is incomplete.
    Truncated,
    /// Unknown pose/dress mapping or absent matching imported figure.
    Figure,
    /// Observed animation frame is outside the imported step map.
    Step,
    /// Observed timer does not fit the clean frame quantity.
    Timer,
    /// Observed boost stage has no clean variant.
    BoostStage,
    /// Independent source readiness is outside its initialized range.
    Cannon,
}

fn word(wram: &[u8], at: usize) -> Result<u16, SeedFault> {
    let bytes = wram
        .get(at..at.saturating_add(2))
        .ok_or(SeedFault::Truncated)?;
    Ok(u16::from_le_bytes(
        bytes.try_into().map_err(|_| SeedFault::Truncated)?,
    ))
}

/// Decode independently observed current quantities through checked immutable content.
/// This codec does not decide frame admission or materialize a later selected figure.
///
/// # Errors
/// Incomplete or invalid quantities, unknown pose/dress key or missing imported clip relation.
pub fn decode(wram: &[u8], clips: &Clips) -> Result<Seed, SeedFault> {
    let id = PoseId(word(wram, 0x0a1c)?);
    let playing = figure_of(id).ok_or(SeedFault::Figure)?;
    let imported = clips.of(id).ok_or(SeedFault::Figure)?;
    if imported.figure != playing {
        return Err(SeedFault::Figure);
    }
    let step = imported
        .step_of(AnimationFrame(word(wram, 0x0a96)?))
        .ok_or(SeedFault::Step)?;
    let timer = u8::try_from(word(wram, 0x0a94)?).map_err(|_| SeedFault::Timer)?;
    let [laps, stage] = word(wram, 0x0b3e)?.to_le_bytes();
    let stage = BoostStage::ALL
        .get(usize::from(stage))
        .copied()
        .ok_or(SeedFault::BoostStage)?;
    Ok(Seed {
        cannon: cannon(wram)?,
        state: State {
            playhead: Playhead {
                step,
                time_left: Frames(timer),
            },
            stage,
            laps_left: Laps::new(laps).map_or(LapsLeft::WithoutBooster, LapsLeft::WithBooster),
        },
        playing,
    })
}

/// Decode the admitted initial observation, never a later Drawing branch or expected value.
/// # Errors
/// Missing source bytes or a source quantity outside the initialized range.
pub fn cannon(wram: &[u8]) -> Result<samus_appearance::CannonDelay, SeedFault> {
    let observed = word(wram, 0x0aaa)?;
    let remaining = 2_u16.checked_sub(observed).ok_or(SeedFault::Cannon)?;
    let remaining = u8::try_from(remaining).map_err(|_| SeedFault::Cannon)?;
    samus_appearance::CannonDelay::new(remaining).ok_or(SeedFault::Cannon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_readiness_is_checked_and_independent_of_later_fields() {
        let mut bytes = vec![0; 131_072];
        for (source, expected) in [(0_u16, Some(2)), (1, Some(1)), (2, Some(0)), (3, None)] {
            bytes
                .get_mut(0x0aaa..0x0aac)
                .unwrap()
                .copy_from_slice(&source.to_le_bytes());
            assert_eq!(
                cannon(&bytes)
                    .ok()
                    .map(samus_appearance::CannonDelay::remaining),
                expected
            );
        }
        assert_eq!(cannon(&[]), Err(SeedFault::Truncated));
    }
}
