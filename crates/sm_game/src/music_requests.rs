//! Ordered request algorithms over the actual audio state owner.

use crate::scene::state::DisplayError;
use crate::session_owners::{AudioHost, QueuedMusicMeaning, QueuedSoundMeaning};

// Complete only after a successful numeric prefix. This neither publishes a
// request nor supplies a meaning for a rejected slot; Legacy skips the closure.
fn complete(host: &mut impl AudioHost) -> Result<(), DisplayError> {
    host.audio_mutation(|_, _, _| Ok(()))?;
    Ok(())
}

// The callback receives the actual host; completed stores survive its failure.
// Normal callbacks never enter the semantic hook through this guard.
struct CallbackFailure<'a, H: AudioHost> {
    host: &'a mut H,
    completed: bool,
}
impl<H: AudioHost> Drop for CallbackFailure<'_, H> {
    fn drop(&mut self) {
        if !self.completed {
            // Owned hosts retain the pending prefix and poison on this error.
            // Legacy skips the closure. Ignore the signalling result so the
            // original callback error or unwind payload remains unchanged.
            let _ = self.host.audio_mutation(|requests, _, _| {
                requests.pending_mutation = true;
                Err::<(), _>(DisplayError::Poisoned)
            });
        }
    }
}
fn callback<H: AudioHost>(
    host: &mut H,
    operation: impl FnOnce(&mut H) -> Result<(), DisplayError>,
) -> Result<(), DisplayError> {
    let mut guard = CallbackFailure {
        host,
        completed: false,
    };
    let result = operation(guard.host);
    guard.completed = result.is_ok();
    result
}

pub fn enqueue(
    track: u16,
    delay: u16,
    reject_full: bool,
    host: &mut impl AudioHost,
) -> Result<(), DisplayError> {
    if host.audio_word(0x998) >= 40 {
        return Ok(());
    }
    if reject_full && host.audio_word(0x639).wrapping_add(2) & 14 == host.audio_word(0x63b) {
        return Ok(());
    }
    let write = host.audio_word(0x639);
    let slot = usize::from(write >> 1);
    host.audio_store_word(0x619 + 2 * slot, track);
    host.audio_store_word(0x629 + 2 * slot, delay.max(8));
    host.audio_store_word(0x639, write.wrapping_add(2) & 14);
    host.audio_mutation(|requests, _, stamp| {
        requests.publish_music(slot, track, stamp.tick, stamp.source_interval)
    })?;
    Ok(())
}

pub fn retire(host: &mut impl AudioHost) -> Result<(), DisplayError> {
    let read = host.audio_word(0x63b);
    let slot = usize::from(read >> 1);
    host.audio_store_word(0x619 + 2 * slot, 0);
    host.audio_store_word(0x629 + 2 * slot, 0);
    host.audio_store_word(0x63b, read.wrapping_add(2) & 14);
    host.audio_mutation(|requests, _, _| requests.clear_music(slot))?;
    Ok(())
}

pub fn has_queued_music(host: &impl AudioHost) -> u8 {
    u8::from(
        (0..8)
            .rev()
            .any(|slot| host.audio_word(0x629 + 2 * slot) != 0),
    )
}

pub fn reset_delays(host: &mut impl AudioHost) -> Result<(), DisplayError> {
    host.audio_store_word(0x63f, 0);
    for slot in 0..8 {
        host.audio_store_word(0x629 + 2 * slot, 0);
    }
    complete(host)
}

pub fn clear_queue(host: &mut impl AudioHost) -> Result<(), DisplayError> {
    for slot in (0..8).rev() {
        host.audio_store_word(0x619 + 2 * slot, 0);
        host.audio_store_word(0x629 + 2 * slot, 0);
    }
    host.audio_store_word(0x63b, host.audio_word(0x639));
    host.audio_store_word(0x63f, 0);
    host.audio_store_word(0x63d, 0);
    host.audio_mutation(|requests, _, _| {
        for slot in (0..8).rev() {
            requests.clear_music(slot)?;
        }
        requests.clear_music_entry()
    })?;
    Ok(())
}

/// The native wrapper retains bank upload and APU writes; owned callers provide
/// their actual sound consumer. The meaning is the captured authored record;
/// numeric compatibility values and post-callback fresh reads remain separate.
pub enum MusicEffect {
    UploadBank,
    WriteTrack(u8),
}

pub fn advance_music<H: AudioHost>(
    host: &mut H,
    mut effect: impl FnMut(MusicEffect, Option<&QueuedMusicMeaning>, &mut H) -> Result<(), DisplayError>,
) -> Result<(), DisplayError> {
    let previous = host.audio_word(0x63f);
    host.audio_store_word(0x63f, previous.wrapping_sub(1));
    if host.audio_word(0x63f) & 0x8000 == 0 {
        if previous != 1 {
            return complete(host);
        }
        // Capture the authored record for this selected entry, never reinterpret
        // its numeric bank/track selector as an asset or phase.
        let record = host.audio_mutation(|requests, _, _| {
            let record = requests
                .music
                .entry_meaning
                .as_ref()
                .ok_or(DisplayError::Incomplete)?;
            if record.request != requests.music.entry {
                return Err(DisplayError::Identity);
            }
            Ok(record.clone())
        })?;
        if host.audio_word(0x63d) & 0x8000 != 0 {
            host.audio_store_word(0x7f3, u16::from(host.audio_word(0x63d) as u8));
            host.audio_store_byte(0x64c, 0xff);
            callback(host, |host| {
                effect(MusicEffect::UploadBank, record.as_ref(), host)
            })?;
            host.audio_store_byte(0x64c, 0);
            if let Some(record) = record {
                host.audio_mutation(|requests, audio, stamp| {
                    requests.accept_music(audio, record, stamp)
                })?;
            }
            retire(host)?;
            host.audio_store_word(0x686, 8);
            return complete(host);
        }
        let track = host.audio_word(0x63d) as u8 & 0x7f;
        host.audio_store_word(0x7f5, host.audio_word(0x63d) & 0x7f);
        let command = MusicEffect::WriteTrack(host.audio_word(0x63d) as u8 & 0x7f);
        callback(host, |host| effect(command, record.as_ref(), host))?;
        host.audio_store_byte(0x64c, track);
        if let Some(record) = record {
            host.audio_mutation(|requests, audio, stamp| {
                requests.accept_music(audio, record, stamp)
            })?;
        }
        host.audio_store_word(0x686, 8);
        retire(host)?;
    }
    if host.audio_word(0x63b) == host.audio_word(0x639) {
        host.audio_store_word(0x63f, 0);
    } else {
        let slot = usize::from(host.audio_word(0x63b) >> 1);
        let entry = host.audio_word(0x619 + 2 * slot);
        host.audio_store_word(0x63d, entry);
        host.audio_store_word(0x63f, host.audio_word(0x629 + 2 * slot));
        host.audio_mutation(|requests, _, _| {
            let record = requests.capture_music(slot, entry)?;
            requests.select_music_entry(record)
        })?;
    }
    complete(host)
}

pub fn queue_sfx(
    channel: usize,
    sound: u8,
    max_queued: u8,
    host: &mut impl AudioHost,
) -> Result<u16, DisplayError> {
    let read_pos = 0x643 + channel;
    let write_pos = 0x646 + channel;
    let limit_pos = 0x653 + channel;
    let buffer = 0x656 + 16 * channel;
    host.audio_store_byte(limit_pos, max_queued);
    let queued = host
        .audio_byte(write_pos)
        .wrapping_sub(host.audio_byte(read_pos))
        & 15;
    if queued >= max_queued {
        complete(host)?;
        return Ok(u16::from_be_bytes([sound, queued]));
    }
    if host.audio_word(0x5f5) != 0
        || host.audio_word(0x998) >= 40
        || host.audio_word(0x592) & 0x8000 != 0
    {
        complete(host)?;
        return Ok(u16::from_be_bytes([queued, sound]));
    }
    let write = host.audio_byte(write_pos);
    let mut next = write.wrapping_add(1);
    if next >= 16 {
        next = 0;
    }
    let slot = buffer + usize::from(write);
    if next == host.audio_byte(read_pos) {
        if sound < host.audio_byte(slot) {
            host.audio_store_byte(slot, sound);
            host.audio_mutation(|requests, _, stamp| {
                requests.publish_sound(
                    channel,
                    usize::from(write),
                    sound,
                    stamp.tick,
                    stamp.source_interval,
                )
            })?;
        }
    } else {
        host.audio_store_byte(slot, sound);
        host.audio_store_byte(write_pos, next);
        host.audio_store_byte(buffer + usize::from(next), 0);
        host.audio_mutation(|requests, _, stamp| {
            requests.publish_sound(
                channel,
                usize::from(write),
                sound,
                stamp.tick,
                stamp.source_interval,
            )?;
            requests.clear_sound(channel, usize::from(next))
        })?;
    }
    complete(host)?;
    Ok(u16::from_be_bytes([queued, sound]))
}

pub fn advance_sound_effects<H: AudioHost>(
    host: &mut H,
    mut write: impl FnMut(u8, u8, Option<&QueuedSoundMeaning>, &mut H) -> Result<(), DisplayError>,
) -> Result<(), DisplayError> {
    if (host.audio_byte(0x686) as i8).wrapping_sub(1) >= 0 {
        host.audio_store_byte(0x686, host.audio_byte(0x686).wrapping_sub(1));
        for channel in 0..3 {
            callback(host, |host| write(channel as u8, 0, None, host))?;
            host.audio_store_byte(0x64d + channel, 0);
            host.audio_mutation(|requests, _, _| requests.clear_current_sound(channel))?;
        }
        return Ok(());
    }
    for channel in 0..3 {
        let state = [0, 1, 2, 3, 4][usize::from(host.audio_byte(0x649 + channel))];
        match state {
            0 | 3 => {
                if state == 3 {
                    let _last_sent = host.audio_byte(0x64d + channel);
                    host.audio_store_byte(0x649 + channel, 0);
                }
                if host.audio_byte(0x643 + channel) != host.audio_byte(0x646 + channel) {
                    let slot = (usize::from(host.audio_byte(0x643 + channel)) + channel * 16) as u8;
                    let sound = host.audio_byte(0x656 + usize::from(slot));
                    let record = host.audio_mutation(|requests, _, _| {
                        let local_slot = usize::from(slot)
                            .checked_sub(channel * 16)
                            .ok_or(DisplayError::Source)?;
                        requests.capture_sound(channel, local_slot, sound)
                    })?;
                    callback(host, |host| {
                        write(channel as u8, sound, record.as_ref(), host)
                    })?;
                    host.audio_store_byte(0x64d + channel, sound);
                    host.audio_store_byte(0x643 + channel, slot.wrapping_add(1) & 15);
                    let value = host.audio_byte(0x649 + channel).wrapping_add(1);
                    host.audio_store_byte(0x649 + channel, value);
                    if let Some(record) = record {
                        host.audio_mutation(|requests, _, stamp| {
                            requests.select_current_sound(channel, record, stamp)
                        })?;
                    }
                }
            }
            1 => {
                let _last_sent = host.audio_byte(0x64d + channel);
                let value = host.audio_byte(0x649 + channel).wrapping_add(1);
                host.audio_store_byte(0x649 + channel, value);
                // The original waits 2 frames before clearing the request.
                host.audio_store_byte(0x650 + channel, 2);
            }
            2 | 4 => {
                let previous = host.audio_byte(0x650 + channel);
                host.audio_store_byte(0x650 + channel, previous.wrapping_sub(1));
                if previous == 1 {
                    if state == 2 {
                        host.audio_store_byte(0x64d + channel, 0);
                        let value = host.audio_byte(0x649 + channel).wrapping_add(1);
                        host.audio_store_byte(0x649 + channel, value);
                        host.audio_mutation(|requests, _, _| {
                            requests.clear_current_sound(channel)
                        })?;
                    } else {
                        host.audio_store_byte(0x649 + channel, 0);
                    }
                }
            }
            _ => unreachable!(),
        }
    }
    complete(host)
}

pub fn reset_sound_queues(host: &mut impl AudioHost) -> Result<(), DisplayError> {
    for offset in 0x643..0x64c {
        host.audio_store_byte(offset, 0);
    }
    complete(host)
}
