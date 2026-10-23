//! Complete animated-tile lifecycle and interpreter over ordinary state.
#![forbid(unsafe_code)]
use super::state::{AnimatedTiles, Field, Upload};
use Field::*;
pub(crate) trait Record {
    fn byte(&self, displacement: usize) -> u8;
    fn word(&self, displacement: usize) -> u16;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SoundChannel {
    One,
    Two,
    Three,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StatueProjectile {
    EyeGlow,
    Soul,
}
/// Effects are synchronous. A runtime host relinquishes its owner before
/// opaque execution and imports the returned state; an owned host composes directly.
pub(crate) trait Host {
    type Record: Record;
    fn tiles(&self) -> &AnimatedTiles;
    fn tiles_mut(&mut self) -> &mut AnimatedTiles;
    fn record(&mut self, address: u16) -> Self::Record;
    fn music(&mut self, track: u16);
    fn sound(&mut self, channel: SoundChannel, sound: u16);
    fn check_boss(&mut self, mask: u16) -> bool;
    fn set_boss(&mut self, mask: u16);
    fn check_event(&mut self, event: u16) -> bool;
    fn set_event(&mut self, event: u16);
    fn boss_at(&mut self, area: u8) -> u8;
    fn statue(&mut self) -> u16;
    fn set_statue(&mut self, value: u16);
    fn projectile(&mut self, kind: StatueProjectile, parameter: u16);
    fn palette_color(&mut self, index: usize, value: u16);
    fn target_color(&mut self, index: usize, value: u16);
    fn spawn_palette(&mut self, definition: u16);
    fn upload(&mut self, request: Upload);
    fn unreachable(&mut self) -> u16;
    fn outside_field(&mut self, field: Field, index: u16) -> u16;
    fn put_outside_field(&mut self, field: Field, index: u16, value: u16);
    fn put_outside_counter_byte(&mut self, index: u16, value: u8);
    fn probe(&mut self, _name: &'static str) {}
}
fn field(host: &mut impl Host, column: Field, index: u16) -> u16 {
    match host.tiles().slots.get(usize::from(index >> 1)) {
        Some(slot) => slot.field(column),
        None => host.outside_field(column, index),
    }
}
fn put(host: &mut impl Host, column: Field, index: u16, value: u16) {
    match host.tiles_mut().slots.get_mut(usize::from(index >> 1)) {
        Some(slot) => *slot.field_mut(column) = value,
        None => host.put_outside_field(column, index, value),
    }
}
fn counter_byte(host: &mut impl Host, index: u16, value: u8) {
    match host.tiles_mut().slots.get_mut(usize::from(index >> 1)) {
        Some(slot) => {
            let mut bytes = slot.counter.to_le_bytes();
            bytes[usize::from(index & 1)] = value;
            slot.counter = u16::from_le_bytes(bytes);
        }
        None => host.put_outside_counter_byte(index, value),
    }
}
fn decrement(host: &mut impl Host, column: Field, index: u16) -> u16 {
    let previous = field(host, column, index);
    put(host, column, index, previous.wrapping_sub(1));
    previous
}
pub(crate) fn spawn(host: &mut impl Host, definition: u16) {
    let Some(slot) = host.tiles().slots.iter().rposition(|slot| slot.id == 0) else {
        return;
    };
    host.tiles_mut().slots[slot].id = definition;
    host.tiles_mut().slots[slot].counter = 0;
    let data = host.record(definition);
    host.tiles_mut().slots[slot].program = data.word(0);
    host.tiles_mut().slots[slot].source = 0;
    host.tiles_mut().slots[slot].size = data.word(2);
    host.tiles_mut().slots[slot].vram = data.word(4);
    host.tiles_mut().slots[slot].delay = 1;
    host.probe("animated_tiles.spawn.initialized");
}
pub(crate) fn advance(host: &mut impl Host) {
    if host.tiles().flags & 0x8000 != 0 {
        let mut index = 10i32;
        while index >= 0 {
            host.tiles_mut().selected = index as u16;
            if field(host, Id, index as u16) != 0 {
                process(host);
                index = i32::from(host.tiles().selected);
            }
            index -= 2;
        }
    }
}
fn goto(host: &mut impl Host, cursor: u16) -> u16 {
    host.record(cursor).word(0)
}
fn goto_relative(host: &mut impl Host, cursor: u16) -> u16 {
    host.tiles_mut().last_instruction = cursor;
    cursor.wrapping_add(host.record(cursor).byte(0) as i8 as u16)
}
pub(crate) fn command(host: &mut impl Host, address: u32, index: u16, cursor: u16) -> u16 {
    host.probe("animated_tiles.command.dispatched");
    match address {
        8880306 => {
            put(host, Id, index, 0);
            0
        }
        8880311 => goto(host, cursor),
        8880316 => goto_relative(host, cursor),
        8880340 | 8880348 => {
            let relative = address == 8880348;
            if decrement(host, Counter, index) == 1 {
                cursor.wrapping_add(if relative { 1 } else { 2 })
            } else if relative {
                goto_relative(host, cursor)
            } else {
                goto(host, cursor)
            }
        }
        8880355 => {
            let data = host.record(cursor);
            counter_byte(host, index, data.byte(0));
            cursor.wrapping_add(1)
        }
        8880368 | 8880380 | 8880392 | 8880404 => {
            let data = host.record(cursor);
            let sound = u16::from(data.byte(0));
            match address {
                8880368 => host.music(sound),
                8880380 => host.sound(SoundChannel::One, sound),
                8880392 => host.sound(SoundChannel::Two, sound),
                _ => host.sound(SoundChannel::Three, sound),
            }
            cursor.wrapping_add(1)
        }
        8880416 | 8880447 | 8880899 => {
            let data = host.record(cursor);
            let (width, taken) = match address {
                8880416 => (1, host.check_boss(u16::from(data.word(0) as u8))),
                8880447 => (2, host.check_event(data.word(0))),
                _ => (2, data.byte(0) & host.boss_at(data.byte(1)) != 0),
            };
            let next = cursor.wrapping_add(width);
            if taken {
                goto(host, next)
            } else {
                next.wrapping_add(2)
            }
        }
        8880435 | 8880464 => {
            let data = host.record(cursor);
            if address == 8880435 {
                host.set_boss(u16::from(data.byte(0)));
                cursor.wrapping_add(1)
            } else {
                host.set_event(data.word(0));
                cursor.wrapping_add(2)
            }
        }
        8880570 => {
            if !host.check_boss(1) {
                put(host, Delay, index, 1);
                0
            } else {
                cursor
            }
        }
        8880928 | 8880943 => {
            let data = host.record(cursor);
            let kind = if address == 8880928 {
                StatueProjectile::EyeGlow
            } else {
                StatueProjectile::Soul
            };
            let parameter = data.word(0);
            host.projectile(kind, parameter);
            cursor.wrapping_add(2)
        }
        8880958 => {
            if host.statue() & 0x8000 == 0 {
                cursor.wrapping_add(2)
            } else {
                goto(host, cursor)
            }
        }
        8880969 | 8880978 => {
            let previous = host.statue();
            let data = host.record(cursor);
            let mask = data.word(0);
            host.set_statue(if address == 8880969 {
                previous | mask
            } else {
                previous & !mask
            });
            cursor.wrapping_add(2)
        }
        8880987 => {
            let data = host.record(cursor);
            let first = usize::from(data.word(0) >> 1);
            // Widen once: the last two writes do not wrap the original displacement.
            for offset in 0..3 {
                host.palette_color(first + offset, 0);
            }
            cursor.wrapping_add(2)
        }
        8881010 => {
            let data = host.record(cursor);
            host.spawn_palette(data.word(0));
            cursor.wrapping_add(2)
        }
        8881023 => {
            let data = host.record(cursor);
            let mut color = data.word(0);
            for value in [
                0x3800, 0x7f58, 0x6ed5, 0x5a71, 0x49ee, 0x356a, 0x24e7, 0x1083,
            ] {
                host.target_color(usize::from(color >> 1), value);
                color = color.wrapping_add(2);
            }
            cursor.wrapping_add(2)
        }
        _ => host.unreachable(),
    }
}
pub(crate) fn process(host: &mut impl Host) {
    let index = host.tiles().selected;
    if decrement(host, Delay, index) != 1 {
        return;
    }
    let mut cursor = field(host, Program, index);
    let duration = loop {
        let data = host.record(cursor);
        let instruction = data.word(0);
        if instruction & 0x8000 == 0 {
            break instruction;
        }
        host.tiles_mut().last_instruction = instruction;
        cursor = command(
            host,
            0x870000 | u32::from(instruction),
            index,
            cursor.wrapping_add(2),
        );
        if cursor == 0 {
            return;
        }
    };
    put(host, Delay, index, duration);
    let data = host.record(cursor);
    put(host, Source, index, data.word(2));
    put(host, Program, index, cursor.wrapping_add(4));
    host.probe("animated_tiles.frame.requested");
}
pub(crate) fn upload_pending(host: &mut impl Host) {
    if host.tiles().flags & 0x8000 == 0 {
        return;
    }
    for slot in (0..6).rev() {
        let Some(request) = host.tiles().pending_upload(slot) else {
            continue;
        };
        host.probe("animated_tiles.upload.started");
        host.upload(request);
        host.tiles_mut().acknowledge_upload(slot);
        host.probe("animated_tiles.upload.acknowledged");
    }
}
