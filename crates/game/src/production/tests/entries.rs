use super::super::*;
use crate::state::owned::OwnedState;
use core::num::Wrapping;
use types::Pair;

#[test]
fn camera_and_appearance_complete_state_pins_hold() {
    assert_eq!(
        canonical::model::pin_holds::<crate::state::owned::CameraPin>(),
        Ok(())
    );
    assert_eq!(
        canonical::model::pin_holds::<crate::state::owned::AppearancePin>(),
        Ok(())
    );
}

fn blocks() -> Production {
    let state = world_blocks::State {
        tiles: world_blocks::Grid::new(vec![vec![world_blocks::Tile {
            kind: world_blocks::CellKind::Air,
            mark: world_blocks::Mark::Unmarked,
            look: types::Look::default(),
            inside: world_blocks::Inside::HoldsNoPlace,
        }]])
        .unwrap(),
        places: world_blocks::Places::default(),
        screens: world_blocks::Grid::new(vec![vec![types::ScreenKind::Open]]).unwrap(),
    };
    Production::admit(
        ProductionSeed::Blocks(Box::new(state)),
        ProductionContent::Blocks,
        ProductionRules::Blocks(BlockRules::new(world_blocks::Rules::reference())),
    )
    .unwrap()
}

#[test]
fn block_start_rolls_back_prior_placement_and_reports_missing_complete_codec() {
    let mut production = blocks();
    let before = production.state().clone();
    let outside = Pair { x: 1, y: 0 };
    let placements = [
        world_blocks::Placement::OtherPart,
        world_blocks::Placement::Scroll {
            cell: outside,
            changes: Vec::new(),
        },
    ];
    assert_eq!(
        production.advance(AdvanceInput::Blocks(BlocksInput::StartRoom(&placements))),
        Err(ProductionFault::Blocks(world_blocks::OutsideTheRoom(
            outside
        )))
    );
    assert_eq!(production.state(), &before);
    assert_eq!(
        production.save(),
        Err(SaveFault::Unsupported(Scope::Blocks))
    );
    assert_eq!(
        production.advance(AdvanceInput::Blocks(BlocksInput::StartRoom(&[
            world_blocks::Placement::OtherPart
        ]))),
        Ok(ProductionOutputs::Blocks(BlocksOutputs::RoomStarted))
    );
    let OwnedState::Blocks(state) = production.state() else {
        panic!("blocks scope")
    };
    assert_eq!(
        state
            .places
            .0
            .iter()
            .filter(|place| **place == world_blocks::Place::OtherPart)
            .count(),
        1
    );
}

#[test]
fn progress_carries_ordered_world_reports_without_a_partial_save() {
    let mut production = Production::admit(
        ProductionSeed::Progress(Box::default()),
        ProductionContent::Progress,
        ProductionRules::Progress(run_progress::Rules::default()),
    )
    .unwrap();
    for reports in [
        &[run_progress::Report::ZebetiteDestroyed][..],
        &[run_progress::Report::ZebetiteDestroyed][..],
    ] {
        assert_eq!(
            production.advance(AdvanceInput::Progress(reports)),
            Ok(ProductionOutputs::Progress)
        );
    }
    let OwnedState::Progress(state) = production.state() else {
        panic!("progress scope")
    };
    assert_eq!(state.zebetites, Wrapping(2));
    assert_eq!(
        production.save(),
        Err(SaveFault::Unsupported(Scope::Progress))
    );
}

#[test]
fn passage_carries_the_complete_scene_without_inventing_work_completion() {
    let scene = world_doors::Scene {
        her: Pair {
            x: Wrapping(123),
            y: Wrapping(456),
        },
        seen: Pair {
            x: Wrapping(111),
            y: Wrapping(222),
        },
        camera: Pair {
            x: Wrapping(333),
            y: Wrapping(444),
        },
        palette: world_doors::Palette {
            screen: [types::Colour::BLACK; world_doors::COLOURS],
            goal: [types::Colour::from_channels([1, 2, 3]); world_doors::COLOURS],
        },
    };
    let mut production = Production::admit(
        ProductionSeed::Doors(Box::new(PassageState {
            doors: world_doors::Doors::Play,
            scene,
        })),
        ProductionContent::Doors,
        ProductionRules::Doors(world_doors::Rules::default()),
    )
    .unwrap();
    let mut inputs = world_doors::Inputs {
        touched: true,
        turn_ready: false,
        publishes: false,
        sounds_sent: false,
        music_ready: false,
        timer_running: false,
        items_dark: false,
        doorway: None,
        groups: [world_doors::ColourGroup::Other; world_doors::COLOURS],
    };
    let ProductionOutputs::Doors(first) = production.advance(AdvanceInput::Doors(&inputs)).unwrap()
    else {
        panic!("passage output")
    };
    assert!(!first.report.passing);
    assert_eq!(first.scene, scene);
    inputs.touched = false;
    let ProductionOutputs::Doors(next) = production.advance(AdvanceInput::Doors(&inputs)).unwrap()
    else {
        panic!("passage output")
    };
    assert!(next.report.passing);
    assert_eq!(next.report.finishing, world_doors::Finishing::Overrun);
    assert_eq!(next.scene, scene);
    assert_eq!(next.doors, first.doors);
    assert_eq!(production.save(), Err(SaveFault::Unsupported(Scope::Doors)));
    let before = production.state().clone();
    assert_eq!(
        production.advance(AdvanceInput::Missing(Scope::Doors)),
        Err(ProductionFault::Unsupported(Unsupported::MissingInput(
            Scope::Doors
        )))
    );
    assert_eq!(production.state(), &before);
}
