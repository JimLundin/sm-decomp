//! Actual borrowed publication and sound owner facts, independent of hardware interrupt timing.
use canonical::Canonical;

/// Current software publication request retained by its actual owner.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoftwareRequest {
    /// The preceding publication request has been acknowledged.
    Acknowledged,
}

/// Actual publication owner at the completed-return origin.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublicationOrigin {
    /// Current software request; interrupt counters cannot supply this fact.
    pub prior_request: SoftwareRequest,
}

/// Actual owner receipt permitting continuation after the preceding publication.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameContinuationAfterNmi {
    /// The retained actual owner that offered this continuation.
    pub publication: PublicationOrigin,
}

/// Actual current music service countdown disposition.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicCountdown {
    /// The actual current countdown permits service.
    Ready,
}

/// Actual current music queue disposition.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicQueue {
    /// The current producer and consumer positions coincide.
    Empty,
}

/// Actual sound owner's current service inputs, without executing a service transition.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MusicServiceReceipt {
    /// Current countdown disposition.
    pub countdown: MusicCountdown,
    /// Current queue disposition.
    pub queue: MusicQueue,
}

/// Completed music owner's sound effect disposition.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicRequest {
    /// No cue, physical publication or reply observation was requested.
    None,
}

/// Separately borrowed actual music owner completion, consumed at its terminal opportunity.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct MusicCompletionReceipt {
    /// Actual current input receipt that must still match before this completion is applied.
    pub dependency: MusicServiceReceipt,
    /// Actual terminal owner disposition, separate from its input.
    pub terminal: MusicServiceReceipt,
    /// Actual completed owner's sound effect disposition.
    pub request: MusicRequest,
}
