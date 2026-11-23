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
fn ceres_run_preserves_opening_sound_and_actual_shake() {
    let rules = boss_ceres_ridley::Rules::default();
    let door = boss_ceres_ridley::CeresDoor {
        kind: boss_ceres_ridley::DoorKind::Door,
        facing: types::Side::Left,
        position: Pair {
            x: Wrapping(128),
            y: Wrapping(128),
        },
        phase: boss_ceres_ridley::Phase::Closed {
            wait: boss_ceres_ridley::Wait::new(1).unwrap(),
        },
    };
    let mut production = Production::admit(
        ProductionSeed::Ceres(door),
        ProductionContent::Ceres,
        ProductionRules::Ceres(rules),
    )
    .unwrap();
    let mut inputs = boss_ceres_ridley::Inputs {
        her: door.position,
        stage: boss_ceres_ridley::Stage::Escape,
        random: 0,
        shake_frames_left: 0,
    };
    let ProductionOutputs::Ceres(output) =
        production.advance(AdvanceInput::Ceres(&inputs)).unwrap()
    else {
        panic!("ceres output")
    };
    assert_eq!(output.sound, Some(boss_ceres_ridley::OpeningSound));
    assert_eq!(
        output.shake,
        Some(boss_ceres_ridley::Shake {
            strength: 3,
            length: 4
        })
    );
    assert_eq!(production.state(), &OwnedState::Ceres(output.door));
    inputs.shake_frames_left = 4;
    let ProductionOutputs::Ceres(next) = production.advance(AdvanceInput::Ceres(&inputs)).unwrap()
    else {
        panic!("ceres output")
    };
    assert_eq!(next.sound, None);
    assert_eq!(next.shake, None);
    assert_eq!(production.save(), Err(SaveFault::Unsupported(Scope::Ceres)));
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
        work_done: false,
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

#[test]
fn hud_boss_entry_keeps_selection_and_refuses_an_incomplete_checkpoint() {
    let selected = interface_hud::Selection::OneShot(interface_hud::SelectableItem::Ammo(
        interface_hud::Ammo::Missiles,
    ));
    let state = interface_hud::HudState {
        countdown: interface_hud::Countdown::Dormant,
        grid: interface_hud::HudGrid::filled(interface_hud::Tile::new(
            types::Look::default(),
            interface_hud::ColourSet::new(0).unwrap(),
        )),
        shown: interface_hud::Shown::default(),
        selection: selected,
        explored: interface_hud::MapCellSet::default(),
        mini_map: interface_hud::MiniMapPower::On,
    };
    let mut production = Production::admit(
        ProductionSeed::Hud(Box::new(state)),
        ProductionContent::Hud(Box::default()),
        ProductionRules::Hud(HudRules::new(interface_hud::Rules::default())),
    )
    .unwrap();
    assert_eq!(
        production.advance(AdvanceInput::Hud(HudInput::Boss {
            boss: interface_hud::Boss::MotherBrain,
            room_cell: Pair { x: 2, y: 3 }
        })),
        Ok(ProductionOutputs::Hud(Vec::new()))
    );
    let OwnedState::Hud(state) = production.state() else {
        panic!("hud scope")
    };
    assert_eq!(state.selection, selected);
    assert_eq!(state.mini_map, interface_hud::MiniMapPower::Off);
    assert_eq!(production.save(), Err(SaveFault::Unsupported(Scope::Hud)));
}

#[test]
fn hud_play_selects_once_and_preserves_both_ordered_events() {
    use interface_hud as hud;
    let state = hud::HudState {
        countdown: interface_hud::Countdown::Dormant,
        grid: hud::HudGrid::filled(hud::Tile::new(
            types::Look::default(),
            hud::ColourSet::new(0).unwrap(),
        )),
        shown: hud::Shown::default(),
        selection: hud::Selection::None,
        explored: hud::MapCellSet::default(),
        mini_map: hud::MiniMapPower::On,
    };
    let mut production = Production::admit(
        ProductionSeed::Hud(Box::new(state)),
        ProductionContent::Hud(Box::default()),
        ProductionRules::Hud(HudRules::new(hud::Rules::default())),
    )
    .unwrap();
    let map = hud::HerMap::default();
    let inputs = hud::FrameInputs {
        frame: hud::FrameCount(0),
        status: hud::Status {
            energy: hud::Energy {
                now: 99,
                capacity: 99,
            },
            reserve: hud::Reserve {
                energy: 0,
                mode: hud::ReserveMode::Manual,
            },
            ammunition: hud::Ammunition::new(enum_map::EnumMap::from_fn(|_| hud::Stock {
                count: 10,
                capacity: 10,
            }))
            .unwrap(),
            equipped: hud::Equipped::default(),
        },
        controls: hud::Controls {
            select: hud::SelectButton::Pressed,
            cancel: hud::CancelButton::NotPressed,
            can_switch: true,
            muted_by: None,
        },
        dropped_before_status_bar: hud::Dropped::Nothing,
        dropped_after_status_bar: hud::Dropped::Nothing,
        collected: None,
        mother_brain_falls: false,
        mini_map: hud::MiniMapInputs {
            runs: false,
            turned_on: false,
            position: Pair { x: 0, y: 0 },
            room: hud::Room {
                cell: Pair { x: 0, y: 0 },
                blocks: Pair { x: 1, y: 1 },
            },
            map: &map,
            map_collected: false,
        },
    };
    let missiles = hud::SelectableItem::Ammo(hud::Ammo::Missiles);
    assert_eq!(
        production.advance(AdvanceInput::Hud(HudInput::Play(&inputs))),
        Ok(ProductionOutputs::Hud(vec![
            hud::HudEvent::Switched {
                item: Some(missiles)
            },
            hud::HudEvent::Click
        ]))
    );
    let OwnedState::Hud(state) = production.state() else {
        panic!("hud scope")
    };
    assert_eq!(state.selection, hud::Selection::Item(missiles));
}
