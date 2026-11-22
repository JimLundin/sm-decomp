use super::*;
use audio_transport::{CpuMasterTicks, Request, SenderProgress};

fn empty() -> SenderState {
    SenderState {
        queue: VecDeque::new(),
        progress: SenderProgress::Ready,
        reply: Request::Absent,
        now: CpuMasterTicks(0),
        pending: None,
    }
}

#[test]
fn restored_positions_follow_actual_append_without_reconstruction() {
    let senders = Senders::new(empty(), empty(), empty()).unwrap();
    let window = QueueWindow::new(5, 5).unwrap();
    let queues = Queues::new(window, window, window, &senders).unwrap();
    let mut restored: Queues = canonical::from_bytes(&canonical::to_bytes(&queues)).unwrap();
    let before = empty();
    let mut after = before.clone();
    after.queue.push_back(CueId::Footstep);
    restored
        .offered(SoundGroup::Contact, &before, &after)
        .unwrap();
    assert_eq!(restored.window(SoundGroup::Contact).positions(), [5, 6]);
    assert_eq!(restored.window(SoundGroup::Action), window);
    let snapshot = restored.clone();
    assert_eq!(
        restored.offered(SoundGroup::Contact, &before, &after),
        Err(Fault::Queue)
    );
    assert_eq!(restored, snapshot);
}

#[test]
fn invalid_restored_cursor_and_changed_progress_reject_atomically() {
    let senders = Senders::new(empty(), empty(), empty()).unwrap();
    let window = QueueWindow::new(0, 0).unwrap();
    let mut queues = Queues::new(window, window, window, &senders).unwrap();
    let before = empty();
    let mut after = before.clone();
    after.queue.push_back(CueId::Footstep);
    after.progress = SenderProgress::AwaitCue(CueId::Footstep);
    assert_eq!(
        queues.offered(SoundGroup::Contact, &before, &after),
        Err(Fault::Transition)
    );
    assert_eq!(queues.window(SoundGroup::Contact), window);
    queues.contact.tail = 16;
    let restored: Queues = canonical::from_bytes(&canonical::to_bytes(&queues)).unwrap();
    assert_eq!(restored.validate(&senders), Err(Fault::Queue));
}

#[test]
fn completed_actual_receipt_advances_only_the_dequeued_head() {
    let mut contact = empty();
    contact.queue.push_back(CueId::Footstep);
    let senders = Senders::new(empty(), empty(), contact.clone()).unwrap();
    let empty_window = QueueWindow::new(3, 3).unwrap();
    let contact_window = QueueWindow::new(5, 6).unwrap();
    let mut queues = Queues::new(empty_window, empty_window, contact_window, &senders).unwrap();
    let result = audio_transport::SenderResult {
        progress: SenderProgress::AwaitCue(CueId::Footstep),
        queue: VecDeque::new(),
        presentation: Some((CpuMasterTicks(312), Request::Cue(CueId::Footstep))),
        completion: CpuMasterTicks(498),
    };
    let receipt = Receipt {
        group: SoundGroup::Contact,
        before: contact.progress.clone(),
        queue: contact.queue,
        reply: Request::Absent,
        chosen: audio_transport::SenderCompletion {
            origin: CpuMasterTicks(100),
            observation: None,
            result: Some(result.clone()),
        },
    };
    let mut after = empty();
    after.progress = result.progress;
    after.now = result.completion;
    let mut wrong = after.clone();
    wrong.queue.push_back(CueId::Footstep);
    assert_eq!(queues.completed(&receipt, &wrong), Err(Fault::Transition));
    assert_eq!(queues.window(SoundGroup::Contact), contact_window);
    queues.completed(&receipt, &after).unwrap();
    let restored: Queues = canonical::from_bytes(&canonical::to_bytes(&queues)).unwrap();
    assert_eq!(restored.window(SoundGroup::Contact).positions(), [6, 6]);
    assert_eq!(restored.window(SoundGroup::Ambient), empty_window);
    assert_eq!(queues.completed(&receipt, &after), Err(Fault::Queue));
}

#[test]
fn marked_footstep_appends_to_the_live_empty_ring_and_wraps() {
    let senders = Senders::new(empty(), empty(), empty()).unwrap();
    let open = QueueWindow::new(15, 15).unwrap();
    let mut queues = Queues::new(open, open, open, &senders).unwrap();
    let before = empty();
    let mut with_cue = before.clone();
    with_cue.queue.push_back(CueId::Footstep);

    assert_eq!(
        queues.footstep(&before, &before, audio_transport::OfferPermission::Enabled),
        Err(Fault::Receipt)
    );
    assert_eq!(queues.window(SoundGroup::Contact), open);
    queues
        .footstep(
            &before,
            &with_cue,
            audio_transport::OfferPermission::Enabled,
        )
        .unwrap();
    assert_eq!(queues.window(SoundGroup::Contact).positions(), [15, 0]);

    let snapshot = queues.clone();
    assert_eq!(
        queues.footstep(
            &with_cue,
            &with_cue,
            audio_transport::OfferPermission::Enabled
        ),
        Err(Fault::Receipt)
    );
    assert_eq!(queues, snapshot);

    let result = audio_transport::SenderResult {
        progress: SenderProgress::AwaitCue(CueId::Footstep),
        queue: VecDeque::new(),
        presentation: Some((CpuMasterTicks(312), Request::Cue(CueId::Footstep))),
        completion: CpuMasterTicks(498),
    };
    let pending = Receipt {
        group: SoundGroup::Contact,
        before: with_cue.progress,
        queue: with_cue.queue,
        reply: with_cue.reply,
        chosen: audio_transport::SenderCompletion {
            origin: CpuMasterTicks(100),
            observation: None,
            result: Some(result.clone()),
        },
    };
    let mut consumed = empty();
    consumed.progress = result.progress;
    consumed.now = result.completion;
    queues.completed(&pending, &consumed).unwrap();
    assert_eq!(queues.window(SoundGroup::Contact).positions(), [0, 0]);
}
