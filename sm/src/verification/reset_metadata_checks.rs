//! Independent full-byte reset projection, registered under enemy_projectiles.
#![forbid(unsafe_code)]
use crate::game_state::Game;
pub(crate) fn check() {
    let mut game = Game::new();
    for (at, byte) in game.g_ram.iter_mut().enumerate() {
        *byte = (at.wrapping_mul(37) ^ (at >> 8)) as u8;
    }
    let mut expected = game.g_ram.to_vec();
    // This is the literal historical 144-word span, including foreign gaps,
    // not merely the three owned 18-word columns.
    expected[0xf380..0xf4a0].fill(0);
    expected[0xf410..0xf434].fill(0xff);
    super::reset_room_metadata(&mut game);
    assert_eq!(game.g_ram.as_slice(), expected.as_slice());
    assert!(!game.g_fail);
}
