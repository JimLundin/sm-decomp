//! Selection and update retain captured actor keys across ordered foreign effects.
#![forbid(unsafe_code)]
use super::{
    access::{AccessMode, ActorAccess, ActorCollections, ActorKey, CollectionWindow},
    api::DeathKind,
    combat, contacts,
    content::{ActorDefinitionKey, ActorRoutine, DefinitionRoutine},
    drawing,
    effects::ActorHost,
};
use crate::game::{
    enemy_projectiles::host::Host as EprojHost, room_mechanisms::host::Effects as MechanismEffects,
    sprite_objects,
};
use sm_game::player_projectiles::access::Store;

fn frozen(h: &impl ActorAccess) -> bool {
    h.session().actors.diagnostics.freeze | h.inputs().frozen != 0
}
pub(in crate::game) fn select(h: &mut (impl ActorAccess + ActorCollections)) {
    let controls = &mut h.session_mut().actors.controls;
    controls.selection_visits = controls.selection_visits.wrapping_add(1);
    controls.selected = 0;
    h.session_mut().actors.schedule.active_write_position = 0;
    h.session_mut().actors.schedule.interactive_write_position = 0;
    let all = h.session().actors.controls.process_all != 0;
    loop {
        let index = h.session().actors.controls.selected;
        let key = ActorKey::from_legacy(index);
        let definition = h.resolve_actor(key, AccessMode::Native).definition;
        if definition != 0 && definition != 0xdaff {
            let properties = h.resolve_actor(key, AccessMode::Native).properties;
            if properties & 0x200 != 0 {
                h.edit_actor(key, AccessMode::Native, |actor| actor.definition = 0);
                h.session_mut().actor_identity.retire(index / 64);
            } else {
                let selected = {
                    let actor = h.resolve_actor(key, AccessMode::Native);
                    all || properties & 0x800 != 0
                        || actor.ai_handlers & 4 != 0
                        || (actor
                            .radius
                            .x
                            .wrapping_add(actor.x.whole)
                            .wrapping_sub(h.inputs().camera_x) as i16)
                            >= 0
                            && (actor
                                .radius
                                .x
                                .wrapping_add(h.inputs().camera_x)
                                .wrapping_add(256)
                                .wrapping_sub(actor.x.whole) as i16)
                                >= 0
                            && (actor
                                .y
                                .whole
                                .wrapping_add(8)
                                .wrapping_sub(h.inputs().camera_y)
                                as i16)
                                >= 0
                            && (h
                                .inputs()
                                .camera_y
                                .wrapping_add(248)
                                .wrapping_sub(actor.y.whole) as i16)
                                >= 0
                };
                if selected {
                    let active = h.session().actors.schedule.active_write_position;
                    let value = h.session().actors.controls.selected;
                    set_active(
                        h,
                        usize::from(active >> 1),
                        value,
                        if all {
                            AccessMode::LittleEndian
                        } else {
                            AccessMode::Native
                        },
                    );
                    if all {
                        set_interactive(
                            h,
                            usize::from(active >> 1),
                            index,
                            AccessMode::LittleEndian,
                        );
                    }
                    h.session_mut().actors.schedule.active_write_position = active.wrapping_add(2);
                    h.probe("actors.select.active");
                    if h.resolve_actor(key, AccessMode::Native).properties & 0x400 == 0 {
                        let interactive = h.session().actors.schedule.interactive_write_position;
                        set_interactive(
                            h,
                            usize::from(interactive >> 1),
                            index,
                            AccessMode::Native,
                        );
                        h.session_mut().actors.schedule.interactive_write_position =
                            interactive.wrapping_add(2);
                    }
                }
            }
        }
        h.session_mut().actors.controls.selected =
            h.session().actors.controls.selected.wrapping_add(64);
        if h.session().actors.controls.selected.wrapping_sub(2048) & 0x8000 == 0 {
            break;
        }
    }
    let active = usize::from(h.session().actors.schedule.active_write_position >> 1);
    set_active(h, active, 0xffff, AccessMode::Native);
    let interactive = usize::from(h.session().actors.schedule.interactive_write_position >> 1);
    set_interactive(h, interactive, 0xffff, AccessMode::Native);
}
pub(in crate::game) fn shake_active_enemies(h: &mut (impl ActorAccess + ActorCollections)) {
    let mut cursor = 0usize;
    loop {
        let index = active(h, cursor);
        if index == 0xffff {
            break;
        }
        h.edit_actor(ActorKey::from_legacy(index), AccessMode::Native, |actor| {
            actor.shake = 2
        });
        cursor += 1;
    }
}
pub(in crate::game) fn update_frozen_enemy(h: &mut impl ActorAccess) {
    let key = combat::current(h);
    h.edit_actor(key, AccessMode::Native, |actor| actor.flash = 0);
    let timer = h.resolve_actor(key, AccessMode::Native).frozen;
    if timer == 0 || {
        h.edit_actor(key, AccessMode::Native, |actor| {
            actor.frozen = timer.wrapping_sub(1)
        });
        h.session().inventory.beams.active & 2 == 0
    } {
        h.edit_actor(key, AccessMode::Native, |actor| {
            actor.ai_handlers &= !4;
            actor.frozen = actor.ai_handlers;
        });
    }
}
pub(in crate::game) fn advance_selected<H: ActorHost + EprojHost + MechanismEffects>(h: &mut H) {
    if h.session().actors.controls.first_free != 0 {
        let shake = h.session().actors.controls.shake_request;
        if shake != 0xffff {
            h.edit_actor(ActorKey::from_legacy(shake), AccessMode::Native, |actor| {
                actor.shake = 64
            });
            h.session_mut().actors.controls.shake_request = 0xffff;
        }
        let mut cursor = 0usize;
        loop {
            let index = active(h, cursor);
            if index == 0xffff {
                break;
            }
            h.session_mut().actors.controls.selected = index;
            let key = ActorKey::from_legacy(index);
            process_actor(h, key);
            let flash = h.resolve_actor(key, AccessMode::Native).flash;
            if flash != 0 && !frozen(h) {
                let flash = flash.wrapping_sub(1);
                h.edit_actor(key, AccessMode::Native, |actor| {
                    actor.flash = flash;
                    if flash.wrapping_sub(8) & 0x8000 != 0 {
                        actor.ai_handlers &= !2;
                    }
                });
            }
            cursor += 1;
        }
    }
    sprite_objects::update(h);
    h.session_mut().actors.controls.phase_counter =
        h.session().actors.controls.phase_counter.wrapping_add(1);
    h.session_mut().actors.schedule.colliding = [0xffff; 4];
}
fn process_actor<H: ActorHost + EprojHost + MechanismEffects>(h: &mut H, key: ActorKey) {
    let run_ai = if h.resolve_actor(key, AccessMode::Native).properties & 0x400 == 0 {
        let invincibility = h.resolve_actor(key, AccessMode::Native).invincibility;
        if invincibility != 0 {
            h.edit_actor(key, AccessMode::Native, |actor| {
                actor.invincibility = invincibility.wrapping_sub(1)
            });
            true
        } else if h.session().actors.diagnostics.sprite_contacts_disabled == 0 {
            if !frozen(h) {
                contacts::process_contacts(h);
                if h.resolve_actor(key, AccessMode::Native).definition == 0 {
                    return;
                }
            }
            h.resolve_actor(key, AccessMode::Native).extra_properties & 1 == 0
        } else {
            true
        }
    } else {
        true
    };
    if run_ai {
        h.session_mut().actors.schedule.compatibility_179e[2] = 0;
        let ran_ai = if !frozen(h) {
            let actor = h.resolve_actor(key, AccessMode::Native);
            let handler = if actor.ai_handlers == 0 {
                0
            } else {
                actor.ai_handlers.trailing_zeros() as u16 + 1
            };
            let bank = actor.bank;
            let definition = actor.definition.wrapping_add(2 * handler);
            drop(actor);
            let capture = h.capture_actor_definition(ActorDefinitionKey(definition));
            let routine = h.definition_routine_offset(&capture, DefinitionRoutine::Main);
            h.actor_ai(ActorRoutine(u32::from(bank) << 16 | u32::from(routine)));
            h.probe("actors.advance.ai.completed");
            true
        } else {
            let definition = h.resolve_actor(key, AccessMode::Native).definition;
            let capture = h.capture_actor_definition(ActorDefinitionKey(definition));
            let routine = h.definition_routine_offset(&capture, DefinitionRoutine::TimeFrozen);
            if routine != 0 {
                let bank = h.resolve_actor(key, AccessMode::Native).bank;
                h.actor_ai(ActorRoutine(u32::from(bank) << 16 | u32::from(routine)));
                true
            } else {
                false
            }
        };
        if ran_ai && !frozen(h) {
            h.edit_actor(key, AccessMode::Native, |actor| {
                actor.frame = actor.frame.wrapping_add(1)
            });
            if h.resolve_actor(key, AccessMode::Native).properties & 0x2000 != 0 {
                h.session_mut().room.enemy_processing_stage = 2;
                super::animation::advance_animation(h);
            }
        }
    }
    if h.resolve_actor(key, AccessMode::Native).extra_properties & 1 != 0
        && (h.resolve_actor(key, AccessMode::Native).flash == 1
            || h.resolve_actor(key, AccessMode::Native).frozen == 1)
    {
        combat::cause(h, 0);
        combat::die(h, DeathKind::Normal { animation: 0 });
    }
    if (h.resolve_actor(key, AccessMode::Native).extra_properties & 4 != 0
        || !super::queries::current_normal_sprite_outside_screen(h))
        && h.resolve_actor(key, AccessMode::Native).properties & 0x300 == 0
        && h.session().actors.schedule.compatibility_179e[2] & 1 == 0
    {
        drawing::queue_current_actor(h);
    }
}

pub(super) fn active(h: &(impl ActorAccess + ActorCollections), index: usize) -> u16 {
    h.collection_word(
        CollectionWindow::Active,
        index.checked_mul(2).unwrap(),
        AccessMode::LittleEndian,
    )
}
pub(super) fn interactive(h: &(impl ActorAccess + ActorCollections), index: usize) -> u16 {
    h.collection_word(
        CollectionWindow::Interactive,
        index.checked_mul(2).unwrap(),
        AccessMode::LittleEndian,
    )
}
fn set_active(h: &mut impl ActorCollections, index: usize, value: u16, mode: AccessMode) {
    h.write_collection_word(
        CollectionWindow::Active,
        index.checked_mul(2).unwrap(),
        value,
        if mode == AccessMode::Native {
            Store::SliceNative
        } else {
            Store::PrefixLe
        },
    );
}
fn set_interactive(h: &mut impl ActorCollections, index: usize, value: u16, mode: AccessMode) {
    h.write_collection_word(
        CollectionWindow::Interactive,
        index.checked_mul(2).unwrap(),
        value,
        if mode == AccessMode::Native {
            Store::SliceNative
        } else {
            Store::PrefixLe
        },
    );
}
