//! Complete palette execution over ordinary state and ordered semantic effects.
#![forbid(unsafe_code)]
use super::state::{Field, Session};
use Field::*;

/// One captured record retains its image for exactly the lifetime of that record.
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

/// The host retains the sole Session while the core executes. Opaque effects
/// may consume it and install a fresh observation; no state borrow crosses them.
pub(crate) trait Host {
    type Record: Record;
    fn session(&self) -> &Session;
    fn session_mut(&mut self) -> &mut Session;
    fn record(&mut self, address: u16) -> Self::Record;
    fn sound(&mut self, channel: SoundChannel, sound: u16);
    fn music(&mut self, track: u16);
    fn unreachable(&mut self);
    // Compatibility exceptions are explicitly outside the ordinary slot/color domain.
    fn outside_field(&mut self, field: Field, index: u16) -> u16;
    fn put_outside_field(&mut self, field: Field, index: u16, value: u16);
    fn put_outside_counter_byte(&mut self, index: u16, value: u8);
    fn outside_flag(&mut self, index: u16) -> u16;
    fn outside_color(&mut self, displacement: u16, value: u16);
    fn color_written(&mut self, _index: u16, _value: u16) {}
    fn probe(&mut self, _name: &'static str) {}
}
fn field(host: &mut impl Host, column: Field, index: u16) -> u16 {
    match host.session().effects.slots.get(usize::from(index >> 1)) {
        Some(slot) => slot.field(column),
        None => host.outside_field(column, index),
    }
}
fn put(host: &mut impl Host, column: Field, index: u16, value: u16) {
    match host
        .session_mut()
        .effects
        .slots
        .get_mut(usize::from(index >> 1))
    {
        Some(slot) => *slot.field_mut(column) = value,
        None => host.put_outside_field(column, index, value),
    }
}
fn put_native(host: &mut impl Host, column: Field, index: u16, value: u16) {
    put(host, column, index, u16::from_le_bytes(value.to_ne_bytes()));
}
fn decrement(host: &mut impl Host, column: Field, index: u16) -> u16 {
    let old = field(host, column, index);
    put(host, column, index, old.wrapping_sub(1));
    old
}
fn counter_byte(host: &mut impl Host, index: u16, value: u8) {
    match host
        .session_mut()
        .effects
        .slots
        .get_mut(usize::from(index >> 1))
    {
        Some(slot) => {
            let mut bytes = slot.counter.to_le_bytes();
            bytes[usize::from(index & 1)] = value;
            slot.counter = u16::from_le_bytes(bytes);
        }
        None => host.put_outside_counter_byte(index, value),
    }
}
fn flag_word(host: &mut impl Host, index: u16) -> u16 {
    // This pre-action intentionally tests flags, the selector, or a neighboring ID.
    // Odd and wider accesses are compatibility exceptions, not bounded slot reads.
    let value = match index {
        0 => host.session().effects.flags,
        2 => host.session().effects.selected,
        4..=18 if index & 1 == 0 => host.session().effects.slots[usize::from((index - 4) / 2)].id,
        _ => return host.outside_flag(index),
    };
    u16::from_ne_bytes(value.to_le_bytes())
}
fn color(host: &mut impl Host, displacement: u16, value: u16) {
    let index = displacement >> 1;
    match host.session_mut().colors.get_mut(usize::from(index)) {
        Some(color) => {
            *color = value;
            host.color_written(index, value);
        }
        None => host.outside_color(displacement, value),
    }
    host.probe("palette.color.write");
}
pub(crate) fn spawn(host: &mut impl Host, definition: u16) {
    let Some(slot) = host
        .session()
        .effects
        .slots
        .iter()
        .rposition(|slot| slot.id == 0)
    else {
        return;
    };
    let slot = (slot * 2) as u16;
    host.probe("palette.spawn.allocated");
    put(host, Id, slot, definition);
    put(host, Color, slot, 0);
    put(host, PreAction, slot, 0xc526);
    let data = host.record(definition);
    put(host, Program, slot, data.word(2));
    put(host, Delay, slot, 1);
    put(host, Counter, slot, 0);
    let data = host.record(definition);
    setup(host, 0x8d0000 | u32::from(data.word(0)), slot);
}
pub(crate) fn advance(host: &mut impl Host) {
    if host.session().effects.flags & 0x8000 != 0 {
        let mut slot = 14i32;
        while slot >= 0 {
            host.session_mut().effects.selected = slot as u16;
            if field(host, Id, slot as u16) != 0 {
                process(host, slot as u16);
                slot = i32::from(host.session().effects.selected);
            }
            slot -= 2;
        }
    }
}
pub(crate) fn process(host: &mut impl Host, k: u16) {
    let before = 0x8d0000 | u32::from(field(host, PreAction, k));
    pre(host, before, k);
    let mut slot = host.session().effects.selected;
    if decrement(host, Delay, slot) != 1 {
        return;
    }
    let mut cursor = field(host, Program, slot);
    let duration = loop {
        let data = host.record(cursor);
        let instruction = data.word(0);
        if instruction & 0x8000 == 0 {
            break instruction;
        }
        (slot, cursor) = command(
            host,
            0x8d0000 | u32::from(instruction),
            slot,
            cursor.wrapping_add(2),
        );
        if cursor == 0 {
            return;
        }
    };
    put(host, Delay, slot, duration);
    host.probe("palette.phase.colors");
    let mut destination = field(host, Color, slot);
    loop {
        let instruction = loop {
            let data = host.record(cursor);
            let value = data.word(2);
            if value & 0x8000 != 0 {
                break value;
            }
            color(host, destination, value);
            destination = destination.wrapping_add(2);
            cursor = cursor.wrapping_add(2);
        };
        (destination, cursor) =
            command(host, 0x8d0000 | u32::from(instruction), destination, cursor);
        if cursor == 0 {
            return;
        }
    }
}
pub(crate) fn pre(host: &mut impl Host, ea: u32, k: u16) {
    match ea {
        9291046 => {}
        9298443 => {
            if host.session().inputs.cinematic == 0xb35f {
                put(host, Program, k, 0xe192);
                put(host, Delay, k, 1);
            }
        }
        9298656 => {
            if host.session().inputs.enemy_health == 0 {
                put(host, Id, k, 0);
            }
        }
        9298809 => {
            if host.session().inputs.equipped_items & 0x21 == 0 {
                let damage = host.session().damage;
                let damage = (u32::from(damage.fraction) | (u32::from(damage.amount) << 16))
                    .wrapping_add(0x4000);
                host.session_mut().damage.fraction = damage as u16;
                host.session_mut().damage.amount = (damage >> 16) as u16;
                if host.session().inputs.frame & 7 == 0 && host.session().inputs.health > 0x46 {
                    host.sound(SoundChannel::Three, 0x2d);
                }
            }
            if host.session().effects.heat != host.session().effects.previous_heat {
                host.session_mut().effects.previous_heat = host.session().effects.heat;
                let offset = host.session().effects.heat.wrapping_mul(2);
                put(host, Delay, k, 1);
                let table = if host.session().inputs.equipped_items & 0x20 != 0 {
                    0xe3e0
                } else if host.session().inputs.equipped_items & 1 != 0 {
                    0xe400
                } else {
                    0xe420
                };
                let data = host.record(table);
                put(host, Program, k, data.word(usize::from(offset)));
            }
        }
        9301081 | 9301380 => {
            if host.session().inputs.player_y < 0x380 {
                put(host, Delay, k, 1);
                put(
                    host,
                    Program,
                    k,
                    if ea == 9301081 { 0xeb43 } else { 0xec76 },
                );
            }
        }
        9301701 => {
            if host.session().inputs.miniboss_dead {
                put(host, Id, k, 0);
            }
        }
        9303585 => {
            if flag_word(host, k) != 0 {
                put(host, Id, k, 0);
            }
        }
        _ => host.unreachable(),
    }
}
pub(crate) fn command(host: &mut impl Host, ea: u32, k: u16, j: u16) -> (u16, u16) {
    host.probe("palette.command");
    match ea {
        9291157 => {
            let selected = host.session().effects.selected;
            put_native(host, Program, selected, j.wrapping_add(4));
            host.probe("palette.continuation.save");
            (0, 0)
        }
        9291161 => (k.wrapping_add(4), j.wrapping_add(2)),
        9291170 => (k.wrapping_add(6), j.wrapping_add(2)),
        9291179 => (k.wrapping_add(8), j.wrapping_add(2)),
        9291188 => (k.wrapping_add(16), j.wrapping_add(2)),
        9291197 => (k.wrapping_add(18), j.wrapping_add(2)),
        9291206 => (k.wrapping_add(30), j.wrapping_add(2)),
        9291215 => {
            put(host, Id, k, 0);
            (k, 0)
        }
        9291220 => {
            let data = host.record(j);
            put(host, PreAction, k, data.word(0));
            (k, j.wrapping_add(2))
        }
        9291229 => {
            put(host, PreAction, k, 0xc5e3);
            (k, j)
        }
        9291294 => {
            let data = host.record(j);
            (k, data.word(0))
        }
        9291321 => {
            let cursor = if decrement(host, Counter, k) == 1 {
                j.wrapping_add(2)
            } else {
                let data = host.record(j);
                data.word(0)
            };
            (k, cursor)
        }
        9291336 => {
            let data = host.record(j);
            counter_byte(host, k, data.byte(0));
            (k, j.wrapping_add(1))
        }
        9291349 => {
            let data = host.record(j);
            put(host, Color, k, data.word(0));
            (k, j.wrapping_add(2))
        }
        9291358 => {
            let data = host.record(j);
            host.music(u16::from(data.byte(0)));
            (k, j.wrapping_add(1))
        }
        9291370 | 9291379 | 9291388 => {
            let data = host.record(j);
            let sound = data.word(0);
            host.sound(
                match ea {
                    9291370 => SoundChannel::One,
                    9291379 => SoundChannel::Two,
                    _ => SoundChannel::Three,
                },
                sound,
            );
            (k, j.wrapping_add(1))
        }
        9302470 => {
            let data = host.record(j);
            host.session_mut().effects.heat = u16::from(data.byte(0));
            (k, j.wrapping_add(1))
        }
        _ => {
            host.unreachable();
            (0, 0)
        }
    }
}
pub(crate) fn setup(host: &mut impl Host, ea: u32, slot: u16) {
    host.probe("palette.setup");
    match ea {
        9291397 => {}
        9298436 => put(host, PreAction, slot, 0xe20b),
        9299008 => {
            let script = if host.session().inputs.equipped_items & 0x20 != 0 {
                0xe8b6
            } else if host.session().inputs.equipped_items & 1 != 0 {
                0xe68a
            } else {
                0xe45e
            };
            put(host, Program, slot, script);
        }
        9303856 => {
            if host.session().inputs.miniboss_dead {
                put(host, Id, slot, 0);
            }
        }
        _ => host.unreachable(),
    }
}
