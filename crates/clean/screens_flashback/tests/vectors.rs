//! Runs every vector of the screens-flashback vector file through the model, and checks the
//! catalogue against the vectors.

#[cfg(test)]
mod runner {
    use core::num::Wrapping;
    use std::collections::BTreeMap;

    use enum_map::Enum;
    use screens_flashback::{
        Art, BehaviourId, ContactLine, Drawing, Effects, HitThreshold, HurtColours, IntroRequest,
        ReturnDissolve, RingKnockback, RoomPlay, Rules, ShakeHold, ShakeTravel, ShotCandidate,
        ShotId, ShotKind, admit_start, beat, catalogue, pictures,
    };
    use serde::Deserialize;
    use types::flashback::{
        Animation, Brain, BrainPhase, BreathStep, Burst, BurstPlace, BurstStep, DrawRank,
        EncounterState, FlashBeats, HoldBeats, PairSchedule, PendingStart, PriorHits, Ring,
        RingPair, RingPath, RingStep, RoomBackgroundY, ShakeBeats,
    };
    use types::{Pair, Parity, Side};

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/screens-flashback.toml");

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
        rules: Option<RulesDoc>,
        steps: Option<Vec<InputsDoc>>,
        before: Option<StateDoc>,
        inputs: Option<InputsDoc>,
        expect: ExpectDoc,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct RulesDoc {
        contact: Option<ContactDoc>,
        shake_travel: Option<u8>,
        shake_hold: Option<u16>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ContactDoc {
        limit: Option<u8>,
        hits: Option<u8>,
    }

    #[derive(Deserialize, Default, Clone)]
    #[serde(deny_unknown_fields)]
    struct InputsDoc {
        advance: Option<bool>,
        start: Option<bool>,
        start_offset: Option<i16>,
        odd: Option<bool>,
        return_zero: Option<bool>,
        samus_x: Option<u16>,
        shots: Option<Vec<ShotDoc>>,
    }

    #[derive(Deserialize, Clone)]
    #[serde(deny_unknown_fields)]
    struct ShotDoc {
        id: u8,
        kind: String,
        x: u16,
    }

    /// A field that is either a word ("none") or a value.
    #[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
    #[serde(untagged)]
    enum Maybe<T> {
        Word(String),
        Value(T),
    }

    fn none<T>() -> Maybe<T> {
        Maybe::Word("none".to_owned())
    }

    fn maybe<T>(value: Option<T>) -> Maybe<T> {
        value.map_or_else(none, Maybe::Value)
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Default)]
    #[serde(deny_unknown_fields)]
    struct StateDoc {
        pending_start: Option<bool>,
        brain: Option<Maybe<BrainDoc>>,
        offset: Option<i16>,
        pair: Option<Maybe<PairDoc>>,
        rings: Option<BTreeMap<String, RingDoc>>,
        bursts: Option<BTreeMap<String, BurstDoc>>,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
    #[serde(deny_unknown_fields)]
    struct BrainDoc {
        phase: String,
        progress: Option<u16>,
        flash: u8,
        step: usize,
        hold: u16,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
    #[serde(deny_unknown_fields)]
    struct PairDoc {
        which: String,
        hold: u16,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
    #[serde(deny_unknown_fields)]
    struct RingDoc {
        x: u32,
        y: u32,
        step: usize,
        hold: u16,
        rank: u8,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
    #[serde(deny_unknown_fields)]
    struct BurstDoc {
        x: u16,
        y: u16,
        step: i8,
        hold: u16,
        rank: u8,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ExpectDoc {
        state: Option<StateDoc>,
        output: Option<OutputDoc>,
    }

    type DrawDoc = (String, u16, u16, u8, u8);

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct OutputDoc {
        draw: Option<Vec<DrawDoc>>,
        impact: Option<Maybe<u8>>,
        palette: Option<String>,
        cue: Option<bool>,
        hide: Option<bool>,
        ring_hit: Option<Maybe<RingHitDoc>>,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
    #[serde(deny_unknown_fields)]
    struct RingHitDoc {
        direction: String,
        invincible: u8,
        knockback: u8,
    }

    const RING_NAMES: [(&str, RingPath); 4] = [
        ("first_toward", RingPath::FirstToward),
        ("first_across", RingPath::FirstAcross),
        ("second_toward", RingPath::SecondToward),
        ("second_across", RingPath::SecondAcross),
    ];

    const BURST_NAMES: [(&str, BurstPlace); 8] = [
        ("small_near", BurstPlace::Near),
        ("small_low", BurstPlace::Low),
        ("small_high", BurstPlace::High),
        ("large_center", BurstPlace::Centre),
        ("large_upper_right", BurstPlace::UpperRight),
        ("large_lower_left", BurstPlace::LowerLeft),
        ("large_upper_left", BurstPlace::UpperLeft),
        ("large_lower_right", BurstPlace::LowerRight),
    ];

    fn by_name<K: Copy>(names: &[(&str, K)], name: &str) -> K {
        names
            .iter()
            .find(|(known, _)| *known == name)
            .unwrap_or_else(|| panic!("unknown name {name}"))
            .1
    }

    fn name_of<K: Copy + PartialEq>(names: &[(&'static str, K)], key: K) -> String {
        names
            .iter()
            .find(|(_, known)| *known == key)
            .expect("every key is named")
            .0
            .to_owned()
    }

    fn hold(beats: u16) -> HoldBeats {
        HoldBeats::new(beats).expect("a live hold")
    }

    fn rules_of(doc: Option<&RulesDoc>) -> Rules {
        let mut rules = Rules::REFERENCE;
        let Some(doc) = doc else { return rules };
        if let Some(contact) = &doc.contact {
            if let Some(limit) = contact.limit {
                rules.contact.limit = ContactLine(limit);
            }
            if let Some(hits) = contact.hits {
                rules.contact.hits = HitThreshold::new(hits).expect("a hit threshold");
            }
        }
        if let Some(travel) = doc.shake_travel {
            rules.shake_travel = ShakeTravel::new(travel).expect("a shake travel");
        }
        if let Some(beats) = doc.shake_hold {
            rules.shake_hold = ShakeHold::new(beats).expect("a shake hold");
        }
        rules
    }

    fn brain_of(doc: &BrainDoc) -> Brain {
        let progress = doc.progress.unwrap_or(0);
        let phase = match doc.phase.as_str() {
            "endures" => BrainPhase::Endures {
                hits: PriorHits(u8::try_from(progress).expect("hits fit a byte")),
            },
            "shaking" => BrainPhase::Shaking {
                elapsed: ShakeBeats(progress),
            },
            "returning" => BrainPhase::Returning,
            other => panic!("unknown phase {other}"),
        };
        Brain {
            phase,
            flash: FlashBeats::new(doc.flash).expect("a flash"),
            breath: Animation {
                step: BreathStep::from_usize(doc.step),
                hold: hold(doc.hold),
            },
        }
    }

    fn pair_of(doc: &PairDoc) -> PairSchedule {
        let pair = match doc.which.as_str() {
            "first" => RingPair::First,
            "second" => RingPair::Second,
            other => panic!("unknown pair {other}"),
        };
        PairSchedule {
            pair,
            hold: hold(doc.hold),
        }
    }

    fn burst_step_of(step: i8) -> BurstStep {
        let index = usize::try_from(step.wrapping_add(1)).expect("a burst step from -1");
        BurstStep::from_usize(index)
    }

    fn state_of(doc: &StateDoc) -> EncounterState {
        let mut state = EncounterState::before_encounter(RoomBackgroundY(doc.offset.unwrap_or(0)));
        if doc.pending_start == Some(true) {
            state.start = PendingStart::Pending;
        }
        if let Some(Maybe::Value(brain)) = &doc.brain {
            state.brain = Some(brain_of(brain));
        }
        if let Some(Maybe::Value(pair)) = &doc.pair {
            state.next_pair = Some(pair_of(pair));
        }
        for (name, ring) in doc.rings.iter().flatten() {
            state.rings[by_name(&RING_NAMES, name)] = Some(Ring {
                position: Pair {
                    x: Wrapping(ring.x),
                    y: Wrapping(ring.y),
                },
                clip: Animation {
                    step: RingStep::from_usize(ring.step),
                    hold: hold(ring.hold),
                },
                rank: DrawRank(ring.rank),
            });
        }
        for (name, burst) in doc.bursts.iter().flatten() {
            state.bursts[by_name(&BURST_NAMES, name)] = Some(Burst {
                at: Pair {
                    x: Wrapping(burst.x),
                    y: Wrapping(burst.y),
                },
                clip: Animation {
                    step: burst_step_of(burst.step),
                    hold: hold(burst.hold),
                },
                rank: DrawRank(burst.rank),
            });
        }
        state
    }

    fn brain_doc(brain: &Brain) -> BrainDoc {
        let (phase, progress) = match brain.phase {
            BrainPhase::Endures { hits } => ("endures", Some(u16::from(hits.0))),
            BrainPhase::Shaking { elapsed } => ("shaking", Some(elapsed.0)),
            BrainPhase::Returning => ("returning", None),
        };
        BrainDoc {
            phase: phase.to_owned(),
            progress,
            flash: brain.flash.get(),
            step: brain.breath.step.into_usize(),
            hold: brain.breath.hold.get(),
        }
    }

    fn doc_of(state: &EncounterState) -> StateDoc {
        let pair = state.next_pair.map(|schedule| PairDoc {
            which: match schedule.pair {
                RingPair::First => "first",
                RingPair::Second => "second",
            }
            .to_owned(),
            hold: schedule.hold.get(),
        });
        let rings = state
            .rings
            .iter()
            .filter_map(|(path, ring)| {
                ring.map(|ring| {
                    let doc = RingDoc {
                        x: ring.position.x.0,
                        y: ring.position.y.0,
                        step: ring.clip.step.into_usize(),
                        hold: ring.clip.hold.get(),
                        rank: ring.rank.0,
                    };
                    (name_of(&RING_NAMES, path), doc)
                })
            })
            .collect();
        let bursts = state
            .bursts
            .iter()
            .filter_map(|(place, burst)| {
                burst.map(|burst| {
                    let step = i8::try_from(burst.clip.step.into_usize())
                        .expect("a small step")
                        .wrapping_sub(1);
                    let doc = BurstDoc {
                        x: burst.at.x.0,
                        y: burst.at.y.0,
                        step,
                        hold: burst.clip.hold.get(),
                        rank: burst.rank.0,
                    };
                    (name_of(&BURST_NAMES, place), doc)
                })
            })
            .collect();
        StateDoc {
            pending_start: Some(state.start == PendingStart::Pending),
            brain: Some(maybe(state.brain.as_ref().map(brain_doc))),
            offset: Some(state.background_y.0),
            pair: Some(maybe(pair)),
            rings: Some(rings),
            bursts: Some(bursts),
        }
    }

    fn draw_doc(drawing: Drawing) -> DrawDoc {
        let name = match drawing.art {
            Art::MotherBrain => "brain".to_owned(),
            Art::Ring(path) => name_of(&RING_NAMES, path),
            Art::Burst(place) => name_of(&BURST_NAMES, place),
        };
        (
            name,
            drawing.at.x.0,
            drawing.at.y.0,
            drawing.image.0,
            drawing.palette.0,
        )
    }

    fn knockback_doc(knockback: RingKnockback) -> RingHitDoc {
        RingHitDoc {
            direction: match knockback.direction {
                Side::Left => "left",
                Side::Right => "right",
            }
            .to_owned(),
            invincible: knockback.invincible_beats,
            knockback: knockback.knockback_beats,
        }
    }

    fn output_doc(effects: Option<Effects>, state: &EncounterState) -> OutputDoc {
        let palette = effects
            .and_then(|effects| effects.palette)
            .map_or("none", |colours| match colours {
                HurtColours::White => "white",
                HurtColours::Normal => "base",
            });
        let intro = effects.and_then(|effects| effects.intro);
        OutputDoc {
            draw: Some(pictures(state).into_iter().map(draw_doc).collect()),
            impact: Some(maybe(
                effects.and_then(|effects| effects.missile_impact.map(|shot| shot.0)),
            )),
            palette: Some(palette.to_owned()),
            cue: Some(intro == Some(IntroRequest::CuePageTwo)),
            hide: Some(intro == Some(IntroRequest::HideSamus)),
            ring_hit: Some(maybe(
                effects.and_then(|effects| effects.ring_knockback.map(knockback_doc)),
            )),
        }
    }

    fn shot_of(doc: &ShotDoc) -> ShotCandidate {
        let kind = match doc.kind.as_str() {
            "missile" => ShotKind::Missile,
            "super" => ShotKind::SuperMissile,
            "beam" => ShotKind::Beam,
            other => panic!("unknown shot kind {other}"),
        };
        ShotCandidate {
            id: ShotId(doc.id),
            kind,
            x: Wrapping(doc.x),
        }
    }

    fn run_step(state: &mut EncounterState, rules: Rules, inputs: &InputsDoc) -> Option<Effects> {
        if inputs.start == Some(true) {
            admit_start(state, RoomBackgroundY(inputs.start_offset.unwrap_or(8)));
        }
        if inputs.advance == Some(false) {
            return None;
        }
        let parity = if inputs.odd == Some(true) {
            Parity::Odd
        } else {
            Parity::Even
        };
        let dissolve = if inputs.return_zero == Some(true) {
            ReturnDissolve::ReachesZero
        } else {
            ReturnDissolve::Continuing
        };
        let play = RoomPlay {
            samus_x: Wrapping(inputs.samus_x.unwrap_or(155)),
            shots: inputs.shots.iter().flatten().map(shot_of).collect(),
        };
        Some(beat(state, rules, parity, dissolve, &play))
    }

    fn compare<T: PartialEq + core::fmt::Debug>(
        failures: &mut Vec<String>,
        id: &str,
        field: &str,
        expected: Option<&T>,
        actual: Option<&T>,
    ) {
        if let Some(expected) = expected
            && Some(expected) != actual
        {
            failures.push(format!(
                "{id}: {field}: expected {expected:?}, got {actual:?}"
            ));
        }
    }

    /// Compares each named field the expectation gives with the actual one.
    macro_rules! fields {
        ($failures:expr, $id:expr, $expected:expr, $actual:expr; $($field:ident),+) => {
            $(compare(
                $failures,
                $id,
                stringify!($field),
                $expected.$field.as_ref(),
                $actual.$field.as_ref(),
            );)+
        };
    }

    fn check(vector: &Vector, failures: &mut Vec<String>) {
        assert!(
            ["recorded", "constructed"].contains(&vector.source.as_str()),
            "{}: unknown source",
            vector.id
        );
        let rules = rules_of(vector.rules.as_ref());
        let mut state = vector
            .before
            .as_ref()
            .map_or_else(|| state_of(&StateDoc::default()), state_of);
        let steps = vector
            .steps
            .clone()
            .unwrap_or_else(|| vec![vector.inputs.clone().unwrap_or_default()]);
        let mut effects = None;
        for inputs in &steps {
            effects = run_step(&mut state, rules, inputs);
        }
        let id = vector.id.as_str();
        if let Some(expected) = &vector.expect.state {
            let actual = doc_of(&state);
            fields!(failures, id, expected, actual; pending_start, brain, offset, pair, rings, bursts);
        }
        if let Some(expected) = &vector.expect.output {
            let actual = output_doc(effects, &state);
            fields!(failures, id, expected, actual; draw, impact, palette, cue, hide, ring_hit);
        }
    }

    fn vectors() -> Vec<Vector> {
        toml::from_str::<VectorFile>(VECTORS)
            .expect("the vector file parses")
            .vector
    }

    #[test]
    fn every_vector_passes() {
        let vectors = vectors();
        let mut failures = Vec::new();
        for vector in &vectors {
            check(vector, &mut failures);
        }
        assert!(
            failures.is_empty(),
            "{} of {} vectors' fields fail:\n{}",
            failures.len(),
            vectors.len(),
            failures.join("\n")
        );
    }

    fn id_named(name: &str) -> BehaviourId {
        catalogue()
            .into_iter()
            .map(|behaviour| behaviour.id)
            .find(|id| format!("{id:?}") == name)
            .unwrap_or_else(|| panic!("unknown entry {name}"))
    }

    #[test]
    fn every_vector_names_known_entries() {
        for vector in vectors() {
            for entry in &vector.entries {
                id_named(entry);
            }
        }
    }

    #[test]
    fn every_entry_checked_by_vectors_has_one() {
        let named: Vec<BehaviourId> = vectors()
            .iter()
            .flat_map(|vector| vector.entries.iter().map(|entry| id_named(entry)))
            .collect();
        for behaviour in catalogue() {
            if behaviour.checked == types::Checked::ByVectors {
                assert!(
                    named.contains(&behaviour.id),
                    "{:?} has no vector",
                    behaviour.id
                );
            }
        }
    }

    #[test]
    fn the_catalogue_has_one_entry_per_id() {
        let catalogue = catalogue();
        assert_eq!(
            catalogue.len(),
            enum_map::EnumMap::<BehaviourId, ()>::default().len()
        );
        for (index, behaviour) in catalogue.iter().enumerate() {
            assert_eq!(behaviour.id, BehaviourId::from_usize(index));
        }
    }
}
