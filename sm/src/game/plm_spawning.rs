//! PLM slot allocation and ordered initialization for each source of room objects.
//! RAM stays authoritative; setup retains the caller's collision displacement.
use crate::game::constants::bank_84::addr_kDefaultPlmDrawInstruction;
use crate::game::sm_84::CallPlmHeaderFunc;
use crate::game_state::Game;
use crate::ram_map::checked_word;
use crate::rom::RomRecord;
use crate::types::SpawnHardcodedPlmArgs;

const header_record: unsafe fn(u16, &mut Game) -> RomRecord = crate::rtl::rom_bank_record::<0x84>;

#[forbid(unsafe_code)]
fn free_slot(game: &Game) -> Option<usize> {
    (0..40)
        .rev()
        .find(|&slot| checked_word!(game.plm_header_ptr[slot]) == 0)
}

#[forbid(unsafe_code)]
fn arm(slot: usize, instruction: u16, game: &mut Game) {
    checked_word!(game.plm_instr_list_ptrs[slot] = instruction);
    checked_word!(game.plm_instruction_timer[slot] = 1);
    checked_word!(game.plm_instruction_draw_ptr[slot] = addr_kDefaultPlmDrawInstruction as u16);
    checked_word!(game.plm_timers[slot] = 0);
}

enum RoomEntry {
    Scratch,
    Rom(RomRecord),
}
#[forbid(unsafe_code)]
impl RoomEntry {
    fn byte(&self, offset: usize, game: &Game) -> u8 {
        match self {
            Self::Scratch => game.g_ram[0x12 + offset],
            Self::Rom(record) => record.byte(offset),
        }
    }
    fn word(&self, offset: usize, game: &Game) -> u16 {
        u16::from_le_bytes([self.byte(offset, game), self.byte(offset + 1, game)])
    }
}

pub unsafe fn SpawnHardcodedPlm(args: SpawnHardcodedPlmArgs, game: &mut Game) {
    spawn_hardcoded_plm(args, core::ptr::null_mut(), game);
}

// Nested setup spawns retain the calling collision's displacement.
pub(crate) unsafe fn spawn_hardcoded_plm(
    args: SpawnHardcodedPlmArgs,
    displacement: *mut i32,
    game: &mut Game,
) {
    let Some(slot) = free_slot(game) else { return };
    let row = u16::from(args.block_y) * u16::from(checked_word!(game.room_width_in_blocks) as u8);
    checked_word!(
        game.plm_block_indices[slot] = row.wrapping_add(u16::from(args.block_x)).wrapping_mul(2)
    );
    checked_word!(game.plm_header_ptr[slot] = args.header);
    checked_word!(game.plm_room_arguments[slot] = 0);
    checked_word!(game.plm_variables[slot] = 0);
    checked_word!(game.plm_pre_instrs[slot] = 0x8469);
    let header = header_record(args.header, game);
    arm(slot, header.word(2), game);
    checked_word!(game.plm_id = (slot * 2) as u16);
    CallPlmHeaderFunc(
        0x840000 | u32::from(header.word(0)),
        (slot * 2) as u16,
        displacement,
        addr_kDefaultPlmDrawInstruction as u16,
        game,
    );
}

pub unsafe fn SpawnRoomPLM(address: u16, game: &mut Game) {
    let Some(slot) = free_slot(game) else { return };
    let entry = if address == 0x12 {
        RoomEntry::Scratch
    } else {
        RoomEntry::Rom(crate::rtl::rom_bank_record::<0x8f>(address, game))
    };
    let y = u16::from(entry.byte(3, game));
    let row = y * u16::from(checked_word!(game.room_width_in_blocks) as u8);
    let x = u16::from(entry.byte(2, game));
    checked_word!(game.plm_block_indices[slot] = row.wrapping_add(x).wrapping_mul(2));
    checked_word!(game.plm_room_arguments[slot] = entry.word(4, game));
    let header_address = entry.word(0, game);
    checked_word!(game.plm_header_ptr[slot] = entry.word(0, game));
    checked_word!(game.plm_variables[slot] = 0);
    checked_word!(game.plm_pre_instrs[slot] = 0x84e6);
    let header = header_record(header_address, game);
    // Room entries and their headers explicitly read little-endian words.
    arm(slot, u16::from_le(header.word(2)), game);
    checked_word!(game.plm_id = (slot * 2) as u16);
    CallPlmHeaderFunc(
        0x840000 | u32::from(u16::from_le(header.word(0))),
        (slot * 2) as u16,
        core::ptr::null_mut(),
        addr_kDefaultPlmDrawInstruction as u16,
        game,
    );
}

/// Prepare a PLM at the current collision block and invoke its legacy setup.
pub unsafe fn SpawnPLM(header: u16, game: &mut Game) -> u8 {
    SpawnPLMWithCollision(header, core::ptr::null_mut(), game)
}

/// # Safety
/// Setup requires initialized runtime state. A non-null displacement must stay
/// live and writable through setup and any nested PLM spawns.
pub unsafe fn SpawnPLMWithCollision(header: u16, displacement: *mut i32, game: &mut Game) -> u8 {
    let Some(slot) = free_slot(game) else {
        return 0;
    };
    checked_word!(game.plm_block_indices[slot] = game.cur_block_index().wrapping_mul(2));
    checked_word!(game.plm_header_ptr[slot] = header);
    checked_word!(game.plm_pre_instrs[slot] = 0x853d);
    // Collision spawning uses checked window reads and reacquires ROM for setup.
    let address = 0x840000 | u32::from(header);
    arm(slot, game.read_rom_word(address, 2), game);
    checked_word!(game.plm_room_arguments[slot] = 0);
    checked_word!(game.plm_variables[slot] = 0);
    checked_word!(game.plm_id = (slot * 2) as u16);
    CallPlmHeaderFunc(
        0x840000 | u32::from(game.read_rom_word(address, 0)),
        (slot * 2) as u16,
        displacement,
        0,
        game,
    )
}
