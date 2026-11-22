//! Actual caller admission, service milestones and continuation for the three Senders.

use super::{State, sender_contract, sender_stage};
use crate::production::timing::{self, Event, EventKind, WorkId};
use audio_transport::{OfferPermission, SoundGroup, sender::SenderProfile};
use canonical::Canonical;
use data::timing::{MasterTicks, Profile};

const CONTACT_REPLY: WorkId = WorkId(1307);
const CONTACT_READ_PREFIX: u64 = 62;

fn latest_contact_retry(
    events: &[Event],
    binding: sender_stage::CueBinding,
    before: MasterTicks,
) -> Option<MasterTicks> {
    let mut awaiting_retry = false;
    let mut latest = None;
    for event in events.iter().take_while(|event| event.at < before) {
        if let EventKind::AudioPortRead { port, value } = event.kind
            && port == binding.port
        {
            awaiting_retry = value == 0;
        }
        if awaiting_retry
            && matches!(&event.kind, EventKind::Publication(write)
                if write.port == binding.port && write.value == binding.value)
        {
            awaiting_retry = false;
            latest = Some(event.at);
        }
    }
    latest
}

impl State {
    pub(super) fn validate_contact_checkpoint(
        &self,
        content: &super::Content,
    ) -> Result<(), super::Fault> {
        use audio_transport::Request;
        let invalid = || super::Fault::InvalidPlayerEntry;
        let active = self.sender_stage.contact_reply();
        let completed = self.completed_contact_read;
        let expects_active = matches!(
            self.sender_progress,
            Cursor::ContactRead
                | Cursor::Body {
                    service: Service::Contact,
                    prefix: CONTACT_READ_PREFIX,
                    ..
                }
        );
        if expects_active != active.is_some() {
            return Err(invalid());
        }
        if completed.is_some()
            && (active.is_some()
                || !matches!(
                    self.sender_progress,
                    Cursor::Tail(Service::Contact) | Cursor::Done
                ))
        {
            return Err(invalid());
        }
        let events = self
            .publications
            .active()
            .map_err(super::Fault::Publications)?;
        let mut reads = events.iter().filter_map(|event| {
            if let EventKind::AudioPortRead { port, value } = event.kind {
                Some((event.at, port, value))
            } else {
                None
            }
        });
        let sample = reads.next();
        let read = active.or(completed);
        if reads.next().is_some()
            || read.map(|read| read.at) != sample.map(|sample| sample.0)
            || read.is_some_and(|read| read.intrinsic != sender_contract::ContactRetry::SAMPLE)
        {
            return Err(invalid());
        }
        if sample.is_none() && self.contact_retried_at.is_none() {
            return Ok(());
        }
        let binding = content.sender_cue.ok_or_else(invalid)?;
        let all = &self.publications.events;
        if self.contact_retried_at
            != latest_contact_retry(
                all,
                binding,
                MasterTicks(self.timing.hardware.now.0.saturating_add(1)),
            )
        {
            return Err(invalid());
        }
        if let Some((at, port, value)) = sample {
            let contract = content
                .sender_physical
                .as_ref()
                .and_then(|profile| profile.contact_retry.as_ref())
                .ok_or_else(invalid)?;
            let reply = contract.reply_at(latest_contact_retry(all, binding, at), at);
            let observed = if reply == Request::Cue(binding.cue) {
                binding.value
            } else {
                0
            };
            if at > self.timing.hardware.now || port != binding.port || value != observed {
                return Err(invalid());
            }
        }
        Ok(())
    }
}

/// Caller service order, independent of actual producer offer progress.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Service {
    /// Action library.
    Action,
    /// Ambient library.
    Ambient,
    /// Contact library.
    Contact,
}

impl Service {
    const fn group(self) -> SoundGroup {
        match self {
            Self::Action => SoundGroup::Action,
            Self::Ambient => SoundGroup::Ambient,
            Self::Contact => SoundGroup::Contact,
        }
    }

    const fn body(self) -> WorkId {
        WorkId(match self {
            Self::Action => 1301,
            Self::Ambient => 1303,
            Self::Contact => 1305,
        })
    }

    const fn tail(self) -> WorkId {
        WorkId(match self {
            Self::Action => 1302,
            Self::Ambient => 1304,
            Self::Contact => 1306,
        })
    }
}

/// Retained caller progress; the clean Sender remains the only service owner.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Cursor {
    /// Admission has not begun.
    #[default]
    New,
    /// Actual admission work is charging.
    Admission,
    /// Current owners can be qualified for this service.
    Ready(Service),
    /// Contact's actual physical reply edge has not yet occurred.
    ContactReading,
    /// The actual reply has been consumed and the read exposure is finishing.
    ContactRead,
    /// A qualified body is charging with its intrinsic milestones retained.
    Body {
        /// Actual selected library.
        service: Service,
        /// Caller continuation charged after clean completion.
        tail: Profile,
        /// Unconsumed intrinsic publication boundary.
        publication: Option<u64>,
        /// Intrinsic completion boundary, excluding hardware stalls.
        completion: u64,
        /// Intrinsic charge furnished by a preceding reply-read prefix.
        prefix: u64,
    },
    /// The actual caller continuation is charging.
    Tail(Service),
    /// All three services and their continuations actually completed.
    Done,
}

/// The actual Sender input at the handoff from gameplay's finished sound offer.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct History {
    senders: super::Senders,
    queues: sender_contract::Queues,
}

impl History {
    pub(super) fn capture(state: &State) -> Option<Self> {
        if state.sender_stage != sender_stage::Stage::Idle {
            return None;
        }
        Some(Self {
            senders: state.senders.clone(),
            queues: state.sender_queues.clone()?,
        })
    }

    pub(super) fn use_in_history(&self, earlier: &mut State) {
        earlier.senders.clone_from(&self.senders);
        earlier.sender_queues = Some(self.queues.clone());
        // The captured Sender inputs precede service admission and any unresolved reply.
        earlier.sender_stage = sender_stage::Stage::Idle;
    }
}

/// Failed caller order, source qualification or actual hardware milestone.
#[derive(Debug, PartialEq, Eq)]
pub enum Fault {
    /// Missing owners or an unexpected work/event.
    Order,
    /// Clean Sender milestone failed.
    Sender(sender_stage::Fault),
    /// Current queue or full physical contract failed.
    Contract(sender_contract::Fault),
    /// Actual timing work could not begin.
    Timing(timing::Fault),
}

impl Cursor {
    pub(super) const fn active(&self) -> Option<WorkId> {
        match self {
            Self::Admission => Some(WorkId(1300)),
            Self::ContactReading | Self::ContactRead => Some(CONTACT_REPLY),
            Self::Body { service, .. } => Some(service.body()),
            Self::Tail(service) => Some(service.tail()),
            Self::New | Self::Ready(_) | Self::Done => None,
        }
    }

    /// Begin admission or the next actual service on a private frame candidate.
    /// # Errors
    /// Current owners, whole contract or hardware work admission failed.
    pub fn begin(
        &mut self,
        state: &mut State,
        content: &sender_contract::PhysicalContent,
        binding: sender_stage::CueBinding,
        wall: MasterTicks,
    ) -> Result<(), Fault> {
        let mut candidate = state.clone();
        let current = candidate.sender_current.ok_or(Fault::Order)?;
        let queues = candidate.sender_queues.as_ref().ok_or(Fault::Order)?;
        let retry = content.contact_retry.as_ref().filter(|_| {
            candidate.senders.contact.0.progress
                == audio_transport::SenderProgress::AwaitCue(audio_transport::CueId::Footstep)
        });
        let next = match *self {
            Self::New => {
                let admission = &content.admission;
                let profile = if retry.is_some() {
                    admission.qualify_retry(&candidate.senders, queues, current)
                } else {
                    admission.qualify(&candidate.senders, queues, current)
                }
                .map_err(Fault::Contract)?;
                candidate
                    .timing
                    .begin(WorkId(1300), profile)
                    .map_err(Fault::Timing)?;
                Self::Admission
            }
            Self::Ready(Service::Contact) if retry.is_some() => {
                let retry = retry.ok_or(Fault::Order)?;
                candidate
                    .sender_stage
                    .begin_contact_reply(&mut candidate.senders, CONTACT_REPLY, wall)
                    .map_err(Fault::Sender)?;
                candidate
                    .timing
                    .begin(CONTACT_REPLY, retry.prefix.clone())
                    .map_err(Fault::Timing)?;
                Self::ContactReading
            }
            Self::Ready(service) => {
                queues
                    .validate(&candidate.senders)
                    .map_err(Fault::Contract)?;
                let receipt = candidate
                    .sender_stage
                    .choose(
                        &mut candidate.senders,
                        service.group(),
                        wall,
                        SenderProfile::FastBus,
                    )
                    .map_err(Fault::Sender)?;
                let contract = match service {
                    Service::Action => &content.action,
                    Service::Ambient => &content.ambient,
                    Service::Contact => &content.contact,
                };
                let qualified = contract
                    .qualify(receipt, queues, current, binding)
                    .map_err(Fault::Contract)?;
                candidate
                    .timing
                    .begin(service.body(), qualified.body)
                    .map_err(Fault::Timing)?;
                candidate
                    .sender_stage
                    .bind(service.body(), wall)
                    .map_err(Fault::Sender)?;
                Self::Body {
                    service,
                    tail: qualified.tail,
                    publication: qualified.publication,
                    completion: qualified.completion,
                    prefix: 0,
                }
            }
            Self::Admission
            | Self::ContactReading
            | Self::ContactRead
            | Self::Body { .. }
            | Self::Tail(_)
            | Self::Done => {
                return Err(Fault::Order);
            }
        };
        *state = candidate;
        *self = next;
        Ok(())
    }

    pub(super) fn sample_contact_if_due(
        &mut self,
        state: &mut State,
        content: &sender_contract::PhysicalContent,
        active: WorkId,
        spent: Option<MasterTicks>,
        binding: sender_stage::CueBinding,
    ) -> Result<Option<Event>, Fault> {
        if active != CONTACT_REPLY || !matches!(self, Self::ContactReading) {
            return Ok(None);
        }
        let at = sender_contract::ContactRetry::SAMPLE;
        if spent.is_some_and(|spent| spent.0 > at) {
            return Err(Fault::Order);
        }
        if spent == Some(MasterTicks(at)) {
            use audio_transport::Request;
            let retry = content.contact_retry.as_ref().ok_or(Fault::Order)?;
            let now = state.timing.hardware.now;
            let reply = retry.reply_at(state.contact_retried_at, now);
            let value = match reply {
                Request::Absent => 0,
                Request::Cue(cue) if cue == binding.cue => binding.value,
                Request::Cue(_) => return Err(Fault::Order),
            };
            state
                .sender_stage
                .consume_contact_reply(
                    &mut state.senders,
                    [CONTACT_REPLY, Service::Contact.body()],
                    now,
                    at,
                    reply,
                )
                .map_err(Fault::Sender)?;
            *self = Self::ContactRead;
            return Ok(Some(Event {
                at: now,
                kind: EventKind::AudioPortRead {
                    port: binding.port,
                    value,
                },
            }));
        }
        Ok(None)
    }

    pub(super) fn complete_contact_prefix(
        &mut self,
        state: &mut State,
        content: &sender_contract::PhysicalContent,
        binding: sender_stage::CueBinding,
        active: WorkId,
    ) -> Result<bool, Fault> {
        if active != CONTACT_REPLY {
            return Ok(false);
        }
        if !matches!(self, Self::ContactRead) {
            return Err(Fault::Order);
        }
        let retry = content.contact_retry.as_ref().ok_or(Fault::Order)?;
        let receipt = state.sender_stage.receipt().ok_or(Fault::Order)?;
        let queues = state.sender_queues.as_ref().ok_or(Fault::Order)?;
        let qualified = retry
            .qualify(receipt, queues, binding, &content.contact.tail)
            .map_err(Fault::Contract)?;
        let service = Service::Contact;
        state
            .timing
            .begin(service.body(), qualified.body)
            .map_err(Fault::Timing)?;
        *self = Self::Body {
            service,
            publication: qualified.publication,
            completion: qualified.completion,
            tail: qualified.tail,
            prefix: CONTACT_READ_PREFIX,
        };
        Ok(true)
    }

    /// Consume an event routed using the work active before the hardware step.
    /// # Errors
    /// Unexpected CPU effect, repeated publication or different actual milestone.
    pub fn observe(
        &mut self,
        state: &mut State,
        active: WorkId,
        intrinsic: MasterTicks,
        event: &Event,
        binding: sender_stage::CueBinding,
    ) -> Result<(), Fault> {
        match &event.kind {
            EventKind::Payload(_)
            | EventKind::BulkService(_)
            | EventKind::LineService(_)
            | EventKind::Interrupt(_)
            | EventKind::ControllerStart
            | EventKind::ControllerEnd
            | EventKind::ControllerCommitted => return Ok(()),
            EventKind::Sample(_, _) | EventKind::AudioPortRead { .. } => return Err(Fault::Order),
            EventKind::Publication(_) => {}
        }
        let Self::Body {
            service,
            publication: Some(at),
            prefix,
            ..
        } = self
        else {
            return Err(Fault::Order);
        };
        if active != service.body() || intrinsic.0.checked_add(*prefix) != Some(*at) {
            return Err(Fault::Order);
        }
        let expected = state
            .sender_stage
            .publication(binding)
            .map_err(Fault::Sender)?;
        if event.kind != EventKind::Publication(expected) {
            return Err(Fault::Order);
        }
        let mut candidate = state.clone();
        candidate
            .sender_stage
            .present(&mut candidate.senders, active, *at, event.at)
            .map_err(Fault::Sender)?;
        if *service == Service::Contact && *prefix == CONTACT_READ_PREFIX {
            candidate.contact_retried_at = Some(event.at);
        }
        *state = candidate;
        if let Self::Body { publication, .. } = self {
            *publication = None;
        }
        Ok(())
    }

    /// Advance only after hardware reports actual matching work completion.
    /// # Errors
    /// Missing publication, incorrect work or clean dequeue/continuation failure.
    pub fn completed(
        &mut self,
        state: &mut State,
        work: WorkId,
        wall: MasterTicks,
    ) -> Result<(), Fault> {
        let mut candidate = state.clone();
        let next = match self {
            Self::Admission if work == WorkId(1300) => Self::Ready(Service::Action),
            Self::Body {
                service,
                tail,
                publication: None,
                completion,
                prefix,
            } if work == service.body() => {
                if *service == Service::Contact && *prefix == CONTACT_READ_PREFIX {
                    candidate.completed_contact_read =
                        Some(candidate.sender_stage.contact_reply().ok_or(Fault::Order)?);
                }
                let receipt = candidate
                    .sender_stage
                    .complete(&mut candidate.senders, work, *completion, wall)
                    .map_err(Fault::Sender)?;
                let sender = match service {
                    Service::Action => &candidate.senders.action.0,
                    Service::Ambient => &candidate.senders.ambient.0,
                    Service::Contact => &candidate.senders.contact.0,
                };
                candidate
                    .sender_queues
                    .as_mut()
                    .ok_or(Fault::Order)?
                    .completed(&receipt, sender)
                    .map_err(Fault::Contract)?;
                candidate
                    .timing
                    .begin(service.tail(), tail.clone())
                    .map_err(Fault::Timing)?;
                Self::Tail(*service)
            }
            Self::Tail(service) if work == service.tail() => match service {
                Service::Action => Self::Ready(Service::Ambient),
                Service::Ambient => Self::Ready(Service::Contact),
                Service::Contact => Self::Done,
            },
            Self::New
            | Self::Admission
            | Self::Ready(_)
            | Self::ContactReading
            | Self::ContactRead
            | Self::Body { .. }
            | Self::Tail(_)
            | Self::Done => return Err(Fault::Order),
        };
        *state = candidate;
        *self = next;
        Ok(())
    }
}

/// Deliver newly completed appearance offers and update physical positions atomically.
/// # Errors
/// Unsupported producer request, invalid prefix or invalid physical append.
pub fn deliver_appearance(state: &mut State, permission: OfferPermission) -> Result<(), Fault> {
    let mut candidate = state.clone();
    let before = candidate.senders.contact.0.clone();
    candidate
        .sound_offers
        .appearance(&mut candidate.senders, &candidate.appearance, permission)
        .map_err(Fault::Sender)?;
    let after = &candidate.senders.contact.0;
    let queues = candidate.sender_queues.as_mut().ok_or(Fault::Order)?;
    if before.queue.is_empty()
        && after
            .queue
            .iter()
            .copied()
            .eq([audio_transport::CueId::Footstep])
    {
        queues
            .footstep(&before, after, permission)
            .map_err(Fault::Contract)?;
    } else {
        queues
            .offered(SoundGroup::Contact, &before, after)
            .map_err(Fault::Contract)?;
    }
    *state = candidate;
    Ok(())
}
