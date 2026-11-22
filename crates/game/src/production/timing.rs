//! Retained platform time for admitted semantic work and hardware device effects.
//!
//! Charges may pause at any master tick; transfer arbitration is synchronous.
//! Missing content, unsupported nesting and invalid work reject state and outputs together.

mod devices;
#[cfg(test)]
mod fixture_tests;
mod hardware;
#[cfg(test)]
pub(super) mod tests;
mod transfers;

use canonical::Canonical;
use data::timing::{
    ChannelSlot, ContentRegion, Cycle, DeviceEffect, DeviceWrite, ExposureKind, Interrupt,
    IrqPolicy, MasterTicks, Profile, Run, Sample,
};
pub use hardware::{
    Beam, BeamPosition, Channel, Controller, Hardware, InterruptLine, Interrupts, LineStatus,
    LineTransfer, Math, PadChange, Payload, PhysicalLine, PhysicalPad, Thresholds, Transfers,
};

/// Unsupported or invalid hardware service mechanism.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Invalid imported continuation.
    InvalidState,
    /// Empty, incompatible or zero-length exposure profile.
    InvalidProfile,
    /// No nonzero charge or execution budget supplied.
    InvalidCharge,
    /// Master time cannot be represented.
    TimeOverflow,
    /// New pad changes must be at or after live hardware time.
    PadChangeInPast,
    /// Pad presentation times must increase across the pending stream.
    PadChangesUnordered,
    /// Transfer source region was not admitted.
    MissingContent,
    /// Multiple imported regions own one transfer source.
    ContentOverlap,
    /// Transfer destination has no admitted device mechanism.
    UnsupportedDevice,
    /// Line service interrupts bulk transfer outside the admitted premise.
    NestedTransfer,
    /// Overlapping arithmetic starts or an unfinished multiplication sample.
    MathBusy,
    /// No appropriate completed interrupt boundary.
    InterruptBoundary,
    /// Work must finish before a new ordinary request is submitted.
    WorkBusy,
}

/// Independently selected actual semantic request identity.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkId(pub u32);

/// Typed result of a physical sample.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SampleValue {
    /// Physical byte.
    Byte(u8),
    /// Held interrupt line.
    Line(bool),
    /// Physical blanking and polling state.
    Blanking {
        /// Vertical blank.
        vertical: bool,
        /// Horizontal blank.
        horizontal: bool,
        /// Automatic controller active.
        controller: bool,
    },
}

/// Exactly-once physical output or admission.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub enum EventKind {
    /// Completed typed CPU publication.
    Publication(DeviceWrite),
    /// Typed physical device sample.
    Sample(Sample, SampleValue),
    /// Delayed global transfer write.
    Payload(Payload),
    /// Beginning of one bulk channel's body.
    BulkService(ChannelSlot),
    /// Beginning of one line channel's payload body.
    LineService(ChannelSlot),
    /// Native interrupt admission after the selected exposure.
    Interrupt(Interrupt),
    /// Automatic controller serial sequence begins.
    ControllerStart,
    /// Automatic controller serial sequence completes.
    ControllerEnd,
    /// Next independently supplied pad becomes committed.
    ControllerCommitted,
}

/// Ordered physical event with actual retained wall time.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Event {
    /// Seed-relative wall instant.
    pub at: MasterTicks,
    /// Physical output.
    pub kind: EventKind,
}

/// One exposure's resolved wall charges, held across arbitrary partial pauses.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub enum Charge {
    /// One uninterrupted charge chunk.
    Whole {
        /// Uncharged intrinsic ticks.
        remaining: u8,
    },
    /// First read-like chunk, followed by the final four ticks.
    BeforeSample {
        /// Uncharged first-chunk ticks.
        remaining: u8,
    },
    /// The final read-like four ticks.
    AfterSample {
        /// Uncharged final-chunk ticks.
        remaining: u8,
    },
}

/// Exposure cursor stages, each with one transition owner.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Sample permitted interrupt transitions.
    Poll,
    /// Apply write-side arithmetic edge.
    Before,
    /// Resolve pending-sensitive charging and arbitrate synchronous transfers.
    Arbitrate,
    /// Retained partial wall charge.
    Charge(Charge),
    /// Publish the completed write.
    Publish,
    /// Apply read/internal arithmetic edge.
    After,
    /// Finish the exposure before admitting an interrupt.
    Finish,
}

/// Retained finite semantic projection cursor.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Cursor {
    /// Current run ordinal.
    pub run: u32,
    /// Completed repetitions in that run.
    pub occurrence: u32,
    /// Current exposure stage.
    pub stage: Stage,
    /// Intrinsic work charged, excluding all hardware stalls.
    pub intrinsic: MasterTicks,
    /// Completed physical exposure count retained for exact continuation.
    pub completed: u64,
}

impl Cursor {
    fn charge_progress(&self, run: &Run) -> Result<(u8, u8), Fault> {
        let ticks = run.exposure.cycle.ticks();
        let read_like = matches!(
            run.exposure.kind,
            ExposureKind::Read | ExposureKind::PendingInternal
        );
        match self.stage {
            Stage::Poll | Stage::Before | Stage::Arbitrate => Ok((0, 0)),
            Stage::Publish | Stage::After | Stage::Finish => Ok((ticks, 0)),
            Stage::Charge(Charge::Whole { remaining }) => {
                if run.exposure.kind == ExposureKind::Read || remaining == 0 || remaining > ticks {
                    return Err(Fault::InvalidState);
                }
                let progress = ticks.wrapping_sub(remaining);
                Ok((progress, progress))
            }
            Stage::Charge(Charge::BeforeSample { remaining }) => {
                let first = ticks.wrapping_sub(4);
                if !read_like || remaining == 0 || remaining > first {
                    return Err(Fault::InvalidState);
                }
                let progress = first.wrapping_sub(remaining);
                Ok((progress, progress))
            }
            Stage::Charge(Charge::AfterSample { remaining }) => {
                if !read_like || remaining == 0 || remaining > 4 {
                    return Err(Fault::InvalidState);
                }
                Ok((ticks.wrapping_sub(remaining), 4_u8.wrapping_sub(remaining)))
            }
        }
    }
}

/// Continuation owner for a pending actual request.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub enum WorkKind {
    /// Ordinary actual semantic request.
    Semantic(WorkId),
    /// Hardware busy-poll cadence, completed by actual interrupt admission.
    Wait(WorkId),
    /// Native entry preceding an actual admitted handler.
    Native {
        /// Actual interrupt source.
        interrupt: Interrupt,
        /// IRQ policy to restore when the handler completes.
        restore: IrqPolicy,
        /// Already admitted handler profile, independent of expected time.
        handler: Option<HandlerPortion>,
        /// Actual semantic handler identity.
        identity: WorkId,
    },
    /// Handler including its admitted native return opportunities.
    Handler {
        /// Actual handler identity.
        identity: WorkId,
        /// IRQ policy of the suspended parent.
        restore: IrqPolicy,
        /// Actual semantic request identity of this portion.
        portion: WorkId,
        /// Whether this portion includes the completed native return.
        completion: HandlerCompletion,
    },
}

/// Actual boundary reached by a handler work portion.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandlerCompletion {
    /// Return to the actual semantic owner while the interrupt remains active.
    Continue,
    /// This portion includes the completed native return.
    NativeReturn,
}

/// Already admitted optional first handler portion.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct HandlerPortion {
    /// Actual work request identity.
    pub identity: WorkId,
    /// Immutable physical projection of this request.
    pub profile: Profile,
    /// Actual completed boundary.
    pub completion: HandlerCompletion,
}

/// Retained interrupt context waiting for a fresh actual semantic request.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct PendingHandler {
    /// Owner established by the native admission.
    pub identity: WorkId,
    /// Suspended parent's IRQ policy.
    pub restore: IrqPolicy,
}

/// Work and admitted immutable inputs whose complete pending continuation is saved.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Work {
    /// Current work owner.
    pub kind: WorkKind,
    /// Finite immutable admitted physical projection.
    pub profile: Profile,
    /// Owned progress through the projection.
    pub cursor: Cursor,
}

/// Complete saved platform service, including suspended semantic parents.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// Single hardware authority.
    pub hardware: Hardware,
    /// Suspended parents followed by current work.
    pub work: Vec<Work>,
    /// Completed exposure awaiting semantic handler admission.
    pub interrupt_boundary: Option<Interrupt>,
    /// Native entry or an ordinary handler portion completed; the parent stays suspended.
    pub pending_handler: Option<PendingHandler>,
}

/// Actual supported stopping condition returned to the semantic coordinator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Progress {
    /// One physical effect or bounded partial charge was performed.
    Advanced,
    /// Ordinary work completed.
    Completed(WorkId),
    /// Actual interrupt is pending after its qualifying terminal exposure.
    Interrupt(Interrupt),
    /// Native entry completed; handler is now active.
    HandlerEntered(WorkId),
    /// Handler and return completed; parent is resumed with its old mask.
    HandlerReturned(WorkId),
    /// An ordinary handler portion completed without returning from the interrupt.
    HandlerPortionCompleted(WorkId),
    /// Fresh semantic work is required before this interrupt can continue.
    HandlerNeedsWork(WorkId),
    /// No work has been admitted.
    Idle,
}

fn validate_profile(profile: &Profile) -> Result<(), Fault> {
    if profile.runs.is_empty() {
        return Err(Fault::InvalidProfile);
    }
    for run in &profile.runs {
        if run.count == 0 {
            return Err(Fault::InvalidProfile);
        }
        validate_exposure(&run.exposure)?;
    }
    Ok(())
}

fn validate_exposure(exposure: &data::timing::Exposure) -> Result<(), Fault> {
    match exposure.kind {
        ExposureKind::Internal | ExposureKind::PendingInternal => {
            if exposure.cycle != Cycle::Six || exposure.effect != DeviceEffect::None {
                return Err(Fault::InvalidProfile);
            }
        }
        ExposureKind::Read => {
            if matches!(exposure.effect, DeviceEffect::Publish(_)) {
                return Err(Fault::InvalidProfile);
            }
        }
        ExposureKind::Write => {
            if matches!(exposure.effect, DeviceEffect::Sample(_)) {
                return Err(Fault::InvalidProfile);
            }
        }
    }
    if let DeviceEffect::Publish(write) = &exposure.effect {
        write.validate().map_err(|_| Fault::InvalidProfile)?;
    }
    Ok(())
}

const fn work(kind: WorkKind, profile: Profile) -> Work {
    Work {
        kind,
        profile,
        cursor: Cursor {
            run: 0,
            occurrence: 0,
            stage: Stage::Poll,
            intrinsic: MasterTicks(0),
            completed: 0,
        },
    }
}

fn native_entry() -> Profile {
    let make = |count, cycle, kind| Run {
        count,
        exposure: data::timing::Exposure {
            cycle,
            kind,
            poll_before: false,
            effect: DeviceEffect::None,
        },
    };
    Profile {
        runs: vec![
            make(1, Cycle::Six, ExposureKind::Read),
            make(1, Cycle::Six, ExposureKind::Internal),
            make(4, Cycle::Eight, ExposureKind::Write),
            make(2, Cycle::Eight, ExposureKind::Read),
        ],
    }
}

fn wait_turn() -> Profile {
    let make = |count, cycle, kind, poll_before| Run {
        count,
        exposure: data::timing::Exposure {
            cycle,
            kind,
            poll_before,
            effect: DeviceEffect::None,
        },
    };
    Profile {
        runs: vec![
            make(3, Cycle::Six, ExposureKind::Read, false),
            make(1, Cycle::Eight, ExposureKind::Read, true),
            make(2, Cycle::Six, ExposureKind::Read, false),
            make(1, Cycle::Six, ExposureKind::Internal, true),
        ],
    }
}

fn pad_changes_ordered(changes: &[PadChange]) -> bool {
    changes.is_sorted_by(|earlier, later| earlier.at < later.at)
}

impl State {
    pub(super) fn schedule_pad_changes(&mut self, changes: &[PadChange]) -> Result<(), Fault> {
        let pending = &mut self.hardware.controller.pending_pad;
        if let Some(first) = changes.first() {
            if first.at < self.hardware.now {
                return Err(Fault::PadChangeInPast);
            }
            if pending.last().is_some_and(|last| first.at <= last.at)
                || !pad_changes_ordered(changes)
            {
                return Err(Fault::PadChangesUnordered);
            }
            pending.extend_from_slice(changes);
        }
        Ok(())
    }

    /// Validate an independently decoded complete continuation before admitting it.
    ///
    /// # Errors
    /// Invalid hardware phase, pending progress, profiles or charge debt.
    pub fn validate(&self) -> Result<(), Fault> {
        self.hardware.validate_phase()?;
        if !pad_changes_ordered(&self.hardware.controller.pending_pad) {
            return Err(Fault::InvalidState);
        }
        self.handler_waiting()?;
        if self.work.len() > 2 {
            return Err(Fault::InvalidState);
        }
        if self.work.len() == 2
            && (!self.work.first().is_some_and(|pending| {
                matches!(pending.kind, WorkKind::Semantic(_)) && pending.cursor.stage == Stage::Poll
            }) || !self.work.last().is_some_and(|pending| {
                matches!(
                    pending.kind,
                    WorkKind::Native { .. } | WorkKind::Handler { .. }
                )
            }))
        {
            return Err(Fault::InvalidState);
        }
        for pending in &self.work {
            pending.validate_progress()?;
        }
        if self.work.last().is_some_and(|pending| {
            matches!(
                pending.kind,
                WorkKind::Native { .. } | WorkKind::Handler { .. }
            )
        }) && self.hardware.interrupts.policy != IrqPolicy::Masked
        {
            return Err(Fault::InvalidState);
        }
        let debt = self
            .work
            .last()
            .and_then(|pending| {
                usize::try_from(pending.cursor.run)
                    .ok()
                    .and_then(|ordinal| pending.profile.runs.get(ordinal).map(|run| (pending, run)))
            })
            .map_or(Ok(0), |(pending, run)| {
                pending
                    .cursor
                    .charge_progress(run)
                    .map(|(_, debt)| u64::from(debt))
            })?;
        if debt != self.hardware.charge_debt {
            return Err(Fault::InvalidState);
        }
        if self.interrupt_boundary.is_some()
            && (self.work.is_empty()
                || self.work.len() != 1
                || !self.work.last().is_some_and(|pending| {
                    pending.cursor.stage == Stage::Poll
                        && matches!(pending.kind, WorkKind::Semantic(_) | WorkKind::Wait(_))
                })
                || self.hardware.interrupts.pending() != self.interrupt_boundary)
        {
            return Err(Fault::InvalidState);
        }
        Ok(())
    }

    /// Admit a complete actual semantic work receipt into an idle service.
    ///
    /// # Errors
    /// Existing work or invalid finite hardware projection.
    pub fn begin(&mut self, identity: WorkId, profile: Profile) -> Result<(), Fault> {
        self.validate()?;
        validate_profile(&profile)?;
        if !self.work.is_empty()
            || self.interrupt_boundary.is_some()
            || self.pending_handler.is_some()
        {
            return Err(Fault::WorkBusy);
        }
        self.work.push(work(WorkKind::Semantic(identity), profile));
        Ok(())
    }

    /// Begin the qualified busy-poll hardware cadence until an actual interrupt.
    /// The service decides no gameplay consequence of waiting or waking.
    ///
    /// # Errors
    /// Invalid continuation or existing work.
    pub fn begin_wait(&mut self, identity: WorkId) -> Result<(), Fault> {
        self.validate()?;
        if !self.work.is_empty()
            || self.interrupt_boundary.is_some()
            || self.pending_handler.is_some()
        {
            return Err(Fault::WorkBusy);
        }
        self.work.push(work(WorkKind::Wait(identity), wait_turn()));
        Ok(())
    }

    /// Supply the actual semantic handler at a reached hardware interrupt boundary.
    /// The handler projection includes its admitted return work; native entry is owned here.
    ///
    /// # Errors
    /// Missing boundary, unsupported native nesting or invalid handler projection.
    pub fn enter_interrupt(
        &mut self,
        identity: WorkId,
        handler: Profile,
    ) -> Result<Vec<Event>, Fault> {
        self.enter_native_portion(
            identity,
            Some(HandlerPortion {
                identity,
                profile: handler,
                completion: HandlerCompletion::NativeReturn,
            }),
        )
    }

    /// Admit native entry once, then return to the actual owner for fresh handler work.
    ///
    /// # Errors
    /// Missing completed exposure boundary or unsupported native nesting.
    pub fn enter_native(&mut self, identity: WorkId) -> Result<Vec<Event>, Fault> {
        self.enter_native_portion(identity, None)
    }

    /// Admit an actual handler portion after entry or the previous portion completes.
    /// Only an explicit native-return portion restores the suspended parent.
    ///
    /// # Errors
    /// No waiting handler, invalid continuation or unqualified physical projection.
    pub fn begin_handler_portion(
        &mut self,
        portion: WorkId,
        profile: Profile,
        completion: HandlerCompletion,
    ) -> Result<(), Fault> {
        self.validate()?;
        validate_profile(&profile)?;
        let pending = self.pending_handler.take().ok_or(Fault::WorkBusy)?;
        self.work.push(work(
            WorkKind::Handler {
                identity: pending.identity,
                restore: pending.restore,
                portion,
                completion,
            },
            profile,
        ));
        Ok(())
    }

    fn enter_native_portion(
        &mut self,
        identity: WorkId,
        handler: Option<HandlerPortion>,
    ) -> Result<Vec<Event>, Fault> {
        self.validate()?;
        if let Some(portion) = &handler {
            validate_profile(&portion.profile)?;
        }
        let interrupt = self.interrupt_boundary.ok_or(Fault::InterruptBoundary)?;
        if self.work.len() != 1
            || !self.work.first().is_some_and(|pending| {
                matches!(pending.kind, WorkKind::Semantic(_) | WorkKind::Wait(_))
            })
        {
            return Err(Fault::InterruptBoundary);
        }
        if self
            .work
            .first()
            .is_some_and(|pending| matches!(pending.kind, WorkKind::Wait(_)))
        {
            self.work.pop();
        }
        let restore = self.hardware.interrupts.policy;
        match interrupt {
            Interrupt::Nmi => self.hardware.interrupts.nmi.pending = false,
            Interrupt::Irq => self.hardware.interrupts.irq.pending = false,
        }
        self.hardware.interrupts.policy = IrqPolicy::Masked;
        self.interrupt_boundary = None;
        self.work.push(work(
            WorkKind::Native {
                interrupt,
                restore,
                handler,
                identity,
            },
            native_entry(),
        ));
        Ok(vec![Event {
            at: self.hardware.now,
            kind: EventKind::Interrupt(interrupt),
        }])
    }

    /// Perform one physical effect or up to a nonzero intrinsic master-tick budget.
    /// Refresh and synchronous transfer arbitration can advance additional wall ticks.
    /// The budget selects a pause, never a target wall instant or completion criterion.
    ///
    /// # Errors
    /// Invalid continuation, unavailable device/content or unqualified hardware mechanism.
    /// All state and output changes in this call are discarded on failure.
    /// Call [`Self::validate`] after decoding a complete saved continuation.
    pub fn step(
        &mut self,
        budget: MasterTicks,
        regions: &[ContentRegion],
    ) -> Result<(Progress, Vec<Event>), Fault> {
        if budget.0 == 0 {
            return Err(Fault::InvalidCharge);
        }
        self.hardware.validate_phase()?;
        if let Some(identity) = self.handler_waiting()? {
            return Ok((Progress::HandlerNeedsWork(identity), Vec::new()));
        }
        if let Some(interrupt) = self.interrupt_boundary {
            if self.hardware.interrupts.pending() != Some(interrupt) {
                return Err(Fault::InvalidState);
            }
            return Ok((Progress::Interrupt(interrupt), Vec::new()));
        }
        let Some(pending) = self.work.last() else {
            return Ok((Progress::Idle, Vec::new()));
        };
        if pending.profile.runs.is_empty() {
            return Err(Fault::InvalidProfile);
        }
        let ordinal = usize::try_from(pending.cursor.run).map_err(|_| Fault::InvalidState)?;
        if ordinal > pending.profile.runs.len() {
            return Err(Fault::InvalidState);
        }
        let Some(run) = pending.profile.runs.get(ordinal) else {
            if pending.cursor.stage != Stage::Poll || pending.cursor.occurrence != 0 {
                return Err(Fault::InvalidState);
            }
            return Ok((self.finish_work(), Vec::new()));
        };
        if run.count == 0 || pending.cursor.occurrence >= run.count {
            return Err(Fault::InvalidState);
        }
        validate_exposure(&run.exposure)?;
        if u64::from(pending.cursor.charge_progress(run)?.1) != self.hardware.charge_debt {
            return Err(Fault::InvalidState);
        }
        let mut hardware = self.hardware.clone();
        let mut cursor = pending.cursor.clone();
        let mut events = Vec::new();
        let interrupt =
            hardware.advance_exposure(&mut cursor, run, budget, regions, &mut events)?;
        if interrupt.is_some()
            && (self.work.len() != 1
                || !matches!(pending.kind, WorkKind::Semantic(_) | WorkKind::Wait(_)))
        {
            return Err(Fault::InterruptBoundary);
        }
        self.work.last_mut().ok_or(Fault::InvalidState)?.cursor = cursor;
        self.hardware = hardware;
        self.interrupt_boundary = interrupt;
        Ok((
            interrupt.map_or(Progress::Advanced, Progress::Interrupt),
            events,
        ))
    }

    fn finish_work(&mut self) -> Progress {
        let Some(finished) = self.work.pop() else {
            return Progress::Idle;
        };
        match finished.kind {
            WorkKind::Semantic(identity) => Progress::Completed(identity),
            WorkKind::Wait(identity) => {
                self.work.push(work(WorkKind::Wait(identity), wait_turn()));
                Progress::Advanced
            }
            WorkKind::Native {
                restore,
                handler,
                identity,
                ..
            } => {
                if let Some(portion) = handler {
                    self.work.push(work(
                        WorkKind::Handler {
                            identity,
                            restore,
                            portion: portion.identity,
                            completion: portion.completion,
                        },
                        portion.profile,
                    ));
                } else {
                    self.pending_handler = Some(PendingHandler { identity, restore });
                }
                Progress::HandlerEntered(identity)
            }
            WorkKind::Handler {
                identity,
                restore,
                portion,
                completion,
            } => match completion {
                HandlerCompletion::Continue => {
                    self.pending_handler = Some(PendingHandler { identity, restore });
                    Progress::HandlerPortionCompleted(portion)
                }
                HandlerCompletion::NativeReturn => {
                    self.hardware.interrupts.policy = restore;
                    Progress::HandlerReturned(identity)
                }
            },
        }
    }

    fn handler_waiting(&self) -> Result<Option<WorkId>, Fault> {
        let Some(pending) = &self.pending_handler else {
            return Ok(None);
        };
        if self.hardware.charge_debt != 0
            || self.hardware.interrupts.policy != IrqPolicy::Masked
            || self.work.len() > 1
            || self.interrupt_boundary.is_some()
            || !self.work.iter().all(|parent| {
                matches!(parent.kind, WorkKind::Semantic(_)) && parent.cursor.stage == Stage::Poll
            })
        {
            return Err(Fault::InvalidState);
        }
        Ok(Some(pending.identity))
    }
}

impl Hardware {
    fn advance_charge(
        &mut self,
        charge: &mut Charge,
        budget: MasterTicks,
        effect: &DeviceEffect,
        intrinsic: &mut MasterTicks,
        events: &mut Vec<Event>,
    ) -> Result<Option<Stage>, Fault> {
        let remaining = match charge {
            Charge::Whole { remaining }
            | Charge::BeforeSample { remaining }
            | Charge::AfterSample { remaining } => remaining,
        };
        if *remaining == 0 {
            return Err(Fault::InvalidState);
        }
        let amount =
            u8::try_from(budget.0.min(u64::from(*remaining))).map_err(|_| Fault::InvalidCharge)?;
        self.charge(u64::from(amount), amount == *remaining, events)?;
        *remaining = remaining.wrapping_sub(amount);
        intrinsic.0 = intrinsic
            .0
            .checked_add(u64::from(amount))
            .ok_or(Fault::TimeOverflow)?;
        if *remaining == 0 {
            return Ok(Some(match charge {
                Charge::BeforeSample { .. } => {
                    if let DeviceEffect::Sample(sample) = *effect {
                        self.sample_device(sample, events)?;
                    }
                    Stage::Charge(Charge::AfterSample { remaining: 4 })
                }
                Charge::Whole { .. } | Charge::AfterSample { .. } => Stage::Publish,
            }));
        }

        Ok(None)
    }
}

impl Hardware {
    fn validate_phase(&self) -> Result<(), Fault> {
        let beam = &self.beam;
        let length = if beam.odd && beam.position.line == 240 {
            1360
        } else {
            1364
        };
        if beam.position.line >= 262
            || beam.line_ticks != length
            || beam.position.horizontal >= length
            || beam.position.horizontal & 0x1 != 0
            || beam.subtick > 1
            || beam.residue > 7
            || !(12..=19).contains(&beam.thresholds.initialize_at)
            || !self.math.valid_progress()
            || self.interrupts.horizontal_target > 511
            || self.interrupts.vertical_target > 511
            || self.controller.serial > 16
        {
            return Err(Fault::InvalidState);
        }
        if self.charge_debt > 11 {
            return Err(Fault::InvalidState);
        }
        if beam.history.last() != Some(&beam.position) {
            return Err(Fault::InvalidState);
        }
        Ok(())
    }

    fn advance_exposure(
        &mut self,
        cursor: &mut Cursor,
        run: &Run,
        budget: MasterTicks,
        regions: &[ContentRegion],
        events: &mut Vec<Event>,
    ) -> Result<Option<Interrupt>, Fault> {
        let exposure = &run.exposure;
        match &mut cursor.stage {
            Stage::Poll => {
                if exposure.poll_before {
                    self.interrupts.sample();
                }
                cursor.stage = Stage::Before;
            }
            Stage::Before => {
                if exposure.kind == ExposureKind::Write {
                    self.math.edge();
                }
                cursor.stage = Stage::Arbitrate;
            }
            Stage::Arbitrate => {
                let read_like = exposure.kind == ExposureKind::Read
                    || (exposure.kind == ExposureKind::PendingInternal
                        && self.interrupts.pending().is_some());
                self.arbitrate(exposure.cycle, regions, events)?;
                cursor.stage = Stage::Charge(if read_like {
                    Charge::BeforeSample {
                        remaining: exposure.cycle.ticks().wrapping_sub(4),
                    }
                } else {
                    Charge::Whole {
                        remaining: exposure.cycle.ticks(),
                    }
                });
            }
            Stage::Charge(charge) => {
                let next = self.advance_charge(
                    charge,
                    budget,
                    &exposure.effect,
                    &mut cursor.intrinsic,
                    events,
                )?;
                if let Some(stage) = next {
                    cursor.stage = stage;
                }
            }
            Stage::Publish => {
                if let DeviceEffect::Publish(publication) = &exposure.effect {
                    self.publish(publication, events)?;
                }
                cursor.stage = Stage::After;
            }
            Stage::After => {
                if exposure.kind != ExposureKind::Write {
                    self.math.edge();
                }
                cursor.stage = Stage::Finish;
            }
            Stage::Finish => {
                cursor.completed = cursor.completed.checked_add(1).ok_or(Fault::TimeOverflow)?;
                cursor.occurrence = cursor
                    .occurrence
                    .checked_add(1)
                    .ok_or(Fault::InvalidState)?;
                if cursor.occurrence == run.count {
                    cursor.run = cursor.run.checked_add(1).ok_or(Fault::InvalidState)?;
                    cursor.occurrence = 0;
                }
                cursor.stage = Stage::Poll;
                if exposure.poll_before
                    && let Some(interrupt) = self.interrupts.pending()
                {
                    return Ok(Some(interrupt));
                }
            }
        }
        Ok(None)
    }
}

impl Work {
    fn validate_progress(&self) -> Result<(), Fault> {
        validate_profile(&self.profile)?;
        if let WorkKind::Native {
            handler: Some(portion),
            ..
        } = &self.kind
        {
            validate_profile(&portion.profile)?;
        }
        let ordinal = usize::try_from(self.cursor.run).map_err(|_| Fault::InvalidState)?;
        if ordinal > self.profile.runs.len() {
            return Err(Fault::InvalidState);
        }
        if let Some(run) = self.profile.runs.get(ordinal) {
            if self.cursor.occurrence >= run.count {
                return Err(Fault::InvalidState);
            }
        } else if self.cursor.stage != Stage::Poll || self.cursor.occurrence != 0 {
            return Err(Fault::InvalidState);
        }
        let mut completed = 0_u64;
        let mut intrinsic = 0_u64;
        for run in self.profile.runs.iter().take(ordinal) {
            completed = completed
                .checked_add(u64::from(run.count))
                .ok_or(Fault::InvalidState)?;
            intrinsic = intrinsic
                .checked_add(
                    u64::from(run.count)
                        .checked_mul(u64::from(run.exposure.cycle.ticks()))
                        .ok_or(Fault::InvalidState)?,
                )
                .ok_or(Fault::InvalidState)?;
        }
        completed = completed
            .checked_add(u64::from(self.cursor.occurrence))
            .ok_or(Fault::InvalidState)?;
        if let Some(run) = self.profile.runs.get(ordinal) {
            let (progress, _) = self.cursor.charge_progress(run)?;
            intrinsic = intrinsic
                .checked_add(
                    u64::from(self.cursor.occurrence)
                        .checked_mul(u64::from(run.exposure.cycle.ticks()))
                        .ok_or(Fault::InvalidState)?,
                )
                .and_then(|ticks| ticks.checked_add(u64::from(progress)))
                .ok_or(Fault::InvalidState)?;
        }
        if completed != self.cursor.completed || intrinsic != self.cursor.intrinsic.0 {
            return Err(Fault::InvalidState);
        }

        Ok(())
    }
}
