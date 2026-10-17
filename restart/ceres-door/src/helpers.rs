//! Small bank 80/A9 dependencies owned by the actor. Queue consumption stays native.
use crate::{
    Result,
    memory::{Rom, Wram},
};

pub(crate) fn sound_preflight(ram: &Wram<'_>, channel: usize) -> Result<()> {
    if ram.0[0x646 + channel] >= 16 || ram.0[0x643 + channel] >= 16 {
        return Err("invalid Ceres door sound ring cursor".into());
    }
    Ok(())
}

pub(crate) fn sound(ram: &mut Wram<'_>, channel: usize, sound: u8) -> Result<()> {
    sound_preflight(ram, channel)?;
    let write = ram.0[0x646 + channel];
    let read = ram.0[0x643 + channel];
    ram.0[0x653 + channel] = 6;
    if write.wrapping_sub(read) & 15 >= 6
        || ram.word(0x5f5) != 0
        || ram.word(0x998) >= 0x28
        || ram.word(0x592) & 0x8000 != 0
    {
        return Ok(());
    }
    let next = (write + 1) & 15;
    let queue = 0x656 + 16 * channel;
    if next == read {
        if sound < ram.0[queue + usize::from(write)] {
            ram.0[queue + usize::from(write)] = sound;
        }
    } else {
        ram.0[queue + usize::from(write)] = sound;
        ram.0[0x646 + channel] = next;
        ram.0[queue + usize::from(next)] = 0;
    }
    Ok(())
}

/// Preserve the byte-wide high product and the end-around carry in NextRandom.
pub(crate) fn next_random(value: u16) -> u16 {
    let low_product = (value & 0xff) * 5;
    let high_product = ((value >> 8) as u8).wrapping_mul(5);
    let carry = (low_product >> 8) + u16::from(high_product) + 1;
    let product = (low_product & 0xff) | ((carry & 0xff) << 8);
    product.wrapping_add(carry >> 8).wrapping_add(17)
}

pub(crate) fn quake(ram: &mut Wram<'_>, kind: u16) {
    if ram.word(0x93f) >= 2 && ram.word(0x1840) == 0 {
        let small = ram.word(0x5e5) & 0xfff < 0x80;
        ram.set(0x1840, if small { 4 } else { 2 });
        ram.set(0x183e, if small { kind + 6 } else { kind });
    }
}

pub(crate) struct Animation<'a> {
    descriptors: &'a [u8],
    tail: usize,
    palette: Option<&'a [u8]>,
}

/// Validate every descriptor and its trailing terminator before any writes.
/// C appends either nine-byte VRAM or seven-byte CGRAM records to this queue.
pub(crate) fn animation<'a>(ram: &Wram<'_>, rom: &Rom<'a>) -> Result<Animation<'a>> {
    let frame = ram.word(0x5b6);
    let start = rom.word(0xf900 + (frame & 2))?;
    let tail = usize::from(ram.word(0x334));
    let mut length = 0usize;
    loop {
        let address = usize::from(start) + length;
        if address > 0xffff {
            return Err("Ceres door Mode7 descriptors cross bank A6".into());
        }
        let flags = rom.bytes(address as u16, 1)?[0];
        let size = if flags & 0x80 != 0 {
            9
        } else if flags & 0x40 != 0 {
            7
        } else {
            break;
        };
        // The queue occupies 02D0..0330. Each append also writes a zero byte.
        if tail + length + size >= 0x60 {
            return Err("Ceres door Mode7 queue exceeds its WRAM allocation".into());
        }
        rom.bytes(address as u16, size)?;
        length += size;
    }
    if tail >= 0x60 {
        return Err("invalid Ceres door Mode7 queue tail".into());
    }
    let palette =
        if ram.word(0xc400) == 0 { Some(rom.bytes((2 * (frame & 0x38)).wrapping_sub(0x78f), 12)?) } else { None };
    Ok(Animation { descriptors: rom.bytes(start, length)?, tail, palette })
}

impl Animation<'_> {
    pub fn apply(self, ram: &mut Wram<'_>) {
        let length = self.descriptors.len();
        if length != 0 {
            let start = 0x2d0 + self.tail;
            ram.0[start..start + length].copy_from_slice(self.descriptors);
            ram.0[start + length] = 0;
        }
        ram.set(0x334, (self.tail + length) as u16);
        if let Some(palette) = self.palette {
            ram.0[0xc052..0xc05e].copy_from_slice(palette);
        }
    }
}
