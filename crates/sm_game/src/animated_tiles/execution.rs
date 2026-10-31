//! Animated-tile execution over the shared visual Session.
use crate::{
    program_banks::{self, Bank},
    progress_flags,
    session::Session,
    visual_host::{AnimationMember, Host, StatueProjectile},
    visual_state::{PaletteBuffer, Upload},
};
use sm_data::animated_programs::Command;

fn program_word(host: &mut impl Host, cursor: u16, displacement: usize) -> u16 {
    program_banks::word(host, Bank::Animated, cursor, displacement)
}
fn program_byte(host: &mut impl Host, cursor: u16, displacement: usize) -> u8 {
    program_banks::byte(host, Bank::Animated, cursor, displacement)
}

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
    let entry = &mut host
        .session_mut()
        .presentation
        .visual_mut()
        .animated_tiles
        .slots[slot];
    entry.definition = definition;
    entry.counter = 0;
    // Each store precedes the next read, so a short record leaves the stores
    // before it, as the original's do.
    let program = program_word(host, definition, 0);
    slot_mut(host, slot).program = program;
    slot_mut(host, slot).source = 0;
    let size = program_word(host, definition, 2);
    slot_mut(host, slot).size = size;
    let vram = program_word(host, definition, 4);
    slot_mut(host, slot).vram = vram;
    slot_mut(host, slot).delay = 1;
    host.probe("animated_tiles.spawn.initialized");
}
fn slot_mut(host: &mut impl Host, slot: usize) -> &mut crate::visual_state::AnimationSlot {
    &mut host
        .session_mut()
        .presentation
        .visual_mut()
        .animated_tiles
        .slots[slot]
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
    program_word(host, cursor, 0)
}
fn goto_relative(host: &mut impl Host, cursor: u16) -> u16 {
    host.session_mut()
        .presentation
        .visual_mut()
        .animated_tiles
        .last_instruction = cursor;
    cursor.wrapping_add(program_byte(host, cursor, 0) as i8 as u16)
}
/// Runs the command whose handler is at $87:`handler` for animation
/// `index`; returns the next cursor, 0 to stop the frame.
pub fn command(host: &mut impl Host, handler: u16, index: u16, cursor: u16) -> u16 {
    host.probe("animated_tiles.command.dispatched");
    let Some(command) = Command::decode(handler) else {
        return host.invalid_animation_instruction();
    };
    match command {
        Command::Delete => {
            animation!(host, index, definition = 0);
            0
        }
        Command::Goto => goto(host, cursor),
        Command::GotoRelative => goto_relative(host, cursor),
        Command::DecrementGoto { relative } => {
            if decrement_counter(host, index) == 1 {
                cursor.wrapping_add(if relative { 1 } else { 2 })
            } else if relative {
                goto_relative(host, cursor)
            } else {
                goto(host, cursor)
            }
        }
        Command::SetCounter => {
            let counter = program_byte(host, cursor, 0);
            counter_byte(host, index, counter);
            cursor.wrapping_add(1)
        }
        Command::QueueMusic => {
            let track = program_byte(host, cursor, 0);
            host.music_delayed8(u16::from(track));
            cursor.wrapping_add(1)
        }
        Command::QueueSound { library } => {
            let sound = program_byte(host, cursor, 0);
            _ = host.sound(library, 6, u16::from(sound));
            cursor.wrapping_add(1)
        }
        Command::GotoIfBossDefeated | Command::GotoIfEvent | Command::GotoIfBossBits => {
            let (width, taken) = match command {
                Command::GotoIfBossDefeated => {
                    let boss = program_word(host, cursor, 0) as u8;
                    (1, progress_flags::boss_defeated(host, u16::from(boss)))
                }
                Command::GotoIfEvent => {
                    let event = program_word(host, cursor, 0);
                    (2, progress_flags::event_happened(host, event))
                }
                _ => {
                    let bits = program_byte(host, cursor, 0);
                    let area = program_byte(host, cursor, 1);
                    (
                        2,
                        bits & progress_flags::boss_byte_at(host, u16::from(area)) != 0,
                    )
                }
            };
            let next = cursor.wrapping_add(width);
            if taken {
                goto(host, next)
            } else {
                next.wrapping_add(2)
            }
        }
        Command::MarkBossDefeated => {
            let boss = program_byte(host, cursor, 0);
            progress_flags::mark_boss_defeated(host, u16::from(boss));
            cursor.wrapping_add(1)
        }
        Command::SetEvent => {
            let event = program_word(host, cursor, 0);
            progress_flags::set_event(host, event);
            cursor.wrapping_add(2)
        }
        Command::WaitForBoss => {
            if !progress_flags::boss_defeated(host, 1) {
                animation!(host, index, delay = 1);
                0
            } else {
                cursor
            }
        }
        Command::SpawnEyeGlow | Command::SpawnSoul => {
            let kind = if command == Command::SpawnEyeGlow {
                StatueProjectile::EyeGlow
            } else {
                StatueProjectile::Soul
            };
            let parameter = program_word(host, cursor, 0);
            host.spawn_statue_projectile(kind, parameter);
            cursor.wrapping_add(2)
        }
        Command::GotoIfStatueAnimating => {
            if host.session().mechanisms.statue_animation & 0x8000 == 0 {
                cursor.wrapping_add(2)
            } else {
                goto(host, cursor)
            }
        }
        Command::SetStatueBits | Command::ClearStatueBits => {
            let previous = host.session().mechanisms.statue_animation;
            let mask = program_word(host, cursor, 0);
            host.session_mut().mechanisms.statue_animation = if command == Command::SetStatueBits {
                previous | mask
            } else {
                previous & !mask
            };
            cursor.wrapping_add(2)
        }
        Command::ClearColours => {
            let first = usize::from(program_word(host, cursor, 0) >> 1);
            // Widen once: the last two writes do not wrap the original displacement.
            for offset in 0..3 {
                write_color(host, PaletteBuffer::Current, first + offset, 0);
            }
            cursor.wrapping_add(2)
        }
        Command::SpawnPaletteEffect => {
            let effect = program_word(host, cursor, 0);
            host.spawn_palette(effect);
            cursor.wrapping_add(2)
        }
        Command::WriteStatuePalette => {
            let mut color = program_word(host, cursor, 0);
            for value in [
                0x3800, 0x7f58, 0x6ed5, 0x5a71, 0x49ee, 0x356a, 0x24e7, 0x1083,
            ] {
                write_color(host, PaletteBuffer::Target, usize::from(color >> 1), value);
                color = color.wrapping_add(2);
            }
            cursor.wrapping_add(2)
        }
    }
}
pub fn process(host: &mut impl Host) {
    let index = host.session().presentation.visual().animated_tiles.selected;
    if decrement_delay(host, index) != 1 {
        return;
    }
    let mut cursor = animation!(host, index, program);
    let duration = loop {
        let instruction = program_word(host, cursor, 0);
        if instruction & 0x8000 == 0 {
            break instruction;
        }
        host.session_mut()
            .presentation
            .visual_mut()
            .animated_tiles
            .last_instruction = instruction;
        cursor = command(host, instruction, index, cursor.wrapping_add(2));
        if cursor == 0 {
            return;
        }
    };
    animation!(host, index, delay = duration);
    let source = program_word(host, cursor, 2);
    animation!(host, index, source = source);
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

/// The compatibility entries: each runs on a fresh lease of `game`.
pub mod leased {
    use crate::runtime_access::RuntimeAccess;
    use crate::session_access::SessionAccess;

    pub fn set_enabled<H: RuntimeAccess + ?Sized>(enabled: bool, game: &mut H) {
        super::set_enabled(game.runtime().session_mut(), enabled);
    }
    pub fn clear<H: RuntimeAccess + ?Sized>(game: &mut H) {
        super::clear(game.runtime().session_mut());
    }
    pub fn spawn<H: RuntimeAccess + ?Sized>(definition: u16, game: &mut H) {
        super::spawn(&mut game.runtime(), definition);
    }
    pub fn advance<H: RuntimeAccess + ?Sized>(game: &mut H) {
        super::advance(&mut game.runtime());
    }
    pub fn upload_pending<H: RuntimeAccess + ?Sized>(game: &mut H) {
        super::upload_pending(&mut game.runtime());
    }
}
