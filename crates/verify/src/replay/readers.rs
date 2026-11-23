//! Input, actual projection and expectation readers have disjoint access to their data.

use super::compare::Observation;
use super::index::{Case, ObservationId};
use super::result::Outcome;
use super::{
    AudioExpectedReader, AudioInputReader, actual_audio, actual_audio_from, add_publications,
    audio, data_failure, movement,
};
use crate::cache::Cache;
use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules,
};
use game::state::owned::OwnedState;

pub(super) enum InputReader {
    EarlyMap(Box<super::early_map::InputTape>),
    HealthTail(super::health_tail::Inputs),
    PassageAudio(super::passage_audio::Input),
    SenderOffer(Vec<super::sender_offer::Offer>),
    MovementTouchdown(Box<super::movement_touchdown::Inputs>),
    NativeSearch(Box<super::native_search::Inputs>),
    Recorded(Box<super::recorded::Inputs>),
    Audio(Box<audio::InputTape>),
    Movement {
        cache: Box<Cache>,
        content: Box<movement::Content>,
    },
}

pub(super) enum ExpectedReader {
    EarlyMap(Box<super::early_map::Expectations>),
    HealthTail(Box<[super::health_tail::Expected; 2]>),
    PassageAudio(ExpectedCut),
    SenderOffer(Vec<Observation>),
    MovementTouchdown(super::movement_touchdown::Expected),
    NativeSearch(super::native_search::Expected),
    Recorded(Box<super::recorded::Expected>),
    Audio(Box<audio::ExpectedTape>),
    Movement { cache: Box<Cache>, row: String },
}

#[derive(Clone)]
pub(super) enum ActualReader {
    EarlyMap,
    HealthTail,
    PassageAudio,
    SenderOffer,
    MovementTouchdown(Option<Box<game::production::movement_touchdown::Outputs>>),
    NativeSearch(Option<Box<samus_movement::Outcome>>),
    Recorded(super::recorded::Actual),
    Audio {
        publications: Vec<audio::Publication>,
        replies: [u8; 4],
    },
    Movement,
}

#[derive(Clone)]
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
    pub fn pending_origin(&self, case: &Case, production: &Production) -> Option<Production> {
        (case.checkpoints && matches!(self, Self::Audio(_) | Self::PassageAudio(_)))
            .then(|| production.clone())
    }

    pub fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        match self {
            Self::EarlyMap(inputs) => inputs.advance(production, cut),
            Self::HealthTail(inputs) => inputs
                .advance(production, cut)
                .map(ProductionOutputs::HealthTail),
            Self::PassageAudio(input) => input.advance(production, cut),
            Self::MovementTouchdown(inputs) => inputs.advance(production, cut),
            Self::SenderOffer(offers) => super::sender_offer::advance(production, offers, cut),
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
                let initial_pose = samus.0.pose;
                let inputs = movement::input(
                    &record(
                        cache,
                        cut.checked_sub(1)
                            .ok_or_else(|| data_failure(Some(cut), "movement preceding cut"))?,
                    )?
                    .wram,
                    initial_pose,
                    content,
                )?;
                production
                    .advance(AdvanceInput::Movement {
                        inputs: inputs.stimuli(),
                        world: &inputs.world,
                    })
                    .map_err(|fault| {
                        movement::advance_failure(
                            cut,
                            initial_pose,
                            "admitted movement body content",
                            fault,
                        )
                    })
            }
        }
    }
}

fn movement_state_fields(state: &samus_movement::Samus) -> Vec<Observation> {
    use types::Parts;

    vec![
        Observation::quantity("movement.position.x", i64::from(state.position.x.0)),
        Observation::quantity("movement.position.y", i64::from(state.position.y.0)),
        Observation::quantity(
            "movement.vertical_speed",
            i64::from(state.vertical.speed.size().0),
        ),
    ]
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
            (
                Self::HealthTail,
                InputReader::HealthTail(_),
                ProductionOutputs::HealthTail(outputs),
            ) => super::health_tail::actual(production, &outputs, cut),
            (
                Self::MovementTouchdown(retained),
                InputReader::MovementTouchdown(_),
                ProductionOutputs::MovementTouchdown(outputs),
            ) => {
                let fields = super::movement_touchdown::actual(production, &outputs)?;
                *retained = Some(outputs);
                Ok(fields)
            }
            (
                Self::SenderOffer,
                InputReader::SenderOffer(_),
                ProductionOutputs::AudioSender(outputs),
            ) if outputs.is_empty() => super::sender_offer::actual(production),
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
            (Self::EarlyMap, InputReader::EarlyMap(_), ProductionOutputs::Connected(_)) => {
                super::early_map::actual(production, Some(cut))
            }
            (
                Self::Movement,
                InputReader::Movement { .. },
                ProductionOutputs::Movement(outputs),
            ) => {
                let OwnedState::Movement(state) = production.state() else {
                    return Err(data_failure(Some(cut), "movement owned scope"));
                };
                let mut actual = movement_state_fields(&state.0);
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
            (actual, inputs, output) => actual.accept_live_sound(inputs, production, output, cut),
        }
    }

    fn accept_live_sound(
        &self,
        inputs: &InputReader,
        production: &Production,
        output: ProductionOutputs,
        cut: u64,
    ) -> Result<Vec<Observation>, Outcome> {
        match (self, inputs, output) {
            (
                Self::PassageAudio,
                InputReader::PassageAudio(_),
                ProductionOutputs::PassageAudio(output),
            ) => super::passage_audio::actual(production, &output),
            _ => Err(data_failure(Some(cut), "actual projection scope")),
        }
    }

    pub fn initial(&self, production: &Production) -> Result<Vec<Observation>, Outcome> {
        match self {
            Self::EarlyMap => super::early_map::actual(production, None),
            Self::PassageAudio => super::passage_audio::initial(production),
            Self::HealthTail => super::health_tail::initial(production),
            Self::SenderOffer => super::sender_offer::actual(production),
            Self::MovementTouchdown(_) => super::movement_touchdown::initial(production),
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
                let OwnedState::Movement(state) = production.state() else {
                    return Err(data_failure(None, "movement null scope"));
                };
                let mut null = movement_state_fields(&state.0);
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
            (Self::MovementTouchdown(restored), Self::MovementTouchdown(direct)) => {
                restored == direct
            }
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
                Self::EarlyMap
                | Self::PassageAudio
                | Self::NativeSearch(_)
                | Self::Recorded(_)
                | Self::Movement
                | Self::SenderOffer
                | Self::HealthTail,
                Self::EarlyMap
                | Self::PassageAudio
                | Self::NativeSearch(_)
                | Self::Recorded(_)
                | Self::Movement
                | Self::SenderOffer
                | Self::HealthTail,
            ) => true,
            (Self::Audio { .. } | Self::MovementTouchdown(_), _)
            | (_, Self::Audio { .. } | Self::MovementTouchdown(_)) => false,
        }
    }

    pub const fn publication_count(&self) -> usize {
        match self {
            Self::Audio { publications, .. } => publications.len(),
            Self::EarlyMap
            | Self::PassageAudio
            | Self::NativeSearch(_)
            | Self::Movement
            | Self::Recorded(_)
            | Self::SenderOffer
            | Self::HealthTail
            | Self::MovementTouchdown(_) => 0,
        }
    }
}

impl ExpectedReader {
    pub fn read(&self, case: &Case, cut: u64) -> Result<ExpectedCut, Outcome> {
        match self {
            Self::EarlyMap(expected) => expected.read(cut),
            Self::HealthTail(expected) => super::health_tail::Expected::read(expected, cut),
            Self::PassageAudio(expected) => Ok(expected.clone()),
            Self::MovementTouchdown(expected) => expected.read(cut),
            Self::SenderOffer(fields) => Ok(ExpectedCut {
                fields: fields.clone(),
                semantics: fields.clone(),
                semantic_count: fields.len(),
                output_count: 0,
                replies: 0,
            }),
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
