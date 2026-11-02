//! Compatibility entry adapters and retained level/graphics backend leaves.
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::word;
use sm_game::decompression::decompress_to_ram;

pub(crate) fn pause_draygon(game: &mut Game) {
    word!(game.IRQHANDLER_NEXT_HANDLER = 4);
}

pub(crate) fn unpause_draygon(game: &mut Game) -> crate::types::CoroutineRet {
    if u16::from_ne_bytes(
        sm_game::hdma_objects::HdmaTable::hdma_word(
            game,
            sm_game::hdma::Word::Object {
                index: 1,
                field: sm_game::hdma::ObjectWord::Channels,
            },
        )
        .to_le_bytes(),
    ) == 8
    {
        word!(game.IRQHANDLER_NEXT_HANDLER = 12);
    }
    0
}

#[forbid(unsafe_code)]
pub(crate) fn decompress_level(source: u32, game: &mut Game) {
    decompress_to_ram(source, 0x10000, game);
    copy_level_planes(game);
}

impl sm_game::room_state::EnemyGraphicsHost for Game {
    fn rom_record(&mut self, bank: u8, address: u16) -> crate::rom::RomRecord {
        // SAFETY: room loading runs with the backend installed.
        unsafe { crate::rtl::rom_record(bank, address, self) }
    }
}

pub(crate) use sm_game::room_state::copy_level_planes;

/// Retained physical-port palette leaf. Owned room execution uses the Session kernel.
pub(crate) fn load_sprite_palettes(game: &mut Game) {
    sm_game::word_transfers!(game, descending, 16; le(TARGET_PALETTES, 223) <- rom(0x9a8000, 223));
    sm_game::word_transfers!(game, descending, 16; native(0xc3de) <- native(0xc1de));
    sm_game::word_transfers!(game, descending, 16; native(0xc31e) <- rom(0x9afc00, 15));
}

impl sm_game::room_state::MainCodeHost for Runtime<'_> {
    fn command_player(&mut self, command: u16) {
        self.foreign(|game| crate::game::session_runtime::player_command(command, game));
    }
    fn ceres_rotation_step(&mut self, word: isize) -> sm_data::ceres_elevator::RotationStep {
        self.foreign(|game| sm_game::ceres_elevator::step(game, word))
    }
    fn create_explosion(&mut self, x: u16, y: u16, kind: u16) {
        self.foreign(|game| crate::game::sprite_effect_runtime::create_sprite(x, y, kind, 0, game));
    }
    fn queue_sky_rows(&mut self, ocean: bool) {
        use sm_game::scrolling_sky::{queue_land_rows, queue_ocean_rows};
        self.foreign(|game| {
            if ocean {
                queue_ocean_rows(game)
            } else {
                queue_land_rows(game)
            }
        });
    }
    fn spawn_room_projectile(&mut self, definition: u16, parameter: u16) {
        self.foreign(|game| {
            crate::game::enemy_projectiles::spawn_with_room_gfx(definition, parameter, game);
        });
    }
    fn handle_ceres_getaway(&mut self) {
        sm_game::actors::ceres_ridley::handle_getaway_if_active(self);
    }
}

#[path = "../verification/room_level_copy.rs"]
mod level_copy_checks;
pub(crate) use level_copy_checks::check_level_copy;

use sm_game::room_state::fx::Initializer;

impl sm_game::room_state::fx::FxHost for Runtime<'_> {
    fn initialize_fx(&mut self, initializer: Initializer) {
        self.owned_call(|runtime| {
            sm_game::owned_hdma_update::lifecycle::FxHost::room_initialize_fx(runtime, initializer);
        });
    }
    fn spawn_fx_palette(&mut self, definition: u16) {
        self.foreign(|game| crate::game::palette_effects::spawn(definition, game));
    }
    fn spawn_fx_animated_tiles(&mut self, definition: u16) {
        self.foreign(|game| crate::game::animated_tiles::spawn(definition, game));
    }
}

pub(crate) fn advance_room(game: &mut Game) {
    let script = word!(game.ROOM_MAIN_CODE_PTR);
    if script != 0 {
        sm_game::room_state::execute_main_code(script, &mut Runtime::new(game));
    }
}

impl sm_game::owned_hdma_update::lifecycle::FxHost for Runtime<'_> {}
impl sm_game::owned_hdma_update::lifecycle::ResetHost for Runtime<'_> {
    fn room_mode7(&mut self, value: u16) {
        sm_game::actors::session_slots::SlotAccess::set_slot_word(self, 0x783, value);
    }
    fn room_register(&mut self, register: u16, value: u8) {
        sm_game::owned_hdma_update::context::Context::hdma_register(self, register, value);
    }
}
