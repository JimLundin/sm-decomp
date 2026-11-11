//! Runs every vector of `scenarios/vectors/samus-looks.toml` through the model. A key or a name
//! the file holds that this test doesn't know fails it.

#[cfg(test)]
mod vectors {
    use core::num::Wrapping;
    use std::collections::BTreeMap;
    use std::fmt::Debug;

    use clean_model_samus_looks::{BehaviourId, Rules};
    use clean_model_samus_looks::{
        Boost, Branch, CATALOGUE, Clip, Content, Control, Frames, Hold, Inputs, Instruction, Laps,
        Liquid, Move, Pace, Paces, Playhead, Pose, Slowdown, State, Step, update,
    };
    use clean_types::{
        Action, Aim, AnimationSound, Body, BoostStage, Checked, ClipEnd, Facing, Fixed, Gun, Item,
        LandedFrom, Legs, Set, Sprite, Turn, fixed,
    };
    use enum_map::EnumMap;
    use serde::Deserialize;

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/samus-looks.toml");

    /// The vector file: the paces, the clips by name, and the vectors.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VectorFile {
        paces: PacesTable,
        clips: BTreeMap<String, ClipTable>,
        vector: Vec<Vector>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct PacesTable {
        plain: Vec<u8>,
        boosted: [Vec<u8>; 5],
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ClipTable {
        steps: Vec<String>,
        kick: Option<u8>,
        fast_fall: Option<u8>,
        rising: Option<u8>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Vector {
        id: String,
        source: String,
        entries: Vec<String>,
        before: Before,
        inputs: VectorInputs,
        after: After,
    }

    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields, default)]
    struct Before {
        step: u8,
        left: u8,
        stage: u8,
        laps: u8,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct After {
        step: u8,
        left: u8,
        stage: u8,
        laps: u8,
        #[serde(default = "no_end")]
        end: String,
        #[serde(default)]
        turned_far: bool,
        #[serde(default)]
        sounds: Vec<String>,
    }

    fn no_end() -> String {
        "None".to_owned()
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields, default)]
    #[expect(
        clippy::struct_excessive_bools,
        reason = "the vector file gives each of these inputs as a flag of its own"
    )]
    struct VectorInputs {
        pose: String,
        clip: String,
        pose_after: Option<String>,
        clip_after: Option<String>,
        energy: u16,
        items: Vec<String>,
        submerged: bool,
        run_held: bool,
        handed_over: bool,
        dashing: bool,
        dash_began: bool,
        dash_lost: bool,
        standing_restarts: bool,
        wall_jump_readied: bool,
        heading_down: bool,
        slowdown: u8,
        start_slowdown: u8,
        vertical_speed: String,
        sounds_waiting: u8,
    }

    /// The file header's default energy.
    const DEFAULT_ENERGY: u16 = 99;
    /// The file header's default vertical speed.
    const DEFAULT_VERTICAL_SPEED: &str = "+0000.0000";

    /// The file header's defaults.
    impl Default for VectorInputs {
        fn default() -> Self {
            Self {
                pose: String::new(),
                clip: String::new(),
                pose_after: None,
                clip_after: None,
                energy: DEFAULT_ENERGY,
                items: Vec::new(),
                submerged: false,
                run_held: false,
                handed_over: false,
                dashing: false,
                dash_began: false,
                dash_lost: false,
                standing_restarts: false,
                wall_jump_readied: false,
                heading_down: false,
                slowdown: 0,
                start_slowdown: 0,
                vertical_speed: DEFAULT_VERTICAL_SPEED.to_owned(),
                sounds_waiting: 0,
            }
        }
    }

    const BODIES: &[Body] = &[Body::Upright, Body::Crouched, Body::Ball, Body::SpringBall];
    const ACTIONS: &[Action] = &[
        Action::AgainstWall,
        Action::Crouching,
        Action::Falling,
        Action::Jumping,
        Action::Landing(LandedFrom::Jump),
        Action::Landing(LandedFrom::Spin),
        Action::Moonwalking,
        Action::Morphing,
        Action::Rolling,
        Action::Running,
        Action::Spinning,
        Action::StandingUp,
        Action::Still,
        Action::TakingOff,
        Action::Turning(Turn::Fall),
        Action::Turning(Turn::Ground),
        Action::Turning(Turn::Jump),
        Action::Turning(Turn::MoonwalkJump),
        Action::Unmorphing,
        Action::WallJumping,
    ];
    const FACINGS: &[Facing] = &[Facing::Left, Facing::Right, Facing::Screen];
    const AIMS: &[Aim] = &[
        Aim::Forward,
        Aim::Up,
        Aim::UpForward,
        Aim::DownForward,
        Aim::Down,
    ];
    const SPRITES: &[Sprite] = &[Sprite::ScrewAttack, Sprite::SpaceJump, Sprite::OtherSuit];
    const ITEMS: &[Item] = &[Item::SpeedBooster, Item::SpaceJump, Item::ScrewAttack];
    const SOUNDS: &[AnimationSound] = &[
        AnimationSound::PlainSpin,
        AnimationSound::SpaceJump,
        AnimationSound::ScrewAttack,
        AnimationSound::Boosting,
    ];

    /// The member of `all` the file names `name`, by its Rust name; any other name fails the test.
    fn named<T: Debug + Copy>(all: &[T], name: &str) -> T {
        *all.iter()
            .find(|member| format!("{member:?}") == name)
            .unwrap_or_else(|| panic!("unknown name {name:?}"))
    }

    /// "BODY ACTION FACING AIM", then "Drawn", "Striding" and a sprite, each if given.
    fn pose(text: &str) -> Pose {
        let words: Vec<&str> = text.split_whitespace().collect();
        let [body, action, facing, aim, rest @ ..] = words.as_slice() else {
            panic!("unwritten pose {text:?}");
        };
        let mut pose = Pose {
            motion: clean_types::Pose {
                body: named(BODIES, body),
                action: named(ACTIONS, action),
                facing: named(FACINGS, facing),
                aim: named(AIMS, aim),
                gun: Gun::Stowed,
                legs: Legs::Together,
            },
            sprite: Sprite::Plain,
        };
        for word in rest {
            match *word {
                "Drawn" => pose.motion.gun = Gun::Drawn,
                "Striding" => pose.motion.legs = Legs::Striding,
                sprite => pose.sprite = named(SPRITES, sprite),
            }
        }
        pose
    }

    /// A step's number, written in a cell.
    fn step_number(text: &str) -> Step {
        Step(
            text.parse()
                .unwrap_or_else(|_| panic!("unwritten step {text:?}")),
        )
    }

    /// The spec's "Written" column (§1.4): `n` (1 or more), `rest`, `again s`, `breathe`, `end`,
    /// `end-unless-jump`, `branch x y z`; any other cell fails the test.
    fn instruction(cell: &str) -> Instruction {
        let words: Vec<&str> = cell.split_whitespace().collect();
        match words.as_slice() {
            ["rest"] => Instruction::Control(Control::Rest),
            ["breathe"] => Instruction::Control(Control::Breathe),
            ["end"] => Instruction::Control(Control::End),
            ["end-unless-jump"] => Instruction::Control(Control::EndUnlessJump),
            ["again", to] => Instruction::Control(Control::Again(step_number(to))),
            ["branch", spin, space_jump, screw_attack] => {
                Instruction::Control(Control::Branch(Branch {
                    spin: step_number(spin),
                    space_jump: step_number(space_jump),
                    screw_attack: step_number(screw_attack),
                }))
            }
            [frames] => Instruction::Show(hold(
                frames
                    .parse()
                    .unwrap_or_else(|_| panic!("unwritten cell {cell:?}")),
            )),
            _ => panic!("unwritten cell {cell:?}"),
        }
    }

    /// A hold the content gives, a clip's or a pace's: 1 frame or more (§1.4), and at most
    /// §1.5's bound, `Hold::LONGEST`; any other fails the test.
    fn hold(frames: u8) -> Hold {
        Hold::new(frames).unwrap_or_else(|| panic!("a hold of {frames} frames is outside the spec"))
    }

    /// A slowdown the file gives: 0 to 3 frames (§1.3); any other fails the test.
    fn slowdown(frames: u8) -> Slowdown {
        Slowdown::new(frames)
            .unwrap_or_else(|| panic!("a slowdown of {frames} frames is outside the spec"))
    }

    /// The digits of each part of a vertical speed.
    const PART_DIGITS: usize = 4;
    /// The base the parts are written in.
    const HEX: u32 = 16;

    /// The header's vertical speed, "+PPPP.SSSS" or "-PPPP.SSSS": a sign, then the pixel part and
    /// the subpixel part, four hex digits each, downward positive; any other text fails the test.
    fn vertical_speed(text: &str) -> Fixed {
        let part = |digits: &str| {
            assert_eq!(
                digits.len(),
                PART_DIGITS,
                "unwritten vertical speed {text:?}"
            );
            u16::from_str_radix(digits, HEX)
                .unwrap_or_else(|_| panic!("unwritten vertical speed {text:?}"))
        };
        let (pixel, sub) = text
            .get(1..)
            .and_then(|digits| digits.split_once('.'))
            .unwrap_or_else(|| panic!("unwritten vertical speed {text:?}"));
        let size = fixed(Wrapping(part(pixel)), part(sub));
        match text.get(..1) {
            Some("+") => size,
            Some("-") => -size,
            _ => panic!("unwritten vertical speed {text:?}"),
        }
    }

    fn clip(table: &ClipTable) -> Clip {
        Clip {
            instructions: table.steps.iter().map(|cell| instruction(cell)).collect(),
            kick: table.kick.map(Step),
            fast_fall: table.fast_fall.map(Step),
            rising: table.rising.map(Step),
        }
    }

    fn pace(holds: &[u8]) -> Pace {
        Pace(holds.iter().copied().map(hold).collect())
    }

    fn state(step: u8, left: u8, stage: u8, laps: u8) -> State {
        State {
            playhead: Playhead {
                step: Step(step),
                frames_left: Frames(left),
            },
            boost: Boost {
                stage: BoostStage::ALL[usize::from(stage)],
                laps_left: Laps(laps),
            },
        }
    }

    /// A vector's content: the clip of her pose and the clip of the pose she takes.
    struct VectorContent<'a> {
        pose: Pose,
        clip: &'a Clip,
        clip_after: &'a Clip,
        paces: &'a Paces,
    }

    impl Content for VectorContent<'_> {
        fn clip(&self, pose: &Pose) -> &Clip {
            if *pose == self.pose {
                self.clip
            } else {
                self.clip_after
            }
        }

        fn paces(&self) -> &Paces {
            self.paces
        }
    }

    fn inputs(given: &VectorInputs) -> Inputs {
        let start = pose(&given.pose);
        let mut moves = Set::<Move>::default();
        moves[Move::DashBegan] = given.dash_began;
        moves[Move::StandingRestart] = given.standing_restarts;
        moves[Move::WallJumpReadied] = given.wall_jump_readied;
        moves[Move::HandedOver] = given.handed_over;
        moves[Move::DashLost] = given.dash_lost;
        let mut items = Set::<Item>::default();
        for item in &given.items {
            items[named(ITEMS, item)] = true;
        }
        Inputs {
            pose: start,
            pose_after: given.pose_after.as_deref().map_or(start, pose),
            moves,
            dashing: given.dashing || given.dash_began,
            run_held: given.run_held,
            heading_down: given.heading_down,
            vertical_speed: vertical_speed(&given.vertical_speed),
            energy: given.energy,
            items,
            liquid: Liquid {
                slowdown: slowdown(given.slowdown),
                start_slowdown: slowdown(given.start_slowdown),
                submerged: given.submerged,
            },
            sounds_waiting: given.sounds_waiting,
        }
    }

    fn end(name: &str) -> Option<ClipEnd> {
        match name {
            "None" => None,
            other => Some(named(&[ClipEnd::End, ClipEnd::EndUnlessJump], other)),
        }
    }

    fn behaviour(name: &str) -> BehaviourId {
        named(&CATALOGUE.map(|behaviour| behaviour.id), name)
    }

    fn read_file() -> (VectorFile, BTreeMap<String, Clip>, Paces) {
        let file: VectorFile = toml::from_str(VECTORS).expect("the vector file reads");
        let clips = file
            .clips
            .iter()
            .map(|(name, table)| (name.clone(), clip(table)))
            .collect();
        let paces = Paces {
            plain: pace(&file.paces.plain),
            boosted: EnumMap::from_array(file.paces.boosted.each_ref().map(|holds| pace(holds))),
        };
        (file, clips, paces)
    }

    /// Runs one vector; its mismatches, if any.
    fn run(vector: &Vector, clips: &BTreeMap<String, Clip>, paces: &Paces) -> Option<String> {
        let given = &vector.inputs;
        let inputs = inputs(given);
        let clip_named = |name: &str| clips.get(name).unwrap_or_else(|| panic!("no clip {name}"));
        let content = VectorContent {
            pose: inputs.pose,
            clip: clip_named(&given.clip),
            clip_after: clip_named(given.clip_after.as_deref().unwrap_or(&given.clip)),
            paces,
        };
        assert!(
            inputs.pose != inputs.pose_after || given.clip_after.is_none(),
            "{}: another clip for the same pose",
            vector.id
        );
        let b = &vector.before;
        let before = state(b.step, b.left, b.stage, b.laps);
        let (after, report) = update(before, &inputs, &content, &Rules::VANILLA);

        let a = &vector.after;
        let expected = (
            state(a.step, a.left, a.stage, a.laps),
            end(&a.end),
            a.turned_far,
            a.sounds
                .iter()
                .map(|sound| named(SOUNDS, sound))
                .collect::<Vec<_>>(),
        );
        let actual = (
            after,
            report.end,
            report.turned_far,
            report.sound.into_iter().collect::<Vec<_>>(),
        );
        (expected != actual).then(|| {
            format!(
                "{} ({}):\n  expected {expected:?}\n  actual   {actual:?}",
                vector.id, vector.source
            )
        })
    }

    #[test]
    fn every_vector_passes() {
        let (file, clips, paces) = read_file();
        for vector in &file.vector {
            for entry in &vector.entries {
                behaviour(entry);
            }
        }
        let failures: Vec<String> = file
            .vector
            .iter()
            .filter_map(|vector| run(vector, &clips, &paces))
            .collect();
        assert!(
            failures.is_empty(),
            "{} of {} vectors fail:\n{}",
            failures.len(),
            file.vector.len(),
            failures.join("\n")
        );
    }

    #[test]
    fn every_behaviour_checked_by_vectors_has_one() {
        let (file, _, _) = read_file();
        let exercised: Vec<BehaviourId> = file
            .vector
            .iter()
            .flat_map(|vector| vector.entries.iter().map(|entry| behaviour(entry)))
            .collect();
        let missing: Vec<BehaviourId> = CATALOGUE
            .iter()
            .filter(|behaviour| behaviour.checked == Checked::ByVectors)
            .map(|behaviour| behaviour.id)
            .filter(|id| !exercised.contains(id))
            .collect();
        assert!(missing.is_empty(), "no vector exercises {missing:?}");
    }
}
