//! Bank $80 sound queues. Queueing changes WRAM; the existing host drains audio.
use crate::{Result, Wram};

pub(crate) fn queue(r: &mut Wram<'_>, channel: usize, sound: u8, maximum: u8) -> Result<()> {
    let write = r.byte(0x646 + channel);
    let read = r.byte(0x643 + channel);
    if write >= 16 || read >= 16 {
        return Err("invalid sound ring cursor".into());
    }
    r.set_byte(0x653 + channel, maximum);
    if write.wrapping_sub(read) & 15 >= maximum {
        return Ok(());
    }
    if r.word(0x5f5) != 0 || r.word(0x998) >= 40 || r.word(0x592) & 0x8000 != 0 {
        return Ok(());
    }
    let next = (write + 1) & 15;
    let base = 0x656 + channel * 16;
    if next == read {
        if sound < r.byte(base + usize::from(write)) {
            r.set_byte(base + usize::from(write), sound);
        }
    } else {
        r.set_byte(base + usize::from(write), sound);
        r.set_byte(0x646 + channel, next);
        r.set_byte(base + usize::from(next), 0);
    }
    Ok(())
}
