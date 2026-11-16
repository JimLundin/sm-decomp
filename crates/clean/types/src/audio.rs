//! Typed content and retained state. Content relations are owned by the importer.
#![expect(
    clippy::doc_markdown,
    reason = "Spec entry identifiers are consistently written as plain catalogue labels"
)]
pub use crate::pitch::{EffectPitch, MusicalPitch};
use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::num::NonZeroU8;

/// Spec: VisibleCompletion. Normalized nonnegative rational AudioCycles, without a width limit.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "InstantFraction", into = "InstantFraction")]
pub struct AudioInstant(num_rational::BigRational);
/// Spec: VisibleCompletion. Input fractions require nonnegative numerators and positive denominators.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstantFraction {
    /// Nonnegative numerator.
    pub numerator: num_bigint::BigInt,
    /// Strictly positive denominator.
    pub denominator: num_bigint::BigInt,
}
/// Spec: VisibleCompletion. Invalid signs and zero denominators are rejected at the input edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FractionError {
    /// Negative numerator.
    NegativeNumerator,
    /// Zero or negative denominator.
    NonpositiveDenominator,
}
impl std::fmt::Display for FractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl TryFrom<InstantFraction> for AudioInstant {
    type Error = FractionError;
    fn try_from(value: InstantFraction) -> Result<Self, Self::Error> {
        use num_traits::{Signed, Zero};
        if value.denominator <= num_bigint::BigInt::zero() {
            return Err(FractionError::NonpositiveDenominator);
        }
        if value.numerator.is_negative() {
            return Err(FractionError::NegativeNumerator);
        }
        Ok(Self(num_rational::BigRational::new(
            value.numerator,
            value.denominator,
        )))
    }
}
impl From<AudioInstant> for InstantFraction {
    fn from(value: AudioInstant) -> Self {
        Self {
            numerator: value.0.numer().clone(),
            denominator: value.0.denom().clone(),
        }
    }
}
impl AudioInstant {
    /// Spec: VisibleCompletion. Construct an exact normalized fraction from unsigned components.
    /// # Errors
    /// A zero denominator returns `FractionError::NonpositiveDenominator`.
    pub fn new(
        numerator: num_bigint::BigUint,
        denominator: num_bigint::BigUint,
    ) -> Result<Self, FractionError> {
        Self::try_from(InstantFraction {
            numerator: numerator.into(),
            denominator: denominator.into(),
        })
    }
    /// Spec: VisibleCompletion. Read the normalized exact fraction.
    #[must_use]
    pub const fn rational(&self) -> &num_rational::BigRational {
        &self.0
    }
    /// Spec: VisibleCompletion. Exact normalized sum.
    #[must_use]
    pub fn sum(&self, other: &Self) -> Self {
        use std::ops::Add;
        Self(Add::add(&self.0, &other.0))
    }
    /// Spec: VisibleCompletion. Nonnegative difference, zero for earlier instants.
    #[must_use]
    pub fn difference(&self, other: &Self) -> Self {
        use std::ops::Sub;
        if self < other {
            return Self(num_rational::BigRational::from_integer(0.into()));
        }
        Self(Sub::sub(&self.0, &other.0))
    }
    /// Spec: VisibleCompletion. Whole nonnegative AudioCycles as an instant.
    #[must_use]
    pub fn whole(value: num_bigint::BigUint) -> Self {
        Self(num_rational::BigRational::from_integer(value.into()))
    }
}
/// Spec: SenderTiming. Unbounded whole CPU master ticks, distinct from audio work.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactCpuMasterTicks(pub num_bigint::BigUint);
/// Spec: SenderTiming. CPU clock quantity.
pub type CpuMasterTicks = ExactCpuMasterTicks;
/// Spec: SenderTiming. Ordinary caller convenience for whole CPU ticks.
#[must_use]
#[expect(non_snake_case, reason = "Preserve the ordinary caller constructor")]
pub fn CpuMasterTicks(value: u64) -> CpuMasterTicks {
    ExactCpuMasterTicks(value.into())
}
/// Spec: MusicTiming. Unbounded nonnegative integral AudioCycles.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactAudioCycles(pub num_bigint::BigUint);
/// Spec: MusicTiming. Immutable event work and accumulated work use exact whole cycles.
pub type AudioCycles = ExactAudioCycles;
/// Spec: MusicTiming. Ordinary caller convenience for integral work.
#[must_use]
#[expect(non_snake_case, reason = "Preserve the ordinary caller constructor")]
pub fn AudioCycles(value: u64) -> AudioCycles {
    ExactAudioCycles(value.into())
}
/// Spec: MusicTiming. Immutable consumption-only whole AudioCycles; preview adds none.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtraHandlingWork(pub num_bigint::BigUint);
impl ExtraHandlingWork {
    /// Spec: MusicTiming. Ordinary caller construction without restricting larger imported counts.
    #[must_use]
    pub fn new(value: u64) -> Self {
        Self(value.into())
    }
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
/// Spec: StagedRequests. Request group fixes the cue family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Enum, Serialize, Deserialize)]
pub enum SoundGroup {
    /// Action sounds.
    Action,
    /// Background sounds.
    Ambient,
    /// Contact sounds.
    Contact,
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
/// Spec: RequestDelivery and RequestRelease. Retained sender progress.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SenderProgress {
    /// May send.
    Ready,
    /// Await matching reply.
    AwaitCue(CueId),
    /// Clear countdown.
    ClearDelay(NonZeroU8),
    /// Await absence.
    AwaitClear,
    /// Reset countdown.
    ResetDelay(NonZeroU8),
}
/// Spec: SenderTiming. Saved CPU sender service.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SenderState {
    /// Oldest first, at most fifteen entries.
    pub queue: VecDeque<CueId>,
    /// Committed progress.
    pub progress: SenderProgress,
    /// Latest physical reply presented by platform.
    pub reply: Request,
    /// Current CPU time.
    pub now: CpuMasterTicks,
    /// Unfinished service, if any.
    pub pending: Option<SenderCompletion>,
}
/// Spec: SenderTiming. Milestones not yet consumed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SenderCompletion {
    /// Service origin.
    pub origin: CpuMasterTicks,
    /// Reply observation still due.
    pub observation: Option<CpuMasterTicks>,
    /// Result selected only at actual observation.
    pub result: Option<SenderResult>,
}
/// Spec: SenderTiming. Completed semantic result and remaining presentation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SenderResult {
    /// Result progress.
    pub progress: SenderProgress,
    /// Result queue.
    pub queue: VecDeque<CueId>,
    /// Remaining presentation at absolute CPU time.
    pub presentation: Option<(CpuMasterTicks, Request)>,
    /// Absolute completion instant.
    pub completion: CpuMasterTicks,
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
/// Spec: SongControls and MusicTiming. Imported semantic events.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MusicEvent {
    /// Bundled timing and immutable extra work.
    Timing {
        /// Positive duration.
        duration: MusicTurns,
        /// Optional articulation.
        articulation: Option<Articulation>,
        /// Consumption-only work.
        extra: ExtraHandlingWork,
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
    /// Spec: EffectResources and EffectTiming. Resolves the admitted ordinary instrument-work class.
    /// Retained effect-instrument playback is outside this transport result.
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
/// Spec: EffectResources. Complete ordered release history, independent of playback.
///
/// Construct from the caller's admitted history: one entry for Action/Ambient, two for Contact.
/// This group/content shape relation is an importer invariant. Inactive is not an inferred seed.
/// Each count lives here throughout preparation, sounding, release, dormancy and completion;
/// playback never stores a second copy. Pending work owns a separate completed result.
pub type ReleaseHistory = Vec<ReleaseMemory>;
/// Spec: StagedRequests. Handled, staged and published are separate.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupState {
    /// Spec: EffectResources. All admitted part histories, even absent or finished parts.
    /// Supply independently of active-part count at fresh construction; restore unchanged.
    pub history: ReleaseHistory,
    /// Last handled line.
    pub handled: Request,
    /// Stable staged request.
    pub staged: Request,
    /// Last physical reply.
    pub reply: Request,
    /// Retained playback.
    pub playback: Playback,
}
/// Spec: IndependentCadence. Independent threshold remainders.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CadencePhase {
    /// Zero through seven.
    pub effect: u8,
    /// Zero through 255.
    pub music: u8,
}
/// Spec: ClockObservation. Exact rational timer phase in zero up to, but excluding, 2048 AudioCycles.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "AudioInstant", into = "AudioInstant")]
pub struct AudioCyclePhase(AudioInstant);
/// Spec: ClockObservation. A saved timer phase must lie inside one timer period.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhaseOutOfRange;
impl std::fmt::Display for PhaseOutOfRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("timer phase outside its period")
    }
}
impl TryFrom<AudioInstant> for AudioCyclePhase {
    type Error = PhaseOutOfRange;
    fn try_from(value: AudioInstant) -> Result<Self, Self::Error> {
        const TIMER_PERIOD: u64 = 2048;
        if value >= AudioInstant::whole(TIMER_PERIOD.into()) {
            return Err(PhaseOutOfRange);
        }
        Ok(Self(value))
    }
}
impl From<AudioCyclePhase> for AudioInstant {
    fn from(value: AudioCyclePhase) -> Self {
        value.0
    }
}
impl AudioCyclePhase {
    /// Spec: ClockObservation. Read the exact retained timer phase.
    #[must_use]
    pub const fn instant(&self) -> &AudioInstant {
        &self.0
    }
    /// Spec: ClockObservation. Accrue exact time, retaining the phase and returning whole overflows.
    #[must_use]
    pub fn accrue(&self, elapsed: &AudioInstant) -> (Self, num_bigint::BigUint) {
        use std::ops::{Div, Mul};
        const TIMER_PERIOD: u64 = 2048;
        let total = self.0.sum(elapsed);
        let periods = total.rational().to_integer().magnitude().div(TIMER_PERIOD);
        let remainder = total.difference(&AudioInstant::whole(Mul::mul(&periods, TIMER_PERIOD)));
        (Self(remainder), periods)
    }
}
/// Spec: ClockObservation. Timer phase and four-bit pending count.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockState {
    /// Rational phase below 2048.
    pub phase: AudioCyclePhase,
    /// Four-bit count.
    pub unread: u8,
    /// Retained eight-cycle polling origin.
    pub lattice: AudioInstant,
}
/// Spec: ResidentMusicReply and VisibleCompletion. Resident semantic identity and its original
/// external reply byte are supplied independently of music-selection absence.
///
/// Construct with a struct literal: `identity` is the caller's opaque semantic name (any String),
/// and `original_byte` admits every byte, zero through 255. No fixture alias or byte is inferred.
/// Save and restore both fields unchanged; neither sampling nor arrangement restart replaces them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResidentMusicReply {
    /// Opaque semantic identity supplied with the admitted resident arrangement.
    pub identity: String,
    /// Original physical encoding, resolved only at the external publication boundary.
    pub original_byte: u8,
}
/// Spec: VisibleCompletion. Effects acknowledge requests; resident music has its own identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reply {
    /// Effect request acknowledgement, including genuine request absence.
    Effect(Request),
    /// Resident identity and preserved original external encoding.
    Music(ResidentMusicReply),
}
/// Spec: ReplyCadence. Selection absence does not make the resident music reply absent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplyLine {
    /// Sound line.
    Group(SoundGroup),
    /// Resident arrangement line.
    Music,
}
/// Spec: VisibleCompletion. Pending completed result is not committed state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompletionResult {
    /// Effect completion.
    Effect {
        /// Group identity.
        group: SoundGroup,
        /// Completed group, including every admitted history entry.
        result: GroupState,
        /// Complete semantic echo membership after this turn; committed only at publication.
        echo_voices: BTreeSet<VoiceId>,
    },
    /// Musical completion.
    Music(SongState),
}
/// Spec: StableRequests. The pair owns its publication/retry anchor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StableSample {
    /// Sample line.
    pub line: ReplyLine,
    /// Retry-adjusted anchor.
    pub anchor: AudioInstant,
    /// First observation, if consumed.
    pub first: Option<Request>,
}
/// Spec: ReplyCadence. Remaining dispatch after a stable pair.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Continuation {
    /// Next sound group.
    Effect(SoundGroup),
    /// Music or idle start after effects.
    AfterEffects,
    /// Poll after music.
    AfterMusic,
}
/// Spec: VisibleCompletion. Complete saved scheduler lifecycle.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioPhase {
    /// Polling opportunity.
    ObserveClock {
        /// Next opportunity.
        at: AudioInstant,
    },
    /// Read batch pending distribution.
    Distribute {
        /// Completion.
        at: AudioInstant,
        /// Retained batch.
        batch: u8,
    },
    /// Work owns its result and deadline.
    Work {
        /// Completion instant.
        at: AudioInstant,
        /// Semantic result.
        result: CompletionResult,
    },
    /// Stable pair after publication.
    Sample {
        /// Retained pair.
        sample: StableSample,
        /// Dispatch after acceptance.
        next: Continuation,
    },
    /// Work or idle interval starts later.
    MusicStart {
        /// Start anchor.
        at: AudioInstant,
    },
    /// Idle interval before polling.
    Idle {
        /// End instant.
        at: AudioInstant,
    },
}
/// Spec: VisibleCompletion. Caller saves every retained field, without inputs replayed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// Exact current time.
    pub now: AudioInstant,
    /// Clock memory.
    pub clock: ClockState,
    /// Cadence remainders.
    pub cadence: CadencePhase,
    /// Music still due in this interval.
    pub music_due: bool,
    /// Committed groups.
    pub groups: EnumMap<SoundGroup, GroupState>,
    /// Committed song.
    pub song: SongState,
    /// Spec: VisibleCompletion. Supply identity and original byte at initialization. Restore this
    /// field with the entire State, including pending work; do not reinitialize or replay inputs.
    pub resident_music_reply: ResidentMusicReply,
    /// External voice reservations.
    pub reservations: BTreeSet<EffectVoiceId>,
    /// Visible environment request values.
    pub requests: EnumMap<SoundGroup, Request>,
    /// Retained scheduler.
    pub phase: AudioPhase,
}
/// Spec: EffectResources and WholeArrangement. Caller owns validated resident content.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Content {
    /// Admitted effect clips.
    pub effects: BTreeMap<CueId, EffectClip>,
    /// Two tracks in order.
    pub tracks: Vec<TrackContent>,
}
