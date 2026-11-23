//! One admitted CPU audio sender group and its actual clean service entry.

use super::{AdmissionFault, ProductionFault, Unsupported};
use crate::state::owned::SenderPin;
use audio_transport::sender::{ReplyInput, SenderOutput, SenderProfile};
use audio_transport::{
    CpuMasterTicks, CueId, OfferPermission, QueueAllowance, Request, SenderProgress, SenderState,
    SoundGroup,
};
use canonical::Canonical;
use canonical::model::Model;
use std::collections::VecDeque;

/// The three actual request queues, each with its complete pending service.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Senders {
    /// Action requests.
    pub action: Model<SenderPin>,
    /// Ambient requests.
    pub ambient: Model<SenderPin>,
    /// Contact requests, including actual foot contacts.
    pub contact: Model<SenderPin>,
}

impl Senders {
    /// Admit each independent current queue without beginning a service.
    ///
    /// # Errors
    /// A queue, held reply or pending result belongs to a different group.
    pub fn new(
        action: SenderState,
        ambient: SenderState,
        contact: SenderState,
    ) -> Result<Self, AdmissionFault> {
        let senders = Self {
            action: Model(Box::new(action)),
            ambient: Model(Box::new(ambient)),
            contact: Model(Box::new(contact)),
        };
        senders.validate()?;
        Ok(senders)
    }

    /// Validate all current and pending sender quantities after restoration.
    ///
    /// # Errors
    /// A queue contains an unadmitted cue or too many requests.
    pub fn validate(&self) -> Result<(), AdmissionFault> {
        crate::production::sender::validate(&self.action.0, SoundGroup::Action)?;
        crate::production::sender::validate(&self.ambient.0, SoundGroup::Ambient)?;
        crate::production::sender::validate(&self.contact.0, SoundGroup::Contact)
    }

    /// One held sender, selected by physical request group.
    #[must_use]
    pub fn get(&self, group: SoundGroup) -> &SenderState {
        match group {
            SoundGroup::Action => &self.action.0,
            SoundGroup::Ambient => &self.ambient.0,
            SoundGroup::Contact => &self.contact.0,
        }
    }

    pub(crate) fn current(&mut self, group: SoundGroup) -> &mut SenderState {
        match group {
            SoundGroup::Action => &mut self.action.0,
            SoundGroup::Ambient => &mut self.ambient.0,
            SoundGroup::Contact => &mut self.contact.0,
        }
    }
}

/// One causal offer or independently scheduled service operation.
pub enum SenderInput<'a> {
    /// An actual producer offers one cue between services.
    Offer {
        /// The semantic cue; its group must match the admitted sender.
        cue: CueId,
        /// The producer's queue allowance.
        allowance: QueueAllowance,
        /// The producer's actual sound permission.
        permission: OfferPermission,
    },
    /// Begin a service only under the independently admitted platform timing profile.
    Begin(SenderProfile),
    /// Resume through an exact CPU stop with actual physical reply updates.
    Advance {
        /// Absolute CPU master ticks, distinct from audio cycles.
        stop: CpuMasterTicks,
        /// Ordered replies strictly after the held CPU instant and through this stop.
        replies: &'a [ReplyInput],
    },
}

fn request_group(request: Request, group: SoundGroup) -> bool {
    match request {
        Request::Absent => true,
        Request::Cue(cue) => audio_transport::effects::group(cue) == group,
    }
}

fn progress_group(progress: &SenderProgress, group: SoundGroup) -> bool {
    match progress {
        SenderProgress::AwaitCue(cue) => audio_transport::effects::group(*cue) == group,
        SenderProgress::Ready
        | SenderProgress::ClearDelay(_)
        | SenderProgress::AwaitClear
        | SenderProgress::ResetDelay(_) => true,
    }
}

pub(super) fn validate(state: &SenderState, group: SoundGroup) -> Result<(), AdmissionFault> {
    let queue_valid = |queue: &VecDeque<CueId>| {
        queue.len() <= 15
            && queue
                .iter()
                .all(|cue| audio_transport::effects::group(*cue) == group)
    };
    if !queue_valid(&state.queue)
        || !request_group(state.reply, group)
        || !progress_group(&state.progress, group)
        || state
            .pending
            .as_ref()
            .and_then(|pending| pending.result.as_ref())
            .is_some_and(|result| {
                !queue_valid(&result.queue)
                    || !progress_group(&result.progress, group)
                    || result
                        .presentation
                        .as_ref()
                        .is_some_and(|(_, request)| !request_group(*request, group))
            })
    {
        return Err(AdmissionFault::SenderQueue);
    }
    Ok(())
}

pub(super) fn advance(
    state: &mut SenderState,
    group: SoundGroup,
    input: SenderInput<'_>,
) -> Result<Vec<SenderOutput>, ProductionFault> {
    match input {
        SenderInput::Offer {
            cue,
            allowance,
            permission,
        } => {
            if state.pending.is_some() {
                return Err(ProductionFault::Unsupported(Unsupported::SenderService));
            }
            if audio_transport::effects::group(cue) != group {
                return Err(ProductionFault::Unsupported(Unsupported::SenderCueGroup));
            }
            audio_transport::sender::offer(state, cue, allowance, permission);
            Ok(Vec::new())
        }
        SenderInput::Begin(profile) => {
            audio_transport::sender::begin(state, profile);
            Ok(Vec::new())
        }
        SenderInput::Advance { stop, replies } => {
            if stop.0 < state.now.0 {
                return Err(ProductionFault::TimeReversed);
            }
            if replies
                .iter()
                .any(|input| input.at.0 <= state.now.0 || input.at.0 > stop.0)
            {
                return Err(ProductionFault::PresentationOutsideInterval);
            }
            if replies.windows(2).any(|pair| {
                pair.first()
                    .zip(pair.last())
                    .is_some_and(|(before, after)| before.at.0 > after.at.0)
            }) {
                return Err(ProductionFault::PresentationsUnordered);
            }
            if replies
                .iter()
                .any(|input| !request_group(input.reply, group))
            {
                return Err(ProductionFault::Unsupported(Unsupported::SenderCueGroup));
            }
            let elapsed =
                audio_transport::ExactCpuMasterTicks(std::ops::Sub::sub(&stop.0, &state.now.0));
            Ok(audio_transport::sender::advance_with_inputs(
                state, elapsed, replies,
            ))
        }
    }
}

pub(super) fn offer_touchdown(
    sender: &mut audio_transport::SenderState,
    request: types::SoundRequest,
    permission: OfferPermission,
) -> Result<(CueId, QueueAllowance), ProductionFault> {
    if request.identity != types::SoundIdentity::SamusLandedOrWallJumped
        || request.library != types::SoundLibrary::Library3
        || request.queue_allowance != types::QueueAllowance(6)
    {
        return Err(ProductionFault::Unsupported(
            Unsupported::MovementTouchdownSound(request),
        ));
    }
    let cue = CueId::Land;
    let allowance = QueueAllowance::Six;
    advance(
        sender,
        SoundGroup::Contact,
        SenderInput::Offer {
            cue,
            allowance,
            permission,
        },
    )?;
    Ok((cue, allowance))
}

/// Actual current sound-disable owner.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sounds {
    /// Requests may reach transport admission.
    Enabled,
    /// The source-qualified enqueue arm is unavailable.
    Disabled,
}

/// Actual current scene's demo activity.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Demo {
    /// The scene is outside the demo range.
    Inactive,
    /// The scene requires another source arm.
    Active,
}

/// Actual current power-bomb activity, distinct from cinematic sound suppression.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerBomb {
    /// No power bomb is active.
    Inactive,
    /// The source-qualified enqueue arm is unavailable.
    Active,
}

/// Actual owners read at `SoundOffer`; queue quantities remain in their sole owner.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OfferCurrent {
    /// Actual sound-enabled status.
    pub sounds: Sounds,
    /// Actual scene demo activity.
    pub demo: Demo,
    /// Actual power-bomb activity.
    pub power_bomb: PowerBomb,
}

impl OfferCurrent {
    /// Current suppression for the ordinary sender offer, in native gate order.
    #[must_use]
    pub const fn transport_permission(self) -> audio_transport::OfferPermission {
        use audio_transport::OfferPermission;
        match (self.sounds, self.demo, self.power_bomb) {
            (Sounds::Disabled, _, _) => OfferPermission::Disabled,
            (Sounds::Enabled, Demo::Active, _) => OfferPermission::Demo,
            (Sounds::Enabled, Demo::Inactive, PowerBomb::Active) => OfferPermission::Explosion,
            (Sounds::Enabled, Demo::Inactive, PowerBomb::Inactive) => OfferPermission::Enabled,
        }
    }

    /// Qualify only the independently accepted unsuppressed transport arm.
    #[must_use]
    pub const fn permission(self) -> Option<audio_transport::OfferPermission> {
        match (self.sounds, self.demo, self.power_bomb) {
            (Sounds::Enabled, Demo::Inactive, PowerBomb::Inactive) => {
                Some(audio_transport::OfferPermission::Enabled)
            }
            _ => None,
        }
    }
}

/// Invalid current queue representation or producer transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueFault {
    /// Cursors do not represent the actual sole clean FIFO.
    Queue,
    /// The observed producer transition is not an append or retained dequeue.
    Transition,
    /// The offer is not the one-cue enqueue its permission allows.
    Receipt,
}

/// Physical positions only; the actual cue FIFO belongs to the clean sender.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct QueueWindow {
    head: u8,
    tail: u8,
}

impl QueueWindow {
    /// Import independently qualified current positions once at the frame origin.
    /// # Errors
    /// Positions outside the sixteen-slot physical representation.
    pub const fn new(head: u8, tail: u8) -> Result<Self, QueueFault> {
        if head >= 16 || tail >= 16 {
            return Err(QueueFault::Queue);
        }
        Ok(Self { head, tail })
    }

    /// Current physical positions, used only to qualify a source envelope.
    #[must_use]
    pub const fn positions(self) -> [u8; 2] {
        [self.head, self.tail]
    }

    fn validate(self, queue: &VecDeque<CueId>) -> Result<(), QueueFault> {
        if self.head >= 16
            || self.tail >= 16
            || usize::from(self.tail.wrapping_sub(self.head) % 16) != queue.len()
        {
            return Err(QueueFault::Queue);
        }
        Ok(())
    }
}

/// Origin-derived positions for the three actual sender owners.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Queues {
    action: QueueWindow,
    ambient: QueueWindow,
    contact: QueueWindow,
}

fn sender(senders: &Senders, group: SoundGroup) -> &SenderState {
    match group {
        SoundGroup::Action => &senders.action.0,
        SoundGroup::Ambient => &senders.ambient.0,
        SoundGroup::Contact => &senders.contact.0,
    }
}

impl Queues {
    /// Import positions alongside, rather than in place of, actual clean queues.
    /// # Errors
    /// A physical window disagrees with its actual current sender FIFO.
    pub fn new(
        action: QueueWindow,
        ambient: QueueWindow,
        contact: QueueWindow,
        senders: &Senders,
    ) -> Result<Self, QueueFault> {
        let queues = Self {
            action,
            ambient,
            contact,
        };
        queues.validate(senders)?;
        Ok(queues)
    }

    /// Check restored positions against all actual current FIFOs.
    /// # Errors
    /// An invalid window or lost queue transition.
    pub fn validate(&self, senders: &Senders) -> Result<(), QueueFault> {
        for group in [SoundGroup::Action, SoundGroup::Ambient, SoundGroup::Contact] {
            self.window(group).validate(&sender(senders, group).queue)?;
        }
        Ok(())
    }

    /// The current physical window, never reconstructed from queue length.
    #[must_use]
    pub const fn window(&self, group: SoundGroup) -> QueueWindow {
        match group {
            SoundGroup::Action => self.action,
            SoundGroup::Ambient => self.ambient,
            SoundGroup::Contact => self.contact,
        }
    }

    const fn window_mut(&mut self, group: SoundGroup) -> &mut QueueWindow {
        match group {
            SoundGroup::Action => &mut self.action,
            SoundGroup::Ambient => &mut self.ambient,
            SoundGroup::Contact => &mut self.contact,
        }
    }

    /// Track actual append results on the same candidate as the producer commit.
    /// # Errors
    /// A producer replaced existing cues, shortened the FIFO or changed progress.
    pub fn offered(
        &mut self,
        group: SoundGroup,
        before: &SenderState,
        after: &SenderState,
    ) -> Result<(), QueueFault> {
        let mut window = self.window(group);
        window.validate(&before.queue)?;
        if before.progress != after.progress
            || before.reply != after.reply
            || before.now != after.now
            || before.pending != after.pending
            || after.queue.len() < before.queue.len()
            || !before
                .queue
                .iter()
                .eq(after.queue.iter().take(before.queue.len()))
        {
            return Err(QueueFault::Transition);
        }
        let added = after
            .queue
            .len()
            .checked_sub(before.queue.len())
            .ok_or(QueueFault::Transition)?;
        let added = u8::try_from(added).map_err(|_| QueueFault::Queue)?;
        window.tail = window.tail.wrapping_add(added) % 16;
        window.validate(&after.queue)?;
        *self.window_mut(group) = window;
        Ok(())
    }
}
