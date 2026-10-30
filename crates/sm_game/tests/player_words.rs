//! PlayerPosition's word table: each PlayerWord reads the Session::player field
//! that its RAM address projects to, and no other.
use sm_game::player_position::PlayerWord;
use sm_game::room_entry::world::WorldBinding;
use sm_game::session::Session;
use sm_game::session_encoding::write_projected_byte;

/// The RAM map's words for Samus's position and damage timers.
const ADDRESSES: [(PlayerWord, usize); 10] = [
    (PlayerWord::X, 0xaf6),
    (PlayerWord::XFraction, 0xaf8),
    (PlayerWord::Y, 0xafa),
    (PlayerWord::YFraction, 0xafc),
    (PlayerWord::PreviousX, 0xb10),
    (PlayerWord::PreviousXFraction, 0xb12),
    (PlayerWord::PreviousY, 0xb14),
    (PlayerWord::PreviousYFraction, 0xb16),
    (PlayerWord::Invincibility, 0x18a8),
    (PlayerWord::Knockback, 0x18aa),
];

#[test]
fn every_player_word_reads_the_field_its_address_projects_to() {
    assert_eq!(
        PlayerWord::ALL.map(PlayerWord::address),
        ADDRESSES.map(|(_, at)| at)
    );
    for (index, &(word, at)) in ADDRESSES.iter().enumerate() {
        assert_eq!(word.address(), at, "{word:?} address");
        let value = 0x1357u16.wrapping_mul(index as u16 + 1) ^ 0x8001;
        let mut session = Session::with_world(WorldBinding::canonical_legacy());
        let [low, high] = value.to_le_bytes();
        assert!(
            write_projected_byte(&mut session, at, low),
            "{at:#x} is projected"
        );
        assert!(
            write_projected_byte(&mut session, at + 1, high),
            "{at:#x} is projected"
        );
        assert_eq!(word.get(&session.player), value, "{word:?} reads {at:#x}");
        for &(other, _) in &ADDRESSES {
            if other != word {
                assert_eq!(
                    other.get(&session.player),
                    0,
                    "storing {at:#x} moved {other:?}"
                );
            }
        }
    }
}
