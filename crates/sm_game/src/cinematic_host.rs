//! What the bank $8B cinematic functions need: the display, palette and
//! presentation owners, the cartridge, sound, the graphics loads and the
//! cinematic runtime operations, plus palette effects, delayed music, Mode 7
//! transfers and the session operations the compatibility runtime still
//! implements.
use crate::effects::SoundEffects;
use crate::ram::CompatibilityRam;
use crate::word_transfers::LoadedRom;

/// What the cinematic functions read and store on either host: the display,
/// palette, presentation and level data owners, the cartridge, sound and the
/// graphics loads. The Nintendo logo runs on it at boot, before the frame's
/// lease opens.
pub trait Scene:
    crate::fx_domain::Controls
    + crate::fx_domain::Sources
    + crate::fx_lighting::Access
    + crate::display::preparation::Access
    + crate::dialogue_ram::Access
    + crate::presentation_workspace::ContinuationAccess
    + crate::presentation_workspace::Access
    + crate::presentation_workspace::PaletteAccess
    + crate::palette_effects::fading::Host
    + crate::presentation_workspace::QueueAccess
    + crate::display::transfer::TilemapQueue
    + CompatibilityRam
    + crate::runtime_access::SessionRam
    + crate::player_position::PlayerPosition
    + crate::hud::HudPlayer
    + LoadedRom
    + crate::display::video::Video
    + SoundEffects
    + crate::display_refresh::DisplayOwner
    + crate::hdma_objects::HdmaTable
    + crate::session_owners::AudioHost
    + crate::cartridge::NativeDiagnostics
    + crate::cartridge::LoadedImage
    + crate::decompression::DecompressionHost
    + crate::records::Records
    + crate::display::queued_tilemaps::Scrolling
    + crate::display_refresh::ReadbackQueue
    + crate::screen_fade::Owner
    + crate::sprite_drawing::ObjectAccess
    + crate::sprite_output::MapHost
    + crate::display_refresh::GraphicsLoad
    + crate::effects::DisplayRefresh
    + crate::menus::state::Access
    + crate::display::transfer::Mode7Queue
    + crate::room_state::LevelReads
    + crate::display::queued_tilemaps::BackgroundBlocks
    + crate::menus::host::Tables
    + crate::gameplay_context::Access
{
}
impl<T: ?Sized> Scene for T where
    T: crate::fx_domain::Controls
        + crate::fx_domain::Sources
        + crate::fx_lighting::Access
        + crate::display::preparation::Access
        + crate::dialogue_ram::Access
        + crate::presentation_workspace::ContinuationAccess
        + crate::presentation_workspace::Access
        + crate::presentation_workspace::PaletteAccess
        + crate::palette_effects::fading::Host
        + crate::presentation_workspace::QueueAccess
        + crate::display::transfer::TilemapQueue
        + CompatibilityRam
        + crate::runtime_access::SessionRam
        + crate::player_position::PlayerPosition
        + crate::hud::HudPlayer
        + LoadedRom
        + crate::display::video::Video
        + SoundEffects
        + crate::display_refresh::DisplayOwner
        + crate::hdma_objects::HdmaTable
        + crate::session_owners::AudioHost
        + crate::cartridge::NativeDiagnostics
        + crate::cartridge::LoadedImage
        + crate::decompression::DecompressionHost
        + crate::records::Records
        + crate::display::queued_tilemaps::Scrolling
        + crate::display_refresh::ReadbackQueue
        + crate::screen_fade::Owner
        + crate::sprite_drawing::ObjectAccess
        + crate::sprite_output::MapHost
        + crate::display_refresh::GraphicsLoad
        + crate::effects::DisplayRefresh
        + crate::menus::state::Access
        + crate::display::transfer::Mode7Queue
        + crate::room_state::LevelReads
        + crate::display::queued_tilemaps::BackgroundBlocks
        + crate::menus::host::Tables
        + crate::gameplay_context::Access
{
}

/// The lease the cinematic game states run on: the frame's first lease, kept
/// open through the state.
pub trait Host:
    Sized
    + Scene
    + crate::menus::host::CinematicRuntime
    + crate::menus::save_slots::SlotHost
    + crate::runtime_access::PlayerLease
    + crate::lease::Lease
    + crate::shooting_star_motion::Lease
    + crate::hdma_effects::HdmaEffects
    + crate::actors::mother_brain_rainbow::Host
{
    fn spawn_power_bomb(&mut self);
    /// Set up Samus for the intro.
    fn initialize_player(&mut self);
    fn disable_scanline_irq(&mut self);
    /// Queue the ending's Mode 7 tile upload of `page`.
    fn advance_ending_tile_upload(&mut self, page: u16);
    /// Prepare Samus's projectiles and run her frame.
    fn advance_intro_player(&mut self);
    /// The original's failed native assertion `expression` in `function`.
    fn native_assertion(
        &mut self,
        expression: &'static [u8],
        line: u32,
        function: &'static [u8],
    ) -> !;
}
