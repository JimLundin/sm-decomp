use super::runtime_helpers::{advance_until, restore_next, retained_frame_case, retained_return};
use super::*;
use data::timing::{MasterTicks, Sample, TransferPort};
use frame::{late_hud::Stage, suffix_caller::HudQueue};
use game::production::timing::{Event, EventKind, SampleValue};
use game::production::{AdmissionFault, AdvanceInput, ProductionOutputs};
use game::state::owned::OwnedState;

#[test]
#[ignore = "requires the retained native frame and checked player ROM"]
fn actual_status_in_flight_restores_one_hud_upload_through_native_return() {
    let (case, mut admitted) = retained_return();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs");
    };
    let budget = inputs.budget;
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content");
    };
    let hud = content.late_hud.clone().unwrap();
    advance_until(&mut admitted, budget, |state| {
        matches!(state.late_hud, Stage::Charging { .. })
            && state
                .timing
                .work
                .last()
                .is_some_and(|work| work.profile == hud.status && work.cursor.intrinsic.0 > 0)
    });

    let change_retained_energy = |state: &mut frame::State| {
        let Stage::Charging { energy, .. } = &mut state.late_hud else {
            panic!("in-flight status");
        };
        *energy ^= 1;
    };
    assert_rejected_at_admission(&admitted, change_retained_energy);
    assert_rejected_at_admission(&admitted, choose_status_with_stale_energy);
    assert_rejected_at_admission(&admitted, |state| {
        change_retained_energy(state);
        choose_status_with_stale_energy(state);
    });
    assert_rejected_at_admission(&admitted, |state| {
        let work = state.timing.work.last_mut().unwrap();
        let run = work.profile.runs.last_mut().unwrap();
        run.count = run.count.checked_add(1).unwrap();
    });
    assert_rejected_at_admission(&admitted, |state| state.late_hud = Stage::StatusReady);

    let checkpoint = admitted.production.save().unwrap();
    let mut restored = Production::restore(
        &checkpoint,
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    let mut actual = admitted.actual;
    let super::super::readers::ActualReader::Frame(receiver) = &mut actual else {
        panic!("frame receiver");
    };
    receiver
        .events
        .clone_from(&frame_state(&admitted.production).publications.events);
    let direct = finish_with_hud_upload(&mut admitted.production, budget, &hud.queue);
    let resumed = finish_with_hud_upload(&mut restored, budget, &hud.queue);
    assert_observation_eq!(resumed, direct, "resumed native output differs");
    assert_observation_eq!(
        restored.save().unwrap(),
        admitted.production.save().unwrap()
    );

    let cut = case.end.cut_inclusive;
    let observed = actual
        .accept(
            &admitted.inputs,
            &restored,
            ProductionOutputs::Frame(resumed),
            cut,
        )
        .unwrap();
    let expected = admitted.expected.read(&case, cut).unwrap();
    super::super::compared(cut, &expected.fields, &observed).unwrap();
}

#[test]
#[ignore = "requires the retained native frame and checked player ROM"]
fn completed_hud_queue_rejects_a_changed_destination_at_checkpoint_admission() {
    let (_, mut admitted) = retained_return();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs");
    };
    let budget = inputs.budget;
    advance_until(&mut admitted, budget, |state| {
        matches!(state.late_hud, Stage::Completed(HudQueue::Queued(_)))
    });
    let checkpoint = admitted.production.save().unwrap();
    let restored = Production::restore(
        &checkpoint,
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    assert_observation_eq!(restored.state(), admitted.production.state());

    assert_rejected_at_admission(&admitted, |state| {
        let Stage::Completed(HudQueue::Queued(request)) = &mut state.late_hud else {
            panic!("queued HUD request");
        };
        request.template.target.0 ^= 1;
    });
}

#[test]
#[ignore = "requires the ordinary pending-loss native frame fixture"]
fn changed_status_restores_its_live_samples_and_rejects_contradictory_history() {
    let (_, mut admitted) = retained_frame_case("frame/health-hud-pending-loss");
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs");
    };
    let budget = inputs.budget;
    advance_until(&mut admitted, budget, |state| {
        matches!(state.late_hud, Stage::Gauging(_))
            && state.timing.hardware.math.division_edges_remaining() != 0
    });
    reject_gauge_write_and_register_changes(&admitted);
    let alternate = alternate_status_choice(frame_state(&admitted.production));
    assert_rejected_at_admission(&admitted, |state| {
        let Stage::Gauging(gauge) = &mut state.late_hud else {
            panic!("in-flight gauge");
        };
        gauge.chosen.clone_from(&alternate);
    });
    restore_next(&mut admitted, budget).unwrap();

    advance_until(&mut admitted, budget, |state| {
        matches!(state.late_hud, Stage::Gauging(_))
            && !state.timing.work.is_empty()
            && state.publications.events.iter().any(is_gauge_quotient)
    });
    reject_gauge_sample_changes(&admitted);
    restore_next(&mut admitted, budget).unwrap();

    advance_until(&mut admitted, budget, |state| {
        matches!(state.late_hud, Stage::Gauging(_)) && state.timing.work.is_empty()
    });
    reject_gauge_sample_changes(&admitted);
    restore_next(&mut admitted, budget).unwrap();
    advance_until(&mut admitted, budget, |state| {
        matches!(state.late_hud, Stage::Completed(HudQueue::Queued(_)))
    });
    reject_coherent_alternate_publication(&admitted, alternate);
}

fn alternate_status_choice(state: &frame::State) -> frame::hud::Hud {
    let Stage::Gauging(gauge) = &state.late_hud else {
        panic!("in-flight gauge");
    };
    let frame::hud::Hud::Countdown { ordinary, .. } = &gauge.chosen else {
        panic!("chosen ordinary HUD");
    };
    let interface_hud::Observation::Chosen(answer) = ordinary.0.status() else {
        panic!("chosen Status");
    };
    let mut pictures = answer.pictures;
    pictures.digits.rotate_left(1);
    let mut altered = state.hud.clone();
    let frame::hud::Hud::Countdown { ordinary, .. } = &mut altered else {
        panic!("HUD before Status");
    };
    assert!(
        ordinary
            .0
            .choose_status(&answer.rules, &pictures, answer.inputs)
    );
    altered
}

fn reject_coherent_alternate_publication(admitted: &AdmittedCase, mut alternate: frame::hud::Hud) {
    let frame::hud::Hud::Countdown { ordinary, .. } = &mut alternate else {
        panic!("alternate Status choice");
    };
    assert!(ordinary.0.apply());
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content");
    };
    let descriptor = &content.late_hud.as_ref().unwrap().queue;
    assert_rejected_at_admission(admitted, |state| {
        let regions = state.content.regions();
        let haze = regions
            .iter()
            .find(|region| region.start == content.haze_bands_address)
            .unwrap()
            .clone();
        let replaced = [haze.start, descriptor.source];
        let initial = regions
            .into_iter()
            .filter(|region| !replaced.contains(&region.start))
            .collect();
        let mut forged = frame::TransferContent::new(initial).unwrap();
        forged
            .complete(frame::Materialization {
                buffer: frame::Buffer::HazeBackdropBands,
                region: haze,
            })
            .unwrap();
        forged
            .complete(frame::Materialization {
                buffer: frame::Buffer::HudTilemap,
                region: data::timing::ContentRegion {
                    start: descriptor.source,
                    bytes: frame::hud::tilemap(alternate.state()).unwrap(),
                },
            })
            .unwrap();
        state.hud = alternate;
        state.content = forged;
    });
}

fn reject_gauge_write_and_register_changes(admitted: &AdmittedCase) {
    use data::timing::{DeviceWrite, Publication};
    assert_rejected_at_admission(admitted, |state| state.timing.hardware.math.remaining = 0);
    assert_rejected_at_admission(admitted, |state| state.timing.hardware.math.shift ^= 1);
    assert_rejected_at_admission(admitted, |state| state.timing.hardware.math.product ^= 1);
    for change in 0..3 {
        assert_rejected_at_admission(admitted, |state| {
            let trigger = state
                .publications
                .events
                .iter()
                .rev()
                .find_map(|event| {
                    let EventKind::Publication(write) = &event.kind else {
                        return None;
                    };
                    matches!(write.effect, Publication::Divisor(_)).then_some(write.clone())
                })
                .unwrap();
            let dividend = state
                .publications
                .events
                .iter_mut()
                .rev()
                .find(|event| {
                    matches!(
                        &event.kind,
                        EventKind::Publication(DeviceWrite {
                            effect: Publication::DividendLow(_),
                            ..
                        })
                    )
                })
                .unwrap();
            let EventKind::Publication(write) = &dividend.kind else {
                panic!("first dividend write")
            };
            dividend.kind = match change {
                0 => EventKind::ControllerCommitted,
                1 => EventKind::Publication(trigger),
                _ => EventKind::Publication(
                    DeviceWrite::from_cpu_port(write.port.0, write.value ^ 1).unwrap(),
                ),
            };
        });
    }
}

#[test]
#[ignore = "requires both common-origin Health and HUD Frame arms"]
fn authentic_stock_health_and_drawing_cannot_replace_the_changed_arm() {
    let (_, mut stock) = retained_frame_case("frame/health-hud-stock");
    let (_, mut changed) = retained_frame_case("frame/health-hud-pending-loss");
    let InputReader::Frame(inputs) = &changed.inputs else {
        panic!("frame inputs");
    };
    let budget = inputs.budget;
    let stock_origin = frame_state(&stock.production);
    let changed_origin = frame_state(&changed.production);
    assert_rejected_at_admission(&changed, |state| {
        state.health.clone_from(&stock_origin.health);
    });
    assert_rejected_at_admission(&stock, |state| {
        state.health.clone_from(&changed_origin.health);
    });

    advance_until(&mut stock, budget, |state| state.maintenance.in_flight());
    advance_until(&mut changed, budget, |state| state.maintenance.in_flight());
    let stock_work = frame_state(&stock.production);
    assert_rejected_at_admission(&changed, |state| {
        state.maintenance.clone_from(&stock_work.maintenance);
    });
    restore_next(&mut changed, budget).unwrap();

    let queued =
        |state: &frame::State| matches!(state.late_hud, Stage::Completed(HudQueue::Queued(_)));
    advance_until(&mut stock, budget, queued);
    advance_until(&mut changed, budget, queued);
    let authentic_queue = frame_state(&changed.production).late_hud.clone();
    let stock_queue = frame_state(&stock.production);
    reject_stock_drawing(&changed, stock_queue);
    assert_rejected_at_admission(&changed, |state| state.late_hud = Stage::Retired);

    let returned = |state: &frame::State| matches!(state.late_hud, Stage::Retired);
    advance_until(&mut stock, budget, returned);
    advance_until(&mut changed, budget, returned);
    let stock_return = frame_state(&stock.production);
    reject_stock_drawing(&changed, stock_return);
    assert_rejected_at_admission(&changed, |state| state.hud.clone_from(&stock_return.hud));
    assert_rejected_at_admission(&changed, |state| state.late_hud = authentic_queue);
    let saved = stock.production.save_completed_frame().unwrap();
    let restored =
        Production::restore_completed_frame(&saved, stock.content.clone(), stock.rules.clone())
            .unwrap();
    assert_observation_eq!(restored.state(), stock.production.state());
}

fn reject_stock_drawing(admitted: &AdmittedCase, stock: &frame::State) {
    assert_rejected_at_admission(admitted, |state| {
        state.s6_drawing.clone_from(&stock.s6_drawing);
    });
    assert_rejected_at_admission(admitted, |state| {
        state.frame_history.clone_from(&stock.frame_history);
    });
}

fn is_gauge_quotient(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Sample(Sample::QuotientLow, SampleValue::Byte(_))
    )
}

fn reject_gauge_sample_changes(admitted: &AdmittedCase) {
    for change in [None, Some(Sample::QuotientHigh), Some(Sample::QuotientLow)] {
        assert_rejected_at_admission(admitted, |state| {
            let event = state
                .publications
                .events
                .iter_mut()
                .find(|event| is_gauge_quotient(event))
                .unwrap();
            let EventKind::Sample(Sample::QuotientLow, SampleValue::Byte(value)) = event.kind
            else {
                panic!("live quotient sample");
            };
            event.kind = change.map_or(EventKind::ControllerCommitted, |sample| {
                let value = if sample == Sample::QuotientLow {
                    value ^ 1
                } else {
                    value
                };
                EventKind::Sample(sample, SampleValue::Byte(value))
            });
        });
    }
}

fn choose_status_with_stale_energy(state: &mut frame::State) {
    let Stage::Charging { chosen, .. } = &mut state.late_hud else {
        panic!("in-flight status");
    };
    let frame::hud::Hud::Countdown { ordinary, .. } = chosen else {
        panic!("chosen ordinary HUD");
    };
    let interface_hud::Observation::Chosen(answer) = ordinary.0.status() else {
        panic!("chosen status");
    };
    let mut inputs = answer.inputs;
    inputs.energy.now ^= 1;
    let mut before = state.hud.clone();
    let frame::hud::Hud::Countdown { ordinary, .. } = &mut before else {
        panic!("ordinary HUD before status choice");
    };
    assert!(
        ordinary
            .0
            .choose_status(&answer.rules, &answer.pictures, inputs)
    );
    *chosen = before;
}

#[track_caller]
fn assert_rejected_at_admission(admitted: &AdmittedCase, change: impl FnOnce(&mut frame::State)) {
    let mut altered = frame_state(&admitted.production).clone();
    change(&mut altered);
    let forged = Production::admit(
        ProductionSeed::Frame(Box::new(altered)),
        admitted.content.clone(),
        admitted.rules.clone(),
    );
    assert!(matches!(
        forged,
        Err(AdmissionFault::Frame(fault)) if *fault == frame::Fault::InvalidPlayerEntry
    ));
}

fn frame_state(production: &Production) -> &frame::State {
    let OwnedState::Frame(state) = production.state() else {
        panic!("frame state");
    };
    state
}

fn finish_with_hud_upload(
    production: &mut Production,
    budget: MasterTicks,
    template: &frame::suffix_caller::HudTransferTemplate,
) -> frame::Outputs {
    let mut events = Vec::new();
    let mut enqueued = None;
    for _ in 0..500_000 {
        let ProductionOutputs::Frame(output) =
            production.advance(AdvanceInput::Frame { budget }).unwrap()
        else {
            panic!("frame output");
        };
        events.extend(output.events);
        let state = frame_state(production);
        match (&state.late_hud, output.progress) {
            (Stage::Charging { .. }, frame::Progress::Advanced) => assert!(enqueued.is_none()),
            (Stage::Completed(HudQueue::Queued(request)), frame::Progress::Advanced) => {
                assert_observation_eq!(&request.template, template);
                enqueued.get_or_insert(events.len());
            }
            (Stage::Retired, frame::Progress::Completed) => {
                let enqueue = enqueued.expect("one HUD request queued before native return");
                let (status, _) = state.hud.publication().unwrap();
                let tilemap = frame::hud::tilemap(status).unwrap();
                assert_eq!(tilemap.len(), usize::from(template.length));
                assert_eq!(hud_upload_count(&events, &tilemap), 1);
                assert_eq!(hud_upload_count(&events[enqueue..], &tilemap), 1);
                return frame::Outputs {
                    progress: output.progress,
                    events,
                };
            }
            _ => panic!("unexpected HUD queue transition"),
        }
    }
    panic!("native frame did not return");
}

fn hud_upload_count(events: &[Event], tilemap: &[u8]) -> usize {
    // One native bulk body writes the completed HUD tilemap to the VRAM data ports.
    const VRAM_DATA_LOW: TransferPort = TransferPort(0x18);
    const VRAM_DATA_HIGH: TransferPort = TransferPort(0x19);
    events
        .iter()
        .enumerate()
        .filter(|(_, event)| matches!(event.kind, EventKind::BulkService(_)))
        .filter(|(index, _)| {
            let body = events[index.wrapping_add(1)..]
                .iter()
                .take_while(|event| {
                    !matches!(
                        event.kind,
                        EventKind::BulkService(_)
                            | EventKind::LineService(_)
                            | EventKind::Publication(_)
                    )
                })
                .filter_map(|event| {
                    if let EventKind::Payload(payload) = &event.kind {
                        Some((payload.target, payload.value))
                    } else {
                        None
                    }
                });
            let expected = tilemap.iter().enumerate().map(|(index, byte)| {
                let port = if index.is_multiple_of(2) {
                    VRAM_DATA_LOW
                } else {
                    VRAM_DATA_HIGH
                };
                (port, *byte)
            });
            body.eq(expected)
        })
        .count()
}
