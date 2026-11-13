//! Runs every vector of `scenarios/vectors/world-doors.toml` through the model, refusing any key
//! or name the file's header doesn't define, and checks the catalogue against the vectors.

#[cfg(test)]
mod vectors {
    use core::num::Wrapping;
    use std::collections::BTreeSet;

    use serde::Deserialize;
    use types::{Checked, Colour, Fixed, Heading, Pair, Parts, Position, Px, Set, fixed};
    use world_doors::{
        Activity, Arrival, Axis, CATALOGUE, COLOURS, ColourGroup, Cost, CostEvent, Doors, Exports,
        FadeTime, Fetching, Finishing, HerSound, Inputs, Palette, Passage, Phase, RoomCall,
        RoomSound, Rules, Scene, SlideScreen, WorkFrames, catalogue,
    };

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/world-doors.toml");

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VectorFile {
        vector: Vec<Vector>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Vector {
        id: String,
        #[expect(
            dead_code,
            reason = "read only to refuse a source the header doesn't name"
        )]
        source: Source,
        entries: Vec<String>,
        rules: Option<RulesRecord>,
        before: PhaseRecord,
        her: [String; 2],
        seen: [String; 2],
        camera: [String; 2],
        #[serde(default)]
        colours: Vec<ColourRecord>,
        #[serde(default)]
        inputs: InputsRecord,
        after: AfterRecord,
    }

    #[derive(Deserialize)]
    enum Source {
        #[serde(rename = "constructed")]
        Constructed,
    }

    /// The phase, its data, the arrival and the slide's frames, as `before` and `after` give them.
    #[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
    #[serde(deny_unknown_fields)]
    struct PhaseRecord {
        phase: PhaseName,
        asked: Option<bool>,
        t: Option<u16>,
        n: Option<u8>,
        started: Option<bool>,
        placed: Option<bool>,
        fetching: Option<FetchingName>,
        arrival: Option<ArrivalRecord>,
        slide: Option<u16>,
    }

    impl PhaseRecord {
        const fn named(phase: PhaseName) -> Self {
            Self {
                phase,
                asked: None,
                t: None,
                n: None,
                started: None,
                placed: None,
                fetching: None,
                arrival: None,
                slide: None,
            }
        }
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum PhaseName {
        Arriving,
        BeforeTheSlide,
        Brightening,
        Darkening,
        Landing,
        Music,
        Play,
        Quieting,
        Squaring,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum FetchingName {
        Contents,
        Done,
        Tiles,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
    #[serde(deny_unknown_fields)]
    struct ArrivalRecord {
        way: WayName,
        view: [String; 2],
        speed: Option<String>,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    enum WayName {
        Down,
        Left,
        Right,
        Up,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct RulesRecord {
        steps: Option<u8>,
        slide: Option<SlideRecord>,
        speeds: Option<SpeedsRecord>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SlideRecord {
        step: Option<u16>,
        across: Option<u16>,
        down_up: Option<u16>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SpeedsRecord {
        across: Option<String>,
        down_up: Option<String>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ColourRecord {
        now: String,
        goal: String,
        group: GroupName,
    }

    #[derive(Deserialize, Clone, Copy)]
    enum GroupName {
        Icon,
        Lit,
        Other,
        Timer,
        Visor,
    }

    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields)]
    struct InputsRecord {
        touched: Option<bool>,
        done: Option<bool>,
        sounds_sent: Option<bool>,
        music_ready: Option<bool>,
        timer: Option<bool>,
        items_dark: Option<bool>,
        doorway: Option<ArrivalRecord>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct AfterRecord {
        phase: PhaseName,
        asked: Option<bool>,
        t: Option<u16>,
        n: Option<u8>,
        started: Option<bool>,
        placed: Option<bool>,
        fetching: Option<FetchingName>,
        arrival: Option<ArrivalRecord>,
        slide: Option<u16>,
        her: [String; 2],
        seen: [String; 2],
        camera: [String; 2],
        #[serde(default)]
        colours: Vec<GoalRecord>,
        runs: Vec<RunName>,
        calls: Vec<CallName>,
        sounds: Vec<SoundName>,
        her_sounds: Vec<HerSoundName>,
        cost: Option<CostName>,
        finished: bool,
        restart: bool,
        screen: Option<ScreenName>,
        minimap_on: Option<bool>,
    }

    impl AfterRecord {
        fn phase_record(&self) -> PhaseRecord {
            PhaseRecord {
                phase: self.phase,
                asked: self.asked,
                t: self.t,
                n: self.n,
                started: self.started,
                placed: self.placed,
                fetching: self.fetching,
                arrival: self.arrival.clone(),
                slide: self.slide,
            }
        }
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct GoalRecord {
        now: String,
        goal: String,
    }

    #[derive(Deserialize, Clone, Copy)]
    #[serde(rename_all = "lowercase")]
    enum RunName {
        Blocks,
        Drawn,
        Enemies,
        Projectiles,
        Tiles,
        Timer,
    }

    #[derive(Deserialize, Clone, Copy)]
    enum CallName {
        LeaveRoom,
        DrawTopRow,
        BuildRoom,
        FetchTiles,
        FetchContents,
        AnimateTiles,
        PlayMusic,
        ReadLiquid,
    }

    #[derive(Deserialize, Clone, Copy)]
    enum SoundName {
        #[serde(rename = "held")]
        Held,
        #[serde(rename = "let through")]
        LetThrough,
        #[serde(rename = "silence")]
        Silence,
    }

    #[derive(Deserialize, Clone, Copy)]
    enum HerSoundName {
        #[serde(rename = "at the door")]
        AtTheDoor,
        #[serde(rename = "spin again")]
        SpinAgain,
    }

    #[derive(Deserialize, Clone, Copy)]
    enum CostName {
        BuildRoom,
        FadeStep,
        FetchContents,
        FetchTiles,
        Other,
        Play,
    }

    #[derive(Deserialize, Clone, Copy)]
    enum ScreenName {
        FrontAndSprites,
        Sprites,
    }

    fn hex(text: &str) -> u16 {
        assert_eq!(text.len(), 4, "a hex part is 4 digits: {text}");
        u16::from_str_radix(text, 16).unwrap_or_else(|_| panic!("not hex: {text}"))
    }

    fn ps(text: &str) -> Fixed {
        let (pixel, sub) = text
            .split_once('.')
            .unwrap_or_else(|| panic!("not P.S: {text}"));
        fixed(Wrapping(hex(pixel)), hex(sub))
    }

    fn ps_text(value: Fixed) -> String {
        format!("{:04X}.{:04X}", value.pixel().0, value.sub())
    }

    fn position(texts: &[String; 2]) -> Position {
        Pair {
            x: ps(&texts[0]),
            y: ps(&texts[1]),
        }
    }

    fn pixels(texts: &[String; 2]) -> Pair<Px> {
        Pair {
            x: Wrapping(hex(&texts[0])),
            y: Wrapping(hex(&texts[1])),
        }
    }

    fn colour(text: &str) -> Colour {
        Colour::new(hex(text)).unwrap_or_else(|| panic!("not a 15-bit colour: {text}"))
    }

    const fn way(name: WayName) -> Heading {
        match name {
            WayName::Down => Heading::Down,
            WayName::Left => Heading::Left,
            WayName::Right => Heading::Right,
            WayName::Up => Heading::Up,
        }
    }

    const fn way_name(way: Heading) -> WayName {
        match way {
            Heading::Down => WayName::Down,
            Heading::Left => WayName::Left,
            Heading::Right => WayName::Right,
            Heading::Up => WayName::Up,
        }
    }

    fn arrival(record: &ArrivalRecord) -> Arrival {
        Arrival {
            way: way(record.way),
            view: pixels(&record.view),
            speed: record.speed.as_deref().map(ps),
        }
    }

    fn arrival_record(arrival: Arrival) -> ArrivalRecord {
        ArrivalRecord {
            way: way_name(arrival.way),
            view: [
                format!("{:04X}", arrival.view.x.0),
                format!("{:04X}", arrival.view.y.0),
            ],
            speed: arrival.speed.map(ps_text),
        }
    }

    const fn fetching(name: FetchingName) -> Fetching {
        match name {
            FetchingName::Contents => Fetching::Contents,
            FetchingName::Done => Fetching::Done,
            FetchingName::Tiles => Fetching::Tiles,
        }
    }

    const fn fetching_name(fetching: Fetching) -> FetchingName {
        match fetching {
            Fetching::Contents => FetchingName::Contents,
            Fetching::Done => FetchingName::Done,
            Fetching::Tiles => FetchingName::Tiles,
        }
    }

    fn phase(record: &PhaseRecord) -> Option<Phase> {
        let needs = |name: &str| -> ! { panic!("{:?} needs {name}", record.phase) };
        let t = || FadeTime(record.t.unwrap_or_else(|| needs("t")));
        let n = || record.n.unwrap_or_else(|| needs("n"));
        let flag = |field: Option<bool>, name: &str| field.unwrap_or_else(|| needs(name));
        Some(match record.phase {
            PhaseName::Play => return None,
            PhaseName::Arriving => Phase::Arriving {
                placed: flag(record.placed, "placed"),
                fetching: fetching(record.fetching.unwrap_or_else(|| needs("fetching"))),
            },
            PhaseName::BeforeTheSlide => {
                Phase::BeforeTheSlide(WorkFrames::new(n()).expect("n is 0 to 2"))
            }
            PhaseName::Brightening => Phase::Brightening(t()),
            PhaseName::Darkening => Phase::Darkening(t()),
            PhaseName::Landing => Phase::Landing(WorkFrames::new(n()).expect("n is 0 to 1")),
            PhaseName::Music => Phase::Music {
                started: flag(record.started, "started"),
            },
            PhaseName::Quieting => Phase::Quieting {
                asked: flag(record.asked, "asked"),
            },
            PhaseName::Squaring => Phase::Squaring,
        })
    }

    /// The doors a record builds; the record must hold nothing its phase doesn't carry.
    fn doors(record: &PhaseRecord) -> Doors {
        let doors = phase(record).map_or(Doors::Play, |phase| {
            Doors::Passing(Passage {
                phase,
                arrival: record.arrival.as_ref().map(arrival),
                slide_frames: record.slide.unwrap_or(0),
            })
        });
        assert_eq!(&record_of(doors), record, "a key its phase doesn't carry");
        doors
    }

    fn phase_record(phase: Phase) -> PhaseRecord {
        match phase {
            Phase::Arriving { placed, fetching } => PhaseRecord {
                placed: Some(placed),
                fetching: Some(fetching_name(fetching)),
                ..PhaseRecord::named(PhaseName::Arriving)
            },
            Phase::BeforeTheSlide(n) => PhaseRecord {
                n: Some(n.get()),
                ..PhaseRecord::named(PhaseName::BeforeTheSlide)
            },
            Phase::Brightening(t) => PhaseRecord {
                t: Some(t.0),
                ..PhaseRecord::named(PhaseName::Brightening)
            },
            Phase::Darkening(t) => PhaseRecord {
                t: Some(t.0),
                ..PhaseRecord::named(PhaseName::Darkening)
            },
            Phase::Landing(n) => PhaseRecord {
                n: Some(n.get()),
                ..PhaseRecord::named(PhaseName::Landing)
            },
            Phase::Music { started } => PhaseRecord {
                started: Some(started),
                ..PhaseRecord::named(PhaseName::Music)
            },
            Phase::Quieting { asked } => PhaseRecord {
                asked: Some(asked),
                ..PhaseRecord::named(PhaseName::Quieting)
            },
            Phase::Squaring => PhaseRecord::named(PhaseName::Squaring),
        }
    }

    fn record_of(doors: Doors) -> PhaseRecord {
        match doors {
            Doors::Play => PhaseRecord::named(PhaseName::Play),
            Doors::Passing(passage) => PhaseRecord {
                arrival: passage.arrival.map(arrival_record),
                slide: (passage.slide_frames > 0).then_some(passage.slide_frames),
                ..phase_record(passage.phase)
            },
        }
    }

    fn rules(record: Option<&RulesRecord>) -> Rules {
        let mut rules = Rules::default();
        let Some(record) = record else {
            return rules;
        };
        if let Some(steps) = record.steps {
            rules.fade_steps = steps;
        }
        let slide = record.slide.as_ref();
        if let Some(step) = slide.and_then(|slide| slide.step) {
            rules.slide_step = Wrapping(step);
        }
        if let Some(across) = slide.and_then(|slide| slide.across) {
            rules.slide_length[Axis::Across] = across;
        }
        if let Some(down_up) = slide.and_then(|slide| slide.down_up) {
            rules.slide_length[Axis::DownUp] = down_up;
        }
        let speeds = record.speeds.as_ref();
        if let Some(across) = speeds.and_then(|speeds| speeds.across.as_deref()) {
            rules.carry_speed[Axis::Across] = ps(across);
        }
        if let Some(down_up) = speeds.and_then(|speeds| speeds.down_up.as_deref()) {
            rules.carry_speed[Axis::DownUp] = ps(down_up);
        }
        rules
    }

    const fn group(name: GroupName) -> ColourGroup {
        match name {
            GroupName::Icon => ColourGroup::Icon,
            GroupName::Lit => ColourGroup::Lit,
            GroupName::Other => ColourGroup::Other,
            GroupName::Timer => ColourGroup::Timer,
            GroupName::Visor => ColourGroup::Visor,
        }
    }

    /// The vector's colours, first of the 256; the rest black and headed for black, in no group.
    fn palette_and_groups(colours: &[ColourRecord]) -> (Palette, [ColourGroup; COLOURS]) {
        assert!(colours.len() <= COLOURS, "more colours than the screen has");
        let mut palette = Palette {
            screen: [Colour::BLACK; COLOURS],
            goal: [Colour::BLACK; COLOURS],
        };
        let mut groups = [ColourGroup::Other; COLOURS];
        for (i, record) in colours.iter().enumerate() {
            palette.screen[i] = colour(&record.now);
            palette.goal[i] = colour(&record.goal);
            groups[i] = group(record.group);
        }
        (palette, groups)
    }

    fn inputs(record: &InputsRecord, groups: [ColourGroup; COLOURS]) -> Inputs {
        Inputs {
            touched: record.touched.unwrap_or(false),
            work_done: record.done.unwrap_or(true),
            sounds_sent: record.sounds_sent.unwrap_or(true),
            music_ready: record.music_ready.unwrap_or(true),
            timer_running: record.timer.unwrap_or(false),
            items_dark: record.items_dark.unwrap_or(false),
            doorway: record.doorway.as_ref().map(arrival),
            groups,
        }
    }

    fn runs(names: &[RunName]) -> Set<Activity> {
        let mut set = Set::default();
        for &name in names {
            set[match name {
                RunName::Blocks => Activity::Blocks,
                RunName::Drawn => Activity::Drawn,
                RunName::Enemies => Activity::Enemies,
                RunName::Projectiles => Activity::Projectiles,
                RunName::Tiles => Activity::Tiles,
                RunName::Timer => Activity::Timer,
            }] = true;
        }
        set
    }

    fn call(names: &[CallName]) -> Option<RoomCall> {
        assert!(names.len() <= 1, "at most one room's call a frame");
        names.first().map(|&name| match name {
            CallName::LeaveRoom => RoomCall::LeaveRoom,
            CallName::DrawTopRow => RoomCall::DrawTopRow,
            CallName::BuildRoom => RoomCall::BuildRoom,
            CallName::FetchTiles => RoomCall::FetchTiles,
            CallName::FetchContents => RoomCall::FetchContents,
            CallName::AnimateTiles => RoomCall::AnimateTiles,
            CallName::PlayMusic => RoomCall::PlayMusic,
            CallName::ReadLiquid => RoomCall::ReadLiquid,
        })
    }

    fn sounds(names: &[SoundName]) -> Set<RoomSound> {
        let mut set = Set::default();
        for &name in names {
            set[match name {
                SoundName::Held => RoomSound::Held,
                SoundName::LetThrough => RoomSound::LetThrough,
                SoundName::Silence => RoomSound::Silence,
            }] = true;
        }
        set
    }

    fn her_sound(names: &[HerSoundName]) -> Option<HerSound> {
        assert!(
            names.len() <= 1,
            "at most one request to her sounds a frame"
        );
        names.first().map(|&name| match name {
            HerSoundName::AtTheDoor => HerSound::AtTheDoor,
            HerSoundName::SpinAgain => HerSound::SpinAgain,
        })
    }

    const fn cost(name: Option<CostName>) -> Cost {
        match name {
            None => Cost::OwnWait,
            Some(CostName::Play) => Cost::Play,
            Some(CostName::BuildRoom) => Cost::Event(CostEvent::BuildRoom),
            Some(CostName::FadeStep) => Cost::Event(CostEvent::FadeStep),
            Some(CostName::FetchContents) => Cost::Event(CostEvent::FetchContents),
            Some(CostName::FetchTiles) => Cost::Event(CostEvent::FetchTiles),
            Some(CostName::Other) => Cost::Event(CostEvent::Other),
        }
    }

    fn finishing(after: &AfterRecord) -> Finishing {
        let screen = after.screen.map(|name| match name {
            ScreenName::FrontAndSprites => SlideScreen::FrontAndSprites,
            ScreenName::Sprites => SlideScreen::Sprites,
        });
        match (after.finished, after.restart, screen) {
            (false, false, None) => Finishing::Overrun,
            (true, false, None) => Finishing::Finished,
            (true, true, Some(screen)) => Finishing::Restarts(screen),
            other => panic!("finished, restart and screen don't agree: {other:?}"),
        }
    }

    /// The vector's frame through the model: the doors it starts from and what it answers.
    fn frame(vector: &Vector) -> (Doors, Exports) {
        let doors_before = doors(&vector.before);
        let (palette, groups) = palette_and_groups(&vector.colours);
        let scene = Scene {
            her: position(&vector.her),
            seen: pixels(&vector.seen),
            camera: position(&vector.camera),
            palette,
        };
        let exports = doors_before.step(
            scene,
            &inputs(&vector.inputs, groups),
            &rules(vector.rules.as_ref()),
        );
        (doors_before, exports)
    }

    /// Each way the frame's answer differs from the vector's `after`.
    fn differences(vector: &Vector) -> Vec<String> {
        let (doors_before, exports) = frame(vector);
        let after = &vector.after;
        let expected_doors = doors(&after.phase_record());
        let mut found = Vec::new();
        let mut check = |field: &str, expected: String, actual: String| {
            if expected != actual {
                found.push(format!("{field}: expected {expected}, got {actual}"));
            }
        };
        let scene = exports.scene;
        let report = exports.report;
        check(
            "phase",
            format!("{:?}", record_of(expected_doors)),
            format!("{:?}", record_of(exports.doors)),
        );
        for (name, expected, actual) in [
            ("her", position(&after.her), scene.her),
            ("camera", position(&after.camera), scene.camera),
        ] {
            check(
                name,
                format!("{} {}", ps_text(expected.x), ps_text(expected.y)),
                format!("{} {}", ps_text(actual.x), ps_text(actual.y)),
            );
        }
        check(
            "seen",
            format!("{:?}", pixels(&after.seen)),
            format!("{:?}", scene.seen),
        );
        assert_eq!(
            after.colours.len(),
            vector.colours.len(),
            "after gives each colour before gives"
        );
        for (i, record) in after.colours.iter().enumerate() {
            check(
                &format!("colour {i}"),
                format!(
                    "{:04X} {:04X}",
                    colour(&record.now).bits(),
                    colour(&record.goal).bits()
                ),
                format!(
                    "{:04X} {:04X}",
                    scene.palette.screen[i].bits(),
                    scene.palette.goal[i].bits()
                ),
            );
        }
        check(
            "runs",
            format!("{:?}", runs(&after.runs)),
            format!("{:?}", report.runs),
        );
        check(
            "calls",
            format!("{:?}", call(&after.calls)),
            format!("{:?}", report.call),
        );
        check(
            "sounds",
            format!("{:?}", sounds(&after.sounds)),
            format!("{:?}", report.room_sounds),
        );
        check(
            "her_sounds",
            format!("{:?}", her_sound(&after.her_sounds)),
            format!("{:?}", report.her_sound),
        );
        check(
            "cost",
            format!("{:?}", cost(after.cost)),
            format!("{:?}", report.cost),
        );
        check(
            "finished",
            format!("{:?}", finishing(after)),
            format!("{:?}", report.finishing),
        );
        check(
            "minimap_on",
            format!("{:?}", after.minimap_on.unwrap_or(false)),
            format!("{:?}", report.minimap_on),
        );
        check(
            "passing",
            format!("{:?}", !matches!(doors_before, Doors::Play)),
            format!("{:?}", report.passing),
        );
        found
    }

    fn vectors() -> Vec<Vector> {
        toml::from_str::<VectorFile>(VECTORS)
            .expect("the vector file reads")
            .vector
    }

    fn entry_names() -> BTreeSet<String> {
        CATALOGUE
            .iter()
            .map(|behaviour| format!("{:?}", behaviour.id))
            .collect()
    }

    #[test]
    fn every_vector_passes() {
        let known = entry_names();
        let vectors = vectors();
        let mut ids = BTreeSet::new();
        let mut failures = Vec::new();
        for vector in &vectors {
            assert!(
                ids.insert(vector.id.clone()),
                "a repeated id: {}",
                vector.id
            );
            for entry in &vector.entries {
                assert!(
                    known.contains(entry),
                    "{}: an unknown entry {entry}",
                    vector.id
                );
            }
            for difference in differences(vector) {
                failures.push(format!("{}: {difference}", vector.id));
            }
        }
        assert!(
            failures.is_empty(),
            "{} of {} vectors' fields differ:\n{}",
            failures.len(),
            vectors.len(),
            failures.join("\n")
        );
    }

    #[test]
    fn the_catalogue_has_one_row_for_each_entry() {
        for (id, behaviour) in catalogue() {
            assert_eq!(behaviour.id, id, "a missing or repeated row");
        }
    }

    #[test]
    fn every_entry_checked_by_vectors_has_one() {
        let exercised: BTreeSet<String> = vectors()
            .into_iter()
            .flat_map(|vector| vector.entries)
            .collect();
        for behaviour in CATALOGUE {
            let name = format!("{:?}", behaviour.id);
            let by_vectors = behaviour.checked == Checked::ByVectors;
            assert_eq!(
                exercised.contains(&name),
                by_vectors,
                "{name}: checked by vectors is {by_vectors}"
            );
        }
    }
}
