use super::*;
use samus_appearance::{Frames, LapsLeft, Playhead, SuspendedAppearance};
use types::{Action, BoostStage};

fn owner() -> Work {
    Work::new(SuspendedAppearance::new(
        samus_appearance::State {
            playhead: Playhead {
                step: samus_appearance::Step::FIRST,
                time_left: Frames::LAST,
            },
            stage: BoostStage::Stage0,
            laps_left: LapsLeft::WithoutBooster,
        },
        None,
        Vec::new(),
    ))
}

#[test]
fn current_read_is_retained_before_charge_and_delivered_once_after_fresh_restore() {
    let mut work = owner();
    let initial = work.model.0.state;
    choose_read(
        &mut work,
        AppearanceRead::Draw {
            previous: Action::Spinning,
            current: Action::Still,
        },
    )
    .unwrap();
    assert_eq!(work.model.0.sounds.len(), 0);
    assert_eq!(work.model.0.state, initial);
    let retained = work.clone();
    assert_eq!(
        choose_read(
            &mut work,
            AppearanceRead::Draw {
                previous: Action::Still,
                current: Action::Still,
            },
        ),
        Err(StageFault::Order)
    );
    assert_eq!(work, retained);
    let saved = canonical::to_bytes(&work);
    drop(work);
    let mut work: Work = canonical::from_bytes(&saved).unwrap();
    let outputs = complete(&mut work).unwrap();
    assert_eq!(
        outputs.sounds,
        [samus_appearance::PublishedSound::Appearance(
            samus_appearance::SoundRequest::SpinEnded
        )]
    );
    let saved = canonical::to_bytes(&work);
    drop(work);
    let mut work: Work = canonical::from_bytes(&saved).unwrap();
    let terminal = work.clone();
    assert_eq!(complete(&mut work), Err(StageFault::Order));
    assert_eq!(work, terminal);
    assert_eq!(work.deliver().unwrap().sounds.len(), 0);
    assert_eq!(work.model.0.state, initial);
    assert_eq!(work.model.0.completed.len(), 1);
}

#[test]
fn separate_actual_draws_publish_separate_requests_without_resetting_progress() {
    let mut work = owner();
    let input = AppearanceRead::Draw {
        previous: Action::Spinning,
        current: Action::Still,
    };
    choose_read(&mut work, input.clone()).unwrap();
    let first = complete(&mut work).unwrap();
    choose_read(&mut work, input).unwrap();
    let saved = canonical::to_bytes(&work);
    drop(work);
    let mut work: Work = canonical::from_bytes(&saved).unwrap();
    assert_eq!(complete(&mut work).unwrap(), first);
    assert_eq!(work.model.0.completed.len(), 2);
    assert_eq!(work.model.0.sounds.len(), 2);
    assert_eq!(work.deliver().unwrap().sounds.len(), 0);
}

#[test]
fn animation_completion_consumes_the_retained_answer_not_changed_unread_inputs() {
    use samus_appearance::{Clip, Dash, Figure, Inputs, Liquid, Pace, Paces, SoundsWaiting};
    let figure = Figure {
        pose: types::Pose {
            body: types::Body::Upright,
            action: Action::Falling,
            facing: types::Facing::Right,
            aim: types::Aim::Forward,
            gun: types::Gun::Stowed,
            legs: types::Legs::Together,
        },
        sprite: types::Sprite::Plain,
    };
    let clip = Clip {
        instructions: Vec::new(),
        kick: None,
        fast_fall: None,
        rising: None,
        contacts: std::collections::BTreeSet::new(),
    };
    let inputs = AnimationInputs {
        inputs: Inputs {
            figure,
            figure_after: figure,
            moves: Set::default(),
            dash: Dash::Off,
            run_held: false,
            heading_down: true,
            vertical_speed: core::num::Wrapping(0),
            energy: 99,
            items: Set::default(),
            liquid: Liquid::NONE,
            sounds_waiting: SoundsWaiting::NONE,
        },
        current_clip: clip.clone(),
        selected_from: figure,
        selected_clip: clip,
        paces: Paces {
            plain: Pace(Vec::new()),
            boosted: enum_map::EnumMap::from_fn(|_| Pace(Vec::new())),
        },
        rules: samus_appearance::Rules::VANILLA,
    };
    let mut work = owner();
    work.model.0.state.playhead.time_left = Frames(2);
    let answer = choose_animation(&mut work, inputs, AnimationDecision::ClipPlaying)
        .unwrap()
        .unwrap();
    assert_eq!(work.model.0.state, answer.before);
    let saved = canonical::to_bytes(&work);
    drop(work);
    let mut work: Work = canonical::from_bytes(&saved).unwrap();
    work.model
        .0
        .animation
        .as_mut()
        .unwrap()
        .inputs
        .figure_after
        .sprite = types::Sprite::ScrewAttack;
    complete(&mut work).unwrap();
    assert_eq!(work.model.0.state, answer.after);
    assert_eq!(
        work.model.0.completed,
        [samus_appearance::Receipt::Animation(Box::new(answer))]
    );
    assert_eq!(complete(&mut work), Err(StageFault::Order));
}
