use super::*;
use core::num::Wrapping;
use samus_appearance::{Figure, Liquid};
use samus_movement::Outcome;
use types::{Action, Aim, Body, Facing, Gun, Legs, Pair, Pose, Side, Sprite};

const fn playing() -> Figure {
    Figure {
        pose: Pose {
            body: Body::Upright,
            action: Action::Running,
            facing: Facing::Right,
            aim: Aim::Forward,
            gun: Gun::Stowed,
            legs: Legs::Together,
        },
        sprite: Sprite::Plain,
    }
}

fn publications() -> Outcome {
    samus_movement::suspended::SuspendedMovement::new(samus_movement::Samus {
        position: Pair {
            x: Wrapping(0),
            y: Wrapping(0),
        },
        pose: playing().pose,
        run: samus_movement::Run {
            speed: Wrapping(0),
            way: Side::Right,
            carry: samus_movement::Carry::FollowsPad,
            dash: Wrapping(0),
            dashing: Dashing::Off,
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
    .unwrap()
    .outputs
}

fn neighbors() -> AnimationNeighbors {
    AnimationNeighbors {
        run_held: true,
        energy: 17,
        items: Set::default(),
        liquid: Liquid::NONE,
        sounds_waiting: SoundsWaiting::new(3).unwrap(),
    }
}

#[test]
fn current_figure_and_movement_publications_supply_animation() {
    let mut movement = publications();
    movement.standing_animation = StandingAnimation::Restarts;
    movement.wall_test = WallTest::Readied;
    movement.handed_over = true;
    movement.dash_began = true;
    movement.dash_lost = true;
    movement.heading_down = true;
    movement.vertical_speed_after_moves = Wrapping((-65_536_i32).cast_unsigned());
    let mut current_figure = playing();
    current_figure.pose.facing = Facing::Left;
    current_figure.sprite = Sprite::SpaceJump;
    let inputs = neighbors().with_movement(current_figure, &movement);
    assert_eq!(inputs.figure, current_figure);
    assert_eq!(inputs.figure_after, current_figure);
    assert!(inputs.moves[MoveEvent::StandingRestart]);
    assert!(inputs.moves[MoveEvent::WallJumpReadied]);
    assert!(inputs.moves[MoveEvent::HandedOver]);
    assert!(inputs.moves[MoveEvent::DashLost]);
    assert_eq!(inputs.dash, Dash::Began);
    assert!(inputs.heading_down && inputs.run_held);
    assert_eq!(
        inputs.vertical_speed,
        Wrapping((-65_536_i32).cast_unsigned())
    );
    assert_eq!(inputs.energy, 17);
    assert_eq!(inputs.liquid, Liquid::NONE);
    assert_eq!(inputs.sounds_waiting.count(), 3);
}

#[test]
fn fresh_completed_dash_and_moves_replace_prior_publications() {
    let mut movement = publications();
    movement.dashing_after_moves = Dashing::On;
    let inputs = neighbors().with_movement(playing(), &movement);
    assert_eq!(inputs.dash, Dash::On);
    assert_eq!(inputs.moves, Set::default());
    movement.dashing_after_moves = Dashing::Off;
    let inputs = neighbors().with_movement(playing(), &movement);
    assert_eq!(inputs.dash, Dash::Off);
}
