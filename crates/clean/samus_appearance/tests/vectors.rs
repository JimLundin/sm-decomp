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
        Activity, Ahead, AppearanceState, BehaviourId, Branch, CATALOGUE, CannonDelay, CannonEvent,
        Charge, ChargeGlowPosition, Clip, ColourSequences, Content, Control, Dash, DryColourReset,
        Figure, Frame, FrameOutputs, FramePart, Frames, Hold, Inputs, Instruction, Laps, LapsLeft,
        Liquid, MoveEvent, Pace, Paces, PaletteOpportunity, Playhead, Rules, Slowdown,
        SoundRequest, SoundsWaiting, State, Step, VisorDelay, VisorStep, update,
    };
    use serde::Deserialize;
    use types::{
        Action, Aim, Body, BoostStage, Checked, ClipEnd, Facing, Fixed, Gun, Item, LandedFrom,
        Legs, Set, Sprite, Switch, Turn, fixed, set_of,
    };

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/samus-appearance.toml");

    /// The vector file: her State at initialization, the paces, the clips by name, the animation
    /// vectors, the sound opportunities, the combined frames, the cannon histories and the
    /// palette histories.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VectorFile {
        initialization: Initialization,
        paces: PacesTable,
        clips: BTreeMap<String, ClipTable>,
        vector: Vec<Vector>,
        opportunity: Vec<OpportunityVector>,
        appearance_work: Vec<AppearanceWork>,
        cannon_readiness: Vec<cannon_vectors::CannonHistory>,
        ordinary_dry_colours: colour_vectors::ColourVectors,
    }

    /// Her appearance State as initialization leaves it, quantity by quantity.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Initialization {
        step: u8,
        left: u8,
        stage: u8,
        laps: u8,
        kept: String,
        cannon_delay: u8,
        visor_delay: u8,
        visor_step: usize,
        charge_glow_position: usize,
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
        #[serde(default)]
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

    /// The header's default kept action.
    const DEFAULT_KEPT: &str = "Still";

    /// Her State's animation part, each quantity default 0, her kept action, default `Still`,
    /// and the cannon delay and colour-sequence quantities, each where the file names it.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields, default)]
    struct Before {
        step: u8,
        left: u8,
        stage: u8,
        laps: u8,
        kept: String,
        cannon_delay: Option<u8>,
        visor_delay: Option<u8>,
        visor_step: Option<usize>,
        charge_glow_position: Option<usize>,
    }

    /// The file header's defaults.
    impl Default for Before {
        fn default() -> Self {
            Self {
                step: 0,
                left: 0,
                stage: 0,
                laps: 0,
                kept: DEFAULT_KEPT.to_owned(),
                cannon_delay: None,
                visor_delay: None,
                visor_step: None,
                charge_glow_position: None,
            }
        }
    }

    /// The four animation quantities after the frame, her kept action, the cannon delay and the
    /// colour-sequence quantities (each absent: as before), and the frame's sounds.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct After {
        step: u8,
        left: u8,
        stage: u8,
        laps: u8,
        kept: Option<String>,
        cannon_delay: Option<u8>,
        visor_delay: Option<u8>,
        visor_step: Option<usize>,
        charge_glow_position: Option<usize>,
        #[serde(default)]
        sounds: Vec<String>,
    }

    /// The cannon delay and colour-sequence quantities a `before` or `after` names, each
    /// optional.
    struct NamedMemory {
        cannon_delay: Option<u8>,
        visor_delay: Option<u8>,
        visor_step: Option<usize>,
        charge_glow_position: Option<usize>,
    }

    impl NamedMemory {
        /// `base`, with each quantity this names set to its value.
        fn over(self, base: AppearanceState) -> AppearanceState {
            let colours = base.colours;
            AppearanceState {
                cannon: self.cannon_delay.map_or(base.cannon, |frames| {
                    CannonDelay::new(frames)
                        .unwrap_or_else(|| panic!("a delay of {frames} is outside the spec"))
                }),
                colours: ColourSequences {
                    visor_delay: self.visor_delay.map_or(colours.visor_delay, VisorDelay),
                    visor_step: self.visor_step.map_or(colours.visor_step, VisorStep),
                    charge_glow_position: self
                        .charge_glow_position
                        .map_or(colours.charge_glow_position, ChargeGlowPosition),
                },
                ..base
            }
        }
    }

    /// The frame's inputs, one name for each fact in every kind of case, and whether its work is
    /// prepared.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields, default)]
    #[expect(
        clippy::struct_excessive_bools,
        reason = "the vector file gives each of these inputs as a flag of its own"
    )]
    struct VectorInputs {
        pose: String,
        clip: Option<String>,
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
        cinematic: bool,
        boss: bool,
        special_colours: bool,
        shoot_held: bool,
        charge: u16,
        prepared: bool,
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
                clip: None,
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
                cinematic: false,
                boss: false,
                special_colours: false,
                shoot_held: false,
                charge: 0,
                prepared: false,
            }
        }
    }

    /// One sound rule of the frame: its kind, the State before and after, the frame's inputs,
    /// and what it asks for.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct OpportunityVector {
        id: String,
        source: String,
        kind: OpportunityKind,
        entries: Vec<String>,
        before: Before,
        after: After,
        inputs: VectorInputs,
        expected: ExpectedOpportunity,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
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

    /// A part of a combined frame, as the file names it.
    #[derive(Clone, Copy, Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum PartName {
        Initialization,
        Movement,
        Preparation,
        Drawing,
    }

    /// One whole frame combining rules checked alone elsewhere, with one set of inputs.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct AppearanceWork {
        id: String,
        source: String,
        entries: Vec<String>,
        parts: Vec<PartName>,
        before: Before,
        inputs: VectorInputs,
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
    const REQUESTS: &[SoundRequest] = &[
        SoundRequest::Footstep,
        SoundRequest::Boosting,
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
            contacts: table.contacts.iter().copied().map(Step).collect(),
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

    /// The State an animation vector or a sound opportunity starts from besides what it names:
    /// a cannon delay and colours apart from initialization's, which a frame without item
    /// controls, initialization or a palette opportunity leaves alone.
    fn untouched() -> AppearanceState {
        AppearanceState {
            cannon: untouched_cannon(),
            colours: UNTOUCHED_COLOURS,
            ..AppearanceState::INITIAL
        }
    }

    /// Her appearance State before a frame, as the file gives it: the cannon delay and colour
    /// sequences `unnamed` holds, where it names none.
    fn before_state(given: &Before, unnamed: AppearanceState) -> AppearanceState {
        let memory = NamedMemory {
            cannon_delay: given.cannon_delay,
            visor_delay: given.visor_delay,
            visor_step: given.visor_step,
            charge_glow_position: given.charge_glow_position,
        };
        memory.over(AppearanceState {
            animation: state(given.step, given.left, given.stage, given.laps),
            kept_action: named(ACTIONS, &given.kept),
            ..unnamed
        })
    }

    /// Her appearance State after a frame, as the file gives it: each quantity besides the four
    /// it always states as `before`, where it doesn't name one.
    fn after_state(given: &After, before: AppearanceState) -> AppearanceState {
        let memory = NamedMemory {
            cannon_delay: given.cannon_delay,
            visor_delay: given.visor_delay,
            visor_step: given.visor_step,
            charge_glow_position: given.charge_glow_position,
        };
        memory.over(AppearanceState {
            animation: state(given.step, given.left, given.stage, given.laps),
            kept_action: given
                .kept
                .as_deref()
                .map_or(before.kept_action, |kept| named(ACTIONS, kept)),
            ..before
        })
    }

    /// The frame of `given`, with the parts in `parts`, and its preparation if `given` says its
    /// work is prepared. Her item controls don't run, and it has no palette opportunity.
    fn frame(given: &VectorInputs, mut parts: Set<FramePart>) -> Frame {
        parts[FramePart::Preparation] |= given.prepared;
        Frame {
            inputs: inputs(given),
            parts,
            cannon: CannonEvent::ControlsSkipped,
            palette: PaletteOpportunity::Absent,
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

    /// The file's clips and the run's paces, read once.
    struct Library {
        clips: BTreeMap<String, Clip>,
        paces: Paces,
    }

    impl Library {
        fn clip(&self, name: &str) -> &Clip {
            self.clips
                .get(name)
                .unwrap_or_else(|| panic!("no clip {name}"))
        }
    }

    fn read_file() -> (VectorFile, Library) {
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
        (file, Library { clips, paces })
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
        let mut activities = Set::<Activity>::default();
        activities[Activity::Cinematic] = given.cinematic;
        activities[Activity::Boss] = given.boss;
        activities[Activity::SpecialColourTime] = given.special_colours;
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
            activities,
            shoot_held: given.shoot_held,
            charge: Charge::new(given.charge)
                .unwrap_or_else(|| panic!("a charge of {} is outside the spec", given.charge)),
        }
    }

    /// The clip a case's moving frame plays where the case names none: the file's `long`, an
    /// invented clip of shown steps that no rule a case checks reads, as the recorded vectors
    /// use it.
    const UNREAD_CLIP: &str = "long";

    /// The content of a moving frame: the clip of the figure she starts the frame in and of the
    /// one she takes, as `given` names them.
    fn moving_content<'a>(
        id: &str,
        given: &VectorInputs,
        library: &'a Library,
    ) -> VectorContent<'a> {
        let inputs = inputs(given);
        assert!(
            inputs.figure != inputs.figure_after || given.clip_after.is_none(),
            "{id}: another clip for the same pose"
        );
        let clip = given.clip.as_deref().unwrap_or(UNREAD_CLIP);
        let clip_after = given.clip_after.as_deref().unwrap_or(clip);
        VectorContent {
            clips: vec![
                (inputs.figure, library.clip(clip)),
                (inputs.figure_after, library.clip(clip_after)),
            ],
            paces: &library.paces,
        }
    }

    fn end(name: &str) -> Option<ClipEnd> {
        match name {
            "None" => None,
            other => Some(named(&[ClipEnd::End, ClipEnd::EndUnlessJump], other)),
        }
    }

    /// The sounds the file names, in its order.
    fn sounds(names: &[String]) -> Vec<SoundRequest> {
        names.iter().map(|name| named(REQUESTS, name)).collect()
    }

    /// Runs one animation vector as a frame with her movement alone; its mismatches, if any.
    fn run(vector: &Vector, library: &Library) -> Option<String> {
        let given = &vector.inputs;
        assert!(
            !given.prepared,
            "{}: a vector has her moves alone",
            vector.id
        );
        let content = moving_content(&vector.id, given, library);
        let before = before_state(&vector.before, untouched());
        let frame = frame(given, set_of([FramePart::Movement]));
        let ahead = Ahead {
            end: end(&vector.ahead.end),
            turned_far: vector.ahead.turned_far,
        };
        assert_eq!(
            Ahead::of(&kept(before), content.clip(&frame.inputs.figure)),
            ahead,
            "{}: the read-ahead samus-movement asks before its step",
            vector.id
        );
        let (after, outputs) = play(before, &frame, &content, &rules(&vector.rules));
        let expected = (
            after_state(&vector.after, before),
            sounds(&vector.after.sounds),
            None,
            DryColourReset::NoOpportunity,
            before.cannon.may_follow(),
        );
        let actual = (
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

    /// The part of the frame a sound rule's case plays: her movement for her stride and the pose
    /// she takes, which come with her clip playing, or the passage entry or drawing alone.
    fn part_of(kind: OpportunityKind) -> FramePart {
        match kind {
            OpportunityKind::Contact | OpportunityKind::PoseSelected => FramePart::Movement,
            OpportunityKind::NormalDraw => FramePart::Drawing,
            OpportunityKind::PassageEntry => FramePart::PassageEntry,
        }
    }

    /// Her stride's graphical contact is the figure she runs in as the frame starts and her
    /// playhead then, before her clip plays.
    fn check_contact(outputs: &FrameOutputs, frame: &Frame, at_start: State, id: &str) {
        let Some(contact) = outputs.contact else {
            return;
        };
        assert_eq!(contact.figure.pose.action, Action::Running, "{id}: contact");
        assert_eq!(
            contact.figure, frame.inputs.figure,
            "{id}: the contact's figure is hers as the frame starts"
        );
        assert_eq!(
            contact.playhead, at_start.playhead,
            "{id}: contact playhead"
        );
    }

    /// One sound rule's case through the one update, as a whole frame. Her stride and the pose
    /// she takes are a frame with her movement alone: her clip plays and the clip of the pose she
    /// takes starts, beside the rule's sounds. A drawing (with her preparation, where the case
    /// says) or a passage entry is a frame without her movement. Each leaves her whole State as
    /// `after` says.
    fn run_opportunity(vector: &OpportunityVector, library: &Library) {
        let id = &vector.id;
        let given = &vector.inputs;
        let part = part_of(vector.kind);
        let before = before_state(&vector.before, untouched());
        let frame = frame(given, set_of([part]));
        let content = moving_content(id, given, library);
        let (after, outputs) = play(before, &frame, &content, &Rules::VANILLA);
        assert_eq!(
            after,
            after_state(&vector.after, before),
            "{id} ({}): her whole State, as the frame leaves it",
            vector.source
        );
        assert_eq!(outputs.contact.is_some(), vector.expected.contact, "{id}");
        assert_eq!(
            outputs.sounds,
            sounds(&vector.expected.sounds),
            "{id}: ordered sounds"
        );
        check_contact(&outputs, &frame, before.animation, id);
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
            assert!(
                vector.after.sounds.is_empty(),
                "{}: sounds are expected",
                vector.id
            );
            run_opportunity(vector, &library);
        }
        println!(
            "{} sound-opportunity vectors passed",
            file.opportunity.len()
        );
    }

    /// The model's part for a part the file names; her initialization is none of these, but the
    /// frame's cannon part, in place of her item controls.
    const fn frame_part(name: PartName) -> Option<FramePart> {
        match name {
            PartName::Initialization => None,
            PartName::Movement => Some(FramePart::Movement),
            PartName::Preparation => Some(FramePart::Preparation),
            PartName::Drawing => Some(FramePart::Drawing),
        }
    }

    /// The frame of a combined case: its one set of inputs and the parts it names, each once.
    fn work_frame(work: &AppearanceWork) -> Frame {
        let id = &work.id;
        let mut parts = Set::<FramePart>::default();
        let mut cannon = CannonEvent::ControlsSkipped;
        for &name in &work.parts {
            if let Some(part) = frame_part(name) {
                assert!(!parts[part], "{id}: a second {part:?}");
                parts[part] = true;
            } else {
                assert_eq!(
                    cannon,
                    CannonEvent::ControlsSkipped,
                    "{id}: initialized twice"
                );
                cannon = CannonEvent::SamusInitialized;
            }
        }
        Frame {
            cannon,
            ..frame(&work.inputs, parts)
        }
    }

    /// Plays one combined frame in the one update. Its `before` holds initialization's cannon
    /// delay and colour sequences where it names none.
    fn run_work(work: &AppearanceWork, library: &Library) {
        let id = &work.id;
        let frame = work_frame(work);
        let content = moving_content(id, &work.inputs, library);
        let before = before_state(&work.before, AppearanceState::INITIAL);
        let (after, outputs) = play(before, &frame, &content, &Rules::VANILLA);
        assert!(work.after.sounds.is_empty(), "{id}: sounds are top-level");
        assert_eq!(
            after,
            after_state(&work.after, before),
            "{id} ({}): her State, with one animation update at most",
            work.source
        );
        assert_eq!(
            outputs.sounds,
            sounds(&work.sounds),
            "{id}: every request, in order"
        );
        assert_eq!(outputs.contact.is_some(), work.contact, "{id}: contact");
        check_contact(&outputs, &frame, before.animation, id);
    }

    /// The file's initialization table is the State the crate gives her at initialization, and
    /// it saves and restores as it is.
    #[test]
    fn her_state_at_initialization_is_the_files() {
        let (file, _) = read_file();
        let given = &file.initialization;
        let expected = AppearanceState {
            animation: state(given.step, given.left, given.stage, given.laps),
            kept_action: named(ACTIONS, &given.kept),
            cannon: CannonDelay::new(given.cannon_delay).expect("a delay the spec allows"),
            colours: ColourSequences {
                visor_delay: VisorDelay(given.visor_delay),
                visor_step: VisorStep(given.visor_step),
                charge_glow_position: ChargeGlowPosition(given.charge_glow_position),
            },
        };
        assert_eq!(AppearanceState::INITIAL, expected);
        assert_eq!(kept(AppearanceState::INITIAL), expected);
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
            run_work(work, &library);
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
