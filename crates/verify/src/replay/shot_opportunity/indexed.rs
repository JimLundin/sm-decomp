//! Hook the independent five-cut packet into the shared carried-state replay.

mod audio;
#[cfg(test)]
mod sound_tests;

use super::{
    Completion, Continuation, DrawStore, EXPECTED_PIN, Environment, INPUT_PIN, Input, Opportunity,
    PACKET, START_PIN, Sources, batch_from_bytes, camera, data_failure, movement, oam_bytes, wait,
    weapons, word,
};
use crate::{
    lock::Sha256,
    replay::{
        compare::Observation,
        index::{Case, InputProvider, Start},
        production_failure,
        readers::{ActualReader, AdmittedCase, ExpectedCut, ExpectedReader, InputReader},
        result::Outcome,
        shot_drawing,
    },
};
use data::assets::samus::{key::Dress, map};
use game::{
    production::{
        AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules,
        ProductionSeed, frame::shot_opportunity,
    },
    state::{owned::OwnedState, video::SpriteCount},
};
use serde::de::DeserializeOwned;
use std::path::Path;
use types::{Parts, Pose};
use weapons_projectiles as beam;

pub(in crate::replay) struct Inputs {
    steps: Vec<Step>,
    world: game::production::world::MovementWorld,
    camera: camera::OwnedInput,
    dress: Dress,
}

struct Step {
    physical_pad: u16,
    phase: beam::FlickerPhase,
    stores: Vec<DrawStore>,
}

impl TryFrom<Opportunity> for Step {
    type Error = Outcome;

    fn try_from(input: Opportunity) -> Result<Self, Outcome> {
        Ok(Self {
            physical_pad: input.physical_pad,
            phase: input.phase(),
            stores: input.draw_stores()?,
        })
    }
}

pub(in crate::replay) struct Expectations(Vec<ExpectedCut>);

const FIRST: u64 = 157;
const LAST: u64 = 161;

fn bad(cut: Option<u64>) -> Outcome {
    data_failure(cut, "scoped ordinary shot fixture")
}

fn member(root: &Path, name: &str, pin: &str) -> Result<Vec<u8>, Outcome> {
    let bytes = std::fs::read(root.join(name)).map_err(|_| bad(None))?;
    if Sha256::try_from(pin).ok() != Some(Sha256::of(&bytes)) {
        return Err(bad(None));
    }
    Ok(bytes)
}

fn toml<T: DeserializeOwned>(root: &Path, name: &str, pin: &str) -> Result<T, Outcome> {
    let bytes = member(root, name, pin)?;
    toml::from_str(std::str::from_utf8(&bytes).map_err(|_| bad(None))?).map_err(|_| bad(None))
}

fn at(cut: u64) -> Result<usize, Outcome> {
    cut.checked_sub(FIRST)
        .and_then(|index| usize::try_from(index).ok())
        .ok_or_else(|| bad(Some(cut)))
}

pub(in crate::replay) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let Start::NativeCheckpoint { frame: 156, .. } = case.start else {
        return Err(bad(None));
    };
    let root = sources.native.join(PACKET);
    let input: Input = toml(&root, "inputs.toml", INPUT_PIN)?;
    if input.row != case.row
        || input.admission_cut != 156
        || !input.opportunity.iter().map(|row| row.cut).eq(FIRST..=LAST)
    {
        return Err(bad(None));
    }
    let start = member(&root, "start.record", START_PIN)?;
    let mut record = oracle_record::lsnes::blank();
    oracle_record::lsnes::decode(&start, &mut record).map_err(|_| bad(None))?;
    admit_record(input, &record.wram, sources, &root, case.inputs)
}

fn admit_record(
    input: Input,
    wram: &[u8],
    sources: &Sources,
    root: &Path,
    provider: InputProvider,
) -> Result<AdmittedCase, Outcome> {
    let movement_content = movement::Content::load(&sources.rom)?;
    let power = weapons::Content::load(&sources.rom)?;
    let origin = super::admit(wram, &power)?;
    let player = movement::seed(wram)?;
    let motion = movement::input(wram, player.pose, &movement_content)?;
    let camera = camera::seed(wram, &camera::Content)?;
    let camera_input = camera::input(wram, &camera::Content)?;
    let (_, dress) =
        map::shown(data::pose::PoseId(word(wram, 0x0a1c)?)).ok_or_else(|| bad(None))?;
    let initial_batch = wram.get(0x0370..0x0592).ok_or_else(|| bad(None))?;
    if Sha256::try_from(input.initial_oam_and_cursor_sha256.as_str()).ok()
        != Some(Sha256::of(initial_batch))
    {
        return Err(bad(None));
    }
    let shoot_mask = origin.bindings.shoot.0;
    let mut continuation = Continuation::new(
        origin.clone(),
        player,
        camera,
        motion.stimuli(),
        batch_from_bytes(initial_batch)?,
    );
    let sound_only = provider != InputProvider::ScopedShotOpportunityInputs;
    let no_fire = provider == InputProvider::ShotSoundNoFire;
    let expected = if sound_only {
        let (audio, expected) = audio::admit(sources, wram, no_fire)?;
        continuation = continuation.with_audio(audio);
        Expectations(vec![expected])
    } else {
        Expectations::load(root, &power)?
    };
    let content = ProductionContent::ShotOpportunity(Box::new(shot_opportunity::Content {
        admitted: origin,
        movement: movement_content.movement,
        dress,
        power: power.weapons,
    }));
    let rules = ProductionRules::ShotOpportunity;
    let production = Production::admit(
        ProductionSeed::ShotOpportunity(Box::new(continuation)),
        content.clone(),
        rules.clone(),
    )
    .map_err(|_| bad(None))?;
    let last = if sound_only { FIRST } else { LAST };
    let steps = input
        .opportunity
        .into_iter()
        .take_while(|step| step.cut <= last)
        .map(|row| {
            let mut step = Step::try_from(row)?;
            if no_fire {
                step.physical_pad &= !shoot_mask;
            }
            Ok(step)
        })
        .collect::<Result<_, Outcome>>()?;
    Ok(AdmittedCase {
        production,
        content,
        rules,
        inputs: InputReader::ShotOpportunity(Box::new(Inputs {
            steps,
            world: motion.world,
            camera: camera_input,
            dress,
        })),
        expected: ExpectedReader::ShotOpportunity(Box::new(expected)),
        actual: ActualReader::ShotOpportunity(dress),
        schedule: (FIRST..=last).collect(),
    })
}

impl Inputs {
    pub fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        let index = at(cut)?;
        let current = self.steps.get(index).ok_or_else(|| bad(Some(cut)))?;
        let frame = self.camera.frame();
        production
            .advance(AdvanceInput::ShotOpportunity {
                environment: Environment {
                    world: &self.world,
                    camera_room: frame.room,
                    camera_lead: frame.lead,
                },
                inputs: shot_opportunity::Inputs {
                    sampled: current.physical_pad,
                    phase: current.phase,
                    before: &current.stores,
                },
            })
            .map_err(|fault| production_failure(cut, fault))
    }

    pub fn actual(&self, result: &Completion, cut: u64) -> Result<Vec<Observation>, Outcome> {
        let actors = &result.actors;
        let input = &actors.pose.dependency;
        let before = result.ingress.view.0;
        let view = result.camera.position();
        let mut fields = vec![
            words(
                "shot.controller",
                [input.held.0, input.pressed.0, actors.previous_pressed],
            ),
            words(
                "shot.pose-history",
                [
                    pose_id(input.pose, self.dress, cut)?,
                    pose_id(result.ingress.prior_pose, self.dress, cut)?,
                ],
            ),
            motion(result.ingress.inheritance),
            words(
                "shot.camera",
                [before.x, before.y, view.x.pixel().0, view.y.pixel().0],
            ),
            words(
                "shot.door-clocks",
                actors.doors.map(|door| wait(door.phase)),
            ),
            hash("shot.oam.pre-power", &oam_bytes(&result.pre_power)),
            oam_terminal(&result.drawing.batch),
            opening(actors.runs.iter().map(|run| run.sound.is_some())),
        ];
        let mut attributed = result.drawing.clone();
        attributed.batch.sprites = SpriteCount(result.after_power);
        fields.extend(shot_drawing::actual(&actors.shots, &attributed, cut)?);
        Ok(fields)
    }
}

pub(in crate::replay) fn actual_audio(
    production: &Production,
    completion: &Completion,
    offers: &[game::production::gameplay_sounds::Offer],
) -> Result<Vec<Observation>, Outcome> {
    audio::actual(production, completion, offers)
}

pub(in crate::replay) fn initial(
    production: &Production,
    dress: Dress,
) -> Result<Vec<Observation>, Outcome> {
    let OwnedState::ShotOpportunity(state) = production.state() else {
        return Err(bad(None));
    };
    if state.audio().is_some() {
        return audio::initial(production);
    }
    let origin = state.before();
    // The complete controller tuple needs the next sampling boundary. The pre-power image
    // needs the current surrounding stores. The cut already holds the terminal OAM image.
    let mut fields = weapons::initial(&origin.shots);
    fields.push(motion(origin.weapon.inheritance));
    fields.push(words(
        "shot.door-clocks",
        origin.doors.map(|door| wait(door.phase)),
    ));
    fields.push(words(
        "shot.pose-history",
        [
            pose_id(origin.pose, dress, 156)?,
            pose_id(origin.prior_pose, dress, 156)?,
        ],
    ));
    let view = origin.weapon.camera.0;
    fields.push(words("shot.camera", [view.x, view.y, view.x, view.y]));
    fields.push(oam_terminal(state.batch()));
    Ok(fields)
}

fn pose_id(pose: Pose, dress: Dress, cut: u64) -> Result<u16, Outcome> {
    map::id_of(pose, dress)
        .map(|id| id.0)
        .ok_or_else(|| bad(Some(cut)))
}

fn words(name: &str, value: impl IntoIterator<Item = u16>) -> Observation {
    Observation {
        field: name.to_owned(),
        bytes: value.into_iter().flat_map(u16::to_le_bytes).collect(),
    }
}

fn motion(inheritance: beam::Inheritance) -> Observation {
    let distances = [
        inheritance.left,
        inheritance.right,
        inheritance.up,
        inheritance.down,
    ];
    words(
        "shot.inherited-motion",
        distances
            .into_iter()
            .flat_map(|distance| [distance.whole, distance.fraction.0])
            .chain([inheritance.camera_vertical_fraction.0]),
    )
}

fn hash(name: &str, bytes: &[u8]) -> Observation {
    hash_observation(name, &Sha256::of(bytes).to_string())
}

fn hash_observation(name: &str, pin: &str) -> Observation {
    Observation {
        field: name.to_owned(),
        bytes: pin.as_bytes().to_vec(),
    }
}

fn oam_terminal(batch: &game::state::video::NmiBatch) -> Observation {
    let mut bytes = oam_bytes(&batch.oam);
    bytes.extend(batch.sprites.0.saturating_mul(4).to_le_bytes());
    hash("shot.oam.final", &bytes)
}

fn opening(sounds: impl IntoIterator<Item = bool>) -> Observation {
    Observation {
        field: "shot.door-opening".to_owned(),
        bytes: sounds.into_iter().map(u8::from).collect(),
    }
}

impl Expectations {
    fn load(root: &Path, power: &weapons::Content) -> Result<Self, Outcome> {
        let expected: super::Expected = toml(root, "expected.toml", EXPECTED_PIN)?;
        if !expected
            .opportunity
            .iter()
            .map(|row| row.cut)
            .eq(FIRST..=LAST)
            || !expected.oam.iter().map(|row| row.cut).eq(FIRST..=LAST)
            || !expected
                .shot
                .iter()
                .map(|row| row.drawing.number)
                .eq(FIRST..=LAST)
            || expected.opening.cut != FIRST
            || expected.opening.slot != 1
            || expected.opening.after_wait != 5
        {
            return Err(bad(None));
        }
        expected
            .opportunity
            .into_iter()
            .zip(expected.shot)
            .zip(expected.oam)
            .map(|((current, shot), oam)| expected_cut(&current, &shot, &oam, power))
            .collect::<Result<Vec<_>, Outcome>>()
            .map(Self)
    }

    pub fn read(&self, cut: u64) -> Result<ExpectedCut, Outcome> {
        let saved = self.0.get(at(cut)?).ok_or_else(|| bad(Some(cut)))?;
        Ok(ExpectedCut {
            fields: saved.fields.clone(),
            semantics: saved.semantics.clone(),
            semantic_count: saved.semantic_count,
            output_count: saved.output_count,
            replies: 0,
        })
    }
}

fn expected_cut(
    current: &super::Current,
    shot: &super::Shot,
    oam: &super::ExpectedOam,
    power: &weapons::Content,
) -> Result<ExpectedCut, Outcome> {
    let mut semantics = vec![
        words(
            "shot.controller",
            [
                current.controller_held,
                current.controller_new,
                current.previous_new,
            ],
        ),
        words(
            "shot.pose-history",
            [current.pose_id, current.prior_pose_id],
        ),
        words("shot.inherited-motion", current.inherited),
        words(
            "shot.camera",
            current
                .camera_before
                .into_iter()
                .chain(current.camera_presentation),
        ),
        words("shot.door-clocks", current.door_timers_after),
        hash_observation("shot.oam.pre-power", &oam.pre_power_sha256),
        hash_observation("shot.oam.final", &oam.final_backing_and_cursor_sha256),
    ];
    let state = expected_beam(shot, power)?;
    let mut state_fields = weapons::initial(&state).into_iter();
    semantics.extend(state_fields.by_ref().take(1));
    let mut fields = semantics.clone();
    fields.extend(state_fields);
    fields.push(opening([false, current.door_opened]));
    fields.extend(shot.drawing.outputs(power)?);
    Ok(ExpectedCut {
        semantic_count: semantics.len(),
        output_count: fields.len().saturating_sub(semantics.len()),
        replies: 0,
        fields,
        semantics,
    })
}

fn expected_beam(shot: &super::Shot, power: &weapons::Content) -> Result<beam::State, Outcome> {
    let mut state = beam::State::reset(enum_map::EnumMap::from_fn(|_| beam::Axes {
        x: beam::PixelFraction(0),
        y: beam::PixelFraction(0),
    }));
    let cut = shot.drawing.number;
    if !shot.slots.iter().map(|slot| slot.index).eq(0..5) {
        return Err(bad(Some(cut)));
    }
    state.count = beam::ShotCount(shot.count);
    state.cooldown = beam::FrameWord(shot.cooldown);
    for (target, seen) in state.slots.values_mut().zip(&shot.slots) {
        *target = expected_slot(seen, power, cut)?;
    }
    Ok(state)
}

fn expected_slot(
    seen: &super::Slot,
    power: &weapons::Content,
    cut: u64,
) -> Result<beam::ShotState, Outcome> {
    let trail = beam::FrameWord(seen.trail_countdown);
    let fraction = beam::Axes {
        x: beam::PixelFraction(seen.x_fraction),
        y: beam::PixelFraction(seen.y_fraction),
    };
    if seen.next_clip_frame == 0 {
        return Ok(beam::ShotState::Free {
            trail,
            retained_fraction: fraction,
        });
    }
    if seen.heading_and_consumed & !31 != 0 {
        return Err(bad(Some(cut)));
    }
    let heading = weapons::heading(seen.heading_and_consumed & 15)?;
    let (clip, next_frame) = power.flight_frame(heading, seen.next_clip_frame)?;
    Ok(beam::ShotState::Flying {
        consumed: seen.heading_and_consumed & 16 != 0,
        body: beam::ShotBody {
            position: beam::Axes {
                x: beam::FixedPosition::from_parts(seen.x, fraction.x),
                y: beam::FixedPosition::from_parts(seen.y, fraction.y),
            },
            heading,
            speed: beam::Axes {
                x: beam::BeamSpeed(seen.x_speed),
                y: beam::BeamSpeed(seen.y_speed),
            },
            radius: beam::Axes {
                x: beam::Radius(seen.x_radius),
                y: beam::Radius(seen.y_radius),
            },
            damage: beam::Damage::new(seen.damage).ok_or_else(|| bad(Some(cut)))?,
            trail,
            clip,
            next_frame,
            remaining_hold: beam::FrameWord(seen.hold),
            picture: Some(power.picture(seen.picture)?),
            trail_appearance: None,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::{index::Index, run, runner};

    #[test]
    #[ignore = "requires the checked five-cut shot fixture and player ROM"]
    fn carried_opening_selects_current_fire_and_runs_slot_one() {
        let sources = Sources {
            rom: std::env::var_os("SM_ROM").unwrap().into(),
            native: std::env::var_os("SM_AUDIO_NATIVE").unwrap().into(),
            ..Sources::default()
        };
        let index =
            Index::parse(include_str!("../../../../../scenarios/replay/index.toml")).unwrap();
        let case = index
            .cases
            .iter()
            .find(|case| case.id.0 == "weapons-contact/M0-carried-shot-opportunities")
            .unwrap();
        let result = run(case, &sources);
        assert_eq!(result.outcome, Outcome::Pass);
        assert_eq!(result.counts.cuts, 5);
        assert_eq!(result.counts.restorations, 5);
        assert_eq!(result.counts.null_fields, 7);
        check_admission_relations(case, &sources);
        check_no_fire(case, &sources);
        check_carried_oam_fault(case, &sources);
    }

    fn check_admission_relations(case: &Case, sources: &Sources) {
        use game::production::AdmissionFault;
        let admitted = admit(case, sources).unwrap();
        let OwnedState::ShotOpportunity(state) = admitted.production.state() else {
            panic!("wrong admitted state");
        };
        let ProductionContent::ShotOpportunity(mut content) = admitted.content.clone() else {
            panic!("wrong admitted content");
        };
        let InputReader::ShotOpportunity(inputs) = &admitted.inputs else {
            panic!("wrong admitted inputs");
        };
        let mut restored = Production::restore(
            &admitted.production.save().unwrap(),
            admitted.content,
            admitted.rules.clone(),
        )
        .unwrap();
        let mut direct = admitted.production.clone();
        assert!(
            inputs
                .advance(&mut restored, FIRST)
                .unwrap()
                .eq(&inputs.advance(&mut direct, FIRST).unwrap())
        );

        content.admitted.bindings.shoot.0 = 0x1000;
        assert!(matches!(
            Production::admit(
                ProductionSeed::ShotOpportunity(state.clone()),
                ProductionContent::ShotOpportunity(content.clone()),
                admitted.rules.clone()
            ),
            Err(AdmissionFault::ShotOpportunity)
        ));
        content.admitted.bindings = state.before().bindings;
        let mut origin = state.before().clone();
        origin.weapon.camera.0.x = origin.weapon.camera.0.x.wrapping_add(1);
        content.admitted.weapon.camera = origin.weapon.camera;
        let bytes = member(&sources.native.join(PACKET), "start.record", START_PIN).unwrap();
        let mut record = oracle_record::lsnes::blank();
        oracle_record::lsnes::decode(&bytes, &mut record).unwrap();
        let player = movement::seed(&record.wram).unwrap();
        let data = movement::Content::load(&sources.rom).unwrap();
        let motion = movement::input(&record.wram, player.pose, &data).unwrap();
        let camera = camera::seed(&record.wram, &camera::Content).unwrap();
        let changed = Box::new(Continuation::new(
            origin,
            player,
            camera,
            motion.stimuli(),
            state.batch().clone(),
        ));
        assert!(matches!(
            Production::admit(
                ProductionSeed::ShotOpportunity(changed),
                ProductionContent::ShotOpportunity(content),
                admitted.rules
            ),
            Err(AdmissionFault::ShotOpportunity)
        ));
    }

    fn check_no_fire(case: &Case, sources: &Sources) {
        let mut admitted = admit(case, sources).unwrap();
        let InputReader::ShotOpportunity(inputs) = &mut admitted.inputs else {
            panic!("wrong control scope");
        };
        let OwnedState::ShotOpportunity(start) = admitted.production.state() else {
            panic!("wrong admitted owner");
        };
        let first = inputs
            .advance(&mut admitted.production.clone(), FIRST)
            .unwrap();
        inputs.steps[0].physical_pad &= !start.before().bindings.shoot.0;
        let control = inputs.advance(&mut admitted.production, FIRST).unwrap();
        let (
            ProductionOutputs::ShotOpportunity(first),
            ProductionOutputs::ShotOpportunity(control),
        ) = (first, control)
        else {
            panic!("wrong scoped output");
        };
        assert_eq!(
            first.actors.pose.dependency.pose.aim,
            types::Aim::DownForward
        );
        assert_eq!(
            first.actors.pose.terminal.prospective.action,
            types::Action::Moonwalking
        );
        assert_ne!(
            first.actors.pose.terminal.prospective.action,
            control.actors.pose.terminal.prospective.action
        );
        assert_eq!(control.actors.shots.count.0, 0);
        assert!(first.actors.runs.eq(&control.actors.runs));
        assert!(first.actors.runs[0].sound.is_none() && first.actors.runs[1].sound.is_some());
        assert!(matches!(
            first.actors.doors[1].phase,
            boss_ceres_ridley::Phase::Opening { quarter: boss_ceres_ridley::Quarter::First, wait }
                if wait.frames() == 5
        ));
    }

    fn check_carried_oam_fault(case: &Case, sources: &Sources) {
        let mut admitted = admit(case, sources).unwrap();
        let InputReader::ShotOpportunity(inputs) = &mut admitted.inputs else {
            panic!("wrong drawing control scope");
        };
        inputs.steps[0].phase = beam::FlickerPhase::Even;
        for (cut, field) in [(157, "shot.oam.final"), (158, "shot.oam.pre-power")] {
            assert_eq!(
                runner::execute(
                    case,
                    &admitted.inputs,
                    &admitted.expected,
                    &mut admitted.production,
                    &mut admitted.actual,
                    cut,
                )
                .err(),
                Some(Outcome::Difference {
                    cut,
                    field: field.to_owned()
                }),
            );
        }
    }
}
