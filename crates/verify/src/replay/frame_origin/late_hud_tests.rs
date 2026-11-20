use super::runtime_helpers::{advance_until, retained_return};
use super::*;
use data::timing::{MasterTicks, TransferPort};
use frame::{late_hud::Stage, suffix_caller::HudQueue};
use game::production::timing::{Event, EventKind};
use game::production::{AdvanceInput, ProductionOutputs};
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
