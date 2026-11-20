use super::*;
use core::num::Wrapping;
use data::spritemap::{SpriteAttributes, SpriteSize, Spritemap, SpritemapEntry};
use samus_appearance::{Frames, LapsLeft, Playhead, SuspendedAppearance};

fn figure() -> Figure {
    Figure {
        pose: types::Pose {
            body: types::Body::Upright,
            action: types::Action::Running,
            facing: types::Facing::Right,
            aim: types::Aim::Forward,
            gun: types::Gun::Stowed,
            legs: types::Legs::Together,
        },
        sprite: types::Sprite::Plain,
    }
}

fn movement(x: u16) -> Movement {
    Movement::new(samus_movement::Samus {
        pose: figure().pose,
        position: Pair {
            x: types::px(Wrapping(x)),
            y: types::px(Wrapping(80)),
        },
        run: samus_movement::Run {
            speed: Wrapping(0),
            way: types::Side::Right,
            carry: samus_movement::Carry::FollowsPad,
            dash: Wrapping(0),
            dashing: samus_movement::Dashing::Off,
        },
        vertical: samus_movement::Vertical {
            speed: Wrapping(0),
            phase: samus_movement::Phase::Ground,
            bounces: samus_movement::Bounces::None,
        },
        spin: samus_movement::Spin::Off,
        seat: samus_movement::Seat::Unseated,
        jump_held: samus_movement::JumpHeld::NONE,
    })
}

fn appearance_owner(step: Step) -> Work {
    let mut work = Work::new(SuspendedAppearance::new(
        samus_appearance::State {
            playhead: Playhead {
                step,
                time_left: Frames(4),
            },
            stage: types::BoostStage::Stage0,
            laps_left: LapsLeft::WithoutBooster,
        },
        None,
        Vec::new(),
    ));
    super::super::appearance_stage::choose_read(
        &mut work,
        AppearanceRead::Draw {
            previous: types::Action::Running,
            current: figure().pose.action,
        },
    )
    .unwrap();
    super::super::appearance_stage::complete(&mut work).unwrap();
    work
}

fn builder() -> SpriteBuilderOrigin {
    let mut backing = [0; 544];
    for bytes in backing.get_mut(..512).unwrap().as_chunks_mut::<4>().0 {
        *bytes = [9, 20, 23, 30];
    }
    SpriteBuilderOrigin {
        backing,
        cursor: 0,
        prepared_flags: 7,
        prepared_top: 3,
        prepared_bottom: 4,
    }
}

fn premises() -> Premises {
    Premises {
        camera: [90, 50],
        shake: [-1, 2],
        normal_visible: true,
        no_additional_selected_pieces: true,
        unselected_draws_empty: true,
    }
}

fn content() -> Content {
    let sheet = SpriteSheet {
        sprites: vec![
            Spritemap {
                entries: vec![SpritemapEntry {
                    x: -2,
                    y: 3,
                    size: SpriteSize::Large,
                    attributes: SpriteAttributes::from_word(0x0045),
                }],
            },
            Spritemap {
                entries: vec![SpritemapEntry {
                    x: 4,
                    y: 255,
                    size: SpriteSize::Small,
                    attributes: SpriteAttributes::from_word(0x0067),
                }],
            },
        ],
        uploads: vec![samus::TileUpload {
            first: vec![1, 2],
            second: vec![3, 4],
        }],
    };
    let picture = |step, top, bottom| Picture {
        figure: figure(),
        step,
        frame: samus::Frame {
            hold: samus::Hold::Shows(4),
            top,
            bottom,
            chest: false,
            origin_y: -4,
            tiles: Some(samus::FrameTiles {
                top: samus::TilesId(0),
                bottom: None,
            }),
        },
    };
    Content::new(
        sheet,
        vec![
            picture(Step::FIRST, Some(samus::SpriteId(1)), None),
            picture(Step(1), Some(samus::SpriteId(0)), Some(samus::SpriteId(1))),
        ],
    )
    .unwrap()
}

fn restore<T: canonical::Canonical>(value: T) -> T {
    let bytes = canonical::to_bytes(&value);
    drop(value);
    canonical::from_bytes(&bytes).unwrap()
}

#[test]
fn selected_actual_position_and_playing_step_produce_retained_complete_oam_once() {
    let content = content();
    let movement = movement(100);
    let appearance = appearance_owner(Step(1));
    let inputs = Inputs {
        movement: &movement,
        appearance: &appearance,
        contacts: &Contacts::default(),
        figure: figure(),
        premises: premises(),
    };
    let mut builder = builder();
    let mut stage = Stage::default();
    let receipt = stage
        .choose(&mut builder, &content, &inputs)
        .unwrap()
        .clone();
    assert_eq!(receipt.centre, [100, 80]);
    assert_eq!(receipt.shown_step, ShownStep(1));
    assert_eq!(receipt.pieces, 2);
    assert_eq!(&builder.backing[..8], &[7, 31, 0x45, 0, 13, 27, 0x67, 0]);
    assert_eq!(builder.backing[512], 2);
    assert_eq!(builder.cursor, 8);
    assert_eq!(content.uploads(&receipt).unwrap()[0].1.first, [1, 2]);
    let chosen = builder.clone();
    assert_eq!(
        stage.choose(&mut builder, &content, &inputs),
        Err(Fault::Order)
    );
    assert_eq!(builder, chosen);
    let mut stage = restore(stage);
    let mut builder = restore(builder);
    assert_eq!(stage.receipt(), Some(&receipt));
    stage.bind(crate::production::timing::WorkId(91)).unwrap();
    assert_eq!(
        stage.ready(crate::production::timing::WorkId(91), &builder),
        Err(Fault::Order)
    );
    let mut stage = restore(stage);
    stage
        .complete(crate::production::timing::WorkId(91))
        .unwrap();
    assert_eq!(
        stage.complete(crate::production::timing::WorkId(91)),
        Err(Fault::Order)
    );
    stage
        .finalise(&mut builder, crate::production::timing::WorkId(92))
        .unwrap();
    let mut builder = restore(builder);
    let mut stage = restore(stage);
    assert_eq!(
        stage.ready(crate::production::timing::WorkId(93), &builder),
        Err(Fault::Completion)
    );
    let output = stage
        .ready(crate::production::timing::WorkId(92), &builder)
        .unwrap();
    assert_eq!(output.len(), 544);
    assert_eq!(&output[..8], &[7, 31, 0x45, 0, 13, 27, 0x67, 0]);
    assert_eq!(&output[8..12], &[9, crate::exact::oam::HIDDEN_Y, 23, 30]);
    assert_eq!(output[512], 2);
    assert_eq!(builder.cursor, 0);
    assert_eq!(
        (
            builder.prepared_flags,
            builder.prepared_top,
            builder.prepared_bottom
        ),
        (7, 3, 4)
    );
    assert_eq!(
        stage.ready(crate::production::timing::WorkId(92), &builder),
        Err(Fault::Order)
    );
    assert_eq!(
        stage.finalise(&mut builder, crate::production::timing::WorkId(92)),
        Err(Fault::Order)
    );
}

#[test]
fn changed_current_movement_and_actual_playing_step_change_selected_exact_bytes() {
    let content = content();
    let appearance = appearance_owner(Step::FIRST);
    let left = movement(100);
    let right = movement(107);
    let draw = |movement: &Movement| {
        let mut builder = builder();
        Stage::default()
            .choose(
                &mut builder,
                &content,
                &Inputs {
                    movement,
                    appearance: &appearance,
                    contacts: &Contacts::default(),
                    figure: figure(),
                    premises: premises(),
                },
            )
            .unwrap();
        builder
    };
    let first = draw(&left);
    let later = draw(&right);
    assert_eq!(first.cursor, 4);
    assert_eq!(first.backing[2], 0x67);
    assert_eq!(later.backing[0], first.backing[0] + 7);
    assert_eq!(later.backing[2], first.backing[2]);
}

#[test]
fn missing_current_drawing_and_bad_capacity_reject_without_mutating_builder() {
    let content = content();
    let movement = movement(100);
    let mut appearance = appearance_owner(Step(1));
    let mut builder = builder();
    let initial = builder.clone();
    let mut stage = Stage::default();
    let mut premise = premises();
    premise.no_additional_selected_pieces = false;
    assert_eq!(
        stage.choose(
            &mut builder,
            &content,
            &Inputs {
                movement: &movement,
                appearance: &appearance,
                contacts: &Contacts::default(),
                figure: figure(),
                premises: premise
            }
        ),
        Err(Fault::Unsupported)
    );
    assert_eq!(builder, initial);
    assert_eq!(stage, Stage::Available);
    appearance.model.0.steps.clear();
    appearance.model.0.consumed = 0;
    assert_eq!(
        stage.choose(
            &mut builder,
            &content,
            &Inputs {
                movement: &movement,
                appearance: &appearance,
                contacts: &Contacts::default(),
                figure: figure(),
                premises: premises()
            }
        ),
        Err(Fault::Unsupported)
    );
    assert_eq!(builder, initial);
    let appearance = appearance_owner(Step(1));
    builder.cursor = 3;
    let invalid = builder.clone();
    assert_eq!(
        stage.choose(
            &mut builder,
            &content,
            &Inputs {
                movement: &movement,
                appearance: &appearance,
                contacts: &Contacts::default(),
                figure: figure(),
                premises: premises()
            }
        ),
        Err(Fault::Capacity)
    );
    assert_eq!(builder, invalid);
}

fn appearance_with_contact() -> Work {
    let mut appearance = appearance_owner(Step::FIRST);
    appearance.model.0.state.playhead.time_left = Frames::LAST;
    let clip = samus_appearance::Clip {
        instructions: vec![samus_appearance::Instruction::Show(
            samus_appearance::Hold::new(4).unwrap(),
        )],
        contacts: std::iter::once(Step::FIRST).collect(),
        kick: None,
        fast_fall: None,
        rising: None,
    };
    let playhead = appearance.model.0.state.playhead;
    super::super::appearance_stage::choose_read(
        &mut appearance,
        AppearanceRead::Contact {
            figure: figure(),
            playhead,
            stage: types::BoostStage::Stage0,
            clip,
            suppressors: types::Set::default(),
        },
    )
    .unwrap();
    let outputs = super::super::appearance_stage::complete(&mut appearance).unwrap();
    assert_eq!(outputs.graphics.len(), 1);
    super::super::appearance_stage::choose_read(
        &mut appearance,
        AppearanceRead::Draw {
            previous: figure().pose.action,
            current: figure().pose.action,
        },
    )
    .unwrap();
    super::super::appearance_stage::complete(&mut appearance).unwrap();
    appearance
}

#[test]
fn absent_neighbours_cannot_suppress_a_selected_graphical_contact() {
    let content = content();
    let movement = movement(100);
    let appearance = appearance_with_contact();
    let mut builder = builder();
    let initial = builder.clone();
    assert_eq!(
        Stage::default().choose(
            &mut builder,
            &content,
            &Inputs {
                movement: &movement,
                appearance: &appearance,
                contacts: &Contacts::default(),
                figure: figure(),
                premises: premises(),
            }
        ),
        Err(Fault::GraphicalContact)
    );
    assert_eq!(builder, initial);

    let selected = appearance.clone();
    let mut contacts = Contacts::default();
    assert_eq!(
        choose_contacts(
            &mut contacts,
            &appearance,
            Part2Premises { dry_ceres: false },
            crate::production::timing::WorkId(94)
        ),
        Err(Fault::GraphicalContact)
    );
    assert_eq!(contacts, Contacts::default());
    let actual = choose_contacts(
        &mut contacts,
        &appearance,
        Part2Premises { dry_ceres: true },
        crate::production::timing::WorkId(94),
    )
    .unwrap();
    assert_eq!((actual.first, actual.end), (0, 1));
    assert_eq!(appearance, selected);
    let mut contacts = restore(contacts);
    assert_eq!(
        consume_contacts(&mut contacts, crate::production::timing::WorkId(95)),
        Err(Fault::Completion)
    );
    assert_eq!(
        consume_contacts(&mut contacts, crate::production::timing::WorkId(94)),
        Ok(actual)
    );
    assert_eq!(
        consume_contacts(&mut contacts, crate::production::timing::WorkId(94)),
        Err(Fault::Order)
    );
    assert_eq!(appearance.model.0.graphics.len(), 1);
    assert_eq!(contacts.consumed(), 1);
    assert_eq!(contacts.validate(&appearance), Ok(()));
    Stage::default()
        .choose(
            &mut builder,
            &content,
            &Inputs {
                movement: &movement,
                appearance: &appearance,
                contacts: &contacts,
                figure: figure(),
                premises: premises(),
            },
        )
        .unwrap();
}
