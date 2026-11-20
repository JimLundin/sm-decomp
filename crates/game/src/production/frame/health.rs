//! One complete current health authority and its finite actor/interrupt preservation.

use super::{
    borrowed_player_preparation as actors, interrupts::DrawingRequest,
    movement_support_contract::ActorCompletionPin,
};
use canonical::{
    Canonical,
    model::{Model, Pinned},
};

/// Complete health quantities; delivery and neighbouring qualification stay outside this pin.
pub struct HealthPin;
impl Pinned for HealthPin {
    type State = samus_health::State;
    const NAME: &'static str = "production-current-samus-health";
    const SHAPE: u64 = 0x952D_4037_4B4E_F0FA;
}

/// Gameplay clock supplied by the independently qualified neighbouring owner.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Time {
    /// This frame admits periodic loss.
    Running,
    /// Requires an unimplemented stopped-time producer.
    Stopped,
}

/// Independently qualified suit flags; the health model implements neither reduction.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Suits {
    /// Actual Varia equipment disposition.
    pub varia: bool,
    /// Actual Gravity equipment disposition.
    pub gravity: bool,
}

/// Actual current neighbouring activity; only the inactive arm is qualified here.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activity {
    /// No activity in this finite source envelope.
    Inactive,
    /// Requires a separately qualified producer.
    Active,
}

/// Current region of the pause opportunity, independent from scene scheduling.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    /// The source-qualified return before reading Start.
    Ceres,
    /// Another pause path is unimplemented.
    Other,
}

/// Finite original neighbour context, never a borrowed health result.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Context {
    /// Actual current gameplay clock.
    pub time: Time,
    /// Actual current damage-reducing suits.
    pub suits: Suits,
    /// Current power-bomb activity; another pause arm is unsupported.
    pub power_bomb: Activity,
    /// Current elevator activity; another pause arm is unsupported.
    pub elevator: Activity,
    /// Ceres region admits the pause return before Start is read.
    pub region: Region,
    /// Actual inactive protection shared with actor contact and Drawing.
    pub protection: Activity,
}

/// Independently admitted original quantities and context, retained as read-only provenance.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Admission {
    /// Complete initial health, including both fractions and pending loss.
    pub initial: Model<HealthPin>,
    /// Explicit finite borrowed neighbouring inputs.
    pub context: Context,
}

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
enum Freshness {
    Preserved,
    Invalidated,
}
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
enum InterruptHistory {
    Unseen,
    Begun,
    Returned,
}

/// A missing lifetime join, unsupported context or writer invalidates this frame scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// A same-value or changing writer/activity invalidated the source qualification.
    Activity,
    /// An actual producer or original input no longer matches this authority.
    Owner,
    /// Stopped time, suits, protection or pause conditions require another producer.
    Context,
}

/// Complete live health and actual no-writer history; no maintenance runs on construction.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Health {
    current: Model<HealthPin>,
    origin: Admission,
    context: Context,
    actor: Option<Model<ActorCompletionPin>>,
    interrupts: InterruptHistory,
    freshness: Freshness,
    maintained: bool,
}

impl Context {
    /// Admit only the independently reviewed finite neighbouring arm.
    /// # Errors
    /// Wider clocks, equipment, protection and pause arms are not implemented.
    pub fn qualify(self) -> Result<(), Fault> {
        if self.time != Time::Running
            || self.suits.varia
            || self.suits.gravity
            || self.power_bomb != Activity::Inactive
            || self.elevator != Activity::Inactive
            || self.region != Region::Ceres
            || self.protection != Activity::Inactive
        {
            return Err(Fault::Context);
        }
        Ok(())
    }
}

impl Health {
    /// Construct all five quantities directly from the original owner without applying loss.
    /// # Errors
    /// Unsupported neighbouring context cannot manufacture a qualified origin.
    pub fn from_origin(admission: Admission) -> Result<Self, Fault> {
        admission.context.qualify()?;
        Ok(Self {
            current: admission.initial,
            context: admission.context,
            origin: admission,
            actor: None,
            interrupts: InterruptHistory::Unseen,
            freshness: Freshness::Preserved,
            maintained: false,
        })
    }

    /// Sole current complete health, readable without selecting periodic maintenance.
    #[must_use]
    pub const fn state(&self) -> &samus_health::State {
        &self.current.0
    }

    /// Any external health, damage, equipment, protection or clock write invalidates this scope.
    /// Same-value writes are activity, not proof that the old input remains fresh.
    pub const fn invalidate(&mut self) {
        self.freshness = Freshness::Invalidated;
    }

    /// Explicit external replacement cannot preserve the finite no-writer qualification.
    pub const fn write(&mut self, state: samus_health::State) {
        self.current = Model(state);
        self.invalidate();
    }

    /// Explicit clock, equipment or protection writes invalidate even identical inputs.
    pub const fn write_context(&mut self, context: Context) {
        self.context = context;
        self.invalidate();
    }

    /// Observe only the actual successfully admitted HUD interrupt pair.
    pub const fn hud_irq(&mut self, request: DrawingRequest) {
        match (self.interrupts, request) {
            (InterruptHistory::Unseen, DrawingRequest::Begin) => {
                self.interrupts = InterruptHistory::Begun;
            }
            (InterruptHistory::Begun, DrawingRequest::End) => {
                self.interrupts = InterruptHistory::Returned;
            }
            _ => self.invalidate(),
        }
    }

    /// Join the actual returned bounded actor operation, including the shared protection alias.
    /// # Errors
    /// Missing no-hurt qualification, changed clocks/protection or duplicate completion.
    pub fn actor_completed(
        &mut self,
        completion: &actors::ActorUpdateCompletion,
    ) -> Result<(), Fault> {
        self.validate_origin(&self.origin)?;
        let dependency = &completion.dependency;
        if self.actor.is_some()
            || completion.solid_boxes().is_err()
            || dependency.contact.samus_invincibility != 0
            || dependency.contact.damage_mode != actors::ContactDamageDisposition::Ordinary
            || dependency.clocks.simulation != actors::ClockDisposition::Running
        {
            return Err(Fault::Owner);
        }
        self.actor = Some(Model(completion.clone()));
        Ok(())
    }

    /// Validate complete original input and retained source lifetime before a dependent effect.
    /// # Errors
    /// Source changes, writes and unsupported neighbouring inputs reject atomically.
    pub fn validate_origin(&self, admission: &Admission) -> Result<(), Fault> {
        if self.freshness != Freshness::Preserved {
            return Err(Fault::Activity);
        }
        admission.context.qualify()?;
        if &self.origin != admission
            || self.context != admission.context
            || (!self.maintained && self.current != self.origin.initial)
        {
            return Err(Fault::Owner);
        }
        Ok(())
    }

    /// Complete owned source qualification for canonical checkpoint admission.
    /// # Errors
    /// Invalidated or changed current inputs cannot regain freshness by restoration.
    pub fn validate_current(&self) -> Result<(), Fault> {
        self.validate_origin(&self.origin)
    }

    /// Read fresh current whole energy for the actual pre-Maintenance Animation opportunity.
    /// # Errors
    /// The same actual actor and both qualified IRQ admissions must have completed.
    pub fn at_animation(
        &self,
        admission: &Admission,
        actors: &actors::ActorUpdateCompletion,
    ) -> Result<u16, Fault> {
        self.validate_origin(admission)?;
        if self.maintained
            || self.actor.as_ref().map(|receipt| &receipt.0) != Some(actors)
            || self.interrupts != InterruptHistory::Returned
        {
            return Err(Fault::Owner);
        }
        Ok(self.current.0.energy.whole.get())
    }

    pub(super) fn complete_maintenance(
        &mut self,
        before: &samus_health::State,
        result: samus_health::State,
    ) -> Result<(), Fault> {
        self.validate_current()?;
        if self.maintained
            || self.state() != before
            || self.interrupts != InterruptHistory::Returned
        {
            return Err(Fault::Owner);
        }
        self.current = Model(result);
        self.maintained = true;
        Ok(())
    }

    pub(super) const fn maintained(&self) -> bool {
        self.maintained
    }

    /// Actual actor completion retained independently from the original source declaration.
    #[must_use]
    pub fn actor(&self) -> Option<&actors::ActorUpdateCompletion> {
        self.actor.as_ref().map(|receipt| &receipt.0)
    }
}

impl super::State {
    pub(super) fn validate_health_preservation(
        &self,
        content: &super::Content,
        gameplay_publications: &[crate::production::timing::Event],
    ) -> Result<(), crate::production::ProductionFault> {
        use crate::production::{ProductionFault, Unsupported, timing};
        let missing = || {
            ProductionFault::Unsupported(Unsupported::Frame(super::Dependency::HealthCurrentInputs))
        };
        let (Some(owner), Some(admission)) = (&self.health, &content.health) else {
            return if self.health.is_none() && content.health.is_none() {
                Ok(())
            } else {
                Err(missing())
            };
        };
        owner.validate_origin(admission).map_err(|_| missing())?;
        if self.scene_countdown.is_none() || content.scene_countdown.is_none() {
            return Err(missing());
        }
        if self
            .drawing_origins
            .as_ref()
            .is_some_and(|drawing| drawing.invincibility.0 != 0)
        {
            return Err(missing());
        }
        if let super::Caller::PlayerContinuation { remaining, .. } = &self.caller
            && owner.actor() != remaining.0.actors.as_ref()
        {
            return Err(missing());
        }
        for event in gameplay_publications {
            match &event.kind {
                timing::EventKind::Interrupt(data::timing::Interrupt::Nmi) => return Err(missing()),
                timing::EventKind::Publication(write)
                    if write.port == data::timing::WRAM_DATA_PORT =>
                {
                    return Err(missing());
                }
                timing::EventKind::Payload(payload)
                    if payload.target.publication_port() == data::timing::WRAM_DATA_PORT =>
                {
                    return Err(missing());
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_health_shape_remains_pinned() {
        assert_eq!(canonical::model::pin_holds::<HealthPin>(), Ok(()));
    }
}
