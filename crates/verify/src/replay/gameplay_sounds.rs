//! Projections for actual gameplay offer order, permission and FIFO windows.

use super::{compare::Observation, data_failure, result::Outcome, sender_offer::cue_tag};
use audio_transport::{CueId, OfferPermission, QueueAllowance, SenderProgress, SoundGroup};
use game::production::gameplay_sounds::{GameplaySounds, Offer};
use game::production::sender::{Demo, OfferCurrent, PowerBomb, Sounds};
use std::collections::VecDeque;

pub(super) const HOLD: u8 = u8::MAX;

pub(super) const GROUPS: [SoundGroup; 3] =
    [SoundGroup::Action, SoundGroup::Ambient, SoundGroup::Contact];

pub(super) const fn current(disabled: bool, demo: bool, explosion: bool) -> OfferCurrent {
    OfferCurrent {
        sounds: if disabled {
            Sounds::Disabled
        } else {
            Sounds::Enabled
        },
        demo: if demo { Demo::Active } else { Demo::Inactive },
        power_bomb: if explosion {
            PowerBomb::Active
        } else {
            PowerBomb::Inactive
        },
    }
}

pub(super) fn word(bytes: &[u8], offset: usize) -> Result<u16, Outcome> {
    let pair = bytes
        .get(offset..offset.saturating_add(2))
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| data_failure(None, "live sound word"))?;
    Ok(u16::from_le_bytes(pair))
}

pub(super) fn wram_current(wram: &[u8]) -> Result<OfferCurrent, Outcome> {
    Ok(current(
        word(wram, 0x05f5)? != 0,
        word(wram, 0x0998)? >= 0x28,
        word(wram, 0x0592)? & 0x8000 != 0,
    ))
}

fn field(name: &str, bytes: Vec<u8>) -> Observation {
    Observation {
        field: name.to_owned(),
        bytes,
    }
}

#[derive(Default)]
pub(super) struct Offers {
    pub order: Vec<u8>,
    pub permissions: Vec<u8>,
    pub prefixes: Vec<u8>,
    pub accepted: Vec<u8>,
}

impl Offers {
    pub fn observe(&mut self, offer: &Offer, groups: &[SoundGroup], ring_positions: bool) {
        self.ingress(
            offer.group,
            offer.cue,
            offer.allowance,
            offer.permission,
            offer.accepted,
        );
        for &group in groups {
            let before = offer.before.senders().get(group);
            let positions = offer.before.queues().window(group).positions();
            prefix(
                &mut self.prefixes,
                before.queue.iter().copied(),
                ring_positions.then_some(positions),
            );
        }
    }

    pub fn ingress(
        &mut self,
        group: SoundGroup,
        cue: CueId,
        limit: QueueAllowance,
        permission: OfferPermission,
        accepted: bool,
    ) {
        self.order.extend([group_tag(group), cue_tag(cue)]);
        self.permissions
            .extend([limit_tag(limit), permission_tag(permission)]);
        self.accepted.push(u8::from(accepted));
    }

    pub fn observations(self) -> Vec<Observation> {
        vec![
            field("live-sound.order", self.order),
            field("live-sound.permission", self.permissions),
            field("live-sound.prefix", self.prefixes),
            field("live-sound.accepted", self.accepted),
        ]
    }
}

pub(super) fn prefix(
    bytes: &mut Vec<u8>,
    queue: impl ExactSizeIterator<Item = CueId>,
    positions: Option<[u8; 2]>,
) {
    if let Some(positions) = positions {
        bytes.extend(positions);
    }
    bytes.push(u8::try_from(queue.len()).unwrap_or(u8::MAX));
    bytes.extend(queue.map(cue_tag));
}

pub(super) fn fifo(bytes: Vec<u8>) -> Observation {
    field("live-sound.fifo", bytes)
}

pub(super) fn hold(held: bool) -> Observation {
    field("live-sound.hold", vec![u8::from(held)])
}

pub(super) fn queues(
    audio: &GameplaySounds,
    groups: &[SoundGroup],
    ring_positions: bool,
) -> Observation {
    let mut bytes = Vec::new();
    for &group in groups {
        let sender = audio.senders().get(group);
        append_fifo(
            &mut bytes,
            &sender.queue,
            &sender.progress,
            ring_positions.then_some(audio.queues().window(group).positions()),
        );
    }
    fifo(bytes)
}

pub(super) fn append_fifo(
    bytes: &mut Vec<u8>,
    queue: &VecDeque<CueId>,
    progress: &SenderProgress,
    positions: Option<[u8; 2]>,
) {
    prefix(bytes, queue.iter().copied(), positions);
    bytes.extend(super::sender_offer::progress_bytes(progress));
}

const fn group_tag(group: SoundGroup) -> u8 {
    match group {
        SoundGroup::Action => 0,
        SoundGroup::Ambient => 1,
        SoundGroup::Contact => 2,
    }
}

const fn limit_tag(limit: QueueAllowance) -> u8 {
    match limit {
        QueueAllowance::One => 1,
        QueueAllowance::Three => 3,
        QueueAllowance::Six => 6,
        QueueAllowance::Nine => 9,
        QueueAllowance::Fifteen => 15,
    }
}

const fn permission_tag(permission: OfferPermission) -> u8 {
    match permission {
        OfferPermission::Enabled => 0,
        OfferPermission::Disabled => 1,
        OfferPermission::Demo => 2,
        OfferPermission::Explosion => 3,
    }
}

pub(super) fn expected_cut(
    semantics: Vec<Observation>,
    outputs: Vec<Observation>,
) -> super::readers::ExpectedCut {
    // Compare causal ingress before the terminal FIFO, even when a swap leaves it intact.
    let semantic_count = semantics.len();
    let output_count = outputs.len();
    let fields = outputs
        .into_iter()
        .chain(semantics.iter().cloned())
        .collect();
    super::readers::ExpectedCut {
        fields,
        semantics,
        semantic_count,
        output_count,
        replies: 0,
    }
}
