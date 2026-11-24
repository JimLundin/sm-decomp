//! Runs every vector of `scenarios/vectors/boss-ceres-ridley.toml` through the model, the doors'
//! and the shaft's, and checks the catalogue against the vectors. Every table refuses a key it
//! doesn't know, and every name must be one the spec gives.

#[cfg(test)]
mod vectors {
    use core::num::{NonZeroU16, Wrapping};

    use boss_ceres_ridley::{
        AlternatingLook, BeaconStep, BehaviourId, CeresDoor, ColourSet, Cover, DoorKind, FadeSteps,
        Figure, FigureLook, Inputs, Lights, LightsInputs, Pad, PadAndCover, PadAndCoverStep,
        PadFigure, PadWait, Phase, Quarter, Rules, Shake, Stage, Wait, catalogue,
    };
    use serde::Deserialize;
    use types::{Checked, Pair, Px, Side};

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/boss-ceres-ridley.toml");

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VectorFile {
        vector: Vec<Vector>,
    }

    /// A vector of either part: the doors' fields, or the shaft's (the pad and the cover, or the
    /// lights). Which part it is follows from the result it names.
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
        // The doors.
        door: Option<DoorRow>,
        before: Option<PhaseRow>,
        her: Option<[String; 2]>,
        stage: Option<StageName>,
        random: Option<String>,
        shaking: Option<u16>,
        after: Option<After>,
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

    impl Vector {
        /// Whether the vector names any of the doors' own fields.
        const fn names_doors(&self) -> bool {
            self.door.is_some()
                || self.before.is_some()
                || self.her.is_some()
                || self.stage.is_some()
                || self.random.is_some()
                || self.shaking.is_some()
                || self.after.is_some()
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
        small: Option<ShakeRow>,
        strong: Option<ShakeRow>,
        wait: Option<u16>,
        floor: Option<u16>,
        rhythm: Option<u16>,
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

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
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
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    #[serde(deny_unknown_fields)]
    struct FigureRow {
        look: LookName,
        facing: SideName,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    #[serde(deny_unknown_fields)]
    struct ShakeRow {
        pixels: u16,
        frames: u16,
    }

    #[derive(Deserialize, Clone, Copy)]
    enum KindName {
        Door,
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
        Open,
        Opening,
        Placed,
        Starting,
        Wall,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum LookName {
        Ajar1,
        Ajar2,
        Ajar3,
        Closed,
        Open,
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

    /// Builds the phase a vector's `before` names. `Placed` and `Wall` carry no wait: the spec
    /// writes theirs as 1, and the vector must too.
    fn phase(row: &PhaseRow) -> Phase {
        let carries_step = matches!(row.phase, PhaseName::Closing | PhaseName::Opening);
        assert_eq!(
            carries_step,
            row.step.is_some(),
            "a step for Closing and Opening only"
        );
        let carries_wait = !matches!(row.phase, PhaseName::Placed | PhaseName::Wall);
        assert!(carries_wait || row.wait == 1, "Placed and Wall wait 1");
        match row.phase {
            PhaseName::Closed => Phase::Closed {
                wait: wait(row.wait),
            },
            PhaseName::Closing => Phase::Closing {
                quarter: quarter(row.step),
                wait: wait(row.wait),
            },
            PhaseName::Open => Phase::Open {
                wait: wait(row.wait),
            },
            PhaseName::Opening => Phase::Opening {
                quarter: quarter(row.step),
                wait: wait(row.wait),
            },
            PhaseName::Placed => Phase::Placed,
            PhaseName::Starting => Phase::Starting {
                wait: wait(row.wait),
            },
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
            Phase::Open { wait } => (PhaseName::Open, None, wait.frames()),
            Phase::Opening { quarter, wait } => {
                (PhaseName::Opening, Some(step(quarter)), wait.frames())
            }
            Phase::Placed => (PhaseName::Placed, None, 1),
            Phase::Starting { wait } => (PhaseName::Starting, None, wait.frames()),
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
        }
    }

    const fn figure_row(figure: Figure) -> FigureRow {
        FigureRow {
            look: look_name(figure.look),
            facing: side_name(figure.facing),
        }
    }

    const fn shake(row: ShakeRow) -> Shake {
        Shake {
            strength: row.pixels,
            length: row.frames,
        }
    }

    const fn shake_row(shake: Shake) -> ShakeRow {
        ShakeRow {
            pixels: shake.strength,
            frames: shake.length,
        }
    }

    const fn colours_name(colours: ColourSet) -> ColoursName {
        match colours {
            ColourSet::Calm => ColoursName::Calm,
            ColourSet::Escape => ColoursName::Escape,
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
        }
    }

    fn door(row: &DoorRow, phase: Phase) -> CeresDoor {
        CeresDoor {
            kind: match row.kind {
                KindName::Door => DoorKind::Door,
                KindName::ShaftDoor => DoorKind::ShaftDoor,
            },
            facing: side(row.facing),
            position: position(&row.position),
            phase,
        }
    }

    /// The door as a vector's `after` reads it, with what the run or the placing answered.
    fn after(
        door: CeresDoor,
        sound: bool,
        shake: Option<Shake>,
        colours: Option<ColourSet>,
    ) -> After {
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
            colours: colours.map(colours_name),
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

    /// `RoomStart`, `TheDoorsColours`: the room places the door; it reads nothing of her.
    fn placed(vector: &Vector) -> After {
        assert!(vector.before.is_none() && vector.her.is_none() && vector.random.is_none());
        let model = door(door_row(vector), Phase::Placed);
        let placement =
            CeresDoor::place(model.kind, model.facing, model.position, door_stage(vector));
        after(placement.door, false, None, Some(placement.colours))
    }

    /// One run of the door from its `before`, reading what the vector gives.
    fn run(vector: &Vector) -> After {
        let before = vector.before.as_ref().expect("a run starts from a before");
        let her = vector.her.as_ref().expect("a run reads where she was");
        let inputs = Inputs {
            her: position(her),
            stage: door_stage(vector),
            random: vector.random.as_deref().map_or(0, hex),
            shake_frames_left: vector.shaking.unwrap_or(0),
        };
        let answer = door(door_row(vector), phase(before)).run(&inputs, &rules(&vector.rules));
        after(answer.door, answer.sound.is_some(), answer.shake, None)
    }

    /// The doors' vector's check: what the model answers against its `after`.
    fn check_door(vector: &Vector, expected: &After) -> Option<String> {
        assert!(!vector.names_pad() && !vector.names_lights());
        let actual = if vector.placing {
            placed(vector)
        } else {
            run(vector)
        };
        (actual != *expected)
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
        assert!(!vector.names_doors() && !vector.names_lights());
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
        assert!(!vector.names_doors() && !vector.names_pad() && !vector.placing);
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

    /// The vector's check, by the result it names; exactly one.
    fn check(vector: &Vector) -> Option<String> {
        match (&vector.after, &vector.shaft_after, &vector.lights_after) {
            (Some(after), None, None) => check_door(vector, after),
            (None, Some(after), None) => check_pad(vector, after),
            (None, None, Some(after)) => check_lights(vector, after),
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
