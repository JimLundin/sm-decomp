//! The spec's vectors (`scenarios/vectors/samus-ground.toml`) run through the model, over the
//! flat collision stand-in its header defines.

use super::{
    Body, Collision, Contacts, Enemies, Facing, FallKind, Fixed, Ground, Inputs, Locomotion, Moved,
    Pad, Position, Radius, Side, SpeedMode, Stop, Vertical,
};
use crate::session::{SpeedCap, Switches};
use data::movement::{ExtraRun, GroundMotion, HorizontalSpeeds, Speed, SpeedEntry, SpeedTable};
use data::pose::MovementType;
use toml::Value;

const VECTORS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../scenarios/vectors/samus-ground.toml"
));

fn fixed(value: &Value) -> Fixed {
    let (pixel, sub) = value.as_str().unwrap().split_once('.').unwrap();
    Fixed::new(
        u16::from_str_radix(pixel, 16).unwrap(),
        u16::from_str_radix(sub, 16).unwrap(),
    )
}

fn word(value: &Value) -> u16 {
    u16::try_from(value.as_integer().unwrap()).unwrap()
}

fn flag(table: &Value, key: &str) -> bool {
    table.get(key).and_then(Value::as_bool).unwrap_or(false)
}

/// The spec's normal speed table (`SpeedTableRow`) and extra run (`ExtraRunCap`).
fn speeds() -> HorizontalSpeeds {
    let entry = |accel: (u16, u16), max: (u16, u16)| SpeedEntry {
        acceleration: Speed::new(accel.0, accel.1),
        max: Speed::new(max.0, max.1),
        deceleration: Speed::new(0, 0x8000),
    };
    let still = entry((0, 0xC000), (0, 0));
    HorizontalSpeeds {
        normal: SpeedTable::from_iter([
            (MovementType::Standing, still),
            (MovementType::Running, entry((0, 0x3000), (2, 0xC000))),
            (
                MovementType::MorphBallOnGround,
                entry((0, 0xC000), (3, 0x4000)),
            ),
            (
                MovementType::SpringBallOnGround,
                entry((0, 0xC000), (3, 0x4000)),
            ),
            (MovementType::Crouching, still),
            (MovementType::TurningAroundOnGround, still),
            (
                MovementType::CrouchingTransition,
                entry((0, 0xC000), (1, 0x4000)),
            ),
            (MovementType::Moonwalking, entry((0, 0xC000), (0, 0x8000))),
            (MovementType::RanIntoWall, still),
        ]),
        extra_run: ExtraRun {
            acceleration: Speed::new(0, 0x1000),
            booster_max: Speed::new(7, 0),
            max: Speed::new(2, 0),
        },
    }
}

/// The flat stand-in: block-aligned solid floor, walls and ceiling; everything else is air.
struct Flat {
    floor_top: Option<u16>,
    wall_right: Option<u16>,
    wall_left: Option<u16>,
    ceiling_bottom: Option<u16>,
    solid_enemy_floor: bool,
}

impl Flat {
    fn of(world: &Value) -> Self {
        let get = |key| world.get(key).map(word);
        Self {
            floor_top: get("floor_top"),
            wall_right: get("wall_right"),
            wall_left: get("wall_left"),
            ceiling_bottom: get("ceiling_bottom"),
            solid_enemy_floor: flag(world, "solid_enemy_floor"),
        }
    }

    fn solid_column(&self, x: u16) -> bool {
        self.wall_right.is_some_and(|wall| x >= wall) || self.wall_left.is_some_and(|wall| x < wall)
    }

    fn solid_row(&self, y: u16) -> bool {
        self.floor_top.is_some_and(|floor| y >= floor)
            || self.ceiling_bottom.is_some_and(|ceiling| y < ceiling)
    }
}

const fn not_below_zero(pixels: u16) -> u16 {
    if pixels.cast_signed().is_negative() {
        0
    } else {
        pixels
    }
}

impl Collision for Flat {
    fn move_horizontal(&mut self, body: Body, distance: Fixed, _: Enemies) -> Moved {
        let (x, y, r) = (
            body.position.x.pixel(),
            body.position.y.pixel(),
            body.radius,
        );
        let target = body.position.x.plus(distance).pixel();
        let rightward = !distance.pixel().cast_signed().is_negative();
        let edge = if rightward {
            target.wrapping_add(r.x).wrapping_sub(1)
        } else {
            target.wrapping_sub(r.x)
        };
        let mut rows = y.wrapping_sub(r.y)..=y.wrapping_add(r.y).wrapping_sub(1);
        if !(self.solid_column(edge) || rows.any(|row| self.solid_row(row))) {
            return Moved {
                distance,
                stop: None,
            };
        }
        let (side, pixels, sub) = if rightward {
            let pixels = (edge & !0xF).wrapping_sub(r.x).wrapping_sub(x);
            (Side::Right, not_below_zero(pixels), 0xFFFF)
        } else {
            let pixels = (edge | 0xF)
                .wrapping_add(1)
                .wrapping_add(r.x)
                .wrapping_sub(x);
            (
                Side::Left,
                if pixels.cast_signed() > 0 { 0 } else { pixels },
                0,
            )
        };
        let stop = Some(Stop {
            side,
            subpixel: Some(sub),
        });
        Moved {
            distance: Fixed::new(pixels, 0),
            stop,
        }
    }

    fn move_down(&mut self, body: Body, distance: Fixed, enemies: Enemies) -> Moved {
        if self.solid_enemy_floor && matches!(enemies, Enemies::Test) {
            let stop = Some(Stop {
                side: Side::Below,
                subpixel: None,
            });
            return Moved {
                distance: Fixed::default(),
                stop,
            };
        }
        let (y, r) = (body.position.y.pixel(), body.radius);
        let edge = body
            .position
            .y
            .plus(distance)
            .pixel()
            .wrapping_add(r.y)
            .wrapping_sub(1);
        if !self.solid_row(edge) {
            return Moved {
                distance,
                stop: None,
            };
        }
        let pixels = not_below_zero((edge & !0xF).wrapping_sub(r.y).wrapping_sub(y));
        let stop = Some(Stop {
            side: Side::Below,
            subpixel: Some(0xFFFF),
        });
        Moved {
            distance: Fixed::new(pixels, 0),
            stop,
        }
    }

    fn enemy_blocks(&self, _: Body, _: Side, _: Fixed) -> bool {
        false
    }
}

fn vertical(name: &str) -> Vertical {
    match name {
        "None" => Vertical::None,
        "Landed" => Vertical::Landed,
        "HitCeiling" => Vertical::HitCeiling,
        "Falling(Airborne)" => Vertical::Falling(FallKind::Airborne),
        "Falling(MorphBall)" => Vertical::Falling(FallKind::MorphBall),
        "Falling(SpringBall)" => Vertical::Falling(FallKind::SpringBall),
        "Falling(NoChange)" => Vertical::Falling(FallKind::NoChange),
        other => panic!("unknown vertical {other}"),
    }
}

fn mode(name: &str) -> SpeedMode {
    match name {
        "Accelerating" => SpeedMode::Accelerating,
        "TurningAround" => SpeedMode::TurningAround,
        "Decelerating" => SpeedMode::Decelerating,
        other => panic!("unknown mode {other}"),
    }
}

fn locomotion(name: &str) -> Locomotion {
    let ball = |spring, at_rest| Locomotion::Ball { spring, at_rest };
    match name {
        "FacingForward" => Locomotion::FacingForward,
        "Standing" => Locomotion::Standing { plain: false },
        "StandingPlain" => Locomotion::Standing { plain: true },
        "Running" => Locomotion::Running,
        "BallAtRest" => ball(false, true),
        "BallRolling" => ball(false, false),
        "SpringBallAtRest" => ball(true, true),
        "SpringBallRolling" => ball(true, false),
        "Crouching" => Locomotion::Crouching,
        "TurningOnGround" => Locomotion::TurningOnGround,
        "Transition" => Locomotion::Transition,
        "Moonwalking" => Locomotion::Moonwalking,
        "RanIntoWall" => Locomotion::RanIntoWall,
        other => panic!("unknown locomotion {other}"),
    }
}

fn inputs(step: &Value) -> Inputs {
    let pad: Vec<&str> = step["pad"]
        .as_array()
        .unwrap()
        .iter()
        .map(|button| button.as_str().unwrap())
        .collect();
    let radius = step["radius"].as_array().unwrap();
    let air = flag(step, "airborne").then(|| {
        vertical(
            step.get("air_vertical")
                .and_then(Value::as_str)
                .unwrap_or("None"),
        )
    });
    Inputs {
        locomotion: locomotion(step["locomotion"].as_str().unwrap()),
        facing: match step["facing"].as_str().unwrap() {
            "Left" => Facing::Left,
            "Right" => Facing::Right,
            "Forward" => Facing::Forward,
            other => panic!("unknown facing {other}"),
        },
        radius: Radius {
            x: word(&radius[0]),
            y: word(&radius[1]),
        },
        pad: Pad {
            run: pad.contains(&"run"),
            shoot: pad.contains(&"shoot"),
        },
        new_running_pose: flag(step, "new_running_pose"),
        air,
        elevator_moving: flag(step, "elevator_moving"),
        on_slope: flag(step, "on_slope"),
    }
}

fn before(table: &Value) -> Ground {
    let position = table["position"].as_array().unwrap();
    let speed = |key| table.get(key).map_or_else(Fixed::default, fixed);
    Ground {
        position: Position {
            x: fixed(&position[0]),
            y: fixed(&position[1]),
        },
        base_speed: speed("base_speed"),
        extra_run: speed("extra_run"),
        momentum: flag(table, "momentum"),
        mode: table
            .get("mode")
            .map_or(SpeedMode::Accelerating, |m| mode(m.as_str().unwrap())),
    }
}

fn ps(value: Fixed) -> String {
    format!("{:04X}.{:04X}", value.pixel(), value.sub())
}

/// Every difference of one step's results from its expectations, as "field: expected, actual".
fn differences(step: &Value, samus: &Ground, contacts: Contacts) -> Vec<String> {
    let mut found = Vec::new();
    let mut check = |field: &str, expected: String, actual: String| {
        if expected != actual {
            found.push(format!("{field}: expected {expected}, actual {actual}"));
        }
    };
    if let Some(after) = step.get("after") {
        if let Some(position) = after.get("position").and_then(Value::as_array) {
            check("position.x", ps(fixed(&position[0])), ps(samus.position.x));
            check("position.y", ps(fixed(&position[1])), ps(samus.position.y));
        }
        let speeds = [
            ("position_x", samus.position.x),
            ("position_y", samus.position.y),
            ("base_speed", samus.base_speed),
            ("extra_run", samus.extra_run),
        ];
        for (key, actual) in speeds {
            if let Some(expected) = after.get(key) {
                check(key, ps(fixed(expected)), ps(actual));
            }
        }
        if let Some(expected) = after.get("momentum") {
            check(
                "momentum",
                expected.as_bool().unwrap().to_string(),
                samus.momentum.to_string(),
            );
        }
        if let Some(expected) = after.get("mode") {
            let expected = mode(expected.as_str().unwrap());
            check("mode", format!("{expected:?}"), format!("{:?}", samus.mode));
        }
    }
    if let Some(expected) = step.get("contacts") {
        let flags = [
            ("wall_stop", contacts.wall_stop),
            ("ran_into_wall", contacts.ran_into_wall),
            ("end_vertical_motion", contacts.requests.end_vertical_motion),
            (
                "restart_standing_animation",
                contacts.requests.restart_standing_animation,
            ),
        ];
        for (key, actual) in flags {
            if let Some(value) = expected.get(key) {
                check(
                    key,
                    value.as_bool().unwrap().to_string(),
                    actual.to_string(),
                );
            }
        }
        if let Some(value) = expected.get("vertical") {
            let expected = vertical(value.as_str().unwrap());
            check(
                "vertical",
                format!("{expected:?}"),
                format!("{:?}", contacts.vertical),
            );
        }
    }
    found
}

#[test]
fn every_vector_passes() {
    let file: Value = VECTORS.parse().unwrap();
    let vectors = file["vector"].as_array().unwrap();
    let speeds = speeds();
    let mut failures = Vec::new();
    for vector in vectors {
        let id = vector["id"].as_str().unwrap();
        let cap = vector.get("rules").and_then(|rules| rules.get("speed_cap"));
        let speed_cap = match cap.and_then(Value::as_str) {
            Some("Exact") => SpeedCap::Exact,
            Some("Wordwise") | None => SpeedCap::Wordwise,
            Some(other) => panic!("unknown speed_cap {other}"),
        };
        let switches = Switches { speed_cap };
        let mut world = Flat::of(&vector["world"]);
        let mut samus = before(&vector["before"]);
        for (frame, step) in vector["step"].as_array().unwrap().iter().enumerate() {
            let contacts = samus
                .step(
                    &inputs(step),
                    switches,
                    &speeds,
                    &GroundMotion::VANILLA,
                    &mut world,
                )
                .unwrap();
            for difference in differences(step, &samus, contacts) {
                failures.push(format!("{id} step {}: {difference}", frame + 1));
            }
        }
    }
    let failed: std::collections::BTreeSet<&str> = failures
        .iter()
        .filter_map(|failure| failure.split(' ').next())
        .collect();
    assert!(
        failures.is_empty(),
        "{} of {} vectors fail:\n{}",
        failed.len(),
        vectors.len(),
        failures.join("\n")
    );
}

/// `SpeedCap`'s variants differ exactly where the pixel words agree and the subpixel is not
/// below the cap's: 2.1000 against 2.C000.
#[test]
fn speed_cap_variants_differ_on_the_wordwise_case() {
    let (speed, max) = (Fixed::new(2, 0x1000), Fixed::new(2, 0xC000));
    assert_eq!(speed.capped(max, SpeedCap::Wordwise), max);
    assert_eq!(speed.capped(max, SpeedCap::Exact), speed);
    let below = Fixed::new(1, 0xF000);
    assert_eq!(
        below.capped(max, SpeedCap::Wordwise),
        below.capped(max, SpeedCap::Exact)
    );
}
