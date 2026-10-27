//! Independent full-byte reset projection, registered under enemy_projectiles.
#![forbid(unsafe_code)]
use crate::game_state::Game;
pub(crate) fn check() {
    let mut game = Game::new();
    for at in 0..game.ram().len() {
        let value = (at.wrapping_mul(37) ^ (at >> 8)) as u8;
        game.ram().write_byte(at, value);
    }
    let mut expected = (0..game.ram().len())
        .map(|at| game.ram().read_byte(at))
        .collect::<Vec<_>>();
    // This is the literal historical 144-word span, including foreign gaps,
    // not merely the three owned 18-word columns.
    expected[0xf380..0xf4a0].fill(0);
    expected[0xf410..0xf434].fill(0xff);
    super::reset_room_metadata(&mut game);
    assert_eq!(
        (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>()
            .as_slice(),
        expected.as_slice()
    );
    assert!(!game.g_fail);
}
