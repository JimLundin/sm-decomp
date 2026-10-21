use crate::game::sm_91::{SamusFunc_F433, Samus_SetAnimationFrameIfPoseChanged};
use crate::game_state::Game;
use crate::ram_map::{ram_ptr, word};

pub(crate) unsafe fn refresh_pose(game: &mut Game) {
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
}

pub(crate) unsafe fn record_pose_history(game: &mut Game) {
    // Each u16 copy keeps the adjacent direction and movement-type bytes together.
    word!(game.samus_last_different_pose = word!(game.samus_prev_pose));
    *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa22));
    word!(game.samus_prev_pose = word!(game.samus_pose));
    *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
}

pub(crate) unsafe fn finish_pose_change(game: &mut Game) {
    refresh_pose(game);
    record_pose_history(game);
}
