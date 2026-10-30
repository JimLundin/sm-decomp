//! Visual adapters on the one aggregate Session, with explicit legacy effects.
pub(crate) use crate::game::cinematic::{
    advance_cinematic_fade, clear_cinematic_colors, compose_cinematic_palette,
    fade_cinematic_colors, prepare_cinematic_palette_fade, queue_cinematic_tilemap,
    set_cinematic_blend, set_cinematic_tilemap_updates,
};
pub(crate) use crate::game::cinematic_display::{
    configure_cinematic_display, draw_intro_sprites, reset_cinematic_transform,
};
pub(crate) use crate::game::cinematic_objects::{
    CinematicObjects, advance_cinematic_background_pre, advance_intro_mother_brain,
    draw_cinematic_sprites, execute_cinematic_sprite_instruction, spawn_cinematic_burst,
    spawn_cinematic_sprite_at,
};
pub(crate) use crate::game::credits::{advance_credits, prepare_credit_roll};
pub(crate) use crate::game::ending_graphics::advance_ending_logo_palette;
pub(crate) use crate::game::ending_screens::{
    advance_ending_gunship_spin, advance_ending_samus_jump, advance_ending_sprite_pair,
    advance_postcredits_samus_message, advance_postcredits_samus_reveal, advance_postcredits_zoom,
    fade_ending_gunship_backdrop, move_ending_cloud, update_ending_critters, wait_ending_cloud,
};
pub(crate) use crate::game::hdma_objects::{
    clear_hdma, clear_hdma_channels, initialize_room_effects, set_hdma_enabled, spawn_bg3_scroll,
    spawn_hdma, spawn_hdma_words, update_hdma,
};
pub(crate) use crate::game::intro_objects::{
    advance_ceres_station, advance_intro_baby_pursuit, advance_intro_landing_mosaic,
    advance_intro_rinka, advance_zebes_destruction_zoom, advance_zebes_flight, advance_zebes_stars,
    chase_intro_baby_y, launch_cinematic_sprite, update_cinematic_transform, wait_cinematic_scene,
};
use crate::game::projectile_presentation_ram as sprite_output;
pub(crate) use crate::game::room_visuals::{
    darken_fireflea_room, initialize_bomb_torizo_haze, initialize_ceres_elevator,
    initialize_ceres_ridley, initialize_draygon_layer, initialize_fireflea_lighting,
    initialize_intro_wave, initialize_morph_eye_beam, initialize_phantoon_motion,
    initialize_room_haze, initialize_title_gradient, initialize_tourian_statue, initialize_weather,
    shake_room_with_sound,
};
pub(crate) use crate::game::shooting_star_motion::{
    advance_shooting_stars, initialize_shooting_stars,
};
pub(crate) use crate::game::title_screen::{
    advance_title_pan, advance_title_wait, request_title_skip, start_title_pan,
};
use crate::ram_map::word;
pub(crate) use sm_game::cinematic::{
    advance_cinematic_coordinate, capture_cinematic_palette, clear_cinematic_background,
    clear_cinematic_sprites, drift_cinematic_sprite, enable_cinematic_background,
    fade_cinematic_fixed_color, scaled_cinematic_value, set_cinematic_backdrop, set_fixed_color,
};

use crate::{
    game::{
        animated_tiles, enemy_projectiles, palette_effects, player_projectiles,
        session_runtime::Runtime,
        visual_host::{self, AnimationMember, PaletteMember, Source, StatueProjectile},
        visual_state::{PaletteBuffer, Upload},
    },
    game_state::Game,
    rom::RomRecord,
    types::SpawnHdmaObject_Args,
};

pub(crate) fn advance_crocomire(game: &mut Game) {
    use crate::game::actors::crocomire_reactions::{self as crocomire};
    use crate::game::enemy_layouts::EnemyLayout72 as Crocomire;
    use crate::ram_map::slot;
    static DEFINITION: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0x10, 0x9563);
    if let Some(request) = crocomire::execute_phase(game) {
        crate::game::music::QueueSfx2_Max6(request as u16, game);
        crocomire::prepare_scroll_hdma(game);
        slot!(
            game,
            Crocomire[0].var_1F = spawn_hdma(0xa4, &DEFINITION, game)
        );
    }
    crocomire::samus_collision(game);
    crocomire::update_palette(game);
}

impl visual_host::Content for Runtime<'_> {
    type Record = RomRecord;
    fn record(&mut self, source: Source, bank: u8, address: u16) -> RomRecord {
        match source {
            Source::Loaded => player_projectiles::host::Host::record(self, bank, address),
            Source::Native => player_projectiles::host::Host::native_record(self, bank, address),
        }
    }
    fn legacy_native_record(&mut self, bank: u8, address: u16) -> RomRecord {
        Runtime::legacy_native_record(self, bank, address)
    }
    fn table(&self, source: Source, address: u32) -> RomRecord {
        let image = match source {
            Source::Loaded => self.loaded_image().expect("ROM must be loaded"),
            Source::Native => self.native_image(),
        };
        player_projectiles::content::captured(image, address)
    }
}
fn palette_address(member: PaletteMember, handle: u16) -> usize {
    let base = match member {
        PaletteMember::Definition => 0x1e7d,
        PaletteMember::ColorOffset => 0x1e8d,
        PaletteMember::PreAction => 0x1ead,
        PaletteMember::Program => 0x1ebd,
        PaletteMember::Delay => 0x1ecd,
        PaletteMember::Counter => 0x1edd,
    };
    base + usize::from(handle & !1)
}
fn animation_address(member: AnimationMember, handle: u16) -> usize {
    let base = match member {
        AnimationMember::Definition => 0x1ef5,
        AnimationMember::Counter => 0x1f01,
        AnimationMember::Program => 0x1f0d,
        AnimationMember::Delay => 0x1f19,
        AnimationMember::Source => 0x1f25,
    };
    base + usize::from(handle & !1)
}
impl visual_host::Aliases for Runtime<'_> {
    fn palette_word(&self, member: PaletteMember, handle: u16) -> u16 {
        self.word(palette_address(member, handle))
    }
    fn set_palette_word_slice(&mut self, member: PaletteMember, handle: u16, value: u16) {
        self.write_slice_le(palette_address(member, handle), value);
    }
    fn set_palette_counter_byte(&mut self, handle: u16, value: u8) {
        self.set_byte(0x1edd + usize::from(handle), value);
    }
    fn palette_flag_native(&self, displacement: u16) -> u16 {
        self.native_word(0x1e79 + usize::from(displacement))
    }
    fn animation_word(&self, member: AnimationMember, handle: u16) -> u16 {
        self.word(animation_address(member, handle))
    }
    fn set_animation_word_prefix(&mut self, member: AnimationMember, handle: u16, value: u16) {
        self.write_word_le(value, animation_address(member, handle));
    }
    fn set_animation_counter_byte(&mut self, handle: u16, value: u8) {
        self.set_byte(0x1f01 + usize::from(handle), value);
    }
    fn set_palette_color_slice(&mut self, displacement: u16, value: u16) {
        self.write_slice_le(0xc000 + usize::from(displacement & !1), value);
    }
    fn set_color_prefix(&mut self, buffer: PaletteBuffer, index: usize, value: u16) {
        let base: usize = match buffer {
            PaletteBuffer::Current => 0xc000,
            PaletteBuffer::Target => 0xc200,
        };
        let at = base
            .checked_add(index.checked_mul(2).expect("palette index overflow"))
            .expect("palette address overflow");
        self.write_word_le(value, at);
    }
}
impl sm_game::scene::atmosphere::Host for Runtime<'_> {
    type Error = std::convert::Infallible;
    fn frame_and_kind(&mut self, slot: usize) -> Result<u16, Self::Error> {
        Ok(sm_game::session_access::SessionAccess::atmospheric(self).slots[slot].frame_and_kind)
    }
    fn cursor(&mut self) -> Result<u16, Self::Error> {
        Ok(self.word(0x590))
    }
    fn x(&mut self, slot: usize) -> Result<u16, Self::Error> {
        Ok(sm_game::session_access::SessionAccess::atmospheric(self).slots[slot].x)
    }
    fn camera_x(&mut self) -> Result<u16, Self::Error> {
        Ok(self.word(0x911))
    }
    fn y(&mut self, slot: usize) -> Result<u16, Self::Error> {
        Ok(sm_game::session_access::SessionAccess::atmospheric(self).slots[slot].y)
    }
    fn camera_y(&mut self) -> Result<u16, Self::Error> {
        Ok(self.word(0x915))
    }
    fn write_x(&mut self, entry: usize, value: u8) -> Result<(), Self::Error> {
        self.set_byte(0x370 + entry, value);
        Ok(())
    }
    fn write_y(&mut self, entry: usize, value: u8) -> Result<(), Self::Error> {
        self.set_byte(0x371 + entry, value);
        Ok(())
    }
    fn tile_pointer(&mut self, kind: u16) -> Result<u16, Self::Error> {
        use visual_host::{Content, Record};
        Ok(self
            .table(Source::Loaded, 0x908bff)
            .native_word(2 * usize::from(kind)))
    }
    fn read_tile(&mut self, address: u16) -> Result<u16, Self::Error> {
        use visual_host::Record;
        Ok(self.legacy_native_record(0x90, address).native_word(0))
    }
    fn write_tile(&mut self, entry: usize, value: u16) -> Result<(), Self::Error> {
        self.write_slice_native(0x372 + entry, value);
        Ok(())
    }
    fn publish_cursor(&mut self, value: u16) -> Result<(), Self::Error> {
        self.write_word_le(value, 0x590);
        Ok(())
    }
    fn written(&mut self) -> Result<(), Self::Error> {
        sm_game::session_access::SessionAccess::probe(self, "atmosphere.tile.written");
        Ok(())
    }
}

impl visual_host::Effects for Runtime<'_> {
    #[cfg(sm_verify_dll)]
    fn samus_palette_event(&mut self, kind: &'static str, argument: u16) {
        samus_observer::event(kind, argument);
    }
    #[cfg(sm_verify_dll)]
    fn samus_palette_inject(&mut self, name: &'static str) {
        samus_observer::inject(name, self.session_mut());
    }
    #[cfg(sm_verify_dll)]
    fn palette_publications_observed(&self) -> bool {
        crate::player_transitions_rust_observer::active()
    }
    #[cfg(sm_verify_dll)]
    fn palette_publication(&mut self, kind: u16, at: usize, before: u16, after: u16) {
        crate::player_transitions_rust_observer::publication(kind, at, before, after);
    }
    #[cfg(sm_verify_dll)]
    fn observe_speed_boost_cancel(&mut self, values: [u16; 3]) {
        crate::player_collision_movement_cases::observe_owned_cancel(values);
    }
    fn draw_samus_sprite(&mut self, index: u16, x: u16, y: u16) {
        if self.has_owned_display() {
            let producer = self
                .session()
                .presentation
                .publications()
                .expect("Samus publications")
                .body;
            self.draw_owned_samus(producer, index, (x, y));
            return;
        }
        // Retain the complete native export/action/import interval, including null maps.
        self.foreign(|game| sprite_output::samus(index, (x, y), game, &mut sprite_output::Native));
    }
    fn draw_atmospheric_tile(&mut self, slot: usize, kind: u16) {
        sm_game::scene::atmosphere::draw_tile(self, slot, kind).unwrap();
    }

    fn sound(&mut self, channel: u8, limit: u8, sound: u16) {
        player_projectiles::host::Host::sound(self, channel, limit, sound);
    }
    fn music_delayed8(&mut self, track: u16) {
        if self.has_owned_display() {
            self.owned_call(|runtime| {
                crate::game::music::requests::enqueue(track, 8, true, runtime)
                    .expect("audio enqueue");
            });
            return;
        }
        self.foreign(|game| crate::game::music::QueueMusic_Delayed8(track, game));
    }
    fn resume_movement_sound(&mut self) {
        #[cfg(sm_verify_dll)]
        samus_observer::event("command", 0x1c);
        self.foreign(|game| {
            crate::game::session_runtime::player_command(0x1c, game);
        });
    }
    fn upload(&mut self, request: Upload) {
        self.foreign(|game| animated_tiles::native::upload(request, game));
    }
    fn invalid_palette_instruction(&mut self) {
        self.foreign(|game| {
            crate::rtl::Unreachable(game);
        });
    }
    fn invalid_animation_instruction(&mut self) -> u16 {
        self.foreign(animated_tiles::native::unreachable)
    }
    fn color_written(&mut self, _index: u16, _value: u16) {}
}
impl visual_host::Composition for Runtime<'_> {
    fn spawn_palette(&mut self, definition: u16) {
        palette_effects::execution::spawn(self, definition);
    }
    fn spawn_statue_projectile(&mut self, kind: StatueProjectile, parameter: u16) {
        use crate::game::constants::bank_86::{
            addr_kEproj_TourianStatueEyeGlow, addr_kEproj_TourianStatueSoul,
        };
        let definition = match kind {
            StatueProjectile::EyeGlow => addr_kEproj_TourianStatueEyeGlow,
            StatueProjectile::Soul => addr_kEproj_TourianStatueSoul,
        };
        enemy_projectiles::lifecycle::spawn_room(self, definition as u16, parameter);
    }
}
impl visual_host::Host for Runtime<'_> {}

#[forbid(unsafe_code)]
fn queue_cannon_tiles(runtime: &mut Runtime<'_>, tiles: player_projectiles::cannon::Tiles) {
    use player_projectiles::cannon::Tiles;
    use visual_host::Content;
    let (size, destination) = match tiles {
        Tiles::Beam(_) => (256, 0x6300),
        Tiles::Cannon(_) => (32, 0x61f0),
    };
    let saved = runtime.word(0x330);
    runtime.write_slice_native(0xd0 + usize::from(saved), size);
    // Size publication precedes source acquisition and every source read.
    let source = match tiles {
        Tiles::Beam(index) => runtime
            .table(Source::Loaded, 0x90c3b1)
            .word(usize::from(index) * 2),
        Tiles::Cannon(pointer) => runtime.legacy_native_record(0x90, pointer).word(0),
    };
    runtime.write_slice_native(0xd0 + usize::from(saved.wrapping_add(2)), source);
    runtime.set_byte(0xd0 + usize::from(saved.wrapping_add(4)), 0x9a);
    runtime.write_slice_native(0xd0 + usize::from(saved.wrapping_add(5)), destination);
    runtime.write_word_le(saved.wrapping_add(7), 0x330);
}

#[forbid(unsafe_code)]
fn cannon_tile(runtime: &mut Runtime<'_>, x: u16, y: u16, body_y: u16, character: u16) {
    let saved = runtime.word(0x590);
    let x = x
        .wrapping_add(runtime.session().player.x)
        .wrapping_sub(runtime.word(0x911));
    if x < 256 {
        let entry = 0x370 + usize::from(runtime.word(0x590));
        runtime.set_byte(entry, x as u8);
        let y = y
            .wrapping_add(runtime.session().player.y)
            .wrapping_sub(body_y)
            .wrapping_sub(runtime.word(0x915));
        if y < 256 {
            runtime.set_byte(entry + 1, y as u8);
            runtime.write_slice_native(entry + 2, character);
            runtime.write_word_le(saved.wrapping_add(4), 0x590);
        }
    }
}

fn draw_player_with_cannon(runtime: &mut Runtime<'_>) {
    // Layer order is captured before atmospheric output can alias the mode word.
    let mode = runtime.session().projectiles.controls.cannon_drawing_mode & 0xf;
    crate::game::player_transitions::draw_atmospheric(runtime);
    if mode != 0 && mode != 2 {
        player_projectiles::cannon::draw(runtime, cannon_tile, queue_cannon_tiles);
    }
    runtime.owned_player_call(|runtime| {
        sm_game::samus_drawing::draw_body(runtime, sm_game::scene::samus_body::Body::Normal)
    });
    if mode == 2 {
        player_projectiles::cannon::draw(runtime, cannon_tile, queue_cannon_tiles);
    }
}
pub(crate) fn update_beam_tiles_and_palette(game: &mut Game) {
    player_projectiles::cannon::update_beam(&mut Runtime::new(game), queue_cannon_tiles);
}
#[path = "../verification/animated_runtime_checks.rs"]
mod animation_guard_checks;
#[path = "../verification/palette_runtime_checks.rs"]
mod palette_guard_checks;
pub(crate) fn check_palette_guard(game: &mut Game) -> bool {
    palette_guard_checks::run(game)
}
pub(crate) fn check_animation_guard(game: &mut Game) -> bool {
    animation_guard_checks::run(game)
}
#[path = "../verification/samus_palette_observer.rs"]
pub(crate) mod samus_observer;

impl sm_game::hdma_effects::Host for Runtime<'_> {
    fn byte(&self, at: usize) -> u8 {
        Runtime::byte(self, at)
    }
    fn set_byte(&mut self, at: usize, value: u8) {
        Runtime::set_byte(self, at, value)
    }
    fn write_slice(&mut self, at: usize, bytes: &[u8]) {
        Runtime::write_slice(self, at, bytes)
    }
    fn store(&mut self, at: usize, value: u16, store: sm_game::player_projectiles::access::Store) {
        Runtime::store(self, at, value, store)
    }
    fn word(&self, at: usize) -> u16 {
        Runtime::word(self, at)
    }
    fn native_word(&self, at: usize) -> u16 {
        Runtime::native_word(self, at)
    }
    fn write_slice_le(&mut self, at: usize, value: u16) {
        Runtime::write_slice_le(self, at, value)
    }
    fn write_slice_native(&mut self, at: usize, value: u16) {
        Runtime::write_slice_native(self, at, value)
    }
    fn calculate_xray_window(
        &mut self,
        x: u16,
        y: u16,
        angle: u16,
        width: u16,
        off_screen: bool,
        table: usize,
    ) {
        self.foreign(|game| {
            crate::game::beam_windows::calculate_xray_window(
                x, y, angle, width, off_screen, table, game,
            )
        });
    }
}

pub(crate) fn draw_player_frame(game: &mut Game) {
    Runtime::new(game).owned_player_call(draw_player_frame_owned);
}

pub(crate) fn draw_player_frame_owned(h: &mut Runtime<'_>) {
    use crate::game::player_transitions::state::DrawMode;
    player_projectiles::cannon::update(h);
    h.refresh_player_observations();
    match h.session().player.transitions.draw_handler {
        DrawMode::Normal | DrawMode::Grapple => {
            let mode = h.session().player.transitions.draw_handler;
            if mode == DrawMode::Normal {
                h.foreign(player_projectiles::flare);
            }
            let grapple = h.session().player.grapple;
            if mode == DrawMode::Normal || (grapple.wrapping_add(0x37aa) as i16) >= 0 {
                h.owned_player_call(draw_player_with_cannon);
                h.owned_player_call(|runtime| sm_game::samus_drawing::draw_echoes(runtime));
            } else {
                if matches!(grapple, 0xc703 | 0xc759) {
                    h.foreign(crate::game::grapple_swing::restore_start_positions);
                }
                h.foreign(player_projectiles::grapple_flare);
                h.owned_player_call(draw_player_with_cannon);
                h.foreign(crate::game::grapple_swing::update_tiles);
                if h.session().player.grapple_length != 0 {
                    h.foreign(crate::game::grapple_swing::draw_rope);
                }
            }
        }
        DrawMode::ShinesparkEnd => {
            h.owned_player_call(|runtime| {
                sm_game::samus_drawing::draw_body(runtime, sm_game::scene::samus_body::Body::Normal)
            });
            for index in [2, 0] {
                h.owned_player_call(|runtime| {
                    sm_game::samus_drawing::draw_shinespark_crash_echoes(index, runtime)
                });
            }
        }
        DrawMode::Frozen => {
            h.owned_player_call(|runtime| {
                sm_game::samus_drawing::draw_body(runtime, sm_game::scene::samus_body::Body::Frozen)
            });
        }
        DrawMode::Elevator if h.inputs().frame & 1 == 0 => {
            h.owned_player_call(|runtime| {
                sm_game::samus_drawing::draw_body(runtime, sm_game::scene::samus_body::Body::Frozen)
            });
        }
        DrawMode::Disabled | DrawMode::Empty | DrawMode::Elevator => {}
        DrawMode::Unknown(_) => {
            h.foreign(|game| crate::rtl::Unreachable(game));
        }
    }
    h.foreign(player_projectiles::draw_projectiles);
    h.owned_call(crate::game::session_runtime::finish_player_frame);
}

pub(crate) fn project_cinematic_point(
    point: crate::types::Point16U,
    game: &mut Game,
) -> crate::types::Point16U {
    project_cinematic_point_reads(point, |at| {
        crate::game::session_encoding::read(&|at| game.ram().read_byte(at), at)
    })
}
pub(crate) fn project_cinematic_point_reads(
    point: crate::types::Point16U,
    mut read: impl FnMut(usize) -> u16,
) -> crate::types::Point16U {
    let x = point.x.wrapping_sub(read(0x80));
    let y = read(0x82).wrapping_sub(point.y);
    let projected_x =
        scaled_cinematic_value(x, read(0x78)).wrapping_add(scaled_cinematic_value(read(0x7a), y));
    let projected_y =
        scaled_cinematic_value(read(0x7c), x).wrapping_add(scaled_cinematic_value(read(0x78), y));
    crate::types::Point16U {
        x: projected_x.wrapping_add(read(0x80)),
        y: read(0x82).wrapping_sub(projected_y),
    }
}

/// Position Crocomire's second body part and align its BG2 image with the camera.
pub(crate) fn align_crocomire_background(game: &mut Game) {
    use crate::game::enemy_layouts::EnemyLayout72 as Crocomire;
    use crate::ram_map::slot;
    let primary = i32::from(word!(game.cur_enemy_index));
    let secondary = i32::from(word!(game.cur_enemy_index)) + 64;
    slot!(
        game,
        Crocomire[secondary].base.x_pos = slot!(game, Crocomire[secondary].var_A)
            .wrapping_add(slot!(game, Crocomire[primary].base.x_pos))
    );
    slot!(
        game,
        Crocomire[secondary].base.y_pos = slot!(game, Crocomire[primary].base.y_pos)
    );
    let relative =
        slot!(game, Crocomire[primary].base.x_pos).wrapping_sub(word!(game.layer1_x_pos)) as i16;
    let offset = 51u16.wrapping_sub(relative as u16);
    word!(
        game.reg_BG2HOFS = if (-128..384).contains(&relative) && crate::rtl::abs16(offset) < 284 {
            offset
        } else {
            256
        }
    );
}
