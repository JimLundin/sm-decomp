use super::{alter_transport_save, opened_context, transport_test_step};
use crate::production::{
    Production, ProductionContent, ProductionOutputs, ProductionRules, ProductionSeed,
};
use audio_transport::{
    AudioInstant, AudioPhase, CueId, Request, time,
    transport::{Input, Output},
};

fn fresh(effect_remainder: u8, music_remainder: u8) -> (ProductionContent, Production) {
    let (bank, template) = opened_context();
    let mut raw = template.transport.0;
    raw.now = time::cycles(0);
    raw.clock.lattice = time::halves(811);
    raw.phase = AudioPhase::ObserveClock {
        at: raw.clock.lattice.clone(),
    };
    raw.cadence.effect = effect_remainder;
    raw.cadence.music = music_remainder;
    let load = super::super::Load {
        senders: template.senders,
        requests: raw.requests,
        history: enum_map::EnumMap::from_fn(|group| raw.groups[group].history.clone()),
        timer_phase: raw.clock.phase.clone(),
        timer_unread: raw.clock.unread,
    };
    let loaded = super::State::opening_ceres(raw, load, &bank).unwrap();
    let content = ProductionContent::ResidentAudio(Box::new(bank));
    let production = Production::admit(
        ProductionSeed::ResidentAudio(Box::new(loaded)),
        content.clone(),
        ProductionRules::Audio,
    )
    .unwrap();
    (content, production)
}

fn step(production: &mut Production, at: AudioInstant, inputs: &[Input]) -> Vec<Output> {
    let ProductionOutputs::Audio(outputs) = transport_test_step(production, at, inputs) else {
        panic!("transport outputs");
    };
    outputs
}

fn deadline(production: &Production) -> AudioInstant {
    match &production.transport().unwrap().phase {
        AudioPhase::ObserveClock { at }
        | AudioPhase::Distribute { at, .. }
        | AudioPhase::MusicStart { at }
        | AudioPhase::Work { at, .. }
        | AudioPhase::Idle { at } => at.clone(),
        AudioPhase::Sample { sample, .. } => sample
            .anchor
            .sum(&time::halves(if sample.first.is_some() { 19 } else { 9 })),
    }
}

fn until_distribute(production: &mut Production) {
    loop {
        let output = step_next(production);
        if matches!(output.as_slice(), [Output::Clock { batch: 1, .. }]) {
            assert!(matches!(
                production.transport().unwrap().phase,
                AudioPhase::Distribute { batch: 1, .. }
            ));
            return;
        }
        assert!(matches!(
            output.as_slice(),
            [Output::Clock { batch: 0, .. }]
        ));
    }
}

fn next_clock(production: &mut Production) -> AudioInstant {
    for _ in 0..7 {
        let output = step(production, deadline(production), &[]);
        if let [Output::Clock { at, .. }] = output.as_slice() {
            return at.clone();
        }
    }
    panic!("expected clock observation");
}

fn retry(production: &mut Production) {
    assert!(matches!(
        step(production, deadline(production), &[]).as_slice(),
        [Output::Observation {
            request: Request::Absent,
            ..
        }]
    ));
    let at = deadline(production);
    let cue = Input::cue(at.clone(), CueId::Spin);
    assert!(matches!(
        step(production, at, &[cue]).as_slice(),
        [Output::Observation {
            request: Request::Cue(CueId::Spin),
            ..
        }]
    ));
    let held = production.transport().unwrap();
    let AudioPhase::Sample { sample, .. } = &held.phase else {
        panic!("retry sample");
    };
    assert_eq!(sample.anchor, held.now.sum(&time::halves(9)));
    assert!(sample.first.is_none());
}

#[test]
fn restore_enforces_finite_phase_windows() {
    let (content, mut production) = fresh(0, 0);
    until_distribute(&mut production);
    for (expected, excess) in [(53, 1), (58, 38), (1747, 1)] {
        let now = &production.transport().unwrap().now;
        let at = deadline(&production);
        assert_eq!(at.difference(now), time::halves(expected));
        let held = production.save().unwrap();
        if matches!(
            production.transport().unwrap().phase,
            AudioPhase::Idle { .. }
        ) {
            let changed = alter_transport_save(&held, |state| state.music_due = true);
            assert!(
                Production::restore(&changed, content.clone(), ProductionRules::Audio).is_err()
            );
        }
        let changed = alter_transport_save(&held, |state| {
            let (AudioPhase::Distribute { at, .. }
            | AudioPhase::MusicStart { at }
            | AudioPhase::Idle { at }) = &mut state.phase
            else {
                panic!("finite phase");
            };
            *at = at.sum(&time::halves(excess));
        });
        assert!(Production::restore(&changed, content.clone(), ProductionRules::Audio).is_err());
        let mut restored =
            Production::restore(&held, content.clone(), ProductionRules::Audio).unwrap();
        assert_eq!(restored.save().unwrap(), held);
        assert_eq!(
            next_clock(&mut restored),
            next_clock(&mut production.clone())
        );
        let _ = step(&mut production, at, &[]);
    }
}

#[test]
fn restore_enforces_sample_windows() {
    let (content, mut production) = fresh(7, 224);
    until_distribute(&mut production);
    assert_eq!(step_next(&mut production).len(), 0);
    assert!(matches!(
        step_next(&mut production).as_slice(),
        [Output::Publication { .. }]
    ));
    retry(&mut production);
    let cutoff = production
        .transport()
        .unwrap()
        .now
        .sum(&AudioInstant::new(1u64.into(), 3u64.into()).unwrap());
    assert_eq!(step(&mut production, cutoff, &[]).len(), 0);
    let saved = production.save().unwrap();
    assert_eq!(
        Production::restore(&saved, content.clone(), ProductionRules::Audio)
            .unwrap()
            .save()
            .unwrap(),
        saved
    );
    let changed = alter_transport_save(&saved, |state| {
        let AudioPhase::Sample { sample, .. } = &mut state.phase else {
            panic!("retry sample");
        };
        sample.anchor = sample.anchor.sum(&time::halves(1));
    });
    assert!(Production::restore(&changed, content.clone(), ProductionRules::Audio).is_err());
    let mut restored =
        Production::restore(&saved, content.clone(), ProductionRules::Audio).unwrap();
    for _ in 0..10 {
        if matches!(
            production.transport().unwrap().phase,
            AudioPhase::MusicStart { .. }
        ) {
            break;
        }
        assert_eq!(step_next(&mut restored), step_next(&mut production));
    }
    let state = production.transport().unwrap();
    assert!(matches!(state.phase, AudioPhase::MusicStart { .. }));
    assert_eq!(
        deadline(&production).difference(&state.now),
        time::halves(95)
    );
    let saved = production.save().unwrap();
    let mut restored = Production::restore(&saved, content, ProductionRules::Audio).unwrap();
    assert_eq!(next_clock(&mut restored), next_clock(&mut production));

    let (content, mut music) = fresh(0, 224);
    until_distribute(&mut music);
    for _ in 0..3 {
        let _ = step_next(&mut music);
    }
    let saved = music.save().unwrap();
    assert!(matches!(
        music.transport().unwrap().phase,
        AudioPhase::Sample { .. }
    ));
    assert!(!music.transport().unwrap().music_due);
    let changed = alter_transport_save(&saved, |state| state.music_due = true);
    assert!(Production::restore(&changed, content.clone(), ProductionRules::Audio).is_err());
    let changed = alter_transport_save(&saved, |state| {
        let AudioPhase::Sample { sample, .. } = &mut state.phase else {
            panic!("Music sample");
        };
        sample.anchor = sample.anchor.sum(&time::halves(1));
    });
    assert!(Production::restore(&changed, content, ProductionRules::Audio).is_err());
}

fn step_next(production: &mut Production) -> Vec<Output> {
    let at = deadline(production);
    step(production, at, &[])
}
