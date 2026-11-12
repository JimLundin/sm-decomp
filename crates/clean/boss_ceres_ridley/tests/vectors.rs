//! Runs every vector of `scenarios/vectors/boss-ceres-ridley.toml` through the model, and checks
//! the catalogue against the vectors. Every table refuses a key it doesn't know, and every name
//! must be one the spec gives.

#[cfg(test)]
mod vectors {
    use core::num::Wrapping;

    use boss_ceres_ridley::{
        BehaviourId, CeresDoor, ColourSet, DoorKind, Figure, FigureLook, Inputs, Phase, Quarter,
        Rules, Shake, Stage, Wait, catalogue,
    };
    use serde::Deserialize;
    use types::{Checked, Pair, Px, Side};

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/boss-ceres-ridley.toml");

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VectorFile {
        vector: Vec<Vector>,
    }

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
        door: DoorRow,
        before: Option<PhaseRow>,
        her: Option<[String; 2]>,
        #[serde(default)]
        stage: StageName,
        random: Option<String>,
        #[serde(default)]
        shaking: u16,
        after: After,
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

    /// `RoomStart`, `TheDoorsColours`: the room places the door; it reads nothing of her.
    fn placed(vector: &Vector) -> After {
        assert!(vector.before.is_none() && vector.her.is_none() && vector.random.is_none());
        let model = door(&vector.door, Phase::Placed);
        let placement = CeresDoor::place(
            model.kind,
            model.facing,
            model.position,
            stage(vector.stage),
        );
        after(placement.door, false, None, Some(placement.colours))
    }

    /// One run of the door from its `before`, reading what the vector gives.
    fn run(vector: &Vector) -> After {
        let before = vector.before.as_ref().expect("a run starts from a before");
        let her = vector.her.as_ref().expect("a run reads where she was");
        let inputs = Inputs {
            her: position(her),
            stage: stage(vector.stage),
            random: vector.random.as_deref().map_or(0, hex),
            shake_frames_left: vector.shaking,
        };
        let answer = door(&vector.door, phase(before)).run(&inputs, &rules(&vector.rules));
        after(answer.door, answer.sound.is_some(), answer.shake, None)
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
                let actual = if vector.placing {
                    placed(vector)
                } else {
                    run(vector)
                };
                (actual != vector.after)
                    .then(|| format!("{}: expected {:?}, got {actual:?}", vector.id, vector.after))
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
