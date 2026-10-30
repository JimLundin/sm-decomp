//! What the menus need from the runtime: named and raw work RAM, SRAM, the
//! cartridge, register writes, sound and brightness, plus the sprite,
//! HDMA, display-refresh and room-entry operations still
//! implemented by the compatibility runtime.
use crate::display::sequence::Registers;
use crate::effects::{DisplayRefresh, SoundEffects};
use crate::ram::{CompatibilityRam, WorkRamAccess};
use crate::word_transfers::LoadedRom;

/// Battery-backed save RAM.
pub trait SaveRam {
    fn word(&self, at: usize) -> u16;
    fn set_word(&self, at: usize, value: u16);
    fn bytes(&self, range: core::ops::Range<usize>) -> Vec<u8>;
    fn write_bytes(&self, at: usize, input: &[u8]);
    fn copy_within(&self, source: core::ops::Range<usize>, destination: usize);
    fn fill(&self, range: core::ops::Range<usize>, value: u8);
}

/// The runtime operations the menus call.
pub trait MenuRuntime {
    fn set_hdma_enabled(&mut self, enabled: bool);
    fn clear_hdma(&mut self);
    /// Queue a tilemap transfer whose source and destination are read from
    /// the host after the size word is written.
    fn queue_native_transfer(
        &mut self,
        cursor: u16,
        size: u16,
        source: &dyn Fn(&Self) -> u16,
        bank: u8,
        destination: &dyn Fn(&Self) -> u16,
    ) -> u16;
    /// Upload `bytes` from `source` to VRAM word address `destination`.
    fn load_vram_words(&mut self, source: u32, destination: u16, bytes: u16);
    fn decompress_to_ram(&mut self, source: u32, destination: usize) -> usize;
    fn file_select_fade(&mut self) -> u8;
    fn boss_byte(&mut self, area: u16) -> u8;
    /// Begin entering the room at the current load station.
    fn enter_load_station(&mut self);
    fn request_reset(&mut self);
    fn flush_sram(&mut self);
    fn invalid_window(&mut self, address: u32);
    /// The original's failed drawing or bounds assertion.
    fn legacy_assertion(&self, drawing: bool) -> !;
    /// The original's unreachable native routine.
    fn unreachable_native(&mut self) -> u16;
    /// Verification observers; the defaults observe nothing.
    fn observe_map_write(&self, _byte: usize, _index: usize, _stage: u8) {}
    fn observe_flags_save(&mut self, _stage: u8) {}
}

pub trait Host:
    CompatibilityRam
    + WorkRamAccess
    + LoadedRom
    + Registers
    + SoundEffects
    + DisplayRefresh
    + crate::sprite_output::SpriteHost
    + crate::cartridge::NativeDiagnostics
    + MenuRuntime
{
    type Sram: SaveRam + ?Sized;
    fn sram(&self) -> &Self::Sram;
    fn pause_menu_tables(&self) -> &super::map_menu::ImportedTables;
    fn equipment_screen_tables(&self) -> &super::equipment_menu::ImportedTables;
    fn save_slot_offsets(&self) -> &Option<super::save_slots::ImportedSlots>;
    fn area_select_palettes(&self) -> &Option<super::menu_tilemap::ImportedAreaPalettes>;
}
