//! Content capture and exceptional imported aliases for the shared Session.
#![forbid(unsafe_code)]
use super::host::*;
use crate::game::session_runtime::{Runtime, invalid_window};
use crate::ram_map::word_arrays as A;
use crate::rom::RomImage;
impl Host for Runtime<'_> {}
impl Content for Runtime<'_> {
    type Script = super::script::EprojScript<RomImage>;
    type Loaded = super::content::LoadedRecord;
    type Native = super::content::NativeRecord;
    fn script(&self, bank_address: u16) -> Self::Script {
        Self::Script::new(self.loaded_image().expect("ROM loaded"), bank_address)
    }
    fn loaded_record(&mut self, bank: u8, bank_address: u16) -> Self::Loaded {
        let address = (u32::from(bank) << 16) | u32::from(bank_address);
        if address & 0x8000 == 0 {
            self.foreign(|game| invalid_window(game, address));
        }
        Self::Loaded::new(
            self.loaded_image().expect("ROM loaded"),
            crate::rtl::lorom_offset(address) as usize,
        )
    }
    fn native_record(&mut self, bank: u8, bank_address: u16) -> Self::Native {
        let address = (u32::from(bank) << 16) | u32::from(bank_address);
        if address & 0x8000 == 0 {
            self.foreign(|game| invalid_window(game, address));
        }
        Self::Native::new(
            self.native_image(),
            crate::rtl::lorom_offset(address) as usize,
        )
    }
    fn screen_shake_word(&self, word_index: isize) -> u16 {
        self.loaded_image()
            .expect("ROM loaded")
            .table_word(crate::rtl::lorom_offset(0x86846b) as usize, word_index)
    }
    fn waffle_sine_sample(&mut self, phase: u8) -> u8 {
        self.foreign(|game| game.read_rom_byte(0xa0b143, usize::from(phase)))
    }
}
fn enemy_base(member: EnemyMember) -> usize {
    match member {
        EnemyMember::Definition => 0x1997,
        EnemyMember::Graphics => 0x19bb,
        EnemyMember::Counter => 0x19df,
        EnemyMember::PreAction => 0x1a03,
        EnemyMember::XFraction => 0x1a27,
        EnemyMember::X => 0x1a4b,
        EnemyMember::YFraction => 0x1a6f,
        EnemyMember::Y => 0x1a93,
        EnemyMember::VelocityX => 0x1ab7,
        EnemyMember::VelocityY => 0x1adb,
        EnemyMember::BehaviorA => 0x1aff,
        EnemyMember::BehaviorB => 0x1b23,
        EnemyMember::Program => 0x1b47,
        EnemyMember::Spritemap => 0x1b6b,
        EnemyMember::Delay => 0x1b8f,
        EnemyMember::Radius => 0x1bb3,
        EnemyMember::Properties => 0x1bd7,
        EnemyMember::BehaviorC => 0x1bfb,
        EnemyMember::Angle => 0x97dc,
        EnemyMember::DropFlags => 0xf380,
        EnemyMember::DropDefinition => 0xf3c8,
        EnemyMember::KilledEnemy => 0xf410,
    }
}
fn indexed(base: usize, index: isize) -> usize {
    base.checked_add_signed(index.checked_mul(2).unwrap())
        .unwrap()
}
impl ImportedAliases for Runtime<'_> {
    fn enemy_word(&self, member: EnemyMember, signed_slot: isize) -> u16 {
        self.word(indexed(enemy_base(member), signed_slot))
    }
    fn set_enemy_word(&mut self, member: EnemyMember, signed_slot: isize, value: u16) {
        self.write_slice_le(indexed(enemy_base(member), signed_slot), value);
    }
    fn enemy_byte(&self, member: EnemyMember, byte_index: isize) -> u8 {
        self.byte(enemy_base(member).checked_add_signed(byte_index).unwrap())
    }
    fn set_enemy_byte(&mut self, member: EnemyMember, byte_index: isize, value: u8) {
        self.set_byte(
            enemy_base(member).checked_add_signed(byte_index).unwrap(),
            value,
        );
    }
    fn shot_word(&self, member: ShotMember, signed_slot: isize) -> u16 {
        let base = match member {
            ShotMember::Kind => A::projectile_type,
            ShotMember::Damage => A::projectile_damage,
            ShotMember::Direction => A::projectile_dir,
            ShotMember::Variable => A::projectile_variables,
            ShotMember::X => A::projectile_x_pos,
            ShotMember::Y => A::projectile_y_pos,
            ShotMember::XRadius => A::projectile_x_radius,
            ShotMember::YRadius => A::projectile_y_radius,
        };
        self.word(indexed(base, signed_slot))
    }
    fn set_shot_direction(&mut self, signed_slot: isize, value: u16) {
        self.write_slice_le(indexed(A::projectile_dir, signed_slot), value);
    }
    fn mechanism_word(&self, member: MechanismMember, signed_slot: isize) -> u16 {
        let base = match member {
            MechanismMember::Block => A::plm_block_indices,
            MechanismMember::RoomArgument => A::plm_room_arguments,
        };
        self.word(indexed(base, signed_slot))
    }
    fn terrain_word(&self, block_index: u16) -> u16 {
        self.word(0x10002 + usize::from(block_index) * 2)
    }
    fn behavior_byte(&self, block_index: u16) -> u8 {
        self.byte(0x16402 + usize::from(block_index))
    }
}
