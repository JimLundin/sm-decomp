use super::*;
use data::dma::BusAddress;
use data::timing::{
    ChannelConfig, DeviceWrite, Exposure, Publication, PublicationPort, TransferPattern,
    TransferPort,
};

pub(in crate::production) fn state(line: u16, horizontal: u16) -> State {
    let position = BeamPosition { line, horizontal };
    let history = std::array::from_fn(|index| BeamPosition {
        line,
        horizontal: horizontal.saturating_sub(
            u16::try_from(5_usize.wrapping_sub(index))
                .unwrap()
                .wrapping_mul(2),
        ),
    });
    State {
        hardware: Hardware {
            now: MasterTicks(0),
            beam: Beam {
                odd: false,
                position,
                history,
                subtick: 0,
                residue: 0,
                line_ticks: 1364,
                refreshed: true,
                thresholds: Thresholds {
                    initialize_at: 12,
                    initialize: true,
                    visible: true,
                },
            },
            charge_debt: 0,
            interrupts: Interrupts::default(),
            math: Math::default(),
            controller: Controller::default(),
            transfers: Transfers::default(),
        },
        work: Vec::new(),
        interrupt_boundary: None,
        pending_handler: None,
        interrupted_wait: None,
    }
}
fn profile(runs: Vec<Run>) -> Profile {
    Profile { runs }
}
fn run(
    count: u32,
    cycle: Cycle,
    kind: ExposureKind,
    poll_before: bool,
    effect: DeviceEffect,
) -> Run {
    Run {
        count,
        exposure: Exposure {
            cycle,
            kind,
            poll_before,
            effect,
        },
    }
}
fn internal(count: u32) -> Profile {
    profile(vec![internal_run(count)])
}
fn internal_run(count: u32) -> Run {
    run(
        count,
        Cycle::Six,
        ExposureKind::Internal,
        false,
        DeviceEffect::None,
    )
}
fn finish(state: &mut State, budget: u64, content: &[ContentRegion]) -> Vec<Event> {
    let mut events = Vec::new();
    for _ in 0..100_000 {
        let (progress, output) = state.step(MasterTicks(budget), content).unwrap();
        events.extend(output);
        match progress {
            Progress::Completed(_) | Progress::Idle => return events,
            Progress::Advanced | Progress::HandlerEntered(_) | Progress::HandlerReturned(_) => {}
            Progress::HandlerPortionCompleted(_) | Progress::HandlerNeedsWork(_) => {
                panic!("fixture needs fresh handler work")
            }
            Progress::Interrupt(_) => panic!("unexpected interrupt in fixture"),
        }
    }
    panic!("fixture did not finish")
}
fn restore(state: &State) -> State {
    canonical::from_bytes(&canonical::to_bytes(state)).unwrap()
}

#[test]
fn interrupted_wait_refuses_a_resume_cursor_without_its_spent_work() {
    let mut value = state(225, 304);
    value.hardware.interrupts.nmi.transition = true;
    value.begin_wait(WorkId(70)).unwrap();
    loop {
        let (progress, _) = value.step(MasterTicks(8), &[]).unwrap();
        if progress == Progress::Interrupt(Interrupt::Nmi) {
            break;
        }
        assert_eq!(progress, Progress::Advanced);
    }
    value.enter_native(WorkId(71)).unwrap();
    assert_eq!(
        restore(&value).wait_resume(),
        Some(WaitResume::PendingBusyBranch)
    );
    value.interrupted_wait.as_mut().unwrap().wait.cursor.run = 4;
    assert_eq!(value.validate(), Err(Fault::InvalidState));
}

#[test]
fn partial_charge_retains_subpoll_debt_and_emits_sample_once() {
    let mut full = state(20, 20);
    full.hardware.math.product = 1234;
    full.begin(
        WorkId(1),
        profile(vec![run(
            1,
            Cycle::Eight,
            ExposureKind::Read,
            false,
            DeviceEffect::Sample(Sample::ProductLow),
        )]),
    )
    .unwrap();
    let mut split = full.clone();
    let expected = finish(&mut full, 100, &[]);
    for _ in 0..4 {
        split.step(MasterTicks(1), &[]).unwrap();
    }
    assert_eq!(split.hardware.now, MasterTicks(1));
    assert_eq!(split.hardware.beam.subtick, 1);
    assert_eq!(split.hardware.charge_debt, 1);
    split = restore(&split);
    let actual = finish(&mut split, 1, &[]);
    assert_eq!(actual, expected);
    assert_eq!(canonical::to_bytes(&split), canonical::to_bytes(&full));
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].at, MasterTicks(4));
}

#[test]
fn pending_sensitive_charge_is_resolved_once_before_partial_resume() {
    let mut value = state(20, 20);
    value.hardware.interrupts.irq.transition = true;
    value
        .begin(
            WorkId(2),
            profile(vec![run(
                1,
                Cycle::Six,
                ExposureKind::PendingInternal,
                true,
                DeviceEffect::None,
            )]),
        )
        .unwrap();
    for _ in 0..4 {
        value.step(MasterTicks(1), &[]).unwrap();
    }
    assert_eq!(
        value.work[0].cursor.stage,
        Stage::Charge(Charge::BeforeSample { remaining: 1 })
    );
    value.hardware.interrupts.irq.pending = false;
    value = restore(&value);
    value.step(MasterTicks(1), &[]).unwrap();
    assert_eq!(
        value.work[0].cursor.stage,
        Stage::Charge(Charge::AfterSample { remaining: 4 })
    );
    finish(&mut value, 1, &[]);
    assert_eq!(value.hardware.now, MasterTicks(6));
}

#[test]
fn interrupted_parent_resumes_without_repeating_its_completed_write() {
    let mut value = state(20, 20);
    value.hardware.interrupts.irq.transition = true;
    value
        .begin(
            WorkId(3),
            profile(vec![
                run(
                    1,
                    Cycle::Eight,
                    ExposureKind::Write,
                    true,
                    DeviceEffect::Publish(
                        DeviceWrite::from_cpu_port(TransferPort(0).publication_port().0, 7)
                            .unwrap(),
                    ),
                ),
                run(
                    1,
                    Cycle::Six,
                    ExposureKind::Internal,
                    false,
                    DeviceEffect::None,
                ),
            ]),
        )
        .unwrap();
    let mut prefix = Vec::new();
    loop {
        let (progress, events) = value.step(MasterTicks(1), &[]).unwrap();
        prefix.extend(events);
        if progress == Progress::Interrupt(Interrupt::Irq) {
            break;
        }
    }
    assert_eq!(value.hardware.now, MasterTicks(8));
    assert_eq!(prefix.len(), 1);
    prefix.extend(value.enter_interrupt(WorkId(4), internal(1)).unwrap());
    let mut resumed = restore(&value);
    let suffix = finish(&mut value, 100, &[]);
    assert_eq!(finish(&mut resumed, 1, &[]), suffix);
    assert_eq!(canonical::to_bytes(&resumed), canonical::to_bytes(&value));
    assert_eq!(value.hardware.now, MasterTicks(80));
    assert_eq!(value.hardware.interrupts.policy, IrqPolicy::Eligible);
    assert_eq!(
        prefix
            .iter()
            .filter(|event| matches!(event.kind, EventKind::Publication(_)))
            .count(),
        1
    );
    assert_eq!(suffix, Vec::new());
}

#[test]
fn multiplication_only_uses_exposure_edges_and_survives_restore() {
    let mut value = state(20, 20);
    value
        .hardware
        .publish(&write(&Publication::Multiplicand(7)), &mut Vec::new())
        .unwrap();
    value
        .hardware
        .publish(&write(&Publication::Multiplier(9)), &mut Vec::new())
        .unwrap();
    value.hardware.charge(100, true, &mut Vec::new()).unwrap();
    assert_eq!(value.hardware.math.remaining, 8);
    value.begin(WorkId(5), internal(8)).unwrap();
    for _ in 0..10 {
        value.step(MasterTicks(1), &[]).unwrap();
    }
    let mut resumed = restore(&value);
    finish(&mut value, 100, &[]);
    finish(&mut resumed, 1, &[]);
    assert_eq!(value.hardware.math.product, 63);
    assert_eq!(value.hardware.math.remaining, 0);
    assert_eq!(canonical::to_bytes(&resumed), canonical::to_bytes(&value));
}

#[test]
fn original_controller_charge_end_debt_and_commitment_survive_restore() {
    let mut value = state(225, 100);
    value.hardware.interrupts.enable.controller = true;
    value.hardware.controller.committed_pad = PhysicalPad(32768);
    value
        .schedule_pad_changes(&[PadChange {
            at: MasterTicks(0),
            pad: PhysicalPad(0),
        }])
        .unwrap();
    value.hardware.controller.accumulator = 250;
    let mut events = Vec::new();
    value.hardware.charge(3, false, &mut events).unwrap();
    let mut resumed = restore(&value);
    value.hardware.charge(3, true, &mut events).unwrap();
    let mut resumed_events = Vec::new();
    resumed
        .hardware
        .charge(3, true, &mut resumed_events)
        .unwrap();
    assert_eq!(events, resumed_events);
    assert_eq!(events.len(), 1);
    assert_eq!(value.hardware.controller.serial, 1);
    assert_eq!(value.hardware.controller.words, [1, 0, 0, 0]);
    assert_eq!(value.hardware.controller.pending_pad.len(), 1);
    assert_eq!(canonical::to_bytes(&resumed), canonical::to_bytes(&value));
    value.hardware.charge(512, true, &mut Vec::new()).unwrap();
    assert_eq!(value.hardware.controller.serial, 2);
    assert_eq!(value.hardware.controller.accumulator, 256);
}

#[test]
fn refresh_follows_charge_end_and_never_advances_intrinsic_work_or_math() {
    let mut value = state(20, 536);
    value.hardware.beam.refreshed = false;
    value.hardware.math.remaining = 8;
    value.begin(WorkId(6), internal(1)).unwrap();
    let mut split = value.clone();
    finish(&mut value, 100, &[]);
    finish(&mut split, 1, &[]);
    assert_eq!(value.hardware.now, MasterTicks(46));
    assert_eq!(value.hardware.math.remaining, 7);
    assert_eq!(canonical::to_bytes(&split), canonical::to_bytes(&value));
}

fn bulk() -> (State, Vec<ContentRegion>) {
    let mut value = state(20, 100);
    let [first, _, _, _, _, _, _, _] = &mut value.hardware.transfers.channels;
    first.config = ChannelConfig {
        source_bank: 2,
        target: TransferPort(10),
        pattern: TransferPattern::Pair,
        ..ChannelConfig::default()
    };
    first.source_offset = 20;
    first.pointer = 2;
    first.bulk_enabled = true;
    value.hardware.transfers.pending_bulk = true;
    (
        value,
        vec![ContentRegion {
            start: BusAddress {
                bank: 2,
                offset: 20,
            },
            bytes: vec![17, 19],
        }],
    )
}

#[test]
fn armed_bulk_permits_cpu_cycle_then_flushes_global_pipeline_in_order() {
    let (mut value, content) = bulk();
    value.begin(WorkId(7), internal(2)).unwrap();
    for _ in 0..3 {
        value.step(MasterTicks(100), &content).unwrap();
    }
    assert!(value.hardware.transfers.active);
    assert_eq!(value.hardware.now, MasterTicks(0));
    let mut resumed = restore(&value);
    let expected = finish(&mut value, 100, &content);
    assert_eq!(finish(&mut resumed, 1, &content), expected);
    assert_eq!(canonical::to_bytes(&resumed), canonical::to_bytes(&value));
    let payloads: Vec<_> = expected
        .iter()
        .filter_map(|event| {
            if let EventKind::Payload(payload) = event.kind {
                Some((event.at, payload))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        payloads,
        vec![
            (
                MasterTicks(36),
                Payload {
                    target: TransferPort(10),
                    value: 17
                }
            ),
            (
                MasterTicks(44),
                Payload {
                    target: TransferPort(11),
                    value: 19
                }
            )
        ]
    );
    assert_eq!(value.hardware.now, MasterTicks(54));
    assert_eq!(value.hardware.transfers.intrinsic, 38);
    assert!(value.hardware.transfers.pipeline.is_none());
}

#[test]
fn unsupported_transfer_discards_staged_wall_state_and_payloads() {
    let (mut value, content) = bulk();
    value.begin(WorkId(8), internal(2)).unwrap();
    loop {
        if value.work[0].cursor.run == 0
            && value.work[0].cursor.occurrence == 1
            && value.work[0].cursor.stage == Stage::Arbitrate
        {
            break;
        }
        value.step(MasterTicks(100), &content).unwrap();
    }
    let before = canonical::to_bytes(&value);
    assert_eq!(
        value.step(MasterTicks(100), &[]),
        Err(Fault::MissingContent)
    );
    assert_eq!(canonical::to_bytes(&value), before);
    finish(&mut value, 100, &content);
}

#[test]
fn zero_alignment_and_completion_residue_charge_full_cycles() {
    let mut value = state(20, 96);
    let mut events = Vec::new();
    value.hardware.transfers.active = true;
    value.hardware.transfers.pending_line = Some(LineTransfer::Visible);
    let [channel, _, _, _, _, _, _, _] = &mut value.hardware.transfers.channels;
    channel.line.enabled = true;
    channel.line.completed = true;
    value
        .hardware
        .arbitrate(Cycle::Eight, &[], &mut events)
        .unwrap();
    assert_eq!(value.hardware.transfers.intrinsic, 16);
    assert_eq!(value.hardware.now, MasterTicks(24));
}

#[test]
fn nmi_uses_delayed_history_then_held_poll_before_transition() {
    let mut value = state(224, 1360);
    value.hardware.interrupts.enable.nmi = true;
    value.hardware.charge(6, true, &mut Vec::new()).unwrap();
    assert!(value.hardware.interrupts.nmi.physical.valid);
    assert!(value.hardware.interrupts.nmi.physical.hold);
    assert!(!value.hardware.interrupts.nmi.transition);
    value.hardware.charge(4, true, &mut Vec::new()).unwrap();
    assert!(value.hardware.interrupts.nmi.transition);
}

#[test]
fn invalid_or_busy_device_work_rejects_atomically() {
    let mut value = state(20, 20);
    assert_eq!(
        value.begin(WorkId(9), Profile { runs: Vec::new() }),
        Err(Fault::InvalidProfile)
    );
    value.hardware.math.remaining = 4;
    value
        .begin(
            WorkId(9),
            profile(vec![run(
                1,
                Cycle::Eight,
                ExposureKind::Write,
                false,
                DeviceEffect::Publish(write(&Publication::Multiplier(3))),
            )]),
        )
        .unwrap();
    while value.work[0].cursor.stage != Stage::Publish {
        value.step(MasterTicks(100), &[]).unwrap();
    }
    let before = canonical::to_bytes(&value);
    assert_eq!(value.step(MasterTicks(100), &[]), Err(Fault::MathBusy));
    assert_eq!(canonical::to_bytes(&value), before);
}

#[test]
fn variable_short_line_changes_next_line_residue_and_input_commitment_once() {
    let mut value = state(240, 1358);
    value.hardware.beam.odd = true;
    value.hardware.beam.line_ticks = 1360;
    value.hardware.controller.committed_pad = PhysicalPad(1);
    value
        .schedule_pad_changes(&[PadChange {
            at: MasterTicks(0),
            pad: PhysicalPad(2),
        }])
        .unwrap();
    let mut events = Vec::new();
    value.hardware.charge(2, true, &mut events).unwrap();
    assert_eq!(
        value.hardware.beam.position,
        BeamPosition {
            line: 241,
            horizontal: 0
        }
    );
    assert_eq!(value.hardware.beam.residue, 0);
    assert_eq!(value.hardware.controller.committed_pad, PhysicalPad(2));
    assert_eq!(events.len(), 1);
    let mut resumed = restore(&value);
    resumed.hardware.charge(2, true, &mut events).unwrap();
    assert_eq!(events.len(), 1);
}

fn write(effect: &Publication) -> DeviceWrite {
    if let Publication::Multiplicand(value) = effect {
        DeviceWrite::from_cpu_port(data::timing::MULTIPLICAND_PORT.0, *value).unwrap()
    } else if let Publication::Multiplier(value) = effect {
        DeviceWrite::from_cpu_port(data::timing::MULTIPLIER_PORT.0, *value).unwrap()
    } else {
        panic!("unqualified constructed publication")
    }
}

#[test]
fn refresh_within_bulk_delays_payloads_without_charging_math_or_transfer_work() {
    let (mut delayed, content) = bulk();
    delayed.hardware.beam.position.horizontal = 520;
    delayed.hardware.beam.history = state(20, 520).hardware.beam.history;
    delayed.hardware.beam.refreshed = false;
    delayed.hardware.math.remaining = 8;
    let mut unstalled = delayed.clone();
    unstalled.hardware.beam.refreshed = true;
    delayed.begin(WorkId(10), internal(2)).unwrap();
    unstalled.begin(WorkId(10), internal(2)).unwrap();
    let delayed_events = finish(&mut delayed, 1, &content);
    let unstalled_events = finish(&mut unstalled, 100, &content);
    assert_eq!(delayed_events.len(), unstalled_events.len());
    for (delayed, unstalled) in delayed_events.iter().zip(&unstalled_events) {
        assert_eq!(delayed.kind, unstalled.kind);
        if matches!(delayed.kind, EventKind::Payload(_)) {
            assert_eq!(delayed.at.0, unstalled.at.0.wrapping_add(40));
        } else {
            assert_eq!(delayed.at, unstalled.at);
        }
    }
    assert_eq!(
        delayed.hardware.now.0,
        unstalled.hardware.now.0.wrapping_add(40)
    );
    assert_eq!(
        delayed.hardware.transfers.intrinsic,
        unstalled.hardware.transfers.intrinsic
    );
    assert_eq!(delayed.hardware.math.remaining, 6);
}

#[test]
fn all_line_payloads_precede_updates_and_the_last_update_flushes_global_byte() {
    let mut value = state(20, 100);
    for (channel, (offset, pattern)) in value
        .hardware
        .transfers
        .channels
        .iter_mut()
        .zip([(20, TransferPattern::Pair), (30, TransferPattern::Single)])
    {
        channel.config.source_bank = 2;
        channel.config.pattern = pattern;
        channel.config.target = TransferPort(u8::try_from(offset).unwrap());
        channel.table = offset;
        channel.lines = 1;
        channel.line = LineStatus {
            enabled: true,
            completed: false,
            payload: true,
        };
    }
    value.hardware.transfers.active = true;
    value.hardware.transfers.pending_line = Some(LineTransfer::Visible);
    value.begin(WorkId(11), internal(1)).unwrap();
    let content = [
        ContentRegion {
            start: BusAddress {
                bank: 2,
                offset: 20,
            },
            bytes: vec![11, 13, 0],
        },
        ContentRegion {
            start: BusAddress {
                bank: 2,
                offset: 30,
            },
            bytes: vec![17, 0],
        },
    ];
    let events = finish(&mut value, 1, &content);
    let payloads: Vec<_> = events
        .iter()
        .filter_map(|event| {
            if let EventKind::Payload(payload) = event.kind {
                Some((event.at, payload.value))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        payloads,
        [
            (MasterTicks(28), 11),
            (MasterTicks(36), 13),
            (MasterTicks(44), 17)
        ]
    );
    let services: Vec<_> = events
        .iter()
        .filter_map(|event| {
            if let EventKind::LineService(slot) = event.kind {
                Some((event.at, slot))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        services,
        [
            (MasterTicks(12), ChannelSlot::One),
            (MasterTicks(28), ChannelSlot::Two)
        ]
    );
    assert_eq!(value.hardware.transfers.intrinsic, 52);
    assert_eq!(value.hardware.now, MasterTicks(60));
    assert!(value.hardware.transfers.pipeline.is_none());
    assert!(
        value
            .hardware
            .transfers
            .channels
            .iter()
            .take(2)
            .all(|channel| channel.line.completed)
    );
}

#[test]
fn completed_final_indirect_channel_reads_one_pointer_byte_and_no_second_byte() {
    let mut value = state(20, 100);
    let [channel, _, _, _, _, _, _, _] = &mut value.hardware.transfers.channels;
    channel.config.source_bank = 2;
    channel.config.indirect = true;
    channel.source_offset = 20;
    channel.line.enabled = true;
    value.hardware.transfers.active = true;
    value.hardware.transfers.pending_line = Some(LineTransfer::Initialize);
    value.begin(WorkId(12), internal(1)).unwrap();
    let content = [ContentRegion {
        start: BusAddress {
            bank: 2,
            offset: 20,
        },
        bytes: vec![0, 81],
    }];
    assert_eq!(finish(&mut value, 1, &content), Vec::new());
    let [channel, _, _, _, _, _, _, _] = &value.hardware.transfers.channels;
    assert!(channel.line.completed);
    assert_eq!(channel.pointer, 20736);
    assert_eq!(channel.table, 22);
    assert_eq!(value.hardware.transfers.intrinsic, 28);
    assert_eq!(value.hardware.now, MasterTicks(36));
}

#[test]
fn partial_channel_write_preserves_every_unrelated_cursor_and_exact_byte_identity() {
    let (mut value, _) = bulk();
    let before = value.hardware.transfers.channels[0].clone();
    let publication =
        DeviceWrite::from_cpu_port(data::timing::FIRST_CHANNEL_TARGET_PORT.0, 43).unwrap();
    let mut events = Vec::new();
    value.hardware.publish(&publication, &mut events).unwrap();
    let channel = &value.hardware.transfers.channels[0];
    assert_eq!(channel.source_offset, before.source_offset);
    assert_eq!(channel.pointer, before.pointer);
    assert_eq!(channel.table, before.table);
    assert_eq!(channel.lines, before.lines);
    assert_eq!(channel.line, before.line);
    assert_eq!(channel.bulk_enabled, before.bulk_enabled);
    assert_eq!(channel.config.target, TransferPort(43));
    assert_eq!(
        events,
        [Event {
            at: MasterTicks(0),
            kind: EventKind::Publication(publication)
        }]
    );
}

#[test]
fn unqualified_device_or_byte_mutation_mismatch_rejects_before_any_work() {
    let mut value = state(20, 20);
    let before = canonical::to_bytes(&value);
    let mut publication = write(&Publication::Multiplier(7));
    publication.value = 9;
    assert_eq!(
        value.begin(
            WorkId(13),
            profile(vec![run(
                1,
                Cycle::Eight,
                ExposureKind::Write,
                false,
                DeviceEffect::Publish(publication)
            )])
        ),
        Err(Fault::InvalidProfile)
    );
    assert_eq!(canonical::to_bytes(&value), before);
    let publication = DeviceWrite {
        port: PublicationPort(65535),
        value: 0,
        effect: Publication::Downstream,
    };
    assert_eq!(
        value.begin(
            WorkId(13),
            profile(vec![run(
                1,
                Cycle::Eight,
                ExposureKind::Write,
                false,
                DeviceEffect::Publish(publication)
            )])
        ),
        Err(Fault::InvalidProfile)
    );
    assert_eq!(canonical::to_bytes(&value), before);
}

#[test]
fn controller_commit_inside_a_charge_has_the_same_crossing_instant_when_split() {
    let mut full = state(240, 1362);
    full.schedule_pad_changes(&[PadChange {
        at: MasterTicks(0),
        pad: PhysicalPad(2),
    }])
    .unwrap();
    full.begin(WorkId(14), internal(1)).unwrap();
    let mut split = full.clone();
    let full_events = finish(&mut full, 100, &[]);
    let split_events = finish(&mut split, 1, &[]);
    assert_eq!(full_events, split_events);
    assert_eq!(
        full_events,
        [Event {
            at: MasterTicks(2),
            kind: EventKind::ControllerCommitted
        }]
    );
    assert_eq!(canonical::to_bytes(&full), canonical::to_bytes(&split));
}

#[test]
fn controller_changes_straddling_the_latch_survive_until_the_next_sample() {
    let mut value = state(240, 1362);
    let held = PhysicalPad(0x8000);
    let changes = [
        PadChange {
            at: MasterTicks(1),
            pad: PhysicalPad(0x4000),
        },
        PadChange {
            at: MasterTicks(2),
            pad: held,
        },
        PadChange {
            at: MasterTicks(3),
            pad: PhysicalPad::default(),
        },
    ];
    value.schedule_pad_changes(&changes).unwrap();
    let mut events = Vec::new();
    value.hardware.charge(2, true, &mut events).unwrap();
    assert_eq!(value.hardware.controller.committed_pad, held);
    assert_eq!(value.hardware.controller.pending_pad, changes[2..]);
    let mut value = restore(&value);

    // Advance to the next serial poll, which reads the latched state despite the release.
    value.hardware.interrupts.enable.controller = true;
    while value.hardware.beam.position.line != 225 {
        value.hardware.charge(128, true, &mut events).unwrap();
    }
    while value.hardware.controller.serial < 16 {
        value.hardware.charge(128, true, &mut events).unwrap();
    }
    assert_eq!(value.hardware.controller.words, [held.0, 0, 0, 0]);

    // Two more latches exercise release followed by holding the released state.
    for _ in 0..5700 {
        value.hardware.charge(128, true, &mut events).unwrap();
    }
    assert_eq!(
        value.hardware.controller.committed_pad,
        PhysicalPad::default()
    );
    assert_eq!(value.hardware.controller.pending_pad, []);
    assert_eq!(value.hardware.controller.words, [0; 4]);
    assert_eq!(
        events
            .iter()
            .filter(|event| event.kind == EventKind::ControllerCommitted)
            .count(),
        2
    );
}

#[test]
fn an_unqualified_transfer_device_discards_the_entire_synchronous_service() {
    let (mut value, content) = bulk();
    value.hardware.transfers.channels[0].config.target = TransferPort(255);
    value.begin(WorkId(15), internal(2)).unwrap();
    while value.work[0].cursor.occurrence != 1 || value.work[0].cursor.stage != Stage::Arbitrate {
        value.step(MasterTicks(100), &content).unwrap();
    }
    let before = canonical::to_bytes(&value);
    assert_eq!(
        value.step(MasterTicks(100), &content),
        Err(Fault::UnsupportedDevice)
    );
    assert_eq!(canonical::to_bytes(&value), before);
}

#[test]
fn restored_partial_work_without_its_charge_debt_is_rejected_without_repair() {
    let mut value = state(20, 20);
    value.begin(WorkId(16), internal(1)).unwrap();
    for _ in 0..4 {
        value.step(MasterTicks(1), &[]).unwrap();
    }
    value.validate().unwrap();
    value.hardware.charge_debt = 0;
    value = restore(&value);
    let before = canonical::to_bytes(&value);
    assert_eq!(value.validate(), Err(Fault::InvalidState));
    assert_eq!(value.step(MasterTicks(1), &[]), Err(Fault::InvalidState));
    assert_eq!(canonical::to_bytes(&value), before);
}

fn through(value: &mut State, boundary: Progress) -> Vec<Event> {
    let mut output = Vec::new();
    for _ in 0..100_000 {
        let (progress, events) = value.step(MasterTicks(1), &[]).unwrap();
        output.extend(events);
        if progress == boundary {
            return output;
        }
    }
    panic!("fixture did not reach its actual boundary")
}

#[test]
fn handler_portions_retain_entry_mask_and_parent_until_the_explicit_native_return() {
    let mut portions = state(20, 20);
    portions.hardware.interrupts.irq.transition = true;
    portions
        .begin(
            WorkId(40),
            profile(vec![
                run(1, Cycle::Six, ExposureKind::Read, true, DeviceEffect::None),
                run(
                    1,
                    Cycle::Six,
                    ExposureKind::Internal,
                    false,
                    DeviceEffect::None,
                ),
            ]),
        )
        .unwrap();
    through(&mut portions, Progress::Interrupt(Interrupt::Irq));
    let mut whole = portions.clone();
    let publication = run(
        1,
        Cycle::Six,
        ExposureKind::Write,
        false,
        DeviceEffect::Publish(write(&Publication::Multiplicand(7))),
    );
    let returned = run(
        1,
        Cycle::Six,
        ExposureKind::Internal,
        false,
        DeviceEffect::None,
    );
    let mut whole_events = whole
        .enter_interrupt(
            WorkId(41),
            profile(vec![publication.clone(), returned.clone()]),
        )
        .unwrap();
    whole_events.extend(finish(&mut whole, 100, &[]));
    let mut portion_events = portions.enter_native(WorkId(41)).unwrap();
    portion_events.extend(through(&mut portions, Progress::HandlerEntered(WorkId(41))));
    assert_eq!(portions.hardware.now, MasterTicks(66));
    assert_eq!(portions.hardware.interrupts.policy, IrqPolicy::Masked);
    assert_eq!(portions.hardware.math.multiplicand, 0);
    portions = restore(&portions);
    portions.validate().unwrap();
    assert_eq!(
        portions.step(MasterTicks(100), &[]).unwrap(),
        (Progress::HandlerNeedsWork(WorkId(41)), Vec::new())
    );
    portions
        .begin_handler_portion(
            WorkId(42),
            profile(vec![publication]),
            HandlerCompletion::Continue,
        )
        .unwrap();
    portion_events.extend(through(
        &mut portions,
        Progress::HandlerPortionCompleted(WorkId(42)),
    ));
    assert_eq!(portions.hardware.now, MasterTicks(72));
    assert_eq!(portions.hardware.interrupts.policy, IrqPolicy::Masked);
    assert_eq!(portions.work[0].cursor.completed, 1);
    assert_eq!(portions.hardware.math.multiplicand, 7);
    portions = restore(&portions);
    portions.validate().unwrap();
    assert_eq!(
        portions.step(MasterTicks(100), &[]).unwrap(),
        (Progress::HandlerNeedsWork(WorkId(41)), Vec::new())
    );
    assert_eq!(
        portions.begin(WorkId(99), internal(1)),
        Err(Fault::WorkBusy)
    );
    portions
        .begin_handler_portion(
            WorkId(43),
            profile(vec![returned]),
            HandlerCompletion::NativeReturn,
        )
        .unwrap();
    portion_events.extend(through(
        &mut portions,
        Progress::HandlerReturned(WorkId(41)),
    ));
    assert_eq!(portions.hardware.now, MasterTicks(78));
    assert_eq!(portions.hardware.interrupts.policy, IrqPolicy::Eligible);
    portion_events.extend(finish(&mut portions, 1, &[]));
    assert_eq!(portions.hardware.now, MasterTicks(84));
    assert_eq!(portion_events, whole_events);
    assert_eq!(canonical::to_bytes(&portions), canonical::to_bytes(&whole));
}

#[test]
fn review_suspended_parent_must_be_at_a_completed_exposure_boundary() {
    let mut value = state(20, 20);
    value.hardware.interrupts.irq.transition = true;
    value
        .begin(
            WorkId(80),
            profile(vec![
                run(1, Cycle::Six, ExposureKind::Read, true, DeviceEffect::None),
                run(
                    1,
                    Cycle::Six,
                    ExposureKind::Internal,
                    false,
                    DeviceEffect::None,
                ),
            ]),
        )
        .unwrap();
    through(&mut value, Progress::Interrupt(Interrupt::Irq));
    value.enter_native(WorkId(81)).unwrap();
    value.validate().unwrap();
    // No supported admission can suspend the parent before its completed exposure.
    // Its own counters agree, but the suspended one-tick debt has no hardware owner.
    let parent = value.work.first_mut().unwrap();
    parent.cursor.stage = Stage::Charge(Charge::Whole { remaining: 5 });
    parent.cursor.intrinsic = MasterTicks(7);
    value = restore(&value);
    assert_eq!(value.validate(), Err(Fault::InvalidState));
}

fn division_write(port: PublicationPort, byte: u8) -> Run {
    run(
        1,
        Cycle::Six,
        ExposureKind::Write,
        false,
        DeviceEffect::Publish(DeviceWrite::from_cpu_port(port.0, byte).unwrap()),
    )
}

fn division_read(sample: Sample) -> Run {
    run(
        1,
        Cycle::Six,
        ExposureKind::Read,
        false,
        DeviceEffect::Sample(sample),
    )
}

#[test]
fn division_samples_at_the_read_before_its_edge_and_resumes_once() {
    use data::timing::{DIVIDEND_HIGH_PORT, DIVIDEND_LOW_PORT, DIVISOR_PORT};

    let mut initial = state(20, 20);
    initial.hardware.math.operand = 0xa5c0;
    initial
        .begin(
            WorkId(1),
            profile(vec![
                division_write(DIVIDEND_LOW_PORT, 0xbc),
                division_write(DIVIDEND_HIGH_PORT, 0x34),
                division_write(DIVISOR_PORT, 0),
                internal_run(15),
                division_read(Sample::QuotientHigh),
                division_read(Sample::QuotientHigh),
                division_read(Sample::ProductLow),
                division_read(Sample::ProductHigh),
            ]),
        )
        .unwrap();
    let mut direct = initial.clone();
    let expected = finish(&mut direct, 100, &[]);
    let samples: Vec<_> = expected
        .iter()
        .filter_map(|event| {
            if let EventKind::Sample(sample, SampleValue::Byte(byte)) = event.kind {
                Some((event.at, sample, byte))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        samples,
        [
            (MasterTicks(110), Sample::QuotientHigh, 0x7f),
            (MasterTicks(116), Sample::QuotientHigh, 0xff),
            (MasterTicks(122), Sample::ProductLow, 0xbc),
            (MasterTicks(128), Sample::ProductHigh, 0x34),
        ]
    );
    let mut split = initial;
    let mut events = Vec::new();
    while split.hardware.now < MasterTicks(61) {
        events.extend(split.step(MasterTicks(1), &[]).unwrap().1);
    }
    assert_ne!(split.hardware.charge_debt, 0);
    assert_eq!(split.hardware.math.division_edges_remaining(), 9);
    assert_eq!(split.hardware.math.operand, 0xe07f);
    assert_eq!(split.hardware.math.product, 0x34bc);
    let mut restored = restore(&split);
    restored.validate().unwrap();
    events.extend(finish(&mut restored, 1, &[]));
    assert_eq!(events, expected);
    assert_eq!(restored, direct);
    assert_eq!(restored.hardware.math.division_edges_remaining(), 0);
}

#[test]
fn division_uses_live_operands_and_shared_result_registers() {
    use data::timing::{DIVIDEND_HIGH_PORT, DIVIDEND_LOW_PORT, DIVISOR_PORT};

    let mut timing = state(20, 20);
    for (low, quotient, remainder) in [(0xfe, 339_u16, 107), (0x00, 338_u16, 46)] {
        let mut writes = vec![division_write(DIVIDEND_LOW_PORT, low)];
        if low == 0xfe {
            writes.push(division_write(DIVIDEND_HIGH_PORT, 0xff));
        }
        writes.extend([
            division_write(DIVISOR_PORT, 193),
            internal_run(16),
            division_read(Sample::QuotientLow),
            division_read(Sample::QuotientHigh),
            division_read(Sample::ProductLow),
            division_read(Sample::ProductHigh),
        ]);
        timing.begin(WorkId(1), profile(writes)).unwrap();
        let samples: Vec<_> = finish(&mut timing, 100, &[])
            .into_iter()
            .filter_map(|event| {
                if let EventKind::Sample(_, SampleValue::Byte(byte)) = event.kind {
                    Some(byte)
                } else {
                    None
                }
            })
            .collect();
        let [quotient_low, quotient_high] = quotient.to_le_bytes();
        assert_eq!(samples, [quotient_low, quotient_high, remainder, 0]);
    }
}
