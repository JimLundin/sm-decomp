//! Boss native palette copies, querying each source before its ordered store.
#![forbid(unsafe_code)]
pub trait Host {
    fn native_palette_record(&mut self, bank: u8, source: u16) -> sm_data::cartridge::RomRecord;
    fn store_live_palette(&mut self, index: u16, value: u16);
    fn store_target_palette(&mut self, index: u16, value: u16);
}
fn copy<H: Host + ?Sized>(
    mut destination: u16,
    bank: u8,
    mut source: u16,
    count: u16,
    game: &mut H,
    store: fn(&mut H, u16, u16),
) {
    let mut remaining = i32::from(count);
    loop {
        let value = game.native_palette_record(bank, source).word(0);
        store(game, destination >> 1, value);
        destination = destination.wrapping_add(2);
        source = source.wrapping_add(2);
        remaining -= 1;
        if remaining == 0 {
            break;
        }
    }
}
pub fn copy_rom_colors_to_palette<H: Host + ?Sized>(
    destination: u16,
    bank: u8,
    source: u16,
    count: u16,
    game: &mut H,
) {
    copy(destination, bank, source, count, game, |g, i, v| {
        g.store_live_palette(i, v)
    });
}
pub fn copy_rom_colors_to_target_palette<H: Host + ?Sized>(
    bank: u8,
    destination: u16,
    source: u16,
    count: u16,
    game: &mut H,
) {
    copy(destination, bank, source, count, game, |g, i, v| {
        g.store_target_palette(i, v)
    });
}
impl Host for crate::actors::owned_execution::PreparedActorState<'_> {
    fn native_palette_record(&mut self, bank: u8, source: u16) -> sm_data::cartridge::RomRecord {
        self.native_actor_record(bank, source)
    }
    fn store_live_palette(&mut self, index: u16, value: u16) {
        crate::enemy_projectiles::host::Graphics::set_palette_buffer(
            self,
            isize::try_from(index).expect("palette index"),
            value,
        );
    }
    fn store_target_palette(&mut self, index: u16, value: u16) {
        use crate::session_access::SessionAccess;
        if let Some(target) = self
            .session_mut()
            .presentation
            .visual_mut()
            .targets
            .get_mut(usize::from(index))
        {
            *target = value;
            self.refresh_inputs();
        } else {
            let at = 0xc200 + (2 * usize::from(index));
            let [low, high] = value.to_le_bytes();
            self.set_current_byte(at, low);
            self.set_current_byte(at + 1, high);
        }
    }
}
