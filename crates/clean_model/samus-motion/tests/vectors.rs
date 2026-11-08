//! Runs every vector in `scenarios/vectors/samus-motion.toml` through the model.

#![cfg(test)]

use clean_model_samus_motion::{
    Bounces, CATALOGUE, Carry, DashIs, Frame, Met, Moment, Outcome, Rules, Run, Samus, Seat,
    SpeedCap, Spin, Vertical, WallTest, classify,
};
use clean_types::{
    Action, Aim, Block, Body, Buttons, Cell, Equipment, Facing, Fixed, Gun, Heading, Hitbox,
    LandedFrom, Legs, Pair, Parity, Parts, Pose, Position, Radius, Slope, SlopeFaces, Surface,
    Touch, Turn, Way, World, ZERO, button, fixed as quantity,
};

/// A signed `P.S` quantity as the vectors write it (the header's notation): a size and a sign.
#[derive(Clone, Copy)]
struct Signed {
    /// Leftward or upward.
    negative: bool,
    /// The size.
    size: Fixed,
}

/// Her horizontal speed as the player reads it, rightward positive.
const fn speed_x(samus: &Samus) -> Signed {
    Signed {
        negative: matches!(samus.run.way, Heading::Left),
        size: samus.run.speed,
    }
}

/// Her vertical speed as the player reads it, downward positive.
fn speed_y(samus: &Samus) -> Signed {
    Signed {
        negative: samus.vy.negative(),
        size: samus.vy.size(),
    }
}
use std::iter::Peekable;
use std::str::Chars;

const VECTORS: &str = include_str!("../../../../scenarios/vectors/samus-motion.toml");

/// How many tables of a kind the file holds, counted from its `[[kind]]` headers, apart from
/// the parser.
fn count_in_file(kind: &str) -> usize {
    let header = format!("[[{kind}]]");
    VECTORS.lines().filter(|l| l.trim() == header).count()
}

// ---- A parser for the subset of TOML the vector file uses.

#[derive(Clone, Debug)]
enum Value {
    Str(String),
    Int(i64),
    Bool(bool),
    Array(Vec<Self>),
    Table(Vec<(String, Self)>),
}

impl Value {
    fn get(&self, key: &str) -> Option<&Self> {
        self.pairs_or_none()
            .and_then(|pairs| pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v))
    }
    fn pairs_or_none(&self) -> Option<&[(String, Self)]> {
        if let Self::Table(pairs) = self {
            Some(pairs)
        } else {
            None
        }
    }
    fn pairs(&self) -> &[(String, Self)] {
        self.pairs_or_none()
            .unwrap_or_else(|| panic!("not a table: {self:?}"))
    }
    fn str(&self) -> &str {
        if let Self::Str(s) = self {
            s
        } else {
            panic!("not a string: {self:?}")
        }
    }
    fn int(&self) -> i64 {
        if let Self::Int(n) = self {
            *n
        } else {
            panic!("not an integer: {self:?}")
        }
    }
    fn bool(&self) -> bool {
        if let Self::Bool(b) = self {
            *b
        } else {
            panic!("not a bool: {self:?}")
        }
    }
    fn array(&self) -> &[Self] {
        if let Self::Array(items) = self {
            items
        } else {
            panic!("not an array: {self:?}")
        }
    }
}

struct Parser<'a> {
    chars: Peekable<Chars<'a>>,
}

impl Parser<'_> {
    fn skip_space(&mut self) {
        while self.chars.next_if(|c| c.is_whitespace()).is_some() {}
    }
    fn expect(&mut self, want: char) {
        self.skip_space();
        let got = self.chars.next();
        assert_eq!(got, Some(want), "expected {want:?}");
    }
    fn word(&mut self) -> String {
        self.skip_space();
        let mut word = String::new();
        while let Some(c) = self
            .chars
            .next_if(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-' || *c == '+')
        {
            word.push(c);
        }
        word
    }
    fn value(&mut self) -> Value {
        self.skip_space();
        match self.chars.peek().copied() {
            Some('"') => {
                self.chars.next();
                let mut s = String::new();
                while let Some(c) = self.chars.next_if(|c| *c != '"') {
                    s.push(c);
                }
                self.expect('"');
                Value::Str(s)
            }
            Some('[') => {
                self.chars.next();
                let mut items = Vec::new();
                loop {
                    self.skip_space();
                    if self.chars.next_if_eq(&']').is_some() {
                        break Value::Array(items);
                    }
                    items.push(self.value());
                    self.skip_space();
                    self.chars.next_if_eq(&',');
                }
            }
            Some('{') => {
                self.chars.next();
                let mut pairs = Vec::new();
                loop {
                    self.skip_space();
                    if self.chars.next_if_eq(&'}').is_some() {
                        break Value::Table(pairs);
                    }
                    let key = self.word();
                    self.expect('=');
                    pairs.push((key, self.value()));
                    self.skip_space();
                    self.chars.next_if_eq(&',');
                }
            }
            _ => match self.word().as_str() {
                "true" => Value::Bool(true),
                "false" => Value::Bool(false),
                n => Value::Int(n.parse().unwrap_or_else(|_| panic!("bad value {n:?}"))),
            },
        }
    }
}

/// The tables of one kind (`vector` or `pose_step`), each as a `Value::Table`.
fn tables(text: &str, kind: &str) -> Vec<Value> {
    parse(text)
        .into_iter()
        .filter(|(k, _)| k == kind)
        .map(|(_, v)| v)
        .collect()
}

/// The `[[...]]` tables, each with its kind.
fn parse(text: &str) -> Vec<(String, Value)> {
    // A comment runs from a '#' outside a string to the line's end.
    let uncommented = |line: &str| {
        let mut quoted = false;
        let end = line
            .char_indices()
            .find(|&(_, c)| {
                quoted ^= c == '"';
                c == '#' && !quoted
            })
            .map_or(line.len(), |(at, _)| at);
        line[..end].to_owned()
    };
    let stripped: String = text.lines().map(uncommented).collect::<Vec<_>>().join("\n");
    let mut parser = Parser {
        chars: stripped.chars().peekable(),
    };
    let mut tables: Vec<(String, Vec<(String, Value)>)> = Vec::new();
    loop {
        parser.skip_space();
        if parser.chars.peek().is_none() {
            break;
        }
        if parser.chars.next_if_eq(&'[').is_some() {
            parser.expect('[');
            let kind = parser.word();
            assert!(kind == "vector" || kind == "pose_step", "table {kind:?}");
            parser.expect(']');
            parser.expect(']');
            tables.push((kind, Vec::new()));
            continue;
        }
        let key = parser.word();
        parser.expect('=');
        let value = parser.value();
        tables
            .last_mut()
            .expect("a key before any table")
            .1
            .push((key, value));
    }
    tables
        .into_iter()
        .map(|(kind, pairs)| (kind, Value::Table(pairs)))
        .collect()
}

// ---- Reading the vectors' quantities.

fn fixed(text: &str) -> Fixed {
    let (pixel, sub) = text.split_once('.').expect("P.S");
    quantity(
        u16::from_str_radix(pixel, 16).expect("pixel part"),
        u16::from_str_radix(sub, 16).expect("subpixel part"),
    )
}

fn signed(text: &str) -> Signed {
    let negative = text.starts_with('-');
    Signed {
        negative,
        size: fixed(text.trim_start_matches(['-', '+'])),
    }
}

/// A button by the name the vectors give it.
fn named(name: &str) -> Buttons {
    match name {
        "run" => button::RUN,
        "jump" => button::JUMP,
        "shoot" => button::SHOOT,
        "left" => button::LEFT,
        "right" => button::RIGHT,
        "up" => button::UP,
        "down" => button::DOWN,
        "aim_up" => button::AIM_UP,
        "aim_down" => button::AIM_DOWN,
        other => panic!("button {other:?}"),
    }
}

/// A list of buttons, `pad` or `pressed`.
fn pad(value: &Value) -> Buttons {
    value
        .array()
        .iter()
        .fold(Buttons::default(), |pad, b| pad.union(named(b.str())))
}

/// The buttons a frame gives as just pressed: `pressed` (none if it is missing), and jump
/// as `jump_pressed` says.
fn pressed(frame: &Value) -> Buttons {
    let jump = frame.get("jump_pressed").expect("jump_pressed").bool();
    let others = frame.get("pressed").map_or_else(Buttons::default, pad);
    if jump {
        others.union(button::JUMP)
    } else {
        others
    }
}

/// A pose's body size, `radius = [x, y]`.
fn radius(value: &Value) -> Radius {
    let (x, y) = pair(value.get("radius").expect("pose.radius"));
    Pair { x, y }
}

/// A pose (spec §2, the header's `pose`): body, action, facing; aim (default `Forward`), gun
/// (default `Stowed`), turn (`Turning` only, default `Ground`), `landed_from` (`Landing` only)
/// and legs (default `Together`).
fn pose(value: &Value) -> Pose {
    known_keys(
        value,
        "pose",
        &[
            "body",
            "action",
            "facing",
            "radius",
            "aim",
            "gun",
            "turn",
            "landed_from",
            "legs",
        ],
    );
    let field = |k: &str| value.get(k).unwrap_or_else(|| panic!("pose.{k}")).str();
    let optional = |k: &str, default: &'static str| value.get(k).map_or(default, Value::str);
    let turn = match optional("turn", "Ground") {
        "Fall" => Turn::Fall,
        "Ground" => Turn::Ground,
        "Jump" => Turn::Jump,
        "MoonwalkJump" => Turn::MoonwalkJump,
        other => panic!("turn {other:?}"),
    };
    Pose {
        body: match field("body") {
            "Upright" => Body::Upright,
            "Crouched" => Body::Crouched,
            "Ball" => Body::Ball,
            "SpringBall" => Body::SpringBall,
            other => panic!("body {other:?}"),
        },
        action: match field("action") {
            "AgainstWall" => Action::AgainstWall,
            "Crouching" => Action::Crouching,
            "Falling" => Action::Falling,
            "Jumping" => Action::Jumping,
            "Landing" => Action::Landing(match field("landed_from") {
                "Jump" => LandedFrom::Jump,
                "Spin" => LandedFrom::Spin,
                other => panic!("landed_from {other:?}"),
            }),
            "Moonwalking" => Action::Moonwalking,
            "Morphing" => Action::Morphing,
            "Rolling" => Action::Rolling,
            "Running" => Action::Running,
            "Spinning" => Action::Spinning,
            "StandingUp" => Action::StandingUp,
            "Still" => Action::Still,
            "TakingOff" => Action::TakingOff,
            "Turning" => Action::Turning(turn),
            "Unmorphing" => Action::Unmorphing,
            "WallJumping" => Action::WallJumping,
            other => panic!("action {other:?}"),
        },
        facing: match field("facing") {
            "Left" => Facing::Left,
            "Right" => Facing::Right,
            "Screen" => Facing::Screen,
            other => panic!("facing {other:?}"),
        },
        aim: match optional("aim", "Forward") {
            "Forward" => Aim::Forward,
            "Up" => Aim::Up,
            "UpForward" => Aim::UpForward,
            "DownForward" => Aim::DownForward,
            "Down" => Aim::Down,
            other => panic!("aim {other:?}"),
        },
        gun: match optional("gun", "Stowed") {
            "Drawn" => Gun::Drawn,
            "Stowed" => Gun::Stowed,
            other => panic!("gun {other:?}"),
        },
        legs: match optional("legs", "Together") {
            "Striding" => Legs::Striding,
            "Together" => Legs::Together,
            other => panic!("legs {other:?}"),
        },
    }
}

/// The vectors' stand-in room: a floor from `floor_top` down, a wall from `wall_right`
/// rightward, a ceiling above `ceiling_bottom`, single blocks at cells (sloped, solid,
/// stepped or another kind), solid enemies, air elsewhere.
struct Room {
    floor_top: Option<u32>,
    wall_right: Option<u32>,
    ceiling_bottom: Option<u32>,
    blocks: Vec<((u16, u16), Block)>,
    /// The cells whose reaction lets her through this frame.
    through: Vec<(u16, u16)>,
    enemies: Vec<Hitbox>,
}

impl World for Room {
    fn block(&self, cell: Cell) -> Block {
        let Cell { column, row } = cell;
        if let Some(&(_, block)) = self.blocks.iter().find(|(at, _)| *at == (column, row)) {
            return block;
        }
        let from = |cell: u16, face: Option<u32>| {
            face.is_some_and(|face| u32::from(cell).wrapping_mul(16) >= face)
        };
        let above = |cell: u16, face: Option<u32>| {
            face.is_some_and(|face| u32::from(cell).wrapping_add(1).wrapping_mul(16) <= face)
        };
        if from(row, self.floor_top)
            || from(column, self.wall_right)
            || above(row, self.ceiling_bottom)
        {
            Block::Solid
        } else {
            Block::Air
        }
    }
    fn lets_through(&self, cell: Cell, _: Way) -> bool {
        self.through.contains(&(cell.column, cell.row))
    }
    fn solid_enemies(&self) -> &[Hitbox] {
        &self.enemies
    }
}

/// A pair of pixels, `[x, y]`.
fn pair(value: &Value) -> (u16, u16) {
    let xy = value.array();
    let px = |v: &Value| u16::try_from(v.int()).expect("pixel");
    (px(&xy[0]), px(&xy[1]))
}

/// The cell whose top-left corner is `[x, y]`.
fn corner(value: &Value) -> (u16, u16) {
    let (x, y) = pair(value);
    (x / 16, y / 16)
}

/// A sloped block: its corner and its surface at its 16 columns; 16 is an empty column (the
/// surface just past the far edge), 17 a column far past (spec `SlopesCarry`).
fn slope(value: &Value, faces: SlopeFaces) -> ((u16, u16), Block) {
    known_keys(value, "slope", &["block", "surface", "factor"]);
    let mut surface = [Surface::Row(0); 16];
    let rows = value.get("surface").expect("slope surface").array();
    assert_eq!(rows.len(), surface.len(), "slope surface");
    for (to, row) in surface.iter_mut().zip(rows) {
        *to = match row.int() {
            16 => Surface::JustPast,
            17 => Surface::FarPast,
            n => Surface::Row(u8::try_from(n).expect("surface row")),
        };
    }
    // The walk factor, "P.SS" in hex: a pixel and 1/256ths; 1.00 by default.
    let factor = value.get("factor").map_or(0x100, |f| {
        let (pixel, sub) = f.str().split_once('.').expect("factor");
        assert_eq!(sub.len(), 2, "factor");
        u16::from_str_radix(&format!("{pixel}{sub}"), 16).expect("factor")
    });
    let cell = corner(value.get("block").expect("slope block"));
    (
        cell,
        Block::Slope(Slope {
            faces,
            surface,
            factor,
        }),
    )
}

/// A stepped block: its corner and its quarters, upper row then lower, "#" solid.
fn stepped(value: &Value) -> ((u16, u16), Block) {
    known_keys(value, "stepped", &["block", "quarters"]);
    let rows = value.get("quarters").expect("stepped quarters").array();
    let row = |v: &Value| {
        let solid: Vec<bool> = v.str().chars().map(|c| c == '#').collect();
        assert_eq!(solid.len(), 2, "stepped quarters row");
        [solid[0], solid[1]]
    };
    let cell = corner(value.get("block").expect("stepped block"));
    (cell, Block::Stepped([row(&rows[0]), row(&rows[1])]))
}

/// One of the other kinds of cell (spec `OtherBlockKinds`): its corner, its kind, and
/// whether the world's answer lets her through this frame.
fn other_cell(value: &Value) -> ((u16, u16), Block, bool) {
    known_keys(value, "cell", &["block", "kind", "answer", "names"]);
    let through = value.get("answer").map_or("Solid", Value::str) == "Air";
    // OtherBlockKinds: each kind meets her motion as solid, as air, or as it answers.
    let block = match value.get("kind").expect("cell kind").str() {
        "DoorToRoom" | "AirWithSpikes" | "AirShotsActOn" | "AirBombsActOn" | "ReactingAir" => {
            Block::Air
        }
        "DoorToNowhere" | "Spikes" | "BreaksToShots" | "HoldsGrapple" => Block::Solid,
        "BreaksToBombs" | "ReactingBlock" => Block::Answered,
        "StandIn" => Block::StandsFor(value.get("names").map(|n| {
            let (column, row) = corner(n);
            Cell { column, row }
        })),
        other => panic!("cell kind {other:?}"),
    };
    (
        corner(value.get("block").expect("cell block")),
        block,
        through,
    )
}

/// The world's room.
fn room(world: &Value) -> Room {
    known_keys(
        world,
        "world",
        &[
            "floor_top",
            "wall_right",
            "ceiling_bottom",
            "slope_floor",
            "slope_ceiling",
            "solid_block",
            "stepped",
            "enemy",
            "elevator_moving",
            "cells",
        ],
    );
    let face = |k: &str| world.get(k).map(|v| u32::try_from(v.int()).expect("face"));
    let mut blocks = Vec::new();
    let mut through = Vec::new();
    for cell in world.get("cells").map(Value::array).unwrap_or_default() {
        let (at, block, lets_through) = other_cell(cell);
        blocks.push((at, block));
        if lets_through {
            through.push(at);
        }
    }
    // One sloped floor block, or a list of them (the header's format).
    match world.get("slope_floor") {
        Some(Value::Array(list)) => blocks.extend(list.iter().map(|v| slope(v, SlopeFaces::Floor))),
        Some(v) => blocks.push(slope(v, SlopeFaces::Floor)),
        None => {}
    }
    if let Some(v) = world.get("slope_ceiling") {
        blocks.push(slope(v, SlopeFaces::Ceiling));
    }
    if let Some(v) = world.get("solid_block") {
        blocks.push((corner(v), Block::Solid));
    }
    if let Some(v) = world.get("stepped") {
        blocks.push(stepped(v));
    }
    let enemies = world
        .get("enemy")
        .map(|e| {
            known_keys(e, "enemy", &["centre", "radius"]);
            let (x, y) = pair(e.get("centre").expect("enemy centre"));
            let (rx, ry) = pair(e.get("radius").expect("enemy radius"));
            Hitbox {
                centre: Pair { x, y },
                radius: Pair { x: rx, y: ry },
            }
        })
        .into_iter()
        .collect();
    Room {
        floor_top: face("floor_top"),
        wall_right: face("wall_right"),
        ceiling_bottom: face("ceiling_bottom"),
        blocks,
        through,
        enemies,
    }
}

/// The keys of a `[[vector]]` (the header's format).
const VECTOR_KEYS: &[&str] = &[
    "id",
    "source",
    "entries",
    "rules",
    "equipment",
    "world",
    "before",
    "lead_in",
    "pose",
    "next_pose",
    "frame",
    "pad",
    "jump_pressed",
    "pressed",
    "animation_ends",
    "after",
];

/// The keys of a `[[pose_step]]`.
const POSE_STEP_KEYS: &[&str] = &[
    "id",
    "source",
    "entries",
    "pose",
    "pad",
    "jump_pressed",
    "pressed",
    "met",
    "after",
];

/// Fails on a key the format doesn't give for this table, rather than skipping it.
fn known_keys(table: &Value, what: &str, keys: &[&str]) {
    for (key, _) in table.pairs() {
        assert!(keys.contains(&key.as_str()), "{what}: unknown key {key:?}");
    }
}

/// A flag in an optional table, false when missing.
fn flag(table: Option<&Value>, k: &str) -> bool {
    table.and_then(|t| t.get(k)).is_some_and(Value::bool)
}

/// A vector's rules and equipment.
fn settings(vector: &Value) -> (Rules, Equipment) {
    let rules = match vector
        .get("rules")
        .and_then(|r| r.get("speed_cap"))
        .map(Value::str)
    {
        None | Some("Wordwise") => Rules::REFERENCE,
        Some("Exact") => Rules {
            speed_cap: SpeedCap::Exact,
            ..Rules::REFERENCE
        },
        Some(other) => panic!("speed_cap {other:?}"),
    };
    if let Some(rules) = vector.get("rules") {
        known_keys(rules, "rules", &["speed_cap"]);
    }
    let equipment = vector.get("equipment");
    if let Some(e) = equipment {
        known_keys(
            e,
            "equipment",
            &[
                "hi_jump",
                "space_jump",
                "spring_ball",
                "screw_attack",
                "other_suit",
            ],
        );
    }
    let equipment = Equipment {
        hi_jump: flag(equipment, "hi_jump"),
        space_jump: flag(equipment, "space_jump"),
        spring_ball: flag(equipment, "spring_ball"),
        screw_attack: flag(equipment, "screw_attack"),
        other_suit: flag(equipment, "other_suit"),
    };
    (rules, equipment)
}

/// Samus at a vector's start (spec §1.2): her motion follows the pad unless she is skidding,
/// her dash is hers to keep, she remembers no spin and no seat, an upward speed is a rise,
/// and in a jump, a spin, a fall or a wall jump (a turn in a jump or a fall included) she is
/// in the air: a vertical speed of 0 there means she has just started to fall. In any other
/// pose she is on the ground, keeping any downward speed she has (`FollowingTheGround`).
fn start(before: &Value, pose: Pose, radius: Radius) -> Samus {
    known_keys(
        before,
        "before",
        &["position", "speed_x", "dash", "speed_y", "bounces"],
    );
    // A missing quantity is 0.
    let quantity = |k: &str| signed(before.get(k).map_or("+0000.0000", Value::str));
    let (speed_x, dash, speed_y) = (
        quantity("speed_x"),
        quantity("dash").size,
        quantity("speed_y"),
    );
    let xy = before.get("position").expect("before.position").array();
    let position = Position {
        x: fixed(xy[0].str()),
        y: fixed(xy[1].str()),
    };
    let bounces = match before.get("bounces").map_or(0, Value::int) {
        0 => Bounces::None,
        1 => Bounces::Once,
        _ => Bounces::Twice,
    };
    let facing = pose.facing.side();
    let way = if speed_x.size == ZERO {
        facing
    } else if speed_x.negative {
        Heading::Left
    } else {
        Heading::Right
    };
    let airborne = matches!(
        pose.action,
        Action::Jumping
            | Action::Spinning
            | Action::Falling
            | Action::WallJumping
            | Action::Turning(Turn::Jump | Turn::Fall)
    );
    let vertical = if speed_y.negative {
        Vertical::Rising
    } else if airborne {
        Vertical::Falling
    } else {
        Vertical::Ground
    };
    Samus {
        position,
        run: Run {
            speed: speed_x.size,
            way,
            carry: if way == facing {
                Carry::FollowsPad
            } else {
                Carry::CarriesOn
            },
            dash,
            dash_is: DashIs::Hers,
        },
        vy: if speed_y.negative {
            -speed_y.size
        } else {
            speed_y.size
        },
        vertical,
        bounces,
        pose,
        radius,
        spin: Spin::Off,
        seat: Seat::Unseated,
    }
}

/// One of a vector's frames: the pose it starts in and her body's size there, the buttons
/// held and those just pressed, and whether her pose's animation ends on it.
#[derive(Clone, Copy)]
struct Step {
    /// The pose at the frame's start.
    pose: Pose,
    /// Her body's size there.
    radius: Radius,
    /// The buttons held.
    pad: Buttons,
    /// The buttons just pressed.
    pressed: Buttons,
    /// `AnimationEnds`: the frame's own `animation_ends` (default false), never inferred.
    animation_ends: bool,
}

/// A pose with its body size, as the vectors give one.
type Sized = (Pose, Radius);

/// One frame of a vector or its lead-in.
fn step_of(f: &Value) -> Step {
    let start = f.get("pose").expect("pose");
    Step {
        pose: pose(start),
        radius: radius(start),
        pad: pad(f.get("pad").expect("pad")),
        pressed: pressed(f),
        animation_ends: flag(Some(f), "animation_ends"),
    }
}

/// A vector's frames: the lead-in's, then the checked one.
fn frames(vector: &Value) -> Vec<Step> {
    let lead_in = vector.get("lead_in").map(Value::array).unwrap_or_default();
    for f in lead_in {
        known_keys(
            f,
            "lead_in",
            &["pose", "pad", "jump_pressed", "pressed", "animation_ends"],
        );
    }
    lead_in.iter().chain([vector]).map(step_of).collect()
}

/// The frame parity a vector's first frame has: the lead-in's frames alternate back from
/// the checked frame's.
fn first_parity(vector: &Value, frames: usize) -> Parity {
    let checked = match vector.get("frame").map_or("Even", Value::str) {
        "Even" => Parity::Even,
        "Odd" => Parity::Odd,
        other => panic!("frame {other:?}"),
    };
    if frames % 2 == 1 {
        checked
    } else {
        other(checked)
    }
}

/// The other parity.
fn other(p: Parity) -> Parity {
    if p == Parity::Even {
        Parity::Odd
    } else {
        Parity::Even
    }
}

/// A pose's body size in a frame: the frame's own end where it names that pose (a record's
/// body can lag its pose, so the same pose may come with two sizes), else the frame's start,
/// else any the vector gives (spec §1.3: the game's data).
fn body_size(p: Pose, step: Step, end: Option<Sized>, known: &[Sized]) -> Radius {
    end.filter(|e| e.0 == p)
        .or_else(|| (step.pose == p).then_some((step.pose, step.radius)))
        .or_else(|| known.iter().find(|k| k.0 == p).copied())
        .map_or(step.radius, |k| k.1)
}

/// Runs one vector; returns its mismatches.
fn run(vector: &Value) -> Vec<String> {
    let id = vector.get("id").expect("id").str();
    known_keys(vector, id, VECTOR_KEYS);
    let (rules, equipment) = settings(vector);
    let world = vector.get("world").expect("world");
    let room = room(world);
    let elevator_moving = flag(Some(world), "elevator_moving");
    let frames = frames(vector);
    let mut samus = start(
        vector.get("before").expect("before"),
        frames[0].pose,
        frames[0].radius,
    );
    // The pose each frame ends in, where the vector says it: the next frame's pose, and the
    // checked frame's next_pose. Every pose the vector gives a body size for: each frame's
    // start, the checked frame's next_pose, and the pose it ends in.
    let last_next = vector.get("next_pose").map(|p| (pose(p), radius(p)));
    let ends: Vec<Option<Sized>> = frames
        .iter()
        .skip(1)
        .map(|f| Some((f.pose, f.radius)))
        .chain([last_next])
        .collect();
    let after = vector.get("after").expect("after");
    let mut known: Vec<Sized> = frames.iter().map(|f| (f.pose, f.radius)).collect();
    known.extend(last_next);
    known.extend(after.get("pose").map(|p| (pose(p), radius(p))));
    let mut parity = first_parity(vector, frames.len());
    let mut misses = Vec::new();
    let mut outcome = None;
    for (n, (&step, &end)) in frames.iter().zip(&ends).enumerate() {
        let body = |p: Pose| body_size(p, step, end, &known);
        let frame = Frame {
            pad: step.pad,
            pressed: step.pressed,
            equipment,
            spin_turned_far: false,
            elevator_moving,
            parity,
            animation_ends: step.animation_ends,
            body: &body,
        };
        parity = other(parity);
        outcome = Some(samus.step(&frame, &room, &rules));
        // The first pose that differs; the later ones follow from it. The checked frame's
        // after.pose, where given, is the pose it ends in instead of next_pose.
        let checked = n.wrapping_add(1) == frames.len();
        let expected = end
            .map(|e| e.0)
            .filter(|_| !(checked && after.get("pose").is_some()));
        if misses.is_empty() && expected.is_some_and(|e| e != samus.pose) {
            misses.push(format!(
                "{id}: frame {n} ends in pose {:?}, expected {expected:?}",
                samus.pose
            ));
        }
    }
    let outcome = outcome.expect("a checked frame");
    misses.extend(compare(id, after, &samus, &outcome));
    misses
}

/// Runs one pose step through the chart; returns its mismatch.
fn run_pose_step(step: &Value) -> Option<String> {
    let id = step.get("id").expect("id").str();
    known_keys(step, id, POSE_STEP_KEYS);
    let start = pose(step.get("pose").expect("pose"));
    let held = pad(step.get("pad").expect("pad"));
    let pressed = pressed(step);
    let (met, animation_ends) = match step.get("met").expect("met").str() {
        "nothing" => (None, false),
        "floor" => (Some(Met::Floor), false),
        "no_floor" => (Some(Met::NoFloor), false),
        "wall" => (Some(Met::Wall), false),
        "wall_kick" => (Some(Met::WallKick), false),
        "animation_end" => (None, true),
        other => panic!("{id}: met {other:?}"),
    };
    // A step gives no vertical motion: she is off the ground once her moves are made in
    // the poses of the air and after walking off a ledge, and a turn is on the ground.
    let airborne = met == Some(Met::NoFloor)
        || (met != Some(Met::Floor)
            && matches!(
                start.action,
                Action::Jumping | Action::Spinning | Action::Falling | Action::WallJumping
            ));
    // A step gives no speed either: her horizontal speed at the frame's start is 0.
    let moment = Moment {
        situation: classify(start, !airborne),
        pose: start,
        moving: false,
        airborne,
        pad: held,
        pressed,
        equipment: Equipment::default(),
        met,
    };
    let chosen = moment.choose(animation_ends).pose;
    let after = step.get("after").expect("after");
    known_keys(after, "after", &["pose"]);
    let expected = pose(after.get("pose").expect("after.pose"));
    (chosen != expected).then(|| format!("{id}: pose expected {expected:?}, actual {chosen:?}"))
}

/// A cell as the vectors write it, `[x, y]`, its top-left corner in px.
fn show_cell(cell: Cell) -> String {
    format!(
        "[{}, {}]",
        u32::from(cell.column) * 16,
        u32::from(cell.row) * 16
    )
}

/// A reached cell as the vectors write it.
fn show_touch(touch: Touch) -> String {
    let way = match touch.way {
        Way::Move(Heading::Left) => "Left",
        Way::Move(Heading::Right) => "Right",
        Way::Move(Heading::Up) => "Up",
        Way::Move(Heading::Down) => "Down",
        Way::Look => "Look",
    };
    format!("{} {way}", show_cell(touch.cell))
}

/// The reached cells a vector lists, as `show_touch` writes them.
fn listed_touches(value: &Value) -> String {
    let listed: Vec<String> = value
        .array()
        .iter()
        .map(|t| {
            known_keys(t, "reached", &["cell", "way"]);
            let (x, y) = pair(t.get("cell").expect("reached cell"));
            let way = t.get("way").expect("reached way").str();
            format!("[{x}, {y}] {way}")
        })
        .collect();
    listed.join("; ")
}

/// The cells a vector lists as she is inside, as `show_cell` writes them.
fn listed_cells(value: &Value) -> String {
    let cells: Vec<String> = value
        .array()
        .iter()
        .map(pair)
        .map(|(x, y)| format!("[{x}, {y}]"))
        .collect();
    cells.join("; ")
}

/// The listed `after` quantities that differ.
fn compare(id: &str, after: &Value, samus: &Samus, outcome: &Outcome) -> Vec<String> {
    let mut misses = Vec::new();
    let mut check = |field: &str, expected: String, actual: String| {
        if expected != actual {
            misses.push(format!(
                "{id}: {field} expected {expected}, actual {actual}"
            ));
        }
    };
    let show = |f: Fixed| format!("{:04X}.{:04X}", f.pixel(), f.sub());
    let show_signed = |s: Signed| {
        let sign = if s.negative && s.size != ZERO {
            '-'
        } else {
            '+'
        };
        format!("{sign}{}", show(s.size))
    };
    for (field, value) in after.pairs() {
        match field.as_str() {
            "position" => {
                let xy = value.array();
                check(
                    "position.x",
                    show(fixed(xy[0].str())),
                    show(samus.position.x),
                );
                check(
                    "position.y",
                    show(fixed(xy[1].str())),
                    show(samus.position.y),
                );
            }
            "position_x" => check(
                "position.x",
                show(fixed(value.str())),
                show(samus.position.x),
            ),
            "speed_x" => check(
                "speed_x",
                show_signed(signed(value.str())),
                show_signed(speed_x(samus)),
            ),
            "speed_y" => check(
                "speed_y",
                show_signed(signed(value.str())),
                show_signed(speed_y(samus)),
            ),
            "dash" => check("dash", show(fixed(value.str())), show(samus.run.dash)),
            "camera_from" => check(
                "camera_from",
                show(fixed(value.str())),
                show(outcome.camera_from),
            ),
            "reached" => {
                let actual: Vec<String> = outcome.reached.iter().copied().map(show_touch).collect();
                check("reached", listed_touches(value), actual.join("; "));
            }
            "inside" => {
                let actual: Vec<String> = outcome.inside.iter().copied().map(show_cell).collect();
                check("inside", listed_cells(value), actual.join("; "));
            }
            "bounces" => {
                let actual = match samus.bounces {
                    Bounces::None => 0,
                    Bounces::Once => 1,
                    Bounces::Twice => 2,
                };
                check("bounces", value.int().to_string(), actual.to_string());
            }
            // The pose the frame ends in, its body size included.
            "pose" => check(
                "pose",
                format!("{:?} {:?}", pose(value), radius(value)),
                format!("{:?} {:?}", samus.pose, samus.radius),
            ),
            "wall_test" => {
                let actual = match outcome.wall_test {
                    WallTest::Kicked => "Kicked",
                    WallTest::Neither => "Neither",
                    WallTest::Readied => "Readied",
                };
                check("wall_test", value.str().to_owned(), actual.to_owned());
            }
            other => panic!("{id}: after.{other} is not a key the format gives"),
        }
    }
    misses
}

#[test]
fn every_vector() {
    let vectors = tables(VECTORS, "vector");
    assert_eq!(vectors.len(), count_in_file("vector"), "vectors parsed");
    assert!(!vectors.is_empty(), "no vectors");
    let failing: Vec<Vec<String>> = vectors.iter().map(run).filter(|m| !m.is_empty()).collect();
    assert!(
        failing.is_empty(),
        "{} of {} vectors fail:\n{}",
        failing.len(),
        vectors.len(),
        failing.concat().join("\n")
    );
}

#[test]
fn every_pose_step() {
    let steps = tables(VECTORS, "pose_step");
    assert_eq!(steps.len(), count_in_file("pose_step"), "pose steps parsed");
    assert!(!steps.is_empty(), "no pose steps");
    let misses: Vec<String> = steps.iter().filter_map(run_pose_step).collect();
    assert!(
        misses.is_empty(),
        "{} of {} pose steps fail:\n{}",
        misses.len(),
        steps.len(),
        misses.join("\n")
    );
}

#[test]
fn catalogue_matches_the_vectors() {
    let ids: Vec<String> = CATALOGUE.iter().map(|b| format!("{:?}", b.id)).collect();
    for (n, id) in ids.iter().enumerate() {
        assert!(!ids[..n].contains(id), "{id} twice in the catalogue");
    }
    let vectors: Vec<Value> = parse(VECTORS).into_iter().map(|(_, v)| v).collect();
    let exercised: Vec<&str> = vectors
        .iter()
        .flat_map(|v| {
            v.get("entries")
                .expect("entries")
                .array()
                .iter()
                .map(Value::str)
        })
        .collect();
    for entry in &exercised {
        assert!(
            ids.iter().any(|id| id == entry),
            "{entry} is not in the catalogue"
        );
    }
    // An entry the spec checks on the original alone (its checked line names no oracle record
    // and no constructed vector) needs no vector.
    let unexercised: Vec<&str> = ids
        .iter()
        .map(String::as_str)
        .filter(|id| !exercised.contains(id) && !original_only(&checked_line(id)))
        .collect();
    assert!(
        unexercised.is_empty(),
        "entries no vector exercises: {unexercised:?}"
    );
}

/// The behaviour spec, for its entries' checked lines.
const SPEC: &str = include_str!("../../../../docs/behaviours/samus-motion.md");

/// An entry's `- checked:` line in the spec, its continuation lines joined.
fn checked_line(id: &str) -> String {
    let heading = format!("### {id}");
    let mut entry = SPEC
        .lines()
        .skip_while(|l| *l != heading)
        .skip(1)
        .take_while(|l| !l.starts_with("### ") && !l.starts_with("---"))
        .skip_while(|l| !l.starts_with("- checked:"));
    let first = entry
        .next()
        .unwrap_or_else(|| panic!("{id}: no checked line in the spec"));
    entry
        .take_while(|l| l.starts_with("  "))
        .fold(first.to_owned(), |line, more| line + " " + more.trim())
}

/// Whether a checked line checks on the original alone.
fn original_only(line: &str) -> bool {
    line.contains("original only") && !line.contains("oracle") && !line.contains("constructed")
}
