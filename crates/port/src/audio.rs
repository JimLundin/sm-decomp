//! Audio queue bookkeeping translated from snesrev/sm src/sm_80.c.
//! Upstream license: crates/decomp/LICENSE.upstream.txt.
use crate::Wram;

const MUSIC_TRACK: usize = 0x619;
const MUSIC_DELAY: usize = 0x629;
const MUSIC_WRITE: usize = 0x639;
const MUSIC_READ: usize = 0x63b;
const GAME_STATE: usize = 0x998;

/// Detect any nonzero delay, independently of the music ring's cursors.
pub fn has_queued_music(ram: &Wram<'_>) -> bool {
    (0..8).any(|slot| ram.word(MUSIC_DELAY + slot * 2) != 0)
}

/// Queue a track using canonical even music-ring offsets 0..=14.
/// The explicit-delay routine can overwrite a full ring, as upstream does.
pub fn queue_music(ram: &mut Wram<'_>, track: u16, delay: u16, fixed_delay: bool) -> Result<(), &'static str> {
    if ram.word(GAME_STATE) >= 40 {
        return Ok(());
    }
    let write = ram.word(MUSIC_WRITE);
    let read = ram.word(MUSIC_READ);
    if write > 14 || write & 1 != 0 || read > 14 || read & 1 != 0 {
        return Err("invalid music ring cursor");
    }
    let next = (write + 2) & 14;
    if fixed_delay && next == read {
        return Ok(());
    }
    ram.set_word(MUSIC_TRACK + usize::from(write), track);
    ram.set_word(MUSIC_DELAY + usize::from(write), delay.max(8));
    ram.set_word(MUSIC_WRITE, next);
    Ok(())
}

const SFX_READ: usize = 0x643;
const SFX_WRITE: usize = 0x646;
const SFX_MAX: usize = 0x653;
const SFX_QUEUE: usize = 0x656;

/// Enqueue a sound on one of three independent 16-byte rings.
/// C stores the low byte of `packed` as the quota before checking any gates.
pub fn queue_sfx(ram: &mut Wram<'_>, channel: usize, packed: u16) -> Result<(), &'static str> {
    if channel >= 3 {
        return Err("invalid sound channel");
    }
    let write = ram.byte(SFX_WRITE + channel);
    let read = ram.byte(SFX_READ + channel);
    if write >= 16 || read >= 16 {
        return Err("invalid sound ring cursor");
    }
    let maximum = packed as u8;
    ram.set_byte(SFX_MAX + channel, maximum);
    if write.wrapping_sub(read) & 15 >= maximum {
        return Ok(());
    }
    if ram.word(0x5f5) != 0 || ram.word(GAME_STATE) >= 40 || ram.word(0x592) & 0x8000 != 0 {
        return Ok(());
    }
    let sound = (packed >> 8) as u8;
    let next = (write + 1) & 15;
    let base = SFX_QUEUE + channel * 16;
    if next == read {
        if sound < ram.byte(base + usize::from(write)) {
            ram.set_byte(base + usize::from(write), sound);
        }
    } else {
        ram.set_byte(base + usize::from(write), sound);
        ram.set_byte(SFX_WRITE + channel, next);
        ram.set_byte(base + usize::from(next), 0);
    }
    Ok(())
}

pub fn dispatch(ram: &mut Wram<'_>, symbol: &str, a: u16, b: u16) -> Result<u32, &'static str> {
    match symbol {
        "HasQueuedMusic" => Ok(u32::from(has_queued_music(ram))),
        "QueueMusic_Delayed8" => queue_music(ram, a, 8, true).map(|()| 0),
        "QueueMusic_DelayedY" => queue_music(ram, a, b, false).map(|()| 0),
        _ => {
            let (prefix, suffix) = symbol.split_once('_').ok_or("unknown audio routine")?;
            let channel = match prefix {
                "QueueSfx1" => 0,
                "QueueSfx2" => 1,
                "QueueSfx3" => 2,
                _ => return Err("unknown audio routine"),
            };
            let packed = match suffix {
                "Internal" => a,
                "Max15" => (a << 8) | 15,
                "Max9" => (a << 8) | 9,
                "Max3" => (a << 8) | 3,
                "Max1" => (a << 8) | 1,
                "Max6" => (a << 8) | 6,
                _ => return Err("unknown audio routine"),
            };
            queue_sfx(ram, channel, packed).map(|()| 0)
        }
    }
}
