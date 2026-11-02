//! Ordered presentation producers shared by the migrated actor kernels.
#![forbid(unsafe_code)]
use crate::actors::remaining_core::Core;

/// Native acquisition is a species input, with its original lazy diagnostic.
pub trait NativeCapture: Core {
    fn native_actor_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord;
}
impl NativeCapture for crate::actors::owned_execution::PreparedActorState<'_> {
    fn native_actor_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord {
        crate::actors::owned_execution::PreparedActorState::native_actor_record(self, bank, address)
    }
}

/// Each source word reacquires the native allocation after the original
/// low-window diagnostic. Destination masking precedes the palette store.
pub fn native_palette<H: NativeCapture>(
    host: &mut H,
    target: bool,
    mut destination: u16,
    bank: u8,
    mut source: u16,
    count: u16,
) {
    let mut left = i32::from(count);
    loop {
        let value = host.native_actor_record(bank, source).word(0);
        host.set_remaining_color(target, usize::from(destination & !1), value, false);
        destination = destination.wrapping_add(2);
        source = source.wrapping_add(2);
        left -= 1;
        if left == 0 {
            break;
        }
    }
}
