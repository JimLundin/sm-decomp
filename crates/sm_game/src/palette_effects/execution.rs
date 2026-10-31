//! Palette instructions on the sole shared gameplay Session.
use crate::program_banks::{self, Bank};
use crate::{
    gameplay_flag_host::BossWordOrder,
    progress_flags,
    session::Session,
    visual_host::{Host, PaletteMember},
};
use sm_data::palette_programs::Command;

fn program_word(host: &mut impl Host, cursor: u16, displacement: usize) -> u16 {
    program_banks::word(host, Bank::Palette, cursor, displacement)
}
fn program_byte(host: &mut impl Host, cursor: u16, displacement: usize) -> u8 {
    program_banks::byte(host, Bank::Palette, cursor, displacement)
}

// Only exceptional handles use a semantic selector. Valid slots access members directly.
macro_rules! alias_member {
    (definition) => {
        PaletteMember::Definition
    };
    (color_offset) => {
        PaletteMember::ColorOffset
    };
    (pre_action) => {
        PaletteMember::PreAction
    };
    (program) => {
        PaletteMember::Program
    };
    (delay) => {
        PaletteMember::Delay
    };
    (counter) => {
        PaletteMember::Counter
    };
}
macro_rules! field {
    ($host:expr,$member:ident,$handle:expr $(,)?) => {{
        let handle = $handle;
        match $host
            .session()
            .presentation
            .visual()
            .palette_effects
            .slots
            .get(usize::from(handle >> 1))
        {
            Some(slot) => slot.$member,
            None => $host.palette_word(alias_member!($member), handle),
        }
    }};
}
macro_rules! put {
    ($host:expr,$member:ident,$handle:expr,$value:expr $(,)?) => {{
        let handle = $handle;
        let value = $value;
        match $host
            .session_mut()
            .presentation
            .visual_mut()
            .palette_effects
            .slots
            .get_mut(usize::from(handle >> 1))
        {
            Some(slot) => slot.$member = value,
            None => $host.set_palette_word_slice(alias_member!($member), handle, value),
        }
    }};
}
macro_rules! put_native {
    ($host:expr,$member:ident,$handle:expr,$value:expr $(,)?) => {
        put!(
            $host,
            $member,
            $handle,
            u16::from_le_bytes(($value).to_ne_bytes())
        )
    };
}
macro_rules! decrement {
    ($host:expr,$member:ident,$handle:expr $(,)?) => {{
        let handle = $handle;
        let old = field!($host, $member, handle);
        put!($host, $member, handle, old.wrapping_sub(1));
        old
    }};
}
pub fn set_enabled(session: &mut Session, enabled: bool) {
    let flags = &mut session.presentation.visual_mut().palette_effects.flags;
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
        .palette_effects
        .slots
        .iter_mut()
        .rev()
    {
        slot.definition = 0;
    }
}
fn counter_byte(host: &mut impl Host, index: u16, value: u8) {
    match host
        .session_mut()
        .presentation
        .visual_mut()
        .palette_effects
        .slots
        .get_mut(usize::from(index >> 1))
    {
        Some(slot) => {
            let mut bytes = slot.counter.to_le_bytes();
            bytes[usize::from(index & 1)] = value;
            slot.counter = u16::from_le_bytes(bytes);
        }
        None => host.set_palette_counter_byte(index, value),
    }
}
fn flag_word(host: &mut impl Host, index: u16) -> u16 {
    // This pre-action intentionally tests flags, the selector, or a neighboring ID.
    // Odd and wider accesses are compatibility exceptions, not bounded slot reads.
    let value = match index {
        0 => host.session().presentation.visual().palette_effects.flags,
        2 => {
            host.session()
                .presentation
                .visual()
                .palette_effects
                .selected
        }
        4..=18 if index & 1 == 0 => {
            host.session().presentation.visual().palette_effects.slots[usize::from((index - 4) / 2)]
                .definition
        }
        _ => return host.palette_flag_native(index),
    };
    u16::from_ne_bytes(value.to_le_bytes())
}
fn color(host: &mut impl Host, displacement: u16, value: u16) {
    let index = displacement >> 1;
    match host
        .session_mut()
        .presentation
        .visual_mut()
        .colors
        .get_mut(usize::from(index))
    {
        Some(color) => {
            *color = value;
            host.color_written(index, value);
        }
        None => host.set_palette_color_slice(displacement, value),
    }
    host.probe("palette.color.write");
}
pub fn spawn(host: &mut impl Host, definition: u16) {
    let Some(slot) = host
        .session()
        .presentation
        .visual()
        .palette_effects
        .slots
        .iter()
        .rposition(|slot| slot.definition == 0)
    else {
        return;
    };
    let slot = (slot * 2) as u16;
    host.probe("palette.spawn.allocated");
    put!(host, definition, slot, definition);
    put!(host, color_offset, slot, 0);
    put!(host, pre_action, slot, 0xc526);
    let program = program_word(host, definition, 2);
    put!(host, program, slot, program);
    put!(host, delay, slot, 1);
    put!(host, counter, slot, 0);
    let setup_routine = program_word(host, definition, 0);
    setup(host, 0x8d0000 | u32::from(setup_routine), slot);
}
pub fn advance(host: &mut impl Host) {
    if host.session().presentation.visual().palette_effects.flags & 0x8000 != 0 {
        let mut slot = 14i32;
        while slot >= 0 {
            host.session_mut()
                .presentation
                .visual_mut()
                .palette_effects
                .selected = slot as u16;
            if field!(host, definition, slot as u16) != 0 {
                process(host, slot as u16);
                slot = i32::from(
                    host.session()
                        .presentation
                        .visual()
                        .palette_effects
                        .selected,
                );
            }
            slot -= 2;
        }
    }
}
pub fn process(host: &mut impl Host, k: u16) {
    let before = 0x8d0000 | u32::from(field!(host, pre_action, k));
    pre(host, before, k);
    let mut slot = host
        .session()
        .presentation
        .visual()
        .palette_effects
        .selected;
    if decrement!(host, delay, slot) != 1 {
        return;
    }
    let mut cursor = field!(host, program, slot);
    let duration = loop {
        let instruction = program_word(host, cursor, 0);
        if instruction & 0x8000 == 0 {
            break instruction;
        }
        (slot, cursor) = command(host, instruction, slot, cursor.wrapping_add(2));
        if cursor == 0 {
            return;
        }
    };
    put!(host, delay, slot, duration);
    host.probe("palette.phase.colors");
    let mut destination = field!(host, color_offset, slot);
    loop {
        let instruction = loop {
            let value = program_word(host, cursor, 2);
            if value & 0x8000 != 0 {
                break value;
            }
            color(host, destination, value);
            destination = destination.wrapping_add(2);
            cursor = cursor.wrapping_add(2);
        };
        (destination, cursor) = command(host, instruction, destination, cursor);
        if cursor == 0 {
            return;
        }
    }
}
pub fn pre(host: &mut impl Host, ea: u32, k: u16) {
    match ea {
        9291046 => {}
        9298443 => {
            if host.inputs().cinematic == 0xb35f {
                put!(host, program, k, 0xe192);
                put!(host, delay, k, 1);
            }
        }
        9298656 => {
            if host.inputs().enemy_zero_health == 0 {
                put!(host, definition, k, 0);
            }
        }
        9298809 => {
            if host.session().inventory.equipment.active & 0x21 == 0 {
                let fraction = u16::from_ne_bytes(
                    host.session().player.periodic_damage_fraction.to_le_bytes(),
                );
                let amount =
                    u16::from_ne_bytes(host.session().player.periodic_damage.to_le_bytes());
                let damage = (u32::from(fraction) | (u32::from(amount) << 16)).wrapping_add(0x4000);
                host.session_mut().player.periodic_damage_fraction =
                    u16::from_le_bytes((damage as u16).to_ne_bytes());
                host.session_mut().player.periodic_damage =
                    u16::from_le_bytes(((damage >> 16) as u16).to_ne_bytes());
                if host.inputs().frame & 7 == 0 && host.session().inventory.health.current > 0x46 {
                    host.sound(3, 6, 0x2d);
                }
            }
            if host.session().presentation.visual().palette_effects.heat
                != host
                    .session()
                    .presentation
                    .visual()
                    .palette_effects
                    .previous_heat
            {
                host.session_mut()
                    .presentation
                    .visual_mut()
                    .palette_effects
                    .previous_heat = host.session().presentation.visual().palette_effects.heat;
                let offset = host
                    .session()
                    .presentation
                    .visual()
                    .palette_effects
                    .heat
                    .wrapping_mul(2);
                put!(host, delay, k, 1);
                let table = if host.session().inventory.equipment.active & 0x20 != 0 {
                    0xe3e0
                } else if host.session().inventory.equipment.active & 1 != 0 {
                    0xe400
                } else {
                    0xe420
                };
                let program = program_word(host, table, usize::from(offset));
                put!(host, program, k, program);
            }
        }
        9301081 | 9301380 => {
            if host.session().player.y < 0x380 {
                put!(host, delay, k, 1);
                put!(
                    host,
                    program,
                    k,
                    if ea == 9301081 { 0xeb43 } else { 0xec76 },
                );
            }
        }
        9301701 => {
            if progress_flags::boss_word_at(host, host.session().room.area, BossWordOrder::Native)
                & 2
                != 0
            {
                put!(host, definition, k, 0);
            }
        }
        9303585 => {
            if flag_word(host, k) != 0 {
                put!(host, definition, k, 0);
            }
        }
        _ => host.invalid_palette_instruction(),
    }
}
/// Runs the command whose handler is at $8D:`handler` with colour offset or
/// slot `k` and cursor `j`; returns them updated, cursor 0 to stop the phase.
pub fn command(host: &mut impl Host, handler: u16, k: u16, j: u16) -> (u16, u16) {
    host.probe("palette.command");
    let Some(command) = Command::decode(handler) else {
        host.invalid_palette_instruction();
        return (0, 0);
    };
    match command {
        Command::Done => {
            let selected = host
                .session()
                .presentation
                .visual()
                .palette_effects
                .selected;
            put_native!(host, program, selected, j.wrapping_add(4));
            host.probe("palette.continuation.save");
            (0, 0)
        }
        Command::Skip { colours } => (k.wrapping_add(2 * colours), j.wrapping_add(2)),
        Command::Delete => {
            put!(host, definition, k, 0);
            (k, 0)
        }
        Command::SetPreInstruction => {
            let pre_action = program_word(host, j, 0);
            put!(host, pre_action, k, pre_action);
            (k, j.wrapping_add(2))
        }
        Command::ClearPreInstruction => {
            put!(host, pre_action, k, 0xc5e3);
            (k, j)
        }
        Command::Goto => (k, program_word(host, j, 0)),
        Command::DecrementGoto => {
            let cursor = if decrement!(host, counter, k) == 1 {
                j.wrapping_add(2)
            } else {
                program_word(host, j, 0)
            };
            (k, cursor)
        }
        Command::SetCounter => {
            let counter = program_byte(host, j, 0);
            counter_byte(host, k, counter);
            (k, j.wrapping_add(1))
        }
        Command::SetColourIndex => {
            let colour_offset = program_word(host, j, 0);
            put!(host, color_offset, k, colour_offset);
            (k, j.wrapping_add(2))
        }
        Command::QueueMusic => {
            let track = program_byte(host, j, 0);
            host.music_delayed8(u16::from(track));
            (k, j.wrapping_add(1))
        }
        Command::QueueSound { library } => {
            let sound = program_word(host, j, 0);
            host.sound(library, 6, sound);
            (k, j.wrapping_add(1))
        }
        Command::SetHeat => {
            let heat = program_byte(host, j, 0);
            host.session_mut()
                .presentation
                .visual_mut()
                .palette_effects
                .heat = u16::from(heat);
            (k, j.wrapping_add(1))
        }
    }
}
pub fn setup(host: &mut impl Host, ea: u32, slot: u16) {
    host.probe("palette.setup");
    match ea {
        9291397 => {}
        9298436 => put!(host, pre_action, slot, 0xe20b),
        9299008 => {
            let script = if host.session().inventory.equipment.active & 0x20 != 0 {
                0xe8b6
            } else if host.session().inventory.equipment.active & 1 != 0 {
                0xe68a
            } else {
                0xe45e
            };
            put!(host, program, slot, script);
        }
        9303856 => {
            if progress_flags::boss_word_at(host, host.session().room.area, BossWordOrder::Native)
                & 2
                != 0
            {
                put!(host, definition, slot, 0);
            }
        }
        _ => host.invalid_palette_instruction(),
    }
}
