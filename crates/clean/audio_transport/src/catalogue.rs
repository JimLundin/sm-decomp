//! One catalogue row per named spec entry, with source-expected check families.
/// Named behaviour identifiers, one per spec entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, enum_map::Enum)]
pub enum BehaviourId {
    /// Queue admission.
    OfferAcceptance,
    /// Oldest delivery and acknowledgement.
    RequestDelivery,
    /// Clear and reset lifetime.
    RequestRelease,
    /// Each frame's effect and music turns around its sender service.
    FrameCadence,
    /// Replies each turn publishes.
    PublishedReplies,
    /// Handled versus staged requests.
    StagedRequests,
    /// Note countdown and release.
    MusicalProgress,
    /// Finite repetition and continuation.
    FinitePhrases,
    /// Both roots restart on first completion.
    WholeArrangement,
    /// Wrapping loudness ramp.
    LoudnessProgress,
    /// Physical effect ownership.
    EffectResources,
    /// Upward connected slides.
    ConnectedPitchMotion,
    /// Retained song choices.
    SongControls,
}
pub use types::{Kind, Switch};
/// Named rule, classification and supplied check family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// What the player sees.
    pub summary: &'static str,
    /// Spec kind.
    pub kind: Kind,
    /// Spec switch.
    pub switch: Switch,
    /// Supplied invented vector families exercising this entry.
    pub checked_by: &'static [&'static str],
}
/// Spec catalogue, exhaustively keyed by the compiler.
#[must_use]
pub fn catalogue() -> enum_map::EnumMap<BehaviourId, Behaviour> {
    enum_map::enum_map! {
        BehaviourId::OfferAcceptance => Behaviour { summary: "admit a sound offer only while sound is permitted and its queue allowance has space", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["offer", "frame"] },
        BehaviourId::RequestDelivery => Behaviour { summary: "deliver the oldest queued cue and wait for its acknowledgement", kind: Kind::Quirk, switch: Switch::Fixed, checked_by: &["sender", "frame"] },
        BehaviourId::RequestRelease => Behaviour { summary: "clear a handled cue before sending the next one", kind: Kind::Quirk, switch: Switch::Fixed, checked_by: &["sender", "frame"] },
        BehaviourId::FrameCadence => Behaviour { summary: "a frame runs its effect turns before and after its sender service, and its music turns", kind: Kind::Timing, switch: Switch::Fixed, checked_by: &["frame"] },
        BehaviourId::PublishedReplies => Behaviour { summary: "each effect turn publishes every group's handled request, each music turn the resident reply", kind: Kind::Output, switch: Switch::Fixed, checked_by: &["staged", "frame", "resident_reply"] },
        BehaviourId::StagedRequests => Behaviour { summary: "handle the staged request before publishing, then stage the request line", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["staged", "frame"] },
        BehaviourId::MusicalProgress => Behaviour { summary: "time the next musical event and its release", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["progress", "frame"] },
        BehaviourId::FinitePhrases => Behaviour { summary: "a finite phrase repeat returns to its enclosing phrase", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["phrase"] },
        BehaviourId::WholeArrangement => Behaviour { summary: "the first completed root restarts the repeating arrangement", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["restart", "content_admission", "resident_reply"] },
        BehaviourId::LoudnessProgress => Behaviour { summary: "interpolate loudness for a finite number of music turns", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["ramp", "control", "inactive_ramp"] },
        BehaviourId::EffectResources => Behaviour { summary: "prepared sounds acquire free voices and keep them until playback finishes", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["effect", "effect_history", "effect_echo"] },
        BehaviourId::ConnectedPitchMotion => Behaviour { summary: "a connected sound slides up to its target and keeps its final fraction", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["effect"] },
        BehaviourId::SongControls => Behaviour { summary: "retain musical choices until a later musical event replaces them", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["control", "effect_echo"] },
    }
}
