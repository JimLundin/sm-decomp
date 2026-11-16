//! Admitted audio transport. Callers own validated content, requests and elapsed time.
//! Audio instants are normalized nonnegative unbounded rational AudioCycles. Work and
//! CPU master ticks are distinct unbounded whole quantities. Every due milestone is
//! processed through the stop, and saved pending work resumes without early publication.
//! Echo membership covers both music voices and all six effect voices independently of ownership.
//! Spec: EffectResources. Construct `GroupState::history` (`ReleaseHistory`) from the caller's
//! complete admitted ordered history: one entry for Action/Ambient and two for Contact, regardless
//! of currently active parts. It is the only retained release count for each part throughout its
//! lifecycle; `LivePart` owns its allocated voice and playback progress, while
//! `Playback::Prepared` identifies a cue with no allocated voice or consumed span. Neither owns
//! release history. Inspect entries directly, using `ReleaseMemory::remaining` for an observation
//! count. Fresh initialization does not infer or reset history. Restoration deserializes the
//! entire State unchanged.
//! `effects::turn` requires the group identity, its current `GroupState` with complete history,
//! validated `Content`, occupied effect voices and current semantic echo membership. It returns
//! the completed group, exact work and complete `EffectResult::echo_voices` for deferred publication.
//! Allocation removes only acquired voices from membership, preserving unrelated members, gains
//! and echo-write enable. Preparation and ownership release preserve membership.
//! `CompletionResult::Effect` saves that membership with the complete group history, and
//! `transport::advance` commits both only at publication. Prior committed history and
//! `State::song.mix.echo_voices` remain readable meanwhile; restoring pending work preserves its
//! result without starting a second allocation.
//! This is semantic echo selection, not a waveform, DSP write or hardware-output backend.
//!
//! Spec: VisibleCompletion. Initialize `State::resident_music_reply` with a `ResidentMusicReply`
//! struct literal containing the caller's opaque `identity` String and `original_byte` (0..=255).
//! These are distinct semantic and encoding values, not a selection request. `Reply::Music`
//! publications expose both; `Reply::Effect` retains effect request-absence semantics.
//! Initialize the polling episode with `transport::initialize` only for newly constructed state.
//! Restore the entire serialized `State` directly, including its resident reply and pending phase.
//! Callers supply the same validated `Content` identity when resuming with `transport::advance`;
//! restoration does not initialize, advance or replay inputs.
#![expect(
    clippy::doc_markdown,
    reason = "Spec entry identifiers are plain catalogue labels throughout the model"
)]
/// Producer's admitted cue queue allowance, distinct from movement request metadata.
pub use types::audio::CueAllowance as QueueAllowance;
pub use types::audio::*;
pub mod catalogue;
pub mod effects;
pub mod music;
pub mod sender;
pub mod time;
pub mod transport;
