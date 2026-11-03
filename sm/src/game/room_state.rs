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

pub(crate) use sm_game::room_state::{copy_level_planes, load_sprite_palettes};

impl sm_game::room_state::MainCodeHost for Runtime<'_> {
    fn command_player(&mut self, command: u16) {
        self.owned_player_call(|runtime| {
            crate::game::session_runtime::player_command_in(runtime, command)
        });
    }
    fn ceres_rotation_step(&mut self, word: isize) -> sm_data::ceres_elevator::RotationStep {
        self.owned_call(|runtime| sm_game::ceres_elevator::step(runtime, word))
    }
    fn create_explosion(&mut self, x: u16, y: u16, kind: u16) {
        self.owned_call(|runtime| {
            crate::game::sprite_effect_runtime::create_sprite_in(runtime, x, y, kind, 0)
        });
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
        self.owned_call(|runtime| {
            sm_game::enemy_projectiles::lifecycle::spawn_room(runtime, definition, parameter);
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
        use crate::game::visual_runtime::*;
        use sm_game::session_access::SessionAccess as _;
        use sm_game::{liquid_effects, scrolling_sky};
        // A liquid starts on the Session; its HDMA objects then spawn on the game.
        let mut water_background = false;
        match initializer {
            Initializer::Lava | Initializer::Acid => {
                liquid_effects::start_hot(&mut self.session_mut().room);
                self.words_stored();
            }
            Initializer::Water => {
                water_background = liquid_effects::start_water(&mut self.session_mut().room);
                self.words_stored();
            }
            _ => {}
        }
        self.foreign(|game| match initializer {
            Initializer::Lava => liquid_effects::initialize_lava(game),
            Initializer::Acid => liquid_effects::initialize_acid(game),
            Initializer::Water => liquid_effects::initialize_water(water_background, game),
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
        self.owned_call(|runtime| {
            crate::game::palette_effects::execution::spawn(runtime, definition)
        });
    }
    fn spawn_fx_animated_tiles(&mut self, definition: u16) {
        self.owned_call(|runtime| {
            crate::game::animated_tiles::execution::spawn(runtime, definition)
        });
    }
}

pub(crate) fn advance_room(game: &mut Game) {
    let script = word!(game.ROOM_MAIN_CODE_PTR);
    if script != 0 {
        sm_game::room_state::execute_main_code(script, &mut Runtime::new(game));
    }
}
