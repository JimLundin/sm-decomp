// Movies do not intentionally select full-u16 queue aliases or truncate the grapple tables.
#![forbid(unsafe_code)]
use crate::{game_state::Game, rom::RomImage};
use std::panic::{AssertUnwindSafe, catch_unwind};
fn put(game: &mut Game, at: usize, value: u16) {
    game.ram().copy_in(at, &value.to_le_bytes());
}
fn word(game: &Game, at: usize) -> u16 {
    u16::from_le_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])
}
fn fixture(tail: u16, timer: u16, length: usize) -> Box<Game> {
    let mut game = Game::new();
    let mut bytes = vec![0; length];
    for (at, value) in [(0xdc342, 0x8000u16), (0xdc344, 0x9000), (0xdc346, 0x1111)] {
        if at + 2 <= bytes.len() {
            bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
    }
    game.loaded_rom = Some(RomImage::from(bytes));
    put(&mut game, 0x330, tail);
    put(&mut game, 0xd3e, timer);
    put(&mut game, 0xd40, 0x8000);
    put(&mut game, 0xcd0, 0);
    put(&mut game, 0xcfa, 0x00ff);
    game
}
pub fn verify_grapple_tile_alias_prefixes() {
    let mut charge = fixture(0xc00, 1, 0xdc348);
    sm_game::grapple_swing::update_tiles(&mut *charge);
    assert_eq!(word(&charge, 0xcd0), 33);
    assert_eq!(word(&charge, 0xcd9), 0x1111);
    let mut tiles = fixture(0xc70, 1, 0xdc348);
    sm_game::grapple_swing::update_tiles(&mut *tiles);
    assert_eq!(word(&tiles, 0xd40), 32);
    assert_eq!(word(&tiles, 0xd42), 32);
    let mut timer = fixture(0xc6e, 1, 0xdc348);
    sm_game::grapple_swing::update_tiles(&mut *timer);
    assert_eq!(word(&timer, 0xd3e), 32);
    let mut early = fixture(0, 0, 0);
    assert!(
        catch_unwind(AssertUnwindSafe(|| sm_game::grapple_swing::update_tiles(
            &mut *early
        )))
        .is_err()
    );
    assert_eq!(word(&early, 0xd3e), 5);
    assert_eq!(word(&early, 0xd40), 0x8200);
    assert_eq!(word(&early, 0x330), 0);
    let mut late = fixture(0xc00, 0, 0xdc346);
    assert!(
        catch_unwind(AssertUnwindSafe(|| sm_game::grapple_swing::update_tiles(
            &mut *late
        )))
        .is_err()
    );
    assert_eq!(word(&late, 0xd3e), 5);
    assert_eq!(word(&late, 0xd40), 0x8200);
    assert_eq!(word(&late, 0xcd0), 32);
    assert_eq!(word(&late, 0x330), 0xc07);
    assert_eq!(word(&late, 0xcd7), 128);
}
