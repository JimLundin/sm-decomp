//! Drawing reads the current room quake; its phase advances after the play-clock return.

use super::{Content, Dependency, State};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use data::timing::{Profile, Template, TemplateEffect};
use world_quake::{Opportunity, Outcome};

pub(super) const WORK: timing::WorkId = timing::WorkId(1427);

pub(super) struct QuakePin;

impl Pinned for QuakePin {
    type State = world_quake::State;
    const NAME: &'static str = "production-room-quake";
    const SHAPE: u64 = 0xfabe_ae87_908e_7c97;
}

/// The room's complete published quake state and pending physical result.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stage(Progress);

#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Progress {
    #[default]
    Unavailable,
    Ready(Model<QuakePin>),
    Charging {
        current: Model<QuakePin>,
        next: Model<QuakePin>,
    },
    Complete(Model<QuakePin>),
}

impl Stage {
    /// Carry both quake quantities from the original frame admission.
    #[must_use]
    pub const fn from_origin(quake: world_quake::State) -> Self {
        Self(Progress::Ready(Model(quake)))
    }

    /// Read the complete current quake, including during its hardware work.
    #[must_use]
    pub const fn current(&self) -> Option<world_quake::State> {
        match self.0 {
            Progress::Unavailable => None,
            Progress::Ready(current)
            | Progress::Complete(current)
            | Progress::Charging { current, .. } => Some(current.0),
        }
    }

    /// A physical quiet opportunity is holding its unpublished result.
    #[must_use]
    pub const fn is_charging(&self) -> bool {
        matches!(self.0, Progress::Charging { .. })
    }

    /// The physical phase-work opportunity has returned and published its result.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        matches!(self.0, Progress::Complete(_))
    }

    pub(super) fn holds(&self, state: &State) -> bool {
        let mut active = state
            .timing
            .work
            .iter()
            .filter(|work| work.kind == timing::WorkKind::Semantic(WORK));
        if self.is_charging() {
            state.play_time.is_complete() && active.next().is_some() && active.next().is_none()
        } else {
            active.next().is_none() && (!self.is_complete() || state.play_time.is_complete())
        }
    }
}

/// Physical envelope of the ordinary quiet phase opportunity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract(Profile);

impl Contract {
    /// Bind the reached quiet quake profile.
    ///
    /// # Errors
    /// Device effects or invalid physical work require a separate envelope.
    pub fn new(template: &Template) -> Result<Self, ProductionFault> {
        if template
            .runs
            .iter()
            .any(|run| run.effect != TemplateEffect::None)
        {
            return Err(missing());
        }
        template.bind(&[]).map(Self).map_err(|_| missing())
    }
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::RoomShakePreparation))
}

impl State {
    pub(super) fn advance_room_quake(&mut self, content: &Content) -> Result<(), ProductionFault> {
        let Contract(profile) = content.room_quake.as_ref().ok_or_else(missing)?;
        if !self.play_time.is_complete() || !self.room_quake.holds(self) {
            return Err(missing());
        }
        match self.room_quake.0 {
            Progress::Ready(current) => {
                let mut next = current.0;
                let Outcome::Quiet(effects) = world_quake::step(&mut next, Opportunity::Ordinary)
                else {
                    return Err(missing());
                };
                if effects.actor_shake_requested {
                    return Err(missing());
                }
                self.timing
                    .begin(WORK, profile.clone())
                    .map_err(ProductionFault::FrameHardware)?;
                self.room_quake = Stage(Progress::Charging {
                    current,
                    next: Model(next),
                });
                Ok(())
            }
            Progress::Charging { .. }
                if self.timing.work.iter().any(|work| {
                    work.kind == timing::WorkKind::Semantic(WORK) && &work.profile == profile
                }) =>
            {
                Ok(())
            }
            Progress::Unavailable | Progress::Charging { .. } | Progress::Complete(_) => {
                Err(missing())
            }
        }
    }

    pub(super) const fn complete_room_quake(
        &mut self,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let Stage(Progress::Charging { next, .. }) = self.room_quake else {
            return Err(missing());
        };
        if !self.play_time.is_complete() || content.room_quake.is_none() {
            return Err(missing());
        }
        self.room_quake = Stage(Progress::Complete(next));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_quake_shape_remains_pinned() {
        assert_eq!(canonical::model::pin_holds::<QuakePin>(), Ok(()));
    }
}
