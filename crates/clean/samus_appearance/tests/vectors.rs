//! Runs every vector of `scenarios/vectors/samus-appearance.toml` through the model's one
//! update, a frame at a time, with her appearance State saved and restored between frames. A key
//! or a name the file holds that this test doesn't know fails it.

#[cfg(test)]
mod vectors {
    mod cannon_vectors;
    mod colour_vectors;
    mod hurt_vectors;

    use core::num::Wrapping;
    use std::collections::BTreeMap;

    use enum_map::EnumMap;
    use samus_appearance::{
        Activity, Ahead, Animation, AppearanceContent, AppearanceState, BehaviourId, Branch,
        CannonDelay, Charge, ChargeGlowPosition, Clip, ColourSequences, Control, Dash, Figure,
        Frame, FrameOutputs, FramePart, Frames, Hold, HurtCount, HurtOpportunity, Inputs,
        Instruction, ItemControls, Laps, LapsLeft, Liquid, MissingClip, MoveEvent, Pace, Paces,
        PaletteOpportunity, Playhead, PulseColours, Rules, ShownFrames, Slowdown, Sound,
        SoundsWaiting, Step, VisorDelay, VisorStep, catalogue, update,
    };
    use serde::Deserialize;
    use serde::de::DeserializeOwned;
    use types::{
        Action, BoostStage, Checked, ClipEnd, Fixed, Gun, Item, Legs, Set, Sprite, Switch,
    };
    use types::{fixed, set_of};

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/samus-appearance.toml");

    /// The vector file: her State at initialization, the paces, the clips by name, the animation
    /// vectors, the sound opportunities, the combined frames, the cannon histories, the palette
    /// histories and the hurt colours frames.
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
        hurt_colours: Vec<hurt_vectors::HurtVector>,
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
        visor_step: u8,
        charge_glow_position: u8,
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
        restart_frames: Option<u8>,
        fast_fall_speed: Option<String>,
        fast_fall_frames: Option<u8>,
        rising_frames: Option<u8>,
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
        visor_step: Option<u8>,
        charge_glow_position: Option<u8>,
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
        visor_step: Option<u8>,
        charge_glow_position: Option<u8>,
        #[serde(default)]
        sounds: Vec<String>,
    }

    /// The cannon delay and colour-sequence quantities a `before` or `after` names, each
    /// optional.
    struct NamedMemory {
        cannon_delay: Option<u8>,
        visor_delay: Option<u8>,
        visor_step: Option<u8>,
        charge_glow_position: Option<u8>,
    }

    impl NamedMemory {
        /// `base`, with each quantity this names set to its value.
        fn over(self, base: AppearanceState) -> AppearanceState {
            let colours = base.colours;
            AppearanceState {
                cannon: self.cannon_delay.map_or(base.cannon, |frames| {
                    CannonDelay::new(frames).unwrap_or_else(|| {
                        panic!("expected a cannon delay of 0 to 2 frames, found {frames}")
                    })
                }),
                colours: ColourSequences {
                    visor_delay: self
                        .visor_delay
                        .map_or(colours.visor_delay, VisorDelay::new),
                    visor_step: self.visor_step.map_or(colours.visor_step, VisorStep::new),
                    charge_glow_position: self
                        .charge_glow_position
                        .map_or(colours.charge_glow_position, ChargeGlowPosition::new),
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
        dash: String,
        submerged: bool,
        run_held: bool,
        handed_over: bool,
        standing_restarts: bool,
        wall_jump_readied: bool,
        heading_down: bool,
        step_slowdown: u8,
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
    /// The file header's default dash.
    const DEFAULT_DASH: &str = "Off";
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
                dash: DEFAULT_DASH.to_owned(),
                submerged: false,
                run_held: false,
                handed_over: false,
                standing_restarts: false,
                wall_jump_readied: false,
                heading_down: false,
                step_slowdown: 0,
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

    /// The member the file names `name`, by its Rust name, read as the type's own serde form
    /// reads it; any other name fails the test.
    fn named<T: DeserializeOwned>(name: &str) -> T {
        toml::Value::String(name.to_owned())
            .try_into()
            .unwrap_or_else(|_| panic!("expected the name of a member, found {name:?}"))
    }

    /// An action as the file writes it: a name, or a name with the turn or the landing it holds
    /// in brackets, "Turning(Ground)".
    fn action(name: &str) -> Action {
        let Some((variant, held)) = name.strip_suffix(')').and_then(|rest| rest.split_once('('))
        else {
            return named(name);
        };
        let held = toml::Value::String(held.to_owned());
        toml::Value::Table(std::iter::once((variant.to_owned(), held)).collect())
            .try_into()
            .unwrap_or_else(|_| panic!("expected an action, found {name:?}"))
    }

    /// A pose with its sprite: "BODY ACTION FACING AIM", then "Drawn", "Striding" and a sprite,
    /// each if given.
    fn figure(text: &str) -> Figure {
        let words: Vec<&str> = text.split_whitespace().collect();
        let [body, action_name, facing, aim, rest @ ..] = words.as_slice() else {
            panic!("expected a pose \"BODY ACTION FACING AIM\", found {text:?}");
        };
        let mut figure = Figure {
            pose: types::Pose {
                body: named(body),
                action: action(action_name),
                facing: named(facing),
                aim: named(aim),
                gun: Gun::Stowed,
                legs: Legs::Together,
            },
            sprite: Sprite::Plain,
        };
        for word in rest {
            match *word {
                "Drawn" => figure.pose.gun = Gun::Drawn,
                "Striding" => figure.pose.legs = Legs::Striding,
                sprite => figure.sprite = named(sprite),
            }
        }
        figure
    }

    /// A step's number, written in a cell.
    fn step_number(text: &str) -> Step {
        Step(
            text.parse()
                .unwrap_or_else(|_| panic!("expected a step number, found {text:?}")),
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
                    .unwrap_or_else(|_| panic!("expected a step's cell, found {cell:?}")),
            )),
            _ => panic!("expected a step's cell, found {cell:?}"),
        }
    }

    /// A hold the content gives, a clip's or a pace's: 1 frame or more, and at most
    /// `Hold::LONGEST`; any other fails the test.
    fn hold(frames: u8) -> Hold {
        Hold::new(frames)
            .unwrap_or_else(|| panic!("expected a hold of 1 to 127 frames, found {frames}"))
    }

    /// A tuner's frames shown: 1 to 126; any other fails the test.
    fn shown(frames: u8) -> ShownFrames {
        ShownFrames::new(frames)
            .unwrap_or_else(|| panic!("expected 1 to 126 frames shown, found {frames}"))
    }

    /// A slowdown the file gives: 0 to 3 frames; any other fails the test.
    fn slowdown(frames: u8) -> Slowdown {
        Slowdown::new(frames)
            .unwrap_or_else(|| panic!("expected a slowdown of 0 to 3 frames, found {frames}"))
    }

    /// The digits of each part of a vertical speed.
    const PART_DIGITS: usize = 4;
    /// The base the parts are written in.
    const HEX: u32 = 16;

    /// A speed's size, "PPPP.SSSS": the pixel part and the subpixel part, four hex digits each;
    /// any other text fails the test.
    fn speed_size(text: &str) -> Fixed {
        let part = |digits: &str| {
            assert_eq!(
                digits.len(),
                PART_DIGITS,
                "expected a speed \"PPPP.SSSS\", found {text:?}"
            );
            u16::from_str_radix(digits, HEX)
                .unwrap_or_else(|_| panic!("expected a speed \"PPPP.SSSS\", found {text:?}"))
        };
        let (pixel, sub) = text
            .split_once('.')
            .unwrap_or_else(|| panic!("expected a speed \"PPPP.SSSS\", found {text:?}"));
        fixed(Wrapping(part(pixel)), part(sub))
    }

    /// The header's vertical speed, "+PPPP.SSSS" or "-PPPP.SSSS": a sign, then its size, downward
    /// positive; any other text fails the test.
    fn vertical_speed(text: &str) -> Fixed {
        let expected = format!("expected a vertical speed \"+PPPP.SSSS\", found {text:?}");
        let (sign, size) = text.split_at_checked(1).expect(&expected);
        match sign {
            "+" => speed_size(size),
            "-" => -speed_size(size),
            _ => panic!("{expected}"),
        }
    }

    /// A vector's tuning: the game's, with the parameters its `rules` name set to its values.
    fn rules(given: &RulesTable) -> Rules {
        let game = Rules::VANILLA;
        Rules {
            breathing_threshold: given
                .breathing_threshold
                .unwrap_or(game.breathing_threshold),
            restart_frames: given.restart_frames.map_or(game.restart_frames, shown),
            fast_fall_speed: given
                .fast_fall_speed
                .as_deref()
                .map_or(game.fast_fall_speed, speed_size),
            fast_fall_frames: given.fast_fall_frames.map_or(game.fast_fall_frames, shown),
            rising_frames: given.rising_frames.map_or(game.rising_frames, shown),
            laps_per_stage: given.laps_per_stage.map_or(game.laps_per_stage, |laps| {
                EnumMap::from_array(laps.map(|count| {
                    Laps::new(count).unwrap_or_else(|| {
                        panic!("expected a stage of 1 lap or more, found {count}")
                    })
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
    fn animation(step: u8, left: u8, stage: u8, laps: u8) -> Animation {
        Animation {
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
        CannonDelay::new(1).expect("expected a delay of 1 to be in range")
    }

    /// The colour sequences the animation and sound frames start from: none of them has a
    /// palette opportunity, so each leaves them as they were.
    const UNTOUCHED_COLOURS: ColourSequences = ColourSequences {
        visor_delay: VisorDelay::new(19),
        visor_step: VisorStep::new(2),
        charge_glow_position: ChargeGlowPosition::new(4),
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
            animation: animation(given.step, given.left, given.stage, given.laps),
            kept_action: action(&given.kept),
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
            animation: animation(given.step, given.left, given.stage, given.laps),
            kept_action: given.kept.as_deref().map_or(before.kept_action, action),
            ..before
        })
    }

    /// The frame of `given`, with the parts in `parts`, and its preparation if `given` says its
    /// work is prepared. Her item controls don't run, and it has no palette opportunity and no
    /// hurt colours.
    fn frame(given: &VectorInputs, mut parts: Set<FramePart>) -> Frame {
        parts[FramePart::Preparation] |= given.prepared;
        Frame {
            inputs: inputs(given),
            parts,
            item_controls: ItemControls::Skipped,
            palette: PaletteOpportunity::Absent,
            hurt: HurtOpportunity::Absent,
        }
    }

    /// Her appearance State as a caller keeps it between frames: saved whole with serde and
    /// restored fresh from the saved text alone, the same.
    fn kept(state: AppearanceState) -> AppearanceState {
        let text = toml::to_string(&state).expect("expected the appearance State to save");
        let restored: AppearanceState =
            toml::from_str(&text).expect("expected the saved appearance State to restore");
        assert_eq!(
            toml::to_string(&restored).expect("expected the restored State to save"),
            text,
            "expected the State restored as saved"
        );
        restored
    }

    /// Plays one frame from the State the caller kept, and keeps the State it leaves.
    fn play(
        state: AppearanceState,
        frame: &Frame,
        content: &impl AppearanceContent,
        rules: &Rules,
    ) -> (AppearanceState, FrameOutputs) {
        let (after, outputs) = update(kept(state), frame, content, rules)
            .unwrap_or_else(|MissingClip(figure)| panic!("expected a clip for {figure:?}"));
        (kept(after), outputs)
    }

    /// A frame's content: the clip of each figure the file names one for, the run's paces and
    /// the invented colours of her hurt pulse.
    struct VectorContent<'a> {
        clips: Vec<(Figure, &'a Clip)>,
        library: &'a Library,
    }

    impl AppearanceContent for VectorContent<'_> {
        fn clip(&self, figure: &Figure) -> Option<&Clip> {
            self.clips
                .iter()
                .find(|(given, _)| given == figure)
                .map(|&(_, clip)| clip)
        }

        fn paces(&self) -> &Paces {
            &self.library.paces
        }

        fn pulse_colours(&self) -> &PulseColours {
            &self.library.pulse_colours
        }
    }

    /// The file's clips and the run's paces, read once, and the hurt pulse's invented colours.
    struct Library {
        clips: BTreeMap<String, Clip>,
        paces: Paces,
        pulse_colours: PulseColours,
    }

    impl Library {
        fn clip(&self, name: &str) -> &Clip {
            self.clips
                .get(name)
                .unwrap_or_else(|| panic!("expected a clip named {name}"))
        }

        /// Content holding no clip, with the paces and pulse colours.
        const fn without_clips(&self) -> VectorContent<'_> {
            VectorContent {
                clips: Vec::new(),
                library: self,
            }
        }
    }

    fn read_file() -> (VectorFile, Library) {
        let file: VectorFile = toml::from_str(VECTORS).expect("expected the vector file to read");
        let clips = file
            .clips
            .iter()
            .map(|(name, table)| (name.clone(), clip(table)))
            .collect();
        let paces = Paces {
            plain: pace(&file.paces.plain),
            boosted: EnumMap::from_array(file.paces.boosted.each_ref().map(|holds| pace(holds))),
        };
        let library = Library {
            clips,
            paces,
            pulse_colours: hurt_vectors::invented_pulse_colours(),
        };
        (file, library)
    }

    /// The behaviour the file names `name`, among the catalogue's own ids.
    fn behaviour(name: &str) -> BehaviourId {
        catalogue()
            .into_values()
            .map(|behaviour| behaviour.id)
            .find(|id| format!("{id:?}") == name)
            .unwrap_or_else(|| panic!("expected a behaviour of the catalogue, found {name:?}"))
    }

    fn inputs(given: &VectorInputs) -> Inputs {
        let start = figure(&given.pose);
        let mut moves = Set::<MoveEvent>::default();
        moves[MoveEvent::StandingRestart] = given.standing_restarts;
        moves[MoveEvent::WallJumpReadied] = given.wall_jump_readied;
        moves[MoveEvent::HandedOver] = given.handed_over;
        let mut items = Set::<Item>::default();
        for item in &given.items {
            items[named::<Item>(item)] = true;
        }
        let mut activities = Set::<Activity>::default();
        activities[Activity::Cinematic] = given.cinematic;
        activities[Activity::Boss] = given.boss;
        activities[Activity::SpecialColourTime] = given.special_colours;
        Inputs {
            figure: start,
            figure_after: given.pose_after.as_deref().map_or(start, figure),
            moves,
            dash: named::<Dash>(&given.dash),
            run_held: given.run_held,
            heading_down: given.heading_down,
            vertical_speed: vertical_speed(&given.vertical_speed),
            energy: given.energy,
            // No frame of these tables has her energy fall: the hurt frames give that themselves.
            previous_energy: given.energy,
            items,
            liquid: Liquid {
                step_slowdown: slowdown(given.step_slowdown),
                start_slowdown: slowdown(given.start_slowdown),
                submerged: given.submerged,
            },
            sounds_waiting: SoundsWaiting::new(given.sounds_waiting).unwrap_or_else(|| {
                panic!(
                    "expected 0 to 15 sounds waiting, found {}",
                    given.sounds_waiting
                )
            }),
            activities,
            shoot_held: given.shoot_held,
            charge: Charge::new(given.charge).unwrap_or_else(|| {
                panic!(
                    "expected a charge of 0 to 32767 frames, found {}",
                    given.charge
                )
            }),
        }
    }

    /// The content of a frame: the clip `given` names for the figure she starts the frame in,
    /// and for the one she takes (by default the same clip), each where `given` names one.
    fn moving_content<'a>(
        id: &str,
        given: &VectorInputs,
        library: &'a Library,
    ) -> VectorContent<'a> {
        let inputs = inputs(given);
        assert!(
            inputs.figure != inputs.figure_after || given.clip_after.is_none(),
            "{id}: expected one clip for one figure"
        );
        let clip_after = given.clip_after.as_deref().or(given.clip.as_deref());
        let clips = [
            (inputs.figure, given.clip.as_deref()),
            (inputs.figure_after, clip_after),
        ]
        .into_iter()
        .filter_map(|(figure, name)| Some((figure, library.clip(name?))))
        .collect();
        VectorContent { clips, library }
    }

    fn end(name: &str) -> Option<ClipEnd> {
        match name {
            "None" => None,
            other => Some(named(other)),
        }
    }

    /// The sounds the file names, in its order.
    fn sounds(names: &[String]) -> Vec<Sound> {
        names.iter().map(|name| named(name)).collect()
    }

    /// The sounds of a frame's requests, in its order, each request carrying the queue and
    /// allowance of its sound.
    fn asked(outputs: &FrameOutputs) -> Vec<Sound> {
        outputs
            .sounds
            .iter()
            .map(|offer| {
                assert_eq!(
                    *offer,
                    offer.sound.offer(),
                    "expected the queue and allowance of {:?}",
                    offer.sound
                );
                offer.sound
            })
            .collect()
    }

    /// Runs one animation vector as a frame with her movement alone; its mismatches, if any.
    fn run(vector: &Vector, library: &Library) -> Option<String> {
        let given = &vector.inputs;
        assert!(
            !given.prepared,
            "{}: expected a vector to have her moves alone",
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
            Ahead::of(&kept(before), frame.inputs.figure, &content),
            Ok(ahead),
            "{}: expected the read-ahead samus-movement asks before its step",
            vector.id
        );
        let (after, outputs) = play(before, &frame, &content, &rules(&vector.rules));
        let expected = (
            after_state(&vector.after, before),
            sounds(&vector.after.sounds),
            None,
            None,
            before.cannon.may_follow(),
        );
        let actual = (
            after,
            asked(&outputs),
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
            "expected every vector to pass; {} of {} fail:\n{}",
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
        let catalogue = catalogue();
        let fixed: Vec<String> = file
            .vector
            .iter()
            .filter(|vector| rules(&vector.rules) != Rules::VANILLA)
            .flat_map(|vector| {
                vector
                    .entries
                    .iter()
                    .filter(|entry| catalogue[behaviour(entry)].switch != Switch::Parameter)
                    .map(|entry| format!("{}: {entry}", vector.id))
            })
            .collect();
        assert!(
            fixed.is_empty(),
            "expected adjusted vectors to exercise parameters alone, found {fixed:?}"
        );
    }

    /// A frame with her movement whose content lacks the clip of a figure she starts or ends it
    /// in, and the read-ahead of a figure with no clip, give that figure back as missing.
    #[test]
    fn a_figure_with_no_clip_is_given_back() {
        let (_, library) = read_file();
        let given = VectorInputs {
            pose: "Upright Still Right Forward".to_owned(),
            pose_after: Some("Upright TakingOff Right Forward".to_owned()),
            ..VectorInputs::default()
        };
        let frame = frame(&given, set_of([FramePart::Movement]));
        let none = library.without_clips();
        let state = AppearanceState::INITIAL;
        assert_eq!(
            update(state, &frame, &none, &Rules::VANILLA),
            Err(MissingClip(frame.inputs.figure)),
            "expected the starting figure missing"
        );
        assert_eq!(
            Ahead::of(&state, frame.inputs.figure, &none),
            Err(MissingClip(frame.inputs.figure)),
            "expected the read-ahead's figure missing"
        );
        let only_start = VectorContent {
            clips: vec![(frame.inputs.figure, library.clip("stand"))],
            library: &library,
        };
        assert_eq!(
            update(state, &frame, &only_start, &Rules::VANILLA),
            Err(MissingClip(frame.inputs.figure_after)),
            "expected the figure she takes missing"
        );
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
    fn check_contact(outputs: &FrameOutputs, frame: &Frame, at_start: Animation, id: &str) {
        let Some(contact) = outputs.contact else {
            return;
        };
        assert_eq!(
            contact.figure.pose.action,
            Action::Running,
            "{id}: expected the contact of a run"
        );
        assert_eq!(
            contact.figure, frame.inputs.figure,
            "{id}: expected the contact's figure to be hers as the frame starts"
        );
        assert_eq!(
            contact.playhead, at_start.playhead,
            "{id}: expected the contact's playhead to be hers as the frame starts"
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
            "{id} ({}): expected her whole State as `after` gives it",
            vector.source
        );
        assert_eq!(
            outputs.contact.is_some(),
            vector.expected.contact,
            "{id}: expected the graphical contact as `expected` gives it"
        );
        assert_eq!(
            asked(&outputs),
            sounds(&vector.expected.sounds),
            "{id}: expected the sounds as `expected` gives them, in order"
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
                "{}: expected the sounds under `expected`",
                vector.id
            );
            run_opportunity(vector, &library);
        }
        println!(
            "{} sound-opportunity vectors passed",
            file.opportunity.len()
        );
    }

    /// The model's part for a part the file names.
    const fn frame_part(name: PartName) -> FramePart {
        match name {
            PartName::Initialization => FramePart::Initialization,
            PartName::Movement => FramePart::Movement,
            PartName::Preparation => FramePart::Preparation,
            PartName::Drawing => FramePart::Drawing,
        }
    }

    /// The frame of a combined case: its one set of inputs and the parts it names, each once.
    fn work_frame(work: &AppearanceWork) -> Frame {
        let mut parts = Set::<FramePart>::default();
        for &name in &work.parts {
            let part = frame_part(name);
            assert!(!parts[part], "{}: expected {part:?} once", work.id);
            parts[part] = true;
        }
        frame(&work.inputs, parts)
    }

    /// Plays one combined frame in the one update. Its `before` holds initialization's cannon
    /// delay and colour sequences where it names none.
    fn run_work(work: &AppearanceWork, library: &Library) {
        let id = &work.id;
        let frame = work_frame(work);
        let content = moving_content(id, &work.inputs, library);
        let before = before_state(&work.before, AppearanceState::INITIAL);
        let (after, outputs) = play(before, &frame, &content, &Rules::VANILLA);
        assert!(
            work.after.sounds.is_empty(),
            "{id}: expected the sounds at the case's top level"
        );
        assert_eq!(
            after,
            after_state(&work.after, before),
            "{id} ({}): expected her State as `after` gives it, from one animation update at most",
            work.source
        );
        assert_eq!(
            asked(&outputs),
            sounds(&work.sounds),
            "{id}: expected every request `sounds` gives, in order"
        );
        assert_eq!(
            outputs.contact.is_some(),
            work.contact,
            "{id}: expected the graphical contact `contact` gives"
        );
        check_contact(&outputs, &frame, before.animation, id);
    }

    /// The file's initialization table is the State the crate gives her at initialization, and
    /// it saves and restores as it is. The table names no hurt count: initialization leaves no
    /// pulse under way.
    #[test]
    fn her_state_at_initialization_is_the_files() {
        let (file, _) = read_file();
        let given = &file.initialization;
        let expected = AppearanceState {
            animation: animation(given.step, given.left, given.stage, given.laps),
            kept_action: action(&given.kept),
            cannon: CannonDelay::new(given.cannon_delay)
                .expect("expected a cannon delay of 0 to 2 frames"),
            colours: ColourSequences {
                visor_delay: VisorDelay::new(given.visor_delay),
                visor_step: VisorStep::new(given.visor_step),
                charge_glow_position: ChargeGlowPosition::new(given.charge_glow_position),
            },
            hurt_count: HurtCount::NONE,
        };
        assert_eq!(
            AppearanceState::INITIAL,
            expected,
            "expected the file's [initialization]"
        );
        assert_eq!(
            kept(AppearanceState::INITIAL),
            expected,
            "expected the State at initialization to restore as saved"
        );
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
            .chain(file.hurt_colours.iter().flat_map(|frame| &frame.entries))
            .map(|entry| behaviour(entry))
            .chain(file.ordinary_dry_colours.entries().map(behaviour))
            .collect();
        let missing: Vec<BehaviourId> = catalogue()
            .into_values()
            .filter(|behaviour| behaviour.checked == Checked::ByVectors)
            .map(|behaviour| behaviour.id)
            .filter(|id| !exercised.contains(id))
            .collect();
        assert!(
            missing.is_empty(),
            "expected a vector for every behaviour checked by vectors; none exercises {missing:?}"
        );
    }
}
