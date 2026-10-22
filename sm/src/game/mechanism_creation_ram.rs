//! Checked compatibility boundary for the shared mechanism creation protocol.
use crate::game::constants::bank_84::addr_kDefaultPlmDrawInstruction;
use crate::game::mechanism_creation::{self, Context, Field, Source, SourceKind};
use crate::game::sm_84::CallPlmHeaderFunc;
use crate::game_state::Game;
use crate::ram_map::checked_word;
use crate::rom::RomRecord;

enum Entry {
    Scratch,
    Record(RomRecord),
}
enum Definition {
    Collision(u32),
    Placed(RomRecord, SourceKind),
}
struct Adapter<'a> {
    game: &'a mut Game,
    displacement: *mut i32,
}
impl Context for Adapter<'_> {
    type Header = u16;
    type Block = u16;
    type Pre = u16;
    type Program = u16;
    type Entry = Entry;
    type Definition = Definition;
    type Setup = u16;
    fn vacant(&self, slot: usize) -> bool {
        let game = &*self.game;
        checked_word!(game.plm_header_ptr[slot]) == 0
    }
    fn write(&mut self, slot: usize, field: Field<Self>) {
        let game = &mut *self.game;
        match field {
            Field::Block(v) => checked_word!(game.plm_block_indices[slot] = v),
            Field::Header(v) => checked_word!(game.plm_header_ptr[slot] = v),
            Field::Argument(v) => checked_word!(game.plm_room_arguments[slot] = v),
            Field::Auxiliary(v) => checked_word!(game.plm_variables[slot] = v),
            Field::Before(v) => checked_word!(game.plm_pre_instrs[slot] = v),
            Field::Program(v) => checked_word!(game.plm_instr_list_ptrs[slot] = v),
            Field::Remaining(v) => checked_word!(game.plm_instruction_timer[slot] = v),
            Field::DefaultDraw => checked_word!(
                game.plm_instruction_draw_ptr[slot] = addr_kDefaultPlmDrawInstruction as u16
            ),
            Field::IdleTimer => checked_word!(game.plm_timers[slot] = 0),
            Field::Selected => checked_word!(game.plm_id = (slot * 2) as u16),
        }
    }
    fn current_block(&mut self) -> u16 {
        self.game.cur_block_index().wrapping_mul(2)
    }
    fn width(&mut self) -> u8 {
        let game = &*self.game;
        checked_word!(game.room_width_in_blocks) as u8
    }
    fn block(index: u16) -> u16 {
        index
    }
    fn header(value: u16) -> u16 {
        value
    }
    fn before(kind: SourceKind) -> u16 {
        match kind {
            SourceKind::Collision => 0x853d,
            SourceKind::Hardcoded => 0x8469,
            SourceKind::Room => 0x84e6,
        }
    }
    fn entry(&mut self, address: u16) -> Entry {
        if address == 0x12 {
            Entry::Scratch
        } else {
            Entry::Record(unsafe { crate::rtl::rom_bank_record::<0x8f>(address, self.game) })
        }
    }
    fn byte(&mut self, entry: &Entry, offset: usize) -> u8 {
        match entry {
            Entry::Scratch => self.game.g_ram[0x12 + offset],
            Entry::Record(record) => record.byte(offset),
        }
    }
    fn definition(&mut self, header: u16, kind: SourceKind) -> Definition {
        if kind == SourceKind::Collision {
            Definition::Collision(0x840000 | u32::from(header))
        } else {
            Definition::Placed(
                unsafe { crate::rtl::rom_bank_record::<0x84>(header, self.game) },
                kind,
            )
        }
    }
    fn program(&mut self, definition: &Definition) -> u16 {
        match definition {
            Definition::Collision(address) => self.game.read_rom_word(*address, 2),
            Definition::Placed(record, SourceKind::Room) => u16::from_le(record.word(2)),
            Definition::Placed(record, _) => record.word(2),
        }
    }
    fn setup(&mut self, definition: &Definition) -> u16 {
        match definition {
            Definition::Collision(address) => self.game.read_rom_word(*address, 0),
            Definition::Placed(record, SourceKind::Room) => u16::from_le(record.word(0)),
            Definition::Placed(record, _) => record.word(0),
        }
    }
    fn execute(&mut self, slot: usize, setup: u16, kind: SourceKind) -> u8 {
        let accumulator = if kind == SourceKind::Collision {
            0
        } else {
            addr_kDefaultPlmDrawInstruction as u16
        };
        unsafe {
            CallPlmHeaderFunc(
                0x840000 | u32::from(setup),
                (slot * 2) as u16,
                self.displacement,
                accumulator,
                self.game,
            )
        }
    }
}

/// # Safety
/// Setup requires initialized runtime state. A non-null displacement must remain
/// live and writable throughout setup and its nested mechanism creations.
pub unsafe fn create(source: Source<u16, u16, u16>, displacement: *mut i32, game: &mut Game) -> u8 {
    mechanism_creation::create(&mut Adapter { game, displacement }, source)
}
