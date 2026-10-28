//! Single actor/common projection and changed-field codec.
#![forbid(unsafe_code)]
use super::{
    access::{AccessMode, ActorKey},
    state::{Actor, SpawnRecord},
};
use crate::game::session::Session;

// Both borrow modes use exactly the same lane map. Byte-only bank/hurt-time
// lanes are intentionally absent; no Rust struct layout is serialized.
macro_rules! presentation_visual {
    ($s:ident, (&)) => {
        $s.presentation.visual()
    };
    ($s:ident, (&mut)) => {
        $s.presentation.visual_mut()
    };
}
macro_rules! presentation_queue {
    ($s:ident, entry, $index:expr, (&)) => {
        $s.presentation.actor_entry($index)
    };
    ($s:ident, entry, $index:expr, (&mut)) => {
        $s.presentation.actor_entry_mut($index)
    };
    ($s:ident, size, $index:expr, (&)) => {
        $s.presentation.actor_size($index)
    };
    ($s:ident, size, $index:expr, (&mut)) => {
        $s.presentation.actor_size_mut($index)
    };
}

macro_rules! actor_word {
    ($actor:expr, $offset:expr, ($($borrow:tt)*)) => {
        match $offset {
            0x00 => Some($($borrow)* $actor.definition),
            0x02 => Some($($borrow)* $actor.x.whole),
            0x04 => Some($($borrow)* $actor.x.fraction),
            0x06 => Some($($borrow)* $actor.y.whole),
            0x08 => Some($($borrow)* $actor.y.fraction),
            0x0a => Some($($borrow)* $actor.radius.x),
            0x0c => Some($($borrow)* $actor.radius.y),
            0x0e => Some($($borrow)* $actor.properties),
            0x10 => Some($($borrow)* $actor.extra_properties),
            0x12 => Some($($borrow)* $actor.ai_handlers),
            0x14 => Some($($borrow)* $actor.health),
            0x16 => Some($($borrow)* $actor.spritemap),
            0x18 => Some($($borrow)* $actor.timer),
            0x1a => Some($($borrow)* $actor.instruction),
            0x1c => Some($($borrow)* $actor.instruction_delay),
            0x1e => Some($($borrow)* $actor.palette),
            0x20 => Some($($borrow)* $actor.tile),
            0x22 => Some($($borrow)* $actor.layer),
            0x24 => Some($($borrow)* $actor.flash),
            0x26 => Some($($borrow)* $actor.frozen),
            0x28 => Some($($borrow)* $actor.invincibility),
            0x2a => Some($($borrow)* $actor.shake),
            0x2c => Some($($borrow)* $actor.frame),
            0x3a => Some($($borrow)* $actor.pre_instruction),
            offset @ 0x30..=0x38 => Some($($borrow)* $actor.variables[(offset - 0x30) / 2]),
            offset @ 0x3c..=0x3e => Some($($borrow)* $actor.parameters[(offset - 0x3c) / 2]),
            _ => None,
        }
    };
}
macro_rules! spawn_word {
    ($spawn:expr, $offset:expr, ($($borrow:tt)*)) => {
        match $offset {
            0x00 => Some($($borrow)* $spawn.contact_mode),
            0x02 => Some($($borrow)* $spawn.death_cause),
            0x04 => Some($($borrow)* $spawn.scratch_04),
            0x06 => Some($($borrow)* $spawn.tile),
            0x08 => Some($($borrow)* $spawn.palette),
            0x10 => Some($($borrow)* $spawn.draw_offset.x),
            0x12 => Some($($borrow)* $spawn.draw_offset.y),
            0x1e => Some($($borrow)* $spawn.definition),
            0x20 => Some($($borrow)* $spawn.position.x),
            0x22 => Some($($borrow)* $spawn.position.y),
            0x24 => Some($($borrow)* $spawn.instruction),
            0x26 => Some($($borrow)* $spawn.properties),
            0x28 => Some($($borrow)* $spawn.extra_properties),
            offset @ 0x0a..=0x0e => Some($($borrow)* $spawn.scratch_0a[(offset - 0x0a) / 2]),
            offset @ 0x14..=0x1c => Some($($borrow)* $spawn.scratch_14[(offset - 0x14) / 2]),
            offset @ 0x2a..=0x2c => Some($($borrow)* $spawn.parameters[(offset - 0x2a) / 2]),
            _ => None,
        }
    };
}

// Only the new common bytes. Existing room counters at E50/E52 and all
// previously owned bytes stay with the original Session codec. Extended
// species RAM 7800..97DC is not owned here; angles97DC..9800 stay with eproj.
const COMMON_RANGES: [(usize, usize); 18] = [
    (0x0e12, 0x0e14),
    (0x0e1c, 0x0e20),
    (0x0e40, 0x0e42),
    (0x0e44, 0x0e50),
    (0x0e54, 0x0e84),
    (0x0e84, 0x0f78),
    (0x0f78, 0x1778),
    (0x1778, 0x1784),
    (0x1784, 0x179a),
    (0x179a, 0x183c),
    (0x183c, 0x183e),
    (0x1842, 0x1844),
    (0x1846, 0x184a),
    (0x185c, 0x186e),
    (0x18ae, 0x18b0),
    (0x7000, 0x7800),
    (0xef5c, 0xef74),
    (0xef74, 0xef78),
];
const COMMON_END: usize = 0xef78;

macro_rules! common_word {
    ($s:ident, $at:expr, ($($borrow:tt)*)) => {
        match $at {
            at @ 0x0f78..=0x1776 => actor_word!(
                $s.actors.slots[(at - 0x0f78) / 64], (at - 0x0f78) % 64, ($($borrow)*)),
            at @ 0x7000..=0x77fe => spawn_word!(
                $s.actors.saved[(at - 0x7000) / 64], (at - 0x7000) % 64, ($($borrow)*)),
            0x0e12 => Some($($borrow)* $s.actors.diagnostics.sprite_contacts_disabled),
            0x0e1c => Some($($borrow)* $s.actors.controls.shake_request),
            0x0e1e => Some($($borrow)* $s.actors.drawing.tilemap_pending),
            0x0e40 => Some($($borrow)* $s.actors.combat.last_power),
            0x0e44 => Some($($borrow)* $s.actors.controls.phase_counter),
            0x0e46 => Some($($borrow)* $s.actors.controls.selection_visits),
            0x0e48 => Some($($borrow)* $s.actors.controls.population_scratch),
            0x0e4a => Some($($borrow)* $s.actors.controls.compatibility_new_index),
            0x0e4c => Some($($borrow)* $s.actors.controls.first_free),
            0x0e4e => Some($($borrow)* $s.actors.controls.population_count),
            0x0e54 => Some($($borrow)* $s.actors.controls.selected),
            0x0e56 => Some($($borrow)* $s.actors.controls.compatibility_selected_backup),
            0x0e58 => Some($($borrow)* $s.actors.controls.compatibility_data_pointer),
            0x0e7a => Some($($borrow)* $s.actors.staging.descriptor_write_position),
            0x0e7c => Some($($borrow)* $s.actors.staging.transfer_source),
            0x0e7e => Some($($borrow)* $s.actors.staging.transfer_destination),
            0x1778 => Some($($borrow)* presentation_visual!($s, ($($borrow)*)).fireflea.timer),
            0x177a => Some($($borrow)* presentation_visual!($s, ($($borrow)*)).fireflea.flash_index),
            0x177c => Some($($borrow)* presentation_visual!($s, ($($borrow)*)).fireflea.compatibility_word),
            0x177e => Some($($borrow)* presentation_visual!($s, ($($borrow)*)).fireflea.darkness),
            0x1780 => Some($($borrow)* presentation_visual!($s, ($($borrow)*)).fireflea.compatibility_limit),
            0x1782 => Some($($borrow)* presentation_visual!($s, ($($borrow)*)).fireflea.compatibility_shade),
            0x179a => Some($($borrow)* $s.actors.drawing.tilemap_size),
            0x179c => Some($($borrow)* $s.actors.schedule.boss_id),
            0x17a4 => Some($($borrow)* $s.actors.schedule.active_write_position),
            0x17a6 => Some($($borrow)* $s.actors.schedule.interactive_write_position),
            0x17a8 => Some($($borrow)* $s.actors.schedule.compatibility_active_cursor),
            0x17aa => Some($($borrow)* $s.actors.schedule.compatibility_interactive_cursor),
            0x183c => Some($($borrow)* $s.actors.controls.process_all),
            0x1842 => Some($($borrow)* $s.timing.frame_counter),
            0x1846 => Some($($borrow)* $s.actors.compatibility.debug_enemy_index),
            0x1848 => Some($($borrow)* $s.actors.compatibility.log_index),
            0x185c => Some($($borrow)* $s.actors.diagnostics.index),
            0x185e => Some($($borrow)* $s.actors.diagnostics.freeze),
            0x1860 => Some($($borrow)* $s.actors.diagnostics.text.x),
            0x1862 => Some($($borrow)* $s.actors.diagnostics.text.y),
            0x1864 => Some($($borrow)* $s.actors.diagnostics.population_entry),
            0x1866 => Some($($borrow)* $s.actors.diagnostics.population_key),
            0x1868 => Some($($borrow)* $s.actors.diagnostics.spawn.x),
            0x186a => Some($($borrow)* $s.actors.diagnostics.spawn.y),
            0x18ae => Some($($borrow)* $s.actors.controls.projectile_contacts_disabled),
            0xef74 => Some($($borrow)* $s.actors.graphics.compatibility_write_position),
            0xef76 => Some($($borrow)* $s.actors.graphics.compatibility_next_tile),
            at @ 0x0e84..=0x0f66 => Some(presentation_queue!($s, entry, (at - 0x0e84) / 2, ($($borrow)*))),
            at @ 0x0f68..=0x0f76 => Some(presentation_queue!($s, size, (at - 0x0f68) / 2, ($($borrow)*))),
            at @ 0x1794..=0x1798 => Some($($borrow)* $s.actors.drawing.hook_work[(at - 0x1794) / 2]),
            at @ 0x179e..=0x17a2 => Some($($borrow)* $s.actors.schedule.compatibility_179e[(at - 0x179e) / 2]),
            at @ 0x17ac..=0x17ea => Some($($borrow)* $s.actors.schedule.active[(at - 0x17ac) / 2]),
            at @ 0x17ec..=0x182a => Some($($borrow)* $s.actors.schedule.interactive[(at - 0x17ec) / 2]),
            at @ 0x182c..=0x1832 => Some($($borrow)* $s.actors.schedule.colliding[(at - 0x182c) / 2]),
            at @ 0x1834..=0x183a => Some($($borrow)* $s.actors.schedule.compatibility_distances[(at - 0x1834) / 2]),
            at @ 0xef5c..=0xef62 => Some($($borrow)* $s.actors.graphics.definitions[(at - 0xef5c) / 2]),
            at @ 0xef64..=0xef6a => Some($($borrow)* $s.actors.graphics.tile_offsets[(at - 0xef64) / 2]),
            at @ 0xef6c..=0xef72 => Some($($borrow)* $s.actors.graphics.palette_routes[(at - 0xef6c) / 2]),
            _ => None,
        }
    };
}
macro_rules! common_byte {
    ($s:ident, $at:expr, ($($borrow:tt)*)) => {
        match $at {
            at @ 0x0f78..=0x1777 if (at - 0x0f78) % 64 == 0x2e =>
                Some($($borrow)* $s.actors.slots[(at - 0x0f78) / 64].bank),
            at @ 0x0f78..=0x1777 if (at - 0x0f78) % 64 == 0x2f =>
                Some($($borrow)* $s.actors.slots[(at - 0x0f78) / 64].hurt_time),
            at @ 0x7000..=0x77ff if (at - 0x7000) % 64 >= 0x2e =>
                Some($($borrow)* $s.actors.saved[(at - 0x7000) / 64].name[(at - 0x7000) % 64 - 0x2e]),
            at @ 0x0e5a..=0x0e79 => Some($($borrow)* $s.actors.staging.descriptor_prefix[at - 0x0e5a]),
            at @ 0x0e80..=0x0e83 => Some($($borrow)* $s.actors.staging.gap[at - 0x0e80]),
            at @ 0x1784..=0x178b => Some($($borrow)* $s.actors.compatibility.pre_hook[at - 0x1784]),
            at @ 0x178c..=0x178f => Some($($borrow)* $s.actors.drawing.hook_address[at - 0x178c]),
            at @ 0x1790..=0x1793 => Some($($borrow)* $s.actors.compatibility.post_hook[at - 0x1790]),
            at @ 0x186c..=0x186d => Some($($borrow)* $s.actors.diagnostics.gap[at - 0x186c]),
            _ => None,
        }
    };
}

pub(in crate::game) fn projected_byte(s: &Session, at: usize) -> Option<u8> {
    if let Some(value) = common_byte!(s, at, (&)) {
        return Some(*value);
    }
    common_word!(s, at & !1, (&)).map(|word| word.to_le_bytes()[at & 1])
}

/// Returns false without mutation for every address owned elsewhere.
pub(in crate::game) fn write_projected_byte(s: &mut Session, at: usize, value: u8) -> bool {
    if let Some(byte) = common_byte!(s, at, (&mut)) {
        *byte = value;
        return true;
    }
    if let Some(word) = common_word!(s, at & !1, (&mut)) {
        let mut bytes = word.to_le_bytes();
        bytes[at & 1] = value;
        *word = u16::from_le_bytes(bytes);
        return true;
    }
    false
}

/// Imports only new common fields after a whole-span preflight. No partial
/// Session mutation on short input, and no identity/lifetime reconstruction.
pub(in crate::game) fn import_common(
    s: &mut Session,
    bytes: &impl Fn(usize) -> u8,
    byte_len: usize,
) {
    assert!(byte_len >= COMMON_END, "short actor common import");
    for (start, end) in COMMON_RANGES {
        for at in start..end {
            let written = write_projected_byte(s, at, bytes(at));
            debug_assert!(written, "missing actor common byte {at:x}");
        }
    }
}

/// Each new owned byte is emitted once, including raw byte-array pairs.
pub(in crate::game) fn visit_common(s: &Session, mut visit: impl FnMut(usize, u16)) {
    for (start, end) in COMMON_RANGES {
        for at in (start..end).step_by(2) {
            let low = projected_byte(s, at).expect("actor common low byte");
            let high = projected_byte(s, at + 1).expect("actor common high byte");
            visit(at, u16::from_le_bytes([low, high]));
        }
    }
}

/// Preflight is before the first write, including any byte-array tail.
pub(in crate::game) fn export_common(
    s: &Session,
    byte_len: usize,
    mut span: impl FnMut(usize, &[u8]),
) {
    assert!(byte_len >= COMMON_END, "short actor common export");
    visit_common(s, |at, value| span(at, &value.to_le_bytes()));
}

pub(super) fn read_actor<H: ?Sized>(
    host: &H,
    key: ActorKey,
    mode: AccessMode,
    read_byte: fn(&H, usize) -> u8,
) -> Actor {
    let base = 0x0f78 + usize::from(key.byte_index());
    let mut actor = Actor::default();
    for offset in (0..64).step_by(2) {
        let bytes = [
            read_byte(host, base + offset),
            read_byte(host, base + offset + 1),
        ];
        if offset == 0x2e {
            actor.bank = bytes[0];
            actor.hurt_time = bytes[1];
        } else {
            let value = match mode {
                AccessMode::LittleEndian => u16::from_le_bytes(bytes),
                AccessMode::Native => u16::from_ne_bytes(bytes),
            };
            *actor_word!(actor, offset, (&mut)).expect("actor word lane") = value;
        }
    }
    actor
}

/// Publishes complete changed word fields and independently changed byte fields.
/// Caller guarantees a non-diagnostic, infallible writer for the admitted u16
/// window (exclusive end at most 0x10FB7, within the 0x20000 RAM allocation).
/// No reads, content lookup, identity transitions or effects occur during commit.
pub(super) fn write_actor_changes<H: ?Sized>(
    host: &mut H,
    key: ActorKey,
    mode: AccessMode,
    original: &Actor,
    edited: &Actor,
    write_byte: fn(&mut H, usize, u8),
) {
    let base = 0x0f78 + usize::from(key.byte_index());
    for offset in (0..64).step_by(2) {
        if offset == 0x2e {
            for (lane, before, after) in [
                (0, original.bank, edited.bank),
                (1, original.hurt_time, edited.hurt_time),
            ] {
                if before != after {
                    write_byte(host, base + offset + lane, after);
                }
            }
        } else {
            let before = actor_word!(original, offset, (&)).expect("actor original lane");
            let after = actor_word!(edited, offset, (&)).expect("actor edited lane");
            if before != after {
                let bytes = match mode {
                    AccessMode::LittleEndian => after.to_le_bytes(),
                    AccessMode::Native => after.to_ne_bytes(),
                };
                write_byte(host, base + offset, bytes[0]);
                write_byte(host, base + offset + 1, bytes[1]);
            }
        }
    }
}

pub(super) fn read_spawn<H: ?Sized>(
    host: &H,
    key: ActorKey,
    mode: AccessMode,
    read_byte: fn(&H, usize) -> u8,
) -> SpawnRecord {
    let base = 0x7000 + usize::from(key.byte_index());
    let mut spawn = SpawnRecord::default();
    for offset in (0..0x2e).step_by(2) {
        let bytes = [
            read_byte(host, base + offset),
            read_byte(host, base + offset + 1),
        ];
        let value = match mode {
            AccessMode::LittleEndian => u16::from_le_bytes(bytes),
            AccessMode::Native => u16::from_ne_bytes(bytes),
        };
        *spawn_word!(spawn, offset, (&mut)).expect("spawn word lane") = value;
    }
    for (offset, byte) in spawn.name.iter_mut().enumerate() {
        *byte = read_byte(host, base + 0x2e + offset);
    }
    spawn
}

/// The saved window also admits every u16 key (exclusive end at most1703F).
/// As for actor edits, the supplied writer is infallible within actual RAM.
pub(super) fn write_spawn_changes<H: ?Sized>(
    host: &mut H,
    key: ActorKey,
    mode: AccessMode,
    original: &SpawnRecord,
    edited: &SpawnRecord,
    write_byte: fn(&mut H, usize, u8),
) {
    let base = 0x7000 + usize::from(key.byte_index());
    for offset in (0..0x2e).step_by(2) {
        let before = spawn_word!(original, offset, (&)).expect("spawn original lane");
        let after = spawn_word!(edited, offset, (&)).expect("spawn edited lane");
        if before != after {
            let bytes = match mode {
                AccessMode::LittleEndian => after.to_le_bytes(),
                AccessMode::Native => after.to_ne_bytes(),
            };
            write_byte(host, base + offset, bytes[0]);
            write_byte(host, base + offset + 1, bytes[1]);
        }
    }
    for (offset, (&before, &after)) in original.name.iter().zip(&edited.name).enumerate() {
        if before != after {
            write_byte(host, base + 0x2e + offset, after);
        }
    }
}

// The supplied audio component extends this codec's explicit LE field encoding.
// It shares the existing bounded checkpoint cursor, with no machine/RAM access.
use super::snapshot::CheckpointReader;
use crate::game::scene::{
    component::DisplayStartMetadata,
    records::{AssetId, AssetRef, Revision},
};
use crate::game_state::persistent_session::{
    AudioRequestCatalog, AudioRequests, MusicContinuation, MusicMeaning, MusicRequests,
    QueuedMusicMeaning, QueuedSoundMeaning, SoundMeaning, SoundRequests,
};
use std::sync::Arc;

fn audio_asset(bytes: &mut Vec<u8>, asset: AssetRef) {
    bytes.extend_from_slice(&asset.id.0.to_le_bytes());
    bytes.extend_from_slice(&asset.revision.0);
}
fn audio_read_asset(read: &mut CheckpointReader<'_>) -> Result<AssetRef, String> {
    Ok(AssetRef {
        id: AssetId(read.id()?),
        revision: Revision(read.bytes(32)?.try_into().unwrap()),
    })
}
fn audio_continuation(bytes: &mut Vec<u8>, value: &MusicContinuation) {
    match value {
        MusicContinuation::Retain => bytes.push(0),
        MusicContinuation::Silence => bytes.push(1),
        MusicContinuation::Program { program, phase } => {
            bytes.push(2);
            audio_asset(bytes, *program);
            bytes.extend_from_slice(&phase.to_le_bytes());
        }
    }
}
fn audio_read_continuation(read: &mut CheckpointReader<'_>) -> Result<MusicContinuation, String> {
    match read.byte()? {
        0 => Ok(MusicContinuation::Retain),
        1 => Ok(MusicContinuation::Silence),
        2 => Ok(MusicContinuation::Program {
            program: audio_read_asset(read)?,
            phase: read.counter()?,
        }),
        _ => Err("unknown audio checkpoint continuation".into()),
    }
}
fn audio_music_meaning(bytes: &mut Vec<u8>, value: &MusicMeaning) {
    match value {
        MusicMeaning::Silence => bytes.push(0),
        MusicMeaning::Bank { bank, after } => {
            bytes.push(1);
            audio_asset(bytes, *bank);
            audio_continuation(bytes, after);
        }
        MusicMeaning::Program { program, phase } => {
            bytes.push(2);
            audio_asset(bytes, *program);
            bytes.extend_from_slice(&phase.to_le_bytes());
        }
    }
}
fn audio_music_record(bytes: &mut Vec<u8>, value: &Option<QueuedMusicMeaning>) {
    let Some(value) = value else {
        bytes.push(0);
        return;
    };
    bytes.push(1);
    bytes.extend_from_slice(&value.request.to_le_bytes());
    bytes.extend_from_slice(&value.source_tick.to_le_bytes());
    bytes.extend_from_slice(&value.source_interval.to_le_bytes());
    audio_music_meaning(bytes, &value.meaning);
}
fn audio_read_music_record(
    read: &mut CheckpointReader<'_>,
) -> Result<Option<QueuedMusicMeaning>, String> {
    match read.byte()? {
        0 => return Ok(None),
        1 => {}
        _ => return Err("unknown audio checkpoint music option".into()),
    }
    let request = read.word()?;
    let source_tick = read.counter()?;
    let source_interval = read.counter()?;
    let meaning = match read.byte()? {
        0 => MusicMeaning::Silence,
        1 => MusicMeaning::Bank {
            bank: audio_read_asset(read)?,
            after: audio_read_continuation(read)?,
        },
        2 => MusicMeaning::Program {
            program: audio_read_asset(read)?,
            phase: read.counter()?,
        },
        _ => return Err("unknown audio checkpoint music meaning".into()),
    };
    Ok(Some(QueuedMusicMeaning {
        request,
        source_tick,
        source_interval,
        meaning,
    }))
}
fn audio_sound_meaning(bytes: &mut Vec<u8>, value: &SoundMeaning) {
    match value {
        SoundMeaning::Stop => bytes.push(0),
        SoundMeaning::Program { program, phase } => {
            bytes.push(1);
            audio_asset(bytes, *program);
            bytes.extend_from_slice(&phase.to_le_bytes());
        }
    }
}
fn audio_sound_record(bytes: &mut Vec<u8>, value: &Option<QueuedSoundMeaning>) {
    let Some(value) = value else {
        bytes.push(0);
        return;
    };
    bytes.push(1);
    bytes.push(value.request);
    bytes.extend_from_slice(&value.source_tick.to_le_bytes());
    bytes.extend_from_slice(&value.source_interval.to_le_bytes());
    audio_sound_meaning(bytes, &value.meaning);
}
fn audio_read_sound_record(
    read: &mut CheckpointReader<'_>,
) -> Result<Option<QueuedSoundMeaning>, String> {
    match read.byte()? {
        0 => return Ok(None),
        1 => {}
        _ => return Err("unknown audio checkpoint sound option".into()),
    }
    let request = read.byte()?;
    let source_tick = read.counter()?;
    let source_interval = read.counter()?;
    let meaning = match read.byte()? {
        0 => SoundMeaning::Stop,
        1 => SoundMeaning::Program {
            program: audio_read_asset(read)?,
            phase: read.counter()?,
        },
        _ => return Err("unknown audio checkpoint sound meaning".into()),
    };
    Ok(Some(QueuedSoundMeaning {
        request,
        source_tick,
        source_interval,
        meaning,
    }))
}
fn audio_words<const N: usize>(read: &mut CheckpointReader<'_>) -> Result<[u16; N], String> {
    let mut values = Vec::with_capacity(N);
    for _ in 0..N {
        values.push(read.word()?);
    }
    values
        .try_into()
        .map_err(|_| "audio checkpoint word count".into())
}
fn audio_music_records<const N: usize>(
    read: &mut CheckpointReader<'_>,
) -> Result<[Option<QueuedMusicMeaning>; N], String> {
    let mut values = Vec::with_capacity(N);
    for _ in 0..N {
        values.push(audio_read_music_record(read)?);
    }
    values
        .try_into()
        .map_err(|_| "audio checkpoint music record count".into())
}
fn audio_sound_records<const N: usize>(
    read: &mut CheckpointReader<'_>,
) -> Result<[Option<QueuedSoundMeaning>; N], String> {
    let mut values = Vec::with_capacity(N);
    for _ in 0..N {
        values.push(audio_read_sound_record(read)?);
    }
    values
        .try_into()
        .map_err(|_| "audio checkpoint sound record count".into())
}
fn audio_catalog_bytes(catalog: &AudioRequestCatalog) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(catalog.music.len() as u64).to_le_bytes());
    for (selector, meaning) in &catalog.music {
        bytes.extend_from_slice(&selector.to_le_bytes());
        audio_music_meaning(&mut bytes, meaning);
    }
    bytes.extend_from_slice(&(catalog.sounds.len() as u64).to_le_bytes());
    for ((channel, selector), meaning) in &catalog.sounds {
        bytes.extend_from_slice(&(*channel as u64).to_le_bytes());
        bytes.push(*selector);
        audio_sound_meaning(&mut bytes, meaning);
    }
    bytes
}
impl AudioRequests {
    /// Complete detached request component. Capture rejects any pending publication.
    pub(crate) fn encode_checkpoint(
        &self,
        metadata: &DisplayStartMetadata,
        required: &AudioRequestCatalog,
    ) -> Result<Vec<u8>, String> {
        self.validate(metadata, required)
            .map_err(|error| format!("audio checkpoint {error:?}"))?;
        if metadata.identity.reference != metadata.reference.fingerprint {
            return Err("audio checkpoint Reference identity mismatch".into());
        }
        let mut bytes = Vec::from(b"SMAUDIO\x02".as_slice());
        let reference = audio_reference_bytes(&metadata.reference);
        bytes.extend_from_slice(&(reference.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&reference);
        bytes.push(u8::from(self.pending_mutation));
        let catalog_bytes = audio_catalog_bytes(&self.catalog);
        bytes.extend_from_slice(&(catalog_bytes.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&catalog_bytes);
        bytes.extend_from_slice(&self.uploading.to_le_bytes());
        bytes.extend_from_slice(&self.downtime.to_le_bytes());
        for word in self.music.tracks.iter().chain(&self.music.delays) {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        for word in [
            self.music.write,
            self.music.read,
            self.music.entry,
            self.music.timer,
            self.music.data_index,
            self.music.track_index,
        ] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes.push(self.music.current_track);
        for record in &self.music.meanings {
            audio_music_record(&mut bytes, record);
        }
        audio_music_record(&mut bytes, &self.music.entry_meaning);
        audio_music_record(&mut bytes, &self.music.current_meaning);
        for sound in &self.sounds {
            bytes.extend_from_slice(&[
                sound.read,
                sound.write,
                sound.state,
                sound.current,
                sound.clear_delay,
                sound.maximum_queued,
            ]);
            bytes.extend_from_slice(&sound.queue);
            for record in &sound.meanings {
                audio_sound_record(&mut bytes, record);
            }
            audio_sound_record(&mut bytes, &sound.current_meaning);
        }
        Ok(bytes)
    }
    /// Decode into a new value against independent declarations, before adoption.
    /// Catalog meanings are compared to explicit encoded records, never inferred
    /// from queue numbers. Unknown tags, trailing bytes and missing records reject.
    pub(crate) fn decode_checkpoint(
        bytes: &[u8],
        metadata: &DisplayStartMetadata,
        catalog: Arc<AudioRequestCatalog>,
    ) -> Result<Self, String> {
        let mut read = CheckpointReader(bytes);
        if read.bytes(8)? != b"SMAUDIO\x02" {
            return Err("unsupported audio checkpoint schema".into());
        }
        let reference_size = usize::try_from(read.counter()?)
            .map_err(|_| "audio checkpoint Reference length overflow")?;
        if metadata.identity.reference != metadata.reference.fingerprint
            || read.bytes(reference_size)? != audio_reference_bytes(&metadata.reference)
        {
            return Err("audio checkpoint Reference declaration changed".into());
        }
        let pending_mutation = match read.byte()? {
            0 => false,
            _ => return Err("incomplete audio checkpoint publication".into()),
        };
        let catalog_size = usize::try_from(read.counter()?)
            .map_err(|_| "audio checkpoint catalog length overflow")?;
        if read.bytes(catalog_size)? != audio_catalog_bytes(&catalog) {
            return Err("audio checkpoint catalog declarations changed".into());
        }
        let uploading = read.word()?;
        let downtime = read.word()?;
        let tracks = audio_words(&mut read)?;
        let delays = audio_words(&mut read)?;
        let write = read.word()?;
        let music_read = read.word()?;
        let entry = read.word()?;
        let timer = read.word()?;
        let data_index = read.word()?;
        let track_index = read.word()?;
        let current_track = read.byte()?;
        let meanings = audio_music_records(&mut read)?;
        let entry_meaning = audio_read_music_record(&mut read)?;
        let current_meaning = audio_read_music_record(&mut read)?;
        let music = MusicRequests {
            tracks,
            delays,
            write,
            read: music_read,
            entry,
            timer,
            data_index,
            track_index,
            current_track,
            meanings,
            entry_meaning,
            current_meaning,
        };
        let mut sounds = Vec::with_capacity(3);
        for _ in 0..3 {
            sounds.push(SoundRequests {
                read: read.byte()?,
                write: read.byte()?,
                state: read.byte()?,
                current: read.byte()?,
                clear_delay: read.byte()?,
                maximum_queued: read.byte()?,
                queue: read.bytes(16)?.try_into().unwrap(),
                meanings: audio_sound_records(&mut read)?,
                current_meaning: audio_read_sound_record(&mut read)?,
            });
        }
        if !read.0.is_empty() {
            return Err("trailing audio checkpoint fields".into());
        }
        let result = Self {
            music,
            sounds: sounds
                .try_into()
                .map_err(|_| "audio checkpoint channel count")?,
            uploading,
            downtime,
            catalog,
            pending_mutation,
        };
        result
            .validate(metadata, &result.catalog)
            .map_err(|error| format!("audio checkpoint {error:?}"))?;
        Ok(result)
    }
}

use crate::game::scene::start::{
    ActionBindings, AudioProgramDeclaration, CommittedPhase, ContinuationHistory,
    InitialAudioIntent, InputHistory, OngoingMusic, PortHistory, ProgramAnchor, ReferenceProfile,
    StartIdentity,
};
use crate::game_state::persistent_session::{Control, CurrentControl, SessionStartRequirements};

fn audio_reference_bytes(value: &ReferenceProfile) -> Vec<u8> {
    let mut bytes = Vec::new();
    for identity in [
        value.fingerprint,
        value.release,
        value.capture,
        value.arithmetic_policy,
        value.input_policy,
        value.observation_policy,
        value.color_policy,
        value.slowdown_policy,
        value.glitch_policy,
        value.audio_timing_policy,
    ] {
        bytes.extend_from_slice(&identity);
    }
    bytes.extend_from_slice(&value.ticks_numerator.to_le_bytes());
    bytes.extend_from_slice(&value.ticks_denominator.to_le_bytes());
    for word in [value.image.width, value.image.height] {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    bytes.extend_from_slice(&value.image.crop_origin.x.to_le_bytes());
    bytes.extend_from_slice(&value.image.crop_origin.y.to_le_bytes());
    bytes.push(match value.image.color {
        crate::game::scene::records::ColorInterpretation::Bgr15Native => 0,
    });
    bytes.push(match value.image.quantization {
        crate::game::scene::records::SceneQuantization::Native => 0,
        crate::game::scene::records::SceneQuantization::Reference => 1,
    });
    bytes
}
fn control_identity(read: &mut CheckpointReader<'_>) -> Result<[u8; 32], String> {
    Ok(read.bytes(32)?.try_into().unwrap())
}
fn control_bool(read: &mut CheckpointReader<'_>) -> Result<bool, String> {
    match read.byte()? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err("unknown Control checkpoint boolean".into()),
    }
}
fn control_anchor(bytes: &mut Vec<u8>, value: &ProgramAnchor) {
    bytes.extend_from_slice(&value.program.0.to_le_bytes());
    bytes.extend_from_slice(&value.revision.0);
    bytes.extend_from_slice(&value.source_interval.to_le_bytes());
    bytes.extend_from_slice(&value.phase.to_le_bytes());
    bytes.extend_from_slice(&value.timing_policy);
}
fn control_read_anchor(read: &mut CheckpointReader<'_>) -> Result<ProgramAnchor, String> {
    Ok(ProgramAnchor {
        program: AssetId(read.id()?),
        revision: Revision(control_identity(read)?),
        source_interval: read.counter()?,
        phase: read.counter()?,
        timing_policy: control_identity(read)?,
    })
}
impl Control {
    /// Lossless current Control component; immutable inputs are explicitly supplied.
    pub(crate) fn encode_checkpoint(
        &self,
        required: &SessionStartRequirements,
    ) -> Result<Vec<u8>, String> {
        let Self::Owned(current) = self else {
            return Err("no supplied Control checkpoint owner".into());
        };
        let m = &current.metadata;
        crate::game::scene::start::validate_control_metadata(m, &required.display)
            .map_err(|error| format!("Control checkpoint {error:?}"))?;
        let audio_requests = current
            .requests
            .encode_checkpoint(m, &required.audio_catalog)?;
        let mut bytes = Vec::from(b"SMCONTROL\x01".as_slice());
        bytes.extend_from_slice(&m.identity.schema.to_le_bytes());
        for identity in [
            m.identity.gameplay,
            m.identity.reference,
            m.identity.capture,
            m.identity.media.0,
        ] {
            bytes.extend_from_slice(&identity);
        }
        bytes.extend_from_slice(&m.identity.committed_tick.to_le_bytes());
        bytes.extend_from_slice(&m.identity.next_input_record.to_le_bytes());
        let reference = audio_reference_bytes(&m.reference);
        bytes.extend_from_slice(&(reference.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&reference);
        bytes.extend_from_slice(&m.samus_content.revision.0);
        let input = m.input_history;
        bytes.extend_from_slice(&input.capture);
        bytes.extend_from_slice(&input.policy);
        for port in input.ports {
            for word in [
                port.sampled,
                port.previous,
                port.pressed,
                port.repeated,
                port.repeat_counter,
            ] {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        bytes.push(u8::from(input.debug_second_port));
        for word in [
            input.repeat_first,
            input.repeat_next,
            input.released,
            input.timed_held,
            input.previous_timed_held,
            input.newly_held,
            input.held_timer,
            input.held_timer_reset,
            input.samus_held,
            input.samus_pressed,
            input.bindings.shoot,
            input.bindings.jump,
            input.bindings.run,
            input.bindings.cancel,
            input.bindings.switch,
            input.bindings.aim_down,
            input.bindings.aim_up,
            input.bug_fix_counter,
        ] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        let continuation = m.continuation;
        bytes.extend_from_slice(&continuation.capture);
        bytes.extend_from_slice(&continuation.observation_policy);
        bytes.push(match continuation.phase {
            CommittedPhase::InitialObservation => 0,
            CommittedPhase::SettledInterval => 1,
        });
        bytes.extend_from_slice(&continuation.next_input_record.to_le_bytes());
        bytes.extend_from_slice(&[
            continuation.outer,
            continuation.primary,
            continuation.secondary,
            continuation.music,
            continuation.message,
            continuation.auxiliary,
        ]);
        match continuation.captured_mode {
            None => bytes.push(0),
            Some(mode) => {
                bytes.push(1);
                bytes.extend_from_slice(&mode.to_le_bytes());
            }
        }
        bytes.extend_from_slice(&m.audio.capture);
        bytes.extend_from_slice(&m.audio.revision.0);
        bytes.extend_from_slice(&m.audio.timing_policy);
        bytes.extend_from_slice(&(m.audio.declarations.len() as u64).to_le_bytes());
        for value in m.audio.declarations.iter() {
            bytes.extend_from_slice(&value.program.0.to_le_bytes());
            bytes.extend_from_slice(&value.bank.0.to_le_bytes());
            bytes.extend_from_slice(&value.revision.0);
            bytes.extend_from_slice(&value.definition);
        }
        match m.audio.music {
            OngoingMusic::Silent => bytes.push(0),
            OngoingMusic::Playing(anchor) => {
                bytes.push(1);
                control_anchor(&mut bytes, &anchor);
            }
        }
        bytes.extend_from_slice(&(m.audio.ambience.len() as u64).to_le_bytes());
        for anchor in m.audio.ambience.iter() {
            control_anchor(&mut bytes, anchor);
        }
        bytes.extend_from_slice(&(audio_requests.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&audio_requests);
        Ok(bytes)
    }
    /// Reconstruct only serialized mutable state. Resolution uses the declared
    /// Reference/catalog; no before/mine/default/live-RAM values are imported.
    pub(crate) fn decode_checkpoint(
        bytes: &[u8],
        required: &SessionStartRequirements,
    ) -> Result<Self, String> {
        let mut read = CheckpointReader(bytes);
        if read.bytes(10)? != b"SMCONTROL\x01" {
            return Err("unsupported Control checkpoint schema".into());
        }
        let identity = StartIdentity {
            schema: read.word()?,
            gameplay: control_identity(&mut read)?,
            reference: control_identity(&mut read)?,
            capture: control_identity(&mut read)?,
            media: Revision(control_identity(&mut read)?),
            committed_tick: read.counter()?,
            next_input_record: read.counter()?,
        };
        let reference_size =
            usize::try_from(read.counter()?).map_err(|_| "Control reference length overflow")?;
        if read.bytes(reference_size)? != audio_reference_bytes(&required.display.reference) {
            return Err("Control checkpoint Reference declaration changed".into());
        }
        if control_identity(&mut read)? != required.display.samus_content.revision.0 {
            return Err("Control checkpoint Samus catalog revision changed".into());
        }
        let capture = control_identity(&mut read)?;
        let policy = control_identity(&mut read)?;
        let mut ports = Vec::with_capacity(2);
        for _ in 0..2 {
            ports.push(PortHistory {
                sampled: read.word()?,
                previous: read.word()?,
                pressed: read.word()?,
                repeated: read.word()?,
                repeat_counter: read.word()?,
            });
        }
        let input_history = InputHistory {
            capture,
            policy,
            ports: ports.try_into().map_err(|_| "Control port count")?,
            debug_second_port: control_bool(&mut read)?,
            repeat_first: read.word()?,
            repeat_next: read.word()?,
            released: read.word()?,
            timed_held: read.word()?,
            previous_timed_held: read.word()?,
            newly_held: read.word()?,
            held_timer: read.word()?,
            held_timer_reset: read.word()?,
            samus_held: read.word()?,
            samus_pressed: read.word()?,
            bindings: ActionBindings {
                shoot: read.word()?,
                jump: read.word()?,
                run: read.word()?,
                cancel: read.word()?,
                switch: read.word()?,
                aim_down: read.word()?,
                aim_up: read.word()?,
            },
            bug_fix_counter: read.word()?,
        };
        let capture = control_identity(&mut read)?;
        let observation_policy = control_identity(&mut read)?;
        let phase = match read.byte()? {
            0 => CommittedPhase::InitialObservation,
            1 => CommittedPhase::SettledInterval,
            _ => return Err("unknown Control continuation phase".into()),
        };
        let next_input_record = read.counter()?;
        let outer = read.byte()?;
        let primary = read.byte()?;
        let secondary = read.byte()?;
        let music = read.byte()?;
        let message = read.byte()?;
        let auxiliary = read.byte()?;
        let captured_mode = match read.byte()? {
            0 => None,
            1 => Some(read.word()?),
            _ => return Err("unknown Control captured mode option".into()),
        };
        let continuation = ContinuationHistory {
            capture,
            observation_policy,
            phase,
            next_input_record,
            outer,
            primary,
            secondary,
            music,
            message,
            auxiliary,
            captured_mode,
        };
        let capture = control_identity(&mut read)?;
        let revision = Revision(control_identity(&mut read)?);
        let timing_policy = control_identity(&mut read)?;
        let count = usize::try_from(read.counter()?)
            .map_err(|_| "Control audio declaration length overflow")?;
        if count != required.display.audio_programs.len() {
            return Err("Control audio declaration count changed".into());
        }
        let mut declarations = Vec::with_capacity(count);
        for _ in 0..count {
            declarations.push(AudioProgramDeclaration {
                program: AssetId(read.id()?),
                bank: AssetId(read.id()?),
                revision: Revision(control_identity(&mut read)?),
                definition: control_identity(&mut read)?,
            });
        }
        let music = match read.byte()? {
            0 => OngoingMusic::Silent,
            1 => OngoingMusic::Playing(control_read_anchor(&mut read)?),
            _ => return Err("unknown Control music anchor".into()),
        };
        let count =
            usize::try_from(read.counter()?).map_err(|_| "Control ambience length overflow")?;
        if count > read.0.len() / 84 {
            return Err("truncated Control ambience".into());
        }
        let mut ambience = Vec::with_capacity(count);
        for _ in 0..count {
            ambience.push(control_read_anchor(&mut read)?);
        }
        let audio = InitialAudioIntent {
            capture,
            revision,
            timing_policy,
            declarations: declarations.into(),
            music,
            ambience: ambience.into(),
        };
        let metadata = DisplayStartMetadata {
            identity,
            reference: required.display.reference,
            input_history,
            continuation,
            audio,
            samus_content: required.display.samus_content.clone(),
        };
        crate::game::scene::start::validate_control_metadata(&metadata, &required.display)
            .map_err(|error| format!("Control checkpoint {error:?}"))?;
        let requests_size =
            usize::try_from(read.counter()?).map_err(|_| "Control requests length overflow")?;
        let requests = AudioRequests::decode_checkpoint(
            read.bytes(requests_size)?,
            &metadata,
            required.audio_catalog.clone(),
        )?;
        if !read.0.is_empty() {
            return Err("trailing Control checkpoint fields".into());
        }
        Ok(Self::Owned(Box::new(CurrentControl { metadata, requests })))
    }
}
