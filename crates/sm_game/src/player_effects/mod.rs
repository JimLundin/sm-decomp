//! Complete player callback bodies composed over one current Session.
#![forbid(unsafe_code)]
pub mod grapple;
pub mod input;
pub mod save;
use crate::{
    player_projectiles::host::{PaletteEffect, PlayerStep},
    player_transitions as transitions,
    visual_state::PaletteBuffer,
};

/// Window operations select the existing HDMA/window owner. No second window,
/// display or palette allocation belongs to player behavior.
pub trait Host:
    input::Host
    + grapple::Host
    + crate::player_movement::effects::Host
    + crate::visual_host::Content<Record = sm_data::cartridge::RomRecord>
{
    fn set_xray_window(&mut self, row: u16, value: u16);
    fn setup_xray_tables(&mut self);
    fn spawn_player_hdma(
        &mut self,
        bank: u8,
        definition: crate::display::hdma_spawn::HdmaObjectDefinition,
    );
    fn spawn_player_hdma_record(&mut self, bank: u8, definition: sm_data::cartridge::RomRecord);
    fn set_time_frozen(&mut self, frozen: u16);
    fn finish_ceres_arrival(&mut self);
}
pub fn player_step(h: &mut impl Host, step: PlayerStep) {
    match step {
        PlayerStep::Radius => {
            use crate::player_collision::effects::Content;
            use crate::visual_host::Record;
            let table = Content::pose_table(h);
            h.session_mut().player.contact.radius =
                u16::from(table.byte(usize::from(h.session().player.pose) * 8 + 6));
        }
        PlayerStep::SuitIndex => crate::samus_palette::update_suit_index(h.session_mut()),
        PlayerStep::Input => input::dispatch(h),
        PlayerStep::Acceleration => crate::player_movement::speed::update_vertical_acceleration(h),
        PlayerStep::InsideBlock => crate::player_collision::inside::sample(h),
        PlayerStep::Grapple => grapple::update(h),
        PlayerStep::Xray => request_scope(h),
        PlayerStep::RestoreControl => {
            command(h, 0x14);
        }
    }
}
pub fn command<H: Host>(h: &mut H, command: u16) -> u16 {
    transitions::command(h, command, |h| h.finish_ceres_arrival())
}
pub fn palette(h: &mut impl Host, effect: PaletteEffect) -> u8 {
    match effect {
        PaletteEffect::Suit => crate::samus_palette::load_suit(h, PaletteBuffer::Current),
        PaletteEffect::SuitTarget => crate::samus_palette::load_suit(h, PaletteBuffer::Target),
        PaletteEffect::Spawn(definition) => crate::palette_effects::execution::spawn(h, definition),
    }
    0
}
pub fn request_scope(h: &mut impl Host) {
    if h.inputs().frozen != 0
        || h.inputs().run_button & h.inputs().held == 0
        || !transitions::xray::request(h)
    {
        return;
    }
    h.set_time_frozen(1);
    h.session_mut().player.transitions.interruption =
        transitions::state::InterruptionAction::from_raw(5);
    for row in (0..256).rev() {
        h.set_xray_window(row, 255);
    }
    transitions::xray::set_world_running(h.session_mut(), false);
    h.setup_xray_tables();
    let demo = &mut h.session_mut().player.demo;
    demo.pre_action = 0;
    demo.delay = 0;
    demo.program = 0;
    demo.held = 0;
    demo.pressed = 0;
    h.session_mut().player.transitions.xray_angle =
        if h.session().player.movement_and_facing.to_le_bytes()[0] == 4 {
            192
        } else {
            64
        };
    // Original capture happens only after every gate and setup store succeeded.
    use crate::visual_host::{Content, Source};
    let record = Content::table(h, Source::Native, 0x91caf2);
    // Allocation chooses a free slot before reading instruction_list, and
    // reads DMA control only after that slot's initialization is published.
    h.spawn_player_hdma_record(0x91, record);
}
pub fn begin_suit<H: Host>(h: &mut H, gravity: bool) {
    transitions::suit::prepare(h, gravity, |h, row, value| h.set_xray_window(row, value));
    command(h, 0x15);
    transitions::suit::center(h);
    crate::visual_host::Effects::sound(h, 2, 6, 0x56);
    h.spawn_player_hdma(
        0x91,
        crate::display::hdma_spawn::HdmaObjectDefinition::new(
            0x41,
            0x26,
            if gravity { 0xd67a } else { 0xd5a2 },
        ),
    );
}
pub fn spawn_power_bomb(h: &mut impl Host) {
    if h.inputs().frozen != 0 {
        h.session_mut().projectiles.power_bomb.explosion_status = 0x4000;
    } else {
        crate::effects::SoundEffects::queue_sound(h, 0, 1, 15);
        h.session_mut().projectiles.power_bomb.explosion_status = 0x8000;
        h.spawn_player_hdma(
            0x88,
            crate::display::hdma_spawn::HdmaObjectDefinition::new(0x40, 0x28, 0x8ace),
        );
        h.spawn_player_hdma(
            0x88,
            crate::display::hdma_spawn::HdmaObjectDefinition::new(0x40, 0x29, 0x8b80),
        );
    }
}
