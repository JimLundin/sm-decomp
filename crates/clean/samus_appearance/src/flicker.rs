//! Body visibility at an ordinary room drawing opportunity.

use types::Parity;

/// The facts for one ordinary body draw, after any hit and before spending protection or push time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrdinaryBodyDraw {
    /// Health has at least one protection frame left at the draw.
    pub protected_while_drawn: bool,
    /// Movement has at least one push frame left at the draw, regardless of pose.
    pub push_time_left: bool,
    /// A special suit colour effect has time left at the draw, separate from the hurt colour pulse.
    pub timed_special_colour: bool,
    /// Parity of finished frames at the start of ordinary gameplay work.
    pub finished_frame_parity: Parity,
}

impl OrdinaryBodyDraw {
    /// Spec: `ProtectionFlicker`. Omit all body placements on odd frames while protected,
    /// unless push time or special suit colour time remains. Body tile requests are unaffected.
    #[must_use]
    pub const fn body_drawn(self) -> bool {
        !self.protected_while_drawn
            || self.push_time_left
            || self.timed_special_colour
            || matches!(self.finished_frame_parity, Parity::Even)
    }
}
