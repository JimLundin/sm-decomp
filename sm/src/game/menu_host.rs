//! The compatibility runtime as the `sm_game::menus` host.
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::menu_state::SaveSelection;
use sm_game::save_file::SaveRam;
use sm_game::session_access::SessionAccess as _;

/// The RAM address of a sprite output word.
pub(crate) fn sprite_word_address(word: sm_game::sprite_output::SpriteWord) -> usize {
    use sm_game::sprite_output::SpriteWord::*;
    match word {
        EchoX(i) => 0xab0usize.checked_add(2 * i).expect("echo offset"),
        EchoY(i) => 0xab8usize.checked_add(2 * i).expect("echo offset"),
        EchoSpeed(i) => 0xac0usize.checked_add(2 * i).expect("echo offset"),
        TopHalf => 0xac8,
        BottomHalf => 0xaca,
        GrappleEndX => 0xd08,
        GrappleEndY => 0xd0c,
    }
}
/// The layer 1 camera words sprite output reads.
pub(crate) const SPRITE_CAMERA: [usize; 2] = [0x911, 0x915];
/// The RAM address of shot `slot`'s spritemap pointer.
pub(crate) fn projectile_sprite_address(slot: u16) -> usize {
    0xcb8 + 2 * usize::from(slot >> 1)
}

/// The sprite output's diagnostics, which a lease's drawing reports on the
/// game.
impl Game {
    /// The spritemap's invalid ROM access warning.
    pub(crate) fn warn_invalid_sprite_rom(&mut self, address: u32) {
        // SAFETY: a C-string literal and one integer argument, as the format expects.
        unsafe {
            crate::c_library::printf(
                c"RomPtr - Invalid access 0x%x!
"
                .as_ptr(),
                address,
            );
        }
    }
    /// The spritemap's failure, which marks the game failed.
    pub(crate) fn set_sprite_failure(&mut self) {
        self.g_fail = true;
    }
}

impl sm_game::sprite_output::MapHost for Game {
    fn warn_invalid_rom(&mut self, address: u32) {
        self.warn_invalid_sprite_rom(address);
    }
    fn set_failure(&mut self) {
        self.set_sprite_failure();
    }
}

/// The original's `RomPtr_RAM` assertion on a RAM-table address.
pub(crate) fn assert_ram_pointer(address: u16) {
    // SAFETY: the failure path reports through the C assertion binding and does not return.
    unsafe {
        crate::native_assertion! {
            '_c2rust_label: address < 0x2000;
            b"addr < 0x2000\0", b"src/sm_rtl.h\0", 51, b"uint8 *RomPtr_RAM(uint16_t)\0"
        };
    }
}

/// A save on the lease: the projected words and the save selection are its
/// Session's, and SRAM and its slot offsets the game's, which the lease does
/// not hold.
impl sm_game::menus::save_slots::SlotHost for Runtime<'_> {
    type SlotRam = SaveRam;
    fn slot_ram(&self) -> &SaveRam {
        &self.samus_game().g_sram
    }
    fn slot_offsets(&self) -> &Option<sm_game::menus::save_slots::ImportedSlots> {
        &self.samus_game().save_slots
    }
    fn flush_slots(&mut self) {
        // It reads only SRAM.
        unsafe { crate::rtl::rtl_write_sram(self.samus_game()) }
    }
    fn save_selection(&self) -> SaveSelection {
        self.session().menu_screens.save_selection
    }
    /// The store the game's persistent menu commit makes on its lease.
    fn commit_save_selection(&mut self, _before: &SaveSelection, after: SaveSelection) {
        self.session_mut().menu_screens.save_selection = after;
        self.refresh_inputs();
    }
    fn observe_save(&mut self, _stage: u8) {
        #[cfg(sm_verify_dll)]
        {
            let length = self.samus_game().ram().len();
            crate::flags_save_observer::observe_image(
                _stage,
                || (0..length).map(|at| self.byte(at)).collect(),
                &self.samus_game().g_sram,
            );
        }
    }
}

/// The area map leaves on the lease: the flags store on its Session.
impl sm_game::menus::save_slots::MapHost for Runtime<'_> {
    fn store_gameplay_flags(
        &mut self,
        store: impl FnOnce(&mut sm_game::gameplay_flag_state::GameplayFlags),
    ) {
        store(&mut self.session_mut().gameplay_flags);
        self.words_stored();
    }
}

/// A save on Game: SRAM, its slot offsets and the save selection in the menu
/// state.
impl sm_game::menus::save_slots::SlotHost for Game {
    type SlotRam = SaveRam;
    fn slot_ram(&self) -> &SaveRam {
        &self.g_sram
    }
    fn slot_offsets(&self) -> &Option<sm_game::menus::save_slots::ImportedSlots> {
        &self.save_slots
    }
    fn flush_slots(&mut self) {
        // SAFETY: the menus run from the frame loop with the SNES backend installed.
        unsafe { crate::rtl::rtl_write_sram(self) }
    }
    fn save_selection(&self) -> SaveSelection {
        sm_game::menus::state::load(self)
    }
    fn commit_save_selection(&mut self, before: &SaveSelection, after: SaveSelection) {
        use sm_game::menus::state::Access as _;
        self.commit_menu(before, after);
    }
    fn observe_save(&mut self, _stage: u8) {
        #[cfg(sm_verify_dll)]
        crate::flags_save_observer::observe(_stage, self)
    }
}

// The controllers read through their device registers, as the game reads them.
impl Game {
    /// `controller`'s word as the auto-joypad read holds it now, for the oracle
    /// checkpoint's latch.
    #[cfg(any(feature = "frontend", sm_verify_dll))]
    pub(crate) fn sampled_controller_word(
        &mut self,
        controller: sm_game::menus::host::Controller,
    ) -> u16 {
        use sm_game::menus::host::Controller;
        self.read_register_word(match controller {
            Controller::One => crate::rtl::JOY1L,
            Controller::Two => crate::rtl::JOY2L,
        } as u16)
    }
    /// Controller 1's word as auto-joypad read latches it at this vblank: the
    /// device's, or the next recorded frame's for an oracle that reads it in NMI.
    pub(crate) fn latched_controller_word(&mut self) -> u16 {
        crate::oracle_mode::nmi_input()
            .map(u16::reverse_bits)
            .unwrap_or_else(|| self.read_register_word(crate::rtl::JOY1L as u16))
    }
}

impl sm_game::cartridge::NativeDiagnostics for Game {
    fn native_rom_diagnostic(&mut self, address: u32) {
        crate::game::session_runtime::native_rom_diagnostic(address, self)
    }
}

impl sm_game::decompression::DecompressionHost for Game {
    fn decompress_source(&self) -> u32 {
        self.decompress_src
    }
    fn set_decompress_source(&mut self, source: u32) {
        self.decompress_src = source;
    }
    fn report_invalid_rom_address(&mut self, address: u32) {
        crate::rtl::report_invalid_rom_address(address, self)
    }
    fn read_rom_byte(&self, address: u32, offset: usize) -> u8 {
        Game::read_rom_byte(self, address, offset)
    }
}

// The moved menus and screens address these records by layout.
const _: () = assert!(core::mem::size_of::<crate::types::VramWriteEntry>() == 7);
const _: () = assert!(core::mem::offset_of!(crate::types::PairU16, j) == 2);
const _: () = assert!(
    core::mem::offset_of!(crate::types::EnemyData, ai_preinstr)
        == core::mem::offset_of!(sm_game::menus::prelude::EnemyData, ai_preinstr)
);

// The menus address these enemy record fields by offset.
const _: () = assert!(core::mem::offset_of!(crate::types::EnemyData, x_pos) == 2);
const _: () = assert!(core::mem::offset_of!(crate::types::EnemyData, palette_index) == 30);

// The intro's next-page arrow is a word of the menu tilemap at $7E:3000.
const _: () = assert!(
    sm_game::intro_sequence::NEXT_PAGE_ARROW
        == 0x3000 + core::mem::offset_of!(crate::types::Ram3000, menu.menu_tilemap) + 286
);
