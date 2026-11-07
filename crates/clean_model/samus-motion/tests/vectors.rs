//! Runs every vector in `scenarios/vectors/samus-motion.toml` through the model.

#[cfg(test)]
mod vectors {
    use clean_model_samus_motion::{
        Bounces, CATALOGUE, Contact, Frame, Met, Rules, Samus, Situation, SpeedCap, Turn,
        next_pose, pad_chart,
    };
    use clean_types::{
        Action, Aim, AirKind, Block, Body, Button, Cell, Equipment, Facing, Fixed, Heading, Hitbox,
        Pad, Parity, Pose, Position, Quarters, Radius, Signed, Slope, SlopeFaces, SolidKind, World,
    };
    use std::iter::Peekable;
    use std::str::Chars;

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/samus-motion.toml");

    /// The vectors in the file.
    const VECTOR_COUNT: usize = 158;
    /// The pose steps in the file.
    const POSE_STEP_COUNT: usize = 33;

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
        Fixed::from_parts(
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

    fn pad(value: &Value) -> Pad {
        value.array().iter().fold(Pad::default(), |pad, b| {
            pad.with(match b.str() {
                "run" => Button::Run,
                "jump" => Button::Jump,
                "shoot" => Button::Shoot,
                "left" => Button::Left,
                "right" => Button::Right,
                "up" => Button::Up,
                "down" => Button::Down,
                "aim_up" => Button::AimUp,
                "aim_down" => Button::AimDown,
                other => panic!("button {other:?}"),
            })
        })
    }

    /// A pose's body size, `radius = [x, y]`.
    fn radius(value: &Value) -> Radius {
        let (x, y) = pair(value.get("radius").expect("pose.radius"));
        Radius { x, y }
    }

    fn pose(value: &Value) -> Pose {
        let field = |k: &str| value.get(k).unwrap_or_else(|| panic!("pose.{k}")).str();
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
                "Moonwalking" => Action::Moonwalking,
                "Morphing" => Action::Morphing,
                "Rolling" => Action::Rolling,
                "Running" => Action::Running,
                "Spinning" => Action::Spinning,
                "StandingUp" => Action::StandingUp,
                "Still" => Action::Still,
                "TakingOff" => Action::TakingOff,
                "Turning" => Action::Turning,
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
            aim: match value.get("aim").map_or("Forward", Value::str) {
                "Forward" => Aim::Forward,
                "Up" => Aim::Up,
                "UpForward" => Aim::UpForward,
                "DownForward" => Aim::DownForward,
                "Down" => Aim::Down,
                other => panic!("aim {other:?}"),
            },
        }
    }

    /// The vectors' stand-in room: a floor from `floor_top` down, a wall from `wall_right`
    /// rightward, a ceiling above `ceiling_bottom`, single blocks at cells (sloped, solid or
    /// stepped), solid enemies, air elsewhere.
    struct Room {
        floor_top: Option<u32>,
        wall_right: Option<u32>,
        ceiling_bottom: Option<u32>,
        blocks: Vec<((u16, u16), Block)>,
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
                Block::Solid(SolidKind::Plain)
            } else {
                Block::Air(AirKind::Plain)
            }
        }
        fn lets_through(&self, _: Cell, _: Heading) -> bool {
            false
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

    /// A sloped block: its corner and its surface's 16 rows.
    fn slope(value: &Value, faces: SlopeFaces) -> ((u16, u16), Block) {
        let mut surface = [0_u8; 16];
        let rows = value.get("surface").expect("slope surface").array();
        assert_eq!(rows.len(), surface.len(), "slope surface");
        for (to, row) in surface.iter_mut().zip(rows) {
            *to = u8::try_from(row.int()).expect("surface row");
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
        let rows = value.get("quarters").expect("stepped quarters").array();
        let row = |v: &Value| {
            let solid: Vec<bool> = v.str().chars().map(|c| c == '#').collect();
            assert_eq!(solid.len(), 2, "stepped quarters row");
            [solid[0], solid[1]]
        };
        let cell = corner(value.get("block").expect("stepped block"));
        (
            cell,
            Block::Stepped(Quarters([row(&rows[0]), row(&rows[1])])),
        )
    }

    /// The world's room.
    fn room(world: &Value) -> Room {
        let face = |k: &str| world.get(k).map(|v| u32::try_from(v.int()).expect("face"));
        let mut blocks = Vec::new();
        if let Some(v) = world.get("slope_floor") {
            blocks.push(slope(v, SlopeFaces::Floor));
        }
        if let Some(v) = world.get("slope_ceiling") {
            blocks.push(slope(v, SlopeFaces::Ceiling));
        }
        if let Some(v) = world.get("solid_block") {
            blocks.push((corner(v), Block::Solid(SolidKind::Plain)));
        }
        if let Some(v) = world.get("stepped") {
            blocks.push(stepped(v));
        }
        let enemies = world
            .get("enemy")
            .map(|e| {
                let (x, y) = pair(e.get("centre").expect("enemy centre"));
                let (rx, ry) = pair(e.get("radius").expect("enemy radius"));
                let radius = Radius { x: rx, y: ry };
                Hitbox { x, y, radius }
            })
            .into_iter()
            .collect();
        Room {
            floor_top: face("floor_top"),
            wall_right: face("wall_right"),
            ceiling_bottom: face("ceiling_bottom"),
            blocks,
            enemies,
        }
    }

    /// One frame's inputs: `base` with the frame's pad, body size (pose fit's, out of scope) and
    /// whether its pose's animation ends. Animation timing is out of scope too: a vector says an
    /// animation ends by the pose that follows one that lasts only as long as its animation (spec
    /// `AnimationEnds`).
    fn frame_inputs(
        base: Frame,
        (start, radius, pad, jump_pressed): (Pose, Radius, Pad, bool),
        end: Option<Pose>,
    ) -> Frame {
        let animated = matches!(
            start.action,
            Action::TakingOff
                | Action::Turning
                | Action::Crouching
                | Action::StandingUp
                | Action::Morphing
                | Action::Unmorphing
        );
        let pressed = if jump_pressed {
            Pad::default().with(Button::Jump)
        } else {
            Pad::default()
        };
        Frame {
            pad,
            pressed,
            animation_ends: animated && end.is_some_and(|e| e.action != start.action),
            radius,
            ..base
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
        let equipment = vector.get("equipment");
        let equipment = Equipment {
            hi_jump: flag(equipment, "hi_jump"),
            space_jump: flag(equipment, "space_jump"),
        };
        (rules, equipment)
    }

    /// Runs one vector; returns its mismatches.
    fn run(vector: &Value) -> Vec<String> {
        let id = vector.get("id").expect("id").str();
        let (rules, equipment) = settings(vector);
        let world = vector.get("world").expect("world");
        let room = room(world);
        let elevator_moving = flag(Some(world), "elevator_moving");
        let checked_parity = match vector.get("frame").map_or("Even", Value::str) {
            "Even" => Parity::Even,
            "Odd" => Parity::Odd,
            other => panic!("{id}: frame {other:?}"),
        };

        // The frames: the lead-in's, then the checked one, each with the pose it starts in, its
        // body size (pose fit's, out of scope) and the pad.
        let frame_of = |f: &Value| {
            let start = f.get("pose").expect("pose");
            (
                pose(start),
                radius(start),
                pad(f.get("pad").expect("pad")),
                f.get("jump_pressed").expect("jump_pressed").bool(),
            )
        };
        let mut frames: Vec<(Pose, Radius, Pad, bool)> = vector
            .get("lead_in")
            .map(Value::array)
            .unwrap_or_default()
            .iter()
            .map(frame_of)
            .collect();
        frames.push(frame_of(vector));

        let before = vector.get("before").expect("before");
        let quantity = |k: &str| {
            before
                .get(k)
                .map_or_else(Signed::default, |v| signed(v.str()))
        };
        let position = before.get("position").expect("before.position").array();
        let bounces = match before.get("bounces").map_or(0, Value::int) {
            0 => Bounces::None,
            1 => Bounces::Once,
            _ => Bounces::Twice,
        };
        let (first_pose, first_radius, _, _) = frames[0];
        let mut samus = Samus::start(
            Position {
                x: fixed(position[0].str()),
                y: fixed(position[1].str()),
            },
            (
                quantity("speed_x"),
                quantity("dash").size,
                quantity("speed_y"),
            ),
            bounces,
            (first_pose, first_radius),
        );
        // The pose each frame ends in, where the vector says it: the next frame's pose, and the
        // checked frame's next_pose.
        let last_next = vector.get("next_pose").map(pose);
        let ends: Vec<Option<Pose>> = frames
            .iter()
            .skip(1)
            .map(|f| Some(f.0))
            .chain([last_next])
            .collect();
        // The lead-in's frames alternate back from the checked frame's parity.
        let mut parity = if frames.len() % 2 == 1 {
            checked_parity
        } else {
            checked_parity.other()
        };
        let base = Frame {
            pad: Pad::default(),
            pressed: Pad::default(),
            equipment,
            spin_turned_far: false,
            elevator_moving,
            parity,
            animation_ends: false,
            radius: first_radius,
        };
        let mut misses = Vec::new();
        for (n, (&inputs, end)) in frames.iter().zip(&ends).enumerate() {
            let frame = frame_inputs(Frame { parity, ..base }, inputs, *end);
            parity = parity.other();
            samus.step(&frame, &room, &rules);
            // The first pose that differs; the later ones follow from it.
            if let Some(expected) = end
                && samus.pose != *expected
                && misses.is_empty()
            {
                misses.push(format!(
                    "{id}: frame {n} ends in pose {:?}, expected {expected:?}",
                    samus.pose
                ));
            }
        }
        misses.extend(compare(id, vector.get("after").expect("after"), &samus));
        misses
    }

    /// Runs one pose step through the chart; returns its mismatch.
    fn run_pose_step(step: &Value) -> Option<String> {
        let id = step.get("id").expect("id").str();
        let start = pose(step.get("pose").expect("pose"));
        let held = pad(step.get("pad").expect("pad"));
        let mut pressed = pad(step.get("pressed").expect("pressed"));
        if step.get("jump_pressed").expect("jump_pressed").bool() {
            pressed = pressed.with(Button::Jump);
        }
        let mut met = Met::default();
        let mut animation_ends = false;
        match step.get("met").expect("met").str() {
            "nothing" => {}
            "floor" => met.contact = Some(Contact::Floor),
            "no_floor" => met.contact = Some(Contact::NoFloor),
            "wall" => met.wall = true,
            "wall_kick" => met.contact = Some(Contact::WallKick),
            "animation_end" => animation_ends = true,
            other => panic!("{id}: met {other:?}"),
        }
        // A step gives no vertical motion: she is off the ground once her moves are made in
        // the poses of the air and after walking off a ledge, and a turn is on the ground.
        let airborne = met.contact == Some(Contact::NoFloor)
            || (met.contact != Some(Contact::Floor)
                && matches!(
                    start.action,
                    Action::Jumping | Action::Spinning | Action::Falling | Action::WallJumping
                ));
        let situation = Situation {
            pose: start,
            airborne,
            turn: Turn::Ground,
        };
        let chart = pad_chart(situation, held, pressed);
        let chosen = next_pose(situation, chart, met, animation_ends).pose;
        let after = step.get("after").expect("after");
        let expected = pose(after.get("pose").expect("after.pose"));
        (chosen != expected).then(|| format!("{id}: pose expected {expected:?}, actual {chosen:?}"))
    }

    /// The listed `after` quantities that differ.
    fn compare(id: &str, after: &Value, samus: &Samus) -> Vec<String> {
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
            let sign = if s.negative && !s.size.is_zero() {
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
                    show_signed(samus.speed_x_signed()),
                ),
                "speed_y" => check(
                    "speed_y",
                    show_signed(signed(value.str())),
                    show_signed(samus.speed_y_signed()),
                ),
                "dash" => check("dash", show(fixed(value.str())), show(samus.dash.size)),
                "bounces" => {
                    let actual = match samus.bounces {
                        Bounces::None => 0,
                        Bounces::Once => 1,
                        Bounces::Twice => 2,
                    };
                    check("bounces", value.int().to_string(), actual.to_string());
                }
                other => panic!("{id}: after.{other}"),
            }
        }
        misses
    }

    #[test]
    fn every_vector() {
        let vectors = tables(VECTORS, "vector");
        assert_eq!(vectors.len(), VECTOR_COUNT, "vector count");
        let misses: Vec<String> = vectors.iter().flat_map(run).collect();
        assert!(
            misses.is_empty(),
            "{} mismatches:\n{}",
            misses.len(),
            misses.join("\n")
        );
    }

    #[test]
    fn every_pose_step() {
        let steps = tables(VECTORS, "pose_step");
        assert_eq!(steps.len(), POSE_STEP_COUNT, "pose step count");
        let misses: Vec<String> = steps.iter().filter_map(run_pose_step).collect();
        assert!(
            misses.is_empty(),
            "{} mismatches:\n{}",
            misses.len(),
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
        let unexercised: Vec<&str> = ids
            .iter()
            .map(String::as_str)
            .filter(|id| !exercised.contains(id))
            .collect();
        assert!(
            unexercised.is_empty(),
            "entries no vector exercises: {unexercised:?}"
        );
    }
}
