//! Gameplay sprite allocation, animation and ordered drawing on the shared Session.
use crate::{
    session::{Point, Session},
    sprite_effect_host::{Host, SpriteMember},
    sprite_effect_state::{
        LegacySpriteHandle, LegacySpriteMap, LegacySpriteWordIndex, SpawnResult, SpriteDraw,
        SpriteSlot, SpriteSpawn,
    },
    visual_host::{Record, Source},
};

// Ordinary active slots use direct fields; only broad legacy indices use projection.
macro_rules! read_slot {
    ($host:expr,$index:expr,$member:ident,$alias:ident) => {{
        let index = $index;
        match $host
            .session()
            .sprite_effects
            .active
            .get(usize::from(index))
        {
            Some(slot) => slot.$member,
            None => $host.sprite_word(SpriteMember::$alias, LegacySpriteWordIndex(index)),
        }
    }};
}
macro_rules! write_slot {
    ($host:expr,$index:expr,$member:ident,$alias:ident,$value:expr) => {{
        let index = $index;
        let value = $value;
        match $host
            .session_mut()
            .sprite_effects
            .active
            .get_mut(usize::from(index))
        {
            Some(slot) => slot.$member = value,
            None => $host.set_sprite_word_le_slice(
                SpriteMember::$alias,
                LegacySpriteWordIndex(index),
                value,
            ),
        }
    }};
}
macro_rules! write_selected_native {
    ($host:expr,$member:ident,$alias:ident,$value:expr) => {{
        let value = $value;
        let index = $host.session().sprite_effects.selected >> 1;
        match $host
            .session_mut()
            .sprite_effects
            .active
            .get_mut(usize::from(index))
        {
            Some(slot) => slot.$member = u16::from_le_bytes(value.to_ne_bytes()),
            None => $host.set_sprite_word_native_slice(
                SpriteMember::$alias,
                LegacySpriteWordIndex(index),
                value,
            ),
        }
    }};
}

pub fn spawn(host: &mut impl Host, request: SpriteSpawn) -> SpawnResult {
    let Some(index) = host
        .session()
        .sprite_effects
        .active
        .iter()
        .rposition(|slot| slot.instruction == 0)
    else {
        host.probe("sprite.spawn.full");
        return SpawnResult::Full;
    };
    {
        let slot = &mut host.session_mut().sprite_effects.active[index];
        slot.appearance = 0;
        slot.x.fraction = 0;
        slot.y.fraction = 0;
        slot.disable_flags = 0;
        slot.x.whole = request.position.x;
        slot.y.whole = request.position.y;
        slot.appearance = request.appearance;
    }
    let table = host.table(Source::Loaded, 0xb4bda8);
    let instruction = table.native_word(usize::from(request.definition.0) * 2);
    host.session_mut().sprite_effects.active[index].instruction = instruction;
    let record = host.record(Source::Loaded, 0xb4, instruction);
    let timer = record.native_word(0);
    // Allocation retains its captured destination even if diagnostics changed selected.
    host.session_mut().sprite_effects.active[index].timer = timer;
    host.probe("sprite.spawn.allocated");
    SpawnResult::Allocated(SpriteSlot(index as u8))
}

pub fn legacy_result(result: SpawnResult) -> LegacySpriteHandle {
    LegacySpriteHandle(match result {
        SpawnResult::Allocated(slot) => u16::from(slot.0) * 2,
        SpawnResult::Full => 0xffff,
    })
}

fn instruction(host: &mut impl Host, opcode: u16) {
    match opcode {
        0xbcf0 => {
            let index = host.session().sprite_effects.selected >> 1;
            let previous = read_slot!(host, index, instruction, Instruction);
            write_slot!(
                host,
                index,
                instruction,
                Instruction,
                previous.wrapping_sub(4)
            );
            write_slot!(host, index, timer, Timer, 0x7fff);
            host.probe("sprite.command.hold");
        }
        0xbd07 => {
            write_selected_native!(host, instruction, Instruction, 0u16);
            host.probe("sprite.command.retire");
        }
        0xbd12 => {
            let index = host.session().sprite_effects.selected >> 1;
            let current = match host.session().sprite_effects.active.get(usize::from(index)) {
                Some(slot) => u16::from_ne_bytes(slot.instruction.to_le_bytes()),
                None => {
                    host.sprite_word_native(SpriteMember::Instruction, LegacySpriteWordIndex(index))
                }
            };
            let record = host.record(Source::Loaded, 0xb4, current);
            let target = record.native_word(2);
            write_selected_native!(host, instruction, Instruction, target);
            let record = host.record(Source::Loaded, 0xb4, target);
            let timer = record.native_word(0);
            write_selected_native!(host, timer, Timer, timer);
            host.probe("sprite.command.jump");
        }
        _ => host.invalid_sprite_instruction(),
    }
}

pub fn update(host: &mut impl Host) {
    if host.inputs().debug_enemy_freeze | host.inputs().frozen != 0 {
        return;
    }
    host.session_mut().sprite_effects.selected = 62;
    loop {
        let index = host.session().sprite_effects.selected >> 1;
        if read_slot!(host, index, instruction, Instruction) != 0
            && read_slot!(host, index, disable_flags, DisableFlags) & 1 == 0
        {
            let mut timer = read_slot!(host, index, timer, Timer);
            if timer & 0x8000 == 0 {
                write_slot!(host, index, timer, Timer, timer.wrapping_sub(1));
                if timer == 1 {
                    let next = read_slot!(host, index, instruction, Instruction).wrapping_add(4);
                    write_slot!(host, index, instruction, Instruction, next);
                    let record = host.record(Source::Loaded, 0xb4, next);
                    timer = record.native_word(0);
                    if timer & 0x8000 == 0 {
                        write_selected_native!(host, timer, Timer, timer);
                    }
                }
            }
            // A jump targets a new record but never chains another opcode this tick.
            if timer & 0x8000 != 0 {
                instruction(host, timer);
                host.probe("sprite.update.command");
            }
            host.probe("sprite.update.executed");
        }
        host.session_mut().sprite_effects.selected =
            host.session().sprite_effects.selected.wrapping_sub(2);
        if host.session().sprite_effects.selected & 0x8000 != 0 {
            break;
        }
    }
}

pub fn draw(host: &mut impl Host) {
    for index in (0..32).rev() {
        if host.session().sprite_effects.active[index].instruction == 0 {
            continue;
        }
        let x = host.session().sprite_effects.active[index]
            .x
            .whole
            .wrapping_sub(host.inputs().camera_x);
        if (x as i16).wrapping_add(16) < 0 || (i32::from(x) - 272) & 0x8000 == 0 {
            continue;
        }
        let y = (host.session().sprite_effects.active[index].y.whole as i16)
            .wrapping_sub(host.inputs().camera_y as i16);
        if y < 0 || (i32::from(y) - 272) & 0x8000 == 0 {
            continue;
        }
        let palette = host.session().sprite_effects.active[index].appearance & 0xe00;
        let base_tile = host.session().sprite_effects.active[index].appearance & 0x1ff;
        let cursor = host.session().sprite_effects.active[index].instruction;
        let record = host.record(Source::Loaded, 0xb4, cursor);
        let map = LegacySpriteMap(record.native_word(2));
        host.submit_sprite(SpriteDraw {
            map,
            position: Point { x, y: y as u16 },
            palette,
            base_tile,
        });
        host.probe("sprite.draw.submitted");
    }
}

pub fn clear_for_population_load(session: &mut Session) {
    // EF78 survives; all seven other columns and every serialized tail are cleared.
    for (index, slot) in session.sprite_effects.active.iter_mut().enumerate() {
        if index != 0 {
            slot.instruction = 0;
        }
        slot.timer = 0;
        slot.appearance = 0;
        slot.x.whole = 0;
        slot.x.fraction = 0;
        slot.y.whole = 0;
        slot.y.fraction = 0;
        slot.disable_flags = 0;
    }
    let tails = &mut session.sprite_compatibility;
    tails.instruction.fill(0);
    tails.timer.fill(0);
    tails.appearance.fill(0);
    tails.x.fill(0);
    tails.x_fraction.fill(0);
    tails.y.fill(0);
    tails.y_fraction.fill(0);
    tails.disable_flags.fill(0);
    crate::probe::hit("sprite.population.clear");
}

pub fn retire_all(session: &mut Session) {
    for slot in &mut session.sprite_effects.active {
        slot.instruction = 0;
    }
}
