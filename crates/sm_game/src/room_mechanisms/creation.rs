//! Descending allocation and ordered creation over the owned Session.
use super::access::*;
use super::host::*;
use crate::progress as flags;
#[derive(Clone, Copy, PartialEq)]
enum SourceKind {
    Collision,
    Hardcoded,
    Room,
}
enum Entry<R> {
    Scratch,
    Record(R),
}
enum Definition<R> {
    Collision(ContentAddress),
    Placed(R, SourceKind),
}
fn entry_byte(c: &impl Host, entry: &Entry<impl InstructionRecord>, offset: usize) -> u8 {
    match entry {
        Entry::Scratch => {
            let scratch = &c.session().spawn_scratch;
            let word = match offset / 2 {
                0 => scratch.header,
                1 => scratch.coordinates,
                2 => scratch.argument,
                _ => unreachable!(),
            };
            word.to_le_bytes()[offset % 2]
        }
        Entry::Record(record) => record.byte(offset as isize),
    }
}
fn entry_word(c: &impl Host, entry: &Entry<impl InstructionRecord>, offset: usize) -> u16 {
    u16::from_le_bytes([
        entry_byte(c, entry, offset),
        entry_byte(c, entry, offset + 1),
    ])
}
fn definition_word(
    c: &mut impl Host,
    definition: &Definition<impl InstructionRecord>,
    offset: usize,
) -> u16 {
    match definition {
        Definition::Collision(address) => c.definition_word(*address, offset),
        Definition::Placed(record, SourceKind::Room) => u16::from_le(record.word(offset as isize)),
        Definition::Placed(record, _) => record.word(offset as isize),
    }
}
/// Spawn the hardcoded PLM `header` at block (`x`, `y`) on a fresh lease of
/// `game` (SpawnHardcodedPlm).
pub fn spawn_hardcoded<H>(x: u8, y: u8, header: u16, game: &mut H)
where
    H: crate::runtime_access::RuntimeAccess + ?Sized,
{
    let source = SpawnSource::Hardcoded { x, y, header };
    spawn(&mut game.runtime(), source, &mut Contact::Absent);
}

pub fn spawn(c: &mut impl Host, source: SpawnSource, displacement: &mut Contact<'_>) -> u8 {
    let Some(slot) = (0..40)
        .rev()
        .find(|&slot| c.session().mechanisms.slots[slot].header == 0)
    else {
        return 0;
    };
    let (header, kind) = match source {
        SpawnSource::Collision { header } => {
            c.probe("mechanisms.spawn.collision");
            let block = c.session().room.collision_cursor.wrapping_mul(2);
            c.session_mut().mechanisms.slots[slot].block = block;
            c.session_mut().mechanisms.slots[slot].header = header;
            c.session_mut().mechanisms.slots[slot].pre_action = 0x853d;
            (header, SourceKind::Collision)
        }
        SpawnSource::Hardcoded { x, y, header } => {
            c.probe("mechanisms.spawn.hardcoded");
            let row = u16::from(y) * u16::from(c.session().room.width as u8);
            c.session_mut().mechanisms.slots[slot].block =
                row.wrapping_add(u16::from(x)).wrapping_mul(2);
            c.session_mut().mechanisms.slots[slot].header = header;
            c.session_mut().mechanisms.slots[slot].room_argument = 0;
            c.session_mut().mechanisms.slots[slot].work1 = 0;
            c.session_mut().mechanisms.slots[slot].pre_action = 0x8469;
            (header, SourceKind::Hardcoded)
        }
        SpawnSource::Room {
            entry: ProgramCursor(address),
        } => {
            c.probe("mechanisms.spawn.room");
            let entry = if address == 0x12 {
                Entry::Scratch
            } else {
                Entry::Record(c.loaded_record(ContentAddress {
                    bank: 0x8f,
                    offset: address,
                }))
            };
            let y = u16::from(entry_byte(c, &entry, 3));
            let row = y * u16::from(c.session().room.width as u8);
            let x = u16::from(entry_byte(c, &entry, 2));
            c.session_mut().mechanisms.slots[slot].block = row.wrapping_add(x).wrapping_mul(2);
            let argument = entry_word(c, &entry, 4);
            c.session_mut().mechanisms.slots[slot].room_argument = argument;
            let header = entry_word(c, &entry, 0);
            let stored = entry_word(c, &entry, 0);
            c.session_mut().mechanisms.slots[slot].header = stored;
            c.session_mut().mechanisms.slots[slot].work1 = 0;
            c.session_mut().mechanisms.slots[slot].pre_action = 0x84e6;
            (header, SourceKind::Room)
        }
        SpawnSource::Prepared {
            header,
            block,
            before,
            argument,
        } => {
            c.session_mut().mechanisms.slots[slot].block = block;
            c.session_mut().mechanisms.slots[slot].header = header;
            c.session_mut().mechanisms.slots[slot].room_argument = argument;
            c.session_mut().mechanisms.slots[slot].work1 = 0;
            c.session_mut().mechanisms.slots[slot].pre_action = before;
            (header, SourceKind::Hardcoded)
        }
    };
    let definition = if kind == SourceKind::Collision {
        Definition::Collision(ContentAddress {
            bank: 0x84,
            offset: header,
        })
    } else {
        Definition::Placed(
            c.loaded_record(ContentAddress {
                bank: 0x84,
                offset: header,
            }),
            kind,
        )
    };
    let program = definition_word(c, &definition, 2);
    c.session_mut().mechanisms.slots[slot].program = program;
    c.session_mut().mechanisms.slots[slot].instruction_delay = 1;
    c.session_mut().mechanisms.slots[slot].draw_program = 0x8da0;
    c.session_mut().mechanisms.slots[slot].counter = 0;
    if kind == SourceKind::Collision {
        c.session_mut().mechanisms.slots[slot].room_argument = 0;
        c.session_mut().mechanisms.slots[slot].work1 = 0;
    }
    c.session_mut().mechanisms.selected = (slot * 2) as u16;
    let setup = definition_word(c, &definition, 0);
    super::setup::setup(
        c,
        0x840000 | u32::from(setup),
        slot,
        displacement,
        if kind == SourceKind::Collision {
            0
        } else {
            0x8da0
        },
    )
}
pub fn close_door(c: &mut (impl Host + crate::room_entry::content::EntryContent)) {
    use crate::room_entry::content::Axis;
    let exit = c.session().room.entry.exit;
    let door = c.capture_closing_door(&exit);
    let row = u16::from(door.closing_position(Axis::Y)) * u16::from(c.session().room.width as u8);
    let block = row
        .wrapping_add(u16::from(door.closing_position(Axis::X)))
        .wrapping_mul(2);
    if let Some(slot) = (0..40)
        .rev()
        .find(|&slot| c.session().mechanisms.slots[slot].block == block)
        && c.session().mechanisms.slots[slot].header != 0
    {
        let argument = c.session().mechanisms.slots[slot].room_argument;
        if argument & 0x8000 != 0 || !flags::door_opened(c, argument) {
            c.session_mut().mechanisms.slots[slot].instruction_delay = 1;
            let header = c.session().mechanisms.slots[slot].header;
            if header != 0 {
                let record = c.loaded_record(ContentAddress {
                    bank: 0x84,
                    offset: header,
                });
                c.session_mut().mechanisms.slots[slot].program = record.word(4);
                return;
            }
        }
    }
    let direction = c.session().room.entry.direction;
    let header = c.capture_closing_header(direction);
    if header.word() != 0 {
        c.session_mut().spawn_scratch.header = u16::from_le_bytes(header.word().to_ne_bytes());
        let exit = c.session().room.entry.exit;
        let door = c.capture_closing_door(&exit);
        c.session_mut().spawn_scratch.coordinates =
            u16::from_le_bytes(door.closing_coordinates_native().to_ne_bytes());
        // The spawn reads its room argument from $8F:0016, the direct-page scratch word $16,
        // which still holds whatever its last writer left there.
        spawn(
            c,
            SpawnSource::Room {
                entry: ProgramCursor(0x12),
            },
            &mut Contact::Absent,
        );
    }
}

pub fn coordinates(c: &mut impl Host, slot: usize) {
    let byte_block = mechanism!(c, slot, block);
    coordinates_from_block(c.session_mut(), byte_block);
}
pub fn coordinates_from_block(session: &mut crate::session::Session, byte_block: u16) {
    let block = byte_block >> 1;
    let width = u16::from(session.room.width as u8);
    session.mechanisms.block_y = block.checked_div(width).unwrap_or(0xffff);
    let width = u16::from(session.room.width as u8);
    session.mechanisms.block_x = if width == 0 { block } else { block % width };
}

/// Empty every mechanism slot, last first, and restart graphics allocation.
pub fn clear_session(session: &mut crate::session::Session) {
    for slot in session.mechanisms.slots.iter_mut().rev() {
        slot.header = 0;
    }
    session.mechanisms.next_graphics = 0;
}
