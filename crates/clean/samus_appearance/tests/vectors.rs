//! Runs every vector of `scenarios/vectors/samus-appearance.toml` through the model's one
//! update, a frame at a time, with her appearance State saved and restored between frames. A key
//! or a name the file holds that this test doesn't know fails it.

#[cfg(test)]
mod vectors {
    mod cannon_vectors;
    mod colour_vectors;

    use core::num::Wrapping;
    use std::collections::BTreeMap;
    use std::fmt::Debug;

    use enum_map::EnumMap;
    use samus_appearance::{
        Ahead, AppearanceState, BehaviourId, Branch, CATALOGUE, CannonDelay, CannonEvent, Charge,
        ChargeGlowPosition, CinematicActivity, Clip, ColourSequences, ContactSuppressor, Content,
        Control, Dash, DryColourReset, Figure, Frame, FrameOutputs, FrameSound, Frames, Hold,
        Inputs, Instruction, Laps, LapsLeft, Liquid, MoveEvent, NormalDrawing, Pace, Paces,
        PaletteOpportunity, PassageEntry, Playhead, Rules, Selection, Slowdown, SoundRequest,
        SoundsWaiting, State, Step, Stride, VisorDelay, VisorStep, update,
    };
    use serde::Deserialize;
    use types::{
        Action, Aim, AnimationSound, Body, BoostStage, Checked, ClipEnd, Facing, Fixed, Gun, Item,
        LandedFrom, Legs, Set, Sprite, Switch, Turn, fixed,
    };

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/samus-appearance.toml");

    /// The vector file: the paces, the clips by name, the animation vectors, the sound
    /// opportunities and their contact content, the combined frames, the cannon histories and
    /// the palette histories.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VectorFile {
        paces: PacesTable,
        clips: BTreeMap<String, ClipTable>,
        vector: Vec<Vector>,
        contact_content: BTreeMap<String, ContactTable>,
        opportunity: Vec<OpportunityVector>,
        appearance_work: Vec<AppearanceWork>,
        cannon_readiness: Vec<cannon_vectors::CannonHistory>,
        ordinary_dry_colours: colour_vectors::ColourVectors,
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
    struct ContactTable {
        steps: Vec<String>,
        contacts: Vec<u8>,
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

    /// The four animation quantities, each default 0.
    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields, default)]
    struct Before {
        step: u8,
        left: u8,
        stage: u8,
        laps: u8,
    }

    /// The four animation quantities after the frame, and the animation update's sounds.
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

    /// One sound rule of the frame alone: its kind, the State before and after, its inputs,
    /// typed by its kind, and what it asks for.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct OpportunityVector {
        id: String,
        source: String,
        kind: OpportunityKind,
        entries: Vec<String>,
        before: Before,
        after: Before,
        inputs: toml::Value,
        expected: ExpectedOpportunity,
    }

    #[derive(Clone, Copy, Debug, Deserialize)]
    enum OpportunityKind {
        Contact,
        PoseSelected,
        NormalDraw,
        PassageEntry,
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
        reason = "the selection vector supplies figure difference and owner predicates as flags"
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

    /// One frame combining an animation vector, if it names one, and sound opportunities.
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
    const ANIMATION_SOUNDS: &[AnimationSound] = &[
        AnimationSound::PlainSpin,
        AnimationSound::SpaceJump,
        AnimationSound::ScrewAttack,
        AnimationSound::Boosting,
    ];
    const REQUESTS: &[SoundRequest] = &[
        SoundRequest::Footstep,
        SoundRequest::PlainSpin,
        SoundRequest::SpaceJump,
        SoundRequest::ScrewAttack,
        SoundRequest::SpinEnded,
        SoundRequest::SamusSilence,
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

    /// The four animation quantities, as the file numbers them; 0 laps left is her run without
    /// the speed booster.
    fn state(step: u8, left: u8, stage: u8, laps: u8) -> State {
        State {
            playhead: Playhead {
                step: Step(step),
                time_left: Frames(left),
            },
            stage: BoostStage::ALL[usize::from(stage)],
            laps_left: Laps::new(laps).map_or(LapsLeft::WithoutBooster, LapsLeft::WithBooster),
        }
    }

    fn retained(given: &Before) -> State {
        state(given.step, given.left, given.stage, given.laps)
    }

    fn after_state(given: &After) -> State {
        state(given.step, given.left, given.stage, given.laps)
    }

    /// The cannon delay the animation and sound frames start from: no frame of theirs runs her
    /// item controls, so each leaves it as it was.
    fn untouched_cannon() -> CannonDelay {
        CannonDelay::new(1).expect("a delay of 1 is in range")
    }

    /// The colour sequences the animation and sound frames start from: none of them has a
    /// palette opportunity, so each leaves them as they were.
    const UNTOUCHED_COLOURS: ColourSequences = ColourSequences {
        visor_delay: VisorDelay(19),
        visor_step: VisorStep(2),
        charge_glow_position: ChargeGlowPosition(4),
    };

    /// Her whole appearance State, with `animation` as its four animation quantities.
    fn appearance(animation: State) -> AppearanceState {
        AppearanceState {
            animation,
            cannon: untouched_cannon(),
            colours: UNTOUCHED_COLOURS,
        }
    }

    /// A frame with none of its parts: her item controls don't run, and it has no stride,
    /// animation update, pose selection, palette opportunity, passage entry or drawing. Each
    /// vector adds the parts it plays.
    fn bare_frame() -> Frame {
        Frame {
            cannon: CannonEvent::ControlsSkipped,
            stride: None,
            animation: None,
            selection: None,
            palette: PaletteOpportunity::Absent,
            passage: None,
            drawing: None,
        }
    }

    /// Her appearance State as a caller keeps it between frames: saved whole with serde and
    /// restored fresh from the saved text alone, the same.
    fn kept(state: AppearanceState) -> AppearanceState {
        let text = toml::to_string(&state).expect("the appearance State saves");
        let restored: AppearanceState =
            toml::from_str(&text).expect("the appearance State restores");
        assert_eq!(
            toml::to_string(&restored).expect("the restored State saves"),
            text,
            "restored as saved"
        );
        restored
    }

    /// Plays one frame from the State the caller kept, and keeps the State it leaves.
    fn play(
        state: AppearanceState,
        frame: &Frame,
        content: &impl Content,
        rules: &Rules,
    ) -> (AppearanceState, FrameOutputs) {
        let (after, outputs) = update(kept(state), frame, content, rules);
        (kept(after), outputs)
    }

    /// A frame's content: the clip of each figure the frame reads, and the run's paces.
    struct VectorContent<'a> {
        clips: Vec<(Figure, &'a Clip)>,
        paces: &'a Paces,
    }

    impl Content for VectorContent<'_> {
        fn clip(&self, figure: &Figure) -> &Clip {
            self.clips
                .iter()
                .find(|(given, _)| given == figure)
                .map_or_else(|| panic!("no clip for {figure:?}"), |&(_, clip)| clip)
        }

        fn paces(&self) -> &Paces {
            self.paces
        }
    }

    /// The file's clips, the contact content's clips and the run's paces, read once.
    struct Library {
        clips: BTreeMap<String, Clip>,
        contact_clips: BTreeMap<String, Clip>,
        paces: Paces,
    }

    impl Library {
        fn clip(&self, name: &str) -> &Clip {
            self.clips
                .get(name)
                .unwrap_or_else(|| panic!("no clip {name}"))
        }

        fn contact_clip(&self, name: &str) -> &Clip {
            self.contact_clips
                .get(name)
                .unwrap_or_else(|| panic!("no contact clip {name}"))
        }
    }

    fn read_file() -> (VectorFile, Library) {
        let file: VectorFile = toml::from_str(VECTORS).expect("the vector file reads");
        let clips = file
            .clips
            .iter()
            .map(|(name, table)| (name.clone(), clip(table)))
            .collect();
        let contact_clips = file
            .contact_content
            .iter()
            .map(|(name, table)| {
                let clip = Clip {
                    instructions: table.steps.iter().map(|cell| instruction(cell)).collect(),
                    kick: None,
                    fast_fall: None,
                    rising: None,
                    contacts: table.contacts.iter().copied().map(Step).collect(),
                };
                (name.clone(), clip)
            })
            .collect();
        let paces = Paces {
            plain: pace(&file.paces.plain),
            boosted: EnumMap::from_array(file.paces.boosted.each_ref().map(|holds| pace(holds))),
        };
        let library = Library {
            clips,
            contact_clips,
            paces,
        };
        (file, library)
    }

    fn behaviour(name: &str) -> BehaviourId {
        named(&CATALOGUE.map(|behaviour| behaviour.id), name)
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

    /// A vector's animation update: its inputs, and the clips of the figure she starts the
    /// frame in and the one she takes.
    fn animation<'a>(vector: &Vector, library: &'a Library) -> (Inputs, Vec<(Figure, &'a Clip)>) {
        let given = &vector.inputs;
        let inputs = inputs(given);
        assert!(
            inputs.figure != inputs.figure_after || given.clip_after.is_none(),
            "{}: another clip for the same pose",
            vector.id
        );
        let clip_after = given.clip_after.as_deref().unwrap_or(&given.clip);
        let clips = vec![
            (inputs.figure, library.clip(&given.clip)),
            (inputs.figure_after, library.clip(clip_after)),
        ];
        (inputs, clips)
    }

    fn end(name: &str) -> Option<ClipEnd> {
        match name {
            "None" => None,
            other => Some(named(&[ClipEnd::End, ClipEnd::EndUnlessJump], other)),
        }
    }

    /// A sound of the frame by the name the file gives it: an animation update's or an
    /// opportunity's.
    fn sound_name(sound: FrameSound) -> String {
        match sound {
            FrameSound::Animation(sound) => format!("{sound:?}"),
            FrameSound::Opportunity(request) => format!("{request:?}"),
        }
    }

    /// Runs one animation vector as a frame with its animation update alone; its mismatches, if
    /// any.
    fn run(vector: &Vector, library: &Library) -> Option<String> {
        let (inputs, clips) = animation(vector, library);
        let content = VectorContent {
            clips,
            paces: &library.paces,
        };
        let before = appearance(retained(&vector.before));
        let frame = Frame {
            animation: Some(inputs),
            ..bare_frame()
        };
        let (after, outputs) = play(before, &frame, &content, &rules(&vector.rules));
        let ahead = Ahead {
            end: end(&vector.ahead.end),
            turned_far: vector.ahead.turned_far,
        };
        assert_eq!(
            Ahead::of(before.animation.playhead, content.clip(&inputs.figure)),
            ahead,
            "{}: the reading samus-movement makes before its step",
            vector.id
        );
        let expected = (
            Some(ahead),
            appearance(after_state(&vector.after)),
            vector
                .after
                .sounds
                .iter()
                .map(|sound| FrameSound::Animation(named(ANIMATION_SOUNDS, sound)))
                .collect::<Vec<_>>(),
            None,
            DryColourReset::NoOpportunity,
            before.cannon.may_follow(),
        );
        let actual = (
            outputs.ahead,
            after,
            outputs.sounds,
            outputs.contact,
            outputs.dry_colours,
            outputs.cannon_may_follow,
        );
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
        let (file, library) = read_file();
        for vector in &file.vector {
            for entry in &vector.entries {
                behaviour(entry);
            }
        }
        let failures: Vec<String> = file
            .vector
            .iter()
            .filter_map(|vector| run(vector, &library))
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
        let (file, _) = read_file();
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

    fn cinematic(active: bool) -> CinematicActivity {
        if active {
            CinematicActivity::Active
        } else {
            CinematicActivity::Inactive
        }
    }

    /// The stride a contact vector gives, with the clip of her figure. It reads her step, the
    /// time left and the boost stage from `at_start`, the State as the frame starts, which the
    /// vector's inputs repeat.
    fn stride<'a>(
        vector: &OpportunityVector,
        at_start: State,
        library: &'a Library,
    ) -> (Stride, (Figure, &'a Clip)) {
        let given: ContactInputs = vector
            .inputs
            .clone()
            .try_into()
            .expect("typed contact inputs");
        assert_eq!(
            state(given.step, given.left, given.stage, 0).playhead,
            at_start.playhead,
            "{}: the stride's step and time left are the State's",
            vector.id
        );
        assert_eq!(
            BoostStage::ALL[usize::from(given.stage)],
            at_start.stage,
            "{}: the stride's stage is the State's",
            vector.id
        );
        let mut suppressors = Set::<ContactSuppressor>::default();
        suppressors[ContactSuppressor::Cinematic] = given.cinematic;
        suppressors[ContactSuppressor::Boss] = given.boss;
        suppressors[ContactSuppressor::SpecialColours] = given.special_colours;
        let figure = figure(&format!("Upright {} Right Forward", given.movement));
        let stride = Stride {
            figure,
            suppressors,
        };
        (stride, (figure, library.contact_clip(&given.clip)))
    }

    /// The compact selection vectors give actions and whether the figure changes, not whole
    /// figures. A changed figure takes the other suit's sprite, so it differs even where the
    /// action stays; an unchanged one is the figure she starts in, whatever `to_action` names
    /// (C-start-same tests the no-different-figure condition).
    fn selection(vector: &OpportunityVector) -> Selection {
        let given: SelectionInputs = vector
            .inputs
            .clone()
            .try_into()
            .expect("typed selection inputs");
        let from = figure(&format!("Upright {} Right Forward", given.from_action));
        let to_action = named(ACTIONS, &given.to_action);
        let selected = if given.changed {
            Figure {
                pose: types::Pose {
                    action: to_action,
                    ..from.pose
                },
                sprite: Sprite::OtherSuit,
            }
        } else {
            from
        };
        let mut items = Set::<Item>::default();
        items[Item::SpaceJump] = given.space_jump;
        items[Item::ScrewAttack] = given.screw_attack;
        Selection {
            from,
            selected,
            items,
            submerged: given.submerged,
            cinematic: cinematic(given.cinematic),
        }
    }

    fn drawing(vector: &OpportunityVector) -> NormalDrawing {
        let given: DrawInputs = vector
            .inputs
            .clone()
            .try_into()
            .expect("typed drawing inputs");
        NormalDrawing {
            previous: named(ACTIONS, &given.previous),
            current: named(ACTIONS, &given.current),
        }
    }

    fn passage(vector: &OpportunityVector) -> PassageEntry {
        let given: DoorInputs = vector
            .inputs
            .clone()
            .try_into()
            .expect("typed passage inputs");
        PassageEntry {
            current: named(ACTIONS, &given.current),
            shoot_held: given.shoot_held,
            charge: Charge::new(given.charge)
                .unwrap_or_else(|| panic!("{}: charge outside the spec", vector.id)),
        }
    }

    /// Adds the opportunity `vector` gives to `frame`, which has none of its kind yet: each
    /// applies at most once a frame. A stride's clip joins `clips`.
    fn add_opportunity<'a>(
        frame: &mut Frame,
        clips: &mut Vec<(Figure, &'a Clip)>,
        vector: &OpportunityVector,
        at_start: State,
        library: &'a Library,
    ) {
        let once = |taken: bool| assert!(!taken, "{}: a second {:?}", vector.id, vector.kind);
        match vector.kind {
            OpportunityKind::Contact => {
                once(frame.stride.is_some());
                let (stride, clip) = stride(vector, at_start, library);
                frame.stride = Some(stride);
                clips.push(clip);
            }
            OpportunityKind::PoseSelected => {
                once(frame.selection.is_some());
                frame.selection = Some(selection(vector));
            }
            OpportunityKind::NormalDraw => {
                once(frame.drawing.is_some());
                frame.drawing = Some(drawing(vector));
            }
            OpportunityKind::PassageEntry => {
                once(frame.passage.is_some());
                frame.passage = Some(passage(vector));
            }
        }
    }

    /// Her stride's graphical contact is the figure she runs in and her playhead as the frame
    /// starts, before the animation update.
    fn check_contact(outputs: &FrameOutputs, clips: &[(Figure, &Clip)], at_start: State, id: &str) {
        let Some(contact) = outputs.contact else {
            return;
        };
        assert_eq!(contact.figure.pose.action, Action::Running, "{id}: contact");
        assert!(
            clips.iter().any(|(figure, _)| *figure == contact.figure),
            "{id}: the contact's figure is the stride's"
        );
        assert_eq!(
            contact.playhead, at_start.playhead,
            "{id}: contact playhead"
        );
    }

    #[test]
    #[expect(
        clippy::print_stdout,
        reason = "report the number of supplied opportunity vectors executed"
    )]
    fn every_sound_opportunity_passes() {
        let (file, library) = read_file();
        for vector in &file.opportunity {
            for entry in &vector.entries {
                behaviour(entry);
            }
            let before = appearance(retained(&vector.before));
            let mut frame = bare_frame();
            let mut clips = Vec::new();
            add_opportunity(&mut frame, &mut clips, vector, before.animation, &library);
            let content = VectorContent {
                clips,
                paces: &library.paces,
            };
            let (after, outputs) = play(before, &frame, &content, &Rules::VANILLA);
            let id = &vector.id;
            assert_eq!(
                after,
                appearance(retained(&vector.after)),
                "{id} ({}): the State",
                vector.source
            );
            assert_eq!(after, before, "{id}: no opportunity changes the State");
            assert_eq!(outputs.ahead, None, "{id}: no animation update");
            assert_eq!(outputs.contact.is_some(), vector.expected.contact, "{id}");
            let expected: Vec<FrameSound> = vector
                .expected
                .sounds
                .iter()
                .map(|sound| FrameSound::Opportunity(named(REQUESTS, sound)))
                .collect();
            assert_eq!(outputs.sounds, expected, "{id}: ordered sounds");
            check_contact(&outputs, &content.clips, before.animation, id);
        }
        println!(
            "{} sound-opportunity vectors passed",
            file.opportunity.len()
        );
    }

    /// Plays one combined frame: the animation update of the vector it names, if any, with the
    /// sound opportunities it names, all in the one update.
    fn run_work(work: &AppearanceWork, file: &VectorFile, library: &Library) {
        let id = &work.id;
        let vector = work.animation_vector.as_ref().map(|name| {
            file.vector
                .iter()
                .find(|vector| vector.id == *name)
                .unwrap_or_else(|| panic!("{id}: no animation vector {name}"))
        });
        let opportunities: Vec<&OpportunityVector> = work
            .opportunities
            .iter()
            .map(|name| {
                file.opportunity
                    .iter()
                    .find(|opportunity| opportunity.id == *name)
                    .unwrap_or_else(|| panic!("{id}: no opportunity {name}"))
            })
            .collect();
        let at_start = vector.map_or_else(
            || retained(&opportunities.first().expect("an opportunity").before),
            |vector| retained(&vector.before),
        );
        let mut frame = bare_frame();
        let mut clips = Vec::new();
        if let Some(vector) = vector {
            let (inputs, animation_clips) = animation(vector, library);
            frame.animation = Some(inputs);
            clips.extend(animation_clips);
        }
        for opportunity in &opportunities {
            add_opportunity(&mut frame, &mut clips, opportunity, at_start, library);
        }
        let content = VectorContent {
            clips,
            paces: &library.paces,
        };
        let tuning = vector.map_or(Rules::VANILLA, |vector| rules(&vector.rules));
        let (after, outputs) = play(appearance(at_start), &frame, &content, &tuning);
        assert!(work.after.sounds.is_empty(), "{id}: sounds are top-level");
        assert_eq!(
            after,
            appearance(after_state(&work.after)),
            "{id} ({}): all four quantities, one animation update at most",
            work.source
        );
        let sounds: Vec<String> = outputs.sounds.iter().copied().map(sound_name).collect();
        assert_eq!(sounds, work.sounds, "{id}: every request, in order");
        assert_eq!(outputs.contact.is_some(), work.contact, "{id}: contact");
        check_contact(&outputs, &content.clips, at_start, id);
    }

    #[test]
    #[expect(
        clippy::print_stdout,
        reason = "report the number of supplied combined frames executed"
    )]
    fn every_combined_frame_passes() {
        let (file, library) = read_file();
        for work in &file.appearance_work {
            for entry in &work.entries {
                behaviour(entry);
            }
            run_work(work, &file, &library);
        }
        println!("{} combined frames passed", file.appearance_work.len());
    }

    #[test]
    fn every_behaviour_checked_by_vectors_has_one() {
        let (file, _) = read_file();
        let exercised: Vec<BehaviourId> = file
            .vector
            .iter()
            .flat_map(|vector| &vector.entries)
            .chain(file.opportunity.iter().flat_map(|vector| &vector.entries))
            .chain(file.appearance_work.iter().flat_map(|work| &work.entries))
            .chain(
                file.cannon_readiness
                    .iter()
                    .flat_map(|history| &history.entries),
            )
            .map(|entry| behaviour(entry))
            .chain(file.ordinary_dry_colours.entries().map(behaviour))
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
