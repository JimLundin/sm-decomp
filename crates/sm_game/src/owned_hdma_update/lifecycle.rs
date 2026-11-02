//! Room effect lifecycle on the actual current Session owner.
#![forbid(unsafe_code)]
use crate::display::hdma_spawn::HdmaObjectDefinition;
use crate::display::preparation::Field as DisplayField;
use crate::fx_state::Field as FxField;
use crate::scene::hdma_sources::LiveOwner;
use crate::session_access::SessionAccess;

pub trait FxHost:
    super::context::Context + crate::visual_host::Host + crate::room_state::setup::Host
{
    fn room_spawn_hdma(&mut self, control: u8, target: u8, program: u16) {
        crate::actors::boss_hdma_owned::spawn_hdma_owned(
            0x88,
            &HdmaObjectDefinition::new(control, target, program),
            self,
        );
    }
    fn room_fx_word(&mut self, field: FxField, value: u16) {
        crate::fx_encoding::write_control_word(
            self.session_mut(),
            crate::fx_encoding::address(field),
            value,
        );
        self.refresh_inputs();
    }
    fn room_display_byte(&mut self, field: DisplayField, value: u8) {
        crate::display::preparation::Access::store_display_control_byte(
            self,
            field,
            value,
            crate::display::preparation::Store::Owned,
        );
    }
    fn room_source_native_word(&mut self, source: LiveOwner, offset: usize, value: u16) {
        let next = offset.checked_add(1).expect("room source extent");
        // Each original byte publication resolves the live canonical source.
        let [first, second] = value.to_ne_bytes();
        crate::fx_encoding::store_session_source_byte(self.session_mut(), source, offset, first)
            .expect("room FX source");
        crate::fx_encoding::store_session_source_byte(self.session_mut(), source, next, second)
            .expect("room FX source");
        self.refresh_inputs();
    }
    fn room_initialize_fx(&mut self, initializer: crate::room_state::fx::Initializer) {
        use crate::constants::bank_87::*;
        use crate::room_state::fx::Initializer;
        match initializer {
            Initializer::Lava | Initializer::Acid => {
                self.session_mut().room.liquid.rising = 0xb343;
                self.session_mut().room.liquid.lava_acid.whole = self.session().room.liquid_height;
                self.room_spawn_hdma(0x42, 0x12, 0xc3e1);
                self.room_spawn_hdma(0x42, 0x10, 0xc3f0);
                crate::actors::boss_hdma_owned::spawn_bg3_scroll(self);
                let tiles = if matches!(initializer, Initializer::Lava) {
                    ADDR_K_ANIMTILES_LAVA
                } else {
                    ADDR_K_ANIMTILES_ACID
                };
                crate::animated_tiles::execution::spawn(
                    self,
                    u16::try_from(tiles).expect("liquid animation"),
                );
            }
            Initializer::Water => {
                self.session_mut().room.liquid.rising = 0xc428;
                self.session_mut().room.liquid.surface.whole = self.session().room.liquid_height;
                self.room_spawn_hdma(0x42, 0x11, 0xd856);
                if self.session().room.liquid_options & 2 != 0 {
                    self.room_spawn_hdma(0x42, 0xf, 0xd847);
                }
                crate::actors::boss_hdma_owned::spawn_bg3_scroll(self);
            }
            Initializer::Weather(weather) => {
                use crate::visual_state::Weather;
                let (control, program, tiles) = match weather {
                    Weather::Rain => (0x43, 0xd96c, Some(ADDR_K_ANIMTILES_RAIN)),
                    Weather::Spores => (0x42, 0xda2d, Some(ADDR_K_ANIMTILES_SPORES)),
                    Weather::Fog => (0x43, 0xdb19, None),
                };
                self.room_display_byte(DisplayField::Background3Map, 92);
                self.room_spawn_hdma(control, 0x11, program);
                if let Some(tiles) = tiles {
                    crate::actors::boss_hdma_owned::spawn_bg3_scroll(self);
                    crate::animated_tiles::execution::spawn(
                        self,
                        u16::try_from(tiles).expect("weather animation"),
                    );
                }
            }
            Initializer::Land => self.room_initialize_land(),
            Initializer::BandedSky => {
                self.session_mut().room.liquid.surface.whole = 1248;
                self.session_mut().room.liquid.kind = 6;
                self.room_spawn_hdma(2, 0x12, 0xad63);
                self.room_spawn_hdma(0x42, 0x11, 0xad4e);
                self.room_spawn_hdma(0x42, 0xd, 0xad39);
                self.room_sky_vertical_scroll();
            }
            Initializer::FirefleaLighting => {
                crate::fx_lighting::begin_current(self);
                self.room_spawn_hdma(0x42, 0x11, 0xb0ac);
                self.room_source_native_word(LiveOwner::WindowLeft, 0, 0);
                crate::fx_lighting::finish_current(self);
            }
            Initializer::TourianStatue => {
                if crate::progress::event_happened(self, 0xa) {
                    crate::room_state::setup::Host::setup_mechanism(self, 6, 12, 0xb777);
                    crate::room_scrolls::state::store_pair(self, 0, 514u16.to_ne_bytes());
                }
                self.room_display_byte(DisplayField::Screens1, 74);
                self.session_mut().room.liquid.rising = 0xc428;
                self.session_mut().room.liquid.surface.whole = self.session().room.liquid_height;
                self.room_spawn_hdma(0x42, 0x11, 0xd856);
                self.room_spawn_hdma(0x42, 0x10, 0xdcfa);
                crate::actors::boss_hdma_owned::spawn_bg3_scroll(self);
            }
            Initializer::CeresRidley => {
                self.room_spawn_hdma(0x40, 5, 0xd906);
                self.room_spawn_hdma(0, 0x2c, 0xd91d);
            }
            Initializer::CeresElevator => self.room_spawn_hdma(0x40, 5, 0xd939),
            Initializer::Haze => self.room_initialize_haze(),
        }
        self.refresh_inputs();
    }
    fn room_initialize_haze(&mut self) {
        let program = if crate::progress::boss_defeated(self, 1) {
            0xdeeb
        } else {
            0xded3
        };
        self.room_spawn_hdma(0x40, 0x32, program);
    }
    fn room_initialize_land(&mut self) {
        self.session_mut().room.entry.layer2_x |= 1;
        self.session_mut().room.entry.layer2_y |= 1;
        self.room_spawn_hdma(0x42, 0xf, 0xad76);
        self.room_fx_word(FxField::SkyVisibleHeight, 224);
        self.room_fx_word(FxField::SkyCompatibilityWord, 0);
    }
    fn room_initialize_ocean(&mut self) {
        self.room_display_byte(DisplayField::Screens1, 74);
        self.room_spawn_hdma(0x42, 0xf, 0xad89);
        self.room_fx_word(FxField::SkyVisibleHeight, 224);
        self.room_fx_word(FxField::SkyCompatibilityWord, 0);
    }
    fn room_sky_vertical_scroll(&mut self) {
        crate::owned_hdma_update::vertical_sky_scroll(self);
    }
}

pub trait ResetHost:
    Sized
    + SessionAccess
    + crate::display::preparation::Access
    + crate::fx_domain::Controls
    + crate::display_scroll::Access
{
    fn room_register(&mut self, register: u16, value: u8);
    fn room_mode7(&mut self, value: u16);
    fn room_fx_word(&mut self, field: FxField, value: u16) {
        crate::fx_encoding::write_control_word(
            self.session_mut(),
            crate::fx_encoding::address(field),
            value,
        );
        self.refresh_inputs();
    }
    fn room_display_byte(&mut self, field: DisplayField, value: u8) {
        crate::display::preparation::Access::store_display_control_byte(
            self,
            field,
            value,
            crate::display::preparation::Store::Owned,
        );
    }
    fn room_initialize_special_effects(&mut self) {
        self.room_fx_word(FxField::EarthquakeSoundIndex, 0);
        self.room_fx_word(FxField::EarthquakeSoundTimer, 0);
        if matches!(
            self.session().room_key(),
            0x9804 | 0x96ba | 0xb32e | 0xb457 | 0xdd58 | 0xdede
        ) {
            self.room_fx_word(FxField::EarthquakeSoundTimer, u16::MAX);
        }
        self.session_mut().minimap.disabled = 0;
        self.refresh_inputs();
        for channel in (0x20..0x80u16).step_by(0x10) {
            for register in [0, 1, 2, 3, 4, 5, 6, 8, 9] {
                self.room_register(
                    0x4300 + channel + register,
                    if register == 1 { 0x13 } else { 0 },
                );
                self.refresh_inputs();
            }
        }
        self.session_mut().room.liquid.surface.fraction = 0;
        self.session_mut().room.liquid.surface.whole = u16::MAX;
        self.session_mut().room.liquid.lava_acid.fraction = 0;
        self.session_mut().room.liquid.lava_acid.whole = u16::MAX;
        for field in [
            FxField::HudHorizontal,
            FxField::HudVertical,
            FxField::EffectHorizontal,
            FxField::EffectVertical,
        ] {
            self.room_fx_word(field, 0);
        }
        self.room_mode7(0);
        self.session_mut().room.camera.distance_index = 0;
        self.session_mut().mechanisms.statue_animation = 0;
        self.session_mut().mechanisms.statue_finished = 0;
        self.session_mut().feedback.earthquake_timer = 0;
        self.session_mut().room.fx_layer_flags = 0;
        self.session_mut().projectiles.power_bomb.explosion_status = 0;
        self.session_mut().projectiles.power_bomb.flag = 0;
        self.session_mut().room.power_bomb_radius = 0;
        self.session_mut().room.liquid.tilemap = 0;
        self.session_mut().room.liquid.kind = 0;
        self.session_mut().room.liquid.base_fraction = 0x8000;
        self.session_mut().room.liquid_height = 0;
        self.session_mut().room.liquid_target = 0;
        self.session_mut().room.liquid_velocity = 0;
        self.session_mut().room.liquid_options = 0;
        self.session_mut().room.liquid_delay = 0;
        self.session_mut().room.liquid.tide_phase = 0;
        self.session_mut().room.liquid.offset.fraction = 0;
        self.session_mut().room.liquid.offset.whole = 0;
        let mut blending = self.room_blending();
        blending.primary = 2;
        self.publish_room_blending(blending, true);
        crate::display_scroll::write(self, crate::display_scroll::Word::Bg3H, 0);
        crate::display_scroll::write(self, crate::display_scroll::Word::Bg3V, 0);
        self.session_mut().room.camera.layer2 = [0, 0];
        self.session_mut().room.entry.irq_policy = 0;
        for (hook, lane, value) in [
            (crate::suspended_work::Hook::Pause, 1, 0x8800u16),
            (crate::suspended_work::Hook::Unpause, 1, 0x8800),
            (crate::suspended_work::Hook::Pause, 0, 0x83e1),
            (crate::suspended_work::Hook::Unpause, 0, 0x83e1),
        ] {
            self.session_mut().suspended_work.store_hook_word(
                hook,
                lane,
                u16::from_le_bytes(value.to_ne_bytes()),
                crate::scene::state::WordOrder::Little,
            );
            self.refresh_inputs();
        }
        for (register, value) in [(0x2181, 0xf0), (0x2182, 0xff), (0x2183, 1)] {
            self.room_register(register, value);
            self.refresh_inputs();
        }
        self.room_display_byte(DisplayField::HdmaEnable, 0);
        for (field, value) in [
            (DisplayField::FixedColor0, 32),
            (DisplayField::FixedColor1, 64),
            (DisplayField::FixedColor2, 0x80),
            (DisplayField::Mosaic, 0),
            (DisplayField::MainLayers, 19),
            (DisplayField::ColorSelection, 0),
            (DisplayField::ColorMath, 0),
            (DisplayField::Characters0, 0),
            (DisplayField::Characters1, 4),
            (DisplayField::Screens1, 73),
            (DisplayField::Screens2, 90),
            (DisplayField::Background3Map, 90),
        ] {
            self.room_display_byte(field, value);
        }
    }
}
