//! Retained producer materializations consumed by frame hardware transfers.

use canonical::Canonical;
use data::timing::ContentRegion;

#[cfg(test)]
mod tests;

mod publications;
pub use publications::{PublicationFault, Publications};
pub mod animation_caller;
pub mod appearance_caller;
pub mod appearance_stage;
pub mod background_caller;
pub mod borrowed;
pub mod borrowed_gameplay;
pub mod borrowed_player_preparation;
pub mod borrowed_prelude_receipts;
pub mod borrowed_preparation;
pub mod borrowed_receipts;
mod caller;
pub mod camera_stage;
pub mod contact_contract;
pub mod countdown_caller;
pub mod countdown_scene;
mod down_caller;
pub mod drawing_owners;
mod early_hud_caller;
pub mod frame_history;
pub mod frame_return;
pub mod frame_tail;
mod ground_caller;
pub mod haze;
pub mod health;
pub mod hud;
pub mod hud_camera_caller;
pub mod hud_controls_inputs;
pub mod hud_controls_owner;
pub mod hud_map_caller;
pub mod hud_stage;
pub mod inside_contract;
mod inside_provenance;
pub mod inside_world;
pub mod interruption_caller;
pub mod late_hud;
pub mod late_movement;
pub mod layer_blend;
pub mod liquid_history;
pub mod maintenance;
pub mod motion_reset_contract;
pub mod movement_caller;
pub mod movement_down_contract;
pub mod movement_ground_contract;
pub mod movement_pair_contract;
pub mod movement_stage;
pub mod movement_support_contract;
mod native_return;
mod next_opportunity;
mod ordinary_caller;
/// Actual play-clock opportunity and retained state.
pub mod play_time;
pub mod player_continuation;
mod player_continuation_caller;
pub mod pose_fit_caller;
pub mod pose_input;
pub mod post_down_caller;
pub mod post_down_contract;
pub mod random;
pub mod rendering_stage;
/// Actual quiet room-quake opportunity and Drawing input.
pub mod room_quake;
pub mod room_tail;
mod run_caller;
pub mod s6_camera;
pub mod s6_drawing;
pub mod samus_palette;
mod support_caller;
pub use ordinary_caller::OrdinaryWork;
pub mod sender_caller;
pub mod sender_contract;
pub mod sender_stage;
pub mod suffix_caller;
pub use caller::{
    Caller, Content, Dependency, MotionResetCompletion, MotionResetContent, Outputs,
    PlayerEntryContent, PlayerEntryStage, Prelude, PreludeSource, Preparation, PreparationContent,
    PreparationStage, Progress, Rules,
};
pub mod inputs;
pub mod interrupts;
pub mod movement;
mod state;
pub use state::{Fault, Senders, State};

pub(crate) fn validate_pins() -> Result<(), canonical::model::Unpinned> {
    canonical::model::pin_holds::<haze::HazePin>()?;
    canonical::model::pin_holds::<health::HealthPin>()?;
    canonical::model::pin_holds::<hud::CountdownPin>()?;
    canonical::model::pin_holds::<countdown_caller::AnswerPin>()?;
    canonical::model::pin_holds::<crate::state::owned::MovementPin>()?;
    canonical::model::pin_holds::<crate::state::owned::CameraPin>()?;
    canonical::model::pin_holds::<movement::WorkPin>()?;
    canonical::model::pin_holds::<movement::ReceiptsPin>()?;
    canonical::model::pin_holds::<movement_support_contract::MovementRulesPin>()?;
    canonical::model::pin_holds::<movement_support_contract::ActorCompletionPin>()?;
    canonical::model::pin_holds::<inputs::MovementInputsPin>()?;
    canonical::model::pin_holds::<inputs::WorldPin>()?;
    canonical::model::pin_holds::<inside_world::InsideWorldPin>()?;
    canonical::model::pin_holds::<inputs::PreparedActionPin>()?;
    canonical::model::pin_holds::<inputs::FigurePin>()?;
    canonical::model::pin_holds::<crate::production::appearance::WorkPin>()?;
    canonical::model::pin_holds::<crate::production::appearance::CannonDelayPin>()?;
    canonical::model::pin_holds::<crate::production::appearance::CannonObservationPin>()?;
    canonical::model::pin_holds::<contact_contract::CurrentPin>()?;
    canonical::model::pin_holds::<countdown_scene::CurrentPin>()?;
    canonical::model::pin_holds::<hud::HudPin>()?;
    canonical::model::pin_holds::<hud::WorkPin>()?;
    canonical::model::pin_holds::<hud_stage::DisplayPremisesPin>()?;
    canonical::model::pin_holds::<hud_stage::DisplayReceiptPin>()?;
    canonical::model::pin_holds::<hud_stage::ControlAnswerPin>()?;
    canonical::model::pin_holds::<borrowed_player_preparation::PlayerContinuationOriginsPin>()?;
    canonical::model::pin_holds::<borrowed_player_preparation::PlayerContinuationOffersPin>()?;
    canonical::model::pin_holds::<borrowed_player_preparation::PlayerContinuationCompletionsPin>()?;
    canonical::model::pin_holds::<borrowed_player_preparation::PlayerEntryOriginsPin>()?;
    canonical::model::pin_holds::<borrowed_player_preparation::PlayerEntryOffersPin>()?;
    canonical::model::pin_holds::<borrowed_player_preparation::PlayerEntryCompletionsPin>()?;
    canonical::model::pin_holds::<sender_stage::ReceiptPin>()?;
    canonical::model::pin_holds::<samus_palette::SequencesPin>()?;
    canonical::model::pin_holds::<samus_palette::WorkPin>()?;
    canonical::model::pin_holds::<play_time::TimePin>()?;
    canonical::model::pin_holds::<room_quake::QuakePin>()?;
    canonical::model::pin_holds::<crate::state::owned::SenderPin>()
}

/// A mutable buffer published by its completed actual producer.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Buffer {
    /// Completed backdrop bands from admitted haze work.
    HazeBackdropBands,
    /// Completed status/map tiles from the current HUD owner.
    HudTilemap,
    /// Completed sprite list using current movement, appearance and drawing inputs.
    SpriteDrawList,
}

/// A buffer whose actual producer completed before its hardware consumers.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Materialization {
    /// Producer identity, independent of physical addresses or source ordinals.
    pub buffer: Buffer,
    /// Complete produced content resolved at the hardware edge.
    pub region: ContentRegion,
}

/// Current transfer content and once-only producer completion progress.
/// Initial content includes immutable references and independently admitted unchanged buffers.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct TransferContent {
    initial: Vec<ContentRegion>,
    buffers: Vec<RetainedBuffer>,
}

#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
struct RetainedBuffer {
    materialization: Materialization,
    ready: bool,
}

/// Unsupported materialization admission leaves content and progress unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentFault {
    /// One completed producer was submitted again.
    AlreadyCompleted(Buffer),
    /// The physical backing no longer agrees with the retained producer result.
    RetainedBacking(Buffer),
    /// Empty or wrapping region cannot represent complete producer content.
    InvalidRegion,
    /// Distinct content owners overlap in the hardware address space.
    Overlap,
}

impl TransferContent {
    /// Admit initial transfer content without publishing a future producer buffer.
    ///
    /// # Errors
    /// Empty, wrapping or overlapping content regions.
    pub fn new(initial: Vec<ContentRegion>) -> Result<Self, ContentFault> {
        validate_regions(initial.iter())?;
        Ok(Self {
            initial,
            buffers: Vec::new(),
        })
    }

    /// Install a complete actual producer result once at its semantic Ready boundary.
    /// The frame validates selected model dependencies before calling this operation.
    ///
    /// # Errors
    /// Repeated completion or invalid/overlapping regions reject atomically.
    pub fn complete(&mut self, materialization: Materialization) -> Result<(), ContentFault> {
        let buffer = materialization.buffer;
        let owns_buffer = |held: &RetainedBuffer| held.materialization.buffer == buffer;
        if self
            .buffers
            .iter()
            .any(|held| owns_buffer(held) && held.ready)
        {
            return Err(ContentFault::AlreadyCompleted(buffer));
        }
        validate_regions(
            self.initial
                .iter()
                .chain(
                    self.buffers
                        .iter()
                        .filter(|held| !owns_buffer(held))
                        .map(|held| &held.materialization.region),
                )
                .chain(std::iter::once(&materialization.region)),
        )?;
        let next = RetainedBuffer {
            materialization,
            ready: true,
        };
        if let Some(held) = self.buffers.iter_mut().find(|held| owns_buffer(held)) {
            *held = next;
        } else {
            self.buffers.push(next);
        }
        Ok(())
    }

    fn retain(&mut self, materialization: &Materialization) -> Result<(), ContentFault> {
        let buffer = materialization.buffer;
        let held = self
            .buffers
            .iter_mut()
            .find(|held| held.materialization.buffer == buffer)
            .ok_or(ContentFault::RetainedBacking(buffer))?;
        if held.ready {
            return Err(ContentFault::AlreadyCompleted(buffer));
        }
        if held.materialization != *materialization {
            return Err(ContentFault::RetainedBacking(buffer));
        }
        held.ready = true;
        Ok(())
    }

    fn ready_buffer(&self, buffer: Buffer) -> Option<&Materialization> {
        self.buffers
            .iter()
            .find(|held| held.ready && held.materialization.buffer == buffer)
            .map(|held| &held.materialization)
    }

    fn begin_opportunity(&mut self) {
        for held in &mut self.buffers {
            held.ready = false;
        }
    }

    /// Complete current content supplied to hardware; absent future buffers stay absent.
    #[must_use]
    pub fn regions(&self) -> Vec<ContentRegion> {
        self.initial
            .iter()
            .chain(self.buffers.iter().map(|held| &held.materialization.region))
            .cloned()
            .collect()
    }

    /// Validate a complete imported continuation before allowing hardware consumers.
    ///
    /// # Errors
    /// Duplicate completions or invalid content ownership.
    pub fn validate(&self) -> Result<(), ContentFault> {
        for (ordinal, held) in self.buffers.iter().enumerate() {
            let buffer = held.materialization.buffer;
            if self
                .buffers
                .iter()
                .take(ordinal)
                .any(|earlier| earlier.materialization.buffer == buffer)
            {
                return Err(ContentFault::AlreadyCompleted(buffer));
            }
        }
        validate_regions(
            self.initial
                .iter()
                .chain(self.buffers.iter().map(|held| &held.materialization.region)),
        )
    }
}

fn validate_regions<'a>(
    regions: impl Iterator<Item = &'a ContentRegion>,
) -> Result<(), ContentFault> {
    let mut occupied = Vec::new();
    for region in regions {
        let length = u32::try_from(region.bytes.len()).map_err(|_| ContentFault::InvalidRegion)?;
        let start = u32::from(region.start.offset);
        let end = start
            .checked_add(length)
            .filter(|end| *end <= 65536)
            .ok_or(ContentFault::InvalidRegion)?;
        if length == 0 {
            return Err(ContentFault::InvalidRegion);
        }
        if occupied.iter().any(|&(bank, other_start, other_end)| {
            bank == region.start.bank && start < other_end && other_start < end
        }) {
            return Err(ContentFault::Overlap);
        }
        occupied.push((region.start.bank, start, end));
    }
    Ok(())
}
