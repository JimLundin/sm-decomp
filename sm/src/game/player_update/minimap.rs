//! Owned exploration marking, HUD map tiles and boss-room suppression.
#![forbid(unsafe_code)]
use super::effects::Host;
use crate::game::visual_host::Record;

use sm_game::progress::exploration::{MapWindow, explored_byte, mark_byte, mark_explored};
pub(crate) fn disable_for_boss(h: &mut impl Host) {
    h.session_mut().minimap.disabled = 1;
    for column in 0..5 {
        for row in 0..3 {
            h.session_mut().minimap.tiles[row][column] = 11295;
        }
    }
    let mut descriptor = 5;
    loop {
        if h.minimap_boss_id() == h.boss_map_id(descriptor) {
            break;
        }
        if descriptor == 0 {
            return;
        }
        descriptor -= 1;
    }
    let coordinates = h.boss_coordinates(descriptor);
    let mut index = 0;
    loop {
        let record = h.boss_coordinate(coordinates, index);
        if record.native_word(0) & 0x8000 != 0 {
            break;
        }
        // The first coordinate is reread after the sentinel check.
        mark_explored(h, record.native_word(0), record.native_word(2));
        index += 1;
    }
}
pub(crate) fn update(h: &mut impl Host) {
    if h.session().minimap.disabled != 0
        || h.session().player.x >> 4 >= h.session().room.width
        || h.session().player.y >> 4 >= h.session().room.height
    {
        return;
    }
    let (x, y) = (h.session().player.x, h.session().player.y);
    let MapWindow {
        half,
        column,
        y,
        bit,
        byte_index,
    } = mark_explored(h, x, y);
    let crossed_byte = bit < 2;
    let shift = (bit + 6) & 7;
    let origin = byte_index.wrapping_sub(if crossed_byte { 5 } else { 4 });
    let mut explored = [0u16; 3];
    for row in 0..3 {
        let offset = origin as isize + 4 * row as isize;
        // Preserve native-word interpretation without an unaligned reference.
        explored[row] = (0xfc00u16 >> shift)
            & u16::from_ne_bytes([explored_byte(h, offset), explored_byte(h, offset + 1)])
                .swap_bytes();
    }
    let area = h.session().room.area;
    let map = h.area_exploration_mask(area);
    let mut mapped = [0u16; 3];
    for row in 0..3 {
        mapped[row] = map.native_word(usize::from(origin) + 4 * row).swap_bytes();
    }
    if origin & 3 == 3 && (if crossed_byte { shift } else { bit }) >= 6 {
        let other_half = if half != 0 {
            origin.wrapping_sub(124)
        } else {
            origin.wrapping_add(125)
        } as u8;
        let lane = if half == 32 { 8 } else { 0 };
        for row in 0..3 {
            let offset = usize::from(other_half) + 4 * row;
            let explored_byte = u16::from(explored_byte(h, offset as isize));
            let mapped_byte = u16::from(map.byte(offset));
            mapped[row] = (mapped[row] & !(0xff << lane)) | (mapped_byte << lane);
            explored[row] = (explored[row] & !(0xff << lane)) | (explored_byte << lane);
        }
    }
    for row in 0..3 {
        explored[row] <<= shift;
        mapped[row] <<= shift;
    }
    let tile = column.wrapping_add(half.wrapping_add(y).swap_bytes() >> 3);
    let adjustment = if half != 0 && tile & 0x1f < 2 {
        1026
    } else {
        34
    };
    let mut source_offset = tile.wrapping_sub(adjustment).wrapping_mul(2);
    let area = h.session().room.area;
    let tiles = h.area_map_tiles(area);
    for column in 0..5 {
        for row in 0..3 {
            let source = 2 * (32 * row + usize::from(source_offset >> 1));
            let mut tile = 31;
            if mapped[row] & 0x8000 != 0 {
                tile = tiles.native_word(source);
                if !h.area_map_available() {
                    tile = 31;
                }
            }
            mapped[row] = mapped[row].wrapping_mul(2);
            h.session_mut().minimap.tiles[row][column] = tile & 0xc3ff | 0x2c00;
            let visited = explored[row] & 0x8000 != 0;
            explored[row] = explored[row].wrapping_mul(2);
            if visited {
                // The mapped HUD write above survives a failing visited-tile read.
                h.session_mut().minimap.tiles[row][column] =
                    tiles.native_word(source) & 0xc3ff | 0x2800;
                if row == 1 && column == 2 && h.session().minimap.tiles[row][column] & 0x1ff == 40 {
                    // Subtract from the logical offset, without wrapping to u16.
                    mark_byte(h, byte_index as isize - 4, 0x80 >> bit);
                }
            }
        }
        source_offset = source_offset.wrapping_add(2);
        if source_offset & 0x3f == 0 {
            source_offset = source_offset.wrapping_add(1984);
        }
    }
    if h.minimap_blink() & 8 == 0 {
        h.session_mut().minimap.tiles[1][2] |= 0x1c00;
    }
    h.probe("player.minimap.updated");
}
