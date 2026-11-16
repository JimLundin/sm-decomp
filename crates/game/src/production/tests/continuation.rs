use super::super::*;
use crate::state::owned::OwnedState;
use core::num::Wrapping;
use types::Pair;

#[test]
fn stopped_camera_carries_fractional_receipt_through_checkpoint() {
    let position = Pair {
        x: Wrapping(128 * 65536),
        y: Wrapping(128 * 65536),
    };
    let camera = world_camera::Camera::restore(
        world_camera::ViewPosition(Pair {
            x: Wrapping(0),
            y: Wrapping(0),
        }),
        world_camera::RememberedPosition(position),
        types::VerticalFraction(0xBEEF),
    );
    let content = ProductionContent::Camera;
    let rules = ProductionRules::Camera(CameraRules::new(world_camera::Rules::REFERENCE));
    let mut production = Production::admit(
        ProductionSeed::Camera(camera),
        content.clone(),
        rules.clone(),
    )
    .unwrap();
    let room = world_camera::Room::new(
        vec![types::ScreenKind::Open; 4],
        Pair { x: 2, y: 2 },
        world_camera::Heights {
            while_not_rising: Wrapping(112),
            while_rising: Wrapping(128),
        },
    )
    .unwrap();
    let mut frame = world_camera::Frame {
        samus: world_camera::Samus {
            position: Pair {
                y: Wrapping((128 * 65536) + 0x8000),
                ..position
            },
            placed: None,
            facing: types::Facing::Right,
            backwards: false,
            rising: false,
        },
        room: &room,
        lead: world_camera::LeadSetting::Ahead,
        time_stopped: true,
    };
    let ProductionOutputs::Camera(first) =
        production.advance(AdvanceInput::Camera(&frame)).unwrap()
    else {
        panic!("camera output")
    };
    assert_eq!(first.position(), camera.position());
    assert_eq!(first.vertical_fraction(), types::VerticalFraction(0x8000));
    assert_eq!(first.seen(), frame.samus.position);
    let mut restored = Production::restore(&production.save().unwrap(), content, rules).unwrap();
    frame.samus.position.y = Wrapping((128 * 65536) + 0xC000);
    let expected = production.advance(AdvanceInput::Camera(&frame)).unwrap();
    assert_eq!(restored.advance(AdvanceInput::Camera(&frame)), Ok(expected));
    assert_eq!(restored.state(), production.state());
    let OwnedState::Camera(held) = restored.state() else {
        panic!("camera scope")
    };
    assert_eq!(held.0.vertical_fraction(), types::VerticalFraction(0x4000));
}

fn animation() -> (
    Production,
    ProductionContent,
    ProductionRules,
    samus_appearance::Inputs,
) {
    use samus_appearance as animation;
    let figure = animation::Figure {
        pose: super::fixtures::samus().pose,
        sprite: types::Sprite::Plain,
    };
    let content = ProductionContent::Appearance(
        AppearanceContent::new(
            vec![FigureClip {
                figure,
                clip: animation::Clip {
                    instructions: vec![
                        animation::Instruction::Show(animation::Hold::new(4).unwrap()),
                        animation::Instruction::Show(animation::Hold::new(6).unwrap()),
                    ],
                    kick: None,
                    fast_fall: None,
                    rising: None,
                    contacts: std::collections::BTreeSet::new(),
                },
            }],
            animation::Paces {
                plain: animation::Pace(Vec::new()),
                boosted: enum_map::EnumMap::from_fn(|_| animation::Pace(Vec::new())),
            },
        )
        .unwrap(),
    );
    let rules = ProductionRules::Appearance(animation::Rules::VANILLA);
    let state = animation::State {
        playhead: animation::Playhead {
            step: animation::Step::FIRST,
            time_left: animation::Frames(3),
        },
        stage: types::BoostStage::Stage0,
        laps_left: animation::LapsLeft::WithoutBooster,
    };
    let production = Production::admit(
        ProductionSeed::Appearance(state),
        content.clone(),
        rules.clone(),
    )
    .unwrap();
    let inputs = animation::Inputs {
        figure,
        figure_after: figure,
        moves: types::Set::default(),
        dash: animation::Dash::Off,
        run_held: false,
        heading_down: true,
        vertical_speed: Wrapping(0),
        energy: 99,
        items: types::Set::default(),
        liquid: animation::Liquid::NONE,
        sounds_waiting: animation::SoundsWaiting::NONE,
    };
    (production, content, rules, inputs)
}

#[test]
fn animation_carries_hold_through_restore_and_missing_clip_refuses_without_advancing() {
    let (mut production, content, rules, inputs) = animation();
    assert_eq!(
        production.advance(AdvanceInput::Appearance(&inputs)),
        Ok(ProductionOutputs::Appearance(None))
    );
    let saved = production.save().unwrap();
    let OwnedState::Appearance(held) = production.state() else {
        panic!("appearance scope")
    };
    assert_eq!(held.0.playhead.time_left, samus_appearance::Frames(2));
    let mut absent = inputs;
    absent.figure_after.sprite = types::Sprite::ScrewAttack;
    assert_eq!(
        production.advance(AdvanceInput::Appearance(&absent)),
        Err(ProductionFault::Appearance(ClipFault::Missing(
            absent.figure_after
        )))
    );
    assert_eq!(production.save().unwrap(), saved);
    let mut restored = Production::restore(&saved, content, rules).unwrap();
    for _ in 0..2 {
        let expected = production
            .advance(AdvanceInput::Appearance(&inputs))
            .unwrap();
        assert_eq!(
            restored.advance(AdvanceInput::Appearance(&inputs)),
            Ok(expected)
        );
    }
    assert_eq!(restored.state(), production.state());
    let OwnedState::Appearance(held) = restored.state() else {
        panic!("appearance scope")
    };
    assert_eq!(held.0.playhead.step, samus_appearance::Step::SECOND);
    assert_eq!(held.0.playhead.time_left, samus_appearance::Frames(6));
}
