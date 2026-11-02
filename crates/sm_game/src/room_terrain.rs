//! Live terrain queries and ordered installation over the single room owner.
use crate::room_mechanisms::host::{ByteOrder, ImportedAliases};
use crate::session_access::SessionAccess;

pub fn level_word<H: SessionAccess + ImportedAliases + ?Sized>(
    h: &H,
    offset: usize,
    order: ByteOrder,
) -> u16 {
    if offset & 1 == 0
        && let Some(cell) = h.session().room.terrain.cells().get(offset / 2)
    {
        return match order {
            ByteOrder::Little => cell.level,
            ByteOrder::Native => u16::from_ne_bytes(cell.level.to_le_bytes()),
        };
    }
    ImportedAliases::terrain_word(h, offset, order)
}
pub fn set_level_word<H: SessionAccess + ImportedAliases + ?Sized>(
    h: &mut H,
    offset: usize,
    value: u16,
    order: ByteOrder,
) {
    if offset & 1 == 0
        && let Some(cell) = h.session_mut().room.terrain.cells_mut().get_mut(offset / 2)
    {
        cell.level = match order {
            ByteOrder::Little => value,
            ByteOrder::Native => u16::from_le_bytes(value.to_ne_bytes()),
        };
    } else {
        ImportedAliases::set_terrain_word(h, offset, value, order);
    }
}
pub fn bts<H: SessionAccess + ImportedAliases + ?Sized>(h: &H, block: usize) -> u8 {
    h.session().room.terrain.cells().get(block).map_or_else(
        || ImportedAliases::behavior_byte(h, block),
        |cell| cell.behavior,
    )
}

/// Exceptional decompressor extent, relative to the actual decompressed allocation.
/// Implementations retain checked allocation/failure behavior, never synthesize zeros.
pub trait PlaneAliases: SessionAccess {
    fn imported_level_byte(&self, offset: isize) -> u8;
    fn set_imported_level_byte(&mut self, offset: isize, value: u8);
}
const LEVEL: usize = 2;
const BTS: usize = LEVEL + crate::session::TERRAIN_CELLS * 2;
const LAYER2: usize = BTS + crate::session::TERRAIN_CELLS;
const ALLOCATION_BYTES: usize = LAYER2 + crate::session::LAYER2_WORDS * 2;

fn byte<H: PlaneAliases + ?Sized>(h: &H, offset: usize) -> u8 {
    let terrain = &h.session().room.terrain;
    match offset {
        0..LEVEL => terrain.decompressed_size.to_le_bytes()[offset],
        LEVEL..BTS => {
            let index = offset - LEVEL;
            terrain.cells()[index / 2].level.to_le_bytes()[index % 2]
        }
        BTS..LAYER2 => terrain.cells()[offset - BTS].behavior,
        LAYER2..ALLOCATION_BYTES => {
            let index = offset - LAYER2;
            terrain.layer2()[index / 2].to_le_bytes()[index % 2]
        }
        _ => h.imported_level_byte(isize::try_from(offset).expect("level allocation offset")),
    }
}
fn set_byte<H: PlaneAliases + ?Sized>(h: &mut H, offset: usize, value: u8) {
    let terrain = &mut h.session_mut().room.terrain;
    let (word, lane) = match offset {
        0..LEVEL => (&mut terrain.decompressed_size, offset),
        LEVEL..BTS => {
            let index = offset - LEVEL;
            (&mut terrain.cells_mut()[index / 2].level, index % 2)
        }
        BTS..LAYER2 => {
            terrain.cells_mut()[offset - BTS].behavior = value;
            return;
        }
        LAYER2..ALLOCATION_BYTES => {
            let index = offset - LAYER2;
            (&mut terrain.layer2_mut()[index / 2], index % 2)
        }
        _ => {
            h.set_imported_level_byte(
                isize::try_from(offset).expect("level allocation offset"),
                value,
            );
            return;
        }
    };
    let mut bytes = word.to_le_bytes();
    bytes[lane] = value;
    *word = u16::from_le_bytes(bytes);
}

/// Called after the actual decoder's ordered writes, including its header.
/// No validation of dimensions, normalized padding or independent terrain cache.
pub fn copy_level_planes<H: PlaneAliases + ?Sized>(h: &mut H) {
    let size = usize::from(h.session().room.terrain.decompressed_size);
    for (source, destination, count) in [
        (LEVEL + size + size / 2, LAYER2, size),
        (LEVEL + size, BTS, size / 2),
    ] {
        if source < destination + count
            && destination < source + count
            && source + count <= ALLOCATION_BYTES
            && destination + count <= ALLOCATION_BYTES
        {
            let original: Vec<u8> = (source..source + count)
                .map(|offset| byte(h, offset))
                .collect();
            for (index, value) in original.into_iter().enumerate() {
                set_byte(h, destination + index, value);
            }
        } else {
            for index in 0..count {
                let value = byte(h, source + index);
                set_byte(h, destination + index, value);
            }
        }
    }
}

/// Decoder and clear operations use this finite allocation sink in their original order.
/// The caller selects the actual stream and clear span; failure leaves its written prefix.
pub fn install_byte<H: PlaneAliases + ?Sized>(h: &mut H, offset: usize, value: u8) {
    set_byte(h, offset, value);
}

struct PreparedOutput<'a, H: ?Sized> {
    host: &'a mut H,
    stream: core::slice::Iter<'a, u8>,
    produced: usize,
}
impl<H: PlaneAliases + ?Sized> crate::room_entry::prepared_tables::Output
    for PreparedOutput<'_, H>
{
    fn next_byte(&mut self) -> u8 {
        *self.stream.next().expect("validated prepared room stream")
    }
    fn read(&mut self, position: i32) -> u8 {
        match usize::try_from(position) {
            Ok(offset) => byte(self.host, offset),
            Err(_) => self
                .host
                .imported_level_byte(isize::try_from(position).expect("level allocation offset")),
        }
    }
    fn write(&mut self, position: i32, value: u8, _back_reference: bool) {
        match usize::try_from(position) {
            Ok(offset) => set_byte(self.host, offset, value),
            Err(_) => self.host.set_imported_level_byte(
                isize::try_from(position).expect("level allocation offset"),
                value,
            ),
        }
        self.produced = self
            .produced
            .max(usize::try_from(position).expect("level output position") + 1);
    }
}
/// Execute the selected immutable stream; backreferences query CURRENT terrain bytes.
/// Decode failure leaves precisely its writes; plane copying begins only after decode.
pub fn load_prepared<H: PlaneAliases + ?Sized>(stream: &[u8], host: &mut H) -> usize {
    let mut output = PreparedOutput {
        host,
        stream: stream.iter(),
        produced: 0,
    };
    crate::room_entry::prepared_tables::decode(0, &mut output);
    let produced = output.produced;
    copy_level_planes(output.host);
    produced
}
