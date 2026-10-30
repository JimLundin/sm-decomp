//! Finite external flag edges; all owned bytes use the actual aggregate codec.
use super::{
    Composed,
    game::{gameplay_flag_host::*, session_access::SessionAccess, session_encoding},
};
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct ExternalBytes {
    door_after: u8,
    event_far: u8,
    boss_far: u8,
}
fn address(column: LegacyFlagColumn, index: isize) -> usize {
    let base: usize = match column {
        LegacyFlagColumn::Events => 0xd820,
        LegacyFlagColumn::Bosses => 0xd828,
        LegacyFlagColumn::RoomObjects => 0xd830,
        LegacyFlagColumn::Items => 0xd870,
        LegacyFlagColumn::Doors => 0xd8b0,
    };
    base.checked_add_signed(index)
        .expect("prepared flag address")
}
impl Composed {
    pub(super) fn flag_byte_at(&self, at: usize) -> u8 {
        session_encoding::projected_byte(&self.world.player, at).unwrap_or_else(|| match at {
            0xd8f0 => self.world.flag_external.door_after,
            0xf81f => self.world.flag_external.event_far,
            0x1d827 => self.world.flag_external.boss_far,
            _ => panic!("unprepared external flag byte {at:x}"),
        })
    }
    pub(super) fn write_flag_byte(&mut self, at: usize, value: u8) {
        if session_encoding::write_projected_byte(&mut self.world.player, at, value) {
            return;
        }
        match at {
            0xd8f0 => self.world.flag_external.door_after = value,
            0xf81f => self.world.flag_external.event_far = value,
            0x1d827 => self.world.flag_external.boss_far = value,
            _ => panic!("unprepared external flag byte {at:x}"),
        }
    }
    fn flag_pair(&self, at: usize) -> [u8; 2] {
        [self.flag_byte_at(at), self.flag_byte_at(at + 1)]
    }
    fn write_flag_pair(&mut self, at: usize, value: [u8; 2]) {
        self.flag_pair(at); // Whole-word preflight before the first byte write.
        self.write_flag_byte(at, value[0]);
        self.write_flag_byte(at + 1, value[1]);
    }
}
impl FlagAliases for Composed {
    fn flag_byte(&self, column: LegacyFlagColumn, index: isize) -> u8 {
        self.flag_byte_at(address(column, index))
    }
    fn set_flag_byte(&mut self, column: LegacyFlagColumn, index: isize, value: u8) {
        self.write_flag_byte(address(column, index), value);
    }
    fn door_adjacent_word_le(&self, index: isize) -> u16 {
        u16::from_le_bytes(self.flag_pair(address(LegacyFlagColumn::Doors, index)))
    }
    fn boss_word(&self, area: u16, order: BossWordOrder) -> u16 {
        let bytes = self.flag_pair(address(LegacyFlagColumn::Bosses, area as isize));
        match order {
            BossWordOrder::Little => u16::from_le_bytes(bytes),
            BossWordOrder::Native => u16::from_ne_bytes(bytes),
        }
    }
    fn set_boss_word_le(&mut self, area: u16, value: u16) {
        self.write_flag_pair(
            address(LegacyFlagColumn::Bosses, area as isize),
            value.to_le_bytes(),
        );
    }
    fn set_boss_word_native(&mut self, area: u16, value: u16) {
        self.write_flag_pair(
            address(LegacyFlagColumn::Bosses, area as isize),
            value.to_ne_bytes(),
        );
    }
}
impl sm_game::progress::FlagHost for Composed {
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
impl Host for Composed {}
