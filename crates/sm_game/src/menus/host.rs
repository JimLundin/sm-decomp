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
pub trait MenuRuntime:
    crate::player_position::PlayerPosition + crate::player_position::PlayerFacts
{
    fn set_hdma_enabled(&mut self, enabled: bool);
    /// Clear the whole HDMA objects flag word, stopping the objects.
    fn clear_hdma_flags(&mut self);
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
    fn file_select_fade(&mut self) -> u8;
    fn boss_byte(&mut self, area: u16) -> u8;
    /// Begin entering the room at the current load station.
    fn enter_load_station(&mut self);
    fn request_reset(&mut self);
    fn spawn_palette_effect(&mut self, definition: u16);
    /// Run bank $8B's cinematic sprite pre-instruction `pre` for `object`.
    fn cinematic_sprite_pre_instruction(&mut self, pre: u32, object: u16);
    /// Run bank $8B's cinematic sprite instruction; returns the next script cursor.
    fn cinematic_sprite_instruction(&mut self, command: u32, object: u16, operand: u16) -> u16;
    /// Append a ROM Mode 7 transfer list to the Mode 7 queue.
    fn enqueue_mode7_rom(&mut self, bank: u8, cursor: u16);
    /// Retire the player projectile in `slot`.
    fn retire_projectile(&mut self, slot: u16);
    /// Write a DMA channel register on the owned backend.
    fn write_dma_register(&mut self, register: u16, value: u8);
    /// Advance the music queue, as the HDMA update did first each frame.
    fn handle_music_queue(&mut self);
    /// Compose this frame's layer blending through the session view.
    fn update_layer_blending(&mut self);
    /// Run an HDMA effect through the session view.
    fn hdma_effect(&mut self, effect: crate::hdma_objects::HdmaEffect, slot: u16);
    /// Gameplay event and boss flags, as the original's checks return them.
    fn event_happened(&mut self, event: u16) -> u16;
    fn set_event_happened(&mut self, event: u16);
    fn boss_bit_for_current_area(&mut self, bit: u16) -> u8;
    /// Spawn the hardcoded PLM `id` at block (`x`, `y`).
    fn spawn_hardcoded_plm(&mut self, x: u8, y: u8, id: u16);
    /// Spawn an enemy projectile that uses the room's graphics; returns its slot.
    fn spawn_room_enemy_projectile(&mut self, definition: u16, parameter: u16) -> u16;
    /// Draw Samus's current frame, then (separately) the player projectiles.
    fn draw_player_frame(&mut self);
    fn draw_projectiles(&mut self);
    /// The original's `RomPtr_RAM` assertion on a RAM-table address.
    fn assert_ram_pointer(&mut self, address: u16);
    fn enable_display_nmi(&mut self, enabled: bool);
    /// Samus's pending pose change, as the intro's demo applies it.
    fn finish_pose_change(&mut self);
    fn demo_start(&mut self);
    fn demo_load(&mut self, mode: u16, input: u16);
    /// The message box DMA length the original kept outside RAM.
    fn message_box_das0l_value(&self) -> u16;
    fn set_message_box_das0l_value(&mut self, value: u16);
    /// Emit the queued dialogue sound effects as Cues.
    fn advance_dialogue_sound_effects(&mut self);
    /// Run the HDMA object handler, which also handles the music queue ($88:84B9).
    fn run_hdma_objects(&mut self);
    /// The carry the HDMA object handler ($88:84B9) is running with: the main game loop's, set
    /// from power-on, except while a message box restores the display, where it is clear.
    fn hdma_handler_carry(&self) -> bool;
    /// Restore Samus's movement sound after a message box.
    fn restore_movement_sound(&mut self);
    /// Wait for the next display refresh; nonzero yields to the frame loop.
    fn display_wait(&mut self, update: bool) -> u8;
    fn set_boss_byte(&mut self, area: u16, value: u8);
    /// The registered native image, if any, without the missing-image panic.
    fn native_image_binding(&self) -> Option<sm_data::cartridge::RomImage>;
    fn spawn_bg3_scroll(&mut self);
    fn spawn_animated_tiles(&mut self, definition: u16);
    fn shake_room_with_sound(&mut self, kind: u16);
    fn has_queued_music(&mut self) -> u8;
    fn queue_music_delayed8(&mut self, track: u16);
    /// Spawn an HDMA object from bank `bank`; returns its slot.
    fn spawn_hdma(
        &mut self,
        bank: u8,
        args: &crate::display::hdma_spawn::HdmaObjectDefinition,
    ) -> u16;
    /// The original's unreachable-state failure.
    fn rtl_unreachable(&mut self) -> bool;
    fn queue_fx_clear(&mut self);
    /// Run Samus's pose command `command`; returns the original's result.
    fn player_command(&mut self, command: u16) -> u16;
    fn update_beam_tiles_and_palette(&mut self);
    fn flush_sram(&mut self);
    fn invalid_window(&mut self, address: u32);
    /// The original's failed drawing or bounds assertion.
    fn legacy_assertion(&self, drawing: bool) -> !;
    /// The original's unreachable native routine.
    fn unreachable_native(&mut self) -> u16;
    /// Verification observers; the defaults observe nothing.
    fn observe_map_write(&self, _byte: usize, _index: usize, _stage: u8) {}
    fn observe_flags_save(&mut self, _stage: u8) {}
    fn observe_selected_continue(&mut self, _load_result: u8) {}
}

pub trait Host:
    CompatibilityRam
    + crate::runtime_access::SessionRam
    + crate::player_position::PlayerPosition
    + crate::hud::HudPlayer
    + WorkRamAccess
    + LoadedRom
    + Registers
    + SoundEffects
    + DisplayRefresh
    + crate::sprite_output::SpriteHost
    + crate::cartridge::NativeDiagnostics
    + crate::decompression::DecompressionHost
    + crate::records::Records
    + MenuRuntime
    + crate::records::Records
{
    type Sram: SaveRam + ?Sized;
    fn sram(&self) -> &Self::Sram;
    fn pause_menu_tables(&self) -> &super::map_menu::ImportedTables;
    fn equipment_screen_tables(&self) -> &super::equipment_menu::ImportedTables;
    fn save_slot_offsets(&self) -> &Option<super::save_slots::ImportedSlots>;
    fn demo_sets(&self) -> &Option<crate::demo_rooms::ImportedSets>;
    fn demo_scenes(&self) -> &Option<crate::demo_rooms::ImportedScenes>;
    fn intro_palette(&self) -> &Option<crate::intro_pages::ImportedIntroPalette>;
    /// The loaded image's little-endian word at LoROM `address` plus `offset`.
    fn read_rom_word(&self, address: u32, offset: usize) -> u16;
    fn visual_tables(&self) -> &crate::visual_tables::ImportedTables;
    fn area_select_palettes(&self) -> &Option<super::menu_tilemap::ImportedAreaPalettes>;
    /// Controller 1's word as auto-joypad read latches it at this vblank: the
    /// live controller, or the next recorded frame's word during a replay.
    fn nmi_controller_word(&mut self) -> u16;
}
