//! Every vector of `screens-file-select.toml` run through the model, and the catalogue checked
//! against the vectors' entries.

#[cfg(test)]
mod tests {
    use screens_file_select::{
        BehaviourId, FrameEnd, FrameOfWork, FrameReport, MenuAction, MenuSound, Rules, SaveMemory,
        Unsupported, behaviour, catalogue, frame_end,
    };
    use serde::Deserialize;
    use types::file_select::{
        CopyAndClear, FadeWait, FileSelectStage, FileSelectState, HelmetTurn, LoadStep, MenuItem,
        MenuPace, MenuPhase, MenuState, Slot, TurnLeft, WayOut,
    };
    use types::power_on::{Level, ScreenLight};
    use types::{Set, set_of};

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/screens-file-select.toml");

    /// The load step the menu's `LoadEnding` phase stands for.
    const LOAD_ENDING_STEP: u8 = 4;
    /// Where a vector leaves out whether copy and clear are offered, the frame end doesn't read it.
    const UNSTATED_OFFER: CopyAndClear = CopyAndClear::NotOffered;
    /// Where a vector leaves out the fade wait, the frame end doesn't read it.
    const UNSTATED_WAIT: FadeWait = FadeWait(0);

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VectorFile {
        vector: Vec<Vector>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Vector {
        id: String,
        source: String,
        entries: Vec<String>,
        rules: Option<RulesOverride>,
        before: Before,
        finished: u8,
        pressed: Option<Vec<Vec<String>>>,
        input: Option<SaveAnswer>,
        after: After,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct RulesOverride {
        menu_pace: Option<u16>,
        helmet_turn: Option<u8>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Before {
        stage: String,
        load_step: Option<u8>,
        screen: (String, u8),
        cursor: Option<String>,
        offered: Option<bool>,
        fade_wait: Option<u16>,
        chosen: Option<String>,
        turn_left: Option<u8>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct After {
        stage: Option<String>,
        load_step: Option<u8>,
        screen: Option<(String, u8)>,
        cursor: Option<String>,
        offered: Option<bool>,
        fade_wait: Option<u16>,
        chosen: Option<String>,
        turn_left: Option<u8>,
        requests: Option<Vec<String>>,
        frame_goes_on: Option<bool>,
        unsupported: Option<bool>,
    }

    /// The files saved and the slot last chosen, as a vector's `input` states them.
    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields)]
    struct SaveAnswer {
        last_slot: Option<String>,
        files: Vec<String>,
    }

    impl SaveMemory for SaveAnswer {
        fn files(&self) -> Set<Slot> {
            self.files.iter().map(|name| (slot(name), true)).collect()
        }

        fn last_chosen(&self) -> Option<Slot> {
            self.last_slot.as_deref().map(slot)
        }
    }

    /// The State as the vectors' fields name it, each field present where the State has it.
    #[derive(Debug, PartialEq, Eq)]
    struct Flat {
        stage: String,
        load_step: Option<u8>,
        screen: (String, u8),
        cursor: Option<String>,
        offered: Option<bool>,
        fade_wait: Option<u16>,
        chosen: Option<String>,
        turn_left: Option<u8>,
    }

    fn slot(name: &str) -> Slot {
        match name {
            "slot-a" => Slot::First,
            "slot-b" => Slot::Second,
            "slot-c" => Slot::Third,
            other => panic!("unknown slot {other}"),
        }
    }

    const fn slot_name(slot: Slot) -> &'static str {
        match slot {
            Slot::First => "slot-a",
            Slot::Second => "slot-b",
            Slot::Third => "slot-c",
        }
    }

    fn item(name: &str) -> MenuItem {
        match name {
            "copy" => MenuItem::Copy,
            "clear" => MenuItem::Clear,
            "exit" => MenuItem::Exit,
            other => MenuItem::File(slot(other)),
        }
    }

    const fn item_name(item: MenuItem) -> &'static str {
        match item {
            MenuItem::File(slot) => slot_name(slot),
            MenuItem::Copy => "copy",
            MenuItem::Clear => "clear",
            MenuItem::Exit => "exit",
        }
    }

    fn screen((kind, level): &(String, u8)) -> ScreenLight {
        let level = Level::new(*level).expect("a level 0 to 15");
        match kind.as_str() {
            "blank" => ScreenLight::Blank(level),
            "lit" => ScreenLight::Lit(level),
            other => panic!("unknown screen {other}"),
        }
    }

    fn screen_flat(screen: ScreenLight) -> (String, u8) {
        match screen {
            ScreenLight::Blank(level) => ("blank".to_owned(), level.get()),
            ScreenLight::Lit(level) => ("lit".to_owned(), level.get()),
        }
    }

    fn load_step(step: u8) -> LoadStep {
        match step {
            0 => LoadStep::Step0,
            1 => LoadStep::Step1,
            2 => LoadStep::Step2,
            3 => LoadStep::Step3,
            other => panic!("no load step {other} before the menu"),
        }
    }

    const fn load_step_number(step: LoadStep) -> u8 {
        match step {
            LoadStep::Step0 => 0,
            LoadStep::Step1 => 1,
            LoadStep::Step2 => 2,
            LoadStep::Step3 => 3,
        }
    }

    const fn offer(offered: bool) -> CopyAndClear {
        if offered {
            CopyAndClear::Offered
        } else {
            CopyAndClear::NotOffered
        }
    }

    fn way_out(name: &str) -> Option<WayOut> {
        match name {
            "options-begin" => Some(WayOut::OptionsBegin),
            "title-returns" => Some(WayOut::TitleReturns),
            "copy-begins" => Some(WayOut::CopyBegins),
            "clear-begins" => Some(WayOut::ClearBegins),
            _ => None,
        }
    }

    const fn way_out_name(way: WayOut) -> &'static str {
        match way {
            WayOut::OptionsBegin => "options-begin",
            WayOut::TitleReturns => "title-returns",
            WayOut::CopyBegins => "copy-begins",
            WayOut::ClearBegins => "clear-begins",
        }
    }

    fn phase(before: &Before) -> MenuPhase {
        match before.stage.as_str() {
            "loading" => MenuPhase::LoadEnding,
            "brightening" => MenuPhase::Brightening,
            "choosing" => MenuPhase::Choosing,
            "confirming" => MenuPhase::Confirming {
                chosen: slot(before.chosen.as_deref().expect("a chosen slot")),
                turn_left: TurnLeft::new(before.turn_left.expect("a turn left")).expect("1 to 255"),
            },
            "leaving" => MenuPhase::Leaving,
            other => {
                MenuPhase::Out(way_out(other).unwrap_or_else(|| panic!("unknown stage {other}")))
            }
        }
    }

    fn state(before: &Before) -> FileSelectState {
        let stage = match (before.stage.as_str(), before.load_step) {
            ("loading", Some(step)) if step < LOAD_ENDING_STEP => {
                FileSelectStage::Loading(load_step(step))
            }
            _ => FileSelectStage::Menu(MenuState {
                phase: phase(before),
                cursor: item(
                    before
                        .cursor
                        .as_deref()
                        .expect("a cursor once the menu is ready"),
                ),
                copy_and_clear: before.offered.map_or(UNSTATED_OFFER, offer),
                fade_wait: before.fade_wait.map_or(UNSTATED_WAIT, FadeWait),
            }),
        };
        FileSelectState {
            stage,
            screen: screen(&before.screen),
        }
    }

    fn flat(model: FileSelectState) -> Flat {
        let screen = screen_flat(model.screen);
        let menu = match model.stage {
            FileSelectStage::Loading(step) => {
                return Flat {
                    stage: "loading".to_owned(),
                    load_step: Some(load_step_number(step)),
                    screen,
                    cursor: None,
                    offered: None,
                    fade_wait: None,
                    chosen: None,
                    turn_left: None,
                };
            }
            FileSelectStage::Menu(menu) => menu,
        };
        let (stage, load_step, chosen, turn_left) = match menu.phase {
            MenuPhase::LoadEnding => ("loading", Some(LOAD_ENDING_STEP), None, None),
            MenuPhase::Brightening => ("brightening", None, None, None),
            MenuPhase::Choosing => ("choosing", None, None, None),
            MenuPhase::Confirming { chosen, turn_left } => (
                "confirming",
                None,
                Some(slot_name(chosen).to_owned()),
                Some(turn_left.get()),
            ),
            MenuPhase::Leaving => ("leaving", None, None, None),
            MenuPhase::Out(way) => (way_out_name(way), None, None, None),
        };
        Flat {
            stage: stage.to_owned(),
            load_step,
            screen,
            cursor: Some(item_name(menu.cursor).to_owned()),
            offered: Some(menu.copy_and_clear == CopyAndClear::Offered),
            fade_wait: Some(menu.fade_wait.0),
            chosen,
            turn_left,
        }
    }

    fn action(button: &str) -> Option<MenuAction> {
        match button {
            "A" | "Start" => Some(MenuAction::Choose),
            "Up" => Some(MenuAction::Up),
            "Down" => Some(MenuAction::Down),
            "B" => Some(MenuAction::Back),
            "X" | "Y" | "Left" | "Right" | "Select" | "L" | "R" => None,
            other => panic!("unknown button {other}"),
        }
    }

    fn frame_end_of(vector: &Vector) -> FrameEnd {
        let pressed = vector.pressed.as_deref().unwrap_or_default();
        match vector.finished {
            0 => {
                assert!(
                    pressed.is_empty(),
                    "{}: presses on a busy frame end",
                    vector.id
                );
                FrameEnd::Busy
            }
            1 => {
                assert!(
                    pressed.len() <= 1,
                    "{}: one list per finished frame",
                    vector.id
                );
                let actions = pressed
                    .iter()
                    .flatten()
                    .filter_map(|button| action(button))
                    .map(|action| (action, true))
                    .collect();
                FrameEnd::Finished(actions)
            }
            other => panic!("{}: finished must be 0 or 1, not {other}", vector.id),
        }
    }

    fn rules(vector: &Vector) -> Rules {
        let mut rules = Rules::REFERENCE;
        if let Some(adjusted) = &vector.rules {
            if let Some(pace) = adjusted.menu_pace {
                rules.menu_pace = MenuPace::new(pace).expect("a pace 1 to 256");
            }
            if let Some(turn) = adjusted.helmet_turn {
                rules.helmet_turn = HelmetTurn::new(turn).expect("a turn 1 to 255");
            }
        }
        rules
    }

    const fn sound_name(sound: MenuSound) -> &'static str {
        match sound {
            MenuSound::MenuOpens => "menu-opens",
            MenuSound::CursorMoved => "cursor-moved",
            MenuSound::ChoiceMade => "choice-made",
        }
    }

    /// A field the vector checks, and how the model's value differs from it.
    fn compare<T: PartialEq + std::fmt::Debug>(
        failures: &mut Vec<String>,
        field: &str,
        expected: Option<&T>,
        actual: Option<&T>,
    ) {
        if let Some(expected) = expected
            && Some(expected) != actual
        {
            failures.push(format!("{field}: expected {expected:?}, got {actual:?}"));
        }
    }

    fn check_report(failures: &mut Vec<String>, after: &After, report: &FrameReport) {
        let expected = after.requests.clone().unwrap_or_default();
        let actual: Vec<String> = report
            .sounds
            .iter()
            .map(|&sound| sound_name(sound).to_owned())
            .collect();
        compare(failures, "requests", Some(&expected), Some(&actual));
        let goes_on = report.frame == FrameOfWork::GoesOn;
        compare(
            failures,
            "frame_goes_on",
            Some(&after.frame_goes_on.unwrap_or(false)),
            Some(&goes_on),
        );
    }

    fn check_state(failures: &mut Vec<String>, after: &After, state: &Flat) {
        compare(failures, "stage", after.stage.as_ref(), Some(&state.stage));
        compare(
            failures,
            "load_step",
            after.load_step.as_ref(),
            state.load_step.as_ref(),
        );
        compare(
            failures,
            "screen",
            after.screen.as_ref(),
            Some(&state.screen),
        );
        compare(
            failures,
            "cursor",
            after.cursor.as_ref(),
            state.cursor.as_ref(),
        );
        compare(
            failures,
            "offered",
            after.offered.as_ref(),
            state.offered.as_ref(),
        );
        compare(
            failures,
            "fade_wait",
            after.fade_wait.as_ref(),
            state.fade_wait.as_ref(),
        );
        compare(
            failures,
            "chosen",
            after.chosen.as_ref(),
            state.chosen.as_ref(),
        );
        compare(
            failures,
            "turn_left",
            after.turn_left.as_ref(),
            state.turn_left.as_ref(),
        );
    }

    /// The failures of one vector, none when it passes.
    fn run(vector: &Vector) -> Vec<String> {
        let mut state = state(&vector.before);
        let save = vector.input.as_ref();
        let empty = SaveAnswer::default();
        let result = frame_end(
            &mut state,
            frame_end_of(vector),
            save.unwrap_or(&empty),
            &rules(vector),
        );
        let mut failures = Vec::new();
        match (result, vector.after.unsupported.unwrap_or(false)) {
            (Err(Unsupported(_)), true) => {}
            (Err(Unsupported(way)), false) => failures.push(format!("unsupported after {way:?}")),
            (Ok(_), true) => failures.push("expected unsupported".to_owned()),
            (Ok(report), false) => {
                check_report(&mut failures, &vector.after, &report);
                check_state(&mut failures, &vector.after, &flat(state));
            }
        }
        failures
    }

    fn vectors() -> Vec<Vector> {
        toml::from_str::<VectorFile>(VECTORS)
            .expect("the vector file reads")
            .vector
    }

    fn behaviour_id(name: &str) -> BehaviourId {
        catalogue()
            .values()
            .map(|entry| entry.id)
            .find(|id| format!("{id:?}") == name)
            .unwrap_or_else(|| panic!("no catalogue entry {name}"))
    }

    #[test]
    fn every_vector_passes() {
        let vectors = vectors();
        assert!(!vectors.is_empty());
        let failures: Vec<String> = vectors
            .iter()
            .flat_map(|vector| {
                assert!(
                    ["recorded", "constructed"].contains(&vector.source.as_str()),
                    "{}: unknown source {}",
                    vector.id,
                    vector.source
                );
                run(vector)
                    .into_iter()
                    .map(move |failure| format!("{}: {failure}", vector.id))
            })
            .collect();
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn every_entry_is_exercised_and_known() {
        let exercised: Set<BehaviourId> = set_of([]);
        let exercised = vectors()
            .iter()
            .flat_map(|vector| vector.entries.iter())
            .fold(exercised, |mut set, name| {
                set[behaviour_id(name)] = true;
                set
            });
        for (id, entry) in catalogue() {
            assert_eq!(entry.id, id, "the catalogue files {id:?} under its own id");
            assert_eq!(behaviour(id), entry);
            // Every entry has a vector, those the spec checks by vectors among them.
            assert!(exercised[id], "{id:?} ({:?}) has no vector", entry.checked);
        }
    }
}
