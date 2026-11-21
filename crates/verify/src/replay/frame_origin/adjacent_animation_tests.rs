use super::*;
use crate::replay::readers::ExpectedReader;
use frame::{animation_caller::Stage as Animation, appearance_caller::Kind};
use game::production::ProductionRules;

fn advance_to_return(
    production: &mut Production,
    end: u64,
    budget: MasterTicks,
    events: &mut Vec<Event>,
) {
    advance_frame_with_budget(
        production,
        end,
        budget,
        |state| matches!(state.animation, Animation::Complete(_)),
        |output| {
            assert_eq!(output.progress, frame::Progress::Advanced);
            events.extend(output.events);
        },
    );
}

pub(super) fn check(admitted: &AdmittedCase, mut direct: Production, origin: u64, cut: u64) {
    let ExpectedReader::Frame(expected) = &admitted.expected else {
        panic!("frame expected reader")
    };
    let source = expected.adjacent_animation.as_ref().unwrap();
    let observed = source.read(cut).unwrap();
    let start = origin.checked_add(observed.start_ticks).unwrap();
    let end = origin.checked_add(observed.return_ticks).unwrap();
    let ready = current(&direct);
    assert_eq!(ready.timing.hardware.now.0, start);
    let interval = current(&admitted.production).publications.events.len();
    let mut events = ready.publications.events[interval..].to_vec();
    let mut from_timer = restore(admitted, &direct);
    let mut timer_events = events.clone();
    let identity = Kind::Animation.identity();
    assert_eq!(identity, WorkId(1101));
    let starting_channel = ready.timing.hardware.transfers.channels[2].clone();
    reject_contracts(admitted, &direct);
    advance_frame_with_budget(
        &mut direct,
        end,
        MasterTicks(1),
        |state| hud_work_in_flight(state, identity),
        |output| events.extend(output.events),
    );
    let charging = current(&direct);
    let work = charging.timing.work.last().unwrap();
    assert_eq!(work.kind, WorkKind::Semantic(identity));
    let exposures = work
        .profile
        .runs
        .iter()
        .map(|run| u64::from(run.count))
        .sum::<u64>();
    let intrinsic = work
        .profile
        .runs
        .iter()
        .map(|run| {
            u64::from(run.count)
                .checked_mul(u64::from(run.exposure.cycle.ticks()))
                .unwrap()
        })
        .sum::<u64>();
    assert_eq!(
        [exposures, intrinsic],
        [observed.exposures, observed.intrinsic_ticks]
    );
    let Animation::Charging(selection) = &charging.animation else {
        panic!("Animation charging")
    };
    let receipt = selection.receipt().unwrap();
    assert_eq!(
        receipt.chosen.model.0.pending.as_ref().unwrap().answer,
        samus_appearance::Answer::CountDown
    );
    assert_observation_eq!(
        charging.appearance.model.0.state,
        receipt.before.model.0.state
    );
    reject_contracts(admitted, &direct);
    reject_charge(admitted, &direct, identity);
    reject_changed_offer(admitted, &direct);

    let mut from_charge = restore(admitted, &direct);
    let mut charge_events = events.clone();
    finish_direct(
        &mut direct,
        [start, end],
        &observed,
        &starting_channel,
        &mut events,
    );

    advance_to_return(&mut from_timer, end, MasterTicks(7), &mut timer_events);
    advance_to_return(&mut from_charge, end, MasterTicks(1), &mut charge_events);
    assert_observation_eq!(events, timer_events);
    assert_observation_eq!(events, charge_events);
    assert_observation_eq!(direct.save().unwrap(), from_timer.save().unwrap());
    assert_observation_eq!(direct.save().unwrap(), from_charge.save().unwrap());
    reject_contracts(admitted, &direct);
    reject_missing_decrement(admitted, &direct);
    reject_changed_offer(admitted, &direct);
    let mut complete = restore(admitted, &direct);
    source.compare(expected, &complete, cut, &events).unwrap();
    let before = complete.save().unwrap();
    assert!(matches!(
        complete.advance(AdvanceInput::Frame {
            budget: MasterTicks(1)
        }),
        Err(ProductionFault::Unsupported(Unsupported::Frame(
            Dependency::InterruptionPreparation
        )))
    ));
    assert_observation_eq!(complete.save().unwrap(), before);
}

fn finish_direct(
    direct: &mut Production,
    [start, end]: [u64; 2],
    observed: &crate::replay::frame::adjacent_animation::Observed,
    starting_channel: &game::production::timing::Channel,
    events: &mut Vec<Event>,
) {
    let mut refreshes = 0_u64;
    let mut previous_line_paid = current(direct).timing.hardware.beam.refreshed;
    let mut advanced_channel = false;
    while !matches!(current(direct).animation, Animation::Complete(_)) {
        assert!(current(direct).timing.hardware.now.0 <= end);
        let ProductionOutputs::Frame(output) = direct
            .advance(AdvanceInput::Frame {
                budget: MasterTicks(13),
            })
            .unwrap()
        else {
            panic!("frame output")
        };
        events.extend(output.events);
        let hardware = &current(direct).timing.hardware;
        if !previous_line_paid && hardware.beam.refreshed {
            refreshes = refreshes.checked_add(1).unwrap();
        }
        previous_line_paid = hardware.beam.refreshed;
        let channel = &hardware.transfers.channels[2];
        if channel.lines != starting_channel.lines {
            assert_eq!(
                channel.lines,
                starting_channel.lines.checked_sub(1).unwrap()
            );
            assert_eq!(channel.table, starting_channel.table);
            assert_eq!(channel.pointer, starting_channel.pointer);
            assert!(channel.line.enabled && !channel.line.completed && !channel.line.payload);
            advanced_channel = true;
        }
    }
    assert!(advanced_channel);
    assert_eq!(refreshes, 1);
    let hardware = &current(direct).timing.hardware;
    assert_eq!(
        hardware.now.0.checked_sub(start).unwrap(),
        observed
            .intrinsic_ticks
            .checked_add(observed.refresh_ticks)
            .unwrap()
            .checked_add(observed.descriptor_only_hdma_ticks)
            .unwrap()
    );
}

fn reject_contracts(admitted: &AdmittedCase, production: &Production) {
    use data::assets::world::rooms::{CERES_ELEVATOR_ROOM, CERES_RIDLEY_ROOM};
    use frame::inside_world::InsideWorld;
    let (ProductionContent::Frame(content), ProductionRules::Frame(rules)) =
        (&admitted.content, &admitted.rules)
    else {
        panic!("frame scope")
    };
    let state = current(production);
    let alternatives: [fn(&mut frame::animation_caller::Adjacent); 4] = [
        |adjacent| {
            adjacent.room = if adjacent.room == CERES_RIDLEY_ROOM {
                CERES_ELEVATOR_ROOM
            } else {
                CERES_RIDLEY_ROOM
            }
        },
        |adjacent| adjacent.quiet_fx = false,
        |adjacent| adjacent.x_divisor = 1,
        |adjacent| adjacent.origin = [0; 32],
    ];
    for (index, mutate) in alternatives.into_iter().enumerate() {
        let mut changed = Box::new(state.clone());
        let mut content = content.clone();
        let adjacent = content.adjacent_animation.as_mut().unwrap();
        mutate(adjacent);
        if index == 0 {
            // Keep the live room and geometry agreed with each other, and move them
            // to the independently wrong room. The Animation admission stays fixed.
            let room = content.movement_room.as_mut().unwrap();
            let wrong = adjacent.room;
            adjacent.room = room.identity.unwrap();
            room.identity = Some(wrong);
            changed.inside_world = Some(canonical::model::Model(
                InsideWorld::for_room(wrong, room.width, room.cells.clone(), room.boundary)
                    .unwrap(),
            ));
        }
        reject_animation_contract(admitted, changed, content, rules);
    }
    let mut wrong_family = content.clone();
    wrong_family.adjacent_animation.as_mut().unwrap().physical = content.animation.clone().unwrap();
    reject_animation_contract(admitted, Box::new(state.clone()), wrong_family, rules);
}

fn reject_animation_contract(
    admitted: &AdmittedCase,
    mut changed: Box<State>,
    content: Box<frame::Content>,
    rules: &frame::Rules,
) {
    if matches!(changed.animation, Animation::Pending) {
        let before = changed.clone();
        assert!(matches!(
            changed.advance(&content, rules, MasterTicks(1)),
            Err(ProductionFault::Unsupported(Unsupported::Frame(
                Dependency::AnimationPreparation
            )))
        ));
        assert_observation_eq!(changed, before);
    } else {
        assert_scene_checkpoint_rejected(admitted, changed, ProductionContent::Frame(content));
    }
}

fn reject_charge(admitted: &AdmittedCase, production: &Production, identity: WorkId) {
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let prior = adjacent_health_tests::first_animation_bill(current(&admitted.production), content);
    for bill in [prior, {
        let mut shifted = current(production)
            .timing
            .work
            .last()
            .unwrap()
            .profile
            .clone();
        let exposure = &mut shifted.runs.last_mut().unwrap().exposure;
        exposure.poll_before = !exposure.poll_before;
        shifted
    }] {
        assert!(super::super::rejects_changed_frame(
            admitted,
            production,
            |state| {
                let work = state.timing.work.last_mut().unwrap();
                assert_eq!(work.kind, WorkKind::Semantic(identity));
                work.profile = bill;
            }
        ));
    }
    assert!(super::super::rejects_changed_frame(
        admitted,
        production,
        |state| {
            state.liquid_history.as_mut().unwrap().invalidate();
        }
    ));
}

fn reject_changed_offer(admitted: &AdmittedCase, production: &Production) {
    assert!(super::super::rejects_changed_frame(
        admitted,
        production,
        |state| {
            use frame::appearance_caller::Stage;
            use samus_appearance::Receipt;
            let (receipt, charging) = match &mut state.animation {
                Animation::Charging(Stage::Charging { receipt, .. }) => (receipt, true),
                Animation::Complete(done) => (&mut done.receipt, false),
                Animation::Pending | Animation::Charging(Stage::Idle | Stage::Offered(_)) => {
                    panic!("selected adjacent Animation")
                }
            };
            let answer = receipt.chosen.model.0.pending.as_mut().unwrap();
            answer.inputs.run_held = !answer.inputs.run_held;
            let Some(Receipt::Animation(completed)) = receipt.result.model.0.completed.last_mut()
            else {
                panic!("completed ClipPlaying offer")
            };
            completed.inputs.run_held = answer.inputs.run_held;
            assert!(receipt.validate().is_ok());
            state.appearance = if charging {
                receipt.chosen.clone()
            } else {
                receipt.result.clone()
            };
        }
    ));
}

fn reject_missing_decrement(admitted: &AdmittedCase, production: &Production) {
    assert!(super::super::rejects_changed_frame(
        admitted,
        production,
        |state| {
            let Animation::Complete(done) = &mut state.animation else {
                panic!("Animation return")
            };
            let hold = done.receipt.before.model.0.state.playhead.time_left;
            state.appearance.model.0.state.playhead.time_left = hold;
            done.receipt.result.model.0.state.playhead.time_left = hold;
        }
    ));
}
