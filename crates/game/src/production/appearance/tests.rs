use super::*;
use samus_appearance::{FrameSound, SoundRequest};

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
    let footstep = FrameSound::Opportunity(SoundRequest::Footstep);
    assert_eq!(
        offer_sound(&mut state, footstep, OfferPermission::Enabled),
        Ok(())
    );
    assert_eq!(state, before);
    let unmapped = FrameSound::Opportunity(SoundRequest::SpaceJump);
    assert_eq!(
        offer_sound(&mut state, unmapped, OfferPermission::Enabled),
        Err(super::super::ProductionFault::Unsupported(
            super::super::Unsupported::AppearanceSound(unmapped)
        ))
    );
    assert_eq!(state, before);
    state.queue.clear();
    assert_eq!(
        offer_sound(&mut state, footstep, OfferPermission::Enabled),
        Ok(())
    );
    assert_eq!(
        state.queue,
        std::collections::VecDeque::from([CueId::Footstep])
    );
}
