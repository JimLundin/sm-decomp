//! Complete orbiting-part and body-history producer operations.
#![forbid(unsafe_code)]
use super::compat::Execution;
use super::state::SlotField as F;

// Private semantic fields at the remaining enemy compatibility edge.
#[derive(Clone, Copy)]
pub(super) enum WaffleField {
    Behavior,
    Reverse,
    Radius,
    PhaseFraction,
    Phase,
    StartPhase,
    PhaseSpeedFraction,
    PhaseSpeed,
    CenterX,
    CenterY,
    PartCursor,
    Spacing,
    PhaseLead,
    EndPhase,
    FirstTransition,
    SecondTransition,
    HeadStage,
    Palette,
    Projectile,
    PartStage,
    Sprite,
    SpriteStage,
    X,
    Y,
    Properties,
}
#[derive(Clone, Copy)]
pub(super) enum BotwoonField {
    HistoryCursor,
    HistorySpacing,
    Projectile,
    Visible,
    TransitionPending,
    TransitionCursor,
    TailState,
    Pattern,
    X,
    Y,
}
use BotwoonField as B;
use WaffleField as W;

// Both helpers reacquire the current enemy handle on each call, as the old
// helpers did; their callers retain their separately captured parent handles.
fn classify_phase(phase: u16, ctx: &Execution<'_>) -> (u16, u16) {
    let enemy = usize::from(ctx.current_enemy());
    if ctx.waffle_field(enemy, W::Reverse) != 0 {
        if (phase as i16).wrapping_sub(ctx.waffle_field(enemy, W::FirstTransition) as i16) >= 0 {
            (2, 1)
        } else if (phase as i16).wrapping_sub(ctx.waffle_field(enemy, W::SecondTransition) as i16)
            >= 0
        {
            (1, 0)
        } else {
            (0, 0)
        }
    } else if (phase as i16).wrapping_sub(ctx.waffle_field(enemy, W::FirstTransition) as i16) < 0 {
        (2, 0)
    } else if (phase as i16).wrapping_sub(ctx.waffle_field(enemy, W::SecondTransition) as i16) < 0 {
        (1, 1)
    } else {
        (0, 0)
    }
}
fn clamp_phase(phase: u16, ctx: &Execution<'_>) -> (u16, bool) {
    let enemy = usize::from(ctx.current_enemy());
    if ctx.waffle_field(enemy, W::Reverse) == 0 {
        if (phase as i16).wrapping_sub(ctx.waffle_field(enemy, W::EndPhase) as i16) >= 0 {
            if (phase as i16).wrapping_sub(ctx.waffle_field(enemy, W::StartPhase) as i16) < 0 {
                return (phase, false);
            }
            return (ctx.waffle_field(enemy, W::StartPhase), false);
        }
    } else if (phase as i16).wrapping_sub(ctx.waffle_field(enemy, W::EndPhase) as i16) < 0 {
        if (phase as i16).wrapping_sub(ctx.waffle_field(enemy, W::StartPhase) as i16) < 0 {
            return (ctx.waffle_field(enemy, W::StartPhase), false);
        }
        return (phase, false);
    }
    (ctx.waffle_field(enemy, W::EndPhase), true)
}
fn phase_sound(stage: u16, ctx: &mut Execution<'_>) {
    if stage != 2 {
        ctx.queue_sound(2, 6, 0x5e);
    }
}
fn orbit_component(phase: u16, amplitude: u16, quarter_turn: u16, ctx: &mut Execution<'_>) -> u16 {
    let phase = phase.wrapping_add(quarter_turn) as u8;
    let product = u16::from(ctx.waffle_sine_sample(phase & 0x7f)) * u16::from(amplitude as u8);
    let integer = product >> 8;
    if phase & 0x80 != 0 {
        integer.wrapping_neg()
    } else {
        integer
    }
}

pub(super) fn advance_waffle_orbit(ctx: &mut Execution<'_>) {
    let enemy = usize::from(ctx.current_enemy());
    let radius = ctx.waffle_field(enemy, W::Radius);
    let (stage, sprite) = classify_phase(ctx.waffle_field(enemy, W::Phase), ctx);
    if stage != ctx.waffle_field(enemy, W::HeadStage) {
        let x = ctx.waffle_field(enemy, W::X);
        let y = ctx.waffle_field(enemy, W::Y);
        let palette = ctx.waffle_field(enemy, W::Palette);
        ctx.create_sprite(x, y, 46, palette);
        let x = ctx.waffle_field(enemy, W::X);
        let y = ctx.waffle_field(enemy, W::Y);
        let palette = ctx.waffle_field(enemy, W::Palette);
        ctx.create_sprite(x, y, sprite.wrapping_add(44), palette);
        phase_sound(stage, ctx);
    }
    ctx.set_waffle_field(enemy, W::HeadStage, stage);
    let (phase, _) = clamp_phase(ctx.waffle_field(enemy, W::Phase), ctx);
    let center = ctx.waffle_field(enemy, W::CenterX);
    let amplitude = ctx.waffle_field(enemy, W::Radius);
    let x = center.wrapping_add(orbit_component(phase, amplitude, 64, ctx));
    ctx.set_waffle_field(enemy, W::X, x);
    let (phase, _) = clamp_phase(ctx.waffle_field(enemy, W::Phase), ctx);
    let center = ctx.waffle_field(enemy, W::CenterY);
    let amplitude = ctx.waffle_field(enemy, W::Radius);
    let y = center.wrapping_add(orbit_component(phase, amplitude, 128, ctx));
    ctx.set_waffle_field(enemy, W::Y, y);
    advance_waffle_parts(radius, ctx);
    advance_waffle_sprites(radius, ctx);
    // Original AddToHiLo captures speed high/low, reads phase low/high,
    // then publishes low/high. Its two halves may overlap enemy/owned aliases.
    let delta = (u32::from(ctx.waffle_field(enemy, W::PhaseSpeed)) << 16)
        | u32::from(ctx.waffle_field(enemy, W::PhaseSpeedFraction));
    let low = ctx.waffle_field(enemy, W::PhaseFraction);
    let high = ctx.waffle_field(enemy, W::Phase);
    let value = ((u32::from(high) << 16) | u32::from(low)).wrapping_add(delta);
    ctx.set_waffle_field(enemy, W::PhaseFraction, value as u16);
    ctx.set_waffle_field(enemy, W::Phase, (value >> 16) as u16);
}

fn advance_waffle_parts(radius: u16, ctx: &mut Execution<'_>) {
    let enemy = usize::from(ctx.current_enemy());
    ctx.set_waffle_field(enemy, W::PartCursor, 8);
    let mut phase = ctx
        .waffle_field(enemy, W::PhaseLead)
        .wrapping_add(ctx.waffle_field(enemy, W::Phase));
    let mut reached_end;
    loop {
        phase = phase.wrapping_sub(ctx.waffle_field(enemy, W::Spacing));
        let part =
            usize::from(ctx.current_enemy()) + usize::from(ctx.waffle_field(enemy, W::PartCursor));
        let projectile = ctx.waffle_field(part, W::Projectile);
        let (stage, sprite) = classify_phase(phase, ctx);
        if stage != ctx.waffle_field(part, W::PartStage) {
            let slot = (projectile >> 1) as isize;
            let x = ctx.slot(F::X, slot);
            let y = ctx.slot(F::Y, slot);
            let palette = ctx.waffle_field(enemy, W::Palette);
            ctx.create_sprite(x, y, 46, palette);
            let x = ctx.slot(F::X, slot);
            let y = ctx.slot(F::Y, slot);
            let palette = ctx.waffle_field(enemy, W::Palette);
            ctx.create_sprite(x, y, sprite.wrapping_add(44), palette);
            phase_sound(stage, ctx);
        }
        ctx.set_waffle_field(part, W::PartStage, stage);
        let (clamped, _) = clamp_phase(phase, ctx);
        let slot = (projectile >> 1) as isize;
        let center = ctx.waffle_field(enemy, W::CenterX);
        let x = center.wrapping_add(orbit_component(clamped, radius, 64, ctx));
        ctx.set_slot(F::X, slot, x);
        let (clamped, last_reached_end) = clamp_phase(phase, ctx);
        reached_end = last_reached_end;
        let center = ctx.waffle_field(enemy, W::CenterY);
        let y = center.wrapping_add(orbit_component(clamped, radius, 128, ctx));
        ctx.set_slot(F::Y, slot, y);
        let next = ctx.waffle_field(enemy, W::PartCursor).wrapping_sub(2);
        ctx.set_waffle_field(enemy, W::PartCursor, next);
        if next == 0 {
            break;
        }
    }
    if reached_end {
        ctx.set_waffle_field(enemy, W::Behavior, 0x9615);
        ctx.set_waffle_field(
            enemy,
            W::Properties,
            ctx.waffle_field(enemy, W::Properties) & !0x800,
        );
    }
}

fn advance_waffle_sprites(radius: u16, ctx: &mut Execution<'_>) {
    let enemy = usize::from(ctx.current_enemy());
    ctx.set_waffle_field(enemy, W::PartCursor, 6);
    let mut phase = ctx.waffle_field(enemy, W::Phase);
    loop {
        phase = phase.wrapping_sub(ctx.waffle_field(enemy, W::Spacing));
        let part =
            usize::from(ctx.current_enemy()) + usize::from(ctx.waffle_field(enemy, W::PartCursor));
        let sprite_handle = ctx.waffle_field(part, W::Sprite);
        let (stage, sprite) = classify_phase(phase, ctx);
        if stage != ctx.waffle_field(part, W::SpriteStage) {
            let palette = ctx.waffle_field(enemy, W::Palette);
            let slot = sprite_handle >> 1;
            let x = ctx.waffle_sprite_x(slot);
            let y = ctx.waffle_sprite_y(slot);
            ctx.create_sprite(x, y, 46, palette);
            let x = ctx.waffle_sprite_x(slot);
            let y = ctx.waffle_sprite_y(slot);
            ctx.create_sprite(x, y, sprite.wrapping_add(44), palette);
            phase_sound(stage, ctx);
        }
        ctx.set_waffle_field(part, W::SpriteStage, stage);
        let (clamped, _) = clamp_phase(phase, ctx);
        let component = orbit_component(clamped, radius, 64, ctx);
        let x = ctx.waffle_field(enemy, W::CenterX).wrapping_add(component);
        let (clamped, _) = clamp_phase(phase, ctx);
        let center = ctx.waffle_field(enemy, W::CenterY);
        let y = center.wrapping_add(orbit_component(clamped, radius, 128, ctx));
        ctx.set_waffle_sprite_x(sprite_handle >> 1, x);
        ctx.set_waffle_sprite_y(sprite_handle >> 1, y);
        let next = ctx.waffle_field(enemy, W::PartCursor).wrapping_sub(2);
        ctx.set_waffle_field(enemy, W::PartCursor, next);
        if next == 0 {
            break;
        }
    }
}

pub(super) fn record_botwoon_head(ctx: &mut Execution<'_>) {
    let enemy = usize::from(ctx.current_enemy());
    let cursor = ctx.botwoon_field(enemy, B::HistoryCursor);
    ctx.set_botwoon_history_x(cursor, ctx.botwoon_field(enemy, B::X));
    ctx.set_botwoon_history_y(cursor, ctx.botwoon_field(enemy, B::Y));
}

pub(super) fn follow_botwoon_history(ctx: &mut Execution<'_>) {
    let enemy = usize::from(ctx.current_enemy());
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
                ctx.set_slot(F::Properties, slot, ctx.slot(F::Properties, slot) & !0x2000);
                ctx.set_slot(F::DropFlags, slot, 1);
            } else {
                ctx.set_slot(F::Properties, slot, ctx.slot(F::Properties, slot) | 0x2000);
                ctx.set_slot(F::DropFlags, slot, 2);
            }
            if part == 0 {
                ctx.set_botwoon_field(enemy, B::TransitionPending, 0);
                ctx.set_botwoon_field(enemy, B::TailState, 0);
                ctx.set_botwoon_field(enemy, B::TransitionCursor, 0xffff);
                if ctx.botwoon_field(enemy, B::Pattern) == 0 {
                    ctx.set_botwoon_field(enemy, B::TailState, 1);
                }
            }
        }
        let slot = (projectile >> 1) as isize;
        ctx.set_slot(F::X, slot, ctx.botwoon_history_x(cursor));
        ctx.set_slot(F::Y, slot, ctx.botwoon_history_y(cursor));
        cursor = cursor.wrapping_sub(ctx.botwoon_field(enemy, B::HistorySpacing)) & 0x3ff;
    }
}

pub(super) fn advance_botwoon_history(ctx: &mut Execution<'_>) {
    let enemy = usize::from(ctx.current_enemy());
    let cursor = ctx.botwoon_field(enemy, B::HistoryCursor).wrapping_add(4) & 0x3ff;
    ctx.set_botwoon_field(enemy, B::HistoryCursor, cursor);
}

pub(super) fn orient_botwoon_body(ctx: &mut Execution<'_>) {
    let enemy = usize::from(ctx.current_enemy());
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
                .botwoon_field(enemy, B::X)
                .wrapping_sub(ctx.slot(F::X, slot));
            let y = ctx
                .botwoon_field(enemy, B::Y)
                .wrapping_sub(ctx.slot(F::Y, slot));
            (x, y)
        } else {
            if part == 0 {
                bias = bias.wrapping_add(512);
            }
            let x = ctx.slot(F::X, slot + 1).wrapping_sub(ctx.slot(F::X, slot));
            let y = ctx.slot(F::Y, slot + 1).wrapping_sub(ctx.slot(F::Y, slot));
            (x, y)
        };
        let orientation =
            2 * (i32::from(bias.wrapping_add(ctx.calculate_angle_from_xy(x, y))) >> 5);
        ctx.set_slot(F::BehaviorA, slot, orientation as u16);
    }
}
