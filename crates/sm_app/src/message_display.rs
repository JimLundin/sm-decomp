//! The device's existing row writer consumes the authenticated prepared frame.
#![forbid(unsafe_code)]
pub fn row(game: &sm::game_state::Game, y: usize) -> Option<&[[u8; 4]]> {
    if y >= 224 {
        return None;
    }
    let frame = game.prepared_message_frame()?;
    if frame.pixels.len() != 256 * 224 {
        return None;
    }
    frame.pixels.get(y * 256..(y + 1) * 256)
}
