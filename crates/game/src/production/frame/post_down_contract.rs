//! Current transport predicates and distinct whole post-Down envelopes.

pub use crate::production::sender::{Demo, OfferCurrent, PowerBomb, Sounds};

/// One complete source-qualified envelope for each distinct consumer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Physical {
    /// Complete `SoundOffer` owner, including its enqueue completion.
    pub sound_offer: data::timing::Template,
    /// Independently accepted current `SoundOffer` gates on the recurring grounded arm.
    pub recurring_footstep: Option<OfferCurrent>,
    /// Complete Poststep owner, ending before `HUDMap` is called.
    pub poststep: data::timing::Template,
}

/// Wrong selected arm or incomplete physical envelope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fault;

impl Physical {
    /// Bind the complete actual unsuppressed transport envelope.
    /// # Errors
    /// Different current owners, unresolved effects or incomplete envelope.
    pub fn sound_offer(&self, current: OfferCurrent) -> Result<data::timing::Profile, Fault> {
        current.permission().ok_or(Fault)?;
        bind(&self.sound_offer, 818)
    }

    /// Bind the actual Stage0 echo-return arm, separately from `HUDMap`.
    /// # Errors
    /// Intermediate/boosting stage, unresolved effects or incomplete envelope.
    pub fn poststep(&self, stage: types::BoostStage) -> Result<data::timing::Profile, Fault> {
        if stage != types::BoostStage::Stage0 {
            return Err(Fault);
        }
        bind(&self.poststep, 274)
    }
}

fn bind(
    template: &data::timing::Template,
    expected_ticks: u64,
) -> Result<data::timing::Profile, Fault> {
    let mut ticks = 0_u64;
    for run in &template.runs {
        if !matches!(run.effect, data::timing::TemplateEffect::None) {
            return Err(Fault);
        }
        ticks = ticks
            .checked_add(u64::from(run.count).saturating_mul(u64::from(run.cycle.ticks())))
            .ok_or(Fault)?;
    }
    if ticks != expected_ticks {
        return Err(Fault);
    }
    template.bind(&[]).map_err(|_| Fault)
}
