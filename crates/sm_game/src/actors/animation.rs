//! Common animation rules. Captured slots and script allocations survive callbacks.
#![forbid(unsafe_code)]
use crate::actors::{
    access::{AccessMode::LittleEndian as LE, ActorKey},
    content::ActorRoutine,
    effects::{ActorAssertion, ActorHost},
};
use crate::{
    enemy_projectiles::host::Host,
    enemy_script::{EnemyScript, bank_base},
};

pub fn advance_animation<H: ActorHost + Host>(h: &mut H) {
    let key = ActorKey::from_legacy(h.session().actors.controls.selected);
    if h.resolve_actor(key, LE).ai_handlers & 4 != 0 {
        return;
    }
    let timer = h.resolve_actor(key, LE).instruction_delay;
    let decremented = h.resolve_actor(key, LE).instruction_delay.wrapping_sub(1);
    h.edit_actor(key, LE, |a| a.instruction_delay = decremented);
    if timer != 1 {
        let properties = h.resolve_actor(key, LE).extra_properties & !0x8000;
        h.edit_actor(key, LE, |a| a.extra_properties = properties);
        return;
    }
    if h.resolve_actor(key, LE).instruction & 0x8000 == 0 {
        h.actor_assertion(ActorAssertion::AnimationInstructionHighBit);
    }
    let base = bank_base(h.resolve_actor(key, LE).bank);
    let image = h.actor_script_image();
    let mut pc = EnemyScript::at(
        image.clone(),
        usize::try_from(base + isize::try_from(h.resolve_actor(key, LE).instruction).unwrap())
            .expect("enemy script outside ROM"),
    );
    while pc.word(0) & 0x8000 != 0 {
        let address = (u32::from(h.resolve_actor(key, LE).bank) << 16) | u32::from(pc.word(0));
        let current = ActorKey::from_legacy(h.session().actors.controls.selected);
        #[cfg(sm_verify_dll)]
        h.actor_reference_opcode(15, address);
        let next = dispatch_instruction(h, address, current, pc.offset(1));
        #[cfg(sm_verify_dll)]
        h.actor_reference_opcode(16, address);
        let Some(next) = next.resolve(&image, base) else {
            return;
        };
        pc = next;
    }
    let delay = pc.word(0);
    h.edit_actor(key, LE, |a| a.instruction_delay = delay);
    let map = pc.word(2);
    h.edit_actor(key, LE, |a| a.spritemap = map);
    let instruction = pc.advance(4).offset_from(base) as u16;
    h.edit_actor(key, LE, |a| a.instruction = instruction);
    let properties = h.resolve_actor(key, LE).extra_properties | 0x8000;
    h.edit_actor(key, LE, |a| a.extra_properties = properties);
    h.probe("actors.animation.frame.committed");
}

pub fn dispatch_instruction<H: ActorHost + Host>(
    h: &mut H,
    address: u32,
    key: ActorKey,
    pc: EnemyScript,
) -> EnemyScript {
    match address {
        10846332 | 11174012 => {
            let value = h.resolve_actor(key, LE).properties | 0x200;
            h.edit_actor(key, LE, |a| a.properties = value);
            EnemyScript::Stop
        }
        10649872 | 10846480 | 10977552 | 11043088 | 11174160 | 11698448 | 11763984 => {
            let timer = h.resolve_actor(key, LE).timer;
            let next = h.resolve_actor(key, LE).timer.wrapping_sub(1);
            h.edit_actor(key, LE, |a| a.timer = next);
            if timer == 1 {
                pc.offset(1)
            } else {
                EnemyScript::jump(pc.word(0))
            }
        }
        10649891 | 10846499 | 10977571 | 11043107 | 11174179 | 11698467 | 11764003 => {
            let timer = pc.word(0);
            h.edit_actor(key, LE, |a| a.timer = timer);
            pc.offset(1)
        }
        10649903 | 10715439 | 10780975 | 10846511 | 10912047 | 10977583 | 11043119 | 11108655
        | 11174191 | 11698479 | 11764015 => {
            let base = bank_base(h.resolve_actor(key, LE).bank);
            let instruction = pc.advance(-2).offset_from(base) as u16;
            h.edit_actor(key, LE, |a| a.instruction = instruction);
            EnemyScript::Stop
        }
        10846522 | 11174202 | 11698490 => {
            let base = bank_base(h.resolve_actor(key, LE).bank);
            let delay = pc.word(0);
            h.edit_actor(key, LE, |a| a.instruction_delay = delay);
            let instruction = pc.advance(2).offset_from(base) as u16;
            h.edit_actor(key, LE, |a| a.instruction = instruction);
            EnemyScript::Stop
        }
        11174219 => {
            let tail = h.vram_write_queue_tail();
            let cursor = h.vram_write_queue_tail();
            let value = pc.word(0);
            h.set_vram_queue_size(cursor, value);
            let value = pc.word(2);
            h.set_vram_queue_source(cursor, value);
            let value = pc.word(3);
            h.write_collection_word(
                crate::actors::access::CollectionWindow::VramQueue,
                usize::from(cursor) + 3,
                value,
                crate::actors::access::CollectionStore::SliceLe,
            );
            let value = pc.word(5);
            h.set_vram_queue_destination(cursor, value);
            h.set_vram_write_queue_tail(tail.wrapping_add(7));
            pc.advance(7)
        }
        10649971 | 10715507 | 11043187 => {
            let value = h.resolve_actor(key, LE).properties | 0x800;
            h.edit_actor(key, LE, |a| a.properties = value);
            pc
        }
        10649981 | 10715517 | 11043197 => {
            let value = h.resolve_actor(key, LE).properties & !0x800;
            h.edit_actor(key, LE, |a| a.properties = value);
            pc
        }
        10649837 | 10715373 | 10780909 | 10846445 | 10911981 | 10977517 | 11043053 | 11174125
        | 11698413 | 11763949 => EnemyScript::jump(pc.word(0)),
        _ => h.actor_instruction(ActorRoutine::from_legacy(address), key, pc),
    }
}
