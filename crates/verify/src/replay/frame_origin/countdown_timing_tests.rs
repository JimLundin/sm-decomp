use super::runtime_helpers::{admitted, advance_until};
use super::*;
use frame::countdown_caller::{COUNTDOWN, Stage};
use game::{
    production::{AdvanceInput, timing::WorkKind},
    state::owned::OwnedState,
};

fn intrinsic(state: &frame::State, consumed: u64) -> u64 {
    state
        .timing
        .work
        .iter()
        .find(|work| work.kind == WorkKind::Semantic(COUNTDOWN))
        .map_or(consumed, |work| work.cursor.intrinsic.0)
}

#[test]
#[ignore = "requires qualified countdown origin and actual retained hardware service"]
fn actual_countdown_intrinsic_and_wall_progress_include_one_retained_refresh() {
    let mut admitted = admitted();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    advance_until(&mut admitted, budget, |state| {
        matches!(
            state.caller,
            frame::Caller::PlayerContinuation {
                post_down: frame::post_down_caller::Stage::Complete(_),
                ..
            }
        )
    });
    advance_until(&mut admitted, budget, |state| state.hud_map.is_complete());
    let source_intrinsic = countdown_profile_ticks(&admitted.content);
    let OwnedState::Frame(initial) = admitted.production.state() else {
        panic!("frame state")
    };
    let initial_wall = initial.timing.hardware.now.0;
    let initial_events = initial.publications.events.len();
    let mut actual_intrinsic = 0_u64;
    let mut refresh_ticks = 0_u64;
    let mut refresh_transitions = 0_u64;
    for _ in 0..100_000 {
        let OwnedState::Frame(before) = admitted.production.state() else {
            panic!("frame state")
        };
        if matches!(before.countdown, Stage::Complete { .. }) {
            break;
        }
        let before = before.as_ref().clone();
        let previous_intrinsic = intrinsic(&before, actual_intrinsic);
        admitted
            .production
            .advance(AdvanceInput::Frame { budget })
            .unwrap();
        let OwnedState::Frame(after) = admitted.production.state() else {
            panic!("frame state")
        };
        actual_intrinsic = intrinsic(after, previous_intrinsic);
        let intrinsic_delta = actual_intrinsic.checked_sub(previous_intrinsic).unwrap();
        let wall_delta = after
            .timing
            .hardware
            .now
            .0
            .checked_sub(before.timing.hardware.now.0)
            .unwrap();
        let extra_wall = wall_delta.checked_sub(intrinsic_delta).unwrap();
        assert_eq!(after.publications.events.len(), initial_events);
        assert!(
            after
                .timing
                .work
                .iter()
                .all(|work| work.kind == WorkKind::Semantic(COUNTDOWN))
        );
        assert_eq!(
            after.timing.hardware.transfers.intrinsic,
            before.timing.hardware.transfers.intrinsic
        );
        if extra_wall != 0 {
            assert!(!before.timing.hardware.beam.refreshed);
            assert!(after.timing.hardware.beam.refreshed);
            assert_eq!(
                before.timing.hardware.beam.position.line,
                after.timing.hardware.beam.position.line
            );
            refresh_transitions = refresh_transitions.checked_add(1).unwrap();
            refresh_ticks = refresh_ticks.checked_add(extra_wall).unwrap();
        }
    }
    let OwnedState::Frame(completed) = admitted.production.state() else {
        panic!("frame state")
    };
    assert!(matches!(completed.countdown, Stage::Complete { .. }));
    let actual_wall = completed
        .timing
        .hardware
        .now
        .0
        .checked_sub(initial_wall)
        .unwrap();
    assert_eq!(completed.publications.events.len(), initial_events);
    assert_eq!(actual_intrinsic, source_intrinsic);
    assert_eq!(actual_intrinsic, 716);
    assert_eq!(actual_wall, 756);
    assert_eq!(refresh_transitions, 1);
    assert_eq!(refresh_ticks, 40);
    assert_eq!(
        actual_wall,
        actual_intrinsic.checked_add(refresh_ticks).unwrap()
    );
}

fn countdown_profile_ticks(content: &ProductionContent) -> u64 {
    let ProductionContent::Frame(content) = content else {
        panic!("frame content")
    };
    content
        .countdown
        .as_ref()
        .unwrap()
        .template
        .runs
        .iter()
        .map(|run| u64::from(run.count).saturating_mul(u64::from(run.cycle.ticks())))
        .sum()
}
