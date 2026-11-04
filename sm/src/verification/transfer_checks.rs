//! Verify-only checks of the lease's transfer port (session-authoritative-tick.md,
//! stage 4b): a DMA the lease starts reads the bytes its Session holds.
use crate::game::session_encoding as encoding;
use crate::game_state::Game;
use sm_game::session::Session;

/// Every work RAM byte in `reads` that the Session projects holds that byte in
/// RAM: the lease published it before the DMA start that reads it.
pub(crate) fn assert_published(state: &Session, game: &Game, reads: &[usize]) {
    let ram = game.ram();
    assert!(
        reads
            .iter()
            .all(|&at| encoding::projected_byte(state, at)
                .is_none_or(|byte| byte == ram.read_byte(at))),
        "a DMA start reads work RAM the lease has not published"
    );
}
