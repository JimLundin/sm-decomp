//! Carried jump history on draws that pause movement.

use canonical::model::Pinned;

/// Saved draw history retains both the pending movement check and the previous draw.
pub struct Pin;

impl Pinned for Pin {
    type State = samus_movement::DrawnJumpHistory;
    const NAME: &'static str = "production-passage-jump-history";
    const SHAPE: u64 = 0x0F29_1EC0_8E67_76A4;
}

pub(super) fn advance(
    state: &mut samus_movement::DrawnJumpHistory,
    sample: Option<samus_movement::JumpButton>,
) {
    if let Some(jump) = sample {
        super::played::samus::passage_jump_draw(&mut state.jump_held, &mut state.last_draw, jump);
    }
}
