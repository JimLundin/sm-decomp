//! Synchronous enemy drop generation and checked definition access.
#![forbid(unsafe_code)]
pub use crate::game::drop_rules::DropSource;
use crate::game::drop_rules::{PICKUPS, Spread, Weights};

use super::pre_helpers::store;
use super::{Execution, SlotField as F};

use crate::types::EnemyDef;
use core::mem::offset_of;

pub(super) fn spawn(definition: u16, index: u16, parameter: u16, ctx: &mut Execution<'_>) {
    ctx.state_mut().spawn.drop_definition = definition;
    {
        super::lifecycle::spawn_from_enemy(ctx, parameter, index, 0xf337);
    }
}

pub(super) fn scatter(source: DropSource, index: u16, parameter: u16, ctx: &mut Execution<'_>) {
    let (definition, spread) = source.pattern();
    let count = match spread {
        Spread::Nearby(count) => count,
        Spread::Area(..) => 16,
    };
    for _ in 0..count {
        match spread {
            Spread::Nearby(_) => {
                // X precedes RNG; Y and the random high byte are live afterward.
                let x = ctx.special_death_item_drop_x_origin_pos();
                let random = { ctx.next_random() };
                ctx.state_mut().spawn.point.x = x.wrapping_add(random & 0x1f).wrapping_sub(16);
                ctx.state_mut().spawn.point.y = ctx
                    .special_death_item_drop_y_origin_pos()
                    .wrapping_add((ctx.random_number() & 0x1f00) >> 8)
                    .wrapping_sub(16);
            }
            Spread::Area(x, y, mask) => {
                ctx.state_mut().spawn.point.x = ({ ctx.next_random() } & mask).wrapping_add(x);
                ctx.state_mut().spawn.point.y =
                    ((ctx.random_number() & 0x3f00) >> 8).wrapping_add(y);
            }
        }
        {
            spawn(definition, index, parameter, ctx);
        }
    }
}

pub(super) fn select(index: u16, ctx: &mut Execution<'_>) -> u16 {
    let definition = ctx.slot(F::DropDefinition, (index >> 1) as isize);
    if definition == 0 {
        return 6;
    }
    let address =
        { ctx.record(0xa0, definition) }.word(offset_of!(EnemyDef, item_drop_chances_ptr));
    if address == 0 {
        return 6;
    }
    let random = loop {
        let value = { ctx.next_random() } as u8;
        if value != 0 {
            break value;
        }
    };
    let health = ctx.samus_reserve_health().wrapping_add(ctx.samus_health());
    if health < 30 {
        {
            let value = 1;
            ctx.set_health_drop_bias_flag(value);
        };
    } else if health >= 50 {
        {
            let value = 0;
            ctx.set_health_drop_bias_flag(value);
        };
    }
    let table = { ctx.record(0xb4, address) };
    let weights = Weights::new(
        ctx.health_drop_bias_flag() as u8 != 0,
        [
            ctx.samus_health() != ctx.samus_max_health()
                || ctx.samus_reserve_health() != ctx.samus_max_reserve_health(),
            ctx.samus_missiles() != ctx.samus_max_missiles(),
            ctx.samus_super_missiles() != ctx.samus_max_super_missiles(),
            ctx.samus_power_bombs() != ctx.samus_max_power_bombs(),
        ],
        |offset| table.byte(offset),
    );
    let mut cumulative = 0u16;
    for (slot, pickup) in PICKUPS.into_iter().enumerate() {
        if weights.enabled(slot) {
            // Each selected row diagnoses and captures the then-current ROM separately.
            let value = { ctx.record(0xb4, address.wrapping_add(slot as u16)) }.byte(0);
            cumulative = cumulative.wrapping_add(weights.contribution(slot, value));
            if cumulative >= u16::from(random) {
                return pickup;
            }
        }
    }
    6
}

fn award(slot: usize, ctx: &mut Execution<'_>) {
    use crate::game::drop_rules::{REWARDS, Resource};

    let (resource, amount, sound) = REWARDS
        [usize::from(ctx.slot(F::BehaviorA, (slot) as isize) >> 1)]
    .expect("non-null function pointer");
    match resource {
        Resource::Health => ctx.restore_health(amount),
        Resource::Missiles => ctx.restore_missiles(amount),
        Resource::SuperMissiles => ctx.restore_super_missiles(amount),
        Resource::PowerBombs => ctx.restore_power_bombs(amount),
    }
    ctx.queue_sound(2, 1, u16::from(sound));
}

/// Advance expiry, then attempt grapple collection before body contact.
pub(super) fn collect(index: u16, ctx: &mut Execution<'_>) {
    use crate::game::contact_rules::axis;
    let slot = usize::from(index >> 1);
    let remaining = ctx.slot(F::BehaviorB, slot as isize).wrapping_sub(1);
    ctx.set_slot(F::BehaviorB, slot as isize, remaining);
    if remaining != 0 {
        // The query precedes the live lifetime and coordinate reads.
        let grapple = { ctx.grapple_pickup_query() } != 0
            && ctx.slot(F::BehaviorB, (slot) as isize).wrapping_sub(384) & 0x8000 != 0
            && ((ctx
                .slot(F::X, (slot) as isize)
                .wrapping_sub(ctx.grapple_beam_end_x_pos()) as i16)
                .wrapping_abs() as u16)
                < 16
            && ((ctx
                .slot(F::Y, (slot) as isize)
                .wrapping_sub(ctx.grapple_beam_end_y_pos()) as i16)
                .wrapping_abs() as u16)
                < 16;
        if !grapple {
            let x_radius = u16::from(ctx.slot_byte(F::Radius, slot as isize * 2));
            let y_radius = u16::from(ctx.slot_byte(F::Radius, slot as isize * 2 + 1));
            if !axis(
                ctx.player_x(),
                ctx.slot(F::X, (index >> 1) as isize),
                ctx.player_x_radius(),
                x_radius,
            ) || !axis(
                ctx.player_y(),
                ctx.slot(F::Y, (index >> 1) as isize),
                ctx.player_y_radius(),
                y_radius,
            ) {
                return;
            }
        }
        award(slot, ctx);
    }
    // Award and sound complete before the original slot changes to its exit script.
    dismiss(slot, ctx);
}

#[forbid(unsafe_code)]
fn dismiss(slot: usize, ctx: &mut Execution<'_>) {
    store(0xeca3, F::Program, (slot) as isize, ctx);
    store(1, F::Delay, (slot) as isize, ctx);
    store(0x3000, F::Properties, (slot) as isize, ctx);
    store(0xefdf, F::PreAction, (slot) as isize, ctx);
}

#[forbid(unsafe_code)]
fn begin_pickup(slot: usize, kind: u16, ctx: &mut Execution<'_>) {
    let code = kind.wrapping_mul(2);
    store(code, F::BehaviorA, (slot) as isize, ctx);
    let instruction = ctx.image().table_word(
        crate::rtl::lorom_offset(0x86ef04) as usize,
        (code >> 1) as isize,
    );
    store(instruction, F::Program, (slot) as isize, ctx);
    store(1, F::Delay, (slot) as isize, ctx);
    store(400, F::BehaviorB, (slot) as isize, ctx);
}

pub(super) fn initialize(index: u16, ctx: &mut Execution<'_>) {
    let slot = usize::from(index >> 1);
    store(ctx.state().spawn.point.x, F::X, (slot) as isize, ctx);
    store(ctx.state().spawn.point.y, F::Y, (slot) as isize, ctx);
    store(0, F::Graphics, (slot) as isize, ctx);
    store(
        ctx.state().spawn.drop_definition,
        F::DropDefinition,
        (slot) as isize,
        ctx,
    );
    let kind = { select(index, ctx) };
    if kind != 0 && kind.wrapping_sub(6) & 0x8000 != 0 {
        begin_pickup(slot, kind, ctx);
        store(0xffff, F::KilledEnemy, (slot) as isize, ctx);
    } else {
        dismiss(slot, ctx);
    }
}

pub(super) fn convert(index: u16, ctx: &mut Execution<'_>) -> u16 {
    let kind = { select(index, ctx) };
    let slot = usize::from(index >> 1);
    // Conversion tests the incoming index, unlike initialization's drop-code test.
    if index != 0 && kind.wrapping_sub(6) & 0x8000 != 0 {
        begin_pickup(slot, kind, ctx);
        store(0xefe0, F::PreAction, (slot) as isize, ctx);
        store(
            ctx.slot(F::Properties, (slot) as isize) & !0x4000,
            F::Properties,
            (slot) as isize,
            ctx,
        );
        ctx.slot(F::Program, (slot) as isize)
    } else {
        // Conversion writes its exit address last.
        store(1, F::Delay, (slot) as isize, ctx);
        store(0x3000, F::Properties, (slot) as isize, ctx);
        store(0xefdf, F::PreAction, (slot) as isize, ctx);
        store(0xeca3, F::Program, (slot) as isize, ctx);
        0xeca3
    }
}

/// Read the original marker before the population respawn effect can mutate slots.
pub(super) fn respawn(index: u16, ctx: &mut Execution<'_>) {
    let marker = ctx.slot(F::KilledEnemy, (index >> 1) as isize);
    if (marker as i16) <= -2 {
        ctx.respawn_enemy(marker & 0x7fff);
    }
}

pub(super) fn reset_room_metadata(ctx: &mut Execution<'_>) {
    // The original 144-word span includes other columns and foreign bytes.
    // Preserve its descending order through the qualified compatibility edge.
    for i in (0..144).rev() {
        ctx.set_slot(F::DropFlags, i, 0);
    }
    for i in (0..18).rev() {
        ctx.set_slot(F::KilledEnemy, i, 0xffff);
    }
}
