#![cfg(not(doctest))]
//! The screens-title model, built from its behaviour spec alone.
//!
//! The title sequence, one frame end at a time: black while the title's pictures load, the
//! opening lighting up and, on a skip button, fading out at once, black while the title tune
//! loads, the title screen fading in and counting down to the demos, and, on a skip button
//! there, the title screen slowly dimming until file select begins.
//!
//! [`end_frame`] takes the [`TitleState`] and how the frame ended, and gives the State after it
//! with the tunes asked for. The State itself is `types::title`'s, so the game can save it.

pub use types::power_on::{Level, ScreenLight};
pub use types::title::{DemoCountdown, DemoWait, DimDelay, LeavePace, TitleStage, TitleState};

pub use catalogue::{Behaviour, BehaviourId, catalogue};
pub use frame::{Button, FrameEnd, FrameReport, Pressed, TuneRequest, Unsupported};
pub use rules::{FastStep, Rules};
pub use title::end_frame;

/// The values an adjusted game may change: the fast step, the demo wait and the leaving pace.
mod rules {
    use types::title::{DemoWait, LeavePace};

    /// Spec: `SkipDims`. The quickest fast step: 15 levels, dark at once.
    const LARGEST_FAST_STEP: u8 = 15;
    /// Spec: `SkipDims`. The reference fast step, 2 levels per finished frame.
    const REFERENCE_FAST_STEP: u8 = 2;

    /// Spec: `SkipDims`, `TitleBrightens`. How many levels the skipped opening darkens and the
    /// title screen brightens on each finished frame: 1 to 15.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct FastStep(u8);

    impl FastStep {
        /// The reference game's step, 2 levels.
        pub const REFERENCE: Self = Self(REFERENCE_FAST_STEP);

        /// The step of this many levels, if it is one (1 to 15).
        #[must_use]
        pub const fn new(levels: u8) -> Option<Self> {
            if levels >= 1 && levels <= LARGEST_FAST_STEP {
                Some(Self(levels))
            } else {
                None
            }
        }

        /// Its levels.
        #[must_use]
        pub const fn get(self) -> u8 {
            self.0
        }
    }

    /// The title's rules. [`Rules::REFERENCE`] is the vanilla game; any other is an adjusted
    /// one.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Rules {
        /// Spec: `SkipDims`, `TitleBrightens`. One step for both fades.
        pub fast_step: FastStep,
        /// Spec: `TitleWaits`. The title screen's wait before the demos.
        pub demo_wait: DemoWait,
        /// Spec: `TitleLeaves`. Finished frames per level as the title screen is left.
        pub leave_pace: LeavePace,
    }

    impl Rules {
        /// The vanilla game: a fast step of 2, a demo wait of 900 and a leaving pace of 2.
        pub const REFERENCE: Self = Self {
            fast_step: FastStep::REFERENCE,
            demo_wait: DemoWait::REFERENCE,
            leave_pace: LeavePace::REFERENCE,
        };
    }
}

/// What a frame end brings in, the buttons and the finished frames, and what it reports.
mod frame {
    use enum_map::Enum;
    use types::Set;
    use types::title::TitleState;

    /// A button of the controller.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
    pub enum Button {
        /// A.
        A,
        /// B.
        B,
        /// X.
        X,
        /// Y.
        Y,
        /// The left shoulder.
        L,
        /// The right shoulder.
        R,
        /// Start.
        Start,
        /// Select.
        Select,
        /// Up on the pad.
        Up,
        /// Down on the pad.
        Down,
        /// Left on the pad.
        Left,
        /// Right on the pad.
        Right,
    }

    impl Button {
        /// Spec: `OpeningSkipped`, `TitleWaits`. A skip button is A, B or Start; the title
        /// acts on no other.
        #[must_use]
        pub const fn skips(self) -> bool {
            match self {
                Self::A | Self::B | Self::Start => true,
                Self::X
                | Self::Y
                | Self::L
                | Self::R
                | Self::Select
                | Self::Up
                | Self::Down
                | Self::Left
                | Self::Right => false,
            }
        }
    }

    /// The buttons just pressed as a frame of work began.
    pub type Pressed = Set<Button>;

    /// How a frame ended: how many frames of work finished since the last frame end, each with
    /// the buttons just pressed as it began, oldest first.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum FrameEnd {
        /// No frame of work finished: the work runs on through the frame end.
        NoneFinished,
        /// One frame of work finished.
        OneFinished(Pressed),
        /// Two frames of work finished, the first before the second.
        TwoFinished([Pressed; 2]),
    }

    impl FrameEnd {
        /// The finished frames' presses, one per frame, oldest first.
        #[must_use]
        pub const fn finished(&self) -> &[Pressed] {
            match self {
                Self::NoneFinished => &[],
                Self::OneFinished(pressed) => core::slice::from_ref(pressed),
                Self::TwoFinished(pressed) => pressed,
            }
        }
    }

    /// A tune the title asks the audio side for.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum TuneRequest {
        /// The title's set of tunes.
        TitleMusic,
        /// The opening's theme.
        OpeningTheme,
        /// The title screen's theme.
        TitleTheme,
    }

    /// What a frame end gives: the State after it, and the tunes asked for in it, oldest first.
    /// The screen, the countdown and any way out taken are the State's.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct FrameReport {
        /// The State at the frame end.
        pub state: TitleState,
        /// The tunes asked for, oldest first; empty where none was.
        pub tunes: Vec<TuneRequest>,
    }

    /// A frame end after a way out (the opening playing on, the demos or file select), which
    /// belongs to the piece that follows.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Unsupported;
}

/// The title's rules, one finished frame of work at a time.
mod title {
    use types::power_on::{Level, ScreenLight};
    use types::title::{DemoCountdown, DimDelay, TitleStage, TitleState};

    use crate::frame::{FrameEnd, FrameReport, Pressed, TuneRequest, Unsupported};
    use crate::rules::Rules;

    /// Spec: `TitleLeaves`. The leaving title screen dims one level at a time.
    const LEAVE_STEP: u8 = 1;

    /// The title at one frame end: each finished frame of work in turn.
    ///
    /// The whole of the first frame comes before the second, each reading the buttons pressed
    /// as it began. A frame end with none finished changes only the loading screen.
    ///
    /// # Errors
    ///
    /// [`Unsupported`] when a frame end, or a frame of work in it, comes after a way out.
    pub fn end_frame(
        state: TitleState,
        rules: Rules,
        frame_end: &FrameEnd,
    ) -> Result<FrameReport, Unsupported> {
        let mut report = FrameReport {
            state,
            tunes: Vec::new(),
        };
        match frame_end.finished() {
            [] => report.state = still_working(state)?,
            finished => {
                for pressed in finished {
                    finish_frame(&mut report, rules, pressed)?;
                }
            }
        }
        Ok(report)
    }

    /// A frame end at which no frame of work finished. Spec: `TitleLoads`: the loading screen
    /// goes blank and dark. Every other stage stays as it is.
    const fn still_working(state: TitleState) -> Result<TitleState, Unsupported> {
        match state.stage {
            TitleStage::Loading => Ok(TitleState {
                stage: TitleStage::Loading,
                screen: ScreenLight::Blank(Level::DARK),
            }),
            TitleStage::OpeningPlays | TitleStage::DemoBegins | TitleStage::FileSelectBegins => {
                Err(Unsupported)
            }
            TitleStage::Begun
            | TitleStage::SkipDimming
            | TitleStage::Preparing
            | TitleStage::Brightening
            | TitleStage::TitleScreen { .. }
            | TitleStage::Leaving { .. } => Ok(state),
        }
    }

    /// One finished frame of work, by the stage it finds.
    fn finish_frame(
        report: &mut FrameReport,
        rules: Rules,
        pressed: &Pressed,
    ) -> Result<(), Unsupported> {
        let state = &mut report.state;
        match state.stage {
            // Spec: `TitleBegins`. The first finished frame is the title's; then it loads.
            TitleStage::Begun => state.stage = TitleStage::Loading,
            TitleStage::Loading => finish_load(report, rules, pressed),
            TitleStage::SkipDimming => skip_dim(state, &mut report.tunes, rules),
            // Spec: `TitlePrepared`. One dark frame sets the title screen up.
            TitleStage::Preparing => state.stage = TitleStage::Brightening,
            TitleStage::Brightening => brighten(state, rules),
            TitleStage::TitleScreen { countdown } => wait(state, rules, countdown, pressed),
            TitleStage::Leaving {
                countdown,
                next_dim,
            } => leave(state, rules, countdown, next_dim),
            TitleStage::OpeningPlays | TitleStage::DemoBegins | TitleStage::FileSelectBegins => {
                return Err(Unsupported);
            }
        }
        Ok(())
    }

    /// Whether a skip button (A, B or Start) is among the buttons just pressed.
    fn skip_pressed(pressed: &Pressed) -> bool {
        pressed
            .iter()
            .any(|(button, &is_pressed)| is_pressed && button.skips())
    }

    /// Spec: `TitleLoads`. The load's frame asks for the title's tunes then the opening's theme,
    /// and lights the opening at full. Spec: `OpeningSkipped`: on a skip button the opening
    /// starts its fade-out in this same frame; otherwise it plays on.
    fn finish_load(report: &mut FrameReport, rules: Rules, pressed: &Pressed) {
        report
            .tunes
            .extend([TuneRequest::TitleMusic, TuneRequest::OpeningTheme]);
        report.state.screen = ScreenLight::Lit(Level::FULL);
        if skip_pressed(pressed) {
            skip_dim(&mut report.state, &mut report.tunes, rules);
        } else {
            report.state.stage = TitleStage::OpeningPlays;
        }
    }

    /// Spec: `SkipDims`. The skipped opening darkens by the fast step; once that would reach
    /// dark it goes blank and dark and the title tune is asked for. Presses do nothing.
    fn skip_dim(state: &mut TitleState, tunes: &mut Vec<TuneRequest>, rules: Rules) {
        *state = dimmed(state.screen.level(), rules.fast_step.get()).map_or_else(
            || {
                tunes.push(TuneRequest::TitleTheme);
                TitleState {
                    stage: TitleStage::Preparing,
                    screen: ScreenLight::Blank(Level::DARK),
                }
            },
            |level| TitleState {
                stage: TitleStage::SkipDimming,
                screen: ScreenLight::Lit(level),
            },
        );
    }

    /// Spec: `TitleBrightens`. The title screen brightens by the fast step from its level, a
    /// blank screen counting as dark; once that would reach full it shows at full and the
    /// countdown starts at the whole demo wait. Presses do nothing.
    fn brighten(state: &mut TitleState, rules: Rules) {
        let level = match state.screen {
            ScreenLight::Blank(_) => Level::DARK,
            ScreenLight::Lit(level) => level,
        };
        *state = brightened(level, rules.fast_step.get()).map_or_else(
            || TitleState {
                stage: TitleStage::TitleScreen {
                    countdown: DemoCountdown::whole(rules.demo_wait),
                },
                screen: ScreenLight::Lit(Level::FULL),
            },
            |level| TitleState {
                stage: TitleStage::Brightening,
                screen: ScreenLight::Lit(level),
            },
        );
    }

    /// Spec: `TitleWaits`. The countdown loses a frame. Run out, the demos begin, whatever was
    /// pressed; otherwise a skip button leaves the title screen, a whole leaving pace before
    /// its first dim.
    fn wait(state: &mut TitleState, rules: Rules, countdown: DemoCountdown, pressed: &Pressed) {
        let countdown = countdown.counted_down();
        state.stage = if countdown == DemoCountdown::RAN_OUT {
            TitleStage::DemoBegins
        } else if skip_pressed(pressed) {
            TitleStage::Leaving {
                countdown,
                next_dim: DimDelay::whole(rules.leave_pace),
            }
        } else {
            TitleStage::TitleScreen { countdown }
        };
    }

    /// Spec: `TitleLeaves`. The delay loses a frame; when it runs out the screen dims a level
    /// and the delay starts again at the leaving pace, and at dark file select begins. The
    /// countdown stays as it was, and presses do nothing.
    fn leave(state: &mut TitleState, rules: Rules, countdown: DemoCountdown, next_dim: DimDelay) {
        *state = match (
            next_dim.counted_down(),
            dimmed(state.screen.level(), LEAVE_STEP),
        ) {
            (Some(next_dim), _) => TitleState {
                stage: TitleStage::Leaving {
                    countdown,
                    next_dim,
                },
                screen: state.screen,
            },
            (None, Some(level)) => TitleState {
                stage: TitleStage::Leaving {
                    countdown,
                    next_dim: DimDelay::whole(rules.leave_pace),
                },
                screen: ScreenLight::Lit(level),
            },
            (None, None) => TitleState {
                stage: TitleStage::FileSelectBegins,
                screen: ScreenLight::Blank(Level::DARK),
            },
        };
    }

    /// The level `step` below `level`, if that is still above dark.
    fn dimmed(level: Level, step: u8) -> Option<Level> {
        level
            .get()
            .checked_sub(step)
            .and_then(Level::new)
            .filter(|&level| level != Level::DARK)
    }

    /// The level `step` above `level`, if that is still below full.
    fn brightened(level: Level, step: u8) -> Option<Level> {
        Level::new(level.get().saturating_add(step)).filter(|&level| level != Level::FULL)
    }
}

/// The catalogue: one behaviour per spec entry.
mod catalogue {
    use enum_map::{Enum, EnumMap};
    use types::{Checked, Kind, Switch};

    /// A spec entry of the title.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
    pub enum BehaviourId {
        /// The first finished frame is the title's; then it loads.
        TitleBegins,
        /// Black while loading; then the opening lights and asks for its tunes.
        TitleLoads,
        /// A skip button starts the opening's fast fade-out at once.
        OpeningSkipped,
        /// The skipped opening darkens by the fast step; at dark the title tune is asked for.
        SkipDims,
        /// One dark frame sets the title screen up.
        TitlePrepared,
        /// The title screen brightens by the fast step up to full.
        TitleBrightens,
        /// The title screen counts down to the demos; a skip button leaves for file select.
        TitleWaits,
        /// The left title screen dims a level every leaving pace; at dark file select begins.
        TitleLeaves,
    }

    /// One behaviour of the catalogue.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Behaviour {
        /// Its spec entry.
        pub id: BehaviourId,
        /// What the player sees.
        pub summary: &'static str,
        /// What kind of behaviour it is.
        pub kind: Kind,
        /// How it can be adjusted.
        pub switch: Switch,
        /// How the spec checks it.
        pub checked: Checked,
    }

    /// The catalogue's rows, each naming its entry.
    const ROWS: [Behaviour; 8] = [
        Behaviour {
            id: BehaviourId::TitleBegins,
            summary: "the first frame that finishes after the game settles is the title's; then it loads",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        Behaviour {
            id: BehaviourId::TitleLoads,
            summary: "black while the title's pictures load; when the load's frame finishes, the opening lights at full and asks for its tunes",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        Behaviour {
            id: BehaviourId::OpeningSkipped,
            summary: "a skip button pressed while the opening shows starts its fast fade-out at once",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        Behaviour {
            id: BehaviourId::SkipDims,
            summary: "the skipped opening darkens by the fast step on each finished frame; at dark it asks for the title tune",
            kind: Kind::Mechanic,
            switch: Switch::Parameter,
            checked: Checked::ByVectors,
        },
        Behaviour {
            id: BehaviourId::TitlePrepared,
            summary: "one dark frame sets the title screen up; it then brightens",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        Behaviour {
            id: BehaviourId::TitleBrightens,
            summary: "the title screen brightens by the fast step on each finished frame, up to full",
            kind: Kind::Mechanic,
            switch: Switch::Parameter,
            checked: Checked::ByVectors,
        },
        Behaviour {
            id: BehaviourId::TitleWaits,
            summary: "the title screen counts down to the demos; a skip button pressed first leaves for file select",
            kind: Kind::Mechanic,
            switch: Switch::Parameter,
            checked: Checked::ByVectors,
        },
        Behaviour {
            id: BehaviourId::TitleLeaves,
            summary: "leaving, the title screen dims one level every leaving pace of finished frames; at dark file select begins",
            kind: Kind::Mechanic,
            switch: Switch::Parameter,
            checked: Checked::ByVectors,
        },
    ];

    /// The catalogue, one behaviour per spec entry.
    #[must_use]
    pub fn catalogue() -> EnumMap<BehaviourId, Behaviour> {
        types::by_member(ROWS, |row| row.id)
    }
}

/// Every vector of `scenarios/vectors/screens-title.toml`, run through the model, and the
/// catalogue checked against them.
#[cfg(test)]
mod vectors {
    use serde::Deserialize;

    use super::*;

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
        #[serde(default)]
        rules: RuleOverrides,
        before: Before,
        finished: usize,
        #[serde(default)]
        pressed: Vec<Vec<String>>,
        after: After,
    }

    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields)]
    struct RuleOverrides {
        fast_step: Option<u8>,
        demo_wait: Option<u16>,
        leave_pace: Option<u8>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Before {
        stage: String,
        screen: (String, u8),
        countdown: Option<u16>,
        next_dim: Option<u8>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct After {
        stage: Option<String>,
        screen: Option<(String, u8)>,
        countdown: Option<u16>,
        next_dim: Option<u8>,
        #[serde(default)]
        requests: Vec<String>,
        #[serde(default)]
        unsupported: bool,
    }

    fn vectors() -> Vec<Vector> {
        toml::from_str::<VectorFile>(&vector_file::read("screens-title").unwrap())
            .unwrap()
            .vector
    }

    fn rules(overrides: &RuleOverrides) -> Rules {
        let reference = Rules::REFERENCE;
        Rules {
            fast_step: overrides
                .fast_step
                .map_or(reference.fast_step, |k| FastStep::new(k).unwrap()),
            demo_wait: overrides
                .demo_wait
                .map_or(reference.demo_wait, |w| DemoWait::new(w).unwrap()),
            leave_pace: overrides
                .leave_pace
                .map_or(reference.leave_pace, |p| LeavePace::new(p).unwrap()),
        }
    }

    fn screen((light, level): &(String, u8)) -> ScreenLight {
        let level = Level::new(*level).unwrap();
        match light.as_str() {
            "blank" => ScreenLight::Blank(level),
            "lit" => ScreenLight::Lit(level),
            other => panic!("unknown screen {other}"),
        }
    }

    fn stage(before: &Before) -> TitleStage {
        let countdown = || DemoCountdown::new(before.countdown.unwrap()).unwrap();
        let next_dim = || DimDelay::new(before.next_dim.unwrap()).unwrap();
        match before.stage.as_str() {
            "begun" => TitleStage::Begun,
            "loading" => TitleStage::Loading,
            "skip-dimming" => TitleStage::SkipDimming,
            "preparing" => TitleStage::Preparing,
            "brightening" => TitleStage::Brightening,
            "title-screen" => TitleStage::TitleScreen {
                countdown: countdown(),
            },
            "leaving" => TitleStage::Leaving {
                countdown: countdown(),
                next_dim: next_dim(),
            },
            "opening-plays" => TitleStage::OpeningPlays,
            "demo-begins" => TitleStage::DemoBegins,
            "file-select-begins" => TitleStage::FileSelectBegins,
            other => panic!("unknown stage {other}"),
        }
    }

    const fn stage_name(stage: TitleStage) -> &'static str {
        match stage {
            TitleStage::Begun => "begun",
            TitleStage::Loading => "loading",
            TitleStage::SkipDimming => "skip-dimming",
            TitleStage::Preparing => "preparing",
            TitleStage::Brightening => "brightening",
            TitleStage::TitleScreen { .. } => "title-screen",
            TitleStage::Leaving { .. } => "leaving",
            TitleStage::OpeningPlays => "opening-plays",
            TitleStage::DemoBegins => "demo-begins",
            TitleStage::FileSelectBegins => "file-select-begins",
        }
    }

    fn button(name: &str) -> Button {
        match name {
            "A" => Button::A,
            "B" => Button::B,
            "X" => Button::X,
            "Y" => Button::Y,
            "L" => Button::L,
            "R" => Button::R,
            "Start" => Button::Start,
            "Select" => Button::Select,
            "Up" => Button::Up,
            "Down" => Button::Down,
            "Left" => Button::Left,
            "Right" => Button::Right,
            other => panic!("unknown button {other}"),
        }
    }

    fn tune(name: &str) -> TuneRequest {
        match name {
            "title-music" => TuneRequest::TitleMusic,
            "opening-theme" => TuneRequest::OpeningTheme,
            "title-theme" => TuneRequest::TitleTheme,
            other => panic!("unknown tune {other}"),
        }
    }

    fn frame_end(vector: &Vector) -> FrameEnd {
        let pressed = |frame: usize| -> Pressed {
            vector
                .pressed
                .get(frame)
                .map(|names| names.iter().map(|name| (button(name), true)).collect())
                .unwrap_or_default()
        };
        assert!(
            vector.pressed.len() <= vector.finished,
            "{}: more presses than finished frames",
            vector.id
        );
        match vector.finished {
            0 => FrameEnd::NoneFinished,
            1 => FrameEnd::OneFinished(pressed(0)),
            2 => FrameEnd::TwoFinished([pressed(0), pressed(1)]),
            other => panic!("{}: {other} finished frames", vector.id),
        }
    }

    /// The differences between what the vector expects and what the model gives.
    fn mismatches(vector: &Vector) -> Vec<String> {
        let before = TitleState {
            stage: stage(&vector.before),
            screen: screen(&vector.before.screen),
        };
        let outcome = end_frame(before, rules(&vector.rules), &frame_end(vector));
        let after = &vector.after;
        let report = match (outcome, after.unsupported) {
            (Err(Unsupported), true) => return Vec::new(),
            (Ok(report), false) => report,
            (outcome, _) => return vec![format!("outcome {outcome:?}")],
        };
        let state = report.state;
        let mut found = Vec::new();
        if let Some(expected) = &after.stage
            && expected != stage_name(state.stage)
        {
            found.push(format!("stage {expected} != {:?}", state.stage));
        }
        if let Some(expected) = &after.screen
            && screen(expected) != state.screen
        {
            found.push(format!("screen {expected:?} != {:?}", state.screen));
        }
        if let Some(expected) = after.countdown
            && Some(expected) != state.stage.countdown().map(DemoCountdown::get)
        {
            found.push(format!("countdown {expected} != {:?}", state.stage));
        }
        if let Some(expected) = after.next_dim {
            let next_dim = if let TitleStage::Leaving { next_dim, .. } = state.stage {
                Some(next_dim.get())
            } else {
                None
            };
            if Some(expected) != next_dim {
                found.push(format!("next_dim {expected} != {next_dim:?}"));
            }
        }
        let tunes: Vec<TuneRequest> = after.requests.iter().map(|name| tune(name)).collect();
        if tunes != report.tunes {
            found.push(format!("requests {tunes:?} != {:?}", report.tunes));
        }
        found
    }

    #[test]
    fn every_vector_passes() {
        let vectors = vectors();
        assert!(!vectors.is_empty());
        let failures: Vec<String> = vectors
            .iter()
            .filter_map(|vector| {
                assert!(
                    ["recorded", "constructed"].contains(&vector.source.as_str()),
                    "{}: unknown source {}",
                    vector.id,
                    vector.source
                );
                let found = mismatches(vector);
                (!found.is_empty()).then(|| format!("{}: {}", vector.id, found.join("; ")))
            })
            .collect();
        assert!(
            failures.is_empty(),
            "failing vectors:\n{}",
            failures.join("\n")
        );
    }

    fn behaviour_id(name: &str) -> BehaviourId {
        catalogue()
            .values()
            .map(|behaviour| behaviour.id)
            .find(|id| format!("{id:?}") == name)
            .unwrap_or_else(|| panic!("unknown entry {name}"))
    }

    #[test]
    fn the_catalogue_names_each_entry_once() {
        assert!(catalogue().iter().all(|(id, behaviour)| behaviour.id == id));
    }

    #[test]
    fn every_entry_checked_by_vectors_has_a_vector() {
        let named: Vec<BehaviourId> = vectors()
            .iter()
            .flat_map(|vector| vector.entries.iter().map(|name| behaviour_id(name)))
            .collect();
        for (id, behaviour) in catalogue() {
            if behaviour.checked == types::Checked::ByVectors {
                assert!(named.contains(&id), "{id:?} has no vector");
            }
        }
    }
}
