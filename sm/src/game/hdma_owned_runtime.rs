//! Shared common HDMA update on the existing aggregate Runtime.
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::owned_hdma_update::{bindings::VisualTable, context::Context};

pub(crate) fn update_hdma(carry: bool, game: &mut Game) {
    // The original Game path completes music, power-bomb setup and blending
    // before the interpreter asks whether a Runtime can be admitted.
    sm_game::hdma_objects::update_hdma(carry, game);
}
impl sm_game::actors::boss_hdma_owned::Host for Runtime<'_> {
    fn write_dma_channel(
        &mut self,
        channel: usize,
        field: sm_game::actors::boss_hdma_owned::DmaField,
        value: u8,
    ) {
        let offset = match field {
            sm_game::actors::boss_hdma_owned::DmaField::Control => 0,
            sm_game::actors::boss_hdma_owned::DmaField::Target => 1,
        };
        let address = 0x4300 + u16::try_from(channel).expect("DMA channel") * 16 + offset;
        self.hdma_register(address, value);
    }
}
impl Context for Runtime<'_> {
    fn current_byte(&self, at: usize) -> u8 {
        Runtime::byte(self, at)
    }
    fn set_current_byte(&mut self, at: usize, value: u8) {
        Runtime::set_byte(self, at, value)
    }
    fn write_current_slice(&mut self, at: usize, bytes: &[u8]) {
        Runtime::write_slice(self, at, bytes)
    }
    fn hdma_music(&mut self) -> Result<(), sm_game::scene::state::DisplayError> {
        sm_game::music_requests::advance_music(self, |effect, meaning, host| {
            let selector = match effect {
                sm_game::music_requests::MusicEffect::UploadBank => {
                    Some(sm_data::OriginalMusicBank(
                        sm_game::session_owners::AudioHost::audio_word(host, 0x63d).to_le_bytes()
                            [0],
                    ))
                }
                sm_game::music_requests::MusicEffect::WriteTrack(_) => None,
            };
            host.foreign(|game| {
                crate::game::session_runtime::prepared_music_effect(selector, effect, meaning, game)
            });
            Ok(())
        })
    }
    fn hdma_register(&mut self, address: u16, value: u8) {
        self.foreign(|game| unsafe { crate::rtl::write_reg(address, value, game) });
    }
    fn hdma_readback(
        &mut self,
        register: usize,
        offset: u16,
        destination: u16,
        size: u16,
        retain: bool,
    ) {
        self.foreign(|game| {
            sm_game::display_refresh::enqueue_readback(
                register,
                offset,
                destination,
                size,
                retain,
                game,
            )
        });
    }
    fn hdma_visual_word(&mut self, table: VisualTable, index: isize) -> u16 {
        visual_word(self.samus_game(), table, index)
    }
    fn hdma_overlay_origin_assertion(&mut self) {
        self.foreign(|game| sm_game::xray_background::Host::overlay_origin_assertion(game));
    }
    fn hdma_unreachable(&mut self) {
        self.foreign(crate::rtl::report_unreachable);
    }
    fn hdma_rainbow(&mut self, initialize: bool) {
        if initialize {
            sm_game::actors::mother_brain_rainbow::initialize_hdma(self)
        } else {
            sm_game::actors::mother_brain_rainbow::advance_hdma(self)
        }
    }
    fn hdma_player_command(&mut self, command: u16) {
        assert!(matches!(command, 0xb | 0x1e));
        sm_game::player_transitions::command(
            self,
            command,
            sm_game::player_transitions::commands::finish_ceres_arrival,
        );
    }
}
fn visual_word(game: &Game, table: VisualTable, index: isize) -> u16 {
    match table {
        VisualTable::Bg3Velocity => sm_game::imported::word(
            &game.visual_tables.bg3_x_velocities,
            game,
            sm_data::hdma_objects::BG3_X_VELOCITY_ADDRESS,
            index,
        ),
        VisualTable::TitleGradient => sm_game::imported::word(
            &game.visual_tables.title_gradients,
            game,
            sm_data::title_screen::GRADIENT_ADDRESS,
            index,
        ),
    }
}
impl sm_game::owned_hdma_update::bindings::Ports
    for crate::game::actors::prepared_execution::PreparedActorPorts<'_>
{
    fn hdma_register(
        &mut self,
        owner: &mut sm_game::persistent_session::PersistentSession,
        address: u16,
        value: u8,
    ) {
        self.physical_effect(owner, |game| unsafe {
            crate::rtl::write_reg(address, value, game)
        });
    }
    fn hdma_readback(
        &mut self,
        owner: &mut sm_game::persistent_session::PersistentSession,
        register: usize,
        offset: u16,
        destination: u16,
        size: u16,
        retain: bool,
    ) {
        self.physical_effect(owner, |game| {
            sm_game::display_refresh::enqueue_readback(
                register,
                offset,
                destination,
                size,
                retain,
                game,
            )
        });
    }
    fn hdma_visual_word(
        &mut self,
        _owner: &mut sm_game::persistent_session::PersistentSession,
        _allocations: &mut sm_game::actors::owned_execution::ActorAllocations,
        table: VisualTable,
        index: isize,
    ) -> u16 {
        visual_word(self.game, table, index)
    }
    fn hdma_overlay_origin_assertion(
        &mut self,
        owner: &mut sm_game::persistent_session::PersistentSession,
    ) {
        self.physical_effect(owner, |game| {
            sm_game::xray_background::Host::overlay_origin_assertion(game)
        });
    }
    fn hdma_unreachable(&mut self, owner: &mut sm_game::persistent_session::PersistentSession) {
        self.physical_effect(owner, crate::rtl::report_unreachable);
    }
    fn hdma_physical_control_byte(
        &self,
        _state: &sm_game::session::Session,
        field: sm_game::display::preparation::Field,
    ) -> u8 {
        sm_game::display::preparation::Access::physical_display_control_byte(self.game, field)
    }
    fn hdma_store_physical_control_byte(
        &mut self,
        owner: &mut sm_game::persistent_session::PersistentSession,
        field: sm_game::display::preparation::Field,
        value: u8,
    ) {
        self.physical_effect(owner, |game| {
            sm_game::display::preparation::Access::store_physical_display_control_byte(
                game, field, value,
            )
        });
    }
}
