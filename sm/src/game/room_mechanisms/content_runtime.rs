//! Owned content records and live dynamic/native drawing at the runtime edge.
#![forbid(unsafe_code)]
use super::host::{
    Content, ContentAddress, ContentTable, Diagnostic, DrawCursor, DrawProgramId, Effects,
};
use crate::{
    game::session_runtime::{Runtime, invalid_window},
    rom::RomRecord,
};

pub enum TerrainProgram {
    Loaded(RomRecord),
    // Identity only: every read resolves the current aggregate/foreign projection.
    Dynamic(usize),
}
fn address(value: ContentAddress) -> u32 {
    (u32::from(value.bank) << 16) | u32::from(value.offset)
}
impl Content for Runtime<'_> {
    type Record = RomRecord;
    type TerrainProgram = TerrainProgram;
    fn loaded_record(&mut self, location: ContentAddress) -> RomRecord {
        let absolute = address(location);
        if location.offset & 0x8000 == 0 {
            self.foreign(|g| invalid_window(g, absolute));
        }
        RomRecord {
            image: self.loaded_image().expect("ROM must be loaded"),
            base: crate::rtl::lorom_offset(absolute) as usize,
        }
    }
    fn native_record(&mut self, location: ContentAddress) -> RomRecord {
        let absolute = address(location);
        #[cfg(sm_verify_dll)]
        self.foreign(|game| {
            crate::game::room_mechanism_checks::observe_native_draw_read(
                location.bank,
                location.offset,
                game,
            )
        });
        if location.offset & 0x8000 == 0 {
            self.foreign(|g| invalid_window(g, absolute));
        }
        RomRecord {
            image: self.native_image(),
            base: crate::rtl::lorom_offset(absolute) as usize,
        }
    }
    fn definition_word(&mut self, location: ContentAddress, offset: usize) -> u16 {
        self.foreign(|game| game.read_rom_word(address(location), offset))
    }
    fn table_word(&self, table: ContentTable, index: isize) -> u16 {
        let (base, little) = match table {
            ContentTable::Gate { up, byte_offset } => (
                crate::rtl::lorom_offset(if up { 0x84c764 } else { 0x84c70a }) as usize
                    + byte_offset,
                false,
            ),
            ContentTable::MetroidWatcher => (0x25b28, false),
            ContentTable::ItemGraphicDestination => {
                (crate::rtl::lorom_offset(0x8487cd) as usize, true)
            }
            ContentTable::ItemTileOffset => (crate::rtl::lorom_offset(0x8487d5) as usize, true),
            ContentTable::ItemTileNumber => (crate::rtl::lorom_offset(0x8487dd) as usize, true),
            ContentTable::ItemReveal => (crate::rtl::lorom_offset(0x84839d) as usize, false),
        };
        let image = self.loaded_image().expect("ROM must be loaded");
        if !little {
            return image.table_word(base, index);
        }
        let at = base
            .checked_add_signed(index.checked_mul(2).expect("ROM table offset overflow"))
            .expect("ROM table offset overflow");
        u16::from_le_bytes([image.read(at), image.read(at + 1)])
    }
    fn capture_terrain_program(&mut self, id: DrawProgramId) -> TerrainProgram {
        if id.0 & 0x8000 != 0 {
            TerrainProgram::Loaded(RomRecord {
                image: self.loaded_image().expect("ROM must be loaded"),
                base: crate::rtl::lorom_offset(0x840000 | u32::from(id.0)) as usize,
            })
        } else {
            if id.0 >= 0x2000 {
                self.diagnostic(Diagnostic::InvalidRamDraw(id.0));
            }
            TerrainProgram::Dynamic(usize::from(id.0))
        }
    }
    fn terrain_program_word(&self, program: &TerrainProgram, offset: usize) -> u16 {
        match program {
            TerrainProgram::Loaded(record) => record.word(offset),
            TerrainProgram::Dynamic(base) => self.native_word(
                base.checked_add(offset)
                    .expect("mechanism terrain cursor overflow"),
            ),
        }
    }
    fn drawing_word(&mut self, cursor: DrawCursor) -> u16 {
        if cursor.0 & 0x8000 != 0 {
            // Every draw operand independently selects the live native binding.
            Content::native_record(
                self,
                ContentAddress {
                    bank: 0x84,
                    offset: cursor.0,
                },
            )
            .word(0)
        } else {
            if cursor.0 >= 0x2000 {
                self.diagnostic(Diagnostic::InvalidRamDraw(cursor.0));
            }
            self.native_word(usize::from(cursor.0))
        }
    }
    fn drawing_byte(&mut self, cursor: DrawCursor) -> u8 {
        if cursor.0 & 0x8000 != 0 {
            Content::native_record(
                self,
                ContentAddress {
                    bank: 0x84,
                    offset: cursor.0,
                },
            )
            .byte(0)
        } else {
            if cursor.0 >= 0x2000 {
                self.diagnostic(Diagnostic::InvalidRamDraw(cursor.0));
            }
            self.byte(usize::from(cursor.0))
        }
    }
}
