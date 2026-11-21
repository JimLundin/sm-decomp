use super::runtime_helpers::retained_return;
use data::timing::MasterTicks;
use game::{
    production::{
        AdvanceInput, Production, ProductionFault, ProductionOutputs,
        timing::{Controller, EventKind, Fault, PadChange, PhysicalPad},
    },
    state::owned::OwnedState,
};

fn controller(production: &Production) -> &Controller {
    let OwnedState::Frame(state) = production.state() else {
        panic!("retained frame expected");
    };
    &state.timing.hardware.controller
}

fn until_controller_commit(production: &mut Production, budget: MasterTicks) -> MasterTicks {
    for _ in 0..500_000 {
        let ProductionOutputs::Frame(outputs) =
            production.advance(AdvanceInput::Frame { budget }).unwrap()
        else {
            panic!("retained frame output expected");
        };
        let mut commits = outputs
            .events
            .iter()
            .filter(|event| event.kind == EventKind::ControllerCommitted);
        if let Some(commit) = commits.next() {
            assert!(commits.next().is_none());
            return commit.at;
        }
    }
    panic!("physical controller latch absent");
}

#[test]
#[ignore = "requires reviewed complete retained frame source and player ROM"]
fn actual_frame_samples_host_changes_through_the_physical_latch_after_restore() {
    let (_, admitted) = retained_return();
    let mut direct = admitted.production;
    let mut recorded = direct.clone();
    let crossing = until_controller_commit(&mut recorded, MasterTicks(10_000));
    let held = if controller(&recorded).committed_pad == PhysicalPad(0x8000) {
        PhysicalPad(0x4000)
    } else {
        PhysicalPad(0x8000)
    };
    let press = PadChange {
        at: MasterTicks(crossing.0.checked_sub(1).unwrap()),
        pad: held,
    };
    let release = PadChange {
        at: MasterTicks(crossing.0.checked_add(1).unwrap()),
        pad: PhysicalPad::default(),
    };
    direct
        .schedule_frame_pad_changes(&[press, release])
        .unwrap();

    let saved = direct.save().unwrap();
    assert_eq!(
        direct.schedule_frame_pad_changes(&[release]),
        Err(ProductionFault::FrameHardware(Fault::PadChangesUnordered))
    );
    assert_observation_eq!(direct.save().unwrap(), saved);

    let mut split =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    assert_eq!(
        until_controller_commit(&mut direct, MasterTicks(10_000)),
        crossing
    );
    assert_eq!(
        until_controller_commit(&mut split, MasterTicks(3)),
        crossing
    );
    for production in [&direct, &split] {
        assert_observation_eq!(controller(production).committed_pad, held);
        assert_observation_eq!(controller(production).pending_pad, [release]);
    }
    let after_latch = split.save().unwrap();
    assert_eq!(
        split.schedule_frame_pad_changes(&[press]),
        Err(ProductionFault::FrameHardware(Fault::PadChangeInPast))
    );
    assert_observation_eq!(split.save().unwrap(), after_latch);

    let mut restored = Production::restore(&after_latch, admitted.content, admitted.rules).unwrap();
    let input = || AdvanceInput::Frame {
        budget: MasterTicks(3),
    };
    let outputs = split.advance(input()).unwrap();
    assert_observation_eq!(restored.advance(input()).unwrap(), outputs);
    assert_observation_eq!(restored.state(), split.state());
    let ProductionOutputs::Frame(outputs) = outputs else {
        panic!("retained frame output expected");
    };
    assert!(
        outputs
            .events
            .iter()
            .all(|event| event.kind != EventKind::ControllerCommitted)
    );
    assert_observation_eq!(controller(&restored).committed_pad, held);
    assert_observation_eq!(controller(&restored).pending_pad, [release]);
}
