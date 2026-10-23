//! PLM slot allocation and ordered initialization for each source of room objects.
//! RAM stays authoritative; setup retains the caller's collision displacement.
use crate::game::mechanism_displacement::Displacement;
use crate::game::constants::bank_8f::addr_kDoorClosingPlmIds;
use crate::game::{mechanism_creation::Source, mechanism_creation_ram};
use crate::game::progress_flags::PrepareBitAccess;
use crate::game_state::Game;
use crate::ram_map::checked_word;
use crate::rom::RomRecord;
use crate::types::{DoorDef, SpawnHardcodedPlmArgs};
use core::mem::offset_of;

const header_record: unsafe fn(u16, &mut Game) -> RomRecord = crate::rtl::rom_bank_record::<0x84>;

pub unsafe fn SpawnHardcodedPlm(args: SpawnHardcodedPlmArgs, game: &mut Game) {
    unsafe { spawn_hardcoded_plm(args, &mut Displacement::Absent, game) }
}

// Nested setup spawns retain the calling collision's displacement.
pub(crate) unsafe fn spawn_hardcoded_plm(
    args: SpawnHardcodedPlmArgs,
    displacement: &mut Displacement<'_>,
    game: &mut Game,
) {
    unsafe {
        mechanism_creation_ram::create(
            Source::Hardcoded {
                x: args.block_x,
                y: args.block_y,
                header: args.header,
            },
            displacement,
            game,
        );
    }
}

pub unsafe fn SpawnRoomPLM(address: u16, game: &mut Game) {
    unsafe {
        mechanism_creation_ram::create(Source::Room { address }, &mut Displacement::Absent, game);
    }
}

/// Prepare a PLM at the current collision block and invoke its legacy setup.
pub unsafe fn SpawnPLM(header: u16, game: &mut Game) -> u8 {
    unsafe { SpawnPLMWithCollision(header, &mut Displacement::Absent, game) }
}

/// # Safety
/// Setup requires initialized runtime state. The active contact remains borrowed through setup and nested PLM spawns.
pub unsafe fn SpawnPLMWithCollision(
    header: u16,
    displacement: &mut Displacement<'_>,
    game: &mut Game,
) -> u8 {
    unsafe { mechanism_creation_ram::create(Source::Collision { header }, displacement, game) }
}

#[forbid(unsafe_code)]
fn door_cap_to_close(block: u16, game: &mut Game) -> Option<(usize, u16)> {
    // Stop at the first matching block, even when its slot is empty.
    let slot = (0..40)
        .rev()
        .find(|&slot| checked_word!(game.plm_block_indices[slot]) == block)?;
    if checked_word!(game.plm_header_ptr[slot]) == 0 {
        return None;
    }
    let argument = checked_word!(game.plm_room_arguments[slot]);
    if argument & 0x8000 == 0 {
        let index = usize::from(PrepareBitAccess(argument, game));
        if checked_word!(game.bitmask) & u16::from(game.g_ram[0xd8b0 + index]) != 0 {
            return None;
        }
    }
    checked_word!(game.plm_instruction_timer[slot] = 1);
    let header = checked_word!(game.plm_header_ptr[slot]);
    (header != 0).then_some((slot, header))
}

/// Restart an existing colored cap, or spawn the direction's closing object.
pub(crate) unsafe fn close_door(game: &mut Game) {
    unsafe {
        let door = crate::rtl::rom_bank_record::<0x83>(checked_word!(game.door_def_ptr), game);
        let row = u16::from(door.byte(offset_of!(DoorDef, y_pos_plm)))
            * u16::from(checked_word!(game.room_width_in_blocks) as u8);
        let block = row
            .wrapping_add(u16::from(door.byte(offset_of!(DoorDef, x_pos_plm))))
            .wrapping_mul(2);
        if let Some((slot, header)) = door_cap_to_close(block, game) {
            checked_word!(game.plm_instr_list_ptrs[slot] = header_record(header, game).word(4));
            return;
        }
        let address = checked_word!(game.door_direction)
            .wrapping_mul(2)
            .wrapping_add(addr_kDoorClosingPlmIds as u16);
        let header = crate::rtl::rom_bank_record::<0x8f>(address, game);
        if header.word(0) != 0 {
            game.g_ram[0x12..0x14].copy_from_slice(&header.word(0).to_ne_bytes());
            let door = crate::rtl::rom_bank_record::<0x83>(checked_word!(game.door_def_ptr), game);
            game.g_ram[0x14..0x16]
                .copy_from_slice(&door.word(offset_of!(DoorDef, x_pos_plm)).to_ne_bytes());
            game.g_ram[0x16..0x18].copy_from_slice(&0u16.to_ne_bytes());
            SpawnRoomPLM(0x12, game);
        }
    }
}
