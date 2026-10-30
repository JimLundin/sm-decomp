//! The lease's PlayerPosition seam: each accessor reads its own word from the
//! imported Session, and each store changes only its own word, which the lease
//! exports back to the same RAM address.
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::player_position::PlayerPosition;

type Read = fn(&Runtime<'_>) -> u16;
type Store = fn(&mut Runtime<'_>, u16);

const READS: &[(usize, Read)] = &[
    (0xaf6, |r| r.player_x()),
    (0xafa, |r| r.player_y()),
    (0xaf8, |r| r.player_x_fraction()),
    (0xafc, |r| r.player_y_fraction()),
    (0xb10, |r| r.previous_player_x()),
    (0xb14, |r| r.previous_player_y()),
    (0x18a8, |r| r.invincibility_timer()),
    (0x18aa, |r| r.knockback_timer()),
];

const STORES: &[(usize, Store)] = &[
    (0xaf6, |r, v| r.set_player_x(v)),
    (0xafa, |r, v| r.set_player_y(v)),
    (0xaf8, |r, v| r.set_player_x_fraction(v)),
    (0xb10, |r, v| r.set_previous_player_x(v)),
    (0xb14, |r, v| r.set_previous_player_y(v)),
    (0xb12, |r, v| r.set_previous_player_x_fraction(v)),
    (0xb16, |r, v| r.set_previous_player_y_fraction(v)),
    (0x18a8, |r, v| r.set_invincibility_timer(v)),
    (0x18aa, |r, v| r.set_knockback_timer(v)),
];

fn value(index: usize) -> u16 {
    0x1357u16.wrapping_mul(index as u16 + 1) ^ 0x8001
}

fn word(game: &Game, at: usize) -> u16 {
    u16::from_le_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])
}

pub(crate) fn run_checked() {
    for (index, &(at, read)) in READS.iter().enumerate() {
        let mut game = Game::new();
        let [low, high] = value(index).to_le_bytes();
        game.ram().write_byte(at, low);
        game.ram().write_byte(at + 1, high);
        let runtime = Runtime::new(&mut game);
        assert_eq!(read(&runtime), value(index), "player.lease.read {at:#x}");
    }
    for (index, &(at, store)) in STORES.iter().enumerate() {
        let mut game = Game::new();
        let mut runtime = Runtime::new(&mut game);
        store(&mut runtime, value(index));
        for &(other, read) in READS {
            if other != at {
                assert_eq!(
                    read(&runtime),
                    0,
                    "player.lease.store {at:#x} touched {other:#x}"
                );
            }
        }
        drop(runtime);
        assert_eq!(word(&game, at), value(index), "player.lease.export {at:#x}");
    }
}
