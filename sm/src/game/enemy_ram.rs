//! Byte-backed position and size fields in an enemy's original RAM slot.
use crate::game_state::Game;
use crate::types::EnemyData;

pub(crate) struct EnemyCoordinates<'a> {
    bytes: &'a mut [u8],
}

impl Game {
    pub(crate) fn enemy_coordinates(&mut self, index: u16) -> EnemyCoordinates<'_> {
        let start = 0xf78 + usize::from(index);
        EnemyCoordinates {
            bytes: &mut self.g_ram[start..start + core::mem::size_of::<EnemyData>()],
        }
    }
}

impl EnemyCoordinates<'_> {
    fn read(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.bytes[offset], self.bytes[offset + 1]])
    }
    fn write(&mut self, offset: usize, value: u16) {
        self.bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    pub(crate) fn x_pos(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, x_pos))
    }
    pub(crate) fn set_x_pos(&mut self, value: u16) {
        self.write(core::mem::offset_of!(EnemyData, x_pos), value);
    }
    pub(crate) fn x_subpos(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, x_subpos))
    }
    pub(crate) fn set_x_subpos(&mut self, value: u16) {
        self.write(core::mem::offset_of!(EnemyData, x_subpos), value);
    }
    pub(crate) fn y_pos(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, y_pos))
    }
    pub(crate) fn set_y_pos(&mut self, value: u16) {
        self.write(core::mem::offset_of!(EnemyData, y_pos), value);
    }
    pub(crate) fn y_subpos(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, y_subpos))
    }
    pub(crate) fn set_y_subpos(&mut self, value: u16) {
        self.write(core::mem::offset_of!(EnemyData, y_subpos), value);
    }
    pub(crate) fn x_width(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, x_width))
    }
    pub(crate) fn set_x_width(&mut self, value: u16) {
        self.write(core::mem::offset_of!(EnemyData, x_width), value);
    }
    pub(crate) fn y_height(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, y_height))
    }
    pub(crate) fn set_y_height(&mut self, value: u16) {
        self.write(core::mem::offset_of!(EnemyData, y_height), value);
    }
}
