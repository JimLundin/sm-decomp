//! Common HDMA scheduling on the current Session; callbacks share its interval.
#![forbid(unsafe_code)]
use crate::enemy_script::{EnemyScript as Script, bank_base};
use crate::hdma::{OBJECTS, ObjectWord, Word};
use crate::hdma_dispatch::{HDMA_CALLS, HDMA_PRE_ACTIONS, HdmaCall, HdmaPreAction};
use crate::hdma_objects::{HdmaEffect, HdmaTable};
use crate::scene::state::{DisplayError, WordOrder};

mod beam;
pub mod bindings;
pub mod context;
mod effects;
mod explosion;
mod layer;
pub mod lifecycle;
mod liquid;
mod player;
mod sky;
mod visuals;
mod xray;

/// Finite scheduler capabilities. Held contexts use their current owner.
/// The compatibility adapter retains each original callback admission point.
pub trait Host: HdmaTable + crate::cartridge::NativeImage {
    fn enable_byte(&self) -> u8;
    fn store_enable_byte(&mut self, value: u8);
    fn random_number(&self) -> u16;
    fn event_happened(&mut self, event: u16) -> bool;
    fn pre_instruction(&mut self, action: HdmaPreAction, captured: usize, carry: bool);
    fn call(&mut self, action: HdmaCall, captured: usize);
    fn effect(&mut self, effect: HdmaEffect, captured: usize);
    fn channel_register(&mut self, address: u16, value: u8);
    fn bg3_velocity(&mut self, index: isize) -> u16;
    fn select_ceres_table(&mut self, value: u16);
    fn statue_scroll(&mut self, value: u16);
    fn statue_tiles(&mut self, value: u16);
    fn restart_title_gradient(&mut self);
    fn unreachable(&mut self);
}

fn object(h: &(impl HdmaTable + ?Sized), index: usize, field: ObjectWord) -> u16 {
    h.hdma_word(Word::Object { index, field })
}
fn store(h: &mut (impl HdmaTable + ?Sized), index: usize, field: ObjectWord, value: u16) {
    h.store_hdma_word(Word::Object { index, field }, value, WordOrder::Little);
}

/// Music precedes the power-bomb activation test even when HDMA is disabled.
/// The fresh selector after each callback remains distinct from the captured
/// object running that callback and its instruction stream.
pub fn update(h: &mut impl context::Context, carry: bool) -> Result<(), DisplayError> {
    h.hdma_music()?;
    if h.session().timing.time_frozen == 0
        && h.session().projectiles.power_bomb.explosion_status & 0x4000 != 0
    {
        h.session_mut().projectiles.power_bomb.explosion_status = 0x8000;
        use crate::display::hdma_spawn::HdmaObjectDefinition;
        crate::actors::boss_hdma_owned::spawn_hdma_owned(
            0x88,
            &HdmaObjectDefinition::new(0x40, 0x28, 0x8ace),
            h,
        );
        crate::actors::boss_hdma_owned::spawn_hdma_owned(
            0x88,
            &HdmaObjectDefinition::new(0x40, 0x29, 0x8b80),
            h,
        );
    }
    let blending = h.room_blending().primary;
    let mut fx = h.fx_controls();
    fx.blending = blending;
    h.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
    if h.hdma_word(Word::Enabled) & 0x8000 != 0 {
        run_objects(h, carry);
        layer::update(h);
    }
    Ok(())
}

fn run_objects(h: &mut impl Host, carry: bool) {
    h.store_enable_byte(0);
    let mut slot = 0usize;
    loop {
        let selected = slot.to_le_bytes();
        h.store_hdma_word(
            Word::Selected,
            u16::from_le_bytes([selected[0], selected[1]]),
            WordOrder::Little,
        );
        let index = slot >> 1;
        // The original loop can reacquire a selector changed by a callback.
        // Keep its checked-table failure rather than masking it to six slots.
        let channels = object(h, index, ObjectWord::Channels).to_le_bytes()[0];
        if channels != 0 {
            let enabled = h.enable_byte() | channels;
            h.store_enable_byte(enabled);
            run_program(h, index, carry);
            slot = usize::from(h.hdma_word(Word::Selected));
        }
        slot += 2;
        if slot == 2 * OBJECTS {
            break;
        }
    }
}

fn run_program(h: &mut impl Host, index: usize, carry: bool) {
    let address = (u32::from(object(h, index, ObjectWord::PreInstructionBank)) << 16)
        | u32::from(object(h, index, ObjectWord::PreInstruction));
    if let Some(&(_, action)) = HDMA_PRE_ACTIONS.iter().find(|(at, _)| *at == address) {
        h.pre_instruction(action, index, carry);
    } else {
        h.unreachable();
    }
    let timer = object(h, index, ObjectWord::InstructionTimer);
    store(
        h,
        index,
        ObjectWord::InstructionTimer,
        timer.wrapping_sub(1),
    );
    if timer != 1 {
        return;
    }
    let bank = object(h, index, ObjectWord::BankSlot).to_le_bytes()[1];
    let base = bank_base(bank);
    let position = base
        + isize::try_from(object(h, index, ObjectWord::InstructionList))
            .expect("HDMA instruction offset");
    // Capture the allocation only at the original program fetch. Each command
    // still performs its operand reads at the original observation point.
    let image = h.native_image();
    let mut script = Script::at(
        image.clone(),
        usize::try_from(position).expect("HDMA program outside ROM"),
    );
    while script.word(0) & 0x8000 != 0 {
        let next = execute(h, script.word(0), index, script.advance(2));
        let Some(next) = next.resolve(&image, base) else {
            return;
        };
        script = next;
    }
    store(h, index, ObjectWord::InstructionTimer, script.word(0));
    store(h, index, ObjectWord::Table, script.word(2));
    // Offset truncation is the original 16-bit program cursor wrap.
    let next = script.advance(4).offset_from(base).to_le_bytes();
    store(
        h,
        index,
        ObjectWord::InstructionList,
        u16::from_le_bytes([next[0], next[1]]),
    );
}

fn execute(h: &mut impl Host, handler: u16, index: usize, script: Script) -> Script {
    use sm_data::hdma_programs::{ChannelRegister, Command};
    let Some(command) = Command::decode(handler) else {
        h.unreachable();
        return Script::Stop;
    };
    match command {
        Command::Nothing => {}
        Command::Delete => {
            store(h, index, ObjectWord::Channels, 0);
            return Script::Stop;
        }
        Command::SetPreInstruction => {
            store(h, index, ObjectWord::PreInstruction, script.word(0));
            h.store_hdma_byte(
                Word::Object {
                    index,
                    field: ObjectWord::PreInstructionBank,
                },
                false,
                script.byte(2),
            );
            return script.advance(3);
        }
        Command::ClearPreInstruction => store(h, index, ObjectWord::PreInstruction, 0x858a),
        Command::Call => {
            let address = script.load24(0);
            if let Some(&(_, action)) = HDMA_CALLS.iter().find(|(at, _)| *at == address) {
                h.call(action, index);
            } else {
                h.unreachable();
            }
            return script.advance(3);
        }
        Command::Goto | Command::GotoRelative | Command::DecrementGoto { .. } => {
            let relative = matches!(
                command,
                Command::GotoRelative | Command::DecrementGoto { relative: true }
            );
            if let Command::DecrementGoto { .. } = command {
                let timer = object(h, index, ObjectWord::Timer);
                store(h, index, ObjectWord::Timer, timer.wrapping_sub(1));
                if timer == 1 {
                    return script.advance(if relative { 1 } else { 2 });
                }
            }
            return if relative {
                script.advance(isize::from(i8::from_ne_bytes([script.byte(0)])))
            } else {
                Script::jump(script.word(0))
            };
        }
        Command::SetTimer => {
            h.store_hdma_byte(
                Word::Object {
                    index,
                    field: ObjectWord::Timer,
                },
                false,
                script.byte(0),
            );
            return script.advance(1);
        }
        Command::SetChannelRegister(register) => {
            let address: u16 = match register {
                ChannelRegister::Control => 0x4300,
                ChannelRegister::Target => 0x4301,
                ChannelRegister::TableBank => 0x4304,
                ChannelRegister::IndirectBank => 0x4307,
            };
            let channel = object(h, index, ObjectWord::BankSlot) & 0xff;
            let value = script.byte(0);
            h.channel_register(address + channel, value);
            return script.advance(1);
        }
        Command::SetTable => {
            store(h, index, ObjectWord::Table, script.word(0));
            return script.advance(2);
        }
        Command::Sleep => {
            let base = bank_base(object(h, index, ObjectWord::BankSlot).to_le_bytes()[1]);
            let cursor = (script.offset_from(base) - 2).to_le_bytes();
            store(
                h,
                index,
                ObjectWord::InstructionList,
                u16::from_le_bytes([cursor[0], cursor[1]]),
            );
            return Script::Stop;
        }
        Command::SetVariableB => store(h, index, ObjectWord::B, 1),
        Command::SetVariableC => store(h, index, ObjectWord::C, 112),
        Command::SelectCeresTable => h.select_ceres_table(9),
        Command::PickBg3Velocity => {
            let random = h.random_number();
            let velocity =
                h.bg3_velocity(isize::try_from(((random >> 1) & 6) >> 1).expect("velocity index"));
            store(h, index, ObjectWord::D, velocity);
        }
        Command::EventScroll => {
            store(h, index, ObjectWord::C, 0);
            store(h, index, ObjectWord::A, 0);
            let opened = h.event_happened(0xa);
            let offset = if opened { 0xff10 } else { 0 };
            store(h, index, ObjectWord::B, offset);
            h.statue_scroll(offset);
            if opened {
                return Script::jump(script.word(0));
            }
            h.statue_tiles(1);
            return script.advance(2);
        }
        Command::InitializePhantoonWave => h.effect(HdmaEffect::InitializePhantoonWave, index),
        Command::InitializeEyeBeam => h.effect(HdmaEffect::InitializeEyeBeam, index),
        Command::ResetWave => h.effect(HdmaEffect::ResetWave, index),
        Command::RestartTitleGradient => h.restart_title_gradient(),
    }
    script
}

pub fn vertical_sky_scroll(host: &mut impl context::Context) {
    sky::vertical_scroll(host)
}

/// Interpreter only, after caller-side music/bomb preparation. The Game caller
/// uses the same kernel through retained fallback callback capabilities.
pub fn run_prepared(host: &mut impl Host, carry: bool) {
    run_objects(host, carry)
}
pub fn reveal_room_block(tile: u16, x: u16, y: u16, host: &mut impl context::Context) {
    xray::reveal_room_block(tile, x, y, host)
}
pub fn calculate_xray_window(
    window: crate::beam_windows::XrayWindow,
    host: &mut impl context::Context,
) {
    beam::calculate_xray_window(window, host)
}
pub fn restore_xray(
    step: crate::player_transitions::effects::XrayRestore,
    host: &mut impl context::Context,
) {
    let index = usize::from(host.hdma_word(crate::hdma::Word::Selected) >> 1);
    use crate::player_transitions::effects::XrayRestore;
    match step {
        XrayRestore::Left => {
            sm_data::table::check_index(index, crate::hdma::OBJECTS);
            xray::restore_left(index, host)
        }
        XrayRestore::Right => xray::restore_right(host),
        XrayRestore::Finish => {
            sm_data::table::check_index(index, crate::hdma::OBJECTS);
            xray::finish_restore(index, host)
        }
    }
}
