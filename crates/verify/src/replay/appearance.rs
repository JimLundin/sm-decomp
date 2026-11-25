//! Independent appearance observations, her whole State decoded from a record, and bounded
//! borrowed inputs.

use super::{compare::Observation, data_failure, result::Outcome};
use data::pose::animation::{AnimationDelays, AnimationFrame, RunPaces};
use data::pose::{MovementType, PoseId, Poses};
use data::rom::Rom;
use game::production::{AppearanceContent, FigureClip};
use game::samus_appearance_bridge::{Clips, Imported};
use samus_appearance::{
    Activity, Animation, AppearanceState, CannonDelay, Charge, ChargeGlowPosition, ColourSequences,
    Dash, Frame, FramePart, Frames, HurtCount, HurtOpportunity, Inputs, ItemControls, Laps,
    LapsLeft, Liquid, MoveEvent, PaletteOpportunity, Playhead, VisorDelay, VisorStep,
};
use types::{Action, BoostStage, Item, Set};

pub struct Content {
    pub appearance: AppearanceContent,
    pub(super) clips: Clips,
    /// The pose definitions, whose movement types decode her kept action.
    poses: Poses,
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
        Ok(Self {
            appearance: production_content(&rom, &clips)?,
            clips,
            poses,
        })
    }
}

/// The appearance content production plays: `clips`, the run's paces and her hurt pulse's
/// colours imported from `rom`.
pub(super) fn production_content(rom: &Rom, clips: &Clips) -> Result<AppearanceContent, Outcome> {
    let palettes = import::samus::palettes::import(rom.lorom())
        .map_err(|_| data_failure(None, "her palettes content"))?;
    let pulse = game::production::pulse_colours(&palettes)
        .ok_or_else(|| data_failure(None, "her hurt pulse colours"))?;
    let figures = clips
        .poses()
        .iter()
        .map(|entry| FigureClip {
            figure: entry.figure,
            clip: entry.clip.clone(),
        })
        .collect();
    AppearanceContent::new(figures, clips.paces().clone(), pulse)
        .map_err(|_| data_failure(None, "animation content relation"))
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

/// Her animation's four quantities on `imported`'s clip.
fn animation(wram: &[u8], imported: &Imported) -> Result<Animation, Outcome> {
    let step = imported
        .step_of(AnimationFrame(word(wram, 0x0a96)?))
        .ok_or_else(|| data_failure(None, "observed playhead"))?;
    let timer =
        u8::try_from(word(wram, 0x0a94)?).map_err(|_| data_failure(None, "observed timer"))?;
    let [laps, stage] = word(wram, 0x0b3e)?.to_le_bytes();
    let stage = BoostStage::ALL
        .get(usize::from(stage))
        .copied()
        .ok_or_else(|| data_failure(None, "observed boost"))?;
    Ok(Animation {
        playhead: Playhead {
            step,
            time_left: Frames(timer),
        },
        stage,
        laps_left: Laps::new(laps).map_or(LapsLeft::WithoutBooster, LapsLeft::WithBooster),
    })
}

/// Her kept action, from the movement type her last preparation kept (`0A11`).
///
/// The record keeps a movement type, which several actions may share. The seed takes her pose's
/// action when the type is her pose's; otherwise the one action of every pose of hers whose
/// definition has that type. A type her poses share between actions is Unsupported. Every frame
/// these cases play is prepared before drawing reads the kept action, so the seed's value is
/// carried and never read.
fn kept_action(wram: &[u8], content: &Content, current: Action) -> Result<Action, Outcome> {
    let kept = wram
        .get(0x0a11)
        .copied()
        .and_then(|kept| MovementType::try_from(kept).ok())
        .ok_or_else(|| data_failure(None, "kept movement type"))?;
    let movement = |pose| {
        content
            .poses
            .get(pose)
            .map(|definition| definition.movement)
            .ok_or_else(|| data_failure(None, "kept movement type pose"))
    };
    if movement(PoseId(word(wram, 0x0a1c)?))? == kept {
        return Ok(current);
    }
    let mut actions = Vec::new();
    for entry in &data::assets::samus::map::MAP {
        if movement(entry.id)? == kept && !actions.contains(&entry.key.action) {
            actions.push(entry.key.action);
        }
    }
    match actions.as_slice() {
        [action] => Ok(*action),
        _ => Err(unsupported(
            "appearance kept movement type of no single action",
        )),
    }
}

/// The cannon delay (`0AAA`: eligible controls left, `2 - word`) and the colour sequences
/// (`0A72`: the visor delay in its low byte, twice the visor's entry in its high; `0B62`: two
/// source units a charge-glow picture). Other effects reuse that storage, so the glow decode
/// holds for the ordinary charge sequence only; it is enough here because the sequences are
/// carried and never compared.
fn remembered(wram: &[u8]) -> Result<(CannonDelay, ColourSequences), Outcome> {
    let cannon = 2u16
        .checked_sub(word(wram, 0x0aaa)?)
        .and_then(|left| u8::try_from(left).ok())
        .and_then(CannonDelay::new)
        .ok_or_else(|| data_failure(None, "observed cannon readiness"))?;
    let [visor_delay, visor_entry] = word(wram, 0x0a72)?.to_le_bytes();
    let glow = word(wram, 0x0b62)?;
    if visor_entry % 2 != 0 || glow % 2 != 0 {
        return Err(data_failure(None, "observed colour sequences"));
    }
    let glow_picture =
        u8::try_from(glow / 2).map_err(|_| data_failure(None, "observed charge-glow picture"))?;
    Ok((
        cannon,
        ColourSequences {
            visor_delay: VisorDelay::new(visor_delay),
            visor_step: VisorStep::new(visor_entry / 2),
            charge_glow_position: ChargeGlowPosition::new(glow_picture),
        },
    ))
}

/// Her whole appearance State, with `imported` the figure she is drawn in.
fn whole(wram: &[u8], content: &Content, imported: &Imported) -> Result<AppearanceState, Outcome> {
    let (cannon, colours) = remembered(wram)?;
    Ok(AppearanceState {
        animation: animation(wram, imported)?,
        kept_action: kept_action(wram, content, imported.figure.pose.action)?,
        cannon,
        colours,
        hurt_count: hurt_count(wram)?,
    })
}

/// Her hurt count (`0A48`), 0 to 59.
pub(super) fn hurt_count(wram: &[u8]) -> Result<HurtCount, Outcome> {
    u8::try_from(word(wram, 0x0a48)?)
        .ok()
        .and_then(HurtCount::new)
        .ok_or_else(|| data_failure(None, "observed hurt count"))
}

/// Her hurt count as a compared field.
pub(super) fn hurt_observation(count: HurtCount) -> Observation {
    Observation::quantity("appearance.hurt-count", i64::from(count.frames()))
}

pub fn seed(wram: &[u8], content: &Content) -> Result<AppearanceState, Outcome> {
    whole(wram, content, figure(wram, content)?)
}

/// Independently decode the complete running owner and playing Figure for a start.
pub(super) fn running_owner(
    wram: &[u8],
    content: &Content,
) -> Result<(AppearanceState, samus_appearance::Figure), Outcome> {
    let (state, figure) = owner(wram, content)?;
    running(figure)?;
    Ok((state, figure))
}

/// Her animation and the running figure she is drawn in, as a comparison reads them.
pub(super) fn running_animation(
    wram: &[u8],
    content: &Content,
) -> Result<(Animation, samus_appearance::Figure), Outcome> {
    let imported = drawn(wram, content)?;
    running(imported.figure)?;
    Ok((animation(wram, imported)?, imported.figure))
}

fn running(figure: samus_appearance::Figure) -> Result<(), Outcome> {
    if figure.pose.action != types::Action::Running
        || figure.sprite != types::Sprite::Plain
        || !matches!(
            figure.pose.facing,
            types::Facing::Left | types::Facing::Right
        )
    {
        return Err(unsupported("running side-facing figure"));
    }
    Ok(())
}

/// The figure she is drawn in, in any pose with imported content.
fn drawn<'a>(wram: &[u8], content: &'a Content) -> Result<&'a Imported, Outcome> {
    content
        .clips
        .of(PoseId(word(wram, 0x0a1c)?))
        .ok_or_else(|| data_failure(None, "figure content"))
}

/// Her whole appearance State and the figure she is drawn in, in any pose with imported
/// content.
pub(super) fn owner(
    wram: &[u8],
    content: &Content,
) -> Result<(AppearanceState, samus_appearance::Figure), Outcome> {
    let imported = drawn(wram, content)?;
    Ok((whole(wram, content, imported)?, imported.figure))
}

/// Her animation and the figure she is drawn in, as a comparison reads them, in any pose with
/// imported content.
pub(super) fn animation_owner(
    wram: &[u8],
    content: &Content,
) -> Result<(Animation, samus_appearance::Figure), Outcome> {
    let imported = drawn(wram, content)?;
    Ok((animation(wram, imported)?, imported.figure))
}

/// The recorded frame from `prior`: a frame of play, prepared, moved and drawn, as the admitted
/// handlers run it. Her item controls and the palette opportunity are outside the case: the
/// frame reports the controls skipped and no palette opportunity.
pub fn input(prior: &[u8], content: &Content) -> Result<Frame, Outcome> {
    let imported = figure(prior, content)?;
    let held = word(prior, 0x008b)?;
    let shoot_held = (held & word(prior, 0x09b2)?) != 0;
    let mut moves = Set::default();
    moves[MoveEvent::StandingRestart] = matches!(word(prior, 0x0a1c)?, 1 | 2) && shoot_held;
    // WallJumpReadied has no kick on this clip. HandedOver is read only when
    // figures differ, and the admitted interval has no dash (`0B3C` is 0), so none is lost.
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
    let inputs = Inputs {
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
        // Her hurt colours are outside the case, which reads no fall.
        previous_energy: word(prior, 0x09c2)?,
        items,
        // The step slowdown is observed zero at both cuts; the start slowdown and
        // submerged are unread.
        liquid: Liquid::NONE,
        sounds_waiting: samus_appearance::SoundsWaiting::new(waiting)
            .ok_or_else(|| data_failure(None, "appearance sound queue count"))?,
        activities: activities(prior)?,
        shoot_held,
        charge: charge(prior)?,
    };
    let mut parts = Set::default();
    for part in [
        FramePart::Preparation,
        FramePart::Movement,
        FramePart::Drawing,
    ] {
        parts[part] = true;
    }
    Ok(Frame {
        inputs,
        parts,
        item_controls: ItemControls::Skipped,
        palette: PaletteOpportunity::Absent,
        hurt: HurtOpportunity::Absent,
    })
}

/// The owners active in `wram` that her appearance reads: the cinematic (`1F51`), a boss
/// (`179C`) and special colour time (`0A68`).
pub(super) fn activities(wram: &[u8]) -> Result<Set<Activity>, Outcome> {
    let mut activities = Set::default();
    for (at, activity) in [
        (0x1f51, Activity::Cinematic),
        (0x179c, Activity::Boss),
        (0x0a68, Activity::SpecialColourTime),
    ] {
        activities[activity] = word(wram, at)? != 0;
    }
    Ok(activities)
}

/// How long her shot has charged (`0CD0`).
pub(super) fn charge(wram: &[u8]) -> Result<Charge, Outcome> {
    Charge::new(word(wram, 0x0cd0)?).ok_or_else(|| data_failure(None, "observed charge"))
}

pub fn expected(wram: &[u8], content: &Content) -> Result<Vec<Observation>, Outcome> {
    // Direct record conversion uses no model transition or reconstructed outputs.
    Ok(initial(animation(wram, figure(wram, content)?)?, content))
}

pub fn actual(state: &AppearanceState, content: &Content) -> Vec<Observation> {
    initial(state.animation, content)
}

pub fn initial(state: Animation, _content: &Content) -> Vec<Observation> {
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
