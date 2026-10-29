//! Direct ordinary aggregate access with finite imported-selector fallbacks.
#![forbid(unsafe_code)]
use super::host::{ByteOrder, Host};
#[macro_export]
macro_rules! mechanism_member {
    (header) => {
        $crate::room_mechanisms::host::SlotMember::Header
    };
    (block) => {
        $crate::room_mechanisms::host::SlotMember::Block
    };
    (pre_action) => {
        $crate::room_mechanisms::host::SlotMember::PreAction
    };
    (program) => {
        $crate::room_mechanisms::host::SlotMember::Program
    };
    (counter) => {
        $crate::room_mechanisms::host::SlotMember::Counter
    };
    (room_argument) => {
        $crate::room_mechanisms::host::SlotMember::RoomArgument
    };
    (work0) => {
        $crate::room_mechanisms::host::SlotMember::Work0
    };
    (instruction_delay) => {
        $crate::room_mechanisms::host::SlotMember::InstructionDelay
    };
    (draw_program) => {
        $crate::room_mechanisms::host::SlotMember::DrawProgram
    };
    (return_program) => {
        $crate::room_mechanisms::host::SlotMember::ReturnProgram
    };
    (work1) => {
        $crate::room_mechanisms::host::SlotMember::Work1
    };
}
#[macro_export]
macro_rules! mechanism {
    ($h:ident,$index:expr,$field:ident) => {{
        let index = isize::try_from($index).expect("mechanism slot index");
        if let Some(slot) = usize::try_from(index)
            .ok()
            .and_then(|i| $h.session().mechanisms.slots.get(i))
        {
            slot.$field
        } else {
            $h.mechanism_word(
                $crate::room_mechanisms::access::mechanism_member!($field),
                index,
            )
        }
    }};
    ($h:ident,$index:expr,$field:ident = $value:expr) => {
        $crate::room_mechanisms::access::mechanism!(
            $h,
            $index,
            $field = $value,
            $crate::room_mechanisms::host::StoreOrder::SliceLittle
        )
    };
    ($h:ident,$index:expr,$field:ident = $value:expr,$order:expr) => {{
        let value: u16 = $value;
        let index = isize::try_from($index).expect("mechanism slot index");
        if let Some(slot) = usize::try_from(index)
            .ok()
            .and_then(|i| $h.session_mut().mechanisms.slots.get_mut(i))
        {
            slot.$field = match $order {
                $crate::room_mechanisms::host::StoreOrder::SliceNative => {
                    u16::from_le_bytes(value.to_ne_bytes())
                }
                _ => value,
            };
        } else {
            $h.set_mechanism_word(
                $crate::room_mechanisms::access::mechanism_member!($field),
                index,
                value,
                $order,
            );
        }
    }};
    ($h:ident,$index:expr,$field:ident += $value:expr) => {{
        let index = isize::try_from($index).expect("mechanism slot index");
        let old = $crate::room_mechanisms::access::mechanism!($h, index, $field);
        let value: u16 = old.wrapping_add($value);
        $crate::room_mechanisms::access::mechanism!($h, index, $field = value);
    }};
    ($h:ident,$index:expr,$field:ident -= $value:expr) => {{
        let index = isize::try_from($index).expect("mechanism slot index");
        let old = $crate::room_mechanisms::access::mechanism!($h, index, $field);
        let value: u16 = old.wrapping_sub($value);
        $crate::room_mechanisms::access::mechanism!($h, index, $field = value);
    }};
    ($h:ident,$index:expr,$field:ident |= $value:expr) => {{
        let index = isize::try_from($index).expect("mechanism slot index");
        let old = $crate::room_mechanisms::access::mechanism!($h, index, $field);
        let value: u16 = old | $value;
        $crate::room_mechanisms::access::mechanism!($h, index, $field = value);
    }};
    ($h:ident,$index:expr,$field:ident &= $value:expr) => {{
        let index = isize::try_from($index).expect("mechanism slot index");
        let old = $crate::room_mechanisms::access::mechanism!($h, index, $field);
        let value: u16 = old & $value;
        $crate::room_mechanisms::access::mechanism!($h, index, $field = value);
    }};
    ($h:ident,$index:expr,$field:ident ^= $value:expr) => {{
        let index = isize::try_from($index).expect("mechanism slot index");
        let old = $crate::room_mechanisms::access::mechanism!($h, index, $field);
        let value: u16 = old ^ $value;
        $crate::room_mechanisms::access::mechanism!($h, index, $field = value);
    }};
}
#[macro_export]
macro_rules! mechanism_byte {
    ($h:ident,$index:expr,$field:ident,$lane:expr) => {{
        let index = isize::try_from($index).expect("mechanism slot index");
        let lane: usize = $lane;
        if let Some(slot) = usize::try_from(index)
            .ok()
            .and_then(|i| $h.session().mechanisms.slots.get(i))
            .filter(|_| lane < 2)
        {
            slot.$field.to_le_bytes()[lane]
        } else {
            $h.mechanism_byte(
                $crate::room_mechanisms::access::mechanism_member!($field),
                index,
                lane,
            )
        }
    }};
    ($h:ident,$index:expr,$field:ident,$lane:expr => $value:expr) => {{
        let value: u8 = $value;
        let index = isize::try_from($index).expect("mechanism slot index");
        let lane: usize = $lane;
        if let Some(slot) = usize::try_from(index)
            .ok()
            .and_then(|i| $h.session_mut().mechanisms.slots.get_mut(i))
            .filter(|_| lane < 2)
        {
            let mut bytes = slot.$field.to_le_bytes();
            bytes[lane] = value;
            slot.$field = u16::from_le_bytes(bytes);
        } else {
            $h.set_mechanism_byte(
                $crate::room_mechanisms::access::mechanism_member!($field),
                index,
                lane,
                value,
            );
        }
    }};
}
pub use crate::{mechanism, mechanism_byte, mechanism_member};

pub fn terrain_word<H: Host>(h: &H, byte_block: usize, order: ByteOrder) -> u16 {
    if byte_block & 1 == 0 {
        if let Some(cell) = h.session().room.terrain.cells().get(byte_block / 2) {
            return match order {
                ByteOrder::Little => cell.level,
                ByteOrder::Native => u16::from_ne_bytes(cell.level.to_le_bytes()),
            };
        }
    }
    h.terrain_word(byte_block, order)
}
pub fn set_terrain_word<H: Host>(h: &mut H, byte_block: usize, value: u16, order: ByteOrder) {
    if byte_block & 1 == 0 {
        if let Some(cell) = h
            .session_mut()
            .room
            .terrain
            .cells_mut()
            .get_mut(byte_block / 2)
        {
            cell.level = match order {
                ByteOrder::Little => value,
                ByteOrder::Native => u16::from_le_bytes(value.to_ne_bytes()),
            };
            return;
        }
    }
    h.set_terrain_word(byte_block, value, order);
}
pub fn terrain_high<H: Host>(h: &H, byte_block: u16) -> u8 {
    if byte_block & 1 == 0 {
        if let Some(cell) = h
            .session()
            .room
            .terrain
            .cells()
            .get(usize::from(byte_block / 2))
        {
            return (cell.level >> 8) as u8;
        }
    }
    h.terrain_high(byte_block)
}
pub fn set_terrain_high<H: Host>(h: &mut H, byte_block: u16, value: u8) {
    if byte_block & 1 == 0 {
        if let Some(cell) = h
            .session_mut()
            .room
            .terrain
            .cells_mut()
            .get_mut(usize::from(byte_block / 2))
        {
            cell.level = (cell.level & 0xff) | (u16::from(value) << 8);
            return;
        }
    }
    h.set_terrain_high(byte_block, value);
}
pub fn behavior_byte<H: Host>(h: &H, index: usize) -> u8 {
    if let Some(cell) = h.session().room.terrain.cells().get(index) {
        cell.behavior
    } else {
        h.behavior_byte(index)
    }
}
pub fn set_behavior_byte<H: Host>(h: &mut H, index: usize, value: u8) {
    if let Some(cell) = h.session_mut().room.terrain.cells_mut().get_mut(index) {
        cell.behavior = value;
    } else {
        h.set_behavior_byte(index, value);
    }
}
pub fn set_behavior_pair<H: Host>(h: &mut H, index: usize, value: u16) {
    if index
        .checked_add(1)
        .is_some_and(|end| end < h.session().room.terrain.cells().len())
    {
        let bytes = value.to_ne_bytes();
        h.session_mut().room.terrain.cells_mut()[index].behavior = bytes[0];
        h.session_mut().room.terrain.cells_mut()[index + 1].behavior = bytes[1];
    } else {
        h.set_behavior_pair(index, value);
    }
}
pub fn projectile_kind<H: Host>(h: &H, index: isize, order: ByteOrder) -> u16 {
    if let Some(shot) = usize::try_from(index)
        .ok()
        .and_then(|i| h.session().projectiles.shots.get(i))
    {
        shot.kind
    } else {
        h.projectile_kind(index, order)
    }
}
