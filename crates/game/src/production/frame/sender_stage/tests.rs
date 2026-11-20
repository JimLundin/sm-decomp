use super::*;
use audio_transport::{CueId, SenderProgress};
use samus_appearance::{
    AppearanceRead, Frames, LapsLeft, Playhead, Step, SuspendedAppearance, WorkStep,
};
use types::{Action, BoostStage};

#[derive(Canonical)]
struct Owner {
    senders: Senders,
    stage: Stage,
    offers: Offers,
    appearance: appearance::Work,
}

fn empty() -> SenderState {
    SenderState {
        queue: std::collections::VecDeque::new(),
        progress: SenderProgress::Ready,
        reply: Request::Absent,
        now: CpuMasterTicks(0),
        pending: None,
    }
}

fn owner() -> Owner {
    let figure = samus_appearance::Figure {
        pose: types::Pose {
            body: types::Body::Upright,
            action: Action::Running,
            facing: types::Facing::Right,
            aim: types::Aim::Forward,
            gun: types::Gun::Stowed,
            legs: types::Legs::Together,
        },
        sprite: types::Sprite::Plain,
    };
    let playhead = Playhead {
        step: Step::FIRST,
        time_left: Frames::LAST,
    };
    let clip = samus_appearance::Clip {
        instructions: vec![samus_appearance::Instruction::Show(
            samus_appearance::Hold::new(3).unwrap(),
        )],
        kick: None,
        fast_fall: None,
        rising: None,
        contacts: std::iter::once(Step::FIRST).collect(),
    };
    let mut appearance = appearance::Work::new(SuspendedAppearance::new(
        samus_appearance::State {
            playhead,
            stage: BoostStage::Stage0,
            laps_left: LapsLeft::WithoutBooster,
        },
        None,
        vec![WorkStep::Appearance(AppearanceRead::Contact {
            figure,
            playhead,
            stage: BoostStage::Stage0,
            clip,
            suppressors: types::Set::default(),
        })],
    ));
    assert!(appearance.complete());
    Owner {
        senders: Senders::new(empty(), empty(), empty()).unwrap(),
        stage: Stage::Idle,
        offers: Offers::default(),
        appearance,
    }
}

fn restore(owner: Owner) -> Owner {
    let bytes = canonical::to_bytes(&owner);
    drop(owner);
    canonical::from_bytes(&bytes).unwrap()
}

fn intrinsic(origin: &CpuMasterTicks, milestone: &CpuMasterTicks) -> u64 {
    u64::try_from(std::ops::Sub::sub(&milestone.0, &origin.0)).unwrap()
}

#[test]
fn receipt_shape_is_pinned() {
    assert_eq!(canonical::model::pin_holds::<ReceiptPin>(), Ok(()));
}

#[test]
fn actual_contact_and_sender_milestones_restore_without_reoffering_or_early_completion() {
    let mut owner = owner();
    owner
        .offers
        .appearance(
            &mut owner.senders,
            &owner.appearance,
            OfferPermission::Enabled,
        )
        .unwrap();
    assert_eq!(
        owner.senders.contact.0.queue,
        std::collections::VecDeque::from([CueId::Footstep])
    );
    let mut owner = restore(owner);
    owner
        .offers
        .appearance(
            &mut owner.senders,
            &owner.appearance,
            OfferPermission::Enabled,
        )
        .unwrap();
    assert_eq!(
        owner.senders.contact.0.queue,
        std::collections::VecDeque::from([CueId::Footstep])
    );
    let chosen = owner
        .stage
        .choose(
            &mut owner.senders,
            SoundGroup::Contact,
            MasterTicks(1000),
            SenderProfile::FastBus,
        )
        .unwrap()
        .clone();
    assert_eq!(owner.senders.contact.0.progress, SenderProgress::Ready);
    let result = chosen.chosen.result.as_ref().unwrap();
    let presentation = intrinsic(
        &chosen.chosen.origin,
        &result.presentation.as_ref().unwrap().0,
    );
    let completion = intrinsic(&chosen.chosen.origin, &result.completion);
    owner.stage.bind(WorkId(9), MasterTicks(1000)).unwrap();
    let mut owner = restore(owner);
    assert_eq!(
        owner
            .stage
            .complete(&mut owner.senders, WorkId(9), completion, MasterTicks(9000)),
        Err(Fault::Milestone)
    );
    assert_eq!(owner.senders.contact.0.progress, SenderProgress::Ready);
    assert_eq!(
        owner.stage.present(
            &mut owner.senders,
            WorkId(9),
            presentation,
            MasterTicks(8000)
        ),
        Ok(Request::Cue(CueId::Footstep))
    );
    let mut owner = restore(owner);
    assert_eq!(
        owner.stage.present(
            &mut owner.senders,
            WorkId(9),
            presentation,
            MasterTicks(8000)
        ),
        Err(Fault::Milestone)
    );
    let consumed = owner
        .stage
        .complete(&mut owner.senders, WorkId(9), completion, MasterTicks(9000))
        .unwrap();
    assert_eq!(consumed, chosen);
    assert_eq!(
        owner.senders.contact.0.progress,
        SenderProgress::AwaitCue(CueId::Footstep)
    );
    assert!(owner.senders.contact.0.queue.is_empty());
    let mut owner = restore(owner);
    assert_eq!(
        owner
            .stage
            .complete(&mut owner.senders, WorkId(9), completion, MasterTicks(9000)),
        Err(Fault::Order)
    );
    owner
        .offers
        .appearance(
            &mut owner.senders,
            &owner.appearance,
            OfferPermission::Enabled,
        )
        .unwrap();
    assert!(owner.senders.contact.0.queue.is_empty());
}

#[test]
fn empty_service_and_unqualified_reply_arm_are_distinct_actual_choices() {
    let mut owner = owner();
    let chosen = owner
        .stage
        .choose(
            &mut owner.senders,
            SoundGroup::Action,
            MasterTicks(222),
            SenderProfile::FastBus,
        )
        .unwrap()
        .clone();
    let completion = intrinsic(
        &chosen.chosen.origin,
        &chosen.chosen.result.as_ref().unwrap().completion,
    );
    owner.stage.bind(WorkId(1), MasterTicks(222)).unwrap();
    let mut owner = restore(owner);
    owner
        .stage
        .complete(&mut owner.senders, WorkId(1), completion, MasterTicks(2000))
        .unwrap();
    assert_eq!(owner.senders.action.0.progress, SenderProgress::Ready);
    owner.senders.action.0.progress = SenderProgress::AwaitCue(CueId::Beam);
    let before = owner.senders.clone();
    assert_eq!(
        owner.stage.choose(
            &mut owner.senders,
            SoundGroup::Action,
            MasterTicks(2100),
            SenderProfile::FastBus
        ),
        Err(Fault::ReplyObservation)
    );
    assert_eq!(owner.senders, before);
    assert_eq!(owner.stage, Stage::Idle);
}
