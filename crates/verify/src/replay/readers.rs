//! Input, actual projection and expectation readers have disjoint access to their data.

use super::compare::Observation;
use super::index::Case;
use super::result::Outcome;
use super::{audio, data_failure, movement};
use crate::cache::Cache;
use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules,
};
use game::state::owned::OwnedState;

pub(super) struct Tape<T> {
    first: u64,
    values: Vec<T>,
}
impl<T> Tape<T> {
    pub(super) const fn new(first: u64, values: Vec<T>) -> Self {
        Self { first, values }
    }

    pub(super) fn at(&self, cut: u64) -> Result<&T, Outcome> {
        cut.checked_sub(self.first)
            .and_then(|n| usize::try_from(n).ok())
            .and_then(|n| self.values.get(n))
            .ok_or_else(|| data_failure(Some(cut), "cut outside recorded tape"))
    }
}

pub(super) enum InputReader {
    Blackout(Tape<game::production::blackout::Cut>),
    Intro(super::played::intro::indexed::Inputs),
    IntroRooms(super::intro_rooms::Inputs),
    Flashback(super::flashback::Inputs),
    Playback(CutTape<game::production::playback::Cut>),
    SaveFiles(Box<super::save_files::Inputs>),
    IntroMissiles(super::intro_missiles::Inputs),
    Discovery(CutTape<game::production::played::discovery::Frame>),
    LandingSite(CutTape<game::production::played::landing_site::Work>),
    EarlyMap(Box<super::early_map::InputTape>),
    HealthTail(super::health_tail::Inputs),
    SenderOffer(Vec<audio_transport::Offer>),
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
    Blackout(Tape<ExpectedCut>),
    Intro(super::played::intro::indexed::Expected),
    RecordedCuts(CutTape<ExpectedCut>),
    SaveFiles(super::save_files::Expected),
    EarlyMap(Box<super::early_map::Expectations>),
    HealthTail(Vec<super::health_tail::Expected>),
    SenderOffer(Vec<Observation>),
    MovementTouchdown(super::movement_touchdown::Expected),
    NativeSearch(super::native_search::Expected),
    Recorded(Box<super::recorded::Expected>),
    Audio(Box<audio::ExpectedTape>),
    Movement(Box<Cache>),
}

#[derive(Clone, PartialEq, Eq)]
pub(super) enum ActualReader {
    Blackout(bool),
    StandingBody,
    ShaftHud,
    Intro,
    IntroRooms,
    Flashback,
    Playback,
    SaveFiles,
    IntroMissiles,
    Discovery,
    LandingSite,
    EarlyMap { carry_cut: u64 },
    HealthTail,
    SenderOffer(audio_transport::SoundGroup),
    MovementTouchdown(Option<Box<game::production::movement_touchdown::Outputs>>),
    NativeSearch(Option<Box<samus_movement::Outcome>>),
    Recorded(super::recorded::Actual),
    Audio(Vec<audio::Alias>),
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

pub(super) enum FieldRole {
    Semantic,
    Output,
}

impl ExpectedCut {
    pub(super) fn select(
        mut fields: Vec<Observation>,
        role: impl Fn(&str) -> Option<FieldRole>,
    ) -> Self {
        let mut semantics = Vec::new();
        fields.retain(|field| match role(&field.field) {
            Some(FieldRole::Semantic) => {
                semantics.push(field.clone());
                true
            }
            Some(FieldRole::Output) => true,
            None => false,
        });
        Self {
            semantic_count: semantics.len(),
            output_count: fields.len().saturating_sub(semantics.len()),
            semantics,
            fields,
            replies: 0,
        }
    }
}

pub(super) struct CutTape<T> {
    pub first: u64,
    pub values: Vec<T>,
}

impl<T> CutTape<T> {
    pub(super) fn at(&self, cut: u64) -> Result<&T, Outcome> {
        cut.checked_sub(self.first)
            .and_then(|offset| usize::try_from(offset).ok())
            .and_then(|index| self.values.get(index))
            .ok_or_else(|| data_failure(Some(cut), "recorded sequence cut"))
    }
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
            Self::Blackout(tape) => tape.advance(production, cut),
            Self::Intro(inputs) => inputs.advance(production, cut),
            Self::Flashback(inputs) => inputs.advance(production, cut),
            Self::Playback(inputs) => super::playback::advance(production, inputs, cut),
            Self::SaveFiles(inputs) => inputs.advance(production, cut),
            Self::IntroMissiles(inputs) => inputs.advance(production, cut),
            Self::Discovery(inputs) => production
                .advance(AdvanceInput::Discovery(*inputs.at(cut)?))
                .map_err(|fault| super::production_failure(cut, fault)),
            Self::LandingSite(inputs) => production
                .advance(AdvanceInput::LandingSite(*inputs.at(cut)?))
                .map_err(|fault| super::production_failure(cut, fault)),
            Self::EarlyMap(inputs) => inputs.advance(production, cut),
            Self::IntroRooms(inputs) => inputs.advance(production, cut),
            Self::HealthTail(inputs) => inputs
                .advance(production, cut)
                .map(ProductionOutputs::HealthTail),
            Self::MovementTouchdown(inputs) => inputs.advance(production, cut),
            Self::SenderOffer(offers) => super::sender_offer::advance(production, offers, cut),
            Self::NativeSearch(inputs) => inputs.advance(production, cut),
            Self::Recorded(reader) => reader.advance(production, cut),
            Self::Audio(tape) => audio::advance(production, tape, cut),
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
        movement::landing_pose(state.pose),
        movement::movement_pose(state.pose),
        movement::jump_part(state.jump_held),
        Observation::quantity("movement.base-speed", i64::from(state.run.speed.0)),
        Observation::quantity("movement.position.x", i64::from(state.position.x.0)),
        Observation::quantity("movement.position.y", i64::from(state.position.y.0)),
        Observation::quantity(
            "movement.vertical_speed",
            i64::from(state.vertical.speed.size().0),
        ),
    ]
}

fn movement_actual(
    production: &Production,
    outputs: &samus_movement::Outcome,
    cut: u64,
) -> Result<Vec<Observation>, Outcome> {
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
    actual.push(movement::wall(outputs.wall_test));
    actual.push(movement::reached(&outputs.reached));
    actual.push(Observation::quantity(
        "movement.landing.floor",
        i64::from(
            outputs
                .floor_response
                .is_some_and(|floor| floor.kind == types::FloorResponseKind::UprightTouchdown),
        ),
    ));
    Ok(actual)
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
            (Self::Intro, InputReader::Intro(_), ProductionOutputs::Intro(report)) => {
                super::played::intro::indexed::actual(production, report.as_deref())
            }
            (Self::IntroRooms, InputReader::IntroRooms(_), ProductionOutputs::IntroRooms(_)) => {
                super::intro_rooms::actual(production)
            }
            (Self::Flashback, InputReader::Flashback(_), ProductionOutputs::Flashback(outputs)) => {
                super::flashback::actual(production, &outputs, Some(cut))
            }
            (Self::Playback, InputReader::Playback(_), ProductionOutputs::Playback(outputs)) => {
                super::playback::actual(production, outputs)
            }
            (Self::SaveFiles, InputReader::SaveFiles(_), ProductionOutputs::SaveFiles(output)) => {
                Ok(vec![super::save_files::observed(output.image)])
            }
            (
                Self::IntroMissiles,
                InputReader::IntroMissiles(_),
                ProductionOutputs::IntroMissiles(output),
            ) => super::intro_missiles::actual(production, Some(&output)),
            (Self::Discovery, InputReader::Discovery(_), ProductionOutputs::Discovery(outputs)) => {
                super::discovery::actual(production, Some(&outputs))
            }
            (Self::ShaftHud, InputReader::Blackout(_), ProductionOutputs::Blackout(out)) => {
                super::blackout::hud::actual(production, &out.regular_publication)
            }
            (Self::StandingBody, InputReader::Blackout(_), ProductionOutputs::Blackout(out)) => {
                super::blackout::standing::actual(production, &out)
            }
            (Self::Blackout(late), InputReader::Blackout(_), ProductionOutputs::Blackout(out)) => {
                super::blackout::actual(production, &out, *late)
            }
            (Self::LandingSite, InputReader::LandingSite(_), ProductionOutputs::LandingSite) => {
                super::landing_site::actual(production)
            }
            (
                Self::HealthTail,
                InputReader::HealthTail(_),
                ProductionOutputs::HealthTail(outputs),
            ) => super::health_tail::actual(production, outputs),
            (
                Self::MovementTouchdown(retained),
                InputReader::MovementTouchdown(_),
                ProductionOutputs::MovementTouchdown(outputs),
            ) => {
                let fields = super::movement_touchdown::actual(production, &outputs)?;
                *retained = Some(outputs);
                Ok(fields)
            }
            (Self::SenderOffer(group), InputReader::SenderOffer(_), ProductionOutputs::Audio) => {
                super::sender_offer::actual(production, *group)
            }
            (
                Self::NativeSearch(retained),
                InputReader::NativeSearch(_),
                ProductionOutputs::Movement(outputs),
            ) => {
                let fields = super::touched_cells::actual(&outputs)?;
                *retained = Some(Box::new(outputs));
                Ok(fields)
            }
            (Self::Recorded(actual), InputReader::Recorded(_), outputs) => {
                actual.accept(production, &outputs, cut)
            }
            (Self::Audio(aliases), InputReader::Audio(_), ProductionOutputs::Audio) => {
                audio::actual(production, aliases)
            }
            (
                Self::EarlyMap { carry_cut },
                InputReader::EarlyMap(_),
                ProductionOutputs::Connected(_),
            ) => super::early_map::actual(production, Some(cut), cut == *carry_cut),
            (
                Self::Movement,
                InputReader::Movement { .. },
                ProductionOutputs::Movement(outputs),
            ) => movement_actual(production, &outputs, cut),
            _ => Err(data_failure(Some(cut), "actual projection scope")),
        }
    }

    /// Observe the carried State before sampling this cut, with no new actions or publications.
    pub fn held(&self, production: &Production, cut: u64) -> Result<Vec<Observation>, Outcome> {
        match self {
            Self::Intro => super::played::intro::indexed::initial(production),
            Self::Flashback => super::flashback::actual(
                production,
                &game::production::played::flashback::Publication::default(),
                None,
            ),
            Self::Playback => super::playback::initial(production),
            Self::SaveFiles => production
                .battery()
                .map(super::save_files::observed)
                .map(|battery| vec![battery])
                .ok_or_else(|| data_failure(None, "battery null scope")),
            Self::IntroMissiles => super::intro_missiles::actual(production, None),
            Self::Discovery => super::discovery::actual(production, None),
            Self::ShaftHud => super::blackout::hud::actual(production, &[]),
            Self::StandingBody => super::blackout::standing::actual(
                production,
                &game::production::blackout::Output::default(),
            ),
            Self::Blackout(late) => super::blackout::actual(
                production,
                &game::production::blackout::Output::default(),
                *late,
            ),
            Self::LandingSite => super::landing_site::actual(production),
            Self::EarlyMap { carry_cut } => {
                super::early_map::actual(production, None, cut == *carry_cut)
            }
            Self::IntroRooms => super::intro_rooms::actual(production),
            Self::HealthTail => super::health_tail::initial(production),
            Self::SenderOffer(group) => super::sender_offer::actual(production, *group),
            Self::MovementTouchdown(_) => super::movement_touchdown::initial(production),
            Self::NativeSearch(_) => super::touched_cells::initial(),
            Self::Recorded(actual) => actual.initial(production),
            Self::Audio(aliases) => audio::actual(production, aliases),
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
                null.push(Observation::quantity("movement.landing.floor", 0));
                null.push(Observation::quantity("movement.wall_test", 0));
                null.push(Observation {
                    field: "movement.reached".to_owned(),
                    bytes: Vec::new(),
                });
                Ok(null)
            }
        }
    }

    /// Whether a restored run published the same retained outputs as the direct one.
    pub fn same_retained_outputs(&self, direct: &Self) -> bool {
        match (self, direct) {
            (Self::MovementTouchdown(restored), Self::MovementTouchdown(direct)) => {
                restored == direct
            }
            (Self::MovementTouchdown(_), _) | (_, Self::MovementTouchdown(_)) => false,
            _ => true,
        }
    }
}

impl ExpectedReader {
    pub fn read(&self, case: &Case, cut: u64) -> Result<ExpectedCut, Outcome> {
        match self {
            Self::Intro(expected) => expected.read(cut),
            Self::Blackout(tape) => tape.at(cut).cloned(),
            Self::RecordedCuts(expected) => expected.at(cut).cloned(),
            Self::SaveFiles(expected) => expected.read(cut),
            Self::EarlyMap(expected) => expected.read(cut),
            Self::HealthTail(expected) => super::health_tail::Expected::read(expected, cut),
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
                let capture = tape
                    .cuts
                    .iter()
                    .find(|(number, _)| *number == cut)
                    .map(|(_, expected)| expected)
                    .ok_or_else(|| data_failure(Some(cut), "missing native expected cut"))?;
                Ok(ExpectedCut {
                    fields: capture.fields.clone(),
                    semantics: capture
                        .fields
                        .iter()
                        .take(capture.semantic_count)
                        .cloned()
                        .collect(),
                    semantic_count: capture.semantic_count,
                    // The replies compared, and the three request lines.
                    output_count: capture.replies.saturating_add(3),
                    replies: capture.replies,
                })
            }
            Self::Movement(cache) => movement::expected_cut(&record(cache, cut)?.wram, case, cut),
        }
    }
}
