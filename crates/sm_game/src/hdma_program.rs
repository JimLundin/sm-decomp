//! HDMA programs use current owner fields and captured object identities.
use crate::enemy_script::{EnemyScript as Script, bank_base};
use crate::hdma::{HdmaObjects, OBJECTS};
use crate::hdma_dispatch::DispatchHost;
use crate::hdma_objects::HdmaEffect;
use crate::hdma_objects::HdmaTable;
use crate::menus::host::Host;
use sm_data::cartridge::RomImage;

/// The owned-Session view the interpreter runs on.
pub trait HdmaLease:
    crate::session_access::SessionAccess
    + crate::runtime_access::SessionRam
    + crate::cartridge::NativeImage
    + HdmaTable
    + crate::diagnostics::Fatal
    + crate::hdma_effects::HdmaEffects
    + crate::actors::mother_brain_rainbow::Host
    + crate::player_transitions::effects::SamusLease
    + crate::liquid_effects::Host
    + crate::progress::FlagHost
    + crate::actors::boss_lease::BossState
    + crate::fx_lighting::Access
{
    type Game: DispatchHost + ?Sized;
    /// Run `action` on the compatibility host with the Session published.
    fn foreign<T>(&mut self, action: impl FnOnce(&mut Self::Game) -> T) -> T;
}

/// Capture the selected slot before publishing to the compatibility host.
pub fn with_current_object<L: HdmaLease + ?Sized, T>(
    lease: &mut L,
    action: impl FnOnce(usize, &mut L::Game) -> T,
) -> T {
    let index = usize::from(lease.hdma_word(crate::hdma::Word::Selected) >> 1);
    sm_data::table::check_index(index, OBJECTS);
    lease.foreign(|game| action(index, game))
}

/// A host that can lease its Session to the interpreter.
pub trait LeaseHdma: crate::runtime_access::RuntimeAccess {
    type Lease<'a>: HdmaLease
    where
        Self: 'a;
    /// One owned-Session view for the frame's objects.
    fn hdma_lease(&mut self) -> Self::Lease<'_>;
}

/// What one interpreter frame needs from the lease.
pub(crate) trait Interpreter: HdmaTable {
    type Game: DispatchHost + ?Sized;
    /// Work RAM outside the object table.
    fn byte(&self, at: usize) -> u8;
    fn set_byte(&mut self, at: usize, value: u8);
    fn image(&self) -> RomImage;
    fn foreign<T>(&mut self, action: impl FnOnce(&mut Self::Game) -> T) -> T;
    /// The original's unreachable-state failure, reported by the lease.
    fn unreachable(&mut self);
    type Lease: HdmaLease<Game = Self::Game>;
    /// Run `action` on the lease where a foreign call ran it on the game: the
    /// foreign entry's preflight, with the inputs observed afresh on entry and
    /// again on return.
    fn on_lease<T>(&mut self, action: impl FnOnce(&mut Self::Lease) -> T) -> T;
    /// Run HDMA effect `effect` for the object in byte slot `slot` on the
    /// lease, where a nested lease inside a foreign call ran it.
    fn effect(&mut self, effect: HdmaEffect, slot: u16) {
        self.on_lease(|lease| crate::hdma_effects::apply(effect, slot, lease));
    }
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.byte(at), self.byte(at + 1)])
    }
    fn with_object<T>(
        &mut self,
        index: usize,
        action: impl FnOnce(usize, &mut Self::Game) -> T,
    ) -> T {
        sm_data::table::check_index(index, OBJECTS);
        self.foreign(|game| action(index, game))
    }
    fn set_word(&mut self, at: usize, value: u16) {
        let [low, high] = value.to_le_bytes();
        self.set_byte(at, low);
        self.set_byte(at + 1, high);
    }
}

struct Leased<L>(L);

impl<L: HdmaLease> HdmaTable for Leased<L> {
    fn hdma_word(&self, word: crate::hdma::Word) -> u16 {
        self.0.hdma_word(word)
    }
    fn store_hdma_word(
        &mut self,
        word: crate::hdma::Word,
        value: u16,
        order: crate::scene::state::WordOrder,
    ) {
        self.0.store_hdma_word(word, value, order);
    }
    fn store_hdma_byte(&mut self, word: crate::hdma::Word, high: bool, value: u8) {
        self.0.store_hdma_byte(word, high, value);
    }
}

impl<L: HdmaLease> Interpreter for Leased<L> {
    type Game = L::Game;
    fn byte(&self, at: usize) -> u8 {
        self.0.byte(at)
    }
    fn set_byte(&mut self, at: usize, value: u8) {
        self.0.set_byte(at, value);
    }
    fn image(&self) -> RomImage {
        crate::cartridge::NativeImage::native_image(&self.0)
    }
    fn foreign<T>(&mut self, action: impl FnOnce(&mut L::Game) -> T) -> T {
        self.0.foreign(action)
    }
    fn unreachable(&mut self) {
        crate::diagnostics::Fatal::unreachable(&mut self.0)
    }
    type Lease = L;
    fn on_lease<T>(&mut self, action: impl FnOnce(&mut L) -> T) -> T {
        crate::lease::Lease::owned_player_call(&mut self.0, action)
    }
}

/// The table's RAM words, in `session_encoding` order.
pub(crate) fn table_words(objects: &mut HdmaObjects, mut each: impl FnMut(usize, &mut u16)) {
    each(0x18b0, &mut objects.enabled);
    each(0x18b2, &mut objects.selected);
    for (index, object) in objects.objects.iter_mut().enumerate() {
        let at = 0x18b4 + 2 * index;
        for (field, word) in [
            &mut object.channels,
            &mut object.bank_slot,
            &mut object.instruction_list,
            &mut object.table,
            &mut object.instruction_timer,
            &mut object.pre_instruction,
            &mut object.pre_instruction_bank,
            &mut object.timer,
            &mut object.a,
            &mut object.b,
            &mut object.c,
            &mut object.d,
            &mut object.e,
            &mut object.f,
        ]
        .into_iter()
        .enumerate()
        {
            each(at + 12 * field, word);
        }
    }
}

/// Run every active object's program once, as the enabled branch of
/// `update_hdma`.
///
/// The objects always run on the lease. A host that cannot open one cannot
/// publish this frame's Session either, so no completed frame ran the RAM
/// copy the interpreter once fell back to.
pub fn run_objects<H: DispatchHost + LeaseHdma + ?Sized>(handler_carry: bool, game: &mut H) {
    run(&mut Leased(game.hdma_lease()), handler_carry);
}

fn run<I: Interpreter>(it: &mut I, handler_carry: bool) {
    it.set_byte(0x85, 0);
    let mut slot = 0usize;
    loop {
        it.store_hdma_word(
            crate::hdma::Word::Selected,
            slot as u16,
            crate::scene::state::WordOrder::Little,
        );
        // The loop is the only writer of `selected`, so `slot` stays an
        // even offset below 12 and names object `slot / 2`.
        let channels = crate::hdma_word!(&*it, slot >> 1, Channels) as u8;
        if channels != 0 {
            let enabled = it.byte(0x85) | channels;
            it.set_byte(0x85, enabled);
            run_program(slot >> 1, it, handler_carry);
            slot = usize::from(it.hdma_word(crate::hdma::Word::Selected));
        }
        slot += 2;
        if slot == 2 * OBJECTS {
            break;
        }
    }
}

fn run_program<I: Interpreter>(index: usize, it: &mut I, handler_carry: bool) {
    let pre_instruction = (u32::from(crate::hdma_word!(it, index, PreInstructionBank)) << 16)
        | u32::from(crate::hdma_word!(it, index, PreInstruction));
    crate::hdma_dispatch::run_pre_action(pre_instruction, index, it, handler_carry);
    let timer = crate::hdma_word!(it, index, InstructionTimer);
    crate::hdma_word!(it, index, InstructionTimer => timer.wrapping_sub(1));
    if timer != 1 {
        return;
    }
    let base = bank_base((crate::hdma_word!(it, index, BankSlot) >> 8) as u8);
    let program = crate::hdma_word!(it, index, InstructionList);
    let image = it.image();
    let position = base + program as isize;
    let mut script = Script::at(
        image.clone(),
        usize::try_from(position).expect("HDMA program outside ROM"),
    );
    while script.word(0) & 0x8000 != 0 {
        let next = execute(script.word(0), index, script.advance(2), it);
        let Some(next) = next.resolve(&image, base) else {
            return;
        };
        script = next;
    }
    crate::hdma_word!(it, index, InstructionTimer => script.word(0));
    crate::hdma_word!(it, index, Table => script.word(2));
    crate::hdma_word!(it, index, InstructionList => script.advance(4).offset_from(base) as u16);
}

fn execute<I: Interpreter>(handler: u16, index: usize, script: Script, it: &mut I) -> Script {
    use crate::display::registers::{A1B0, BBAD0, DAS00, DMAP0};
    use sm_data::hdma_programs::{ChannelRegister, Command};
    let slot = 2 * index as u16;
    let Some(command) = Command::decode(handler) else {
        it.unreachable();
        return Script::Stop;
    };
    match command {
        Command::Nothing => {}
        Command::Delete => {
            crate::hdma_word!(it, index, Channels => 0);
            return Script::Stop;
        }
        Command::SetPreInstruction => {
            crate::hdma_word!(it, index, PreInstruction => script.word(0));
            crate::hdma_byte!(it, index, PreInstructionBank, false => script.byte(2));
            return script.advance(3);
        }
        Command::ClearPreInstruction => crate::hdma_word!(it, index, PreInstruction => 0x858a),
        Command::Call => {
            let address = script.load24(0);
            crate::hdma_dispatch::run_call(address, index, it);
            return script.advance(3);
        }
        Command::Goto | Command::GotoRelative | Command::DecrementGoto { .. } => {
            let relative = matches!(
                command,
                Command::GotoRelative | Command::DecrementGoto { relative: true }
            );
            if let Command::DecrementGoto { .. } = command {
                let timer = crate::hdma_word!(it, index, Timer);
                crate::hdma_word!(it, index, Timer => timer.wrapping_sub(1));
                if timer == 1 {
                    return script.advance(if relative { 1 } else { 2 });
                }
            }
            return if relative {
                script.advance(script.byte(0) as i8 as isize)
            } else {
                Script::jump(script.word(0))
            };
        }
        Command::SetTimer => {
            crate::hdma_byte!(it, index, Timer, false => script.byte(0));
            return script.advance(1);
        }
        Command::SetChannelRegister(register) => {
            let register = match register {
                ChannelRegister::Control => DMAP0,
                ChannelRegister::Target => BBAD0,
                ChannelRegister::TableBank => A1B0,
                ChannelRegister::IndirectBank => DAS00,
            } as u16;
            let channel = crate::hdma_word!(it, index, BankSlot) & 0xff;
            let value = script.byte(0);
            it.foreign(|game| {
                crate::hdma_objects::write_channel_register(register + channel, value, game)
            });
            return script.advance(1);
        }
        Command::SetTable => {
            crate::hdma_word!(it, index, Table => script.word(0));
            return script.advance(2);
        }
        Command::Sleep => {
            let base = bank_base((crate::hdma_word!(it, index, BankSlot) >> 8) as u8);
            crate::hdma_word!(it, index, InstructionList => (script.offset_from(base) - 2) as u16);
            return Script::Stop;
        }
        Command::SetVariableB => crate::hdma_word!(it, index, B => 1),
        Command::SetVariableC => crate::hdma_word!(it, index, C => 112),
        Command::SelectCeresTable => it.set_word(crate::ram::words::HDMA_DATA_TABLE_IN_CERES, 9),
        Command::PickBg3Velocity => {
            let random = it.word(crate::ram::words::RANDOM_NUMBER);
            let velocity = it.foreign(|game| {
                crate::imported::word(
                    &game.visual_tables().bg3_x_velocities,
                    game,
                    sm_data::hdma_objects::BG3_X_VELOCITY_ADDRESS,
                    ((random >> 1 & 6) >> 1) as isize,
                )
            });
            crate::hdma_word!(it, index, D => velocity);
        }
        Command::EventScroll => {
            crate::hdma_word!(it, index, C => 0);
            crate::hdma_word!(it, index, A => 0);
            // The event flag ran on a nested lease of the game; it runs on this one.
            let opened = it.on_lease(|lease| crate::progress::event_happened(lease, 0xa));
            let offset: u16 = if opened { 0xff10 } else { 0 };
            crate::hdma_word!(it, index, B => offset);
            let [low, high] = offset.to_ne_bytes();
            it.set_byte(0x9e00, low);
            it.set_byte(0x9e01, high);
            if opened {
                return Script::jump(script.word(0));
            }
            let [low, high] = 1u16.to_ne_bytes();
            it.set_byte(0xcd20, low);
            it.set_byte(0xcd21, high);
            return script.advance(2);
        }
        Command::InitializePhantoonWave => it.effect(HdmaEffect::InitializePhantoonWave, slot),
        Command::InitializeEyeBeam => it.effect(HdmaEffect::InitializeEyeBeam, slot),
        Command::ResetWave => it.effect(HdmaEffect::ResetWave, slot),
        Command::RestartTitleGradient => it.foreign(crate::hdma_objects::restart_title_gradient),
    }
    script
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ram::{word_arrays as arrays, words};

    #[test]
    fn table_words_follow_the_ram_names() {
        let mut objects = HdmaObjects {
            enabled: 1,
            selected: 2,
            ..HdmaObjects::default()
        };
        for (index, object) in objects.objects.iter_mut().enumerate() {
            let code = |field: u16| 0x100 * (index as u16 + 1) + field;
            *object = crate::hdma::HdmaObject {
                channels: code(0),
                bank_slot: code(1),
                instruction_list: code(2),
                table: code(3),
                instruction_timer: code(4),
                pre_instruction: code(5),
                pre_instruction_bank: code(6),
                timer: code(7),
                a: code(8),
                b: code(9),
                c: code(10),
                d: code(11),
                e: code(12),
                f: code(13),
            };
        }
        let fields = [
            arrays::HDMA_OBJECT_CHANNELS_BITMASK,
            arrays::HDMA_OBJECT_BANK_SLOT,
            arrays::HDMA_OBJECT_INSTRUCTION_LIST_POINTERS,
            arrays::HDMA_OBJECT_TABLE_POINTERS,
            arrays::HDMA_OBJECT_INSTRUCTION_TIMERS,
            arrays::HDMA_OBJECT_PRE_INSTRUCTIONS,
            arrays::HDMA_OBJECT_PRE_INSTRUCTION_BANK,
            arrays::HDMA_OBJECT_TIMERS,
            arrays::HDMA_OBJECT_A,
            arrays::HDMA_OBJECT_B,
            arrays::HDMA_OBJECT_C,
            arrays::HDMA_OBJECT_D,
            arrays::HDMA_OBJECT_E,
            arrays::HDMA_OBJECT_F,
        ];
        let mut seen = Vec::new();
        table_words(&mut objects, |at, word| {
            let expected = match *word {
                1 => words::HDMA_OBJECTS_ENABLE_FLAG,
                2 => words::HDMA_OBJECT_INDEX,
                code => {
                    let index = usize::from(code >> 8) - 1;
                    fields[usize::from(code & 0xff)] + 2 * index
                }
            };
            assert_eq!(at, expected, "word {word:#06x}");
            seen.push(at);
        });
        seen.sort();
        assert_eq!(seen, (0x18b0..0x195c).step_by(2).collect::<Vec<_>>());
    }
}
