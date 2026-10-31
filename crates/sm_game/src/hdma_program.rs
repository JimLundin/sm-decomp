//! The per-frame HDMA object interpreter on the Session-owned object table
//! (step 2 of `crate::hdma`).
//!
//! The interpreter reads and writes `HdmaObjects` as typed fields. It runs
//! on one owned-Session lease per frame when the host can open one, and
//! otherwise on a copy of the table decoded from RAM. Host callbacks (the
//! pre-instructions, calls and effects) still run on the compatibility host:
//! the lease publishes the Session around each one and imports it after, and
//! the RAM copy is stored before each one and reloaded after. No table value
//! is held across a callback.

use crate::enemy_script::{EnemyScript as Script, bank_base};
use crate::hdma::{HdmaObject, HdmaObjects, OBJECTS};
use crate::hdma_dispatch::DispatchHost;
use crate::hdma_objects::Bridged;
use crate::hdma_objects::HdmaEffect;
use crate::menus::host::{Host, MenuRuntime as _};
use sm_data::cartridge::RomImage;

/// The owned-Session view the interpreter runs on.
pub trait HdmaLease:
    crate::session_access::SessionAccess
    + crate::runtime_access::SessionRam
    + crate::cartridge::NativeImage
{
    type Game: DispatchHost + ?Sized;
    /// Run `action` on the compatibility host with the Session published.
    fn foreign<T>(&mut self, action: impl FnOnce(&mut Self::Game) -> T) -> T;
}

/// Run `action` on the lease's host with the object the interpreter is
/// running lent to it, as `Interpreter::with_object`.
pub fn with_current_object<L: HdmaLease + ?Sized, T>(
    lease: &mut L,
    action: impl FnOnce(&mut HdmaObject, &mut L::Game) -> T,
) -> T {
    let index = usize::from(lease.session().hdma.selected >> 1);
    let mut object = lease.session().hdma.objects[index];
    let result = lease.foreign(|game| action(&mut object, game));
    lease.session_mut().hdma.objects[index] = object;
    result
}

/// A host that can lease its Session to the interpreter.
pub trait LeaseHdma: crate::runtime_access::RuntimeAccess {
    type Lease<'a>: HdmaLease
    where
        Self: 'a;
    /// One owned-Session view; open it only while `runtime_admissible`.
    fn hdma_lease(&mut self) -> Self::Lease<'_>;
}

/// What one interpreter frame needs from either the lease or the RAM copy.
pub(crate) trait Interpreter {
    type Game: DispatchHost + ?Sized;
    fn objects(&mut self) -> &mut HdmaObjects;
    /// Work RAM outside the object table.
    fn byte(&self, at: usize) -> u8;
    fn set_byte(&mut self, at: usize, value: u8);
    fn image(&self) -> RomImage;
    fn foreign<T>(&mut self, action: impl FnOnce(&mut Self::Game) -> T) -> T;
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.byte(at), self.byte(at + 1)])
    }
    /// Run `action` on the compatibility host with object `index` lent to it
    /// as typed fields. `action` must not read the host's copy of that object;
    /// its words are restored from the lent object after.
    fn with_object<T>(
        &mut self,
        index: usize,
        action: impl FnOnce(&mut HdmaObject, &mut Self::Game) -> T,
    ) -> T {
        let mut object = self.objects().objects[index];
        let result = self.foreign(|game| action(&mut object, game));
        self.objects().objects[index] = object;
        result
    }
    fn set_word(&mut self, at: usize, value: u16) {
        let [low, high] = value.to_le_bytes();
        self.set_byte(at, low);
        self.set_byte(at + 1, high);
    }
}

struct Leased<L>(L);

impl<L: HdmaLease> Interpreter for Leased<L> {
    type Game = L::Game;
    fn objects(&mut self) -> &mut HdmaObjects {
        &mut self.0.session_mut().hdma
    }
    fn byte(&self, at: usize) -> u8 {
        self.0.byte(at)
    }
    fn set_byte(&mut self, at: usize, value: u8) {
        self.0.set_byte(at, value);
    }
    fn image(&self) -> RomImage {
        self.0.native_image()
    }
    fn foreign<T>(&mut self, action: impl FnOnce(&mut L::Game) -> T) -> T {
        self.0.foreign(action)
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
pub fn run_objects<H: DispatchHost + LeaseHdma + ?Sized>(handler_carry: bool, game: &mut H) {
    if game.runtime_admissible() {
        run(&mut Leased(game.hdma_lease()), handler_carry);
    } else {
        let mut bridge = Bridged::load(game);
        run(&mut bridge, handler_carry);
        bridge.store();
    }
}

fn run<I: Interpreter>(it: &mut I, handler_carry: bool) {
    it.set_byte(0x85, 0);
    let mut slot = 0usize;
    loop {
        it.objects().selected = slot as u16;
        // The loop is the only writer of `selected`, so `slot` stays an
        // even offset below 12 and names object `slot / 2`.
        let channels = it.objects().objects[slot >> 1].channels as u8;
        if channels != 0 {
            let enabled = it.byte(0x85) | channels;
            it.set_byte(0x85, enabled);
            run_program(slot >> 1, it, handler_carry);
            slot = usize::from(it.objects().selected);
        }
        slot += 2;
        if slot == 2 * OBJECTS {
            break;
        }
    }
}

fn run_program<I: Interpreter>(index: usize, it: &mut I, handler_carry: bool) {
    let object = it.objects().objects[index];
    let pre_instruction =
        (u32::from(object.pre_instruction_bank) << 16) | u32::from(object.pre_instruction);
    crate::hdma_dispatch::run_pre_action(pre_instruction, index, it, handler_carry);
    let object = &mut it.objects().objects[index];
    let timer = object.instruction_timer;
    object.instruction_timer = timer.wrapping_sub(1);
    if timer != 1 {
        return;
    }
    let base = bank_base((object.bank_slot >> 8) as u8);
    let program = object.instruction_list;
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
    let object = &mut it.objects().objects[index];
    object.instruction_timer = script.word(0);
    object.table = script.word(2);
    object.instruction_list = script.advance(4).offset_from(base) as u16;
}

fn set_low_byte(word: &mut u16, value: u8) {
    *word = (*word & 0xff00) | u16::from(value);
}

fn execute<I: Interpreter>(handler: u16, index: usize, script: Script, it: &mut I) -> Script {
    use crate::display::registers::{A1B0, BBAD0, DAS00, DMAP0};
    use sm_data::hdma_programs::{ChannelRegister, Command};
    let slot = 2 * index as u16;
    let Some(command) = Command::decode(handler) else {
        it.foreign(|game| game.rtl_unreachable());
        return Script::Stop;
    };
    match command {
        Command::Nothing => {}
        Command::Delete => {
            it.objects().objects[index].channels = 0;
            return Script::Stop;
        }
        Command::SetPreInstruction => {
            let object = &mut it.objects().objects[index];
            object.pre_instruction = script.word(0);
            set_low_byte(&mut object.pre_instruction_bank, script.byte(2));
            return script.advance(3);
        }
        Command::ClearPreInstruction => it.objects().objects[index].pre_instruction = 0x858a,
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
                let object = &mut it.objects().objects[index];
                let timer = object.timer;
                object.timer = timer.wrapping_sub(1);
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
            set_low_byte(&mut it.objects().objects[index].timer, script.byte(0));
            return script.advance(1);
        }
        Command::SetChannelRegister(register) => {
            let register = match register {
                ChannelRegister::Control => DMAP0,
                ChannelRegister::Target => BBAD0,
                ChannelRegister::TableBank => A1B0,
                ChannelRegister::IndirectBank => DAS00,
            } as u16;
            let channel = it.objects().objects[index].bank_slot & 0xff;
            let value = script.byte(0);
            it.foreign(|game| {
                crate::hdma_objects::write_channel_register(register + channel, value, game)
            });
            return script.advance(1);
        }
        Command::SetTable => {
            it.objects().objects[index].table = script.word(0);
            return script.advance(2);
        }
        Command::Sleep => {
            let base = bank_base((it.objects().objects[index].bank_slot >> 8) as u8);
            it.objects().objects[index].instruction_list = (script.offset_from(base) - 2) as u16;
            return Script::Stop;
        }
        Command::SetVariableB => it.objects().objects[index].b = 1,
        Command::SetVariableC => it.objects().objects[index].c = 112,
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
            it.objects().objects[index].d = velocity;
        }
        Command::EventScroll => {
            let object = &mut it.objects().objects[index];
            object.c = 0;
            object.a = 0;
            let opened =
                it.foreign(|game| crate::progress::leased::event_has_happened(0xa, game)) != 0;
            let offset: u16 = if opened { 0xff10 } else { 0 };
            it.objects().objects[index].b = offset;
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
        Command::InitializePhantoonWave => it.foreign(|game| {
            crate::hdma_effects::apply_leased(HdmaEffect::InitializePhantoonWave, slot, game)
        }),
        Command::InitializeEyeBeam => it.foreign(|game| {
            crate::hdma_effects::apply_leased(HdmaEffect::InitializeEyeBeam, slot, game)
        }),
        Command::ResetWave => {
            it.foreign(|game| crate::hdma_effects::apply_leased(HdmaEffect::ResetWave, slot, game))
        }
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
