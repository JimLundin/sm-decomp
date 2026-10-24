//! Live checked fields for the owned contact scan; cartridge, bus and setup effects stay explicit.
use crate::game::mechanism_displacement::Displacement;
use crate::game::samus_room_contact::{self as contact, Axis, Context, Field};
use crate::game_state::Game;
struct Fields<'a>(&'a mut Game);
fn offset(field: Field) -> usize {
    use Field::*;
    match field {
        Width => 0x7a5,
        Direction => 0xb02,
        Block => 0xdc4,
        Frame => 0x5b6,
        Collision => 0xdd0,
        SlopeAdjusted => 0xdba,
        Quicksand => 0x1e71,
        SlopeBts => 0x1e77,
        YSpeedFraction => 0xb2c,
        YSpeed => 0xb2e,
        MultiplyLeft => 0x5e9,
        MultiplyRight => 0x5eb,
        Area => 0x79f,
        Invincibility => 0x18a8,
        KnockbackTimer => 0x18aa,
        Damage => 0xa50,
        Knockback => 0xa54,
        Facing => 0xa1e,
    }
}
fn word(game: &Game, at: usize) -> u16 {
    u16::from_le_bytes([game.g_ram[at], game.g_ram[at + 1]])
}
fn put(game: &mut Game, at: usize, value: u16) {
    game.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn axis_offset(axis: Axis) -> usize {
    match axis {
        Axis::X => 0xaf6,
        Axis::Y => 0xafa,
    }
}
impl Context for Fields<'_> {
    type Header = u16;
    fn read(&self, field: Field) -> u16 {
        if field == Field::Facing {
            u16::from(self.0.g_ram[offset(field)])
        } else {
            word(self.0, offset(field))
        }
    }
    fn write(&mut self, field: Field, value: u16) {
        put(self.0, offset(field), value);
    }
    fn position(&self, axis: Axis) -> u16 {
        word(self.0, axis_offset(axis))
    }
    fn fraction(&self, axis: Axis) -> u16 {
        word(self.0, axis_offset(axis) + 2)
    }
    fn radius(&self, axis: Axis) -> u16 {
        word(self.0, if axis == Axis::X { 0xafe } else { 0xb00 })
    }
    fn set_fraction(&mut self, axis: Axis, value: u16) {
        put(self.0, axis_offset(axis) + 2, value);
    }
    fn add_position(&mut self, axis: Axis, amount: i32) {
        let at = axis_offset(axis);
        let integer = word(self.0, at);
        let fraction = word(self.0, at + 2);
        let result = ((u32::from(integer) << 16) | u32::from(fraction)).wrapping_add(amount as u32);
        put(self.0, at + 2, result as u16);
        put(self.0, at, (result >> 16) as u16);
    }
    fn level(&self, block: u16) -> u16 {
        word(self.0, 0x10002 + usize::from(block) * 2)
    }
    fn bts(&self, block: u16) -> u8 {
        self.0.g_ram[0x16402 + usize::from(block)]
    }
    fn square_occupied(&self, index: u16) -> bool {
        crate::game::sm_94::kTab948E54[usize::from(index)] != 0
    }
    fn slope_factor(&self, index: u8) -> u16 {
        crate::game::sm_94::kBlockColl_Horiz_Slope_NonSquare_Tab[usize::from(index) * 2 + 1]
    }
    fn slope_height(&self, index: usize) -> u8 {
        crate::game::sm_94::kAlignYPos_Tab0[index]
    }
    fn vertical_slope_bts(&mut self, block: u16) -> u16 {
        let address = 0x7f6402 + u32::from(block);
        unsafe {
            let low = crate::snes::snes::snes_read(self.0.g_snes, address, self.0);
            let high = crate::snes::snes::snes_read(self.0.g_snes, address + 1, self.0);
            u16::from_le_bytes([low, high])
        }
    }
    fn boss_defeated(&mut self) -> bool {
        crate::game::gameplay_flag_runtime::CheckBossBitForCurArea(1, self.0) & 1 != 0
    }
    fn special_header(&mut self, bts: u8, solid: bool) -> u16 {
        if bts & 0x80 != 0 {
            let area = word(self.0, 0x79f);
            let table = self.0.read_rom_word(
                if solid { 0x9492e9 } else { 0x9492d9 },
                usize::from(area) * 2,
            );
            if table & 0x8000 == 0 {
                unsafe {
                    crate::rtl::rom_bank_ptr::<0x94>(table, self.0);
                }
            }
            self.0
                .read_rom_word(0x948000 | u32::from(table), usize::from(bts & 0x7f) * 2)
        } else {
            self.0.read_rom_word(0x949139, usize::from(bts) * 2)
        }
    }
    fn bomb_header(&mut self, bts: u8) -> u16 {
        self.0.read_rom_word(0x94936b, usize::from(bts) * 2)
    }
    fn setup(&mut self, header: u16, displacement: &mut i32) -> u8 {
        unsafe {
            crate::game::room_mechanisms::spawn_contact(
                header,
                &mut Displacement::Local(displacement),
                self.0,
            )
        }
    }
    fn try_enter_door(&mut self) -> bool {
        use crate::game::{
            room_entry::{self, ArrivalRequest, ContactSite, EntryPoll},
            session_runtime::Runtime,
        };
        !matches!(
            room_entry::begin(
                &mut Runtime::new(self.0),
                ArrivalRequest::Contact(ContactSite::CurrentCollision)
            ),
            EntryPoll::Rejected
        )
    }
}
/// # Safety
/// Native setup and physical bus effects require initialized runtime state.
pub unsafe fn move_right(amount: i32, game: &mut Game) -> i32 {
    contact::move_right(&mut Fields(game), amount)
}
/// # Safety
/// Native setup and physical bus effects require initialized runtime state.
pub unsafe fn move_down(amount: i32, game: &mut Game) -> i32 {
    contact::move_down(&mut Fields(game), amount)
}
/// # Safety
/// Native setup and physical bus effects require initialized runtime state.
pub unsafe fn wall_jump(amount: i32, game: &mut Game) -> i32 {
    contact::wall_jump(&mut Fields(game), amount)
}
/// # Safety
/// Native setup and physical bus effects require initialized runtime state.
pub unsafe fn changed_pose(amount: i32, game: &mut Game) -> i32 {
    contact::changed_pose(&mut Fields(game), amount)
}
