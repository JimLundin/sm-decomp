//! Encapsulated display construction and committed component persistence.

use std::sync::Arc;

use super::{
    commit::{OwnedDisplay, ResidentDisplayDescription},
    hdma_sources::HdmaCatalog,
    records::{Scene, SceneMedia},
    samus::SamusCatalog,
    start::{
        ContinuationHistory, DisplayStartDescription, InitialAudioIntent, InputHistory,
        ReferenceProfile, StartError, StartIdentity, ValidatedDisplayStart,
    },
    state::{DisplayDescription, DisplayError},
};

/// One-way starting data transfer to the corresponding G owners. The display
/// component does not keep another writable input, continuation or audio copy.
#[derive(Clone, Debug, PartialEq)]
pub struct DisplayStartMetadata {
    pub identity: StartIdentity,
    pub reference: ReferenceProfile,
    pub input_history: InputHistory,
    pub continuation: ContinuationHistory,
    pub audio: InitialAudioIntent,
    pub samus_content: Arc<SamusCatalog>,
}

pub struct OpenedDisplay {
    pub display: DisplayComponent,
    pub metadata: DisplayStartMetadata,
}

/// The mutable owner is encapsulated. Public construction descriptions may be
/// edited before validation; callers never receive a mutable live display borrow.
#[derive(Clone, Debug, PartialEq)]
pub struct DisplayComponent {
    owner: OwnedDisplay,
}

/// Complete display-only persistence, including the final resident state after
/// late settlement and its separate immutable visible history. Full G persistence
/// additionally supplies its actual input, continuation and semantic audio owners.
#[derive(Clone)]
pub struct DisplayCheckpoint {
    description: DisplayDescription,
    resident: ResidentDisplayDescription,
    committed: Scene,
    media: Arc<SceneMedia>,
    hdma_content: Arc<HdmaCatalog>,
}

impl DisplayComponent {
    /// Consumes the sealed component start without simulation or device delivery.
    pub fn open(start: ValidatedDisplayStart) -> Result<OpenedDisplay, StartError> {
        let parts = start.into_parts();
        let owner = OwnedDisplay::open(
            parts.display,
            parts.resident,
            parts.committed_scene,
            parts.media,
            parts.hdma_content,
        )
        .map_err(StartError::Display)?;
        Ok(OpenedDisplay {
            display: Self { owner },
            metadata: DisplayStartMetadata {
                identity: parts.identity,
                reference: parts.reference,
                input_history: parts.input_history,
                continuation: parts.continuation,
                audio: parts.audio,
                samus_content: parts.samus_content,
            },
        })
    }

    pub fn committed_scene(&self) -> &Scene {
        self.owner.committed()
    }

    /// Captures only a committed display phase. This neither renders nor settles
    /// a partially completed interval, and is not a complete Session checkpoint.
    pub fn checkpoint(&self) -> Result<DisplayCheckpoint, DisplayError> {
        let (description, resident, committed, media, hdma_content) =
            self.owner.checkpoint_parts()?;
        Ok(DisplayCheckpoint {
            description,
            resident,
            committed,
            media,
            hdma_content,
        })
    }

    /// Complete replacement validation precedes adoption. The caller transfers
    /// the returned metadata to its actual G owners after successful replacement.
    pub fn reset(
        &mut self,
        start: ValidatedDisplayStart,
    ) -> Result<DisplayStartMetadata, StartError> {
        let replacement = Self::open(start)?;
        *self = replacement.display;
        Ok(replacement.metadata)
    }

    /// Actual producer/settlement integration stays inside G, never the renderer.
    pub fn owner(&self) -> &OwnedDisplay {
        &self.owner
    }
    pub fn owner_mut(&mut self) -> &mut OwnedDisplay {
        &mut self.owner
    }
}

impl DisplayCheckpoint {
    /// Encode committed display state with independently supplied G metadata and
    /// immutable requirements. Validation never executes a producer or delivery.
    pub fn encode_checkpoint(
        &self,
        metadata: &DisplayStartMetadata,
        required: &super::start::DisplayStartRequirements,
    ) -> Result<Vec<u8>, String> {
        let (description, media) = self.clone().into_start_description(metadata.clone());
        ValidatedDisplayStart::construct(description, media, required)
            .map_err(|error| format!("display checkpoint start validation: {error:?}"))?;
        super::checkpoint_codec::encode(
            &self.description,
            &self.resident,
            &self.committed,
            &self.media,
            metadata,
            required,
        )
    }

    /// Decode into a detached validated checkpoint. The supplied final resident
    /// image stays separate from every immutable visible Scene line.
    pub fn decode_checkpoint(
        bytes: &[u8],
        media: Arc<SceneMedia>,
        metadata: &DisplayStartMetadata,
        required: &super::start::DisplayStartRequirements,
    ) -> Result<Self, String> {
        let start = super::checkpoint_codec::decode(bytes, media, metadata, required)?;
        let opened = DisplayComponent::open(start)
            .map_err(|error| format!("display checkpoint opening: {error:?}"))?;
        opened
            .display
            .checkpoint()
            .map_err(|error| format!("display checkpoint capture: {error:?}"))
    }

    pub fn committed_scene(&self) -> &Scene {
        &self.committed
    }

    pub fn resident(&self) -> &ResidentDisplayDescription {
        &self.resident
    }

    /// Reopening still requires complete current G metadata and independently
    /// validated requirements. A display checkpoint does not synthesize them.
    pub fn into_start_description(
        self,
        metadata: DisplayStartMetadata,
    ) -> (DisplayStartDescription, Arc<SceneMedia>) {
        (
            DisplayStartDescription {
                identity: Some(metadata.identity),
                reference: Some(metadata.reference),
                input_history: Some(metadata.input_history),
                continuation: Some(metadata.continuation),
                audio: Some(metadata.audio),
                display: Some(self.description),
                resident: Some(self.resident),
                hdma_content: Some(self.hdma_content),
                samus_content: Some(metadata.samus_content),
                committed_scene: Some(self.committed),
            },
            self.media,
        )
    }
}
