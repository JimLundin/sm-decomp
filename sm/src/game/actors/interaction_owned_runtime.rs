//! Shared species services on the existing aggregate Runtime's Session.
#![forbid(unsafe_code)]
use super::Runtime;
use sm_game::actors::{
    boss_lease::BossState as LegacyBossState, owned_common_lifecycle::BossState, remaining_owned,
};

impl BossState for Runtime<'_> {
    fn frame(&self) -> u16 {
        LegacyBossState::frame(self)
    }
    fn direction_step(&mut self, phase: u16, amplitude: u16) -> sm_game::geometry::Vector32 {
        LegacyBossState::direction_step(self, phase, amplitude)
    }
    fn actor_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord {
        sm_game::actors::content_records::ContentSource::content_record(self, bank, address)
    }
    fn area_boss_byte(&mut self) -> u8 {
        LegacyBossState::area_boss_byte(self)
    }
}
impl sm_game::actors::boss_fields_owned::State for Runtime<'_> {
    fn exceptional_byte(&self, at: usize) -> u8 {
        self.byte(at)
    }
    fn set_exceptional_byte(&mut self, at: usize, value: u8) {
        self.set_byte(at, value);
    }
}
impl remaining_owned::Core for Runtime<'_> {
    fn remaining_unknown(&mut self) -> ! {
        self.foreign(crate::game::animated_tiles::native::unreachable);
        unreachable!()
    }
    fn remaining_player_command(&mut self, command: u16) -> u16 {
        sm_game::player_transitions::command(
            self,
            command,
            sm_game::player_transitions::commands::finish_ceres_arrival,
        )
    }
    fn remaining_color_alias_byte(&self, target: bool, offset: usize) -> u8 {
        self.byte((if target { 0xc200 } else { 0xc000 }) + offset)
    }
    fn set_remaining_color_alias_byte(&mut self, target: bool, offset: usize, value: u8) {
        self.set_byte((if target { 0xc200 } else { 0xc000 }) + offset, value);
    }
}
impl remaining_owned::magdollite::Host for Runtime<'_> {
    fn magdollite_glow_colors(&self, source: isize) -> [u16; 4] {
        sm_game::actors::magdollite::Host::magdollite_glow_colors(self.game, source)
    }
    fn magdollite_arm(&self, address: u32, index: isize) -> u16 {
        sm_game::actors::magdollite::Host::magdollite_arm(self.game, address, index)
    }
    fn warn_lava_height(&mut self, value: i32) {
        super::warn_lava_height(value);
    }
}
