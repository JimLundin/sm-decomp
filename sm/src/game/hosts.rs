//! Game's implementations of the sm_game host traits whose modules moved.
#![forbid(unsafe_code)]
use crate::game_state::Game;
use sm_game::inventory::{self, Word};
use sm_game::player_position::{PlayerPosition, PlayerWord};

impl sm_game::explosion_window::Host for Game {
    fn initialize_crystal_flash(&mut self) -> bool {
        sm_game::player_movement::forced::initialize_crystal_flash(
            &mut crate::game::session_runtime::Runtime::new(self),
        )
    }
    fn window_row(&self) -> u16 {
        self.k_out
    }
    fn set_window_row(&mut self, row: u16) {
        self.k_out = row;
    }
}

impl sm_game::runtime_access::RuntimeAccess for Game {
    type Runtime<'a> = crate::game::session_runtime::Runtime<'a>;
    fn runtime(&mut self) -> Self::Runtime<'_> {
        crate::game::session_runtime::Runtime::new(self)
    }
    fn runtime_admissible(&self) -> bool {
        self.session_lease_admissible()
    }
}

impl sm_game::hdma_program::LeaseHdma for Game {
    type Lease<'a> = crate::game::session_runtime::Runtime<'a>;
    fn hdma_lease(&mut self) -> Self::Lease<'_> {
        crate::game::session_runtime::Runtime::new(self)
    }
}

impl sm_game::hdma_program::HdmaLease for crate::game::session_runtime::Runtime<'_> {
    type Game = Game;
    fn foreign<T>(&mut self, action: impl FnOnce(&mut Game) -> T) -> T {
        crate::game::session_runtime::Runtime::foreign(self, action)
    }
}

impl sm_game::runtime_access::SessionRam for crate::game::session_runtime::Runtime<'_> {
    fn byte(&self, at: usize) -> u8 {
        crate::game::session_runtime::Runtime::byte(self, at)
    }
    fn set_byte(&mut self, at: usize, value: u8) {
        crate::game::session_runtime::Runtime::set_byte(self, at, value)
    }
    fn write_slice(&mut self, at: usize, bytes: &[u8]) {
        crate::game::session_runtime::Runtime::write_slice(self, at, bytes)
    }
}

impl sm_game::actors::boulder::Host for Game {
    fn enemy_speed32(&self, offset: usize) -> u32 {
        crate::game::enemy_speeds::word32(&self.enemy_speeds32, &self.capture_native_rom(), offset)
    }
}

impl sm_game::actors::deaths::Deaths for Game {
    fn normal_death(&mut self, animation: u16) {
        crate::game::actors::combat::die(
            &mut crate::game::session_runtime::Runtime::new(self),
            crate::game::actors::api::DeathKind::Normal { animation },
        );
    }
    fn scatter_enemy_drop(
        &mut self,
        source: sm_game::enemy_projectiles::drops::DropSource,
        index: u16,
    ) {
        crate::game::enemy_projectiles::scatter_drop(source, index, self);
    }
}

impl sm_game::actors::zebetites::Host for Game {
    fn event_happened(&mut self, event: u16) -> u16 {
        crate::game::gameplay_flag_runtime::event_has_happened(event, self)
    }
    fn set_event(&mut self, event: u16) {
        crate::game::gameplay_flag_runtime::mark_event(event, self);
    }
    fn clear_event(&mut self, event: u16) {
        crate::game::gameplay_flag_runtime::clear_event_happened(event, self);
    }
    fn spawn_population(&mut self, bank: u8, population: u16) -> u16 {
        crate::game::actors::population::spawn_legacy(
            &mut crate::game::session_runtime::Runtime::new(self),
            bank,
            population,
        )
    }
}

/// The compat game reads and stores Samus's words as the named word accesses
/// did: through the projected bytes, low then high.
impl PlayerPosition for Game {
    fn player_word(&self, word: PlayerWord) -> u16 {
        sm_game::projection::word(self, word.address())
    }
    fn set_player_word(&mut self, word: PlayerWord, value: u16) {
        sm_game::projection::set_word(self, word.address(), value)
    }
}

/// A current lease reads Samus's words from `Session::player`, a dirty one the
/// projected word; stores take the lease's byte path (refresh, epoch, alias).
impl PlayerPosition for crate::game::session_runtime::Runtime<'_> {
    fn player_word(&self, word: PlayerWord) -> u16 {
        self.owned_player(|player| word.get(player), word.address())
    }
    fn set_player_word(&mut self, word: PlayerWord, value: u16) {
        self.write_word_le(value, word.address())
    }
}

/// The raw work-RAM byte reads these facts replaced, unchanged.
impl sm_game::player_position::PlayerFacts for Game {
    fn facing(&self) -> u8 {
        self.ram().read_byte(0xa1e)
    }
    fn movement_type(&self) -> u8 {
        self.ram().read_byte(0xa1f)
    }
    fn grapple_angle(&self) -> u8 {
        self.ram().read_byte(0xcfb)
    }
    fn grapple_mirrored_angle(&self) -> u8 {
        self.ram().read_byte(0xcfd)
    }
}

impl sm_game::actors::mother_brain_tubes::Host for Game {
    fn end_screen_flash(&mut self) {
        crate::game::sm_a9::end_screen_flashing(self);
    }

    fn start_earthquake(&mut self, kind: u16) {
        crate::game::mother_brain_visuals::start_earthquake(kind, self);
    }
}

// The phase-3 neck functions are the ones `dispatch_neck_function` dispatched on.
const _: () = {
    use crate::game::sm_a9::*;
    use sm_game::actors::mother_brain_neck::{animation as a, function as f};
    const fn at(word: u16) -> u32 {
        0xa9_0000 | word as u32
    }
    assert!(at(f::NORMAL) == 11125552 && at(f::IDLE) == 11125587);
    assert!(f::IDLE == ADDR_LOCRET_A9C353 as u16);
    assert!(at(f::SETUP_RECOIL_RECOVERY) == 11125588 && at(f::RECOIL_RECOVERY) == 11125627);
    assert!(at(f::SETUP_HYPER_BEAM_RECOIL) == 11125653 && at(f::HYPER_BEAM_RECOIL) == 11125709);
    assert!(a::RECOVER == ADDR_K_MOTHER_BRAIN_ILIST_9DBB as u16);
    assert!(a::HYPER_BEAM_HIT == ADDR_K_MOTHER_BRAIN_ILIST_9BE7 as u16);
};

impl sm_game::actors::mother_brain_neck::Host for Game {
    fn sine_times(&mut self, length: u16, angle: u16) -> u16 {
        crate::game::sm_a9::sin_times(length, angle, self)
    }

    fn cosine_times(&mut self, length: u16, angle: u16) -> u16 {
        crate::game::sm_a9::cos_times(length, angle, self)
    }
}

impl sm_game::actors::phantoon::Host for Game {
    fn run_function(&mut self, address: u32) {
        crate::game::session_runtime::run_a7_function(address, self);
    }

    fn direction_to_player(&mut self) -> sm_game::actors::api::PlayerDirection {
        crate::game::actors::api::direction_to_player(
            &mut crate::game::session_runtime::Runtime::new(self),
        )
    }

    fn spawn_hdma(&mut self, bank: u8, args: &sm_game::display::hdma_spawn::HdmaObjectDefinition) {
        crate::game::visual_runtime::spawn_hdma(bank, args, self);
    }

    fn start_wave(&mut self, direction: u16, speed: u16) {
        crate::game::visual_runtime::initialize_phantoon_motion(direction, speed, self);
    }
}

impl sm_game::actors::kraid::Host for Game {
    fn queue_kraid_tilemap(&mut self, tail: u16, source: u16) {
        // The two fixed tail reads have no intervening effects and select the same entry.
        crate::game::display_refresh::queue_native_transfer(
            tail,
            704,
            |_| source,
            0xa7,
            |game| ((game.ram().read_byte(0x59) as i32 & 0xfc) << 8) as u16,
            self,
        );
    }

    fn touches_player_from_above(&mut self, k: u16) -> bool {
        crate::game::actors::api::touches_player_from_above(
            &crate::game::session_runtime::Runtime::new(self),
            sm_game::actors::access::ActorKey::from_legacy(k),
        )
    }

    fn boss_word_native(&mut self, area: u16) -> u16 {
        crate::game::gameplay_flag_runtime::boss_word_at(
            area,
            sm_game::gameplay_flag_host::BossWordOrder::Native,
            self,
        )
    }

    fn set_boss_word_native(&mut self, area: u16, value: u16) {
        crate::game::gameplay_flag_runtime::set_boss_word_native(area, value, self);
    }

    fn set_shot_ai(&mut self, enemy_ptr: u16, shot_ai: u16) {
        crate::game::session_runtime::set_kraid_shot_ai(enemy_ptr, shot_ai, self);
    }

    #[cfg(sm_verify_dll)]
    fn observe_kraid_flags(&mut self, step: u8) {
        crate::kraid_flags_observer::observe(step, self);
    }
}

impl sm_game::actors::etecoon::Host for Game {
    fn run_function(&mut self, address: u32) {
        crate::game::session_runtime::run_a7_function(address, self);
    }

    fn probe_horizontal(&mut self, k: u16, amount: i32) -> bool {
        crate::game::actors::api::probe_horizontal(
            &mut crate::game::session_runtime::Runtime::new(self),
            sm_game::actors::access::ActorKey::from_legacy(k),
            amount,
        ) == crate::game::actors::api::MovementResult::Blocked
    }
}

impl sm_game::actors::dachora_run::Host for Game {
    fn run_function(&mut self, address: u32) {
        crate::game::session_runtime::run_a7_function(address, self);
    }
}

impl sm_game::actors::ridley::Host for Game {
    fn run_function(&mut self, function: u16) {
        sm_game::actors::ceres_ridley::run_function(self, function);
    }

    fn spawn_population(&mut self, bank: u8, population: u16) -> u16 {
        crate::game::actors::population::spawn_legacy(
            &mut crate::game::session_runtime::Runtime::new(self),
            bank,
            population,
        )
    }

    fn set_boss_bit(&mut self, mask: u16) {
        crate::game::gameplay_flag_runtime::set_boss_bits_for_current_area(mask, self);
    }

    fn hardware_sine(&mut self, amplitude: u16, angle: u16) -> u16 {
        crate::game::sm_a9::sin_times(amplitude, angle, self)
    }

    fn hardware_cosine(&mut self, amplitude: u16, angle: u16) -> u16 {
        crate::game::sm_a9::cos_times(amplitude, angle, self)
    }

    fn move_liquid(&mut self, target: u16, velocity: u16, delay: u16) {
        let mut runtime = crate::game::session_runtime::Runtime::new(self);
        let room = &mut runtime.session_mut().room;
        room.liquid_target = target;
        room.liquid_velocity = velocity;
        room.liquid_delay = delay;
    }
}

impl sm_game::actors::ceres_ridley::Host for Game {
    fn boss_byte(&mut self, area: u16) -> u8 {
        crate::game::gameplay_flag_runtime::boss_byte_at(area, self)
    }

    fn queue_mode7_transfer(&mut self, bank: u8, list: u16) {
        crate::game::mode7_transfers::enqueue_rom(bank, list, self);
    }

    fn begin_ceres_push(&mut self) {
        sm_game::player_movement::forced::begin_ceres_push(
            crate::game::session_runtime::Runtime::new(self).session_mut(),
        );
    }

    fn queue_escape_timer_tiles(&mut self) -> bool {
        crate::game::rom_vram_entries::queue_escape_timer_tiles(self) & 1 != 0
    }

    fn queue_getaway_tiles(&mut self) {
        crate::game::rom_vram_entries::queue_getaway_tiles(self);
    }

    fn queue_escape_tile_table(&mut self) {
        crate::game::rom_vram_entries::queue_escape_tile_table(self);
    }
}

impl sm_game::actors::gunship::Host for Game {
    fn spawn_palette_effect(&mut self, definition: u16) {
        crate::game::palette_effects::spawn(definition, self);
    }
    fn face_player_forward(&mut self) {
        crate::game::session_runtime::player_face_forward(self);
    }
    fn event_happened(&mut self, event: u16) -> bool {
        crate::game::gameplay_flag_runtime::event_has_happened(event, self) != 0
    }
    fn restore(&mut self, resource: sm_game::item_pickups::Resource, amount: u16) {
        crate::game::item_pickups_runtime::restore(resource, amount, self);
    }
    fn message_box_response(&mut self, kind: u16) -> i32 {
        sm_game::message_boxes::poll(kind, self)
    }
    fn save_game(&mut self, slot: u16) {
        sm_game::menus::save_slots::save(slot, self);
    }
}

impl sm_game::display::transfer::TilemapQueue for crate::game::session_runtime::Runtime<'_> {
    fn tilemap_queue_tail(&self) -> u16 {
        self.word(crate::ram_map::words::VRAM_WRITE_QUEUE_TAIL)
    }
    fn set_tilemap_queue_tail(&mut self, tail: u16) {
        self.write_slice_le(crate::ram_map::words::VRAM_WRITE_QUEUE_TAIL, tail);
    }
    fn set_tilemap_queue_byte(&mut self, at: usize, value: u8) {
        self.set_byte(at, value);
    }
}

impl sm_game::escape_typewriter::Host for Game {
    fn colour(&self, index: usize) -> u16 {
        crate::ram_map::word!(self.PALETTE_BUFFER[index])
    }
    fn set_colour(&mut self, index: usize, value: u16) {
        crate::ram_map::word!(self.PALETTE_BUFFER[index] = value);
    }
    fn area(&self) -> u16 {
        crate::ram_map::word!(self.AREA_INDEX)
    }
}

impl sm_game::display::transfer::TilemapQueue for Game {
    fn tilemap_queue_tail(&self) -> u16 {
        crate::ram_map::word!(self.VRAM_WRITE_QUEUE_TAIL)
    }
    fn set_tilemap_queue_tail(&mut self, tail: u16) {
        crate::ram_map::word!(self.VRAM_WRITE_QUEUE_TAIL = tail);
    }
    fn set_tilemap_queue_byte(&mut self, at: usize, value: u8) {
        self.ram().write_byte(at, value);
    }
}

impl sm_game::actors::fire_arc::Host for Game {
    fn advance_orbit(&mut self) {
        crate::game::enemy_projectiles::advance_fire_arc_orbit(self);
    }
}

impl sm_game::actors::ceres_doors::Host for Game {
    fn rom_script(&mut self, bank: u8, address: u16) -> sm_game::enemy_script::EnemyScript {
        crate::game::session_runtime::steam_script(bank, address, self)
    }

    fn project_cinematic_point(&mut self, x: u16, y: u16) -> (u16, u16) {
        let point =
            sm_game::cinematic::project_cinematic_point(sm_game::geometry::Point { x, y }, self);
        (point.x, point.y)
    }

    fn queue_mode7_transfer(&mut self, bank: u8, list: u16) {
        crate::game::mode7_transfers::enqueue_rom(bank, list, self);
    }
}

impl sm_game::actors::statues::Host for Game {
    fn run_function(&mut self, address: u32) {
        crate::game::session_runtime::run_aa_function(address, self);
    }

    fn boss_word_native(&mut self, area: u16) -> u16 {
        crate::game::gameplay_flag_runtime::boss_word_at(
            area,
            sm_game::gameplay_flag_host::BossWordOrder::Native,
            self,
        )
    }

    fn start_lowering_acid(&mut self) {
        let mut runtime = crate::game::session_runtime::Runtime::new(self);
        let room = &mut runtime.session_mut().room;
        room.liquid_delay = 32;
        room.liquid_velocity = 64;
    }

    fn lower_acid(&mut self) {
        crate::game::room_mechanisms::lower_acid(self);
    }

    fn select_block_at(&mut self, x: u16, y: u16) {
        crate::game::actors::api::select_block_at(
            &mut crate::game::session_runtime::Runtime::new(self),
            sm_game::geometry::Point { x, y },
        );
    }

    fn spawn_collision_plm(&mut self, header: u16) {
        crate::game::session_runtime::spawn_collision_plm(header, self);
    }
}

impl sm_game::lease::Lease for crate::game::session_runtime::Runtime<'_> {
    fn owned_call<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R {
        crate::game::session_runtime::Runtime::owned_call(self, action)
    }
    fn refresh_player_observations(&mut self) {
        crate::game::session_runtime::Runtime::refresh_player_observations(self)
    }
    fn owned_player_call<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R {
        crate::game::session_runtime::Runtime::owned_player_call(self, action)
    }
    fn has_owned_display(&self) -> bool {
        crate::game::session_runtime::Runtime::has_owned_display(self)
    }
    fn draw_owned_samus(&mut self, index: u16, x: u16, y: u16) {
        let producer = self
            .session()
            .presentation
            .publications()
            .expect("Samus publications")
            .body;
        crate::game::session_runtime::Runtime::draw_owned_samus(self, producer, index, (x, y));
    }
    fn native_samus_sprite(&mut self, index: u16, x: u16, y: u16) {
        use sm_game::sprite_output;
        self.foreign(|game| sprite_output::samus(index, (x, y), game, &mut sprite_output::Native));
    }
    fn native_music_delayed8(&mut self, track: u16) {
        self.foreign(|game| crate::game::music::queue_music_delayed_8(track, game));
    }
    fn native_player_command(&mut self, command: u16) {
        self.foreign(|game| {
            crate::game::session_runtime::player_command(command, game);
        });
    }
    fn native_upload(&mut self, request: sm_game::visual_state::Upload) {
        self.foreign(|game| crate::game::animated_tiles::native::upload(request, game));
    }
    fn native_unreachable(&mut self) {
        self.foreign(|game| {
            crate::rtl::report_unreachable(game);
        });
    }
    fn native_animation_unreachable(&mut self) -> u16 {
        self.foreign(crate::game::animated_tiles::native::unreachable)
    }
}

impl sm_game::player_drawing::Host for crate::game::session_runtime::Runtime<'_> {
    fn draw_flare(&mut self) {
        self.foreign(crate::game::player_projectiles::flare);
    }
    fn draw_grapple_flare(&mut self) {
        self.foreign(crate::game::player_projectiles::grapple_flare);
    }
    fn restore_grapple_start_positions(&mut self) {
        self.owned_call(|runtime| sm_game::grapple_swing::restore_start_positions(runtime));
    }
    fn update_grapple_tiles(&mut self) {
        self.foreign(sm_game::grapple_swing::update_tiles);
    }
    fn draw_grapple_rope(&mut self) {
        self.owned_player_call(|runtime| sm_game::grapple_rope::draw(runtime));
    }
    fn draw_projectiles(&mut self) {
        self.foreign(crate::game::player_projectiles::draw_projectiles);
    }
    fn finish_player_frame(&mut self) {
        self.owned_call(crate::game::session_runtime::finish_player_frame);
    }
}

/// Queue the active beam's tiles and write its palette on a fresh lease.
pub(crate) fn update_beam_tiles_and_palette(game: &mut Game) {
    sm_game::player_drawing::update_beam_tiles_and_palette(
        &mut crate::game::session_runtime::Runtime::new(game),
    );
}

/// Draw Samus's frame on a fresh lease, inside the player driver's owned call.
pub(crate) fn draw_player_frame(game: &mut Game) {
    crate::game::session_runtime::Runtime::new(game)
        .owned_player_call(sm_game::player_drawing::draw_player_frame);
}

/// The HUD's values on the game: the inventory view over the projected bytes,
/// and the named words for the HUD items and grapple, read the same way.
impl sm_game::hud::HudPlayer for Game {
    fn health(&self) -> u16 {
        inventory::get(self, Word::Health)
    }
    fn max_health(&self) -> u16 {
        inventory::get(self, Word::MaxHealth)
    }
    fn ammunition(&self, index: usize) -> u16 {
        inventory::get(self, Word::ammunition(index).0)
    }
    fn max_ammunition(&self, index: usize) -> u16 {
        inventory::get(self, Word::ammunition(index).1)
    }
    fn reserve_health(&self) -> u16 {
        inventory::get(self, Word::ReserveHealth)
    }
    fn reserve_mode(&self) -> u16 {
        inventory::get(self, Word::ReserveMode)
    }
    fn equipped_items(&self) -> u16 {
        inventory::get(self, Word::EquippedItems)
    }
    fn selected_item(&self) -> u16 {
        let game = self;
        sm_game::ram_word!(game.HUD_ITEM_INDEX)
    }
    fn automatic_item(&self) -> u16 {
        let game = self;
        sm_game::ram_word!(game.SAMUS_AUTO_CANCEL_HUD_ITEM_INDEX)
    }
    fn grapple_function(&self) -> u16 {
        let game = self;
        sm_game::ram_word!(game.GRAPPLE_BEAM_FUNCTION)
    }
}

impl sm_game::hud::HudFrameHost for Game {
    fn raw_frame_counter_byte(&self) -> u8 {
        self.ram()
            .read_byte(sm_game::ram::bytes::NMI_FRAME_COUNTER_BYTE)
    }
}

/// On a lease the HUD reads Samus's values from the owned Session.
impl sm_game::hud::HudPlayer for crate::game::session_runtime::Runtime<'_> {
    fn health(&self) -> u16 {
        self.owned_session(|s| s.inventory.health.current, 0x9c2)
    }
    fn max_health(&self) -> u16 {
        self.owned_session(|s| s.inventory.health.maximum, 0x9c4)
    }
    fn ammunition(&self, index: usize) -> u16 {
        let at = [0x9c6, 0x9ca, 0x9ce][index];
        self.owned_session(|s| s.inventory.ammunition[index].current, at)
    }
    fn max_ammunition(&self, index: usize) -> u16 {
        let at = [0x9c8, 0x9cc, 0x9d0][index];
        self.owned_session(|s| s.inventory.ammunition[index].maximum, at)
    }
    fn reserve_health(&self) -> u16 {
        self.owned_session(|s| s.inventory.reserve_health, 0x9d6)
    }
    fn reserve_mode(&self) -> u16 {
        self.owned_session(|s| s.inventory.reserve_mode, 0x9c0)
    }
    fn equipped_items(&self) -> u16 {
        self.owned_session(|s| s.inventory.equipment.active, 0x9a2)
    }
    fn selected_item(&self) -> u16 {
        self.owned_session(|s| s.projectiles.controls.selected_weapon, 0x9d2)
    }
    fn automatic_item(&self) -> u16 {
        self.owned_session(|s| s.projectiles.controls.automatic_weapon, 0xa04)
    }
    fn grapple_function(&self) -> u16 {
        self.owned_session(|s| s.player.grapple, 0xd32)
    }
}

impl sm_game::hud::HudFrameHost for crate::game::session_runtime::Runtime<'_> {
    fn raw_frame_counter_byte(&self) -> u8 {
        // As on the game: the raw work-RAM byte, not the Display projection.
        self.samus_game()
            .ram()
            .read_byte(sm_game::ram::bytes::NMI_FRAME_COUNTER_BYTE)
    }
}
