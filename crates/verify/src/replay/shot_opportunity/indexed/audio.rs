//! Independent recorded Action and Contact ingress brackets for the carried shot.

use super::{Completion, ExpectedCut, Observation, Production, Sources, bad, opening};
use crate::replay::{gameplay_sounds as seen, sender_offer};
use audio_transport::{Request, SenderProgress, SenderState, SoundGroup};
use game::{
    production::{Senders, frame::post_down_contract::Sounds, gameplay_sounds::GameplaySounds},
    state::owned::OwnedState,
};

const SELECTED: [SoundGroup; 2] = [SoundGroup::Action, SoundGroup::Contact];

pub(super) fn admit(
    sources: &Sources,
    wram: &[u8],
    no_fire: bool,
) -> Result<(GameplaySounds, ExpectedCut), super::Outcome> {
    let [beam, door] = sender_offer::shot_windows(&sources.native)?;
    let expected = expectation(&beam, &door, no_fire)?;
    if wram.get(0x064a) != Some(&0) || beam.seed.now != door.seed.now {
        return Err(bad(Some(super::FIRST)));
    }
    let ambient = SenderState {
        queue: std::collections::VecDeque::new(),
        progress: SenderProgress::Ready,
        reply: Request::Absent,
        now: beam.seed.now.clone(),
        pending: None,
    };
    let senders = Senders::new(beam.seed, ambient, door.seed).map_err(|_| bad(None))?;
    let queues =
        crate::replay::frame_sender_contract::queues(wram, &senders).map_err(|_| bad(None))?;
    let current = seen::wram_current(wram)?;
    let audio = GameplaySounds::new(senders, queues, current).map_err(|_| bad(None))?;
    Ok((audio, expected))
}

fn expectation(
    beam: &sender_offer::ShotWindow,
    door: &sender_offer::ShotWindow,
    no_fire: bool,
) -> Result<ExpectedCut, super::Outcome> {
    let mut offers = seen::Offers::default();
    let mut final_fifos = Vec::new();
    for (group, window) in SELECTED.into_iter().zip([beam, door]) {
        let skipped = group == SoundGroup::Action && no_fire;
        let accepted = window.queue.len() == window.prefix.len().saturating_add(1)
            && window.queue.back() == Some(&window.cue);
        if !accepted || window.seed.progress != window.progress {
            return Err(bad(Some(super::FIRST)));
        }
        if !skipped {
            offers.ingress(
                group,
                window.cue,
                window.allowance,
                window.permission,
                accepted,
            );
            seen::prefix(&mut offers.prefixes, window.prefix.iter().copied(), None);
        }
        let queue = if skipped {
            &window.prefix
        } else {
            &window.queue
        };
        seen::append_fifo(&mut final_fifos, queue, &window.progress, None);
    }
    let mut outputs = offers.observations();
    outputs.push(opening([false, true]));
    Ok(seen::expected_cut(vec![seen::fifo(final_fifos)], outputs))
}

pub(super) fn actual(
    production: &Production,
    completion: &Completion,
    ingress: &[game::production::gameplay_sounds::Offer],
) -> Result<Vec<Observation>, super::Outcome> {
    let mut offers = seen::Offers::default();
    for offer in ingress {
        offers.observe(offer, &[offer.group], false);
    }
    let mut output = offers.observations();
    let audio = owner(production)?;
    if audio.current().sounds != Sounds::Enabled {
        return Err(bad(Some(super::FIRST)));
    }
    output.push(opening(
        completion.actors.runs.iter().map(|run| run.sound.is_some()),
    ));
    output.push(seen::queues(audio, &SELECTED, false));
    Ok(output)
}

pub(super) fn initial(production: &Production) -> Result<Vec<Observation>, super::Outcome> {
    Ok(vec![seen::queues(owner(production)?, &SELECTED, false)])
}

fn owner(production: &Production) -> Result<&GameplaySounds, super::Outcome> {
    let OwnedState::ShotOpportunity(state) = production.state() else {
        return Err(bad(None));
    };
    state.audio().ok_or_else(|| bad(None))
}
