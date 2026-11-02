//! Kraid's suspended pause/unpause operations. The source kernel uses typed
//! continuation, display, work cursors and finite upload/readback operations.
#![forbid(unsafe_code)]
use crate::actors::kraid_sinking_owned;

#[derive(Clone, Copy)]
pub enum Graphics {
    RoomBackground,
    StandardBg3Tiles,
    SavedKraidTiles,
    KraidTopHalfTilemap,
}
pub trait Host: kraid_sinking_owned::Host {
    fn secondary_continuation(&self) -> u8;
    fn set_secondary_continuation(&mut self, value: u8);
    fn bg12_tile_base(&self) -> u8;
    fn set_blank(&mut self, blank: bool);
    fn wait_for_nmi(&mut self) -> u8;
    fn load_graphics(&mut self, graphics: Graphics, destination: u16, words: u16);
    fn sinking_counter(&self) -> u16;
    fn set_sinking_counter(&mut self, counter: u16);
    fn saved_queue_cursor(&self) -> u16;
    fn set_saved_queue_cursor(&mut self, cursor: u16);
    fn readback_kraid_tiles(&mut self);
}
fn blank_and_wait(blank: bool, point: u8, host: &mut impl Host) -> u8 {
    if host.secondary_continuation() != point {
        host.set_blank(blank);
    }
    let yielded = host.wait_for_nmi();
    host.set_secondary_continuation(if yielded != 0 { point } else { 0 });
    yielded
}
fn load_kraid_tiles(host: &mut impl Host) {
    let tiles = u16::from(host.bg12_tile_base().wrapping_add(62)) << 8;
    host.load_graphics(Graphics::SavedKraidTiles, tiles, 0x400);
}
pub fn unpause_dead(host: &mut impl Host) -> u8 {
    if host.secondary_continuation() != 3 {
        let yielded = blank_and_wait(true, 2, host);
        if yielded != 0 {
            return yielded;
        }
        let tiles = (((16 * u16::from(host.bg12_tile_base())) & 0xf).wrapping_add(63)) << 8;
        host.load_graphics(Graphics::RoomBackground, tiles, 0x200);
        host.load_graphics(Graphics::StandardBg3Tiles, 0x4000, 0x800);
    }
    transfer_top_half(host)
}
pub fn unpause_alive(host: &mut impl Host) -> u8 {
    if host.secondary_continuation() != 3 {
        let yielded = blank_and_wait(true, 2, host);
        if yielded != 0 {
            return yielded;
        }
        load_kraid_tiles(host);
    }
    transfer_top_half(host)
}
pub fn transfer_top_half(host: &mut impl Host) -> u8 {
    if host.secondary_continuation() != 3 {
        let tilemap = u16::from(host.bg2_tilemap_base() & 0xfc) << 8;
        host.load_graphics(Graphics::KraidTopHalfTilemap, tilemap, 0x800);
    }
    blank_and_wait(false, 3, host)
}
pub fn unpause_sinking(host: &mut impl Host) -> u8 {
    'done: {
        let mut queue_transfer = match host.secondary_continuation() {
            0 | 2 => {
                let yielded = blank_and_wait(true, 2, host);
                if yielded != 0 {
                    return yielded;
                }
                load_kraid_tiles(host);
                if i16::from_ne_bytes(host.kraid_y().to_ne_bytes()).wrapping_sub(
                    i16::from_ne_bytes(
                        kraid_sinking_owned::sinking_row(0, host)
                            .word(0)
                            .to_ne_bytes(),
                    ),
                ) < 0
                {
                    break 'done;
                }
                host.set_sinking_counter(0);
                host.set_saved_queue_cursor(host.tilemap_queue_tail());
                true
            }
            1 => false,
            _ => break 'done,
        };
        loop {
            if queue_transfer {
                let row =
                    kraid_sinking_owned::sinking_row(usize::from(host.sinking_counter()), host);
                if !(row.word(0) != 0xffff
                    && i16::from_ne_bytes(host.kraid_y().to_ne_bytes())
                        .wrapping_sub(i16::from_ne_bytes(row.word(0).to_ne_bytes()))
                        >= 0)
                {
                    break 'done;
                }
                let base = host.bg2_tilemap_base();
                kraid_sinking_owned::queue_row_clear(
                    host.saved_queue_cursor(),
                    row.word(2),
                    base,
                    host,
                );
            }
            let yielded = host.wait_for_nmi();
            if yielded != 0 {
                host.set_secondary_continuation(1);
                return yielded;
            }
            host.set_sinking_counter(host.sinking_counter().wrapping_add(1));
            queue_transfer = true;
        }
    }
    blank_and_wait(true, 4, host)
}
pub fn pause(host: &mut impl Host) {
    host.readback_kraid_tiles();
}
