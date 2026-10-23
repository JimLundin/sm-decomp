//! Current encoding of the single semantic rule interface.
#![forbid(unsafe_code)]
use super::{
    fields::{ContentTable, Fields, Global, SoundChannel, WordOrder},
    port::Execution,
    state::Column,
};
use crate::game::mechanism_displacement::Displacement;
impl Global {
    fn address(self) -> usize {
        match self {
            Self::Facing => 0xa1e,
            Self::Selected => 0x1c27,
            Self::BlockX => 0x1c29,
            Self::BlockY => 0x1c2b,
            Self::EquippedItems => 0x9a2,
            Self::Health => 0x9c2,
            Self::MaxHealth => 0x9c4,
            Self::Missiles => 0x9c6,
            Self::MaxMissiles => 0x9c8,
            Self::SaveLocked => 0x1e75,
            Self::CurrentBlock => 0xdc4,

            Self::NewKeys => 0x8f,
            Self::LiquidOptions => 0x197e,
            Self::QuakeType => 0x183e,
            Self::ConveyorGateSpeed => 0xb2e,
            Self::ExternalXFraction => 0xb56,
            Self::ExternalX => 0xb58,
            Self::MovementSpeedTable => 0xa6c,

            Self::Area => 0x79f,
            Self::CollectedItems => 0x9a4,
            Self::PowerBombActive => 0x592,
            Self::LiquidTarget => 0x197a,
            Self::LiquidHeight => 0x1978,
            Self::LiquidVelocity => 0x197c,
            Self::LiquidDelay => 0x1980,
            Self::QuakeTimer => 0x1840,
            Self::SoundDisabled => 0x5f5,
            Self::EnemyDeaths => 0xe50,
            Self::EnemyQuota => 0xe52,

            Self::Contact(field) => {
                use super::rules::contact_physics::Field::*;
                match field {
                    YRadius => 0xb00,
                    YPosition => 0xafa,
                    Deceleration => 0xb4c,
                    Momentum => 0xb3c,
                    BoostCounter => 0xb3e,
                    EchoSound => 0xb40,
                    ExtraRunFraction => 0xb44,
                    ExtraRunSpeed => 0xb42,
                    BaseSpeed => 0xb46,
                    Equipment => 0x9a2,
                    ReactionPoint => 0x1e73,
                    YDirection => 0xb36,
                    YSpeedFraction => 0xb2c,
                    YSpeed => 0xb2e,
                    YSpeedMiddle => 0xb2d,
                    ExternalYFraction => 0xb5a,
                    ExternalY => 0xb5c,
                    ExternalYMiddle => 0xb5b,
                    YAccelerationFraction => 0xb32,
                    YAcceleration => 0xb34,
                    AutoJump => 0xaf4,
                    CollisionDirection => 0xb02,
                    ContactDamage => 0xa6e,
                    SandVisits => 0x1e71,
                }
            }

            Self::RoomWidth => 0x7a5,
            Self::ProjectileSelection => 0xdde,
            Self::PlayerX => 0xaf6,
            Self::PlayerY => 0xafa,
            Self::PlayerRadius => 0xb00,
            Self::ContactProtection => 0x18a8,
            Self::PeriodicDamage => 0xa50,
            Self::LoadedItems => 0xd91a,
            Self::FrozenTime => 0xa78,
            Self::SpeedBoost => 0xb3e,
            Self::Pose => 0xa1c,
            Self::CollisionDirection => 0xb02,
        }
    }
}
impl Fields for Execution<'_> {
    fn slot(&self, column: Column, slot: usize) -> u16 {
        self.port().field(column, slot)
    }
    fn set_slot(&mut self, column: Column, slot: usize, value: u16) {
        self.port_mut().put_field(column, slot, value);
    }
    fn slot_byte(&self, column: Column, slot: usize, lane: usize) -> u8 {
        self.port().byte(column.address() + 2 * slot + lane)
    }
    fn global(&self, field: Global) -> u16 {
        self.port().read_le(field.address())
    }
    fn set_global(&mut self, field: Global, value: u16) {
        self.port_mut().write_slice_le(field.address(), value);
    }
    fn level(&self, block: u16) -> u16 {
        self.port().read_le(0x10002 + usize::from(block & !1))
    }
    fn set_level(&mut self, block: u16, value: u16) {
        self.port_mut()
            .write_slice_le(0x10002 + usize::from(block & !1), value);
    }
    fn level_high(&self, block: u16) -> u8 {
        self.port().byte(0x10003 + usize::from(block))
    }
    fn set_level_high(&mut self, block: u16, value: u8) {
        self.port_mut()
            .set_byte(0x10003 + usize::from(block), value);
    }
    fn set_behavior(&mut self, block: u16, value: u8) {
        self.port_mut()
            .set_byte(0x16402 + usize::from(block >> 1), value);
    }
    fn projectile_word(&self, index: isize) -> u16 {
        let at = 0xc18usize
            .checked_add_signed(2 * index)
            .expect("projectile word offset");
        self.port().read_native(at)
    }
    fn native_level(&self, block: u16) -> u16 {
        self.port().read_native(0x10002 + usize::from(block & !1))
    }
    fn projectile_little_word(&self, index: isize) -> u16 {
        let at = 0xc18usize
            .checked_add_signed(2 * index)
            .expect("projectile word offset");
        self.port().read_le(at)
    }
    fn behavior(&self, block: u16, lane: usize) -> u8 {
        self.port().byte(0x16402 + usize::from(block >> 1) + lane)
    }
    fn set_behavior_word(&mut self, block: u16, value: u16) {
        self.port_mut()
            .write_slice_native(0x16402 + usize::from(block >> 1), value);
    }
    fn set_signed_slot(&mut self, column: Column, slot: isize, value: u16) {
        let at = column
            .address()
            .checked_add_signed(slot * 2)
            .expect("mechanism signed slot");
        self.port_mut().write_slice_le(at, value);
    }
    fn missing_scroll(&mut self) -> ! {
        self.foreign(|_| loop {})
    }
    fn clear_base_fraction_sign(&mut self) {
        let value = self.port().byte(0xb49) & 0x7f;
        self.port_mut().set_byte(0xb49, value);
    }
    fn displacement(&self, value: &Displacement<'_>) -> i32 {
        match value {
            Displacement::Absent => panic!("missing contact displacement"),
            Displacement::Local(value) => **value,
            Displacement::RamOffset(at) => {
                i32::from_ne_bytes(core::array::from_fn(|i| self.port().byte(*at + i)))
            }
        }
    }
    fn set_displacement(&mut self, target: &mut Displacement<'_>, value: i32) {
        match target {
            Displacement::Absent => panic!("missing contact displacement"),
            Displacement::Local(target) => **target = value,
            Displacement::RamOffset(at) => self.port_mut().write_slice(*at, &value.to_ne_bytes()),
        }
    }
    fn missing_displacement(&mut self) {
        self.foreign(super::effects::missing_displacement);
    }
    fn event(&mut self, event: u16) -> bool {
        self.foreign(|g| crate::game::progress_flags::CheckEventHappened(event, g) != 0)
    }
    fn mark_event(&mut self, event: u16) {
        self.foreign(|g| crate::game::progress_flags::SetEventHappened(event, g));
    }
    fn boss(&mut self, mask: u16) -> u16 {
        self.foreign(|g| u16::from(crate::game::progress_flags::CheckBossBitForCurArea(mask, g)))
    }
    fn sound(&mut self, channel: SoundChannel, maximum: u8, sound: u16) {
        self.foreign(|g| super::effects::sound(channel, maximum, sound, g));
    }
    fn music(&mut self, track: u16) {
        self.foreign(|g| crate::game::music::QueueMusic_Delayed8(track, g));
    }
    fn native_scroll_pair(&mut self, byte_index: usize, value: u16) {
        self.port_mut()
            .write_slice_native(0xcd20 + byte_index, value);
    }
    fn content_word(&self, table: ContentTable, index: isize) -> u16 {
        let base = match table {
            ContentTable::MetroidWatcher => 0x25b28,
            ContentTable::Gate { up, offset } => {
                crate::rtl::lorom_offset(if up { 0x84c764 } else { 0x84c70a }) as usize + offset
            }
        };
        self.port().rom().table_word(base, index)
    }
    fn eproj(&mut self, definition: u16, parameter: u16) {
        self.foreign(|g| super::effects::eproj(definition, parameter, g));
    }
    fn player_control(&mut self, operation: u16) {
        self.foreign(|g| super::effects::player_control(operation, g));
    }
    fn door_opened(&mut self, door: u16) -> bool {
        self.foreign(|g| {
            let index = crate::game::progress_flags::PrepareBitAccess(door, g);
            u16::from_le_bytes([g.g_ram[0x5e7], g.g_ram[0x5e8]])
                & u16::from(g.g_ram[0xd8b0 + usize::from(index)])
                != 0
        })
    }
    fn set_wide_level(&mut self, block: usize, value: u16, order: WordOrder) {
        let bytes = match order {
            WordOrder::Little => value.to_le_bytes(),
            WordOrder::Native => value.to_ne_bytes(),
        };
        self.port_mut().write_slice(0x10002 + (block & !1), &bytes);
    }
    fn set_wide_behavior(&mut self, block: usize, value: u8) {
        self.port_mut().set_byte(0x16402 + (block >> 1), value);
    }
    fn behavior_at(&self, index: usize, lane: usize) -> u8 {
        self.port().byte(0x16402 + index + lane)
    }
    fn set_behavior_at(&mut self, index: usize, value: u16) {
        self.port_mut().write_slice_native(0x16402 + index, value);
    }
    fn global_byte(&self, field: Global, lane: usize) -> u8 {
        self.port().byte(field.address() + lane)
    }
    fn map_used(&self) -> bool {
        self.port()
            .byte(0xd908 + usize::from(self.global(Global::Area)))
            != 0
    }
    fn statue_signal(&mut self) {
        self.port_mut().write_slice_native(
            0xf78 + core::mem::offset_of!(crate::types::EnemyData, parameter_1),
            1,
        );
    }
    fn spawn(
        &mut self,
        source: super::creation::Source,
        displacement: &mut Displacement<'_>,
    ) -> u8 {
        Execution::spawn(self, source, displacement)
    }
}
