//! Every vector of `scenarios/vectors/screens-intro.toml`, run through the model as one frame,
//! both by `step` and by `begin_frame` then `finish`, which must agree, and the catalogue
//! checked against them. The file's header gives its format; a key the format
//! doesn't name fails the read.

#[cfg(test)]
mod vectors {
    use core::fmt::Debug;
    use core::num::Wrapping;
    use std::collections::BTreeMap;

    use enum_map::EnumMap;
    use screens_intro::{
        Dissolve, DissolveSets, Equipment, FightReply, FlashbackRequest, FrameInputs,
        IndexedColour, IntroContent, IntroEvent, Letter, MusicRequest, PictureLayout, PictureName,
        Report, Rules, Shows, Signal, Step, TextTiles, TrackReply, Tune, begin_frame, catalogue,
        step,
    };
    use serde::Deserialize;
    use types::intro::{
        Baby, Blanking, Brightness, BrightnessLevel, Caret, CaretScript, ClickTurn, ColourSet,
        EyesScript, FadeFrames, FadingColours, FightVisibility, Glow, Glows, IntroStage,
        IntroState, PageScript, Progress, ScriptPlace, StageWait, StepAt, TextPlace,
    };
    use types::{Checked, Colour, PALETTE_SLOTS, Pair, Palette};

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/screens-intro.toml");
    /// The vectors write colours in hex.
    const HEX: u32 = 16;
    /// No colour at all: the full colours a vector leaves out.
    const ALL_BLACK: Palette = Palette([Colour::BLACK; PALETTE_SLOTS]);

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
            reason = "read only to refuse a source the format doesn't name"
        )]
        source: Source,
        entries: Vec<String>,
        rules: Option<RulesPart>,
        before: Part,
        inputs: Option<Inputs>,
        after: Part,
        output: Option<Output>,
        gone: Option<Vec<Gone>>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "lowercase")]
    enum Source {
        Recorded,
        Constructed,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct RulesPart {
        rhythm: u16,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Part {
        stage: Option<IntroStage>,
        dissolve: Option<u16>,
        wait: Option<u16>,
        colours: Option<BTreeMap<String, [[u16; 3]; 2]>>,
        height: Option<i16>,
        fight_visible: Option<bool>,
        baby: Option<Placed>,
        page: Option<Placed>,
        eyes: Option<Placed>,
        caret: Option<Placed>,
        glows: Option<Vec<[u16; 4]>>,
        click: Option<bool>,
        level: Option<u8>,
        screen: Option<Blanking>,
        count: Option<u16>,
    }

    /// A script's place, with the place of the picture it moves (the baby's y, the caret's x and
    /// y).
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Placed {
        x: Option<u16>,
        y: Option<u16>,
        script: Option<String>,
        step: Option<i64>,
        hold: Option<u16>,
        count: Option<u16>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Inputs {
        press: Option<bool>,
        track_waits: Option<bool>,
        return_to_text: Option<bool>,
        scene: Option<Scene>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Scene {
        scripts: Option<BTreeMap<String, Vec<StepPart>>>,
        sets: Option<Sets>,
        full: Option<BTreeMap<String, [u16; 3]>>,
        caret_colours: Option<BTreeMap<String, String>>,
        caret_home: Option<[u16; 2]>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Sets {
        to_scene: Option<SetsOfDissolve>,
        to_text: Option<SetsOfDissolve>,
        into_fight: Option<SetsOfDissolve>,
        out_of_fight: Option<SetsOfDissolve>,
        text: Option<Vec<u8>>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SetsOfDissolve {
        dims: Vec<u8>,
        brightens: Vec<u8>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct StepPart {
        picture: Option<String>,
        letter: Option<String>,
        column: Option<u16>,
        row: Option<u16>,
        hold: Option<u16>,
        back: Option<usize>,
        repeat: Option<usize>,
        times: Option<u16>,
        signal: Option<String>,
        end: Option<bool>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Output {
        shown: Option<BTreeMap<String, String>>,
        typed: Option<Vec<Typed>>,
        clicks: Option<usize>,
        cleared: Option<bool>,
        caret_shown: Option<bool>,
        equipment_returned: Option<bool>,
        equipment_lent: Option<bool>,
        music: Option<Vec<String>>,
        encounter: Option<Encounter>,
        picture: Option<Picture>,
        fight_lights: Option<bool>,
        text_updates: Option<bool>,
    }

    /// The flashback request a vector expects.
    #[derive(Deserialize, Clone, Copy)]
    #[serde(rename_all = "lowercase")]
    enum Encounter {
        Start,
        End,
    }

    /// The picture layout a vector expects.
    #[derive(Deserialize, Clone, Copy)]
    #[serde(rename_all = "lowercase")]
    enum Picture {
        Blend,
        Fight,
        Text,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct Typed {
        letter: String,
        column: u16,
        row: u16,
    }

    #[derive(Deserialize, Clone, Copy)]
    #[serde(rename_all = "lowercase")]
    enum Gone {
        Baby,
        Page,
        Eyes,
        Caret,
    }

    /// A script of the scene, by the name the vectors give it.
    enum Named {
        Page(PageScript),
        Eyes(EyesScript),
        Baby,
        Caret(CaretScript),
    }

    fn named(name: &str) -> Named {
        match name {
            "page1" => Named::Page(PageScript::One),
            "page2" => Named::Page(PageScript::Two),
            "page5" => Named::Page(PageScript::Five),
            "page6" => Named::Page(PageScript::Six),
            "eyes" => Named::Eyes(EyesScript::Blinking),
            "eyes_six" => Named::Eyes(EyesScript::Staring),
            "baby" => Named::Baby,
            "caret_steady" => Named::Caret(CaretScript::Steady),
            "caret_blink" => Named::Caret(CaretScript::Blinking),
            other => panic!("unknown script {other}"),
        }
    }

    fn page_script(name: Option<&String>) -> PageScript {
        match named(name.expect("a page names its script")) {
            Named::Page(page) => page,
            Named::Eyes(_) | Named::Baby | Named::Caret(_) => panic!("not a page's script"),
        }
    }

    fn eyes_script(name: Option<&String>) -> EyesScript {
        match named(name.expect("the eyes name their script")) {
            Named::Eyes(eyes) => eyes,
            Named::Page(_) | Named::Baby | Named::Caret(_) => panic!("not the eyes' script"),
        }
    }

    fn caret_script(name: Option<&String>) -> CaretScript {
        match named(name.expect("the caret names its script")) {
            Named::Caret(caret) => caret,
            Named::Page(_) | Named::Baby | Named::Eyes(_) => panic!("not the caret's script"),
        }
    }

    fn index(key: &str) -> u8 {
        key.parse().expect("a colour index is 0 to 255")
    }

    fn step_at(step: i64) -> StepAt {
        if step < 0 {
            StepAt::Start
        } else {
            StepAt::Step(usize::try_from(step).unwrap())
        }
    }

    fn progress(placed: &Placed) -> Progress {
        Progress {
            step: step_at(placed.step.expect("a script's step")),
            hold: placed.hold.expect("a script's hold"),
            repeat_left: placed.count,
        }
    }

    fn glow([column, row, next, left]: [u16; 4]) -> Glow {
        Glow {
            at: TextPlace { column, row },
            next: ColourSet::new(u8::try_from(next).unwrap()).unwrap(),
            left,
        }
    }

    fn click(on: bool) -> ClickTurn {
        if on { ClickTurn::On } else { ClickTurn::Off }
    }

    fn fight(visible: bool) -> FightVisibility {
        if visible {
            FightVisibility::InFront
        } else {
            FightVisibility::Hidden
        }
    }

    fn colours(part: &Part) -> FadingColours {
        let mut colours = FadingColours::still_black();
        for (key, [levels, steps]) in part.colours.iter().flatten() {
            let colour = colours.colour_mut(index(key)).unwrap();
            colour.levels = *levels;
            colour.steps = *steps;
        }
        colours
    }

    /// The state a vector's `before` builds; a part it leaves out is absent or at its rest.
    fn build(part: &Part) -> IntroState {
        IntroState {
            stage: part.stage.expect("before names the stage"),
            wait: StageWait::new(part.wait.unwrap_or(0)).unwrap(),
            colours: colours(part),
            dissolve: part.dissolve.unwrap_or(0),
            height: part.height.unwrap_or(0),
            fight: fight(part.fight_visible.unwrap_or(false)),
            baby: part.baby.as_ref().map(|baby| Baby {
                y: Wrapping(baby.y.expect("the baby's y")),
                progress: progress(baby),
            }),
            page: part.page.as_ref().map(|page| ScriptPlace {
                script: page_script(page.script.as_ref()),
                progress: progress(page),
            }),
            eyes: part.eyes.as_ref().map(|eyes| ScriptPlace {
                script: eyes_script(eyes.script.as_ref()),
                progress: progress(eyes),
            }),
            caret: part.caret.as_ref().map(|caret| Caret {
                place: Pair {
                    x: Wrapping(caret.x.expect("the caret's x")),
                    y: Wrapping(caret.y.expect("the caret's y")),
                },
                script: ScriptPlace {
                    script: caret_script(caret.script.as_ref()),
                    progress: progress(caret),
                },
            }),
            glows: Glows::new(part.glows.iter().flatten().copied().map(glow).collect()).unwrap(),
            click: click(part.click.unwrap_or(false)),
            brightness: Brightness {
                level: part.level.map_or(BrightnessLevel::FULL, |level| {
                    BrightnessLevel::new(level).unwrap()
                }),
                blanking: part.screen.unwrap_or(Blanking::Shown),
            },
            fade: FadeFrames::new(part.count.unwrap_or(0)).unwrap(),
        }
    }

    fn signal(name: &str) -> Signal {
        match name {
            "page5" => Signal::PageFive,
            "wait" => Signal::WaitForPress,
            "fight" => Signal::WaitForPressThenFight,
            "wait_discovery" => Signal::WaitForDiscovery,
            "blink" => Signal::CaretBlinks,
            "finish" => Signal::Finish,
            other => panic!("unknown signal {other}"),
        }
    }

    fn shows(part: &StepPart) -> Shows {
        if let Some(picture) = &part.picture {
            return Shows::Image(PictureName(picture.clone()));
        }
        part.letter.as_ref().map_or(Shows::Nothing, |letter| {
            Shows::Text(Letter {
                glyph: letter.chars().next().expect("a letter"),
                at: TextPlace {
                    column: part.column.expect("a letter's column"),
                    row: part.row.expect("a letter's row"),
                },
            })
        })
    }

    fn script_step(part: &StepPart) -> Step {
        if let Some(to) = part.back {
            return Step::Back(to);
        }
        if let Some(from) = part.repeat {
            let times = part.times.expect("a repeat's times");
            return Step::Repeat { from, times };
        }
        if let Some(name) = &part.signal {
            return Step::Hand(signal(name));
        }
        if part.end == Some(true) {
            return Step::End;
        }
        Step::Show {
            shows: shows(part),
            hold: part.hold.expect("a step's hold"),
        }
    }

    fn no_sets() -> DissolveSets {
        DissolveSets {
            dims: Vec::new(),
            brightens: Vec::new(),
        }
    }

    fn sets(part: Option<&SetsOfDissolve>) -> DissolveSets {
        part.map_or_else(no_sets, |part| DissolveSets {
            dims: part.dims.clone(),
            brightens: part.brightens.clone(),
        })
    }

    fn add_script(content: &mut IntroContent, name: &str, steps: &[StepPart]) {
        let script = steps.iter().map(script_step).collect();
        match named(name) {
            Named::Page(page) => content.pages[page] = script,
            Named::Eyes(eyes) => content.eyes[eyes] = script,
            Named::Baby => content.baby = script,
            Named::Caret(caret) => content.caret[caret] = script,
        }
    }

    fn colour(hex: &str) -> Colour {
        Colour::new(u16::from_str_radix(hex, HEX).unwrap()).unwrap()
    }

    fn full(scene: &Scene) -> Palette {
        let mut full = ALL_BLACK;
        for (key, channels) in scene.full.iter().flatten() {
            full.0[usize::from(index(key))] = Colour::from_channels(*channels);
        }
        full
    }

    /// The scene's content a vector's inputs give; what they leave out is empty.
    fn content(scene: Option<&Scene>) -> IntroContent {
        let mut content = IntroContent {
            pages: EnumMap::default(),
            eyes: EnumMap::default(),
            baby: Vec::new(),
            caret: EnumMap::default(),
            full: ALL_BLACK,
            sets: EnumMap::from_fn(|_| no_sets()),
            text: Vec::new(),
            caret_colours: Vec::new(),
            caret_home: Pair {
                x: Wrapping(0),
                y: Wrapping(0),
            },
        };
        let Some(scene) = scene else {
            return content;
        };
        for (name, steps) in scene.scripts.iter().flatten() {
            add_script(&mut content, name, steps);
        }
        let given = scene.sets.as_ref();
        content.sets[Dissolve::ToScene] = sets(given.and_then(|sets| sets.to_scene.as_ref()));
        content.sets[Dissolve::ToText] = sets(given.and_then(|sets| sets.to_text.as_ref()));
        content.sets[Dissolve::IntoFight] = sets(given.and_then(|sets| sets.into_fight.as_ref()));
        content.sets[Dissolve::OutOfFight] =
            sets(given.and_then(|sets| sets.out_of_fight.as_ref()));
        content.text = given.and_then(|sets| sets.text.clone()).unwrap_or_default();
        content.full = full(scene);
        content.caret_colours = scene
            .caret_colours
            .iter()
            .flatten()
            .map(|(key, hex)| IndexedColour {
                index: index(key),
                colour: colour(hex),
            })
            .collect();
        if let Some([x, y]) = scene.caret_home {
            content.caret_home = Pair {
                x: Wrapping(x),
                y: Wrapping(y),
            };
        }
        content
    }

    /// The failures of one vector.
    struct Failures<'a> {
        id: &'a str,
        found: Vec<String>,
    }

    impl Failures<'_> {
        fn check<T: PartialEq + Debug + Copy>(
            &mut self,
            what: &str,
            expected: Option<T>,
            actual: T,
        ) {
            if let Some(expected) = expected
                && expected != actual
            {
                self.found.push(format!(
                    "{}: {what}: expected {expected:?}, got {actual:?}",
                    self.id
                ));
            }
        }

        fn missing(&mut self, what: &str) {
            self.found
                .push(format!("{}: {what}: expected there, absent", self.id));
        }

        fn script(&mut self, what: &str, expected: &Placed, actual: Progress) {
            let step = expected.step.map(step_at);
            self.check(&format!("{what} step"), step, actual.step);
            self.check(&format!("{what} hold"), expected.hold, actual.hold);
            self.check(
                &format!("{what} count"),
                Some(expected.count),
                actual.repeat_left,
            );
        }
    }

    fn check_scripts(failures: &mut Failures, after: &Part, state: &IntroState) {
        if let Some(expected) = &after.baby {
            match state.baby {
                Some(baby) => {
                    failures.check("baby y", expected.y.map(Wrapping), baby.y);
                    failures.script("baby", expected, baby.progress);
                }
                None => failures.missing("baby"),
            }
        }
        if let Some(expected) = &after.page {
            match state.page {
                Some(page) => {
                    let script = expected.script.as_ref().map(|name| page_script(Some(name)));
                    failures.check("page script", script, page.script);
                    failures.script("page", expected, page.progress);
                }
                None => failures.missing("page"),
            }
        }
        if let Some(expected) = &after.eyes {
            match state.eyes {
                Some(eyes) => {
                    let script = expected.script.as_ref().map(|name| eyes_script(Some(name)));
                    failures.check("eyes script", script, eyes.script);
                    failures.script("eyes", expected, eyes.progress);
                }
                None => failures.missing("eyes"),
            }
        }
    }

    fn check_caret(failures: &mut Failures, after: &Part, state: &IntroState) {
        let Some(expected) = &after.caret else {
            return;
        };
        let Some(caret) = state.caret else {
            failures.missing("caret");
            return;
        };
        failures.check("caret x", expected.x.map(Wrapping), caret.place.x);
        failures.check("caret y", expected.y.map(Wrapping), caret.place.y);
        let script = expected
            .script
            .as_ref()
            .map(|name| caret_script(Some(name)));
        failures.check("caret script", script, caret.script.script);
        failures.script("caret", expected, caret.script.progress);
    }

    fn check_colours(failures: &mut Failures, after: &Part, state: &IntroState) {
        for (key, [levels, steps]) in after.colours.iter().flatten() {
            let colour = state.colours.0[usize::from(index(key))];
            failures.check(
                &format!("colour {key} levels"),
                Some(*levels),
                colour.levels,
            );
            failures.check(&format!("colour {key} steps"), Some(*steps), colour.steps);
        }
    }

    fn check_after(failures: &mut Failures, after: &Part, state: &IntroState) {
        failures.check("stage", after.stage, state.stage);
        failures.check("dissolve", after.dissolve, state.dissolve);
        failures.check("wait", after.wait, state.wait.frames());
        failures.check("height", after.height, state.height);
        failures.check("fight visible", after.fight_visible.map(fight), state.fight);
        check_colours(failures, after, state);
        check_scripts(failures, after, state);
        check_caret(failures, after, state);
        let glows: Option<Vec<Glow>> =
            (after.glows.as_ref()).map(|glows| glows.iter().copied().map(glow).collect());
        failures.check("glows", glows.as_deref(), state.glows.as_slice());
        failures.check("click", after.click.map(click), state.click);
        let level = after
            .level
            .map(|level| BrightnessLevel::new(level).unwrap());
        failures.check("level", level, state.brightness.level);
        failures.check("screen", after.screen, state.brightness.blanking);
        failures.check("fade count", after.count, state.fade.frames());
    }

    fn check_gone(failures: &mut Failures, gone: &[Gone], state: &IntroState) {
        for part in gone {
            let (name, there) = match part {
                Gone::Baby => ("baby", state.baby.is_some()),
                Gone::Page => ("page", state.page.is_some()),
                Gone::Eyes => ("eyes", state.eyes.is_some()),
                Gone::Caret => ("caret", state.caret.is_some()),
            };
            failures.check(&format!("{name} gone"), Some(false), there);
        }
    }

    fn check_shown(failures: &mut Failures, shown: &BTreeMap<String, String>, report: &Report) {
        if shown.is_empty() {
            failures.check("colours taken", Some(false), report.colours.is_some());
            return;
        }
        let Some(palette) = &report.colours else {
            failures.missing("colours shown");
            return;
        };
        for (key, hex) in shown {
            let actual = palette.0[usize::from(index(key))];
            failures.check(&format!("shown {key}"), Some(colour(hex)), actual);
        }
    }

    fn happened(report: &Report, event: IntroEvent) -> bool {
        report.events.contains(&event)
    }

    fn check_output(failures: &mut Failures, output: &Output, report: &Report) {
        if let Some(shown) = &output.shown {
            check_shown(failures, shown, report);
        }
        let typed: Vec<Typed> = report
            .typed
            .iter()
            .map(|letter| Typed {
                letter: letter.glyph.to_string(),
                column: letter.at.column,
                row: letter.at.row,
            })
            .collect();
        failures.check("typed", output.typed.as_ref(), &typed);
        let clicks = report
            .events
            .iter()
            .filter(|&&event| event == IntroEvent::Click);
        failures.check("clicks", output.clicks, clicks.count());
        let cleared = happened(report, IntroEvent::TextCleared);
        failures.check("cleared", output.cleared, cleared);
        failures.check("caret shown", output.caret_shown, report.caret.is_some());
        let returned = happened(report, IntroEvent::EquipmentSet(Equipment::RETURNED));
        failures.check("equipment returned", output.equipment_returned, returned);
        let lent = happened(report, IntroEvent::EquipmentSet(Equipment::LENT));
        failures.check("equipment lent", output.equipment_lent, lent);
        let music: Vec<String> = report.music.iter().map(|&request| asked(request)).collect();
        failures.check("music", output.music.as_ref(), &music);
        check_flashback(failures, output, report);
    }

    /// The flashback's outputs: its requests, the picture layout, the lights and the text tiles.
    fn check_flashback(failures: &mut Failures, output: &Output, report: &Report) {
        let requests: Vec<FlashbackRequest> = report
            .events
            .iter()
            .filter_map(|event| {
                if let IntroEvent::Flashback(request) = event {
                    Some(*request)
                } else {
                    None
                }
            })
            .collect();
        let expected = output.encounter.map(|encounter| {
            vec![match encounter {
                Encounter::Start => FlashbackRequest::StartZebesFight,
                Encounter::End => FlashbackRequest::HideFightSamusAndShots,
            }]
        });
        failures.check("encounter", expected.as_ref(), &requests);
        let layouts: Vec<PictureLayout> = report
            .events
            .iter()
            .filter_map(|event| {
                if let IntroEvent::Layout(layout) = event {
                    Some(*layout)
                } else {
                    None
                }
            })
            .collect();
        let expected = output.picture.map(|picture| {
            vec![match picture {
                Picture::Blend => PictureLayout::SplitBlend,
                Picture::Fight => PictureLayout::PlainFight,
                Picture::Text => PictureLayout::PlainText,
            }]
        });
        failures.check("picture", expected.as_ref(), &layouts);
        let lights = happened(report, IntroEvent::FlashingLightsBegin);
        failures.check("fight lights", output.fight_lights, lights);
        let tiles = report.events.iter().find_map(|event| {
            if let IntroEvent::TextUpdates(tiles) = event {
                Some(*tiles == TextTiles::Resumed)
            } else {
                None
            }
        });
        failures.check("text updates", output.text_updates.map(Some), tiles);
    }

    /// A music request as the vectors write it.
    fn asked(request: MusicRequest) -> String {
        match request {
            MusicRequest::Stop => "stop".to_owned(),
            MusicRequest::Load(Tune::OpeningFirst) => "load first".to_owned(),
            MusicRequest::Load(Tune::OpeningSecond) => "load second".to_owned(),
            MusicRequest::Load(Tune::Pages) => "load pages".to_owned(),
            MusicRequest::Play { after } => format!("play {after}"),
        }
    }

    fn rules(vector: &Vector) -> Rules {
        vector
            .rules
            .as_ref()
            .map_or(Rules::REFERENCE, |rules| Rules {
                rhythm: FadeFrames::new(rules.rhythm).unwrap(),
            })
    }

    /// One vector run as one frame: its failures.
    fn run(vector: &Vector) -> Vec<String> {
        let mut state = build(&vector.before);
        let inputs = vector.inputs.as_ref();
        let pressed = inputs.and_then(|inputs| inputs.press).unwrap_or(false);
        let waits = inputs
            .and_then(|inputs| inputs.track_waits)
            .unwrap_or(false);
        let returns = inputs
            .and_then(|inputs| inputs.return_to_text)
            .unwrap_or(false);
        let frame = FrameInputs {
            pressed,
            track: if waits {
                TrackReply::TrackWaits
            } else {
                TrackReply::NoneWaits
            },
            fight: if returns {
                FightReply::ReadyToReturn
            } else {
                FightReply::Fighting
            },
        };
        let scene = content(inputs.and_then(|inputs| inputs.scene.as_ref()));
        let mut split_state = state.clone();
        let report = step(&mut state, frame, &scene, rules(vector));
        let split_report = begin_frame(
            &mut split_state,
            frame.pressed,
            frame.track,
            &scene,
            rules(vector),
        )
        .finish(frame.fight);
        let mut failures = Failures {
            id: &vector.id,
            found: Vec::new(),
        };
        failures.check("two-part frame's state", Some(&state), &split_state);
        failures.check("two-part frame's report", Some(&report), &split_report);
        check_after(&mut failures, &vector.after, &state);
        check_gone(
            &mut failures,
            vector.gone.as_deref().unwrap_or_default(),
            &state,
        );
        if let Some(output) = &vector.output {
            check_output(&mut failures, output, &report);
        }
        failures.found
    }

    fn vectors() -> Vec<Vector> {
        toml::from_str::<VectorFile>(VECTORS)
            .expect("the vector file reads")
            .vector
    }

    #[test]
    fn every_vector_passes() {
        let vectors = vectors();
        assert!(!vectors.is_empty());
        let failures: Vec<String> = vectors.iter().flat_map(run).collect();
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    /// Every entry the catalogue checks by vectors is exercised by one, and every vector names
    /// only the catalogue's entries.
    #[test]
    fn the_catalogue_matches_the_vectors() {
        let vectors = vectors();
        let names: Vec<String> = catalogue()
            .into_values()
            .map(|behaviour| format!("{:?}", behaviour.id))
            .collect();
        for vector in &vectors {
            for entry in &vector.entries {
                assert!(
                    names.contains(entry),
                    "{}: unknown entry {entry}",
                    vector.id
                );
            }
        }
        for behaviour in catalogue().into_values() {
            let name = format!("{:?}", behaviour.id);
            let exercised = vectors.iter().any(|vector| vector.entries.contains(&name));
            let by_vectors = behaviour.checked == Checked::ByVectors;
            assert!(!by_vectors || exercised, "{name} has no vector");
        }
    }
}
