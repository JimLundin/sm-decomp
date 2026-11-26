//! Typed audio content and the audio-transport State saved between frames. Relations between
//! content fields are the importer's.
#![expect(
    clippy::doc_markdown,
    reason = "Spec entry identifiers are consistently written as plain catalogue labels"
)]
pub use crate::pitch::{EffectPitch, MusicalPitch};
use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::num::NonZeroU8;

/// Consume one turn of a positive count: the count left, or none once its last turn is consumed.
///
/// The sender's delays (RequestRelease), finite phrase repeats (FinitePhrases), loudness ramps
/// (LoudnessProgress) and effect release counts (EffectResources) all count down this way.
#[must_use]
pub const fn count_down(count: NonZeroU8) -> Option<NonZeroU8> {
    NonZeroU8::new(count.get().saturating_sub(1))
}
/// Spec: MusicalProgress. Positive musical duration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MusicTurns(pub NonZeroU8);
/// Spec: EffectResources. Positive effect duration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectTurns(pub NonZeroU8);
/// Spec: LoudnessProgress. Fractional loudness, wrapping at sixteen bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Loudness(pub u16);
/// Spec: SongControls. Resolved instrument identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimbreId(pub u16);
/// Spec: SongControls. Fractional offset is zero in this boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PitchOffset {
    /// No transposition.
    Zero,
}
/// Spec: EffectResources. Effect pool identities; never music voices.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EffectVoiceId {
    /// Highest voice.
    V7,
    /// Next voice.
    V6,
    /// Next voice.
    V5,
    /// Next voice.
    V4,
    /// Next voice.
    V3,
    /// Lowest voice.
    V2,
}
/// Spec: SongControls. Shared identity universe; echo membership does not allocate voices.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VoiceId {
    /// Arrangement's first track.
    FirstMusic,
    /// Arrangement's second track.
    SecondMusic,
    /// Ordered effect-pool identity.
    Effect(EffectVoiceId),
}
/// Spec: StagedRequests. A sound group: one sound queue, one request line and one reply line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Enum, Serialize, Deserialize)]
pub enum SoundGroup {
    /// Action sounds.
    Action,
    /// Background sounds.
    Ambient,
    /// Contact sounds.
    Contact,
}
impl SoundGroup {
    /// Spec: FrameCadence. An effect turn and a sender service take the groups in this order.
    pub const IN_TURN_ORDER: [Self; 3] = [Self::Action, Self::Ambient, Self::Contact];
}
/// Spec: StagedRequests. Only admitted cues.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CueId {
    /// Footsteps.
    Footstep,
    /// Spinning.
    Spin,
    /// End spin.
    EndSpin,
    /// Landing or wall jump.
    Land,
    /// Power beam.
    Beam,
    /// Samus silence.
    SamusSilence,
    /// Background silence.
    BackgroundSilence,
    /// Protected door.
    Door,
    /// Shaft door's explosion: an ordinary Ambient cue with one finite effect part.
    ShaftExplosion,
}
impl CueId {
    /// Spec: OfferAcceptance. The group a cue belongs to, whose queue it joins.
    #[must_use]
    pub const fn group(self) -> SoundGroup {
        match self {
            Self::Spin | Self::EndSpin | Self::Beam | Self::SamusSilence => SoundGroup::Action,
            Self::BackgroundSilence | Self::ShaftExplosion => SoundGroup::Ambient,
            Self::Footstep | Self::Land | Self::Door => SoundGroup::Contact,
        }
    }
}
/// Spec: StagedRequests. Absence is not silence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Request {
    /// Released line.
    Absent,
    /// Present cue.
    Cue(CueId),
}
/// Spec: OfferAcceptance. Producer's admitted queue limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CueAllowance {
    /// One cue.
    One,
    /// Three cues.
    Three,
    /// Six cues.
    Six,
    /// Nine cues.
    Nine,
    /// Fifteen cues.
    Fifteen,
}
/// Spec: OfferAcceptance. Producer-owned suppression state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfferPermission {
    /// Sound permitted.
    Enabled,
    /// Sound disabled.
    Disabled,
    /// Demo playback.
    Demo,
    /// Explosion suppression.
    Explosion,
}
/// Spec: RequestDelivery and RequestRelease. Where a group's sender is in delivering a cue.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SenderProgress {
    /// May present the oldest queued cue.
    Ready,
    /// Presented this cue; waits for the group's reply to match it.
    AwaitCue(CueId),
    /// Acknowledged; services left before the request line is cleared.
    ClearDelay(NonZeroU8),
    /// Presented absence; waits for the group's reply to be absence.
    AwaitClear,
    /// Services left before the sender is ready again.
    ResetDelay(NonZeroU8),
}
/// Spec: OfferAcceptance. The most cues a sound queue holds.
pub const QUEUE_CAPACITY: usize = 15;
/// Spec: OfferAcceptance. A sound queue, oldest cue first, holding at most fifteen cues.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "VecDeque<CueId>", into = "VecDeque<CueId>")]
pub struct CueQueue(VecDeque<CueId>);
/// Spec: OfferAcceptance. A saved queue longer than fifteen cues.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QueueOverfull;
impl std::fmt::Display for QueueOverfull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a sound queue holds at most fifteen cues")
    }
}
impl TryFrom<VecDeque<CueId>> for CueQueue {
    type Error = QueueOverfull;
    fn try_from(cues: VecDeque<CueId>) -> Result<Self, Self::Error> {
        if cues.len() > QUEUE_CAPACITY {
            return Err(QueueOverfull);
        }
        Ok(Self(cues))
    }
}
impl From<CueQueue> for VecDeque<CueId> {
    fn from(queue: CueQueue) -> Self {
        queue.0
    }
}
impl CueQueue {
    /// The cues waiting, oldest first.
    #[must_use]
    pub const fn cues(&self) -> &VecDeque<CueId> {
        &self.0
    }
    /// How many cues wait.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Whether no cue waits.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Append a cue as the newest; a full queue refuses it.
    pub fn push(&mut self, cue: CueId) {
        if self.0.len() < QUEUE_CAPACITY {
            self.0.push_back(cue);
        }
    }
    /// Remove and return the oldest cue.
    pub fn pop_oldest(&mut self) -> Option<CueId> {
        self.0.pop_front()
    }
}
/// Spec: RequestDelivery. One group's sound queue and its sender's progress.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SenderState {
    /// Accepted cues not yet presented.
    pub queue: CueQueue,
    /// Where delivery stands.
    pub progress: SenderProgress,
}
/// Spec: LoudnessProgress. Inactive configuration is absent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoudnessRamp {
    /// Positive remaining turns.
    pub remaining: NonZeroU8,
    /// Whole target.
    pub target: u8,
    /// Signed wrapping fractional increment.
    pub increment: i16,
}
/// Spec: FinitePhrases. A repeated position owns its root continuation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackPosition {
    /// Root semantic boundary, including completion.
    Root {
        /// Next semantic event.
        cursor: usize,
    },
    /// Nonnested finite phrase boundary.
    Repeated {
        /// Phrase identity.
        phrase: usize,
        /// Next event.
        cursor: usize,
        /// Remaining repeat count.
        remaining: NonZeroU8,
        /// Root continuation.
        continuation: usize,
    },
}
/// Spec: SongControls. Articulation belongs to Timing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Articulation {
    /// Units out of 256.
    pub ratio: u8,
    /// Whole note strength.
    pub strength: u8,
}
/// Spec: SongControls and MusicalProgress. Imported musical events.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MusicEvent {
    /// A duration choice, with its optional articulation bundled.
    Timing {
        /// Positive duration.
        duration: MusicTurns,
        /// Optional articulation.
        articulation: Option<Articulation>,
    },
    /// Sound a non-noise pitch, admitted range 19 through 51.
    Tone(MusicalPitch),
    /// Retain pitch.
    Sustain,
    /// Whole immediate loudness.
    Loudness(u8),
    /// Start finite ramp.
    Ramp {
        /// Positive length.
        turns: NonZeroU8,
        /// Whole target.
        target: u8,
    },
    /// Whole master loudness.
    MasterLoudness(u8),
    /// Resolved timbre.
    Timbre(TimbreId),
    /// Zero fractional offset.
    Offset(PitchOffset),
    /// Retained percussion identity.
    Percussion(u16),
    /// Echo gains and membership.
    EchoMix {
        /// Voice membership.
        voices: BTreeSet<VoiceId>,
        /// Nonnegative gain at most 127.
        left: u8,
        /// Nonnegative gain at most 127.
        right: u8,
    },
    /// Fixed delay and filter.
    EchoSpace {
        /// Fixed arrangement delay.
        delay: u8,
        /// Nonnegative feedback at most 127.
        feedback: u8,
        /// Resolved filter identity.
        filter: u16,
    },
    /// Enter a finite phrase.
    Repeat {
        /// Phrase identity.
        phrase: usize,
        /// Positive repeat count.
        count: NonZeroU8,
    },
    /// Complete this phrase.
    Complete,
}
/// Spec: FinitePhrases. Validated graph, root at zero.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackContent {
    /// Ordered semantic phrases.
    pub phrases: Vec<Vec<MusicEvent>>,
}
/// Spec: MusicalProgress. Fully retained musical track.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackState {
    /// Completed semantic boundary.
    pub position: TrackPosition,
    /// Positive active note countdown.
    pub note: MusicTurns,
    /// Nonnegative release countdown.
    pub release: u8,
    /// Retained duration.
    pub duration: MusicTurns,
    /// Retained articulation ratio.
    pub articulation: u8,
    /// Admitted semitone pitch.
    pub pitch: MusicalPitch,
    /// Fractional loudness.
    pub loudness: Loudness,
    /// Active configuration only.
    pub ramp: Option<LoudnessRamp>,
    /// Resolved non-noise timbre.
    pub timbre: TimbreId,
    /// Zero pitch offset.
    pub offset: PitchOffset,
    /// Whole strength.
    pub strength: u8,
}
/// Spec: SongControls. Arrangement-wide scalar choices.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SongMix {
    /// Fractional master loudness.
    pub master: Loudness,
    /// Percussion choice.
    pub percussion: u16,
    /// Echo voice membership.
    pub echo_voices: BTreeSet<VoiceId>,
    /// Scaled left gain.
    pub echo_left: u16,
    /// Scaled right gain.
    pub echo_right: u16,
    /// Fixed delay.
    pub echo_delay: u8,
    /// Whole feedback.
    pub feedback: u8,
    /// Filter identity.
    pub filter: u16,
    /// Echo write status.
    pub echo_enabled: bool,
}
/// Spec: WholeArrangement. Two tracks in arrangement order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SongState {
    /// Tracks in order.
    pub tracks: Vec<TrackState>,
    /// Retained mix.
    pub mix: SongMix,
}
/// Spec: ConnectedPitchMotion. Connected slide retains terminal fraction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PitchMotion {
    /// Nonzero speed below 256.
    pub speed: NonZeroU8,
    /// Target above initial pitch.
    pub target: EffectPitch,
}
/// Spec: EffectResources. Imported span.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectSpan {
    /// Positive duration.
    pub duration: EffectTurns,
    /// Semitone pitch in zero through 71.
    pub pitch: EffectPitch,
    /// Spec: EffectResources. Admitted effect instrument; its sound is the sequencer's.
    pub timbre: TimbreId,
    /// Connected slide, if any.
    pub motion: Option<PitchMotion>,
}
/// Spec: EffectResources. Finite or intro plus repeated spans.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectPartContent {
    /// Spans in physical part order.
    pub spans: Vec<EffectSpan>,
    /// Loop span position, if repeating.
    pub loop_start: Option<usize>,
}
/// Spec: EffectResources. Validated admitted clip.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectClip {
    /// Physical parts retain identity.
    pub parts: Vec<EffectPartContent>,
}
/// Spec: EffectResources. Inactive release has no saved countdown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseMemory {
    /// No release delay has been started.
    Inactive,
    /// Positive inherited or active release countdown.
    Active(NonZeroU8),
}
impl ReleaseMemory {
    /// Nonnegative count for observation; zero denotes inactive output, not saved memory.
    #[must_use]
    pub const fn remaining(self) -> u8 {
        match self {
            Self::Inactive => 0,
            Self::Active(count) => count.get(),
        }
    }
}
/// Spec: EffectResources. Sounding and releasing parts share instruction memory.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivePart {
    /// Allocated voice.
    pub voice: EffectVoiceId,
    /// Next span cursor.
    pub cursor: usize,
    /// Eight-bit countdown, including retained zero wrap.
    pub instruction: u8,
    /// Current semitone.
    pub pitch: EffectPitch,
    /// Fraction, including terminal fraction.
    pub fraction: u8,
    /// Active slide only below target.
    pub motion: Option<PitchMotion>,
}
/// Spec: EffectResources. A finished part owns no voice.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartState {
    /// Live part, possibly before first span.
    Live(LivePart),
    /// Finished physical part.
    Finished,
}
/// Spec: StagedRequests. Preparation allocates only on a later turn.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Playback {
    /// No parts.
    Empty,
    /// No span consumed or voice allocated.
    Prepared {
        /// Clip identity.
        cue: CueId,
    },
    /// Parts with permanent physical order.
    Active {
        /// Clip identity.
        cue: CueId,
        /// Parts in identity order.
        parts: Vec<PartState>,
    },
}
impl Playback {
    /// Spec: EffectResources. The voices its live parts own; prepared and finished parts own none.
    #[must_use]
    pub fn owned_voices(&self) -> BTreeSet<EffectVoiceId> {
        let Self::Active { parts, .. } = self else {
            return BTreeSet::new();
        };
        parts
            .iter()
            .filter_map(|part| match part {
                PartState::Live(live) => Some(live.voice),
                PartState::Finished => None,
            })
            .collect()
    }
}
/// Spec: EffectResources. Complete ordered release history, independent of playback.
///
/// The caller supplies one entry for Action and Ambient and two for Contact, whatever is playing.
/// Each part's count lives here through preparation, sounding, release, dormancy and completion;
/// playback keeps no second copy.
pub type ReleaseHistory = Vec<ReleaseMemory>;
/// Spec: StagedRequests. The audio side of one sound group: handled, staged and published apart.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupState {
    /// Spec: EffectResources. Every admitted part's release history, even absent or finished parts.
    pub history: ReleaseHistory,
    /// The staged request the group last handled.
    pub handled: Request,
    /// The request line's value at the group's last effect turn.
    pub staged: Request,
    /// Spec: PublishedReplies. The reply the group's last effect turn published.
    pub reply: Request,
    /// What the group is playing.
    pub playback: Playback,
}
/// Spec: PublishedReplies. The resident arrangement's reply: a semantic identity, and the original
/// reply byte it resolves to at the boundary.
///
/// It comes with the resident arrangement, independently of any music-selection request; music
/// turns and arrangement restarts never change it. Every byte, zero through 255, is admitted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResidentMusicReply {
    /// The caller's name for the resident arrangement's reply.
    pub identity: String,
    /// Original physical reply byte.
    pub original_byte: u8,
}
/// Everything audio transport remembers from one frame to the next, saved and restored whole.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// Each group's sound queue and sender progress.
    pub senders: EnumMap<SoundGroup, SenderState>,
    /// Spec: RequestDelivery. Each group's request line: the value its sender last presented.
    pub request_lines: EnumMap<SoundGroup, Request>,
    /// Each group's audio side: staged, handled, published reply, history and playback.
    pub groups: EnumMap<SoundGroup, GroupState>,
    /// The resident song's progress and controls.
    pub song: SongState,
    /// Spec: PublishedReplies. The reply the last music turn published on the music line.
    pub music_reply: ResidentMusicReply,
    /// The resident arrangement's reply, supplied with it and never changed here.
    pub resident_music_reply: ResidentMusicReply,
    /// Effect voices reserved outside the three sound groups.
    pub reservations: BTreeSet<EffectVoiceId>,
}
/// Spec: EffectResources and WholeArrangement. Caller owns validated resident content.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Content {
    /// Admitted effect clips. The shaft explosion clip has one finite part.
    pub effects: BTreeMap<CueId, EffectClip>,
    /// Two tracks in order.
    pub tracks: Vec<TrackContent>,
}
