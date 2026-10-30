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
    advance_crocomire, align_crocomire_background, darken_fireflea_room,
    initialize_bomb_torizo_haze, initialize_ceres_elevator, initialize_ceres_ridley,
    initialize_draygon_layer, initialize_fireflea_lighting, initialize_intro_wave,
    initialize_morph_eye_beam, initialize_phantoon_motion, initialize_room_haze,
    initialize_title_gradient, initialize_tourian_statue, initialize_weather,
    shake_room_with_sound,
};
pub(crate) use crate::game::shooting_star_motion::{
    advance_shooting_stars, initialize_shooting_stars,
};
pub(crate) use crate::game::title_screen::{
    advance_title_pan, advance_title_wait, request_title_skip, start_title_pan,
};
pub(crate) use sm_game::cinematic::{
    advance_cinematic_coordinate, capture_cinematic_palette, clear_cinematic_background,
    clear_cinematic_sprites, drift_cinematic_sprite, enable_cinematic_background,
    fade_cinematic_fixed_color, set_cinematic_backdrop, set_fixed_color,
};

pub(crate) use crate::game::hosts::{draw_player_frame, update_beam_tiles_and_palette};
pub(crate) use crate::game::visual_checks::{
    check_animation_guard, check_palette_guard, samus_observer,
};
use crate::{
    game::{
        animated_tiles, enemy_projectiles, palette_effects, player_projectiles,
        session_runtime::Runtime,
        visual_host::{self, AnimationMember, PaletteMember, Source, StatueProjectile},
        visual_state::{PaletteBuffer, Upload},
    },
    rom::RomRecord,
};

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
        sm_game::scene::atmosphere::draw_tile(
            &mut sm_game::player_transitions::atmosphere::SessionTiles(self),
            slot,
            kind,
        )
        .unwrap();
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

impl sm_game::hdma_effects::Host for Runtime<'_> {
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
