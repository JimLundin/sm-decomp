//! HDMA programs use current owner fields and captured object identities.
use crate::enemy_script::{EnemyScript as Script, bank_base};
use crate::hdma::OBJECTS;
use crate::hdma_objects::HdmaEffect;
use crate::hdma_objects::HdmaTable;
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
    + crate::fx_domain::Sources
    + crate::sequence_state::Access
    + crate::effects::SoundEffects
    + crate::hdma_objects::GradientHost
    + crate::xray_background::Host
    + crate::explosion_window::Host
    + crate::room_visuals::StatueHost
    + crate::display::video::Video
{
    /// Word `index` of the BG3 X velocity table ($88:D992), from the imported
    /// visual tables, or the loaded image for another one.
    fn bg3_x_velocity(&self, index: isize) -> u16;
}

/// Run `action` for the selected object on the lease: the slot is captured first, and the inputs are observed
/// afresh on entry and again on return.
pub fn with_current_object<L: HdmaLease, T>(
    lease: &mut L,
    action: impl FnOnce(usize, &mut L) -> T,
) -> T {
    let index = usize::from(lease.hdma_word(crate::hdma::Word::Selected) >> 1);
    sm_data::table::check_index(index, OBJECTS);
    crate::lease::Lease::owned_player_call(lease, |lease| action(index, lease))
}

/// What one interpreter frame needs from the lease.
pub(crate) trait Interpreter: HdmaTable {
    /// Work RAM outside the object table.
    fn byte(&self, at: usize) -> u8;
    fn set_byte(&mut self, at: usize, value: u8);
    fn image(&self) -> RomImage;
    /// The original's unreachable-state failure, reported by the lease.
    fn unreachable(&mut self);
    type Lease: HdmaLease;
    /// The lease itself, for the compatibility effects it still leaves for.
    fn lease(&mut self) -> &mut Self::Lease;
    /// Run `action` on the lease, after a preflight, with the inputs observed afresh on entry and
    /// again on return.
    fn on_lease<T>(&mut self, action: impl FnOnce(&mut Self::Lease) -> T) -> T;
    /// Run HDMA effect `effect` for the object in byte slot `slot` on the
    /// lease.
    fn effect(&mut self, effect: HdmaEffect, slot: u16) {
        self.on_lease(|lease| crate::hdma_effects::apply(effect, slot, lease));
    }
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.byte(at), self.byte(at + 1)])
    }
    /// Run `action` for object `index` on the lease, after checking the index.
    fn on_object<T>(
        &mut self,
        index: usize,
        action: impl FnOnce(usize, &mut Self::Lease) -> T,
    ) -> T {
        sm_data::table::check_index(index, OBJECTS);
        self.on_lease(|lease| action(index, lease))
    }
    fn set_word(&mut self, at: usize, value: u16) {
        let [low, high] = value.to_le_bytes();
        self.set_byte(at, low);
        self.set_byte(at + 1, high);
    }
}

struct Leased<'a, L>(&'a mut L);

impl<L: HdmaLease> HdmaTable for Leased<'_, L> {
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

impl<L: HdmaLease> Interpreter for Leased<'_, L> {
    fn byte(&self, at: usize) -> u8 {
        self.0.byte(at)
    }
    fn set_byte(&mut self, at: usize, value: u8) {
        self.0.set_byte(at, value);
    }
    fn image(&self) -> RomImage {
        crate::cartridge::NativeImage::native_image(&*self.0)
    }
    fn lease(&mut self) -> &mut L {
        self.0
    }
    fn unreachable(&mut self) {
        crate::diagnostics::Fatal::unreachable(self.0)
    }
    type Lease = L;
    fn on_lease<T>(&mut self, action: impl FnOnce(&mut L) -> T) -> T {
        crate::lease::Lease::owned_player_call(self.0, action)
    }
}

/// [`run_objects`] on a lease already open.
pub fn run_objects_on<L: HdmaLease>(handler_carry: bool, lease: &mut L) {
    run(&mut Leased(lease), handler_carry);
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
    use sm_data::hdma_programs::Command;
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
            let [bank_slot, _] = crate::hdma_word!(it, index, BankSlot).to_le_bytes();
            let value = script.byte(0);
            crate::hdma_objects::write_channel_byte(bank_slot, register, value, it.lease());
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
            let row = isize::from((((random >> 1) & 6) >> 1).cast_signed());
            let velocity = it.on_lease(|lease| lease.bg3_x_velocity(row));
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
        Command::RestartTitleGradient => it.on_lease(crate::hdma_objects::restart_title_gradient),
    }
    script
}
