//! Actual prepared HDMA consumer and its explicit retained resource ports.
#![forbid(unsafe_code)]
use super::effects::Effects as _;
use crate::actors::owned_execution::{ActorAllocations, PreparedActorState};
use crate::display::preparation::{Access, Field, Store};
use crate::fx_domain::{Controls, Sources};
use crate::hdma_dispatch::{HdmaCall, HdmaPreAction};
use crate::hdma_objects::HdmaEffect;
use crate::persistent_session::PersistentSession;
use crate::scene::hdma_sources::LiveOwner;
use crate::scene::state::DisplayError;
use crate::session_access::SessionAccess;

#[derive(Clone, Copy)]
pub enum VisualTable {
    Bg3Velocity,
    TitleGradient,
}
/// Physical readback delivery, arbitrary register aliases and the original
/// cached-content reads remain explicit. These are retained dependencies,
/// not ownership eliminations or a second mutable gameplay owner.
pub trait Ports {
    fn hdma_register(&mut self, owner: &mut PersistentSession, address: u16, value: u8);
    fn hdma_readback(
        &mut self,
        owner: &mut PersistentSession,
        register: usize,
        offset: u16,
        destination: u16,
        size: u16,
        retain: bool,
    );
    fn hdma_visual_word(
        &mut self,
        owner: &mut PersistentSession,
        allocations: &mut ActorAllocations,
        table: VisualTable,
        index: isize,
    ) -> u16;
    fn hdma_overlay_origin_assertion(&mut self, owner: &mut PersistentSession);
    fn hdma_unreachable(&mut self, owner: &mut PersistentSession);
    fn hdma_physical_control_byte(&self, state: &crate::session::Session, field: Field) -> u8;
    fn hdma_store_physical_control_byte(
        &mut self,
        owner: &mut PersistentSession,
        field: Field,
        value: u8,
    );
}

impl Access for PreparedActorState<'_> {
    fn display_control_byte(&self, field: Field) -> u8 {
        crate::display::preparation::display_byte(
            &self
                .session()
                .presentation
                .owner()
                .expect("HDMA display")
                .state,
            field,
        )
    }
    fn store_display_control_byte(&mut self, field: Field, value: u8, _store: Store) {
        crate::display::preparation::set_display_byte(
            &mut self
                .session_mut()
                .presentation
                .owner_mut()
                .expect("HDMA display")
                .state,
            field,
            value,
        );
        self.refresh_inputs();
    }
    fn display_history_word(&self, word: crate::display::preparation::Word) -> u16 {
        crate::display::preparation::display_word(
            &self
                .session()
                .presentation
                .owner()
                .expect("HDMA display history")
                .state,
            word,
        )
    }
    fn store_display_history_word(
        &mut self,
        word: crate::display::preparation::Word,
        value: u16,
        _store: Store,
    ) {
        let mut bytes = self.display_history_word(word).to_le_bytes();
        for (lane, value) in value.to_le_bytes().into_iter().enumerate() {
            bytes[lane] = value;
            crate::display::preparation::set_display_word(
                &mut self
                    .session_mut()
                    .presentation
                    .owner_mut()
                    .expect("HDMA history")
                    .state,
                word,
                u16::from_le_bytes(bytes),
            );
            self.refresh_inputs();
        }
    }
    fn hud_tile(&self, index: isize) -> u16 {
        let at = 0xc608usize
            .checked_add_signed(index.checked_mul(2).expect("HUD index overflow"))
            .expect("HUD address outside allocation");
        assert!(at < 0x1ffff);
        if let Some(value) = usize::try_from(index).ok().and_then(|index| {
            self.session()
                .presentation
                .owner()
                .expect("HDMA HUD")
                .state
                .hud
                .tiles
                .get(index)
        }) {
            return *value;
        }
        self.current_word(at)
    }
    fn store_hud_tile(&mut self, index: isize, value: u16, _store: Store) {
        let at = 0xc608usize
            .checked_add_signed(index.checked_mul(2).expect("HUD index overflow"))
            .expect("HUD address outside allocation");
        assert!(at < 0x1ffff);
        for (lane, value) in value.to_le_bytes().into_iter().enumerate() {
            if let Some(tile) = usize::try_from(index).ok().and_then(|index| {
                self.session_mut()
                    .presentation
                    .owner_mut()
                    .expect("HDMA HUD")
                    .state
                    .hud
                    .tiles
                    .get_mut(index)
            }) {
                let mut bytes = tile.to_le_bytes();
                bytes[lane] = value;
                *tile = u16::from_le_bytes(bytes);
                self.refresh_inputs();
            } else {
                self.set_current_byte(at + lane, value);
            }
        }
    }
    fn physical_display_control_byte(&self, field: Field) -> u8 {
        self.aliases
            .hdma_physical_control_byte(self.session(), field)
    }
    fn store_physical_display_control_byte(&mut self, field: Field, value: u8) {
        self.aliases
            .hdma_store_physical_control_byte(self.owner, field, value);
        self.refresh_inputs();
    }
}

impl<T: super::context::Context> super::Host for T {
    fn enable_byte(&self) -> u8 {
        self.display_control_byte(Field::HdmaEnable)
    }
    fn store_enable_byte(&mut self, value: u8) {
        self.store_display_control_byte(Field::HdmaEnable, value, Store::Owned)
    }
    fn random_number(&self) -> u16 {
        self.session().timing.random_number
    }
    fn event_happened(&mut self, event: u16) -> bool {
        crate::progress::event_happened(self, event)
    }
    fn pre_instruction(&mut self, action: HdmaPreAction, index: usize, carry: bool) {
        use HdmaPreAction::*;
        let slot = 2 * u16::try_from(index).expect("captured HDMA slot");
        match action {
            Nothing => {}
            SetupXrayBlending => super::player::apply_blending(
                self,
                crate::player_transitions::xray::BlendPhase::Setup,
            ),
            AdvanceXrayBackground => super::player::advance_background(self),
            XrayBeamInitial => super::player::advance_beam(
                self,
                crate::player_transitions::xray::BeamPhase::Initial,
            ),
            XrayBeamWidening => super::player::advance_beam(
                self,
                crate::player_transitions::xray::BeamPhase::Widening,
            ),
            XrayBeamFull => {
                super::player::advance_beam(self, crate::player_transitions::xray::BeamPhase::Full)
            }
            RestoreLeft => super::xray::restore_left(index, self),
            RestoreRight => super::xray::restore_right(self),
            FinishRestore => super::xray::finish_restore(index, self),
            PowerBombBlending => super::explosion::set_power_bomb_blending(slot, self),
            PowerBombAfterglow => super::explosion::power_bomb_afterglow(index, self),
            PowerBombScaled => super::explosion::update_scaled(
                index,
                slot,
                super::explosion::Phase::PowerBomb,
                self,
            ),
            PowerBombPrescaled => super::explosion::update_prescaled(
                index,
                slot,
                super::explosion::Phase::PowerBomb,
                self,
            ),
            PreFlashScaled => super::explosion::update_scaled(
                index,
                slot,
                super::explosion::Phase::PreFlash,
                self,
            ),
            PreFlashPrescaled => super::explosion::update_prescaled(
                index,
                slot,
                super::explosion::Phase::PreFlash,
                self,
            ),
            CrystalFlashBlending => super::explosion::set_crystal_flash_blending(slot, self),
            CrystalFlashAfterglow => super::explosion::crystal_flash_afterglow(index, self),
            CrystalFlashScaled => super::explosion::update_scaled(
                index,
                slot,
                super::explosion::Phase::CrystalFlash,
                self,
            ),
            SkyVerticalScroll => super::sky::vertical_scroll(self),
            SkyHorizontalScroll => super::sky::update_horizontal_scroll(index, self),
            SkyParallax => super::sky::update_parallax(self),
            AdvanceSkyParallax => super::sky::advance_parallax(self),
            FirefleaLighting => super::visuals::advance_fireflea_lighting(self),
            LavaSurface => super::liquid::lava_surface(index, carry, self),
            LavaBackground => super::liquid::lava_background(index, self),
            WaterSurface => super::liquid::water_surface(index, carry, self),
            WaterBackground => super::liquid::water_background(index, self),
            WaterWave => super::liquid::water_wave(index, self),
            WaterFlat => super::liquid::water_flat(self),
            Rain => super::visuals::scroll_weather(slot, crate::visual_state::Weather::Rain, self),
            Spores => {
                super::visuals::scroll_weather(slot, crate::visual_state::Weather::Spores, self)
            }
            Fog => super::visuals::scroll_weather(slot, crate::visual_state::Weather::Fog, self),
            StatueWaiting => super::visuals::advance_tourian_statue(
                index,
                crate::visual_state::TourianStatuePhase::Waiting,
                carry,
                self,
            ),
            StatueShaking => super::visuals::advance_tourian_statue(
                index,
                crate::visual_state::TourianStatuePhase::Shaking,
                carry,
                self,
            ),
            StatueSinking => super::visuals::advance_tourian_statue(
                index,
                crate::visual_state::TourianStatuePhase::Sinking,
                carry,
                self,
            ),
            StatueFinished => super::visuals::advance_tourian_statue(
                index,
                crate::visual_state::TourianStatuePhase::Finished,
                carry,
                self,
            ),
            StatueBlending => {
                let mut fx = self.fx_controls();
                fx.blending = 44;
                self.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
            }
            BeginBrightHaze => self.begin_haze(slot, 0x80),
            BeginDimHaze => self.begin_haze(slot, 0x20),
            HazeFadingIn => self.advance_haze(slot, crate::visual_state::HazePhase::FadingIn),
            HazeFadedIn => self.advance_haze(slot, crate::visual_state::HazePhase::FadedIn),
            HazeFadingOut => self.advance_haze(slot, crate::visual_state::HazePhase::FadingOut),
            DraygonLayer => self.select_draygon_layer(slot),
            SuitPickup => super::player::advance_pickup(self, false),
            GravitySuitPickup => super::player::advance_pickup(self, true),
            PhantoonLayer => self.select_phantoon_layer(),
            PhantoonWave => self.advance_hdma_wave(slot, crate::visual_state::HdmaWave::Phantoon),
            RainbowBeam => self.hdma_rainbow(false),
            EyeBeamWidening => {
                self.advance_eye_beam(slot, crate::visual_state::EyeBeamPhase::Widening)
            }
            EyeBeamFull => self.advance_eye_beam(slot, crate::visual_state::EyeBeamPhase::Full),
            EyeBeamFading => self.advance_eye_beam(slot, crate::visual_state::EyeBeamPhase::Fading),
            TitleGradientBackdrop => self.hdma_title_gradient(true, true),
            TitleGradient => self.hdma_title_gradient(false, true),
            IntroCrossfade => {
                if self.session().enemy_projectiles.slots[0].x.whole == 1 {
                    self.advance_hdma_program();
                }
            }
            IntroWave => self.advance_hdma_wave(slot, crate::visual_state::HdmaWave::Intro),
        }
    }
    fn call(&mut self, action: HdmaCall, index: usize) {
        use HdmaCall::*;
        let slot = 2 * u16::try_from(index).expect("captured HDMA call slot");
        match action {
            InitializeRainbowBeam => self.hdma_rainbow(true),
            BeginPreFlash => super::explosion::begin_pre_flash(self),
            BeginPrescaledPreFlash => super::explosion::begin_prescaled_pre_flash(self),
            BeginExplosion => super::explosion::begin_explosion(self),
            BeginPrescaledExplosion => super::explosion::begin_prescaled_explosion(self),
            FinishPowerBomb => super::explosion::finish_power_bomb(index, slot, self),
            BeginCrystalFlash => super::explosion::begin_crystal_flash(self),
            FinishCrystalFlash => super::explosion::finish_crystal_flash(index, slot, self),
            Nothing => {}
            FreezeAndBackup => super::xray::freeze_and_backup(index, self),
            CaptureBg1Right => super::xray::capture_bg1_right(self),
            CaptureBg1Left => super::xray::capture_bg1_left(self),
            BuildOverlay => super::xray::build_overlay(self),
            CaptureBg2Right => super::xray::capture_bg2_right(self),
            UploadLeft => super::xray::upload_left(self),
            UploadRight => super::xray::upload_right(self),
            SetBackdrop => super::xray::set_backdrop(self),
            InitializeSuitPickup => super::player::initialize_hdma(self),
        }
    }
    fn effect(&mut self, effect: HdmaEffect, index: usize) {
        use HdmaEffect::*;
        let slot = 2 * u16::try_from(index).expect("captured HDMA effect slot");
        match effect {
            InitializeRainbowBeam => self.hdma_rainbow(true),
            AdvanceRainbowBeam => self.hdma_rainbow(false),
            AdvanceProgram => self.advance_hdma_program(),
            InitializePhantoonWave => self.initialize_phantoon_wave(slot),
            InitializeEyeBeam => self.initialize_eye_beam(slot),
            ResetWave => self.reset_hdma_wave(slot),
            InitializeWavePair { base, primary } => self.initialize_wave_pair(base, primary),
            SetPhantoonMotion { direction, speed } => self.set_phantoon_motion(direction, speed),
            ScrollWeather(weather) => self.scroll_weather_layers(slot, weather),
            BeginHaze { color } => self.begin_haze(slot, color),
            AdvanceHaze(phase) => self.advance_haze(slot, phase),
            SelectDraygonLayer => self.select_draygon_layer(slot),
            SelectPhantoonLayer => self.select_phantoon_layer(),
            AdvanceWave(wave) => self.advance_hdma_wave(slot, wave),
            AdvanceEyeBeam(phase) => self.advance_eye_beam(slot, phase),
            AdvanceXrayBackground => super::player::advance_background(self),
            AdvanceXrayBeam(phase) => super::player::advance_beam(self, phase),
            InitializeSuitPickup => super::player::initialize_hdma(self),
            AdvanceSuitPickup { gravity } => super::player::advance_pickup(self, gravity),
        }
    }
    fn channel_register(&mut self, address: u16, value: u8) {
        self.hdma_register(address, value)
    }
    fn bg3_velocity(&mut self, index: isize) -> u16 {
        self.hdma_visual_word(VisualTable::Bg3Velocity, index)
    }
    fn select_ceres_table(&mut self, value: u16) {
        self.hdma_slice_le(crate::ram::words::HDMA_DATA_TABLE_IN_CERES, value)
    }
    fn statue_scroll(&mut self, value: u16) {
        self.store_source_word(
            LiveOwner::WindowLeft,
            0,
            crate::fx_domain::native_order(),
            value,
        )
    }
    fn statue_tiles(&mut self, value: u16) {
        crate::room_scrolls::state::store_pair(self, 0, value.to_ne_bytes())
    }
    fn restart_title_gradient(&mut self) {
        self.store_display_control_byte(Field::ColorSelection, 0, Store::Owned);
        self.hdma_title_gradient(true, false);
    }
    fn unreachable(&mut self) {
        self.hdma_unreachable()
    }
}

impl PreparedActorState<'_> {
    pub fn update_hdma(&mut self, handler_carry: bool) -> Result<(), DisplayError> {
        super::update(self, handler_carry)
    }
}

impl super::context::Context for PreparedActorState<'_> {
    fn current_byte(&self, at: usize) -> u8 {
        PreparedActorState::current_byte(self, at)
    }
    fn set_current_byte(&mut self, at: usize, value: u8) {
        PreparedActorState::set_current_byte(self, at, value)
    }
    fn write_current_slice(&mut self, at: usize, bytes: &[u8]) {
        PreparedActorState::write_current_slice(self, at, bytes)
    }
    fn hdma_music(&mut self) -> Result<(), DisplayError> {
        crate::music_requests::advance_music(self, |effect, meaning, host| {
            host.aliases.room_music_effect(host.owner, effect, meaning)
        })
    }
    fn hdma_register(&mut self, address: u16, value: u8) {
        self.aliases.hdma_register(self.owner, address, value);
        self.refresh_inputs()
    }
    fn hdma_readback(
        &mut self,
        register: usize,
        offset: u16,
        destination: u16,
        size: u16,
        retain: bool,
    ) {
        self.aliases
            .hdma_readback(self.owner, register, offset, destination, size, retain);
        self.refresh_inputs();
    }
    fn hdma_visual_word(&mut self, table: VisualTable, index: isize) -> u16 {
        let value = self
            .aliases
            .hdma_visual_word(self.owner, self.allocations, table, index);
        self.refresh_inputs();
        value
    }
    fn hdma_overlay_origin_assertion(&mut self) {
        self.aliases.hdma_overlay_origin_assertion(self.owner);
        self.refresh_inputs()
    }
    fn hdma_unreachable(&mut self) {
        self.aliases.hdma_unreachable(self.owner);
        self.refresh_inputs()
    }
    fn hdma_rainbow(&mut self, initialize: bool) {
        if initialize {
            crate::actors::mother_brain_rainbow_owned::initialize_hdma(self)
        } else {
            crate::actors::mother_brain_rainbow_owned::advance_hdma(self)
        }
    }
    fn hdma_player_command(&mut self, command: u16) {
        crate::player_effects::command(self, command);
    }
}

impl Controls for PreparedActorState<'_> {
    fn room_blending(&self) -> crate::fx_domain::RoomBlending {
        crate::fx_encoding::session_room_blending(self.session())
    }
    fn publish_room_blending(&mut self, value: crate::fx_domain::RoomBlending, primary: bool) {
        let (at, value) = if primary {
            (0x1982, value.primary)
        } else {
            (0x1984, value.liquid)
        };
        crate::fx_encoding::write_control_word(self.session_mut(), at, value);
        self.refresh_inputs();
    }
    fn fx_controls(&self) -> crate::fx_state::FxState {
        crate::fx_encoding::session_controls(self.session())
    }
    fn publish_fx_controls(
        &mut self,
        value: crate::fx_state::FxState,
        fields: &[crate::fx_state::Field],
    ) {
        for &field in fields {
            crate::fx_encoding::write_control_word(
                self.session_mut(),
                crate::fx_encoding::address(field),
                field.get(&value),
            );
        }
        self.refresh_inputs();
    }
}
impl Sources for PreparedActorState<'_> {
    fn store_source_signed_byte(&mut self, owner: LiveOwner, offset: isize, value: u8) {
        self.store_source_byte(
            owner,
            usize::try_from(offset).expect("prepared source outside allocation"),
            value,
        );
    }
    fn source_byte(&self, owner: LiveOwner, offset: usize) -> u8 {
        crate::fx_encoding::session_source_byte(self.session(), owner, offset)
            .expect("current HDMA source")
    }
    fn store_source_byte(&mut self, owner: LiveOwner, offset: usize, value: u8) {
        crate::fx_encoding::store_session_source_byte(self.session_mut(), owner, offset, value)
            .expect("HDMA source publication");
        self.refresh_inputs();
    }
    fn add_parallax_source(&mut self, selector: u16, delta: u32) {
        let at = usize::from(selector);
        let read = |host: &Self, position| match crate::fx_encoding::physical_source_coordinate(
            host.session(),
            position,
        )
        .expect("parallax topology")
        {
            Some((owner, offset)) => host.source_byte(owner, offset),
            None => host.current_byte(position),
        };
        let value = u32::from_le_bytes([
            read(self, at),
            read(self, at + 1),
            read(self, at + 2),
            read(self, at + 3),
        ])
        .wrapping_add(delta);
        for (lane, value) in value.to_le_bytes().into_iter().enumerate() {
            let position = at + lane;
            match crate::fx_encoding::physical_source_coordinate(self.session(), position)
                .expect("parallax topology")
            {
                Some((owner, offset)) => self.store_source_byte(owner, offset, value),
                None => self.set_current_byte(position, value),
            }
        }
    }
}
