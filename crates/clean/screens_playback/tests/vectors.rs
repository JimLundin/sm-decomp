//! Runs every screens-playback vector through the model, and checks the catalogue against them.
//!
//! The recorded vectors carry no pad content: the scene's imported timelines belong to the later
//! importer. Each recorded vector whose snapshot sits in a timeline gets an invented short hold
//! sequence here instead, written so that its before state lies inside the timeline and its
//! recorded report follows from it. The constructed vectors carry their own holds.

#[cfg(test)]
mod vectors {
    use core::cell::RefCell;
    use std::collections::HashMap;

    use serde::Deserialize;
    use types::playback::{PadAction, PadSample, ScenePhase, TimelinePlace, TimelineProgress};
    use types::{Checked, Set};

    use screens_playback::{
        CrossfadeCount, DemoPad, Handoff, PadHold, ScenePad, SceneStart, TurnReadings, WholeX,
        catalogue, play_turn,
    };

    const SPEC: &str = include_str!("../../../../docs/behaviours/screens-playback.md");

    /// Invented holds, keyed by recorded vector id, for the timelines each one reads; `default`
    /// fills every timeline a vector never reads. None of it is the recorded content.
    const INVENTED: &str = r#"
[default]
battle = [{ turns = 1, held = ["fire"], pressed = ["fire"] }]
approach = [{ turns = 1, held = ["left"], pressed = ["left"] }]
watch = [{ turns = 1, held = ["left"], pressed = ["left"] }]

["R-battle-first"]
battle = [{ turns = 2, held = [], pressed = [] }]

["R-battle-hold"]
battle = [
    { turns = 90, held = [], pressed = [] },
    { turns = 3, held = ["fire"], pressed = ["fire"] },
]

["R-battle-partial"]
battle = [
    { turns = 98, held = [], pressed = [] },
    { turns = 5, held = ["fire"], pressed = ["fire"] },
]

["R-battle-paused"]
battle = [
    { turns = 99, held = [], pressed = [] },
    { turns = 4, held = ["fire"], pressed = ["fire"] },
]

["R-battle-before-pad"]
battle = [
    { turns = 120, held = [], pressed = [] },
    { turns = 10, held = ["fire"], pressed = ["fire"] },
]

["R-battle-refire"]
battle = [
    { turns = 129, held = [], pressed = [] },
    { turns = 2, held = ["fire"], pressed = ["fire"] },
    { turns = 2, held = ["fire"], pressed = ["fire"] },
]

["R-battle-jump"]
battle = [
    { turns = 257, held = [], pressed = [] },
    { turns = 2, held = ["left"], pressed = ["left"] },
    { turns = 2, held = ["jump", "left"], pressed = ["jump"] },
]

["R-battle-lock"]
battle = [
    { turns = 390, held = [], pressed = [] },
    { turns = 2, held = ["fire"], pressed = ["fire"] },
]

["R-approach-keeps-left"]
approach = [
    { turns = 90, held = ["left"], pressed = ["left"] },
    { turns = 1, held = ["left"], pressed = ["left"] },
]

["R-approach-outside-line"]
approach = [{ turns = 91, held = ["left"], pressed = ["left"] }]

["R-approach-reaches-baby"]
approach = [{ turns = 92, held = ["left"], pressed = ["left"] }]
watch = [
    { turns = 2, held = [], pressed = [] },
    { turns = 1, held = ["left"], pressed = ["left"] },
]

["R-watch-aim"]
watch = [
    { turns = 300, held = [], pressed = [] },
    { turns = 2, held = ["aim_up"], pressed = ["aim_up"] },
    { turns = 1, held = ["left"], pressed = ["left"] },
]

["R-watch-keeps-left"]
watch = [
    { turns = 711, held = [], pressed = [] },
    { turns = 1, held = ["left"], pressed = ["left"] },
]

["R-discovery-release"]
watch = [
    { turns = 1, held = [], pressed = [] },
    { turns = 711, held = ["left"], pressed = ["left"] },
]
"#;

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
        before: StateRecord,
        input: InputRecord,
        after: StateRecord,
        output: OutputRecord,
        content: Option<ContentRecord>,
    }

    #[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
    #[serde(rename_all = "snake_case")]
    enum PhaseName {
        Off,
        Battle,
        Approach,
        Watch,
        Locked,
        Ended,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct StateRecord {
        phase: PhaseName,
        progress: Option<u32>,
        last: Option<SampleRecord>,
    }

    #[derive(Deserialize, Clone, Copy)]
    #[serde(rename_all = "snake_case")]
    enum ActionName {
        Left,
        Jump,
        Fire,
        AimUp,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SampleRecord {
        held: Vec<ActionName>,
        pressed: Vec<ActionName>,
    }

    #[derive(Deserialize, Clone, Copy)]
    #[serde(rename_all = "snake_case")]
    enum StartName {
        Battle,
        Approach,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct InputRecord {
        actor_turn: bool,
        start: Option<StartName>,
        x: Option<u16>,
        crossfade: Option<u16>,
    }

    #[derive(Deserialize, Clone, Copy)]
    #[serde(rename_all = "snake_case")]
    enum HandoffName {
        StartBattle,
        StartDiscovery,
        StandLeft,
        Release,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct OutputRecord {
        handoffs: Vec<HandoffName>,
        pad: Option<SampleRecord>,
        previous: Option<SampleRecord>,
    }

    #[derive(Deserialize, Clone)]
    #[serde(deny_unknown_fields)]
    struct ContentRecord {
        battle: Option<Vec<HoldRecord>>,
        approach: Option<Vec<HoldRecord>>,
        watch: Option<Vec<HoldRecord>>,
    }

    #[derive(Deserialize, Clone)]
    #[serde(deny_unknown_fields)]
    struct HoldRecord {
        turns: u16,
        held: Vec<ActionName>,
        pressed: Vec<ActionName>,
    }

    const fn action(name: ActionName) -> PadAction {
        match name {
            ActionName::Left => PadAction::Left,
            ActionName::Jump => PadAction::Jump,
            ActionName::Fire => PadAction::Fire,
            ActionName::AimUp => PadAction::AimUp,
        }
    }

    fn set(names: &[ActionName]) -> Set<PadAction> {
        names.iter().map(|&name| (action(name), true)).collect()
    }

    fn sample(record: &SampleRecord) -> PadSample {
        PadSample {
            held: set(&record.held),
            pressed: set(&record.pressed),
        }
    }

    const fn handoff(name: HandoffName) -> Handoff {
        match name {
            HandoffName::StartBattle => Handoff::StartBattle,
            HandoffName::StartDiscovery => Handoff::StartDiscovery,
            HandoffName::StandLeft => Handoff::StandLeft,
            HandoffName::Release => Handoff::Release,
        }
    }

    fn phase(id: &str, record: &StateRecord) -> ScenePhase {
        let place = || TimelinePlace {
            progress: TimelineProgress(
                record
                    .progress
                    .unwrap_or_else(|| panic!("{id}: no progress")),
            ),
            last: sample(
                record
                    .last
                    .as_ref()
                    .unwrap_or_else(|| panic!("{id}: no last pad")),
            ),
        };
        match record.phase {
            PhaseName::Off => ScenePhase::Off,
            PhaseName::Battle => ScenePhase::Battle(place()),
            PhaseName::Approach => ScenePhase::Approach(place()),
            PhaseName::Watch => ScenePhase::Watch(place()),
            PhaseName::Locked => ScenePhase::BattleLocked,
            PhaseName::Ended => ScenePhase::Released,
        }
    }

    fn timeline(id: &str, holds: &[HoldRecord]) -> Vec<PadHold> {
        holds
            .iter()
            .map(|hold| PadHold {
                turns: hold
                    .turns
                    .try_into()
                    .unwrap_or_else(|_| panic!("{id}: a hold lasts at least one turn")),
                sample: PadSample {
                    held: set(&hold.held),
                    pressed: set(&hold.pressed),
                },
            })
            .collect()
    }

    /// The content a vector plays: its own holds, or the invented ones for its id, over the
    /// default invented holds for every timeline it leaves out.
    fn scene_pad(vector: &Vector, invented: &HashMap<String, ContentRecord>) -> ScenePad {
        let fallback = &invented["default"];
        let chosen = vector.content.as_ref().or_else(|| invented.get(&vector.id));
        let pick = |own: Option<&Vec<HoldRecord>>, standard: &Option<Vec<HoldRecord>>| {
            let holds = own
                .or(standard.as_ref())
                .unwrap_or_else(|| panic!("the default invents every timeline"));
            timeline(&vector.id, holds)
        };
        ScenePad {
            battle: pick(chosen.and_then(|c| c.battle.as_ref()), &fallback.battle),
            approach: pick(chosen.and_then(|c| c.approach.as_ref()), &fallback.approach),
            watch: pick(chosen.and_then(|c| c.watch.as_ref()), &fallback.watch),
        }
    }

    /// The vector's readings; reading one it doesn't supply is noted as a failure.
    struct VectorReadings {
        x: Option<u16>,
        crossfade: Option<u16>,
        unsupplied: RefCell<Vec<&'static str>>,
    }

    impl TurnReadings for VectorReadings {
        fn samus_x(&self) -> WholeX {
            WholeX(self.x.unwrap_or_else(|| {
                self.unsupplied.borrow_mut().push("x");
                0
            }))
        }

        fn crossfade(&self) -> CrossfadeCount {
            CrossfadeCount(self.crossfade.unwrap_or_else(|| {
                self.unsupplied.borrow_mut().push("crossfade");
                0
            }))
        }
    }

    fn vectors() -> Vec<Vector> {
        toml::from_str::<VectorFile>(&vector_file::read("screens-playback").unwrap())
            .expect("the vector file reads")
            .vector
    }

    fn invented() -> HashMap<String, ContentRecord> {
        toml::from_str(INVENTED).expect("the invented holds read")
    }

    const fn in_timeline(phase: PhaseName) -> bool {
        matches!(
            phase,
            PhaseName::Battle | PhaseName::Approach | PhaseName::Watch
        )
    }

    fn run(vector: &Vector, invented: &HashMap<String, ContentRecord>) -> Vec<String> {
        let id = &vector.id;
        let scene_pad = scene_pad(vector, invented);
        let mut state = phase(id, &vector.before);
        let start = vector.input.start.map(|start| match start {
            StartName::Battle => SceneStart::Battle,
            StartName::Approach => SceneStart::Discovery,
        });
        let readings = VectorReadings {
            x: vector.input.x,
            crossfade: vector.input.crossfade,
            unsupplied: RefCell::new(Vec::new()),
        };
        let actor_turn: Option<&dyn TurnReadings> = vector.input.actor_turn.then_some(&readings);
        let report = play_turn(&mut state, &scene_pad, start, actor_turn);

        let mut failures = Vec::new();
        for read in readings.unsupplied.borrow().iter() {
            failures.push(format!(
                "{id}: read {read}, which the vector doesn't supply"
            ));
        }
        let after = phase(id, &vector.after);
        if state != after {
            failures.push(format!("{id}: after {state:?}, expected {after:?}"));
        }
        let handoffs: Vec<Handoff> = vector.output.handoffs.iter().map(|&h| handoff(h)).collect();
        if report.handoffs != handoffs {
            failures.push(format!(
                "{id}: handoffs {:?}, expected {handoffs:?}",
                report.handoffs
            ));
        }
        let pad = match (&vector.output.pad, &vector.output.previous) {
            (Some(current), Some(previous)) => Some(DemoPad {
                current: sample(current),
                previous: sample(previous),
            }),
            (None, None) => None,
            _ => panic!("{id}: pad and previous come together"),
        };
        if report.pad != pad {
            failures.push(format!("{id}: pad {:?}, expected {pad:?}", report.pad));
        }
        failures
    }

    #[test]
    fn every_vector_passes() {
        let invented = invented();
        let vectors = vectors();
        let failures: Vec<String> = vectors.iter().flat_map(|v| run(v, &invented)).collect();
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn every_recorded_timeline_snapshot_has_invented_holds() {
        let invented = invented();
        let vectors = vectors();
        for vector in &vectors {
            let recorded = vector.source != "constructed";
            if recorded {
                assert!(vector.content.is_none(), "{}: recorded content", vector.id);
                assert_eq!(
                    in_timeline(vector.before.phase),
                    invented.contains_key(&vector.id),
                    "{}: invented holds exactly where the snapshot reads a timeline",
                    vector.id
                );
            } else {
                assert!(vector.content.is_some(), "{}: no own holds", vector.id);
                assert!(!invented.contains_key(&vector.id), "{}", vector.id);
            }
        }
        for id in invented.keys() {
            assert!(
                id == "default" || vectors.iter().any(|v| &v.id == id),
                "{id}: invented holds for no vector"
            );
        }
    }

    #[test]
    fn the_catalogue_matches_the_vectors() {
        let vectors = vectors();
        let catalogue = catalogue();
        for vector in &vectors {
            for entry in &vector.entries {
                assert!(
                    catalogue.values().any(|b| format!("{:?}", b.id) == *entry),
                    "{}: unknown entry {entry}",
                    vector.id
                );
            }
        }
        for (id, behaviour) in &catalogue {
            assert_eq!(behaviour.id, id);
            let name = format!("{id:?}");
            if behaviour.checked == Checked::ByVectors {
                assert!(
                    vectors.iter().any(|v| v.entries.contains(&name)),
                    "{name}: no vector"
                );
            }
        }
        let spec_entries: Vec<&str> = SPEC
            .lines()
            .filter_map(|line| line.strip_prefix("### "))
            .collect();
        let catalogued: Vec<String> = catalogue.values().map(|b| format!("{:?}", b.id)).collect();
        assert_eq!(catalogued, spec_entries, "one behaviour per spec entry");
    }

    /// The importer's rules on content, which the invented holds keep too: each timeline
    /// nonempty, each fresh press held, and each discovery timeline ending on Left alone.
    fn check_content(id: &str, scene_pad: &ScenePad) {
        let timelines = [&scene_pad.battle, &scene_pad.approach, &scene_pad.watch];
        for timeline in timelines {
            assert!(!timeline.is_empty(), "{id}: an empty timeline");
            for hold in timeline {
                let pressed_held = hold
                    .sample
                    .pressed
                    .iter()
                    .all(|(action, &pressed)| !pressed || hold.sample.held[action]);
                assert!(pressed_held, "{id}: a fresh press not held");
            }
        }
        for discovery in [&scene_pad.approach, &scene_pad.watch] {
            let last = discovery.last().map(|hold| hold.sample.held);
            assert_eq!(
                last,
                Some(PadSample::walking_left().held),
                "{id}: a discovery timeline not ending on Left"
            );
        }
    }

    /// The turns a timeline lasts.
    fn length(timeline: &[PadHold]) -> u32 {
        timeline
            .iter()
            .map(|hold| u32::from(hold.turns.get()))
            .sum()
    }

    #[test]
    fn every_before_state_lies_in_valid_content() {
        let invented = invented();
        for vector in vectors() {
            let scene_pad = scene_pad(&vector, &invented);
            check_content(&vector.id, &scene_pad);
            let (place, timeline) = match phase(&vector.id, &vector.before) {
                ScenePhase::Battle(place) => (place, &scene_pad.battle),
                ScenePhase::Approach(place) => (place, &scene_pad.approach),
                ScenePhase::Watch(place) => (place, &scene_pad.watch),
                ScenePhase::Off | ScenePhase::BattleLocked | ScenePhase::Released => continue,
            };
            assert!(
                place.progress.0 <= length(timeline),
                "{}: progress past the timeline",
                vector.id
            );
        }
    }

    #[test]
    fn the_state_saves_and_restores() {
        #[derive(serde::Serialize, Deserialize, PartialEq, Debug)]
        struct Saved {
            phase: ScenePhase,
        }
        for vector in vectors() {
            let saved = Saved {
                phase: phase(&vector.id, &vector.before),
            };
            let text = toml::to_string(&saved).expect("the state writes");
            let restored: Saved = toml::from_str(&text).expect("the state reads");
            assert_eq!(restored, saved, "{}", vector.id);
        }
    }
}
