//! Samus's inventory, owned by `Session::inventory`.
//!
//! # Design
//!
//! **Cohort.** The words of [`Inventory`]: equipped and collected items and
//! beams, health and its subunit, the three ammunition supplies and their
//! maximums, the reserve tank's mode, capacity, health and missiles, and the
//! hyper beam flag. Each is the little-endian value of its two RAM bytes, as
//! listed in [`Word::address`].
//!
//! **Encoding boundary.** `session_encoding` projects every byte and imports
//! and exports each word with the rest of the Session.
//!
//! **Consumers.** The menus and the HUD run on the compatibility host outside
//! a lease. They read and store through the host's projected bytes with
//! [`current`], [`get`] and [`set`]: the owned Session's projection while one
//! is installed, the byte image otherwise, with the same bytes and order as
//! the named word accesses they replace. No lease is opened. A consumer that
//! runs on a lease uses `session().inventory` directly.
#![forbid(unsafe_code)]
use crate::ram::CompatibilityRam;

pub use crate::item_pickups::{Inventory, Loadout, Supply};

/// An inventory word, by what it holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    EquippedItems,
    CollectedItems,
    EquippedBeams,
    CollectedBeams,
    ReserveMode,
    Health,
    MaxHealth,
    Missiles,
    MaxMissiles,
    SuperMissiles,
    MaxSuperMissiles,
    PowerBombs,
    MaxPowerBombs,
    MaxReserveHealth,
    ReserveHealth,
    ReserveMissiles,
    SubunitHealth,
    HyperBeam,
}

impl Word {
    /// The word's RAM address.
    pub const fn address(self) -> usize {
        match self {
            Self::EquippedItems => 0x9a2,
            Self::CollectedItems => 0x9a4,
            Self::EquippedBeams => 0x9a6,
            Self::CollectedBeams => 0x9a8,
            Self::ReserveMode => 0x9c0,
            Self::Health => 0x9c2,
            Self::MaxHealth => 0x9c4,
            Self::Missiles => 0x9c6,
            Self::MaxMissiles => 0x9c8,
            Self::SuperMissiles => 0x9ca,
            Self::MaxSuperMissiles => 0x9cc,
            Self::PowerBombs => 0x9ce,
            Self::MaxPowerBombs => 0x9d0,
            Self::MaxReserveHealth => 0x9d4,
            Self::ReserveHealth => 0x9d6,
            Self::ReserveMissiles => 0x9d8,
            Self::SubunitHealth => 0xa4c,
            Self::HyperBeam => 0xa76,
        }
    }

    /// The ammunition supply `index` (0 missiles, 1 super missiles, 2 power
    /// bombs): its current and maximum words.
    pub const fn ammunition(index: usize) -> (Self, Self) {
        [
            (Self::Missiles, Self::MaxMissiles),
            (Self::SuperMissiles, Self::MaxSuperMissiles),
            (Self::PowerBombs, Self::MaxPowerBombs),
        ][index]
    }
}

// The addresses are the compat RAM map's named words.
const _: () = {
    use crate::ram::words as w;
    let pairs = [
        (Word::EquippedItems, w::equipped_items),
        (Word::CollectedItems, w::collected_items),
        (Word::EquippedBeams, w::equipped_beams),
        (Word::CollectedBeams, w::collected_beams),
        (Word::ReserveMode, w::reserve_health_mode),
        (Word::Health, w::samus_health),
        (Word::MaxHealth, w::samus_max_health),
        (Word::Missiles, w::samus_missiles),
        (Word::MaxMissiles, w::samus_max_missiles),
        (Word::SuperMissiles, w::samus_super_missiles),
        (Word::MaxSuperMissiles, w::samus_max_super_missiles),
        (Word::PowerBombs, w::samus_power_bombs),
        (Word::MaxPowerBombs, w::samus_max_power_bombs),
        (Word::MaxReserveHealth, w::samus_max_reserve_health),
        (Word::ReserveHealth, w::samus_reserve_health),
        (Word::ReserveMissiles, w::samus_reserve_missiles),
        (Word::SubunitHealth, w::samus_subunit_health),
        (Word::HyperBeam, w::hyper_beam_flag),
    ];
    let mut index = 0;
    while index < pairs.len() {
        assert!(pairs[index].0.address() == pairs[index].1);
        index += 1;
    }
};

/// The inventory as the host's projection holds it.
pub fn current<H: CompatibilityRam + ?Sized>(h: &H) -> Inventory {
    let word = |word: Word| get(h, word);
    let supply = |current: Word, maximum: Word| Supply {
        current: word(current),
        maximum: word(maximum),
    };
    Inventory {
        equipment: Loadout {
            owned: word(Word::CollectedItems),
            active: word(Word::EquippedItems),
        },
        beams: Loadout {
            owned: word(Word::CollectedBeams),
            active: word(Word::EquippedBeams),
        },
        hyper_beam: word(Word::HyperBeam),
        health: supply(Word::Health, Word::MaxHealth),
        health_fraction: word(Word::SubunitHealth),
        reserve_capacity: word(Word::MaxReserveHealth),
        reserve_mode: word(Word::ReserveMode),
        reserve_health: word(Word::ReserveHealth),
        reserve_missiles: word(Word::ReserveMissiles),
        ammunition: [0, 1, 2].map(|index| {
            let (current, maximum) = Word::ammunition(index);
            supply(current, maximum)
        }),
    }
}

/// `word` as the host's projection holds it, low byte then high.
pub fn get<H: CompatibilityRam + ?Sized>(h: &H, word: Word) -> u16 {
    let at = word.address();
    u16::from_le_bytes([h.compatibility_byte(at), h.compatibility_byte(at + 1)])
}

/// Store `value` to `word`, low byte then high, as the named word store did.
pub fn set<H: CompatibilityRam + ?Sized>(h: &mut H, word: Word, value: u16) {
    let at = word.address();
    for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
        h.compatibility_write_byte(at + lane, byte);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room_entry::world::WorldBinding;
    use crate::session::Session;
    use crate::session_encoding::projected_byte;

    /// A host over a Session's projection, as the compat game's is while one
    /// is installed.
    struct Projection(Session);
    impl CompatibilityRam for Projection {
        fn compatibility_byte(&self, at: usize) -> u8 {
            projected_byte(&self.0, at).expect("inventory byte is projected")
        }
        fn compatibility_write_byte(&mut self, at: usize, value: u8) {
            assert!(crate::session_encoding::write_projected_byte(
                &mut self.0,
                at,
                value
            ));
        }
    }

    #[test]
    fn the_view_reads_and_stores_the_owned_inventory() {
        let mut session = Session::with_world(WorldBinding::canonical_legacy());
        let i = &mut session.inventory;
        i.equipment = Loadout {
            owned: 0x1001,
            active: 0x1002,
        };
        i.beams = Loadout {
            owned: 0x2001,
            active: 0x2002,
        };
        i.hyper_beam = 0x3001;
        i.health = Supply {
            current: 0x4001,
            maximum: 0x4002,
        };
        i.health_fraction = 0x4003;
        i.reserve_capacity = 0x5001;
        i.reserve_mode = 0x5002;
        i.reserve_health = 0x5003;
        i.reserve_missiles = 0x5004;
        i.ammunition = [
            Supply {
                current: 0x6001,
                maximum: 0x6002,
            },
            Supply {
                current: 0x7001,
                maximum: 0x7002,
            },
            Supply {
                current: 0x8001,
                maximum: 0x8002,
            },
        ];
        let expected = session.inventory.clone();
        let mut host = Projection(session);
        assert_eq!(current(&host), expected);

        set(&mut host, Word::ReserveHealth, 0x0123);
        set(&mut host, Word::MaxPowerBombs, 0x0456);
        assert_eq!(host.0.inventory.reserve_health, 0x0123);
        assert_eq!(host.0.inventory.ammunition[2].maximum, 0x0456);
        assert_eq!(get(&host, Word::ReserveHealth), 0x0123);
    }
}
