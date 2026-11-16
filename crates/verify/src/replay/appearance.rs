//! Independent standing-animation observations and bounded borrowed inputs.

use super::{compare::Observation, data_failure, result::Outcome};
use data::pose::animation::{AnimationDelays, AnimationFrame, RunPaces};
use data::pose::{PoseId, Poses};
use data::rom::Rom;
use game::production::{AppearanceContent, FigureClip};
use game::samus_appearance_bridge::{Clips, Imported};
use samus_appearance::{
    Content as _, Dash, Frames, Inputs, Laps, LapsLeft, Liquid, MoveEvent, Playhead, State,
};
use types::{BoostStage, Item, Set};

pub struct Content {
    pub appearance: AppearanceContent,
    clips: Clips,
}

impl Content {
    pub fn load(path: &std::path::Path) -> Result<Self, Outcome> {
        let bytes = std::fs::read(path).map_err(|_| data_failure(None, "player ROM file"))?;
        let rom = Rom::new(bytes).map_err(|_| data_failure(None, "player ROM pin"))?;
        let poses = Poses::load(rom.lorom()).map_err(|_| data_failure(None, "pose content"))?;
        let delays = AnimationDelays::load(rom.lorom(), &poses)
            .map_err(|_| data_failure(None, "animation clip content"))?;
        let paces = RunPaces::load(rom.lorom())
            .map_err(|_| data_failure(None, "animation pace content"))?;
        let clips = Clips::import(&delays, &paces)
            .map_err(|_| data_failure(None, "semantic animation content"))?;
        let figures = clips
            .poses()
            .iter()
            .map(|entry| FigureClip {
                figure: entry.figure,
                clip: entry.clip.clone(),
            })
            .collect();
        let appearance = AppearanceContent::new(figures, clips.paces().clone())
            .map_err(|_| data_failure(None, "animation content relation"))?;
        Ok(Self { appearance, clips })
    }
}

fn word(wram: &[u8], at: usize) -> Result<u16, Outcome> {
    let bytes = wram
        .get(at..at.saturating_add(2))
        .ok_or_else(|| data_failure(None, "appearance record word"))?;
    Ok(u16::from_le_bytes(bytes.try_into().map_err(|_| {
        data_failure(None, "appearance record word")
    })?))
}

fn unsupported(dependency: &str) -> Outcome {
    Outcome::Unsupported {
        cut: None,
        dependency: dependency.to_owned(),
    }
}

fn figure<'a>(wram: &[u8], content: &'a Content) -> Result<&'a Imported, Outcome> {
    let id = word(wram, 0x0a1c)?;
    if !matches!(id, 1 | 2 | 0x27 | 0x28) {
        return Err(unsupported("appearance standing figure interval"));
    }
    let imported = content
        .clips
        .of(PoseId(id))
        .ok_or_else(|| data_failure(None, "appearance figure content"))?;
    // These marked movement interventions cannot apply to admitted standing clips.
    if imported.clip.kick.is_some()
        || imported.clip.fast_fall.is_some()
        || imported.clip.rising.is_some()
        || imported.clip.instructions.iter().any(|instruction| {
            !matches!(
                instruction,
                samus_appearance::Instruction::Show(_)
                    | samus_appearance::Instruction::Control(
                        samus_appearance::Control::Breathe | samus_appearance::Control::Again(_)
                    )
            )
        })
    {
        return Err(unsupported("appearance standing clip movement marks"));
    }
    Ok(imported)
}

/// Admission checks only external scheduling and the borrowed producer interval.
/// Neither animation state nor an animation request decides whether a step is admitted.
pub fn validate_step(prior: &[u8], current: &[u8]) -> Result<(), Outcome> {
    for record in [prior, current] {
        if word(record, 0x0998)? != 8
            || word(record, 0x0a78)? != 0
            || !matches!(word(record, 0x0a44)?, 0xe725 | 0xe8aa)
            || !matches!(word(record, 0x0a1c)?, 1 | 2 | 0x27 | 0x28)
            || word(record, 0x0b3c)? != 0
            || word(record, 0x0a9c)? != 0
        {
            return Err(unsupported("appearance ordinary dry standing schedule"));
        }
    }
    if word(current, 0x05b6)?.wrapping_sub(word(prior, 0x05b6)?) != 1 {
        return Err(unsupported("appearance exactly one gameplay tick"));
    }
    // The animation reads equipment and liquid slowdown later than the prior cut;
    // constancy certifies using those preceding-cut values in this scoped case.
    for at in [0x0a1c, 0x0a44, 0x0a58, 0x0a60, 0x09a2, 0x0a9c] {
        if word(prior, at)? != word(current, at)? {
            return Err(unsupported("appearance unchanged borrowed producers"));
        }
    }
    Ok(())
}

pub fn seed(wram: &[u8], content: &Content) -> Result<State, Outcome> {
    let imported = figure(wram, content)?;
    let step = imported
        .step_of(AnimationFrame(word(wram, 0x0a96)?))
        .ok_or_else(|| data_failure(None, "appearance observed step"))?;
    let timer = u8::try_from(word(wram, 0x0a94)?)
        .map_err(|_| data_failure(None, "appearance observed timer"))?;
    let [laps, stage] = word(wram, 0x0b3e)?.to_le_bytes();
    let stage = BoostStage::ALL
        .get(usize::from(stage))
        .copied()
        .ok_or_else(|| data_failure(None, "appearance observed boost stage"))?;
    Ok(State {
        playhead: Playhead {
            step,
            time_left: Frames(timer),
        },
        stage,
        laps_left: Laps::new(laps).map_or(LapsLeft::WithoutBooster, LapsLeft::WithBooster),
    })
}

pub fn input(prior: &[u8], content: &Content) -> Result<Inputs, Outcome> {
    let imported = figure(prior, content)?;
    let held = word(prior, 0x008b)?;
    let mut moves = Set::default();
    moves[MoveEvent::StandingRestart] =
        matches!(word(prior, 0x0a1c)?, 1 | 2) && (held & word(prior, 0x09b2)?) != 0;
    // WallJumpReadied has no kick on this clip. HandedOver is read only when
    // figures differ, and DashLost cannot occur in the admitted dash-off interval.
    let mut items = Set::default();
    let equipped = word(prior, 0x09a2)?;
    for (mask, item) in [
        (0x0008, Item::ScrewAttack),
        (0x0200, Item::SpaceJump),
        (0x2000, Item::SpeedBooster),
    ] {
        items[item] = (equipped & mask) != 0;
    }
    let waiting = prior
        .get(0x0648)
        .copied()
        .ok_or_else(|| data_failure(None, "appearance sound queue write index"))?
        .wrapping_sub(
            *prior
                .get(0x0645)
                .ok_or_else(|| data_failure(None, "appearance sound queue read index"))?,
        )
        & 0x0f;
    Ok(Inputs {
        figure: imported.figure,
        figure_after: imported.figure,
        moves,
        dash: Dash::Off,
        run_held: (held & word(prior, 0x09b6)?) != 0,
        // Standing clips have neither a rising nor fast-fall mark or wall branch.
        // These movement/liquid quantities are consequently unread under the bound.
        heading_down: false,
        vertical_speed: types::ZERO,
        energy: word(prior, 0x09c2)?,
        items,
        // Slowdown is observed zero at both cuts; start/submergence are unread.
        liquid: Liquid::NONE,
        sounds_waiting: samus_appearance::SoundsWaiting::new(waiting)
            .ok_or_else(|| data_failure(None, "appearance sound queue count"))?,
    })
}

pub fn expected(wram: &[u8], content: &Content) -> Result<Vec<Observation>, Outcome> {
    // Direct record conversion uses no model transition or reconstructed outputs.
    Ok(initial(seed(wram, content)?, content))
}

pub fn actual(
    state: State,
    _sound: Option<types::AnimationSound>,
    content: &Content,
) -> Vec<Observation> {
    initial(state, content)
}

pub fn initial(state: State, _content: &Content) -> Vec<Observation> {
    let boost = match state.stage {
        BoostStage::Stage0 => 0,
        BoostStage::Stage1 => 1,
        BoostStage::Stage2 => 2,
        BoostStage::Stage3 => 3,
        BoostStage::Boosting => 4,
    };
    let laps = match state.laps_left {
        LapsLeft::WithoutBooster => 0,
        LapsLeft::WithBooster(laps) => laps.count(),
    };
    [
        ("appearance.step", state.playhead.step.0),
        ("appearance.time-left", state.playhead.time_left.0),
        ("appearance.boost-stage", boost),
        ("appearance.laps-left", laps),
    ]
    .into_iter()
    .map(|(field, value)| Observation::quantity(field, i64::from(value)))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(wram: &mut [u8], at: usize, value: u16) {
        wram[at..at.saturating_add(2)].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn admission_ignores_animation_owned_outputs_but_rejects_producer_changes() {
        let mut prior = vec![0; 0x20000];
        for (at, value) in [(0x0998, 8), (0x0a1c, 1), (0x0a44, 0xe725)] {
            write(&mut prior, at, value);
        }
        let mut current = prior.clone();
        write(&mut current, 0x05b6, 1);
        for at in [0x0a94, 0x0a96, 0x0b3e, 0x0a2c, 0x0a32] {
            write(&mut current, at, 99);
        }
        assert!(validate_step(&prior, &current).is_ok());
        write(&mut current, 0x09a2, 8);
        assert!(matches!(
            validate_step(&prior, &current),
            Err(Outcome::Unsupported { .. })
        ));
        assert!(matches!(
            validate_step(&[], &[]),
            Err(Outcome::DataFailure { .. })
        ));
    }
}
