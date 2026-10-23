//! Complete allocation and script scheduling over the owned projectile pool.
#![forbid(unsafe_code)]
use super::script::EprojScript;
use super::{Execution, SlotField as F};

pub(super) fn set_enabled(ctx: &mut Execution<'_>, enabled: bool) {
    let flags = ctx.state().flags;
    ctx.state_mut().flags = if enabled {
        flags | 0x8000
    } else {
        flags & !0x8000
    };
}
pub(super) fn clear(ctx: &mut Execution<'_>) {
    for slot in ctx.state_mut().slots.iter_mut().rev() {
        slot.definition = 0;
    }
}
fn allocate(ctx: &mut Execution<'_>, definition: u16, graphics: u16) -> Option<u16> {
    let index = ctx
        .state()
        .slots
        .iter()
        .rposition(|slot| slot.definition == 0)?;
    ctx.state_mut().slots[index].graphics = graphics;
    ctx.state_mut().slots[index].definition = definition;
    let record = ctx.record(0x86, definition);
    // Allocation proves this is an ordinary pool slot. Each checked definition
    // read precedes its corresponding write, preserving failure prefixes.
    let slot = &mut ctx.state_mut().slots[index];
    slot.pre_action = record.word(core::mem::offset_of!(crate::types::EprojDef, pre_instr_ptr));
    slot.program = record.word(core::mem::offset_of!(crate::types::EprojDef, instr_list));
    slot.radius = record.word(core::mem::offset_of!(crate::types::EprojDef, radius));
    slot.properties = record.word(core::mem::offset_of!(crate::types::EprojDef, properties));
    slot.delay = 1;
    slot.spritemap = 0x8000;
    slot.behavior_a = 0;
    slot.behavior_b = 0;
    slot.counter = 0;
    slot.x.fraction = 0;
    slot.y.fraction = 0;
    slot.behavior_c = 0;
    let handle = (index * 2) as u16;
    let init = 0x860000
        | u32::from(record.word(core::mem::offset_of!(crate::types::EprojDef, init_code_ptr)));
    super::setup::dispatch(init, handle, ctx);
    crate::verify_probe::hit("eproj.spawn.initialized");
    Some(handle)
}
pub(super) fn spawn_room(ctx: &mut Execution<'_>, definition: u16, parameter: u16) -> u16 {
    ctx.state_mut().spawn.primary = parameter;
    allocate(ctx, definition, 0).unwrap_or_else(|| ctx.slot(F::Definition, 0))
}
pub(super) fn spawn_from_enemy(
    ctx: &mut Execution<'_>,
    parameter: u16,
    enemy: u16,
    definition: u16,
) -> u16 {
    ctx.state_mut().spawn.primary = parameter;
    let graphics = ctx.enemy_tile(enemy) | ctx.enemy_palette(enemy);
    allocate(ctx, definition, graphics).unwrap_or(graphics)
}
pub(super) fn process(ctx: &mut Execution<'_>, handle: u16) {
    let pre_action = 0x860000 | u32::from(ctx.slot(F::PreAction, (handle >> 1) as isize));
    super::pre_actions::dispatch(pre_action, handle, ctx);
    crate::verify_probe::hit("eproj.pre.executed");
    let selected = ctx.state().selected;
    let index = (ctx.state().selected >> 1) as isize;
    if ctx.decrement(F::Delay, index) != 1 {
        return;
    }
    // One captured allocation supplies every command operand until this frame ends.
    let image = ctx.image();
    let mut script = EprojScript::new(image, ctx.slot(F::Program, index));
    while script.word(0) & 0x8000 != 0 {
        let instruction = 0x860000 | u32::from(script.word(0));
        script.seek(script.offset(2).unwrap());
        let Some(next) = super::commands::dispatch(instruction, selected, &script, ctx) else {
            return;
        };
        script.seek(next);
    }
    let index = (selected >> 1) as isize;
    ctx.set_slot(F::Delay, index, script.word(0));
    ctx.set_slot(F::Spritemap, index, script.word(2));
    ctx.set_slot(F::Program, index, script.bank_address().wrapping_add(4));
    crate::verify_probe::hit("eproj.frame.saved");
}
pub(super) fn advance(ctx: &mut Execution<'_>) {
    if ctx.state().flags & 0x8000 == 0 {
        return;
    }
    let mut next = Some(34u16);
    while let Some(mut handle) = next {
        ctx.state_mut().selected = handle;
        if ctx.slot(F::Definition, (handle >> 1) as isize) != 0 {
            process(ctx, handle);
            handle = ctx.state().selected;
        }
        next = handle.checked_sub(2);
    }
}
