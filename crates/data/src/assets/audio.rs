//! Resident song and cue content owned by the cartridge's asset import.

pub use types::audio::{
    Articulation, Content, CueId, EffectClip, EffectPartContent, EffectPitch, EffectSpan,
    EffectTurns, EffectVoiceId, MusicEvent, MusicTurns, MusicalPitch, PitchMotion, PitchOffset,
    ResidentMusicReply, SoundGroup, TimbreId, TrackContent, VoiceId,
};

/// The admitted arrangement and cue bank, with the resources they reference.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResidentAudio {
    /// The sequencer's two ordered tracks and effect graphs.
    pub content: Content,
    /// Resident music identity and its physical reply.
    pub music_reply: ResidentMusicReply,
    /// The arrangement's fixed echo delay, in delay units.
    pub echo_delay: u8,
    /// Indexed by the dense timbre identities used in `content`.
    pub instruments: Vec<Instrument>,
    /// Indexed by the sample identities used in `instruments`.
    pub samples: Vec<SampleSource>,
    /// Correspondence between cues and request/reply port values.
    pub aliases: Vec<CueAlias>,
}

/// A checked sampled instrument.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Instrument {
    /// Which checked sample backs this instrument.
    pub sample: SampleId,
    /// Envelope and gain settings used by the instrument.
    pub envelope: Envelope,
    /// Pitch scale, in 8.8 units.
    pub pitch_scale: u16,
}

/// A dense sample identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SampleId(pub u16);

/// Instrument envelope settings retained for resource identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Envelope {
    /// ADSR mode and attack/decay rates.
    pub attack_decay: u8,
    /// Sustain level and rate.
    pub sustain: u8,
    /// Gain mode and rate.
    pub gain: u8,
}

/// Bounds of a checked sample stream, in mono decoded frames.
///
/// Resident timing uses this resource relation without needing waveform data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SampleSource {
    /// Total decoded frames before the first end marker.
    pub frames: u32,
    /// Start of the repeated tail, when looped.
    pub loop_from: Option<u32>,
    /// Digest of the selected stream through its first end marker.
    pub source_sha256: [u8; 32],
}

/// A supported cue code on one effect port.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CueAlias {
    /// Effect port.
    pub group: SoundGroup,
    /// Physical request and reply value.
    pub byte: u8,
    /// Semantic cue identity.
    pub cue: CueId,
}
