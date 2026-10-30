//! Visual adapters on the one aggregate Session, with explicit legacy effects.
pub(crate) use crate::game::cinematic_objects::advance_intro_mother_brain;
pub(crate) use crate::game::ending_screens::{
    advance_ending_gunship_spin, advance_ending_samus_jump, advance_ending_sprite_pair,
    advance_postcredits_samus_message, advance_postcredits_samus_reveal, advance_postcredits_zoom,
    move_ending_cloud, update_ending_critters, wait_ending_cloud,
};
pub(crate) use crate::game::hdma_objects::{
    clear_hdma, clear_hdma_channels, initialize_room_effects, set_hdma_enabled, spawn_bg3_scroll,
    spawn_hdma, spawn_hdma_words, update_hdma,
};
pub(crate) use crate::game::intro_objects::{
    advance_ceres_station, advance_intro_baby_pursuit, advance_intro_landing_mosaic,
    advance_intro_rinka, advance_zebes_destruction_zoom, advance_zebes_flight, advance_zebes_stars,
    launch_cinematic_sprite, wait_cinematic_scene,
};
pub(crate) use crate::game::room_visuals::{
    advance_crocomire, align_crocomire_background, darken_fireflea_room,
    initialize_bomb_torizo_haze, initialize_ceres_elevator, initialize_ceres_ridley,
    initialize_draygon_layer, initialize_fireflea_lighting, initialize_morph_eye_beam,
    initialize_phantoon_motion, initialize_room_haze, initialize_tourian_statue,
    initialize_weather, shake_room_with_sound,
};
pub(crate) use crate::game::title_screen::{advance_title_pan, advance_title_wait};
pub(crate) use sm_game::cinematic::drift_cinematic_sprite;

pub(crate) use crate::game::hosts::{draw_player_frame, update_beam_tiles_and_palette};
pub(crate) use crate::game::visual_checks::{
    check_animation_guard, check_palette_guard, samus_observer,
};
use crate::{
    game::{
        enemy_projectiles, palette_effects, player_projectiles,
        session_runtime::Runtime,
        visual_host::{self, AnimationMember, PaletteMember, Source, StatueProjectile},
        visual_state::{PaletteBuffer, Upload},
    },
    rom::RomRecord,
};
use sm_game::lease::Lease;
use sm_game::visual_host::session_aliases as aliases;

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
impl visual_host::Aliases for Runtime<'_> {
    fn palette_word(&self, member: PaletteMember, handle: u16) -> u16 {
        aliases::palette_word(self, member, handle)
    }
    fn set_palette_word_slice(&mut self, member: PaletteMember, handle: u16, value: u16) {
        aliases::set_palette_word_slice(self, member, handle, value)
    }
    fn set_palette_counter_byte(&mut self, handle: u16, value: u8) {
        aliases::set_palette_counter_byte(self, handle, value)
    }
    fn palette_flag_native(&self, displacement: u16) -> u16 {
        aliases::palette_flag_native(self, displacement)
    }
    fn animation_word(&self, member: AnimationMember, handle: u16) -> u16 {
        aliases::animation_word(self, member, handle)
    }
    fn set_animation_word_prefix(&mut self, member: AnimationMember, handle: u16, value: u16) {
        aliases::set_animation_word_prefix(self, member, handle, value)
    }
    fn set_animation_counter_byte(&mut self, handle: u16, value: u8) {
        aliases::set_animation_counter_byte(self, handle, value)
    }
    fn set_palette_color_slice(&mut self, displacement: u16, value: u16) {
        aliases::set_palette_color_slice(self, displacement, value)
    }
    fn set_color_prefix(&mut self, buffer: PaletteBuffer, index: usize, value: u16) {
        aliases::set_color_prefix(self, buffer, index, value)
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
        sm_game::lease::draw_samus_sprite(self, index, x, y);
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
        sm_game::lease::music_delayed8(self, track);
    }
    fn resume_movement_sound(&mut self) {
        #[cfg(sm_verify_dll)]
        samus_observer::event("command", 0x1c);
        Lease::native_player_command(self, 0x1c);
    }
    fn upload(&mut self, request: Upload) {
        Lease::native_upload(self, request);
    }
    fn invalid_palette_instruction(&mut self) {
        Lease::native_unreachable(self);
    }
    fn invalid_animation_instruction(&mut self) -> u16 {
        Lease::native_animation_unreachable(self)
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
    fn calculate_xray_window(&mut self, window: sm_game::beam_windows::XrayWindow) {
        self.foreign(|game| crate::game::beam_windows::calculate_xray_window(window, game));
    }
}
