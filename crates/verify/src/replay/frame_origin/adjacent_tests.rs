//! Independent adjacent cut partitions and the production replay seam.

use super::super::{
    Sources, compared,
    frame::{Datum, Inputs},
    index::{Case, Index},
    production_failure,
    readers::{AdmittedCase, ExpectedReader, InputReader},
    result::Outcome,
    runner,
};
use crate::lock::Sha256;
use data::timing::{MasterTicks, Sample};
use game::{
    production::{
        AdvanceInput, Production, ProductionContent, ProductionFault, ProductionOutputs,
        Unsupported,
        frame::{Caller, Dependency},
        timing::EventKind,
        timing::SampleValue,
    },
    state::owned::OwnedState,
};
use std::path::Path;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Controller {
    origin: [u16; 2],
    first: [u16; 2],
    second: [u16; 2],
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Controllers {
    actual: Controller,
    held: Controller,
}

fn read_control(native: &Path) -> Controllers {
    let bytes =
        std::fs::read(native.join("adjacent-native-frame-v1/expected/controller.toml")).unwrap();
    assert_eq!(
        Sha256::of(&bytes).to_string(),
        "befed8dd09f6bfc7f65e2c5d72a228ee978f0c927273c5e721edbffd87586b85"
    );
    toml::from_str(std::str::from_utf8(&bytes).unwrap()).unwrap()
}

fn observe_first_return(case: &Case, admitted: &mut AdmittedCase, controller: &Controller) {
    let OwnedState::Frame(origin) = admitted.production.state() else {
        panic!("frame production");
    };
    assert_observation_eq!(
        origin.timing.hardware.controller.words.first().copied(),
        Some(controller.origin[0])
    );
    assert_observation_eq!(
        origin.frame_history.as_ref().unwrap().controller(),
        controller.origin
    );
    let cut = case.first_cut().unwrap();
    let ProductionOutputs::Frame(outputs) = admitted
        .inputs
        .advance(&mut admitted.production, cut)
        .unwrap()
    else {
        panic!("frame outputs");
    };
    let samples: Vec<_> = outputs
        .events
        .iter()
        .filter_map(|event| {
            if let EventKind::Sample(
                Sample::ControllerLow | Sample::ControllerHigh,
                SampleValue::Byte(value),
            ) = event.kind
            {
                Some(value)
            } else {
                None
            }
        })
        .collect();
    assert_observation_eq!(samples, controller.first[0].to_le_bytes());
    assert_observation_eq!(
        native_controller(&admitted.production),
        Some(controller.first)
    );
    let observed = admitted
        .actual
        .accept(
            &admitted.inputs,
            &admitted.production,
            ProductionOutputs::Frame(outputs),
            cut,
        )
        .unwrap();
    let expected = admitted.expected.read(case, cut).unwrap();
    compared(cut, &expected.fields, &observed).unwrap();
    assert_eq!(admitted.actual.publication_count(), 2_295);
}

fn compare_negative_controls(case: &Case, reader: &ExpectedReader) {
    let first = reader.read(case, case.first_cut().unwrap()).unwrap();
    let cut = case.end.cut_inclusive;
    let second = reader.read(case, cut).unwrap();
    let callback_bytes = &second.fields.first().unwrap().bytes;
    let callbacks: Vec<Datum> = canonical::from_bytes(callback_bytes).unwrap();
    let time: MasterTicks = canonical::from_bytes(&second.fields[1].bytes).unwrap();
    assert_eq!(callbacks.len(), 2_295);
    assert!(
        callbacks
            .iter()
            .all(|callback| { callback.at.0 > 359_922 && callback.at.0 <= time.0 })
    );
    assert_eq!(
        compared(cut, &second.fields, &first.fields),
        Err(Outcome::Difference {
            cut,
            field: "frame.outputs[0].at".to_owned()
        })
    );
    let mut wrong_stream = second.fields.clone();
    let mut duplicate = callbacks;
    duplicate.push(duplicate.last().unwrap().clone());
    wrong_stream.first_mut().unwrap().bytes = canonical::to_bytes(&duplicate);
    assert_eq!(
        compared(cut, &second.fields, &wrong_stream),
        Err(Outcome::Difference {
            cut,
            field: "frame.outputs.count".to_owned()
        })
    );
    let mut stale = second.fields;
    let field = "frame.play_time.frames";
    let prior = first
        .fields
        .iter()
        .find(|value| value.field == field)
        .unwrap();
    let actual = stale.iter_mut().find(|value| value.field == field).unwrap();
    actual.bytes.clone_from(&prior.bytes);
    assert_eq!(
        compared(cut, &reader.read(case, cut).unwrap().fields, &stale),
        Err(Outcome::Difference {
            cut,
            field: field.to_owned()
        })
    );
}

fn native_controller(production: &Production) -> Option<[u16; 2]> {
    let OwnedState::Frame(state) = production.state() else {
        panic!("frame production");
    };
    state.frame_history.as_ref()?.native_controller()
}

fn advance_to_prelude_stage(production: &mut Production, reached: impl Fn(&Caller) -> bool) {
    for _ in 0..20_000 {
        let OwnedState::Frame(state) = production.state() else {
            panic!("frame production");
        };
        if reached(&state.caller) {
            return;
        }
        production
            .advance(AdvanceInput::Frame {
                budget: MasterTicks(13),
            })
            .unwrap();
    }
    panic!("expected prelude stage was never reached");
}

fn check_settling_prefix(admitted: &AdmittedCase, observed_layers: [u8; 9]) {
    use types::haze::{HazeFade, HazePeak};

    let mut production = admitted.production.clone();
    let OwnedState::Frame(start) = production.state() else {
        panic!("frame production");
    };
    let before = start.haze.as_ref().unwrap().current();
    assert_eq!(
        before.fade,
        HazeFade::Brightening(HazePeak::new(16).unwrap())
    );
    let regions = start.content.regions();
    let from = start.timing.hardware.now;
    advance_to_prelude_stage(&mut production, |caller| {
        matches!(caller, Caller::HazeTail { .. })
    });
    let OwnedState::Frame(ready) = production.state() else {
        panic!("frame production");
    };
    assert_eq!(ready.timing.hardware.now.0.checked_sub(from.0), Some(1764));
    let haze = ready.haze.as_ref().unwrap().current();
    assert_eq!(haze.fade, HazeFade::Steady);
    assert_observation_eq!(haze.bands, before.bands);
    assert_observation_eq!(ready.content.regions(), regions);
    let saved = production.save().unwrap();
    let mut restored =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    for production in [&mut production, &mut restored] {
        advance_to_prelude_stage(production, |caller| {
            matches!(caller, Caller::LayerBlend { .. })
        });
        let OwnedState::Frame(terminal) = production.state() else {
            panic!("frame production");
        };
        assert_eq!(
            terminal.timing.hardware.now.0.checked_sub(from.0),
            Some(3332)
        );
        advance_to_prelude_stage(production, |caller| matches!(caller, Caller::Random { .. }));
        let OwnedState::Frame(layered) = production.state() else {
            panic!("frame production");
        };
        assert_eq!(
            layered.timing.hardware.now.0.checked_sub(from.0),
            Some(4240)
        );
        let Caller::Random { layers, .. } = &layered.caller else {
            panic!("completed layer settings");
        };
        assert_observation_eq!(
            game::production::frame::layer_blend::encode(layers.terminal).unwrap(),
            observed_layers
        );
    }
    assert_observation_eq!(production.state(), restored.state());
}

fn compare_continuation(
    case: &Case,
    sources: &Sources,
    admitted: &mut AdmittedCase,
    controller: &Controller,
) {
    let dependency = if matches!(&admitted.content, ProductionContent::Frame(content) if content.settling_haze.is_some())
    {
        let layers = super::admission::observed_layer_blend(case, sources).unwrap();
        check_settling_prefix(admitted, layers);
        Dependency::RandomPreparation
    } else {
        Dependency::HazePreparation
    };
    let mut restored = Production::restore(
        &admitted.production.save().unwrap(),
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    let cut = case.end.cut_inclusive;
    let mut restored_actual = admitted.actual.clone();
    let direct = runner::execute(
        case,
        &admitted.inputs,
        &admitted.expected,
        &mut admitted.production,
        &mut admitted.actual,
        cut,
    );
    let split = InputReader::Frame(Inputs {
        budget: MasterTicks(13),
    });
    let resumed = runner::execute(
        case,
        &split,
        &admitted.expected,
        &mut restored,
        &mut restored_actual,
        cut,
    );
    assert_observation_eq!(restored.state(), admitted.production.state());
    match (direct, resumed) {
        (Ok(_), Ok(_)) => {
            assert_observation_eq!(
                native_controller(&admitted.production),
                Some(controller.second)
            );
            assert_eq!(admitted.actual.publication_count(), 4_590);
        }
        (Err(actual), Err(restored)) => {
            assert_eq!(
                actual,
                production_failure(
                    cut,
                    ProductionFault::Unsupported(Unsupported::Frame(dependency))
                )
            );
            assert_eq!(actual, restored);
            assert_eq!(admitted.actual.publication_count(), 2_295);
        }
        _ => panic!("native continuation outcome differs across budgets"),
    }
    assert_eq!(
        restored_actual.publication_count(),
        admitted.actual.publication_count()
    );
}

#[test]
#[ignore = "requires the pinned adjacent and retained frame fixtures and player ROM"]
fn contiguous_native_returns_keep_independent_observations_and_inputs() {
    let index = std::env::var_os("SM_ADJACENT_INDEX")
        .map_or_else(
            || Index::parse(include_str!("../../../../../scenarios/replay/index.toml")),
            |trial| Index::read(Path::new(&trial)),
        )
        .unwrap();
    let sources = Sources {
        native: std::env::var_os("SM_AUDIO_NATIVE").unwrap().into(),
        rom: std::env::var_os("SM_ROM").unwrap().into(),
        ..Sources::default()
    };
    let control = read_control(&sources.native);
    assert_observation_eq!(control.actual.first, control.held.first);
    assert!(!core::cmp::PartialEq::eq(
        &control.actual.second[0],
        &control.actual.first[0]
    ));
    assert!(!core::cmp::PartialEq::eq(&control.actual.second[1], &0));
    assert_observation_eq!(control.held.second, control.held.first);
    for (id, controller) in [
        ("frame/adjacent-actual-trial", &control.actual),
        ("frame/adjacent-held-trial", &control.held),
    ] {
        let case = index.cases.iter().find(|case| case.id.0 == id).unwrap();
        let mut admitted = super::admit(case, &sources).unwrap();
        assert_eq!(admitted.schedule, [1, 2]);
        compare_negative_controls(case, &admitted.expected);
        observe_first_return(case, &mut admitted, controller);
        compare_continuation(case, &sources, &mut admitted, controller);
    }
}
