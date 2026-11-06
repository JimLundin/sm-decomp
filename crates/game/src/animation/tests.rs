use data::movement::{HorizontalSpeeds, VerticalSpeeds};
use data::pose::animation::{AnimationDelays, DelayList};
use data::pose::transition::PoseTransitions;
use data::pose::{MovementType, PoseFacing, Poses};

use super::*;
use crate::pose;
use crate::state::samus::{Animation, PoseRecord};

/// The turn's pose: turning on the ground, its list ending in an auto-jump transition.
const TURN: PoseId = PoseId(0x25);
/// The pose the turn ends in: standing facing left.
const STANDING_LEFT: PoseId = PoseId(2);
/// A pose whose list loops.
const LOOPING: PoseId = PoseId(1);

/// Pose definition bytes: facing, movement type, next pose, shot, then four more.
fn definition(facing: u8, movement: u8) -> [u8; 8] {
    [facing, movement, 0xFF, 2, 0, 0, 0x15, 0]
}

/// Every pose up to the turn standing facing right but 2 (standing facing left) and the turn
/// (turning on the ground facing left); pose 1 loops on 3 and 4, pose 2 shows 5 frames, the
/// turn's list is the original's: 2, 2, 2, then the transition to pose 2.
fn content() -> Content {
    let bytes: Vec<u8> = (0..=TURN.0)
        .map(|pose| match PoseId(pose) {
            STANDING_LEFT => definition(4, 0),
            TURN => definition(4, 0x0E),
            _ => definition(8, 0),
        })
        .collect::<Vec<_>>()
        .concat();
    let list = |bytes: &[u8]| DelayList::decode(bytes).unwrap();
    let lists = (0..=TURN.0)
        .map(|pose| match PoseId(pose) {
            LOOPING => list(&[0x03, 0x04, 0xFF]),
            STANDING_LEFT => list(&[0x05, 0xFF]),
            TURN => list(&[0x02, 0x02, 0x02, 0xF8, 0x02]),
            _ => list(&[0x08, 0xFF]),
        })
        .collect();
    Content {
        poses: Poses::decode(&bytes).unwrap(),
        transitions: PoseTransitions::new(Vec::new()),
        speeds: HorizontalSpeeds::default(),
        vertical: VerticalSpeeds::default(),
        delays: AnimationDelays::new(lists, list(&[0x01, 0xFF])),
        slopes: data::level::slope::Slopes::default(),
    }
}

fn samus(pose: PoseId, movement: MovementType, frame: u16, timer: u16) -> Samus {
    Samus {
        pose,
        previous: PoseRecord {
            pose,
            facing: PoseFacing::Left,
            movement,
        },
        movement: Movement::from(movement),
        animation: Animation {
            frame: AnimationFrame(frame),
            timer: AnimationTimer(timer),
            ..Animation::default()
        },
        ..Samus::default()
    }
}

/// One frame's animation and pose update, with no request from input.
fn frame(samus: &mut Samus, content: &Content) {
    let requests = PoseRequests::default();
    animate(samus, &Controller::default(), requests, content);
    pose::change(samus, requests, pose::Collisions::default(), content);
}

#[test]
fn the_turns_list_ends_it_in_standing_on_its_last_frame() {
    let content = content();
    let mut samus = samus(TURN, MovementType::TurningAroundOnGround, 0, 2);
    // Frames 0 to 2 show two frames each: the turn holds for five more frames.
    for _ in 0..5 {
        frame(&mut samus, &content);
        assert_eq!(samus.pose, TURN);
        assert_eq!(samus.super_special, SuperSpecial::default());
    }
    assert_eq!(samus.animation.frame, AnimationFrame(2));
    assert_eq!(samus.animation.timer, AnimationTimer(1));
    // The sixth reaches the transition: standing, from the new pose's first frame.
    frame(&mut samus, &content);
    assert_eq!(
        samus.super_special,
        SuperSpecial {
            pose: Some(STANDING_LEFT),
            command: SuperSpecialCommand::TransitionAnimationFinished,
        }
    );
    assert_eq!(samus.pose, STANDING_LEFT);
    assert_eq!(samus.movement.kind(), MovementType::Standing);
    assert_eq!(samus.animation.frame, AnimationFrame(0));
    assert_eq!(samus.animation.timer, AnimationTimer(5));
    assert_eq!(samus.previous.pose, STANDING_LEFT);
    assert_eq!(samus.last_different.pose, TURN);
}

#[test]
fn a_looping_list_goes_back_to_its_start() {
    let content = content();
    let mut samus = samus(LOOPING, MovementType::Standing, 0, 1);
    let mut seen = Vec::new();
    for _ in 0..8 {
        frame(&mut samus, &content);
        seen.push((samus.animation.frame.0, samus.animation.timer.0));
    }
    assert_eq!(
        seen,
        [
            (1, 4),
            (1, 3),
            (1, 2),
            (1, 1),
            (0, 3),
            (0, 2),
            (0, 1),
            (1, 4)
        ]
    );
    assert_eq!(samus.pose, LOOPING);
}

#[test]
fn a_timer_counted_past_zero_still_advances_the_frame() {
    let content = content();
    let mut samus = samus(LOOPING, MovementType::Standing, 0, 0);
    frame(&mut samus, &content);
    assert_eq!(samus.animation.frame, AnimationFrame(1));
    assert_eq!(samus.animation.timer, AnimationTimer(4));
}
