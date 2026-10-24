//! Ordered contact scans with owned actors and separately captured live content.
#![forbid(unsafe_code)]
use super::{
    access::{AccessMode, ActorAccess, ActorKey},
    api::{GrappleContact, GrappleReaction},
    combat::current,
    content::{
        ActorCallbackTarget, ActorCollisionCapture, ActorCollisionWord, ActorCountWidth,
        ActorDefinitionKey, ActorPartTraversal, ActorPartWord, ActorRoutine, DefinitionRoutine,
    },
    effects::ActorHost,
};
use crate::game::{
    constants::bank_a0::{addr_kExtendedSpritemap_Nothing_A0, addr_kSpritemap_Nothing_A0},
    contact_rules::{Bounds, HitboxEdge, axis, is_shot},
    enemy_projectiles::host::Host as EprojHost,
    room_mechanisms::host::Effects as MechanismEffects,
};
fn ignored(ai: u16) -> bool {
    ai == 0x804b || ai == 0x804c
}
fn mark_projectile(h: &mut impl ActorAccess, index: usize) {
    h.session_mut().projectiles.shots[index].direction |= 0x10;
}
fn hit<H: ActorHost>(
    h: &mut H,
    bounds: &Bounds,
    x: u16,
    y: u16,
    source: &ActorCollisionCapture,
    index: usize,
    touch: bool,
) -> bool {
    bounds.hits(
        x,
        y,
        |edge| {
            h.actor_collision_word(
                source,
                index,
                match edge {
                    HitboxEdge::Left => ActorCollisionWord::Left,
                    HitboxEdge::Right => ActorCollisionWord::Right,
                    HitboxEdge::Top => ActorCollisionWord::Top,
                    HitboxEdge::Bottom => ActorCollisionWord::Bottom,
                },
            )
        },
        touch,
    )
}
pub(in crate::game) fn process_contacts<H: ActorHost + EprojHost + MechanismEffects>(h: &mut H) {
    if h.resolve_actor(current(h), AccessMode::Native)
        .extra_properties
        & 4
        != 0
    {
        projectile_contacts(h, Attack::Shot, true);
        projectile_contacts(h, Attack::Bomb, true);
        touch_hitboxes(h);
        h.probe("actors.contact.extended.completed");
    } else {
        projectile_contacts(h, Attack::Shot, false);
        projectile_contacts(h, Attack::Bomb, false);
        touch_body(h);
        h.probe("actors.contact.normal.completed");
    }
}
#[cfg(sm_verify_dll)]
pub(super) fn movement_verification_body<H: ActorHost>(h: &mut H) {
    touch_body(h);
}
fn touch_body<H: ActorHost>(h: &mut H) {
    let key = current(h);
    h.session_mut().room.enemy_processing_stage = 9;
    #[cfg(sm_verify_dll)]
    super::movement_verification::body_stage(h);
    if h.resolve_actor(key, AccessMode::Native).spritemap == 0 {
        return;
    }
    if h.session().player.contact.contact_attack != 0 {
        h.session_mut().player.invincibility = 0;
    } else if h.session().player.invincibility != 0 {
        if h.resolve_actor(key, AccessMode::Native).definition != 0xdaff {
            return;
        }
        let selected = current(h);
        let flag = h.resolve_spawn(selected, AccessMode::Native).contact_mode;
        if flag == 0 || flag == 8 {
            return;
        }
    }
    let definition = h.resolve_actor(key, AccessMode::Native).definition;
    let definition = h.capture_actor_definition(ActorDefinitionKey(definition));
    if ignored(h.definition_routine_offset(&definition, DefinitionRoutine::Touch)) {
        return;
    }
    let overlaps = {
        let actor = h.resolve_actor(key, AccessMode::Native);
        axis(
            h.session().player.x,
            actor.x.whole,
            h.inputs().player_x_radius,
            actor.radius.x,
        ) && axis(
            h.session().player.y,
            actor.y.whole,
            h.session().player.contact.radius,
            actor.radius.y,
        ) && (actor.definition == 0xdaff || actor.frozen == 0)
    };
    if overlaps {
        let bank = h.resolve_actor(key, AccessMode::Native).bank;
        let routine = h.definition_routine_offset(&definition, DefinitionRoutine::Touch);
        h.actor_ai(ActorRoutine(u32::from(bank) << 16 | u32::from(routine)));
    }
}
fn touch_hitboxes<H: ActorHost>(h: &mut H) {
    let key = current(h);
    h.session_mut().room.enemy_processing_stage = 6;
    if h.resolve_actor(key, AccessMode::Native).spritemap == 0 {
        return;
    }
    let definition = h.resolve_actor(key, AccessMode::Native).definition;
    let definition = h.capture_actor_definition(ActorDefinitionKey(definition));
    if ignored(h.definition_routine_offset(&definition, DefinitionRoutine::Touch)) {
        return;
    }
    if h.session().player.contact.contact_attack != 0 {
        h.session_mut().player.invincibility = 0;
    } else if h.session().player.invincibility != 0 {
        return;
    }
    if h.resolve_actor(key, AccessMode::Native).spritemap & 0x8000 == 0 {
        return;
    }
    let bounds = Bounds::new(
        h.session().player.x,
        h.session().player.y,
        h.inputs().player_x_radius,
        h.session().player.contact.radius,
    );
    let pose = h.capture_actor_pose(key);
    let mut remaining = i32::from(h.actor_pose_count(&pose, ActorCountWidth::Byte));
    let mut address = h
        .resolve_actor(key, AccessMode::Native)
        .spritemap
        .wrapping_add(2);
    let mut ordinal = 0usize;
    loop {
        let bank = h.resolve_actor(key, AccessMode::Native).bank;
        let part = h.capture_actor_part_at(
            &pose,
            ordinal,
            ActorPartTraversal::WrappingReacquire { bank, address },
        );
        let x = h
            .actor_part_word(&part, ActorPartWord::X)
            .wrapping_add(h.resolve_actor(key, AccessMode::Native).x.whole);
        let y = h
            .actor_part_word(&part, ActorPartWord::Y)
            .wrapping_add(h.resolve_actor(key, AccessMode::Native).y.whole);
        let bank = h.resolve_actor(key, AccessMode::Native).bank;
        let hitboxes = h.capture_actor_collision_at(&part, bank);
        let count = usize::from(h.actor_collision_count(&hitboxes));
        for index in 0..count {
            if hit(h, &bounds, x, y, &hitboxes, index, true) {
                let bank = h.resolve_actor(key, AccessMode::Native).bank;
                let target = h.actor_collision_target(&hitboxes, index, ActorCollisionWord::Touch);
                h.actor_hitbox_touch(target.bind(bank));
                return;
            }
        }
        address = address.wrapping_add(8);
        ordinal = ordinal.wrapping_add(1);
        remaining = remaining.wrapping_sub(1);
        if remaining == 0 {
            return;
        }
    }
}
#[derive(Clone, Copy, PartialEq)]
enum Attack {
    Shot,
    Bomb,
}
impl Attack {
    fn eligible(self, h: &impl ActorAccess, index: usize, extended: bool) -> bool {
        let shot = &h.session().projectiles.shots[index];
        let kind = shot.kind;
        if self == Self::Shot {
            is_shot(kind)
        } else if extended {
            shot.x != 0 && kind != 0 && kind & 0xf00 == 0x500 && shot.variable == 0
        } else {
            kind != 0 && shot.variable == 0 && (kind & 0xf00 == 0x500 || kind & 0x8000 != 0)
        }
    }
}
fn projectile_contacts<H: ActorHost + EprojHost>(h: &mut H, attack: Attack, extended: bool) {
    let key = current(h);
    let bomb = attack == Attack::Bomb;
    h.session_mut().room.enemy_processing_stage = if extended {
        if bomb { 4 } else { 3 }
    } else if bomb {
        8
    } else {
        7
    };
    if extended {
        if bomb {
            let actor = h.resolve_actor(key, AccessMode::Native);
            if actor.spritemap == 0 || actor.properties & 0x400 != 0 || actor.invincibility != 0 {
                return;
            }
        } else {
            let actor = h.resolve_actor(key, AccessMode::Native);
            if h.session().projectiles.count == 0
                || actor.spritemap == 0
                || actor.spritemap == addr_kExtendedSpritemap_Nothing_A0 as u16
            {
                return;
            }
        }
        let definition = h.resolve_actor(key, AccessMode::Native).definition;
        let definition = h.capture_actor_definition(ActorDefinitionKey(definition));
        if ignored(h.definition_routine_offset(&definition, DefinitionRoutine::Shot)) {
            h.probe("actors.contact.shot.ignored");
            return;
        }
        if bomb {
            if h.session().projectiles.bomb_count == 0 {
                return;
            }
        } else {
            let actor = h.resolve_actor(key, AccessMode::Native);
            if actor.properties & 0x400 != 0
                || actor.invincibility != 0
                || actor.definition == 0xdaff
            {
                return;
            }
        }
    } else if bomb {
        let actor = h.resolve_actor(key, AccessMode::Native);
        if h.session().projectiles.bomb_count == 0
            || actor.spritemap == 0
            || actor.invincibility != 0
            || actor.definition == 0xdaff
        {
            return;
        }
    } else {
        let actor = h.resolve_actor(key, AccessMode::Native);
        if h.session().projectiles.count == 0
            || actor.spritemap == 0
            || actor.spritemap == addr_kSpritemap_Nothing_A0 as u16
            || actor.properties & 0x400 != 0
            || actor.definition == 0xdaff
            || actor.invincibility != 0
        {
            return;
        }
    }
    for index in if bomb { 5..10 } else { 0..5 } {
        if !attack.eligible(h, index, extended) {
            continue;
        }
        let hitbox = if extended {
            let found = projectile_hitbox(h, key, index, bomb);
            if found.is_none() {
                continue;
            }
            found
        } else {
            let actor = h.resolve_actor(key, AccessMode::Native);
            let shot = &h.session().projectiles.shots[index];
            if !axis(shot.x, actor.x.whole, shot.x_radius, actor.radius.x)
                || !axis(shot.y, actor.y.whole, shot.y_radius, actor.radius.y)
            {
                continue;
            }
            None
        };
        if !bomb && h.session().projectiles.shots[index].kind & 0xf00 == 0x200 {
            h.session_mut().feedback.earthquake_timer = 30;
            h.session_mut().feedback.earthquake_kind = 18;
        }
        if bomb && !extended {
            h.session_mut().room.collision_detection_index = index as u16;
        }
        if bomb
            || h.resolve_actor(key, AccessMode::Native).properties & 0x1000 != 0
            || h.session().projectiles.shots[index].kind & 8 == 0
        {
            mark_projectile(h, index);
        }
        if !bomb || extended {
            h.session_mut().room.collision_detection_index = index as u16;
        }
        if let Some(function) = hitbox {
            let bank = h.resolve_actor(key, AccessMode::Native).bank;
            h.actor_hitbox_shot(function.bind(bank), (index * 2) as u16);
        } else {
            let bank = h.resolve_actor(key, AccessMode::Native).bank;
            let definition = h.resolve_actor(key, AccessMode::Native).definition;
            let definition = h.capture_actor_definition(ActorDefinitionKey(definition));
            let routine = h.definition_routine_offset(&definition, DefinitionRoutine::Shot);
            h.actor_ai(ActorRoutine(u32::from(bank) << 16 | u32::from(routine)));
        }
        return;
    }
}
fn projectile_hitbox<H: ActorHost + EprojHost>(
    h: &mut H,
    key: ActorKey,
    index: usize,
    bomb: bool,
) -> Option<ActorCallbackTarget> {
    if h.resolve_actor(key, AccessMode::Native).spritemap & 0x8000 == 0 {
        h.unreachable();
    }
    let pose = h.capture_actor_pose(key);
    let count = h.actor_pose_count(
        &pose,
        if bomb {
            ActorCountWidth::NativeWord
        } else {
            ActorCountWidth::Byte
        },
    );
    for entry in 0..usize::from(count) {
        let part = h.capture_actor_part_at(&pose, entry, ActorPartTraversal::CapturedLinear);
        let x = h
            .actor_part_word(&part, ActorPartWord::X)
            .wrapping_add(h.resolve_actor(key, AccessMode::Native).x.whole);
        let y = h
            .actor_part_word(&part, ActorPartWord::Y)
            .wrapping_add(h.resolve_actor(key, AccessMode::Native).y.whole);
        let bank = h.resolve_actor(key, AccessMode::Native).bank;
        let hitboxes = h.capture_actor_collision_at(&part, bank);
        let shot = &h.session().projectiles.shots[index];
        let bounds = Bounds::new(shot.x, shot.y, shot.x_radius, shot.y_radius);
        let count = usize::from(h.actor_collision_count(&hitboxes));
        for hitbox in 0..count {
            if hit(h, &bounds, x, y, &hitboxes, hitbox, false) {
                return Some(h.actor_collision_target(&hitboxes, hitbox, ActorCollisionWord::Shot));
            }
        }
    }
    None
}
pub(in crate::game) fn grapple_contact<H: ActorHost + EprojHost + MechanismEffects>(
    h: &mut H,
) -> Option<GrappleContact> {
    h.player_control(0xd);
    h.session_mut().room.collision_detection_index = 0;
    let mut index = 0usize;
    loop {
        h.session_mut().actors.controls.selected = super::scheduling::interactive(h, index);
        if h.session().actors.controls.selected == 0xffff {
            return None;
        }
        let key = current(h);
        let overlaps = {
            let actor = h.resolve_actor(key, AccessMode::Native);
            actor.invincibility == 0
                && axis(
                    actor.x.whole,
                    h.session().player.grapple_endpoint.x,
                    actor.radius.x,
                    8,
                )
                && axis(
                    actor.y.whole,
                    h.session().player.grapple_endpoint.y,
                    actor.radius.y,
                    8,
                )
        };
        if overlaps {
            h.edit_actor(key, AccessMode::Native, |actor| actor.ai_handlers = 1);
            let address = h.resolve_actor(key, AccessMode::Native).definition;
            let definition = h.capture_actor_definition(ActorDefinitionKey(address));
            let reaction =
                match h.definition_routine_offset(&definition, DefinitionRoutine::Grapple) {
                    0x8005 => GrappleReaction::Attach,
                    0x800a => GrappleReaction::Ignore,
                    0x800f => GrappleReaction::Cancel,
                    0x8014 => GrappleReaction::AttachAlternate,
                    0x8019 => GrappleReaction::AttachAlternate2,
                    0x801e => GrappleReaction::DamagePlayer,
                    _ => GrappleReaction::None,
                };
            if matches!(
                reaction,
                GrappleReaction::Attach
                    | GrappleReaction::AttachAlternate
                    | GrappleReaction::AttachAlternate2
            ) {
                let selected = current(h);
                super::combat::snap_grapple(h, selected);
            }
            return Some(GrappleContact {
                reaction,
                definition: ActorDefinitionKey(address),
            });
        }
        index += 1;
    }
}
pub(in crate::game) fn apply_power_bomb_to_enemies<H: ActorHost + EprojHost>(h: &mut H) {
    h.session_mut().room.enemy_processing_stage = 5;
    let width = h.session().room.power_bomb_radius >> 8;
    let height = (width + (width >> 1)) >> 1;
    if width == 0 {
        return;
    }
    for index in (0..32u16).rev() {
        let offset = index * 64;
        let key = ActorKey::from_legacy(offset);
        {
            let actor = h.resolve_actor(key, AccessMode::Native);
            if actor.invincibility != 0 || matches!(actor.definition, 0 | 0xdaff) {
                continue;
            }
        }
        let definition = h.resolve_actor(key, AccessMode::Native).definition;
        let definition = h.capture_actor_definition(ActorDefinitionKey(definition));
        let address = h.definition_vulnerability_address(&definition);
        let vulnerability = h.capture_vulnerability_at(address);
        if h.projectile_vulnerability(&vulnerability, 0, 15) & 0x7f != 0
            && axis(
                h.session().projectiles.power_bomb.x,
                h.resolve_actor(key, AccessMode::Native).x.whole,
                0,
                width,
            )
            && axis(
                h.session().projectiles.power_bomb.y,
                h.resolve_actor(key, AccessMode::Native).y.whole,
                0,
                height,
            )
        {
            h.session_mut().actors.controls.selected = offset;
            let reaction = h.definition_routine_offset(&definition, DefinitionRoutine::PowerBomb);
            let bank = h.resolve_actor(key, AccessMode::Native).bank;
            h.actor_ai(ActorRoutine(
                u32::from(bank) << 16 | u32::from(if reaction == 0 { 0x8037 } else { reaction }),
            ));
            h.edit_actor(key, AccessMode::Native, |actor| actor.properties |= 0x800);
        }
    }
}
