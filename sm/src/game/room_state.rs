//! Compatibility entry adapters and retained level/graphics backend leaves.
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::word;

/// Draygon's pause hook: the next scanline interrupt handler, on the lease.
pub(crate) fn pause_draygon(game: &mut Runtime<'_>) {
    use sm_game::room_entry::effects::{EntryEffects as _, ExternalField};
    game.write_external(ExternalField::NextIrqHandler, 4);
}

/// Draygon's unpause hook: the next scanline interrupt handler, on the lease,
/// when HDMA object 1 drives channel 3.
pub(crate) fn unpause_draygon(game: &mut Runtime<'_>) -> crate::types::CoroutineRet {
    use sm_game::room_entry::effects::{EntryEffects as _, ExternalField};
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
        game.write_external(ExternalField::NextIrqHandler, 12);
    }
    0
}

/// The room load's level data step, on a fresh lease of `game`, as the room
/// load runs it on the frame's lease.
#[forbid(unsafe_code)]
pub(crate) fn decompress_level(source: u32, game: &mut Game) {
    Runtime::new(game).decompress_level(crate::game::session_runtime::Compressed::Rom(source));
}

// The room's enemy tilesets load on the lease: each record is the loaded
// image's, through the RomPtr boundary below $8000, as `rtl::rom_record` takes
// it, and each copy publishes, then runs on the lease's transfer port.
impl sm_game::room_state::EnemyGraphicsHost for Runtime<'_> {
    fn rom_record(&mut self, bank: u8, address: u16) -> crate::rom::RomRecord {
        self.actor_legacy_record(bank, address)
    }
}
impl sm_game::display::transfer::RoomDma for Runtime<'_> {
    fn copy_to_vram_now(
        &mut self,
        address: u16,
        source: sm_game::display::transfer::Source,
        bytes: u16,
        assets: Option<&crate::game::room_entry::PreparedRoomAssets>,
    ) {
        crate::game::display_commands::copy_to_vram_on_lease(address, source, bytes, assets, self);
    }
}

// The lease reads the cells from its Session terrain.
impl sm_game::room_state::LevelCells for Runtime<'_> {
    fn cell_level_word(&self, cell: usize) -> u16 {
        sm_game::room_state::lease_level_word(self, cell)
    }
}

impl sm_game::room_state::MainCodeHost for Runtime<'_> {
    fn command_player(&mut self, command: u16) {
        self.owned_player_call(|runtime| {
            crate::game::session_runtime::player_command_in(runtime, command)
        });
    }
    fn ceres_rotation_step(&mut self, word: isize) -> sm_data::ceres_elevator::RotationStep {
        self.owned_player_call(|runtime| sm_game::ceres_elevator::step(runtime, word))
    }
    fn create_explosion(&mut self, x: u16, y: u16, kind: u16) {
        self.owned_player_call(|runtime| {
            crate::game::sprite_effect_runtime::create_sprite_in(runtime, x, y, kind, 0)
        });
    }
    fn queue_sky_rows(&mut self, ocean: bool) {
        use sm_game::scrolling_sky::{queue_land_rows, queue_ocean_rows};
        // The rows' words, source and queue are the lease's; the observations are
        // refreshed on entry and on return, as the former foreign call left them.
        self.owned_player_call(|runtime| {
            if ocean {
                queue_ocean_rows(runtime)
            } else {
                queue_land_rows(runtime)
            }
        });
    }
    fn spawn_room_projectile(&mut self, definition: u16, parameter: u16) {
        self.owned_player_call(|runtime| {
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
        // A liquid starts on the Session; its HDMA objects then spawn on the lease.
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
        // The observations are refreshed on entry and on return, as the former
        // foreign call left them.
        self.owned_player_call(|runtime| match initializer {
            Initializer::Lava => liquid_effects::initialize_lava(runtime),
            Initializer::Acid => liquid_effects::initialize_acid(runtime),
            Initializer::Water => liquid_effects::initialize_water(water_background, runtime),
            Initializer::Weather(weather) => initialize_weather(weather, runtime),
            Initializer::Land => scrolling_sky::initialize_land(runtime),
            Initializer::BandedSky => scrolling_sky::initialize_banded_sky(runtime),
            Initializer::FirefleaLighting => initialize_fireflea_lighting(runtime),
            Initializer::TourianStatue => initialize_tourian_statue(runtime),
            Initializer::CeresRidley => initialize_ceres_ridley(runtime),
            Initializer::CeresElevator => initialize_ceres_elevator(runtime),
            Initializer::Haze => initialize_room_haze(runtime),
        });
    }
    fn spawn_fx_palette(&mut self, definition: u16) {
        self.owned_player_call(|runtime| {
            crate::game::palette_effects::execution::spawn(runtime, definition)
        });
    }
    fn spawn_fx_animated_tiles(&mut self, definition: u16) {
        self.owned_player_call(|runtime| {
            crate::game::animated_tiles::execution::spawn(runtime, definition)
        });
    }
}

/// The FX initializers on the lease: its rules run on itself.
impl sm_game::room_state::fx::SetupHost for Runtime<'_> {
    fn boss_defeated(&mut self, mask: u16) -> bool {
        sm_game::progress::boss_defeated(self, mask)
    }
    fn event_happened(&mut self, event: u16) -> bool {
        sm_game::progress::event_happened(self, event)
    }
    fn spawn_hardcoded_plm(&mut self, x: u8, y: u8, header: u16) {
        sm_game::room_mechanisms::creation::spawn_hardcoded_in(self, x, y, header);
    }
    fn spawn_animated_tiles(&mut self, definition: u16) {
        sm_game::animated_tiles::execution::spawn(self, definition);
    }
    fn store_session(&mut self, refresh: bool, store: impl FnOnce(&mut sm_game::session::Session)) {
        use sm_game::session_access::SessionAccess as _;
        store(self.session_mut());
        if refresh {
            self.words_stored();
        }
    }
}

/// Room setup and door ASM on the lease: the event mark and the door's scroll
/// updates run on itself.
impl sm_game::room_state::ScriptHost for Runtime<'_> {
    fn mark_event(&mut self, event: u16) {
        sm_game::progress::set_event(self, event);
    }
    fn apply_door_scrolls(&mut self, address: u32) -> bool {
        sm_game::room_scrolls::apply(address, self)
    }
}

/// The room's main code, on the gameplay pass's lease.
pub(crate) fn advance_room(runtime: &mut Runtime<'_>) {
    use crate::projection::ProjectedBytes;
    let script = word!(runtime.ROOM_MAIN_CODE_PTR);
    if script != 0 {
        sm_game::room_state::execute_main_code(script, runtime);
    }
}
