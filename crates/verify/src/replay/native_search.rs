//! Independently admitted within-video movement inputs and the cells the step touched.

use super::{compare::Observation, data_failure, movement, result::Outcome};
use crate::{lock::Sha256, source};
use game::production::{AdvanceInput, Production, ProductionOutputs};
use game::state::owned::OwnedState;
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};
use types::{Block, Cell, Hitbox, Way, World};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
pub(super) struct Coordinates {
    pub video_frame: u64,
    pub source_start_index: u64,
    pub source_completion_index: u64,
}

pub(super) struct Packet<'a> {
    path: &'a Path,
    sha256: &'a str,
    coordinates: Coordinates,
    projection: &'a super::index::NativeProjection,
}

impl<'a> Packet<'a> {
    pub fn for_case(case: &'a super::index::Case) -> Option<Self> {
        use super::index::{Start, fixture_component, pinned_directory};
        let Start::NativeOpportunity {
            path,
            sha256,
            video_frame,
            source_start_index,
            source_completion_index,
        } = &case.start
        else {
            return None;
        };
        let coordinates = Coordinates {
            video_frame: *video_frame,
            source_start_index: *source_start_index,
            source_completion_index: *source_completion_index,
        };
        let projection = case
            .expectations
            .projection
            .as_ref()
            .or_else(|| super::legacy_packets::projection(sha256, coordinates))?;
        (pinned_directory(&projection.directory, &projection.sha256)
            && fixture_component(&projection.member))
        .then_some(Self {
            path,
            sha256,
            coordinates,
            projection,
        })
    }
}

#[derive(serde::Deserialize)]
struct Inventory {
    source_start_index: u64,
    source_completion_index: u64,
}

pub(super) struct Inputs {
    pub seed: samus_movement::Samus,
    pub content: movement::Content,
    input: movement::Input,
    video_frame: u64,
    width: u16,
    height: u16,
}

pub(super) struct Expected {
    coordinates: Coordinates,
    projection: super::index::NativeProjection,
    work_root: PathBuf,
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

pub(super) fn load(
    case: &super::index::Case,
    sources: &super::Sources,
) -> Result<(Inputs, Expected), Outcome> {
    let packet = Packet::for_case(case)
        .ok_or_else(|| data_failure(None, "native search packet declaration"))?;
    let input_root = sources.native.join(packet.path);
    let root = input_root.as_path();
    let projection_root = if sources.native.as_os_str().is_empty() {
        root.parent().unwrap_or(root)
    } else {
        sources.native.as_path()
    };
    let pins = std::fs::read(root.join("packet-pins.txt"))
        .map_err(|_| data_failure(None, "native search packet manifest"))?;
    if Sha256::try_from(packet.sha256).ok() != Some(Sha256::of(&pins)) {
        return Err(data_failure(None, "native search admission pin"));
    }
    let pins = String::from_utf8(pins)
        .map_err(|_| data_failure(None, "native search manifest encoding"))?;
    let inventory = pinned(root, &pins, "input-inventory.toml")?;
    let inventory: Inventory = std::str::from_utf8(&inventory)
        .ok()
        .and_then(|s| toml::from_str(s).ok())
        .ok_or_else(|| data_failure(None, "native search input inventory"))?;
    if inventory.source_start_index != packet.coordinates.source_start_index
        || inventory.source_completion_index != packet.coordinates.source_completion_index
    {
        return Err(data_failure(None, "native search input coordinates"));
    }
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
    let content = movement::Content::load(&sources.rom)?;
    let seed = movement::seed(&seed_bytes)?;
    // Reject unavailable typed dependencies at admission without executing gameplay.
    let input = movement::input_at(&frame, &world, seed.pose, &content)?;
    Ok((
        Inputs {
            seed,
            content,
            input,
            video_frame: packet.coordinates.video_frame,
            width,
            height,
        },
        Expected {
            coordinates: packet.coordinates,
            projection: packet.projection.clone(),
            work_root: projection_root.join(&packet.projection.directory),
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
        if cut != self.video_frame {
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

impl Expected {
    pub fn read(&self, cut: u64) -> Result<Vec<Observation>, Outcome> {
        if cut != self.coordinates.video_frame {
            return Err(data_failure(
                Some(cut),
                "native search expectation opportunity",
            ));
        }
        super::touched_cells::expected(&self.work_root, &self.projection, self.coordinates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::{
        Sources, admission, compare,
        index::{Index, Start},
        readers::ActualReader,
        runner, touched_cells,
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
    #[test]
    fn native_projection_uses_the_index_and_preserves_published_packet_pairs() {
        let packet = Path::new("input");
        let mut case = native_case(packet);
        let projection = case.expectations.projection.take().unwrap();
        assert_eq!(
            Packet::for_case(&case).unwrap().projection.member,
            projection.member
        );
        let Start::NativeOpportunity {
            source_completion_index,
            ..
        } = &mut case.start
        else {
            panic!()
        };
        *source_completion_index = source_completion_index.saturating_sub(1);
        assert!(case.validate().is_err());

        case.expectations.projection = Some(projection);
        assert!(case.validate().is_ok());
        case.expectations.projection.as_mut().unwrap().member = "../inputs/world.bin".to_owned();
        assert!(case.validate().is_err());
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
            "input-inventory.toml",
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
        assert_eq!(result.counts.fields, 2);
        assert_eq!(result.counts.null_cuts, 0);
        assert_eq!(result.counts.null_fields, 0);
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
            case.end.cut_inclusive,
        )
        .unwrap();
        for (id, remove) in [("complete-missing", true), ("complete-altered", false)] {
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
            let path = work_root.join(complete_name);
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
                case.end.cut_inclusive,
            )
            .err()
            .unwrap();
            assert!(matches!(
                failure,
                Outcome::DataFailure {
                    cut: Some(cut),
                    ..
                } if cut == case.end.cut_inclusive
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
                load(
                    &native_case(&root),
                    &Sources {
                        rom: rom.clone(),
                        ..Sources::default()
                    }
                ),
                Err(Outcome::DataFailure { cut: None, .. })
            ));
            std::fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    #[ignore = "private independently pinned native search packet"]
    fn native_search_actual_query_refusal_retains_complete_state_and_output() {
        let (packet, rom) = source();
        let case = native_case(&packet);
        let (mut inputs, _) = load(
            &case,
            &Sources {
                rom,
                ..Sources::default()
            },
        )
        .unwrap();
        let mut production = Production::admit(
            ProductionSeed::Movement(inputs.seed),
            ProductionContent::Movement(inputs.content.movement.clone()),
            ProductionRules::Movement(samus_movement::Rules::default()),
        )
        .unwrap();
        let output = inputs
            .advance(&mut production, case.end.cut_inclusive)
            .unwrap();
        let before = production.clone();
        let mut retained = Some(output.clone());
        // This narrows only the qualification envelope: the real step still queries the
        // complete converted world and must refuse before its candidate is retained.
        inputs.width = 0;
        let failure = inputs.advance(&mut production, case.end.cut_inclusive);
        if let Ok(output) = &failure {
            retained = Some(output.clone());
        }
        assert!(matches!(
            failure,
            Err(Outcome::Unsupported {
                cut: Some(cut),
                ..
            }) if cut == case.end.cut_inclusive
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
    fn native_search_null_and_one_cell_omission_are_distinct() {
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
            case.end.cut_inclusive,
        )
        .unwrap();
        let expected = admitted
            .expected
            .read(&case, case.end.cut_inclusive)
            .unwrap()
            .fields;
        assert!(compare::compare(&expected, &touched_cells::initial().unwrap()).is_err());
        let vanilla = retained(&admitted.actual).unwrap();
        let mut omitted = vanilla.clone();
        assert!(omitted.inside.pop().is_some());
        assert!(matches!(
            compare::compare(&expected, &touched_cells::actual(&omitted).unwrap()),
            Err(compare::CompareFault::Difference(field)) if field.starts_with("movement.inside")
        ));
        // Only the omitted cell differs: putting it back restores the complete native output.
        omitted.inside.clone_from(&vanilla.inside);
        assert!(
            &omitted == vanilla,
            "cell-only fault preserves remaining complete native output"
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
            assert!(matches!(
                result.outcome,
                Outcome::Difference {
                    cut,
                    ..
                } if cut == case.end.cut_inclusive
            ));
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
        assert_eq!(result.counts.fields, 2);
        assert_eq!(result.counts.null_cuts, 0);
        assert_eq!(result.counts.null_fields, 0);
        assert_eq!(result.counts.restorations, 1);
    }
}
