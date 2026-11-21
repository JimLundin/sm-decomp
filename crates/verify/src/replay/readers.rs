//! Input, actual projection and expectation readers have disjoint access to their data.

use super::compare::Observation;
use super::frame::HeldAdvance;
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
    SenderOffer(Vec<super::sender_offer::Offer>),
    Frame(super::frame::Inputs),
    NativeSearch(Box<super::native_search::Inputs>),
    Recorded(Box<super::recorded::Inputs>),
    Audio(Box<audio::InputTape>),
    Movement {
        cache: Box<Cache>,
        content: Box<movement::Content>,
    },
}

pub(super) enum ExpectedReader {
    SenderOffer(Vec<Observation>),
    Frame(Box<super::frame::Expected>),
    NativeSearch(super::native_search::Expected),
    Recorded(Box<super::recorded::Expected>),
    Audio(Box<audio::ExpectedTape>),
    Movement { cache: Box<Cache>, row: String },
}

#[derive(Clone)]
pub(super) enum ActualReader {
    SenderOffer,
    Frame(super::frame::Actual),
    NativeSearch(Option<Box<samus_movement::Outcome>>),
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
    #[cfg(test)]
    pub fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        self.advance_observing_stop(production, cut, &mut HeldAdvance::default())
    }

    pub fn advance_observing_stop(
        &self,
        production: &mut Production,
        cut: u64,
        held: &mut HeldAdvance,
    ) -> Result<ProductionOutputs, Outcome> {
        match self {
            Self::SenderOffer(offers) => super::sender_offer::advance(production, offers, cut),
            Self::Frame(inputs) => inputs.advance(production, cut, held),
            Self::NativeSearch(inputs) => inputs.advance(production, cut),
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
    #[cfg(test)]
    pub fn accept(
        &mut self,
        inputs: &InputReader,
        production: &Production,
        outputs: ProductionOutputs,
        cut: u64,
    ) -> Result<Vec<Observation>, Outcome> {
        self.accept_observing_stop(
            inputs,
            production,
            outputs,
            cut,
            &mut HeldAdvance::default(),
        )
    }

    pub fn accept_observing_stop(
        &mut self,
        inputs: &InputReader,
        production: &Production,
        outputs: ProductionOutputs,
        cut: u64,
        held: &mut HeldAdvance,
    ) -> Result<Vec<Observation>, Outcome> {
        match (self, inputs, outputs) {
            (
                Self::SenderOffer,
                InputReader::SenderOffer(_),
                ProductionOutputs::AudioSender(outputs),
            ) if outputs.is_empty() => super::sender_offer::actual(production),
            (Self::Frame(actual), InputReader::Frame(_), ProductionOutputs::Frame(outputs)) => {
                actual
                    .accept(production, &outputs, cut)
                    .inspect_err(|_| held.retain_events(outputs.events))
            }
            (
                Self::NativeSearch(retained),
                InputReader::NativeSearch(_),
                ProductionOutputs::Movement(outputs),
            ) => {
                let mut fields = super::native_search::actual(&outputs);
                fields.extend(super::full_movement::actual(&outputs)?);
                *retained = Some(Box::new(outputs));
                Ok(fields)
            }
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
            Self::SenderOffer => super::sender_offer::actual(production),
            Self::Frame(actual) => actual.initial(production),
            Self::NativeSearch(_) => {
                let mut fields = super::native_search::initial();
                fields.extend(super::full_movement::initial()?);
                Ok(fields)
            }
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

    pub fn same_publication_prefix(&self, direct: &Self) -> bool {
        match (self, direct) {
            (Self::Frame(restored), Self::Frame(direct)) => restored == direct,
            (
                Self::Audio {
                    publications: restored,
                    replies: restored_replies,
                },
                Self::Audio {
                    publications: direct,
                    replies: direct_replies,
                },
            ) => restored == direct && restored_replies == direct_replies,
            (
                Self::NativeSearch(_) | Self::Recorded(_) | Self::Movement | Self::SenderOffer,
                Self::NativeSearch(_) | Self::Recorded(_) | Self::Movement | Self::SenderOffer,
            ) => true,
            (Self::Frame(_) | Self::Audio { .. }, _) | (_, Self::Frame(_) | Self::Audio { .. }) => {
                false
            }
        }
    }

    pub const fn publication_count(&self) -> usize {
        match self {
            Self::Frame(actual) => actual.publication_count(),
            Self::Audio { publications, .. } => publications.len(),
            Self::NativeSearch(_) | Self::Movement | Self::Recorded(_) | Self::SenderOffer => 0,
        }
    }
}

impl ExpectedReader {
    pub fn read(&self, case: &Case, cut: u64) -> Result<ExpectedCut, Outcome> {
        match self {
            Self::SenderOffer(fields) => Ok(ExpectedCut {
                fields: fields.clone(),
                semantics: fields.clone(),
                semantic_count: fields.len(),
                output_count: 0,
                replies: 0,
            }),
            Self::Frame(reader) => reader.read(cut, case.compares_final_frame_health()),
            Self::NativeSearch(expected) => {
                let fields = expected.read(cut)?;
                Ok(ExpectedCut {
                    semantics: fields.clone(),
                    semantic_count: fields.len(),
                    output_count: 0,
                    replies: 0,
                    fields,
                })
            }
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
