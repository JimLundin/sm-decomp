//! Complete orbiting-part and body-history producer operations.
#![forbid(unsafe_code)]
use super::access::enemy;
use super::host::*;
use crate::actors::access::{AccessMode::LittleEndian as Le, ActorKey as Key};
use crate::geometry::Point;
use crate::{
    sprite_effect_controllers,
    sprite_effect_state::{LegacySpriteDefinition, LegacySpriteHandle, SpriteSpawn},
    sprite_objects,
};

use BotwoonField as B;
use FireArcField as W;

// Both helpers reacquire the current enemy handle on each call, as the old
// helpers did; their callers retain their separately captured parent handles.
fn classify_phase(phase: u16, ctx: &impl Host) -> (u16, u16) {
    let enemy = usize::from(ctx.inputs().current_enemy);
    if ctx.fire_arc_field(enemy, W::Reverse) != 0 {
        if (phase as i16).wrapping_sub(ctx.fire_arc_field(enemy, W::FirstTransition) as i16) >= 0 {
            (2, 1)
        } else if (phase as i16).wrapping_sub(ctx.fire_arc_field(enemy, W::SecondTransition) as i16)
            >= 0
        {
            (1, 0)
        } else {
            (0, 0)
        }
    } else if (phase as i16).wrapping_sub(ctx.fire_arc_field(enemy, W::FirstTransition) as i16) < 0
    {
        (2, 0)
    } else if (phase as i16).wrapping_sub(ctx.fire_arc_field(enemy, W::SecondTransition) as i16) < 0
    {
        (1, 1)
    } else {
        (0, 0)
    }
}
fn clamp_phase(phase: u16, ctx: &impl Host) -> (u16, bool) {
    let enemy = usize::from(ctx.inputs().current_enemy);
    if ctx.fire_arc_field(enemy, W::Reverse) == 0 {
        if (phase as i16).wrapping_sub(ctx.fire_arc_field(enemy, W::EndPhase) as i16) >= 0 {
            if (phase as i16).wrapping_sub(ctx.fire_arc_field(enemy, W::StartPhase) as i16) < 0 {
                return (phase, false);
            }
            return (ctx.fire_arc_field(enemy, W::StartPhase), false);
        }
    } else if (phase as i16).wrapping_sub(ctx.fire_arc_field(enemy, W::EndPhase) as i16) < 0 {
        if (phase as i16).wrapping_sub(ctx.fire_arc_field(enemy, W::StartPhase) as i16) < 0 {
            return (ctx.fire_arc_field(enemy, W::StartPhase), false);
        }
        return (phase, false);
    }
    (ctx.fire_arc_field(enemy, W::EndPhase), true)
}
fn phase_sound(stage: u16, ctx: &mut impl Host) {
    if stage != 2 {
        ctx.queue_sound(2, 6, 0x5e);
    }
}
fn orbit_component(phase: u16, amplitude: u16, quarter_turn: u16, ctx: &mut impl Host) -> u16 {
    let phase = phase.wrapping_add(quarter_turn) as u8;
    let product = u16::from(ctx.fire_arc_sine_sample(phase & 0x7f)) * u16::from(amplitude as u8);
    let integer = product >> 8;
    if phase & 0x80 != 0 {
        integer.wrapping_neg()
    } else {
        integer
    }
}

pub fn advance_fire_arc_orbit(ctx: &mut impl Host) {
    let enemy = usize::from(ctx.inputs().current_enemy);
    let radius = ctx.fire_arc_field(enemy, W::Radius);
    let (stage, sprite) = classify_phase(ctx.fire_arc_field(enemy, W::Phase), ctx);
    if stage != ctx.fire_arc_field(enemy, W::HeadStage) {
        let x = ctx
            .resolve_actor(Key::from_legacy(enemy as u16), Le)
            .x
            .whole;
        let y = ctx
            .resolve_actor(Key::from_legacy(enemy as u16), Le)
            .y
            .whole;
        let palette = ctx.fire_arc_field(enemy, W::Palette);
        sprite_objects::spawn(
            ctx,
            SpriteSpawn {
                position: Point { x, y },
                definition: LegacySpriteDefinition(46),
                appearance: palette,
            },
        );
        let x = ctx
            .resolve_actor(Key::from_legacy(enemy as u16), Le)
            .x
            .whole;
        let y = ctx
            .resolve_actor(Key::from_legacy(enemy as u16), Le)
            .y
            .whole;
        let palette = ctx.fire_arc_field(enemy, W::Palette);
        sprite_objects::spawn(
            ctx,
            SpriteSpawn {
                position: Point { x, y },
                definition: LegacySpriteDefinition(sprite.wrapping_add(44)),
                appearance: palette,
            },
        );
        phase_sound(stage, ctx);
    }
    ctx.set_fire_arc_field(enemy, W::HeadStage, stage);
    let (phase, _) = clamp_phase(ctx.fire_arc_field(enemy, W::Phase), ctx);
    let center = ctx.fire_arc_field(enemy, W::CenterX);
    let amplitude = ctx.fire_arc_field(enemy, W::Radius);
    let x = center.wrapping_add(orbit_component(phase, amplitude, 64, ctx));
    ctx.edit_actor(Key::from_legacy(enemy as u16), Le, |actor| {
        actor.x.whole = x
    });
    let (phase, _) = clamp_phase(ctx.fire_arc_field(enemy, W::Phase), ctx);
    let center = ctx.fire_arc_field(enemy, W::CenterY);
    let amplitude = ctx.fire_arc_field(enemy, W::Radius);
    let y = center.wrapping_add(orbit_component(phase, amplitude, 128, ctx));
    ctx.edit_actor(Key::from_legacy(enemy as u16), Le, |actor| {
        actor.y.whole = y
    });
    advance_fire_arc_parts(radius, ctx);
    advance_fire_arc_sprites(radius, ctx);
    // Original AddToHiLo captures speed high/low, reads phase low/high,
    // then publishes low/high. Its two halves may overlap enemy/owned aliases.
    let delta = (u32::from(ctx.fire_arc_field(enemy, W::PhaseSpeed)) << 16)
        | u32::from(ctx.fire_arc_field(enemy, W::PhaseSpeedFraction));
    let low = ctx.fire_arc_field(enemy, W::PhaseFraction);
    let high = ctx.fire_arc_field(enemy, W::Phase);
    let value = ((u32::from(high) << 16) | u32::from(low)).wrapping_add(delta);
    ctx.set_fire_arc_field(enemy, W::PhaseFraction, value as u16);
    ctx.set_fire_arc_field(enemy, W::Phase, (value >> 16) as u16);
}

fn advance_fire_arc_parts(radius: u16, ctx: &mut impl Host) {
    let enemy = usize::from(ctx.inputs().current_enemy);
    ctx.set_fire_arc_field(enemy, W::PartCursor, 8);
    let mut phase = ctx
        .fire_arc_field(enemy, W::PhaseLead)
        .wrapping_add(ctx.fire_arc_field(enemy, W::Phase));
    let mut reached_end;
    loop {
        phase = phase.wrapping_sub(ctx.fire_arc_field(enemy, W::Spacing));
        let part = usize::from(ctx.inputs().current_enemy)
            + usize::from(ctx.fire_arc_field(enemy, W::PartCursor));
        let projectile = ctx.fire_arc_field(part, W::Projectile);
        let (stage, sprite) = classify_phase(phase, ctx);
        if stage != ctx.fire_arc_field(part, W::PartStage) {
            let slot = (projectile >> 1) as isize;
            let x = enemy!(ctx, slot, x);
            let y = enemy!(ctx, slot, y);
            let palette = ctx.fire_arc_field(enemy, W::Palette);
            sprite_objects::spawn(
                ctx,
                SpriteSpawn {
                    position: Point { x, y },
                    definition: LegacySpriteDefinition(46),
                    appearance: palette,
                },
            );
            let x = enemy!(ctx, slot, x);
            let y = enemy!(ctx, slot, y);
            let palette = ctx.fire_arc_field(enemy, W::Palette);
            sprite_objects::spawn(
                ctx,
                SpriteSpawn {
                    position: Point { x, y },
                    definition: LegacySpriteDefinition(sprite.wrapping_add(44)),
                    appearance: palette,
                },
            );
            phase_sound(stage, ctx);
        }
        ctx.set_fire_arc_field(part, W::PartStage, stage);
        let (clamped, _) = clamp_phase(phase, ctx);
        let slot = (projectile >> 1) as isize;
        let center = ctx.fire_arc_field(enemy, W::CenterX);
        let x = center.wrapping_add(orbit_component(clamped, radius, 64, ctx));
        enemy!(ctx, slot, x = x);
        let (clamped, last_reached_end) = clamp_phase(phase, ctx);
        reached_end = last_reached_end;
        let center = ctx.fire_arc_field(enemy, W::CenterY);
        let y = center.wrapping_add(orbit_component(clamped, radius, 128, ctx));
        enemy!(ctx, slot, y = y);
        let next = ctx.fire_arc_field(enemy, W::PartCursor).wrapping_sub(2);
        ctx.set_fire_arc_field(enemy, W::PartCursor, next);
        if next == 0 {
            break;
        }
    }
    if reached_end {
        ctx.set_fire_arc_field(enemy, W::Behavior, 0x9615);
        ctx.edit_actor(Key::from_legacy(enemy as u16), Le, |actor| {
            actor.properties &= !0x800
        });
    }
}

fn advance_fire_arc_sprites(radius: u16, ctx: &mut impl Host) {
    let enemy = usize::from(ctx.inputs().current_enemy);
    ctx.set_fire_arc_field(enemy, W::PartCursor, 6);
    let mut phase = ctx.fire_arc_field(enemy, W::Phase);
    loop {
        phase = phase.wrapping_sub(ctx.fire_arc_field(enemy, W::Spacing));
        let part = usize::from(ctx.inputs().current_enemy)
            + usize::from(ctx.fire_arc_field(enemy, W::PartCursor));
        let sprite_handle = ctx.fire_arc_field(part, W::Sprite);
        let (stage, sprite) = classify_phase(phase, ctx);
        if stage != ctx.fire_arc_field(part, W::SpriteStage) {
            let palette = ctx.fire_arc_field(enemy, W::Palette);
            let handle = LegacySpriteHandle(sprite_handle);
            let position = sprite_effect_controllers::position(ctx, handle);
            sprite_objects::spawn(
                ctx,
                SpriteSpawn {
                    position,
                    definition: LegacySpriteDefinition(46),
                    appearance: palette,
                },
            );
            // Allocation and content diagnostics can change this same slot.
            let position = sprite_effect_controllers::position(ctx, handle);
            sprite_objects::spawn(
                ctx,
                SpriteSpawn {
                    position,
                    definition: LegacySpriteDefinition(sprite.wrapping_add(44)),
                    appearance: palette,
                },
            );
            phase_sound(stage, ctx);
        }
        ctx.set_fire_arc_field(part, W::SpriteStage, stage);
        let (clamped, _) = clamp_phase(phase, ctx);
        let component = orbit_component(clamped, radius, 64, ctx);
        let x = ctx
            .fire_arc_field(enemy, W::CenterX)
            .wrapping_add(component);
        let (clamped, _) = clamp_phase(phase, ctx);
        let center = ctx.fire_arc_field(enemy, W::CenterY);
        let y = center.wrapping_add(orbit_component(clamped, radius, 128, ctx));
        sprite_effect_controllers::set_position(
            ctx,
            LegacySpriteHandle(sprite_handle),
            Point { x, y },
        );
        crate::probe::hit("sprite.waffle.pose");
        let next = ctx.fire_arc_field(enemy, W::PartCursor).wrapping_sub(2);
        ctx.set_fire_arc_field(enemy, W::PartCursor, next);
        if next == 0 {
            break;
        }
    }
}

pub fn record_botwoon_head(ctx: &mut impl Host) {
    let enemy = usize::from(ctx.inputs().current_enemy);
    let cursor = ctx.botwoon_field(enemy, B::HistoryCursor);
    ctx.set_botwoon_history_x(
        cursor,
        ctx.resolve_actor(Key::from_legacy(enemy as u16), Le)
            .x
            .whole,
    );
    ctx.set_botwoon_history_y(
        cursor,
        ctx.resolve_actor(Key::from_legacy(enemy as u16), Le)
            .y
            .whole,
    );
}

pub fn follow_botwoon_history(ctx: &mut impl Host) {
    let enemy = usize::from(ctx.inputs().current_enemy);
    let mut cursor = ctx
        .botwoon_field(enemy, B::HistoryCursor)
        .wrapping_sub(ctx.botwoon_field(enemy, B::HistorySpacing))
        & 0x3ff;
    for part in (0..=24).rev().step_by(2) {
        let projectile = ctx.botwoon_field(part, B::Projectile);
        if ctx.botwoon_field(enemy, B::TransitionCursor) == cursor {
            let was_visible = ctx.botwoon_field(part, B::Visible) == 1;
            ctx.set_botwoon_field(part, B::Visible, ctx.botwoon_field(part, B::Visible) ^ 1);
            let slot = (projectile >> 1) as isize;
            if was_visible {
                enemy!(
                    ctx,
                    slot,
                    properties = enemy!(ctx, slot, properties) & !0x2000
                );
                enemy!(ctx, slot, drop_flags = 1);
            } else {
                enemy!(
                    ctx,
                    slot,
                    properties = enemy!(ctx, slot, properties) | 0x2000
                );
                enemy!(ctx, slot, drop_flags = 2);
            }
            if part == 0 {
                crate::actors::botwoon_death::confirm_tail(ctx, enemy);
            }
        }
        let slot = (projectile >> 1) as isize;
        enemy!(ctx, slot, x = ctx.botwoon_history_x(cursor));
        enemy!(ctx, slot, y = ctx.botwoon_history_y(cursor));
        cursor = cursor.wrapping_sub(ctx.botwoon_field(enemy, B::HistorySpacing)) & 0x3ff;
    }
}

pub fn advance_botwoon_history(ctx: &mut impl Host) {
    let enemy = usize::from(ctx.inputs().current_enemy);
    let cursor = ctx.botwoon_field(enemy, B::HistoryCursor).wrapping_add(4) & 0x3ff;
    ctx.set_botwoon_field(enemy, B::HistoryCursor, cursor);
}

pub fn orient_botwoon_body(ctx: &mut impl Host) {
    let enemy = usize::from(ctx.inputs().current_enemy);
    for part in (0..=24).rev().step_by(2) {
        let projectile = ctx.botwoon_field(part, B::Projectile);
        let mut bias = if ctx.botwoon_field(part, B::Visible) != 0 {
            256u16
        } else {
            0
        };
        let slot = (projectile >> 1) as isize;
        let (x, y) = if part == 24 {
            let x = ctx
                .resolve_actor(Key::from_legacy(enemy as u16), Le)
                .x
                .whole
                .wrapping_sub(enemy!(ctx, slot, x));
            let y = ctx
                .resolve_actor(Key::from_legacy(enemy as u16), Le)
                .y
                .whole
                .wrapping_sub(enemy!(ctx, slot, y));
            (x, y)
        } else {
            if part == 0 {
                bias = bias.wrapping_add(512);
            }
            let x = enemy!(ctx, slot + 1, x).wrapping_sub(enemy!(ctx, slot, x));
            let y = enemy!(ctx, slot + 1, y).wrapping_sub(enemy!(ctx, slot, y));
            (x, y)
        };
        let orientation =
            2 * (i32::from(bias.wrapping_add(ctx.calculate_angle_from_xy(x, y))) >> 5);
        enemy!(ctx, slot, behavior_a = orientation as u16);
    }
}
