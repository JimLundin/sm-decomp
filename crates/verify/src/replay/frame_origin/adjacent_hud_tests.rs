use super::*;
use crate::replay::{
    frame as frame_replay, frame_countdown_scene::AdjacentProjection, frame_origin::Member,
    frame_timing,
};
use data::timing::Interrupt;
use game::production::{
    frame::{
        countdown_caller::{COUNTDOWN, Stage as CountdownStage},
        countdown_scene::{Admission, Choice},
        hud_camera_caller::{HUD_REFRESH, HUD_RETURN, Refresh},
        hud_map_caller::Stage,
        interrupts::DrawingRequest,
        post_down_caller,
    },
    timing::{Event, WorkId, WorkKind},
};
use interface_hud::{CountdownAnswer, CountdownExpiry, CountdownOpportunity, CountdownPicture};

#[derive(serde::Deserialize)]
struct Supplement {
    adjacent_hud_observations: Member,
    adjacent_countdown_scene: Option<Member>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    format: u16,
    clock: u8,
    opening_ticks: u64,
    ready_ticks: u64,
    return_ticks: u64,
    tilemap: Member,
    explored: Member,
}

impl Expected {
    fn read(case: &Case, sources: &Sources) -> (Self, Option<AdjacentProjection>) {
        let reference = case.end.native_returns.as_ref().unwrap();
        let member = Member {
            path: sources.native.join(&reference.path),
            sha256: reference.sha256.clone(),
        };
        let supplement: Supplement = toml::from_str(&member.text().unwrap()).unwrap();
        let supplement_root = member.path.parent().unwrap();
        let mut observations = supplement.adjacent_hud_observations;
        observations.resolve(supplement_root);
        let mut expected: Self = toml::from_str(&observations.text().unwrap()).unwrap();
        assert_eq!(expected.format, 1);
        let root = observations.path.parent().unwrap();
        expected.tilemap.resolve(root);
        expected.explored.resolve(root);
        let countdown = supplement.adjacent_countdown_scene.map(|mut member| {
            member.resolve(supplement_root);
            AdjacentProjection::read(&member.text().unwrap()).unwrap()
        });
        (expected, countdown)
    }
}

fn current(production: &Production) -> &State {
    let OwnedState::Frame(state) = production.state() else {
        panic!("frame production");
    };
    state
}

pub(super) fn check_hud_map(case: &Case, sources: &Sources, admitted: &AdmittedCase) -> Dependency {
    let (expected, countdown) = Expected::read(case, sources);
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content");
    };
    let address = content.hud_refresh.as_ref().unwrap().address;
    let mut direct = restore(admitted, &admitted.production);
    let origin = current(&direct).timing.hardware.now.0;
    let opening = origin.checked_add(expected.opening_ticks).unwrap();
    let ready = origin.checked_add(expected.ready_ticks).unwrap();
    let returned = origin.checked_add(expected.return_ticks).unwrap();
    advance_frame_until(&mut direct, opening, |state| {
        matches!(
            &state.caller,
            Caller::PlayerContinuation {
                post_down: post_down_caller::Stage::Complete(_),
                ..
            }
        ) && matches!(state.hud_map, Stage::Pending)
    });
    assert_eq!(current(&direct).timing.hardware.now.0, opening);
    reject_current_turn_on(admitted, &direct);
    reject_old_clock(admitted, &direct, expected.clock);
    advance_frame_until(&mut direct, ready, |state| {
        hud_work_in_flight(state, HUD_REFRESH)
    });
    let refresh_bill_rejected = reject_changed_bill(admitted, &direct, HUD_REFRESH);
    reject_old_clock(admitted, &direct, expected.clock);
    let mut restored = restore(admitted, &direct);
    let mut receipts = Vec::new();
    let mut return_bill_rejected = false;
    for (production, budget) in [
        (&mut direct, MasterTicks(13)),
        (&mut restored, MasterTicks(1)),
    ] {
        let mut events = Vec::new();
        advance_frame_with_budget(production, ready, budget, phase_ready, |output| {
            events.extend(output.events);
        });
        let state = current(production);
        assert_eq!(state.timing.hardware.now.0, ready);
        assert_ready(state, &expected, address);
        advance_frame_with_budget(
            production,
            returned,
            MasterTicks(1),
            |state| hud_work_in_flight(state, HUD_RETURN),
            |output| events.extend(output.events),
        );
        return_bill_rejected = reject_changed_bill(admitted, production, HUD_RETURN);
        advance_frame_with_budget(
            production,
            returned,
            budget,
            |state| state.hud_map.is_complete(),
            |output| {
                events.extend(output.events);
            },
        );
        let state = current(production);
        assert_eq!(state.timing.hardware.now.0, returned);
        assert_ready(state, &expected, address);
        receipts.push((production.save().unwrap(), events));
    }
    assert_observation_eq!(receipts[0], receipts[1]);
    assert_eq!(
        [refresh_bill_rejected, return_bill_rejected],
        [true; 2],
        "coherently changed active producer and return profiles must both reject"
    );
    let Some(countdown) = countdown else {
        return Dependency::CountdownSceneScheduling;
    };
    let mut undeclared = case.clone();
    undeclared
        .borrowed
        .retain(|borrow| borrow.dependency != "frame.adjacent-countdown-scene");
    assert!(super::super::admit(&undeclared, sources).is_err());
    check_timer(admitted, &mut direct, origin, countdown.timer_boundaries());
    compare_timer_outputs(case, admitted, current(&direct));
    Dependency::HealthCurrentInputs
}

fn hud_work_in_flight(state: &State, identity: WorkId) -> bool {
    state.timing.work.last().is_some_and(|work| {
        work.kind == WorkKind::Semantic(identity) && work.cursor.intrinsic.0 > 0
    })
}

fn reject_changed_bill(admitted: &AdmittedCase, production: &Production, id: WorkId) -> bool {
    super::rejects_changed_frame(admitted, production, |state| {
        let work = state.timing.work.last_mut().unwrap();
        assert_eq!(work.kind, WorkKind::Semantic(id));
        let run = work.profile.runs.last_mut().unwrap();
        run.count = run.count.checked_add(1).unwrap();
        assert!(state.validate().is_ok());
    })
}

const fn phase_ready(state: &State) -> bool {
    matches!(
        state.hud_map,
        Stage::Active {
            phase: Refresh::Ready,
            ..
        }
    )
}

fn assert_ready(state: &State, expected: &Expected, address: data::dma::BusAddress) {
    let regions = state.content.regions();
    let mut tiles = regions.iter().filter(|region| region.start == address);
    assert_observation_eq!(
        tiles.next().unwrap().bytes,
        expected.tilemap.read().unwrap()
    );
    assert!(tiles.next().is_none());
    assert_observation_eq!(
        crate::replay::hud::explored_bitmap(state.hud.state()),
        expected.explored.read().unwrap()
    );
    assert_observation_eq!(
        state
            .hud_display_current
            .as_ref()
            .unwrap()
            .0
            .premises
            .frame
            .0,
        expected.clock
    );
    assert_eq!(state.hud.events().len(), 0);
}

fn reject_old_clock(admitted: &AdmittedCase, production: &Production, clock: u8) {
    let mut altered = current(production).clone();
    assert_observation_eq!(altered.hud_frame_count, Some(clock));
    let old = clock.wrapping_sub(1);
    let blink = interface_hud::Rules::reference().her_blink;
    assert_eq!(
        blink.first_half(interface_hud::FrameCount(old)),
        blink.first_half(interface_hud::FrameCount(clock))
    );
    altered.hud_frame_count = Some(old);
    reject_display(admitted, altered, admitted.content.clone());
}

fn reject_current_turn_on(admitted: &AdmittedCase, production: &Production) {
    let mut altered = admitted.content.clone();
    let ProductionContent::Frame(content) = &mut altered else {
        panic!("frame content");
    };
    let current_events = &mut content
        .hud_refresh
        .as_mut()
        .unwrap()
        .recurring
        .as_mut()
        .unwrap()
        .map_events;
    assert!(!current_events.turned_on);
    current_events.turned_on = true;
    reject_display(admitted, current(production).clone(), altered);
}

fn reject_display(admitted: &AdmittedCase, state: State, content: ProductionContent) {
    use game::production::{AdmissionFault, frame::Fault};

    let active = state.hud_map.identity().is_some();
    let admission = Production::admit(
        ProductionSeed::Frame(Box::new(state)),
        content,
        admitted.rules.clone(),
    );
    if active {
        assert!(matches!(
            admission,
            Err(AdmissionFault::Frame(fault)) if *fault == Fault::InvalidPlayerEntry
        ));
        return;
    }
    let mut invalid = admission.unwrap();
    let before = invalid.save().unwrap();
    assert!(matches!(
        invalid.advance(AdvanceInput::Frame {
            budget: MasterTicks(13)
        }),
        Err(ProductionFault::Unsupported(Unsupported::Frame(
            Dependency::HudDisplayPreparation
        )))
    ));
    assert_observation_eq!(invalid.save().unwrap(), before);
}

fn restore(admitted: &AdmittedCase, production: &Production) -> Production {
    let restored = Production::restore(
        &production.save().unwrap(),
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    assert_observation_eq!(restored.state(), production.state());
    restored
}

fn check_timer(
    admitted: &AdmittedCase,
    direct: &mut Production,
    origin: u64,
    [started, returned]: [u64; 2],
) {
    let returned = origin.checked_add(returned).unwrap();
    let ready = current(direct);
    assert_eq!(
        ready.timing.hardware.now.0,
        origin.checked_add(started).unwrap()
    );
    let prior_nmi = ready
        .publications
        .events
        .iter()
        .find(|event| event.kind == EventKind::Interrupt(Interrupt::Nmi))
        .unwrap();
    let moved_nmi = Event {
        at: ready.timing.hardware.now,
        kind: prior_nmi.kind.clone(),
    };
    reject_stale_scene(admitted, ready, moved_nmi);
    reject_current_invalidations(admitted, ready);
    reject_missing_predecessor(admitted, ready);
    let before = ready.hud.state().clone();
    advance_frame_with_budget(
        direct,
        returned,
        MasterTicks(1),
        |state| hud_work_in_flight(state, COUNTDOWN),
        |output| assert_eq!(output.events.len(), 0),
    );
    assert!(matches!(
        current(direct).countdown,
        CountdownStage::Charging { .. }
    ));
    assert_eq!(current(direct).countdown_delivered.len(), 0);
    reject_changed_timer_checkpoint(admitted, current(direct));
    let mut restored = restore(admitted, direct);
    for (production, budget) in [
        (&mut *direct, MasterTicks(13)),
        (&mut restored, MasterTicks(1)),
    ] {
        advance_frame_with_budget(
            production,
            returned,
            budget,
            |state| matches!(state.countdown, CountdownStage::Complete { .. }),
            |output| assert_eq!(output.events.len(), 0),
        );
        let state = current(production);
        assert_eq!(state.timing.hardware.now.0, returned);
        assert_observation_eq!(state.hud.state(), &before);
        assert_observation_eq!(
            state
                .countdown_delivered
                .iter()
                .map(|answer| answer.0)
                .collect::<Vec<_>>(),
            [CountdownAnswer {
                opportunity: CountdownOpportunity::MaintainCountdown,
                expiry: CountdownExpiry::NoExpiry,
                picture: CountdownPicture::Absent,
            }],
        );
    }
    assert_observation_eq!(direct.save().unwrap(), restored.save().unwrap());
    reject_current_invalidations(admitted, current(direct));
}

fn compare_timer_outputs(case: &Case, admitted: &AdmittedCase, through: &State) {
    let cut = case.end.cut_inclusive;
    let cut_expected = admitted.expected.read(case, cut).unwrap();
    let stream = cut_expected
        .fields
        .iter()
        .find(|field| field.field == "frame.outputs")
        .unwrap();
    let mut expected: Vec<Datum> = canonical::from_bytes(&stream.bytes).unwrap();
    expected.retain(|datum| datum.at <= through.timing.hardware.now);
    let first_len = current(&admitted.production).publications.events.len();
    let actual = frame_replay::project(&through.publications.events[first_len..]).unwrap();
    let difference = frame_timing::difference(
        &canonical::to_bytes(&expected),
        &canonical::to_bytes(&actual),
    );
    assert!(
        difference.is_none(),
        "{} cut {cut} Timer handoff has {} callbacks, expected {}. First difference frame.outputs{}",
        case.id.0,
        actual.len(),
        expected.len(),
        difference.as_deref().unwrap_or_default()
    );
}

fn reject_stale_scene(admitted: &AdmittedCase, ready: &State, moved_nmi: Event) {
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let game::production::ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    let mut active_nmi = Box::new(ready.clone());
    active_nmi.publications.record(vec![moved_nmi]).unwrap();
    active_nmi.publications.deliver().unwrap();
    let mut same_value_write = Box::new(ready.clone());
    same_value_write
        .scene_countdown
        .as_mut()
        .unwrap()
        .0
        .write(Choice::MaintainCountdown);
    let mut missing_end = Box::new(ready.clone());
    missing_end.drawing.as_mut().unwrap().recurring_due = Some(DrawingRequest::End);
    let mut wrong_choice = content.as_ref().clone();
    wrong_choice.adjacent_scene_countdown = Some(Admission {
        choice: Choice::PresentCountdownOnly,
    });
    for (mut changed, content) in [
        (active_nmi, content.as_ref()),
        (same_value_write, content.as_ref()),
        (missing_end, content.as_ref()),
        (Box::new(ready.clone()), &wrong_choice),
    ] {
        let saved = changed.clone();
        assert_eq!(
            changed.advance(content, rules, MasterTicks(1)).err(),
            Some(ProductionFault::Unsupported(Unsupported::Frame(
                Dependency::CountdownSceneScheduling
            )))
        );
        assert_observation_eq!(changed, saved);
    }
}

fn reject_changed_timer_checkpoint(admitted: &AdmittedCase, active: &State) {
    let CountdownStage::Complete {
        offer: prior_offer, ..
    } = current(&admitted.production).countdown
    else {
        panic!("first Timer completed before native return");
    };
    let mut old_offer = Box::new(active.clone());
    let CountdownStage::Charging { offer, .. } = &mut old_offer.countdown else {
        panic!("adjacent Timer is in flight");
    };
    *offer = prior_offer;
    let mut missing_scene = Box::new(active.clone());
    missing_scene.scene_countdown = None;
    let mut wrong_profile = Box::new(active.clone());
    let work = wrong_profile
        .timing
        .work
        .iter_mut()
        .find(|work| work.kind == WorkKind::Semantic(COUNTDOWN))
        .unwrap();
    let run = work.profile.runs.last_mut().unwrap();
    run.exposure.poll_before = !run.exposure.poll_before;
    for changed in [old_offer, missing_scene, wrong_profile] {
        assert_scene_checkpoint_rejected(admitted, changed, admitted.content.clone());
    }
}

fn reject_current_invalidations(admitted: &AdmittedCase, state: &State) {
    use data::timing::{DeviceWrite, Publication, WRAM_DATA_PORT};

    for kind in [
        EventKind::Interrupt(Interrupt::Nmi),
        EventKind::Publication(DeviceWrite {
            port: WRAM_DATA_PORT,
            value: 0,
            effect: Publication::Downstream,
        }),
    ] {
        let mut changed = Box::new(state.clone());
        let event = Event {
            at: changed.timing.hardware.now,
            kind,
        };
        changed.publications.record(vec![event]).unwrap();
        changed.publications.deliver().unwrap();
        assert_scene_checkpoint_rejected(admitted, changed, admitted.content.clone());
    }
}

fn reject_missing_predecessor(admitted: &AdmittedCase, ready: &State) {
    let mut missing_scene = Box::new(ready.clone());
    missing_scene.scene_countdown = None;
    assert_scene_checkpoint_rejected(admitted, missing_scene, admitted.content.clone());
    for choice in [None, Some(Choice::PresentCountdownOnly)] {
        let mut changed = admitted.content.clone();
        let ProductionContent::Frame(content) = &mut changed else {
            panic!("frame content")
        };
        content.scene_countdown = choice.map(|choice| Admission { choice });
        assert_scene_checkpoint_rejected(admitted, Box::new(ready.clone()), changed);
    }
}

fn assert_scene_checkpoint_rejected(
    admitted: &AdmittedCase,
    changed: Box<State>,
    content: ProductionContent,
) {
    use game::production::AdmissionFault;

    // Restore decodes the complete seed and runs this same admission path.
    assert!(changed.validate().is_ok());
    assert!(matches!(
        Production::admit(
            ProductionSeed::Frame(changed),
            content,
            admitted.rules.clone(),
        ),
        Err(AdmissionFault::Frame(fault)) if *fault == frame::Fault::InvalidPlayerEntry
    ));
}
