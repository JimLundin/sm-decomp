//! A complete ordinary integration call with importer-owned validated content and caller memory.
#![cfg(test)]
use core::num::Wrapping;
use samus_movement::{Frame, Rules, Samus, TravelledDistances};
use serde::{Deserialize, Serialize};
use types::{
    Block, Cell, FiringDirection, Hitbox, Pair, Parity, Pose, Radius, Set, Sprite, Way, World,
    button,
};

/// Importer owns relations and content availability; movement never validates them again.
struct Content {
    upright: Radius,
    eligible: Vec<Hitbox>,
}
impl Content {
    fn imported() -> Self {
        // Content validation is deliberately at this test's importer boundary, not in step.
        let upright = Pair {
            x: Wrapping(5),
            y: Wrapping(21),
        };
        assert!(upright.x.0 > 0 && upright.y.0 > 0);
        Self {
            upright,
            eligible: vec![],
        }
    }
    fn body(&self, _pose: Pose) -> Radius {
        // This call only selects upright still/running, both covered by admitted content.
        self.upright
    }
}
impl World for Content {
    fn block(&self, at: Cell) -> Block {
        if at.y >= 12 { Block::Solid } else { Block::Air }
    }
    fn lets_through(&self, _at: Cell, _way: Way) -> bool {
        false
    }
    fn solid_enemies(&self) -> &[Hitbox] {
        &self.eligible
    }
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Checkpoint {
    samus: Samus,
    travels: TravelledDistances,
    pending_aim: Option<FiringDirection>,
}

#[test]
fn complete_ordinary_call_and_publication_lifetimes() {
    let content = Content::imported();
    // Initialization is only for a fresh start. Mid-play restoration restores the entire value.
    let mut initial = Checkpoint {
        samus: Samus {
            position: Pair {
                x: Wrapping(0),
                y: Wrapping(0),
            },
            pose: Pose {
                body: types::Body::Upright,
                action: types::Action::Still,
                facing: types::Facing::Right,
                aim: types::Aim::Forward,
                gun: types::Gun::Stowed,
                legs: types::Legs::Together,
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
        },
        travels: TravelledDistances::fresh(),
        pending_aim: Some(FiringDirection::UpFacingLeft),
    };
    initial.samus.position = Pair {
        x: Wrapping(0x002B_4000),
        y: Wrapping(0x00AB_FFFF),
    };
    let encoded = toml::to_string(&initial).unwrap();
    let mut caller: Checkpoint = toml::from_str(&encoded).unwrap();
    assert_eq!(caller, initial); // Phase, carry, spin, seat, held-jump and dashing all restored.
    let firing_reads = (caller.travels.travels, caller.pending_aim);
    assert_eq!(firing_reads.1, Some(FiringDirection::UpFacingLeft));
    // Weapons consumes preceding memory before the ordinary interval clears it.
    caller.pending_aim = None;
    caller.travels.refresh();
    let body = |pose| content.body(pose);
    let headings = |_| Some(FiringDirection::Right); // Importer-owned valid cannon content.
    let frame = Frame {
        sprite_before_movement: Sprite::Plain, // Appearance's previously selected figure.
        cinematic_active: false,               // Surrounding mode.
        pad: button::RIGHT,
        pressed: button::RIGHT, // Input's bound held/just-pressed actions.
        items: Set::default(),  // Progress's switched-on equipment, not owned equipment.
        spin_turned_far: false,
        animation_ends: None,   // Appearance's first half.
        elevator_moving: false, // Elevator domain, no external carry guessed here.
        parity: Parity::Even,   // Game frame counter.
        body: &body,
        cannon_heading: &headings, // Validated content.
    };
    let outcome = caller
        .samus
        .step(&frame, &content, &Rules::default())
        .unwrap();
    // Fresh complete outputs are delivered at step completion, not recovered from final pose.
    caller.travels.travels = outcome.travelled;
    if let Some(aim) = outcome.pending_aim {
        caller.pending_aim = Some(aim);
    }
    assert_eq!(caller.pending_aim, None);
    let timing_reads = (
        &outcome.movement_work,
        outcome.movement_skip,
        &outcome.collision_tests,
    );
    assert_ne!(timing_reads.0.as_slice(), []);
    assert_eq!(timing_reads.1, None);
    let world_reads = (&outcome.inside, &outcome.reached);
    assert_ne!(world_reads.0.as_slice(), []);
    let sound_reads = &outcome.touchdown_requests; // Transport preserves order and repeats.
    assert_eq!(sound_reads.as_slice(), []);
    let appearance_reads = (
        outcome.floor_response,
        outcome.touchdown_graphics,
        outcome.vertical_speed_after_moves,
        outcome.handed_over,
    );
    assert_eq!(appearance_reads.0, None);
    let camera_reads = (
        caller.samus.position,
        outcome.placed,
        outcome.rising,
        outcome.skidding,
    );
    assert_eq!(camera_reads.1, None);
    // A surrounding mode skipping the new interval retains caller travel and aim verbatim.
    let retained = (caller.travels, caller.pending_aim);
    assert_eq!(retained.0.travels, outcome.travelled);
    let saved = toml::to_string(&caller).unwrap();
    assert_eq!(toml::from_str::<Checkpoint>(&saved).unwrap(), caller);
}
