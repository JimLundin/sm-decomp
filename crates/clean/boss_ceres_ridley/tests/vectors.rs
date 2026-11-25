//! Runs every vector of `scenarios/vectors/boss-ceres-ridley.toml` through the model, the doors',
//! the shaft's and Ridley's (his tail's among them), and checks the catalogue against the
//! vectors. Every table refuses a key it doesn't know, and every name must be one the spec gives.

#[cfg(test)]
mod vectors {
    use core::num::{NonZeroU8, NonZeroU16, Wrapping};

    use boss_ceres_ridley::{
        AlternatingLook, Baby, BabyColours, BabyFigure, BabyLook, BabyRoutine, BeaconStep,
        BehaviourId, BodyColours, Boss, CeresDoor, CeresRidley, ColourSet, Cover, DoorKind,
        DoorPalette, EyeColours, FadeSets, FadeSteps, FidgetFigure, Figure, FigureLook, GlowFigure,
        Her, Inputs, Lights, LightsInputs, Pad, PadAndCover, PadAndCoverStep, PadFigure, PadWait,
        Phase, Quarter, ReadyWait, RidleyContent, RidleyLook, RidleyPhase, RidleyWait, RoarFigure,
        RoarWait, Rules, Shake, ShakeLayers, ShakeStart, ShownFigure, Stage, TailCurl,
        TimedRoarFigure, Wait, WakeWait, catalogue,
    };
    use enum_map::Enum;
    use serde::Deserialize;
    use types::{Checked, Colour, Hit, Pair, Px, Side};

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/boss-ceres-ridley.toml");

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VectorFile {
        vector: Vec<Vector>,
    }

    /// A vector of any part: the doors' fields (the door of Ridley's room and the getaway's walls
    /// among them), the shaft's (the pad and the cover, or the lights), or Ridley's. Which part
    /// it is follows from the result it names; Ridley's part-4b vectors, his tail's, are the
    /// ones whose ids start `T-`.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Vector {
        id: String,
        #[serde(default)]
        placing: bool,
        source: Source,
        entries: Vec<String>,
        #[serde(default)]
        rules: RulesRow,
        // The doors' and Ridley's alike.
        random: Option<String>,
        beaten: Option<bool>,
        // The doors.
        door: Option<DoorRow>,
        before: Option<PhaseRow>,
        her: Option<HerField>,
        stage: Option<StageName>,
        shaking: Option<u16>,
        after: Option<After>,
        // Ridley.
        #[serde(default)]
        ridley_placing: bool,
        ridley: Option<RidleyRow>,
        baby: Option<BabyRow>,
        passing: Option<bool>,
        content: Option<ContentRow>,
        ridley_after: Option<RidleyAfter>,
        // The pad and the cover.
        arrival: Option<[u16; 2]>,
        pad: Option<PadRow>,
        cover: Option<[u16; 2]>,
        her_y: Option<u16>,
        shaft_after: Option<ShaftAfter>,
        // The lights.
        lights: Option<LightsRow>,
        lights_after: Option<LightsAfter>,
    }

    /// A vector's `her`: where she was, as a door reads her, or her as his tail reads her.
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum HerField {
        Door([String; 2]),
        Tail(HerRow),
    }

    /// Her as his tail reads her: her position in decimal, her body's half-sizes and her
    /// protection.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct HerRow {
        position: [u16; 2],
        body: [u16; 2],
        protection: u16,
    }

    impl Vector {
        /// Whether the vector names any of the doors' own fields.
        const fn names_doors(&self) -> bool {
            self.door.is_some()
                || self.before.is_some()
                || matches!(self.her, Some(HerField::Door(_)))
                || self.stage.is_some()
                || self.shaking.is_some()
                || self.after.is_some()
        }

        /// Whether the vector names any of Ridley's own fields.
        const fn names_ridley(&self) -> bool {
            self.ridley_placing
                || self.ridley.is_some()
                || self.baby.is_some()
                || self.passing.is_some()
                || self.content.is_some()
                || matches!(self.her, Some(HerField::Tail(_)))
                || self.ridley_after.is_some()
        }

        /// Whether it is one of his tail's vectors, in the part-4b form.
        fn names_tail(&self) -> bool {
            self.id.starts_with("T-")
        }

        /// Where she was, as a door reads her.
        fn door_her(&self) -> Option<&[String; 2]> {
            match &self.her {
                Some(HerField::Door(her)) => Some(her),
                Some(HerField::Tail(_)) | None => None,
            }
        }

        /// Her as his tail reads her; none where the vector has no her to reach.
        fn tail_her(&self) -> Option<Her> {
            match &self.her {
                Some(HerField::Tail(row)) => Some(Her {
                    position: point(row.position),
                    body: point(row.body),
                    protection: row.protection,
                }),
                Some(HerField::Door(_)) | None => None,
            }
        }

        /// Whether the vector names a field the doors and Ridley share.
        const fn names_shared(&self) -> bool {
            self.random.is_some() || self.beaten.is_some()
        }

        /// The random number it gives, 0 by default.
        fn random(&self) -> u16 {
            self.random.as_deref().map_or(0, hex)
        }

        /// Whether the vector names any of the pad's and the cover's own fields.
        const fn names_pad(&self) -> bool {
            self.arrival.is_some()
                || self.pad.is_some()
                || self.cover.is_some()
                || self.her_y.is_some()
                || self.shaft_after.is_some()
        }

        /// Whether the vector names any of the lights' own fields.
        const fn names_lights(&self) -> bool {
            self.lights.is_some() || self.lights_after.is_some()
        }
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    #[serde(deny_unknown_fields)]
    struct PadRow {
        wait: u16,
        position: [u16; 2],
        look: Option<TurnName>,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct ShaftAfter {
        her_y: u16,
        held: bool,
        pad: Option<PadRow>,
        cover: Option<[u16; 2]>,
        drawn: Option<DrawnRow>,
    }

    /// The figures a step of the pad and the cover reports, each absent for none.
    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct DrawnRow {
        pad: Option<PadFigureRow>,
        cover: Option<[u16; 2]>,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct PadFigureRow {
        look: TurnName,
        position: [u16; 2],
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct LightsRow {
        count: u16,
        fade_steps: Option<u8>,
        stage: Option<StageName>,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct LightsAfter {
        step: Option<u16>,
        platform: Option<TurnName>,
    }

    /// The pad's look and the platform's look alike.
    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum TurnName {
        First,
        Second,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "lowercase")]
    enum Source {
        Constructed,
    }

    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields)]
    struct RulesRow {
        reach: Option<u16>,
        poll: Option<u16>,
        hold: Option<u16>,
        strong_below: Option<u16>,
        small: Option<SizeRow>,
        strong: Option<SizeRow>,
        wait: Option<u16>,
        floor: Option<u16>,
        rhythm: Option<u16>,
        slam: Option<u16>,
        wake: Option<u16>,
        pace: Option<u8>,
        tail_reach: Option<u8>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct DoorRow {
        kind: KindName,
        facing: SideName,
        position: [String; 2],
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct PhaseRow {
        phase: PhaseName,
        step: Option<u8>,
        wait: u16,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    #[serde(deny_unknown_fields)]
    #[expect(
        clippy::struct_excessive_bools,
        reason = "the vector file's after holds shown, solid, sound and drawn_with_ridley as bools"
    )]
    struct After {
        phase: PhaseName,
        step: Option<u8>,
        wait: u16,
        figure: Option<FigureRow>,
        shown: bool,
        solid: bool,
        sound: bool,
        shake: Option<ShakeRow>,
        colours: Option<ColoursName>,
        palette: Option<PaletteName>,
        #[serde(default)]
        drawn_with_ridley: bool,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    #[serde(deny_unknown_fields)]
    struct FigureRow {
        look: LookName,
        facing: SideName,
    }

    /// A rule's shake: how strong and how long.
    #[derive(Deserialize, Clone, Copy)]
    #[serde(deny_unknown_fields)]
    struct SizeRow {
        pixels: u16,
        frames: u16,
    }

    /// A shake a door's run starts.
    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    #[serde(deny_unknown_fields)]
    struct ShakeRow {
        pixels: u16,
        frames: u16,
        #[serde(default)]
        layers: LayersName,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy, Default)]
    enum LayersName {
        Back,
        #[default]
        Whole,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum PaletteName {
        Doors,
        Own,
    }

    #[derive(Deserialize, Clone, Copy)]
    enum KindName {
        Door,
        GetawayWall,
        RidleyDoor,
        ShaftDoor,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum SideName {
        Left,
        Right,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum PhaseName {
        Closed,
        Closing,
        Lingering,
        Locked,
        Open,
        Opening,
        Placed,
        Slammed,
        Slamming,
        Standing,
        Starting,
        Unseen,
        Wall,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum LookName {
        Ajar1,
        Ajar2,
        Ajar3,
        Closed,
        Open,
        Side,
    }

    #[derive(Deserialize, Default, Clone, Copy)]
    enum StageName {
        #[default]
        Calm,
        Escape,
        Getaway,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum ColoursName {
        Calm,
        Escape,
    }

    fn vectors() -> Vec<Vector> {
        toml::from_str::<VectorFile>(VECTORS)
            .expect("the vector file reads")
            .vector
    }

    /// The vectors write positions and random numbers in hex.
    const HEX: u32 = 16;

    fn hex(text: &str) -> u16 {
        u16::from_str_radix(text, HEX).unwrap_or_else(|_| panic!("{text} is not 16-bit hex"))
    }

    fn position([x, y]: &[String; 2]) -> Pair<Px> {
        Pair {
            x: Wrapping(hex(x)),
            y: Wrapping(hex(y)),
        }
    }

    fn wait(frames: u16) -> Wait {
        Wait::new(frames).expect("a wait is at least 1 frame")
    }

    const fn side(name: SideName) -> Side {
        match name {
            SideName::Left => Side::Left,
            SideName::Right => Side::Right,
        }
    }

    const fn side_name(side: Side) -> SideName {
        match side {
            Side::Left => SideName::Left,
            Side::Right => SideName::Right,
        }
    }

    const fn stage(name: StageName) -> Stage {
        match name {
            StageName::Calm => Stage::Calm,
            StageName::Escape => Stage::Escape,
            StageName::Getaway => Stage::Getaway,
        }
    }

    /// The quarters of an opening or a closing, in their order.
    const QUARTERS: [Quarter; 4] = [
        Quarter::First,
        Quarter::Second,
        Quarter::Third,
        Quarter::Fourth,
    ];

    /// The quarter a vector's step, 1 to 4, names.
    fn quarter(n: Option<u8>) -> Quarter {
        let n = n.expect("an opening or a closing names its step");
        QUARTERS
            .into_iter()
            .find(|&quarter| step(quarter) == n)
            .unwrap_or_else(|| panic!("step {n} is not 1 to 4"))
    }

    const fn step(quarter: Quarter) -> u8 {
        match quarter {
            Quarter::First => 1,
            Quarter::Second => 2,
            Quarter::Third => 3,
            Quarter::Fourth => 4,
        }
    }

    /// Builds the phase a vector's `before` names. `Placed`, `Slammed`, `Standing`, `Unseen` and
    /// `Wall` carry no wait: the spec writes theirs as 1, and the vector must too.
    fn phase(row: &PhaseRow) -> Phase {
        let carries_step = matches!(
            row.phase,
            PhaseName::Closing | PhaseName::Opening | PhaseName::Slamming
        );
        assert_eq!(
            carries_step,
            row.step.is_some(),
            "a step for Closing, Opening and Slamming only"
        );
        let carries_wait = !matches!(
            row.phase,
            PhaseName::Placed
                | PhaseName::Slammed
                | PhaseName::Standing
                | PhaseName::Unseen
                | PhaseName::Wall
        );
        assert!(
            carries_wait || row.wait == 1,
            "the phases without a wait wait 1"
        );
        let wait = wait(row.wait);
        match row.phase {
            PhaseName::Closed => Phase::Closed { wait },
            PhaseName::Closing => Phase::Closing {
                quarter: quarter(row.step),
                wait,
            },
            PhaseName::Lingering => Phase::Lingering { wait },
            PhaseName::Locked => Phase::Locked { wait },
            PhaseName::Open => Phase::Open { wait },
            PhaseName::Opening => Phase::Opening {
                quarter: quarter(row.step),
                wait,
            },
            PhaseName::Placed => Phase::Placed,
            PhaseName::Slammed => Phase::Slammed,
            PhaseName::Slamming => Phase::Slamming {
                quarter: quarter(row.step),
                wait,
            },
            PhaseName::Standing => Phase::Standing,
            PhaseName::Starting => Phase::Starting { wait },
            PhaseName::Unseen => Phase::Unseen,
            PhaseName::Wall => Phase::Wall,
        }
    }

    /// Reads a phase back as a vector names it: its name, its step and its wait, 1 for the phases
    /// that carry none.
    fn phase_row(phase: Phase) -> (PhaseName, Option<u8>, u16) {
        match phase {
            Phase::Closed { wait } => (PhaseName::Closed, None, wait.frames()),
            Phase::Closing { quarter, wait } => {
                (PhaseName::Closing, Some(step(quarter)), wait.frames())
            }
            Phase::Lingering { wait } => (PhaseName::Lingering, None, wait.frames()),
            Phase::Locked { wait } => (PhaseName::Locked, None, wait.frames()),
            Phase::Open { wait } => (PhaseName::Open, None, wait.frames()),
            Phase::Opening { quarter, wait } => {
                (PhaseName::Opening, Some(step(quarter)), wait.frames())
            }
            Phase::Placed => (PhaseName::Placed, None, 1),
            Phase::Slammed => (PhaseName::Slammed, None, 1),
            Phase::Slamming { quarter, wait } => {
                (PhaseName::Slamming, Some(step(quarter)), wait.frames())
            }
            Phase::Standing => (PhaseName::Standing, None, 1),
            Phase::Starting { wait } => (PhaseName::Starting, None, wait.frames()),
            Phase::Unseen => (PhaseName::Unseen, None, 1),
            Phase::Wall => (PhaseName::Wall, None, 1),
        }
    }

    const fn look_name(look: FigureLook) -> LookName {
        match look {
            FigureLook::Ajar1 => LookName::Ajar1,
            FigureLook::Ajar2 => LookName::Ajar2,
            FigureLook::Ajar3 => LookName::Ajar3,
            FigureLook::Closed => LookName::Closed,
            FigureLook::Open => LookName::Open,
            FigureLook::Side => LookName::Side,
        }
    }

    const fn figure_row(figure: Figure) -> FigureRow {
        FigureRow {
            look: look_name(figure.look),
            facing: side_name(figure.facing),
        }
    }

    const fn shake(row: SizeRow) -> Shake {
        Shake {
            strength: row.pixels,
            length: row.frames,
        }
    }

    const fn shake_row(start: ShakeStart) -> ShakeRow {
        ShakeRow {
            pixels: start.shake.strength,
            frames: start.shake.length,
            layers: match start.layers {
                ShakeLayers::Back => LayersName::Back,
                ShakeLayers::Whole => LayersName::Whole,
            },
        }
    }

    const fn colours_name(colours: ColourSet) -> ColoursName {
        match colours {
            ColourSet::Calm => ColoursName::Calm,
            ColourSet::Escape => ColoursName::Escape,
        }
    }

    const fn palette_name(palette: DoorPalette) -> PaletteName {
        match palette {
            DoorPalette::Doors => PaletteName::Doors,
            DoorPalette::Own => PaletteName::Own,
        }
    }

    fn rules(row: &RulesRow) -> Rules {
        let reference = Rules::default();
        Rules {
            reach: row.reach.unwrap_or(reference.reach),
            poll: row.poll.map_or(reference.poll, wait),
            hold: row.hold.map_or(reference.hold, wait),
            strong_below: row.strong_below.unwrap_or(reference.strong_below),
            small: row.small.map_or(reference.small, shake),
            strong: row.strong.map_or(reference.strong, shake),
            pad_wait: row.wait.map_or(reference.pad_wait, pad_wait),
            floor: row.floor.map_or(reference.floor, Wrapping),
            rhythm: row.rhythm.map_or(reference.rhythm, |frames| {
                NonZeroU16::new(frames).expect("a rhythm is at least 1 frame")
            }),
            slam: row.slam.map_or(reference.slam, wait),
            wake: row.wake.map_or(reference.wake, |frames| {
                WakeWait::new(frames).expect("a wake is 1 to 32,768 frames")
            }),
            pace: row.pace.map_or(reference.pace, |runs| {
                NonZeroU8::new(runs).expect("a pace is 1 to 255 runs")
            }),
            tail_reach: row.tail_reach.unwrap_or(reference.tail_reach),
        }
    }

    fn door(row: &DoorRow, phase: Phase) -> CeresDoor {
        CeresDoor {
            kind: match row.kind {
                KindName::Door => DoorKind::Door,
                KindName::GetawayWall => DoorKind::GetawayWall,
                KindName::RidleyDoor => DoorKind::RidleyDoor,
                KindName::ShaftDoor => DoorKind::ShaftDoor,
            },
            facing: side(row.facing),
            position: position(&row.position),
            phase,
        }
    }

    /// The door as a vector's `after` reads it, with what its run answered; a placing adds its
    /// colours.
    fn after(door: CeresDoor, sound: bool, shake: Option<ShakeStart>) -> After {
        let (phase, step, wait) = phase_row(door.phase);
        After {
            phase,
            step,
            wait,
            figure: door.figure().map(figure_row),
            shown: door.phase.shown(),
            solid: door.phase.solid(),
            sound,
            shake: shake.map(shake_row),
            colours: None,
            palette: None,
            drawn_with_ridley: door.phase.drawn_with_ridley(),
        }
    }

    fn door_row(vector: &Vector) -> &DoorRow {
        vector
            .door
            .as_ref()
            .expect("a doors' vector names its door")
    }

    fn door_stage(vector: &Vector) -> Stage {
        stage(vector.stage.unwrap_or_default())
    }

    /// `RoomStart`, `TheDoorsColours`: the room places the door; it reads nothing of her, of the
    /// random number or of Ridley.
    fn placed(vector: &Vector) -> After {
        assert!(vector.before.is_none() && vector.her.is_none() && !vector.names_shared());
        let model = door(door_row(vector), Phase::Placed);
        let placement =
            CeresDoor::place(model.kind, model.facing, model.position, door_stage(vector));
        After {
            colours: Some(colours_name(placement.colours.set)),
            palette: Some(palette_name(placement.colours.palette)),
            ..after(placement.door, false, None)
        }
    }

    /// One run of the door from its `before`, reading what the vector gives.
    fn run(vector: &Vector) -> After {
        let before = vector.before.as_ref().expect("a run starts from a before");
        let her = vector.door_her().expect("a run reads where she was");
        let inputs = Inputs {
            her: position(her),
            stage: door_stage(vector),
            random: vector.random(),
            shake_frames_left: vector.shaking.unwrap_or(0),
            ridley_beaten: vector.beaten.unwrap_or(false),
        };
        let answer = door(door_row(vector), phase(before)).run(&inputs, &rules(&vector.rules));
        after(answer.door, answer.sound.is_some(), answer.shake)
    }

    /// The doors' vector's check: what the model answers against its `after`. A placing's
    /// palette is the doors' where the vector doesn't name one.
    fn check_door(vector: &Vector, expected: &After) -> Option<String> {
        assert!(!vector.names_pad() && !vector.names_lights() && !vector.names_ridley());
        let (actual, expected) = if vector.placing {
            let expected = After {
                palette: Some(expected.palette.unwrap_or(PaletteName::Doors)),
                ..*expected
            };
            (placed(vector), expected)
        } else {
            (run(vector), *expected)
        };
        (actual != expected)
            .then(|| format!("{}: expected {expected:?}, got {actual:?}", vector.id))
    }

    // ------------------------------------------------------------ the pad and the cover

    /// Part 2 writes its positions in decimal.
    const fn point([x, y]: [u16; 2]) -> Pair<Px> {
        Pair {
            x: Wrapping(x),
            y: Wrapping(y),
        }
    }

    const fn point_row(point: Pair<Px>) -> [u16; 2] {
        [point.x.0, point.y.0]
    }

    fn pad_wait(frames: u16) -> PadWait {
        PadWait::new(frames).expect("a pad's wait is 0 to 32,768")
    }

    const fn look(name: TurnName) -> AlternatingLook {
        match name {
            TurnName::First => AlternatingLook::First,
            TurnName::Second => AlternatingLook::Second,
        }
    }

    const fn look_turn_name(look: AlternatingLook) -> TurnName {
        match look {
            AlternatingLook::First => TurnName::First,
            AlternatingLook::Second => TurnName::Second,
        }
    }

    fn pad(row: PadRow) -> Pad {
        Pad {
            wait: pad_wait(row.wait),
            position: point(row.position),
            look: row.look.map(look),
        }
    }

    fn pad_row(pad: Pad) -> PadRow {
        PadRow {
            wait: pad.wait.frames(),
            position: point_row(pad.position),
            look: pad.look.map(look_turn_name),
        }
    }

    const fn pad_figure_row(figure: PadFigure) -> PadFigureRow {
        PadFigureRow {
            look: look_turn_name(figure.look),
            position: point_row(figure.position),
        }
    }

    /// The pad and the cover a vector's `pad` and `cover` build.
    fn pad_and_cover(vector: &Vector) -> PadAndCover {
        PadAndCover {
            pad: vector.pad.map(pad),
            cover: vector.cover.map(|position| Cover {
                position: point(position),
            }),
        }
    }

    /// The step a pad's and cover's vector names: the placing, or one run of the pad and then
    /// the cover.
    fn pad_step(vector: &Vector) -> PadAndCoverStep {
        let rules = rules(&vector.rules);
        if vector.placing {
            assert!(vector.pad.is_none() && vector.cover.is_none() && vector.her_y.is_none());
            let arrival = vector.arrival.expect("a placing names the arrival");
            PadAndCover::arrival(point(arrival), &rules)
        } else {
            assert!(vector.arrival.is_none());
            let her_y = vector.her_y.expect("a run reads her height");
            pad_and_cover(vector).run(Wrapping(her_y), &rules)
        }
    }

    /// The pad's and the cover's vector's check: its step against its `shaft_after`, the
    /// figures drawn compared where it names them.
    fn check_pad(vector: &Vector, expected: &ShaftAfter) -> Option<String> {
        assert!(
            !vector.names_doors()
                && !vector.names_lights()
                && !vector.names_ridley()
                && !vector.names_shared()
        );
        let step = pad_step(vector);
        let state = step.pad_and_cover;
        let actual = ShaftAfter {
            her_y: step.her_y.0,
            held: state.holds_her(),
            pad: state.pad.map(pad_row),
            cover: state.cover.map(|cover| point_row(cover.position)),
            drawn: expected.drawn.as_ref().map(|_| DrawnRow {
                pad: step.figures.pad.map(pad_figure_row),
                cover: step.figures.cover.map(point_row),
            }),
        };
        (actual != *expected)
            .then(|| format!("{}: expected {expected:?}, got {actual:?}", vector.id))
    }

    // ------------------------------------------------------------ the lights

    /// What a vector's `lights` give a step of the lights.
    fn lights_inputs(row: &LightsRow) -> LightsInputs {
        LightsInputs {
            finished_frames: row.count,
            fade_steps_taken: FadeSteps::new(row.fade_steps.unwrap_or(0))
                .expect("a fade takes 0 to 14 steps"),
            stage: stage(row.stage.unwrap_or_default()),
        }
    }

    /// The lights' vector's check: one step of the lights against its `lights_after`.
    fn check_lights(vector: &Vector, expected: &LightsAfter) -> Option<String> {
        assert!(
            !vector.names_doors()
                && !vector.names_pad()
                && !vector.names_ridley()
                && !vector.names_shared()
                && !vector.placing
        );
        let row = vector
            .lights
            .as_ref()
            .expect("a lights vector names its lights");
        let lights = Lights::step(lights_inputs(row), &rules(&vector.rules));
        let actual = LightsAfter {
            step: lights.beacon.map(BeaconStep::place),
            platform: lights.platform.map(look_turn_name),
        };
        (actual != *expected)
            .then(|| format!("{}: expected {expected:?}, got {actual:?}", vector.id))
    }

    // ------------------------------------------------------------ Ridley

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct RidleyRow {
        phase: RidleyPhaseName,
        wait: Option<u16>,
        steps: Option<u16>,
        sets: Option<u16>,
        runs: Option<u8>,
        figure: Option<u8>,
        figure_runs: Option<u8>,
        look: Option<RidleyLookName>,
        position: [u16; 2],
        curl: Option<u8>,
    }

    impl RidleyRow {
        /// What its phase carries, as the vector names it.
        const fn cells(&self) -> PhaseCells {
            PhaseCells {
                wait: self.wait,
                steps: self.steps,
                sets: self.sets,
                runs: self.runs,
                figure: self.figure,
                figure_runs: self.figure_runs,
            }
        }
    }

    /// What a phase of Ridley carries, each absent where the phase carries none.
    #[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
    struct PhaseCells {
        wait: Option<u16>,
        steps: Option<u16>,
        sets: Option<u16>,
        runs: Option<u8>,
        figure: Option<u8>,
        figure_runs: Option<u8>,
    }

    /// The invented content of a part-4a vector: the eye's steps, each the number of the set it
    /// names, and how many sets the body's fade has.
    #[derive(Deserialize, Clone)]
    #[serde(deny_unknown_fields)]
    struct ContentRow {
        eye_steps: Vec<u16>,
        body_sets: u16,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct BabyRow {
        routine: RoutineName,
        step: Option<usize>,
        runs: Option<u8>,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct BabyAfter {
        routine: RoutineName,
        step: Option<usize>,
        runs: Option<u8>,
        look: Option<BabyLookName>,
        position: Option<[u16; 2]>,
    }

    /// Ridley as a placing or a run leaves him, and what it hands on; each field absent where
    /// the vector's form leaves it out.
    #[derive(Deserialize, Debug, PartialEq, Eq, Default)]
    #[serde(deny_unknown_fields)]
    struct RidleyAfter {
        present: Option<bool>,
        phase: Option<RidleyPhaseName>,
        wait: Option<u16>,
        steps: Option<u16>,
        sets: Option<u16>,
        runs: Option<u8>,
        figure: Option<u8>,
        figure_runs: Option<u8>,
        look: Option<RidleyLookName>,
        position: Option<[u16; 2]>,
        baby: Option<BabyAfter>,
        shown: Option<bool>,
        solid: Option<bool>,
        boss: Option<bool>,
        music_stops: Option<bool>,
        shake_stops: Option<bool>,
        stage_calm: Option<bool>,
        room_colours: Option<bool>,
        cry: Option<bool>,
        colours: Option<BabyColoursName>,
        tangible: Option<bool>,
        moving: Option<bool>,
        roaring: Option<bool>,
        music: Option<bool>,
        roar: Option<bool>,
        eye_colours: Option<u16>,
        body_colours: Option<u16>,
        curl: Option<u8>,
        tail: Option<[u16; 2]>,
        hit: Option<HitRow>,
    }

    /// The frame's hit of his tail: the side it pushes her and the push's frames.
    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct HitRow {
        side: SideName,
        push: u8,
    }

    const fn hit_row(hit: Hit) -> HitRow {
        HitRow {
            side: side_name(hit.side),
            push: hit.push.frames(),
        }
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum RidleyPhaseName {
        Awake,
        BodyFading,
        EyeFading,
        Flying,
        Holding,
        ReadyToRoar,
        Roaring,
        Waiting,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum RidleyLookName {
        Open,
        Rest,
        Wide,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum RoutineName {
        New,
        Fidget,
        Glow,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum BabyLookName {
        First,
        Second,
        Third,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum BabyColoursName {
        Glow1,
        Glow2,
        Glow3,
        Rest,
    }

    /// The roar's first four figures, in their order.
    const TIMED_ROAR_FIGURES: [TimedRoarFigure; 4] = [
        TimedRoarFigure::First,
        TimedRoarFigure::Second,
        TimedRoarFigure::Third,
        TimedRoarFigure::Fourth,
    ];

    /// The roar's fifth figure, held, with no runs.
    const FIFTH_ROAR_FIGURE: u8 = 5;

    const fn roar_figure_number(figure: TimedRoarFigure) -> u8 {
        match figure {
            TimedRoarFigure::First => 1,
            TimedRoarFigure::Second => 2,
            TimedRoarFigure::Third => 3,
            TimedRoarFigure::Fourth => 4,
        }
    }

    /// The roar's figure a vector's `figure` and `figure_runs` name.
    fn roar_figure(number: Option<u8>, runs: Option<u8>) -> RoarFigure {
        let number = number.expect("a roar names its figure");
        if number == FIFTH_ROAR_FIGURE {
            return RoarFigure::Fifth;
        }
        let figure = TIMED_ROAR_FIGURES
            .into_iter()
            .find(|&figure| roar_figure_number(figure) == number)
            .unwrap_or_else(|| panic!("figure {number} is not 1 to 5"));
        RoarFigure::Timed(
            ShownFigure::new(figure, runs.expect("figures 1 to 4 name their runs"))
                .expect("runs are 1 to the figure's length"),
        )
    }

    fn roar_figure_cells(figure: RoarFigure) -> (Option<u8>, Option<u8>) {
        match figure {
            RoarFigure::Timed(shown) => {
                (Some(roar_figure_number(shown.figure())), Some(shown.runs()))
            }
            RoarFigure::Fifth => (Some(FIFTH_ROAR_FIGURE), None),
        }
    }

    /// A wait of `ReadyToRoar` or `Roaring`, as the vector names it.
    fn runs_left(wait: Option<u16>) -> u8 {
        wait.and_then(|runs| u8::try_from(runs).ok())
            .expect("his wait is named, in runs, 0 to 255")
    }

    /// Builds the phase a vector's `ridley` names; reading it back must give the same cells, so
    /// each phase names only what it carries.
    fn ridley_phase(row: &RidleyRow) -> RidleyPhase {
        let cells = row.cells();
        let phase = match row.phase {
            RidleyPhaseName::Awake => RidleyPhase::Awake,
            RidleyPhaseName::BodyFading => RidleyPhase::BodyFading {
                sets: cells.sets.expect("a body's fade names its sets"),
                runs: cells.runs.expect("a body's fade names its runs"),
            },
            RidleyPhaseName::EyeFading => RidleyPhase::EyeFading {
                steps: cells
                    .steps
                    .and_then(NonZeroU16::new)
                    .expect("an eye's fade names its steps, 1 up"),
            },
            RidleyPhaseName::Flying => RidleyPhase::Flying,
            RidleyPhaseName::Holding => RidleyPhase::Holding,
            RidleyPhaseName::ReadyToRoar => RidleyPhase::ReadyToRoar {
                wait: ReadyWait::new(runs_left(cells.wait)).expect("ready's wait is 0 to 4"),
            },
            RidleyPhaseName::Roaring => RidleyPhase::Roaring {
                wait: RoarWait::new(runs_left(cells.wait)).expect("the roar's wait is 0 to 252"),
                figure: roar_figure(cells.figure, cells.figure_runs),
            },
            RidleyPhaseName::Waiting => RidleyPhase::Waiting {
                wait: cells
                    .wait
                    .and_then(RidleyWait::new)
                    .expect("his wait is 0 to 32,767"),
            },
        };
        assert_eq!(
            ridley_phase_row(phase),
            (row.phase, cells),
            "a phase names only what it carries"
        );
        phase
    }

    fn ridley_phase_row(phase: RidleyPhase) -> (RidleyPhaseName, PhaseCells) {
        let none = PhaseCells::default();
        match phase {
            RidleyPhase::Awake => (RidleyPhaseName::Awake, none),
            RidleyPhase::BodyFading { sets, runs } => (
                RidleyPhaseName::BodyFading,
                PhaseCells {
                    sets: Some(sets),
                    runs: Some(runs),
                    ..none
                },
            ),
            RidleyPhase::EyeFading { steps } => (
                RidleyPhaseName::EyeFading,
                PhaseCells {
                    steps: Some(steps.get()),
                    ..none
                },
            ),
            RidleyPhase::Flying => (RidleyPhaseName::Flying, none),
            RidleyPhase::Holding => (RidleyPhaseName::Holding, none),
            RidleyPhase::ReadyToRoar { wait } => (
                RidleyPhaseName::ReadyToRoar,
                PhaseCells {
                    wait: Some(wait.runs().into()),
                    ..none
                },
            ),
            RidleyPhase::Roaring { wait, figure } => {
                let (figure, figure_runs) = roar_figure_cells(figure);
                (
                    RidleyPhaseName::Roaring,
                    PhaseCells {
                        wait: Some(wait.runs().into()),
                        figure,
                        figure_runs,
                        ..none
                    },
                )
            }
            RidleyPhase::Waiting { wait } => (
                RidleyPhaseName::Waiting,
                PhaseCells {
                    wait: Some(wait.runs()),
                    ..none
                },
            ),
        }
    }

    const fn ridley_look(name: RidleyLookName) -> RidleyLook {
        match name {
            RidleyLookName::Open => RidleyLook::Open,
            RidleyLookName::Rest => RidleyLook::Rest,
            RidleyLookName::Wide => RidleyLook::Wide,
        }
    }

    const fn ridley_look_name(look: RidleyLook) -> RidleyLookName {
        match look {
            RidleyLook::Open => RidleyLookName::Open,
            RidleyLook::Rest => RidleyLookName::Rest,
            RidleyLook::Wide => RidleyLookName::Wide,
        }
    }

    // ------------------------------------------------------------ Ridley's content

    /// The colour a vector's invented content gives the set of this number: every colour of
    /// the set is it, so the set shown reads back as its number.
    fn numbered_colour(number: u16) -> Colour {
        Colour::new(number).expect("a set's number is a colour")
    }

    /// The content a vector's `content` names: each eye step holds the set it names, and the
    /// body's sets are numbered from 1.
    fn ridley_content(row: &ContentRow) -> RidleyContent {
        let eye_steps = row
            .eye_steps
            .iter()
            .map(|&set| EyeColours(core::array::from_fn(|_| numbered_colour(set))))
            .collect();
        let body_sets = (1..=row.body_sets)
            .map(|set| BodyColours(core::array::from_fn(|_| numbered_colour(set))))
            .collect();
        RidleyContent {
            eye_steps: FadeSets::new(eye_steps).expect("the eye has at least one step"),
            body_sets: FadeSets::new(body_sets).expect("the body has at least one set"),
        }
    }

    /// The content part 3's vectors run with, which name none: a stand-in of one eye step and
    /// one body set, which part 3's phases never read.
    fn stand_in_content() -> ContentRow {
        ContentRow {
            eye_steps: vec![0],
            body_sets: 1,
        }
    }

    /// The baby's routine a vector's `baby` names.
    fn routine(row: &BabyRow) -> BabyRoutine {
        let figure = match row.routine {
            RoutineName::New => {
                assert!(
                    row.step.is_none() && row.runs.is_none(),
                    "a new baby has none"
                );
                return BabyRoutine::New;
            }
            RoutineName::Fidget => BabyFigure::Fidget(FidgetFigure::from_usize(
                row.step.expect("a fidget names its step"),
            )),
            RoutineName::Glow => BabyFigure::Glow(GlowFigure::from_usize(
                row.step.expect("a glow names its step"),
            )),
        };
        let runs = row.runs.expect("a figure names its runs");
        BabyRoutine::Showing(
            ShownFigure::new(figure, runs).expect("runs are 1 to the figure's length"),
        )
    }

    const fn baby_look_name(look: BabyLook) -> BabyLookName {
        match look {
            BabyLook::First => BabyLookName::First,
            BabyLook::Second => BabyLookName::Second,
            BabyLook::Third => BabyLookName::Third,
        }
    }

    const fn baby_colours_name(colours: BabyColours) -> BabyColoursName {
        match colours {
            BabyColours::Glow1 => BabyColoursName::Glow1,
            BabyColours::Glow2 => BabyColoursName::Glow2,
            BabyColours::Glow3 => BabyColoursName::Glow3,
            BabyColours::Rest => BabyColoursName::Rest,
        }
    }

    /// The baby as a vector's `ridley_after` reads it.
    fn baby_after(baby: Baby) -> BabyAfter {
        let (routine, step, runs) = match baby.routine {
            BabyRoutine::New => (RoutineName::New, None, None),
            BabyRoutine::Showing(shown) => {
                let (name, step) = match shown.figure() {
                    BabyFigure::Fidget(figure) => (RoutineName::Fidget, figure.into_usize()),
                    BabyFigure::Glow(figure) => (RoutineName::Glow, figure.into_usize()),
                };
                (name, Some(step), Some(shown.runs()))
            }
        };
        BabyAfter {
            routine,
            step,
            runs,
            look: baby.routine.look().map(baby_look_name),
            position: baby.position.map(point_row),
        }
    }

    /// Ridley's state as a vector's `ridley_after` reads it, and that he is shown and passable.
    fn ridley_after(ridley: CeresRidley) -> RidleyAfter {
        let (phase, cells) = ridley_phase_row(ridley.phase);
        RidleyAfter {
            phase: Some(phase),
            wait: cells.wait,
            steps: cells.steps,
            sets: cells.sets,
            runs: cells.runs,
            figure: cells.figure,
            figure_runs: cells.figure_runs,
            look: ridley.look.map(ridley_look_name),
            position: Some(point_row(ridley.position)),
            baby: Some(baby_after(ridley.baby)),
            shown: Some(CeresRidley::SHOWN),
            solid: Some(CeresRidley::SOLID),
            ..RidleyAfter::default()
        }
    }

    /// `RidleyPlaced`: the room places him, or doesn't once he is beaten; either way the vector
    /// reads each of the placing's outputs.
    fn ridley_placed(vector: &Vector) -> RidleyAfter {
        assert!(
            vector.ridley.is_none()
                && vector.baby.is_none()
                && vector.passing.is_none()
                && vector.content.is_none()
        );
        assert!(
            vector.random.is_none(),
            "the placing reads no random number"
        );
        let Some(placing) = CeresRidley::place(vector.beaten.unwrap_or(false)) else {
            return RidleyAfter {
                present: Some(false),
                boss: Some(false),
                music_stops: Some(false),
                shake_stops: Some(false),
                stage_calm: Some(false),
                room_colours: Some(false),
                ..RidleyAfter::default()
            };
        };
        RidleyAfter {
            present: Some(true),
            boss: Some(placing.boss == Boss::CeresRidley),
            music_stops: Some(true),
            shake_stops: Some(true),
            stage_calm: Some(placing.stage == Stage::Calm),
            room_colours: Some(true),
            curl: vector.names_tail().then_some(placing.ridley.curl.get()),
            ..ridley_after(placing.ridley)
        }
    }

    /// The curl a vector's `ridley` names: a part-4b vector names it; the others name none and
    /// run with the curl he is placed with, which their checks don't read.
    fn curl(vector: &Vector, row: &RidleyRow) -> TailCurl {
        assert_eq!(
            vector.names_tail(),
            row.curl.is_some(),
            "{}: the part-4b vectors alone name the curl",
            vector.id
        );
        row.curl.map_or(TailCurl::PLACED, |curl| {
            TailCurl::new(curl).expect("a curl is 8 to 23")
        })
    }

    /// One run of him and the baby from the vector's `ridley` and `baby`, with its content. A
    /// part-4a vector, which names its content, reads his waking's outputs too.
    fn ridley_run(vector: &Vector) -> RidleyAfter {
        assert!(
            vector.beaten.is_none(),
            "a run reads nothing of his being beaten"
        );
        let row = vector.ridley.as_ref().expect("a run starts from his state");
        let baby = vector
            .baby
            .as_ref()
            .expect("a run starts from the baby's state");
        let ridley = CeresRidley {
            phase: ridley_phase(row),
            look: row.look.map(ridley_look),
            position: point(row.position),
            baby: Baby {
                routine: routine(baby),
                position: None,
            },
            curl: curl(vector, row),
        };
        let content = vector.content.clone().unwrap_or_else(stand_in_content);
        let run = ridley.run(
            vector.passing.unwrap_or(false),
            vector.random(),
            vector.tail_her().as_ref(),
            &ridley_content(&content),
            &rules(&vector.rules),
        );
        let part_3 = RidleyAfter {
            cry: Some(run.cry.is_some()),
            colours: run.colours.map(baby_colours_name),
            ..ridley_after(run.ridley)
        };
        if vector.content.is_none() {
            return part_3;
        }
        let phase = run.ridley.phase;
        let part_4a = RidleyAfter {
            tangible: Some(phase.tangible()),
            moving: Some(phase.moving()),
            roaring: Some(phase.roaring()),
            music: Some(run.music.is_some()),
            roar: Some(run.roar.is_some()),
            eye_colours: run
                .eye_colours
                .map(|EyeColours([colour, ..])| colour.bits()),
            body_colours: run
                .body_colours
                .map(|BodyColours([colour, ..])| colour.bits()),
            ..part_3
        };
        if !vector.names_tail() {
            assert!(vector.tail_her().is_none(), "the tail's her is part 4b's");
            return part_4a;
        }
        RidleyAfter {
            curl: Some(run.ridley.curl.get()),
            tail: run.ridley.drawing().tail.map(point_row),
            hit: run.hit.map(hit_row),
            ..part_4a
        }
    }

    /// Ridley's vector's check: his placing or his run against its `ridley_after`.
    fn check_ridley(vector: &Vector, expected: &RidleyAfter) -> Option<String> {
        assert!(!vector.names_doors() && !vector.names_pad() && !vector.names_lights());
        assert!(!vector.placing, "Ridley's placing is ridley_placing");
        let actual = if vector.ridley_placing {
            ridley_placed(vector)
        } else {
            ridley_run(vector)
        };
        (actual != *expected)
            .then(|| format!("{}: expected {expected:?}, got {actual:?}", vector.id))
    }

    /// The vector's check, by the result it names; exactly one.
    fn check(vector: &Vector) -> Option<String> {
        match (
            &vector.after,
            &vector.shaft_after,
            &vector.lights_after,
            &vector.ridley_after,
        ) {
            (Some(after), None, None, None) => check_door(vector, after),
            (None, Some(after), None, None) => check_pad(vector, after),
            (None, None, Some(after), None) => check_lights(vector, after),
            (None, None, None, Some(after)) => check_ridley(vector, after),
            _ => panic!("{}: a vector names exactly one result", vector.id),
        }
    }

    fn entry(name: &str) -> BehaviourId {
        catalogue()
            .into_iter()
            .map(|(id, _)| id)
            .find(|id| format!("{id:?}") == name)
            .unwrap_or_else(|| panic!("{name} is not an entry of the spec"))
    }

    #[test]
    fn every_vector_passes() {
        let vectors = vectors();
        assert!(!vectors.is_empty());
        let failures: Vec<String> = vectors
            .iter()
            .filter_map(|vector| {
                let Source::Constructed = vector.source;
                vector.entries.iter().for_each(|name| {
                    entry(name);
                });
                check(vector)
            })
            .collect();
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn every_entry_checked_by_vectors_has_one() {
        let vectors = vectors();
        for (id, behaviour) in catalogue() {
            assert_eq!(behaviour.id, id);
            let exercised = vectors
                .iter()
                .any(|vector| vector.entries.iter().any(|name| entry(name) == id));
            let by_vectors = behaviour.checked == Checked::ByVectors;
            assert_eq!(exercised, by_vectors, "{id:?}");
        }
    }
}
