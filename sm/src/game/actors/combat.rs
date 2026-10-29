//! Common actor combat over the owned Session; compatibility calls end every borrow.
#![forbid(unsafe_code)]
use super::{
    access::{AccessMode, ActorAccess, ActorKey},
    api::{DeathHandling, DeathKind, GrappleContact, GrappleReaction, ShotHandling},
    content::{
        ActorDefinitionCapture, ActorDefinitionKey, ActorRoutine, DefinitionByte,
        DefinitionRoutine, DefinitionWord,
    },
    effects::ActorHost,
};
use crate::game::{
    combat_rules::{self as rules, GrappleOutcome, HealthResponse, LatchMode, Shot, ShotResponse},
    enemy_projectiles::{
        host::{Host as EprojHost, ShotMember},
        lifecycle, set_shot_direction, shot_word,
    },
    room_mechanisms::host::Effects as MechanismEffects,
    sprite_effect_state::{LegacySpriteDefinition, SpriteSpawn},
    sprite_objects,
};
use sm_game::actors::state::Actor;
use sm_game::geometry::Point;

pub(super) fn current(h: &impl ActorAccess) -> ActorKey {
    ActorKey::from_legacy(h.session().actors.controls.selected)
}
fn capture_definition(h: &mut impl ActorHost, actor: ActorKey) -> ActorDefinitionCapture {
    let key = h.resolve_actor(actor, AccessMode::Native).definition;
    h.capture_actor_definition(ActorDefinitionKey(key))
}
pub(in crate::game) fn suit_damage(h: &impl ActorAccess, amount: u16) -> u16 {
    rules::suit_damage(amount, h.session().inventory.equipment.active)
}
pub(in crate::game) fn select_frozen_ai(h: &mut impl ActorAccess) {
    let actor = current(h);
    h.edit_actor(actor, AccessMode::Native, |actor| actor.ai_handlers = 4);
}
pub(in crate::game) fn resume_ai(h: &mut impl ActorAccess) {
    let actor = current(h);
    h.edit_actor(actor, AccessMode::Native, |actor| {
        actor.ai_handlers = 0;
        actor.invincibility = 0;
        actor.frozen = 0;
        actor.shake = 0;
    });
}
fn freeze(h: &mut impl ActorAccess, actor: ActorKey) {
    let duration = rules::freeze_duration(h.session().room.area);
    h.edit_actor(actor, AccessMode::Native, |actor| {
        actor.frozen = duration;
        actor.ai_handlers |= 4;
        actor.invincibility = 10;
    });
}
/// The collision selector is a full word index, not a wrapping byte handle.
pub(super) fn mark_shot(h: &mut impl EprojHost, index: isize) {
    let direction = shot_word(h, ShotMember::Direction, index) | 0x10;
    set_shot_direction(h, index, direction);
}
fn sprite(h: &mut impl EprojHost, x: u16, y: u16, selector: u16) {
    sprite_objects::spawn(
        h,
        SpriteSpawn {
            position: Point { x, y },
            definition: LegacySpriteDefinition(selector),
            appearance: 0,
        },
    );
}
fn death<H: ActorHost + EprojHost>(h: &mut H, animation: u16, limit: u16, actor: ActorKey) {
    let animation = if animation.wrapping_sub(limit) & 0x8000 == 0 {
        0
    } else {
        animation
    };
    h.session_mut().compatibility_enemy_scratch = animation;
    let selected = h.session().actors.controls.selected;
    lifecycle::spawn_from_enemy(
        h,
        animation,
        selected,
        crate::game::constants::bank_86::addr_kEproj_EnemyDeathExplosion as u16,
    );
    let preserve = h.resolve_actor(actor, AccessMode::Native).properties & 0x4000 != 0;
    h.edit_actor(actor, AccessMode::Native, |actor| *actor = Actor::default());
    if preserve {
        h.edit_actor(actor, AccessMode::Native, |actor| {
            actor.definition = 0xdaff;
            actor.bank = 0xa3;
        });
    }
    let index = actor.byte_index();
    if index < 2048 && index & 63 == 0 {
        if preserve {
            h.session_mut()
                .actor_identity
                .convert_to_placeholder(index / 64);
        } else {
            h.session_mut().actor_identity.retire(index / 64);
        }
    }
}
pub(in crate::game) fn die<H: ActorHost + EprojHost>(h: &mut H, kind: DeathKind) {
    match kind {
        DeathKind::Normal { animation } => {
            let actor = current(h);
            if h.resolve_actor(actor, AccessMode::Native).ai_handlers == 1 {
                h.session_mut().player.grapple = 0xc8c5;
            }
            death(h, animation, 5, actor);
            h.session_mut().room.enemy_deaths = h.session().room.enemy_deaths.wrapping_add(1);
        }
        DeathKind::Rinka { animation } => {
            let actor = current(h);
            death(h, animation, 3, actor);
        }
        DeathKind::Grapple => {
            cause(h, 4);
            die(h, DeathKind::Normal { animation: 0 });
            let actor = current(h);
            h.edit_actor(actor, AccessMode::Native, |actor| actor.ai_handlers = 0);
        }
    }
}
fn contact_without_death<H: ActorHost + EprojHost + MechanismEffects>(h: &mut H) {
    let actor = current(h);
    let definition = capture_definition(h, actor);
    let mode = h.session().player.contact.contact_attack;
    if mode == 0 {
        let amount = h.definition_word(&definition, DefinitionWord::Damage);
        let damage = suit_damage(h, amount);
        h.damage_player(damage);
        h.session_mut().player.invincibility = 96;
        h.session_mut().player.knockback_timer = 5;
        let direction = (h
            .session()
            .player
            .x
            .wrapping_sub(h.resolve_actor(actor, AccessMode::Native).x.whole)
            as i16
            >= 0) as u16;
        h.session_mut().player.knockback_x_direction = direction;
        return;
    }
    let (offset, base_damage) = rules::contact_attack(mode);
    if mode == 4 {
        h.player_control(4);
    }
    let address = h.definition_vulnerability_address(&definition);
    let source = h.capture_vulnerability_at(address.wrapping_add(offset));
    let power = h.contact_vulnerability(&source, 0);
    h.session_mut().actors.combat.last_power = power;
    let damage = (base_damage >> 1).wrapping_mul(power & 0x7f);
    if damage != 0 {
        let duration =
            rules::hurt_duration(h.definition_byte(&definition, DefinitionByte::HurtTime), 0);
        h.edit_actor(actor, AccessMode::Native, |actor| {
            actor.flash = duration;
            actor.ai_handlers |= 2;
        });
        h.session_mut().player.invincibility = 0;
        h.session_mut().player.knockback_timer = 0;
        h.edit_actor(actor, AccessMode::Native, |actor| {
            actor.health = rules::contact_health(actor.health, damage);
        });
        h.queue_sound(2, 1, 0xb);
    }
}
pub(in crate::game) fn contact_reaction<H: ActorHost + EprojHost + MechanismEffects>(
    h: &mut H,
    handling: DeathHandling,
) {
    contact_without_death(h);
    if matches!(handling, DeathHandling::Finalize)
        && h.resolve_actor(current(h), AccessMode::Native).health == 0
    {
        cause(h, 6);
        die(h, DeathKind::Normal { animation: 1 });
    }
}
fn power_bomb_without_death<H: ActorHost + EprojHost>(h: &mut H) {
    let selected = current(h);
    let definition = capture_definition(h, selected);
    let address = h.definition_vulnerability_address(&definition);
    let source = h.capture_vulnerability_at(address);
    let damage = rules::power_bomb_damage(h.projectile_vulnerability(&source, 0, 15));
    if damage == 0 {
        return;
    }
    let actor = current(h);
    h.edit_actor(actor, AccessMode::Native, |actor| actor.invincibility = 48);
    let selected = current(h);
    let definition = capture_definition(h, selected);
    let actor = current(h);
    let duration =
        rules::hurt_duration(h.definition_byte(&definition, DefinitionByte::HurtTime), 8);
    h.edit_actor(actor, AccessMode::Native, |actor| {
        actor.flash = duration;
        actor.ai_handlers |= 2;
        actor.health = actor.health.saturating_sub(damage);
    });
}
pub(in crate::game) fn power_bomb_reaction<H: ActorHost + EprojHost>(
    h: &mut H,
    handling: DeathHandling,
) {
    power_bomb_without_death(h);
    if matches!(handling, DeathHandling::Finalize)
        && h.resolve_actor(current(h), AccessMode::Native).health == 0
    {
        cause(h, 3);
        die(h, DeathKind::Normal { animation: 0 });
    }
}
fn shot_without_death<H: ActorHost + EprojHost>(h: &mut H) -> bool {
    let index = h.session().room.collision_detection_index as isize;
    let shot = Shot {
        damage: shot_word(h, ShotMember::Damage, index),
        kind: shot_word(h, ShotMember::Kind, index),
    };
    let selected = current(h);
    let definition = capture_definition(h, selected);
    let address = h.definition_vulnerability_address(&definition);
    let primary = if let Some((offset, field)) = shot.vulnerability_offset() {
        let source = h.capture_vulnerability_at(address.wrapping_add(offset));
        h.projectile_vulnerability(&source, 0, field)
    } else {
        0
    };
    if shot.beam() {
        h.session_mut().actors.combat.last_power = u16::from(primary);
    }
    let charged = if shot.charged(primary) {
        let source = h.capture_vulnerability_at(address);
        h.projectile_vulnerability(&source, 0, 19)
    } else {
        0
    };
    let damage = match shot.response(primary, charged) {
        ShotResponse::Dud => {
            let index = h.session().room.collision_detection_index as isize;
            mark_shot(h, index);
            let x = shot_word(h, ShotMember::X, index);
            let y = shot_word(h, ShotMember::Y, index);
            sprite(h, x, y, 6);
            h.queue_sound(1, 3, 0x3d);
            return false;
        }
        ShotResponse::Freeze => {
            let actor = current(h);
            if h.resolve_actor(actor, AccessMode::Native).frozen == 0 {
                h.queue_sound(3, 3, 0xa);
            }
            freeze(h, actor);
            return false;
        }
        ShotResponse::Damage(damage) => damage,
    };
    let selected = current(h);
    let definition = capture_definition(h, selected);
    let actor = current(h);
    let duration =
        rules::hurt_duration(h.definition_byte(&definition, DefinitionByte::HurtTime), 8);
    h.edit_actor(actor, AccessMode::Native, |actor| {
        actor.flash = duration;
        actor.ai_handlers |= 2;
    });
    let flash = h.resolve_actor(actor, AccessMode::Native).frozen == 0;
    if flash {
        let definition = capture_definition(h, actor);
        let sound = h.definition_word(&definition, DefinitionWord::HurtSound);
        if sound != 0 {
            h.queue_sound(2, 3, sound);
        }
    }
    let actor = current(h);
    let projectile = shot_word(
        h,
        ShotMember::Kind,
        h.session().room.collision_detection_index as isize,
    );
    if projectile & 8 != 0 {
        h.edit_actor(actor, AccessMode::Native, |actor| actor.invincibility = 16);
    }
    let response = rules::shot_health(
        h.resolve_actor(actor, AccessMode::Native).health,
        damage,
        shot_word(
            h,
            ShotMember::Kind,
            h.session().room.collision_detection_index as isize,
        ),
        h.session().actors.combat.last_power,
        h.resolve_actor(actor, AccessMode::Native).frozen,
    );
    match response {
        HealthResponse::Health(value) => {
            h.edit_actor(actor, AccessMode::Native, |actor| actor.health = value)
        }
        HealthResponse::Freeze => {
            freeze(h, actor);
            h.queue_sound(3, 3, 0xa);
        }
    }
    flash
}
pub(in crate::game) fn shot_reaction<H: ActorHost + EprojHost>(h: &mut H, handling: ShotHandling) {
    let flash = shot_without_death(h);
    if matches!(handling, ShotHandling::ReactionOnly) {
        return;
    }
    let actor = current(h);
    if flash {
        let x = h.resolve_actor(actor, AccessMode::Native).x.whole;
        let y = h.resolve_actor(actor, AccessMode::Native).y.whole;
        sprite(h, x, y, 55);
    }
    if matches!(handling, ShotHandling::Finalize)
        && h.resolve_actor(actor, AccessMode::Native).health == 0
    {
        let projectile = shot_word(
            h,
            ShotMember::Kind,
            h.session().room.collision_detection_index as isize,
        );
        let kind = (projectile >> 8) & 15;
        cause(h, kind);
        let definition = capture_definition(h, actor);
        let animation = h.definition_word(&definition, DefinitionWord::DeathAnimation);
        let animation = if kind == 2 && animation.wrapping_sub(3) & 0x8000 != 0 {
            2
        } else {
            animation
        };
        die(h, DeathKind::Normal { animation });
    }
}
pub(in crate::game) fn create_dud<H: ActorHost + EprojHost>(h: &mut H) {
    let index = h.session().room.collision_detection_index as isize;
    let x = shot_word(h, ShotMember::X, index);
    let y = shot_word(h, ShotMember::Y, index);
    sprite(h, x, y, 6);
    h.queue_sound(1, 3, 0x3d);
    let index = h.session().room.collision_detection_index as isize;
    mark_shot(h, index);
}
pub(in crate::game) fn latch<H: ActorHost>(h: &mut H, mode: LatchMode) {
    let actor = current(h);
    if mode == LatchMode::Passive {
        snap_grapple(h, actor);
    }
    if h.resolve_actor(actor, AccessMode::Native).frozen != 0 {
        h.edit_actor(actor, AccessMode::Native, |actor| actor.ai_handlers = 4);
    } else {
        let bank = h.resolve_actor(actor, AccessMode::Native).bank;
        let definition = capture_definition(h, actor);
        match mode {
            LatchMode::Passive => {
                let duration = rules::hurt_duration(
                    h.definition_byte(&definition, DefinitionByte::HurtTime),
                    0,
                );
                h.edit_actor(actor, AccessMode::Native, |actor| actor.flash = duration);
            }
            LatchMode::Active => {
                let routine = h.definition_routine_offset(&definition, DefinitionRoutine::Main);
                h.actor_ai(ActorRoutine(u32::from(bank) << 16 | u32::from(routine)));
            }
        }
        h.edit_actor(actor, AccessMode::Native, |actor| actor.ai_handlers = 0);
    }
    if mode == LatchMode::Active {
        snap_grapple(h, actor);
    }
}
pub(in crate::game) fn resolve_grapple<H: ActorHost + EprojHost>(
    h: &mut H,
    contact: GrappleContact,
) -> GrappleOutcome {
    let GrappleContact {
        reaction,
        definition: key,
    } = contact;
    match reaction {
        GrappleReaction::None | GrappleReaction::Ignore => GrappleOutcome::Miss,
        GrappleReaction::Cancel => GrappleOutcome::Cancel,
        GrappleReaction::Attach
        | GrappleReaction::AttachAlternate
        | GrappleReaction::AttachAlternate2 => GrappleOutcome::Attach,
        GrappleReaction::DamagePlayer => {
            let definition = h.capture_actor_definition(key);
            let amount = h.definition_word(&definition, DefinitionWord::Damage);
            let damage = suit_damage(h, amount);
            h.damage_player(damage);
            h.session_mut().player.invincibility = 96;
            h.session_mut().player.knockback_timer = 5;
            h.session_mut().player.knockback_x_direction =
                u16::from(h.session().player.movement_and_facing as u8 == 4);
            GrappleOutcome::Cancel
        }
        GrappleReaction::Unrecognized(_) => {
            h.unreachable();
            GrappleOutcome::Attach
        }
    }
}

pub(super) fn cause<H: ActorHost>(h: &mut H, value: u16) {
    let key = current(h);
    h.edit_spawn(key, AccessMode::Native, |spawn| spawn.death_cause = value);
}

pub(super) fn snap_grapple(h: &mut impl ActorAccess, actor: ActorKey) {
    let x = h.resolve_actor(actor, AccessMode::Native).x.whole;
    h.session_mut().player.grapple_endpoint.x = x;
    let y = h.resolve_actor(actor, AccessMode::Native).y.whole;
    h.session_mut().player.grapple_endpoint.y = y;
}
