//! Gameplay offers at their producer boundaries, before physical sender service.

use super::{
    AdmissionFault, ProductionFault, SenderInput, Unsupported,
    sender::{self, OfferCurrent, Queues, Senders, Sounds},
};
use audio_transport::{CueId, OfferPermission, QueueAllowance, SoundGroup};
use canonical::Canonical;

/// The carried sender and permission owners at a gameplay offer.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct GameplaySounds {
    senders: Senders,
    queues: Queues,
    current: OfferCurrent,
}

/// One actual ingress, including the older requests visible to the caller's offer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    /// The selected physical sender.
    pub group: SoundGroup,
    /// The typed cue from the actual producer.
    pub cue: CueId,
    /// The producer's queue limit at the call.
    pub allowance: QueueAllowance,
    /// The effective suppression at the call.
    pub permission: OfferPermission,
    /// All three active FIFOs and their positions immediately before the offer.
    pub before: GameplaySounds,
    /// Whether the selected FIFO accepted the request.
    pub accepted: bool,
}

impl GameplaySounds {
    /// Bind current suppression and actual physical queue positions to the three clean senders.
    ///
    /// # Errors
    /// Returns an admission fault for invalid queues or any pending physical sender service.
    pub fn new(
        senders: Senders,
        queues: Queues,
        current: OfferCurrent,
    ) -> Result<Self, AdmissionFault> {
        let sounds = Self {
            senders,
            queues,
            current,
        };
        sounds.validate()?;
        Ok(sounds)
    }

    pub(super) fn validate(&self) -> Result<(), AdmissionFault> {
        self.senders.validate()?;
        for group in [SoundGroup::Action, SoundGroup::Ambient, SoundGroup::Contact] {
            if self.senders.get(group).pending.is_some() {
                return Err(AdmissionFault::GameplaySoundServicePending(group));
            }
        }
        self.queues
            .validate(&self.senders)
            .map_err(|_| AdmissionFault::SenderQueue)
    }

    /// Current clean sender states.
    #[must_use]
    pub const fn senders(&self) -> &Senders {
        &self.senders
    }

    /// Current physical FIFO positions.
    #[must_use]
    pub const fn queues(&self) -> &Queues {
        &self.queues
    }

    /// Current permission operands.
    #[must_use]
    pub const fn current(&self) -> OfferCurrent {
        self.current
    }

    pub(super) const fn hold(&mut self) {
        self.current.sounds = Sounds::Disabled;
    }

    pub(super) fn offer(
        &mut self,
        cue: CueId,
        allowance: QueueAllowance,
    ) -> Result<Offer, ProductionFault> {
        let group = audio_transport::effects::group(cue);
        let permission = self.current.transport_permission();
        let before = self.clone();
        let earlier = before.senders.get(group);
        let mut later = earlier.clone();
        sender::advance(
            &mut later,
            group,
            SenderInput::Offer {
                cue,
                allowance,
                permission,
            },
        )?;
        let accepted = later.queue.len() > earlier.queue.len();
        self.queues
            .offered(group, earlier, &later)
            .map_err(|_| ProductionFault::Unsupported(Unsupported::SenderService))?;
        *self.senders.current(group) = later;
        Ok(Offer {
            group,
            cue,
            allowance,
            permission,
            before,
            accepted,
        })
    }
}
