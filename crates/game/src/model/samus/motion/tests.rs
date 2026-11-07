//! The spec's vectors (`scenarios/vectors/samus-motion.toml`) run through the model, over the
//! flat collision stand-in its header defines.

use super::{
    Bounce, Contacts, Equipment, Facing, FallKind, Inputs, JumpButton, LandKind, Locomotion,
    MotionRules, Pad, PoseChange, PoseCommand, PoseRequest, PoseStart, Position, Radius, Sideways,
    SpeedMode, Spin, State, VerticalMotion, VerticalResult,
};
use crate::model::collision::{
    Blocker, Body, Collision, Enemies, EnemyHit, Heading, Moved, Stop, WallHit,
};
use crate::model::num::Fixed;
use crate::session::SpeedCap;
use data::movement::{
    AirMotion, ExtraRun, GroundMotion, HorizontalSpeeds, Speed, SpeedEntry, SpeedTable,
    VerticalSpeeds,
};
use data::pose::MovementType;
use toml::Value;

const VECTORS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../scenarios/vectors/samus-motion.toml"
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

fn text<'a>(table: &'a Value, key: &str) -> Option<&'a str> {
    table.get(key).and_then(Value::as_str)
}

/// The spec's normal speed table (§3) and extra run.
fn speeds() -> HorizontalSpeeds {
    use MovementType as M;
    let entry = |accel: u16, max: (u16, u16)| SpeedEntry {
        acceleration: Speed::new(0, accel),
        max: Speed::new(max.0, max.1),
        deceleration: Speed::new(0, 0x8000),
    };
    let rows = [
        (
            [
                M::Standing,
                M::Crouching,
                M::TurningAroundOnGround,
                M::RanIntoWall,
                M::TurningAroundJumping,
                M::TurningAroundFalling,
            ]
            .as_slice(),
            entry(0xC000, (0, 0)),
        ),
        (&[M::Running], entry(0x3000, (2, 0xC000))),
        (
            &[M::MorphBallOnGround, M::SpringBallOnGround],
            entry(0xC000, (3, 0x4000)),
        ),
        (
            &[M::CrouchingTransition, M::NormalJumping, M::SpringBallInAir],
            entry(0xC000, (1, 0x4000)),
        ),
        (
            &[M::SpinJumping, M::WallJumping],
            entry(0xC000, (1, 0x6000)),
        ),
        (
            &[M::Falling, M::MorphBallFalling, M::SpringBallFalling],
            entry(0xC000, (1, 0)),
        ),
        (&[M::Moonwalking], entry(0xC000, (0, 0x8000))),
    ];
    HorizontalSpeeds {
        normal: SpeedTable::from_iter(
            rows.iter()
                .flat_map(|(types, entry)| types.iter().map(move |movement| (*movement, *entry))),
        ),
        extra_run: ExtraRun {
            acceleration: Speed::new(0, 0x1000),
            booster_max: Speed::new(7, 0),
            max: Speed::new(2, 0),
        },
    }
}

const VERTICAL: VerticalSpeeds = VerticalSpeeds {
    gravity: Speed::new(0, 0x1C00),
    jump: Speed::new(4, 0xE000),
    hi_jump: Speed::new(6, 0),
    wall_jump: Speed::new(4, 0xA000),
    wall_hi_jump: Speed::new(5, 0x8000),
};

/// The flat stand-in: block-aligned solids and two kinds of solid enemy; everything else is air.
struct Flat {
    floor_top: Option<u16>,
    wall_right: Option<u16>,
    wall_left: Option<u16>,
    ceiling_bottom: Option<u16>,
    ledge_right: Option<(u16, u16)>,
    solid_enemy_floor: bool,
    enemy_left: Option<u16>,
    enemy_right: Option<u16>,
    slope_seat: i16,
}

fn signed(pixels: u16) -> i32 {
    i32::from(pixels.cast_signed())
}

/// A whole-pixel gap of `pixels`, never negative in the cases the stand-in reaches.
fn gap(pixels: i32) -> Fixed {
    Fixed::new(u16::try_from(pixels).unwrap(), 0)
}

impl Flat {
    fn of(world: &Value) -> Self {
        let get = |key| world.get(key).map(word);
        Self {
            floor_top: get("floor_top"),
            wall_right: get("wall_right"),
            wall_left: get("wall_left"),
            ceiling_bottom: get("ceiling_bottom"),
            ledge_right: world
                .get("ledge_right")
                .map(|ledge| (word(&ledge["x"]), word(&ledge["bottom"]))),
            solid_enemy_floor: flag(world, "solid_enemy_floor"),
            enemy_left: get("enemy_left"),
            enemy_right: get("enemy_right"),
            slope_seat: world
                .get("slope_seat")
                .map_or(0, |seat| i16::try_from(seat.as_integer().unwrap()).unwrap()),
        }
    }

    fn solid(&self, x: u16, y: u16) -> bool {
        self.floor_top.is_some_and(|floor| y >= floor)
            || self.ceiling_bottom.is_some_and(|ceiling| y < ceiling)
            || self.wall_right.is_some_and(|wall| x >= wall)
            || self.wall_left.is_some_and(|wall| x < wall)
            || self
                .ledge_right
                .is_some_and(|(left, bottom)| x >= left && y < bottom)
    }

    fn column_solid(&self, x: u16, body: Body) -> bool {
        let (y, r) = (body.position.y.pixel(), body.radius.y);
        (y.wrapping_sub(r)..=y.wrapping_add(r).wrapping_sub(1)).any(|row| self.solid(x, row))
    }

    fn row_solid(&self, y: u16, body: Body) -> bool {
        let (x, r) = (body.position.x.pixel(), body.radius.x);
        (x.wrapping_sub(r)..=x.wrapping_add(r).wrapping_sub(1)).any(|column| self.solid(column, y))
    }

    /// The solid-enemy test of the header: `Some((touching, gap))` when one is found.
    fn enemy(&self, body: Body, heading: Heading, magnitude: Fixed) -> Option<(bool, Fixed)> {
        let (x, rx) = (body.position.x, signed(body.radius.x));
        let (edge, extra, target, near) = match heading {
            Heading::Left => (
                signed(self.enemy_left?),
                i32::from(x.sub() != magnitude.sub()),
                signed(x.minus(magnitude).pixel()),
                signed(x.pixel()).wrapping_sub(rx),
            ),
            Heading::Right => (
                signed(self.enemy_right?),
                i32::from(!(x.sub() == 0 && magnitude.sub() == 0)),
                signed(x.plus(magnitude).pixel()),
                signed(x.pixel()).wrapping_add(rx),
            ),
            Heading::Up | Heading::Down => return None,
        };
        // Mirrored, so that on either side the enemy is found past `edge`.
        let flip = |value: i32| {
            if matches!(heading, Heading::Left) {
                value.wrapping_neg()
            } else {
                value
            }
        };
        let (edge, near) = (flip(edge), flip(near));
        let reached = flip(target).wrapping_add(extra).wrapping_add(rx);
        (reached > edge && near <= edge).then(|| (near == edge, gap(edge.wrapping_sub(near))))
    }
}

const fn not_below_zero(pixels: u16) -> u16 {
    if pixels.cast_signed().is_negative() {
        0
    } else {
        pixels
    }
}

const fn block(side: Heading, pixels: u16, subpixel: u16) -> Moved {
    Moved {
        distance: Fixed::new(pixels, 0),
        stop: Some(Stop {
            side,
            blocker: Blocker::Block { subpixel },
        }),
        seat: 0,
    }
}

const fn full(distance: Fixed) -> Moved {
    Moved {
        distance,
        stop: None,
        seat: 0,
    }
}

impl Flat {
    fn sideways(&self, body: Body, heading: Heading, distance: Fixed, enemies: Enemies) -> Moved {
        let magnitude = if matches!(heading, Heading::Left) {
            Fixed::ZERO.minus(distance)
        } else {
            distance
        };
        if matches!(enemies, Enemies::Test)
            && let Some((touching, gap)) = self.enemy(body, heading, magnitude)
        {
            let gap = if matches!(heading, Heading::Left) {
                Fixed::ZERO.minus(gap)
            } else {
                gap
            };
            return Moved {
                distance: gap,
                stop: Some(Stop {
                    side: heading,
                    blocker: Blocker::Enemy { touching },
                }),
                seat: 0,
            };
        }
        let (x, r) = (body.position.x.pixel(), body.radius.x);
        let target = body.position.x.plus(distance).pixel();
        let rightward = !distance.negative();
        let edge = if rightward {
            target.wrapping_add(r).wrapping_sub(1)
        } else {
            target.wrapping_sub(r)
        };
        if !self.column_solid(edge, body) {
            return full(distance);
        }
        if rightward {
            let pixels = (edge & !0xF).wrapping_sub(r).wrapping_sub(x);
            block(heading, not_below_zero(pixels), 0xFFFF)
        } else {
            let pixels = (edge | 0xF).wrapping_add(1).wrapping_add(r).wrapping_sub(x);
            block(
                heading,
                if pixels.cast_signed() > 0 { 0 } else { pixels },
                0,
            )
        }
    }
}

impl Collision for Flat {
    fn move_horizontal(
        &mut self,
        body: Body,
        heading: Heading,
        distance: Fixed,
        enemies: Enemies,
    ) -> Moved {
        let seat = match enemies {
            Enemies::Test => self.slope_seat,
            Enemies::Skip => 0,
        };
        Moved {
            seat,
            ..self.sideways(body, heading, distance, enemies)
        }
    }

    fn move_down(&mut self, body: Body, _: Heading, distance: Fixed, enemies: Enemies) -> Moved {
        if self.solid_enemy_floor && matches!(enemies, Enemies::Test) {
            return Moved {
                distance: Fixed::ZERO,
                stop: Some(Stop {
                    side: Heading::Down,
                    blocker: Blocker::Enemy { touching: true },
                }),
                seat: 0,
            };
        }
        let (y, r) = (body.position.y.pixel(), body.radius.y);
        let edge = body
            .position
            .y
            .plus(distance)
            .pixel()
            .wrapping_add(r)
            .wrapping_sub(1);
        if !self.row_solid(edge, body) {
            return full(distance);
        }
        let pixels = not_below_zero((edge & !0xF).wrapping_sub(r).wrapping_sub(y));
        block(Heading::Down, pixels, 0xFFFF)
    }

    fn move_up(&mut self, body: Body, distance: Fixed) -> Moved {
        let (y, r) = (body.position.y.pixel(), body.radius.y);
        let edge = body.position.y.minus(distance).pixel().wrapping_sub(r);
        if !self.row_solid(edge, body) {
            return full(Fixed::ZERO.minus(distance));
        }
        let pixels = (edge | 0xF).wrapping_add(1).wrapping_add(r).wrapping_sub(y);
        block(
            Heading::Up,
            if pixels.cast_signed() > 0 { 0 } else { pixels },
            0,
        )
    }

    fn enemy_ahead(&mut self, body: Body, heading: Heading, distance: Fixed) -> Option<EnemyHit> {
        self.enemy(body, heading, distance)
            .map(|(touching, _)| EnemyHit { touching })
    }

    fn probe_wall(&mut self, body: Body, heading: Heading, reach: u16) -> Option<WallHit> {
        // Solid enemies first: the move's own enemy test, over the reach.
        if let Some((touching, gap)) = self.enemy(body, heading, Fixed::new(reach, 0)) {
            return Some(WallHit {
                distance_px: gap.pixel(),
                subpixel: None,
                touching,
            });
        }
        let (x, r) = (body.position.x.pixel(), body.radius.x);
        if matches!(heading, Heading::Right) {
            let edge = x.wrapping_add(reach).wrapping_add(r).wrapping_sub(1);
            self.column_solid(edge, body).then(|| WallHit {
                distance_px: (edge & !0xF).wrapping_sub(r).wrapping_sub(x),
                subpixel: Some(0xFFFF),
                touching: false,
            })
        } else {
            let edge = x.wrapping_sub(reach).wrapping_sub(r);
            self.column_solid(edge, body).then(|| WallHit {
                distance_px: x.wrapping_sub(r).wrapping_sub((edge | 0xF).wrapping_add(1)),
                subpixel: Some(0),
                touching: false,
            })
        }
    }
}

fn fall_kind(name: &str) -> FallKind {
    match name {
        "Airborne" => FallKind::Airborne,
        "MorphBall" => FallKind::MorphBall,
        "SpringBall" => FallKind::SpringBall,
        "NoChange" => FallKind::NoChange,
        other => panic!("unknown fall kind {other}"),
    }
}

fn land_kind(name: &str) -> LandKind {
    match name {
        "Grounded" => LandKind::Grounded,
        "MorphBall" => LandKind::MorphBall,
        "SpringBall" => LandKind::SpringBall,
        "NoChange" => LandKind::NoChange,
        other => panic!("unknown land kind {other}"),
    }
}

fn result(name: &str) -> VerticalResult {
    let inner = |prefix: &str| {
        name.strip_prefix(prefix)
            .and_then(|rest| rest.strip_suffix(')'))
    };
    match name {
        "None" => VerticalResult::None,
        "HitCeiling" => VerticalResult::HitCeiling,
        "WallJump" => VerticalResult::WallJump,
        _ => inner("Landed(").map_or_else(
            || VerticalResult::Falling(fall_kind(inner("Falling(").unwrap())),
            |kind| VerticalResult::Landed(land_kind(kind)),
        ),
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

fn vertical(name: &str) -> VerticalMotion {
    match name {
        "None" => VerticalMotion::None,
        "Up" => VerticalMotion::Up,
        "Down" => VerticalMotion::Down,
        other => panic!("unknown vertical {other}"),
    }
}

fn bounce(name: &str) -> Bounce {
    match name {
        "None" => Bounce::None,
        "First" => Bounce::First,
        "Second" => Bounce::Second,
        other => panic!("unknown bounce {other}"),
    }
}

fn locomotion(name: &str) -> Locomotion {
    let ball = |spring, at_rest| Locomotion::Ball { spring, at_rest };
    let air = |spring, falling| Locomotion::BallInAir { spring, falling };
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
        "Jumping" => Locomotion::Jumping { take_off: false },
        "JumpTakeOff" => Locomotion::Jumping { take_off: true },
        "SpinJumping" => Locomotion::SpinJumping,
        "Falling" => Locomotion::Falling,
        "BallFalling" => air(false, true),
        "SpringBallFalling" => air(true, true),
        "SpringBallInAir" => air(true, false),
        "WallJumping" => Locomotion::WallJumping,
        "TurningInAir" => Locomotion::TurningInAir,
        other => panic!("unknown locomotion {other}"),
    }
}

fn pad(step: &Value) -> Pad {
    let held: Vec<&str> = step
        .get("pad")
        .and_then(Value::as_array)
        .map(|buttons| buttons.iter().map(|b| b.as_str().unwrap()).collect())
        .unwrap_or_default();
    Pad {
        run: held.contains(&"run"),
        shoot: held.contains(&"shoot"),
        jump: match (held.contains(&"jump"), flag(step, "jump_pressed")) {
            (_, true) => JumpButton::Pressed,
            (true, false) => JumpButton::Held,
            (false, false) => JumpButton::Released,
        },
        sideways: Sideways {
            left: held.contains(&"left"),
            right: held.contains(&"right"),
        },
    }
}

fn equipment(step: &Value) -> Equipment {
    Equipment {
        hi_jump: flag(step, "hi_jump"),
        space_jump: flag(step, "space_jump"),
    }
}

fn inputs(step: &Value, name: &str) -> Inputs {
    let radius = step["radius"].as_array().unwrap();
    Inputs {
        locomotion: locomotion(name),
        facing: match text(step, "facing").unwrap() {
            "Left" => Facing::Left,
            "Right" => Facing::Right,
            "Forward" => Facing::Forward,
            other => panic!("unknown facing {other}"),
        },
        radius: Radius {
            x: word(&radius[0]),
            y: word(&radius[1]),
        },
        pad: pad(step),
        spin: Spin {
            ready: flag(step, "spin_ready"),
            was_spinning: flag(step, "was_spinning"),
        },
        equipment: equipment(step),
        elevator_moving: flag(step, "elevator_moving"),
        on_slope: flag(step, "on_slope"),
    }
}

fn pose_change(step: &Value, vertical_result: VerticalResult) -> Option<PoseChange> {
    let start = text(step, "pose_start").map(|name| match name {
        "Turn" => PoseStart::Turn,
        "BallReverse" => PoseStart::BallReverse,
        "SpinReverse" => PoseStart::SpinReverse,
        "JumpOrFall" => PoseStart::JumpOrFall,
        "TakeOff" => PoseStart::TakeOff {
            from_plain_crouch: false,
        },
        "TakeOffFromPlainCrouch" => PoseStart::TakeOff {
            from_plain_crouch: true,
        },
        "SpinStart" => PoseStart::SpinStart,
        "SpringLaunch" => PoseStart::SpringLaunch,
        "WallJumpStart" => PoseStart::WallJumpStart,
        other => panic!("unknown pose start {other}"),
    });
    let command = text(step, "pose_command").map(|name| match name {
        "Decelerate" => PoseCommand::Decelerate,
        "Stop" => PoseCommand::Stop,
        "KillXSpeed" => PoseCommand::KillXSpeed,
        "KillRunSpeed" => PoseCommand::KillRunSpeed,
        "AnimationEnd" => PoseCommand::AnimationEnd,
        "Vertical" => PoseCommand::Vertical(vertical_result),
        other => panic!("unknown pose command {other}"),
    });
    (start.is_some() || command.is_some() || flag(step, "pose_replaced")).then(|| PoseChange {
        start,
        replaced: flag(step, "pose_replaced"),
        command: command.unwrap_or(PoseCommand::None),
        started_falling: flag(step, "new_pose_falling") && !flag(step, "old_pose_falling"),
    })
}

fn before(table: &Value) -> State {
    let position = table["position"].as_array().unwrap();
    let speed = |key| table.get(key).map_or(Fixed::ZERO, fixed);
    State {
        position: Position {
            x: fixed(&position[0]),
            y: fixed(&position[1]),
        },
        base_speed: speed("base_speed"),
        extra_run: speed("extra_run"),
        momentum: flag(table, "momentum"),
        mode: text(table, "mode").map_or(SpeedMode::Accelerating, mode),
        vertical_speed: speed("vertical_speed"),
        vertical: text(table, "vertical").map_or(VerticalMotion::None, vertical),
        bounce: text(table, "bounce").map_or(Bounce::None, bounce),
    }
}

fn ps(value: Fixed) -> String {
    format!("{:04X}.{:04X}", value.pixel(), value.sub())
}

/// Every difference of one step's results from its expectations, as "field: expected, actual".
fn differences(
    step: &Value,
    samus: &State,
    contacts: Contacts,
    ran_into_wall: bool,
) -> Vec<String> {
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
            ("vertical_speed", samus.vertical_speed),
        ];
        for (key, actual) in speeds {
            if let Some(expected) = after.get(key) {
                check(key, ps(fixed(expected)), ps(actual));
            }
        }
        if let Some(expected) = after.get("momentum") {
            check(
                "momentum",
                format!("{:?}", expected.as_bool()),
                format!("{:?}", Some(samus.momentum)),
            );
        }
        let named = [
            (
                "mode",
                text(after, "mode").map(|m| format!("{:?}", mode(m))),
                format!("{:?}", samus.mode),
            ),
            (
                "vertical",
                text(after, "vertical").map(|v| format!("{:?}", vertical(v))),
                format!("{:?}", samus.vertical),
            ),
            (
                "bounce",
                text(after, "bounce").map(|b| format!("{:?}", bounce(b))),
                format!("{:?}", samus.bounce),
            ),
        ];
        for (key, expected, actual) in named {
            if let Some(expected) = expected {
                check(key, expected, actual);
            }
        }
    }
    if let Some(expected) = step.get("contacts") {
        let flags = [
            ("wall_stop", contacts.wall_stop),
            ("ran_into_wall", ran_into_wall),
            ("wall_jump_armed", contacts.wall_jump_armed),
            (
                "restart_standing_animation",
                contacts.restart_standing_animation,
            ),
        ];
        for (key, actual) in flags {
            if let Some(value) = expected.get(key) {
                check(
                    key,
                    format!("{:?}", value.as_bool()),
                    format!("{:?}", Some(actual)),
                );
            }
        }
        if let Some(value) = text(expected, "vertical_result") {
            check(
                "vertical_result",
                format!("{:?}", result(value)),
                format!("{:?}", contacts.vertical),
            );
        }
    }
    found
}

/// Runs one step of a vector: `step`, `arm_pump_step`, then `pose_writes`, as spec §8 orders them.
fn run(step: &Value, samus: &mut State, rules: &MotionRules, world: &mut Flat) -> (Contacts, bool) {
    // A pose-only step: its vertical result is given, not run.
    let given = || {
        let vertical = text(step, "vertical_result").map_or(VerticalResult::None, result);
        let contacts = Contacts {
            vertical,
            ..Contacts::default()
        };
        (contacts, false)
    };
    let moves = |name| {
        let input = inputs(step, name);
        let contacts = samus.step(&input, rules, world).unwrap();
        // AnimationEndPose: the hand-over is a forced pose to the arm-pump step.
        let forced =
            flag(step, "forced_pose") || text(step, "pose_command") == Some("AnimationEnd");
        let request = if forced {
            PoseRequest::Forced
        } else if flag(step, "new_running_pose") {
            PoseRequest::Pose { running: true }
        } else {
            PoseRequest::None
        };
        let ran = samus.arm_pump_step(&input, contacts.wall_stop, request, rules, world);
        (contacts, ran)
    };
    let (contacts, ran_into_wall) = text(step, "locomotion").map_or_else(given, moves);
    if let Some(change) = pose_change(step, contacts.vertical) {
        samus.pose_writes(
            change,
            !matches!(pad(step).jump, JumpButton::Released),
            equipment(step),
            rules,
        );
    }
    (contacts, ran_into_wall)
}

#[test]
fn every_vector_passes() {
    let file: Value = VECTORS.parse().unwrap();
    let vectors = file["vector"].as_array().unwrap();
    let speeds = speeds();
    let mut failures = Vec::new();
    for vector in vectors {
        let id = vector["id"].as_str().unwrap();
        let speed_cap = match vector
            .get("rules")
            .and_then(|rules| text(rules, "speed_cap"))
        {
            Some("Exact") => SpeedCap::Exact,
            Some("Wordwise") | None => SpeedCap::Wordwise,
            Some(other) => panic!("unknown speed_cap {other}"),
        };
        let rules = MotionRules {
            speeds: &speeds,
            vertical: VERTICAL,
            ground: GroundMotion::VANILLA,
            air: AirMotion::VANILLA,
            speed_cap,
        };
        let mut world = Flat::of(&vector["world"]);
        let mut samus = before(&vector["before"]);
        for (frame, step) in vector["step"].as_array().unwrap().iter().enumerate() {
            let (contacts, ran_into_wall) = run(step, &mut samus, &rules, &mut world);
            for difference in differences(step, &samus, contacts, ran_into_wall) {
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
