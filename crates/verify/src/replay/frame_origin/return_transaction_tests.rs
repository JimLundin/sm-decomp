//! Compare the atomic return with the public step entry on both sides of a held boundary.

use super::*;

#[test]
#[ignore = "requires the retained native Frame fixture and checked player ROM"]
fn complete_return_commits_all_steps_and_a_held_return_rolls_back() {
    let sources = Sources {
        native: std::env::var_os("SM_AUDIO_NATIVE").unwrap().into(),
        rom: std::env::var_os("SM_ROM").unwrap().into(),
        ..Sources::default()
    };
    let index = Index::parse(include_str!("../../../../../scenarios/replay/index.toml")).unwrap();
    let case = index
        .cases
        .iter()
        .find(|case| case.id.0 == "frame/adjacent-actual-trial")
        .unwrap();
    let mut batch = super::super::admit(case, &sources).unwrap().production;
    let mut stepped = batch.clone();
    let budget = MasterTicks(64);
    let output = batch.advance(AdvanceInput::FrameReturn { budget }).unwrap();
    let ProductionOutputs::Frame(completed) = output else {
        panic!("retained frame output required");
    };
    let mut events = Vec::new();
    loop {
        let ProductionOutputs::Frame(step) =
            stepped.advance(AdvanceInput::Frame { budget }).unwrap()
        else {
            panic!("retained frame step required");
        };
        events.extend(step.events);
        if step.progress == frame::Progress::Completed {
            break;
        }
    }
    assert_eq!(completed.progress, frame::Progress::Completed);
    assert!(!events.is_empty() && completed.events == events);
    assert_observation_eq!(batch, stepped);
    let before = batch.save_completed_frame().unwrap();

    let Err(fault) = batch.advance(AdvanceInput::FrameReturn { budget }) else {
        panic!("held return unexpectedly completed");
    };
    assert!(matches!(
        fault,
        ProductionFault::Unsupported(Unsupported::Frame(Dependency::InterruptionPreparation))
    ));
    assert_observation_eq!(batch, stepped);
    assert_observation_eq!(batch.save_completed_frame().unwrap(), before);
    let step_fault = loop {
        match stepped.advance(AdvanceInput::Frame { budget }) {
            Ok(ProductionOutputs::Frame(step)) => {
                assert_eq!(step.progress, frame::Progress::Advanced);
            }
            Ok(_) => panic!("retained frame step required"),
            Err(reached) => break reached,
        }
    };
    assert_eq!(fault, step_fault);
    let (OwnedState::Frame(saved), OwnedState::Frame(partial)) = (batch.state(), stepped.state())
    else {
        panic!("retained frame states required");
    };
    assert!(saved.timing.hardware.now < partial.timing.hardware.now);
}
