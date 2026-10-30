//! The escape critters run in `sm_game::actors::escape_critters`; their
//! table, events and movement come from here.
#![forbid(unsafe_code)]
use crate::game::gameplay_flag_runtime::CheckEventHappened;
use crate::game::imported::{Imported, load};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::rom::RomImage;
use sm_data::escape_etecoons::{ADDRESS, BYTES, EscapeEtecoon, EscapeEtecoons};
pub(crate) use sm_game::actors::escape_critters::*;

// The named bank $B3 addresses are the ones `session_runtime` dispatches on.
const _: () = {
    use sm_game::actors::escape_critters::{function as f, instruction as i};
    const fn at(word: u16) -> u32 {
        0xb3_0000 | word as u32
    }
    assert!(at(f::RUNNING_FOR_ESCAPE) == 11789916 && at(f::STATIONARY) == 11789936);
    assert!(at(f::RUNNING_AROUND_AIMLESSLY) == 11789952);
    assert!(i::ETECOON_GO_TO_IF_LOW_TIDE == 11789637 && i::DACHORA_GO_TO_IF_LOW_TIDE == 11791016);
    assert!(i::DACHORA_GO_TO_IF_CRITTERS_ESCAPED == 11791032);
    assert!(i::ETECOON_X_POSITION_PLUS == 11789840);
    assert!(i::DACHORA_X_POSITION_MINUS_6 == 11791049 && i::DACHORA_X_POSITION_PLUS_6 == 11791063);
};
use {crate::game::actors::api, sm_game::actors::access::ActorKey as Key};

pub(crate) type ImportedTables = Option<Imported<EscapeEtecoons>>;

pub(crate) fn import(image: &RomImage) -> ImportedTables {
    Imported::import(image, ADDRESS, BYTES, EscapeEtecoons::import)
}

impl Host for Game {
    fn event_happened(&mut self, event: u16) -> bool {
        CheckEventHappened(event, self) != 0
    }

    /// Other images and indices run the five original indexed loads.
    fn escape_etecoon(&self, index: isize) -> EscapeEtecoon {
        Imported::read(
            &self.escape_etecoons,
            self,
            |table| table.etecoon(usize::try_from(index).ok()?),
            |rom| EscapeEtecoon {
                x_position: load(rom, 0xb3e718, index),
                y_position: load(rom, 0xb3e71e, index),
                function: load(rom, 0xb3e724, index),
                instruction_list: load(rom, 0xb3e72a, index),
                x_velocity: load(rom, 0xb3e730, index),
            },
        )
    }

    fn move_horizontal_ignoring_slopes(&mut self, k: u16, amount: i32) -> bool {
        api::move_horizontal(
            &mut Runtime::new(self),
            Key::from_legacy(k),
            amount,
            api::SlopePolicy::Ignore,
        ) == api::MovementResult::Blocked
    }

    fn move_vertical(&mut self, k: u16, amount: i32) {
        api::move_vertical(&mut Runtime::new(self), Key::from_legacy(k), amount);
    }
}
