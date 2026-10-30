//! Legacy flag entry adapters over the single aggregate Runtime.
#![forbid(unsafe_code)]
#![allow(non_snake_case)]
use crate::game_state::Game;
use {
    crate::game::{progress_flags as flags, session_runtime::Runtime},
    sm_game::gameplay_flag_host::{self, BossWordOrder, LegacyFlagColumn as Column},
};
fn address(column: Column, index: isize) -> usize {
    let base: usize = match column {
        Column::Events => 0xd820,
        Column::Bosses => 0xd828,
        Column::RoomObjects => 0xd830,
        Column::Items => 0xd870,
        Column::Doors => 0xd8b0,
    };
    base.checked_add_signed(index).unwrap()
}
impl gameplay_flag_host::FlagAliases for Runtime<'_> {
    fn flag_byte(&self, column: Column, index: isize) -> u8 {
        self.byte(address(column, index))
    }
    fn set_flag_byte(&mut self, column: Column, index: isize, value: u8) {
        self.set_byte(address(column, index), value);
    }
    fn door_adjacent_word_le(&self, index: isize) -> u16 {
        self.word(address(Column::Doors, index))
    }
    fn boss_word(&self, area: u16, order: BossWordOrder) -> u16 {
        let at = address(Column::Bosses, area as isize);
        match order {
            BossWordOrder::Little => self.word(at),
            BossWordOrder::Native => self.native_word(at),
        }
    }
    fn set_boss_word_le(&mut self, area: u16, value: u16) {
        self.write_slice_le(address(Column::Bosses, area as isize), value);
    }
    fn set_boss_word_native(&mut self, area: u16, value: u16) {
        self.write_slice_native(address(Column::Bosses, area as isize), value);
    }
}
impl sm_game::progress::FlagHost for Runtime<'_> {
    fn flags(&self) -> &sm_game::gameplay_flag_state::GameplayFlags {
        &self.session().gameplay_flags
    }
    fn flags_mut(&mut self) -> &mut sm_game::gameplay_flag_state::GameplayFlags {
        &mut self.session_mut().gameplay_flags
    }
    fn compatibility(&self) -> &sm_game::gameplay_flag_state::FlagCompatibility {
        &self.session().flag_compatibility
    }
    fn compatibility_mut(&mut self) -> &mut sm_game::gameplay_flag_state::FlagCompatibility {
        &mut self.session_mut().flag_compatibility
    }
    fn area(&self) -> u16 {
        self.session().room.area
    }
}
impl gameplay_flag_host::Host for Runtime<'_> {}
pub(crate) fn PrepareBitAccess(bit: u16, game: &mut Game) -> u16 {
    flags::prepare_flag_mask(
        &mut Runtime::new(game).session_mut().flag_compatibility,
        bit,
    )
}
pub(crate) fn SetBossBitForCurArea(mask: u16, game: &mut Game) {
    flags::mark_boss_defeated(&mut Runtime::new(game), mask);
}
pub(crate) fn CheckBossBitForCurArea(mask: u16, game: &mut Game) -> u8 {
    u8::from(flags::boss_defeated(&mut Runtime::new(game), mask))
}
pub(crate) fn SetEventHappened(event: u16, game: &mut Game) {
    flags::set_event(&mut Runtime::new(game), event);
}
pub(crate) fn ClearEventHappened(event: u16, game: &mut Game) {
    flags::clear_event(&mut Runtime::new(game), event);
}
pub(crate) fn CheckEventHappened(event: u16, game: &mut Game) -> u16 {
    u16::from(flags::event_happened(&mut Runtime::new(game), event))
}
pub(crate) fn boss_byte_at(area: u16, game: &mut Game) -> u8 {
    flags::boss_byte_at(&Runtime::new(game), area)
}
pub(crate) fn set_boss_byte_at(area: u16, value: u8, game: &mut Game) {
    flags::set_boss_byte_at(&mut Runtime::new(game), area, value);
}
pub(crate) fn boss_word_at(area: u16, order: BossWordOrder, game: &mut Game) -> u16 {
    flags::boss_word_at(&Runtime::new(game), area, order)
}
pub(crate) fn set_boss_word_le(area: u16, value: u16, game: &mut Game) {
    flags::set_boss_word_le(&mut Runtime::new(game), area, value);
}
pub(crate) fn set_boss_word_native(area: u16, value: u16, game: &mut Game) {
    flags::set_boss_word_native(&mut Runtime::new(game), area, value);
}
