//! Complete allocation and script scheduling over the owned projectile pool.
use super::access::enemy;
use super::host::{DefinitionField, Host, LoadedRecord};
use crate::actors::access::{AccessMode::LittleEndian as Le, ActorKey as Key};
use crate::enemy_script::EnemyScript;

pub fn set_enabled(h: &mut impl Host, enabled: bool) {
    let flags = h.session().enemy_projectiles.flags;
    h.session_mut().enemy_projectiles.flags = if enabled {
        flags | 0x8000
    } else {
        flags & !0x8000
    };
}
pub fn clear(h: &mut impl Host) {
    for slot in h.session_mut().enemy_projectiles.slots.iter_mut().rev() {
        slot.definition = 0;
    }
}
fn allocate(h: &mut impl Host, definition: u16, graphics: u16) -> Option<u16> {
    let index = h
        .session()
        .enemy_projectiles
        .slots
        .iter()
        .rposition(|slot| slot.definition == 0)?;
    h.session_mut().enemy_projectiles.slots[index].graphics = graphics;
    h.session_mut().enemy_projectiles.slots[index].definition = definition;
    let record = h.loaded_record(0x86, definition);
    // Allocation proves this is an ordinary pool slot. Each checked definition
    // read precedes its corresponding write, preserving failure prefixes.
    let slot = &mut h.session_mut().enemy_projectiles.slots[index];
    slot.pre_action = record.definition_word(DefinitionField::PreAction);
    slot.program = record.definition_word(DefinitionField::Program);
    slot.radius = record.definition_word(DefinitionField::Radius);
    slot.properties = record.definition_word(DefinitionField::Properties);
    slot.delay = 1;
    slot.spritemap = 0x8000;
    slot.behavior_a = 0;
    slot.behavior_b = 0;
    slot.counter = 0;
    slot.x.fraction = 0;
    slot.y.fraction = 0;
    slot.behavior_c = 0;
    let handle = (index * 2) as u16;
    let init = 0x860000 | u32::from(record.definition_word(DefinitionField::Initialize));
    super::setup::dispatch(init, handle, h);
    h.probe("eproj.spawn.initialized");
    Some(handle)
}
/// Spawn `definition` with the room's graphics on a fresh lease of `game`.
pub fn spawn_room_leased<H>(definition: u16, parameter: u16, game: &mut H) -> u16
where
    H: crate::runtime_access::RuntimeAccess + ?Sized,
{
    spawn_room(&mut game.runtime(), definition, parameter)
}
pub fn spawn_room(h: &mut impl Host, definition: u16, parameter: u16) -> u16 {
    h.session_mut().enemy_projectiles.spawn.primary = parameter;
    allocate(h, definition, 0).unwrap_or_else(|| enemy!(h, 0, definition))
}
pub fn spawn_from_enemy(h: &mut impl Host, parameter: u16, enemy: u16, definition: u16) -> u16 {
    h.session_mut().enemy_projectiles.spawn.primary = parameter;
    let graphics = h.resolve_actor(Key::from_legacy(enemy), Le).tile
        | h.resolve_actor(Key::from_legacy(enemy), Le).palette;
    allocate(h, definition, graphics).unwrap_or(graphics)
}
pub fn process(h: &mut impl Host, handle: u16) {
    let pre_action = 0x860000 | u32::from(enemy!(h, (handle >> 1) as isize, pre_action));
    super::pre_actions::dispatch(pre_action, handle, h);
    h.probe("eproj.pre.executed");
    let selected = h.session().enemy_projectiles.selected;
    let index = (h.session().enemy_projectiles.selected >> 1) as isize;
    let old = enemy!(h, index, delay);
    enemy!(h, index, delay = old.wrapping_sub(1));
    if old != 1 {
        return;
    }
    // One captured allocation supplies every command operand until this frame ends.
    let program = enemy!(h, index, program);
    let image = h.script_image();
    let mut script = EnemyScript::at(image.clone(), 0x28000 + usize::from(program));
    while script.word(0) & 0x8000 != 0 {
        let instruction = 0x860000 | u32::from(script.word(0));
        script = script.advance(2);
        let next = super::commands::dispatch(instruction, selected, &script, h);
        let Some(next) = next.resolve(&image, 0x28000) else {
            return;
        };
        script = next;
    }
    let index = (selected >> 1) as isize;
    enemy!(h, index, delay = script.word(0));
    enemy!(h, index, spritemap = script.word(2));
    enemy!(
        h,
        index,
        program = script.bank_address(0x28000).wrapping_add(4)
    );
    h.probe("eproj.frame.saved");
}
pub fn advance(h: &mut impl Host) {
    if h.session().enemy_projectiles.flags & 0x8000 == 0 {
        return;
    }
    let mut next = Some(34u16);
    while let Some(mut handle) = next {
        h.session_mut().enemy_projectiles.selected = handle;
        if enemy!(h, (handle >> 1) as isize, definition) != 0 {
            process(h, handle);
            handle = h.session().enemy_projectiles.selected;
        }
        next = handle.checked_sub(2);
    }
}
