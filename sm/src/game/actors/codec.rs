//! Single actor/common projection and changed-field codec.
#![forbid(unsafe_code)]
use super::{
    access::{AccessMode, ActorKey},
    state::{Actor, SpawnRecord},
};
use crate::game::session::Session;

// Both borrow modes use exactly the same lane map. Byte-only bank/hurt-time
// lanes are intentionally absent; no Rust struct layout is serialized.
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
            0x1778 => Some($($borrow)* $s.visual.fireflea.timer),
            0x177a => Some($($borrow)* $s.visual.fireflea.flash_index),
            0x177c => Some($($borrow)* $s.visual.fireflea.compatibility_word),
            0x177e => Some($($borrow)* $s.visual.fireflea.darkness),
            0x1780 => Some($($borrow)* $s.visual.fireflea.compatibility_limit),
            0x1782 => Some($($borrow)* $s.visual.fireflea.compatibility_shade),
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
            at @ 0x0e84..=0x0f66 => Some($($borrow)* $s.actors.drawing.queued_indices[(at - 0x0e84) / 2]),
            at @ 0x0f68..=0x0f76 => Some($($borrow)* $s.actors.drawing.queue_sizes[(at - 0x0f68) / 2]),
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
