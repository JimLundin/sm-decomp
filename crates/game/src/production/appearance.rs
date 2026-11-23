//! Her clip's look ahead, the footstep's sender offer and the touchdown's graphics work.

#[cfg(test)]
mod tests;

/// The complete causal animation feedback, without advancing the playhead.
#[must_use]
pub fn ahead(
    state: &samus_appearance::State,
    clip: &samus_appearance::Clip,
) -> samus_appearance::Ahead {
    samus_appearance::Ahead::of(state.playhead, clip)
}

/// Route the actual contact request to its admitted sender queue.
/// Permission and queue contents come from their current owners at this opportunity.
///
/// # Errors
/// An unqualified cue mapping or an offer during a pending sender service.
pub fn offer_sound(
    state: &mut audio_transport::SenderState,
    sound: samus_appearance::FrameSound,
    permission: audio_transport::OfferPermission,
) -> Result<(), super::ProductionFault> {
    if sound != samus_appearance::FrameSound::Opportunity(samus_appearance::SoundRequest::Footstep)
    {
        return Err(super::ProductionFault::Unsupported(
            super::Unsupported::AppearanceSound(sound),
        ));
    }
    super::sender::advance(
        state,
        audio_transport::SoundGroup::Contact,
        super::SenderInput::Offer {
            cue: audio_transport::CueId::Footstep,
            allowance: audio_transport::QueueAllowance::Six,
            permission,
        },
    )?;
    Ok(())
}

/// Independently admitted area selection at the touchdown graphics entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchdownArea {
    /// The dry room arm that clears the two landing-atmosphere slots.
    DryClear,
}

/// Graphics work published by Appearance at the touchdown opportunity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchdownGraphics {
    /// Clear both landing-atmosphere type slots.
    ClearLandingSlots {
        /// Figure selected by the completed pose fit.
        figure: samus_appearance::Figure,
    },
}

pub(super) const fn touchdown_graphics(
    figure: samus_appearance::Figure,
    area: TouchdownArea,
) -> TouchdownGraphics {
    match area {
        TouchdownArea::DryClear => TouchdownGraphics::ClearLandingSlots { figure },
    }
}
