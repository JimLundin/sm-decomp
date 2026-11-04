// Movies do not intentionally select full-u16 queue aliases or truncate the grapple tables.
// The update runs on a lease, as the frame runs it; each case reads the lease's
// projection, then the published physical words once the lease closes.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
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
/// Run the update on a lease over `game`, which `fails` says panics; then
/// assert each `(address, word)` in `expected` on the lease's projection and,
/// once it closes, in physical RAM.
fn check(game: &mut Game, fails: bool, expected: &[(usize, u16)]) {
    {
        let mut lease = Runtime::new(game);
        let update = catch_unwind(AssertUnwindSafe(|| {
            sm_game::grapple_swing::update_tiles(&mut lease)
        }));
        assert_eq!(update.is_err(), fails);
        for &(at, value) in expected {
            assert_eq!(lease.word(at), value, "projected word {at:#x}");
        }
    }
    for &(at, value) in expected {
        assert_eq!(word(game, at), value, "published word {at:#x}");
    }
}
pub fn verify_grapple_tile_alias_prefixes() {
    check(
        &mut fixture(0xc00, 1, 0xdc348),
        false,
        &[(0xcd0, 33), (0xcd9, 0x1111)],
    );
    check(
        &mut fixture(0xc70, 1, 0xdc348),
        false,
        &[(0xd40, 32), (0xd42, 32)],
    );
    check(&mut fixture(0xc6e, 1, 0xdc348), false, &[(0xd3e, 32)]);
    check(
        &mut fixture(0, 0, 0),
        true,
        &[(0xd3e, 5), (0xd40, 0x8200), (0x330, 0)],
    );
    check(
        &mut fixture(0xc00, 0, 0xdc346),
        true,
        &[
            (0xd3e, 5),
            (0xd40, 0x8200),
            (0xcd0, 32),
            (0x330, 0xc07),
            (0xcd7, 128),
        ],
    );
    lease_matches_owned_call();
}
/// The lease's update (`player_drawing::Host for Runtime`), inside its owned
/// player call, stores what the bare update on a lease stores, at an ordinary
/// tail and at each tail whose full-u16 cursor reaches the charge word or the
/// point animation's timer and tiles. Both start from a fixture taken through
/// one lease, so only the update differs.
fn lease_matches_owned_call() {
    for tail in [0, 0xc00, 0xc6e, 0xc70] {
        let mut bare = fixture(tail, 1, 0xdc348);
        drop(Runtime::new(&mut bare));
        sm_game::grapple_swing::update_tiles(&mut Runtime::new(&mut bare));
        drop(Runtime::new(&mut bare));
        let mut owned = fixture(tail, 1, 0xdc348);
        drop(Runtime::new(&mut owned));
        sm_game::player_drawing::Host::update_grapple_tiles(&mut Runtime::new(&mut owned));
        drop(Runtime::new(&mut owned));
        let differs =
            (0..bare.ram().len()).find(|&at| bare.ram().read_byte(at) != owned.ram().read_byte(at));
        assert_eq!(differs, None, "tail {tail:#x}");
    }
}
