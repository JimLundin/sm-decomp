//! Finalize the actual sprite builder, then enter the existing native-return owner.

use super::{
    Buffer, Caller, Content, Dependency, Materialization, Outputs, State, native_return,
    suffix_caller,
};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::Canonical;
use data::{
    dma::BusAddress,
    timing::{ContentRegion, MasterTicks, Profile, Template, TemplateEffect},
};
use std::borrow::Cow;

pub use super::native_return::ControllerTail;

pub(super) const FINALIZE: timing::WorkId = timing::WorkId(1421);

/// Physical finalization stages around one repeated inactive-slot write.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finalization {
    setup: Profile,
    hide_slot: Profile,
    finish: Profile,
}

impl Finalization {
    /// Admit setup, one unused slot and finish as effect-free physical work.
    ///
    /// # Errors
    /// Any part contains an unqualified effect or cannot bind its physical work.
    pub fn new(
        setup: &Template,
        hide_slot: &Template,
        finish: &Template,
    ) -> Result<Self, ProductionFault> {
        let bind = |template: &Template| {
            if template
                .runs
                .iter()
                .any(|run| run.effect != TemplateEffect::None)
            {
                return Err(missing());
            }
            template.bind(&[]).map_err(|_| missing())
        };
        Ok(Self {
            setup: bind(setup)?,
            hide_slot: bind(hide_slot)?,
            finish: bind(finish)?,
        })
    }

    /// Physical work after the builder drew the supplied slot count.
    ///
    /// # Errors
    /// The bounded finalizer requires at least one unused slot.
    pub fn from_drawn(&self, drawn: u16) -> Result<Profile, ProductionFault> {
        let unused = crate::state::video::OAM_SPRITES
            .checked_sub(usize::from(drawn))
            .filter(|&unused| unused != 0)
            .ok_or_else(missing)?;
        let runs = self
            .setup
            .runs
            .iter()
            .chain((0..unused).flat_map(|_| &self.hide_slot.runs))
            .chain(&self.finish.runs)
            .cloned()
            .collect();
        Ok(Profile { runs })
    }
}

/// Complete immutable finalization and NMI work with admitted picture bindings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract {
    /// Origin-backed current unselected video inputs.
    pub initial: suffix_caller::InitialBorrowedVideo,
    /// Whole rendering content for revalidating selected upload bytes.
    pub rendering: super::rendering_stage::Content,
    /// Complete admitted semantic picture catalogue.
    pub pictures: Vec<suffix_caller::PictureNmi>,
    /// Whole sprite-list finalization with no device effects.
    pub finalization: Profile,
    /// Whole adjacent finalization with the current drawn sprite count.
    pub adjacent_finalization: Option<Profile>,
    /// Recurring finalization from the actual drawn slot count.
    pub recurring_finalization: Option<Finalization>,
    /// Exact transfer location for the actual finalized list.
    pub builder_address: BusAddress,
    /// Complete software request writer.
    pub preparation: Template,
    /// Complete physical handler body.
    pub nmi: Template,
    /// Common controller prefix and live-selected physical tails for the adjacent NMI.
    pub adjacent_nmi: Option<ControllerTail>,
    /// Prefix ending in the actual controller-busy sample.
    pub prefix_and_poll: Template,
    /// Exact ordered typed NMI publication projections.
    pub recipes: Vec<suffix_caller::PublicationRecipe>,
}

impl Contract {
    fn native_work(&self, state: &State) -> Result<native_return::Work<'_>, ProductionFault> {
        let controller_tail = if state.publications.original_interval() {
            None
        } else {
            Some(self.adjacent_nmi.as_ref().ok_or_else(missing)?)
        };
        Ok(native_return::Work {
            preparation: &self.preparation,
            nmi: &self.nmi,
            controller_tail,
            requires_reset_sample: state.publications.original_interval()
                && self.adjacent_nmi.is_some(),
            prefix_and_poll: &self.prefix_and_poll,
            recipes: &self.recipes,
        })
    }

    fn finalization(&self, state: &State) -> Result<Cow<'_, Profile>, ProductionFault> {
        if state.publications.original_interval() {
            Ok(Cow::Borrowed(&self.finalization))
        } else {
            let drawing = state.rendering.receipt().ok_or_else(missing)?;
            if drawing.first_byte != 0 {
                return Err(missing());
            }
            if let Some(physical) = &self.recurring_finalization {
                return physical.from_drawn(drawing.pieces).map(Cow::Owned);
            }
            self.adjacent_finalization
                .as_ref()
                .filter(|_| drawing.pieces == 9)
                .map(Cow::Borrowed)
                .ok_or_else(missing)
        }
    }

    fn picture(
        &self,
        drawing: &super::rendering_stage::NormalDrawingReceipt,
    ) -> Result<&suffix_caller::PictureNmi, ProductionFault> {
        let mut pictures = self.pictures.iter().filter(|picture| {
            picture.figure == drawing.figure.0 && picture.step.0 == drawing.shown_step.0
        });
        pictures
            .next()
            .filter(|_| pictures.next().is_none())
            .ok_or_else(missing)
    }
}

/// Canonical publication boundary established when real hardware admits the final NMI.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Admission {
    publication: u32,
    at: MasterTicks,
}

/// Current finalizing builder and physical native return progress.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// Drawing exists but its spare slots have not been finalized.
    #[default]
    Pending,
    /// Actual builder is held until the complete physical finalization returns.
    Finalizing(super::borrowed::SpriteBuilderOrigin),
    /// Current complete OAM backing with actual native continuation.
    Ready {
        /// Sole final builder that produced the transfer content.
        builder: super::borrowed::SpriteBuilderOrigin,
        /// Real request, wait, entry and body progress.
        suffix: suffix_caller::Suffix,
        /// Publication boundary of the actual native admission when reached.
        admission: Option<Admission>,
    },
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::NativeReturnPreparation))
}

impl State {
    pub(super) fn validate_finalization_checkpoint(
        &self,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        if matches!(self.return_progress, Stage::Finalizing(_)) {
            let contract = content.return_work.as_ref().ok_or_else(missing)?;
            let profile = contract.finalization(self)?;
            if !self.has_physical_bill(FINALIZE, profile.as_ref()) {
                return Err(missing());
            }
        }
        Ok(())
    }

    pub(super) fn advance_finalization(
        &mut self,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let contract = content.return_work.as_ref().ok_or_else(missing)?;
        match &self.return_progress {
            Stage::Pending => {
                let mut builder = self.s6_drawing.builder().ok_or_else(missing)?.clone();
                self.rendering
                    .finalise(&mut builder, FINALIZE)
                    .map_err(|_| missing())?;
                self.timing
                    .begin(FINALIZE, contract.finalization(self)?.into_owned())
                    .map_err(ProductionFault::FrameHardware)?;
                self.return_progress = Stage::Finalizing(builder);
            }
            Stage::Finalizing(_) => {
                let profile = contract.finalization(self)?;
                if !self.timing.work.iter().any(|work| {
                    work.kind == timing::WorkKind::Semantic(FINALIZE)
                        && &work.profile == profile.as_ref()
                }) {
                    return Err(missing());
                }
            }
            Stage::Ready { .. } => return Err(missing()),
        }
        Ok(())
    }

    pub(super) fn complete_finalization(
        &mut self,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let contract = content.return_work.as_ref().ok_or_else(missing)?;
        let Stage::Finalizing(builder) = &self.return_progress else {
            return Err(missing());
        };
        let bytes = self
            .rendering
            .ready(FINALIZE, builder)
            .map_err(|_| missing())?;
        self.content
            .complete(Materialization {
                buffer: Buffer::SpriteDrawList,
                region: ContentRegion {
                    start: contract.builder_address,
                    bytes,
                },
            })
            .map_err(|_| missing())?;
        self.return_progress = Stage::Ready {
            builder: builder.clone(),
            suffix: suffix_caller::Suffix::Unread,
            admission: None,
        };
        Ok(())
    }

    pub(super) fn advance_frame_senders(
        &mut self,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let fail =
            || ProductionFault::Unsupported(Unsupported::Frame(Dependency::FrameSenderPreparation));
        let mut sender = self.sender_progress.clone();
        let history = if sender == super::sender_caller::Cursor::New {
            if self.sender_history.is_some() {
                return Err(fail());
            }
            self.validate_player_entry().map_err(|_| fail())?;
            Some(super::sender_caller::History::capture(self).ok_or_else(fail)?)
        } else {
            None
        };
        if let Some(identity) = sender.active() {
            if !self
                .timing
                .work
                .iter()
                .any(|work| work.kind == timing::WorkKind::Semantic(identity))
            {
                return Err(fail());
            }
        } else {
            sender
                .begin(
                    self,
                    content.sender_physical.as_ref().ok_or_else(fail)?,
                    content.sender_cue.ok_or_else(fail)?,
                    self.timing.hardware.now,
                )
                .map_err(|_| fail())?;
        }
        self.sender_progress = sender;
        if let Some(history) = history {
            self.sender_history = Some(history);
        }
        Ok(())
    }

    pub(super) fn process_frame_sender(
        &mut self,
        content: &Content,
        active: timing::WorkId,
        progress: timing::Progress,
        events: &mut Vec<timing::Event>,
    ) -> Result<(), ProductionFault> {
        let fail =
            || ProductionFault::Unsupported(Unsupported::Frame(Dependency::FrameSenderPreparation));
        let mut sender = self.sender_progress.clone();
        let binding = content.sender_cue.ok_or_else(fail)?;
        let intrinsic = self
            .timing
            .work
            .iter()
            .find(|work| work.kind == timing::WorkKind::Semantic(active))
            .map(|work| work.cursor.intrinsic);
        let physical = content.sender_physical.as_ref().ok_or_else(fail)?;
        let sample = sender
            .sample_contact_if_due(self, physical, active, intrinsic, binding)
            .map_err(|_| fail())?;
        for event in &*events {
            if matches!(event.kind, timing::EventKind::Publication(_)) {
                sender
                    .observe(self, active, intrinsic.ok_or_else(fail)?, event, binding)
                    .map_err(|_| fail())?;
            }
        }
        events.extend(sample);
        if progress == timing::Progress::Completed(active)
            && !sender
                .complete_contact_prefix(self, physical, binding, active)
                .map_err(|_| fail())?
        {
            sender
                .completed(self, active, self.timing.hardware.now)
                .map_err(|_| fail())?;
        }
        self.sender_progress = sender;
        Ok(())
    }

    // Return the early owners' complete gameplay publication prefix. An earlier
    // unexpected NMI never becomes valid merely because a later NMI was admitted.
    pub(super) fn gameplay_publications(
        &self,
    ) -> Result<(&[timing::Event], bool), ProductionFault> {
        use data::timing::Interrupt;
        use suffix_caller::{NMI_CONTROLLER_TAIL, NMI_ENTRY, NMI_VIDEO, PREPARATION, Suffix, WAIT};
        use timing::{HandlerCompletion, WorkKind};

        let all = self
            .publications
            .active()
            .map_err(ProductionFault::FramePublications)?;
        let Stage::Ready {
            suffix, admission, ..
        } = &self.return_progress
        else {
            return if self.frame_tail == super::frame_tail::Stage::Complete {
                Err(missing())
            } else {
                Ok((all, false))
            };
        };
        if matches!(suffix, Suffix::Unread)
            && admission.is_none()
            && self.frame_tail != super::frame_tail::Stage::Complete
        {
            return Ok((all, false));
        }

        let returned = matches!(suffix, Suffix::Acknowledged { .. });
        if self.frame_tail != super::frame_tail::Stage::Complete
            || !self.room_quake.is_complete()
            || !self.play_time.is_complete()
            || !self
                .frame_history
                .as_ref()
                .is_some_and(|history| history.finalized_for_native(returned))
            || self.timing.pending_handler.is_some()
        {
            return Err(missing());
        }
        let work = self.timing.work.as_slice();
        let kind = if let [work] = work {
            Some(&work.kind)
        } else {
            None
        };
        match (suffix, kind, *admission) {
            (Suffix::Unread | Suffix::Requested { .. }, None, None) if work.is_empty() => {
                Ok((all, false))
            }
            (Suffix::Preparing { .. }, Some(WorkKind::Semantic(PREPARATION)), None)
            | (Suffix::Waiting { .. }, Some(WorkKind::Wait(WAIT, _)), None) => Ok((all, false)),
            (
                Suffix::Entering { .. },
                Some(WorkKind::Native {
                    interrupt: Interrupt::Nmi,
                    identity: NMI_ENTRY,
                    ..
                }),
                Some(admission),
            )
            | (
                Suffix::Video { .. },
                Some(WorkKind::Handler {
                    identity: NMI_ENTRY,
                    portion: NMI_VIDEO,
                    ..
                }),
                Some(admission),
            )
            | (
                Suffix::ControllerTail { .. },
                Some(WorkKind::Handler {
                    identity: NMI_ENTRY,
                    portion: NMI_CONTROLLER_TAIL,
                    completion: HandlerCompletion::NativeReturn,
                    ..
                }),
                Some(admission),
            ) => retained_gameplay_prefix(all, admission, self.timing.hardware.now),
            (Suffix::Acknowledged { .. }, None, Some(admission)) if work.is_empty() => {
                retained_gameplay_prefix(all, admission, self.timing.hardware.now)
            }
            _ => Err(missing()),
        }
    }

    pub(super) fn validate_controller_tail(
        &self,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let Stage::Ready {
            suffix: suffix_caller::Suffix::ControllerTail { controller, .. },
            ..
        } = &self.return_progress
        else {
            return Ok(());
        };
        let tail = content
            .return_work
            .as_ref()
            .ok_or_else(missing)?
            .native_work(self)?
            .controller_tail
            .ok_or_else(missing)?;
        let held = controller.word().ok_or_else(missing)?;
        let profile = tail.at_sample(self, held).map_err(|_| missing())?;
        if !matches!(self.timing.work.as_slice(), [work] if
            matches!(work.kind, timing::WorkKind::Handler {
                identity: suffix_caller::NMI_ENTRY,
                portion: suffix_caller::NMI_CONTROLLER_TAIL,
                completion: timing::HandlerCompletion::NativeReturn,
                ..
            })
            && &work.profile == profile)
        {
            return Err(missing());
        }
        Ok(())
    }

    pub(super) fn validate_native_controller(&self) -> Result<(), ProductionFault> {
        use suffix_caller::Suffix;
        let Stage::Ready {
            suffix, admission, ..
        } = &self.return_progress
        else {
            return Ok(());
        };
        let (Suffix::Video { controller, .. }
        | Suffix::ControllerTail { controller, .. }
        | Suffix::Acknowledged { controller }) = suffix
        else {
            return Ok(());
        };
        let publications = self
            .publications
            .active()
            .map_err(ProductionFault::FramePublications)?;
        let admission = admission.ok_or_else(missing)?;
        let (prefix, _) =
            retained_gameplay_prefix(publications, admission, self.timing.hardware.now)?;
        let events = publications.get(prefix.len()..).ok_or_else(missing)?;
        if !controller.matches_samples(events) {
            return Err(missing());
        }
        if let Suffix::Video { body, sampled, .. } = suffix {
            let count = usize::try_from(*sampled).map_err(|_| missing())?;
            let expected = body.samples.get(..count).ok_or_else(missing)?;
            let recorded = events.iter().filter_map(|event| {
                if let timing::EventKind::Sample(sample, _) = event.kind {
                    Some(sample)
                } else {
                    None
                }
            });
            if !recorded.eq(expected.iter().copied()) {
                return Err(missing());
            }
        }
        Ok(())
    }

    pub(super) fn validate_finalized_builder(&self) -> Result<(), ProductionFault> {
        use super::rendering_stage::Stage as Rendering;
        let (builder, receipt) = match (&self.return_progress, &self.rendering) {
            (Stage::Pending, _) => return Ok(()),
            (Stage::Finalizing(builder), Rendering::Finalizing { work, receipt })
                if *work == FINALIZE =>
            {
                (builder, receipt)
            }
            (Stage::Ready { builder, .. }, Rendering::Ready(receipt)) => (builder, receipt),
            _ => return Err(missing()),
        };
        let mut expected = self.s6_drawing.builder().ok_or_else(missing)?.clone();
        Rendering::Drawn(receipt.clone())
            .finalise(&mut expected, FINALIZE)
            .map_err(|_| missing())?;
        if *builder != expected {
            return Err(missing());
        }
        Ok(())
    }

    pub(super) fn advance_after_shake(
        &mut self,
        content: &Content,
        budget: MasterTicks,
    ) -> Result<Option<Outputs>, ProductionFault> {
        use super::frame_tail::Stage as Tail;
        if matches!(self.frame_tail, Tail::Pending | Tail::Closing) {
            self.advance_frame_tail(content)?;
        } else if self.sender_progress != super::sender_caller::Cursor::Done {
            self.advance_frame_senders(content)?;
        } else if !matches!(self.return_progress, Stage::Ready { .. }) {
            self.advance_finalization(content)?;
        } else if self.frame_tail != Tail::Complete {
            self.advance_frame_tail(content)?;
        } else {
            let queue = self
                .completed_late_hud_queue(content)?
                .ok_or_else(missing)?;
            return self.advance_s7(content, &queue, budget).map(Some);
        }
        Ok(None)
    }

    pub(super) fn advance_s7(
        &mut self,
        content: &Content,
        queue: &suffix_caller::HudQueue,
        budget: MasterTicks,
    ) -> Result<Outputs, ProductionFault> {
        self.validate_controller_tail(content)?;
        let contract = content.return_work.as_ref().ok_or_else(missing)?;
        let Stage::Ready {
            builder,
            suffix: current,
            admission,
        } = &self.return_progress
        else {
            return Err(missing());
        };
        let Caller::PlayerContinuation {
            prepared,
            remaining,
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        let suffix_caller::BackgroundScroll::Complete(background) = &self.background else {
            return Err(missing());
        };
        let drawing = self.rendering.receipt().ok_or_else(missing)?;
        let picture = contract.picture(drawing)?;
        let ready = suffix_caller::ReadyOwners {
            state: self,
            prepared,
            remaining,
            builder,
        };
        let owners = suffix_caller::NmiOwners {
            ready,
            initial: &contract.initial,
            immutable: &picture.immutable,
            selected: &picture.selected,
            rendering: &contract.rendering,
            background,
            background_inputs: &content.background.as_ref().ok_or_else(missing)?.inputs,
            queue,
        };
        let work = contract.native_work(self)?;
        let mut timing = self.timing.clone();
        let mut suffix = current.clone();
        let mut regions = self.transfer_regions();
        for upload in std::iter::once(&picture.selected.top).chain(picture.selected.bottom.as_ref())
        {
            regions.push(upload.first.clone());
            regions.extend(upload.second.clone());
        }
        let (progress, events) = suffix
            .advance_native_return(&owners, &work, &mut timing, &regions, budget)
            .map_err(|fault| match fault {
                suffix_caller::Fault::Hardware(fault) => ProductionFault::FrameHardware(fault),
                suffix_caller::Fault::MissingProducer
                | suffix_caller::Fault::ChangedProducer
                | suffix_caller::Fault::Profile
                | suffix_caller::Fault::Order
                | suffix_caller::Fault::Completion
                | suffix_caller::Fault::MissingCurrentNmiOwnerBinding
                | suffix_caller::Fault::ControllerBusy
                | suffix_caller::Fault::UnsupportedBackgroundControl => missing(),
            })?;
        let admission = self.record_native_admission(current, &suffix, *admission, &events)?;
        let native_completion = if progress == super::Progress::Completed {
            let suffix_caller::Suffix::Acknowledged { controller } = &suffix else {
                return Err(missing());
            };
            Some((
                controller.word().ok_or_else(missing)?,
                prepared.layers.terminal,
            ))
        } else {
            None
        };
        self.timing = timing;
        self.return_progress = Stage::Ready {
            builder: builder.clone(),
            suffix,
            admission,
        };
        if let Some((held, layers)) = native_completion {
            self.complete_requested_return(held, layers)?;
        }
        self.publications
            .record(events)
            .map_err(ProductionFault::FramePublications)?;
        Ok(Outputs {
            progress,
            events: self
                .publications
                .deliver()
                .map_err(ProductionFault::FramePublications)?,
        })
    }

    fn complete_requested_return(
        &mut self,
        held: u16,
        layers: super::layer_blend::PreparedLayerSettings,
    ) -> Result<(), ProductionFault> {
        if !matches!(
            self.late_hud,
            super::late_hud::Stage::Completed(suffix_caller::HudQueue::Queued(_))
        ) || !self
            .frame_history
            .as_mut()
            .is_some_and(|history| history.capture_nmi(held))
        {
            return Err(missing());
        }
        self.late_hud = super::late_hud::Stage::Retired;
        self.hud_frame_count = self.hud_frame_count.map(|frame| frame.wrapping_add(1));
        if let Some(premises) = &mut self.inside_premises {
            premises.complete_requested_nmi();
        }
        if let Some(drawing) = &mut self.drawing {
            let [_, _, _, main, _, _, _, window, math] =
                super::layer_blend::encode(layers).map_err(|_| missing())?;
            drawing.current.video.main_layers = main;
            drawing.current.video.colour_window = window;
            drawing.current.video.colour_math = math;
        }
        Ok(())
    }

    fn record_native_admission(
        &self,
        current: &suffix_caller::Suffix,
        next: &suffix_caller::Suffix,
        admission: Option<Admission>,
        events: &[timing::Event],
    ) -> Result<Option<Admission>, ProductionFault> {
        if matches!(current, suffix_caller::Suffix::Waiting { .. })
            && matches!(next, suffix_caller::Suffix::Entering { .. })
        {
            if admission.is_some() {
                return Err(missing());
            }
            let (local, event) = events
                .iter()
                .enumerate()
                .find(|(_, event)| {
                    event.kind == timing::EventKind::Interrupt(data::timing::Interrupt::Nmi)
                })
                .ok_or_else(missing)?;
            let publication = self
                .publications
                .active()
                .map_err(ProductionFault::FramePublications)?
                .len()
                .checked_add(local)
                .and_then(|index| u32::try_from(index).ok())
                .ok_or_else(missing)?;
            Ok(Some(Admission {
                publication,
                at: event.at,
            }))
        } else {
            Ok(admission)
        }
    }
}

fn retained_gameplay_prefix(
    all: &[timing::Event],
    admission: Admission,
    now: MasterTicks,
) -> Result<(&[timing::Event], bool), ProductionFault> {
    use data::timing::{Interrupt, WRAM_DATA_PORT};
    use timing::EventKind;

    let index = usize::try_from(admission.publication).map_err(|_| missing())?;
    let is_nmi = |event: &timing::Event| event.kind == EventKind::Interrupt(Interrupt::Nmi);
    if admission.at > now
        || !all
            .get(index)
            .is_some_and(|event| is_nmi(event) && event.at == admission.at)
    {
        return Err(missing());
    }
    let (prefix, suffix) = all.split_at_checked(index).ok_or_else(missing)?;
    if prefix.iter().any(|event| match &event.kind {
        EventKind::Publication(write) => write.port == WRAM_DATA_PORT,
        EventKind::Payload(payload) => payload.target.publication_port() == WRAM_DATA_PORT,
        EventKind::Interrupt(Interrupt::Nmi) => true,
        EventKind::Interrupt(Interrupt::Irq)
        | EventKind::Sample(..)
        | EventKind::BulkService(_)
        | EventKind::LineService(_)
        | EventKind::ControllerStart
        | EventKind::ControllerEnd
        | EventKind::ControllerCommitted
        | EventKind::AudioPortRead { .. } => false,
    }) || suffix.iter().skip(1).any(is_nmi)
    {
        return Err(missing());
    }
    Ok((prefix, true))
}
