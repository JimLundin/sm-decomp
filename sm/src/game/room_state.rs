//! Compatibility entry adapters and retained level/graphics backend leaves.
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::word;
use sm_game::decompression::decompress_to_ram;

pub(crate) fn pause_draygon(game: &mut Game) {
    word!(game.IRQHANDLER_NEXT_HANDLER = 4);
}

pub(crate) fn unpause_draygon(game: &mut Game) -> crate::types::CoroutineRet {
    if word!(game.HDMA_OBJECT_CHANNELS_BITMASK[1]) == 8 {
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

pub(crate) use sm_game::room_state::{copy_level_planes, load_sprite_palettes};

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
        self.foreign(sm_game::actors::ceres_ridley::handle_getaway_if_active);
    }
}

#[path = "../verification/room_level_copy.rs"]
mod level_copy_checks;
pub(crate) use level_copy_checks::check_level_copy;

use sm_game::room_state::fx::Initializer;

impl sm_game::room_state::fx::FxHost for Runtime<'_> {
    fn initialize_fx(&mut self, initializer: Initializer) {
        use crate::game::visual_runtime::*;
        use sm_game::{liquid_effects, scrolling_sky};
        self.foreign(|game| match initializer {
            Initializer::Lava => liquid_effects::initialize_lava(game),
            Initializer::Acid => liquid_effects::initialize_acid(game),
            Initializer::Water => liquid_effects::initialize_water(game),
            Initializer::Weather(weather) => initialize_weather(weather, game),
            Initializer::Land => scrolling_sky::initialize_land(game),
            Initializer::BandedSky => scrolling_sky::initialize_banded_sky(game),
            Initializer::FirefleaLighting => initialize_fireflea_lighting(game),
            Initializer::TourianStatue => initialize_tourian_statue(game),
            Initializer::CeresRidley => initialize_ceres_ridley(game),
            Initializer::CeresElevator => initialize_ceres_elevator(game),
            Initializer::Haze => initialize_room_haze(game),
        });
    }
    fn spawn_fx_palette(&mut self, definition: u16) {
        self.foreign(|game| crate::game::palette_effects::spawn(definition, game));
    }
    fn spawn_fx_animated_tiles(&mut self, definition: u16) {
        self.foreign(|game| crate::game::animated_tiles::spawn(definition, game));
    }
}

pub(crate) fn load_fx_entry(index: u16, game: &mut Game) {
    sm_game::room_state::fx::load_entry(&mut Runtime::new(game), index);
}

pub(crate) fn load_fx_header(game: &mut Game) {
    sm_game::room_state::fx::load_header(&mut Runtime::new(game));
}

pub(crate) fn advance_room(game: &mut Game) {
    let script = word!(game.ROOM_MAIN_CODE_PTR);
    if script != 0 {
        sm_game::room_state::execute_main_code(script, &mut Runtime::new(game));
    }
}
