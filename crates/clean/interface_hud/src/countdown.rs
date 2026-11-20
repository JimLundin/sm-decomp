//! Spec: EscapeCountdown. Only maintenance before an escape begins is admitted.
//! Scene qualification, active countdowns and presentation-only choices belong to producers.

/// Spec: EscapeCountdown. The independently owned current escape countdown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Countdown {
    /// No escape countdown has begun; the sole admitted current value.
    Dormant,
}

/// Spec: EscapeCountdown. The answer about expiry, not a health or protection value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CountdownExpiry {
    /// No escape has run out of time at this opportunity.
    NoExpiry,
}

/// Spec: EscapeCountdown. Room presentation contribution, separate from status-bar cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CountdownPicture {
    /// No countdown picture contributed at this opportunity.
    Absent,
}

/// Spec: EscapeCountdown, CountdownContinuation. An actually qualified scene choice.
///
/// The producer must qualify the actual current scene input before supplying this value.
/// Ordinary HUD refresh, a mere offer, presentation-only and unknown choices do not imply
/// maintenance. Unsupported choices and active/expired countdowns require a neighboring
/// unsupported dependency before effects; they are deliberately not variants of this type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CountdownOpportunity {
    /// The scene actually selected countdown maintenance.
    MaintainCountdown,
}

/// Spec: EscapeCountdown. The actual selected opportunity and its computed outputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CountdownAnswer {
    /// Qualified scene choice actually received, not inferred from a HUD refresh.
    pub opportunity: CountdownOpportunity,
    /// Expiry answer for this opportunity.
    pub expiry: CountdownExpiry,
    /// Countdown picture contribution for this opportunity.
    pub picture: CountdownPicture,
}

impl Countdown {
    /// Spec: EscapeCountdown. Dormant maintenance keeps Dormant and reports no expiry/picture.
    pub(crate) const fn answer(self, opportunity: CountdownOpportunity) -> CountdownAnswer {
        match (self, opportunity) {
            (Self::Dormant, CountdownOpportunity::MaintainCountdown) => CountdownAnswer {
                opportunity,
                expiry: CountdownExpiry::NoExpiry,
                picture: CountdownPicture::Absent,
            },
        }
    }
}
