//! HDMA programs use current owner fields and captured object identities.
use crate::hdma::{HdmaObjects, OBJECTS};
use crate::hdma_dispatch::DispatchHost;
use crate::hdma_objects::Bridged;
use crate::hdma_objects::HdmaTable;
use crate::menus::host::{Host, MenuRuntime as _};
use sm_data::cartridge::RomImage;

/// The owned-Session view the interpreter runs on.
pub trait HdmaLease:
    crate::session_access::SessionAccess
    + crate::runtime_access::SessionRam
    + crate::cartridge::NativeImage
    + HdmaTable
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
    type Lease<'a>: HdmaLease + crate::owned_hdma_update::context::Context
    where
        Self: 'a;
    /// One owned-Session view; open it only while `runtime_admissible`.
    fn hdma_lease(&mut self) -> Self::Lease<'_>;
}

/// What one interpreter frame needs from either the lease or the RAM copy.
pub(crate) trait Interpreter: HdmaTable {
    type Game: DispatchHost + ?Sized;
    /// Work RAM outside the object table.
    fn byte(&self, at: usize) -> u8;
    fn set_byte(&mut self, at: usize, value: u8);
    fn image(&self) -> RomImage;
    fn foreign<T>(&mut self, action: impl FnOnce(&mut Self::Game) -> T) -> T;
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
        crate::owned_hdma_update::run_prepared(&mut game.hdma_lease(), handler_carry);
    } else {
        let bridge = Bridged::load(game);
        crate::owned_hdma_update::run_prepared(&mut SharedFallback(bridge), handler_carry);
    }
}

struct SharedFallback<I>(I);
impl<I: Interpreter> HdmaTable for SharedFallback<I> {
    fn hdma_word(&self, word: crate::hdma::Word) -> u16 {
        self.0.hdma_word(word)
    }
    fn store_hdma_word(
        &mut self,
        word: crate::hdma::Word,
        value: u16,
        order: crate::scene::state::WordOrder,
    ) {
        self.0.store_hdma_word(word, value, order)
    }
    fn store_hdma_byte(&mut self, word: crate::hdma::Word, high: bool, value: u8) {
        self.0.store_hdma_byte(word, high, value)
    }
}
impl<I: Interpreter> crate::cartridge::NativeImage for SharedFallback<I> {
    fn native_image(&self) -> RomImage {
        self.0.image()
    }
}
impl<I: Interpreter> crate::owned_hdma_update::Host for SharedFallback<I> {
    fn enable_byte(&self) -> u8 {
        self.0.byte(0x85)
    }
    fn store_enable_byte(&mut self, value: u8) {
        self.0.set_byte(0x85, value)
    }
    fn random_number(&self) -> u16 {
        self.0.word(crate::ram::words::RANDOM_NUMBER)
    }
    fn event_happened(&mut self, event: u16) -> bool {
        self.0
            .foreign(|game| crate::progress::leased::event_has_happened(event, game))
            != 0
    }
    fn pre_instruction(
        &mut self,
        action: crate::hdma_dispatch::HdmaPreAction,
        captured: usize,
        carry: bool,
    ) {
        let address = crate::hdma_dispatch::HDMA_PRE_ACTIONS
            .iter()
            .find(|(_, candidate)| *candidate == action)
            .expect("registered preinstruction")
            .0;
        crate::hdma_dispatch::run_pre_action(address, captured, &mut self.0, carry);
    }
    fn call(&mut self, action: crate::hdma_dispatch::HdmaCall, captured: usize) {
        let address = crate::hdma_dispatch::HDMA_CALLS
            .iter()
            .find(|(_, candidate)| *candidate == action)
            .expect("registered call")
            .0;
        crate::hdma_dispatch::run_call(address, captured, &mut self.0);
    }
    fn effect(&mut self, effect: crate::hdma_objects::HdmaEffect, captured: usize) {
        let slot = 2 * u16::try_from(captured).expect("captured HDMA effect");
        self.0
            .foreign(|game| crate::hdma_effects::apply_leased(effect, slot, game));
    }
    fn channel_register(&mut self, address: u16, value: u8) {
        self.0
            .foreign(|game| crate::hdma_objects::write_channel_register(address, value, game));
    }
    fn bg3_velocity(&mut self, index: isize) -> u16 {
        self.0.foreign(|game| {
            crate::imported::word(
                &game.visual_tables().bg3_x_velocities,
                game,
                sm_data::hdma_objects::BG3_X_VELOCITY_ADDRESS,
                index,
            )
        })
    }
    fn select_ceres_table(&mut self, value: u16) {
        self.0
            .set_word(crate::ram::words::HDMA_DATA_TABLE_IN_CERES, value)
    }
    fn statue_scroll(&mut self, value: u16) {
        let [first, second] = value.to_ne_bytes();
        self.0.set_byte(0x9e00, first);
        self.0.set_byte(0x9e01, second);
    }
    fn statue_tiles(&mut self, value: u16) {
        let [first, second] = value.to_ne_bytes();
        self.0.set_byte(0xcd20, first);
        self.0.set_byte(0xcd21, second);
    }
    fn restart_title_gradient(&mut self) {
        self.0.foreign(crate::hdma_objects::restart_title_gradient)
    }
    fn unreachable(&mut self) {
        self.0.foreign(|game| game.rtl_unreachable());
    }
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
