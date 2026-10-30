//! Animated-tile execution over the shared visual Session.
use crate::{
    progress_flags,
    session::Session,
    visual_host::{AnimationMember, Host, Record, Source, StatueProjectile},
    visual_state::{PaletteBuffer, Upload},
};

macro_rules! member {
    (definition) => {
        AnimationMember::Definition
    };
    (counter) => {
        AnimationMember::Counter
    };
    (program) => {
        AnimationMember::Program
    };
    (delay) => {
        AnimationMember::Delay
    };
    (source) => {
        AnimationMember::Source
    };
}
// Only exceptional handles use the finite alias interface. Ordinary handles,
// including odd handles rounded down, access the actual shared slot member.
macro_rules! animation {
    ($host:ident, $index:expr, $field:ident) => {{
        let index: u16 = $index;
        match $host
            .session()
            .presentation
            .visual()
            .animated_tiles
            .slots
            .get(usize::from(index >> 1))
        {
            Some(slot) => slot.$field,
            None => $host.animation_word(member!($field), index),
        }
    }};
    ($host:ident, $index:expr, $field:ident = $value:expr) => {{
        let index: u16 = $index;
        let value: u16 = $value;
        match $host
            .session_mut()
            .presentation
            .visual_mut()
            .animated_tiles
            .slots
            .get_mut(usize::from(index >> 1))
        {
            Some(slot) => slot.$field = value,
            None => $host.set_animation_word_prefix(member!($field), index, value),
        }
    }};
}
fn counter_byte(host: &mut impl Host, index: u16, value: u8) {
    match host
        .session_mut()
        .presentation
        .visual_mut()
        .animated_tiles
        .slots
        .get_mut(usize::from(index >> 1))
    {
        Some(slot) => {
            let mut bytes = slot.counter.to_le_bytes();
            bytes[usize::from(index & 1)] = value;
            slot.counter = u16::from_le_bytes(bytes);
        }
        None => host.set_animation_counter_byte(index, value),
    }
}
fn decrement_counter(host: &mut impl Host, index: u16) -> u16 {
    let previous = animation!(host, index, counter);
    animation!(host, index, counter = previous.wrapping_sub(1));
    previous
}
fn decrement_delay(host: &mut impl Host, index: u16) -> u16 {
    let previous = animation!(host, index, delay);
    animation!(host, index, delay = previous.wrapping_sub(1));
    previous
}
fn write_color(host: &mut impl Host, buffer: PaletteBuffer, index: usize, value: u16) {
    let colors = match buffer {
        PaletteBuffer::Current => &mut host.session_mut().presentation.visual_mut().colors,
        PaletteBuffer::Target => &mut host.session_mut().presentation.visual_mut().targets,
    };
    if let Some(color) = colors.get_mut(index) {
        *color = value;
    } else {
        host.set_color_prefix(buffer, index, value);
    }
}
pub fn set_enabled(session: &mut Session, enabled: bool) {
    let flags = &mut session.presentation.visual_mut().animated_tiles.flags;
    *flags = if enabled {
        *flags | 0x8000
    } else {
        *flags & !0x8000
    };
}
pub fn clear(session: &mut Session) {
    for slot in session
        .presentation
        .visual_mut()
        .animated_tiles
        .slots
        .iter_mut()
        .rev()
    {
        slot.definition = 0;
    }
}
pub fn spawn(host: &mut impl Host, definition: u16) {
    let Some(slot) = host
        .session()
        .presentation
        .visual()
        .animated_tiles
        .slots
        .iter()
        .rposition(|slot| slot.definition == 0)
    else {
        return;
    };
    host.session_mut()
        .presentation
        .visual_mut()
        .animated_tiles
        .slots[slot]
        .definition = definition;
    host.session_mut()
        .presentation
        .visual_mut()
        .animated_tiles
        .slots[slot]
        .counter = 0;
    let data = host.record(Source::Loaded, 0x87, definition);
    host.session_mut()
        .presentation
        .visual_mut()
        .animated_tiles
        .slots[slot]
        .program = data.native_word(0);
    host.session_mut()
        .presentation
        .visual_mut()
        .animated_tiles
        .slots[slot]
        .source = 0;
    host.session_mut()
        .presentation
        .visual_mut()
        .animated_tiles
        .slots[slot]
        .size = data.native_word(2);
    host.session_mut()
        .presentation
        .visual_mut()
        .animated_tiles
        .slots[slot]
        .vram = data.native_word(4);
    host.session_mut()
        .presentation
        .visual_mut()
        .animated_tiles
        .slots[slot]
        .delay = 1;
    host.probe("animated_tiles.spawn.initialized");
}
pub fn advance(host: &mut impl Host) {
    if host.session().presentation.visual().animated_tiles.flags & 0x8000 != 0 {
        let mut index = 10i32;
        while index >= 0 {
            host.session_mut()
                .presentation
                .visual_mut()
                .animated_tiles
                .selected = index as u16;
            if animation!(host, index as u16, definition) != 0 {
                process(host);
                index = i32::from(host.session().presentation.visual().animated_tiles.selected);
            }
            index -= 2;
        }
    }
}
fn goto(host: &mut impl Host, cursor: u16) -> u16 {
    host.record(Source::Loaded, 0x87, cursor).native_word(0)
}
fn goto_relative(host: &mut impl Host, cursor: u16) -> u16 {
    host.session_mut()
        .presentation
        .visual_mut()
        .animated_tiles
        .last_instruction = cursor;
    cursor.wrapping_add(host.record(Source::Loaded, 0x87, cursor).byte(0) as i8 as u16)
}
pub fn command(host: &mut impl Host, address: u32, index: u16, cursor: u16) -> u16 {
    host.probe("animated_tiles.command.dispatched");
    match address {
        8880306 => {
            animation!(host, index, definition = 0);
            0
        }
        8880311 => goto(host, cursor),
        8880316 => goto_relative(host, cursor),
        8880340 | 8880348 => {
            let relative = address == 8880348;
            if decrement_counter(host, index) == 1 {
                cursor.wrapping_add(if relative { 1 } else { 2 })
            } else if relative {
                goto_relative(host, cursor)
            } else {
                goto(host, cursor)
            }
        }
        8880355 => {
            let data = host.record(Source::Loaded, 0x87, cursor);
            counter_byte(host, index, data.byte(0));
            cursor.wrapping_add(1)
        }
        8880368 | 8880380 | 8880392 | 8880404 => {
            let data = host.record(Source::Loaded, 0x87, cursor);
            let sound = u16::from(data.byte(0));
            match address {
                8880368 => host.music_delayed8(sound),
                8880380 => _ = host.sound(1, 6, sound),
                8880392 => _ = host.sound(2, 6, sound),
                _ => _ = host.sound(3, 6, sound),
            }
            cursor.wrapping_add(1)
        }
        8880416 | 8880447 | 8880899 => {
            let data = host.record(Source::Loaded, 0x87, cursor);
            let (width, taken) = match address {
                8880416 => (
                    1,
                    progress_flags::boss_defeated(host, u16::from(data.native_word(0) as u8)),
                ),
                8880447 => (2, progress_flags::event_happened(host, data.native_word(0))),
                _ => (
                    2,
                    data.byte(0) & progress_flags::boss_byte_at(host, u16::from(data.byte(1))) != 0,
                ),
            };
            let next = cursor.wrapping_add(width);
            if taken {
                goto(host, next)
            } else {
                next.wrapping_add(2)
            }
        }
        8880435 | 8880464 => {
            let data = host.record(Source::Loaded, 0x87, cursor);
            if address == 8880435 {
                progress_flags::mark_boss_defeated(host, u16::from(data.byte(0)));
                cursor.wrapping_add(1)
            } else {
                progress_flags::set_event(host, data.native_word(0));
                cursor.wrapping_add(2)
            }
        }
        8880570 => {
            if !progress_flags::boss_defeated(host, 1) {
                animation!(host, index, delay = 1);
                0
            } else {
                cursor
            }
        }
        8880928 | 8880943 => {
            let data = host.record(Source::Loaded, 0x87, cursor);
            let kind = if address == 8880928 {
                StatueProjectile::EyeGlow
            } else {
                StatueProjectile::Soul
            };
            let parameter = data.native_word(0);
            host.spawn_statue_projectile(kind, parameter);
            cursor.wrapping_add(2)
        }
        8880958 => {
            if host.session().mechanisms.statue_animation & 0x8000 == 0 {
                cursor.wrapping_add(2)
            } else {
                goto(host, cursor)
            }
        }
        8880969 | 8880978 => {
            let previous = host.session().mechanisms.statue_animation;
            let data = host.record(Source::Loaded, 0x87, cursor);
            let mask = data.native_word(0);
            host.session_mut().mechanisms.statue_animation = if address == 8880969 {
                previous | mask
            } else {
                previous & !mask
            };
            cursor.wrapping_add(2)
        }
        8880987 => {
            let data = host.record(Source::Loaded, 0x87, cursor);
            let first = usize::from(data.native_word(0) >> 1);
            // Widen once: the last two writes do not wrap the original displacement.
            for offset in 0..3 {
                write_color(host, PaletteBuffer::Current, first + offset, 0);
            }
            cursor.wrapping_add(2)
        }
        8881010 => {
            let data = host.record(Source::Loaded, 0x87, cursor);
            host.spawn_palette(data.native_word(0));
            cursor.wrapping_add(2)
        }
        8881023 => {
            let data = host.record(Source::Loaded, 0x87, cursor);
            let mut color = data.native_word(0);
            for value in [
                0x3800, 0x7f58, 0x6ed5, 0x5a71, 0x49ee, 0x356a, 0x24e7, 0x1083,
            ] {
                write_color(host, PaletteBuffer::Target, usize::from(color >> 1), value);
                color = color.wrapping_add(2);
            }
            cursor.wrapping_add(2)
        }
        _ => host.invalid_animation_instruction(),
    }
}
pub fn process(host: &mut impl Host) {
    let index = host.session().presentation.visual().animated_tiles.selected;
    if decrement_delay(host, index) != 1 {
        return;
    }
    let mut cursor = animation!(host, index, program);
    let duration = loop {
        let data = host.record(Source::Loaded, 0x87, cursor);
        let instruction = data.native_word(0);
        if instruction & 0x8000 == 0 {
            break instruction;
        }
        host.session_mut()
            .presentation
            .visual_mut()
            .animated_tiles
            .last_instruction = instruction;
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
    animation!(host, index, delay = duration);
    let data = host.record(Source::Loaded, 0x87, cursor);
    animation!(host, index, source = data.native_word(2));
    animation!(host, index, program = cursor.wrapping_add(4));
    host.probe("animated_tiles.frame.requested");
}
pub fn upload_pending(host: &mut impl Host) {
    if host.session().presentation.visual().animated_tiles.flags & 0x8000 == 0 {
        return;
    }
    for slot in (0..6).rev() {
        let current = &host.session().presentation.visual().animated_tiles.slots[slot];
        if current.definition == 0 || current.source == 0 {
            continue;
        }
        let request = Upload {
            source: current.source,
            size: current.size,
            vram: current.vram,
        };
        host.probe("animated_tiles.upload.started");
        host.upload(request);
        host.session_mut()
            .presentation
            .visual_mut()
            .animated_tiles
            .slots[slot]
            .source = 0;
        host.probe("animated_tiles.upload.acknowledged");
    }
}
