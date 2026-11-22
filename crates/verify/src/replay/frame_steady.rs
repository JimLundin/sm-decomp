//! Independent admission and comparison of the bounded Steady Frame continuation.

use super::{
    Sources,
    compare::Observation,
    data_failure,
    frame_origin::admission::{completed_start, quiet_profile},
    index::{Case, Start},
    production_failure,
    readers::{ActualReader, AdmittedCase, ExpectedCut, ExpectedReader, InputReader},
    result::Outcome,
};
use crate::lock::Sha256;
use data::{
    dma::BusAddress,
    timing::{ChannelSlot, MasterTicks, Template},
};
use game::{
    production::{
        AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules,
        ProductionSeed,
        frame::{
            self, Caller, layer_blend::LayerBlendDefaults, opening_prelude::Opening,
            steady_prelude::Qualification,
        },
        timing::Progress,
    },
    state::{
        owned::OwnedState,
        video::{ColourMathLayers, ColourMathSelect, Layers},
    },
};
use serde::{Deserialize, de::DeserializeOwned};
use std::{marker::PhantomData, path::Path};
use world_haze::{HazeBands, HazeColour, HazeFade, HazeLevel, HazeState};

const PACKET: &str = "shot-steady-haze-v1";
const INPUT: &str = "steady-input.toml";
const CONTROL_INPUT: &str = "control-input.toml";
const INPUT_PIN: &str = "318dca4d3df84c9bdffca5d59306ca6fea2b28389bb635239086fd3782c3d6d8";
const CONTROL_INPUT_PIN: &str = "f4fa7465950f7df751b1ae8e1a1819862a79392780946c7af152d4e6263eea2c";
const EXPECTED_PIN: &str = "1e8aa4c4ec0f757069a5034c6b8badb79d39ee9d0dad3049f8d47037206b57da";
const CONTROL_EXPECTED_PIN: &str =
    "e93d061f731cc6366094886c8b86e6dc4e89303640180b6d8141fb7e5b0924a9";
const TEMPLATE_PIN: &str = "b3bb0bd0f0d51a6fe185fe16a76360f8b527333b3c19d2853fa18fad5903d243";
const MASK: u8 = 0x84;
const BANDS: BusAddress = BusAddress {
    bank: 0x7e,
    offset: 0x9d00,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceInput {
    cut: u64,
    fade: String,
    component: String,
    room_departing: bool,
    channel_slots: [u8; 2],
    transport_countdowns: [u16; 2],
    retained_band_levels: [u8; 16],
    main_screen_default: u8,
    sub_screen_default: u8,
    colour_math_select_default: u8,
    colour_math_layers_default: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlInput {
    first_band_level: u8,
    first_countdown: u16,
    window_two_left: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Expected {
    cut: u64,
    fade: String,
    retained_band_sha256: String,
    backdrop_components: [u8; 3],
    blend_intent: String,
    after_transport_countdowns: [u16; 2],
    after_blend: [u8; 9],
    relative_boundaries: [i64; 4],
    uses_physical_device: bool,
    #[serde(skip)]
    control: Option<ControlExpected>,
    #[serde(skip)]
    random: Option<super::frame_random_prelude::Expected>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlExpected {
    band_sha256: String,
    countdowns: [u16; 2],
    window_two: [u8; 4],
}

pub(super) fn member(root: &Path, name: &str, pin: &str) -> Result<String, Outcome> {
    let source = std::fs::read_to_string(root.join(name))
        .map_err(|_| data_failure(None, "Steady fixture member"))?;
    if Some(Sha256::of(source.as_bytes())) != Sha256::try_from(pin).ok() {
        return Err(data_failure(None, "Steady fixture pin"));
    }
    Ok(source)
}

pub(super) fn decode<T: DeserializeOwned>(
    root: &Path,
    name: &str,
    pin: &str,
) -> Result<T, Outcome> {
    toml::from_str(&member(root, name, pin)?)
        .map_err(|_| data_failure(None, "Steady fixture format"))
}

pub(super) fn field_count(case: &Case, cuts: usize) -> Option<usize> {
    let Start::NativeSeed { path, .. } = &case.start else {
        return None;
    };
    let is_control = path.ends_with(CONTROL_INPUT);
    let counts: &[usize] = if super::frame_random_prelude::Arm::for_case(case).is_some() {
        &[3, 5, 7, 8, 2]
    } else if is_control {
        &[2, 5, 7, 9]
    } else {
        &[3, 5, 7, 8]
    };
    Some(counts.get(..cuts)?.iter().sum())
}

fn layers(bits: u8) -> Layers {
    Layers {
        bits: u32::from(bits),
        members: PhantomData,
    }
}

fn read<const N: usize>(wram: &[u8], offset: usize) -> Result<[u8; N], Outcome> {
    wram.get(offset..offset.saturating_add(N))
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| data_failure(None, "Steady origin byte range"))
}

fn word(wram: &[u8], offset: usize) -> Result<u16, Outcome> {
    read(wram, offset).map(u16::from_le_bytes)
}

fn current_steady_slots(wram: &[u8]) -> Result<bool, Outcome> {
    Ok(word(wram, 0x0a78)? == 0
        && word(wram, 0x0592)? & 0xc000 == 0
        && word(wram, 0x18b0)? & 0x8000 != 0
        && matches!(
            read::<11>(wram, 0x18b4)?,
            [4, _, 0, _, 0, _, 0, _, 0, _, 128]
        )
        && word(wram, 0x18f0)? == 0xde74
        && read::<1>(wram, 0x18fc)? == [0x88]
        && word(wram, 0x18fa)? == 0x84b8
        && read::<1>(wram, 0x1906)? == [0x88]
        && word(wram, 0x1914)? == 16
        && read::<1>(wram, 0x1920)? == [0x80])
}

impl SourceInput {
    fn qualification(
        self,
        opening: &Opening,
        wram: &[u8],
        [retain, finish, layers_profile]: [Template; 3],
        control: Option<ControlInput>,
    ) -> Result<Qualification, Outcome> {
        let mut backing: [u8; 16] = read(wram, usize::from(BANDS.offset))?;
        let mut window_two: [u8; 4] = read(wram, 0x65)?;
        let mut countdowns = self.transport_countdowns;
        let mut levels = self.retained_band_levels;
        let recorded_countdowns = [word(wram, 0x18e4)?, word(wram, 0x18ee)?];
        if self.cut != 156
            || self.fade != "Steady"
            || self.component != "Blue"
            || self.room_departing
            || !current_steady_slots(wram)?
            || self.channel_slots != [3, 8]
            || backing.map(|band| band & 31) != levels
            || recorded_countdowns != countdowns
        {
            return Err(data_failure(None, "Steady origin join"));
        }
        if let Some(control) = control {
            if control.first_band_level > 15 {
                return Err(data_failure(None, "Steady control intensity"));
            }
            backing[0] = (backing[0] & !31) | control.first_band_level;
            levels[0] = control.first_band_level;
            countdowns[0] = control.first_countdown;
            window_two[0] = control.window_two_left;
        }
        let mut bands =
            [HazeLevel::new(0).ok_or_else(|| data_failure(None, "Haze level domain"))?; 16];
        for (band, level) in bands.iter_mut().zip(levels) {
            *band =
                HazeLevel::new(level).ok_or_else(|| data_failure(None, "retained band level"))?;
        }
        let hardware = &opening.timing().hardware.transfers.channels;
        let [Some(three), Some(eight)] = [ChannelSlot::Three, ChannelSlot::Eight]
            .map(|slot| hardware.get(slot.ordinal()).cloned())
        else {
            return Err(data_failure(None, "Steady hardware channel slots"));
        };
        let [software_mask] = read(wram, 0x85)?;
        Ok(Qualification {
            model: HazeState {
                colour: HazeColour::Blue,
                fade: HazeFade::Steady,
                bands: HazeBands(bands),
            },
            countdowns,
            backing,
            bands_address: BANDS,
            physical_channels: [three, eight],
            software_mask,
            window_two,
            layer_defaults: LayerBlendDefaults {
                main_screen: layers(self.main_screen_default),
                sub_screen: layers(self.sub_screen_default),
                colour_math_select: ColourMathSelect(self.colour_math_select_default),
                colour_math_layers: ColourMathLayers(self.colour_math_layers_default),
            },
            retain,
            finish,
            layers: layers_profile,
        })
    }
}

fn input_side(
    case: &Case,
    sources: &Sources,
) -> Result<(Opening, frame::opening_prelude::Profiles, bool), Outcome> {
    let Start::NativeSeed { path, sha256 } = &case.start else {
        return Err(data_failure(None, "Steady start contract"));
    };
    let relative = Path::new(PACKET);
    let is_control = path == &relative.join(CONTROL_INPUT) && sha256 == CONTROL_INPUT_PIN;
    let random = super::frame_random_prelude::Arm::for_case(case);
    if random.is_none() && !is_control && (path != &relative.join(INPUT) || sha256 != INPUT_PIN) {
        return Err(data_failure(None, "Steady start identity"));
    }
    let root = sources.native.join(relative);
    let (opening, mut profiles, wram) = completed_start::replay_opening(sources)?;
    let input: SourceInput = decode(&root, INPUT, INPUT_PIN)?;
    let control = is_control
        .then(|| decode(&root, CONTROL_INPUT, CONTROL_INPUT_PIN))
        .transpose()?;
    let physical = quiet_profile::steady(&member(
        &root,
        "steady-prelude-templates.toml",
        TEMPLATE_PIN,
    )?)
    .map_err(|_| data_failure(None, "reached Steady profiles"))?;
    let qualification = input.qualification(&opening, &wram, physical, control)?;
    let mut opening = opening
        .with_steady(&qualification)
        .map_err(|_| data_failure(None, "Steady production admission"))?;
    profiles.steady = Some(Box::new(qualification));
    if let Some(arm) = random {
        let qualification = arm.input(sources, &wram)?;
        opening = opening
            .with_random(&qualification)
            .map_err(|_| data_failure(None, "current Random admission"))?;
        profiles.random = Some(Box::new(qualification));
    }
    Ok((opening, profiles, is_control))
}

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let (opening, profiles, is_control) = input_side(case, sources)?;
    let content = ProductionContent::FrameOpening(Box::new(profiles));
    let rules = ProductionRules::FrameOpening;
    let production = Production::admit(
        ProductionSeed::FrameOpening(Box::new(opening)),
        content.clone(),
        rules.clone(),
    )
    .map_err(|_| data_failure(None, "Steady carried Frame admission"))?;
    let root = sources.native.join(PACKET);
    let mut expected: Expected = decode(&root, "steady-expected.toml", EXPECTED_PIN)?;
    if expected.cut != 157
        || expected.fade != "Steady"
        || expected.blend_intent != "NoBackgroundThree"
        || expected.uses_physical_device
    {
        return Err(data_failure(None, "Steady observation contract"));
    }
    if is_control {
        expected.control = Some(decode(
            &root,
            "control-expected.toml",
            CONTROL_EXPECTED_PIN,
        )?);
    }
    expected.random = super::frame_random_prelude::Arm::for_case(case)
        .map(|arm| arm.expected(sources))
        .transpose()?;
    Ok(AdmittedCase {
        production,
        content,
        rules,
        inputs: InputReader::FrameSteady,
        expected: ExpectedReader::FrameSteady(Box::new(expected)),
        actual: ActualReader::FrameSteady { publications: 0 },
        schedule: (0..=case.end.cut_inclusive).collect(),
    })
}

fn owner(production: &Production) -> Result<&Opening, Outcome> {
    let OwnedState::FrameOpening(owner) = production.state() else {
        return Err(data_failure(None, "Steady actual owner"));
    };
    Ok(owner)
}

const fn stage(caller: &Caller, cut: u64) -> bool {
    matches!(
        (cut, caller),
        (0, Caller::Haze { .. })
            | (1, Caller::HazeTail { .. })
            | (2, Caller::LayerBlend { .. })
            | (3, Caller::Random { .. })
    ) || cut == 4
        && matches!(
            caller,
            Caller::Preparation {
                stage: frame::PreparationStage::Builder,
                charging: false,
                ..
            }
        )
}

pub(super) fn advance(production: &mut Production, cut: u64) -> Result<ProductionOutputs, Outcome> {
    advance_to(production, cut).map(|(output, _)| output)
}

fn advance_to(
    production: &mut Production,
    cut: u64,
) -> Result<(ProductionOutputs, usize), Outcome> {
    let mut previous = owner(production)?.timing().hardware.beam.refreshed;
    let mut refreshes = 0_usize;
    let mut transfers = Vec::new();
    for _ in 0..2048 {
        let result = production
            .advance(AdvanceInput::Frame {
                budget: MasterTicks(64),
            })
            .map_err(|fault| production_failure(cut, fault))?;
        let ProductionOutputs::FrameOpening(
            progress @ (Progress::Advanced | Progress::Completed(_)),
            events,
        ) = result
        else {
            return Err(data_failure(Some(cut), "quiet actual Steady progress"));
        };
        if cut != 4 && !events.is_empty() {
            return Err(data_failure(Some(cut), "quiet actual Steady progress"));
        }
        transfers.extend(events);
        let owner = owner(production)?;
        let current = owner.timing().hardware.beam.refreshed;
        refreshes = refreshes.saturating_add(usize::from(current && !previous));
        previous = current;
        if stage(owner.caller(), cut) {
            return Ok((
                ProductionOutputs::FrameOpening(progress, transfers),
                refreshes,
            ));
        }
    }
    Err(data_failure(Some(cut), "Steady progress bound"))
}

fn bytes(field: &str, value: impl Into<Vec<u8>>) -> Observation {
    Observation {
        field: field.to_owned(),
        bytes: value.into(),
    }
}

fn countdowns(values: [u16; 2]) -> [u8; 4] {
    let [[first_low, first_high], [last_low, last_high]] = values.map(u16::to_le_bytes);
    [first_low, first_high, last_low, last_high]
}

pub(super) fn actual(production: &Production) -> Result<Vec<Observation>, Outcome> {
    let owner = owner(production)?;
    let observed = owner
        .steady_observation()
        .ok_or_else(|| data_failure(None, "live Steady observations"))?;
    let now = i64::try_from(owner.timing().hardware.now.0)
        .map_err(|_| data_failure(None, "wall time range"))?;
    let mut fields = vec![
        Observation::quantity("frame.time", now),
        bytes("frame.steady.backing", Sha256::of(&observed.backing).0),
        bytes("frame.steady.physical_mask", [observed.physical_mask]),
        bytes("frame.steady.software_mask", [observed.software_mask]),
        bytes("frame.steady.timers", countdowns(observed.countdowns)),
        bytes("frame.steady.window_two", observed.window_two),
    ];
    if let Some(seed) = owner.random_seed() {
        fields.push(Observation::quantity(
            "frame.random.seed",
            i64::from(seed.0),
        ));
    }
    if let Caller::HazeTail { prepared, .. }
    | Caller::LayerBlend { haze: prepared, .. }
    | Caller::Random { haze: prepared, .. } = owner.caller()
    {
        fields.push(bytes(
            "frame.steady.fixed_colour",
            prepared.fixed_colour_bytes(),
        ));
        let frame::borrowed_prelude_receipts::LayerBlendDisposition::NoBackgroundThree =
            prepared.blend();
        fields.push(bytes(
            "frame.steady.blend_intent",
            b"NoBackgroundThree".as_slice(),
        ));
    }
    if let Caller::Random { layers, .. } = owner.caller() {
        let settings = frame::layer_blend::encode(layers.terminal)
            .map_err(|_| data_failure(None, "actual layer mirrors"))?;
        fields.push(bytes("frame.steady.layer_mirrors", settings));
    }
    Ok(fields)
}

impl Expected {
    pub(super) fn read(&self, cut: u64) -> Result<ExpectedCut, Outcome> {
        if cut == 4
            && let Some(random) = &self.random
        {
            return Ok(random.read());
        }
        let index =
            usize::try_from(cut).map_err(|_| data_failure(Some(cut), "Steady cut range"))?;
        let now = *self
            .relative_boundaries
            .get(index)
            .ok_or_else(|| data_failure(Some(cut), "Steady observation cut"))?;
        let hash = self
            .control
            .as_ref()
            .map_or(self.retained_band_sha256.as_str(), |control| {
                &control.band_sha256
            });
        let hash =
            Sha256::try_from(hash).map_err(|_| data_failure(Some(cut), "retained band digest"))?;
        let mut fields = vec![
            Observation::quantity("frame.time", now),
            bytes("frame.steady.physical_mask", [MASK]),
        ];
        if cut > 0 || self.control.is_none() {
            fields.push(bytes("frame.steady.backing", hash.0));
        }
        if cut >= 1 {
            fields.push(bytes("frame.steady.fixed_colour", self.backdrop_components));
            fields.push(bytes(
                "frame.steady.blend_intent",
                self.blend_intent.as_bytes(),
            ));
        }
        if cut >= 2 {
            let timers = self
                .control
                .as_ref()
                .map_or(self.after_transport_countdowns, |control| {
                    control.countdowns
                });
            fields.push(bytes("frame.steady.software_mask", [MASK]));
            fields.push(bytes("frame.steady.timers", countdowns(timers)));
        }
        if cut >= 3 {
            fields.push(bytes("frame.steady.layer_mirrors", self.after_blend));
            if let Some(control) = &self.control {
                fields.push(bytes("frame.steady.window_two", control.window_two));
            }
        }
        Ok(ExpectedCut {
            semantic_count: fields.len(),
            semantics: fields.clone(),
            fields,
            output_count: 0,
            replies: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::index::Index;
    use game::production::{ProductionFault, Unsupported};

    fn sources() -> Sources {
        Sources {
            native: std::env::var_os("SM_AUDIO_NATIVE").unwrap().into(),
            rom: std::env::var_os("SM_ROM").unwrap().into(),
            ..Sources::default()
        }
    }

    fn cases() -> Vec<Case> {
        Index::read(Path::new("../../scenarios/replay/index.toml"))
            .unwrap()
            .cases
            .into_iter()
            .filter(|case| {
                case.inputs == super::super::index::InputProvider::NativeSteadyPrelude
                    && super::super::frame_random_prelude::Arm::for_case(case).is_none()
            })
            .collect()
    }

    fn charged(production: &mut Production) {
        for _ in 0..32 {
            production
                .advance(AdvanceInput::Frame {
                    budget: MasterTicks(1),
                })
                .unwrap();
            if owner(production).unwrap().timing().hardware.charge_debt > 0 {
                return;
            }
        }
        panic!("no interior charge");
    }

    #[test]
    #[ignore = "requires the independently reviewed true-resume and Steady fixtures"]
    fn indexed_steady_boundaries_and_interior_restores() {
        let cases = cases();
        assert_eq!(cases.len(), 2);
        let sources = sources();
        for case in cases {
            let result = super::super::run(&case, &sources);
            assert!(
                matches!(result.outcome, Outcome::Unsupported { cut: Some(4), .. }),
                "{:?}",
                result.outcome
            );
            assert_eq!(result.counts.cuts, 4);
            assert_eq!(result.counts.restorations, 4);
            assert_eq!(result.counts.null_cuts, 0);

            let admitted = admit(&case, &sources).unwrap();
            let mut measure = admitted.production.clone();
            advance(&mut measure, 0).unwrap();
            for (cut, expected_refreshes) in [(1, 0), (2, 1), (3, 1)] {
                let (_, refreshes) = advance_to(&mut measure, cut).unwrap();
                assert_eq!(refreshes, expected_refreshes, "physical refresh cut {cut}");
            }
            let mut direct = admitted.production;
            advance(&mut direct, 0).unwrap();
            for cut in 1..=3 {
                charged(&mut direct);
                let saved = direct.save().unwrap();
                let mut restored =
                    Production::restore(&saved, admitted.content.clone(), admitted.rules.clone())
                        .unwrap();
                let outputs = advance(&mut direct, cut).unwrap();
                assert!(outputs.eq(&advance(&mut restored, cut).unwrap()));
                assert!(direct.eq(&restored));
            }
            let before = direct.save().unwrap();
            assert!(matches!(
                direct.advance(AdvanceInput::Frame {
                    budget: MasterTicks(64)
                }),
                Err(ProductionFault::Unsupported(Unsupported::Frame(
                    frame::Dependency::RandomPreparation
                )))
            ));
            assert!(before.eq(&direct.save().unwrap()));
        }
    }

    #[test]
    #[ignore = "requires independent layer defaults and complete native mirror observations"]
    fn actual_layer_operand_fault_has_the_same_wall_time() {
        let case = cases()
            .into_iter()
            .find(|case| case.id.0 == "frame/steady-haze-through-layer-blend")
            .unwrap();
        let sources = sources();
        let (opening, mut profiles, _) = input_side(&case, &sources).unwrap();
        profiles
            .steady
            .as_mut()
            .unwrap()
            .layer_defaults
            .colour_math_layers
            .0 ^= 1;
        let mut admission = admit(&case, &sources).unwrap();
        let mut production = Production::admit(
            ProductionSeed::FrameOpening(Box::new(opening)),
            ProductionContent::FrameOpening(Box::new(profiles)),
            admission.rules,
        )
        .unwrap();
        for cut in 0..3 {
            super::super::runner::execute(
                &case,
                &admission.inputs,
                &admission.expected,
                &mut production,
                &mut admission.actual,
                cut,
            )
            .unwrap();
        }
        let outcome = super::super::runner::execute(
            &case,
            &admission.inputs,
            &admission.expected,
            &mut production,
            &mut admission.actual,
            3,
        );
        assert!(
            matches!(outcome, Err(Outcome::Difference { cut: 3, field }) if field == "frame.steady.layer_mirrors")
        );
    }

    #[test]
    #[ignore = "requires the independently qualified Random arms"]
    fn random_live_requests_resume_after_first_sample() {
        let index = Index::read(Path::new("../../scenarios/replay/index.toml")).unwrap();
        let sources = sources();
        let cases = index
            .cases
            .into_iter()
            .filter(|case| super::super::frame_random_prelude::Arm::for_case(case).is_some());
        let mut witnessed = 0_usize;
        for case in cases {
            let admitted = admit(&case, &sources).unwrap();
            let mut direct = admitted.production;
            for cut in 0..=3 {
                advance(&mut direct, cut).unwrap();
            }
            let mut measurement = direct.clone();
            assert_eq!(advance_to(&mut measurement, 4).unwrap().1, 1);
            restore_random_intervals(&mut direct, &admitted.content, &admitted.rules);
            let completed = direct.save().unwrap();
            assert!(matches!(
                direct.advance(AdvanceInput::Frame {
                    budget: MasterTicks(64)
                }),
                Err(ProductionFault::Unsupported(Unsupported::Frame(
                    frame::Dependency::BuilderPreparation
                )))
            ));
            assert!(completed.eq(&direct.save().unwrap()));
            let result = super::super::run(&case, &sources);
            assert!(
                matches!(result.outcome, Outcome::Unsupported { cut: Some(5), .. }),
                "{:?}",
                result.outcome
            );
            assert_eq!(result.counts.cuts, 5);
            assert_eq!(result.counts.restorations, 5);
            assert_eq!(result.counts.outputs, 7);
            assert_eq!(result.counts.publications, 4);
            witnessed = witnessed.saturating_add(1);
        }
        assert_eq!(witnessed, 3);
    }

    fn restore_random_intervals(
        direct: &mut Production,
        content: &ProductionContent,
        rules: &ProductionRules,
    ) {
        use data::timing::Sample;
        use game::production::timing::{EventKind, SampleValue};
        let mut restored: Option<Production> = None;
        let mut checkpoints = 0_usize;
        for _ in 0..4096 {
            let input = AdvanceInput::Frame {
                budget: MasterTicks(1),
            };
            let output = direct.advance(input).unwrap();
            if let Some(prior) = &mut restored {
                assert!(
                    output.eq(&prior
                        .advance(AdvanceInput::Frame {
                            budget: MasterTicks(1)
                        })
                        .unwrap())
                );
                assert!(Production::eq(direct, prior));
            }
            let ProductionOutputs::FrameOpening(_, events) = output else {
                panic!("expected the real Frame output");
            };
            let caller = owner(direct).unwrap().caller();
            let first_sample = matches!(
                caller,
                Caller::RandomRunning {
                    samples: frame::random::Samples::FirstLow(_),
                    ..
                }
            );
            if events.iter().any(|event| {
                matches!(event.kind, EventKind::Publication(_))
                    || first_sample
                        && matches!(
                            event.kind,
                            EventKind::Sample(Sample::ProductLow, SampleValue::Byte(_))
                        )
            }) {
                let saved = direct.save().unwrap();
                let resumed = Production::restore(&saved, content.clone(), rules.clone()).unwrap();
                assert!(Production::eq(direct, &resumed));
                restored = Some(resumed);
                checkpoints = checkpoints.saturating_add(1);
            }
            if stage(caller, 4) {
                assert_eq!(checkpoints, 5);
                return;
            }
        }
        panic!("Random failed to return to Builder");
    }
}
