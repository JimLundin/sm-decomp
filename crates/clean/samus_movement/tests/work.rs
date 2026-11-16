//! Vector decoding and expected-publication construction. No collision or invented-world logic.
#![cfg(test)]
use core::num::Wrapping;
use serde::Deserialize;
use types::*;

#[test]
fn work_tables_refuse_unknown_keys_and_semantic_names() {
    let motion = r#"motion = { world = "EnemiesThenBlocks", direction = "Right", purpose = "OrdinaryMove", start = [0, 0], radius = [5, 21], intended = 0, accepted = 0, supplied_count = 1, candidates = [{ candidate = 0, centre = [1, 1], radius = [1, 1], result = "Touching", gap = 0 }], result = "EnemyStopped" }"#;
    let support = r#"support = { surface = "Ceiling", centre = [0, 0], radius = [5, 21], cell = [0, 0], result = "PushedDown", placement = 0 }"#;
    let body_fit = r#"body_fit = { old_radius = [5, 21], new_radius = [5, 21], centre = [0, 0], result = "NoGrowth", placement = "None", checks = [] }"#;
    for known in [motion, support, body_fit] {
        assert!(toml::from_str::<ExpectedWork>(known).is_ok());
        let unknown = known.replacen('{', "{ unknown = 1,", 1);
        assert!(toml::from_str::<ExpectedWork>(&unknown).is_err());
        let unknown_result = known
            .replace("EnemyStopped", "IgnoredEnemy")
            .replace("PushedDown", "MissedCeiling")
            .replace("NoGrowth", "Unconsidered");
        assert!(toml::from_str::<ExpectedWork>(&unknown_result).is_err());
    }
    let unknown_box = motion.replace("candidate = 0", "unknown = 1, candidate = 0");
    assert!(toml::from_str::<ExpectedWork>(&unknown_box).is_err());
    let unknown_box_result = motion.replace("Touching", "TooClose");
    assert!(toml::from_str::<ExpectedWork>(&unknown_box_result).is_err());
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
/// Strict decoded work envelope, exactly one publication kind.
pub struct ExpectedWork {
    motion: Option<Request>,
    support: Option<Support>,
    body_fit: Option<Fit>,
}
impl ExpectedWork {
    /// Construct a complete expected publication without running collision decisions.
    /// # Panics
    /// Invalid envelopes or inconsistent request fields are refused at the vector boundary.
    #[must_use]
    pub fn model(&self) -> MovementWork {
        match (&self.motion, &self.support, &self.body_fit) {
            (Some(m), None, None) => MovementWork::Motion(m.model()),
            (None, Some(s), None) => MovementWork::Support(s.model()),
            (None, None, Some(f)) => MovementWork::BodyFit(f.model()),
            _ => panic!("exactly one work kind required"),
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    world: RequestedWorld,
    direction: DirectionName,
    purpose: CollisionPurpose,
    start: [u32; 2],
    radius: [u16; 2],
    intended: i64,
    supplied_count: Option<usize>,
    candidates: Option<Vec<ExpectedBox>>,
    accepted: Option<i64>,
    room: Option<u16>,
    stopped: Option<bool>,
    result: MotionResult,
}
#[derive(Deserialize)]
enum DirectionName {
    Down,
    Left,
    Right,
    Up,
}
impl DirectionName {
    fn model(&self) -> Direction {
        match self {
            Self::Down => Direction::Down,
            Self::Left => Direction::Left,
            Self::Right => Direction::Right,
            Self::Up => Direction::Up,
        }
    }
}
fn position([x, y]: [u32; 2]) -> Position {
    Pair {
        x: Wrapping(x),
        y: Wrapping(y),
    }
}
fn pixels([x, y]: [u16; 2]) -> Pair<Px> {
    Pair {
        x: Wrapping(x),
        y: Wrapping(y),
    }
}
fn travel(n: i64) -> TravelDistance {
    let signed = i32::try_from(n).expect("vector signed travel fits wrapping word");
    TravelDistance(Wrapping(signed.cast_unsigned()))
}
impl Request {
    fn model(&self) -> MotionAttempt {
        if let Some(stopped) = self.stopped {
            assert_eq!(stopped, self.result != MotionResult::Unobstructed);
        }
        let enemy_search = match (&self.supplied_count, &self.candidates, self.world) {
            (None, None, RequestedWorld::Blocks) => None,
            (
                Some(count),
                Some(candidates),
                RequestedWorld::SolidEnemies | RequestedWorld::EnemiesThenBlocks,
            ) => Some(EnemySearch {
                supplied_count: CandidateCount(*count),
                candidates: candidates.iter().map(ExpectedBox::model).collect(),
            }),
            _ => panic!("candidate fields belong only to an actual enemy search"),
        };
        let effect = match (self.accepted, self.room) {
            (Some(a), None) => RequestEffect::Travel(travel(a)),
            (None, Some(r)) => RequestEffect::Clearance(ClearanceRoom(Wrapping(r))),
            _ => panic!("travel and clearance room are distinct quantities"),
        };
        MotionAttempt {
            world: self.world,
            direction: self.direction.model(),
            purpose: self.purpose,
            start: position(self.start),
            radius: pixels(self.radius),
            intended: travel(self.intended),
            enemy_search,
            effect,
            result: self.result,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedBox {
    candidate: usize,
    centre: [u16; 2],
    radius: [u16; 2],
    result: CandidateName,
    gap: Option<i16>,
}
#[derive(Deserialize)]
enum CandidateName {
    Miss,
    AlreadyOverlapping,
    Touching,
    StopsAtGap,
}
impl ExpectedBox {
    fn model(&self) -> EnemyCandidate {
        let gap = || {
            PixelGap(Wrapping(
                self.gap
                    .expect("gap result requires signed gap")
                    .cast_unsigned(),
            ))
        };
        let result = match self.result {
            CandidateName::Miss => {
                assert!(self.gap.is_none());
                CandidateResult::Miss
            }
            CandidateName::AlreadyOverlapping => CandidateResult::AlreadyOverlapping(gap()),
            CandidateName::Touching => {
                assert_eq!(self.gap, Some(0));
                CandidateResult::Touching
            }
            CandidateName::StopsAtGap => CandidateResult::StopsAtGap(gap()),
        };
        EnemyCandidate {
            candidate: CandidateIndex(self.candidate),
            centre: pixels(self.centre),
            radius: pixels(self.radius),
            result,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Support {
    surface: SupportSurface,
    centre: [u32; 2],
    radius: [u16; 2],
    cell: [u16; 2],
    result: SupportResult,
    placement: i16,
}
impl Support {
    fn model(&self) -> SupportCheck {
        let [x, y] = self.cell;
        SupportCheck {
            surface: self.surface,
            centre: position(self.centre),
            radius: pixels(self.radius),
            cell: Cell { x, y },
            result: self.result,
            placement: PlacementDistance(Wrapping(self.placement.cast_unsigned())),
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fit {
    old_radius: [u16; 2],
    new_radius: [u16; 2],
    centre: [u32; 2],
    checks: Option<Vec<Request>>,
    result: BodyFitResult,
    placement: Placement,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum Placement {
    Pixels(i16),
    Absent(Absent),
}
#[derive(Deserialize)]
enum Absent {
    None,
}
impl Fit {
    fn model(&self) -> BodyFitConsideration {
        let placement = match self.placement {
            Placement::Pixels(p) => Some(PlacementDistance(Wrapping(p.cast_unsigned()))),
            Placement::Absent(_) => None,
        };
        BodyFitConsideration {
            old_radius: pixels(self.old_radius),
            new_radius: pixels(self.new_radius),
            centre: position(self.centre),
            checks: self.checks.iter().flatten().map(Request::model).collect(),
            result: self.result,
            placement,
        }
    }
}
