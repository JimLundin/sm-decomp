//! Byte-compatible projection only; gameplay behavior operates on owned state.
#![forbid(unsafe_code)]
use super::state::{EnemyProjectiles, Point, Projectile, Rectangle, SlotField};
use super::transport::Codec;
use crate::game_state::Game;

const COLUMNS: [usize; 22] = [
    0x1997, 0x19bb, 0x19df, 0x1a03, 0x1a27, 0x1a4b, 0x1a6f, 0x1a93, 0x1ab7, 0x1adb, 0x1aff, 0x1b23,
    0x1b47, 0x1b6b, 0x1b8f, 0x1bb3, 0x1bd7, 0x1bfb, 0x97dc, 0xf380, 0xf3c8, 0xf410,
];
const CONTROLS: [usize; 4] = [0x198d, 0x1991, 0x1993, 0x1995];

fn projectile_word(slot: &mut Projectile, column: usize) -> &mut u16 {
    match column {
        0 => &mut slot.definition,
        1 => &mut slot.graphics,
        2 => &mut slot.counter,
        3 => &mut slot.pre_action,
        4 => &mut slot.x.fraction,
        5 => &mut slot.x.whole,
        6 => &mut slot.y.fraction,
        7 => &mut slot.y.whole,
        8 => &mut slot.velocity.x,
        9 => &mut slot.velocity.y,
        10 => &mut slot.behavior_a,
        11 => &mut slot.behavior_b,
        12 => &mut slot.program,
        13 => &mut slot.spritemap,
        14 => &mut slot.delay,
        15 => &mut slot.radius,
        16 => &mut slot.properties,
        17 => &mut slot.behavior_c,
        18 => &mut slot.angle,
        19 => &mut slot.drop.flags,
        20 => &mut slot.drop.enemy_definition,
        21 => &mut slot.drop.killed_enemy,
        _ => unreachable!(),
    }
}
fn control_word(state: &mut EnemyProjectiles, control: usize) -> &mut u16 {
    match control {
        0 => &mut state.flags,
        1 => &mut state.selected,
        2 => &mut state.spawn.primary,
        3 => &mut state.spawn.secondary,
        _ => unreachable!(),
    }
}
fn location(address: usize) -> Option<(usize, usize, usize)> {
    for (column, base) in COLUMNS.into_iter().enumerate() {
        if (base..base + 36).contains(&address) {
            return Some((column, (address - base) / 2, (address - base) % 2));
        }
    }
    for (control, base) in CONTROLS.into_iter().enumerate() {
        if (base..base + 2).contains(&address) {
            return Some((22, control, address - base));
        }
    }
    None
}
impl Codec for EnemyProjectiles {
    fn import(game: &Game) -> Self {
        let mut state = Self::decode(&game.g_ram);
        state.spawn.rectangle = Rectangle {
            origin: Point {
                x: game.eproj_spawn_rect.x,
                y: game.eproj_spawn_rect.y,
            },
            extent: Point {
                x: game.eproj_spawn_rect.w,
                y: game.eproj_spawn_rect.h,
            },
        };
        state.spawn.variant = game.eproj_spawn_r22;
        state.spawn.point = Point {
            x: game.eproj_spawn_pt.x,
            y: game.eproj_spawn_pt.y,
        };
        state.spawn.drop_definition = game.eproj_spawn_varE24;
        state
    }
    fn export(&self, game: &mut Game) {
        self.encode(&mut game.g_ram);
        game.eproj_spawn_rect.x = self.spawn.rectangle.origin.x;
        game.eproj_spawn_rect.y = self.spawn.rectangle.origin.y;
        game.eproj_spawn_rect.w = self.spawn.rectangle.extent.x;
        game.eproj_spawn_rect.h = self.spawn.rectangle.extent.y;
        game.eproj_spawn_r22 = self.spawn.variant;
        game.eproj_spawn_pt.x = self.spawn.point.x;
        game.eproj_spawn_pt.y = self.spawn.point.y;
        game.eproj_spawn_varE24 = self.spawn.drop_definition;
    }
    fn decode(ram: &[u8]) -> Self {
        let mut state = Self::default();
        for (column, base) in COLUMNS.into_iter().enumerate() {
            for (index, slot) in state.slots.iter_mut().enumerate() {
                let at = base + index * 2;
                *projectile_word(slot, column) = u16::from_le_bytes([ram[at], ram[at + 1]]);
            }
        }
        for (control, at) in CONTROLS.into_iter().enumerate() {
            *control_word(&mut state, control) = u16::from_le_bytes([ram[at], ram[at + 1]]);
        }
        state.spawn.angle = ram[0x60b];
        state
    }
    fn encode(&self, ram: &mut [u8]) {
        for (column, base) in COLUMNS.into_iter().enumerate() {
            for (index, slot) in self.slots.iter().enumerate() {
                let mut value = *slot;
                let at = base + index * 2;
                ram[at..at + 2].copy_from_slice(&projectile_word(&mut value, column).to_le_bytes());
            }
        }
        for (at, value) in CONTROLS.into_iter().zip([
            self.flags,
            self.selected,
            self.spawn.primary,
            self.spawn.secondary,
        ]) {
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        ram[0x60b] = self.spawn.angle;
    }
    fn byte(&self, address: usize) -> Option<u8> {
        if address == 0x60b {
            return Some(self.spawn.angle);
        }
        let (column, index, lane) = location(address)?;
        let value = if column == 22 {
            [
                self.flags,
                self.selected,
                self.spawn.primary,
                self.spawn.secondary,
            ][index]
        } else {
            let mut slot = self.slots[index];
            *projectile_word(&mut slot, column)
        };
        Some(value.to_le_bytes()[lane])
    }
    fn put_byte(&mut self, address: usize, value: u8) -> bool {
        if address == 0x60b {
            self.spawn.angle = value;
            return true;
        }
        let Some((column, index, lane)) = location(address) else {
            return false;
        };
        let word = if column == 22 {
            control_word(self, index)
        } else {
            projectile_word(&mut self.slots[index], column)
        };
        let mut bytes = word.to_le_bytes();
        bytes[lane] = value;
        *word = u16::from_le_bytes(bytes);
        true
    }
}

/// Exclusive execution lease. Foreign calls destroy the inner owner before
/// exposing Game, including its native SpawnContext, and import only on return.
pub(super) struct Execution<'g> {
    pub(super) owner: Option<super::transport::Port<'g, EnemyProjectiles>>,
}
impl<'g> Execution<'g> {
    pub(super) fn acquire(game: &'g mut Game) -> Self {
        Self {
            owner: Some(super::transport::Port::acquire(game)),
        }
    }
    pub(super) fn state(&self) -> &EnemyProjectiles {
        &self.owner.as_ref().unwrap().state
    }
    pub(super) fn state_mut(&mut self) -> &mut EnemyProjectiles {
        &mut self.owner.as_mut().unwrap().state
    }
    pub(super) fn foreign<R>(&mut self, call: impl FnOnce(&mut Game) -> R) -> R {
        let owner = self.owner.take().unwrap();
        let (owner, result) = owner.foreign(call);
        self.owner = Some(owner);
        result
    }
    pub(super) fn image(&self) -> crate::rom::RomImage {
        self.owner.as_ref().unwrap().rom()
    }
    pub(super) fn record(&mut self, bank: u8, cursor: u16) -> crate::rom::RomRecord {
        if cursor & 0x8000 == 0 {
            self.foreign(|game| super::legacy::record(bank, cursor, game))
        } else {
            crate::rom::RomRecord {
                image: self.image(),
                base: crate::rtl::lorom_offset((u32::from(bank) << 16) | u32::from(cursor))
                    as usize,
            }
        }
    }
    pub(super) fn slot(&self, field: SlotField, index: isize) -> u16 {
        if let Ok(index) = usize::try_from(index) {
            if let Some(slot) = self.state().slots.get(index) {
                let mut value = *slot;
                return *projectile_word(&mut value, field as usize);
            }
        }
        let address = COLUMNS[field as usize]
            .checked_add_signed(index.checked_mul(2).unwrap())
            .unwrap();
        self.owner.as_ref().unwrap().read_le(address)
    }
    pub(super) fn set_slot(&mut self, field: SlotField, index: isize, value: u16) {
        if let Ok(index) = usize::try_from(index) {
            if let Some(slot) = self.state_mut().slots.get_mut(index) {
                *projectile_word(slot, field as usize) = value;
                return;
            }
        }
        let address = COLUMNS[field as usize]
            .checked_add_signed(index.checked_mul(2).unwrap())
            .unwrap();
        self.owner.as_mut().unwrap().write_slice_le(address, value);
    }
    pub(super) fn slot_byte(&self, field: SlotField, byte_index: isize) -> u8 {
        let address = COLUMNS[field as usize]
            .checked_add_signed(byte_index)
            .unwrap();
        self.owner.as_ref().unwrap().byte(address)
    }
    pub(super) fn set_slot_byte(&mut self, field: SlotField, byte_index: isize, value: u8) {
        let address = COLUMNS[field as usize]
            .checked_add_signed(byte_index)
            .unwrap();
        self.owner.as_mut().unwrap().set_byte(address, value);
    }
    pub(super) fn decrement(&mut self, field: SlotField, index: isize) -> u16 {
        let old = self.slot(field, index);
        self.set_slot(field, index, old.wrapping_sub(1));
        old
    }
}
