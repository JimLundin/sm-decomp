//! Independent adjacent cut partitions and the production replay seam.

use super::super::{
    Sources, compared,
    frame::{Datum, Inputs, random::Draw},
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
        ProductionSeed, Unsupported,
        frame::{
            self, Caller, Dependency, PlayerEntryStage, PreparationStage, State,
            player_continuation,
        },
        timing::EventKind,
        timing::SampleValue,
    },
    state::owned::OwnedState,
};
use std::path::Path;

#[path = "adjacent_hud_tests.rs"]
mod adjacent_hud_tests;

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

fn advance_to_prelude_stage(
    production: &mut Production,
    deadline: u64,
    reached: impl Fn(&Caller) -> bool,
) {
    advance_frame_until(production, deadline, |state| reached(&state.caller));
}

fn collect_to_prelude_stage(
    production: &mut Production,
    deadline: u64,
    reached: impl Fn(&Caller) -> bool,
) -> Vec<game::production::timing::Event> {
    let mut events = Vec::new();
    advance_frame(
        production,
        deadline,
        |state| reached(&state.caller),
        |outputs| events.extend(outputs.events),
    );
    events
}

fn advance_frame_until(
    production: &mut Production,
    deadline: u64,
    reached: impl Fn(&State) -> bool,
) {
    advance_frame(production, deadline, reached, |_| {});
}

fn advance_frame(
    production: &mut Production,
    deadline: u64,
    reached: impl Fn(&State) -> bool,
    collect: impl FnMut(game::production::frame::Outputs),
) {
    advance_frame_with_budget(production, deadline, MasterTicks(13), reached, collect);
}

fn advance_frame_with_budget(
    production: &mut Production,
    deadline: u64,
    budget: MasterTicks,
    reached: impl Fn(&State) -> bool,
    mut collect: impl FnMut(game::production::frame::Outputs),
) {
    loop {
        let OwnedState::Frame(state) = production.state() else {
            panic!("frame production");
        };
        if reached(state) {
            return;
        }
        let tick = state.timing.hardware.now.0;
        let cursor = state.timing.work.last().map(|work| work.cursor.clone());
        let depth = state.timing.work.len();
        let interrupted = state.timing.interrupt_boundary;
        let handler = state.timing.pending_handler.clone();
        assert!(
            tick <= deadline,
            "expected stage absent at wall {tick}, deadline {deadline}, cursor {cursor:?}"
        );
        let ProductionOutputs::Frame(outputs) = production
            .advance(AdvanceInput::Frame { budget })
            .unwrap_or_else(|fault| panic!("{fault:?} at wall {tick}, cursor {cursor:?}"))
        else {
            panic!("frame outputs");
        };
        collect(outputs);
        let OwnedState::Frame(current) = production.state() else {
            panic!("frame production");
        };
        let now = current.timing.hardware.now.0;
        assert!(now >= tick, "master clock moved from {tick} to {now}");
        assert!(
            now != tick
                || reached(current)
                || depth != current.timing.work.len()
                || cursor.as_ref() != current.timing.work.last().map(|work| &work.cursor)
                || interrupted != current.timing.interrupt_boundary
                || handler != current.timing.pending_handler,
            "no progress at wall {now}, cursor {cursor:?}"
        );
    }
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
    advance_to_prelude_stage(
        &mut production,
        from.0.checked_add(1892).unwrap(),
        |caller| matches!(caller, Caller::HazeTail { .. }),
    );
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
        advance_to_prelude_stage(production, from.0.checked_add(3460).unwrap(), |caller| {
            matches!(caller, Caller::LayerBlend { .. })
        });
        let OwnedState::Frame(terminal) = production.state() else {
            panic!("frame production");
        };
        assert_eq!(
            terminal.timing.hardware.now.0.checked_sub(from.0),
            Some(3332)
        );
        advance_to_prelude_stage(production, from.0.checked_add(4368).unwrap(), |caller| {
            matches!(caller, Caller::Random { .. })
        });
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
    original_doors: game::production::frame::actor_doors::Doors,
) {
    let dependency = if matches!(&admitted.content, ProductionContent::Frame(content) if content.settling_haze.is_some())
    {
        let layers = super::admission::observed_layer_blend(case, sources).unwrap();
        check_settling_prefix(admitted, layers);
        check_random_checkpoint(admitted, &observed_draw(admitted, case.end.cut_inclusive));
        check_preparation_checkpoints(admitted);
        let (entry, origin) = check_player_entry_checkpoints(admitted);
        check_remaining_player(admitted, entry, origin, original_doors);
        if matches!(&admitted.content, ProductionContent::Frame(content)
            if content.hud_refresh.as_ref().is_some_and(|hud| hud.recurring.is_some()))
        {
            adjacent_hud_tests::check_hud_map(case, sources, admitted);
            Dependency::CountdownSceneScheduling
        } else if admitted_odd_ground(admitted) {
            Dependency::HudDisplayPreparation
        } else {
            Dependency::PlayerContinuationPreparation
        }
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

fn observed_draw(admitted: &AdmittedCase, cut: u64) -> Draw {
    let ExpectedReader::Frame(expected) = &admitted.expected else {
        panic!("frame expected reader");
    };
    expected.observed_random(cut).unwrap().unwrap()
}

fn check_random_events(events: &[game::production::timing::Event], draw: &Draw) {
    let writes: Vec<_> = events
        .iter()
        .filter_map(|event| {
            if let EventKind::Publication(write) = &event.kind
                && matches!(write.port.0, 0x4202 | 0x4203)
            {
                Some((event.at.0, write.port.0, write.value))
            } else {
                None
            }
        })
        .collect();
    let samples: Vec<_> = events
        .iter()
        .filter_map(|event| {
            if let EventKind::Sample(
                device @ (Sample::ProductLow | Sample::ProductHigh),
                SampleValue::Byte(value),
            ) = event.kind
            {
                Some((event.at.0, device, value))
            } else {
                None
            }
        })
        .collect();
    let write_ports = [0x4202, 0x4203, 0x4202, 0x4203];
    let sample_ports = [Sample::ProductLow, Sample::ProductHigh, Sample::ProductLow];
    assert_observation_eq!(
        writes,
        core::array::from_fn::<_, 4, _>(|i| (
            draw.write_ticks[i],
            write_ports[i],
            draw.write_values[i]
        ))
    );
    assert_observation_eq!(
        samples,
        core::array::from_fn::<_, 3, _>(|i| (
            draw.sample_ticks[i],
            sample_ports[i],
            draw.sample_values[i]
        ))
    );
}

fn reject_forged_random_multiplier(admitted: &AdmittedCase, state: &frame::State) {
    use data::timing::{DeviceEffect, DeviceWrite, Publication};

    let mut forged = state.clone();
    let run = forged
        .timing
        .work
        .first_mut()
        .unwrap()
        .profile
        .runs
        .iter_mut()
        .rev()
        .find(|run| {
            matches!(
                &run.exposure.effect,
                DeviceEffect::Publish(write) if matches!(write.effect, Publication::Multiplier(_))
            )
        })
        .unwrap();
    let DeviceEffect::Publish(write) = &mut run.exposure.effect else {
        panic!("random multiplier publication");
    };
    *write = DeviceWrite::from_cpu_port(write.port.0, write.value.wrapping_add(1)).unwrap();
    let decoded: Box<frame::State> = canonical::from_bytes(&canonical::to_bytes(&forged)).unwrap();
    let mut physical = decoded.as_ref().clone();
    assert!(matches!(
        physical.step_hardware(MasterTicks(13)),
        Err(frame::Fault::InvalidRandomWork)
    ));
    assert_observation_eq!(&physical, decoded.as_ref());
    assert!(matches!(
        Production::admit(
            ProductionSeed::Frame(decoded),
            admitted.content.clone(),
            admitted.rules.clone(),
        ),
        Err(game::production::AdmissionFault::Frame(fault))
            if *fault == frame::Fault::InvalidRandomWork
    ));
}

fn check_random_checkpoint(admitted: &AdmittedCase, draw: &Draw) {
    use game::production::frame::random::Samples;

    let mut direct = admitted.production.clone();
    let random_deadline = draw.complete_tick.checked_add(128).unwrap();
    advance_to_prelude_stage(&mut direct, random_deadline, |caller| {
        matches!(caller, Caller::Random { .. })
    });
    let prefix = collect_to_prelude_stage(&mut direct, random_deadline, |caller| {
        matches!(
            caller,
            Caller::RandomRunning {
                samples: Samples::First(_),
                ..
            }
        )
    });
    let saved = direct.save().unwrap();
    let mut restored =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    let OwnedState::Frame(state) = restored.state() else {
        panic!("frame production");
    };
    reject_forged_random_multiplier(admitted, state);
    let mut altered = state.clone();
    let Caller::RandomRunning {
        requested,
        samples: Samples::First(first),
        ..
    } = &mut altered.caller
    else {
        panic!("restored partial Random samples");
    };
    assert_observation_eq!(Some(requested.0), state.current_random());
    first.0 ^= 1;
    let corrupt = Production::admit(
        ProductionSeed::Frame(altered),
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    let corrupt_checkpoint = corrupt.save().unwrap();
    let mut corrupt = Production::restore(
        &corrupt_checkpoint,
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    assert_atomic_rejection(
        &mut corrupt,
        Dependency::RandomPreparation,
        random_deadline,
        |caller| matches!(caller, Caller::RandomRunning { .. }),
    );
    for production in [&mut direct, &mut restored] {
        let mut events = prefix.clone();
        events.extend(collect_to_prelude_stage(
            production,
            random_deadline,
            |caller| matches!(caller, Caller::Preparation { .. }),
        ));
        check_random_events(&events, draw);
        let OwnedState::Frame(returned) = production.state() else {
            panic!("frame production");
        };
        assert_observation_eq!(returned.current_random(), Some(draw.result));
        assert_eq!(returned.timing.hardware.now.0, draw.complete_tick);
    }
    assert_observation_eq!(direct.state(), restored.state());
}

fn assert_atomic_rejection(
    production: &mut Production,
    dependency: Dependency,
    deadline: u64,
    pending: impl Fn(&Caller) -> bool,
) {
    const BUDGET: MasterTicks = MasterTicks(64);
    loop {
        let before = production.state().clone();
        let OwnedState::Frame(previous) = &before else {
            panic!("frame production");
        };
        let tick = previous.timing.hardware.now.0;
        assert!(
            tick <= deadline,
            "stale {dependency:?} passed its physical deadline"
        );
        match production.advance(AdvanceInput::Frame { budget: BUDGET }) {
            Ok(_) => {
                let OwnedState::Frame(current) = production.state() else {
                    panic!("frame production");
                };
                let now = current.timing.hardware.now.0;
                let still_pending = pending(&current.caller);
                let earlier = previous.timing.work.last();
                let later = current.timing.work.last();
                let (before_cursor, after_cursor) = (
                    earlier.map(|work| &work.cursor),
                    later.map(|work| &work.cursor),
                );
                assert!(
                    still_pending,
                    "stale {dependency:?} left its owner at {now}"
                );
                let elapsed = now.checked_sub(tick).expect("monotonic master clock");
                assert!(
                    elapsed != 0
                        || before_cursor != after_cursor
                        || previous.timing.work.len() != current.timing.work.len()
                        || previous.timing.interrupt_boundary != current.timing.interrupt_boundary
                        || previous.timing.pending_handler != current.timing.pending_handler,
                    "{dependency:?} made no scheduler progress at {now} with cursor {before_cursor:?}"
                );
                if let (Some(earlier), Some(later)) = (earlier, later)
                    && earlier.kind == later.kind
                    && previous.timing.work.len() == current.timing.work.len()
                {
                    let charged = later
                        .cursor
                        .intrinsic
                        .0
                        .checked_sub(earlier.cursor.intrinsic.0);
                    assert!(
                        charged.is_some_and(|amount| amount <= BUDGET.0),
                        "{dependency:?} exceeded intrinsic pause budget at wall {tick} -> {now}, cursor {before_cursor:?} -> {after_cursor:?}"
                    );
                }
            }
            Err(fault) => {
                assert_observation_eq!(production.state(), &before);
                assert_eq!(
                    fault,
                    ProductionFault::Unsupported(Unsupported::Frame(dependency))
                );
                return;
            }
        }
    }
}

fn check_preparation_checkpoints(admitted: &AdmittedCase) {
    use game::production::frame::{
        borrowed::ActorSlot,
        borrowed_gameplay::{GameplayRoute, SceneController},
        borrowed_preparation::{ActorSlotIdentity, DebugAction, PaletteDisposition, PaletteSlot},
    };
    type Change = fn(&mut State);

    let mut direct = admitted.production.clone();
    let OwnedState::Frame(start) = direct.state() else {
        panic!("frame production");
    };
    let origin = start.timing.hardware.now;
    let checks: [(PreparationStage, Dependency, u64, &[Change]); 4] = [
        (
            PreparationStage::Gameplay,
            Dependency::GameplayPreparation,
            5900,
            &[|state| state.scene = Some(SceneController::Other)],
        ),
        (
            PreparationStage::Actors,
            Dependency::ActorPreparation,
            15292,
            &[|state| {
                state.actor_eligibility.as_mut().unwrap().slots[1] = ActorSlot::Occupied {
                    deleted: false,
                    process_offscreen: true,
                    intangible: true,
                };
            }],
        ),
        (
            PreparationStage::Debug,
            Dependency::DebugPreparation,
            16248,
            &[
                |state| state.debug.as_mut().unwrap().shortcut.0 |= 0x0080,
                |state| state.debug.as_mut().unwrap().controller_two.0 |= 0x1000,
            ],
        ),
        (
            PreparationStage::Palette,
            Dependency::PalettePreparation,
            17670,
            &[
                |state| state.palette_objects.as_mut().unwrap().enabled = false,
                |state| state.palette_objects.as_mut().unwrap().slots[0] = PaletteSlot::Occupied,
            ],
        ),
    ];
    for (stage, dependency, completion, changes) in checks {
        let deadline = origin
            .0
            .checked_add(completion)
            .unwrap()
            .checked_add(128)
            .unwrap();
        check_preparation_checkpoint(admitted, &mut direct, stage, dependency, deadline, changes);
        let OwnedState::Frame(state) = direct.state() else {
            panic!("frame production");
        };
        assert_eq!(
            state.timing.hardware.now.0.checked_sub(origin.0),
            Some(completion)
        );
    }
    let OwnedState::Frame(state) = direct.state() else {
        panic!("frame production");
    };
    let Caller::Player { prepared, .. } = &state.caller else {
        panic!("completed recurring preparation");
    };
    let gameplay = prepared.gameplay.as_ref().unwrap();
    assert_observation_eq!(gameplay.scene, SceneController::MainGameplay);
    assert_observation_eq!(gameplay.route, GameplayRoute::MainGameplay);
    let actors = prepared.actors.as_ref().unwrap();
    assert_observation_eq!(
        actors.active.as_slice(),
        &[ActorSlotIdentity(0), ActorSlotIdentity(1)]
    );
    assert_observation_eq!(actors.interactive.as_slice(), &[ActorSlotIdentity(1)]);
    let debug = prepared.debug.as_ref().unwrap();
    assert_observation_eq!(debug.action, DebugAction::None);
    assert!(!debug.skip_processing);
    assert_observation_eq!(
        prepared.palette.as_ref().unwrap().terminal,
        PaletteDisposition::RetainCurrent
    );
}

fn check_preparation_checkpoint(
    admitted: &AdmittedCase,
    direct: &mut Production,
    stage: PreparationStage,
    dependency: Dependency,
    deadline: u64,
    changes: &[fn(&mut State)],
) {
    let charging = |caller: &Caller| {
        matches!(
            caller,
            Caller::Preparation { stage: actual, charging: true, .. } if *actual == stage
        )
    };
    advance_to_prelude_stage(direct, deadline, charging);
    let saved = direct.save().unwrap();
    let mut restored =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    let OwnedState::Frame(restored_frame) = restored.state() else {
        panic!("frame production");
    };
    for change in changes {
        let mut altered = restored_frame.clone();
        change(&mut altered);
        let mut invalid = Production::admit(
            ProductionSeed::Frame(altered),
            admitted.content.clone(),
            admitted.rules.clone(),
        )
        .unwrap();
        assert_atomic_rejection(&mut invalid, dependency, deadline, charging);
    }
    advance_to_prelude_stage(direct, deadline, |caller| !charging(caller));
    advance_to_prelude_stage(&mut restored, deadline, |caller| !charging(caller));
    assert_observation_eq!(direct.state(), restored.state());
}

fn check_player_entry_checkpoints(admitted: &AdmittedCase) -> (Production, u64) {
    use game::production::frame::borrowed_player_preparation::SuitPaletteDisposition;

    let mut direct = admitted.production.clone();
    let OwnedState::Frame(start) = direct.state() else {
        panic!("frame production");
    };
    let origin = start.timing.hardware.now.0;
    let starting_pose = start.movement.state().pose;
    let charging_pose = |caller: &Caller| {
        matches!(
            caller,
            Caller::Player {
                stage: PlayerEntryStage::PoseInput,
                charging: true,
                ..
            }
        )
    };
    advance_to_prelude_stage(
        &mut direct,
        origin.checked_add(17798).unwrap(),
        charging_pose,
    );
    let saved = direct.save().unwrap();
    let mut restored =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    for (next, completion) in [
        (PlayerEntryStage::SuitPalette, 22502),
        (PlayerEntryStage::Environment, 22710),
        (PlayerEntryStage::Inside, 23486),
    ] {
        if next == PlayerEntryStage::Inside {
            reject_changed_surface(admitted, direct.clone(), origin);
        }
        for production in [&mut direct, &mut restored] {
            advance_to_prelude_stage(
                production,
                origin
                    .checked_add(completion)
                    .unwrap()
                    .checked_add(128)
                    .unwrap(),
                |caller| matches!(caller, Caller::Player { stage, charging: false, .. } if *stage == next),
            );
            let OwnedState::Frame(current) = production.state() else {
                panic!("frame production");
            };
            assert_eq!(
                current.timing.hardware.now.0.checked_sub(origin),
                Some(completion)
            );
        }
        assert_observation_eq!(direct.state(), restored.state());
    }
    let OwnedState::Frame(current) = direct.state() else {
        panic!("frame production");
    };
    let Caller::Player { completed, .. } = &current.caller else {
        panic!("completed ordinary player entry");
    };
    let pose = completed.0.pose_input.as_ref().unwrap();
    assert_observation_eq!(pose.dependency.pose, starting_pose);
    assert_observation_eq!(pose.terminal.prospective.aim, types::Aim::Forward);
    assert_observation_eq!(current.movement.state().pose, starting_pose);
    assert_observation_eq!(
        completed.0.suit_palette.as_ref().unwrap().terminal,
        SuitPaletteDisposition::PowerSuit
    );
    assert_observation_eq!(
        completed.0.environment.as_ref().unwrap().dependency.pose,
        starting_pose
    );
    (check_inside_checkpoint(admitted, direct, origin), origin)
}

fn check_inside_checkpoint(
    admitted: &AdmittedCase,
    mut direct: Production,
    origin: u64,
) -> Production {
    use game::production::{frame::interrupts::DrawingRequest, timing::WorkId};

    let inside = |caller: &Caller| {
        matches!(
            caller,
            Caller::Player {
                stage: PlayerEntryStage::Inside,
                charging: true,
                ..
            }
        )
    };
    let drawing_deadline = origin.checked_add(25388).unwrap();
    advance_to_prelude_stage(&mut direct, drawing_deadline, inside);
    let OwnedState::Frame(current) = direct.state() else {
        panic!("frame production");
    };
    assert_observation_eq!(
        current.inside_inputs.as_ref().unwrap().0.parity,
        types::Parity::Odd
    );
    let mut altered = current.clone();
    altered.drawing.as_mut().unwrap().recurring_due = None;
    let mut invalid = Production::admit(
        ProductionSeed::Frame(altered),
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    assert_atomic_rejection(
        &mut invalid,
        Dependency::InterruptOwner,
        drawing_deadline,
        inside,
    );
    advance_frame_until(&mut direct, drawing_deadline, |state| {
        inside(&state.caller)
            && state.timing.work.len() == 2
            && state
                .drawing
                .as_ref()
                .is_some_and(|drawing| drawing.active == Some(DrawingRequest::Begin))
    });
    let saved = direct.save().unwrap();
    let mut restored =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    for production in [&mut direct, &mut restored] {
        advance_to_prelude_stage(production, origin.checked_add(30056).unwrap(), |caller| {
            matches!(
                caller,
                Caller::Player {
                    stage: PlayerEntryStage::HudWeapons,
                    charging: false,
                    ..
                }
            )
        });
        let OwnedState::Frame(state) = production.state() else {
            panic!("frame production");
        };
        assert_eq!(state.timing.hardware.now.0.checked_sub(origin), Some(29928));
        assert_observation_eq!(state.inside_completed.as_ref().unwrap().work, WorkId(15));
        let drawing = state.drawing.as_ref().unwrap();
        assert_observation_eq!(drawing.current.request, DrawingRequest::End);
        assert_observation_eq!(drawing.recurring_due, Some(DrawingRequest::End));
        assert!(drawing.active.is_none());
    }
    assert_observation_eq!(direct.state(), restored.state());
    check_early_hud_checkpoint(admitted, direct, origin)
}

fn check_early_hud_checkpoint(
    admitted: &AdmittedCase,
    mut direct: Production,
    origin: u64,
) -> Production {
    use interface_hud::Selection;

    let charging = |caller: &Caller| {
        matches!(
            caller,
            Caller::Player {
                stage: PlayerEntryStage::HudWeapons,
                charging: true,
                ..
            }
        )
    };
    let deadline = origin.checked_add(32112).unwrap();
    advance_to_prelude_stage(&mut direct, deadline, charging);
    let checkpoint = direct.save().unwrap();
    let mut restored = Production::restore(
        &checkpoint,
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    let OwnedState::Frame(current) = restored.state() else {
        panic!("frame production");
    };
    let mut altered = current.clone();
    altered.hud_frame_count = altered.hud_frame_count.map(|count| count.wrapping_add(1));
    let mut invalid = Production::admit(
        ProductionSeed::Frame(altered),
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    assert_atomic_rejection(
        &mut invalid,
        Dependency::HudWeaponsPreparation,
        deadline,
        charging,
    );
    for production in [&mut direct, &mut restored] {
        advance_to_prelude_stage(production, deadline, |caller| {
            matches!(
                caller,
                Caller::PlayerContinuation {
                    stage: player_continuation::Stage::Projectiles,
                    charging: false,
                    ..
                }
            )
        });
        let OwnedState::Frame(state) = production.state() else {
            panic!("frame production");
        };
        assert_eq!(state.timing.hardware.now.0.checked_sub(origin), Some(31984));
        assert_observation_eq!(state.hud.state().selection, Selection::None);
        assert!(state.appearance.cannon.as_ref().unwrap().0.may_follow());
    }
    assert_observation_eq!(direct.state(), restored.state());
    direct
}

fn remaining_at(caller: &Caller, expected: player_continuation::Stage, busy: bool) -> bool {
    matches!(
        caller,
        Caller::PlayerContinuation { stage, charging, .. }
            if *stage == expected && *charging == busy
    )
}

fn check_remaining_player(
    admitted: &AdmittedCase,
    mut direct: Production,
    origin: u64,
    original_doors: game::production::frame::actor_doors::Doors,
) {
    use player_continuation::Stage;
    let saved = direct.save().unwrap();
    let mut restored =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    for (next, edge) in [
        (Stage::MotionReset, 33756),
        (Stage::Interaction, 34590),
        (Stage::Actors, 35170),
    ] {
        for production in [&mut direct, &mut restored] {
            advance_to_prelude_stage(
                production,
                origin.checked_add(edge).unwrap().checked_add(128).unwrap(),
                |caller| remaining_at(caller, next, false),
            );
            let OwnedState::Frame(state) = production.state() else {
                panic!("frame production");
            };
            assert_eq!(state.timing.hardware.now.0.checked_sub(origin), Some(edge));
        }
        assert_observation_eq!(direct.state(), restored.state());
    }
    check_recurring_doors(admitted, direct, origin, original_doors);
}

fn check_recurring_doors(
    admitted: &AdmittedCase,
    mut direct: Production,
    origin: u64,
    original_doors: game::production::frame::actor_doors::Doors,
) {
    use game::production::frame::borrowed_player_preparation::GraphicsUpdated;
    use game::production::timing::WorkId;
    use player_continuation::Stage;

    let charging = |caller: &Caller| remaining_at(caller, Stage::Actors, true);
    advance_to_prelude_stage(&mut direct, origin.checked_add(35298).unwrap(), charging);
    let saved = direct.save().unwrap();
    let mut restored =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    reject_stale_door_inputs(admitted, &restored, origin, original_doors);
    let deadline = origin.checked_add(48308).unwrap();
    for production in [&mut direct, &mut restored] {
        advance_to_prelude_stage(production, deadline, |caller| {
            remaining_at(caller, Stage::Sprites, false)
        });
        let OwnedState::Frame(state) = production.state() else {
            panic!("frame production");
        };
        assert_eq!(state.timing.hardware.now.0.checked_sub(origin), Some(48180));
        let Caller::PlayerContinuation {
            remaining,
            remaining_work,
            ..
        } = &state.caller
        else {
            panic!("recurring player");
        };
        let actual = remaining.0.actors.as_ref().unwrap();
        assert_eq!(
            remaining_work,
            &[WorkId(17), WorkId(18), WorkId(19), WorkId(20)]
        );
        assert!(
            actual
                .dependency
                .actors
                .iter()
                .all(|actor| actor.instruction_timer == 1)
        );
        assert!(
            actual
                .actors
                .iter()
                .zip(&actual.dependency.actors)
                .all(|(actor, prior)| {
                    actor.instruction_timer == 2
                        && actor.graphics_updated == GraphicsUpdated::Set
                        && actor.geometry == prior.geometry
                        && actor.frame_counter.0 == prior.frame_counter.0.wrapping_add(1)
                })
        );
        assert_observation_eq!(state.health.as_ref().unwrap().actor(), Some(actual));
    }
    assert_observation_eq!(direct.state(), restored.state());
    check_committed_actor_checkpoint(admitted, &direct);
    check_post_actor_prefix(admitted, direct, origin);
}

fn check_post_actor_prefix(admitted: &AdmittedCase, mut direct: Production, origin: u64) {
    use player_continuation::Stage;

    for (stage, edge) in [(Stage::Contact, 55934), (Stage::Run, 56348)] {
        advance_to_prelude_stage(
            &mut direct,
            origin.checked_add(edge).unwrap().checked_add(128).unwrap(),
            |caller| remaining_at(caller, stage, false),
        );
        let OwnedState::Frame(state) = direct.state() else {
            panic!("frame production");
        };
        assert_eq!(state.timing.hardware.now.0.checked_sub(origin), Some(edge));
    }
    check_committed_actor_checkpoint(admitted, &direct);
    reject_stale_contact_counter(admitted, &direct);
    let checkpoint = direct.save().unwrap();
    let mut restored = Production::restore(
        &checkpoint,
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    for production in [&mut direct, &mut restored] {
        advance_frame_until(production, origin.checked_add(62322).unwrap(), |state| {
            state.movement_inputs.is_some()
        });
        let OwnedState::Frame(state) = production.state() else {
            panic!("frame production");
        };
        assert_eq!(state.timing.hardware.now.0.checked_sub(origin), Some(62194));
        assert_observation_eq!(
            state.movement_inputs.as_ref().unwrap().0.parity,
            types::Parity::Odd
        );
    }
    assert_observation_eq!(direct.state(), restored.state());
    if admitted_odd_ground(admitted) {
        check_grounded_down_checkpoint(admitted, direct, origin);
    }
}

fn admitted_odd_ground(admitted: &AdmittedCase) -> bool {
    matches!(&admitted.content, ProductionContent::Frame(content)
    if content.ground_following_down.as_ref().is_some_and(
        game::production::frame::movement_down_contract::PhysicalContract::has_odd_two_column
    ))
}

fn reject_stale_contact_counter(admitted: &AdmittedCase, production: &Production) {
    use game::production::{AdmissionFault, frame::Fault};

    let OwnedState::Frame(state) = production.state() else {
        panic!("frame production");
    };
    let mut altered = state.clone();
    let Caller::PlayerContinuation { remaining, .. } = &mut altered.caller else {
        panic!("completed ContactReset");
    };
    remaining.0.contact.as_mut().unwrap().execution_counter.0 ^= 1;
    let result = Production::admit(
        ProductionSeed::Frame(altered),
        admitted.content.clone(),
        admitted.rules.clone(),
    );
    assert!(
        matches!(result, Err(AdmissionFault::Frame(fault)) if *fault == Fault::InvalidPlayerEntry)
    );
}

fn check_grounded_down_checkpoint(admitted: &AdmittedCase, mut direct: Production, origin: u64) {
    use data::timing::Sample;
    use game::production::{
        frame::movement_down_contract::{Pending, Stage as Down},
        timing::{EventKind, SampleValue, WorkId},
    };
    use std::cell::Cell;
    let charging = |caller: &Caller| {
        matches!(
            caller,
            Caller::PlayerContinuation {
                down: Down::Charging { .. },
                ..
            }
        )
    };
    let low = Cell::new(None);
    let deadline = origin.checked_add(79742).unwrap();
    let down_operands = origin.checked_add(77000).unwrap();
    advance_frame(
        &mut direct,
        origin.checked_add(77368).unwrap(),
        |_| low.get().is_some(),
        |outputs| {
            for event in outputs.events {
                if let EventKind::Sample(Sample::ProductLow, SampleValue::Byte(byte)) = event.kind
                    && event.at.0 >= down_operands
                {
                    low.set(Some(byte));
                }
            }
        },
    );
    let saved = direct.save().unwrap();
    let mut restored =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    let OwnedState::Frame(state) = restored.state() else {
        panic!("frame production");
    };
    let mut altered = state.clone();
    let Caller::PlayerContinuation {
        down: Down::Charging { pending, .. },
        ..
    } = &mut altered.caller
    else {
        panic!("active Down arithmetic");
    };
    let down_bill_rejected = rejects_changed_physical_bill(admitted, &restored, WorkId(1004));
    let mut corrupt = Pending::new(WorkId(1004), pending.requests()).unwrap();
    corrupt
        .sample(WorkId(1004), Sample::ProductLow, low.get().unwrap() ^ 1)
        .unwrap();
    *pending = corrupt;
    let mut invalid = Production::admit(
        ProductionSeed::Frame(altered),
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    assert_atomic_rejection(
        &mut invalid,
        Dependency::PlayerContinuationPreparation,
        deadline,
        charging,
    );
    let [down_receipt_rejected, contact_bill_rejected] =
        check_down_and_contact_checkpoints(admitted, &mut direct, &mut restored, deadline);
    let (coherent_copies_rejected, poststep_bill_rejected) =
        check_quiet_poststep(admitted, &mut direct, &mut restored, deadline);
    assert_eq!(
        [
            down_bill_rejected,
            down_receipt_rejected,
            contact_bill_rejected
        ],
        [true; 3],
        "forged Down bill, completed Down receipt, and quiet Contact bill must all reject"
    );
    assert_eq!(
        coherent_copies_rejected, [true; 3],
        "forged Down copies must reject after Contact and during and after Poststep"
    );
    assert!(
        poststep_bill_rejected,
        "forged quiet Poststep bill restored"
    );
}

fn check_quiet_poststep(
    admitted: &AdmittedCase,
    direct: &mut Production,
    restored: &mut Production,
    deadline: u64,
) -> ([bool; 3], bool) {
    use game::production::{frame::post_down_caller::Stage as Post, timing::WorkId};
    let phases: [fn(&Caller) -> bool; 3] = [
        |caller| {
            matches!(
                caller,
                Caller::PlayerContinuation {
                    post_down: Post::ContactComplete(_),
                    charging: false,
                    ..
                }
            )
        },
        |caller| {
            matches!(
                caller,
                Caller::PlayerContinuation {
                    post_down: Post::Poststep { .. },
                    charging: true,
                    ..
                }
            )
        },
        |caller| {
            matches!(
                caller,
                Caller::PlayerContinuation {
                    post_down: Post::Complete(_),
                    ..
                }
            )
        },
    ];
    let mut poststep_bill_rejected = false;
    let rejected = phases.map(|reached| {
        for production in [&mut *direct, &mut *restored] {
            advance_to_prelude_stage(production, deadline, reached);
        }
        assert_observation_eq!(direct.state(), restored.state());
        if matches!(direct.state(), OwnedState::Frame(state)
            if matches!(state.caller, Caller::PlayerContinuation { post_down: Post::Poststep { .. }, charging: true, .. }))
        {
            poststep_bill_rejected = rejects_changed_physical_bill(admitted, direct, WorkId(1106));
        }
        rejects_coherent_down_forgery(admitted, direct)
    });
    let OwnedState::Frame(state) = direct.state() else {
        panic!("frame production");
    };
    let Caller::PlayerContinuation { post_down, .. } = &state.caller else {
        panic!("completed Poststep");
    };
    assert_eq!(
        post_down.completed_identities(),
        [WorkId(1100), WorkId(1106)]
    );
    (rejected, poststep_bill_rejected)
}

fn rejects_coherent_down_forgery(admitted: &AdmittedCase, production: &Production) -> bool {
    use canonical::{from_bytes, to_bytes};
    use game::production::frame::movement_down_contract::Stage as Down;
    rejects_changed_frame(admitted, production, |state| {
        let Caller::PlayerContinuation {
            down: Down::Complete(down),
            post_down,
            ..
        } = &mut state.caller
        else {
            panic!("completed quiet Contact");
        };
        let authentic = to_bytes(down);
        down.receipt.decisions.0.clear();
        let changed = to_bytes(down);
        let mut bytes = to_bytes(post_down);
        let mut copies = bytes
            .windows(authentic.len())
            .enumerate()
            .filter(|(_, field)| *field == authentic);
        let (offset, _) = copies.next().unwrap();
        assert!(copies.next().is_none());
        let end = offset.checked_add(authentic.len()).unwrap();
        drop(bytes.splice(offset..end, changed));
        *post_down = from_bytes(&bytes).unwrap();
    })
}

fn check_down_and_contact_checkpoints(
    admitted: &AdmittedCase,
    direct: &mut Production,
    restored: &mut Production,
    deadline: u64,
) -> [bool; 2] {
    use game::production::{
        frame::{movement_down_contract::Stage as Down, post_down_caller::Stage as Post},
        timing::WorkId,
    };
    for production in [&mut *direct, &mut *restored] {
        advance_to_prelude_stage(production, deadline, |caller| {
            matches!(
                caller,
                Caller::PlayerContinuation {
                    down: Down::Complete(_),
                    post_down: Post::Idle,
                    charging: false,
                    ..
                }
            )
        });
    }
    assert_observation_eq!(direct.state(), restored.state());
    let down_receipt = rejects_forged_down_completion(admitted, direct);
    for production in [&mut *direct, &mut *restored] {
        advance_to_prelude_stage(production, deadline, |caller| {
            matches!(
                caller,
                Caller::PlayerContinuation {
                    post_down: Post::Contact { .. },
                    charging: true,
                    ..
                }
            )
        });
    }
    assert_observation_eq!(direct.state(), restored.state());
    [
        down_receipt,
        rejects_changed_physical_bill(admitted, direct, WorkId(1100)),
    ]
}

fn rejects_forged_down_completion(admitted: &AdmittedCase, production: &Production) -> bool {
    use game::production::frame::movement_down_contract::Stage as Down;
    rejects_changed_frame(admitted, production, |state| {
        let Caller::PlayerContinuation {
            down: Down::Complete(completed),
            ..
        } = &mut state.caller
        else {
            panic!("completed Down");
        };
        completed.receipt.decisions.0.clear();
    })
}

fn rejects_changed_physical_bill(
    admitted: &AdmittedCase,
    production: &Production,
    identity: game::production::timing::WorkId,
) -> bool {
    use data::timing::DeviceEffect;
    use game::production::timing::WorkKind;
    rejects_changed_frame(admitted, production, |state| {
        let work = state
            .timing
            .work
            .iter_mut()
            .find(|work| work.kind == WorkKind::Semantic(identity))
            .unwrap();
        let future = usize::try_from(work.cursor.run)
            .unwrap()
            .checked_add(1)
            .unwrap();
        let run = work
            .profile
            .runs
            .iter_mut()
            .skip(future)
            .find(|run| run.exposure.effect == DeviceEffect::None)
            .unwrap();
        run.count = run.count.checked_add(1).unwrap();
    })
}

fn rejects_changed_frame(
    admitted: &AdmittedCase,
    production: &Production,
    change: impl FnOnce(&mut State),
) -> bool {
    use game::production::{AdmissionFault, frame::Fault};
    let restore = |production: &Production| {
        Production::restore(
            &production.save().unwrap(),
            admitted.content.clone(),
            admitted.rules.clone(),
        )
    };
    let OwnedState::Frame(before) = production.state() else {
        panic!("frame production");
    };
    let pending = before.timing.work.iter().find_map(|work| match work.kind {
        game::production::timing::WorkKind::Semantic(identity) => Some(identity),
        game::production::timing::WorkKind::Wait(_)
        | game::production::timing::WorkKind::Native { .. }
        | game::production::timing::WorkKind::Handler { .. } => None,
    });
    let at = before.timing.hardware.now.0;
    let authentic = restore(production).unwrap_or_else(|fault| {
        panic!(
            "authentic checkpoint at wall {at} with semantic work {pending:?} rejected: {fault:?}"
        )
    });
    assert_observation_eq!(authentic.state(), production.state());
    let OwnedState::Frame(state) = authentic.state() else {
        panic!("frame production");
    };
    let mut altered = state.clone();
    change(&mut altered);
    matches!(
        Production::admit(
            ProductionSeed::Frame(altered),
            admitted.content.clone(),
            admitted.rules.clone(),
        ).and_then(|admitted| restore(&admitted)),
        Err(AdmissionFault::Frame(fault)) if *fault == Fault::InvalidPlayerEntry
    )
}

fn origin_actor_controls(admitted: &AdmittedCase) -> ([bool; 2], bool) {
    use player_continuation::Stage;
    let mut genuine = admitted.production.clone();
    let deadline = 37_500;
    advance_to_prelude_stage(&mut genuine, deadline, |caller| {
        remaining_at(caller, Stage::Actors, false)
    });
    let OwnedState::Frame(before) = genuine.state() else {
        panic!("frame production");
    };
    let mut swapped = before.clone();
    advance_to_prelude_stage(&mut genuine, 50_000, |caller| {
        remaining_at(caller, Stage::Sprites, false)
    });
    let OwnedState::Frame(after) = genuine.state() else {
        panic!("frame production");
    };
    swapped.recurring_player.as_mut().unwrap().doors =
        after.recurring_player.as_ref().unwrap().doors;
    let mut swapped = Production::admit(
        ProductionSeed::Frame(swapped),
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    let before = swapped.state().clone();
    let rejected = swapped
        .advance(AdvanceInput::Frame {
            budget: MasterTicks(13),
        })
        .is_err_and(|fault| {
            fault
                == ProductionFault::Unsupported(Unsupported::Frame(
                    Dependency::PlayerContinuationPreparation,
                ))
        });
    if rejected {
        assert_observation_eq!(swapped.state(), &before);
    }
    let checks = committed_actor_controls(admitted, &genuine, 1);
    let mut restored = Production::restore(
        &genuine.save().unwrap(),
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    for production in [&mut genuine, &mut restored] {
        advance_to_prelude_stage(production, 58_000, |caller| {
            remaining_at(caller, Stage::Run, false)
        });
    }
    assert_observation_eq!(genuine.state(), restored.state());
    (checks, rejected)
}

fn check_committed_actor_checkpoint(admitted: &AdmittedCase, complete: &Production) {
    assert_eq!(
        committed_actor_controls(admitted, complete, 2),
        [true, true]
    );
}

fn committed_actor_controls(
    admitted: &AdmittedCase,
    complete: &Production,
    stale_wait: u16,
) -> [bool; 2] {
    use game::production::{AdmissionFault, frame::Fault};

    let restore = |production: &Production| {
        Production::restore(
            &production.save().unwrap(),
            admitted.content.clone(),
            admitted.rules.clone(),
        )
    };
    let authentic = restore(complete).unwrap();
    assert_observation_eq!(authentic.state(), complete.state());
    let OwnedState::Frame(state) = authentic.state() else {
        panic!("frame production");
    };
    [false, true].map(|change_view| {
        let mut invalid = state.clone();
        let Caller::PlayerContinuation { remaining, .. } = &mut invalid.caller else {
            panic!("completed ActorUpdate");
        };
        let completion = remaining.0.actors.as_mut().unwrap();
        let actual = completion.clone();
        if change_view {
            use game::production::frame::borrowed_player_preparation::ActorViewDisposition;
            completion.actors[0].view = ActorViewDisposition::BelowPaddedView;
        } else {
            completion.dependency.actors[0].instruction_timer = stale_wait;
        }
        let health = invalid.health.as_mut().unwrap();
        replace_health_actor(health, &actual, completion);
        let result = Production::admit(
            ProductionSeed::Frame(invalid),
            admitted.content.clone(),
            admitted.rules.clone(),
        )
        .and_then(|admitted| restore(&admitted));
        result.is_err_and(|fault| {
            matches!(
                fault, AdmissionFault::Frame(frame) if *frame == Fault::InvalidPlayerEntry
            )
        })
    })
}

fn replace_health_actor(
    health: &mut game::production::frame::health::Health,
    current: &game::production::frame::borrowed_player_preparation::ActorUpdateCompletion,
    altered: &game::production::frame::borrowed_player_preparation::ActorUpdateCompletion,
) {
    use canonical::{from_bytes, model::Model, to_bytes};
    use game::production::frame::movement_support_contract::ActorCompletionPin;

    let before = to_bytes(&Model::<ActorCompletionPin>(current.clone()));
    let after = to_bytes(&Model::<ActorCompletionPin>(altered.clone()));
    let mut bytes = to_bytes(health);
    let mut matches = bytes
        .windows(before.len())
        .enumerate()
        .filter(|(_, field)| *field == before);
    let (offset, _) = matches.next().unwrap();
    assert!(matches.next().is_none());
    let end = offset.checked_add(before.len()).unwrap();
    drop(bytes.splice(offset..end, after));
    *health = from_bytes(&bytes).unwrap();
    assert_observation_eq!(health.actor(), Some(altered));
}

fn reject_stale_door_inputs(
    admitted: &AdmittedCase,
    restored: &Production,
    origin: u64,
    original_doors: game::production::frame::actor_doors::Doors,
) {
    use player_continuation::Stage;
    let OwnedState::Frame(state) = restored.state() else {
        panic!("frame production");
    };
    let mut stale_pose = state.clone();
    let Caller::PlayerContinuation {
        motion_reset: Some(motion),
        ..
    } = &mut stale_pose.caller
    else {
        panic!("completed motion reset");
    };
    motion.previous.0.pose.aim = types::Aim::Up;
    let mut stale_wait = state.clone();
    let owners = stale_wait.recurring_player.as_mut().unwrap();
    assert_observation_eq!(owners.doors == original_doors, false);
    owners.doors = original_doors;
    for altered in [stale_pose, stale_wait] {
        let mut invalid = Production::admit(
            ProductionSeed::Frame(altered),
            admitted.content.clone(),
            admitted.rules.clone(),
        )
        .unwrap();
        assert_atomic_rejection(
            &mut invalid,
            Dependency::PlayerContinuationPreparation,
            origin.checked_add(48308).unwrap(),
            |caller| remaining_at(caller, Stage::Actors, true),
        );
    }
}

fn environment_charging(caller: &Caller) -> bool {
    matches!(
        caller,
        Caller::Player {
            stage: PlayerEntryStage::Environment,
            charging: true,
            ..
        }
    )
}

fn reject_changed_surface(admitted: &AdmittedCase, mut production: Production, origin: u64) {
    let deadline = origin.checked_add(23614).unwrap();
    advance_to_prelude_stage(&mut production, deadline, environment_charging);
    let OwnedState::Frame(current) = production.state() else {
        panic!("frame production");
    };
    let mut altered = current.clone();
    altered.entry_environment.as_mut().unwrap().water_inactive = false;
    let mut invalid = Production::admit(
        ProductionSeed::Frame(altered),
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    assert_atomic_rejection(
        &mut invalid,
        Dependency::PlayerPreparation,
        deadline,
        environment_charging,
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
    let mut origin_controls = Vec::new();
    for (id, controller) in [
        ("frame/adjacent-actual-trial", &control.actual),
        ("frame/adjacent-held-trial", &control.held),
    ] {
        let case = index.cases.iter().find(|case| case.id.0 == id).unwrap();
        let mut admitted = super::admit(case, &sources).unwrap();
        assert_eq!(admitted.schedule, [1, 2]);
        compare_negative_controls(case, &admitted.expected);
        check_random_checkpoint(
            &admitted,
            &observed_draw(&admitted, case.first_cut().unwrap()),
        );
        let OwnedState::Frame(initial) = admitted.production.state() else {
            panic!("frame production");
        };
        let original_doors = initial.recurring_player.as_ref().unwrap().doors;
        origin_controls.push(origin_actor_controls(&admitted));
        observe_first_return(case, &mut admitted, controller);
        compare_continuation(case, &sources, &mut admitted, controller, original_doors);
    }
    assert_eq!(
        origin_controls,
        [([true, true], true); 2],
        "origin actor checkpoint wait/view and live carried wait controls"
    );
}
