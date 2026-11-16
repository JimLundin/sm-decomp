//! Complete constructed work comparisons through the game's neutral output consumer.
//! Expectations are observer vectors; native records do not witness complete requests yet.

use core::num::Wrapping;
use samus_movement::{
    Carry, Dashing, Frame, JumpHeld, Phase, Rules, Run, Samus, Seat, Spin, Vertical,
};
use std::io::Write as _;
use types::{
    Aim, Block, Buttons, Cell, Gun, Hitbox, Legs, Pair, Pose, Radius, Side, Slope, SlopeFaces,
    World, ZERO,
};

#[path = "../../../../clean/samus_movement/tests/work.rs"]
mod expected;

fn word(value: &toml::Value) -> u16 {
    u16::try_from(value.as_integer().unwrap()).unwrap()
}

fn pair(value: &toml::Value) -> Radius {
    let values = value.as_array().unwrap();
    Pair {
        x: Wrapping(word(&values[0])),
        y: Wrapping(word(&values[1])),
    }
}

fn fixed(value: &toml::Value) -> types::Fixed {
    let value = value.as_str().unwrap();
    let unsigned = value.strip_prefix('+').unwrap_or(value);
    let (pixels, subpixels) = unsigned.split_once('.').unwrap();
    Wrapping(
        (u32::from_str_radix(pixels, 16).unwrap() << 16)
            | u32::from_str_radix(subpixels, 16).unwrap(),
    )
}

fn position(value: &toml::Value) -> types::Position {
    let values = value.as_array().unwrap();
    Pair {
        x: fixed(&values[0]),
        y: fixed(&values[1]),
    }
}

fn pose(value: &toml::Value) -> Pose {
    let action = match value["action"].as_str().unwrap() {
        "Still" => types::Action::Still,
        "Running" => types::Action::Running,
        "Jumping" => types::Action::Jumping,
        "Spinning" => types::Action::Spinning,
        "Falling" => types::Action::Falling,
        "AgainstWall" => types::Action::AgainstWall,
        "Landing" => types::Action::Landing(match value["landed_from"].as_str().unwrap() {
            "Jump" => types::LandedFrom::Jump,
            "Spin" => types::LandedFrom::Spin,
            name => panic!("unrecognized constructed landing {name}"),
        }),
        name => panic!("unrecognized constructed action {name}"),
    };
    Pose {
        action,
        body: value["body"].clone().try_into().unwrap(),
        facing: value["facing"].clone().try_into().unwrap(),
        aim: value
            .get("aim")
            .map_or(Aim::Forward, |aim| aim.clone().try_into().unwrap()),
        gun: Gun::Stowed,
        legs: Legs::Together,
    }
}

fn buttons(value: Option<&toml::Value>) -> Buttons {
    let mut buttons = Buttons(0);
    for name in value
        .into_iter()
        .flat_map(|value| value.as_array().unwrap())
    {
        buttons.0 |= match name.as_str().unwrap() {
            "right" => types::button::RIGHT.0,
            "run" => types::button::RUN.0,
            "aim_down" => types::button::AIM_DOWN.0,
            name => panic!("unrecognized constructed button {name}"),
        };
    }
    buttons
}

struct Room {
    floor: u16,
    ceiling: Option<u16>,
    slope: Option<(Cell, Slope)>,
    enemies: Vec<Hitbox>,
}

impl Room {
    fn of(value: &toml::Value) -> Self {
        let slope = value.get("slope_floor").map(|slope| {
            let corner = pair(&slope["block"]);
            let surface: Vec<_> = slope["surface"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| Wrapping(word(value)))
                .collect();
            let factor = slope.get("factor").map_or(256, |value| {
                let (whole, fraction) = value.as_str().unwrap().split_once('.').unwrap();
                (u16::from_str_radix(whole, 16).unwrap() << 8)
                    | u16::from_str_radix(fraction, 16).unwrap()
            });
            (
                Cell {
                    x: corner.x.0 / 16,
                    y: corner.y.0 / 16,
                },
                Slope {
                    faces: SlopeFaces::Floor,
                    depth: surface.try_into().unwrap(),
                    factor: types::SlopeFactor(factor),
                },
            )
        });
        Self {
            floor: word(&value["floor_top"]),
            ceiling: value.get("ceiling_bottom").map(word),
            slope,
            enemies: value
                .get("enemies")
                .into_iter()
                .flat_map(|value| value.as_array().unwrap())
                .map(|enemy| Hitbox {
                    centre: pair(&enemy["centre"]),
                    radius: pair(&enemy["radius"]),
                })
                .collect(),
        }
    }

    fn roster(&self) -> game::samus_movement_bridge::roster::Roster {
        use game::collision::solid_enemy::Obstacle;
        use game::state::samus::Fixed;
        let room = game::collision::Room {
            blocks: &[],
            width: game::state::room::BlockColumns(0),
            slopes: None,
            doors: &[],
        };
        game::samus_movement_bridge::SamusWorld::of(
            room,
            self.enemies.iter().map(|enemy| Obstacle {
                position: game::state::samus::Point {
                    x: Fixed {
                        pixels: enemy.centre.x.0,
                        subpixels: 0,
                    },
                    y: Fixed {
                        pixels: enemy.centre.y.0,
                        subpixels: 0,
                    },
                },
                radius_x: enemy.radius.x.0,
                radius_y: enemy.radius.y.0,
                solid: true,
                frozen: false,
            }),
        )
        .roster()
        .clone()
    }
}

impl World for Room {
    fn block(&self, cell: Cell) -> Block {
        if let Some((at, slope)) = self.slope
            && cell == at
        {
            return Block::Slope(slope);
        }
        if u32::from(cell.y) * 16 >= u32::from(self.floor)
            || self.ceiling.is_some_and(|ceiling| {
                u32::from(cell.y).saturating_add(1).saturating_mul(16) <= u32::from(ceiling)
            })
        {
            Block::Solid
        } else {
            Block::Air
        }
    }
    fn lets_through(&self, _: Cell, _: types::Way) -> bool {
        false
    }
    fn solid_enemies(&self) -> &[Hitbox] {
        &self.enemies
    }
}

fn run(vector: &toml::Value) -> (Samus, game::gameplay_outputs::MovementOutputs) {
    let initial = pose(&vector["pose"]);
    let old_radius = pair(&vector["pose"]["radius"]);
    let target = vector
        .get("next_pose")
        .map(|value| (pose(value), pair(&value["radius"])));
    let substitute = vector["after"]
        .get("pose")
        .map(|value| (pose(value), pair(&value["radius"])));
    let body = |selected| {
        if selected == initial {
            old_radius
        } else if let Some((_, radius)) = target.filter(|(pose, _)| *pose == selected) {
            radius
        } else if let Some((_, radius)) = substitute.filter(|(pose, _)| *pose == selected) {
            radius
        } else {
            panic!("unrecognized constructed body {selected:?}")
        }
    };
    let before = &vector["before"];
    let mut samus = Samus {
        position: position(&before["position"]),
        pose: initial,
        run: Run {
            speed: before.get("speed_x").map_or(ZERO, fixed),
            way: Side::Right,
            carry: Carry::FollowsPad,
            dash: ZERO,
            dashing: Dashing::Off,
        },
        vertical: Vertical {
            speed: before.get("speed_y").map_or(ZERO, fixed),
            phase: if matches!(
                initial.action,
                types::Action::Still
                    | types::Action::Running
                    | types::Action::Landing(_)
                    | types::Action::AgainstWall
            ) {
                Phase::Ground
            } else {
                Phase::Falling
            },
            bounces: match before.get("bounces").map_or(0, word) {
                0 => types::Bounces::None,
                2 => types::Bounces::Twice,
                count => panic!("unrecognized constructed bounce {count}"),
            },
        },
        seat: Seat::Unseated,
        spin: Spin::Off,
        jump_held: JumpHeld {
            frames: Wrapping(0),
            before: samus_movement::FrameBefore::Other,
        },
    };
    let frame = Frame {
        sprite_before_movement: types::Sprite::Plain,
        cinematic_active: false,
        pad: buttons(vector.get("pad")),
        pressed: buttons(vector.get("pressed")),
        items: types::Set::default(),
        spin_turned_far: false,
        elevator_moving: false,
        parity: types::Parity::Even,
        animation_ends: vector
            .get("animation_ends")
            .map(|value| match value.as_str().unwrap() {
                "End" => types::ClipEnd::End,
                "EndUnlessJump" => types::ClipEnd::EndUnlessJump,
                name => panic!("unrecognized constructed animation end {name}"),
            }),
        body: &body,
        cannon_heading: &|_| None,
    };
    let mut rules = Rules::default();
    if let Some(limit) = vector
        .get("rules")
        .and_then(|rules| rules.get("move_limit"))
    {
        rules.move_limit = Wrapping(word(limit));
    }
    let room = Room::of(&vector["world"]);
    let outcome = samus.step(&frame, &room, &rules).unwrap();
    let mut outputs = game::gameplay_outputs::Outputs::default();
    outputs.moved_with_roster(&outcome, &room.roster());
    (samus, outputs.movement.unwrap())
}

fn compare_attempt(actual: &types::MotionAttempt, expected: &types::MotionAttempt, field: &str) {
    assert_eq!(actual.purpose, expected.purpose, "{field}.purpose");
    assert_eq!(actual.direction, expected.direction, "{field}.direction");
    assert_eq!(actual.world, expected.world, "{field}.world");
    assert_eq!(actual.start, expected.start, "{field}.start");
    assert_eq!(actual.radius, expected.radius, "{field}.radius");
    assert_eq!(actual.intended, expected.intended, "{field}.intended");
    match (&actual.enemy_search, &expected.enemy_search) {
        (None, None) => {}
        (Some(actual), Some(expected)) => {
            assert_eq!(
                actual.supplied_count, expected.supplied_count,
                "{field}.enemy_search.supplied_count"
            );
            assert_eq!(
                actual.candidates.len(),
                expected.candidates.len(),
                "{field}.enemy_search.candidates.length"
            );
            for (index, (actual, expected)) in actual
                .candidates
                .iter()
                .zip(&expected.candidates)
                .enumerate()
            {
                assert_eq!(
                    actual.candidate, expected.candidate,
                    "{field}.enemy_search.candidates[{index}].candidate"
                );
                assert_eq!(
                    actual.centre, expected.centre,
                    "{field}.enemy_search.candidates[{index}].centre"
                );
                assert_eq!(
                    actual.radius, expected.radius,
                    "{field}.enemy_search.candidates[{index}].radius"
                );
                assert_eq!(
                    actual.result, expected.result,
                    "{field}.enemy_search.candidates[{index}].result"
                );
            }
        }
        (Some(_), None) | (None, Some(_)) => panic!("{field}.enemy_search.admission"),
    }
    assert_eq!(actual.effect, expected.effect, "{field}.effect");
    assert_eq!(actual.result, expected.result, "{field}.result");
}

fn compare_work(actual: &[types::MovementWork], expected: &[types::MovementWork], id: &str) {
    let counts = |work: &[types::MovementWork]| {
        (
            work.iter()
                .filter(|event| matches!(event, types::MovementWork::Motion(_)))
                .count(),
            work.iter()
                .filter(|event| matches!(event, types::MovementWork::Support(_)))
                .count(),
            work.iter()
                .filter(|event| matches!(event, types::MovementWork::BodyFit(_)))
                .count(),
        )
    };
    let (motion, support, fit) = counts(actual);
    let (expected_motion, expected_support, expected_fit) = counts(expected);
    assert_eq!(motion, expected_motion, "{id}: movement_work.motion.length");
    assert_eq!(
        support, expected_support,
        "{id}: movement_work.support.length"
    );
    assert_eq!(fit, expected_fit, "{id}: movement_work.body_fit.length");
    assert_eq!(actual.len(), expected.len(), "{id}: movement_work.length");
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        let field = format!("{id}: movement_work[{index}]");
        match expected {
            types::MovementWork::Motion(expected) => {
                let types::MovementWork::Motion(actual) = actual else {
                    panic!("{field}.kind")
                };
                compare_attempt(actual, expected, &field);
            }
            types::MovementWork::Support(expected) => {
                let types::MovementWork::Support(actual) = actual else {
                    panic!("{field}.kind")
                };
                assert_eq!(actual, expected, "{field}.support");
            }
            types::MovementWork::BodyFit(expected) => {
                let types::MovementWork::BodyFit(actual) = actual else {
                    panic!("{field}.kind")
                };
                assert_eq!(actual.old_radius, expected.old_radius, "{field}.old_radius");
                assert_eq!(actual.new_radius, expected.new_radius, "{field}.new_radius");
                assert_eq!(actual.centre, expected.centre, "{field}.centre");
                assert_eq!(
                    actual.checks.len(),
                    expected.checks.len(),
                    "{field}.checks.length"
                );
                for (index, (actual, expected)) in
                    actual.checks.iter().zip(&expected.checks).enumerate()
                {
                    compare_attempt(actual, expected, &format!("{field}.checks[{index}]"));
                }
                assert_eq!(actual.result, expected.result, "{field}.result");
                assert_eq!(actual.placement, expected.placement, "{field}.placement");
            }
        }
    }
}

fn compare_cells(actual: &[types::CollisionTest], expected: &toml::Value, id: &str) {
    let expected = expected.as_array().unwrap();
    assert_eq!(actual.len(), expected.len(), "{id}: collision_tests.length");
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        let cell = pair(&expected["cell"]);
        assert_eq!(
            actual.cell,
            Cell {
                x: cell.x.0,
                y: cell.y.0
            },
            "{id}: collision_tests[{index}].cell"
        );
        let direction: types::Direction = expected["direction"].clone().try_into().unwrap();
        assert_eq!(
            actual.direction, direction,
            "{id}: collision_tests[{index}].direction"
        );
        let purpose: types::CollisionPurpose = expected["purpose"].clone().try_into().unwrap();
        assert_eq!(
            actual.purpose, purpose,
            "{id}: collision_tests[{index}].purpose"
        );
        let vertical = expected
            .get("vertical_motion")
            .map(|value| value.clone().try_into().unwrap());
        assert_eq!(
            actual.vertical_motion, vertical,
            "{id}: collision_tests[{index}].vertical_motion"
        );
    }
}

fn compare_roster(
    delivered: &game::gameplay_outputs::MovementOutputs,
    expected: &[types::MovementWork],
    room: &Room,
    id: &str,
) {
    use game::samus_movement_bridge::roster::{Occurrence, SearchEnd};
    let roster = room.roster();
    assert_eq!(
        delivered.roster.as_ref(),
        Some(&roster),
        "{id}: caller_roster"
    );
    let attempts: Vec<_> = expected
        .iter()
        .flat_map(|work| match work {
            types::MovementWork::Motion(attempt) => vec![attempt],
            types::MovementWork::Support(_) => Vec::new(),
            types::MovementWork::BodyFit(fit) => fit.checks.iter().collect(),
        })
        .filter(|attempt| attempt.enemy_search.is_some())
        .collect();
    assert_eq!(
        delivered.searches.len(),
        attempts.len(),
        "{id}: caller_searches.length"
    );
    for (index, (actual, expected)) in delivered.searches.iter().zip(attempts).enumerate() {
        compare_attempt(
            &actual.attempt,
            expected,
            &format!("{id}: caller_searches[{index}].attempt"),
        );
        let search = expected.enemy_search.as_ref().unwrap();
        let accepted = search.candidates.last().is_some_and(|candidate| {
            matches!(
                candidate.result,
                types::CandidateResult::Touching | types::CandidateResult::StopsAtGap(_)
            )
        });
        let (count, end) = if search.supplied_count.0 == 0 {
            (0, SearchEnd::Empty)
        } else if accepted {
            (search.candidates.len(), SearchEnd::Accepted)
        } else {
            (search.supplied_count.0, SearchEnd::Exhausted)
        };
        assert_eq!(
            actual.prefix,
            (0..count).map(Occurrence).collect::<Vec<_>>(),
            "{id}: caller_searches[{index}].prefix"
        );
        assert_eq!(actual.end, end, "{id}: caller_searches[{index}].end");
    }
}

#[test]
fn complete_constructed_work_reaches_gameplay_consumer() {
    let vectors: toml::Value = toml::from_str(include_str!(
        "../../../../../scenarios/vectors/samus-movement.toml"
    ))
    .unwrap();
    let mut compared = 0;
    let mut null = 0;
    for vector in vectors["vector"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|vector| vector["id"].as_str().unwrap().starts_with("C-attempt-"))
    {
        assert_eq!(vector["source"].as_str(), Some("constructed"));
        let expected: Vec<expected::ExpectedWork> =
            vector["after"]["movement_work"].clone().try_into().unwrap();
        let expected: Vec<_> = expected.iter().map(expected::ExpectedWork::model).collect();
        let (samus, delivered) = run(vector);
        let id = vector["id"].as_str().unwrap();
        compare_work(&delivered.work, &expected, id);
        compare_roster(&delivered, &expected, &Room::of(&vector["world"]), id);
        compare_cells(
            &delivered.collision_tests,
            &vector["after"]["collision_tests"],
            id,
        );
        assert_eq!(
            samus.position,
            position(&vector["after"]["position"]),
            "{id}: position"
        );
        if let Some(selected) = vector["after"].get("pose") {
            assert_eq!(samus.pose, pose(selected), "{id}: pose");
        }
        let skip = vector["after"]
            .get("movement_skip")
            .map(|value| value.clone().try_into().unwrap());
        assert_eq!(delivered.skip, skip, "{id}: movement_skip");
        if let Some(seated) = vector["after"].get("seated") {
            assert_eq!(
                samus.seat == Seat::Seated,
                seated.as_bool().unwrap(),
                "{id}: seated"
            );
        }
        null += usize::from(
            expected.is_empty()
                && skip.is_none()
                && position(&vector["before"]["position"]) == samus.position,
        );
        compared += 1;
    }
    assert_eq!((compared, null), (15, 0));
    writeln!(std::io::stdout(),
        "constructed complete work {compared}/15; skipped 0; defects 0; null {null}/15; native complete work 0"
    ).unwrap();
}
