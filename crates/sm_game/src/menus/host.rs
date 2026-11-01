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

/// Samus's rules the menus and frame phases run, each on the game's own lease.
pub trait PlayerRuntime: crate::runtime_access::RuntimeAccess {
    /// Draw Samus's current frame.
    fn draw_player_frame(&mut self) {
        use crate::lease::Lease as _;
        self.runtime()
            .owned_player_call(crate::player_drawing::draw_player_frame);
    }
    /// Draw the player projectiles.
    fn draw_projectiles(&mut self) {
        crate::player_projectiles::presentation::draw_projectiles(&mut self.runtime());
    }
    /// Restore Samus's movement sound after a message box.
    fn restore_movement_sound(&mut self) {
        crate::player_projectiles::feedback::restore_sound(&mut self.runtime());
    }
    /// Retire the player projectile in `slot`.
    fn retire_projectile(&mut self, slot: u16) {
        crate::player_projectiles::lifecycle::retire(&mut self.runtime(), slot);
    }
    /// Run Samus's pose command `command`; returns the original's result.
    fn player_command(&mut self, command: u16) -> u16 {
        use crate::player_transitions::commands::finish_ceres_arrival;
        crate::player_transitions::command(&mut self.runtime(), command, finish_ceres_arrival)
    }
    /// Samus's pending pose change, as the intro's demo applies it.
    fn finish_pose_change(&mut self) {
        crate::player_transitions::finish_pose_change(&mut self.runtime());
    }
    fn demo_start(&mut self) {
        use crate::session_access::SessionAccess as _;
        crate::player_transitions::demo::start(self.runtime().session_mut());
    }
    fn demo_load(&mut self, mode: u16, input: u16) {
        crate::player_transitions::demo::load(&mut self.runtime(), mode, input);
    }
}
impl<T: crate::runtime_access::RuntimeAccess + ?Sized> PlayerRuntime for T {}

/// The frontend's soft-reset request, as the menus raise it.
pub trait ResetRequest {
    fn request_reset(&mut self);
}

/// The runtime operations the menus call.
pub trait MenuRuntime:
    crate::player_position::PlayerPosition
    + crate::player_position::PlayerFacts
    + PlayerRuntime
    + crate::inventory_domain::InventoryAccess
    + super::state::Access
{
    /// Upload `bytes` from `source` to VRAM word address `destination`.
    fn load_vram_words(&mut self, source: u32, destination: u16, bytes: u16);
    /// Run bank $8B's cinematic sprite pre-instruction `pre` for `object`.
    fn cinematic_sprite_pre_instruction(
        &mut self,
        pre: u32,
        object: u16,
        cine: &mut crate::cinematic_state::Cinematic,
    );
    /// Run bank $8B's cinematic sprite instruction; returns the next script cursor.
    fn cinematic_sprite_instruction(
        &mut self,
        command: u32,
        object: u16,
        operand: u16,
        cine: &mut crate::cinematic_state::Cinematic,
    ) -> u16;
    /// Write a DMA channel register on the owned backend.
    fn write_dma_register(&mut self, register: u16, value: u8);
    /// Advance the music queue, as the HDMA update did first each frame.
    fn handle_music_queue(&mut self);
    /// The original's `RomPtr_RAM` assertion on a RAM-table address.
    fn assert_ram_pointer(&mut self, address: u16);
    /// The message box DMA length the original kept outside RAM.
    fn message_box_das0l_value(&self) -> u16;
    fn set_message_box_das0l_value(&mut self, value: u16);
    /// Emit the queued dialogue sound effects as Cues.
    fn advance_dialogue_sound_effects(&mut self);
    /// Run the HDMA object handler, which also handles the music queue ($88:84B9).
    fn run_hdma_objects(&mut self);
    /// Spawn an HDMA object from bank `bank`; returns its slot.
    fn spawn_hdma(
        &mut self,
        bank: u8,
        args: &crate::display::hdma_spawn::HdmaObjectDefinition,
    ) -> u16;
    /// The original's unreachable-state failure.
    fn rtl_unreachable(&mut self) -> bool;
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
    crate::display::preparation::Access
    + CompatibilityRam
    + crate::runtime_access::SessionRam
    + crate::player_position::PlayerPosition
    + crate::hud::HudPlayer
    + WorkRamAccess
    + LoadedRom
    + Registers
    + SoundEffects
    + DisplayRefresh
    + crate::display_refresh::DisplayOwner
    + crate::hdma_objects::HdmaTable
    + crate::session_owners::AudioHost
    + crate::sprite_output::SpriteHost
    + crate::cartridge::NativeDiagnostics
    + crate::decompression::DecompressionHost
    + crate::records::Records
    + MenuRuntime
    + ResetRequest
    + crate::runtime_access::RuntimeAccess
    + crate::actors::session_slots::SlotAccess
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
