//! Descending slot allocation and ordered lazy creation from each actual source.
#![forbid(unsafe_code)]
use super::{
    fields::{Fields, Global},
    port::Execution,
    state::Column,
};
use crate::game::mechanism_displacement::Displacement;
use crate::rom::RomRecord;
#[derive(Clone, Copy, PartialEq)]
enum SourceKind {
    Collision,
    Hardcoded,
    Room,
}
pub(crate) enum Source {
    Collision {
        header: u16,
    },
    Hardcoded {
        x: u8,
        y: u8,
        header: u16,
    },
    Room {
        address: u16,
    },
    Prepared {
        header: u16,
        block: u16,
        before: u16,
        argument: u16,
    },
}
enum Entry {
    Scratch,
    Record(RomRecord),
}
enum Definition {
    Collision(u32),
    Placed(RomRecord, SourceKind),
}
impl Execution<'_> {
    pub(super) fn record<const BANK: u8>(&mut self, address: u16) -> RomRecord {
        self.foreign(|g| super::effects::record::<BANK>(address, g))
    }
    fn entry_byte(&self, entry: &Entry, offset: usize) -> u8 {
        match entry {
            Entry::Scratch => self.port().byte(0x12 + offset),
            Entry::Record(record) => record.byte(offset),
        }
    }
    fn entry_word(&self, entry: &Entry, offset: usize) -> u16 {
        u16::from_le_bytes([
            self.entry_byte(entry, offset),
            self.entry_byte(entry, offset + 1),
        ])
    }
    fn definition_word(&mut self, definition: &Definition, offset: usize) -> u16 {
        match definition {
            Definition::Collision(address) => self.foreign(|g| g.read_rom_word(*address, offset)),
            Definition::Placed(record, SourceKind::Room) => u16::from_le(record.word(offset)),
            Definition::Placed(record, _) => record.word(offset),
        }
    }
    pub(super) fn spawn(&mut self, source: Source, displacement: &mut Displacement<'_>) -> u8 {
        let Some(slot) = (0..40)
            .rev()
            .find(|&slot| self.slot(Column::Header, slot) == 0)
        else {
            return 0;
        };
        let (header, kind) = match source {
            Source::Collision { header } => {
                crate::verify_probe::hit("mechanisms.spawn.collision");
                let block = self.global(Global::CurrentBlock).wrapping_mul(2);
                self.set_slot(Column::Block, slot, block);
                self.set_slot(Column::Header, slot, header);
                self.set_slot(Column::PreAction, slot, 0x853d);
                (header, SourceKind::Collision)
            }
            Source::Hardcoded { x, y, header } => {
                crate::verify_probe::hit("mechanisms.spawn.hardcoded");
                let row = u16::from(y) * u16::from(self.global(Global::RoomWidth) as u8);
                self.set_slot(
                    Column::Block,
                    slot,
                    row.wrapping_add(u16::from(x)).wrapping_mul(2),
                );
                self.set_slot(Column::Header, slot, header);
                self.set_slot(Column::RoomArgument, slot, 0);
                self.set_slot(Column::Work1, slot, 0);
                self.set_slot(Column::PreAction, slot, 0x8469);
                (header, SourceKind::Hardcoded)
            }
            Source::Room { address } => {
                crate::verify_probe::hit("mechanisms.spawn.room");
                let entry = if address == 0x12 {
                    Entry::Scratch
                } else {
                    Entry::Record(self.record::<0x8f>(address))
                };
                let y = u16::from(self.entry_byte(&entry, 3));
                let row = y * u16::from(self.global(Global::RoomWidth) as u8);
                let x = u16::from(self.entry_byte(&entry, 2));
                self.set_slot(Column::Block, slot, row.wrapping_add(x).wrapping_mul(2));
                let argument = self.entry_word(&entry, 4);
                self.set_slot(Column::RoomArgument, slot, argument);
                let header = self.entry_word(&entry, 0);
                let stored = self.entry_word(&entry, 0);
                self.set_slot(Column::Header, slot, stored);
                self.set_slot(Column::Work1, slot, 0);
                self.set_slot(Column::PreAction, slot, 0x84e6);
                (header, SourceKind::Room)
            }
            Source::Prepared {
                header,
                block,
                before,
                argument,
            } => {
                self.set_slot(Column::Block, slot, block);
                self.set_slot(Column::Header, slot, header);
                self.set_slot(Column::RoomArgument, slot, argument);
                self.set_slot(Column::Work1, slot, 0);
                self.set_slot(Column::PreAction, slot, before);
                (header, SourceKind::Hardcoded)
            }
        };
        let definition = if kind == SourceKind::Collision {
            Definition::Collision(0x840000 | u32::from(header))
        } else {
            Definition::Placed(self.record::<0x84>(header), kind)
        };
        let program = self.definition_word(&definition, 2);
        self.set_slot(Column::Program, slot, program);
        self.set_slot(Column::InstructionDelay, slot, 1);
        self.set_slot(Column::DrawProgram, slot, 0x8da0);
        self.set_slot(Column::Counter, slot, 0);
        if kind == SourceKind::Collision {
            self.set_slot(Column::RoomArgument, slot, 0);
            self.set_slot(Column::Work1, slot, 0);
        }
        self.set_global(Global::Selected, (slot * 2) as u16);
        let setup = self.definition_word(&definition, 0);
        self.setup(
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
    pub(super) fn close_door(&mut self) {
        let door = self.record::<0x83>(self.port().read_le(crate::ram_map::words::door_def_ptr));
        let row = u16::from(door.byte(5)) * u16::from(self.global(Global::RoomWidth) as u8);
        let block = row.wrapping_add(u16::from(door.byte(4))).wrapping_mul(2);
        if let Some(slot) = (0..40)
            .rev()
            .find(|&slot| self.slot(Column::Block, slot) == block)
        {
            if self.slot(Column::Header, slot) != 0 {
                let argument = self.slot(Column::RoomArgument, slot);
                if argument & 0x8000 != 0 || !self.door_opened(argument) {
                    self.set_slot(Column::InstructionDelay, slot, 1);
                    let header = self.slot(Column::Header, slot);
                    if header != 0 {
                        let record = self.record::<0x84>(header);
                        self.set_slot(Column::Program, slot, record.word(4));
                        return;
                    }
                }
            }
        }
        let address = self
            .port()
            .read_le(crate::ram_map::words::door_direction)
            .wrapping_mul(2)
            .wrapping_add(0xe68a);
        let header = self.record::<0x8f>(address);
        if header.word(0) != 0 {
            self.port_mut().write_slice_native(0x12, header.word(0));
            let door =
                self.record::<0x83>(self.port().read_le(crate::ram_map::words::door_def_ptr));
            self.port_mut().write_slice_native(0x14, door.word(4));
            self.port_mut().write_slice_native(0x16, 0);
            self.spawn(Source::Room { address: 0x12 }, &mut Displacement::Absent);
        }
    }
}
