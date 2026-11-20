//! Actual ordinary clock opportunity after the room and same-frame Health return.

use super::{Content, Dependency, State, room_tail};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use data::timing::{Profile, Template, TemplateEffect};
use game_time::{Clock, PlayFrames, PlayTime, QualifiedOpportunity};

pub(super) const WORK: timing::WorkId = timing::WorkId(1426);

pub(super) struct TimePin;

impl Pinned for TimePin {
    type State = PlayTime;
    const NAME: &'static str = "production-play-time";
    const SHAPE: u64 = 0x5129_c4b9_2320_323e;
}

/// The clock's complete published state and any pending physical result.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stage(Progress);

#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Progress {
    #[default]
    Unavailable,
    Ready(Model<TimePin>),
    Charging {
        current: Model<TimePin>,
        next: Model<TimePin>,
    },
    Complete(Model<TimePin>),
}

impl Stage {
    /// Carry the complete clock from the original frame admission.
    #[must_use]
    pub const fn from_origin(time: PlayTime) -> Self {
        Self(Progress::Ready(Model(time)))
    }

    /// Read the currently published clock, including during the hardware work.
    #[must_use]
    pub const fn current(&self) -> Option<PlayTime> {
        match self.0 {
            Progress::Unavailable => None,
            Progress::Ready(current)
            | Progress::Complete(current)
            | Progress::Charging { current, .. } => Some(current.0),
        }
    }

    /// The selected physical clock opportunity is awaiting return.
    #[must_use]
    pub const fn is_charging(&self) -> bool {
        matches!(self.0, Progress::Charging { .. })
    }

    /// The selected opportunity has returned and published its complete state.
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
            matches!(state.room_tail, room_tail::Stage::Completed(_))
                && active.next().is_some()
                && active.next().is_none()
        } else {
            active.next().is_none()
                && (!self.is_complete()
                    || matches!(state.room_tail, room_tail::Stage::Completed(_)))
        }
    }
}

/// The reached ordinary no-carry work has no device effects of its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract(Profile);

impl Contract {
    /// Bind the complete physical envelope for one ordinary advance.
    ///
    /// # Errors
    /// A device effect or invalid template lies outside the qualified clock profile.
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

pub(super) const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::PlayTimePreparation))
}

impl State {
    fn qualify_play_time(&self, current: PlayTime) -> Result<(), ProductionFault> {
        let Some(health) = &self.health else {
            return Err(missing());
        };
        let energy = health.state().energy.whole.get();
        if !matches!(self.room_tail, room_tail::Stage::Completed(_))
            || !health.maintained()
            || self.maintenance.energy() != Some(energy)
            || energy == 0
            || current.frames == PlayFrames::MAX
        {
            return Err(missing());
        }
        Ok(())
    }

    pub(super) fn advance_play_time(&mut self, content: &Content) -> Result<(), ProductionFault> {
        let Contract(profile) = content.play_time.as_ref().ok_or_else(missing)?;
        let current = self.play_time.current().ok_or_else(missing)?;
        self.qualify_play_time(current)?;
        match self.play_time.0 {
            Progress::Ready(current) if self.play_time.holds(self) => {
                let next = Clock::restore(current.0).advance(QualifiedOpportunity);
                self.timing
                    .begin(WORK, profile.clone())
                    .map_err(ProductionFault::FrameHardware)?;
                self.play_time = Stage(Progress::Charging {
                    current,
                    next: Model(next),
                });
                Ok(())
            }
            Progress::Charging { .. }
                if self.play_time.holds(self)
                    && self.timing.work.iter().any(|work| {
                        work.kind == timing::WorkKind::Semantic(WORK) && &work.profile == profile
                    }) =>
            {
                Ok(())
            }
            Progress::Unavailable
            | Progress::Ready(_)
            | Progress::Charging { .. }
            | Progress::Complete(_) => Err(missing()),
        }
    }

    pub(super) fn complete_play_time(&mut self, content: &Content) -> Result<(), ProductionFault> {
        let Stage(Progress::Charging { current, next }) = self.play_time else {
            return Err(missing());
        };
        self.qualify_play_time(current.0)?;
        if content.play_time.is_none() {
            return Err(missing());
        }
        self.play_time = Stage(Progress::Complete(next));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_clock_shape_remains_pinned() {
        assert_eq!(canonical::model::pin_holds::<TimePin>(), Ok(()));
    }
}
