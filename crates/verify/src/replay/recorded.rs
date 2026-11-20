//! Finite recorded recipes supply independent facts to the shared production runner.

use super::compare::Observation;
use super::index::{Case, Responsibility, Start};
use super::readers::{
    ActualReader, AdmittedCase, ExpectedCut, ExpectedReader, InputReader, record,
};
use super::result::Outcome;
use super::{
    Sources, appearance, camera, ceres, data_failure, hud, production_failure, progress, weapons,
};
use crate::cache::Cache;
use crate::lock::{Lock, RowId};
use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules, ProductionSeed,
};
use game::state::owned::OwnedState;
use std::sync::Arc;

#[derive(Clone)]
enum Context {
    Ceres(ceres::Slot),
    MovementCeres {
        slot: ceres::Slot,
        content: Arc<super::movement::Content>,
        evidence: super::opportunity::Evidence,
    },
    Weapons(Arc<weapons::Content>),
    Camera,
    Hud(Arc<hud::Content>),
    Appearance(Arc<appearance::Content>),
    Progress,
    Connected {
        content: Arc<super::connected::Content>,
        inputs: Arc<super::connected::Inputs>,
    },
    ConnectedObservations(Arc<super::connected::Content>),
}

pub(super) struct Inputs {
    cache: Box<Cache>,
    recipe: Context,
    row: String,
}

pub(super) struct Expected {
    cache: Box<Cache>,
    recipe: Context,
}

#[derive(Clone)]
pub(super) enum Actual {
    Ceres,
    MovementCeres {
        last: Option<Box<game::production::MovementCeresOutputs>>,
    },
    Weapons,
    Camera,
    Hud(Arc<hud::Content>),
    Appearance(Arc<appearance::Content>),
    Progress,
    Connected {
        content: Arc<super::connected::Content>,
    },
}

struct Recipe {
    source: Context,
    actual: Actual,
    seed: ProductionSeed,
    content: ProductionContent,
    rules: ProductionRules,
}

fn connected_recipe(
    case: &Case,
    wram: &[u8],
    sources: &Sources,
    cache: &Cache,
) -> Result<Recipe, Outcome> {
    let lock =
        std::fs::read(&sources.lock).map_err(|_| data_failure(None, "opportunity source lock"))?;
    if crate::lock::Sha256::of(&lock).to_string()
        != "d12b9153c73a2eb79e6de12eea3b4e6ea0ebf5101a0673d4cb61f2ffaf84dd71"
    {
        return Err(data_failure(None, "reviewed opportunity lock provenance"));
    }
    let slot = ceres::Slot::try_from(
        case.actor_slot
            .ok_or_else(|| data_failure(None, "connected Ceres slot"))?,
    )?;
    let content = Arc::new(super::movement::Content::load(&sources.rom)?);
    let evidence = super::opportunity::admit(
        &sources.native.join("movement-opportunities-v1"),
        cache,
        &content,
    )?;
    let seed = ProductionSeed::MovementCeres(Box::new(game::production::MovementCeresState {
        movement: canonical::model::Model(super::movement::seed(wram)?),
        ceres: ceres::seed(wram, slot)?,
    }));
    let production_content = ProductionContent::MovementCeres(content.movement.clone());
    Ok(Recipe {
        source: Context::MovementCeres {
            slot,
            content,
            evidence,
        },
        actual: Actual::MovementCeres { last: None },
        seed,
        content: production_content,
        rules: ProductionRules::MovementCeres {
            movement: samus_movement::Rules::default(),
            ceres: boss_ceres_ridley::Rules::default(),
        },
    })
}

fn ordinary_recipe(
    case: &Case,
    wram: &[u8],
    sources: &Sources,
    cache: &Cache,
) -> Result<Recipe, Outcome> {
    let content = Arc::new(super::connected::Content {
        movement: super::movement::Content::load(&sources.rom)?,
        appearance: appearance::Content::load(&sources.rom)?,
    });
    let inputs = Arc::new(super::connected::Inputs::load(
        &sources.native.join(super::connected::fixture(case)?),
        &content,
    )?);
    if let Start::CacheCut { frame } = case.start {
        inputs.aligned(frame, &record(cache, frame)?)?;
    }
    let (appearance, playing) = appearance::running_owner(wram, &content.appearance)?;
    let state = game::production::connected::State::new(
        super::movement::seed(wram)?,
        appearance,
        playing,
        camera::seed(wram, &camera::Content)?,
    )
    .map_err(|_| data_failure(None, "connected initial owners"))?;
    Ok(Recipe {
        source: Context::Connected {
            content: content.clone(),
            inputs,
        },
        actual: Actual::Connected {
            content: content.clone(),
        },
        seed: ProductionSeed::Connected(Box::new(state)),
        content: ProductionContent::Connected(Box::new(game::production::connected::Content {
            movement: content.movement.movement.clone(),
            appearance: content.appearance.appearance.clone(),
        })),
        rules: ProductionRules::Connected(Box::new(game::production::connected::Rules {
            movement: samus_movement::Rules::default(),
            appearance: samus_appearance::Rules::VANILLA,
            camera: game::production::CameraRules::new(world_camera::Rules::REFERENCE),
        })),
    })
}

fn recipe(case: &Case, wram: &[u8], sources: &Sources, cache: &Cache) -> Result<Recipe, Outcome> {
    if case.responsibilities
        == [
            Responsibility::Movement,
            Responsibility::Appearance,
            Responsibility::Camera,
        ]
    {
        return ordinary_recipe(case, wram, sources, cache);
    }

    if case.responsibilities == [Responsibility::Movement, Responsibility::Ceres] {
        return connected_recipe(case, wram, sources, cache);
    }
    let (source, actual, seed, content, rules) = match case.responsibilities.as_slice() {
        [Responsibility::Ceres] => {
            let slot = ceres::Slot::try_from(
                case.actor_slot
                    .ok_or_else(|| data_failure(None, "Ceres indexed actor slot"))?,
            )?;
            (
                Context::Ceres(slot),
                Actual::Ceres,
                ProductionSeed::Ceres(ceres::seed(wram, slot)?),
                ProductionContent::Ceres,
                ProductionRules::Ceres(boss_ceres_ridley::Rules::default()),
            )
        }
        [Responsibility::Weapons, Responsibility::Contact] => {
            let content = Arc::new(weapons::Content::load(&sources.rom)?);
            let seed = ProductionSeed::WeaponsContact(Box::new(weapons::seed(wram, &content)?));
            let production_content =
                ProductionContent::WeaponsContact(Box::new(content.weapons.clone()));
            (
                Context::Weapons(content),
                Actual::Weapons,
                seed,
                production_content,
                ProductionRules::WeaponsContact(weapons_projectiles::Rules::REFERENCE),
            )
        }
        [Responsibility::Camera] => (
            Context::Camera,
            Actual::Camera,
            ProductionSeed::Camera(camera::seed(wram, &camera::Content)?),
            ProductionContent::Camera,
            ProductionRules::Camera(game::production::CameraRules::new(
                world_camera::Rules::REFERENCE,
            )),
        ),
        [Responsibility::Hud] => {
            let content = Arc::new(hud::Content::load(&sources.rom)?);
            let seed = ProductionSeed::Hud(Box::new(hud::seed(wram, &content)?));
            let pictures = ProductionContent::Hud(Box::new(content.pictures));
            (
                Context::Hud(content.clone()),
                Actual::Hud(content),
                seed,
                pictures,
                ProductionRules::Hud(game::production::HudRules::new(
                    interface_hud::Rules::default(),
                )),
            )
        }
        [Responsibility::Appearance] => {
            let content = Arc::new(appearance::Content::load(&sources.rom)?);
            let seed = ProductionSeed::Appearance(appearance::seed(wram, &content)?);
            let clips = ProductionContent::Appearance(content.appearance.clone());
            (
                Context::Appearance(content.clone()),
                Actual::Appearance(content),
                seed,
                clips,
                ProductionRules::Appearance(samus_appearance::Rules::VANILLA),
            )
        }
        [Responsibility::Progress] => (
            Context::Progress,
            Actual::Progress,
            ProductionSeed::Progress(Box::new(progress::seed(wram)?)),
            ProductionContent::Progress,
            ProductionRules::Progress(run_progress::Rules::reference()),
        ),
        _ => return Err(data_failure(None, "recorded production recipe")),
    };
    Ok(Recipe {
        source,
        actual,
        seed,
        content,
        rules,
    })
}

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let Start::CacheCut { frame } = case.start else {
        return Err(data_failure(None, "recorded recipe start"));
    };
    let text = std::fs::read_to_string(&sources.lock)
        .map_err(|_| data_failure(None, "record lock file"))?;
    let lock = Lock::parse(&text).map_err(|_| data_failure(None, "record lock schema"))?;
    let row = lock
        .row(&RowId(case.row.clone()))
        .ok_or_else(|| data_failure(None, "indexed lock row"))?;
    let inputs = Box::new(
        Cache::open(&sources.cache, row)
            .map_err(|_| data_failure(None, "input record cache pins"))?,
    );
    let expected = Box::new(
        Cache::open(&sources.cache, row)
            .map_err(|_| data_failure(None, "expected record cache pins"))?,
    );
    let seed_record = record(&inputs, frame)?;
    if case.responsibilities == [Responsibility::Progress] {
        progress::validate_watches(inputs.watched())?;
    }
    let Recipe {
        source: recipe,
        actual,
        seed,
        content,
        rules,
    } = recipe(case, &seed_record.wram, sources, &inputs)?;
    let production = Production::admit(seed, content.clone(), rules.clone())
        .map_err(|_| data_failure(Some(frame), "recorded production admission"))?;
    if case.checkpoints {
        production
            .save()
            .map_err(
                |game::production::SaveFault::Unsupported(scope)| Outcome::Unsupported {
                    cut: None,
                    dependency: format!("checkpoint codec for {scope:?}"),
                },
            )?;
    }
    Ok(AdmittedCase {
        production,
        content,
        rules,
        inputs: InputReader::Recorded(Box::new(Inputs {
            cache: inputs,
            recipe: recipe.clone(),
            row: case.row.clone(),
        })),
        expected: ExpectedReader::Recorded(Box::new(Expected {
            cache: expected,
            recipe: match recipe {
                Context::Connected { content, .. } => Context::ConnectedObservations(content),
                recipe @ (Context::Ceres(_)
                | Context::MovementCeres { .. }
                | Context::Weapons(_)
                | Context::Camera
                | Context::Hud(_)
                | Context::Appearance(_)
                | Context::Progress
                | Context::ConnectedObservations(_)) => recipe,
            },
        })),
        actual: ActualReader::Recorded(actual),
        schedule: (frame
            .checked_add(1)
            .ok_or_else(|| data_failure(Some(frame), "recorded start range"))?
            ..=case.end.cut_inclusive)
            .collect(),
    })
}

fn preceding(cache: &Cache, cut: u64) -> Result<oracle_record::FrameRecord, Outcome> {
    record(
        cache,
        cut.checked_sub(1)
            .ok_or_else(|| data_failure(Some(cut), "recorded preceding cut"))?,
    )
}

impl Inputs {
    pub fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        let prior = preceding(&self.cache, cut)?;
        let outputs = match &self.recipe {
            Context::ConnectedObservations(_) => {
                return Err(data_failure(
                    Some(cut),
                    "expected reader cannot advance production",
                ));
            }
            Context::Connected { inputs, .. } => {
                inputs.aligned(
                    cut.checked_sub(1)
                        .ok_or_else(|| data_failure(Some(cut), "connected preceding source"))?,
                    &prior,
                )?;
                production.advance(AdvanceInput::Connected {
                    inputs: inputs.current(cut)?,
                    advance: game::production::connected::Advance::Opportunity,
                })
            }
            Context::MovementCeres {
                slot,
                content,
                evidence,
            } => {
                evidence.require(cut)?;
                if !super::movement::roster_slots(&prior.wram)?.is_empty() {
                    return Err(Outcome::Unsupported {
                        cut: Some(cut),
                        dependency: "reviewed empty interactive roster".to_owned(),
                    });
                }
                ceres::validate_step(&prior.wram, &record(&self.cache, cut)?.wram, *slot)?;
                let OwnedState::MovementCeres(state) = production.state() else {
                    return Err(data_failure(Some(cut), "connected owned scope"));
                };
                let input = super::movement::input(&prior.wram, state.movement.0.pose, content)?;
                let ceres = ceres::context(&prior.wram, *slot)?;
                let radius = content
                    .movement
                    .radius(state.movement.0.pose)
                    .ok_or_else(|| data_failure(Some(cut), "connected body content"))?;
                let body = |_| radius;
                let heading = |pose| content.movement.cannon_heading(pose);
                let frame = samus_movement::Frame {
                    sprite_before_movement: input.sprite,
                    cinematic_active: input.cinematic,
                    pad: input.pad,
                    pressed: input.pressed,
                    items: input.items,
                    spin_turned_far: input.ahead.turned_far,
                    elevator_moving: input.elevator,
                    parity: input.parity,
                    animation_ends: input.ahead.end,
                    body: &body,
                    cannon_heading: &heading,
                };
                production.advance(AdvanceInput::MovementCeres {
                    frame,
                    world: &input.world,
                    ceres,
                })
            }
            Context::Ceres(slot) => {
                ceres::validate_step(&prior.wram, &record(&self.cache, cut)?.wram, *slot)?;
                let input = ceres::input(&prior.wram, *slot)?;
                production.advance(AdvanceInput::Ceres(&input))
            }
            Context::Weapons(content) => {
                let input = weapons::input(&prior.wram, content, &self.row, cut)?;
                production.advance(AdvanceInput::WeaponsContact {
                    input: &input.update,
                    world: &input.world,
                    contacts: &input.opportunities,
                })
            }
            Context::Camera => {
                camera::validate_step(&prior.wram, &record(&self.cache, cut)?.wram)?;
                let input = camera::input(&prior.wram, &camera::Content)?;
                production.advance(AdvanceInput::Camera(&input.frame()))
            }
            Context::Hud(content) => {
                let current = record(&self.cache, cut)?;
                hud::validate_step(&prior.wram, &current.wram)?;
                hud::validate_events(&current)?;
                let input = hud::input(&prior.wram, content)?;
                production.advance(AdvanceInput::Hud(input.input()))
            }
            Context::Appearance(content) => {
                appearance::validate_step(&prior.wram, &record(&self.cache, cut)?.wram)?;
                let input = appearance::input(&prior.wram, content)?;
                production.advance(AdvanceInput::Appearance(&input))
            }
            Context::Progress => {
                let input = progress::input(&record(&self.cache, cut)?);
                production.advance(AdvanceInput::Progress(&input))
            }
        };
        outputs.map_err(|fault| production_failure(cut, fault))
    }
}

impl Expected {
    pub fn read(&self, case: &Case, cut: u64) -> Result<ExpectedCut, Outcome> {
        let current = record(&self.cache, cut)?;
        let fields = match &self.recipe {
            Context::ConnectedObservations(content) => {
                super::connected::expected(&current.wram, &content.appearance, case, cut)?
            }
            Context::Connected { .. } => {
                return Err(data_failure(
                    Some(cut),
                    "input reader cannot read expectations",
                ));
            }
            Context::MovementCeres { slot, .. } => {
                let mut fields = super::movement::expected(
                    &current.wram,
                    &[
                        super::index::ObservationId::PositionX,
                        super::index::ObservationId::PositionY,
                    ],
                    &case.row,
                    cut,
                )?;
                fields.extend(ceres::expected(&current.wram, *slot)?);
                fields
            }
            Context::Ceres(slot) => ceres::expected(&current.wram, *slot)?,
            Context::Weapons(content) => weapons::expected(&current.wram, content)?,
            Context::Camera => camera::expected(&current.wram)?,
            Context::Hud(content) => hud::expected(&current.wram, content)?,
            Context::Appearance(content) => appearance::expected(&current.wram, content)?,
            Context::Progress => progress::expected(&current.wram)?,
        };
        let semantic_count = case.expectations.fields.len();
        Ok(ExpectedCut {
            semantics: fields.iter().take(semantic_count).cloned().collect(),
            fields,
            semantic_count,
            output_count: case.expectations.outputs.len(),
            replies: 0,
        })
    }
}

impl Actual {
    pub fn accept(
        &mut self,
        production: &Production,
        outputs: &ProductionOutputs,
        cut: u64,
    ) -> Result<Vec<Observation>, Outcome> {
        match (self, production.state(), outputs) {
            (
                Self::Connected { content, .. },
                OwnedState::Connected(state),
                ProductionOutputs::Connected(outputs),
            ) => Ok(super::connected::actual(
                state,
                &content.appearance,
                Some(outputs),
            )),
            (
                Self::MovementCeres { last },
                OwnedState::MovementCeres(state),
                ProductionOutputs::MovementCeres(outputs),
            ) => {
                *last = Some(outputs.clone());
                let mut fields = connected_position(state);
                fields.extend(ceres::actual(&state.ceres, &outputs.ceres));
                Ok(fields)
            }
            (Self::Ceres, OwnedState::Ceres(state), ProductionOutputs::Ceres(outputs)) => {
                Ok(ceres::actual(state, outputs))
            }
            (
                Self::Weapons,
                OwnedState::WeaponsContact(state),
                ProductionOutputs::WeaponsContact(outputs),
            ) => Ok(weapons::actual(&state.0, outputs)),
            (Self::Camera, OwnedState::Camera(state), ProductionOutputs::Camera(outputs)) => {
                Ok(camera::actual(&state.0, outputs))
            }
            (Self::Hud(content), OwnedState::Hud(state), ProductionOutputs::Hud(outputs)) => {
                hud::actual(state, outputs, content)
            }
            (
                Self::Appearance(content),
                OwnedState::Appearance(state),
                ProductionOutputs::Appearance(outputs),
            ) => Ok(appearance::actual(state.0, *outputs, content)),
            (Self::Progress, OwnedState::Progress(state), ProductionOutputs::Progress) => {
                Ok(progress::actual(state))
            }
            _ => Err(data_failure(Some(cut), "recorded actual scope")),
        }
    }

    pub fn initial(&self, production: &Production) -> Result<Vec<Observation>, Outcome> {
        match (self, production.state()) {
            (Self::Connected { content, .. }, OwnedState::Connected(state)) => {
                Ok(super::connected::actual(state, &content.appearance, None))
            }
            (Self::MovementCeres { .. }, OwnedState::MovementCeres(state)) => {
                let mut fields = connected_position(state);
                fields.extend(ceres::initial(&state.ceres));
                Ok(fields)
            }
            (Self::Ceres, OwnedState::Ceres(state)) => Ok(ceres::initial(state)),
            (Self::Weapons, OwnedState::WeaponsContact(state)) => Ok(weapons::initial(&state.0)),
            (Self::Camera, OwnedState::Camera(state)) => Ok(camera::initial(&state.0)),
            (Self::Hud(content), OwnedState::Hud(state)) => hud::initial(state, content),
            (Self::Appearance(content), OwnedState::Appearance(state)) => {
                Ok(appearance::initial(state.0, content))
            }
            (Self::Progress, OwnedState::Progress(state)) => Ok(progress::actual(state)),
            _ => Err(data_failure(None, "recorded initial scope")),
        }
    }
}

fn connected_position(state: &game::production::MovementCeresState) -> Vec<Observation> {
    vec![
        Observation::quantity(
            "movement.position.x",
            i64::from(state.movement.0.position.x.0),
        ),
        Observation::quantity(
            "movement.position.y",
            i64::from(state.movement.0.position.y.0),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::{Context, admit};
    use crate::replay::{
        Sources,
        index::Index,
        readers::{InputReader, record},
    };
    use game::production::{AdvanceInput, HudInput};

    #[test]
    #[ignore = "private pinned HUD capture and player ROM"]
    fn recorded_unselected_powered_map_has_equivalent_drop_and_turn_on_inputs() {
        let index = Index::parse(include_str!("../../../../scenarios/replay/index.toml")).unwrap();
        let case = index
            .cases
            .iter()
            .find(|case| case.id.0 == "hud/stationary-map-blink")
            .unwrap();
        let path = |name| std::env::var_os(name).unwrap().into();
        let sources = Sources {
            cache: path("SM_CACHE"),
            rom: path("SM_ROM"),
            lock: index.lock.clone(),
            ..Sources::default()
        };
        let admitted = admit(case, &sources).unwrap();
        let InputReader::Recorded(reader) = &admitted.inputs else {
            panic!()
        };
        let Context::Hud(content) = &reader.recipe else {
            panic!()
        };
        let prior = record(
            &reader.cache,
            case.first_cut().unwrap().checked_sub(1).unwrap(),
        )
        .unwrap();
        let input = super::hud::input(&prior.wram, content).unwrap();
        let baseline = input.frame();
        let mut reference = admitted.production.clone();
        let expected_outputs = reference
            .advance(AdvanceInput::Hud(HudInput::Play(&baseline)))
            .unwrap();
        let drops = [
            interface_hud::Dropped::Nothing,
            interface_hud::Dropped::Selection,
            interface_hud::Dropped::OneShotItemAlone,
        ];
        let mut compared = 0_usize;
        for before in drops {
            for after in drops {
                for turned_on in [false, true] {
                    let mut alternative = baseline;
                    alternative.dropped_before_status_bar = before;
                    alternative.dropped_after_status_bar = after;
                    alternative.mini_map.turned_on = turned_on;
                    let mut production = admitted.production.clone();
                    let outputs = production
                        .advance(AdvanceInput::Hud(HudInput::Play(&alternative)))
                        .unwrap();
                    assert!(production.state().eq(reference.state()));
                    assert!(outputs.eq(&expected_outputs));
                    compared = compared.saturating_add(1);
                }
            }
        }
        assert_eq!(compared, 18);
    }
}

#[cfg(test)]
mod connection_tests {
    use super::*;
    use crate::replay::{index::Index, runner::execute};
    use oracle_record::codec::{PictureMode, encode};
    use std::fmt::Write as _;

    #[derive(Clone, Copy)]
    enum ExpectedMutation {
        X,
        Y,
        Phase,
        Wait,
        Figure,
        Shown,
        Solid,
    }
    impl ExpectedMutation {
        const fn name(self) -> &'static str {
            match self {
                Self::X => "movement.position.x",
                Self::Y => "movement.position.y",
                Self::Phase => "ceres.phase",
                Self::Wait => "ceres.wait",
                Self::Figure => "ceres.figure",
                Self::Shown => "ceres.shown",
                Self::Solid => "ceres.solid",
            }
        }
    }
    fn altered_expectations(
        sources: &Sources,
        row: &crate::lock::Row,
        cache: &Cache,
        field: Option<ExpectedMutation>,
        target: u64,
    ) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "sm-expected-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let directory = root.join(&row.id.0);
        std::fs::create_dir(&root).unwrap();
        std::fs::create_dir(&directory).unwrap();
        let mut header = std::fs::read_to_string(sources.cache.join(&row.id.0).join("lean.txt"))
            .unwrap()
            .lines()
            .take(7)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        header[0] = crate::cache::HEAD.to_owned();
        header[6] = "pictures no".to_owned();
        let mut index = header.join("\n") + "\n";
        let mut bytes = Vec::new();
        for entry in cache.entries() {
            let start = bytes.len();
            if (6..=10).contains(&entry.frame.0) && (field.is_some() || entry.frame.0 != target) {
                let mut original = record(cache, entry.frame.0).unwrap();
                if entry.frame.0 == target
                    && let Some(field) = field
                {
                    let (address, value): (usize, u16) = match field {
                        ExpectedMutation::X => (
                            0x0af6,
                            u16::from_le_bytes(original.wram[0x0af6..0x0af8].try_into().unwrap())
                                .wrapping_add(1),
                        ),
                        ExpectedMutation::Y => (
                            0x0afa,
                            u16::from_le_bytes(original.wram[0x0afa..0x0afc].try_into().unwrap())
                                .wrapping_add(1),
                        ),
                        ExpectedMutation::Phase => (0x0f92, 0xf5a0),
                        ExpectedMutation::Wait => (0x0f94, 40),
                        ExpectedMutation::Figure => (0x0f8e, 0xfa13),
                        ExpectedMutation::Shown => (
                            0x0f86,
                            u16::from_le_bytes(original.wram[0x0f86..0x0f88].try_into().unwrap())
                                ^ 0x0100,
                        ),
                        ExpectedMutation::Solid => (
                            0x0f86,
                            u16::from_le_bytes(original.wram[0x0f86..0x0f88].try_into().unwrap())
                                ^ 0x0400,
                        ),
                    };
                    original.wram[address..address.saturating_add(2)]
                        .copy_from_slice(&value.to_le_bytes());
                }
                encode(&original, PictureMode::Drop, &mut bytes);
            }
            let length = bytes.len().checked_sub(start).unwrap();
            let kind = match entry.cut {
                crate::cache::CutKind::Requested => "R",
                crate::cache::CutKind::Overrun => "O",
                crate::cache::CutKind::VblankWithoutNmi => "V",
                crate::cache::CutKind::NmiStillRunning => "S",
            };
            writeln!(index, "{} {start} {length} {kind}", entry.frame.0).unwrap();
        }
        std::fs::write(directory.join("index.txt"), index).unwrap();
        std::fs::write(directory.join("records.bin"), bytes).unwrap();
        root
    }
    fn execute_cut(
        case: &Case,
        admitted: &mut AdmittedCase,
        cut: u64,
    ) -> Result<ExpectedCut, Outcome> {
        execute(
            case,
            &admitted.inputs,
            &admitted.expected,
            &mut admitted.production,
            &mut admitted.actual,
            cut,
        )
    }
    #[test]
    #[ignore = "private pinned capture, cache and player ROM"]
    fn expected_position_and_door_sources_cannot_change_complete_owners_or_outputs() {
        use ExpectedMutation as M;
        let index = Index::parse(include_str!("../../../../scenarios/replay/index.toml")).unwrap();
        let case = index
            .cases
            .iter()
            .find(|case| case.id.0 == "movement-ceres/carried-position-proximity")
            .unwrap();
        let path = |name| std::env::var_os(name).unwrap().into();
        let sources = Sources {
            cache: path("SM_CACHE"),
            rom: path("SM_ROM"),
            native: path("SM_AUDIO_NATIVE"),
            lock: std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../")
                .join(&index.lock),
            ..Sources::default()
        };
        let lock = Lock::parse(&std::fs::read_to_string(&sources.lock).unwrap()).unwrap();
        let row = lock.row(&RowId(case.row.clone())).unwrap();
        for (target, changed) in [9, 7].into_iter().flat_map(|target| {
            [
                Some(M::X),
                Some(M::Y),
                Some(M::Phase),
                Some(M::Wait),
                Some(M::Figure),
                Some(M::Shown),
                Some(M::Solid),
                None,
            ]
            .into_iter()
            .map(move |changed| (target, changed))
        }) {
            let mut reference = admit(case, &sources).unwrap();
            let mut altered = admit(case, &sources).unwrap();
            for cut in 7..target {
                execute_cut(case, &mut reference, cut).unwrap();
                execute_cut(case, &mut altered, cut).unwrap();
            }
            let InputReader::Recorded(input) = &reference.inputs else {
                panic!()
            };
            let temporary = altered_expectations(&sources, row, &input.cache, changed, target);
            let ExpectedReader::Recorded(expected) = &mut altered.expected else {
                panic!()
            };
            *expected.cache = Cache::open(&temporary, row).unwrap();
            // Warmed owners retain their produced state; only the expectation handle changes.
            execute_cut(case, &mut reference, target).unwrap();
            let result = execute_cut(case, &mut altered, target);
            if let Some(field) = changed {
                assert!(
                    matches!(result,Err(Outcome::Difference { cut,field: actual }) if cut == target && actual == field.name())
                );
            } else {
                assert!(matches!(
                    result,
                    Err(Outcome::DataFailure { cut: Some(cut), .. }) if cut == target
                ));
            }
            let complete_owners_unchanged = reference.production == altered.production;
            assert!(complete_owners_unchanged);
            let (
                ActualReader::Recorded(Actual::MovementCeres {
                    last: reference_outputs,
                }),
                ActualReader::Recorded(Actual::MovementCeres {
                    last: altered_outputs,
                }),
            ) = (&reference.actual, &altered.actual)
            else {
                panic!()
            };
            assert!(reference_outputs.is_some());
            let complete_outputs_unchanged = reference_outputs == altered_outputs;
            assert!(complete_outputs_unchanged);
            drop(altered);
            let directory = temporary.join(&row.id.0);
            std::fs::remove_file(directory.join("index.txt")).unwrap();
            std::fs::remove_file(directory.join("records.bin")).unwrap();
            std::fs::remove_dir(directory).unwrap();
            std::fs::remove_dir(temporary).unwrap();
        }
    }
}
