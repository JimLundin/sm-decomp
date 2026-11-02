//! Ordered decoding on the suspended-work cursor and current owned actor image.
#![forbid(unsafe_code)]
use crate::actors::owned_execution::PreparedActorState;
use crate::decompression::{Output, decode};
pub trait Host: crate::actors::boss_fields_owned::State {
    fn loaded_decompression_image(&self) -> sm_data::cartridge::RomImage;
    fn invalid_decompression_address(&mut self, address: u32);
}
impl Host for PreparedActorState<'_> {
    fn loaded_decompression_image(&self) -> sm_data::cartridge::RomImage {
        self.allocations.loaded().clone()
    }
    fn invalid_decompression_address(&mut self, address: u32) {
        self.aliases
            .report_invalid_decompression_address(self.owner, address);
    }
}
pub fn next_byte<H: Host>(game: &mut H) -> u8 {
    let address = game.session().suspended_work.decompression_source;
    if address & 0x8000 == 0 {
        game.invalid_decompression_address(address);
        game.refresh_inputs();
    }
    let at = usize::try_from(sm_data::cartridge::lorom_offset(address | 0x8000))
        .expect("decompression source");
    let value = game.loaded_decompression_image().read(at);
    game.session_mut().suspended_work.decompression_source = address.wrapping_add(1);
    if address & 0xffff == 0xffff {
        let source = game.session().suspended_work.decompression_source;
        game.session_mut().suspended_work.decompression_source = source.wrapping_add(0x8000);
    }
    value
}
struct RamOutput<'a, H: Host> {
    game: &'a mut H,
    base: usize,
    produced: usize,
}
impl<H: Host> Output for RamOutput<'_, H> {
    fn next_byte(&mut self) -> u8 {
        next_byte(self.game)
    }
    fn read(&mut self, position: i32) -> u8 {
        let at = self
            .base
            .checked_add_signed(isize::try_from(position).expect("decoded position"))
            .expect("decoded source");
        crate::actors::boss_fields_owned::read_byte(self.game, at)
    }
    fn write(&mut self, position: i32, value: u8, _back_reference: bool) {
        let at = self
            .base
            .checked_add_signed(isize::try_from(position).expect("decoded position"))
            .expect("decoded destination");
        crate::actors::boss_fields_owned::store_byte(self.game, at, value);
        self.produced = self
            .produced
            .max(usize::try_from(position).expect("RAM output position") + 1);
    }
}
pub fn decompress<H: Host>(game: &mut H, source: u32, destination: usize) -> usize {
    game.session_mut().suspended_work.decompression_source = source;
    let mut output = RamOutput {
        game,
        base: destination,
        produced: 0,
    };
    decode(0, &mut output);
    output.produced
}
