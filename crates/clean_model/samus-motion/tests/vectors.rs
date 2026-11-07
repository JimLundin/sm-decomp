//! Runs every vector in `scenarios/vectors/samus-motion.toml` through the model.

#[cfg(test)]
mod vectors {
    use clean_model_samus_motion::{Bounces, CATALOGUE, Frame, Rules, Samus, SpeedCap};
    use clean_types::{
        Action, Aim, Block, Body, Button, Equipment, Facing, Fixed, Hitbox, Pad, Pose, Position,
        Radius, Signed, World,
    };
    use std::iter::Peekable;
    use std::str::Chars;

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/samus-motion.toml");

    /// The vectors in the file.
    const VECTOR_COUNT: usize = 129;

    /// Entries no vector names: a coverage gap in the report (spec §11, question 3).
    const UNEXERCISED: [&str; 4] = [
        "RidingTheElevator",
        "Moonfall",
        "SolidEnemies",
        "SlopesCarry",
    ];

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

    /// The `[[vector]]` tables, each as a `Value::Table`.
    fn parse(text: &str) -> Vec<Value> {
        let stripped: String = text
            .lines()
            .map(|line| line.split('#').next().unwrap_or(""))
            .collect::<Vec<_>>()
            .join("\n");
        let mut parser = Parser {
            chars: stripped.chars().peekable(),
        };
        let mut tables: Vec<Vec<(String, Value)>> = Vec::new();
        loop {
            parser.skip_space();
            if parser.chars.peek().is_none() {
                break;
            }
            if parser.chars.next_if_eq(&'[').is_some() {
                parser.expect('[');
                assert_eq!(parser.word(), "vector");
                parser.expect(']');
                parser.expect(']');
                tables.push(Vec::new());
                continue;
            }
            let key = parser.word();
            parser.expect('=');
            let value = parser.value();
            tables
                .last_mut()
                .expect("a key before any [[vector]]")
                .push((key, value));
        }
        tables.into_iter().map(Value::Table).collect()
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

    fn pose(value: &Value) -> Pose {
        let field = |k: &str| value.get(k).unwrap_or_else(|| panic!("pose.{k}")).str();
        let radius = value.get("radius").expect("pose.radius").array();
        let px = |v: &Value| u16::try_from(v.int()).expect("radius");
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
            radius: Radius {
                x: px(&radius[0]),
                y: px(&radius[1]),
            },
        }
    }

    /// The vectors' stand-in room: a floor from `floor_top` down, a wall from `wall_right`
    /// rightward, a ceiling above `ceiling_bottom`, air elsewhere.
    struct Room {
        floor_top: Option<u32>,
        wall_right: Option<u32>,
        ceiling_bottom: Option<u32>,
    }

    impl World for Room {
        fn block(&self, column: u16, row: u16) -> Block {
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
        fn solid_enemies(&self) -> &[Hitbox] {
            &[]
        }
    }

    /// Runs one vector; returns its mismatches.
    fn run(vector: &Value) -> Vec<String> {
        let id = vector.get("id").expect("id").str();
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
        let flag =
            |table: Option<&Value>, k: &str| table.and_then(|t| t.get(k)).is_some_and(Value::bool);
        let equipment = vector.get("equipment");
        let equipment = Equipment {
            hi_jump: flag(equipment, "hi_jump"),
            space_jump: flag(equipment, "space_jump"),
        };
        let world = vector.get("world").expect("world");
        let face = |k: &str| world.get(k).map(|v| u32::try_from(v.int()).expect("face"));
        let room = Room {
            floor_top: face("floor_top"),
            wall_right: face("wall_right"),
            ceiling_bottom: face("ceiling_bottom"),
        };

        // The frames: the lead-in's, then the checked one, each with the pose it starts in.
        let mut frames: Vec<(Pose, Pad, bool)> = vector
            .get("lead_in")
            .map(Value::array)
            .unwrap_or_default()
            .iter()
            .map(|f| {
                (
                    pose(f.get("pose").expect("lead-in pose")),
                    pad(f.get("pad").expect("lead-in pad")),
                    f.get("jump_pressed").expect("lead-in jump_pressed").bool(),
                )
            })
            .collect();
        frames.push((
            pose(vector.get("pose").expect("pose")),
            pad(vector.get("pad").expect("pad")),
            vector.get("jump_pressed").expect("jump_pressed").bool(),
        ));

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
        let (first_pose, _, _) = frames[0];
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
            first_pose,
        );
        let last_next = vector.get("next_pose").map(pose);
        let nexts = frames.iter().skip(1).map(|f| Some(f.0)).chain([last_next]);
        for (&(_, pad, jump_pressed), next_pose) in frames.iter().zip(nexts) {
            let frame = Frame {
                pad,
                jump_pressed,
                equipment,
                next_pose,
                ..Frame::default()
            };
            samus.step(&frame, &room, &rules);
        }
        compare(id, vector.get("after").expect("after"), &samus)
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
        let vectors = parse(VECTORS);
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
    fn catalogue_matches_the_vectors() {
        let ids: Vec<String> = CATALOGUE.iter().map(|b| format!("{:?}", b.id)).collect();
        for (n, id) in ids.iter().enumerate() {
            assert!(!ids[..n].contains(id), "{id} twice in the catalogue");
        }
        let vectors = parse(VECTORS);
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
        assert_eq!(unexercised, UNEXERCISED, "entries no vector exercises");
    }
}
