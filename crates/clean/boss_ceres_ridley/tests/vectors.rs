//! Runs every vector of `scenarios/vectors/boss-ceres-ridley.toml` through the model, the doors',
//! the shaft's, Ridley's (his tail's and his flight's among them) and the steam's, and checks the catalogue
//! against the vectors. Every table refuses a key it doesn't know, and every name must be one
//! the spec gives.

#[cfg(test)]
mod support;

#[cfg(test)]
mod vectors {
    use super::support::shaft::{OverlayVector, ProjectionVector, ReachVector, RotationVector};
    use super::support::source::Source;
    use super::support::warning::{
        self as warning_vectors, EnglishCellRow, WarningEffectsRow, WarningLanguage, WarningRow,
    };
    use core::num;
    use core::num::Wrapping;

    use boss_ceres_ridley::{
        AlternatingLook, Attack, Baby, BabyColours, BabyFigure, BabyLook, BabyRoutine, BeaconStep,
        BehaviourId, BodyColours, Boss, Bound, CeresDoor, CeresRidley, CoastRuns, ColourSet, Cover,
        DoorKind, DoorPalette, EyeColours, FadeSets, FadeSteps, FidgetFigure, Fight, Figure,
        FigureLength, FigureLook, FigureSequence, FirstPuffWait, FlightPhase, GetawayContent,
        GlowFigure, Her, HoverRuns, Inputs, Lights, LightsInputs, Pad, PadAndCover,
        PadAndCoverStep, PadFigure, PadWait, Phase, Puff, PuffFigure, Quarter, ReadyWait,
        RidleyContent, RidleyLook, RidleyPhase, RidleyRun, RidleyWait, RoarWait, Rules, Shake,
        ShakeLayers, ShakeMotion, ShakeStart, ShownFigure, Stage, TailCurl, TimedRoarFigure,
        TimedTakeOffFigure, Vent, VentFacing, VentPhase, Wait, WakeWait, catalogue, run_vents,
    };
    use boss_ceres_ridley::{
        FightColours, FightFacing, FightFlash, FightParts, FightPictureInputs, FightWings,
        FlashHalf, RoomLayerOrder, Speed, TailIntent, WingBeat, WingCount,
    };
    use boss_ceres_ridley::{FightRibs, RibBeat, RibLook, RibWait};
    use boss_ceres_ridley::{
        FlightTail, LagCount, SwingLag, SwingPace, SwingWay, SwishRuns, TAIL_SEGMENTS, TailAngle,
        TailLength, TailMark, TailSegment,
    };
    use boss_ceres_ridley::{
        FlybyBabyBeat, FlybyBabyLook, FlybyBeats, FlybyContent, FlybyFrame, FlybyPath,
        FlybyProgress, FlybyTurn, FlybyWingBeat, FlybyZoom, GetawaySound, GetawayState,
        GetawayView, OutOfTheWay,
    };
    use enum_map::{Enum, enum_map};
    use serde::Deserialize;
    use types::quake::{QuakeDirection, QuakeDuration, QuakeKind, QuakeReach, QuakeRequest};
    use types::{Checked, Colour, GameFrames, Hit, Pair, Picture, Px, Side};

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VectorFile {
        vector: Vec<Vector>,
        shaft_overlay: Vec<OverlayVector>,
        shaft_rotation: Vec<RotationVector>,
        shaft_reach: Vec<ReachVector>,
        shaft_vent_projection: Vec<ProjectionVector>,
        fight_picture: Vec<FightPictureVector>,
        fight_ribs: Vec<FightRibsVector>,
    }

    impl VectorFile {
        fn check_tilting_room(&self) {
            assert_eq!(self.shaft_overlay.len(), 7);
            assert_eq!(self.shaft_rotation.len(), 6);
            assert_eq!(self.shaft_reach.len(), 10);
            assert_eq!(self.shaft_vent_projection.len(), 3);
            for vector in &self.shaft_overlay {
                vector.source.check(|| vector.check());
            }
            for vector in &self.shaft_rotation {
                vector.source.check(|| vector.check());
            }
            for vector in &self.shaft_reach {
                vector.source.check(|| vector.check());
            }
            for vector in &self.shaft_vent_projection {
                vector.source.check(|| vector.check());
            }
        }

        fn labelled_entries(&self) -> impl Iterator<Item = (&str, &[String])> {
            self.vector
                .iter()
                .map(|v| (v.id.as_str(), v.entries.as_slice()))
                .chain(
                    self.shaft_overlay
                        .iter()
                        .map(|v| (v.id.as_str(), v.entries.as_slice())),
                )
                .chain(
                    self.shaft_rotation
                        .iter()
                        .map(|v| (v.id.as_str(), v.entries.as_slice())),
                )
                .chain(
                    self.shaft_reach
                        .iter()
                        .map(|v| (v.id.as_str(), v.entries.as_slice())),
                )
                .chain(
                    self.shaft_vent_projection
                        .iter()
                        .map(|v| (v.id.as_str(), v.entries.as_slice())),
                )
                .chain(
                    self.fight_picture
                        .iter()
                        .map(|v| (v.id.as_str(), v.entries.as_slice())),
                )
                .chain(
                    self.fight_ribs
                        .iter()
                        .map(|v| (v.id.as_str(), v.entries.as_slice())),
                )
        }
    }

    /// A vector of any part: the doors' fields (the door of Ridley's room and the getaway's walls
    /// among them), the shaft's (the pad and the cover, or the lights), Ridley's, or the
    /// steam's. The fields select what to run and compare; `entries` names the rules checked.
    /// The id is only a label for reporting.
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
        energy: Option<u16>,
        hits_taken: Option<u16>,
        ridley_after: Option<RidleyAfter>,
        flight_tail: Option<FlightTailRow>,
        flight_tail_after: Option<FlightTailRow>,
        // The flyby.
        getaway: Option<GetawayRow>,
        finished: Option<u16>,
        picture: Option<PictureRow>,
        length: Option<usize>,
        getaway_after: Option<GetawayRow>,
        effects: Option<GetawayEffectsRow>,
        // The warning.
        warning: Option<WarningRow>,
        warning_after: Option<WarningRow>,
        warning_effects: Option<WarningEffectsRow>,
        ordinary: Option<bool>,
        language: Option<WarningLanguage>,
        fade: Option<bool>,
        passage: Option<bool>,
        art: Option<Vec<Vec<String>>>,
        text: Option<Vec<EnglishCellRow>>,
        // The pad and the cover.
        arrival: Option<[u16; 2]>,
        pad: Option<PadRow>,
        cover: Option<[u16; 2]>,
        her_y: Option<u16>,
        shaft_after: Option<ShaftAfter>,
        // The lights.
        lights: Option<LightsRow>,
        lights_after: Option<LightsAfter>,
        // The steam.
        #[serde(default)]
        steam_placing: bool,
        vent: Option<VentPlaceRow>,
        vents: Option<Vec<VentRow>>,
        camera: Option<[u16; 2]>,
        vents_after: Option<Vec<VentAfter>>,
        hit: Option<HitRow>,
    }

    /// A vector's `her`: where she was, as a door reads her, or her as a hitter, his tail or
    /// the steam, reads her.
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum HerField {
        Door([String; 2]),
        Hitter(HerRow),
    }

    /// Her as a hitter reads her: her position in decimal, her body's half-sizes and her
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

        /// Whether the vector names any of Ridley's fields, her as his tail reads her among them.
        const fn names_ridley(&self) -> bool {
            self.names_ridley_own() || matches!(self.her, Some(HerField::Hitter(_)))
        }

        /// Whether the vector names any of the fields Ridley alone reads.
        const fn names_ridley_own(&self) -> bool {
            self.ridley_placing
                || self.ridley.is_some()
                || self.baby.is_some()
                || self.passing.is_some()
                || self.content.is_some()
                || self.energy.is_some()
                || self.hits_taken.is_some()
                || self.ridley_after.is_some()
                || self.flight_tail.is_some()
                || self.flight_tail_after.is_some()
        }

        /// Whether the vector names any of the steam's own fields.
        const fn names_steam(&self) -> bool {
            self.steam_placing
                || self.vent.is_some()
                || self.vents.is_some()
                || self.camera.is_some()
                || self.vents_after.is_some()
                || self.hit.is_some()
        }

        /// The curl output distinguishes the resting-tail form from the waking form.
        fn names_tail(&self) -> bool {
            self.ridley_after
                .as_ref()
                .is_some_and(|after| after.curl.is_some())
        }

        /// Flight reads both energy and hits taken.
        const fn names_flight(&self) -> bool {
            self.energy.is_some() || self.hits_taken.is_some()
        }

        /// How the fight stands, as a flight vector names it; the other forms name none, and
        /// run with a stand-in their phases never read.
        fn fight(&self) -> Fight {
            assert_eq!(
                self.names_flight(),
                self.energy.is_some() && self.hits_taken.is_some(),
                "{}: the part-4c-1a vectors alone name her energy and his hits taken",
                self.id
            );
            Fight {
                her_energy: self.energy.unwrap_or(0),
                hits_taken: self.hits_taken.unwrap_or(0),
            }
        }

        /// Where she was, as a door reads her.
        fn door_her(&self) -> Option<&[String; 2]> {
            match &self.her {
                Some(HerField::Door(her)) => Some(her),
                Some(HerField::Hitter(_)) | None => None,
            }
        }

        /// Her as a hitter reads her; none where the vector has no her to reach.
        fn hitter_her(&self) -> Option<Her> {
            match &self.her {
                Some(HerField::Hitter(row)) => Some(Her {
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
        steam_rest: Option<u8>,
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

    #[derive(Deserialize, Default, Clone, Copy, Debug, PartialEq, Eq)]
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

    fn vector_file() -> VectorFile {
        toml::from_str(&vector_file::read("boss-ceres-ridley").unwrap())
            .unwrap_or_else(|error| panic!("the vector file must read: {}", error.message()))
    }

    fn vectors() -> Vec<Vector> {
        vector_file().vector
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
                num::NonZeroU16::new(frames).expect("a rhythm is at least 1 frame")
            }),
            slam: row.slam.map_or(reference.slam, wait),
            wake: row.wake.map_or(reference.wake, |frames| {
                WakeWait::new(frames).expect("a wake is 1 to 32,768 frames")
            }),
            pace: row.pace.map_or(reference.pace, |runs| {
                num::NonZeroU8::new(runs).expect("a pace is 1 to 255 runs")
            }),
            tail_reach: row.tail_reach.unwrap_or(reference.tail_reach),
            steam_rest: row.steam_rest.map_or(reference.steam_rest, |runs| {
                num::NonZeroU8::new(runs).expect("a rest is 1 to 255 runs")
            }),
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
        assert!(
            !vector.names_pad()
                && !vector.names_lights()
                && !vector.names_ridley()
                && !vector.names_steam()
        );
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
                && !vector.names_steam()
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
                && !vector.names_steam()
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
        takeoff_figure: Option<u8>,
        takeoff_runs: Option<u8>,
        fraction: Option<[u8; 2]>,
        speed: Option<[i16; 2]>,
        hover_runs: Option<u8>,
        coast_runs: Option<u8>,
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
                coast_runs: self.coast_runs,
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
        coast_runs: Option<u8>,
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
        colours: Option<RidleyColoursRow>,
        walls: Option<[SideName; 2]>,
        coast_runs: Option<u8>,
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
        takeoff_figure: Option<u8>,
        takeoff_runs: Option<u8>,
        fraction: Option<[u8; 2]>,
        speed: Option<[i16; 2]>,
        hover_runs: Option<u8>,
        held: Option<[u16; 2]>,
        whip: Option<bool>,
        bound: Option<BoundName>,
        shake: Option<FlightShakeRow>,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum BoundName {
        Bottom,
        Left,
        Right,
        Top,
    }

    const fn bound_name(bound: Bound) -> BoundName {
        match bound {
            Bound::Bottom => BoundName::Bottom,
            Bound::Left => BoundName::Left,
            Bound::Right => BoundName::Right,
            Bound::Top => BoundName::Top,
        }
    }

    /// The shake he starts in flight: the back layers, moving sideways, so strong for so long.
    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct FlightShakeRow {
        pixels: u16,
        frames: u16,
    }

    /// His flight's shake as the vector names it; the form gives only the back layers moving
    /// sideways, so the shake must be of them.
    fn flight_shake_row(start: ShakeStart) -> FlightShakeRow {
        assert_eq!(
            (start.layers, start.motion),
            (ShakeLayers::Back, ShakeMotion::Sideways),
            "his flight's shake moves the back layers sideways"
        );
        FlightShakeRow {
            pixels: start.shake.strength,
            frames: start.shake.length,
        }
    }

    /// The frame's `Hit`, named `hit` in the vectors: its side and push time.
    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct HitRow {
        side: SideName,
        push: u8,
        worth: Option<u16>,
    }

    const fn hit_row(hit: Hit) -> HitRow {
        HitRow {
            side: side_name(hit.side),
            push: hit.push.frames(),
            worth: None,
        }
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum RidleyPhaseName {
        Awake,
        CoastingAbove,
        GetawayReady,
        BodyFading,
        EyeFading,
        FakeRetreat,
        Fireballs,
        TakingOff,
        Holding,
        Hovering,
        Lunge,
        ReadyToRoar,
        Retreating,
        Rising,
        Roaring,
        Slowing,
        Swoop,
        Waiting,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum RidleyLookName {
        Open,
        Rest,
        TakeOff1,
        TakeOff2,
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

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(untagged)]
    enum RidleyColoursRow {
        Baby(BabyColoursName),
        Backdrop([BackdropPalette; 3]),
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    enum BackdropPalette {
        BackgroundBody,
        BackgroundWalls,
        SpriteWalls,
    }

    /// The roar's first four figures, in their order, numbered 1 to 4.
    const TIMED_ROAR_FIGURES: [TimedRoarFigure; 4] = [
        TimedRoarFigure::First,
        TimedRoarFigure::Second,
        TimedRoarFigure::Third,
        TimedRoarFigure::Fourth,
    ];

    /// The take-off's first four figures, in their order, numbered 1 to 4.
    const TIMED_TAKEOFF_FIGURES: [TimedTakeOffFigure; 4] = [
        TimedTakeOffFigure::First,
        TimedTakeOffFigure::Second,
        TimedTakeOffFigure::Third,
        TimedTakeOffFigure::Fourth,
    ];

    /// The fifth figure of the roar or the take-off, held, with no runs.
    const FIFTH_FIGURE: u8 = 5;

    /// The figure a vector's number, 1 to 5, and runs name, of the four timed figures given.
    fn figure_sequence<F: FigureLength + PartialEq>(
        timed: [F; 4],
        number: u8,
        runs: Option<u8>,
    ) -> FigureSequence<F> {
        if number == FIFTH_FIGURE {
            assert!(runs.is_none(), "the fifth figure has no runs");
            return FigureSequence::Fifth;
        }
        let figure = timed
            .into_iter()
            .zip(1..)
            .find_map(|(figure, n)| (n == number).then_some(figure))
            .unwrap_or_else(|| panic!("figure {number} is not 1 to 5"));
        FigureSequence::Timed(
            ShownFigure::new(figure, runs.expect("figures 1 to 4 name their runs"))
                .expect("runs are 1 to the figure's length"),
        )
    }

    /// A figure of the roar or the take-off as a vector names it: its number and its runs.
    fn figure_sequence_cells<F: FigureLength + PartialEq>(
        timed: [F; 4],
        figure: FigureSequence<F>,
    ) -> (Option<u8>, Option<u8>) {
        match figure {
            FigureSequence::Timed(shown) => (
                timed
                    .into_iter()
                    .zip(1..)
                    .find_map(|(figure, n)| (figure == shown.figure()).then_some(n)),
                Some(shown.runs()),
            ),
            FigureSequence::Fifth => (Some(FIFTH_FIGURE), None),
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
                    .and_then(num::NonZeroU16::new)
                    .expect("an eye's fade names its steps, 1 up"),
            },
            RidleyPhaseName::FakeRetreat => RidleyPhase::FakeRetreat,
            RidleyPhaseName::GetawayReady => RidleyPhase::GETAWAY_START,
            RidleyPhaseName::CoastingAbove => RidleyPhase::CoastingAbove(
                cells
                    .coast_runs
                    .and_then(CoastRuns::new)
                    .expect("coast runs are 0 to 63"),
            ),
            RidleyPhaseName::Fireballs => RidleyPhase::Attacking(Attack::Fireballs),
            RidleyPhaseName::TakingOff => RidleyPhase::InFlight(FlightPhase::TakingOff),
            RidleyPhaseName::Holding => RidleyPhase::Holding,
            RidleyPhaseName::Hovering => RidleyPhase::InFlight(FlightPhase::Hovering),
            RidleyPhaseName::Lunge => RidleyPhase::Attacking(Attack::Lunge),
            RidleyPhaseName::ReadyToRoar => RidleyPhase::ReadyToRoar {
                wait: ReadyWait::new(runs_left(cells.wait)).expect("ready's wait is 0 to 4"),
            },
            RidleyPhaseName::Retreating => RidleyPhase::Retreating,
            RidleyPhaseName::Rising => RidleyPhase::InFlight(FlightPhase::Rising),
            RidleyPhaseName::Roaring => RidleyPhase::Roaring {
                wait: RoarWait::new(runs_left(cells.wait)).expect("the roar's wait is 0 to 252"),
                figure: figure_sequence(
                    TIMED_ROAR_FIGURES,
                    cells.figure.expect("a roar names its figure"),
                    cells.figure_runs,
                ),
            },
            RidleyPhaseName::Slowing => RidleyPhase::InFlight(FlightPhase::Slowing),
            RidleyPhaseName::Swoop => RidleyPhase::Attacking(Attack::Swoop),
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
            RidleyPhase::FakeRetreat => (RidleyPhaseName::FakeRetreat, none),
            RidleyPhase::Retreating => (RidleyPhaseName::Retreating, none),
            RidleyPhase::Getaway(GetawayState::GetawayReady(_)) => {
                (RidleyPhaseName::GetawayReady, none)
            }
            RidleyPhase::Getaway(_) => panic!("the flyby has its own result row"),
            RidleyPhase::CoastingAbove(runs) => (
                RidleyPhaseName::CoastingAbove,
                PhaseCells {
                    coast_runs: Some(runs.runs()),
                    ..none
                },
            ),
            RidleyPhase::Attacking(Attack::Fireballs) => (RidleyPhaseName::Fireballs, none),
            RidleyPhase::Attacking(Attack::Lunge) => (RidleyPhaseName::Lunge, none),
            RidleyPhase::Attacking(Attack::Swoop) => (RidleyPhaseName::Swoop, none),
            RidleyPhase::InFlight(FlightPhase::TakingOff) => (RidleyPhaseName::TakingOff, none),
            RidleyPhase::InFlight(FlightPhase::Hovering) => (RidleyPhaseName::Hovering, none),
            RidleyPhase::InFlight(FlightPhase::Rising) => (RidleyPhaseName::Rising, none),
            RidleyPhase::InFlight(FlightPhase::Slowing) => (RidleyPhaseName::Slowing, none),
            RidleyPhase::Holding => (RidleyPhaseName::Holding, none),
            RidleyPhase::ReadyToRoar { wait } => (
                RidleyPhaseName::ReadyToRoar,
                PhaseCells {
                    wait: Some(wait.runs().into()),
                    ..none
                },
            ),
            RidleyPhase::Roaring { wait, figure } => {
                let (figure, figure_runs) = figure_sequence_cells(TIMED_ROAR_FIGURES, figure);
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
            RidleyLookName::TakeOff1 => RidleyLook::TakeOff1,
            RidleyLookName::TakeOff2 => RidleyLook::TakeOff2,
            RidleyLookName::Wide => RidleyLook::Wide,
        }
    }

    const fn ridley_look_name(look: RidleyLook) -> RidleyLookName {
        match look {
            RidleyLook::Open => RidleyLookName::Open,
            RidleyLook::Rest => RidleyLookName::Rest,
            RidleyLook::TakeOff1 => RidleyLookName::TakeOff1,
            RidleyLook::TakeOff2 => RidleyLookName::TakeOff2,
            RidleyLook::Wide => RidleyLookName::Wide,
        }
    }

    /// Whether he has taken off: the part-3 to part-4b forms name no look from then on, as it
    /// is part 4c's.
    const fn taken_off(phase: RidleyPhase) -> bool {
        matches!(
            phase,
            RidleyPhase::InFlight(_)
                | RidleyPhase::Attacking(_)
                | RidleyPhase::FakeRetreat
                | RidleyPhase::Retreating
                | RidleyPhase::CoastingAbove(_)
                | RidleyPhase::GETAWAY_START
        )
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
            flyby: flyby_content(1),
            getaway: GetawayContent {
                left_wall: point([12, 34]),
                right_wall: point([210, 56]),
                body_colours: core::array::from_fn(|i| {
                    numbered_colour(10_u16.saturating_add(u16::try_from(i).unwrap()))
                }),
                wall_colours: core::array::from_fn(|i| {
                    numbered_colour(100_u16.saturating_add(u16::try_from(i).unwrap()))
                }),
            },
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
            look: ridley
                .look
                .filter(|_| !taken_off(ridley.phase))
                .map(ridley_look_name),
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

    /// The curl a vector's `ridley` names: a part-4b or part-4c-1a vector names it; the others
    /// name none and run with the curl he is placed with, which their checks don't read.
    fn curl(vector: &Vector, row: &RidleyRow) -> TailCurl {
        assert_eq!(
            vector.names_tail() || vector.names_flight(),
            row.curl.is_some(),
            "{}: the part-4b and part-4c-1a vectors alone name the curl",
            vector.id
        );
        row.curl.map_or(TailCurl::PLACED, |curl| {
            TailCurl::new(curl).expect("a curl is 8 to 23")
        })
    }

    /// The baby a vector's `baby` names; a flight vector names none, and runs with the baby as
    /// he is placed, whose routine its check doesn't read.
    fn baby(vector: &Vector) -> Baby {
        assert_eq!(
            vector.names_flight(),
            vector.baby.is_none(),
            "{}: every form but the part-4c-1a one names the baby",
            vector.id
        );
        vector.baby.as_ref().map_or(Baby::NEW, |row| Baby {
            routine: routine(row),
            position: None,
        })
    }

    /// Ridley as a vector's `ridley` names him. The fields from his flight on are 0, or none,
    /// where it doesn't name them: as he is placed.
    fn ridley(vector: &Vector) -> CeresRidley {
        let row = vector.ridley.as_ref().expect("a run starts from his state");
        let [fraction_x, fraction_y] = row.fraction.unwrap_or_default();
        let [speed_x, speed_y] = row.speed.unwrap_or_default();
        CeresRidley {
            phase: ridley_phase(row),
            look: row.look.map(ridley_look),
            position: point(row.position),
            fraction: Pair {
                x: fraction_x,
                y: fraction_y,
            },
            speed: Pair {
                x: Wrapping(speed_x),
                y: Wrapping(speed_y),
            },
            takeoff: row
                .takeoff_figure
                .map(|number| figure_sequence(TIMED_TAKEOFF_FIGURES, number, row.takeoff_runs)),
            hover_runs: HoverRuns::new(row.hover_runs.unwrap_or(0))
                .expect("his hover's runs are 0 to 123"),
            baby: baby(vector),
            curl: curl(vector, row),
            flight_tail: vector.flight_tail.as_ref().map(FlightTailRow::shape),
            wings: FightWings::PLACED,
            ribs: FightRibs::PLACED,
        }
    }

    /// Flight and retreat outputs. Older Hovers vectors check only the handoff phase.
    fn flight_after(run: &RidleyRun, vector: &Vector) -> RidleyAfter {
        let ridley = run.ridley;
        let (phase, cells) = ridley_phase_row(ridley.phase);
        let retreat = vector.flight_tail_after.is_some()
            && matches!(
                ridley.phase,
                RidleyPhase::Retreating
                    | RidleyPhase::CoastingAbove(_)
                    | RidleyPhase::GETAWAY_START
            );
        if !retreat
            && matches!(
                ridley.phase,
                RidleyPhase::FakeRetreat | RidleyPhase::Retreating
            )
        {
            return RidleyAfter {
                phase: Some(phase),
                ..RidleyAfter::default()
            };
        }
        let (takeoff_figure, takeoff_runs) = ridley.takeoff.map_or((None, None), |figure| {
            figure_sequence_cells(TIMED_TAKEOFF_FIGURES, figure)
        });
        RidleyAfter {
            phase: Some(phase),
            wait: cells.wait,
            figure: cells.figure,
            figure_runs: cells.figure_runs,
            takeoff_figure,
            takeoff_runs,
            look: ridley.look.map(ridley_look_name),
            position: Some(point_row(ridley.position)),
            fraction: Some([ridley.fraction.x, ridley.fraction.y]),
            speed: Some([ridley.speed.x.0, ridley.speed.y.0]),
            hover_runs: Some(ridley.hover_runs.get()),
            curl: Some(ridley.curl.get()),
            tangible: Some(ridley.phase.tangible()),
            moving: Some(ridley.phase.moving()),
            roaring: Some(ridley.phase.roaring()),
            music: Some(run.music.is_some()),
            roar: Some(run.roar.is_some()),
            held: ridley.baby.position.map(point_row),
            bound: run.bound.map(bound_name),
            shake: run.shake.map(flight_shake_row),
            whip: vector
                .flight_tail_after
                .is_some()
                .then_some(run.swish.is_some()),
            hit: run.hit.map(|hit| HitRow {
                worth: retreat.then_some(boss_ceres_ridley::TAIL_DAMAGE),
                ..hit_row(hit)
            }),
            coast_runs: cells.coast_runs,
            walls: run.backdrop.map(|backdrop| {
                [
                    side_name(backdrop.left_wall.door.facing),
                    side_name(backdrop.right_wall.door.facing),
                ]
            }),
            colours: run.backdrop.map(|_| {
                RidleyColoursRow::Backdrop([
                    BackdropPalette::BackgroundBody,
                    BackdropPalette::BackgroundWalls,
                    BackdropPalette::SpriteWalls,
                ])
            }),
            ..RidleyAfter::default()
        }
    }

    /// Vectors name the wall sides and destinations; verify the payloads against the content too.
    fn check_backdrop(backdrop: boss_ceres_ridley::GetawayBackdrop, content: &GetawayContent) {
        for (placement, position) in [
            (backdrop.left_wall, content.left_wall),
            (backdrop.right_wall, content.right_wall),
        ] {
            assert_eq!(placement.door.kind, DoorKind::GetawayWall);
            assert_eq!(placement.door.phase, Phase::Placed);
            assert_eq!(placement.door.position, position);
            assert_eq!(placement.colours.palette, DoorPalette::Doors);
            assert_eq!(placement.colours.set, ColourSet::Calm);
        }
        assert_eq!(boss_ceres_ridley::GetawayColours::FIRST_INDEX, 1);
        assert_eq!(backdrop.colours.background_body, content.body_colours);
        assert_eq!(backdrop.colours.background_walls, content.wall_colours);
        assert_eq!(backdrop.colours.sprite_walls, content.wall_colours);
    }

    /// One run of him and the baby from the vector's `ridley` and `baby`, with its content. A
    /// part-4a vector, which names its content, reads his waking's outputs too; a part-4c-1a
    /// vector reads his flight's.
    fn ridley_run(vector: &Vector) -> RidleyAfter {
        assert!(
            vector.beaten.is_none(),
            "a run reads nothing of his being beaten"
        );
        let content = ridley_content(&vector.content.clone().unwrap_or_else(stand_in_content));
        let run = ridley(vector).run(
            boss_ceres_ridley::RidleyFrame {
                passage_under_way: vector.passing.unwrap_or(false),
                finished: GameFrames(0),
                picture: FightPictureInputs::LEFT_UNHURT,
            },
            vector.random(),
            vector.hitter_her().as_ref(),
            vector.fight(),
            &content,
            &rules(&vector.rules),
        );
        if vector.names_flight() {
            if let Some(expected) = &vector.flight_tail_after {
                check_tail_run(vector, &run, expected);
            }
            if let Some(backdrop) = run.backdrop {
                check_backdrop(backdrop, &content.getaway);
            }
            return flight_after(&run, vector);
        }
        let part_3 = RidleyAfter {
            cry: Some(run.cry.is_some()),
            colours: run
                .colours
                .map(|colours| RidleyColoursRow::Baby(baby_colours_name(colours))),
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
            assert!(vector.hitter_her().is_none(), "the tail's her is part 4b's");
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
        assert!(
            !vector.names_doors()
                && !vector.names_pad()
                && !vector.names_lights()
                && !vector.names_steam()
        );
        assert!(!vector.placing, "Ridley's placing is ridley_placing");
        let actual = if vector.ridley_placing {
            ridley_placed(vector)
        } else {
            ridley_run(vector)
        };
        (actual != *expected)
            .then(|| format!("{}: expected {expected:?}, got {actual:?}", vector.id))
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct FlightTailRow {
        pace: u8,
        mark: Option<u16>,
        swish: u8,
        segments: [TailSegmentRow; TAIL_SEGMENTS],
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct TailSegmentRow {
        angle: u16,
        way: WayRow,
        swinging: bool,
        lag: LagRow,
        length: u16,
        stretching: bool,
        offset: [i16; 2],
        place: [u16; 2],
    }

    #[derive(Clone, Copy, Deserialize)]
    enum WayRow {
        Ahead,
        Back,
    }

    #[derive(Clone, Copy, Deserialize)]
    enum PassedRow {
        Passed,
    }

    #[derive(Clone, Copy, Deserialize)]
    #[serde(untagged)]
    enum LagRow {
        Counting(u8),
        Passed(PassedRow),
    }

    impl FlightTailRow {
        fn shape(&self) -> FlightTail {
            FlightTail {
                pace: match self.pace {
                    1 => SwingPace::Slow,
                    2 => SwingPace::Quick,
                    8 => SwingPace::Whip,
                    pace => panic!("unknown swing pace {pace}"),
                },
                mark: self
                    .mark
                    .map(|mark| TailMark::new(TailAngle(mark)).expect("mark in aim arc")),
                swish: SwishRuns::new(self.swish).expect("swish runs 0 to 15"),
                segments: self.segments.each_ref().map(TailSegmentRow::segment),
            }
        }
    }

    impl TailSegmentRow {
        fn segment(&self) -> TailSegment {
            let [x, y] = self.offset;
            TailSegment {
                angle: TailAngle(self.angle),
                way: match self.way {
                    WayRow::Ahead => SwingWay::Ahead,
                    WayRow::Back => SwingWay::Back,
                },
                swinging: self.swinging,
                lag: match self.lag {
                    LagRow::Counting(n) => {
                        SwingLag::Counting(LagCount::new(n).expect("lag 0 to 30"))
                    }
                    LagRow::Passed(PassedRow::Passed) => SwingLag::Passed,
                },
                length: TailLength::new(self.length).expect("length at most 24 px"),
                stretching: self.stretching,
                offset: Pair { x, y },
                place: point(self.place),
            }
        }
    }

    fn check_tail_run(vector: &Vector, run: &RidleyRun, expected: &FlightTailRow) {
        let expected = expected.shape();
        let actual = run.ridley.flight_tail.expect("flight carries its tail");
        assert_eq!(actual.pace, expected.pace, "{} pace", vector.id);
        assert_eq!(actual.mark, expected.mark, "{} mark", vector.id);
        assert_eq!(actual.swish, expected.swish, "{} swish count", vector.id);
        for (index, (actual, expected)) in actual
            .segments
            .into_iter()
            .zip(expected.segments)
            .enumerate()
        {
            assert_eq!(actual, expected, "{} segment {index}", vector.id);
        }
        assert_eq!(
            run.tail_places,
            Some(expected.places()),
            "{} drawing places",
            vector.id
        );
    }

    // ------------------------------------------------------------ the steam

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum VentFacingName {
        Down,
        Left,
        Right,
        Up,
    }

    const fn vent_facing(name: VentFacingName) -> VentFacing {
        match name {
            VentFacingName::Down => VentFacing::Down,
            VentFacingName::Left => VentFacing::Left,
            VentFacingName::Right => VentFacing::Right,
            VentFacingName::Up => VentFacing::Up,
        }
    }

    /// A vent's phase as the vectors name it; the vector file spells the phase before the
    /// first puff `Waiting`.
    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum VentPhaseName {
        #[serde(rename = "Waiting")]
        AwaitingFirstPuff,
        Placed,
        Puffing,
        Resting,
    }

    /// A vent a placing places: its facing and position.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VentPlaceRow {
        facing: VentFacingName,
        position: [u16; 2],
    }

    /// A vent as a run's `vents` names it: its facing, position, phase and the phase's count.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VentRow {
        facing: VentFacingName,
        position: [u16; 2],
        phase: VentPhaseName,
        wait: Option<u8>,
        figure: Option<u8>,
        runs: Option<u8>,
    }

    /// A vent as `vents_after` reads it: its phase with its count, and whether its puff is shown.
    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    #[serde(deny_unknown_fields)]
    struct VentAfter {
        phase: VentPhaseName,
        wait: Option<u8>,
        figure: Option<u8>,
        runs: Option<u8>,
        shown: bool,
    }

    /// A vent's wait for its first puff, as a vent row's `wait` names it.
    fn first_puff_wait(runs: Option<u8>) -> FirstPuffWait {
        runs.and_then(FirstPuffWait::new)
            .expect("a vent's first puff's wait is named, 1 to 32")
    }

    /// The phase a vent row names; reading it back must give the same cells, so each phase
    /// names only what it carries.
    fn vent_phase(row: &VentRow) -> VentPhase {
        let phase = match row.phase {
            VentPhaseName::AwaitingFirstPuff => {
                VentPhase::AwaitingFirstPuff(first_puff_wait(row.wait))
            }
            VentPhaseName::Placed => VentPhase::Placed(first_puff_wait(row.wait)),
            VentPhaseName::Puffing => VentPhase::Puffing(
                row.figure
                    .and_then(PuffFigure::new)
                    .zip(row.runs)
                    .and_then(|(figure, remaining_hold)| Puff::new(figure, remaining_hold))
                    .expect("a puff names its figure, 1 to 7, and its remaining hold, 1 to 3"),
            ),
            VentPhaseName::Resting => VentPhase::Resting(
                row.runs
                    .and_then(num::NonZeroU8::new)
                    .expect("a rest names its runs, 1 to 255"),
            ),
        };
        assert_eq!(
            vent_phase_cells(phase),
            (row.phase, row.wait, row.figure, row.runs),
            "a vent's phase names only what it carries"
        );
        phase
    }

    /// A vent's phase as the vectors name it: its name, wait, figure and runs (a puff's runs are
    /// its figure's remaining hold).
    const fn vent_phase_cells(
        phase: VentPhase,
    ) -> (VentPhaseName, Option<u8>, Option<u8>, Option<u8>) {
        match phase {
            VentPhase::AwaitingFirstPuff(wait) => (
                VentPhaseName::AwaitingFirstPuff,
                Some(wait.runs()),
                None,
                None,
            ),
            VentPhase::Placed(wait) => (VentPhaseName::Placed, Some(wait.runs()), None, None),
            VentPhase::Puffing(puff) => (
                VentPhaseName::Puffing,
                None,
                Some(puff.figure().get()),
                Some(puff.remaining_hold()),
            ),
            VentPhase::Resting(runs) => (VentPhaseName::Resting, None, None, Some(runs.get())),
        }
    }

    fn vent(row: &VentRow) -> Vent {
        Vent {
            facing: vent_facing(row.facing),
            position: point(row.position),
            phase: vent_phase(row),
        }
    }

    /// A vent as `vents_after` reads it; the puff figure it shows, its `VentPuffs` output, must
    /// be its phase's.
    fn vent_after(vent: Vent) -> VentAfter {
        let (phase, wait, figure, runs) = vent_phase_cells(vent.phase);
        assert_eq!(vent.shown_puff_figure().map(PuffFigure::get), figure);
        VentAfter {
            phase,
            wait,
            figure,
            runs,
            shown: vent.shown(),
        }
    }

    /// The vents a steam vector's step leaves, and the frame's hit: a vent's placing, or one run
    /// of the room's vents.
    fn steam_step(vector: &Vector) -> (Vec<Vent>, Option<Hit>) {
        if vector.steam_placing {
            assert!(
                vector.vents.is_none() && vector.camera.is_none() && vector.her.is_none(),
                "a placed vent's facing, position and wait come from its vent row and the value \
                 its wait is drawn from alone"
            );
            let row = vector.vent.as_ref().expect("a placing names its vent");
            let random = vector.random.as_deref().map(hex);
            let placed = Vent::place(
                vent_facing(row.facing),
                point(row.position),
                random.expect("a placed vent's wait, 1 + (r mod 32), needs the r it is drawn from"),
            );
            return (vec![placed], None);
        }
        assert!(
            vector.vent.is_none() && vector.random.is_none(),
            "a run's vents carry their own counts, so none is drawn from a random number"
        );
        let mut vents: Vec<Vent> = vector
            .vents
            .as_ref()
            .expect("a run names the room's vents")
            .iter()
            .map(vent)
            .collect();
        let camera = point(vector.camera.expect("a run names the camera"));
        let hit = run_vents(
            &mut vents,
            camera,
            vector.hitter_her().as_ref(),
            &rules(&vector.rules),
        );
        (vents, hit)
    }

    /// The steam's vector's check: its step against its `vents_after` and its `hit`.
    fn check_steam(vector: &Vector, expected: &[VentAfter]) -> Option<String> {
        assert!(
            !vector.names_doors()
                && !vector.names_pad()
                && !vector.names_lights()
                && !vector.names_ridley_own()
                && vector.beaten.is_none()
                && !vector.placing
        );
        let (vents, hit) = steam_step(vector);
        let actual: Vec<VentAfter> = vents.into_iter().map(vent_after).collect();
        let actual_hit = hit.map(hit_row);
        (actual != expected || actual_hit != vector.hit).then(|| {
            format!(
                "{}: expected {expected:?} and hit {:?}, got {actual:?} and hit {actual_hit:?}",
                vector.id, vector.hit
            )
        })
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    enum GetawayPhaseRow {
        GetawayReady,
        FlyingPast,
        WarningReady,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct GetawayRow {
        phase: GetawayPhaseRow,
        shown: Option<usize>,
        position: Option<[i16; 2]>,
        turn: Option<u16>,
        baby: Option<usize>,
        wings: Option<usize>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct PictureRow {
        shift: [i16; 2],
        zoom: u16,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    enum PlaneRow {
        Flyby,
        Room,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    enum BabyLookRow {
        First,
        Middle,
        Third,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    enum GetawaySoundRow {
        Getaway,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    enum PushRow {
        OutOfTheWay,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct QuakeRow {
        reach: QuakeReach,
        direction: QuakeDirection,
        pixels: u8,
        frames: u16,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct GetawayEffectsRow {
        stage: Option<StageName>,
        sound: Option<GetawaySoundRow>,
        push: Option<PushRow>,
        quake: Option<QuakeRow>,
        baby_look: Option<BabyLookRow>,
        wing_look: Option<FlybyWingBeat>,
        plane: PlaneRow,
        scroll: [i16; 2],
        origin: [i16; 2],
        matrix: [i32; 4],
        zoom: Option<u16>,
        shade: Option<u16>,
    }

    const BABY_BEATS: [FlybyBabyBeat; 4] = [
        FlybyBabyBeat::First,
        FlybyBabyBeat::ToThird,
        FlybyBabyBeat::Third,
        FlybyBabyBeat::ToFirst,
    ];

    const fn frame(finished: u16) -> boss_ceres_ridley::RidleyFrame {
        boss_ceres_ridley::RidleyFrame {
            passage_under_way: false,
            finished: GameFrames(finished),
            picture: FightPictureInputs::LEFT_UNHURT,
        }
    }

    fn signed_pair([x, y]: [i16; 2]) -> Pair<i16> {
        Pair { x, y }
    }

    fn flyby_state(row: &GetawayRow) -> GetawayState {
        match row.phase {
            GetawayPhaseRow::GetawayReady => GetawayState::GetawayReady(flyby_beats(row)),
            GetawayPhaseRow::FlyingPast => GetawayState::FlyingPast(FlybyProgress {
                shown: num::NonZeroUsize::new(row.shown.unwrap()).unwrap(),
                scroll: signed_pair(row.position.unwrap()),
                turn: FlybyTurn(row.turn.unwrap()),
                beats: flyby_beats(row),
            }),
            GetawayPhaseRow::WarningReady => GetawayState::WarningReady,
        }
    }

    fn flyby_beats(row: &GetawayRow) -> FlybyBeats {
        FlybyBeats {
            baby: BABY_BEATS[row.baby.unwrap()],
            wings: FlybyWingBeat::from_usize(row.wings.unwrap()),
        }
    }

    fn getaway_row(state: GetawayState) -> GetawayRow {
        let (phase, progress, beats) = match state {
            GetawayState::GetawayReady(beats) => (GetawayPhaseRow::GetawayReady, None, Some(beats)),
            GetawayState::FlyingPast(progress) => (
                GetawayPhaseRow::FlyingPast,
                Some(progress),
                Some(progress.beats),
            ),
            GetawayState::WarningReady => (GetawayPhaseRow::WarningReady, None, None),
        };
        GetawayRow {
            phase,
            shown: progress.map(|p| p.shown.get()),
            position: progress.map(|p| [p.scroll.x, p.scroll.y]),
            turn: progress.map(|p| p.turn.0),
            baby: beats.and_then(|b| BABY_BEATS.iter().position(|&beat| beat == b.baby)),
            wings: beats.map(|b| b.wings.into_usize()),
        }
    }

    fn flyby_content(length: usize) -> FlybyContent {
        assert!(length >= 1);
        let first = FlybyFrame {
            shift: signed_pair([0, 0]),
            zoom: FlybyZoom::new(256).unwrap(),
        };
        FlybyContent {
            path: FlybyPath {
                first,
                rest: vec![first; length.saturating_sub(1)],
            },
            body_shades: core::array::from_fn(|shade| {
                [numbered_colour(u16::try_from(shade).unwrap()); 15]
            }),
            baby_looks: enum_map! {
                FlybyBabyLook::First => Picture::new(11).unwrap(),
                FlybyBabyLook::Middle => Picture::new(12).unwrap(),
                FlybyBabyLook::Third => Picture::new(13).unwrap(),
            },
            wing_looks: enum_map! {
                FlybyWingBeat::First => Picture::new(21).unwrap(),
                FlybyWingBeat::Second => Picture::new(22).unwrap(),
            },
        }
    }

    fn effects_row(
        effects: boss_ceres_ridley::GetawayEffects,
        content: &FlybyContent,
    ) -> GetawayEffectsRow {
        let transform = effects.view.transform();
        let mut row = GetawayEffectsRow {
            stage: effects.stage.map(|stage| match stage {
                Stage::Calm => StageName::Calm,
                Stage::Escape => StageName::Escape,
                Stage::Getaway => StageName::Getaway,
            }),
            sound: effects.sound.map(|GetawaySound| {
                assert_eq!(
                    (GetawaySound::EFFECTS_CHANNEL, GetawaySound::QUEUE_LIMIT),
                    (2, 6)
                );
                GetawaySoundRow::Getaway
            }),
            push: effects.push.map(|OutOfTheWay| PushRow::OutOfTheWay),
            quake: effects.quake.map(
                |QuakeRequest {
                     kind:
                         QuakeKind {
                             reach,
                             direction,
                             size,
                         },
                     duration: QuakeDuration(frames),
                 }| QuakeRow {
                    reach,
                    direction,
                    pixels: size.pixels(),
                    frames,
                },
            ),
            plane: PlaneRow::Room,
            scroll: [transform.scroll.x, transform.scroll.y],
            origin: [transform.pivot.x, transform.pivot.y],
            matrix: transform.matrix,
            zoom: None,
            shade: None,
            baby_look: None,
            wing_look: None,
        };
        if let GetawayView::Flyby {
            zoom,
            body_colours,
            baby_look,
            wing_look,
            ..
        } = effects.view
        {
            row.plane = PlaneRow::Flyby;
            row.zoom = Some(zoom.units());
            row.shade = Some(
                u16::try_from(
                    content
                        .body_shades
                        .iter()
                        .position(|colours| *colours == body_colours)
                        .unwrap(),
                )
                .unwrap(),
            );
            row.baby_look = baby_look.map(|art| {
                let (look, _) = content
                    .baby_looks
                    .iter()
                    .find(|(_, picture)| **picture == art)
                    .unwrap();
                match look {
                    FlybyBabyLook::First => BabyLookRow::First,
                    FlybyBabyLook::Middle => BabyLookRow::Middle,
                    FlybyBabyLook::Third => BabyLookRow::Third,
                }
            });
            row.wing_look = wing_look.map(|art| {
                content
                    .wing_looks
                    .iter()
                    .find(|(_, picture)| **picture == art)
                    .unwrap()
                    .0
            });
        }
        row
    }

    fn check_getaway(vector: &Vector, expected: &GetawayRow) -> Option<String> {
        let input = vector.getaway.as_ref().unwrap();
        let state = flyby_state(input);
        assert_eq!(&getaway_row(state), input, "{}: flyby input", vector.id);
        let saved = toml::to_string(&state).unwrap();
        let restored = toml::from_str::<GetawayState>(&saved).unwrap();
        assert_eq!(restored, state, "{}: restored flyby", vector.id);
        let mut content = ridley_content(&stand_in_content());
        content.flyby = flyby_content(vector.length.unwrap_or(112));
        let shown = input.shown.unwrap_or(0);
        if let Some(picture) = &vector.picture {
            let frame = FlybyFrame {
                shift: signed_pair(picture.shift),
                zoom: FlybyZoom::new(picture.zoom).unwrap(),
            };
            if shown == 0 {
                content.flyby.path.first = frame;
            } else {
                content.flyby.path.rest[shown.saturating_sub(1)] = frame;
            }
        }
        let mut ridley = CeresRidley::place(false).unwrap().ridley;
        ridley.phase = RidleyPhase::Getaway(restored);
        let run = ridley.run(
            frame(vector.finished.unwrap()),
            0,
            None,
            vector.fight(),
            &content,
            &Rules::default(),
        );
        let RidleyPhase::Getaway(state) = run.ridley.phase else {
            panic!("{}: lost the flyby state", vector.id)
        };
        assert_eq!((run.hit, run.shake, run.cry), (None, None, None));
        assert!(!run.ridley.phase.tangible());
        let drawing = run.ridley.drawing();
        assert_eq!(
            (drawing.ridley, drawing.baby, drawing.tail),
            (None, None, None)
        );
        let actual = getaway_row(state);
        let effects = effects_row(run.getaway.unwrap(), &content.flyby);
        let expected_effects = vector.effects.as_ref().unwrap();
        (&actual != expected || &effects != expected_effects).then(|| {
            format!(
                "{}: expected {expected:?}, {expected_effects:?}; got {actual:?}, {effects:?}",
                vector.id
            )
        })
    }

    fn check_warning(vector: &Vector, expected: &WarningRow) -> Option<String> {
        let inputs = boss_ceres_ridley::WarningInputs {
            ordinary: vector.ordinary.unwrap_or(true),
            finished: GameFrames(vector.finished.unwrap_or(1)),
            fade: vector.fade.unwrap_or(false),
            passage: vector.passage.unwrap_or(false),
            language: vector.language.unwrap_or(WarningLanguage::English).into(),
            foreground: warning_vectors::foreground(),
        };
        let (actual, effects) = warning_vectors::run(
            vector.warning.as_ref().unwrap(),
            inputs,
            vector.art.as_deref(),
            vector.text.as_deref(),
        );
        let expected_effects = vector.warning_effects.as_ref().unwrap();
        (&actual != expected || &effects != expected_effects).then(|| {
            format!(
                "{}: expected {expected:?}, {expected_effects:?}; got {actual:?}, {effects:?}",
                vector.id
            )
        })
    }

    /// The vector's check, by the result it names; exactly one.
    fn check(vector: &Vector) -> Option<String> {
        match (
            &vector.after,
            &vector.shaft_after,
            &vector.lights_after,
            &vector.ridley_after,
            &vector.vents_after,
            &vector.getaway_after,
            &vector.warning_after,
        ) {
            (Some(after), None, None, None, None, None, None) => check_door(vector, after),
            (None, Some(after), None, None, None, None, None) => check_pad(vector, after),
            (None, None, Some(after), None, None, None, None) => check_lights(vector, after),
            (None, None, None, Some(after), None, None, None) => check_ridley(vector, after),
            (None, None, None, None, Some(after), None, None) => check_steam(vector, after),
            (None, None, None, None, None, Some(after), None) => check_getaway(vector, after),
            (None, None, None, None, None, None, Some(after)) => check_warning(vector, after),
            _ => panic!("{}: a vector names exactly one result", vector.id),
        }
    }

    /// Take-off can hit on the same run the tail comes loose. This position is just inside
    /// the swung tip's reach, and outside the reach of the old resting tip.
    #[test]
    fn takeoff_hit_reads_the_new_tip() {
        let mut ridley = CeresRidley::place(false)
            .expect("unbeaten Ridley is placed")
            .ridley;
        ridley.phase = RidleyPhase::Roaring {
            wait: RoarWait::new(0).expect("the roar is over"),
            figure: FigureSequence::Fifth,
        };
        ridley.curl = TailCurl::new(23).expect("tight curl");
        let her = Her {
            position: point([246, 174]),
            body: point([5, 12]),
            protection: 0,
        };
        let rules = Rules::default();
        let content = ridley_content(&stand_in_content());
        let fight = Fight {
            her_energy: 54,
            hits_taken: 11,
        };
        let run = ridley.run(frame(0), 0xFF00, Some(&her), fight, &content, &rules);
        assert_eq!(run.ridley.curl.get(), 8);
        assert_eq!(
            run.tail_places.expect("take-off reports places")[TAIL_SEGMENTS - 1],
            point([228, 174])
        );
        assert_eq!(
            run.hit.map(hit_row),
            Some(HitRow {
                side: side_name(Side::Right),
                push: 5,
                worth: None,
            })
        );
    }

    #[test]
    fn retreat_hands_off_to_the_flyby_on_the_next_run() {
        let vectors = vectors();
        let seed = vectors
            .iter()
            .find(|vector| vector.id == "RT-first-pull")
            .unwrap();
        let saved = ridley(seed);
        let content = ridley_content(&stand_in_content());
        let fight = Fight {
            her_energy: 24,
            hits_taken: 100,
        };
        let rules = Rules::default();
        let fake = saved.run(frame(0), 0xffff, None, fight, &content, &rules);
        assert_eq!(
            fake.ridley,
            CeresRidley {
                phase: RidleyPhase::FakeRetreat,
                ..saved
            }
        );
        assert_eq!(
            (fake.hit, fake.swish, fake.tail_places, fake.backdrop),
            (None, None, None, None)
        );

        let coast = vectors
            .iter()
            .find(|vector| vector.id == "RT-walls")
            .unwrap();
        let mut coasting = ridley(coast);
        coasting.ribs = saved_ribs((RibBeat::Opening, 1));
        let stopped = coasting.run(
            frame(7),
            coast.random(),
            None,
            coast.fight(),
            &content,
            &rules,
        );
        let ready = stopped.ridley;
        assert_eq!(ready.phase, RidleyPhase::GETAWAY_START);
        assert!(stopped.getaway.is_none());
        assert_eq!(stopped.rib_reveal, Some(RibLook::PartlyOpen));
        assert_eq!(ready.ribs, saved_ribs((RibBeat::Wide, 20)));
        let backdrop = stopped.backdrop.unwrap();
        let run = ready.run(frame(8), 0xffff, None, fight, &content, &rules);
        assert_eq!(run.ridley.baby, ready.baby);
        assert_eq!(run.ridley.flight_tail, ready.flight_tail);
        assert_eq!(run.ridley.wings, ready.wings);
        assert_eq!(run.ridley.ribs, ready.ribs);
        assert_eq!(run.rib_reveal, None);
        assert_eq!(run.parts, None);
        assert!(ready.picture_order(true).baby);
        assert!(!run.ridley.picture_order(true).baby);
        assert_eq!(
            (run.hit, run.swish, run.tail_places, run.backdrop),
            (None, None, None, None)
        );
        let effects = run.getaway.unwrap();
        assert_eq!(effects.sound, Some(GetawaySound));
        assert!(matches!(effects.view, GetawayView::Flyby { .. }));
        let inputs = Inputs {
            stage: effects.stage.unwrap(),
            her: point([0, 0]),
            random: 0,
            shake_frames_left: 0,
            ridley_beaten: false,
        };
        for placement in [backdrop.left_wall, backdrop.right_wall] {
            assert!(placement.door.run(&inputs, &rules).door.phase.shown());
        }
        let returned = run.ridley.run(frame(12), 0, None, fight, &content, &rules);
        assert_eq!(
            returned.ridley.phase,
            RidleyPhase::Getaway(GetawayState::WarningReady)
        );
        assert_eq!(returned.getaway.unwrap().view, GetawayView::Room);
        assert!(
            returned
                .ridley
                .run(frame(13), 0, None, fight, &content, &rules)
                .getaway
                .is_none()
        );
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct FightPictureVector {
        id: String,
        source: Source,
        entries: Vec<String>,
        #[serde(default)]
        placed: bool,
        #[serde(default)]
        takeoff: bool,
        before: Option<[u8; 2]>,
        after: Option<[u8; 2]>,
        speed: Option<[i16; 2]>,
        bands: Option<Vec<[i16; 3]>>,
        body_before: Option<PictureBody>,
        body_after: Option<PictureBody>,
        facing: Option<PictureFacing>,
        anchor: Option<[u16; 2]>,
        #[serde(default)]
        tail_hidden: bool,
        rest: Option<[u16; 2]>,
        angles: Option<[u16; 2]>,
        places: Option<[[u16; 2]; TAIL_SEGMENTS]>,
        tail_case: Option<String>,
        hurt: Option<[u16; 2]>,
        hits: Option<usize>,
        refreshed: Option<u16>,
        looks: Option<PictureLooks>,
        stage: Option<PictureStage>,
        attached: Option<bool>,
        order: Option<PictureOrderRow>,
    }

    #[derive(Clone, Copy, Deserialize, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct PictureBody {
        anchor: [u16; 2],
        speed: [i16; 2],
        figure: u8,
        runs: u8,
        phase: PictureBodyPhase,
    }

    #[derive(Clone, Copy, Deserialize, Debug, PartialEq, Eq)]
    enum PictureBodyPhase {
        TakingOff,
        Rising,
    }

    #[derive(Clone, Copy, Deserialize)]
    enum PictureFacing {
        Left,
        Front,
        Right,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    enum WingSide {
        Left,
        Right,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    enum PartColours {
        Normal,
        Hurt,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct PictureLooks {
        wing: Option<(WingSide, u8)>,
        wing_anchor: Option<[u16; 2]>,
        tail_tip: Option<u8>,
        tail_anchors: Option<[[u16; 2]; TAIL_SEGMENTS]>,
        colours: PartColours,
    }

    impl From<FightParts> for PictureLooks {
        fn from(parts: FightParts) -> Self {
            Self {
                wing: parts.wing.map(|wing| {
                    (
                        match wing.facing {
                            Side::Left => WingSide::Left,
                            Side::Right => WingSide::Right,
                        },
                        wing.beat.get(),
                    )
                }),
                wing_anchor: parts.wing.map(|wing| point_row(wing.anchor)),
                tail_tip: parts.tail.map(|tail| tail.tip.get()),
                tail_anchors: parts.tail.map(|tail| tail.anchors.map(point_row)),
                colours: match parts.colours {
                    FightColours::Normal => PartColours::Normal,
                    FightColours::Hurt => PartColours::Hurt,
                },
            }
        }
    }

    #[derive(Clone, Copy, Deserialize)]
    enum PictureStage {
        #[serde(rename = "Waiting")]
        BeforeOrdinaryAnimation,
        Fighting,
        Getaway,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    enum OrderOpportunity {
        Before,
        After,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct PictureOrderRow {
        opportunity: OrderOpportunity,
        baby: bool,
        door: bool,
    }

    fn picture_speed([x, y]: [i16; 2]) -> Pair<Speed> {
        Pair {
            x: Wrapping(x),
            y: Wrapping(y),
        }
    }

    fn saved_wings([beat, count]: [u8; 2]) -> FightWings {
        FightWings {
            beat: WingBeat::new(beat).unwrap(),
            remaining: WingCount::new(count).unwrap(),
        }
    }

    impl FightPictureVector {
        fn inputs(&self) -> FightPictureInputs {
            let [remaining, turn] = self.hurt.unwrap_or_default();
            FightPictureInputs {
                facing: match self.facing.unwrap_or(PictureFacing::Front) {
                    PictureFacing::Left => FightFacing::Left,
                    PictureFacing::Front => FightFacing::Front,
                    PictureFacing::Right => FightFacing::Right,
                },
                tail_visible: !self.tail_hidden,
                flash: FightFlash {
                    remaining,
                    accepted_shots: self.hits.unwrap_or_default(),
                    half: match turn {
                        0 | 1 => FlashHalf::First,
                        2 | 3 => FlashHalf::Second,
                        _ => panic!("{}: invalid flash turn", self.id),
                    },
                },
            }
        }

        fn check(&self, vectors: &[Vector]) {
            let mut restored = CeresRidley::place(false).unwrap().ridley;
            if let Some(before) = self.before {
                restored.wings = saved_wings(before);
            }
            let (wings, parts, refresh) = self.check_run(restored, vectors);
            if let Some(after) = self.after {
                assert_eq!(wings, saved_wings(after), "{}: wing clock", self.id);
            }
            assert_eq!(
                parts.map(PictureLooks::from),
                self.looks,
                "{}: part looks",
                self.id
            );
            assert_eq!(refresh, self.refreshed, "{}: hurt refresh", self.id);
            if let Some(bands) = &self.bands {
                for &[x, y, pace] in bands {
                    assert_eq!(
                        i16::from(FightWings::pace(picture_speed([x, y]))),
                        pace,
                        "{}: pace at ({x}, {y})",
                        self.id
                    );
                }
            }
            self.check_order(restored);
        }

        fn check_run(
            &self,
            mut restored: CeresRidley,
            vectors: &[Vector],
        ) -> (FightWings, Option<FightParts>, Option<u16>) {
            let picture = self.inputs();
            if let Some(before) = self.body_before {
                let run = self.run_body(restored, before, vectors);
                self.check_body_after(&run);
                return (run.ridley.wings, run.parts, run.hurt_refresh);
            }
            if self.takeoff {
                restored.phase = RidleyPhase::Roaring {
                    wait: RoarWait::new(0).unwrap(),
                    figure: FigureSequence::Fifth,
                };
                restored.speed = picture_speed(self.speed.unwrap());
                let run = picture_turn(restored, picture);
                assert_eq!(
                    run.ridley.phase,
                    RidleyPhase::InFlight(FlightPhase::TakingOff)
                );
                assert_eq!(run.ridley.speed, restored.speed);
                return (run.ridley.wings, None, run.hurt_refresh);
            }
            if self.placed {
                assert!(self.before.is_none());
                return (restored.wings, None, None);
            }
            let wings = self.speed.map_or(restored.wings, |speed| {
                restored.wings.ran(picture_speed(speed))
            });
            let parts = self.looks.as_ref().map(|_| {
                FightParts::at_anchor(
                    wings,
                    point(self.anchor.unwrap_or_default()),
                    self.tail_intent(),
                    picture,
                )
            });
            (wings, parts, picture.flash.refresh())
        }

        fn tail_intent(&self) -> Option<TailIntent> {
            if let Some(place) = self.rest {
                return Some(TailIntent::resting(point(place)));
            }
            self.places
                .zip(self.angles)
                .map(|(places, [penultimate, last])| {
                    TailIntent::linked(places.map(point), TailAngle(penultimate), TailAngle(last))
                })
        }

        fn run_body(
            &self,
            mut restored: CeresRidley,
            before: PictureBody,
            vectors: &[Vector],
        ) -> RidleyRun {
            restored.position = point(before.anchor);
            restored.speed = picture_speed(before.speed);
            restored.takeoff = Some(figure_sequence(
                TIMED_TAKEOFF_FIGURES,
                before.figure,
                Some(before.runs),
            ));
            restored.phase = RidleyPhase::InFlight(match before.phase {
                PictureBodyPhase::TakingOff => FlightPhase::TakingOff,
                PictureBodyPhase::Rising => FlightPhase::Rising,
            });
            if let Some(id) = &self.tail_case {
                let seed = vectors.iter().find(|vector| vector.id == *id).unwrap();
                restored.flight_tail = seed.flight_tail.as_ref().map(FlightTailRow::shape);
                restored.curl = curl(seed, seed.ridley.as_ref().unwrap());
                let her = seed.hitter_her();
                return restored.run(
                    boss_ceres_ridley::RidleyFrame {
                        picture: self.inputs(),
                        ..frame(0)
                    },
                    seed.random(),
                    her.as_ref(),
                    seed.fight(),
                    &ridley_content(&stand_in_content()),
                    &Rules::default(),
                );
            }
            picture_turn(restored, self.inputs())
        }

        fn check_body_after(&self, run: &RidleyRun) {
            let after = self.body_after.unwrap();
            let ridley = run.ridley;
            let (figure, runs) =
                figure_sequence_cells(TIMED_TAKEOFF_FIGURES, ridley.takeoff.unwrap());
            assert_eq!(
                (figure, runs),
                (Some(after.figure), Some(after.runs)),
                "{}: take-off figure",
                self.id
            );
            let phase = match after.phase {
                PictureBodyPhase::TakingOff => FlightPhase::TakingOff,
                PictureBodyPhase::Rising => FlightPhase::Rising,
            };
            assert_eq!(
                ridley.phase,
                RidleyPhase::InFlight(phase),
                "{}: phase",
                self.id
            );
            assert_eq!(
                ridley.position,
                point(after.anchor),
                "{}: body anchor",
                self.id
            );
            assert_eq!(
                ridley.speed,
                picture_speed(after.speed),
                "{}: body speed",
                self.id
            );
            if let Some(places) = self.places {
                assert_eq!(
                    run.tail_places,
                    Some(places.map(point)),
                    "{}: moved tail places",
                    self.id
                );
                let tail = ridley.flight_tail.unwrap();
                let [.., penultimate, last] = tail.segments;
                assert_eq!(
                    Some([penultimate.angle.0, last.angle.0]),
                    self.angles,
                    "{}: moved tail angles",
                    self.id
                );
            }
        }

        fn check_order(&self, mut ridley: CeresRidley) {
            let Some(stage) = self.stage else {
                return;
            };
            ridley.phase = match stage {
                PictureStage::BeforeOrdinaryAnimation => RidleyPhase::Holding,
                PictureStage::Fighting => RidleyPhase::BodyFading { sets: 0, runs: 0 },
                PictureStage::Getaway => RidleyPhase::Getaway(GetawayState::WarningReady),
            };
            let door = if self.attached.unwrap() {
                Phase::Slammed
            } else {
                Phase::Standing
            };
            let order = ridley.picture_order(door.drawn_with_ridley());
            let actual = PictureOrderRow {
                opportunity: match order.opportunity {
                    RoomLayerOrder::Before => OrderOpportunity::Before,
                    RoomLayerOrder::After => OrderOpportunity::After,
                },
                baby: order.baby,
                door: order.door,
            };
            assert_eq!(Some(actual), self.order, "{}: draw order", self.id);
        }
    }

    fn picture_turn(ridley: CeresRidley, picture: FightPictureInputs) -> RidleyRun {
        ridley.run(
            boss_ceres_ridley::RidleyFrame {
                picture,
                ..frame(0)
            },
            0,
            None,
            Fight {
                her_energy: 99,
                hits_taken: 0,
            },
            &ridley_content(&stand_in_content()),
            &Rules::default(),
        )
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct FightRibsVector {
        id: String,
        source: Source,
        entries: Vec<String>,
        row: Option<String>,
        frame: Option<u16>,
        turn: RibTurn,
        before: Option<(RibBeat, u8)>,
        after: (RibBeat, u8),
        reveal: Option<RibLook>,
    }

    #[derive(Deserialize)]
    enum RibTurn {
        Placed,
        Hold,
        Animate,
    }

    fn saved_ribs((beat, remaining): (RibBeat, u8)) -> FightRibs {
        FightRibs {
            beat,
            remaining: RibWait::new(remaining).unwrap(),
        }
    }

    impl FightRibsVector {
        fn check(&self) {
            let oracle = matches!(self.source, Source::Oracle);
            assert_eq!(self.row.is_some(), oracle, "{}: source row", self.id);
            assert_eq!(self.frame.is_some(), oracle, "{}: source frame", self.id);
            let mut ridley = CeresRidley::place(false).unwrap().ridley;
            let (ribs, reveal) = match self.turn {
                RibTurn::Placed => {
                    assert!(
                        self.before.is_none(),
                        "{}: placement has no prior clock",
                        self.id
                    );
                    (ridley.ribs, None)
                }
                RibTurn::Hold | RibTurn::Animate => {
                    ridley.ribs = saved_ribs(self.before.unwrap());
                    if matches!(self.turn, RibTurn::Animate) {
                        ridley.phase = RidleyPhase::BodyFading { sets: 0, runs: 0 };
                    }
                    let run = picture_turn(ridley, FightPictureInputs::LEFT_UNHURT);
                    (run.ridley.ribs, run.rib_reveal)
                }
            };
            assert_eq!(ribs, saved_ribs(self.after), "{}: clock", self.id);
            assert_eq!(reveal, self.reveal, "{}: reveal", self.id);
        }
    }

    #[test]
    fn ribs_clock_round_trips_and_checks_its_wait() {
        let saved = saved_ribs((RibBeat::Closing, 1));
        let serialized = toml::to_string(&saved).unwrap();
        assert_eq!(toml::from_str::<FightRibs>(&serialized).unwrap(), saved);
        for invalid in [
            "beat = 'Closing'\nremaining = 0",
            "beat = 'Closing'\nremaining = 21",
            "beat = 'Closing'",
            "remaining = 1",
        ] {
            assert!(toml::from_str::<FightRibs>(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn first_ordinary_picture_starts_with_body_fade() {
        let mut ridley = CeresRidley::place(false).unwrap().ridley;
        ridley.phase = RidleyPhase::Awake;
        let eye = picture_turn(ridley, FightPictureInputs::LEFT_UNHURT);
        assert!(eye.parts.is_none());
        assert_eq!(eye.rib_reveal, None);
        assert_eq!(eye.ridley.ribs, FightRibs::PLACED);
        assert_eq!(eye.ridley.wings, FightWings::PLACED);
        assert_eq!(
            eye.ridley.picture_order(true).opportunity,
            RoomLayerOrder::Before
        );
        let body = picture_turn(eye.ridley, FightPictureInputs::LEFT_UNHURT);
        assert_eq!(body.rib_reveal, Some(RibLook::Closed));
        assert_eq!(body.ridley.ribs, saved_ribs((RibBeat::Opening, 20)));
        assert_eq!(body.ridley.wings, FightWings::PLACED);
        let parts = body.parts.unwrap();
        assert_eq!(parts.wing.unwrap().anchor, ridley.position);
        assert_eq!(parts.tail, body.ridley.tail().map(TailIntent::resting));
        assert_eq!(
            body.ridley.picture_order(true).opportunity,
            RoomLayerOrder::After
        );
    }

    #[test]
    fn restored_wings_use_bounded_speed_and_shots_refresh_before_colours() {
        let mut ridley = CeresRidley::place(false).unwrap().ridley;
        let saved = saved_wings([7, 3]);
        let serialized = toml::to_string(&saved).unwrap();
        ridley.wings = toml::from_str(&serialized).unwrap();
        ridley.phase = RidleyPhase::InFlight(FlightPhase::Rising);
        ridley.position = point([224, 120]);
        ridley.speed = picture_speed([447, 16]);
        let run = picture_turn(
            ridley,
            FightPictureInputs {
                flash: FightFlash {
                    remaining: 0,
                    accepted_shots: 2,
                    half: FlashHalf::Second,
                },
                ..FightPictureInputs::LEFT_UNHURT
            },
        );
        assert_eq!(run.bound, Some(Bound::Right));
        assert_eq!(run.ridley.speed, picture_speed([0, 0]));
        assert_eq!(run.ridley.wings, saved);
        assert_eq!(run.hurt_refresh, Some(14));
        assert_eq!(run.parts.unwrap().colours, FightColours::Hurt);
    }

    #[test]
    fn wing_save_rejects_out_of_range_or_incomplete_clocks() {
        for invalid in [
            "beat = 10\nremaining = 32",
            "beat = 9\nremaining = 33",
            "beat = 9",
            "remaining = 32",
        ] {
            assert!(toml::from_str::<FightWings>(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn restored_fight_advances_the_saved_beat() {
        let mut ridley = CeresRidley::place(false).unwrap().ridley;
        ridley.phase = RidleyPhase::InFlight(FlightPhase::Rising);
        ridley.position = point([186, 120]);
        ridley.speed = picture_speed([0, -352]);
        ridley.wings = saved_wings([9, 0]);

        let mut restored = ridley;
        let serialized = toml::to_string(&ridley.wings).unwrap();
        restored.wings = toml::from_str(&serialized).unwrap();
        let picture = FightPictureInputs::LEFT_UNHURT;
        let run = picture_turn(restored, picture);
        assert_eq!(run, picture_turn(ridley, picture));
        assert_eq!(run.ridley.wings, saved_wings([0, 32]));
        assert_eq!(run.parts.unwrap().wing.unwrap().beat, run.ridley.wings.beat);
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
        let file = vector_file();
        file.check_tilting_room();
        assert_eq!(file.fight_picture.len(), 24);
        for vector in &file.fight_picture {
            vector.source.check(|| vector.check(&file.vector));
        }
        assert_eq!(file.fight_ribs.len(), 12);
        for vector in &file.fight_ribs {
            vector.source.check(|| vector.check());
        }
        let vectors = file.vector;
        assert_eq!(vectors.len(), 384);
        assert_eq!(
            vectors
                .iter()
                .filter(|vector| vector.warning_after.is_some())
                .count(),
            30
        );
        assert_eq!(
            vectors
                .iter()
                .filter(|vector| vector.getaway_after.is_some())
                .count(),
            12
        );
        let failures: Vec<String> = vectors
            .iter()
            .filter_map(|vector| vector.source.check(|| check(vector)))
            .collect();
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn escape_activation_does_not_republish_after_restore() {
        warning_vectors::escape_activation_survives_restore();
    }

    #[test]
    fn every_entry_checked_by_vectors_has_one() {
        let file = vector_file();
        let exercised: Vec<_> = file
            .labelled_entries()
            .flat_map(|(id, entries)| {
                assert!(!entries.is_empty(), "{id} names no behaviours");
                entries.iter().map(|name| entry(name))
            })
            .collect();
        for (id, behaviour) in catalogue() {
            assert_eq!(behaviour.id, id);
            let exercised = exercised.contains(&id);
            let by_vectors = behaviour.checked == Checked::ByVectors;
            assert_eq!(exercised, by_vectors, "{id:?}");
        }
    }
}
