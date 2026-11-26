//! A complete ordinary integration call with importer-owned validated content and caller memory,
//! and the active frame that ends a passage (`Samus::react_after_draw`).
#![cfg(test)]
use core::num::Wrapping;
use samus_movement::{
    DrawnJumpHistory, Frame, FrameBefore, JumpButton, JumpHeld, Rules, Samus, TravelledDistances,
};
use serde::{Deserialize, Serialize};
use types::{
    Block, Buttons, Cell, FiringDirection, Hitbox, Pair, Parity, Pose, Radius, Set, Sprite, Way,
    World, button,
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
            choices: samus_movement::Choices {
                latest: samus_movement::Chosen::Other,
                before: samus_movement::Chosen::Other,
            },
            seat: samus_movement::Seat::Unseated,
            jump_held: samus_movement::JumpHeld::NONE,
            knockback: samus_movement::Knockback::spent(types::Side::Right),
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
    assert_eq!(caller, initial); // Phase, carry, choices, seat, held-jump and dashing all restored.
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
    // Her moves, then her reaction once the hitter judges the hit (none) from where they end.
    let rules = Rules::default();
    let moves = caller.samus.moves(&frame, &content, &rules).unwrap();
    assert_eq!(moves.position_after_moves().x, Wrapping(0x002B_4000));
    let outcome = caller.samus.react(moves, None);
    assert!(!outcome.push_started);
    // The whole frame's outputs come out of the two steps, not recovered from her final pose.
    caller.travels.travels = outcome.travelled;
    if let Some(aim) = outcome.pending_aim {
        caller.pending_aim = Some(aim);
    }
    assert_eq!(caller.pending_aim, None);
    let world_reads = (&outcome.inside, &outcome.reached);
    assert_ne!(world_reads.0.as_slice(), []);
    assert_ne!(world_reads.1.as_slice(), []);
    // Standing with right pressed she runs: her moves leave her where she stood, and the aim
    // change's step then takes her 1 px on, which the mini-map sees a frame later.
    let minimap_reads = outcome.position_after_moves;
    assert_eq!(minimap_reads.x, Wrapping(0x002B_4000));
    assert_eq!(caller.samus.position.x, Wrapping(0x002C_4000));
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

/// Her standing still on the floor of `Content`, with jump held through as given.
fn standing(jump_held: JumpHeld) -> Samus {
    Samus {
        position: Pair {
            x: Wrapping(0x002B_4000),
            y: Wrapping(0x00AB_FFFF),
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
        choices: samus_movement::Choices {
            latest: samus_movement::Chosen::Other,
            before: samus_movement::Chosen::Other,
        },
        seat: samus_movement::Seat::Unseated,
        jump_held,
        knockback: samus_movement::Knockback::spent(types::Side::Right),
    }
}

/// An invented active frame after a passage's draws: what she starts with, the pad, and the
/// jump of the passage's last history-writing draw.
struct AfterPassage {
    /// What the case shows.
    name: &'static str,
    /// Jump held through at the frame's start.
    start: JumpHeld,
    /// The buttons held.
    pad: Buttons,
    /// The buttons the controller reported just pressed.
    pressed: Buttons,
    /// Jump on the passage's last draw that updated jump history.
    last_draw: JumpButton,
    /// The count `react_after_draw` leaves.
    after_draw: u16,
    /// The count `react` leaves, reading the previous draw from the frame's own pad.
    ordinary: u16,
    /// Whether she takes off: her pose follows the frame's own buttons either way.
    takes_off: bool,
}

/// The count at 5 with nothing the frame before, or at 3 the frame after a landing.
const HELD_5: JumpHeld = JumpHeld {
    frames: Wrapping(5),
    before: FrameBefore::Other,
};
const LANDED_3: JumpHeld = JumpHeld {
    frames: Wrapping(3),
    before: FrameBefore::LandedOrEnded,
};

/// The invented samples, told apart by the previous draw's jump alone.
const AFTER_PASSAGE: [AfterPassage; 7] = [
    AfterPassage {
        name: "held now, released on the last draw, no press reported",
        start: HELD_5,
        pad: button::JUMP,
        pressed: Buttons(0),
        last_draw: JumpButton::Up,
        after_draw: 0,
        ordinary: 6,
        takes_off: false,
    },
    AfterPassage {
        name: "held now and on the last draw, no press reported",
        start: HELD_5,
        pad: button::JUMP,
        pressed: Buttons(0),
        last_draw: JumpButton::Held,
        after_draw: 6,
        ordinary: 6,
        takes_off: false,
    },
    AfterPassage {
        name: "held now and on the last draw, a fresh press reported",
        start: HELD_5,
        pad: button::JUMP,
        pressed: button::JUMP,
        last_draw: JumpButton::Held,
        after_draw: 6,
        ordinary: 0,
        takes_off: true,
    },
    AfterPassage {
        name: "held now, released on the last draw, a fresh press reported",
        start: HELD_5,
        pad: button::JUMP,
        pressed: button::JUMP,
        last_draw: JumpButton::Up,
        after_draw: 0,
        ordinary: 0,
        takes_off: true,
    },
    AfterPassage {
        name: "up now, held on the last draw",
        start: HELD_5,
        pad: Buttons(0),
        pressed: Buttons(0),
        last_draw: JumpButton::Held,
        after_draw: 0,
        ordinary: 0,
        takes_off: false,
    },
    AfterPassage {
        name: "landed the frame before, held now and on the last draw",
        start: LANDED_3,
        pad: button::JUMP,
        pressed: Buttons(0),
        last_draw: JumpButton::Held,
        after_draw: 1,
        ordinary: 1,
        takes_off: true,
    },
    AfterPassage {
        name: "landed the frame before, held now, released on the last draw",
        start: LANDED_3,
        pad: button::JUMP,
        pressed: Buttons(0),
        last_draw: JumpButton::Up,
        after_draw: 0,
        ordinary: 1,
        takes_off: true,
    },
];

/// Spec: `Jumping`. An active frame after a passage counts jump held through against the
/// passage's last history-writing draw, whatever the controller reported pressed; her pose
/// reads the frame's own buttons, and a landing the frame before still makes jump held a press
/// and puts the count back to 0 first. `react` reads the previous draw from the frame's pad.
#[test]
fn an_active_frame_after_a_passage_counts_against_the_last_draw() {
    let content = Content::imported();
    let body = |pose| content.body(pose);
    let headings = |_| None;
    let rules = Rules::default();
    for case in &AFTER_PASSAGE {
        let frame = Frame {
            sprite_before_movement: Sprite::Plain,
            cinematic_active: false,
            pad: case.pad,
            pressed: case.pressed,
            items: Set::default(),
            spin_turned_far: false,
            animation_ends: None,
            elevator_moving: false,
            parity: Parity::Even,
            body: &body,
            cannon_heading: &headings,
        };
        let mut drawn = standing(case.start);
        let moves = drawn.moves(&frame, &content, &rules).unwrap();
        drawn.react_after_draw(moves, None, case.last_draw);
        let mut ordinary = standing(case.start);
        let moves = ordinary.moves(&frame, &content, &rules).unwrap();
        ordinary.react(moves, None);
        let name = case.name;
        assert_eq!(drawn.jump_held.frames.0, case.after_draw, "{name}");
        assert_eq!(ordinary.jump_held.frames.0, case.ordinary, "{name}");
        let took_off = drawn.pose.action == types::Action::TakingOff;
        assert_eq!(took_off, case.takes_off, "{name}");
        // Everything but the count is the same frame either way.
        drawn.jump_held.frames = ordinary.jump_held.frames;
        assert_eq!(drawn, ordinary, "{name}");
    }
}

/// Spec: `Jumping`. A passage's draws and the active frame after them, together: jump released
/// on a draw and held from the next with no press reported, the count stays 0 into the frame.
#[test]
fn a_passage_then_an_active_frame_keep_one_history() {
    let content = Content::imported();
    let body = |pose| content.body(pose);
    let headings = |_| None;
    let rules = Rules::default();
    let mut history = DrawnJumpHistory {
        jump_held: HELD_5,
        last_draw: JumpButton::Held,
    };
    history.draw(JumpButton::Up);
    assert_eq!(history.jump_held.frames.0, 0);
    let mut samus = standing(history.jump_held);
    let frame = Frame {
        sprite_before_movement: Sprite::Plain,
        cinematic_active: false,
        pad: button::JUMP,
        pressed: Buttons(0),
        items: Set::default(),
        spin_turned_far: false,
        animation_ends: None,
        elevator_moving: false,
        parity: Parity::Even,
        body: &body,
        cannon_heading: &headings,
    };
    let moves = samus.moves(&frame, &content, &rules).unwrap();
    samus.react_after_draw(moves, None, history.last_draw);
    assert_eq!(samus.jump_held.frames.0, 0);
    assert_eq!(samus.pose.action, types::Action::Still);
}
