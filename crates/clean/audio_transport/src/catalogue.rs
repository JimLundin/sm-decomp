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
    /// CPU service clock.
    SenderTiming,
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
    /// Effect work.
    EffectTiming,
    /// Retained song choices.
    SongControls,
    /// Consumed versus preview work.
    MusicTiming,
    /// Independent overflow phases.
    IndependentCadence,
    /// Two equal observations.
    StableRequests,
    /// Deferred result publication.
    VisibleCompletion,
    /// Four-bit count and polling lattice.
    ClockObservation,
    /// Ordered publication anchors.
    ReplyCadence,
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
        BehaviourId::OfferAcceptance => Behaviour { summary: "admit permitted offers with space", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["offer"] },
        BehaviourId::RequestDelivery => Behaviour { summary: "deliver oldest cue and retry acknowledgement", kind: Kind::Quirk, switch: Switch::Fixed, checked_by: &["sender"] },
        BehaviourId::RequestRelease => Behaviour { summary: "clear a handled line before next delivery", kind: Kind::Quirk, switch: Switch::Fixed, checked_by: &["sender"] },
        BehaviourId::SenderTiming => Behaviour { summary: "observe and present during CPU sender work", kind: Kind::Timing, switch: Switch::Fixed, checked_by: &["sender"] },
        BehaviourId::StagedRequests => Behaviour { summary: "prepare changed requests on a later turn", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["staged"] },
        BehaviourId::MusicalProgress => Behaviour { summary: "count finite note and release turns", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["progress"] },
        BehaviourId::FinitePhrases => Behaviour { summary: "finite repeat returns to its root", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["phrase"] },
        BehaviourId::WholeArrangement => Behaviour { summary: "first root completion restarts both tracks", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["restart", "content_admission"] },
        BehaviourId::LoudnessProgress => Behaviour { summary: "finite wrapping interpolation retains active configuration", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["ramp", "control", "inactive_ramp"] },
        BehaviourId::EffectResources => Behaviour { summary: "retain all part histories and remove allocated echo members", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["effect", "effect_history", "effect_echo"] },
        BehaviourId::ConnectedPitchMotion => Behaviour { summary: "connected motion retains its terminal fraction", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["effect"] },
        BehaviourId::EffectTiming => Behaviour { summary: "visited parts and resources determine deferred semantic work", kind: Kind::Timing, switch: Switch::Fixed, checked_by: &["effect", "effect_history", "effect_echo"] },
        BehaviourId::SongControls => Behaviour { summary: "retain choices until consumed replacements", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["control"] },
        BehaviourId::MusicTiming => Behaviour { summary: "consume or preview the named semantic path", kind: Kind::Timing, switch: Switch::Fixed, checked_by: &["music_work"] },
        BehaviourId::IndependentCadence => Behaviour { summary: "distribute only the batch remainder", kind: Kind::Mechanic, switch: Switch::Fixed, checked_by: &["cadence"] },
        BehaviourId::StableRequests => Behaviour { summary: "stage two equal actual observations", kind: Kind::Quirk, switch: Switch::Fixed, checked_by: &["sample"] },
        BehaviourId::VisibleCompletion => Behaviour { summary: "publish completed results exactly once", kind: Kind::Output, switch: Switch::Fixed, checked_by: &["completion", "clock", "resident_reply"] },
        BehaviourId::ClockObservation => Behaviour { summary: "work and retries accrue the wrapping clock count", kind: Kind::Timing, switch: Switch::Fixed, checked_by: &["clock", "completion"] },
        BehaviourId::ReplyCadence => Behaviour { summary: "ordered group and resident music publication anchors", kind: Kind::Timing, switch: Switch::Fixed, checked_by: &["completion", "resident_reply"] },
    }
}
