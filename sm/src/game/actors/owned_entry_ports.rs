//! Finite retained presentation ports for the complete owned Draygon/Ceres entries.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
use sm_game::actors::{ceres_ridley, draygon_eye, draygon_flight, ridley, spore_spawn};
use sm_game::display_scroll::{Word, write as scroll};
impl draygon_eye::owned::Host for Runtime<'_> {
    fn place_projectile_spawn(&mut self, x: u16, y: u16) {
        self.session_mut().enemy_projectiles.spawn.point = sm_game::geometry::Point { x, y };
    }
    fn spawn_room_projectile(&mut self, definition: u16, parameter: u16) -> u16 {
        sm_game::enemy_projectiles::lifecycle::spawn_room(self, definition, parameter)
    }
}
impl draygon_flight::entry::Host for Runtime<'_> {
    fn queue_intro_transfer(&mut self) {
        sm_game::display::transfer::queue_tilemap(
            self,
            sm_game::display::transfer::TilemapUpload {
                bytes: 0x600,
                source: sm_game::display::transfer::Source {
                    address: 0x9400,
                    bank: 0xb1,
                },
                column: false,
                vram_address: 0x6d00,
            },
        );
    }
    fn spawn_intro_trails(&mut self) {
        self.foreign(crate::game::sprite_effect_runtime::draygon_spawn_trails);
    }
    fn step_intro_trajectory(&mut self) {
        self.foreign(crate::game::sprite_effect_runtime::draygon_trajectory);
    }
    fn install_initial_palette(&mut self) {
        self.owned_call(draygon_flight::install_initial_palette);
    }
    fn install_bg2_scroll_hook(&mut self) {
        self.write_slice_native(0x178e, 0xa5);
        self.write_slice_native(0x178c, 0x9342);
    }
    fn install_layers(&mut self) {
        self.foreign(crate::game::visual_runtime::initialize_draygon_layer);
    }
    fn disable_minimap_for_boss(&mut self) {
        sm_game::player_update::minimap::disable_for_boss(self);
    }
    fn dispatch_draygon_other(&mut self, address: u32, key: u16) {
        match address {
            10863757 => draygon_eye::owned::look_at_samus(key, false, self),
            10863891 => draygon_eye::owned::look_at_samus(key, true, self),
            10846283 | 10873626 => {}
            10873627 => spore_spawn::owned_return::descend(self),
            10873682 => spore_spawn::owned_return::orbit(key, self),
            10873755 => spore_spawn::owned_return::begin_return(self),
            10873838 => spore_spawn::owned_return::finish_return(self),
            _ => {
                self.foreign(crate::game::animated_tiles::native::unreachable);
            }
        }
    }
    fn set_bg2_scroll(&mut self, horizontal: bool, value: u16) {
        scroll(
            self,
            if horizontal { Word::Bg2H } else { Word::Bg2V },
            value,
        );
    }
}
impl spore_spawn::owned_return::Host for Runtime<'_> {
    fn update_spore_stem(&mut self) {
        sm_game::enemy_projectiles::attachments_spore_maw::update_spore_stem(self);
    }
}
impl sm_game::escape_typewriter::owned::Host for Runtime<'_> {
    fn text_record(&mut self, cursor: u16) -> sm_data::cartridge::RomRecord {
        sm_game::cartridge::native_record(0xa6, cursor, self)
    }
    fn colour(&self, index: usize) -> u16 {
        sm_game::actors::boss_interaction::get_at(
            self,
            sm_game::actors::boss_interaction::Array::PaletteBuffer,
            index,
        )
    }
    fn set_colour(&mut self, index: usize, value: u16) {
        sm_game::actors::boss_interaction::set_at(
            self,
            sm_game::actors::boss_interaction::Array::PaletteBuffer,
            index,
            value,
        );
    }
    fn area_index(&self) -> u16 {
        self.session().room.area
    }
    fn set_bg2_scroll(&mut self, horizontal: bool, value: u16) {
        scroll(
            self,
            if horizontal { Word::Bg2H } else { Word::Bg2V },
            value,
        );
    }
    fn queue_typewriter_tile(&mut self, address: u16) {
        sm_game::display::transfer::queue_tilemap(
            self,
            sm_game::display::transfer::TilemapUpload {
                bytes: 2,
                source: sm_game::display::transfer::Source {
                    address: 0x8034,
                    bank: 0x7e,
                },
                column: address & 0x8000 != 0,
                vram_address: address,
            },
        );
    }
}
impl ceres_ridley::owned::Host for Runtime<'_> {
    fn boss_byte(&mut self, area: u16) -> u8 {
        ceres_ridley::Host::boss_byte(self, area)
    }
    fn disable_minimap_for_boss(&mut self) {
        sm_game::player_update::minimap::disable_for_boss(self);
    }
    fn copy_palette(&mut self, destination: u16, bank: u8, address: u16, colours: u16) {
        self.foreign(|game| {
            sm_game::palette_copies::copy_rom_colors_to_palette(
                destination,
                bank,
                address,
                colours,
                game,
            )
        });
    }
    fn copy_target_palette(&mut self, bank: u8, destination: u16, address: u16, colours: u16) {
        self.foreign(|game| {
            sm_game::palette_copies::copy_rom_colors_to_target_palette(
                bank,
                destination,
                address,
                colours,
                game,
            )
        });
    }
    fn clear_initial_palettes(&mut self, norfair: bool) {
        self.owned_call(|runtime| ceres_ridley::clear_initial_palettes(norfair, runtime));
    }
    fn colour(&self, index: usize) -> u16 {
        sm_game::actors::boss_interaction::get_at(
            self,
            sm_game::actors::boss_interaction::Array::PaletteBuffer,
            index,
        )
    }
    fn set_colour(&mut self, index: usize, value: u16) {
        sm_game::actors::boss_interaction::set_at(
            self,
            sm_game::actors::boss_interaction::Array::PaletteBuffer,
            index,
            value,
        );
    }
    fn eye_fade_step(&mut self, index: u16) -> u8 {
        sm_game::cartridge::NativeImage::native_image(self)
            .read(sm_data::cartridge::lorom_offset(0xa6_e269) as usize + usize::from(index))
    }
    fn ceres_health_palette(&mut self) {
        ridley::ceres_health_palette(self);
    }
    fn emergency_text_glow(&mut self) {
        ridley::emergency_text_glow(self);
    }
    fn set_mode7_irq(&mut self, value: u16) {
        self.write_slice_le(0x783, value);
    }
    fn set_bg1_scroll(&mut self, horizontal: bool, value: u16) {
        scroll(
            self,
            if horizontal { Word::Bg1H } else { Word::Bg1V },
            value,
        );
    }
    fn set_bg_mode(&mut self, value: u8) {
        self.set_byte(0x56, value);
    }
    fn set_mode7_settings(&mut self, value: u8) {
        self.set_byte(0x5f, value);
    }
    fn set_affine(&mut self, index: usize, value: u16) {
        match index {
            0 => sm_game::frame_display_word!(self.AffineWords0Pair = value),
            1 => sm_game::frame_display_word!(self.AffineWords1Pair = value),
            2 => sm_game::frame_display_word!(self.AffineWords2Pair = value),
            3 => sm_game::frame_display_word!(self.AffineWords3Pair = value),
            4 => sm_game::frame_display_word!(self.AffineWords4Pair = value),
            5 => sm_game::frame_display_word!(self.AffineWords5Pair = value),
            _ => unreachable!(),
        }
    }
    fn begin_ceres_push(&mut self) {
        ceres_ridley::Host::begin_ceres_push(self);
    }
    fn queue_mode7_transfer(&mut self, bank: u8, list: u16) {
        ceres_ridley::Host::queue_mode7_transfer(self, bank, list);
    }
    fn queue_escape_timer_tiles(&mut self) -> bool {
        ceres_ridley::Host::queue_escape_timer_tiles(self)
    }
    fn queue_getaway_tiles(&mut self) {
        ceres_ridley::Host::queue_getaway_tiles(self);
    }
    fn queue_escape_tile_table(&mut self) {
        ceres_ridley::Host::queue_escape_tile_table(self);
    }
    fn type_text(&mut self, tiles: u16) -> bool {
        sm_game::escape_typewriter::owned::handle_text(self, tiles)
    }
    fn ceres_eye_fade(&mut self, index: isize) -> u16 {
        sm_game::ceres_ridley_tables::eye_fade(self, index)
    }
    fn ceres_body_fade(&mut self, index: isize) -> u16 {
        sm_game::ceres_ridley_tables::body_fade(self, index)
    }
    fn ceres_background_fade(&mut self, index: isize) -> u16 {
        sm_game::ceres_ridley_tables::background_fade(self, index)
    }
    fn ceres_attack(&mut self, index: isize) -> u16 {
        sm_game::ceres_ridley_tables::attack(self, index)
    }
    fn ceres_getaway_zoom(&mut self, index: isize) -> u16 {
        sm_game::ceres_ridley_tables::getaway_zoom(self, index)
    }
    fn ceres_baby_metroid_frame(&mut self, index: isize) -> u16 {
        sm_game::ceres_ridley_tables::baby_metroid_frame(self, index)
    }
    fn ceres_wing_frame(&mut self, index: isize) -> u16 {
        sm_game::ceres_ridley_tables::wing_frame(self, index)
    }
    fn ceres_getaway_velocity(&mut self, index: isize) -> sm_data::ceres_ridley::GetawayVelocity {
        sm_game::ceres_ridley_tables::getaway_velocity(self, index)
    }
}
