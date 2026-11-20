//! Import of one independently qualified current scene-controller offer.
use game::production::frame::borrowed_gameplay::{
    GameplayDispatchOffer, GameplayRoute, SceneController,
};
use serde::Deserialize;

/// Invalid scene-controller origin or independently qualified opportunity contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameplayReceiptFault {
    /// Malformed source projection.
    Format,
    /// Incomplete actual current WRAM origin or invalid physical operand extent.
    Length,
    /// Unsupported current scene or missing source freshness/owner premises.
    Unsupported,
}
#[derive(Deserialize)]
#[serde(transparent)]
struct ContractFlag(bool);
#[derive(Deserialize)]
struct Projection {
    format: String,
    scene: String,
    gameplay_running: ContractFlag,
    authority: String,
    simulation_haze_result_is_authority: ContractFlag,
    read_after_actual_builder_reset_completion: ContractFlag,
    retain_until_actual_gameplay_dispatch_completion: ContractFlag,
    revalidate_current_dependency_at_actual_dispatch: ContractFlag,
    once_consume_actual_opportunity_identity: ContractFlag,
    broader_scene_modes_admitted: ContractFlag,
    private_wram_offset: usize,
    quantity_width: u8,
    private_current_scene_encoding: u16,
    origin_equal_at_actual_use: ContractFlag,
    preceding_writer_count: u32,
}
impl Projection {
    fn validate(&self) -> Result<(), GameplayReceiptFault> {
        if self.format != "bounded-current-gameplay-dispatch-v1"
            || self.scene != "MainGameplay"
            || !self.gameplay_running.0
            || self.authority != "ActualCurrentSceneControllerOwner"
            || self.simulation_haze_result_is_authority.0
            || !self.read_after_actual_builder_reset_completion.0
            || !self.retain_until_actual_gameplay_dispatch_completion.0
            || !self.revalidate_current_dependency_at_actual_dispatch.0
            || !self.once_consume_actual_opportunity_identity.0
            || self.broader_scene_modes_admitted.0
            || !self.origin_equal_at_actual_use.0
            || self.preceding_writer_count != 0
            || self.quantity_width != 2
        {
            return Err(GameplayReceiptFault::Unsupported);
        }
        Ok(())
    }
    fn current(&self, bytes: &[u8]) -> Result<SceneController, GameplayReceiptFault> {
        if bytes.len() != 131_072 {
            return Err(GameplayReceiptFault::Length);
        }
        let at = self.private_wram_offset;
        let value = u16::from_le_bytes([
            bytes.get(at).copied().ok_or(GameplayReceiptFault::Length)?,
            bytes
                .get(at.wrapping_add(1))
                .copied()
                .ok_or(GameplayReceiptFault::Length)?,
        ]);
        if value != self.private_current_scene_encoding {
            return Err(GameplayReceiptFault::Unsupported);
        }
        Ok(SceneController::MainGameplay)
    }
}

/// Import actual current scene ownership through the independently approved physical operand.
/// Admission pins the descriptor and source freshness proof; source record coordinates
/// are ignored. Haze's simulation result never authorizes this current scene.
/// # Errors
/// Rejects incomplete origins, malformed projections and unqualified current owners.
pub fn current(bytes: &[u8], projection: &str) -> Result<SceneController, GameplayReceiptFault> {
    let receipt: Projection =
        toml::from_str(projection).map_err(|_| GameplayReceiptFault::Format)?;
    receipt.validate()?;
    receipt.current(bytes)
}

/// Retain an independently qualified current scene offer unread until actual dispatch.
/// Builder reset must complete before the actual read/dispatch; game completion consumes
/// the offer once with actual current scene and completed builder dependencies.
/// # Errors
/// Rejects invalid origin or source ownership/freshness contracts.
pub fn offer(
    bytes: &[u8],
    projection: &str,
) -> Result<GameplayDispatchOffer, GameplayReceiptFault> {
    Ok(GameplayDispatchOffer {
        dependency: current(bytes, projection)?,
        route: GameplayRoute::MainGameplay,
    })
}
