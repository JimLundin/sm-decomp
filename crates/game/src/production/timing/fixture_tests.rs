//! Private hardware projections are borrowed stimuli, never production semantic scheduling.

use super::*;
use data::timing::{DeviceWrite, Exposure};
use std::path::Path;

#[derive(Canonical)]
enum RawEffect {
    None,
    Sample(Sample),
    Write(u16, u8),
}
#[derive(Canonical)]
struct RawRun {
    count: u32,
    cycle: Cycle,
    kind: ExposureKind,
    poll: bool,
    effect: RawEffect,
}
#[derive(Canonical)]
struct RawProfile {
    runs: Vec<RawRun>,
}
impl RawProfile {
    fn admit(&self) -> Profile {
        Profile {
            runs: self
                .runs
                .iter()
                .map(|run| Run {
                    count: run.count,
                    exposure: Exposure {
                        cycle: run.cycle,
                        kind: run.kind,
                        poll_before: run.poll,
                        effect: match run.effect {
                            RawEffect::None => DeviceEffect::None,
                            RawEffect::Sample(sample) => DeviceEffect::Sample(sample),
                            RawEffect::Write(port, value) => DeviceEffect::Publish(
                                DeviceWrite::from_cpu_port(port, value).unwrap(),
                            ),
                        },
                    },
                })
                .collect(),
        }
    }
}
#[derive(Canonical, Clone)]
struct Installation {
    owner: WorkId,
    after: u64,
    regions: Vec<ContentRegion>,
}
#[derive(Canonical)]
struct Inputs {
    hardware: Hardware,
    profiles: [RawProfile; 4],
    regions: Vec<ContentRegion>,
    installations: Vec<Installation>,
}
#[derive(Canonical, Clone)]
struct Graph {
    service: State,
    regions: Vec<ContentRegion>,
    installations: Vec<Installation>,
    next_hud: bool,
    nmi: bool,
}
#[derive(Canonical)]
struct Callback {
    at: MasterTicks,
    class: u8,
    port: u16,
    value: u8,
}
#[derive(Canonical)]
struct PhysicalSample {
    at: MasterTicks,
    device: Sample,
    value: SampleValue,
}

fn load<T: Canonical>(path: &Path) -> T {
    canonical::from_bytes(&std::fs::read(path).unwrap())
        .unwrap_or_else(|_| panic!("private fixture codec rejected"))
}

fn install(graph: &mut Graph) {
    let Some(current) = graph.service.work.last() else {
        return;
    };
    let identity = match current.kind {
        WorkKind::Semantic(identity) | WorkKind::Handler { identity, .. } => identity,
        WorkKind::Wait(..) | WorkKind::Native { .. } => return,
    };
    let completed = current.cursor.completed;
    let mut due = Vec::new();
    graph.installations.retain(|installation| {
        if installation.owner == identity && installation.after == completed {
            due.extend(installation.regions.clone());
            false
        } else {
            true
        }
    });
    for region in due {
        let owned = graph
            .regions
            .iter_mut()
            .find(|owned| {
                owned.start.bank == region.start.bank
                    && owned.start.offset <= region.start.offset
                    && usize::from(region.start.offset.wrapping_sub(owned.start.offset))
                        .wrapping_add(region.bytes.len())
                        <= owned.bytes.len()
            })
            .unwrap();
        let offset = usize::from(region.start.offset.wrapping_sub(owned.start.offset));
        for (destination, source) in owned.bytes.iter_mut().skip(offset).zip(region.bytes) {
            *destination = source;
        }
    }
}

fn execute(inputs: &Inputs, budget: u64, checkpoint: bool) -> (Graph, Vec<Event>) {
    let [pre, hud_begin, hud_end, nmi] = &inputs.profiles;
    let mut graph = Graph {
        service: State {
            hardware: inputs.hardware.clone(),
            work: Vec::new(),
            interrupt_boundary: None,
            pending_handler: None,
            interrupted_wait: None,
        },
        regions: inputs.regions.clone(),
        installations: inputs.installations.clone(),
        next_hud: false,
        nmi: false,
    };
    graph.service.begin(WorkId(1), pre.admit()).unwrap();
    install(&mut graph);
    let mut events = Vec::new();
    for iteration in 0..2_000_000 {
        let (progress, output) = graph
            .service
            .step(MasterTicks(budget), &graph.regions)
            .unwrap();
        events.extend(output);
        install(&mut graph);
        match progress {
            Progress::Completed(WorkId(1)) => graph.service.begin_wait(WorkId(5)).unwrap(),
            Progress::Interrupt(Interrupt::Irq) => {
                let (identity, profile) = if graph.next_hud {
                    (WorkId(3), hud_end)
                } else {
                    (WorkId(2), hud_begin)
                };
                graph.next_hud = !graph.next_hud;
                events.extend(
                    graph
                        .service
                        .enter_interrupt(identity, profile.admit())
                        .unwrap(),
                );
            }
            Progress::Interrupt(Interrupt::Nmi) => {
                graph.nmi = true;
                events.extend(
                    graph
                        .service
                        .enter_interrupt(WorkId(4), nmi.admit())
                        .unwrap(),
                );
            }
            Progress::HandlerReturned(WorkId(4)) => {
                assert!(graph.nmi);
                return (graph, events);
            }
            Progress::Advanced | Progress::HandlerEntered(_) | Progress::HandlerReturned(_) => {}
            Progress::HandlerPortionCompleted(_)
            | Progress::HandlerNeedsWork(_)
            | Progress::Completed(_)
            | Progress::Idle => panic!("unsupported fixture continuation"),
        }
        if checkpoint && iteration % 10007 == 0 {
            graph = canonical::from_bytes(&canonical::to_bytes(&graph)).unwrap();
            graph.service.validate().unwrap();
        }
    }
    panic!("hardware fixture did not reach completion")
}

fn callbacks(events: &[Event]) -> Vec<Callback> {
    events
        .iter()
        .filter_map(|event| {
            let (class, port, value) = match &event.kind {
                EventKind::Publication(write) => (
                    write.port.output_class().unwrap(),
                    write.port.0,
                    write.value,
                ),
                EventKind::Payload(payload) => {
                    let port = payload.target.publication_port();
                    (port.output_class().unwrap(), port.0, payload.value)
                }
                EventKind::Interrupt(Interrupt::Nmi) => (7, 0, 0),
                EventKind::Interrupt(Interrupt::Irq) => (8, 0, 0),
                EventKind::ControllerStart => (9, 0, 0),
                EventKind::ControllerEnd => (10, 0, 0),
                EventKind::LineService(_) => (11, 0, 0),
                EventKind::BulkService(_) => (12, 0, 0),
                EventKind::Sample(_, _)
                | EventKind::ControllerCommitted
                | EventKind::AudioPortRead { .. } => return None,
            };
            Some(Callback {
                at: event.at,
                class,
                port,
                value,
            })
        })
        .collect()
}
fn samples(events: &[Event]) -> Vec<PhysicalSample> {
    events
        .iter()
        .filter_map(|event| {
            if let EventKind::Sample(device, value) = event.kind {
                Some(PhysicalSample {
                    at: event.at,
                    device,
                    value,
                })
            } else {
                None
            }
        })
        .collect()
}
fn compare_callbacks(actual: &[Callback], expected: &[Callback]) {
    assert_eq!(actual.len(), expected.len(), "complete callback count");
    for (ordinal, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert!(
            actual.at == expected.at,
            "callback {ordinal} timestamp delta {}",
            actual.at.0.abs_diff(expected.at.0)
        );
        assert!(actual.class == expected.class, "callback {ordinal} class");
        assert!(actual.port == expected.port, "callback {ordinal} device");
        assert!(actual.value == expected.value, "callback {ordinal} payload");
    }
}

#[test]
#[ignore = "requires private disjoint hardware input and independent expectation files"]
fn private_hardware_projection_matches_independent_callbacks_and_endpoint() {
    let input_path =
        std::env::var_os("SM_TIMING_INPUT").expect("explicit private input path required");
    let expected_path = std::env::var_os("SM_TIMING_EXPECTED")
        .expect("explicit independent expectation directory required");
    let inputs: Inputs = load(Path::new(&input_path));
    let (full, events) = execute(&inputs, 100, false);
    let (resumed, resumed_events) = execute(&inputs, 1, true);
    let expected_path = Path::new(&expected_path);
    let expected_callbacks: Vec<Callback> = load(&expected_path.join("callbacks.bin"));
    compare_callbacks(&callbacks(&events), &expected_callbacks);
    compare_callbacks(&callbacks(&resumed_events), &expected_callbacks);
    let expected_samples: Vec<PhysicalSample> = load(&expected_path.join("samples.bin"));
    let actual_samples = samples(&events);
    assert_eq!(
        actual_samples.len(),
        expected_samples.len(),
        "physical sample count"
    );
    for (ordinal, (actual, expected)) in actual_samples.iter().zip(&expected_samples).enumerate() {
        assert!(actual.at == expected.at, "sample {ordinal} timestamp");
        assert!(actual.device == expected.device, "sample {ordinal} device");
        assert!(actual.value == expected.value, "sample {ordinal} value");
    }
    let expected_hardware: Hardware = load(&expected_path.join("hardware.bin"));
    assert!(
        full.service.hardware.diff(&expected_hardware).is_none(),
        "independently observed hardware endpoint"
    );
    assert!(
        full.service.work.is_empty() && full.service.interrupt_boundary.is_none(),
        "completed native return boundary"
    );
    assert!(
        full.installations.is_empty(),
        "all borrowed fixture materializations committed"
    );
    assert!(
        canonical::to_bytes(&full) == canonical::to_bytes(&resumed),
        "complete restored hardware/input graph"
    );
    assert!(
        canonical::to_bytes(&events) == canonical::to_bytes(&resumed_events),
        "exactly-once restored event suffix"
    );
    let frozen = canonical::to_bytes(&full.service);
    let mut null = full.service;
    let (progress, outputs) = null.step(MasterTicks(1), &full.regions).unwrap();
    assert_eq!(progress, Progress::Idle);
    assert!(
        outputs.is_empty() && canonical::to_bytes(&null) == frozen,
        "frozen null advances no time or outputs"
    );
}
