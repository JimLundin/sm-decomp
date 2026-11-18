//! Complete source-independent movement receipt codec; it performs no movement decisions.
use super::{compare::Observation, data_failure, native_search::Kind, result::Outcome};
use serde::{Deserialize, Serialize};
use std::path::Path;
use types::{
    BodyFitResult, CollisionPurpose, Direction, MotionResult, MotionSkipReason, RequestedWorld,
    SupportResult, SupportSurface,
};
pub(super) const PIN: &str = "00fdffb102bbf05fa184010538d66207c4cbf5f43375ecde123886d0e2945443";
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Xy<T> {
    x: T,
    y: T,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Motion {
    #[serde(serialize_with = "named")]
    purpose: CollisionPurpose,
    direction: Direction,
    #[serde(serialize_with = "named")]
    world: RequestedWorld,
    start: Xy<i64>,
    radius: Xy<u16>,
    intended: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    enemy_search: Option<Search>,
    effect: Effect,
    #[serde(serialize_with = "named")]
    result: MotionResult,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Search {
    supplied_count: u64,
    candidates: Vec<Candidate>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Candidate {
    #[serde(rename = "candidate")]
    ordinal: u64,
    centre: Xy<u16>,
    radius: Xy<u16>,
    result: CandidateResult,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    gap: Option<i64>,
}
#[derive(Serialize, Deserialize)]
enum CandidateResult {
    Miss,
    AlreadyOverlapping,
    Touching,
    StopsAtGap,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Effect {
    Travel { distance: i64 },
    Clearance { distance: i64 },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Support {
    #[serde(serialize_with = "named")]
    surface: SupportSurface,
    centre: Xy<i64>,
    radius: Xy<u16>,
    cell: Xy<u16>,
    #[serde(serialize_with = "named")]
    result: SupportResult,
    placement: i64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fit {
    old_radius: Xy<u16>,
    new_radius: Xy<u16>,
    centre: Xy<i64>,
    checks: Vec<Motion>,
    #[serde(serialize_with = "named")]
    result: BodyFitResult,
    placement_present: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    placement: Option<i64>,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Work {
    Motion(Motion),
    Support(Support),
    BodyFit(Fit),
}
#[derive(Serialize, Deserialize)]
enum MotionAtTest {
    ExactlyZero,
    Nonzero,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Test {
    cell: Xy<u16>,
    direction: Direction,
    #[serde(serialize_with = "named")]
    purpose: CollisionPurpose,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    vertical_motion: Option<MotionAtTest>,
}
#[derive(Serialize, Deserialize)]
enum Way {
    Down,
    Left,
    Right,
    Up,
    Look,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Reached {
    cell: Xy<u16>,
    way: Way,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    format: u16,
    id: String,
    source_start: u64,
    source_completion: u64,
    video_frame: u64,
    movement_skip_present: bool,
    #[serde(default)]
    movement_skip: Option<MotionSkipReason>,
    work: Vec<Work>,
    #[serde(default)]
    collision_tests: Vec<Test>,
    #[serde(default)]
    reached: Vec<Reached>,
    inside: Vec<Xy<u16>>,
}
struct Body {
    work: Vec<Work>,
    collision_tests: Vec<Test>,
    reached: Vec<Reached>,
    inside: Vec<Xy<u16>>,
    skip: Option<MotionSkipReason>,
}
fn named<T: core::fmt::Debug, S: serde::Serializer>(
    value: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&format!("{value:?}"))
}
fn count(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}
const fn pixels(value: types::Radius) -> Xy<u16> {
    Xy {
        x: value.x.0,
        y: value.y.0,
    }
}
fn position(value: types::Position) -> Xy<i64> {
    Xy {
        x: i64::from(value.x.0),
        y: i64::from(value.y.0),
    }
}
const fn cell(value: types::Cell) -> Xy<u16> {
    Xy {
        x: value.x,
        y: value.y,
    }
}
fn candidate(value: &types::EnemyCandidate) -> Candidate {
    let (result, gap) = match value.result {
        types::CandidateResult::Miss => (CandidateResult::Miss, None),
        types::CandidateResult::Touching => (CandidateResult::Touching, None),
        types::CandidateResult::AlreadyOverlapping(gap) => (
            CandidateResult::AlreadyOverlapping,
            Some(i64::from(gap.0.0.cast_signed())),
        ),
        types::CandidateResult::StopsAtGap(gap) => (
            CandidateResult::StopsAtGap,
            Some(i64::from(gap.0.0.cast_signed())),
        ),
    };
    Candidate {
        ordinal: count(value.candidate.0),
        centre: pixels(value.centre),
        radius: pixels(value.radius),
        result,
        gap,
    }
}
fn motion(value: &types::MotionAttempt) -> Motion {
    let effect = match value.effect {
        types::RequestEffect::Travel(by) => Effect::Travel {
            distance: i64::from(by.0.0),
        },
        types::RequestEffect::Clearance(room) => Effect::Clearance {
            distance: i64::from(room.0.0),
        },
    };
    Motion {
        purpose: value.purpose,
        direction: value.direction,
        world: value.world,
        start: position(value.start),
        radius: pixels(value.radius),
        intended: i64::from(value.intended.0.0),
        enemy_search: value.enemy_search.as_ref().map(|search| Search {
            supplied_count: count(search.supplied_count.0),
            candidates: search.candidates.iter().map(candidate).collect(),
        }),
        effect,
        result: value.result,
    }
}
fn work(value: &types::MovementWork) -> Work {
    match value {
        types::MovementWork::Motion(value) => Work::Motion(motion(value)),
        types::MovementWork::Support(value) => Work::Support(Support {
            surface: value.surface,
            centre: position(value.centre),
            radius: pixels(value.radius),
            cell: cell(value.cell),
            result: value.result,
            placement: i64::from(value.placement.0.0.cast_signed()),
        }),
        types::MovementWork::BodyFit(value) => Work::BodyFit(Fit {
            old_radius: pixels(value.old_radius),
            new_radius: pixels(value.new_radius),
            centre: position(value.centre),
            checks: value.checks.iter().map(motion).collect(),
            result: value.result,
            placement_present: value.placement.is_some(),
            placement: value.placement.map(|by| i64::from(by.0.0.cast_signed())),
        }),
    }
}
const fn way(value: types::Way) -> Way {
    match value {
        types::Way::Look => Way::Look,
        types::Way::Move(direction) => match direction {
            Direction::Down => Way::Down,
            Direction::Left => Way::Left,
            Direction::Right => Way::Right,
            Direction::Up => Way::Up,
        },
    }
}
fn observation<T: Serialize>(field: &str, value: &T) -> Result<Observation, Outcome> {
    #[derive(Serialize)]
    struct Value<'a, T> {
        value: &'a T,
    }
    let bytes = toml::to_string(&Value { value })
        .map_err(|_| data_failure(None, "complete movement receipt encoding"))?
        .into_bytes();
    Ok(Observation {
        field: field.to_owned(),
        bytes,
    })
}
impl Body {
    fn observations(&self) -> Result<Vec<Observation>, Outcome> {
        Ok(vec![
            observation("movement.work", &self.work)?,
            observation("movement.collision_tests", &self.collision_tests)?,
            observation("movement.reached.complete", &self.reached)?,
            observation("movement.inside", &self.inside)?,
            observation(
                "movement.skip",
                &self.skip.map(|value| format!("{value:?}")),
            )?,
        ])
    }
}
pub(super) fn actual(value: &samus_movement::Outcome) -> Result<Vec<Observation>, Outcome> {
    Body {
        work: value.movement_work.iter().map(work).collect(),
        collision_tests: value
            .collision_tests
            .iter()
            .map(|test| Test {
                cell: cell(test.cell),
                direction: test.direction,
                purpose: test.purpose,
                vertical_motion: test.vertical_motion.map(|value| match value {
                    types::VerticalMotionAtTest::ExactlyZero => MotionAtTest::ExactlyZero,
                    types::VerticalMotionAtTest::Nonzero => MotionAtTest::Nonzero,
                }),
            })
            .collect(),
        reached: value
            .reached
            .iter()
            .map(|touch| Reached {
                cell: cell(touch.cell),
                way: way(touch.way),
            })
            .collect(),
        inside: value.inside.iter().copied().map(cell).collect(),
        skip: value.movement_skip,
    }
    .observations()
}
pub(super) fn initial() -> Result<Vec<Observation>, Outcome> {
    Body {
        work: Vec::new(),
        collision_tests: Vec::new(),
        reached: Vec::new(),
        inside: Vec::new(),
        skip: None,
    }
    .observations()
}
pub(super) fn expected(root: &Path, kind: Kind) -> Result<Vec<Observation>, Outcome> {
    let pins = std::fs::read(root.join("packet-pins.txt"))
        .map_err(|_| data_failure(None, "complete movement expectation manifest"))?;
    if crate::lock::Sha256::try_from(PIN).ok() != Some(crate::lock::Sha256::of(&pins)) {
        return Err(data_failure(
            None,
            "complete movement expectation admission pin",
        ));
    }
    let pins = String::from_utf8(pins)
        .map_err(|_| data_failure(None, "complete movement manifest encoding"))?;
    let (name, completion) = match kind {
        Kind::Miss => ("baseline-expected.toml", 4283),
        Kind::Touch => ("intervention-expected.toml", 4052),
    };
    let bytes = super::native_search::pinned(root, &pins, name)?;
    let text = String::from_utf8(bytes)
        .map_err(|_| data_failure(None, "complete movement expectation encoding"))?;
    let receipt: Receipt = toml::from_str(&text)
        .map_err(|_| data_failure(None, "complete movement expectation schema"))?;
    if receipt.format != 1
        || receipt.id.is_empty()
        || receipt.source_start != 424
        || receipt.source_completion != completion
        || receipt.video_frame != 9233
        || receipt.movement_skip_present != receipt.movement_skip.is_some()
    {
        return Err(data_failure(
            None,
            "complete movement expectation coordinates",
        ));
    }
    Body {
        work: receipt.work,
        collision_tests: receipt.collision_tests,
        reached: receipt.reached,
        inside: receipt.inside,
        skip: receipt.movement_skip,
    }
    .observations()
}

/// Locate a typed receipt difference without exposing any native quantities.
pub(super) fn difference(expected: &[u8], actual: &[u8]) -> Option<String> {
    fn first(expected: &toml::Value, actual: &toml::Value, path: String) -> Option<String> {
        match (expected, actual) {
            (toml::Value::Array(expected), toml::Value::Array(actual)) => {
                if expected.len() != actual.len() {
                    return Some(format!("{path}.count"));
                }
                expected
                    .iter()
                    .zip(actual)
                    .enumerate()
                    .find_map(|(index, (expected, actual))| {
                        first(expected, actual, format!("{path}[{index}]"))
                    })
            }
            (toml::Value::Table(expected), toml::Value::Table(actual)) => {
                for (key, expected) in expected {
                    let Some(actual) = actual.get(key) else {
                        return Some(format!("{path}.{key}.missing"));
                    };
                    if let Some(path) = first(expected, actual, format!("{path}.{key}")) {
                        return Some(path);
                    }
                }
                actual
                    .keys()
                    .find(|key| !expected.contains_key(*key))
                    .map(|key| format!("{path}.{key}.extra"))
            }
            (expected, actual) => (expected != actual).then_some(path),
        }
    }
    let expected: toml::Value = toml::from_str(std::str::from_utf8(expected).ok()?).ok()?;
    let actual: toml::Value = toml::from_str(std::str::from_utf8(actual).ok()?).ok()?;
    first(&expected, &actual, String::new())
        .map(|path| path.strip_prefix(".value").unwrap_or(&path).to_owned())
}
