//! Independently admitted within-video movement inputs and bounded first-search observations.

use super::{compare::Observation, data_failure, movement, result::Outcome};
use crate::{lock::Sha256, source};
use game::production::{AdvanceInput, Production, ProductionOutputs};
use game::state::owned::OwnedState;
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};
use types::{Block, Cell, Hitbox, Way, World};

pub(super) const COMPLETION: u64 = 9233;
pub(super) const PACKET_PIN: &str =
    "2adee318fb765d39da0fcb85a16a326de3c84522946df2b37811fe52782a880c";
pub(super) const TOUCH_PACKET_PIN: &str =
    "e6e41959d1d4ee9b9f8c811642849a18bc23a6a1df91f0acbedda613d2b1de73";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Miss,
    Touch,
}
impl Kind {
    pub const fn pin(self) -> &'static str {
        match self {
            Self::Miss => PACKET_PIN,
            Self::Touch => TOUCH_PACKET_PIN,
        }
    }
}
const PREFIX: &str = "movement.motion.0";

pub(super) struct Inputs {
    pub seed: samus_movement::Samus,
    pub content: movement::Content,
    input: movement::Input,
    width: u16,
    height: u16,
}

pub(super) struct Expected {
    root: PathBuf,
    pins: String,
    kind: Kind,
    pub work_root: PathBuf,
}

pub(super) fn pinned(root: &Path, pins: &str, name: &str) -> Result<Vec<u8>, Outcome> {
    let wanted = pins
        .lines()
        .filter_map(|line| line.split_once("  "))
        .find_map(|(hash, path)| (path == name).then_some(hash))
        .ok_or_else(|| data_failure(None, "native search member declaration"))?;
    let bytes = std::fs::read(root.join(name))
        .map_err(|_| data_failure(None, "native search member file"))?;
    if Sha256::try_from(wanted).ok() != Some(Sha256::of(&bytes)) {
        return Err(data_failure(None, "native search member pin"));
    }
    Ok(bytes)
}

pub(super) fn load(root: &Path, rom: &Path, kind: Kind) -> Result<(Inputs, Expected), Outcome> {
    let pins = std::fs::read(root.join("packet-pins.txt"))
        .map_err(|_| data_failure(None, "native search packet manifest"))?;
    if Sha256::try_from(kind.pin()).ok() != Some(Sha256::of(&pins)) {
        return Err(data_failure(None, "native search admission pin"));
    }
    let pins = String::from_utf8(pins)
        .map_err(|_| data_failure(None, "native search manifest encoding"))?;
    // These exact images have independent full-snapshot association and writer closure.
    // Expected files are deliberately unopened until after production has advanced.
    let frame = pinned(root, &pins, "inputs/frame.bin")?;
    let world = pinned(root, &pins, "inputs/world.bin")?;
    let seed_bytes = pinned(root, &pins, "inputs/seed.bin")?;
    if [frame.len(), world.len(), seed_bytes.len()] != [131_072; 3] {
        return Err(data_failure(None, "native search complete images"));
    }
    let width = source::ROOM_WIDTH_IN_BLOCKS
        .read(&world)
        .map_err(|_| data_failure(None, "native search width"))?;
    let height = source::ROOM_HEIGHT_IN_BLOCKS
        .read(&world)
        .map_err(|_| data_failure(None, "native search height"))?;
    let content = movement::Content::load(rom)?;
    let seed = movement::seed(&seed_bytes)?;
    // Reject unavailable typed dependencies at admission without executing gameplay.
    let input = movement::input_at(&frame, &world, seed.pose, &content)?;
    Ok((
        Inputs {
            seed,
            content,
            input,
            width,
            height,
        },
        Expected {
            root: root.to_owned(),
            pins,
            kind,
            work_root: root.parent().unwrap_or(root).join("native-search-work-v2"),
        },
    ))
}

struct Guard<'a> {
    world: &'a game::production::world::MovementWorld,
    width: u16,
    height: u16,
    outside: RefCell<Option<Cell>>,
}
impl Guard<'_> {
    fn qualify(&self, at: Cell) {
        if (at.x >= self.width || at.y >= self.height) && self.outside.borrow().is_none() {
            *self.outside.borrow_mut() = Some(at);
        }
    }
}
impl World for Guard<'_> {
    fn block(&self, at: Cell) -> Block {
        self.qualify(at);
        self.world.block(at)
    }
    fn lets_through(&self, at: Cell, way: Way) -> bool {
        self.qualify(at);
        self.world.lets_through(at, way)
    }
    fn solid_enemies(&self) -> &[Hitbox] {
        self.world.solid_enemies()
    }
}

impl Inputs {
    pub fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        if cut != COMPLETION {
            return Err(data_failure(
                Some(cut),
                "native search opportunity schedule",
            ));
        }
        let OwnedState::Movement(state) = production.state() else {
            return Err(data_failure(Some(cut), "native search owned scope"));
        };
        let initial_pose = state.0.pose;
        let inputs = &self.input;
        let guard = Guard {
            world: &inputs.world,
            width: self.width,
            height: self.height,
            outside: RefCell::new(None),
        };
        // World cannot return a typed refusal. Keep the entire production candidate private
        // until every actual query is admitted, committing state and complete outputs together.
        let mut candidate = production.clone();
        let outputs = candidate
            .advance(AdvanceInput::Movement {
                inputs: inputs.stimuli(),
                world: &guard,
            })
            .map_err(|fault| {
                movement::advance_failure(cut, initial_pose, "native search body content", fault)
            })?;
        if guard.outside.borrow().is_some() {
            return Err(Outcome::Unsupported {
                cut: Some(cut),
                dependency: "native movement query outside independently admitted grid".to_owned(),
            });
        }
        *production = candidate;
        Ok(outputs)
    }
}

fn quantity(field: &str, value: i64) -> Observation {
    Observation::quantity(format!("{PREFIX}.{field}"), value)
}
fn name(field: &str, value: &str) -> Observation {
    Observation {
        field: format!("{PREFIX}.{field}"),
        bytes: value.as_bytes().to_vec(),
    }
}
fn count_bytes(count: usize) -> [u8; 8] {
    u64::try_from(count).unwrap_or(u64::MAX).to_le_bytes()
}
fn candidates_bytes(candidates: &[types::EnemyCandidate]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&count_bytes(candidates.len()));
    for candidate in candidates {
        bytes.extend_from_slice(&count_bytes(candidate.candidate.0));
        for value in [
            candidate.centre.x,
            candidate.centre.y,
            candidate.radius.x,
            candidate.radius.y,
        ] {
            bytes.extend_from_slice(&value.0.to_le_bytes());
        }
        let (result, gap) = match candidate.result {
            types::CandidateResult::Miss => (0_u8, 0),
            types::CandidateResult::AlreadyOverlapping(gap) => (1, gap.0.0),
            types::CandidateResult::Touching => (2, 0),
            types::CandidateResult::StopsAtGap(gap) => (3, gap.0.0),
        };
        bytes.push(result);
        bytes.extend_from_slice(&gap.to_le_bytes());
    }
    bytes
}

pub(super) fn actual(outputs: &samus_movement::Outcome) -> Vec<Observation> {
    let motion = outputs.movement_work.iter().find_map(|work| match work {
        types::MovementWork::Motion(motion) => Some(motion),
        types::MovementWork::Support(_) | types::MovementWork::BodyFit(_) => None,
    });
    let Some(motion) = motion else {
        return initial();
    };
    let mut fields = vec![
        quantity("present", 1),
        name("purpose", &format!("{:?}", motion.purpose)),
        name("direction", &format!("{:?}", motion.direction)),
        name("world", &format!("{:?}", motion.world)),
        quantity("start.x", i64::from(motion.start.x.0)),
        quantity("start.y", i64::from(motion.start.y.0)),
        quantity("radius.x", i64::from(motion.radius.x.0)),
        quantity("radius.y", i64::from(motion.radius.y.0)),
        quantity("intended", i64::from(motion.intended.0.0)),
    ];
    match motion.effect {
        types::RequestEffect::Travel(by) => {
            fields.push(name("effect", "Travel"));
            fields.push(quantity("accepted", i64::from(by.0.0)));
        }
        types::RequestEffect::Clearance(room) => {
            fields.push(name("effect", "Clearance"));
            fields.push(quantity("accepted", i64::from(room.0.0)));
        }
    }
    fields.push(name("result", &format!("{:?}", motion.result)));
    fields.push(quantity(
        "enemy_search.present",
        i64::from(motion.enemy_search.is_some()),
    ));
    if let Some(search) = &motion.enemy_search {
        fields.push(quantity(
            "enemy_search.supplied_count",
            i64::try_from(search.supplied_count.0).unwrap_or(i64::MAX),
        ));
        fields.push(Observation {
            field: format!("{PREFIX}.enemy_search.candidates"),
            bytes: candidates_bytes(&search.candidates),
        });
    }
    let mut observed = vec![ordinary_right_terrain(outputs)];
    observed.extend(grouped(fields));
    observed
}
fn ordinary_right_terrain(outputs: &samus_movement::Outcome) -> Observation {
    let tests: Vec<_> = outputs
        .collision_tests
        .iter()
        .filter(|test| {
            test.purpose == types::CollisionPurpose::OrdinaryMove
                && test.direction == types::Direction::Right
        })
        .collect();
    let mut bytes = count_bytes(tests.len()).to_vec();
    for test in tests {
        bytes.extend_from_slice(&test.cell.x.to_le_bytes());
        bytes.extend_from_slice(&test.cell.y.to_le_bytes());
        let vertical_motion = match test.vertical_motion {
            None => 0_u8,
            Some(types::VerticalMotionAtTest::ExactlyZero) => 1,
            Some(types::VerticalMotionAtTest::Nonzero) => 2,
        };
        bytes.push(vertical_motion);
    }
    Observation {
        field: format!("{PREFIX}.ordinary_terrain_tests"),
        bytes,
    }
}
fn grouped(mut fields: Vec<Observation>) -> Vec<Observation> {
    let candidate_field = format!("{PREFIX}.enemy_search.candidates");
    let candidates = fields
        .iter()
        .position(|field| field.field == candidate_field)
        .map(|index| fields.remove(index))
        .unwrap_or(Observation {
            field: candidate_field,
            bytes: Vec::new(),
        });
    let mut bytes = Vec::new();
    for field in fields {
        bytes.extend_from_slice(&count_bytes(field.field.len()));
        bytes.extend_from_slice(field.field.as_bytes());
        bytes.extend_from_slice(&count_bytes(field.bytes.len()));
        bytes.extend_from_slice(&field.bytes);
    }
    vec![
        Observation {
            field: format!("{PREFIX}.request"),
            bytes,
        },
        candidates,
    ]
}
pub(super) fn initial() -> Vec<Observation> {
    vec![
        Observation {
            field: format!("{PREFIX}.ordinary_terrain_tests"),
            bytes: count_bytes(0).to_vec(),
        },
        Observation {
            field: format!("{PREFIX}.request"),
            bytes: Vec::new(),
        },
        Observation {
            field: format!("{PREFIX}.enemy_search.candidates"),
            bytes: candidates_bytes(&[]),
        },
    ]
}

fn table(bytes: Vec<u8>) -> Result<toml::Value, Outcome> {
    let text = String::from_utf8(bytes)
        .map_err(|_| data_failure(None, "native search expectation encoding"))?;
    toml::from_str(&text).map_err(|_| data_failure(None, "native search expectation schema"))
}
fn number(value: &toml::Value, key: &str) -> Result<i64, Outcome> {
    value
        .get(key)
        .and_then(toml::Value::as_integer)
        .ok_or_else(|| data_failure(None, "native search expectation quantity"))
}
fn text<'a>(value: &'a toml::Value, key: &str) -> Result<&'a str, Outcome> {
    value
        .get(key)
        .and_then(toml::Value::as_str)
        .ok_or_else(|| data_failure(None, "native search expectation name"))
}
fn position(value: &toml::Value, pixel: &str, subpixel: &str) -> Result<i64, Outcome> {
    number(value, pixel)?
        .checked_mul(65536)
        .and_then(|whole| {
            number(value, subpixel)
                .ok()
                .and_then(|sub| whole.checked_add(sub))
        })
        .ok_or_else(|| data_failure(None, "native search position range"))
}
impl Expected {
    pub fn read(&self, cut: u64) -> Result<Vec<Observation>, Outcome> {
        let mut fields = match self.kind {
            Kind::Miss => self.read_miss(cut)?,
            Kind::Touch => self.read_touch(cut)?,
        };
        fields.extend(super::full_movement::expected(&self.work_root, self.kind)?);
        Ok(fields)
    }
    fn read_miss(&self, cut: u64) -> Result<Vec<Observation>, Outcome> {
        if cut != COMPLETION {
            return Err(data_failure(
                Some(cut),
                "native search expectation opportunity",
            ));
        }
        let complete = table(pinned(
            &self.root,
            &self.pins,
            "expectations/completed-request.toml",
        )?)?;
        let enemy = table(pinned(
            &self.root,
            &self.pins,
            "expectations/enemy-stage.toml",
        )?)?;
        if number(&complete, "video_frame")? != i64::try_from(COMPLETION).unwrap_or(i64::MAX)
            || number(&complete, "complete_step_source_start_index")? != 424
            || number(&complete, "complete_step_source_completion_index")? != 4283
            || number(&complete, "request_ordinal_among_motion_attempts")? != 0
            || number(&complete, "source_wrapper_index")? != 2392
            || number(&complete, "source_enemy_entry_index")? != 2395
            || number(&complete, "supplied_eligible_count")? != 1
            || number(&complete, "considered_candidates")? != 1
            || text(&complete, "candidate_result")? != "Miss"
        {
            return Err(data_failure(
                Some(cut),
                "native search admitted expectation association",
            ));
        }
        let player = enemy
            .get("player")
            .ok_or_else(|| data_failure(None, "native search player expectation"))?;
        let actors = enemy
            .get("actors")
            .and_then(toml::Value::as_array)
            .ok_or_else(|| data_failure(None, "native search ordered candidate expectations"))?;
        let mut candidate_bytes = Vec::new();
        candidate_bytes.extend_from_slice(&count_bytes(actors.len()));
        for (ordinal, actor) in actors.iter().enumerate() {
            candidate_bytes.extend_from_slice(&count_bytes(ordinal));
            for key in ["centre_x", "centre_y", "radius_x", "radius_y"] {
                let value = u16::try_from(number(actor, key)?)
                    .map_err(|_| data_failure(None, "native search candidate quantity range"))?;
                candidate_bytes.extend_from_slice(&value.to_le_bytes());
            }
            if text(actor, "result")? != "SeparatedOnX" {
                return Err(data_failure(
                    None,
                    "native search admitted candidate result",
                ));
            }
            candidate_bytes.push(0);
            candidate_bytes.extend_from_slice(&0_u16.to_le_bytes());
        }
        Ok(grouped(vec![
            quantity("present", 1),
            name("purpose", text(&complete, "purpose")?),
            name("direction", text(&complete, "direction")?),
            name("world", text(&complete, "requested_world")?),
            quantity("start.x", position(player, "centre_x", "subpixel_x")?),
            quantity("start.y", position(player, "centre_y", "subpixel_y")?),
            quantity("radius.x", number(player, "radius_x")?),
            quantity("radius.y", number(player, "radius_y")?),
            quantity("intended", number(&complete, "intended_fixed_16_16")?),
            name("effect", text(&complete, "effect")?),
            quantity("accepted", number(&complete, "accepted_fixed_16_16")?),
            name("result", text(&complete, "result")?),
            quantity("enemy_search.present", 1),
            quantity(
                "enemy_search.supplied_count",
                number(&complete, "supplied_eligible_count")?,
            ),
            Observation {
                field: format!("{PREFIX}.enemy_search.candidates"),
                bytes: candidate_bytes,
            },
        ]))
    }

    fn read_touch(&self, cut: u64) -> Result<Vec<Observation>, Outcome> {
        if cut != COMPLETION {
            return Err(data_failure(
                Some(cut),
                "native Touch expectation opportunity",
            ));
        }
        let expected = table(pinned(
            &self.root,
            &self.pins,
            "expectations/completed-request.toml",
        )?)?;
        for (field, wanted) in [
            ("video_frame", 9233),
            ("source_start_index", 424),
            ("source_completion_index", 4052),
            ("wrapper_start", 2418),
            ("enemy_start", 2421),
            ("motion_ordinal", 0),
            ("supplied_count", 1),
            ("candidate_count", 1),
            ("ordinary_right_terrain_tests", 0),
        ] {
            if number(&expected, field)? != wanted {
                return Err(data_failure(
                    Some(cut),
                    "native Touch expectation association",
                ));
            }
        }
        let candidates = expected
            .get("candidates")
            .and_then(toml::Value::as_array)
            .ok_or_else(|| data_failure(None, "native Touch candidate expectations"))?;
        let mut bytes = count_bytes(candidates.len()).to_vec();
        for candidate in candidates {
            let ordinal = usize::try_from(number(candidate, "candidate")?)
                .map_err(|_| data_failure(None, "native Touch candidate identity"))?;
            bytes.extend_from_slice(&count_bytes(ordinal));
            for field in ["centre_x", "centre_y", "half_width", "half_height"] {
                let value = u16::try_from(number(candidate, field)?)
                    .map_err(|_| data_failure(None, "native Touch candidate geometry"))?;
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            if text(candidate, "result")? != "Touching" {
                return Err(data_failure(None, "native Touch candidate result"));
            }
            bytes.push(2);
            bytes.extend_from_slice(&0_u16.to_le_bytes());
        }
        let mut fields = vec![Observation {
            field: format!("{PREFIX}.ordinary_terrain_tests"),
            bytes: count_bytes(0).to_vec(),
        }];
        fields.extend(grouped(vec![
            quantity("present", 1),
            name("purpose", text(&expected, "purpose")?),
            name("direction", text(&expected, "direction")?),
            name("world", text(&expected, "world")?),
            quantity("start.x", number(&expected, "start_x_fixed")?),
            quantity("start.y", number(&expected, "start_y_fixed")?),
            quantity("radius.x", number(&expected, "body_half_width")?),
            quantity("radius.y", number(&expected, "body_half_height")?),
            quantity("intended", number(&expected, "intended_fixed")?),
            name("effect", text(&expected, "effect")?),
            quantity("accepted", number(&expected, "accepted_fixed")?),
            name("result", text(&expected, "result")?),
            quantity("enemy_search.present", 1),
            quantity(
                "enemy_search.supplied_count",
                number(&expected, "supplied_count")?,
            ),
            Observation {
                field: format!("{PREFIX}.enemy_search.candidates"),
                bytes,
            },
        ]));
        Ok(fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::{
        Sources, admission, compare,
        index::{Index, Start},
        readers::ActualReader,
        runner,
    };
    use game::production::{ProductionContent, ProductionRules, ProductionSeed};

    fn source() -> (PathBuf, PathBuf) {
        (
            PathBuf::from(std::env::var("SM_SEARCH_PACKET").unwrap()),
            PathBuf::from(std::env::var("SM_ROM").unwrap()),
        )
    }
    fn named_case(packet: &Path, id: &str) -> crate::replay::index::Case {
        let index = Index::parse(include_str!("../../../../scenarios/replay/index.toml")).unwrap();
        let mut case = index
            .cases
            .into_iter()
            .find(|case| case.id.0 == id)
            .unwrap();
        assert!(matches!(case.start, Start::NativeOpportunity { .. }));
        if let Start::NativeOpportunity { path, .. } = &mut case.start {
            *path = packet.to_owned();
        }
        case
    }
    fn native_case(packet: &Path) -> crate::replay::index::Case {
        named_case(packet, "movement-native-occupied-first-search")
    }
    fn retained(actual: &ActualReader) -> Option<&samus_movement::Outcome> {
        if let ActualReader::NativeSearch(Some(output)) = actual {
            Some(output)
        } else {
            None
        }
    }
    fn copy(packet: &Path, id: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("search-input-contract-{}-{id}", std::process::id()));
        std::fs::create_dir_all(root.join("inputs")).unwrap();
        std::fs::create_dir_all(root.join("expectations")).unwrap();
        for file in [
            "packet-pins.txt",
            "inputs/seed.bin",
            "inputs/frame.bin",
            "inputs/world.bin",
            "expectations/completed-request.toml",
        ] {
            std::fs::copy(packet.join(file), root.join(file)).unwrap();
        }
        let enemy = packet.join("expectations/enemy-stage.toml");
        if enemy.exists() {
            std::fs::copy(enemy, root.join("expectations/enemy-stage.toml")).unwrap();
        }
        root
    }
    #[test]
    #[ignore = "private independently pinned native search packet"]
    fn native_search_contract() {
        let (packet, rom) = source();
        let case = native_case(&packet);
        let result = crate::replay::run(
            &case,
            &Sources {
                rom,
                ..Sources::default()
            },
        );
        assert_eq!(result.outcome, Outcome::Pass);
        assert_eq!(result.counts.cuts, 1);
        assert_eq!(result.counts.fields, 7);
        assert_eq!(result.counts.null_cuts, 0);
        assert_eq!(result.counts.null_fields, 1);
        assert_eq!(result.counts.restorations, 1);
        std::fs::write(
            std::env::var("SM_SEARCH_RECEIPT").unwrap(),
            toml::to_string(&result).unwrap(),
        )
        .unwrap();
    }
    fn expected_failure(packet: &Path, rom: PathBuf, case_id: &str, complete_name: &str) {
        let case = named_case(packet, case_id);
        let sources = Sources {
            rom,
            native: packet.parent().unwrap().to_owned(),
            ..Sources::default()
        };
        let mut baseline = admission::admit(&case, &sources).unwrap();
        runner::execute(
            &case,
            &baseline.inputs,
            &baseline.expected,
            &mut baseline.production,
            &mut baseline.actual,
            COMPLETION,
        )
        .unwrap();
        for (id, remove, complete) in [
            ("missing", true, false),
            ("altered", false, false),
            ("complete-missing", true, true),
            ("complete-altered", false, true),
        ] {
            let root = copy(packet, &format!("{case_id}-{id}"));
            let case = named_case(&root, case_id);
            let work_root = root.join("native-search-work-v2");
            std::fs::create_dir_all(&work_root).unwrap();
            for name in [
                "packet-pins.txt",
                "baseline-expected.toml",
                "intervention-expected.toml",
            ] {
                std::fs::copy(
                    sources.native.join("native-search-work-v2").join(name),
                    work_root.join(name),
                )
                .unwrap();
            }
            let mutation_sources = Sources {
                rom: sources.rom.clone(),
                native: root.clone(),
                ..Sources::default()
            };
            // Every failure exists before admission and the real production execution.
            let path = if complete {
                work_root.join(complete_name)
            } else {
                root.join("expectations/completed-request.toml")
            };
            if remove {
                std::fs::remove_file(path).unwrap();
            } else {
                std::fs::write(path, "format = 0\n").unwrap();
            }
            let mut admitted = admission::admit(&case, &mutation_sources).unwrap();
            let failure = runner::execute(
                &case,
                &admitted.inputs,
                &admitted.expected,
                &mut admitted.production,
                &mut admitted.actual,
                COMPLETION,
            )
            .err()
            .unwrap();
            assert!(matches!(
                failure,
                Outcome::DataFailure {
                    cut: Some(COMPLETION),
                    ..
                }
            ));
            assert!(
                admitted.production == baseline.production,
                "complete native production equality"
            );
            assert!(
                retained(&admitted.actual) == retained(&baseline.actual),
                "complete native output equality"
            );
            assert!(retained(&admitted.actual).is_some());
            std::fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    #[ignore = "private independently pinned native search packet"]
    fn native_search_expected_failure_preserves_complete_executed_state_and_output() {
        let (packet, rom) = source();
        expected_failure(
            &packet,
            rom,
            "movement-native-occupied-first-search",
            "baseline-expected.toml",
        );
    }
    #[test]
    #[ignore = "private independently admitted native Touch packet"]
    fn native_touch_expected_failure_preserves_complete_executed_state_and_output() {
        let packet = PathBuf::from(std::env::var("SM_TOUCH_PACKET").unwrap());
        let rom = PathBuf::from(std::env::var("SM_ROM").unwrap());
        expected_failure(
            &packet,
            rom,
            "movement-native-enemy-stop-terrain-absence",
            "intervention-expected.toml",
        );
    }
    #[test]
    #[ignore = "private independently pinned native search packet"]
    fn native_search_corrupt_inputs_refuse_before_execution() {
        let (packet, rom) = source();
        for id in ["missing-input", "corrupt-input", "incompatible-admission"] {
            let root = copy(&packet, id);
            if id == "missing-input" {
                std::fs::remove_file(root.join("inputs/world.bin")).unwrap();
            } else if id == "corrupt-input" {
                let path = root.join("inputs/frame.bin");
                let mut bytes = std::fs::read(&path).unwrap();
                bytes[0] ^= 1;
                std::fs::write(path, bytes).unwrap();
            } else {
                std::fs::write(root.join("packet-pins.txt"), "invalid\n").unwrap();
            }
            assert!(matches!(
                load(&root, &rom, Kind::Miss),
                Err(Outcome::DataFailure { cut: None, .. })
            ));
            std::fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    #[ignore = "private independently pinned native search packet"]
    fn native_search_actual_query_refusal_retains_complete_state_and_output() {
        let (packet, rom) = source();
        let (mut inputs, _) = load(&packet, &rom, Kind::Miss).unwrap();
        let mut production = Production::admit(
            ProductionSeed::Movement(inputs.seed),
            ProductionContent::Movement(inputs.content.movement.clone()),
            ProductionRules::Movement(samus_movement::Rules::default()),
        )
        .unwrap();
        let output = inputs.advance(&mut production, COMPLETION).unwrap();
        let before = production.clone();
        let mut retained = Some(output.clone());
        // This narrows only the qualification envelope: the real step still queries the
        // complete converted world and must refuse before its candidate is retained.
        inputs.width = 0;
        let failure = inputs.advance(&mut production, COMPLETION);
        if let Ok(output) = &failure {
            retained = Some(output.clone());
        }
        assert!(matches!(
            failure,
            Err(Outcome::Unsupported {
                cut: Some(COMPLETION),
                ..
            })
        ));
        assert!(
            production == before,
            "atomic complete native production retention"
        );
        assert!(
            retained == Some(output),
            "atomic complete native output retention"
        );
    }
    #[test]
    #[ignore = "private independently pinned native search packet"]
    fn native_search_null_and_candidate_only_omission_are_distinct() {
        let (packet, rom) = source();
        let case = native_case(&packet);
        let mut admitted = admission::admit(
            &case,
            &Sources {
                rom,
                ..Sources::default()
            },
        )
        .unwrap();
        runner::execute(
            &case,
            &admitted.inputs,
            &admitted.expected,
            &mut admitted.production,
            &mut admitted.actual,
            COMPLETION,
        )
        .unwrap();
        let expected = admitted.expected.read(&case, COMPLETION).unwrap().fields;
        assert!(compare::compare(&expected, &initial()).is_err());
        let vanilla = retained(&admitted.actual).unwrap();
        let mut omitted = vanilla.clone();
        let motion = omitted
            .movement_work
            .iter_mut()
            .find_map(|work| match work {
                types::MovementWork::Motion(motion) => Some(motion),
                types::MovementWork::Support(_) | types::MovementWork::BodyFit(_) => None,
            })
            .unwrap();
        assert_eq!(
            motion
                .enemy_search
                .as_mut()
                .unwrap()
                .candidates
                .pop()
                .unwrap()
                .result,
            types::CandidateResult::Miss
        );
        assert_eq!(
            compare::compare(&expected, &actual(&omitted)),
            Err(compare::CompareFault::Difference(format!(
                "{PREFIX}.enemy_search.candidates"
            )))
        );
        // Complete comparison ignores only this explicitly omitted candidate, then equality holds.
        omitted.movement_work = vanilla.movement_work.clone();
        assert!(
            &omitted == vanilla,
            "candidate-only fault preserves remaining complete native output"
        );
    }

    #[test]
    #[ignore = "private independently pinned native search packet"]
    fn native_search_production_fault_receipt() {
        let (packet, rom) = source();
        let case = native_case(&packet);
        let result = crate::replay::run(
            &case,
            &Sources {
                rom,
                ..Sources::default()
            },
        );
        std::fs::write(
            std::env::var("SM_SEARCH_FAULT_RECEIPT").unwrap(),
            toml::to_string(&result).unwrap(),
        )
        .unwrap();
        if std::env::var_os("SM_SEARCH_EXPECT_FAULT").is_some() {
            assert_eq!(
                result.outcome,
                Outcome::Difference {
                    cut: COMPLETION,
                    field: format!("{PREFIX}.enemy_search.candidates")
                }
            );
        } else {
            assert_eq!(result.outcome, Outcome::Pass);
        }
    }

    #[test]
    #[ignore = "private independently admitted native Touch packet"]
    fn native_touch_contract() {
        let packet = PathBuf::from(std::env::var("SM_TOUCH_PACKET").unwrap());
        let rom = PathBuf::from(std::env::var("SM_ROM").unwrap());
        let case = named_case(&packet, "movement-native-enemy-stop-terrain-absence");
        let result = crate::replay::run(
            &case,
            &Sources {
                rom,
                ..Sources::default()
            },
        );
        std::fs::write(
            std::env::var("SM_TOUCH_RECEIPT").unwrap(),
            toml::to_string(&result).unwrap(),
        )
        .unwrap();
        assert_eq!(result.outcome, Outcome::Pass);
        assert_eq!(result.counts.cuts, 1);
        assert_eq!(result.counts.fields, 8);
        assert_eq!(result.counts.null_cuts, 0);
        assert_eq!(result.counts.null_fields, 2);
        assert_eq!(result.counts.restorations, 1);
    }
}
