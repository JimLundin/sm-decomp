//! Byte-backed views of enemy slots in the original RAM layout.
#![forbid(unsafe_code)]
use crate::game_state::Game;
use crate::types::{EnemyData, Ram7800_Kraid};

/// Copied fields from a live enemy slot; callers retain the index between reads.
pub(crate) struct EnemyState<'a> {
    bytes: &'a [u8],
}

impl EnemyState<'_> {
    fn read(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.bytes[offset], self.bytes[offset + 1]])
    }
    pub(crate) fn parameter_1(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, parameter_1))
    }
    pub(crate) fn parameter_2(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, parameter_2))
    }
    pub(crate) fn enemy_ptr(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, enemy_ptr))
    }
    pub(crate) fn x_pos(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, x_pos))
    }
    pub(crate) fn x_subpos(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, x_subpos))
    }
    pub(crate) fn y_pos(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, y_pos))
    }
    pub(crate) fn y_subpos(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, y_subpos))
    }
    pub(crate) fn properties(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, properties))
    }
    pub(crate) fn palette_index(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, palette_index))
    }
    pub(crate) fn vram_tiles_index(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, vram_tiles_index))
    }
}

pub(crate) struct EnemyCoordinates<'a> {
    bytes: &'a mut [u8],
}

impl Game {
    pub(crate) fn enemy_state(&self, index: u16) -> EnemyState<'_> {
        let start = 0xf78 + usize::from(index);
        EnemyState {
            bytes: &self.g_ram[start..start + core::mem::size_of::<EnemyData>()],
        }
    }
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

/// Live animation instruction fields in the original enemy slot.
pub(crate) struct EnemyInstructions<'a> {
    bytes: &'a mut [u8],
}

impl Game {
    pub(crate) fn enemy_instructions(&mut self, index: u16) -> EnemyInstructions<'_> {
        let start = 0xf78 + usize::from(index);
        EnemyInstructions {
            bytes: &mut self.g_ram[start..start + core::mem::size_of::<EnemyData>()],
        }
    }
}

impl EnemyInstructions<'_> {
    pub(crate) fn set_ai_preinstr(&mut self, value: u16) {
        self.write(core::mem::offset_of!(EnemyData, ai_preinstr), value);
    }
    pub(crate) fn properties(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, properties))
    }
    pub(crate) fn set_properties(&mut self, value: u16) {
        self.write(core::mem::offset_of!(EnemyData, properties), value);
    }
    pub(crate) fn timer(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, timer))
    }
    pub(crate) fn set_timer(&mut self, value: u16) {
        self.write(core::mem::offset_of!(EnemyData, timer), value);
    }
    pub(crate) fn timer_low(&self) -> u8 {
        self.bytes[core::mem::offset_of!(EnemyData, timer)]
    }
    pub(crate) fn set_timer_low(&mut self, value: u8) {
        self.bytes[core::mem::offset_of!(EnemyData, timer)] = value;
    }
    fn read(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.bytes[offset], self.bytes[offset + 1]])
    }
    fn write(&mut self, offset: usize, value: u16) {
        self.bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    pub(crate) fn ai_handler_bits(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, ai_handler_bits))
    }
    pub(crate) fn instruction_timer(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, instruction_timer))
    }
    pub(crate) fn set_instruction_timer(&mut self, value: u16) {
        self.write(core::mem::offset_of!(EnemyData, instruction_timer), value);
    }
    pub(crate) fn current_instruction(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, current_instruction))
    }
    pub(crate) fn set_current_instruction(&mut self, value: u16) {
        self.write(core::mem::offset_of!(EnemyData, current_instruction), value);
    }
    pub(crate) fn spritemap_pointer(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, spritemap_pointer))
    }
    pub(crate) fn set_spritemap_pointer(&mut self, value: u16) {
        self.write(core::mem::offset_of!(EnemyData, spritemap_pointer), value);
    }
    pub(crate) fn extra_properties(&self) -> u16 {
        self.read(core::mem::offset_of!(EnemyData, extra_properties))
    }
    pub(crate) fn set_extra_properties(&mut self, value: u16) {
        self.write(core::mem::offset_of!(EnemyData, extra_properties), value);
    }
    pub(crate) fn bank(&self) -> u8 {
        self.bytes[core::mem::offset_of!(EnemyData, bank)]
    }
}

pub(crate) struct MiniKraidSpitState<'a> {
    bytes: &'a [u8],
}

impl Game {
    pub(crate) fn mini_kraid_spit_state(&self, index: u16) -> MiniKraidSpitState<'_> {
        let start = 0x7800 + usize::from(index);
        MiniKraidSpitState {
            bytes: &self.g_ram[start..start + 6],
        }
    }
}

impl MiniKraidSpitState<'_> {
    fn read(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.bytes[offset], self.bytes[offset + 1]])
    }
    pub(crate) fn x_velocity(&self) -> u16 {
        self.read(core::mem::offset_of!(Ram7800_Kraid, kraid_next))
    }
    pub(crate) fn y_velocity(&self) -> u16 {
        self.read(core::mem::offset_of!(Ram7800_Kraid, field_2))
    }
    pub(crate) fn x_offset(&self) -> u16 {
        self.read(core::mem::offset_of!(Ram7800_Kraid, field_4))
    }
}

/// Captured native enemy fields; the handle survives callbacks without a RAM borrow.
#[derive(Clone, Copy)]
pub(crate) struct EnemySlot(u16);

#[repr(usize)]
pub(crate) enum EnemyField {
    Definition = core::mem::offset_of!(EnemyData, enemy_ptr),
    X = core::mem::offset_of!(EnemyData, x_pos),
    Y = core::mem::offset_of!(EnemyData, y_pos),
    Width = core::mem::offset_of!(EnemyData, x_width),
    Height = core::mem::offset_of!(EnemyData, y_height),
    Health = core::mem::offset_of!(EnemyData, health),
    Properties = core::mem::offset_of!(EnemyData, properties),
    ExtraProperties = core::mem::offset_of!(EnemyData, extra_properties),
    AiHandlers = core::mem::offset_of!(EnemyData, ai_handler_bits),
    Map = core::mem::offset_of!(EnemyData, spritemap_pointer),
    Palette = core::mem::offset_of!(EnemyData, palette_index),
    Tile = core::mem::offset_of!(EnemyData, vram_tiles_index),
    Layer = core::mem::offset_of!(EnemyData, layer),
    Flash = core::mem::offset_of!(EnemyData, flash_timer),
    Frozen = core::mem::offset_of!(EnemyData, frozen_timer),
    Invincibility = core::mem::offset_of!(EnemyData, invincibility_timer),
    Shake = core::mem::offset_of!(EnemyData, shake_timer),
    Frame = core::mem::offset_of!(EnemyData, frame_counter),
}

pub(crate) fn native_read(game: &Game, address: usize) -> u16 {
    u16::from_ne_bytes([game.g_ram[address], game.g_ram[address + 1]])
}
pub(crate) fn native_write(game: &mut Game, address: usize, value: u16) {
    game.g_ram[address..address + 2].copy_from_slice(&value.to_ne_bytes());
}
impl EnemySlot {
    pub(crate) fn new(index: u16) -> Self {
        Self(index)
    }
    pub(crate) fn word(self, field: EnemyField, game: &Game) -> u16 {
        native_read(game, 0xf78 + usize::from(self.0) + field as usize)
    }
    pub(crate) fn set(self, field: EnemyField, value: u16, game: &mut Game) {
        native_write(game, 0xf78 + usize::from(self.0) + field as usize, value);
    }
    pub(crate) fn clear(self, game: &mut Game) {
        let start = 0xf78 + usize::from(self.0);
        game.g_ram[start..start + core::mem::size_of::<EnemyData>()].fill(0);
    }
    pub(crate) fn set_bank(self, value: u8, game: &mut Game) {
        game.g_ram[0xf78 + usize::from(self.0) + core::mem::offset_of!(EnemyData, bank)] = value;
    }
    pub(crate) fn bank(self, game: &Game) -> u8 {
        game.g_ram[0xf78 + usize::from(self.0) + core::mem::offset_of!(EnemyData, bank)]
    }
}
