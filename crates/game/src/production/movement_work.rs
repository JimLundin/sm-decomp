//! One current movement authority, including an interrupted ordinary update.

use crate::production::{MovementContent, ProductionFault};
use crate::state::owned::MovementPin;
use canonical::Canonical;
use canonical::model::{Model, Pinned};
use samus_movement::suspended::{MovementDecision, SuspendedMovement};

/// Complete interrupted motion, chosen answers and already performed query outputs.
pub struct WorkPin;
impl Pinned for WorkPin {
    type State = Box<SuspendedMovement>;
    const NAME: &'static str = "production-movement-work";
    const SHAPE: u64 = 0xE057_BE29_1B46_4F8C;
}

/// Actual completed decisions, retained by the caller rather than reconstructed later.
pub struct ReceiptsPin;
impl Pinned for ReceiptsPin {
    type State = Vec<MovementDecision>;
    const NAME: &'static str = "production-movement-completed-decisions";
    const SHAPE: u64 = 0x7913_7573_1460_87B8;
}

/// Sole current model and immutable actual decisions already completed in this update.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Work {
    /// Complete current quantities and pending request or answer.
    pub model: Model<WorkPin>,
    /// Actual offered decisions consumed successfully, in their performed order.
    pub completed: Model<ReceiptsPin>,
}

/// One writable movement authority; active work remains authoritative after completion.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub enum Movement {
    /// Initial complete quantities, before an ordinary update is begun.
    Initial(Model<MovementPin>),
    /// Actual current quantities, pending requests/answers and completed outputs.
    Work(Work),
}

/// Invalid movement scheduling rejects before changing the owned continuation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkFault {
    /// A new update was requested before the active update completed.
    Busy,
    /// No ordinary update has begun.
    NotStarted,
    /// No offered request or answer is available to complete.
    NoOffer,
    /// Unknown ordinary pose or unavailable admitted content.
    Production(ProductionFault),
}

impl Movement {
    /// Hold complete initial quantities without performing any movement decision.
    #[must_use]
    pub const fn new(state: samus_movement::Samus) -> Self {
        Self::Initial(Model(state))
    }

    /// Current complete quantities, including effects already completed during suspension.
    #[must_use]
    pub const fn state(&self) -> &samus_movement::Samus {
        match self {
            Self::Initial(state) => &state.0,
            Self::Work(work) => &work.model.0.state,
        }
    }

    /// Final current quantities and actual publications only after ordinary work completes.
    #[must_use]
    pub fn finished(&self) -> Option<(&samus_movement::Samus, &samus_movement::Outcome)> {
        match self {
            Self::Work(work) if work.model.0.is_complete() => {
                Some((&work.model.0.state, &work.model.0.outputs))
            }
            Self::Initial(_) | Self::Work(_) => None,
        }
    }

    /// Read publications already produced by actual completed movement decisions.
    /// Their extent can precede final pose initialization; callers must qualify the
    /// specific opportunity before consuming them as an animation dependency.
    #[must_use]
    pub const fn outputs(&self) -> Option<&samus_movement::Outcome> {
        match self {
            Self::Initial(_) => None,
            Self::Work(work) => Some(&work.model.0.outputs),
        }
    }

    /// The consumed decisions of the current ordinary opportunity.
    #[must_use]
    pub fn decisions(&self) -> &[MovementDecision] {
        match self {
            Self::Initial(_) => &[],
            Self::Work(work) => &work.completed.0,
        }
    }

    pub(super) fn selected_running_figures(
        &self,
        playing: samus_appearance::Figure,
        content: &crate::production::AppearanceContent,
    ) -> Option<(samus_appearance::Figure, samus_appearance::Figure)> {
        use samus_movement::suspended::LocalRule;
        let MovementDecision::Local {
            rule: LocalRule::PoseInitialization { accepted },
            before,
            answer,
            ..
        } = self.decisions().last()?
        else {
            return None;
        };
        if answer != self.state() || answer.pose != accepted.pose {
            return None;
        }
        let from = samus_appearance::Figure {
            pose: before.pose,
            sprite: playing.sprite,
        };
        let selected = samus_appearance::Figure {
            pose: accepted.pose,
            sprite: playing.sprite,
        };
        if from.pose.action != types::Action::Running
            || selected.pose.action != types::Action::Running
            || content.get(from).is_none()
            || content.get(selected).is_none()
        {
            return None;
        }
        Some((from, selected))
    }

    /// Begin one ordinary update from the actual current quantities.
    ///
    /// # Errors
    /// Unfinished update or unknown ordinary pose; the owner remains unchanged.
    pub fn begin(&mut self) -> Result<(), WorkFault> {
        if matches!(self, Self::Work(work) if !work.model.0.is_complete()) {
            return Err(WorkFault::Busy);
        }
        let work = SuspendedMovement::new(*self.state())
            .map_err(|fault| WorkFault::Production(ProductionFault::Movement(fault)))?;
        *self = Self::Work(Work {
            model: Model(Box::new(work)),
            completed: Model(Vec::new()),
        });
        Ok(())
    }

    /// Offer a request before querying, or an actual local/returned answer before its effect.
    /// Later unmade decisions read current frame/world inputs at this call.
    ///
    /// # Errors
    /// Missing active work, unknown pose or unavailable imported body content.
    pub fn offer(
        &mut self,
        content: &MovementContent,
        rules: &samus_movement::Rules,
        frame: &samus_movement::Frame<'_>,
        world: &dyn types::World,
    ) -> Result<Option<MovementDecision>, WorkFault> {
        let Self::Work(work) = self else {
            return Err(WorkFault::NotStarted);
        };
        let mut candidate = work.model.clone();
        let decision = crate::production::movement::with_frame(
            candidate.0.state.pose,
            content,
            frame,
            |admitted| {
                candidate
                    .0
                    .offer(admitted, world, rules)
                    .map(Option::<&MovementDecision>::cloned)
                    .map_err(ProductionFault::Movement)
            },
        )
        .map_err(WorkFault::Production)?;
        work.model = candidate;
        Ok(decision)
    }

    /// Complete one actual request or retained answer with current admitted inputs.
    /// Requests query once; answers apply once without repeating the query.
    ///
    /// # Errors
    /// Missing offer or invalid current content; the complete candidate is discarded.
    pub fn complete(
        &mut self,
        content: &MovementContent,
        rules: &samus_movement::Rules,
        frame: &samus_movement::Frame<'_>,
        world: &dyn types::World,
    ) -> Result<(), WorkFault> {
        let Self::Work(work) = self else {
            return Err(WorkFault::NotStarted);
        };
        let decision = work.model.0.offered.clone().ok_or(WorkFault::NoOffer)?;
        let mut candidate = work.model.clone();
        crate::production::movement::with_frame(
            candidate.0.state.pose,
            content,
            frame,
            |admitted| {
                candidate
                    .0
                    .complete(admitted, world, rules)
                    .map_err(ProductionFault::Movement)
            },
        )
        .map_err(WorkFault::Production)?;
        work.model = candidate;
        work.completed.0.push(decision);
        Ok(())
    }
}
