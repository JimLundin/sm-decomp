//! Input, actual projection and expectation readers have disjoint access to their data.

use super::compare::Observation;
use super::index::{Case, ObservationId};
use super::result::Outcome;
use super::{
    AudioExpectedReader, AudioInputReader, actual_audio, actual_audio_from, add_publications,
    audio, data_failure, movement, production_failure,
};
use crate::cache::Cache;
use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules,
};
use game::state::owned::OwnedState;

pub(super) enum InputReader {
    Recorded(Box<super::recorded::Inputs>),
    Audio(Box<audio::InputTape>),
    Movement {
        cache: Box<Cache>,
        content: Box<movement::Content>,
    },
}

pub(super) enum ExpectedReader {
    Recorded(Box<super::recorded::Expected>),
    Audio(Box<audio::ExpectedTape>),
    Movement { cache: Box<Cache>, row: String },
}

#[derive(Clone)]
pub(super) enum ActualReader {
    Recorded(super::recorded::Actual),
    Audio {
        publications: Vec<audio::Publication>,
        replies: [u8; 4],
    },
    Movement,
}

pub(super) struct ExpectedCut {
    pub fields: Vec<Observation>,
    pub semantics: Vec<Observation>,
    pub semantic_count: usize,
    pub output_count: usize,
    pub replies: usize,
}

pub(super) struct AdmittedCase {
    pub production: Production,
    pub content: ProductionContent,
    pub rules: ProductionRules,
    pub inputs: InputReader,
    pub expected: ExpectedReader,
    pub actual: ActualReader,
    pub schedule: Vec<u64>,
}

pub(super) fn record(cache: &Cache, cut: u64) -> Result<oracle_record::FrameRecord, Outcome> {
    let entry = cache
        .entries()
        .iter()
        .find(|entry| entry.frame == crate::lock::Frame(cut))
        .ok_or_else(|| data_failure(Some(cut), "record cut"))?;
    cache
        .record(entry)
        .map_err(|_| data_failure(Some(cut), "record bytes"))
}

impl InputReader {
    pub fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        match self {
            Self::Recorded(reader) => reader.advance(production, cut),
            Self::Audio(tape) => {
                let stop = tape
                    .stops
                    .iter()
                    .find(|(at, _)| *at == cut)
                    .map(|(_, stop)| stop)
                    .ok_or_else(|| data_failure(Some(cut), "admitted audio clock stop"))?;
                AudioInputReader { tape }.advance(production, cut, stop)
            }
            Self::Movement { cache, content } => {
                let OwnedState::Movement(samus) = production.state() else {
                    return Err(data_failure(Some(cut), "movement owned scope"));
                };
                let inputs = movement::input(
                    &record(
                        cache,
                        cut.checked_sub(1)
                            .ok_or_else(|| data_failure(Some(cut), "movement preceding cut"))?,
                    )?
                    .wram,
                    samus.0.pose,
                    content,
                )?;
                let radius = content
                    .movement
                    .radius(samus.0.pose)
                    .ok_or_else(|| data_failure(Some(cut), "admitted movement body content"))?;
                // Production replaces these callbacks with its checked imported pose mapping.
                let body = |_| radius;
                let heading = |pose| content.movement.cannon_heading(pose);
                let frame = samus_movement::Frame {
                    sprite_before_movement: inputs.sprite,
                    cinematic_active: inputs.cinematic,
                    pad: inputs.pad,
                    pressed: inputs.pressed,
                    items: inputs.items,
                    spin_turned_far: inputs.ahead.turned_far,
                    elevator_moving: inputs.elevator,
                    parity: inputs.parity,
                    animation_ends: inputs.ahead.end,
                    body: &body,
                    cannon_heading: &heading,
                };
                production
                    .advance(AdvanceInput::Movement {
                        frame,
                        world: &inputs.world,
                    })
                    .map_err(|fault| production_failure(cut, fault))
            }
        }
    }
}

impl ActualReader {
    pub fn accept(
        &mut self,
        inputs: &InputReader,
        production: &Production,
        outputs: ProductionOutputs,
        cut: u64,
    ) -> Result<Vec<Observation>, Outcome> {
        match (self, inputs, outputs) {
            (Self::Recorded(actual), InputReader::Recorded(_), outputs) => {
                actual.accept(production, &outputs, cut)
            }
            (
                Self::Audio {
                    publications,
                    replies,
                },
                InputReader::Audio(tape),
                outputs @ ProductionOutputs::Audio(_),
            ) => {
                let before = publications.len();
                add_publications(tape, outputs, publications, replies, cut)?;
                actual_audio_from(production, publications, *replies, before)
            }
            (
                Self::Movement,
                InputReader::Movement { .. },
                ProductionOutputs::Movement(outputs),
            ) => {
                use types::Parts;
                let OwnedState::Movement(state) = production.state() else {
                    return Err(data_failure(Some(cut), "movement owned scope"));
                };
                let mut actual = vec![
                    Observation::quantity("movement.position.x", i64::from(state.0.position.x.0)),
                    Observation::quantity("movement.position.y", i64::from(state.0.position.y.0)),
                    Observation::quantity(
                        "movement.vertical_speed",
                        i64::from(state.0.vertical.speed.size().0),
                    ),
                ];
                actual.extend(outputs.travelled.iter().map(|(direction, travel)| {
                    Observation::quantity(
                        format!("movement.travel.{direction:?}"),
                        i64::from(travel.0.0),
                    )
                }));
                let wall = match outputs.wall_test {
                    samus_movement::WallTest::Neither => 0,
                    samus_movement::WallTest::Readied => 1,
                    samus_movement::WallTest::Kicked => 2,
                };
                actual.push(Observation::quantity("movement.wall_test", wall));
                actual.push(movement::reached(&outputs));
                Ok(actual)
            }
            _ => Err(data_failure(Some(cut), "actual projection scope")),
        }
    }

    pub fn initial(&self, production: &Production) -> Result<Vec<Observation>, Outcome> {
        match self {
            Self::Recorded(actual) => actual.initial(production),
            Self::Audio {
                publications,
                replies,
            } => actual_audio(production, publications, *replies),
            Self::Movement => {
                use types::Parts;
                let OwnedState::Movement(state) = production.state() else {
                    return Err(data_failure(None, "movement null scope"));
                };
                let mut null = vec![
                    Observation::quantity("movement.position.x", i64::from(state.0.position.x.0)),
                    Observation::quantity("movement.position.y", i64::from(state.0.position.y.0)),
                    Observation::quantity(
                        "movement.vertical_speed",
                        i64::from(state.0.vertical.speed.size().0),
                    ),
                ];
                null.extend(
                    ["Left", "Right", "Up", "Down"]
                        .into_iter()
                        .map(|direction| {
                            Observation::quantity(format!("movement.travel.{direction}"), 0)
                        }),
                );
                null.push(Observation::quantity("movement.wall_test", 0));
                null.push(Observation {
                    field: "movement.reached".to_owned(),
                    bytes: Vec::new(),
                });
                Ok(null)
            }
        }
    }

    pub const fn publication_count(&self) -> usize {
        match self {
            Self::Audio { publications, .. } => publications.len(),
            Self::Movement | Self::Recorded(_) => 0,
        }
    }
}

impl ExpectedReader {
    pub fn read(&self, case: &Case, cut: u64) -> Result<ExpectedCut, Outcome> {
        match self {
            Self::Recorded(reader) => reader.read(case, cut),
            Self::Audio(tape) => {
                let reader = AudioExpectedReader { tape };
                let capture = reader.at(cut)?;
                Ok(ExpectedCut {
                    fields: reader.observations(cut)?,
                    semantics: capture
                        .fields
                        .iter()
                        .map(|field| Observation::quantity(field.field.clone(), field.value))
                        .collect(),
                    semantic_count: capture.fields.len(),
                    output_count: 0,
                    replies: 4,
                })
            }
            Self::Movement { cache, row } => {
                let outputs_selected = case.expectations.output_cut.is_none_or(|at| at == cut);
                let selected: Vec<ObservationId> = case
                    .expectations
                    .fields
                    .iter()
                    .chain(
                        case.expectations
                            .outputs
                            .iter()
                            .filter(|_| outputs_selected),
                    )
                    .copied()
                    .collect();
                let fields = movement::expected(&record(cache, cut)?.wram, &selected, row, cut)?;
                Ok(ExpectedCut {
                    semantics: fields
                        .iter()
                        .take(case.expectations.fields.len())
                        .cloned()
                        .collect(),
                    fields,
                    semantic_count: case.expectations.fields.len(),
                    output_count: if outputs_selected {
                        case.expectations.outputs.len()
                    } else {
                        0
                    },
                    replies: 0,
                })
            }
        }
    }
}
