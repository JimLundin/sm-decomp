//! Current Health, pause and Ceres return share one physical maintenance opportunity.

use super::{Caller, Content, Dependency, State, countdown_caller, health};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::{Canonical, model::Model};
use data::timing::{Profile, Template, TemplateEffect};

pub(super) const WORK: timing::WorkId = timing::WorkId(1310);

/// Current Maintenance through the same actual scene offer used by Countdown.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
struct Receipt {
    health_before: Model<health::HealthPin>,
    health_after: Model<health::HealthPin>,
    scene: super::countdown_scene::Offer,
}

/// Maintained energy becomes available only at actual return.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct Stage(Progress);

#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
enum Progress {
    #[default]
    Pending,
    Charging(Receipt),
    Complete(Receipt),
}

/// Whole source-qualified quiet Health, Ceres pause and timer-tail operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract {
    profile: Profile,
}

impl Contract {
    /// Bind the quiet health/pause envelope, which emits no physical device effects.
    /// # Errors
    /// Device effects or invalid physical work require a different qualified arm.
    pub fn new(template: &Template) -> Result<Self, ProductionFault> {
        if template
            .runs
            .iter()
            .any(|run| run.effect != TemplateEffect::None)
        {
            return Err(missing());
        }
        Ok(Self {
            profile: template.bind(&[]).map_err(|_| missing())?,
        })
    }
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::HealthMaintenance))
}

impl Stage {
    pub(super) fn holds(&self, state: &State) -> bool {
        let mut active = state
            .timing
            .work
            .iter()
            .filter(|work| work.kind == timing::WorkKind::Semantic(WORK));
        let maintained = state
            .health
            .as_ref()
            .is_some_and(health::Health::maintained);
        match &self.0 {
            Progress::Pending => !maintained && active.next().is_none(),
            Progress::Charging(receipt) => {
                !maintained
                    && active.next().is_some()
                    && active.next().is_none()
                    && state
                        .health
                        .as_ref()
                        .is_some_and(|health| health.state() == &receipt.health_before.0)
            }
            Progress::Complete(receipt) => {
                maintained
                    && active.next().is_none()
                    && state
                        .health
                        .as_ref()
                        .is_some_and(|health| health.state() == &receipt.health_after.0)
            }
        }
    }

    /// Whole energy produced after actual Maintenance, for Drawing.
    #[must_use]
    pub const fn energy(&self) -> Option<u16> {
        let Progress::Complete(receipt) = &self.0 else {
            return None;
        };
        Some(receipt.health_after.0.energy.whole.get())
    }
}

impl State {
    fn maintenance_receipt(&self, content: &Content) -> Result<Receipt, ProductionFault> {
        let countdown_caller::Stage::Complete { offer, .. } = &self.countdown else {
            return Err(missing());
        };
        if !self.countdown.holds(self) {
            return Err(missing());
        }
        self.revalidate_countdown_scene_at_ceres(content, *offer)
            .map_err(|_| missing())?;
        let Caller::PlayerContinuation {
            completed,
            remaining,
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        let equipment = completed
            .0
            .environment
            .as_ref()
            .ok_or_else(missing)?
            .dependency
            .equipment;
        let actor = remaining.0.actors.as_ref().ok_or_else(missing)?;
        let health = self.health.as_ref().ok_or_else(missing)?;
        self.energy_for_animation(content, actor)
            .map_err(|_| missing())?;
        if health.maintained() {
            return Err(missing());
        }
        if equipment[types::Item::VariaSuit] || equipment[types::Item::GravitySuit] {
            return Err(missing());
        }
        let before = *health.state();
        let mut after = before;
        let warning = after.health_opportunity(&samus_health::Rules::REFERENCE);
        if warning != samus_health::WarningSound::None {
            return Err(missing());
        }
        Ok(Receipt {
            health_before: Model(before),
            health_after: Model(after),
            scene: *offer,
        })
    }

    pub(super) fn advance_maintenance(&mut self, content: &Content) -> Result<(), ProductionFault> {
        let contract = content.maintenance.as_ref().ok_or_else(missing)?;
        match &self.maintenance.0 {
            Progress::Pending => {
                let receipt = self.maintenance_receipt(content)?;
                self.timing
                    .begin(WORK, contract.profile.clone())
                    .map_err(ProductionFault::FrameHardware)?;
                self.maintenance.0 = Progress::Charging(receipt);
            }
            Progress::Charging(receipt) => {
                if !self.maintenance_current_holds(receipt, content)
                    || !self.timing.work.iter().any(|work| {
                        work.kind == timing::WorkKind::Semantic(WORK)
                            && work.profile == contract.profile
                    })
                {
                    return Err(missing());
                }
            }
            Progress::Complete(_) => return Err(missing()),
        }
        Ok(())
    }

    fn maintenance_current_holds(&self, receipt: &Receipt, content: &Content) -> bool {
        self.maintenance_receipt(content).as_ref() == Ok(receipt)
    }

    pub(super) fn complete_maintenance(
        &mut self,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let Progress::Charging(receipt) = &self.maintenance.0 else {
            return Err(missing());
        };
        if !self.maintenance_current_holds(receipt, content) {
            return Err(missing());
        }
        self.health
            .as_mut()
            .ok_or_else(missing)?
            .complete_maintenance(&receipt.health_before.0, receipt.health_after.0)
            .map_err(|_| missing())?;
        self.maintenance.0 = Progress::Complete(receipt.clone());
        Ok(())
    }
}
