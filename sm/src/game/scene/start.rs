//! Supplied, validated display starts. Construction never advances simulation.
#![forbid(unsafe_code)]

use super::{
    commit::ResidentDisplayDescription,
    hdma_sources::HdmaCatalog,
    records::{
        AssetId, ColorInterpretation, Revision, Scene, SceneError, SceneMedia, SceneQuantization,
        ScreenPosition,
    },
    state::{DisplayDescription, DisplayError, DisplayState},
};
use std::sync::Arc;

pub const DISPLAY_START_SCHEMA: u16 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StartIdentity {
    pub schema: u16,
    pub gameplay: [u8; 32],
    pub reference: [u8; 32],
    /// Concrete initial/restore construction provenance, distinct from the
    /// independent Reference-profile fixture's capture identity.
    pub capture: [u8; 32],
    pub media: Revision,
    pub committed_tick: u64,
    pub next_input_record: u64,
}

/// All identities refer to supplied validated declarations and capture evidence.
/// This record supplies no cadence, color, input or slowdown default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneImagePolicy {
    pub width: u16,
    pub height: u16,
    pub crop_origin: ScreenPosition,
    pub color: ColorInterpretation,
    pub quantization: SceneQuantization,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceProfile {
    pub fingerprint: [u8; 32],
    pub release: [u8; 32],
    pub capture: [u8; 32],
    pub ticks_numerator: u64,
    pub ticks_denominator: u64,
    pub arithmetic_policy: [u8; 32],
    pub input_policy: [u8; 32],
    pub observation_policy: [u8; 32],
    pub color_policy: [u8; 32],
    pub slowdown_policy: [u8; 32],
    pub glitch_policy: [u8; 32],
    pub audio_timing_policy: [u8; 32],
    pub image: SceneImagePolicy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PortHistory {
    pub sampled: u16,
    pub previous: u16,
    pub pressed: u16,
    pub repeated: u16,
    pub repeat_counter: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionBindings {
    pub shoot: u16,
    pub jump: u16,
    pub run: u16,
    pub cancel: u16,
    pub switch: u16,
    pub aim_down: u16,
    pub aim_up: u16,
}

/// Required initial history; values are supplied, including an explicit zero.
/// G consumes this starting description; redraw never samples or updates it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputHistory {
    pub capture: [u8; 32],
    pub policy: [u8; 32],
    pub ports: [PortHistory; 2],
    pub debug_second_port: bool,
    pub repeat_first: u16,
    pub repeat_next: u16,
    pub released: u16,
    pub timed_held: u16,
    pub previous_timed_held: u16,
    pub newly_held: u16,
    pub held_timer: u16,
    pub held_timer_reset: u16,
    pub samus_held: u16,
    pub samus_pressed: u16,
    pub bindings: ActionBindings,
    pub bug_fix_counter: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommittedPhase {
    InitialObservation,
    SettledInterval,
}

/// A committed boundary is evidence of the source phase, not permission to run
/// the missing NMI/HDMA/IRQ work while opening the display.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContinuationHistory {
    pub capture: [u8; 32],
    pub observation_policy: [u8; 32],
    pub phase: CommittedPhase,
    pub next_input_record: u64,
    pub outer: u8,
    pub primary: u8,
    pub secondary: u8,
    pub music: u8,
    pub message: u8,
    pub auxiliary: u8,
    pub captured_mode: Option<u16>,
}

/// Immutable semantic declarations. A's player/mixer and C's accepted batches
/// are outside this starting input and outside G's checkpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioProgramDeclaration {
    pub program: AssetId,
    pub bank: AssetId,
    pub revision: Revision,
    pub definition: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProgramAnchor {
    pub program: AssetId,
    pub revision: Revision,
    pub source_interval: u64,
    pub phase: u64,
    pub timing_policy: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OngoingMusic {
    /// Supplied semantic silence, never an inferred absence of initial audio.
    Silent,
    Playing(ProgramAnchor),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitialAudioIntent {
    pub capture: [u8; 32],
    pub revision: Revision,
    pub timing_policy: [u8; 32],
    pub declarations: Arc<[AudioProgramDeclaration]>,
    pub music: OngoingMusic,
    pub ambience: Arc<[ProgramAnchor]>,
}

/// Immutable requirements resolved by G/D validation, independently of the
/// supplied starting description. Canonical fingerprint encoding remains G-owned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayStartRequirements {
    pub schema: u16,
    pub gameplay: [u8; 32],
    pub hdma_content: Arc<HdmaCatalog>,
    pub atmosphere_content: Arc<super::atmosphere::AtmosphereCatalog>,
    pub capture: [u8; 32],
    pub reference: ReferenceProfile,
    pub media: Revision,
    pub audio_revision: Revision,
    pub audio_programs: Arc<[AudioProgramDeclaration]>,
}

/// Optional fields express missing construction inputs, not runtime defaults.
/// The validated result below contains every required field.
pub struct DisplayStartDescription {
    pub identity: Option<StartIdentity>,
    pub reference: Option<ReferenceProfile>,
    pub input_history: Option<InputHistory>,
    pub continuation: Option<ContinuationHistory>,
    pub audio: Option<InitialAudioIntent>,
    pub display: Option<DisplayDescription>,
    pub resident: Option<ResidentDisplayDescription>,
    pub hdma_content: Option<Arc<HdmaCatalog>>,
    pub committed_scene: Option<Scene>,
}

#[derive(Debug)]
pub enum StartError {
    MissingIdentity,
    Schema,
    Fingerprint,
    MissingReference,
    Reference,
    MissingInputHistory,
    InputHistory,
    MissingContinuation,
    Continuation,
    MissingAudioAnchor,
    AudioDeclaration,
    AudioProgram(AssetId),
    MissingDisplay,
    MissingResidentDisplay,
    MissingResidentMedia,
    MissingHdmaContent,
    Display(DisplayError),
    MissingCommittedHistory,
    Scene(SceneError),
    Revision,
    CommitIdentity,
    ReferenceImage,
}

/// This seals the display component only. It is not a complete G Scenario,
/// Session constructor or canonical gameplay checkpoint.
pub struct ValidatedDisplayStart {
    identity: StartIdentity,
    reference: ReferenceProfile,
    input_history: InputHistory,
    continuation: ContinuationHistory,
    audio: InitialAudioIntent,
    display: DisplayState,
    resident: ResidentDisplayDescription,
    hdma_content: Arc<HdmaCatalog>,
    committed_scene: Scene,
    media: Arc<SceneMedia>,
}

/// One-way ownership transfer to G/display integration, after validation.
pub(crate) struct DisplayStartParts {
    pub identity: StartIdentity,
    pub reference: ReferenceProfile,
    pub input_history: InputHistory,
    pub continuation: ContinuationHistory,
    pub audio: InitialAudioIntent,
    pub display: DisplayState,
    pub resident: ResidentDisplayDescription,
    pub hdma_content: Arc<HdmaCatalog>,
    pub committed_scene: Scene,
    pub media: Arc<SceneMedia>,
}

fn identified(value: &[u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn valid_continuation(history: &ContinuationHistory) -> bool {
    match history.outer {
        0..=3 => history.captured_mode.is_none(),
        // The authoritative GAME_STATE_ENTRIES has45 entries. Validate before
        // adding the outer offset; wrapping a full-u16 mode is not admission.
        10..=54 => history
            .captured_mode
            .is_some_and(|mode| mode < 45 && history.outer == (mode + 10) as u8),
        _ => false,
    }
}

impl ReferenceProfile {
    fn validate(&self) -> Result<(), StartError> {
        if self.ticks_numerator == 0
            || self.ticks_denominator == 0
            || self.image.width == 0
            || self.image.height == 0
            || [
                self.fingerprint,
                self.release,
                self.capture,
                self.arithmetic_policy,
                self.input_policy,
                self.observation_policy,
                self.color_policy,
                self.slowdown_policy,
                self.glitch_policy,
                self.audio_timing_policy,
            ]
            .iter()
            .any(|identity| !identified(identity))
        {
            return Err(StartError::Reference);
        }
        Ok(())
    }
}

impl InitialAudioIntent {
    fn validate(
        &self,
        reference: &ReferenceProfile,
        capture: [u8; 32],
        next_input_record: u64,
    ) -> Result<(), StartError> {
        if self.capture != capture
            || self.timing_policy != reference.audio_timing_policy
            || !identified(&self.revision.0)
        {
            return Err(StartError::MissingAudioAnchor);
        }
        for (index, declaration) in self.declarations.iter().enumerate() {
            if declaration.program.0 == 0
                || declaration.bank.0 == 0
                || declaration.revision != self.revision
                || !identified(&declaration.definition)
                || self.declarations[..index]
                    .iter()
                    .any(|earlier| earlier.program == declaration.program)
            {
                return Err(StartError::AudioDeclaration);
            }
        }
        let validate_anchor = |anchor: &ProgramAnchor| {
            if anchor.program.0 == 0
                || anchor.revision != self.revision
                || anchor.timing_policy != self.timing_policy
                || anchor.source_interval > next_input_record
            {
                return Err(StartError::MissingAudioAnchor);
            }
            if !self
                .declarations
                .iter()
                .any(|declaration| declaration.program == anchor.program)
            {
                return Err(StartError::AudioProgram(anchor.program));
            }
            Ok(())
        };
        if let OngoingMusic::Playing(anchor) = self.music {
            validate_anchor(&anchor)?;
        }
        for anchor in self.ambience.iter() {
            validate_anchor(anchor)?;
        }
        Ok(())
    }
}

impl ValidatedDisplayStart {
    pub fn construct(
        description: DisplayStartDescription,
        media: Arc<SceneMedia>,
        requirements: &DisplayStartRequirements,
    ) -> Result<Self, StartError> {
        let identity = description.identity.ok_or(StartError::MissingIdentity)?;
        if identity.schema != DISPLAY_START_SCHEMA || identity.schema != requirements.schema {
            return Err(StartError::Schema);
        }
        if !identified(&identity.gameplay)
            || !identified(&identity.reference)
            || !identified(&identity.capture)
            || identity.gameplay != requirements.gameplay
            || identity.capture != requirements.capture
        {
            return Err(StartError::Fingerprint);
        }
        let reference = description.reference.ok_or(StartError::MissingReference)?;
        reference.validate()?;
        if reference.fingerprint != identity.reference || reference != requirements.reference {
            return Err(StartError::Fingerprint);
        }
        let input_history = description
            .input_history
            .ok_or(StartError::MissingInputHistory)?;
        if input_history.capture != identity.capture
            || input_history.policy != reference.input_policy
        {
            return Err(StartError::InputHistory);
        }
        let continuation = description
            .continuation
            .ok_or(StartError::MissingContinuation)?;
        if continuation.capture != identity.capture
            || continuation.observation_policy != reference.observation_policy
            || continuation.next_input_record != identity.next_input_record
            || (continuation.phase == CommittedPhase::InitialObservation
                && identity.committed_tick != 0)
            || !valid_continuation(&continuation)
        {
            return Err(StartError::Continuation);
        }
        let audio = description.audio.ok_or(StartError::MissingAudioAnchor)?;
        if audio.revision != requirements.audio_revision
            || audio.declarations.len() != requirements.audio_programs.len()
            || audio.declarations.iter().any(|declaration| {
                !requirements
                    .audio_programs
                    .iter()
                    .any(|required| required == declaration)
            })
        {
            return Err(StartError::AudioDeclaration);
        }
        audio.validate(&reference, identity.capture, identity.next_input_record)?;
        let display = description.display.ok_or(StartError::MissingDisplay)?;
        display.validate().map_err(StartError::Display)?;
        let hdma_content = description
            .hdma_content
            .ok_or(StartError::MissingHdmaContent)?;
        if hdma_content.gameplay() != identity.gameplay
            || requirements.hdma_content.gameplay() != requirements.gameplay
        {
            return Err(StartError::Fingerprint);
        }
        hdma_content
            .validate_expected(&requirements.hdma_content)
            .map_err(StartError::Display)?;
        display
            .validate_hdma_sources(&hdma_content)
            .map_err(StartError::Display)?;
        media.validate().map_err(StartError::Scene)?;
        display
            .validate_media(&media)
            .map_err(StartError::Display)?;
        display
            .atmosphere_catalog
            .validate_expected(&requirements.atmosphere_content)
            .map_err(StartError::Display)?;
        if identity.media != media.revision()
            || identity.media != requirements.media
            || display.revision() != identity.media
        {
            return Err(StartError::Revision);
        }
        let resident = description
            .resident
            .ok_or(StartError::MissingResidentDisplay)?;
        resident
            .validate_state(&display, &media)
            .map_err(StartError::Display)?;
        let committed_scene = description
            .committed_scene
            .ok_or(StartError::MissingCommittedHistory)?;
        committed_scene
            .validate(&media)
            .map_err(StartError::Scene)?;
        if committed_scene.producer_history().len() != committed_scene.lines().len() {
            return Err(StartError::MissingCommittedHistory);
        }
        if committed_scene
            .producer_history()
            .iter()
            .flat_map(|line| line.iter())
            .any(|record| display.producer_ids.validate_id(record.producer).is_err())
        {
            return Err(StartError::Display(DisplayError::Identity));
        }
        let resident_assets = &committed_scene.description().resident;
        if !resident_assets.contains(&resident.image.asset)
            || !resident_assets.contains(&resident.palette.asset)
        {
            return Err(StartError::MissingResidentMedia);
        }
        if committed_scene.tick() != identity.committed_tick
            || committed_scene.gameplay() != identity.gameplay
            || committed_scene.reference() != identity.reference
            || committed_scene.revision() != identity.media
        {
            return Err(StartError::CommitIdentity);
        }
        let geometry = &committed_scene.description().geometry;
        if geometry.width != reference.image.width
            || geometry.height != reference.image.height
            || geometry.crop_origin != reference.image.crop_origin
            || geometry.color != reference.image.color
            || geometry.quantization != reference.image.quantization
        {
            return Err(StartError::ReferenceImage);
        }
        Ok(Self {
            identity,
            reference,
            input_history,
            continuation,
            audio,
            display,
            resident,
            hdma_content,
            committed_scene,
            media,
        })
    }

    pub fn committed_scene(&self) -> &Scene {
        &self.committed_scene
    }

    pub(crate) fn into_parts(self) -> DisplayStartParts {
        DisplayStartParts {
            identity: self.identity,
            reference: self.reference,
            input_history: self.input_history,
            continuation: self.continuation,
            audio: self.audio,
            display: self.display,
            resident: self.resident,
            hdma_content: self.hdma_content,
            committed_scene: self.committed_scene,
            media: self.media,
        }
    }
}
