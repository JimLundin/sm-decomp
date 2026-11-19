use super::*;
use samus_appearance::{
    AppearanceRead, Frames, LapsLeft, Playhead, State, Step, SuspendedAppearance, WorkStep,
};
use types::{Action, BoostStage};

fn work() -> Work {
    Work::new(SuspendedAppearance::new(
        State {
            playhead: Playhead {
                step: Step::FIRST,
                time_left: Frames::NONE,
            },
            stage: BoostStage::Stage0,
            laps_left: LapsLeft::WithoutBooster,
        },
        None,
        vec![
            WorkStep::Appearance(AppearanceRead::Draw {
                previous: Action::Spinning,
                current: Action::Still,
            });
            2
        ],
    ))
}

#[test]
fn work_shape_remains_pinned() {
    assert_eq!(canonical::model::pin_holds::<WorkPin>(), Ok(()));
}

#[test]
fn silent_offer_is_a_read_and_delivery_survives_fresh_restore() {
    let mut owner = work();
    let before = owner.model.0.state;
    assert_eq!(owner.offer(), None);
    assert!(owner.complete());
    let first = owner.deliver().unwrap();
    assert_eq!(first.sounds.len(), 1);
    let saved = canonical::to_bytes(&owner);
    drop(owner);
    let mut restored: Work = canonical::from_bytes(&saved).unwrap();
    assert_eq!(restored.deliver().unwrap().sounds.len(), 0);
    assert_eq!(restored.offer(), None);
    assert!(restored.complete());
    assert_eq!(restored.deliver().unwrap(), first);
    assert!(!restored.complete());
    assert_eq!(restored.deliver().unwrap().sounds.len(), 0);
    assert_eq!(restored.model.0.state, before);
    assert_eq!(restored.model.0.completed.len(), 2);
}

#[test]
fn invalid_delivery_rejects_both_cursors_atomically() {
    let mut owner = work();
    owner.graphics_delivered = 1;
    let before = owner.clone();
    assert_eq!(owner.deliver(), Err(WorkFault::DeliveryCursor));
    assert_eq!(owner, before);
}

#[test]
fn actual_contact_offer_preserves_full_queue_and_rejects_unmapped_request() {
    use audio_transport::{
        CpuMasterTicks, CueId, OfferPermission, Request, SenderProgress, SenderState,
    };
    let mut state = SenderState {
        queue: std::iter::repeat_n(CueId::Land, 6).collect(),
        progress: SenderProgress::Ready,
        reply: Request::Absent,
        now: CpuMasterTicks(0),
        pending: None,
    };
    let before = state.clone();
    assert_eq!(
        offer_sound(
            &mut state,
            samus_appearance::PublishedSound::Appearance(samus_appearance::SoundRequest::Footstep),
            OfferPermission::Enabled
        ),
        Ok(())
    );
    assert_eq!(state, before);
    let unmapped =
        samus_appearance::PublishedSound::Appearance(samus_appearance::SoundRequest::SpaceJump);
    assert_eq!(
        offer_sound(&mut state, unmapped, OfferPermission::Enabled),
        Err(super::super::ProductionFault::Unsupported(
            super::super::Unsupported::AppearanceSound(unmapped)
        ))
    );
    assert_eq!(state, before);
    state.queue.clear();
    assert_eq!(
        offer_sound(
            &mut state,
            samus_appearance::PublishedSound::Appearance(samus_appearance::SoundRequest::Footstep),
            OfferPermission::Enabled
        ),
        Ok(())
    );
    assert_eq!(
        state.queue,
        std::collections::VecDeque::from([CueId::Footstep])
    );
}

#[test]
fn pending_animation_answer_survives_fresh_restore_with_its_actual_context() {
    use samus_appearance::{
        AnimationDecision, Clip, Dash, Figure, Inputs, Liquid, Pace, Paces, Rules, SoundsWaiting,
    };
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
    let content = super::super::AppearanceContent::new(
        vec![super::super::FigureClip {
            figure,
            clip: Clip {
                instructions: Vec::new(),
                kick: None,
                fast_fall: None,
                rising: None,
                contacts: std::collections::BTreeSet::new(),
            },
        }],
        Paces {
            plain: Pace(Vec::new()),
            boosted: enum_map::EnumMap::from_fn(|_| Pace(Vec::new())),
        },
    )
    .unwrap();
    let inputs = Inputs {
        figure,
        figure_after: figure,
        moves: types::Set::default(),
        dash: Dash::Off,
        run_held: false,
        heading_down: true,
        vertical_speed: core::num::Wrapping(0),
        energy: 99,
        items: types::Set::default(),
        liquid: Liquid::NONE,
        sounds_waiting: SoundsWaiting::NONE,
    };
    let mut owner = Work::new(SuspendedAppearance::new(
        State {
            playhead: Playhead {
                step: Step::FIRST,
                time_left: Frames(2),
            },
            ..work().model.0.state
        },
        Some(
            content
                .animation_inputs(inputs, figure, Rules::VANILLA)
                .unwrap(),
        ),
        vec![WorkStep::Animation(AnimationDecision::ClipPlaying)],
    ));
    let decision = owner.offer().unwrap().clone();
    assert_eq!(owner.model.0.state, decision.before);
    let bytes = canonical::to_bytes(&owner);
    drop(owner);
    let mut restored: Work = canonical::from_bytes(&bytes).unwrap();
    assert_eq!(restored.validate(), Ok(()));
    let mut invalid = restored.clone();
    invalid.model.0.steps.clear();
    assert_eq!(invalid.validate(), Err(WorkFault::PendingDecision));
    restored
        .model
        .0
        .animation
        .as_mut()
        .unwrap()
        .inputs
        .figure_after
        .sprite = types::Sprite::ScrewAttack;
    assert_eq!(restored.offer(), Some(&decision));
    assert!(restored.complete());
    assert_eq!(restored.model.0.state, decision.after);
    assert_eq!(restored.model.0.state.playhead.time_left, Frames::LAST);
    assert!(!restored.complete());
    assert_eq!(restored.model.0.completed.len(), 1);
}

#[test]
fn retirement_preserves_live_quantities_and_rejects_incomplete_or_undelivered_work() {
    let mut owner = work();
    let start = owner.clone();
    assert_eq!(owner.retire(), Err(WorkFault::Retirement));
    assert_eq!(owner, start);
    assert!(owner.complete());
    assert!(owner.complete());
    let undelivered = owner.clone();
    assert_eq!(owner.retire(), Err(WorkFault::Retirement));
    assert_eq!(owner, undelivered);
    let live = owner.model.0.state;
    let delivered = owner.deliver().unwrap();
    assert_ne!(delivered.sounds.len(), 0);
    owner.retire().unwrap();
    assert_eq!(owner.model.0.state, live);
    assert_eq!(owner.model.0.steps.len(), 0);
    assert_eq!(owner.model.0.completed.len(), 0);
    assert_eq!(owner.model.0.sounds.len(), 0);
    assert_eq!(owner.model.0.graphics.len(), 0);
    assert_eq!(owner.deliver().unwrap().sounds.len(), 0);
    assert_ne!(delivered.sounds.len(), 0);
}
