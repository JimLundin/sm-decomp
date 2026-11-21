use super::*;
use crate::replay::frame_origin::Member;
use game::production::{
    frame::{
        hud_camera_caller::{HUD_REFRESH, HUD_RETURN, Refresh},
        hud_map_caller::Stage,
        post_down_caller,
    },
    timing::{WorkId, WorkKind},
};

#[derive(serde::Deserialize)]
struct Supplement {
    adjacent_hud_observations: Member,
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
    fn read(case: &Case, sources: &Sources) -> Self {
        let reference = case.end.native_returns.as_ref().unwrap();
        let member = Member {
            path: sources.native.join(&reference.path),
            sha256: reference.sha256.clone(),
        };
        let supplement: Supplement = toml::from_str(&member.text().unwrap()).unwrap();
        let root = member.path.parent().unwrap();
        let mut observations = supplement.adjacent_hud_observations;
        observations.resolve(root);
        let mut expected: Self = toml::from_str(&observations.text().unwrap()).unwrap();
        assert_eq!(expected.format, 1);
        let root = observations.path.parent().unwrap();
        expected.tilemap.resolve(root);
        expected.explored.resolve(root);
        expected
    }
}

fn current(production: &Production) -> &State {
    let OwnedState::Frame(state) = production.state() else {
        panic!("frame production");
    };
    state
}

pub(super) fn check_hud_map(case: &Case, sources: &Sources, admitted: &AdmittedCase) {
    let expected = Expected::read(case, sources);
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content");
    };
    let address = content.hud_refresh.as_ref().unwrap().address;
    let mut direct = admitted.production.clone();
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
    let mut restored = Production::restore(
        &direct.save().unwrap(),
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
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
