//! Independently recorded movement and appearance during the flyby shove.

use super::{ceres_doors, open_rows, read_lock, samus, word};
use crate::replay::{
    Sources, appearance,
    compare::Observation,
    compared, connected, data_failure,
    index::{
        Case, CaseKind, ContentProvider, ExpectedProvider, ObservationId as O, Responsibility,
        Start,
    },
    movement,
    readers::ExpectedCut,
    result::{CaseResult, Outcome},
    runner,
};
use crate::source::WramWord;
use core::num::Wrapping;
use game::production::connected::inputs::Inputs;
use game::production::played;
use game::production::played::samus::{GetawayInputs, GetawayPass, TurnContent};
use samus_movement::Getaway;
use types::{Parts, Side};

const STATE: &[O] = &[
    O::GetawayPhase,
    O::MovementPose,
    O::PositionX,
    O::PositionY,
    O::MovementBaseSpeed,
    O::VerticalSpeed,
    O::GetawayLatestPose,
    O::GetawayOlderPose,
    O::GetawayBodyX,
    O::GetawayBodyY,
    O::AppearanceStep,
    O::AppearanceTimeLeft,
];
const OUTPUTS: &[O] = &[O::TravelLeft, O::TravelRight, O::TravelUp, O::TravelDown];

pub(in crate::replay) fn valid(case: &Case) -> bool {
    case.kind == CaseKind::Scoped
        && matches!(case.start, Start::CacheCut { .. })
        && case.content == ContentProvider::PlayerRomImport
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && case.expectations.fields == STATE
        && case.expectations.outputs == OUTPUTS
        && case.expectations.output_cut.is_none()
        && case.responsibilities == [Responsibility::Movement, Responsibility::Appearance]
        && case.checkpoints
}

fn field(wram: &[u8], name: &'static str, offset: u16) -> Result<u16, Outcome> {
    word(wram, WramWord { name, offset })
}

struct Opportunity {
    samus: Inputs,
    getaway: GetawayInputs,
    doors: Vec<boss_ceres_ridley::CeresDoor>,
}

fn input(wram: &[u8], content: &movement::Content) -> Result<Opportunity, Outcome> {
    let borrowed = movement::borrowed(wram, wram, content)?;
    if borrowed.cinematic || borrowed.elevator {
        return Err(data_failure(None, "scoped dry room movement"));
    }
    let mut inputs = connected::seed_inputs(wram, borrowed.items, borrowed.world)?;
    inputs.held = borrowed.held;
    inputs.pressed = borrowed.pressed;
    inputs.parity = borrowed.parity;
    // The completed picture cursor is a boss fact. The pass precedes path selection in
    // the next room turn, so it is independent of Samus's handler and that turn's result.
    let flying = field(wram, "Ridley flyby mode", 0x0fa8)? == 0xaa4f;
    let picture_cursor = field(wram, "Ridley flyby picture cursor", 0x8026)?;
    let slowdown = u8::try_from(field(wram, "Samus horizontal slowdown", 0x0a66)?)
        .ok()
        .and_then(samus_movement::Slowdown::new)
        .ok_or_else(|| data_failure(None, "Samus horizontal slowdown range"))?;
    let carry = types::fixed(
        Wrapping(field(wram, "Samus horizontal room carry", 0x0b58)?),
        field(wram, "Samus horizontal room carry fraction", 0x0b56)?,
    );
    Ok(Opportunity {
        samus: inputs,
        getaway: GetawayInputs {
            view_left: Wrapping(field(wram, "front layer view left", 0x0911)?),
            pass: (flying && picture_cursor == 2 * 104).then_some(GetawayPass),
            slowdown,
            carry,
        },
        doors: ceres_doors::seed(wram)?,
    })
}

fn phase(mode: Getaway) -> Observation {
    Observation::quantity(
        "movement.getaway-phase",
        match mode {
            Getaway::Idle => 0,
            Getaway::Pending => 1,
            Getaway::Travelling(Side::Left) => 2,
            Getaway::Travelling(Side::Right) => 3,
        },
    )
}

fn appearance_fields(fields: Vec<Observation>) -> impl Iterator<Item = Observation> {
    fields.into_iter().filter(|obs| {
        matches!(
            obs.field.as_str(),
            "appearance.step" | "appearance.time-left"
        )
    })
}

const LATEST: &str = "movement.getaway-latest-pose";
const OLDER: &str = "movement.getaway-older-pose";

fn recorded_pose(wram: &[u8], field: WramWord, label: &str) -> Result<Observation, Outcome> {
    let id = word(wram, field)?;
    let pose = data::assets::samus::map::key_of(data::pose::PoseId(id))
        .ok_or_else(|| data_failure(None, "getaway recorded pose"))?;
    Ok(movement::pose_observation(label, pose))
}

const BODY_X: &str = "movement.getaway-body-x";
const BODY_Y: &str = "movement.getaway-body-y";

fn body_fields(x: u16, y: u16) -> [Observation; 2] {
    [
        Observation::quantity(BODY_X, i64::from(x)),
        Observation::quantity(BODY_Y, i64::from(y)),
    ]
}

fn expected(
    case: &Case,
    cut: u64,
    wram: &[u8],
    content: &appearance::Content,
) -> Result<ExpectedCut, Outcome> {
    let mut semantics = vec![phase(movement::getaway(wram)?)];
    semantics.extend(movement::expected(
        wram,
        &[
            O::MovementPose,
            O::PositionX,
            O::PositionY,
            O::MovementBaseSpeed,
        ],
        &case.row,
        cut,
    )?);
    let magnitude = types::fixed(
        Wrapping(word(wram, crate::source::SAMUS_Y_SPEED)?),
        word(wram, crate::source::SAMUS_Y_SUBSPEED)?,
    )
    .size();
    semantics.push(Observation::quantity(
        "movement.vertical_speed",
        i64::from(magnitude.0),
    ));
    semantics.push(recorded_pose(wram, crate::source::SAMUS_PREV_POSE, LATEST)?);
    semantics.push(recorded_pose(
        wram,
        crate::source::SAMUS_LAST_DIFFERENT_POSE,
        OLDER,
    )?);
    semantics.extend(body_fields(
        field(wram, "Samus sampled half-width", 0x0afe)?,
        field(wram, "Samus sampled half-height", 0x0b00)?,
    ));
    let (animation, _) = appearance::animation_owner(wram, content)?;
    semantics.extend(appearance_fields(appearance::initial(animation, content)));
    let mut fields = semantics.clone();
    fields.extend(movement::expected(wram, OUTPUTS, &case.row, cut)?);
    Ok(ExpectedCut {
        fields,
        semantic_count: semantics.len(),
        semantics,
        output_count: OUTPUTS.len(),
        replies: 0,
    })
}

fn actual(
    state: &played::samus::State,
    content: &appearance::Content,
    radius: types::Radius,
) -> Vec<Observation> {
    let her = &state.movement.0;
    let speed = her
        .getaway
        .downward_speed()
        .unwrap_or_else(|| her.vertical.speed.size());
    let mut fields = vec![
        phase(her.getaway),
        movement::movement_pose(her.pose),
        Observation::quantity("movement.position.x", i64::from(her.position.x.0)),
        Observation::quantity("movement.position.y", i64::from(her.position.y.0)),
        Observation::quantity("movement.base-speed", i64::from(her.run.speed.0)),
        Observation::quantity("movement.vertical_speed", i64::from(speed.0)),
    ];
    fields.push(movement::pose_observation(LATEST, her.choices.latest));
    fields.push(her.choices.before.map_or_else(
        || Observation {
            field: OLDER.to_owned(),
            bytes: Vec::new(),
        },
        |pose| movement::pose_observation(OLDER, pose),
    ));
    fields.extend(body_fields(radius.x.0, radius.y.0));
    fields.extend(appearance_fields(appearance::actual(
        &state.appearance.0,
        content,
    )));
    fields.extend(
        state
            .travelled
            .0
            .travels
            .iter()
            .map(|(direction, distance)| {
                Observation::quantity(
                    format!("movement.travel.{direction:?}"),
                    i64::from(distance.0.0),
                )
            }),
    );
    fields
}

fn advance(
    state: &mut played::samus::State,
    input: &Opportunity,
    content: TurnContent<'_>,
    cut: u64,
) -> Result<samus_movement::Outcome, Outcome> {
    state
        .step_getaway(&input.samus, input.getaway, &input.doors, content)
        .map_err(|fault| Outcome::Unsupported {
            cut: Some(cut),
            dependency: format!("scoped getaway {fault:?}"),
        })
}

pub(in crate::replay) fn run(
    case: &Case,
    sources: &Sources,
    result: &mut CaseResult,
) -> Result<(), Outcome> {
    let Start::CacheCut { frame: start } = case.start else {
        return Err(data_failure(None, "scoped getaway start"));
    };
    let lock = read_lock(sources)?;
    let mut inputs = open_rows(sources, &lock, &case.row)?;
    let mut records = open_rows(sources, &lock, &case.row)?;
    let movement = movement::Content::load(&sources.rom)?;
    let appearance = appearance::Content::load(&sources.rom)?;
    let movement_rules = samus_movement::Rules::default();
    let appearance_rules = samus_appearance::Rules::VANILLA;
    let content = TurnContent {
        movement: &movement.movement,
        appearance: &appearance.appearance,
        movement_rules: &movement_rules,
        appearance_rules: &appearance_rules,
    };
    let seed = inputs.record(start)?;
    let mut state = samus::seed(&seed.wram, &appearance)?;
    if movement::getaway(&seed.wram)? != Getaway::Idle {
        return Err(data_failure(Some(start), "getaway seed before arming"));
    }
    canonical::model::pin_holds::<game::state::owned::MovementPin>()
        .and_then(|()| canonical::model::pin_holds::<played::samus::TravelPin>())
        .map_err(|_| Outcome::CheckpointFailure { cut: start })?;
    let radius = |state: &played::samus::State| {
        movement
            .movement
            .radius(state.movement.0.pose)
            .ok_or_else(|| data_failure(None, "getaway pose body"))
    };
    let mut digest = Vec::new();
    let mut restored = Vec::new();
    for cut in start.saturating_add(1)..=case.end.cut_inclusive {
        let prior = cut.saturating_sub(1);
        let saved = canonical::to_bytes(&state);
        restored.push(
            canonical::from_bytes(&saved).map_err(|_| Outcome::CheckpointFailure { cut: prior })?,
        );
        let input = input(&inputs.record(prior)?.wram, &movement)?;
        let body = radius(&state)?;
        let null = actual(&state, &appearance, body);
        let publication = advance(&mut state, &input, content, cut)?;
        let expected = expected(case, cut, &records.record(cut)?.wram, &appearance)?;
        compared(cut, &expected.fields, &actual(&state, &appearance, body))?;
        runner::record_cut(result, &mut digest, &expected, &null, cut)?;
        for continuation in &mut restored {
            let output = advance(continuation, &input, content, cut)?;
            if *continuation != state || output != publication {
                return Err(Outcome::CheckpointFailure { cut });
            }
        }
    }
    result.counts.restorations = restored.len();
    result.observation_sha256 = crate::lock::Sha256::of(&digest).to_string();
    Ok(())
}
