//! Atomic connection from carried movement to Ceres proximity.
use super::{MovementContent, ProductionFault};
use crate::state::owned::MovementPin;
use canonical::model::Model;
use types::Parts;

/// The complete owners; whole-pixel proximity has no second mutable owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovementCeresState {
    /// Complete carried movement.
    pub movement: Model<MovementPin>,
    /// Complete carried door.
    pub ceres: boss_ceres_ridley::CeresDoor,
}
/// External producers used by Ceres, excluding selected movement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CeresContext {
    /// Independently supplied station stage.
    pub stage: boss_ceres_ridley::Stage,
    /// Independently supplied random word.
    pub random: u16,
    /// Independently supplied shake memory.
    pub shake_frames_left: u16,
}
/// Complete outputs of the atomic opportunity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovementCeresOutputs {
    /// Every movement request and receipt.
    pub movement: samus_movement::Outcome,
    /// Complete door run, sound and shake.
    pub ceres: boss_ceres_ridley::Run,
}
pub(super) fn advance(
    state: &mut MovementCeresState,
    content: &MovementContent,
    movement_rules: &samus_movement::Rules,
    ceres_rules: &boss_ceres_ridley::Rules,
    frame: &samus_movement::Frame<'_>,
    world: &dyn types::World,
    neighbours: CeresContext,
) -> Result<MovementCeresOutputs, ProductionFault> {
    let input = boss_ceres_ridley::Inputs {
        her: types::Pair {
            x: state.movement.0.position.x.pixel(),
            y: state.movement.0.position.y.pixel(),
        },
        stage: neighbours.stage,
        random: neighbours.random,
        shake_frames_left: neighbours.shake_frames_left,
    };
    let ceres = super::entries::ceres_advance(&mut state.ceres, &input, ceres_rules);
    let movement =
        super::movement::advance(&mut state.movement.0, content, movement_rules, frame, world)?;
    Ok(MovementCeresOutputs { movement, ceres })
}
