//! Finite recorded recipes supply independent facts to the shared production runner.

use super::compare::Observation;
use super::index::{Case, ExpectedProvider, FixtureFile, InputProvider, Responsibility, Start};
use super::readers::{
    ActualReader, AdmittedCase, ExpectedCut, ExpectedReader, InputReader, Tape, record,
};
use super::result::Outcome;
use super::{Sources, appearance, camera, data_failure, production_failure, progress, weapons};
use crate::cache::Cache;
use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules, ProductionSeed,
};
use game::state::owned::OwnedState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ObservedCut {
    row: String,
    cut: u64,
    preceding_wram_sha256: String,
}

impl ObservedCut {
    fn valid(&self, case: &Case) -> bool {
        self.row == case.row
            && self.cut > 0
            && super::index::valid_hash(&self.preceding_wram_sha256)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(super) enum Witness {
    WallCells {
        source: ObservedCut,
        wall: samus_movement::WallTest,
        reached: Vec<types::Touch>,
    },
    CameraFollow {
        source: ObservedCut,
    },
    DoorVisits {
        inputs: FixtureFile,
        actors: Vec<u8>,
    },
}

impl Witness {
    pub(super) fn valid_for(case: &Case) -> bool {
        let declared = case.expectations.provider == ExpectedProvider::WitnessedRecordCache;
        declared == case.ordinary_witness.is_some()
            && case
                .ordinary_witness
                .as_ref()
                .is_none_or(|witness| witness.binds(case))
    }

    fn binds(&self, case: &Case) -> bool {
        match self {
            Self::WallCells {
                source, reached, ..
            } => {
                source.valid(case)
                    && case.expectations.output_cut == Some(source.cut)
                    && !reached.is_empty()
                    && reached.iter().all(|touch| {
                        touch.cell.x.checked_mul(16).is_some()
                            && touch.cell.y.checked_mul(16).is_some()
                    })
            }
            Self::CameraFollow { source } => {
                source.valid(case)
                    && case.end.cut_inclusive == source.cut
                    && case.first_cut() == Some(source.cut)
            }
            Self::DoorVisits { inputs, actors } => {
                inputs.valid() && super::shot_drawing::valid_slots(actors)
            }
        }
    }

    pub(super) fn qualify_origin(&self, cache: &Cache) -> Result<(), Outcome> {
        if let Self::CameraFollow { source } | Self::WallCells { source, .. } = self {
            let cut = source
                .cut
                .checked_sub(1)
                .ok_or_else(|| data_failure(None, "missing observed predecessor"))?;
            let prior = record(cache, cut)?;
            if crate::lock::Sha256::of(&prior.wram).to_string() != source.preceding_wram_sha256 {
                return Err(data_failure(Some(cut), "observed predecessor WRAM"));
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
enum Context {
    Weapons(Arc<weapons::Content>),
    DoorInputs(Arc<weapons::Content>, Arc<Tape<weapons::Input>>),
    ShotInputs(
        Arc<weapons::Content>,
        Arc<Tape<super::shot_drawing::InputCut>>,
    ),
    ShotExpected(
        Arc<weapons::Content>,
        Arc<Tape<super::shot_drawing::ExpectedCut>>,
    ),
    Camera,
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
}

pub(super) struct Expected {
    cache: Box<Cache>,
    recipe: Context,
}

#[derive(Clone)]
pub(super) enum Actual {
    Weapons,
    ShotDrawing,
    Camera,
    Appearance(Arc<appearance::Content>),
    Progress,
    Connected {
        content: Arc<super::connected::Content>,
    },
}

impl PartialEq for Actual {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Appearance(content) => {
                matches!(other, Self::Appearance(next) if Arc::ptr_eq(content, next))
            }
            Self::Connected { content } => {
                matches!(other, Self::Connected { content: next } if Arc::ptr_eq(content, next))
            }
            Self::Weapons | Self::ShotDrawing | Self::Camera | Self::Progress => {
                std::mem::discriminant(self) == std::mem::discriminant(other)
            }
        }
    }
}

impl Eq for Actual {}

struct Recipe {
    source: Context,
    actual: Actual,
    seed: ProductionSeed,
    content: ProductionContent,
    rules: ProductionRules,
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
        &sources.native,
        case.connected_packet
            .as_ref()
            .ok_or_else(|| data_failure(None, "connected packet declaration"))?,
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

fn weapons_recipe(
    wram: &[u8],
    sources: &Sources,
    case: &Case,
    cache: &Cache,
) -> Result<Recipe, Outcome> {
    let content = Arc::new(weapons::Content::load(&sources.rom)?);
    let seed = ProductionSeed::WeaponsContact(Box::new(weapons::seed(wram, &content)?));
    let production_content = ProductionContent::WeaponsContact(Box::new(content.weapons.clone()));
    let (source, actual) = if case.inputs == InputProvider::ScopedShotDrawingInputs {
        let fixtures = case.fixture_pair()?;
        (
            Context::ShotInputs(
                content,
                Arc::new(super::shot_drawing::InputCut::load(
                    case,
                    &sources.native,
                    &fixtures.inputs,
                    fixtures
                        .door_slots()
                        .ok_or_else(|| data_failure(None, "shot door roster"))?,
                )?),
            ),
            Actual::ShotDrawing,
        )
    } else {
        let window = case.cache_window()?;
        let first = *window.cuts.start();
        let visits = match case.ordinary_witness() {
            Some(Witness::DoorVisits { inputs, actors }) => {
                let observed =
                    super::shot_drawing::InputCut::load(case, &sources.native, inputs, actors)?;
                window
                    .cuts
                    .map(|cut| {
                        observed
                            .at(cut)?
                            .ordinary(&preceding(cache, cut)?.wram, &content)
                    })
                    .collect::<Result<_, _>>()?
            }
            _ => return Err(data_failure(None, "ordinary door visit source")),
        };
        (
            Context::DoorInputs(content, Arc::new(Tape::new(first, visits))),
            Actual::Weapons,
        )
    };
    Ok(Recipe {
        source,
        actual,
        seed,
        content: production_content,
        rules: ProductionRules::WeaponsContact(weapons_projectiles::Rules::REFERENCE),
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

    let (source, actual, seed, content, rules) = match case.responsibilities.as_slice() {
        [Responsibility::Weapons, Responsibility::Contact] => {
            return weapons_recipe(wram, sources, case, cache);
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
        [Responsibility::Appearance] => {
            let content = Arc::new(appearance::Content::load(&sources.rom)?);
            let seed = ProductionSeed::Appearance(appearance::seed(wram, &content)?);
            let clips = ProductionContent::Appearance(Box::new(content.appearance.clone()));
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
            ProductionSeed::Progress(Box::new(progress::decode(wram)?)),
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
    let inputs = Box::new(super::row_cache(
        sources,
        &case.row,
        case.end.cut_inclusive,
    )?);
    let expected = Box::new(super::row_cache(
        sources,
        &case.row,
        case.end.cut_inclusive,
    )?);
    if let Some(witness) = case.ordinary_witness() {
        witness.qualify_origin(&inputs)?;
    }
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
        })),
        expected: ExpectedReader::Recorded(Box::new(Expected {
            cache: expected,
            recipe: match recipe {
                Context::Connected { content, .. } => Context::ConnectedObservations(content),
                Context::DoorInputs(content, _) => Context::Weapons(content),
                Context::ShotInputs(content, _) => Context::ShotExpected(
                    content,
                    Arc::new(super::shot_drawing::ExpectedCut::load(
                        case,
                        &sources.native,
                        &case.fixture_pair()?.expected,
                    )?),
                ),
                recipe @ (Context::ShotExpected(..)
                | Context::Weapons(_)
                | Context::Camera
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
        if let Context::DoorInputs(_, tape) = &self.recipe {
            let input = tape.at(cut)?;
            return production
                .advance(AdvanceInput::WeaponsContact {
                    input: &input.update,
                    world: &input.world,
                    contacts: &input.opportunities,
                })
                .map_err(|fault| production_failure(cut, fault));
        }
        let prior = preceding(&self.cache, cut)?;
        let outputs = match &self.recipe {
            Context::ConnectedObservations(_)
            | Context::Weapons(_)
            | Context::ShotExpected(..)
            | Context::DoorInputs(..) => {
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
                })
            }
            Context::ShotInputs(content, tape) => {
                return super::shot_drawing::InputCut::advance(
                    tape,
                    &prior.wram,
                    content,
                    cut,
                    production,
                );
            }
            Context::Camera => {
                camera::validate_step(&prior.wram, &record(&self.cache, cut)?.wram)?;
                let input = camera::input(&prior.wram, &camera::Content)?;
                production.advance(AdvanceInput::Camera(&input.frame()))
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
                super::connected::expected(&current.wram, &content.appearance)?
            }
            Context::Connected { .. } | Context::DoorInputs(..) | Context::ShotInputs(..) => {
                return Err(data_failure(
                    Some(cut),
                    "input reader cannot read expectations",
                ));
            }
            Context::ShotExpected(content, tape) => {
                super::shot_drawing::ExpectedCut::read(tape, &current.wram, content, cut)?
            }
            Context::Weapons(content) => weapons::expected(&current.wram, content)?,
            Context::Camera => camera::expected(&current.wram)?,
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
                Self::Weapons,
                OwnedState::WeaponsContact(state),
                ProductionOutputs::WeaponsContact(outputs),
            ) => Ok(weapons::actual(&state.0, outputs)),
            (
                Self::ShotDrawing,
                OwnedState::WeaponsContact(state),
                ProductionOutputs::WeaponsContactDrawing(outputs),
            ) => super::shot_drawing::actual(&state.0, outputs, cut),
            (Self::Camera, OwnedState::Camera(state), ProductionOutputs::Camera(outputs)) => {
                Ok(camera::actual(&state.0, outputs))
            }
            (
                Self::Appearance(content),
                OwnedState::Appearance(state),
                ProductionOutputs::Appearance(_),
            ) => Ok(appearance::actual(&state.0, content)),
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
            (Self::Weapons, OwnedState::WeaponsContact(state)) => Ok(weapons::initial(&state.0)),
            (Self::ShotDrawing, OwnedState::WeaponsContact(state)) => {
                Ok(super::shot_drawing::held(&state.0))
            }
            (Self::Camera, OwnedState::Camera(state)) => Ok(camera::initial(&state.0)),
            (Self::Appearance(content), OwnedState::Appearance(state)) => {
                Ok(appearance::actual(&state.0, content))
            }
            (Self::Progress, OwnedState::Progress(state)) => Ok(progress::actual(state)),
            _ => Err(data_failure(None, "recorded initial scope")),
        }
    }
}
