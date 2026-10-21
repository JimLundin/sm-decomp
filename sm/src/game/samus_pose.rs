use crate::game::sm_91::{SamusFunc_F433, Samus_SetAnimationFrameIfPoseChanged};
use crate::game_state::Game;

pub(crate) unsafe fn refresh_pose(game: &mut Game) {
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
}

pub(crate) fn record_pose_history(game: &mut Game) {
    // Each record is a pose word, direction byte and movement-type byte.
    // Save the previous record before replacing it with the current one.
    for (source, destination) in [(0xa20, 0xa24), (0xa1c, 0xa20)] {
        for byte in 0..4 {
            game.g_ram[destination + byte] = game.g_ram[source + byte];
        }
    }
}

pub(crate) unsafe fn finish_pose_change(game: &mut Game) {
    refresh_pose(game);
    record_pose_history(game);
}
