//! Runs every vector of `scenarios/vectors/samus-appearance.toml` through the model. A key or a name
//! the file holds that this test doesn't know fails it.

#[cfg(test)]
mod vectors {
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "the assertion seeks only initialization receipts"
    )]
    mod additional;
    mod cannon_vectors;
    mod colour_vectors;
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "strict test fixtures and assertion failures use known vector indices and exhaustive rejection fallbacks"
    )]
    mod suspended_vectors;
    use core::num::Wrapping;
    use std::collections::BTreeMap;
    use std::fmt::Debug;

    use enum_map::EnumMap;
    use samus_appearance::{Ahead, BehaviourId, Rules, SoundsWaiting};
    use samus_appearance::{
        Branch, CATALOGUE, Clip, Content, Control, Dash, Figure, Frames, Hold, Inputs, Instruction,
        Laps, LapsLeft, Liquid, MoveEvent, Pace, Paces, Playhead, Slowdown, State, Step, update,
    };
    use serde::Deserialize;
    use types::{
        Action, Aim, AnimationSound, Body, BoostStage, Checked, ClipEnd, Facing, Fixed, Gun, Item,
        LandedFrom, Legs, Set, Sprite, Switch, Turn, fixed,
    };

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/samus-appearance.toml");

    /// The vector file: the paces, the clips by name, and the vectors.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VectorFile {
        paces: PacesTable,
        clips: BTreeMap<String, ClipTable>,
        vector: Vec<Vector>,
        contact_content: BTreeMap<String, ContactTable>,
        opportunity: Vec<OpportunityVector>,
        appearance_work: Vec<AppearanceWork>,
        suspended_update: Vec<suspended_vectors::SuspendedVector>,
        resume_case: Vec<suspended_vectors::ResumeCase>,
        cannon_readiness: Vec<cannon_vectors::CannonHistory>,
        ordinary_run_colours: colour_vectors::ColourVectors,
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
        #[serde(default)]
        rules: RulesTable,
        before: Before,
        #[serde(default)]
        ahead: AheadTable,
        inputs: VectorInputs,
        after: After,
    }

    /// A vector's own values of the spec's parameters; each it doesn't name is the game's.
    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields, default)]
    struct RulesTable {
        breathing_threshold: Option<u16>,
        restart_hold: Option<u8>,
        fast_fall_speed: Option<String>,
        fast_fall_hold: Option<u8>,
        rising_hold: Option<u8>,
        laps_per_stage: Option<[u8; 5]>,
    }

    /// What is known from the frame's start alone.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields, default)]
    struct AheadTable {
        end: String,
        turned_far: bool,
    }

    /// The file header's defaults: no end, not turned far.
    impl Default for AheadTable {
        fn default() -> Self {
            Self {
                end: "None".to_owned(),
                turned_far: false,
            }
        }
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
        #[serde(default)]
        sounds: Vec<String>,
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

    /// A pose with its sprite: "BODY ACTION FACING AIM", then "Drawn", "Striding" and a sprite,
    /// each if given.
    fn figure(text: &str) -> Figure {
        let words: Vec<&str> = text.split_whitespace().collect();
        let [body, action, facing, aim, rest @ ..] = words.as_slice() else {
            panic!("unwritten pose {text:?}");
        };
        let mut figure = Figure {
            pose: types::Pose {
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
                "Drawn" => figure.pose.gun = Gun::Drawn,
                "Striding" => figure.pose.legs = Legs::Striding,
                sprite => figure.sprite = named(SPRITES, sprite),
            }
        }
        figure
    }

    /// A step's number, written in a cell.
    fn step_number(text: &str) -> Step {
        Step(
            text.parse()
                .unwrap_or_else(|_| panic!("unwritten step {text:?}")),
        )
    }

    /// The spec's "Written" column of a clip's steps: `n` (1 or more), `rest`, `again s`,
    /// `breathe`, `end`, `end-unless-jump`, `branch x y z`; any other cell fails the test.
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

    /// A hold the content gives, a clip's or a pace's: 1 frame or more, and at most the spec's
    /// bound, `Hold::LONGEST`; any other fails the test.
    fn hold(frames: u8) -> Hold {
        Hold::new(frames).unwrap_or_else(|| panic!("a hold of {frames} frames is outside the spec"))
    }

    /// A slowdown the file gives: 0 to 3 frames; any other fails the test.
    fn slowdown(frames: u8) -> Slowdown {
        Slowdown::new(frames)
            .unwrap_or_else(|| panic!("a slowdown of {frames} frames is outside the spec"))
    }

    /// The digits of each part of a vertical speed.
    const PART_DIGITS: usize = 4;
    /// The base the parts are written in.
    const HEX: u32 = 16;

    /// A speed's size, "PPPP.SSSS": the pixel part and the subpixel part, four hex digits each;
    /// any other text fails the test.
    fn speed_size(text: &str) -> Fixed {
        let part = |digits: &str| {
            assert_eq!(digits.len(), PART_DIGITS, "unwritten speed {text:?}");
            u16::from_str_radix(digits, HEX).unwrap_or_else(|_| panic!("unwritten speed {text:?}"))
        };
        let (pixel, sub) = text
            .split_once('.')
            .unwrap_or_else(|| panic!("unwritten speed {text:?}"));
        fixed(Wrapping(part(pixel)), part(sub))
    }

    /// The header's vertical speed, "+PPPP.SSSS" or "-PPPP.SSSS": a sign, then its size, downward
    /// positive; any other text fails the test.
    fn vertical_speed(text: &str) -> Fixed {
        let (sign, size) = text
            .split_at_checked(1)
            .unwrap_or_else(|| panic!("unwritten vertical speed {text:?}"));
        match sign {
            "+" => speed_size(size),
            "-" => -speed_size(size),
            _ => panic!("unwritten vertical speed {text:?}"),
        }
    }

    /// A vector's tuning: the game's, with the parameters its `rules` name set to its values.
    fn rules(given: &RulesTable) -> Rules {
        let game = Rules::VANILLA;
        Rules {
            breathing_threshold: given
                .breathing_threshold
                .unwrap_or(game.breathing_threshold),
            restart_hold: given.restart_hold.map_or(game.restart_hold, hold),
            fast_fall_speed: given
                .fast_fall_speed
                .as_deref()
                .map_or(game.fast_fall_speed, speed_size),
            fast_fall_hold: given.fast_fall_hold.map_or(game.fast_fall_hold, hold),
            rising_hold: given.rising_hold.map_or(game.rising_hold, hold),
            laps_per_stage: given.laps_per_stage.map_or(game.laps_per_stage, |laps| {
                EnumMap::from_array(laps.map(|count| {
                    Laps::new(count).unwrap_or_else(|| panic!("a stage of {count} laps"))
                }))
            }),
        }
    }

    fn clip(table: &ClipTable) -> Clip {
        Clip {
            instructions: table.steps.iter().map(|cell| instruction(cell)).collect(),
            kick: table.kick.map(Step),
            fast_fall: table.fast_fall.map(Step),
            rising: table.rising.map(Step),
            contacts: std::collections::BTreeSet::new(),
        }
    }

    fn pace(holds: &[u8]) -> Pace {
        Pace(holds.iter().copied().map(hold).collect())
    }

    fn state(step: u8, left: u8, stage: u8, laps: u8) -> State {
        State {
            playhead: Playhead {
                step: Step(step),
                time_left: Frames(left),
            },
            stage: BoostStage::ALL[usize::from(stage)],
            // 0 laps left is her run without the speed booster.
            laps_left: Laps::new(laps).map_or(LapsLeft::WithoutBooster, LapsLeft::WithBooster),
        }
    }

    /// A vector's content: the clip of her figure and the clip of the figure she takes.
    struct VectorContent<'a> {
        figure: Figure,
        clip: &'a Clip,
        clip_after: &'a Clip,
        paces: &'a Paces,
    }

    impl Content for VectorContent<'_> {
        fn clip(&self, figure: &Figure) -> &Clip {
            if *figure == self.figure {
                self.clip
            } else {
                self.clip_after
            }
        }

        fn paces(&self) -> &Paces {
            self.paces
        }
    }

    /// Whether she dashes as the clip moves on: a dash that began is dashing, whatever `dashing`
    /// says.
    const fn dash(given: &VectorInputs) -> Dash {
        match (given.dash_began, given.dashing) {
            (true, _) => Dash::Began,
            (false, true) => Dash::On,
            (false, false) => Dash::Off,
        }
    }

    fn inputs(given: &VectorInputs) -> Inputs {
        let start = figure(&given.pose);
        let mut moves = Set::<MoveEvent>::default();
        moves[MoveEvent::StandingRestart] = given.standing_restarts;
        moves[MoveEvent::WallJumpReadied] = given.wall_jump_readied;
        moves[MoveEvent::HandedOver] = given.handed_over;
        moves[MoveEvent::DashLost] = given.dash_lost;
        let mut items = Set::<Item>::default();
        for item in &given.items {
            items[named(ITEMS, item)] = true;
        }
        Inputs {
            figure: start,
            figure_after: given.pose_after.as_deref().map_or(start, figure),
            moves,
            dash: dash(given),
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
            sounds_waiting: SoundsWaiting::new(given.sounds_waiting).unwrap_or_else(|| {
                panic!(
                    "{} sounds waiting is outside the spec",
                    given.sounds_waiting
                )
            }),
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

    fn animate(
        vector: &Vector,
        clips: &BTreeMap<String, Clip>,
        paces: &Paces,
    ) -> (Ahead, State, Option<AnimationSound>) {
        let given = &vector.inputs;
        let inputs = inputs(given);
        let clip_named = |name: &str| clips.get(name).unwrap_or_else(|| panic!("no clip {name}"));
        let content = VectorContent {
            figure: inputs.figure,
            clip: clip_named(&given.clip),
            clip_after: clip_named(given.clip_after.as_deref().unwrap_or(&given.clip)),
            paces,
        };
        assert!(
            inputs.figure != inputs.figure_after || given.clip_after.is_none(),
            "{}: another clip for the same pose",
            vector.id
        );
        let b = &vector.before;
        let before = state(b.step, b.left, b.stage, b.laps);
        let ahead = Ahead::of(before.playhead, content.clip);
        let (after, sound) = update(before, &inputs, &content, &rules(&vector.rules));

        (ahead, after, sound)
    }

    /// Runs one vector; its mismatches, if any.
    fn run(vector: &Vector, clips: &BTreeMap<String, Clip>, paces: &Paces) -> Option<String> {
        let (ahead, after, sound) = animate(vector, clips, paces);
        let a = &vector.after;
        let expected = (
            Ahead {
                end: end(&vector.ahead.end),
                turned_far: vector.ahead.turned_far,
            },
            state(a.step, a.left, a.stage, a.laps),
            a.sounds
                .iter()
                .map(|sound| named(SOUNDS, sound))
                .collect::<Vec<_>>(),
        );
        let actual = (ahead, after, sound.into_iter().collect::<Vec<_>>());
        (expected != actual).then(|| {
            format!(
                "{} ({}):\n  expected {expected:?}\n  actual   {actual:?}",
                vector.id, vector.source
            )
        })
    }

    #[test]
    #[expect(
        clippy::print_stdout,
        reason = "report the number of supplied vectors executed"
    )]
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
        println!("{} animation vectors passed", file.vector.len());
    }

    /// A vector that adjusts the game exercises only entries whose switch is a parameter.
    #[test]
    fn every_adjusted_vector_exercises_parameters() {
        let (file, _, _) = read_file();
        let switch_of = |entry: &str| {
            let id = behaviour(entry);
            CATALOGUE
                .iter()
                .find(|behaviour| behaviour.id == id)
                .map(|behaviour| behaviour.switch)
        };
        let fixed: Vec<String> = file
            .vector
            .iter()
            .filter(|vector| rules(&vector.rules) != Rules::VANILLA)
            .flat_map(|vector| {
                vector
                    .entries
                    .iter()
                    .filter(|entry| switch_of(entry) != Some(Switch::Parameter))
                    .map(|entry| format!("{}: {entry}", vector.id))
            })
            .collect();
        assert!(fixed.is_empty(), "adjusted, but not parameters: {fixed:?}");
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ContactTable {
        steps: Vec<String>,
        contacts: Vec<u8>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct OpportunityVector {
        id: String,
        source: String,
        kind: String,
        entries: Vec<String>,
        before: Before,
        after: Before,
        inputs: toml::Value,
        expected: ExpectedOpportunity,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ExpectedOpportunity {
        sounds: Vec<String>,
        contact: bool,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ContactInputs {
        movement: String,
        step: u8,
        left: u8,
        clip: String,
        cinematic: bool,
        boss: bool,
        special_colours: bool,
        stage: u8,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    #[expect(
        clippy::struct_excessive_bools,
        reason = "the selection vector supplies figure difference and owner predicates"
    )]
    struct SelectionInputs {
        from_action: String,
        to_action: String,
        changed: bool,
        space_jump: bool,
        screw_attack: bool,
        submerged: bool,
        cinematic: bool,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct DrawInputs {
        previous: String,
        current: String,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct DoorInputs {
        current: String,
        shoot_held: bool,
        charge: u16,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct AppearanceWork {
        id: String,
        source: String,
        entries: Vec<String>,
        animation_vector: Option<String>,
        opportunities: Vec<String>,
        sounds: Vec<String>,
        contact: bool,
        after: After,
    }

    fn retained(given: &Before) -> State {
        state(given.step, given.left, given.stage, given.laps)
    }

    fn request(name: &str) -> samus_appearance::SoundRequest {
        use samus_appearance::SoundRequest;
        named(
            &[
                SoundRequest::Footstep,
                SoundRequest::PlainSpin,
                SoundRequest::SpaceJump,
                SoundRequest::ScrewAttack,
                SoundRequest::SpinEnded,
                SoundRequest::SamusSilence,
            ],
            name,
        )
    }

    fn contact_clips(file: &VectorFile) -> BTreeMap<String, Clip> {
        file.contact_content
            .iter()
            .map(|(name, table)| {
                (
                    name.clone(),
                    Clip {
                        instructions: table.steps.iter().map(|cell| instruction(cell)).collect(),
                        kick: None,
                        fast_fall: None,
                        rising: None,
                        contacts: table.contacts.iter().copied().map(Step).collect(),
                    },
                )
            })
            .collect()
    }

    /// The compact selection vectors give actions and an explicit figure-difference guard,
    /// not full poses. A changed selection is represented by a changed sprite as needed.
    /// In the unchanged case the selected figure is exactly the original, regardless of the
    /// nominal `to_action`: C-start-same explicitly tests the no-different-figure condition.
    fn selection(given: &SelectionInputs) -> samus_appearance::Opportunity<'static> {
        let from = figure(&format!("Upright {} Right Forward", given.from_action));
        let mut selected = from;
        if given.changed {
            selected.pose.action = named(ACTIONS, &given.to_action);
            selected.sprite = Sprite::OtherSuit;
        } else {
            named(ACTIONS, &given.to_action);
        }
        let mut items = Set::<Item>::default();
        items[Item::SpaceJump] = given.space_jump;
        items[Item::ScrewAttack] = given.screw_attack;
        samus_appearance::Opportunity::PoseSelected {
            from,
            selected,
            items,
            submerged: given.submerged,
            cinematic: if given.cinematic {
                samus_appearance::CinematicActivity::Active
            } else {
                samus_appearance::CinematicActivity::Inactive
            },
        }
    }

    fn read_opportunity(
        vector: &OpportunityVector,
        clips: &BTreeMap<String, Clip>,
    ) -> samus_appearance::OpportunityOutput {
        owned_opportunity(vector, clips).read()
    }

    fn owned_opportunity(
        vector: &OpportunityVector,
        clips: &BTreeMap<String, Clip>,
    ) -> samus_appearance::AppearanceRead {
        use samus_appearance::{Charge, ContactSuppressor, Opportunity};
        let inputs = vector.inputs.clone();
        let opportunity = match vector.kind.as_str() {
            "Contact" => {
                let given: ContactInputs = inputs.try_into().expect("typed contact inputs");
                let mut suppressors = Set::<ContactSuppressor>::default();
                suppressors[ContactSuppressor::Cinematic] = given.cinematic;
                suppressors[ContactSuppressor::Boss] = given.boss;
                suppressors[ContactSuppressor::SpecialColours] = given.special_colours;
                assert_eq!(
                    given.stage, vector.before.stage,
                    "{}: stride stage",
                    vector.id
                );
                Opportunity::Contact {
                    figure: figure(&format!("Upright {} Right Forward", given.movement)),
                    playhead: Playhead {
                        step: Step(given.step),
                        time_left: Frames(given.left),
                    },
                    stage: BoostStage::ALL[usize::from(given.stage)],
                    clip: clips.get(&given.clip).expect("named contact clip"),
                    suppressors,
                }
            }
            "PoseSelected" => selection(&inputs.try_into().expect("typed selection inputs")),
            "NormalDraw" => {
                let given: DrawInputs = inputs.try_into().expect("typed drawing inputs");
                Opportunity::NormalDraw {
                    previous: named(ACTIONS, &given.previous),
                    current: named(ACTIONS, &given.current),
                }
            }
            "PassageEntry" => {
                let given: DoorInputs = inputs.try_into().expect("typed passage inputs");
                Opportunity::PassageEntry {
                    current: named(ACTIONS, &given.current),
                    shoot_held: given.shoot_held,
                    charge: Charge::new(given.charge).expect("valid charge"),
                }
            }
            other => panic!("unknown opportunity {other}"),
        };
        opportunity.into()
    }

    #[test]
    #[expect(
        clippy::print_stdout,
        reason = "report the number of supplied opportunity vectors executed"
    )]
    fn every_sound_opportunity_passes() {
        let (file, _, _) = read_file();
        let clips = contact_clips(&file);
        for vector in &file.opportunity {
            for entry in &vector.entries {
                behaviour(entry);
            }
            let before = retained(&vector.before);
            let output = read_opportunity(vector, &clips);
            assert_eq!(
                before,
                retained(&vector.after),
                "{} ({}): retained State",
                vector.id,
                vector.source
            );
            assert_eq!(
                output.contact.is_some(),
                vector.expected.contact,
                "{}: graphics",
                vector.id
            );
            assert_eq!(
                output.sound.into_iter().collect::<Vec<_>>(),
                vector
                    .expected
                    .sounds
                    .iter()
                    .map(|sound| request(sound))
                    .collect::<Vec<_>>(),
                "{}: ordered sounds",
                vector.id
            );
            if let Some(contact) = output.contact {
                let given: ContactInputs =
                    vector.inputs.clone().try_into().expect("contact inputs");
                assert_eq!(
                    contact.figure,
                    figure(&format!("Upright {} Right Forward", given.movement))
                );
                assert_eq!(
                    contact.playhead,
                    Playhead {
                        step: Step(given.step),
                        time_left: Frames(given.left)
                    }
                );
            }
        }
        println!(
            "{} sound-opportunity vectors passed",
            file.opportunity.len()
        );
    }

    #[test]
    #[expect(
        clippy::print_stdout,
        reason = "report the number of supplied composition vectors executed"
    )]
    fn every_composition_retains_all_outputs() {
        let (file, clips, paces) = read_file();
        let contacts = contact_clips(&file);
        for work in &file.appearance_work {
            for entry in &work.entries {
                behaviour(entry);
            }
            let opportunities: Vec<&OpportunityVector> = work
                .opportunities
                .iter()
                .map(|id| {
                    file.opportunity
                        .iter()
                        .find(|v| v.id == *id)
                        .expect("named opportunity")
                })
                .collect();
            let mut after = retained(
                &opportunities
                    .first()
                    .expect("work has an opportunity")
                    .before,
            );
            let mut sounds = Vec::new();
            let mut graphics = Vec::new();
            if let Some(id) = &work.animation_vector {
                let vector = file
                    .vector
                    .iter()
                    .find(|v| v.id == *id)
                    .expect("named animation vector");
                let (_, updated, sound) = animate(vector, &clips, &paces);
                after = updated;
                sounds.extend(sound.map(|s| format!("{s:?}")));
            }
            for vector in opportunities {
                let retained_before_read = after;
                let output = read_opportunity(vector, &contacts);
                sounds.extend(output.sound.map(|s| format!("{s:?}")));
                graphics.extend(output.contact);
                assert_eq!(after, retained_before_read, "opportunity changed animation");
            }
            let expected = &work.after;
            assert!(
                expected.sounds.is_empty(),
                "composition after sounds must be top-level"
            );
            assert_eq!(
                after,
                state(expected.step, expected.left, expected.stage, expected.laps),
                "{} ({}): all four quantities",
                work.id,
                work.source
            );
            assert_eq!(sounds, work.sounds, "{}: every ordered request", work.id);
            assert_eq!(
                !graphics.is_empty(),
                work.contact,
                "{}: retained graphics",
                work.id
            );
            for graphic in graphics {
                assert_eq!(graphic.figure.pose.action, Action::Running);
                assert_eq!(
                    graphic.playhead,
                    Playhead {
                        step: Step(2),
                        time_left: Frames::LAST
                    }
                );
            }
        }
        println!("{} composition vectors passed", file.appearance_work.len());
    }

    #[test]
    fn every_behaviour_checked_by_vectors_has_one() {
        let (file, _, _) = read_file();
        let exercised: Vec<BehaviourId> = file
            .vector
            .iter()
            .flat_map(|vector| vector.entries.iter().map(|entry| behaviour(entry)))
            .chain(
                file.opportunity
                    .iter()
                    .flat_map(|vector| vector.entries.iter().map(|entry| behaviour(entry))),
            )
            .chain(
                file.appearance_work
                    .iter()
                    .flat_map(|vector| vector.entries.iter().map(|entry| behaviour(entry))),
            )
            .chain(
                file.suspended_update
                    .iter()
                    .flat_map(|vector| vector.entries.iter().map(|entry| behaviour(entry))),
            )
            .chain(
                file.resume_case
                    .iter()
                    .flat_map(|vector| vector.entries.iter().map(|entry| behaviour(entry))),
            )
            .chain(
                file.cannon_readiness
                    .iter()
                    .flat_map(|vector| vector.entries.iter().map(|entry| behaviour(entry))),
            )
            .chain(file.ordinary_run_colours.entries().map(behaviour))
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
