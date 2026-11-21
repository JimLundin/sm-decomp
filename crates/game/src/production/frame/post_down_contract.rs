//! Current transport predicates and distinct whole post-Down envelopes.

use canonical::Canonical;

/// Actual current sound-disable owner.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sounds {
    /// Requests may reach transport admission.
    Enabled,
    /// The source-qualified enqueue arm is unavailable.
    Disabled,
}

/// Actual current scene's demo activity.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Demo {
    /// The scene is outside the demo range.
    Inactive,
    /// The scene requires another source arm.
    Active,
}

/// Actual current power-bomb activity, distinct from cinematic sound suppression.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerBomb {
    /// No power bomb is active.
    Inactive,
    /// The source-qualified enqueue arm is unavailable.
    Active,
}

/// Actual owners read at `SoundOffer`; queue quantities remain in their sole owner.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OfferCurrent {
    /// Actual sound-enabled status.
    pub sounds: Sounds,
    /// Actual scene demo activity.
    pub demo: Demo,
    /// Actual power-bomb activity.
    pub power_bomb: PowerBomb,
}

impl OfferCurrent {
    /// Current suppression for the ordinary sender offer, in native gate order.
    #[must_use]
    pub const fn transport_permission(self) -> audio_transport::OfferPermission {
        use audio_transport::OfferPermission;
        match (self.sounds, self.demo, self.power_bomb) {
            (Sounds::Disabled, _, _) => OfferPermission::Disabled,
            (Sounds::Enabled, Demo::Active, _) => OfferPermission::Demo,
            (Sounds::Enabled, Demo::Inactive, PowerBomb::Active) => OfferPermission::Explosion,
            (Sounds::Enabled, Demo::Inactive, PowerBomb::Inactive) => OfferPermission::Enabled,
        }
    }

    /// Qualify only the independently accepted unsuppressed transport arm.
    #[must_use]
    pub const fn permission(self) -> Option<audio_transport::OfferPermission> {
        match (self.sounds, self.demo, self.power_bomb) {
            (Sounds::Enabled, Demo::Inactive, PowerBomb::Inactive) => {
                Some(audio_transport::OfferPermission::Enabled)
            }
            _ => None,
        }
    }
}

/// One complete source-qualified envelope for each distinct consumer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Physical {
    /// Complete `SoundOffer` owner, including its enqueue completion.
    pub sound_offer: data::timing::Template,
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
