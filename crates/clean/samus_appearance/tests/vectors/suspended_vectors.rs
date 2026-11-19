//! Complete continuation checks, in addition to the specified observation subsequences.
use super::{
    OpportunityVector, RulesTable, Vector, VectorFile, contact_clips, figure, inputs,
    owned_opportunity, read_file, retained, rules, state,
};
use samus_appearance::{
    AnimationDecision, AnimationInputs, Answer, AppearanceRead, OfferedDecision, PublishedSound,
    Receipt, SuspendedAppearance, WorkStep,
};
use samus_appearance::{Clip, Frames, Paces, State, Step};
use serde::Deserialize;
use std::collections::BTreeMap;
use types::{BoostStage, Set};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SuspendedVector {
    id: String,
    source: String,
    base_kind: String,
    base: String,
    pub(super) entries: Vec<String>,
    #[serde(default)]
    rules: RulesTable,
    save_points: String,
    observations: Vec<Observation>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Observation {
    event: String,
    clip: Option<String>,
    answer: Option<String>,
    before_step: Option<u8>,
    before_left: Option<u8>,
    step: Option<u8>,
    left: Option<u8>,
    stage: Option<u8>,
    laps: Option<u8>,
    contact: Option<bool>,
    sounds: Option<Vec<String>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ResumeCase {
    id: String,
    source: String,
    base_kind: String,
    base: String,
    pub(super) entries: Vec<String>,
    boundary: Boundary,
    selection_replace: Option<SelectionReplace>,
    contact_replace: Option<ContactReplace>,
    restoration: String,
    expected: ResumeExpected,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Boundary {
    opportunity: String,
    point: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectionReplace {
    pose: String,
    clip: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContactReplace {
    cinematic: Option<bool>,
    boss: Option<bool>,
    special_colours: Option<bool>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResumeExpected {
    played_step: Option<u8>,
    played_left: Option<u8>,
    selected_pose: Option<String>,
    answer: Option<String>,
    step: u8,
    left: u8,
    stage: u8,
    laps: u8,
    initialization_effects: Option<usize>,
    sounds: Option<Vec<String>>,
    graphics: Option<usize>,
    contact_reads: Option<usize>,
}

fn animation_work(
    vector: &Vector,
    clips: &BTreeMap<String, Clip>,
    paces: &Paces,
) -> SuspendedAppearance {
    let i = inputs(&vector.inputs);
    let data = AnimationInputs {
        inputs: i,
        selected_from: i.figure,
        current_clip: clips[&vector.inputs.clip].clone(),
        selected_clip: clips[vector
            .inputs
            .clip_after
            .as_ref()
            .unwrap_or(&vector.inputs.clip)]
        .clone(),
        paces: paces.clone(),
        rules: rules(&vector.rules),
    };
    let ahead = samus_appearance::Ahead::of(retained(&vector.before).playhead, &data.current_clip);
    let mut work = SuspendedAppearance::new(
        retained(&vector.before),
        Some(data),
        SuspendedAppearance::animation_steps(),
    );
    work.ahead = Some(ahead);
    work
}
fn animation_base<'a>(file: &'a VectorFile, id: &str) -> &'a Vector {
    file.vector
        .iter()
        .find(|v| v.id == id)
        .expect("named animation base")
}
pub(super) fn opportunity_base<'a>(file: &'a VectorFile, id: &str) -> &'a OpportunityVector {
    file.opportunity
        .iter()
        .find(|v| v.id == id)
        .expect("named opportunity base")
}
pub(super) fn make_work(file: &VectorFile, kind: &str, id: &str) -> SuspendedAppearance {
    let (_, clips, paces) = read_file();
    let contacts = contact_clips(file);
    match kind {
        "Animation" => animation_work(animation_base(file, id), &clips, &paces),
        "Opportunity" => {
            let v = opportunity_base(file, id);
            SuspendedAppearance::new(
                retained(&v.before),
                None,
                vec![WorkStep::Appearance(owned_opportunity(v, &contacts))],
            )
        }
        "Work" => {
            let base = file
                .appearance_work
                .iter()
                .find(|v| v.id == id)
                .expect("named work base");
            let mut work = base.animation_vector.as_ref().map_or_else(
                || {
                    SuspendedAppearance::new(
                        retained(&opportunity_base(file, &base.opportunities[0]).before),
                        None,
                        Vec::new(),
                    )
                },
                |id| animation_work(animation_base(file, id), &clips, &paces),
            );
            work.steps.extend(base.opportunities.iter().map(|id| {
                WorkStep::Appearance(owned_opportunity(opportunity_base(file, id), &contacts))
            }));
            work
        }
        _ => panic!("unknown base kind {kind}"),
    }
}

/// Actually serialize, discard the original, then restore a fresh instance. No clone checkpoint.
pub(super) fn restore(work: SuspendedAppearance) -> SuspendedAppearance {
    let text = toml::to_string(&work).expect("complete interrupted work serializes");
    drop(work);
    toml::from_str(&text).expect("fresh interrupted work deserializes")
}
pub(super) fn checkpoint(
    work: SuspendedAppearance,
    expected: &SuspendedAppearance,
) -> SuspendedAppearance {
    let text = toml::to_string(&work).expect("checkpoint serializes completely");
    drop(work);
    let mut continuation: SuspendedAppearance = toml::from_str(&text).expect("fresh continuation");
    continuation.finish();
    assert_eq!(
        &continuation, expected,
        "every save point proves complete continuation"
    );
    toml::from_str(&text).expect("fresh interrupted instance for next opportunity")
}

pub(super) fn sound_names(work: &SuspendedAppearance) -> Vec<String> {
    work.sounds
        .iter()
        .map(|s| match s {
            PublishedSound::Animation(s) => format!("{s:?}"),
            PublishedSound::Appearance(s) => format!("{s:?}"),
        })
        .collect()
}
fn check_base(file: &VectorFile, kind: &str, id: &str, work: &SuspendedAppearance) {
    let (expected, sounds, contacts) = match kind {
        "Animation" => {
            let v = animation_base(file, id);
            assert_eq!(
                work.ahead,
                Some(samus_appearance::Ahead {
                    end: super::end(&v.ahead.end),
                    turned_far: v.ahead.turned_far
                })
            );
            (&v.after, &v.after.sounds, Vec::new())
        }
        "Opportunity" => {
            let v = opportunity_base(file, id);
            assert_eq!(work.state, retained(&v.after));
            assert_eq!(sound_names(work), v.expected.sounds);
            let expected = owned_opportunity(v, &contact_clips(file))
                .read()
                .contact
                .into_iter()
                .collect::<Vec<_>>();
            assert_eq!(work.graphics, expected);
            return;
        }
        "Work" => {
            let v = file
                .appearance_work
                .iter()
                .find(|v| v.id == id)
                .expect("work base");
            let contacts = contact_clips(file);
            let graphics = v
                .opportunities
                .iter()
                .filter_map(|id| {
                    owned_opportunity(opportunity_base(file, id), &contacts)
                        .read()
                        .contact
                })
                .collect();
            (&v.after, &v.sounds, graphics)
        }
        _ => panic!("unknown base kind"),
    };
    assert_eq!(
        work.state,
        state(expected.step, expected.left, expected.stage, expected.laps),
        "{id}: complete State"
    );
    assert_eq!(sound_names(work), *sounds, "{id}: complete ordered sounds");
    assert_eq!(work.graphics, contacts, "{id}: complete graphics");
}

fn partial_state(o: &Observation, s: State) -> bool {
    o.step.is_none_or(|v| s.playhead.step == Step(v))
        && o.left.is_none_or(|v| s.playhead.time_left == Frames(v))
        && o.stage
            .is_none_or(|v| s.stage == BoostStage::ALL[usize::from(v)])
        && o.laps
            .is_none_or(|v| s.laps_left == state(0, 0, 0, v).laps_left)
}
fn answer_name(answer: Answer) -> String {
    format!("{answer:?}")
}
fn offered_matches(o: &Observation, d: &OfferedDecision, clips: &BTreeMap<String, Clip>) -> bool {
    let event = format!("{:?}", d.rule);
    if o.event != event {
        return false;
    }
    let s = if d.rule == AnimationDecision::ClipPlaying {
        d.before
    } else {
        d.after
    };
    partial_state(o, s)
        && o.before_step
            .is_none_or(|v| d.before.playhead.step == Step(v))
        && o.before_left
            .is_none_or(|v| d.before.playhead.time_left == Frames(v))
        && o.clip.as_ref().is_none_or(|v| clips[v] == d.clip)
        && o.answer
            .as_ref()
            .is_none_or(|v| *v == answer_name(d.answer))
        && o.contact.is_none()
        && o.sounds.is_none()
}
fn completed_matches(o: &Observation, r: &Receipt) -> bool {
    match r {
        Receipt::Animation(d) => {
            let event = match d.rule {
                AnimationDecision::ClipPlaying => "ClipPlayed",
                AnimationDecision::SelectedClip => "ClipInitialized",
                AnimationDecision::DashBegan
                | AnimationDecision::StandingRestart
                | AnimationDecision::WallJumpReady
                | AnimationDecision::FastFall
                | AnimationDecision::RisingHold
                | AnimationDecision::DashLoss => "",
            };
            o.event == event
                && partial_state(o, d.after)
                && o.clip.is_none()
                && o.answer.is_none()
                && o.sounds.as_ref().is_none_or(|sounds| {
                    *sounds
                        == d.sound
                            .map(|s| format!("{s:?}"))
                            .into_iter()
                            .collect::<Vec<_>>()
                })
        }
        Receipt::Appearance {
            input,
            state,
            output,
        } => {
            let event = match input {
                AppearanceRead::Contact { .. } => "ContactRead",
                AppearanceRead::Selection { .. } => "SelectionRead",
                AppearanceRead::Draw { .. } => "NormalDraw",
                AppearanceRead::Passage { .. } => "PassageEntry",
            };
            o.event == event
                && partial_state(o, *state)
                && o.contact.is_none_or(|v| v == output.contact.is_some())
                && o.sounds.as_ref().is_none_or(|sounds| {
                    *sounds
                        == output
                            .sound
                            .map(|s| format!("{s:?}"))
                            .into_iter()
                            .collect::<Vec<_>>()
                })
                && o.clip.is_none()
                && o.answer.is_none()
        }
    }
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "report complete additive vector and save-point results"
)]
fn every_suspended_vector() {
    let (file, clips, _) = read_file();
    let mut saves: usize = 0;
    for v in &file.suspended_update {
        assert_eq!(v.source, "constructed");
        assert_eq!(v.save_points, "AllOfferedAndCompleted");
        let mut expected = make_work(&file, &v.base_kind, &v.base);
        if let Some(a) = &expected.animation {
            assert_eq!(rules(&v.rules), a.rules);
        }
        expected.finish();
        check_base(&file, &v.base_kind, &v.base, &expected);
        let mut work = make_work(&file, &v.base_kind, &v.base);
        let mut observed: usize = 0;
        loop {
            let _ = work.offer();
            if let Some(d) = &work.pending {
                if v.observations
                    .get(observed)
                    .is_some_and(|o| offered_matches(o, d, &clips))
                {
                    observed = observed.saturating_add(1);
                }
                work = checkpoint(work, &expected);
                saves = saves.saturating_add(1);
            }
            if !work.complete() {
                break;
            }
            if v.observations
                .get(observed)
                .is_some_and(|o| completed_matches(o, work.completed.last().expect("receipt")))
            {
                observed = observed.saturating_add(1);
            }
            work = checkpoint(work, &expected);
            saves = saves.saturating_add(1);
        }
        assert_eq!(
            observed,
            v.observations.len(),
            "{}: complete observation subsequence",
            v.id
        );
        assert_eq!(
            work, expected,
            "{}: complete restored work and outputs",
            v.id
        );
        let completed_text = toml::to_string(&work).expect("completed work saves");
        work = restore(work);
        work.finish();
        assert_eq!(
            toml::to_string(&work).expect("completed work saves"),
            completed_text,
            "completed effects never repeat"
        );
    }
    println!(
        "{} suspended vectors passed at {saves} offered/completed save points",
        file.suspended_update.len()
    );
}

fn at_boundary(work: &mut SuspendedAppearance, boundary: &Boundary) {
    match boundary.opportunity.as_str() {
        "SelectedClip" => {
            while !matches!(
                work.steps.get(work.consumed),
                Some(WorkStep::Animation(AnimationDecision::SelectedClip))
            ) {
                assert!(work.complete());
            }
            match boundary.point.as_str() {
                "BeforeDecision" => {}
                "AfterDecisionBeforeEffect" => {
                    let _ = work.offer();
                }
                _ => panic!("selection boundary"),
            }
        }
        "Contact" => match boundary.point.as_str() {
            "BeforeRead" => {}
            "AfterCompletedRead" => {
                assert!(work.complete());
            }
            _ => panic!("contact boundary"),
        },
        _ => panic!("unknown boundary"),
    }
}
fn replace(work: &mut SuspendedAppearance, case: &ResumeCase, clips: &BTreeMap<String, Clip>) {
    if let Some(s) = &case.selection_replace {
        let a = work.animation.as_mut().expect("animation inputs");
        a.inputs.figure_after = figure(&s.pose);
        a.selected_clip = clips[&s.clip].clone();
    }
    if let Some(s) = &case.contact_replace {
        for step in &mut work.steps {
            if let WorkStep::Appearance(AppearanceRead::Contact { suppressors, .. }) = step {
                change_suppressors(suppressors, s);
            }
        }
    }
}
fn change_suppressors(
    suppressors: &mut Set<samus_appearance::ContactSuppressor>,
    s: &ContactReplace,
) {
    for (kind, value) in [
        (samus_appearance::ContactSuppressor::Cinematic, s.cinematic),
        (samus_appearance::ContactSuppressor::Boss, s.boss),
        (
            samus_appearance::ContactSuppressor::SpecialColours,
            s.special_colours,
        ),
    ] {
        if let Some(value) = value {
            suppressors[kind] = value;
        }
    }
}
fn check_resume(case: &ResumeCase, work: &SuspendedAppearance) {
    let e = &case.expected;
    assert_eq!(
        work.state,
        state(e.step, e.left, e.stage, e.laps),
        "{}: complete changed State",
        case.id
    );
    assert_eq!(sound_names(work), e.sounds.clone().unwrap_or_default());
    assert_eq!(work.graphics.len(), e.graphics.unwrap_or(0));
    let played = work.completed.iter().find_map(|r| match r {
        Receipt::Animation(d) if d.rule == AnimationDecision::ClipPlaying => Some(d),
        _ => None,
    });
    if let Some(p) = e.played_step {
        assert_eq!(played.expect("played receipt").after.playhead.step, Step(p));
    }
    if let Some(p) = e.played_left {
        assert_eq!(
            played.expect("played receipt").after.playhead.time_left,
            Frames(p)
        );
    }
    let initializations: Vec<_> = work
        .completed
        .iter()
        .filter_map(|r| match r {
            Receipt::Animation(d) if d.rule == AnimationDecision::SelectedClip => Some(d),
            _ => None,
        })
        .collect();
    if let Some(count) = e.initialization_effects {
        assert_eq!(initializations.len(), count);
    }
    if let Some(p) = &e.selected_pose {
        assert_eq!(initializations[0].inputs.figure_after, figure(p));
    }
    if let Some(a) = &e.answer {
        assert_eq!(answer_name(initializations[0].answer), *a);
    }
    if let Some(count) = e.contact_reads {
        assert_eq!(
            work.completed
                .iter()
                .filter(|r| matches!(
                    r,
                    Receipt::Appearance {
                        input: AppearanceRead::Contact { .. },
                        ..
                    }
                ))
                .count(),
            count
        );
    }
}
#[test]
#[expect(clippy::print_stdout, reason = "report changed-resume witness results")]
fn every_changed_resume() {
    let (file, clips, _) = read_file();
    for case in &file.resume_case {
        assert_eq!(case.source, "constructed");
        assert_eq!(case.restoration, "SerializeDropDeserialize");
        let mut uninterrupted = make_work(&file, &case.base_kind, &case.base);
        at_boundary(&mut uninterrupted, &case.boundary);
        replace(&mut uninterrupted, case, &clips);
        uninterrupted.finish();
        check_resume(case, &uninterrupted);
        let mut work = make_work(&file, &case.base_kind, &case.base);
        at_boundary(&mut work, &case.boundary);
        work = restore(work);
        replace(&mut work, case, &clips);
        loop {
            let _ = work.offer();
            work = restore(work);
            if !work.complete() {
                break;
            }
            work = restore(work);
        }
        assert_eq!(
            work, uninterrupted,
            "{}: full changed continuation",
            case.id
        );
        check_resume(case, &work);
    }
    println!("{} changed-resume vectors passed", file.resume_case.len());
}

#[test]
fn every_original_case_also_survives_all_save_points() {
    let (file, _, _) = read_file();
    let cases = file
        .vector
        .iter()
        .map(|v| ("Animation", v.id.as_str()))
        .chain(
            file.opportunity
                .iter()
                .map(|v| ("Opportunity", v.id.as_str())),
        )
        .chain(file.appearance_work.iter().map(|v| ("Work", v.id.as_str())));
    for (kind, id) in cases {
        let mut expected = make_work(&file, kind, id);
        expected.finish();
        check_base(&file, kind, id, &expected);
        let mut work = make_work(&file, kind, id);
        loop {
            let _ = work.offer();
            work = checkpoint(work, &expected);
            if !work.complete() {
                break;
            }
            work = checkpoint(work, &expected);
        }
        assert_eq!(work, expected);
    }
}

#[test]
fn faulty_future_inputs_and_repeated_or_reordered_effects_produce_distinguishable_results() {
    let (file, _, _) = read_file();
    let mut correct = make_work(&file, "Animation", "C-kept-run-advance");
    correct.finish();
    let mut future = make_work(&file, "Animation", "C-kept-run-advance");
    let a = future.animation.as_mut().expect("animation");
    a.current_clip = a.selected_clip.clone();
    future.finish();
    assert_ne!(
        future.state, correct.state,
        "old clip's newly read hold is essential"
    );
    let mut work = make_work(&file, "Work", "C-contact-and-start");
    work.finish();
    assert_eq!(sound_names(&work), ["Footstep", "PlainSpin"]);
    let mut repeated = make_work(&file, "Work", "C-contact-and-start");
    repeated.steps.insert(0, repeated.steps[0].clone());
    repeated.finish();
    assert_ne!(
        (repeated.state, repeated.graphics, repeated.sounds),
        (work.state, work.graphics.clone(), work.sounds.clone())
    );
    let mut reordered = make_work(&file, "Work", "C-contact-and-start");
    reordered.steps.reverse();
    reordered.finish();
    assert_ne!(reordered.sounds, work.sounds);
    let mut twice = make_work(&file, "Animation", "C-kept-run-advance");
    twice.steps.extend(SuspendedAppearance::animation_steps());
    twice.finish();
    assert_ne!(
        twice.state, correct.state,
        "repeated update fails complete State check"
    );
}
