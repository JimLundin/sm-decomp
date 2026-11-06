//! The sprite plane of one line: the range pass over OAM (32 sprites), then the time pass
//! (34 tile slivers) from the last sprite found back to the first, so earlier sprites win.

use super::setup::SpriteSetup;
use super::{Dot, WIDTH};
use data::video::{VRAM_WORDS, VideoMemory};

/// The most sprites one line holds.
const RANGE: usize = 32;
/// The most 8-dot tile slivers one line fetches.
const TIME: u16 = 34;

/// One OAM entry as the line passes read it.
#[derive(Clone, Copy, Debug, Default)]
struct Entry {
    x: i16,
    y: u8,
    tile: u8,
    attributes: u8,
    /// Whether the entry takes OBSEL's large size.
    large: bool,
}

/// The 128 OAM entries, decoded once a frame: OAM doesn't change while the frame is shown.
pub struct Table([Entry; 128]);

impl Table {
    pub fn new(memory: &VideoMemory) -> Self {
        let (low, high) = memory.oam.split_at(512);
        let mut entries = [Entry::default(); 128];
        let bits = high
            .iter()
            .flat_map(|&byte| [0, 2, 4, 6].map(|shift| byte >> shift));
        for ((entry, bytes), bits) in entries.iter_mut().zip(low.as_chunks::<4>().0).zip(bits) {
            let [x, y, tile, attributes] = *bytes;
            let x = i16::from(x) | (i16::from(bits & 1) << 8);
            *entry = Entry {
                x: if x > 255 { x.wrapping_sub(512) } else { x },
                y,
                tile,
                attributes,
                large: bits & 2 != 0,
            };
        }
        Self(entries)
    }
}

/// Draw line `line`'s sprites (evaluated on the line before) into `dots`.
pub fn evaluate(
    setup: SpriteSetup,
    table: &Table,
    vram: &[u16; VRAM_WORDS],
    even_frame: bool,
    line: u16,
    dots: &mut [Dot; WIDTH],
) {
    dots.fill(Dot::default());
    let evaluated = u8::try_from(line.wrapping_sub(1) & 0xff).unwrap_or_default();
    let mut found = [0_u8; RANGE];
    let mut count = 0;
    let size = |sprite: &Entry| {
        setup
            .sizes
            .get(usize::from(sprite.large))
            .copied()
            .unwrap_or(8)
    };
    for (index, sprite) in (0_u8..).zip(&table.0) {
        let row = evaluated.wrapping_sub(sprite.y);
        let height = if setup.interlace {
            size(sprite) / 2
        } else {
            size(sprite)
        };
        if row < height && sprite.x > 0_i16.wrapping_sub_unsigned(u16::from(size(sprite))) {
            let Some(slot) = found.get_mut(count) else {
                break;
            };
            *slot = index;
            count = count.wrapping_add(1);
        }
    }
    let mut slivers = 0_u16;
    for &index in found.iter().take(count).rev() {
        let Some(sprite) = table.0.get(usize::from(index)) else {
            continue;
        };
        let mut row = evaluated.wrapping_sub(sprite.y);
        if setup.interlace {
            row = row.wrapping_mul(2).wrapping_add(u8::from(!even_frame));
        }
        let name = u16::from(sprite.attributes & 1);
        let palette = (sprite.attributes >> 1) & 7;
        let priority = (sprite.attributes >> 4) & 3;
        let horizontal_flip = sprite.attributes & 0x40 != 0;
        if sprite.attributes & 0x80 != 0 {
            row = size(sprite).wrapping_sub(1).wrapping_sub(row);
        }
        let base = setup
            .bases
            .get(usize::from(name))
            .copied()
            .unwrap_or_default();
        for column in (0..size(sprite)).step_by(8) {
            let left = sprite.x.wrapping_add(i16::from(column));
            if left <= -8 || left >= 256 {
                continue;
            }
            slivers = slivers.wrapping_add(1);
            if slivers > TIME {
                return;
            }
            let used = if horizontal_flip {
                size(sprite).wrapping_sub(1).wrapping_sub(column)
            } else {
                column
            };
            let tile = (((sprite.tile >> 4).wrapping_add(row >> 3)) << 4)
                | ((sprite.tile & 15).wrapping_add(used >> 3) & 15);
            let address = base
                .wrapping_add(u16::from(tile).wrapping_mul(16))
                .wrapping_add(u16::from(row & 7));
            let word = |offset: u16| {
                vram.get(usize::from(address.wrapping_add(offset) & 0x7fff))
                    .copied()
                    .unwrap_or_default()
            };
            let (low, high) = (word(0), word(8));
            for dot in 0..8_u16 {
                let shift = if horizontal_flip { dot } else { 7 ^ dot };
                let pixel = ((low >> shift) & 1)
                    | (((low >> shift.wrapping_add(8)) & 1) << 1)
                    | (((high >> shift) & 1) << 2)
                    | (((high >> shift.wrapping_add(8)) & 1) << 3);
                let Ok(at) = usize::try_from(left.wrapping_add(dot.cast_signed())) else {
                    continue;
                };
                if pixel != 0
                    && let Some(target) = dots.get_mut(at)
                {
                    *target = Dot {
                        index: 0x80_u16
                            .wrapping_add(u16::from(palette) << 4)
                            .wrapping_add(pixel),
                        priority,
                    };
                }
            }
        }
    }
}
