//! Prepared producer capabilities and the physical wait/native-return continuation.

use super::borrowed::SpriteBuilderOrigin;
use super::borrowed_player_preparation::PlayerContinuationCompletionsPin;
use super::borrowed_preparation::BuilderCompletionReceipt;
use super::inputs::FigurePin;
use super::rendering_stage::{NormalDrawingReceipt, Stage as RenderingStage};
use super::{Buffer, Materialization, Preparation, State, haze};
use crate::production::timing::{self, WorkId};
use crate::state::owned::MovementPin;
use canonical::{Canonical, model::Model};
use data::timing::{Interrupt, Profile, Sample, Template, TemplateEffect};
use types::Parts;

/// Complete software publication-request writer.
pub const PREPARATION: WorkId = WorkId(1400);
/// Physical wait cadence, independent of frame duration.
pub const WAIT: WorkId = WorkId(1401);
/// Hardware-owned native entry for the actual NMI.
pub const NMI_ENTRY: WorkId = WorkId(1410);
/// Complete source NMI body, including controller reads and native return.
pub const NMI_VIDEO: WorkId = WorkId(1411);

/// Actual producer owners; no captured terminal buffer is an input.
#[derive(Clone, Copy)]
pub struct ReadyOwners<'a> {
    /// Sole current selected, rendering, HUD and transfer owners.
    pub state: &'a State,
    /// Actual completed prelude and S1 producer receipts.
    pub prepared: &'a Preparation,
    /// Actual completed remaining player-owner receipts.
    pub remaining: &'a Model<PlayerContinuationCompletionsPin>,
    /// Actual mutable sprite builder after its finalization Ready gate.
    pub builder: &'a SpriteBuilderOrigin,
}

/// Complete retained producer evidence, separate from the software request writer.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct PreparedVideo {
    builder_reset: BuilderCompletionReceipt,
    builder: SpriteBuilderOrigin,
    haze: haze::Prepared,
    remaining: Model<PlayerContinuationCompletionsPin>,
    movement: Model<MovementPin>,
    figure: Model<FigurePin>,
    drawing: NormalDrawingReceipt,
    hud: super::hud::Hud,
    buffers: [Materialization; 3],
}

/// Missing or stale actual owners, source profile or physical continuation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// A required producer has not published its actual terminal Ready.
    MissingProducer,
    /// A saved producer or live materialization differs from its current owner.
    ChangedProducer,
    /// The whole physical profile or sample binding is incompatible.
    Profile,
    /// The current software continuation cannot accept this operation.
    Order,
    /// The returned actual work identity or completed sample sequence differs.
    Completion,
    /// The physical timing service rejected admission or continuation.
    Hardware(timing::Fault),
    /// Typed actual NMI publication-owner bindings are still required.
    MissingCurrentNmiOwnerBinding,
    /// The actual automatic-controller busy bit takes an unqualified poll-repeat branch.
    ControllerBusy,
    /// Current freeze or scroll disposition needs a different source-qualified preparation.
    UnsupportedBackgroundControl,
}

impl PreparedVideo {
    /// Obtain a capability only from actual completed producer owners and live buffers.
    ///
    /// # Errors
    /// A producer or buffer is absent, stale, or inconsistent with its live owner.
    pub fn qualify(owners: ReadyOwners<'_>) -> Result<Self, Fault> {
        let state = owners.state;
        let builder_reset = owners
            .prepared
            .builder
            .as_ref()
            .ok_or(Fault::MissingProducer)?;
        if owners.remaining.0.projectiles.is_none()
            || owners.remaining.0.interaction.is_none()
            || owners.remaining.0.sprites.is_none()
            || owners.remaining.0.contact.is_none()
        {
            return Err(Fault::MissingProducer);
        }
        let actors = owners
            .remaining
            .0
            .actors
            .as_ref()
            .ok_or(Fault::MissingProducer)?;
        let membership = owners
            .prepared
            .actors
            .as_ref()
            .ok_or(Fault::MissingProducer)?;
        actors.solid_boxes().map_err(|_| Fault::ChangedProducer)?;
        if actors.interactive != membership.interactive || owners.builder.cursor != 0 {
            return Err(Fault::ChangedProducer);
        }
        let RenderingStage::Ready(drawing) = &state.rendering else {
            return Err(Fault::MissingProducer);
        };
        let current = state.movement.state();
        if drawing.centre != [current.position.x.pixel().0, current.position.y.pixel().0]
            || !drawing.figure.eq(&state.playing_figure)
        {
            return Err(Fault::ChangedProducer);
        }
        let (hud, _) = state.hud.publication().ok_or(Fault::MissingProducer)?;
        let hud_bytes = super::hud::tilemap(hud).map_err(|_| Fault::ChangedProducer)?;
        let haze_bytes =
            super::borrowed_prelude_receipts::encode_bands(&owners.prepared.haze.bands)
                .map_err(|_| Fault::ChangedProducer)?;
        let buffer = |kind| {
            state
                .content
                .ready_buffer(kind)
                .cloned()
                .ok_or(Fault::MissingProducer)
        };
        let buffers = [
            buffer(Buffer::HazeBackdropBands)?,
            buffer(Buffer::SpriteDrawList)?,
            buffer(Buffer::HudTilemap)?,
        ];
        if buffers[0].region.bytes.as_slice() != haze_bytes
            || buffers[1].region.bytes.as_slice() != owners.builder.backing
            || buffers[2].region.bytes != hud_bytes
        {
            return Err(Fault::ChangedProducer);
        }
        state
            .content
            .validate()
            .map_err(|_| Fault::ChangedProducer)?;
        Ok(Self {
            builder_reset: builder_reset.clone(),
            builder: owners.builder.clone(),
            haze: owners.prepared.haze.clone(),
            remaining: owners.remaining.clone(),
            movement: Model(*current),
            figure: state.playing_figure,
            drawing: drawing.clone(),
            hud: state.hud.clone(),
            buffers,
        })
    }

    /// Revalidate every saved producer quantity against the current sole owners.
    ///
    /// # Errors
    /// A producer is missing or differs from the retained capability.
    pub fn validate(&self, owners: ReadyOwners<'_>) -> Result<(), Fault> {
        if !self.eq(&Self::qualify(owners)?) {
            return Err(Fault::ChangedProducer);
        }
        Ok(())
    }

    /// Live completed source regions; native device effects read these through `TransferContent`.
    #[must_use]
    pub const fn regions(&self) -> &[Materialization; 3] {
        &self.buffers
    }
}

/// Physical NMI body supplied only after its actual current owner bindings qualify.
/// Native entry is owned by timing; this whole profile includes its actual controller/RTI tail.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct QualifiedNmiBody {
    profile: Profile,
    samples: Vec<Sample>,
}

/// Current samples consumed from actual hardware events, never seeded from a captured reply.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct ControllerRead {
    low: Option<u8>,
    high: Option<u8>,
}

impl ControllerRead {
    /// Actual controller word after both physical byte samples have been consumed.
    #[must_use]
    pub fn word(&self) -> Option<u16> {
        Some(u16::from_le_bytes([self.low?, self.high?]))
    }
}

/// Finite software owner continuation; physical wait iterations remain in timing.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub enum Suffix {
    /// The software request writer has not begun this frame.
    Unread,
    /// The actual complete software writer is physically charging.
    Preparing {
        /// Completed actual producers retained across physical charging and interrupts.
        ready: Box<PreparedVideo>,
    },
    /// The actual writer completed and published its unconditional request.
    Requested {
        /// Completed actual producers retained across physical charging and interrupts.
        ready: Box<PreparedVideo>,
    },
    /// Hardware owns repeated wait polling until an actual interrupt.
    Waiting {
        /// Completed actual producers retained across physical charging and interrupts.
        ready: Box<PreparedVideo>,
    },
    /// The actual NMI has begun its hardware-owned native entry.
    Entering {
        /// Completed actual producers retained across physical charging and interrupts.
        ready: Box<PreparedVideo>,
    },
    /// The actual full NMI body is charging with retained live producer evidence.
    Video {
        /// Completed actual producers retained across physical charging and interrupts.
        ready: Box<PreparedVideo>,
        /// Complete actual NMI profile and its ordered physical sample consumers.
        body: QualifiedNmiBody,
        /// Number of actual source samples already consumed from hardware events.
        sampled: u32,
        /// Actual controller bytes sampled by this NMI, with no preview seed.
        controller: ControllerRead,
    },
    /// The complete NMI returned; its request is acknowledged once.
    Acknowledged {
        /// Actual controller bytes sampled by this NMI, with no preview seed.
        controller: ControllerRead,
    },
}

impl Suffix {
    /// Start the complete S6 software request writer without consulting its old request value.
    ///
    /// # Errors
    /// The writer is out of order, the profile has device effects, or timing rejects admission.
    pub fn begin_preparation(
        &mut self,
        ready: PreparedVideo,
        whole: &Template,
        timing: &mut timing::State,
    ) -> Result<(), Fault> {
        if !matches!(self, Self::Unread) {
            return Err(Fault::Order);
        }
        if whole
            .runs
            .iter()
            .any(|run| run.effect != TemplateEffect::None)
        {
            return Err(Fault::Profile);
        }
        let profile = whole.bind(&[]).map_err(|_| Fault::Profile)?;
        timing
            .begin(PREPARATION, profile)
            .map_err(Fault::Hardware)?;
        *self = Self::Preparing {
            ready: Box::new(ready),
        };
        Ok(())
    }

    /// The unconditional software request becomes visible only at its actual completed writer.
    ///
    /// # Errors
    /// The writer is out of order, the work identity differs, or a producer has changed.
    pub fn complete_preparation(
        &mut self,
        work: WorkId,
        owners: ReadyOwners<'_>,
    ) -> Result<(), Fault> {
        let Self::Preparing { ready } = self else {
            return Err(Fault::Order);
        };
        if work != PREPARATION {
            return Err(Fault::Completion);
        }
        ready.validate(owners)?;
        *self = Self::Requested {
            ready: ready.clone(),
        };
        Ok(())
    }

    /// Begin hardware-owned cadence; no requested duration, counter or expected NMI selects it.
    ///
    /// # Errors
    /// The request is out of order or timing rejects the wait.
    pub fn begin_wait(&mut self, timing: &mut timing::State) -> Result<(), Fault> {
        let Self::Requested { ready } = self else {
            return Err(Fault::Order);
        };
        timing.begin_wait(WAIT).map_err(Fault::Hardware)?;
        *self = Self::Waiting {
            ready: ready.clone(),
        };
        Ok(())
    }

    /// Admit native entry only at an actual NMI hardware boundary; IRQs stay parent drawing work.
    ///
    /// # Errors
    /// The current boundary or poll admission differs, or timing rejects native entry.
    pub fn enter_nmi(
        &mut self,
        progress: timing::Progress,
        timing: &mut timing::State,
        admission: &IdlePollAdmission,
        regions: &[data::timing::ContentRegion],
        prefix_and_poll: &Profile,
    ) -> Result<Vec<timing::Event>, Fault> {
        let Self::Waiting { ready } = self else {
            return Err(Fault::Order);
        };
        if progress != timing::Progress::Interrupt(Interrupt::Nmi) {
            return Err(Fault::Order);
        }
        admission.validate(timing, regions, prefix_and_poll)?;
        let events = timing.enter_native(NMI_ENTRY).map_err(Fault::Hardware)?;
        *self = Self::Entering {
            ready: ready.clone(),
        };
        Ok(events)
    }

    /// Obtain the full source body only after physical entry and fresh producer revalidation.
    ///
    /// # Errors
    /// Entry has not completed, a producer has changed, or timing rejects the body.
    pub fn begin_nmi(
        &mut self,
        progress: timing::Progress,
        owners: ReadyOwners<'_>,
        body: QualifiedNmiBody,
        timing: &mut timing::State,
    ) -> Result<(), Fault> {
        let Self::Entering { ready } = self else {
            return Err(Fault::Order);
        };
        if progress != timing::Progress::HandlerEntered(NMI_ENTRY) {
            return Err(Fault::Completion);
        }
        ready.validate(owners)?;
        timing
            .begin_handler_portion(
                NMI_VIDEO,
                body.profile.clone(),
                timing::HandlerCompletion::NativeReturn,
            )
            .map_err(Fault::Hardware)?;
        *self = Self::Video {
            ready: ready.clone(),
            body,
            sampled: 0,
            controller: ControllerRead::default(),
        };
        Ok(())
    }

    /// Consume actual device samples in admitted physical order; no synthetic controller input.
    ///
    /// # Errors
    /// The body is inactive, a sample is out of order, or the controller is busy.
    pub fn observe(&mut self, events: &[timing::Event]) -> Result<(), Fault> {
        let mut candidate = self.clone();
        candidate.observe_inner(events)?;
        *self = candidate;
        Ok(())
    }

    fn observe_inner(&mut self, events: &[timing::Event]) -> Result<(), Fault> {
        let Self::Video {
            body,
            sampled,
            controller,
            ..
        } = self
        else {
            return Err(Fault::Order);
        };
        for event in events {
            if let timing::EventKind::Sample(device, value) = event.kind {
                let index = usize::try_from(*sampled).map_err(|_| Fault::Profile)?;
                if body.samples.get(index) != Some(&device) {
                    return Err(Fault::Profile);
                }
                *sampled = sampled.checked_add(1).ok_or(Fault::Profile)?;
                match (device, value) {
                    (Sample::ControllerLow, timing::SampleValue::Byte(byte))
                        if controller.low.is_none() =>
                    {
                        controller.low = Some(byte);
                    }
                    (Sample::ControllerHigh, timing::SampleValue::Byte(byte))
                        if controller.low.is_some() && controller.high.is_none() =>
                    {
                        controller.high = Some(byte);
                    }
                    (Sample::AcknowledgeNmi, timing::SampleValue::Line(_))
                    | (
                        Sample::Blanking,
                        timing::SampleValue::Blanking {
                            controller: false, ..
                        },
                    ) => {}
                    (
                        Sample::Blanking,
                        timing::SampleValue::Blanking {
                            controller: true, ..
                        },
                    ) => {
                        return Err(Fault::ControllerBusy);
                    }
                    _ => return Err(Fault::Profile),
                }
            }
        }
        Ok(())
    }

    /// Acknowledge once only after the actual full NMI profile, including native return.
    ///
    /// # Errors
    /// The body is inactive, the return is incomplete, or controller bytes are missing.
    pub fn complete_nmi(&mut self, progress: timing::Progress) -> Result<(), Fault> {
        let Self::Video {
            body,
            sampled,
            controller,
            ..
        } = self
        else {
            return Err(Fault::Order);
        };
        if progress != timing::Progress::HandlerReturned(NMI_ENTRY)
            || usize::try_from(*sampled) != Ok(body.samples.len())
            || controller.word().is_none()
        {
            return Err(Fault::Completion);
        }
        *self = Self::Acknowledged {
            controller: controller.clone(),
        };
        Ok(())
    }
}

/// Short-lived admission proof from deterministic current hardware, never a future reply seed.
/// This proof is not canonical state and owns no writable hardware authority.
#[derive(Clone, Debug)]
pub struct IdlePollAdmission {
    current: timing::State,
    regions: Vec<data::timing::ContentRegion>,
    prefix_and_poll: Profile,
}

impl IdlePollAdmission {
    /// Reject any changed current hardware, pending work, live content or source bindings.
    ///
    /// # Errors
    /// The current timing state, transfer regions, or bound profile differs.
    pub fn validate(
        &self,
        current: &timing::State,
        regions: &[data::timing::ContentRegion],
        prefix_and_poll: &Profile,
    ) -> Result<(), Fault> {
        if !self.current.eq(current)
            || self.regions.as_slice() != regions
            || !self.prefix_and_poll.eq(prefix_and_poll)
        {
            return Err(Fault::ChangedProducer);
        }
        Ok(())
    }
}

/// Preview only actual deterministic physical entry/work up to its actual busy-bit read.
///
/// Simulated clock, transfers, events and samples are discarded; controller bytes are not read.
/// Busy rejects before the real native entry, physical bill or receiver publication begins.
///
/// # Errors
/// The boundary, bindings, samples, or timing are invalid, or the controller is busy.
pub fn preview_idle_poll(
    current: &timing::State,
    regions: &[data::timing::ContentRegion],
    prefix_and_poll: &Profile,
) -> Result<IdlePollAdmission, Fault> {
    if current.interrupt_boundary != Some(Interrupt::Nmi) {
        return Err(Fault::Order);
    }
    let mut blanking_reads = 0u32;
    for run in &prefix_and_poll.runs {
        if let data::timing::DeviceEffect::Sample(sample) = run.exposure.effect {
            match sample {
                Sample::Blanking => {
                    blanking_reads = blanking_reads
                        .checked_add(run.count)
                        .ok_or(Fault::Profile)?;
                }
                Sample::AcknowledgeNmi => {}
                Sample::ProductLow
                | Sample::ProductHigh
                | Sample::ControllerLow
                | Sample::ControllerHigh
                | Sample::AcknowledgeIrq => return Err(Fault::Profile),
            }
        }
    }
    if blanking_reads != 1 {
        return Err(Fault::Profile);
    }
    let mut candidate = current.clone();
    let _ = candidate.enter_native(NMI_ENTRY).map_err(Fault::Hardware)?;
    loop {
        let (progress, events) = candidate
            .step(data::timing::MasterTicks(u64::MAX), regions)
            .map_err(Fault::Hardware)?;
        if events.iter().any(|event| {
            matches!(
                event.kind,
                timing::EventKind::Sample(Sample::ControllerLow | Sample::ControllerHigh, _)
            )
        }) {
            return Err(Fault::Profile);
        }
        match progress {
            timing::Progress::Advanced => {}
            timing::Progress::HandlerEntered(work) if work == NMI_ENTRY => break,
            timing::Progress::Completed(_)
            | timing::Progress::Interrupt(_)
            | timing::Progress::HandlerEntered(_)
            | timing::Progress::HandlerReturned(_)
            | timing::Progress::HandlerPortionCompleted(_)
            | timing::Progress::HandlerNeedsWork(_)
            | timing::Progress::Idle => return Err(Fault::Profile),
        }
    }
    candidate
        .begin_handler_portion(
            NMI_VIDEO,
            prefix_and_poll.clone(),
            timing::HandlerCompletion::Continue,
        )
        .map_err(Fault::Hardware)?;
    loop {
        let (progress, events) = candidate
            .step(data::timing::MasterTicks(u64::MAX), regions)
            .map_err(Fault::Hardware)?;
        for event in events {
            match event.kind {
                timing::EventKind::Sample(
                    Sample::Blanking,
                    timing::SampleValue::Blanking { controller, .. },
                ) => {
                    if controller {
                        return Err(Fault::ControllerBusy);
                    }
                    return Ok(IdlePollAdmission {
                        current: current.clone(),
                        regions: regions.to_vec(),
                        prefix_and_poll: prefix_and_poll.clone(),
                    });
                }
                timing::EventKind::Sample(Sample::ControllerLow | Sample::ControllerHigh, _) => {
                    return Err(Fault::Profile);
                }
                timing::EventKind::Publication(_)
                | timing::EventKind::Sample(..)
                | timing::EventKind::Payload(_)
                | timing::EventKind::BulkService(_)
                | timing::EventKind::LineService(_)
                | timing::EventKind::Interrupt(_)
                | timing::EventKind::ControllerStart
                | timing::EventKind::ControllerEnd
                | timing::EventKind::ControllerCommitted => {}
            }
        }
        if progress != timing::Progress::Advanced {
            return Err(Fault::Profile);
        }
    }
}

#[cfg(test)]
mod admission_tests {
    use super::*;
    use data::timing::{Cycle, DeviceEffect, Exposure, ExposureKind, MasterTicks, Run};

    fn hardware_at_nmi(busy: bool) -> timing::State {
        use timing::{
            Beam, BeamPosition, Controller, Hardware, Interrupts, Math, Thresholds, Transfers,
        };
        let position = BeamPosition {
            line: 225,
            horizontal: 400,
        };
        let mut state = timing::State {
            hardware: Hardware {
                now: MasterTicks(0),
                beam: Beam {
                    odd: false,
                    position,
                    history: std::array::from_fn(|index| BeamPosition {
                        line: 225,
                        horizontal: [390, 392, 394, 396, 398, 400][index],
                    }),
                    subtick: 0,
                    residue: 0,
                    line_ticks: 1364,
                    refreshed: true,
                    thresholds: Thresholds {
                        initialize_at: 12,
                        initialize: true,
                        visible: true,
                    },
                },
                charge_debt: 0,
                interrupts: Interrupts::default(),
                math: Math::default(),
                controller: Controller {
                    active: busy,
                    iteration: if busy { 1 } else { 17 },
                    ..Controller::default()
                },
                transfers: Transfers::default(),
            },
            work: Vec::new(),
            interrupt_boundary: None,
            pending_handler: None,
        };
        state.begin_wait(WAIT).unwrap();
        state.hardware.interrupts.nmi.pending = true;
        state.interrupt_boundary = Some(Interrupt::Nmi);
        state.validate().unwrap();
        state
    }

    fn status_read() -> Profile {
        Profile {
            runs: vec![Run {
                count: 1,
                exposure: Exposure {
                    cycle: Cycle::Six,
                    kind: ExposureKind::Read,
                    poll_before: false,
                    effect: DeviceEffect::Sample(Sample::Blanking),
                },
            }],
        }
    }

    #[test]
    fn private_actual_busy_preview_changes_no_hardware_clock_work_or_input() {
        let state = hardware_at_nmi(true);
        let before = canonical::to_bytes(&state);
        assert!(
            preview_idle_poll(&state, &[], &status_read())
                .is_err_and(|fault| fault == Fault::ControllerBusy)
        );
        assert!(before.eq(&canonical::to_bytes(&state)));
    }

    #[test]
    fn actual_idle_preview_guards_changed_current_before_real_native_begin() {
        let state = hardware_at_nmi(false);
        let profile = status_read();
        let before = canonical::to_bytes(&state);
        let admission = preview_idle_poll(&state, &[], &profile).unwrap();
        assert!(before.eq(&canonical::to_bytes(&state)));
        assert!(admission.validate(&state, &[], &profile).is_ok());
        let mut changed = state.clone();
        changed.hardware.controller.active = true;
        assert!(
            admission
                .validate(&changed, &[], &profile)
                .is_err_and(|fault| fault == Fault::ChangedProducer)
        );
        let mut changed = profile;
        changed.runs[0].count = 2;
        assert!(
            admission
                .validate(&state, &[], &changed)
                .is_err_and(|fault| fault == Fault::ChangedProducer)
        );
    }
}

/// Current video quantities admitted once at the independent first cut.
/// S1 layer/haze and S6 scroll/upload publications are supplied by their completed owners.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct InitialBorrowedVideo {
    /// Current `BackgroundCharacterBasesOneTwo` wire quantity.
    pub background_character_bases_one_two: u8,
    /// Current `BackgroundCharacterBasesThreeFour` wire quantity.
    pub background_character_bases_three_four: u8,
    /// Current `BackgroundFourHorizontal` wire quantity.
    pub background_four_horizontal: u16,
    /// Current `BackgroundFourMap` wire quantity.
    pub background_four_map: u8,
    /// Current `BackgroundFourVertical` wire quantity.
    pub background_four_vertical: u16,
    /// Current `BackgroundMode` wire quantity.
    pub background_mode: u8,
    /// Current `BackgroundOneMap` wire quantity.
    pub background_one_map: u8,
    /// Current `BackgroundThreeHorizontal` wire quantity.
    pub background_three_horizontal: u16,
    /// Current `BackgroundThreeMap` wire quantity.
    pub background_three_map: u8,
    /// Current `BackgroundThreeVertical` wire quantity.
    pub background_three_vertical: u16,
    /// Current `BackgroundTwoHorizontal` wire quantity.
    pub background_two_horizontal: u16,
    /// Current `BackgroundTwoMap` wire quantity.
    pub background_two_map: u8,
    /// Current `BackgroundWindowCombination` wire quantity.
    pub background_window_combination: u8,
    /// Current `ColorMath` wire quantity.
    pub color_math: u8,
    /// Current `ColorWindow` wire quantity.
    pub color_window: u8,
    /// Current Display wire quantity.
    pub display: u8,
    /// Current `DmaChannelSevenSourceOffset` wire quantity.
    pub dma_channel_seven_source_offset: u16,
    /// Current `DmaChannelTwoSourceOffset` wire quantity.
    pub dma_channel_two_source_offset: u16,
    /// Current `InterruptEnable` wire quantity.
    pub interrupt_enable: u8,
    /// Current `ModeSevenSettings` wire quantity.
    pub mode_seven_settings: u8,
    /// Current Mosaic wire quantity.
    pub mosaic: u8,
    /// Current `ObjectColorWindowCombination` wire quantity.
    pub object_color_window_combination: u8,
    /// Current `ObjectLayout` wire quantity.
    pub object_layout: u8,
    /// Current `ScreenSettings` wire quantity.
    pub screen_settings: u8,
    /// Current `WindowOneLeft` wire quantity.
    pub window_one_left: u8,
    /// Current `WindowOneRight` wire quantity.
    pub window_one_right: u8,
    /// Current `WindowTwoLeft` wire quantity.
    pub window_two_left: u8,
    /// Current `WindowTwoRight` wire quantity.
    pub window_two_right: u8,
}

/// Current background offsets and scaled vertical mode, before the camera reader runs.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct BackgroundScrollInputs {
    /// Actual time-freeze owner permits current scroll preparation.
    pub running: bool,
    /// Current horizontal scroll disposition, whose reached branch preserves its previous value.
    pub horizontal: ScrollDisposition,
    /// Current vertical scroll disposition, whose reached branch uses actual parallax arithmetic.
    pub vertical: ScrollDisposition,
    /// Background one's current horizontal offset from the camera.
    pub one_x_offset: u16,
    /// Background one's current vertical offset from the camera.
    pub one_y_offset: u16,
    /// Background two's current vertical offset from the scaled camera.
    pub two_y_offset: u16,
    /// Current low-byte vertical scaling mode; actual arithmetic reads its even numerator.
    pub vertical_mode: u8,
}

/// One actual rendered tile request joined to checked imported definition content.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct SelectedTileUpload {
    /// Checked immutable definition assigned to the actual selected asset by the importer.
    pub definition: data::pose::tiles::TileDefinitionId,
    /// Checked first segment length from the selected definition.
    pub first_length: u16,
    /// Checked second segment length from the selected definition.
    pub second_length: u16,
    /// Actual selected bytes bound to the checked first source span.
    pub first: data::timing::ContentRegion,
    /// Actual selected second span, absent only when the definition requests none.
    pub second: Option<data::timing::ContentRegion>,
}

/// Actual ordered rendered tile requests; fixed descriptor locations never choose them.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct SelectedTileUploads {
    /// Actual top request and checked content.
    pub top: SelectedTileUpload,
    /// Actual bottom request when the drawing model selects one.
    pub bottom: Option<SelectedTileUpload>,
}

/// Checked immutable HUD descriptor bound to the actual completed tilemap buffer.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct HudTransferTemplate {
    /// Physical source binding of the actual HUD materialization.
    pub source: data::dma::BusAddress,
    /// Checked immutable transfer length, equal to the actual HUD buffer length.
    pub length: u16,
    /// Destination expressed in VRAM words.
    pub target: crate::state::video::VramWord,
}

impl HudTransferTemplate {
    /// Append the actual HUD request to the sole current empty upload queue.
    /// Validate every producer and descriptor before publishing the queued request.
    ///
    /// # Errors
    /// The retained producers, queue, or live descriptor backing differs.
    pub fn append(
        &self,
        ready: &PreparedVideo,
        owners: ReadyOwners<'_>,
        queue: &mut HudQueue,
    ) -> Result<HudQueuedTransfer, Fault> {
        ready.validate(owners)?;
        let [_, _, hud] = ready.regions();
        self.append_hud(&hud.region, queue)
    }

    /// Append from actual late status's backing before final sprite readiness.
    /// # Errors
    /// The queue must be empty and the live backing must match the descriptor.
    pub fn append_hud(
        &self,
        hud: &data::timing::ContentRegion,
        queue: &mut HudQueue,
    ) -> Result<HudQueuedTransfer, Fault> {
        if !matches!(queue, HudQueue::Empty)
            || hud.start != self.source
            || hud.bytes.len() != usize::from(self.length)
            || self.length != 192
        {
            return Err(Fault::ChangedProducer);
        }
        let request = HudQueuedTransfer {
            template: self.clone(),
        };
        *queue = HudQueue::Queued(request.clone());
        Ok(request)
    }
    /// Distinct composite word used by the ordered source-high/bank physical writers.
    #[must_use]
    pub const fn source_high_bank_wire_word(&self) -> u16 {
        let [_, high] = self.source.offset.to_le_bytes();
        u16::from_le_bytes([high, self.source.bank])
    }
}

/// Whole completed background scroll preparation and its retained physical products.
pub const BACKGROUND_SCROLL: WorkId = WorkId(1420);

/// Prepared scroll quantities computed from actual current camera and actual hardware samples.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct BackgroundScrollCompletion {
    /// Actual completed camera retained through both arithmetic requests.
    pub camera: Model<crate::state::owned::CameraPin>,
    /// Current offsets and scaling mode retained when the reader began.
    pub inputs: BackgroundScrollInputs,
    /// Actual low-byte parallax product's sampled high byte.
    pub low_product_high: u8,
    /// Actual high-byte parallax product's complete sampled word.
    pub high_product: u16,
    /// Actual prepared background one horizontal scroll.
    pub one_horizontal: u16,
    /// Actual prepared background one vertical scroll.
    pub one_vertical: u16,
    /// Actual scaled layer-two vertical scroll before its background offset.
    pub layer_two_vertical: u16,
    /// Actual prepared background two vertical scroll.
    pub two_vertical: u16,
}

/// Retained two-request parallax continuation; no predicted result supplies a scroll quantity.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum BackgroundScroll {
    /// Current camera has not yet reached its scroll reader.
    #[default]
    Pending,
    /// The actual whole scroll preparation retains all current inputs and hardware samples.
    Charging {
        /// Actual completed current camera, unchanged until this owner finishes.
        camera: Model<crate::state::owned::CameraPin>,
        /// Actual current offsets and scaling mode.
        inputs: BackgroundScrollInputs,
        /// Actual first product high byte; its low byte is intentionally unread.
        first_high: Option<u8>,
        /// Actual second product low byte.
        second_low: Option<u8>,
        /// Actual second product high byte.
        second_high: Option<u8>,
    },
    /// Complete scroll preparation, visible only after actual physical completion.
    Complete(BackgroundScrollCompletion),
}

impl BackgroundScroll {
    /// Bind both actual requests only after the sole current camera completes its follow.
    ///
    /// # Errors
    /// The reader is out of order, lacks a camera, or cannot bind the offered profile.
    pub fn begin(
        &mut self,
        stage: &super::camera_stage::Stage,
        camera: &Model<crate::state::owned::CameraPin>,
        inputs: &BackgroundScrollInputs,
        whole: &Template,
    ) -> Result<Profile, Fault> {
        if !matches!(self, Self::Pending) {
            return Err(Fault::Order);
        }
        if !inputs.running
            || inputs.horizontal != ScrollDisposition::Preserve
            || inputs.vertical != ScrollDisposition::Scaled
        {
            return Err(Fault::UnsupportedBackgroundControl);
        }
        let [_, y] = stage.viewport(camera).ok_or(Fault::MissingProducer)?;
        let [low, high] = y.to_le_bytes();
        let numerator = inputs.vertical_mode & 0xfe;
        let mut values = [numerator, low, high].into_iter();
        let mut writes = Vec::new();
        let mut samples = Vec::new();
        for run in &whole.runs {
            for _ in 0..run.count {
                match run.effect {
                    TemplateEffect::None => {}
                    TemplateEffect::Publish(port) => {
                        let value = values.next().ok_or(Fault::Profile)?;
                        let write = data::timing::DeviceWrite::from_cpu_port(port.0, value)
                            .map_err(|_| Fault::Profile)?;
                        let valid = match (writes.len(), samples.len()) {
                            (0, 0) => {
                                matches!(write.effect, data::timing::Publication::Multiplicand(_))
                            }
                            (1, 0) | (2, 1) => {
                                matches!(write.effect, data::timing::Publication::Multiplier(_))
                            }
                            _ => false,
                        };
                        if !valid {
                            return Err(Fault::Profile);
                        }
                        writes.push(write);
                    }
                    TemplateEffect::Sample(sample) => {
                        let expected = match (writes.len(), samples.len()) {
                            (2, 0) | (3, 2) => Sample::ProductHigh,
                            (3, 1) => Sample::ProductLow,
                            _ => return Err(Fault::Profile),
                        };
                        if sample != expected {
                            return Err(Fault::Profile);
                        }
                        samples.push(sample);
                    }
                }
            }
        }
        if values.next().is_some()
            || samples != [Sample::ProductHigh, Sample::ProductLow, Sample::ProductHigh]
        {
            return Err(Fault::Profile);
        }
        let profile = whole.bind(&writes).map_err(|_| Fault::Profile)?;
        *self = Self::Charging {
            camera: *camera,
            inputs: inputs.clone(),
            first_high: None,
            second_low: None,
            second_high: None,
        };
        Ok(profile)
    }

    /// Consume only actual physical samples from this whole background owner.
    ///
    /// # Errors
    /// The reader is inactive, the work identity differs, or a sample is out of order.
    pub fn sample(&mut self, work: WorkId, device: Sample, byte: u8) -> Result<(), Fault> {
        let Self::Charging {
            first_high,
            second_low,
            second_high,
            ..
        } = self
        else {
            return Err(Fault::Order);
        };
        if work != BACKGROUND_SCROLL {
            return Err(Fault::Completion);
        }
        match (device, *first_high, *second_low, *second_high) {
            (Sample::ProductHigh, None, None, None) => *first_high = Some(byte),
            (Sample::ProductLow, Some(_), None, None) => *second_low = Some(byte),
            (Sample::ProductHigh, Some(_), Some(_), None) => *second_high = Some(byte),
            _ => return Err(Fault::Profile),
        }
        Ok(())
    }

    /// Publish scroll quantities only from complete actual samples and unchanged current inputs.
    ///
    /// # Errors
    /// The reader is inactive or its work, captured inputs, or sampled products differ.
    pub fn complete(
        &mut self,
        work: WorkId,
        current_camera: &Model<crate::state::owned::CameraPin>,
        current_inputs: &BackgroundScrollInputs,
    ) -> Result<(), Fault> {
        let Self::Charging {
            camera,
            inputs,
            first_high,
            second_low,
            second_high,
        } = self
        else {
            return Err(Fault::Order);
        };
        if work != BACKGROUND_SCROLL || &*camera != current_camera || &*inputs != current_inputs {
            return Err(Fault::ChangedProducer);
        }
        let first = first_high.ok_or(Fault::Completion)?;
        let second = u16::from_le_bytes([
            second_low.ok_or(Fault::Completion)?,
            second_high.ok_or(Fault::Completion)?,
        ]);
        let position = camera.0.position();
        let x = position.x.pixel().0;
        let y = position.y.pixel().0;
        let [low, high] = y.to_le_bytes();
        let numerator = u16::from(inputs.vertical_mode & 0xfe);
        let [_, expected_first] = u16::from(low).wrapping_mul(numerator).to_le_bytes();
        if first != expected_first || second != u16::from(high).wrapping_mul(numerator) {
            return Err(Fault::Completion);
        }
        let layer_two_vertical = u16::from(first).wrapping_add(second);
        *self = Self::Complete(BackgroundScrollCompletion {
            camera: *camera,
            inputs: inputs.clone(),
            low_product_high: first,
            high_product: second,
            one_horizontal: x.wrapping_add(inputs.one_x_offset),
            one_vertical: y.wrapping_add(inputs.one_y_offset),
            layer_two_vertical,
            two_vertical: layer_two_vertical.wrapping_add(inputs.two_y_offset),
        });
        Ok(())
    }
}

/// Actual compact HUD transfer request retained until its NMI consumer.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct HudQueuedTransfer {
    /// Checked descriptor bound to the actual completed live HUD buffer.
    pub template: HudTransferTemplate,
}

/// Sole scoped current HUD upload queue; initial emptiness is independently admitted.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub enum HudQueue {
    /// Current queue is empty, before the actual preparation append.
    Empty,
    /// The actual append completed; its request remains until the NMI consumer.
    Queued(HudQueuedTransfer),
}

impl HudQueue {
    /// Admit the reached empty queue from its actual initial full-width cursor.
    ///
    /// # Errors
    /// The externally supplied initial cursor is nonzero.
    pub const fn from_initial_cursor(cursor: u16) -> Result<Self, Fault> {
        if cursor != 0 {
            return Err(Fault::ChangedProducer);
        }
        Ok(Self::Empty)
    }
}

/// Checked immutable NMI operands and selected descriptor fields, kept as distinct quantities.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct ImmutableNmiContent {
    /// Checked `ObjectUploadControlAndTarget` quantity.
    pub object_upload_control_and_target: u16,
    /// Checked `ObjectUploadSourceOffset` quantity.
    pub object_upload_source_offset: u16,
    /// Checked `ObjectUploadSourceBank` quantity.
    pub object_upload_source_bank: u8,
    /// Checked `ObjectUploadLength` quantity.
    pub object_upload_length: u16,
    /// Checked `PaletteUploadControlAndTarget` quantity.
    pub palette_upload_control_and_target: u16,
    /// Checked `PaletteUploadSourceOffset` quantity.
    pub palette_upload_source_offset: u16,
    /// Checked `PaletteUploadSourceBank` quantity.
    pub palette_upload_source_bank: u8,
    /// Checked `PaletteUploadLength` quantity.
    pub palette_upload_length: u16,
    /// Checked `PaletteStart` quantity.
    pub palette_start: u8,
    /// Checked `ObjectPaletteUploadEnable` quantity.
    pub object_palette_upload_enable: u8,
    /// Checked `SamusTileUploadFirstLayout` quantity.
    pub samus_tile_upload_first_layout: u8,
    /// Checked `SamusTileUploadFirstTarget` quantity.
    pub samus_tile_upload_first_target: u16,
    /// Checked `SamusTileUploadFirstControlAndTarget` quantity.
    pub samus_tile_upload_first_control_and_target: u16,
    /// Checked `SamusTileUploadFirstSourceOffset` quantity.
    pub samus_tile_upload_first_source_offset: u16,
    /// Checked `SamusTileUploadFirstBankAndLengthLow` quantity.
    pub samus_tile_upload_first_bank_and_length_low: u16,
    /// Checked `SamusTileUploadFirstLength` quantity.
    pub samus_tile_upload_first_length: u16,
    /// Checked `SamusTileUploadFirstEnable` quantity.
    pub samus_tile_upload_first_enable: u8,
    /// Checked `SamusTileUploadFirstSecondSegmentTarget` quantity.
    pub samus_tile_upload_first_second_segment_target: u16,
    /// Checked `SamusTileUploadFirstSecondSegmentLength` quantity.
    pub samus_tile_upload_first_second_segment_length: u16,
    /// Checked `SamusTileUploadSecondTarget` quantity.
    pub samus_tile_upload_second_target: u16,
    /// Checked `SamusTileUploadSecondControlAndTarget` quantity.
    pub samus_tile_upload_second_control_and_target: u16,
    /// Checked `SamusTileUploadSecondSourceOffset` quantity.
    pub samus_tile_upload_second_source_offset: u16,
    /// Checked `SamusTileUploadSecondBankAndLengthLow` quantity.
    pub samus_tile_upload_second_bank_and_length_low: u16,
    /// Checked `SamusTileUploadSecondLength` quantity.
    pub samus_tile_upload_second_length: u16,
    /// Checked `SamusTileUploadSecondSecondSegmentTarget` quantity.
    pub samus_tile_upload_second_second_segment_target: u16,
    /// Checked `SamusTileUploadSecondSecondSegmentLength` quantity.
    pub samus_tile_upload_second_second_segment_length: u16,
    /// Checked `HudUploadControlAndTarget` quantity.
    pub hud_upload_control_and_target: u16,
    /// Checked `HudUploadLayoutAndTargetLow` quantity.
    pub hud_upload_layout_and_target_low: u16,
    /// Checked `HudUploadEnable` quantity.
    pub hud_upload_enable: u8,
    /// Checked `VramLayoutAfterUpload` quantity.
    pub vram_layout_after_upload: u8,
    /// Checked `VramLayoutAfterBuffer` quantity.
    pub vram_layout_after_buffer: u8,
}

/// A byte or a named word byte projected from a semantic quantity.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum WordPart {
    /// Complete byte quantity.
    Byte,
    /// Low byte of a word quantity.
    Low,
    /// High byte of a word quantity.
    High,
}

// Keep the canonical quantity identities next to their wire projection.
macro_rules! nmi_quantities {
    ($owners:ident, $values:ident; $($(#[$doc:meta])* $name:ident => $projection:expr,)*) => {
        /// Distinct actual NMI quantities. Equal bytes never identify their sources.
        #[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
        pub enum NmiQuantity {
            $($(#[$doc])* $name,)*
        }

        impl NmiQuantity {
            fn value(self, $owners: &NmiOwners<'_>) -> Result<WireValue, Fault> {
                use WireValue::{Byte, Word};
                let $values = NmiValues::prepare($owners)?;
                Ok(match self {
                    $(Self::$name => $projection,)*
                })
            }
        }
    };
}

nmi_quantities! {
    owners, values;
    /// Actual `InitialInterruptEnable` quantity.
    InitialInterruptEnable => Byte(owners.initial.interrupt_enable),
    /// Actual `InitialDisplay` quantity.
    InitialDisplay => Byte(owners.initial.display),
    /// Actual `InitialObjectLayout` quantity.
    InitialObjectLayout => Byte(owners.initial.object_layout),
    /// Actual `InitialBackgroundMode` quantity.
    InitialBackgroundMode => Byte(owners.initial.background_mode),
    /// Actual `InitialMosaic` quantity.
    InitialMosaic => Byte(owners.initial.mosaic),
    /// Actual `InitialBackgroundOneMap` quantity.
    InitialBackgroundOneMap => Byte(owners.initial.background_one_map),
    /// Actual `InitialBackgroundTwoMap` quantity.
    InitialBackgroundTwoMap => Byte(owners.initial.background_two_map),
    /// Actual `InitialBackgroundThreeMap` quantity.
    InitialBackgroundThreeMap => Byte(owners.initial.background_three_map),
    /// Actual `InitialBackgroundFourMap` quantity.
    InitialBackgroundFourMap => Byte(owners.initial.background_four_map),
    /// Actual `InitialBackgroundCharacterBasesOneTwo` quantity.
    InitialBackgroundCharacterBasesOneTwo => Byte(owners.initial.background_character_bases_one_two),
    /// Actual `InitialBackgroundCharacterBasesThreeFour` quantity.
    InitialBackgroundCharacterBasesThreeFour => Byte(owners.initial.background_character_bases_three_four),
    /// Actual `InitialModeSevenSettings` quantity.
    InitialModeSevenSettings => Byte(owners.initial.mode_seven_settings),
    /// Actual `InitialWindowOneLeft` quantity.
    InitialWindowOneLeft => Byte(owners.initial.window_one_left),
    /// Actual `InitialWindowOneRight` quantity.
    InitialWindowOneRight => Byte(owners.initial.window_one_right),
    /// Actual `InitialWindowTwoLeft` quantity.
    InitialWindowTwoLeft => Byte(owners.initial.window_two_left),
    /// Actual `InitialWindowTwoRight` quantity.
    InitialWindowTwoRight => Byte(owners.initial.window_two_right),
    /// Actual `InitialBackgroundWindowCombination` quantity.
    InitialBackgroundWindowCombination => Byte(owners.initial.background_window_combination),
    /// Actual `InitialObjectColorWindowCombination` quantity.
    InitialObjectColorWindowCombination => Byte(owners.initial.object_color_window_combination),
    /// Actual `InitialColorWindow` quantity.
    InitialColorWindow => Byte(owners.initial.color_window),
    /// Actual `InitialColorMath` quantity.
    InitialColorMath => Byte(owners.initial.color_math),
    /// Actual `InitialScreenSettings` quantity.
    InitialScreenSettings => Byte(owners.initial.screen_settings),
    /// Actual `InitialBackgroundTwoHorizontal` quantity.
    InitialBackgroundTwoHorizontal => Word(owners.initial.background_two_horizontal),
    /// Actual `InitialBackgroundThreeHorizontal` quantity.
    InitialBackgroundThreeHorizontal => Word(owners.initial.background_three_horizontal),
    /// Actual `InitialBackgroundThreeVertical` quantity.
    InitialBackgroundThreeVertical => Word(owners.initial.background_three_vertical),
    /// Actual `InitialBackgroundFourHorizontal` quantity.
    InitialBackgroundFourHorizontal => Word(owners.initial.background_four_horizontal),
    /// Actual `InitialBackgroundFourVertical` quantity.
    InitialBackgroundFourVertical => Word(owners.initial.background_four_vertical),
    /// Actual `InitialDmaChannelTwoSourceOffset` quantity.
    InitialDmaChannelTwoSourceOffset => Word(owners.initial.dma_channel_two_source_offset),
    /// Actual `InitialDmaChannelSevenSourceOffset` quantity.
    InitialDmaChannelSevenSourceOffset => Word(owners.initial.dma_channel_seven_source_offset),
    /// Actual `ImmutableObjectUploadControlAndTarget` quantity.
    ImmutableObjectUploadControlAndTarget => Word(owners.immutable.object_upload_control_and_target),
    /// Actual `ImmutableObjectUploadSourceOffset` quantity.
    ImmutableObjectUploadSourceOffset => Word(owners.immutable.object_upload_source_offset),
    /// Actual `ImmutableObjectUploadSourceBank` quantity.
    ImmutableObjectUploadSourceBank => Byte(owners.immutable.object_upload_source_bank),
    /// Actual `ImmutableObjectUploadLength` quantity.
    ImmutableObjectUploadLength => Word(owners.immutable.object_upload_length),
    /// Actual `ImmutablePaletteUploadControlAndTarget` quantity.
    ImmutablePaletteUploadControlAndTarget => Word(owners.immutable.palette_upload_control_and_target),
    /// Actual `ImmutablePaletteUploadSourceOffset` quantity.
    ImmutablePaletteUploadSourceOffset => Word(owners.immutable.palette_upload_source_offset),
    /// Actual `ImmutablePaletteUploadSourceBank` quantity.
    ImmutablePaletteUploadSourceBank => Byte(owners.immutable.palette_upload_source_bank),
    /// Actual `ImmutablePaletteUploadLength` quantity.
    ImmutablePaletteUploadLength => Word(owners.immutable.palette_upload_length),
    /// Actual `ImmutablePaletteStart` quantity.
    ImmutablePaletteStart => Byte(owners.immutable.palette_start),
    /// Actual `ImmutableObjectPaletteUploadEnable` quantity.
    ImmutableObjectPaletteUploadEnable => Byte(owners.immutable.object_palette_upload_enable),
    /// Actual `ImmutableSamusTileUploadFirstLayout` quantity.
    ImmutableSamusTileUploadFirstLayout => Byte(owners.immutable.samus_tile_upload_first_layout),
    /// Actual `ImmutableSamusTileUploadFirstTarget` quantity.
    ImmutableSamusTileUploadFirstTarget => Word(owners.immutable.samus_tile_upload_first_target),
    /// Actual `ImmutableSamusTileUploadFirstControlAndTarget` quantity.
    ImmutableSamusTileUploadFirstControlAndTarget => Word(owners.immutable.samus_tile_upload_first_control_and_target),
    /// Actual `ImmutableSamusTileUploadFirstSourceOffset` quantity.
    ImmutableSamusTileUploadFirstSourceOffset => Word(owners.immutable.samus_tile_upload_first_source_offset),
    /// Actual `ImmutableSamusTileUploadFirstBankAndLengthLow` quantity.
    ImmutableSamusTileUploadFirstBankAndLengthLow => Word(owners.immutable.samus_tile_upload_first_bank_and_length_low),
    /// Actual `ImmutableSamusTileUploadFirstLength` quantity.
    ImmutableSamusTileUploadFirstLength => Word(owners.immutable.samus_tile_upload_first_length),
    /// Actual `ImmutableSamusTileUploadFirstEnable` quantity.
    ImmutableSamusTileUploadFirstEnable => Byte(owners.immutable.samus_tile_upload_first_enable),
    /// Actual `ImmutableSamusTileUploadFirstSecondSegmentTarget` quantity.
    ImmutableSamusTileUploadFirstSecondSegmentTarget => Word(owners.immutable.samus_tile_upload_first_second_segment_target),
    /// Actual `ImmutableSamusTileUploadFirstSecondSegmentLength` quantity.
    ImmutableSamusTileUploadFirstSecondSegmentLength => Word(owners.immutable.samus_tile_upload_first_second_segment_length),
    /// Actual `ImmutableSamusTileUploadSecondTarget` quantity.
    ImmutableSamusTileUploadSecondTarget => Word(owners.immutable.samus_tile_upload_second_target),
    /// Actual `ImmutableSamusTileUploadSecondControlAndTarget` quantity.
    ImmutableSamusTileUploadSecondControlAndTarget => Word(owners.immutable.samus_tile_upload_second_control_and_target),
    /// Actual `ImmutableSamusTileUploadSecondSourceOffset` quantity.
    ImmutableSamusTileUploadSecondSourceOffset => Word(owners.immutable.samus_tile_upload_second_source_offset),
    /// Actual `ImmutableSamusTileUploadSecondBankAndLengthLow` quantity.
    ImmutableSamusTileUploadSecondBankAndLengthLow => Word(owners.immutable.samus_tile_upload_second_bank_and_length_low),
    /// Actual `ImmutableSamusTileUploadSecondLength` quantity.
    ImmutableSamusTileUploadSecondLength => Word(owners.immutable.samus_tile_upload_second_length),
    /// Actual `ImmutableSamusTileUploadSecondSecondSegmentTarget` quantity.
    ImmutableSamusTileUploadSecondSecondSegmentTarget => Word(owners.immutable.samus_tile_upload_second_second_segment_target),
    /// Actual `ImmutableSamusTileUploadSecondSecondSegmentLength` quantity.
    ImmutableSamusTileUploadSecondSecondSegmentLength => Word(owners.immutable.samus_tile_upload_second_second_segment_length),
    /// Actual `ImmutableHudUploadControlAndTarget` quantity.
    ImmutableHudUploadControlAndTarget => Word(owners.immutable.hud_upload_control_and_target),
    /// Actual `ImmutableHudUploadLayoutAndTargetLow` quantity.
    ImmutableHudUploadLayoutAndTargetLow => Word(owners.immutable.hud_upload_layout_and_target_low),
    /// Actual `ImmutableHudUploadEnable` quantity.
    ImmutableHudUploadEnable => Byte(owners.immutable.hud_upload_enable),
    /// Actual `ImmutableVramLayoutAfterUpload` quantity.
    ImmutableVramLayoutAfterUpload => Byte(owners.immutable.vram_layout_after_upload),
    /// Actual `ImmutableVramLayoutAfterBuffer` quantity.
    ImmutableVramLayoutAfterBuffer => Byte(owners.immutable.vram_layout_after_buffer),
    /// Actual `LayerBlendBackgroundWindowOneTwo` quantity.
    LayerBlendBackgroundWindowOneTwo => Byte(values.window_one_two),
    /// Actual `LayerBlendBackgroundWindowThreeFour` quantity.
    LayerBlendBackgroundWindowThreeFour => Byte(values.window_three_four),
    /// Actual `LayerBlendObjectColorWindow` quantity.
    LayerBlendObjectColorWindow => Byte(values.objects),
    /// Actual `LayerBlendMainLayers` quantity.
    LayerBlendMainLayers => Byte(values.main),
    /// Actual `LayerBlendSubLayers` quantity.
    LayerBlendSubLayers => Byte(values.sub),
    /// Actual `LayerBlendMainWindowLayers` quantity.
    LayerBlendMainWindowLayers => Byte(values.main_window),
    /// Actual `LayerBlendSubWindowLayers` quantity.
    LayerBlendSubWindowLayers => Byte(values.sub_window),
    /// Actual `HazeRedWire` quantity.
    HazeRedWire => Byte(values.red),
    /// Actual `HazeGreenWire` quantity.
    HazeGreenWire => Byte(values.green),
    /// Actual `HazeBlueWire` quantity.
    HazeBlueWire => Byte(values.blue),
    /// Actual `HazeEnabledChannels` quantity.
    HazeEnabledChannels => Byte(haze_channels(
                &super::haze::CHANNELS,
            )?),
    /// Actual `BackgroundOneHorizontal` quantity.
    BackgroundOneHorizontal => Word(owners.background.one_horizontal),
    /// Actual `BackgroundOneVertical` quantity.
    BackgroundOneVertical => Word(owners.background.one_vertical),
    /// Actual `BackgroundTwoVertical` quantity.
    BackgroundTwoVertical => Word(owners.background.two_vertical),
    /// Actual `HudLength` quantity.
    HudLength => Word(values.hud.length),
    /// Actual `HudSourceOffset` quantity.
    HudSourceOffset => Word(values.hud.source.offset),
    /// Actual `HudSourceHighBankWireWord` quantity.
    HudSourceHighBankWireWord => Word(values.hud.source_high_bank_wire_word()),
    /// Actual `HudTarget` quantity.
    HudTarget => Word(values.hud.target.0),
    /// Actual `FirstAdvancedSourceOffset` quantity.
    FirstAdvancedSourceOffset => Word(advanced_source_offset(&owners.selected.top)?),
    /// Actual `SecondAdvancedSourceOffset` quantity.
    SecondAdvancedSourceOffset => Word(advanced_source_offset(
                owners
                    .selected
                    .bottom
                    .as_ref()
                    .ok_or(Fault::MissingProducer)?,
            )?),
    /// Actual `ObjectAddressReset` quantity.
    ObjectAddressReset => Word(0),
}

/// Source-qualified physical publication of one actual typed quantity.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublicationRecipe {
    /// Actual owner and quantity read at this opportunity.
    pub quantity: NmiQuantity,
    /// Semantic byte projection, independent of source read width.
    pub part: WordPart,
    /// Qualified physical publication port.
    pub port: data::timing::PublicationPort,
}

/// Checked immutable ROM binding of a particular picture's upload request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PictureNmi {
    /// The validated actual semantic selection key.
    pub figure: samus_appearance::Figure,
    /// The validated playing step.
    pub step: samus_appearance::Step,
    /// Immutable checked native bindings for the selected top and bottom output.
    pub selected: SelectedTileUploads,
    /// Actual immutable operands and descriptor fields for this selection.
    pub immutable: ImmutableNmiContent,
}

/// Current sole producer owners and checked immutable content at the NMI reader.
pub struct NmiOwners<'a> {
    /// Fresh actual completed drawing, HUD, prelude and remaining producers.
    pub ready: ReadyOwners<'a>,
    /// Independently admitted current initial video quantities.
    pub initial: &'a InitialBorrowedVideo,
    /// Checked immutable operands and selected request descriptor fields.
    pub immutable: &'a ImmutableNmiContent,
    /// Actual rendered requests bound to immutable definition identity and selected bytes.
    pub selected: &'a SelectedTileUploads,
    /// Immutable rendering content used to revalidate actual selected bytes.
    pub rendering: &'a super::rendering_stage::Content,
    /// Actual completed scroll quantities and their hardware sample provenance.
    pub background: &'a BackgroundScrollCompletion,
    /// Current scroll input authority, unchanged since background preparation.
    pub background_inputs: &'a BackgroundScrollInputs,
    /// Actual scoped upload queue after the completed HUD append.
    pub queue: &'a HudQueue,
}

enum WireValue {
    Byte(u8),
    Word(u16),
}

impl WireValue {
    const fn project(self, part: WordPart) -> Result<u8, Fault> {
        match (self, part) {
            (Self::Byte(byte), WordPart::Byte) => Ok(byte),
            (Self::Word(word), WordPart::Low) => {
                let [low, _] = word.to_le_bytes();
                Ok(low)
            }
            (Self::Word(word), WordPart::High) => {
                let [_, high] = word.to_le_bytes();
                Ok(high)
            }
            (Self::Byte(_), WordPart::Low | WordPart::High) | (Self::Word(_), WordPart::Byte) => {
                Err(Fault::Profile)
            }
        }
    }
}

struct NmiValues<'a> {
    window_one_two: u8,
    window_three_four: u8,
    objects: u8,
    main: u8,
    sub: u8,
    main_window: u8,
    sub_window: u8,
    red: u8,
    green: u8,
    blue: u8,
    hud: &'a HudTransferTemplate,
}

impl<'a> NmiValues<'a> {
    fn prepare(owners: &NmiOwners<'a>) -> Result<Self, Fault> {
        let [
            window_one_two,
            window_three_four,
            objects,
            main,
            sub,
            main_window,
            sub_window,
            _,
            _,
        ] = super::layer_blend::encode(owners.ready.prepared.layers.terminal)
            .map_err(|_| Fault::ChangedProducer)?;
        let [red, green, blue] = super::borrowed_prelude_receipts::encode_fixed_colour(
            owners.ready.prepared.haze.fixed_colour,
        );
        let HudQueue::Queued(request) = owners.queue else {
            return Err(Fault::MissingProducer);
        };
        Ok(Self {
            window_one_two,
            window_three_four,
            objects,
            main,
            sub,
            main_window,
            sub_window,
            red,
            green,
            blue,
            hud: &request.template,
        })
    }
}

fn haze_channels(channels: &[data::timing::ChannelSlot]) -> Result<u8, Fault> {
    let mut mask = 0u8;
    for slot in channels {
        let bit = match slot {
            data::timing::ChannelSlot::One => 1,
            data::timing::ChannelSlot::Two => 2,
            data::timing::ChannelSlot::Three => 4,
            data::timing::ChannelSlot::Four => 8,
            data::timing::ChannelSlot::Five => 16,
            data::timing::ChannelSlot::Six => 32,
            data::timing::ChannelSlot::Seven => 64,
            data::timing::ChannelSlot::Eight => 128,
        };
        if mask & bit != 0 {
            return Err(Fault::ChangedProducer);
        }
        mask |= bit;
    }
    Ok(mask)
}

fn advanced_source_offset(upload: &SelectedTileUpload) -> Result<u16, Fault> {
    upload
        .first
        .start
        .offset
        .checked_add(upload.first_length)
        .ok_or(Fault::ChangedProducer)
}

impl NmiOwners<'_> {
    fn validate(&self) -> Result<(), Fault> {
        let ready = PreparedVideo::qualify(ReadyOwners {
            state: self.ready.state,
            prepared: self.ready.prepared,
            remaining: self.ready.remaining,
            builder: self.ready.builder,
        })?;
        if !self.background.camera.eq(&self.ready.state.camera)
            || !self.background.inputs.eq(self.background_inputs)
        {
            return Err(Fault::ChangedProducer);
        }
        if !self.background.inputs.running
            || self.background.inputs.horizontal != ScrollDisposition::Preserve
            || self.background.inputs.vertical != ScrollDisposition::Scaled
        {
            return Err(Fault::UnsupportedBackgroundControl);
        }
        let position = self.background.camera.0.position();
        let x = position.x.pixel().0;
        let y = position.y.pixel().0;
        let [low, high] = y.to_le_bytes();
        let numerator = u16::from(self.background.inputs.vertical_mode & 0xfe);
        let [_, expected_low_high] = u16::from(low).wrapping_mul(numerator).to_le_bytes();
        let layer =
            u16::from(self.background.low_product_high).wrapping_add(self.background.high_product);
        if self.background.low_product_high != expected_low_high
            || self.background.high_product != u16::from(high).wrapping_mul(numerator)
            || self.background.layer_two_vertical != layer
            || self.background.one_horizontal != x.wrapping_add(self.background.inputs.one_x_offset)
            || self.background.one_vertical != y.wrapping_add(self.background.inputs.one_y_offset)
            || self.background.two_vertical
                != layer.wrapping_add(self.background.inputs.two_y_offset)
        {
            return Err(Fault::ChangedProducer);
        }
        let HudQueue::Queued(request) = self.queue else {
            return Err(Fault::MissingProducer);
        };
        let [_, _, hud] = ready.regions();
        if request.template.source != hud.region.start
            || usize::from(request.template.length) != hud.region.bytes.len()
            || request.template.length != 192
        {
            return Err(Fault::ChangedProducer);
        }
        let uploads = self
            .rendering
            .uploads(&ready.drawing)
            .map_err(|_| Fault::ChangedProducer)?;
        let bottom = self
            .selected
            .bottom
            .as_ref()
            .ok_or(Fault::MissingProducer)?;
        let [(_, actual_top), (_, actual_bottom)] = uploads.as_slice() else {
            return Err(Fault::ChangedProducer);
        };
        for (selected, actual) in [(&self.selected.top, *actual_top), (bottom, *actual_bottom)] {
            if selected.first.bytes != actual.first
                || selected
                    .second
                    .as_ref()
                    .map(|region| region.bytes.as_slice())
                    .unwrap_or_default()
                    != actual.second.as_slice()
                || selected.first.bytes.len() != usize::from(selected.first_length)
                || selected
                    .second
                    .as_ref()
                    .map_or(0, |region| region.bytes.len())
                    != usize::from(selected.second_length)
            {
                return Err(Fault::ChangedProducer);
            }
        }
        let [first_low, _] = self.selected.top.first_length.to_le_bytes();
        let [second_low, _] = bottom.first_length.to_le_bytes();
        let content = self.immutable;
        if content.samus_tile_upload_first_source_offset != self.selected.top.first.start.offset
            || content.samus_tile_upload_first_bank_and_length_low
                != u16::from_le_bytes([self.selected.top.first.start.bank, first_low])
            || content.samus_tile_upload_first_length != self.selected.top.first_length
            || content.samus_tile_upload_first_second_segment_length
                != self.selected.top.second_length
            || content.samus_tile_upload_second_source_offset != bottom.first.start.offset
            || content.samus_tile_upload_second_bank_and_length_low
                != u16::from_le_bytes([bottom.first.start.bank, second_low])
            || content.samus_tile_upload_second_length != bottom.first_length
            || content.samus_tile_upload_second_second_segment_length != bottom.second_length
        {
            return Err(Fault::ChangedProducer);
        }
        Ok(())
    }
}

impl QualifiedNmiBody {
    /// Resolve the complete body from fresh actual typed owners and checked source recipes.
    /// Every ordered write is retained, including overlapping or repeated physical outputs.
    ///
    /// # Errors
    /// The current publication owners or bindings fail validation.
    pub fn qualify(
        whole: &Template,
        recipes: &[PublicationRecipe],
        owners: &NmiOwners<'_>,
    ) -> Result<Self, Fault> {
        let (profile, samples) = bind_current_nmi(whole, recipes, owners)?;
        if samples
            != [
                Sample::AcknowledgeNmi,
                Sample::Blanking,
                Sample::ControllerLow,
                Sample::ControllerHigh,
            ]
        {
            return Err(Fault::Profile);
        }
        Ok(Self { profile, samples })
    }
}

/// Current layer-two scroll disposition, independent of its original encoding.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollDisposition {
    /// The current layer follows the viewport directly.
    Follow,
    /// The current layer keeps its previous horizontal scroll.
    Preserve,
    /// The current layer uses its actual parallax numerator and physical products.
    Scaled,
}

/// Bind only the immutable pre-poll work and actual status read from the same current owners.
/// This profile admits no controller read or native-return suffix.
///
/// # Errors
/// The profile or supplied owners do not qualify the poll prefix.
pub fn bind_idle_prefix_and_poll(
    whole: &Template,
    recipes: &[PublicationRecipe],
    owners: &NmiOwners<'_>,
) -> Result<Profile, Fault> {
    let (profile, samples) = bind_current_nmi(whole, recipes, owners)?;
    if samples != [Sample::AcknowledgeNmi, Sample::Blanking] {
        return Err(Fault::Profile);
    }
    Ok(profile)
}

fn bind_current_nmi(
    whole: &Template,
    recipes: &[PublicationRecipe],
    owners: &NmiOwners<'_>,
) -> Result<(Profile, Vec<Sample>), Fault> {
    if recipes.len() != 122 {
        return Err(Fault::Profile);
    }
    owners.validate()?;
    let mut publications = Vec::new();
    for recipe in recipes {
        let value = recipe.quantity.value(owners)?.project(recipe.part)?;
        let write = data::timing::DeviceWrite::from_cpu_port(recipe.port.0, value)
            .map_err(|_| Fault::Profile)?;
        publications.push(write);
    }
    let profile = whole.bind(&publications).map_err(|_| Fault::Profile)?;
    let samples: Vec<_> = whole
        .runs
        .iter()
        .flat_map(|run| {
            (0..run.count).filter_map(move |_| {
                if let TemplateEffect::Sample(sample) = run.effect {
                    Some(sample)
                } else {
                    None
                }
            })
        })
        .collect();
    Ok((profile, samples))
}
