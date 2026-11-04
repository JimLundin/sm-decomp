//! The cinematic game states' host: the frame's first lease, kept open
//! through the title sequence, the intro and the ending.
//!
//! Each port keeps the observations the former Game call left: calls that
//! opened a fresh lease run inside `owned_player_call`, which refreshes them on
//! entry and on return, and typed stores refresh them as the projected stores
//! did.
use crate::game::session_runtime::{Compressed, Runtime};
use sm_game::session_access::SessionAccess as _;

// The graphics loads on the lease: the decompression writes the published
// image and the lease takes back the bytes it produced; the channel-1 DMA reads
// the published lease through the transfer port; the VRAM fill writes device
// registers only.
impl sm_game::display_refresh::GraphicsLoad for Runtime<'_> {
    fn start_graphics_dma(&mut self, source: u32) {
        let assets = if source == 0x9ab200 {
            self.session().world.prepared_room_assets()
        } else {
            None
        };
        self.owned_player_call(|runtime| {
            use crate::game::display_commands::Wire as _;
            runtime.publish();
            crate::game::display_commands::Lease(runtime).start_room_dma(2, assets.as_deref());
        });
    }
    /// The address resets as one word or as two bytes; either is VMADDL,
    /// then VMADDH.
    fn fill_vram_low_bytes(&mut self, fill: u8, _word_reset: bool) {
        use crate::rtl::{VMADDH, VMADDL, VMAIN, VMDATAL};
        self.owned_player_call(|runtime| {
            runtime.write_device(VMAIN as u16, 0);
            runtime.write_device(VMADDL as u16, 0);
            runtime.write_device(VMADDH as u16, 0);
            for _ in 0..0x4000 {
                runtime.write_device(VMDATAL as u16, fill);
            }
        });
    }
    fn decompress_graphics(&mut self, source: u32, destination: usize) {
        self.decompress_to_ram(Compressed::Rom(source), destination);
    }
}

impl sm_game::effects::DisplayRefresh for Runtime<'_> {
    fn fade(&mut self, fade_in: bool) {
        sm_game::display_refresh::fade(fade_in, self);
    }
    fn set_blank(&mut self, blank: bool) {
        sm_game::display_refresh::set_blank(blank, self);
    }
}

// The menu state on the lease: the Session's menu, window and screen words, as
// the retained Session's menu commit stores them.
impl sm_game::menus::state::Access for Runtime<'_> {
    fn menu_state(&self) -> sm_game::menus::state::State {
        let session = self.session();
        sm_game::menus::state::State {
            menu: session.menu,
            window: sm_game::fx_encoding::session_window(session),
            screens: session.menu_screens,
        }
    }
    fn commit_menu<C: sm_game::menus::state::Value>(&mut self, before: &C, after: C) {
        if after == *before {
            return;
        }
        let mut state = self.menu_state();
        C::merge(before, &after, &mut state);
        self.store_menu_state(state);
    }
    fn reset_pause_menu(&mut self) {
        let mut state = self.menu_state();
        state.clear_pause_menu();
        self.store_menu_state(state);
    }
}

impl Runtime<'_> {
    fn store_menu_state(&mut self, state: sm_game::menus::state::State) {
        let session = self.session_mut();
        session.menu = state.menu;
        for field in sm_game::menu_window_state::Field::ALL {
            let at = sm_game::menu_window_encoding::address(field);
            if field.get(&state.window) != sm_game::fx_encoding::control_word(session, at) {
                sm_game::fx_encoding::write_control_word(session, at, field.get(&state.window));
            }
        }
        session.menu_screens = state.screens;
        self.refresh_inputs();
    }
}

// The level data's reads: the Session terrain's cells, and the size word
// ($7F:0000), which no Session owner holds, through the lease's projection.
impl sm_game::room_state::LevelReads for Runtime<'_> {
    fn level_size_word(&self) -> u16 {
        self.word(sm_game::ram::words::RAM7_F_START)
    }
    fn cell_level(&self, cell: usize) -> u16 {
        self.session().room.terrain.cells()[cell].level
    }
    fn cell_behavior(&self, cell: usize) -> u8 {
        self.session().room.terrain.cells()[cell].behavior
    }
}

impl sm_game::display::queued_tilemaps::BackgroundBlocks for Runtime<'_> {
    fn background_block(&self, index: usize) -> u16 {
        self.session().queued_tilemaps.background()[index]
    }
}

// The Mode 7 queue ($7E:02D0) and its tail have no Session owner: on a Legacy
// lease they are the RAM bytes Game writes.
impl sm_game::display::transfer::Mode7Queue for Runtime<'_> {
    fn mode7_queue_tail(&self) -> u16 {
        self.word(sm_game::ram::words::MODE7_VRAM_WRITE_QUEUE_TAIL)
    }
    fn set_mode7_queue_tail(&mut self, tail: u16) {
        self.write_word_le(tail, sm_game::ram::words::MODE7_VRAM_WRITE_QUEUE_TAIL);
    }
    fn set_mode7_queue_byte(&mut self, offset: usize, value: u8) {
        self.set_byte(sm_game::display::transfer::MODE7_QUEUE + offset, value);
    }
}

impl sm_game::menus::host::Tables for Runtime<'_> {
    fn intro_palette(&self) -> &Option<sm_game::intro_pages::ImportedIntroPalette> {
        &self.samus_game().intro_palette
    }
    fn visual_tables(&self) -> &sm_game::visual_tables::ImportedTables {
        &self.samus_game().visual_tables
    }
}

impl sm_game::menus::host::CommonRuntime for Runtime<'_> {
    fn spawn_hdma(&mut self, bank: u8, args: &crate::types::HdmaObjectDefinition) -> u16 {
        sm_game::hdma_objects::spawn_hdma_owned(bank, args, self)
    }
    fn rtl_unreachable(&mut self) -> bool {
        crate::rtl::unreachable()
    }
    fn assert_ram_pointer(&mut self, address: u16) {
        crate::game::menu_host::assert_ram_pointer(address);
    }
    fn update_beam_tiles_and_palette(&mut self) {
        self.owned_player_call(sm_game::player_drawing::update_beam_tiles_and_palette);
    }
}

impl sm_game::menus::host::CinematicRuntime for Runtime<'_> {
    fn cinematic_sprite_pre_instruction(
        &mut self,
        pre: u32,
        object: u16,
        cine: &mut sm_game::cinematic_state::Cinematic,
    ) {
        sm_game::cinematic_dispatch::run_sprite_pre_instruction(pre, object, cine, self)
    }
    fn cinematic_sprite_instruction(
        &mut self,
        command: u32,
        object: u16,
        operand: u16,
        cine: &mut sm_game::cinematic_state::Cinematic,
    ) -> u16 {
        sm_game::cinematic_states::run_sprite_instruction(command, object, operand, cine, self)
    }
}

impl sm_game::cinematic_host::Host for Runtime<'_> {
    fn spawn_power_bomb(&mut self) {
        self.owned_player_call(sm_game::explosion_window::spawn_power_bomb);
    }
    fn initialize_player(&mut self) {
        self.owned_player_call(sm_game::player_transitions::initialize);
    }
    fn disable_scanline_irq(&mut self) {
        sm_game::display_refresh::disable_scanline_irq(self);
    }
    fn advance_ending_tile_upload(&mut self, page: u16) {
        use sm_game::display::transfer::MODE7_QUEUE;
        use sm_game::ram::words::MODE7_VRAM_WRITE_QUEUE_TAIL as TAIL;
        let tail = self.word(TAIL);
        let mut write_word = |field: usize, value: u16| {
            let at = MODE7_QUEUE + usize::from(tail) + field;
            let [first, second] = value.to_ne_bytes();
            self.set_byte(at, first);
            self.set_byte(at + 1, second);
        };
        let high_plane = page.wrapping_sub(8) & 0x8000 != 0;
        write_word(0, if high_plane { 192 } else { 128 });
        // Both original tables have 16 pages; bounds failure follows the first store.
        assert!(page < 16);
        let source = page * 0x800;
        write_word(1, source);
        write_word(3, 127);
        write_word(4, 2048);
        write_word(6, source & 0x3fff);
        write_word(8, if high_plane { 128 } else { 0 });
        self.write_word_le(tail.wrapping_add(9), TAIL);
    }
    fn advance_intro_player(&mut self) {
        // Samus's projectiles and her frame run on the state's lease, with the
        // observations refreshed as the fresh lease they opened left them.
        self.owned_player_call(|runtime| {
            crate::game::player_projectiles::prepare(runtime);
            crate::game::session_runtime::player_update_runtime::advance_frame(runtime);
        });
    }
    fn native_assertion(
        &mut self,
        expression: &'static [u8],
        line: u32,
        function: &'static [u8],
    ) -> ! {
        crate::c_library::assertion_failed(expression, b"src/sm_8b.c\0", line, function)
    }
}
