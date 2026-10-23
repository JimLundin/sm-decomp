//! Shared semantic state access used by all room mechanism rules.
#![forbid(unsafe_code)]
use super::{rules::contact_physics::Field as ContactField, state::Column};
use crate::game::mechanism_displacement::Displacement;

#[derive(Clone, Copy)]
pub(super) enum Global {
    Contact(ContactField),
    Facing,
    Selected,
    BlockX,
    BlockY,
    EquippedItems,
    Health,
    MaxHealth,
    Missiles,
    MaxMissiles,
    SaveLocked,
    CurrentBlock,

    Area,
    NewKeys,
    LiquidOptions,
    QuakeType,
    ConveyorGateSpeed,
    ExternalXFraction,
    ExternalX,
    MovementSpeedTable,

    CollectedItems,
    PowerBombActive,
    LiquidTarget,
    LiquidHeight,
    LiquidVelocity,
    LiquidDelay,
    QuakeTimer,
    SoundDisabled,
    EnemyDeaths,
    EnemyQuota,

    RoomWidth,
    ProjectileSelection,
    PlayerX,
    PlayerY,
    PlayerRadius,
    ContactProtection,
    PeriodicDamage,
    LoadedItems,
    FrozenTime,
    SpeedBoost,
    Pose,
    CollisionDirection,
}
#[derive(Clone, Copy)]
pub(super) enum SoundChannel {
    One,
    Two,
    Three,
}
#[derive(Clone, Copy)]
pub(super) enum ContentTable {
    Gate { up: bool, offset: usize },
    MetroidWatcher,
}
#[derive(Clone, Copy)]
pub(super) enum WordOrder {
    Little,
    Native,
}
/// The interpreter has one field interface, including the two terrain views
/// that deliberately differ for odd block identities. It never exposes RAM.
pub(super) trait Fields {
    fn slot(&self, column: Column, slot: usize) -> u16;
    fn set_slot(&mut self, column: Column, slot: usize, value: u16);
    fn slot_byte(&self, column: Column, slot: usize, lane: usize) -> u8;
    fn global(&self, field: Global) -> u16;
    fn set_global(&mut self, field: Global, value: u16);
    fn level(&self, block: u16) -> u16;
    fn set_level(&mut self, block: u16, value: u16);
    fn level_high(&self, block: u16) -> u8;
    fn set_level_high(&mut self, block: u16, value: u8);
    fn set_behavior(&mut self, block: u16, value: u8);
    fn projectile_word(&self, index: isize) -> u16;
    fn native_level(&self, block: u16) -> u16;
    fn projectile_little_word(&self, index: isize) -> u16;
    fn behavior(&self, block: u16, lane: usize) -> u8;
    fn set_behavior_word(&mut self, block: u16, value: u16);
    fn set_signed_slot(&mut self, column: Column, slot: isize, value: u16);
    fn missing_scroll(&mut self) -> !;
    fn advance(&mut self, slot: usize, bytes: u16) {
        self.set_slot(
            Column::Program,
            slot,
            self.slot(Column::Program, slot).wrapping_add(bytes),
        );
    }
    fn clear_base_fraction_sign(&mut self);
    fn displacement(&self, value: &Displacement<'_>) -> i32;
    fn set_displacement(&mut self, target: &mut Displacement<'_>, value: i32);
    fn missing_displacement(&mut self);
    fn event(&mut self, event: u16) -> bool;
    fn mark_event(&mut self, event: u16);
    fn boss(&mut self, mask: u16) -> u16;
    fn sound(&mut self, channel: SoundChannel, maximum: u8, sound: u16);
    fn music(&mut self, track: u16);
    fn native_scroll_pair(&mut self, byte_index: usize, value: u16);
    fn content_word(&self, table: ContentTable, index: isize) -> u16;
    fn eproj(&mut self, definition: u16, parameter: u16);
    fn player_control(&mut self, operation: u16);
    fn door_opened(&mut self, door: u16) -> bool;
    fn set_wide_level(&mut self, block: usize, value: u16, order: WordOrder);
    fn set_wide_behavior(&mut self, block: usize, value: u8);
    fn behavior_at(&self, index: usize, lane: usize) -> u8;
    fn set_behavior_at(&mut self, index: usize, value: u16);
    fn global_byte(&self, field: Global, lane: usize) -> u8;
    fn map_used(&self) -> bool;
    fn coordinates(&mut self, slot: usize) {
        let block = self.slot(Column::Block, slot) >> 1;
        let width = self.global(Global::RoomWidth) as u8;
        self.set_global(
            Global::BlockY,
            if width == 0 {
                0xffff
            } else {
                block / u16::from(width)
            },
        );
        let width = self.global(Global::RoomWidth) as u8;
        self.set_global(
            Global::BlockX,
            if width == 0 {
                block
            } else {
                block % u16::from(width)
            },
        );
    }
    fn statue_signal(&mut self);
    fn spawn(&mut self, source: super::creation::Source, displacement: &mut Displacement<'_>)
    -> u8;
    fn below(&self, block: u16) -> u16 {
        block.wrapping_add(self.global(Global::RoomWidth).wrapping_mul(2))
    }
    fn release(&mut self, slot: usize) {
        self.set_slot(Column::Header, slot, 0);
    }
    fn install(&mut self, block: u16, value: u16) {
        let high = self.level_high(block) & 15 | (value >> 8) as u8;
        self.set_level_high(block, high);
        self.set_behavior(block, value as u8);
    }
}
