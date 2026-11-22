//! Check the recurring publication and display owners from one native admission.

use super::*;
use game::production::timing::{self, WaitResume};

const COMPARISON_PIN: &str = "5cb1b36ac0d7aff69363814c749cd801bfe7bfd60c83002dc3838170f2a07f4c";
const ROOM_PIN: &str = "2ec2f467c5052a3ed7d6aaabe0cf817a323076b6e1adbfb955d950ae1bca8a9c";
const STEADY_PIN: &str = "b3bb0bd0f0d51a6fe185fe16a76360f8b527333b3c19d2853fa18fad5903d243";
const REAIM_PIN: &str = "2755161e107cd817e55569b66a809ea5e095c6745ca64f805ef4976cd6ccc471";
const REAIM_COMPARISON_PIN: &str =
    "7a1c9b5ad8440eafc02a3e46abd2a5a09917dfefa2bda8e45cae19a346d6ba98";
const STALE_HAZE_PIN: &str = "62222cf47f1259f90844e54eb6963ba84d9800a70c9a2491ffac379e9976016c";
const ORDINARY_RIGHT_PIN: &str = "04b57cc511c075733f595cd7db4781f1dfa0b93ef8d5bbba8ab4d7a2acd34c60";
const ORDINARY_RIGHT_COMPARISON_PIN: &str =
    "239fa553de77d71aff08217a355f0cff8f2eca4b14d4c576d43c1fbe81e3a663";

#[derive(serde::Deserialize)]
struct Comparison {
    preceding_return_master_ticks: [u64; 2],
    callbacks: Vec<toml::Value>,
    boundary: [Boundary; 5],
}

#[derive(serde::Deserialize)]
struct Boundary {
    owner: String,
    master_ticks: u64,
    countdowns: Option<[u16; 2]>,
    band_backing: Option<[u8; 16]>,
    prepared_layer_bytes: Option<[u8; 9]>,
    fixed_colour_rgb: Option<[u8; 3]>,
}

#[derive(serde::Deserialize)]
struct RoomGuard {
    owner: String,
    initial_departure: String,
}

struct FirstReturn {
    haze: frame::haze::Haze,
    wait: timing::InterruptedWait,
}

#[derive(canonical::Canonical)]
struct InterruptedWaitImage {
    wait: timing::Work,
    at: MasterTicks,
    interrupt: data::timing::Interrupt,
}

#[derive(serde::Deserialize)]
struct ReaimComparison {
    boundary: [Boundary; 12],
    random_callback: [MathCallback; 7],
    pose: ReaimPose,
}

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
struct MathCallback {
    kind: MathAccess,
    port: MathPort,
    value: u8,
    master_ticks: u64,
}

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
enum MathAccess {
    Read,
    Write,
}

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
enum MathPort {
    Multiplicand,
    Multiplier,
    ProductLow,
    ProductHigh,
}

#[derive(serde::Deserialize)]
struct ReaimPose {
    current_pose: u16,
    deferred_pose: u16,
    bomb_jump_cleared: bool,
}

#[derive(serde::Deserialize)]
struct OrdinaryRightComparison {
    actor_update: ActorUpdate,
    ordinary_right: OrdinaryRight,
    arithmetic_callback: [MathCallback; 4],
}

#[derive(serde::Deserialize)]
struct ActorUpdate {
    begin_master_ticks: u64,
    end_master_ticks: u64,
    door_wait_before: [u16; 2],
    door_wait_after: [u16; 2],
}

#[derive(serde::Deserialize)]
struct OrdinaryRight {
    begin_master_ticks: u64,
    end_master_ticks: u64,
    terrain_queries: usize,
    fractional_carry: bool,
    start_x_16_16: u32,
    end_x_16_16: u32,
    start_y_16_16: u32,
    end_y_16_16: u32,
    requested_x_16_16: u32,
    accepted_x_16_16: u32,
}

fn math_callbacks(events: &[timing::Event]) -> Vec<MathCallback> {
    use data::timing::{MULTIPLICAND_PORT, MULTIPLIER_PORT};
    events
        .iter()
        .filter_map(|event| {
            let (kind, port, value) = match &event.kind {
                EventKind::Publication(write) if write.port == MULTIPLICAND_PORT => {
                    (MathAccess::Write, MathPort::Multiplicand, write.value)
                }
                EventKind::Publication(write) if write.port == MULTIPLIER_PORT => {
                    (MathAccess::Write, MathPort::Multiplier, write.value)
                }
                EventKind::Sample(Sample::ProductLow, SampleValue::Byte(value)) => {
                    (MathAccess::Read, MathPort::ProductLow, *value)
                }
                EventKind::Sample(Sample::ProductHigh, SampleValue::Byte(value)) => {
                    (MathAccess::Read, MathPort::ProductHigh, *value)
                }
                EventKind::Publication(_)
                | EventKind::Sample(..)
                | EventKind::AudioPortRead { .. }
                | EventKind::Payload(_)
                | EventKind::BulkService(_)
                | EventKind::LineService(_)
                | EventKind::Interrupt(_)
                | EventKind::ControllerStart
                | EventKind::ControllerEnd
                | EventKind::ControllerCommitted => return None,
            };
            Some(MathCallback {
                kind,
                port,
                value,
                master_ticks: event.at.0,
            })
        })
        .collect()
}

fn state(production: &Production) -> &State {
    let OwnedState::Frame(state) = production.state() else {
        panic!("frame production");
    };
    state
}

fn clock_at(production: &Production) -> u64 {
    state(production).timing.hardware.now.0
}

fn observe_prelude(
    production: &mut Production,
    deadline: u64,
    budget: MasterTicks,
    mut observe: impl FnMut(usize, &State),
) -> ([u64; 5], Vec<timing::Event>) {
    let mut events = Vec::new();
    let mut tick = |ordinal, reached: fn(&Caller) -> bool| {
        advance_frame_with_budget(
            production,
            deadline,
            budget,
            |state| reached(&state.caller),
            |output| events.extend(output.events),
        );
        observe(ordinal, state(production));
        clock_at(production)
    };
    let clocks = [
        tick(0, |caller| matches!(caller, Caller::Music { .. })),
        tick(1, |caller| matches!(caller, Caller::Haze { .. })),
        tick(2, |caller| matches!(caller, Caller::HazeTail { .. })),
        tick(3, |caller| matches!(caller, Caller::LayerBlend { .. })),
        tick(4, |caller| matches!(caller, Caller::Random { .. })),
    ];
    (clocks, events)
}

fn compare_display(boundary: &Boundary, state: &State, address: data::dma::BusAddress) {
    if let (Some(expected), Some(countdowns)) = (&boundary.band_backing, boundary.countdowns) {
        let haze = state.haze.as_ref().unwrap();
        assert_eq!(haze.current().fade, types::haze::HazeFade::Steady);
        assert_observation_eq!(haze.countdowns(), countdowns);
        let regions = state.content.regions();
        let backing = regions
            .iter()
            .find(|region| region.start == address)
            .unwrap();
        assert_observation_eq!(backing.bytes.as_slice(), expected.as_slice());
    }
    if let Some(expected) = boundary.prepared_layer_bytes {
        let Caller::Random { layers, haze, .. } = &state.caller else {
            panic!("completed LayerBlend");
        };
        assert_observation_eq!(
            frame::layer_blend::encode(layers.terminal).unwrap(),
            expected
        );
        assert_observation_eq!(
            haze.fixed_colour_bytes().map(|component| component & 31),
            boundary.fixed_colour_rgb.unwrap()
        );
        assert_eq!(
            haze.blend(),
            frame::borrowed_prelude_receipts::LayerBlendDisposition::NoBackgroundThree
        );
    }
}

fn sources() -> Sources {
    Sources {
        native: std::env::var_os("SM_AUDIO_NATIVE").unwrap().into(),
        rom: std::env::var_os("SM_ROM").unwrap().into(),
        ..Sources::default()
    }
}

fn opening_case(index: &Index) -> &Case {
    index
        .cases
        .iter()
        .find(|case| case.id.0 == "frame/adjacent-actual-return")
        .unwrap()
}

fn reaim_input(sources: &Sources) -> data::timing::Template {
    use crate::replay::{frame_origin::admission::quiet_profile, frame_steady};
    let input = frame_steady::member(
        &sources.native.join("opening-clock-reaim-input-v1"),
        "recurring-aim-down.toml",
        REAIM_PIN,
    )
    .unwrap();
    quiet_profile::decode(&input, "RecurringAimDown", [615, 3984]).unwrap()
}

fn reaim_comparison(sources: &Sources) -> ReaimComparison {
    crate::replay::frame_steady::decode(
        &sources.native.join("opening-clock-reaim-comparison-v2"),
        "cut-2-reaim-comparison.private.toml",
        REAIM_COMPARISON_PIN,
    )
    .unwrap()
}

fn open_frame(
    case: &Case,
    sources: &Sources,
    qualify: impl FnOnce(&mut frame::Content),
) -> (AdmittedCase, FirstReturn) {
    use crate::replay::{frame_origin::admission::quiet_profile, frame_steady};

    let room: RoomGuard = frame_steady::decode(
        &sources.native.join("opening-clock-cut2-room-v1"),
        "cut-2-room-guard.private.toml",
        ROOM_PIN,
    )
    .unwrap();
    let mut admitted = super::super::admit(case, sources).unwrap();
    let ProductionContent::Frame(content) = &mut admitted.content else {
        panic!("frame content");
    };
    let [head, finish, _] = quiet_profile::steady(
        &frame_steady::member(
            &sources.native.join("shot-steady-haze-v1"),
            "steady-prelude-templates.toml",
            STEADY_PIN,
        )
        .unwrap(),
    )
    .unwrap();
    content.steady_haze = Some(frame::haze::SteadyProfiles { head, finish });
    qualify(content);
    assert_eq!(room.owner, "Room");
    assert_eq!(room.initial_departure, "Inactive");
    let mut initial = state(&admitted.production).clone();
    initial
        .room_tail_origin
        .as_mut()
        .unwrap()
        .departure_inactive = Some(frame::room_tail::InactiveDeparture);
    admitted.production = Production::admit(
        ProductionSeed::Frame(Box::new(initial)),
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    let controllers = read_control(&sources.native);
    observe_first_return(case, &mut admitted, &controllers.actual, false);
    let early = state(&admitted.production);
    let first = FirstReturn {
        haze: early.haze.clone().unwrap(),
        wait: early.timing.interrupted_wait.clone().unwrap(),
    };
    assert_eq!(
        state(&admitted.production).timing.wait_resume(),
        Some(WaitResume::Reload)
    );
    let outputs = admitted
        .inputs
        .advance(&mut admitted.production, 2)
        .unwrap();
    let actual = admitted
        .actual
        .accept(&admitted.inputs, &admitted.production, outputs, 2)
        .unwrap();
    compared(2, &admitted.expected.read(case, 2).unwrap().fields, &actual).unwrap();
    assert_eq!(admitted.actual.publication_count(), 4_590);
    assert_eq!(
        state(&admitted.production).timing.wait_resume(),
        Some(WaitResume::PendingBusyBranch)
    );

    (admitted, first)
}

#[test]
#[ignore = "requires the pinned retained, Steady Haze and opening clock fixtures and player ROM"]
fn third_opportunity_resumes_actual_wait_and_current_steady_haze() {
    use crate::replay::frame_steady;

    let sources = sources();
    let index = Index::parse(include_str!("../../../../../scenarios/replay/index.toml")).unwrap();
    let comparison: Comparison = frame_steady::decode(
        &sources.native.join("opening-clock-cut2-comparison-v1"),
        "cut-2-prelude-comparison.private.toml",
        COMPARISON_PIN,
    )
    .unwrap();
    let case = opening_case(&index);
    let (mut admitted, first) = open_frame(case, &sources, |_| {});
    assert_eq!(
        clock_at(&admitted.production),
        comparison.preceding_return_master_ticks[1]
    );
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content");
    };
    let bands = content.haze_bands_address;

    let checkpoint = admitted.production.save().unwrap();
    let at_return = state(&admitted.production).clone();
    reject_preceding_wait_cursor(&admitted, &first);
    let mut restored = Production::restore(
        &checkpoint,
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    assert_eq!(
        comparison
            .boundary
            .each_ref()
            .map(|boundary| boundary.owner.as_str()),
        [
            "Acknowledgment",
            "Music",
            "HazeRetain",
            "HazeFinish",
            "LayerBlend"
        ]
    );
    let expected = comparison
        .boundary
        .each_ref()
        .map(|boundary| boundary.master_ticks);
    let deadline = expected[4].checked_add(100).unwrap();
    advance_frame_until(&mut restored, deadline, |state| {
        matches!(state.caller, Caller::Acknowledgment { .. })
            && state
                .timing
                .work
                .first()
                .is_some_and(|work| work.cursor.intrinsic.0 != 0)
    });
    let pending = restored.save().unwrap();
    restored =
        Production::restore(&pending, admitted.content.clone(), admitted.rules.clone()).unwrap();
    let mut damaged = state(&restored).clone();
    let exposure = &mut damaged
        .timing
        .work
        .first_mut()
        .unwrap()
        .profile
        .runs
        .first_mut()
        .unwrap()
        .exposure;
    exposure.poll_before = !exposure.poll_before;
    reject_native_checkpoint(&admitted, Box::new(damaged));
    let compare = |ordinal: usize, state: &State| {
        compare_display(comparison.boundary.get(ordinal).unwrap(), state, bands);
    };
    let (actual, events) =
        observe_prelude(&mut admitted.production, deadline, MasterTicks(13), compare);
    assert_eq!(actual, expected);
    assert_eq!(
        crate::replay::frame::project(&events).unwrap().len(),
        comparison.callbacks.len()
    );
    assert_observation_eq!(
        observe_prelude(&mut restored, deadline, MasterTicks(3), compare),
        (actual, events)
    );
    assert_observation_eq!(
        restored.save().unwrap(),
        admitted.production.save().unwrap()
    );
    assert_eq!(state(&restored).timing.wait_resume(), None);
    check_acknowledgment_clock_controls(&admitted, &checkpoint, &at_return, expected[0], deadline);
}

fn reject_preceding_wait_cursor(admitted: &AdmittedCase, first: &FirstReturn) {
    let mut spliced = state(&admitted.production).clone();
    let earlier: InterruptedWaitImage =
        canonical::from_bytes(&canonical::to_bytes(&first.wait)).unwrap();
    let mut current: InterruptedWaitImage = canonical::from_bytes(&canonical::to_bytes(
        spliced.timing.interrupted_wait.as_ref().unwrap(),
    ))
    .unwrap();
    current.wait.cursor = earlier.wait.cursor;
    spliced.timing.interrupted_wait =
        Some(canonical::from_bytes(&canonical::to_bytes(&current)).unwrap());
    assert!(matches!(Production::admit(
        ProductionSeed::Frame(Box::new(spliced)), admitted.content.clone(), admitted.rules.clone()
    ), Err(game::production::AdmissionFault::Frame(fault))
        if *fault == frame::Fault::Timing(timing::Fault::InvalidState)
    ));
}

fn check_acknowledgment_clock_controls(
    admitted: &AdmittedCase,
    checkpoint: &game::production::SavedProduction,
    at_return: &State,
    expected: u64,
    deadline: u64,
) {
    let mut shifted =
        Production::restore(checkpoint, admitted.content.clone(), admitted.rules.clone()).unwrap();
    let mut shifted_state = state(&shifted).clone();
    let beam = &mut shifted_state.timing.hardware.beam;
    // Put the beam just before refresh so the resumed branch crosses it immediately.
    for (position, horizontal) in beam.history.iter_mut().zip((526_u16..).step_by(2)) {
        position.line = beam.position.line;
        position.horizontal = horizontal;
    }
    beam.position = *beam.history.last().unwrap();
    beam.residue = 0;
    beam.refreshed = false;
    shifted = Production::admit(
        ProductionSeed::Frame(Box::new(shifted_state)),
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    advance_frame_until(&mut shifted, deadline, |state| {
        matches!(state.caller, Caller::Music { .. })
    });
    assert_eq!(Some(clock_at(&shifted)), expected.checked_add(40));

    let mut altered_bill = admitted.content.clone();
    let ProductionContent::Frame(content) = &mut altered_bill else {
        panic!("frame content");
    };
    let first = content.acknowledgment.runs.first_mut().unwrap();
    assert_eq!(first.cycle, data::timing::Cycle::Six);
    first.count = first.count.checked_sub(3).unwrap();
    if first.count == 0 {
        content.acknowledgment.runs.remove(0);
    }
    assert!(matches!(
        Production::restore(checkpoint, altered_bill.clone(), admitted.rules.clone()),
        Err(game::production::AdmissionFault::ContentChanged)
    ));
    // Isolate the lost charge while leaving the authentic interrupted cursor intact.
    let mut stale = Production::admit(
        ProductionSeed::Frame(Box::new(at_return.clone())),
        altered_bill,
        admitted.rules.clone(),
    )
    .unwrap();
    advance_frame_until(&mut stale, deadline, |state| {
        matches!(state.caller, Caller::Music { .. })
    });
    assert_eq!(clock_at(&stale).checked_add(18), Some(expected));
}

fn reached_reaim(
    production: &mut Production,
    budget: MasterTicks,
    observed: &ReaimComparison,
) -> Vec<timing::Event> {
    let [
        _,
        _,
        _,
        _,
        prelude,
        random,
        builder,
        gameplay,
        actors,
        debug,
        palette,
        pose,
    ] = &observed.boundary;
    let deadline = pose.master_ticks.checked_add(100).unwrap();
    let (_, mut events) = observe_prelude(production, deadline, budget, |_, _| {});
    assert_eq!(clock_at(production), prelude.master_ticks);
    for (stage, boundary) in [
        (PreparationStage::Builder, random),
        (PreparationStage::Gameplay, builder),
        (PreparationStage::Actors, gameplay),
        (PreparationStage::Debug, actors),
        (PreparationStage::Palette, debug),
    ] {
        advance_frame_with_budget(
            production,
            deadline,
            budget,
            |state| matches!(state.caller, Caller::Preparation { stage: current, charging: false, .. } if current == stage),
            |output| events.extend(output.events),
        );
        assert_eq!(
            clock_at(production),
            boundary.master_ticks,
            "{}",
            boundary.owner
        );
    }
    for (stage, boundary) in [
        (PlayerEntryStage::PoseInput, palette),
        (PlayerEntryStage::SuitPalette, pose),
    ] {
        advance_frame_with_budget(
            production,
            deadline,
            budget,
            |state| matches!(state.caller, Caller::Player { stage: current, charging: false, .. } if current == stage),
            |output| events.extend(output.events),
        );
        assert_eq!(
            clock_at(production),
            boundary.master_ticks,
            "{}",
            boundary.owner
        );
    }
    let Caller::Player { completed, .. } = &state(production).caller else {
        panic!("returned pose input");
    };
    let decision = completed.0.pose_input.as_ref().unwrap();
    let pose = |id| data::assets::samus::map::key_of(data::pose::PoseId(id)).unwrap();
    assert_observation_eq!(decision.dependency.pose, pose(observed.pose.current_pose));
    assert_observation_eq!(
        decision.terminal.prospective,
        pose(observed.pose.deferred_pose)
    );
    assert_observation_eq!(
        state(production).movement.state().pose,
        decision.dependency.pose
    );
    assert_eq!(
        decision.terminal.bomb_jump == frame::borrowed_player_preparation::BombJumpDirection::None,
        observed.pose.bomb_jump_cleared
    );
    compare_math(&events, &observed.random_callback);
    assert_eq!(events.len(), observed.random_callback.len());
    events
}

fn compare_math(events: &[timing::Event], expected: &[MathCallback]) {
    let actual = math_callbacks(events);
    assert_eq!(actual.len(), expected.len());
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(actual.kind, expected.kind, "math event {index} kind");
        assert_eq!(actual.port, expected.port, "math event {index} port");
        assert_eq!(
            actual.master_ticks, expected.master_ticks,
            "math event {index} clock"
        );
        assert!(
            actual.value == expected.value,
            "math event {index} value differs"
        );
    }
}

#[test]
#[ignore = "requires independently pinned recurring aim inputs and native comparison"]
fn opening_clock_carries_random_and_live_reaim_from_the_original_admission() {
    let sources = sources();
    let index = Index::parse(include_str!("../../../../../scenarios/replay/index.toml")).unwrap();
    let observed = reaim_comparison(&sources);
    let (mut admitted, _) = open_frame(opening_case(&index), &sources, |content| {
        content.player_entry.recurring_aim_down = Some(reaim_input(&sources));
    });
    let checkpoint = admitted.production.save().unwrap();
    let mut restored = Production::restore(&checkpoint, admitted.content, admitted.rules).unwrap();
    let direct = reached_reaim(&mut admitted.production, MasterTicks(13), &observed);
    let continued = reached_reaim(&mut restored, MasterTicks(3), &observed);
    assert_observation_eq!(direct, continued);
    assert_observation_eq!(
        restored.save().unwrap(),
        admitted.production.save().unwrap()
    );
}

#[test]
#[ignore = "requires the native stale-Haze control and opening Clock fixtures"]
fn opening_clock_detects_stale_haze_at_the_first_haze_boundary() {
    type BoundaryPredicate = fn(&Caller) -> bool;
    use crate::replay::{frame_origin::admission::quiet_profile, frame_steady};

    let sources = sources();
    let index = Index::parse(include_str!("../../../../../scenarios/replay/index.toml")).unwrap();
    let case = opening_case(&index);
    let observation: Comparison = frame_steady::decode(
        &sources.native.join("opening-clock-cut2-comparison-v1"),
        "cut-2-prelude-comparison.private.toml",
        COMPARISON_PIN,
    )
    .unwrap();
    let stale_profile = frame_steady::member(
        &sources.native.join("opening-clock-stale-haze-control-v1"),
        "stale-settling-haze-control.toml",
        STALE_HAZE_PIN,
    )
    .unwrap();
    let (mut admitted, first) = open_frame(case, &sources, |_| {});
    let ProductionContent::Frame(content) = &mut admitted.content else {
        panic!("frame content");
    };
    content.settling_haze = Some(
        quiet_profile::decode(&stale_profile, "RecurringStaleSettlingHaze", [160, 1052]).unwrap(),
    );
    let mut stale_state = state(&admitted.production).clone();
    stale_state.haze = Some(first.haze);
    let mut stale = Production::admit(
        ProductionSeed::Frame(Box::new(stale_state)),
        admitted.content,
        admitted.rules,
    )
    .unwrap();
    let [ack, music, haze, _, _] = observation.boundary;
    let deadline = haze.master_ticks.checked_add(100).unwrap();
    let mut events = Vec::new();
    let controls: [(BoundaryPredicate, u64); 3] = [
        (
            |caller| matches!(caller, Caller::Music { .. }),
            ack.master_ticks,
        ),
        (
            |caller| matches!(caller, Caller::Haze { .. }),
            music.master_ticks,
        ),
        (
            |caller| matches!(caller, Caller::HazeTail { .. }),
            haze.master_ticks.checked_add(64).unwrap(),
        ),
    ];
    for (matches, expected) in controls {
        advance_frame(
            &mut stale,
            deadline,
            |state| matches(&state.caller),
            |output| events.extend(output.events),
        );
        assert_eq!(clock_at(&stale), expected);
    }
    assert_eq!(events.len(), 0);
}

fn compare_actor_update(observed: &ActorUpdate, production: &mut Production, budget: MasterTicks) {
    use player_continuation::Stage::{Actors, Sprites};
    for (stage, expected) in [
        (Actors, observed.begin_master_ticks),
        (Sprites, observed.end_master_ticks),
    ] {
        advance_frame_with_budget(
            production,
            expected,
            budget,
            |state| matches!(state.caller, Caller::PlayerContinuation { stage: current, charging: false, .. } if current == stage),
            |_| {},
        );
        assert_eq!(clock_at(production), expected, "{stage:?}");
    }
    let Caller::PlayerContinuation { remaining, .. } = &state(production).caller else {
        panic!("returned ActorUpdate");
    };
    let actors = remaining.0.actors.as_ref().unwrap();
    assert_observation_eq!(
        actors
            .dependency
            .actors
            .each_ref()
            .map(|actor| actor.instruction_timer),
        observed.door_wait_before
    );
    assert_observation_eq!(
        actors
            .actors
            .each_ref()
            .map(|actor| actor.instruction_timer),
        observed.door_wait_after
    );
}

fn compare_ordinary_right(
    observed: &OrdinaryRightComparison,
    production: &mut Production,
    budget: MasterTicks,
) {
    use samus_movement::suspended::MovementDecision;
    use types::Parts;
    let motion = &observed.ordinary_right;
    advance_frame_with_budget(
        production,
        motion.begin_master_ticks,
        budget,
        |state| {
            matches!(
                state.caller,
                Caller::PlayerContinuation {
                    run: frame::movement_caller::Run::Complete(_),
                    ordinary: None,
                    charging: false,
                    ..
                }
            )
        },
        |_| {},
    );
    assert_eq!(clock_at(production), motion.begin_master_ticks);
    let mut events = Vec::new();
    advance_frame_with_budget(
        production,
        motion.end_master_ticks,
        budget,
        |state| matches!(&state.caller, Caller::PlayerContinuation { ordinary: Some(ordinary), .. } if ordinary.is_complete()),
        |output| events.extend(output.events),
    );
    assert_eq!(clock_at(production), motion.end_master_ticks);
    let current = state(production);
    let Caller::PlayerContinuation {
        ordinary: Some(ordinary),
        ..
    } = &current.caller
    else {
        panic!("returned OrdinaryRight");
    };
    let travelled = ordinary.completion().unwrap();
    let [
        MovementDecision::MotionRequest(request),
        MovementDecision::MotionAnswer {
            answer, outputs, ..
        },
    ] = travelled.decisions.0.as_slice()
    else {
        panic!("actual ordinary request and answer");
    };
    let (_, carry) = request
        .bounds
        .at
        .x
        .sub()
        .overflowing_add(request.distance.sub());
    assert_eq!(carry, motion.fractional_carry);
    assert_observation_eq!(
        [
            request.bounds.at.x.0,
            request.bounds.at.y.0,
            request.distance.0
        ],
        [
            motion.start_x_16_16,
            motion.start_y_16_16,
            motion.requested_x_16_16
        ]
    );
    assert_observation_eq!(
        [answer.to.x.0, answer.to.y.0, answer.accepted.0.0],
        [
            motion.end_x_16_16,
            motion.end_y_16_16,
            motion.accepted_x_16_16
        ]
    );
    assert_observation_eq!(current.movement.state().position, answer.to);
    assert_eq!(outputs.collision_tests.len(), motion.terrain_queries);
    compare_math(&events, &observed.arithmetic_callback);
}

#[test]
#[ignore = "requires independent OrdinaryRight input and comparison fixtures"]
fn opening_clock_carries_live_door_decrement_and_fractional_travel() {
    use crate::replay::{frame_origin::admission::quiet_profile, frame_steady};
    let sources = sources();
    let index = Index::parse(include_str!("../../../../../scenarios/replay/index.toml")).unwrap();
    let reaim = reaim_comparison(&sources);
    let observed: OrdinaryRightComparison = frame_steady::decode(
        &sources
            .native
            .join("opening-clock-ordinary-right-comparison-v1"),
        "cut-2-ordinary-right-comparison.private.toml",
        ORDINARY_RIGHT_COMPARISON_PIN,
    )
    .unwrap();
    let input = frame_steady::member(
        &sources.native.join("opening-clock-ordinary-right-input-v1"),
        "recurring-ordinary-right.toml",
        ORDINARY_RIGHT_PIN,
    )
    .unwrap();
    let (mut admitted, _) = open_frame(opening_case(&index), &sources, |content| {
        content.player_entry.recurring_aim_down = Some(reaim_input(&sources));
        let players = content.player_continuation.as_mut().unwrap();
        players.recurring_waiting_actors = Some(players.actors.clone());
        content
            .ordinary_right
            .as_mut()
            .unwrap()
            .set_fractional_carry(quiet_profile::recurring_ordinary_right(&input).unwrap());
        content
            .inside_physical
            .as_mut()
            .unwrap()
            .admit_recurring_air_columns(content.ordinary_right.as_ref().unwrap())
            .unwrap();
    });
    let checkpoint = admitted.production.save().unwrap();
    let mut restored = Production::restore(&checkpoint, admitted.content, admitted.rules).unwrap();
    for (production, budget) in [
        (&mut admitted.production, MasterTicks(13)),
        (&mut restored, MasterTicks(3)),
    ] {
        reached_reaim(production, budget, &reaim);
        compare_actor_update(&observed.actor_update, production, budget);
        compare_ordinary_right(&observed, production, budget);
    }
    assert_observation_eq!(
        restored.save().unwrap(),
        admitted.production.save().unwrap()
    );
}

#[test]
#[ignore = "requires the authenticated Contact cohort, third-return assets and player ROM"]
fn opening_clock_carries_contact_clear_delay_and_reply_through_later_returns() {
    let sources = sources();
    let index = Index::parse(include_str!("../../../../../scenarios/replay/index.toml")).unwrap();
    let case = index
        .cases
        .iter()
        .find(|case| case.id.0 == "frame/opening-eight-native-returns")
        .unwrap();
    let mut admitted = super::super::admit(case, &sources).unwrap();
    assert_eq!(admitted.schedule.len(), 8);
    for cut in admitted.schedule.clone() {
        if cut == 6 {
            restore_contact_clear_read(case, &mut admitted, cut);
        } else {
            runner::execute(
                case,
                &admitted.inputs,
                &admitted.expected,
                &mut admitted.production,
                &mut admitted.actual,
                cut,
            )
            .unwrap_or_else(|outcome| panic!("return request {cut} failed: {outcome:?}"));
        }
        if cut == 1 {
            // The later work is installed, and the original queued Ready cue still writes.
            assert_eq!(admitted.actual.publication_count(), 1);
        }
        let checkpoint = admitted.production.save().unwrap();
        let restored = Production::restore(
            &checkpoint,
            admitted.content.clone(),
            admitted.rules.clone(),
        )
        .unwrap();
        assert_observation_eq!(restored, admitted.production);
        admitted.production = restored;
    }
    assert_eq!(admitted.actual.publication_count(), 4);
}

fn restore_contact_clear_read(case: &Case, admitted: &mut AdmittedCase, cut: u64) {
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("opening frame inputs");
    };
    inputs
        .present_opening_preview(&mut admitted.production, cut)
        .unwrap();

    // The independent CPU read precedes the outgoing retry and later incoming zero.
    let read = 1_957_726_u64;
    let mut emitted = Vec::new();
    advance_opening_to(&mut admitted.production, read, &mut emitted);
    let current = state(&admitted.production);
    assert_eq!(current.timing.hardware.now, MasterTicks(read));
    assert_eq!(
        current.sender_progress,
        frame::sender_caller::Cursor::ContactRead
    );
    assert_eq!(
        current.senders.contact.0.progress,
        audio_transport::SenderProgress::AwaitClear
    );
    let [incoming, outgoing] = admitted.production.frame_contact_latches().unwrap();
    assert!(incoming != 0 && outgoing == 0);
    reject_contact_history_splices(admitted, current);

    restore_opening_contact(admitted);
    let retry = 1_957_772_u64;
    let arrival = 1_967_634_u64;
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("opening frame content");
    };
    let port = content.sender_cue.unwrap().port;
    reach_clear_retry_publication(&mut admitted.production, retry, port, &mut emitted);
    restore_opening_contact(admitted);
    advance_opening_to(
        &mut admitted.production,
        arrival.checked_sub(1).unwrap(),
        &mut emitted,
    );
    assert_observation_eq!(
        admitted.production.frame_contact_latches().unwrap(),
        [incoming, outgoing]
    );
    advance_opening_to(&mut admitted.production, arrival, &mut emitted);
    assert_observation_eq!(admitted.production.frame_contact_latches().unwrap(), [0, 0]);

    loop {
        let outputs = admitted
            .production
            .advance(AdvanceInput::Frame {
                budget: MasterTicks(4096),
            })
            .unwrap();
        let ProductionOutputs::Frame(outputs) = outputs else {
            panic!("opening frame outputs");
        };
        emitted.extend(outputs.events);
        if outputs.progress == frame::Progress::Completed {
            break;
        }
    }
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("opening frame inputs");
    };
    inputs
        .compare_opening_preview(&admitted.production, cut)
        .unwrap();
    let endpoint = state(&admitted.production);
    assert_observation_eq!(endpoint.publications.active().unwrap(), emitted.as_slice());
    let outputs = ProductionOutputs::Frame(frame::Outputs {
        progress: frame::Progress::Completed,
        events: emitted,
    });
    let actual = admitted
        .actual
        .accept(&admitted.inputs, &admitted.production, outputs, cut)
        .unwrap();
    let expected = admitted.expected.read(case, cut).unwrap();
    assert_eq!(compared(cut, &expected.fields, &actual), Ok(()));
    reject_shifted_zero_retry(admitted);
}

fn advance_opening_to(production: &mut Production, stop: u64, emitted: &mut Vec<timing::Event>) {
    while clock_at(production) < stop {
        let prior = production.clone();
        let before = clock_at(&prior);
        let budget = MasterTicks(stop.checked_sub(before).unwrap());
        let ProductionOutputs::Frame(outputs) =
            production.advance(AdvanceInput::Frame { budget }).unwrap()
        else {
            panic!("opening frame outputs");
        };
        assert_eq!(outputs.progress, frame::Progress::Advanced);
        emitted.extend(outputs.events);
        let after = clock_at(production);
        assert!(after >= before && (after > before || *production != prior));
    }
    assert_eq!(clock_at(production), stop);
}

fn reach_clear_retry_publication(
    production: &mut Production,
    expected: u64,
    port: data::timing::PublicationPort,
    emitted: &mut Vec<timing::Event>,
) {
    use frame::sender_caller::{Cursor, Service};

    let reached = |events: &[timing::Event]| {
        events.iter().rev().find_map(|event| {
            if let EventKind::Publication(write) = &event.kind
                && write.port == port
                && write.value == 0
            {
                Some(event.at)
            } else {
                None
            }
        })
    };
    advance_opening_to(production, expected, emitted);
    if reached(emitted).is_none() {
        let current = state(production);
        assert!(current.contact_clear_retried_at.is_none());
        assert!(matches!(
            current.sender_progress,
            Cursor::Body {
                service: Service::Contact,
                publication: Some(_),
                ..
            }
        ));
        let ProductionOutputs::Frame(outputs) = production
            .advance(AdvanceInput::Frame {
                budget: MasterTicks(1),
            })
            .unwrap()
        else {
            panic!("opening frame outputs");
        };
        assert_eq!(outputs.progress, frame::Progress::Advanced);
        emitted.extend(outputs.events);
    }
    let at = reached(emitted);
    assert_eq!(
        at,
        Some(MasterTicks(expected)),
        "return request 6 missing outgoing clear retry"
    );
    let current = state(production);
    assert!(matches!(
        current.sender_progress,
        Cursor::Body {
            service: Service::Contact,
            publication: None,
            ..
        }
    ));
    assert_eq!(
        current.contact_clear_retried_at, at,
        "return request 6 retained clear retry"
    );
}

fn restore_opening_contact(admitted: &mut AdmittedCase) {
    let checkpoint = admitted.production.save().unwrap();
    let restored = Production::restore(
        &checkpoint,
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    assert_observation_eq!(restored, admitted.production);
    admitted.production = restored;
}

fn reject_shifted_zero_retry(admitted: &AdmittedCase) {
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("opening frame content");
    };
    let port = content.sender_cue.unwrap().port;
    for earlier in [true, false] {
        let mut altered = state(&admitted.production).clone();
        let retry = altered.publications.events.iter_mut().rev().find(|event| {
            matches!(&event.kind, EventKind::Publication(write) if write.port == port && write.value == 0)
        }).unwrap();
        let actual = retry.at.0;
        retry.at.0 = if earlier {
            actual.checked_sub(1)
        } else {
            actual.checked_add(1)
        }
        .unwrap();
        assert!(altered.publications.validate().is_ok());
        reject_native_checkpoint(admitted, Box::new(altered));
    }
}

#[test]
#[ignore = "requires the three-return indexed native fixtures and player ROM"]
fn opening_clock_three_indexed_returns_with_live_restore_and_contact_fault() {
    let sources = sources();
    let index = Index::parse(include_str!("../../../../../scenarios/replay/index.toml")).unwrap();
    let case = index
        .cases
        .iter()
        .find(|case| case.id.0 == "frame/opening-three-native-returns")
        .unwrap();
    check_indexed_return_receipt(case, &sources);
    let mut admitted = super::super::admit(case, &sources).unwrap();
    for cut in [1, 2] {
        runner::execute(
            case,
            &admitted.inputs,
            &admitted.expected,
            &mut admitted.production,
            &mut admitted.actual,
            cut,
        )
        .unwrap();
    }
    reject_shifted_contact_clock(case, &admitted);
    let frame::countdown_caller::Stage::Complete {
        offer: prior_countdown,
        ..
    } = state(&admitted.production).countdown
    else {
        panic!("preceding completed Countdown");
    };
    let InputReader::Frame(input) = &admitted.inputs else {
        panic!("frame input schedule");
    };
    input
        .present_opening_preview(&mut admitted.production, 3)
        .unwrap();
    let mut events = Vec::new();
    let mut stages = [false; 3];
    loop {
        let ProductionOutputs::Frame(output) = admitted
            .production
            .advance(AdvanceInput::Frame {
                budget: MasterTicks(128),
            })
            .unwrap()
        else {
            panic!("frame outputs");
        };
        events.extend(output.events);
        if output.progress == frame::Progress::Completed {
            break;
        }
        let value = state(&admitted.production);
        let contact = value.sender_progress == frame::sender_caller::Cursor::ContactRead;
        let reached = [
            matches!(value.caller, Caller::HazeTail { .. }),
            contact,
            matches!(
                value.return_progress,
                frame::frame_return::Stage::Ready {
                    suffix: frame::suffix_caller::Suffix::Video { .. },
                    ..
                }
            ),
        ];
        for (at, (saved, reached)) in stages.iter_mut().zip(reached).enumerate() {
            if *saved || !reached {
                continue;
            }
            check_contact_checkpoint_controls(&admitted, value, at, prior_countdown);
            let checkpoint = admitted.production.save().unwrap();
            let resumed = Production::restore(
                &checkpoint,
                admitted.content.clone(),
                admitted.rules.clone(),
            )
            .unwrap();
            assert_observation_eq!(resumed, admitted.production);
            admitted.production = resumed;
            *saved = true;
            break;
        }
    }
    assert_eq!(stages, [true; 3]);
    input
        .compare_opening_preview(&admitted.production, 3)
        .unwrap();
    let expected = admitted.expected.read(case, 3).unwrap();
    let observed = admitted
        .actual
        .accept(
            &admitted.inputs,
            &admitted.production,
            ProductionOutputs::Frame(frame::Outputs {
                progress: frame::Progress::Completed,
                events,
            }),
            3,
        )
        .unwrap();
    compared(3, &expected.fields, &observed).unwrap();
}

fn check_contact_checkpoint_controls(
    admitted: &AdmittedCase,
    value: &State,
    at: usize,
    prior_countdown: frame::countdown_scene::Offer,
) {
    if at == 1 {
        reject_contact_history_splices(admitted, value);
        let mut stale = value.clone();
        let frame::countdown_caller::Stage::Complete { offer, .. } = &mut stale.countdown else {
            panic!("current completed Countdown");
        };
        assert_ne!(*offer, prior_countdown);
        *offer = prior_countdown;
        reject_native_checkpoint(admitted, Box::new(stale));
    }
    if at == 2 {
        reject_contact_history_splices(admitted, value);
        let mut missing = value.clone();
        let sample = missing
            .publications
            .events
            .iter_mut()
            .rev()
            .find(|event| matches!(event.kind, EventKind::AudioPortRead { .. }))
            .unwrap();
        sample.kind = EventKind::ControllerCommitted;
        reject_native_checkpoint(admitted, Box::new(missing));
    }
}

fn check_indexed_return_receipt(case: &Case, sources: &Sources) {
    let measured = crate::replay::run(case, sources);
    assert_eq!(measured.outcome, Outcome::Pass);
    assert_eq!(measured.counts.cuts, 3);
    assert_eq!(measured.counts.fields, 39);
    assert_eq!(measured.counts.publications, 6_916);
    assert_eq!(measured.counts.replies, 2);
    assert_eq!(measured.counts.outputs, 6_916);
    assert_eq!(measured.counts.restorations, 3);
}

fn reject_shifted_contact_clock(case: &Case, admitted: &AdmittedCase) {
    let port_field = admitted
        .expected
        .read(case, 2)
        .unwrap()
        .fields
        .into_iter()
        .find(|field| field.field == "frame.audio.ports")
        .unwrap();
    let mut earlier = state(&admitted.production)
        .publications
        .active()
        .unwrap()
        .to_vec();
    let read = earlier
        .iter_mut()
        .find(|event| matches!(event.kind, EventKind::AudioPortRead { .. }))
        .unwrap();
    read.at.0 = read.at.0.checked_sub(1).unwrap();
    assert_eq!(
        compared(
            2,
            &[port_field],
            &[crate::replay::frame::opening_audio::actual(&earlier)]
        ),
        Err(Outcome::Difference {
            cut: 2,
            field: "frame.audio.ports".to_owned()
        })
    );
}

fn reject_contact_history_splices(admitted: &AdmittedCase, actual: &State) {
    let mut wrong_time = actual.clone();
    let sample = wrong_time
        .publications
        .events
        .iter_mut()
        .rev()
        .find(|event| matches!(event.kind, EventKind::AudioPortRead { .. }))
        .unwrap();
    sample.at.0 = sample.at.0.checked_sub(1).unwrap();
    reject_native_checkpoint(admitted, Box::new(wrong_time));

    let mut wrong_byte = actual.clone();
    let EventKind::AudioPortRead { value, port } = &mut wrong_byte
        .publications
        .events
        .iter_mut()
        .rev()
        .find(|event| matches!(event.kind, EventKind::AudioPortRead { .. }))
        .unwrap()
        .kind
    else {
        panic!("actual Contact sample");
    };
    let read_port = *port;
    let cue = *value;
    *value ^= 1;
    reject_native_checkpoint(admitted, Box::new(wrong_byte));

    let mut wrong_trigger = actual.clone();
    let retry = wrong_trigger.contact_retried_at.unwrap();
    let first = wrong_trigger
        .publications
        .events
        .iter()
        .find(|event| {
            event.at < retry
                && matches!(&event.kind, EventKind::Publication(write)
            if write.port == read_port && write.value == cue)
        })
        .unwrap();
    wrong_trigger.contact_retried_at = Some(first.at);
    reject_native_checkpoint(admitted, Box::new(wrong_trigger));
}

fn ground_corridor_case() -> Case {
    let index = Index::parse(include_str!("../../../../../scenarios/replay/index.toml")).unwrap();
    index
        .cases
        .into_iter()
        .find(|case| case.id.0 == "frame/opening-sixteen-native-returns")
        .unwrap()
}

#[test]
#[ignore = "requires the authenticated opening corridor fixtures and checked player ROM"]
fn opening_ground_corridor_from_one_native_admission() {
    let case = ground_corridor_case();
    let sources = sources();
    if std::env::var_os("SM_OPENING_CORRIDOR_SINGLE_PASS").is_some() {
        let mut admitted = super::super::admit(&case, &sources).unwrap();
        if let InputReader::Frame(inputs) = &mut admitted.inputs {
            // The probe changes only host chunking; real owner work and returns
            // still delimit the independently compared observations.
            inputs.budget = MasterTicks(65_536);
        }
        for cut in admitted.schedule.clone() {
            runner::execute(
                &case,
                &admitted.inputs,
                &admitted.expected,
                &mut admitted.production,
                &mut admitted.actual,
                cut,
            )
            .unwrap_or_else(|outcome| {
                panic!(
                    "return request {cut} failed at master tick {}: {outcome:?}",
                    clock_at(&admitted.production)
                )
            });
            let saved = admitted.production.save().unwrap();
            let restored =
                Production::restore(&saved, admitted.content.clone(), admitted.rules.clone())
                    .unwrap_or_else(|fault| {
                        panic!("restore at native return request {cut}: {fault:?}")
                    });
            assert_observation_eq!(restored, admitted.production);
            if matches!(cut, 9 | 14) {
                checks_mixed_drawing_builder(&admitted, cut + 1);
            }
            if cut >= 11 {
                rejects_changed_camera_fraction(&admitted);
            }
        }
    } else {
        let result = crate::replay::run(&case, &sources);
        if let Some(path) = std::env::var_os("SM_OPENING_CORRIDOR_RECEIPT") {
            std::fs::write(path, toml::to_string_pretty(&result).unwrap()).unwrap();
        }
        assert!(result.passed(), "{:?}", result.outcome);
    }
}

#[test]
#[ignore = "requires the authenticated opening corridor fixtures and checked player ROM"]
fn opening_mixed_drawing_rejects_a_changed_door_tile_before_finalization() {
    let case = ground_corridor_case();
    let mut admitted = super::super::admit(&case, &sources()).unwrap();
    if let InputReader::Frame(inputs) = &mut admitted.inputs {
        inputs.budget = MasterTicks(65_536);
    }
    // Return ten is the first to draw the visible door beside Samus.
    for cut in admitted.schedule.iter().copied().take(9) {
        runner::execute(
            &case,
            &admitted.inputs,
            &admitted.expected,
            &mut admitted.production,
            &mut admitted.actual,
            cut,
        )
        .unwrap_or_else(|outcome| panic!("return request {cut}: {outcome:?}"));
    }
    checks_mixed_drawing_builder(&admitted, 10);
}

fn checks_mixed_drawing_builder(admitted: &AdmittedCase, cut: u64) {
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("opening frame inputs");
    };
    let mut production = admitted.production.clone();
    inputs
        .present_opening_preview(&mut production, cut)
        .unwrap();
    advance_to_ground_drawing(&mut production, |state| {
        state.drawing_energy_read().is_some()
    });
    let charging = canonical::to_bytes(state(&production));
    let saved = production.save().unwrap();
    let mut restored =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();

    for receiver in [&mut production, &mut restored] {
        advance_to_ground_drawing(receiver, |state| state.s6_drawing.builder().is_some());
    }
    assert_observation_eq!(production, restored);
    let completed = state(&production);
    assert_eq!(completed.rendering.receipt().unwrap().first_byte, 24);
    assert!(matches!(
        completed.rendering,
        frame::rendering_stage::Stage::Drawn(_)
    ));
    assert_eq!(
        completed.return_progress,
        frame::frame_return::Stage::Pending
    );
    let builder = completed.s6_drawing.builder().unwrap();
    // The actor append writes the first door piece at the start of the backing,
    // so byte two is that piece's tile.
    let mut altered = builder.clone();
    altered.backing[2] ^= 1;
    rejects_frame_state(admitted, replace_builder(&charging, builder, &altered));
    let drawn = canonical::to_bytes(completed);
    rejects_frame_state(admitted, replace_builder(&drawn, builder, &altered));

    let saved = production.save().unwrap();
    let reopened =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    assert_observation_eq!(production, reopened);
}

/// Replace the sole copy of a builder in a canonical frame image.
fn replace_builder(
    image: &[u8],
    builder: &frame::borrowed::SpriteBuilderOrigin,
    altered: &frame::borrowed::SpriteBuilderOrigin,
) -> State {
    let builder_image = canonical::to_bytes(builder);
    let mut locations = image
        .windows(builder_image.len())
        .enumerate()
        .filter_map(|(index, bytes)| (bytes == builder_image).then_some(index));
    let location = locations.next().unwrap();
    assert!(locations.next().is_none());
    let mut spliced = image.to_vec();
    let end = location.checked_add(builder_image.len()).unwrap();
    spliced[location..end].copy_from_slice(&canonical::to_bytes(altered));
    canonical::from_bytes(&spliced).unwrap()
}

fn advance_to_ground_drawing(production: &mut Production, reached: impl Fn(&State) -> bool) {
    while !reached(state(production)) {
        let ProductionOutputs::Frame(outputs) = production
            .advance(AdvanceInput::Frame {
                budget: MasterTicks(65_536),
            })
            .unwrap()
        else {
            panic!("opening frame outputs");
        };
        assert_eq!(outputs.progress, frame::Progress::Advanced);
    }
}

fn rejects_changed_camera_fraction(admitted: &AdmittedCase) {
    use types::Parts;
    let mut forged = state(&admitted.production).clone();
    let mut view = forged.camera.0.position();
    view.x = view.x.with_sub(view.x.sub() ^ 1);
    forged.camera.0 = forged.camera.0.with_view(world_camera::ViewPosition(view));
    rejects_frame_state(admitted, forged);
}

#[track_caller]
fn rejects_frame_state(admitted: &AdmittedCase, forged: State) {
    assert!(matches!(
        Production::admit(
            ProductionSeed::Frame(Box::new(forged)),
            admitted.content.clone(),
            admitted.rules.clone(),
        ),
        Err(game::production::AdmissionFault::Frame(fault)) if *fault == frame::Fault::InvalidPlayerEntry
    ));
}

#[test]
#[ignore = "requires the authenticated opening corridor fixtures and checked player ROM"]
fn opening_later_contact_ingress_is_causal_across_a_checkpoint() {
    let case = ground_corridor_case();
    let mut admitted = super::super::admit(&case, &sources()).unwrap();
    if std::env::var_os("SM_OPENING_CORRIDOR_SINGLE_PASS").is_some()
        && let InputReader::Frame(inputs) = &mut admitted.inputs
    {
        inputs.budget = MasterTicks(65_536);
    }
    for cut in admitted.schedule.iter().copied().take(12) {
        if cut == 11 {
            rejects_missing_recurring_footstep_gates(&case, &admitted, cut);
        }
        if cut == 12 {
            detects_premature_later_footstep(&case, &admitted, cut);
        }
        runner::execute(
            &case,
            &admitted.inputs,
            &admitted.expected,
            &mut admitted.production,
            &mut admitted.actual,
            cut,
        )
        .unwrap_or_else(|outcome| panic!("return request {cut}: {outcome:?}"));
    }
    let saved = admitted.production.save().unwrap();
    let current = state(&admitted.production);
    let retry = current.contact_later_retried_at.unwrap();
    let due = retry.0.checked_add(255_764).unwrap();
    assert_eq!(
        admitted.production.frame_contact_latches().unwrap().first(),
        Some(&0)
    );

    let mut forged = current.clone();
    forged.contact_later_retried_at = Some(MasterTicks(retry.0.wrapping_add(1)));
    rejects_frame_state(&admitted, forged);

    let mut restored =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("opening frame inputs");
    };
    let mut before_arrival = Vec::new();
    let mut opening_suffix = Vec::new();
    for (production, prefix) in [
        (&mut admitted.production, &mut before_arrival),
        (&mut restored, &mut opening_suffix),
    ] {
        inputs.present_opening_preview(production, 13).unwrap();
        let before = due.checked_sub(1).unwrap();
        advance_until_master_tick(production, before, prefix);
        assert_eq!(
            production.frame_contact_latches().unwrap().first(),
            Some(&0)
        );
        advance_until_master_tick(production, due, prefix);
        assert!(
            production
                .frame_contact_latches()
                .unwrap()
                .first()
                .is_some_and(|incoming| *incoming != 0)
        );
    }
    assert_observation_eq!(before_arrival, opening_suffix);
    assert_observation_eq!(restored, admitted.production);
}

fn rejects_missing_recurring_footstep_gates(case: &Case, admitted: &AdmittedCase, cut: u64) {
    let mut content = admitted.content.clone();
    let ProductionContent::Frame(frame) = &mut content else {
        panic!("opening frame content");
    };
    frame.post_down.as_mut().unwrap().recurring_footstep = None;
    let mut production = Production::admit(
        ProductionSeed::Frame(Box::new(state(&admitted.production).clone())),
        content,
        admitted.rules.clone(),
    )
    .unwrap();
    let before = clock_at(&production);
    let mut actual = admitted.actual.clone();
    let fault = runner::execute(
        case,
        &admitted.inputs,
        &admitted.expected,
        &mut production,
        &mut actual,
        cut,
    )
    .err()
    .expect("the distinct native control rejects");
    assert_eq!(
        fault,
        Outcome::Unsupported {
            cut: Some(cut),
            dependency: "Frame(PlayerContinuationPreparation)".to_owned(),
        }
    );
    assert_eq!(clock_at(&production).checked_sub(before), Some(82_200));
}

fn detects_premature_later_footstep(case: &Case, admitted: &AdmittedCase, cut: u64) {
    let mut content = admitted.content.clone();
    let ProductionContent::Frame(frame) = &mut content else {
        panic!("opening frame content");
    };
    frame
        .sender_physical
        .as_mut()
        .unwrap()
        .contact_continuation
        .as_mut()
        .unwrap()
        .admit_later_reply(MasterTicks(1))
        .unwrap();
    let mut production = Production::admit(
        ProductionSeed::Frame(Box::new(state(&admitted.production).clone())),
        content,
        admitted.rules.clone(),
    )
    .unwrap();
    let mut actual = admitted.actual.clone();
    let fault = runner::execute(
        case,
        &admitted.inputs,
        &admitted.expected,
        &mut production,
        &mut actual,
        cut,
    )
    .err()
    .expect("the distinct native control rejects");
    assert_eq!(
        fault,
        Outcome::Difference {
            cut,
            field: "frame.contact.incoming".to_owned(),
        }
    );
}

fn advance_until_master_tick(
    production: &mut Production,
    target: u64,
    events: &mut Vec<timing::Event>,
) {
    while clock_at(production) < target {
        let previous = production.clone();
        let remaining = target.checked_sub(clock_at(production)).unwrap();
        let ProductionOutputs::Frame(outputs) = production
            .advance(AdvanceInput::Frame {
                budget: MasterTicks(remaining),
            })
            .unwrap()
        else {
            panic!("opening frame outputs");
        };
        assert!(outputs.progress == frame::Progress::Advanced && *production != previous);
        events.extend(outputs.events);
    }
    assert_eq!(clock_at(production), target);
}
