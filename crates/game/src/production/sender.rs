//! The sound permission a producer's offer carries, and Movement's touchdown offer.

use super::{ProductionFault, Unsupported};
use audio_transport::{CueId, Offer, QueueAllowance};
use canonical::Canonical;

/// The Contact offer of Movement's reached touchdown sound.
pub(super) fn offer_touchdown(
    request: types::SoundRequest,
    current: OfferCurrent,
) -> Result<Offer, ProductionFault> {
    if request.identity != types::SoundIdentity::SamusLandedOrWallJumped
        || request.library != types::SoundLibrary::Library3
        || request.queue_allowance != types::QueueAllowance(6)
    {
        return Err(ProductionFault::Unsupported(
            Unsupported::MovementTouchdownSound(request),
        ));
    }
    Ok(Offer {
        cue: CueId::Land,
        allowance: QueueAllowance::Six,
        permission: current.transport_permission(),
    })
}

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
