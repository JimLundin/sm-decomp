//! Synchronous enemy drop generation and checked definition access.
#![forbid(unsafe_code)]
use super::access::{enemy, enemy_byte};
use super::host::{DefinitionField, Host, LoadedRecord};
pub use crate::drop_rules::DropSource;
use crate::drop_rules::{PICKUPS, Spread, Weights};

pub fn spawn(definition: u16, index: u16, h: &mut impl Host) {
    h.session_mut().enemy_projectiles.spawn.drop_definition = definition;
    {
        super::lifecycle::spawn_from_enemy(
            h,
            h.session().compatibility_enemy_scratch,
            index,
            0xf337,
        );
    }
}

pub fn scatter(source: DropSource, index: u16, h: &mut impl Host) {
    let (definition, spread) = source.pattern();
    let count = match spread {
        Spread::Nearby(count) => count,
        Spread::Area(..) => 16,
    };
    for _ in 0..count {
        match spread {
            Spread::Nearby(_) => {
                // X precedes RNG; Y and the random high byte are live afterward.
                let x = h.inputs().drop_origin_x;
                let random = { h.next_random() };
                h.session_mut().enemy_projectiles.spawn.point.x =
                    x.wrapping_add(random & 0x1f).wrapping_sub(16);
                h.session_mut().enemy_projectiles.spawn.point.y = h
                    .inputs()
                    .drop_origin_y
                    .wrapping_add((h.inputs().random & 0x1f00) >> 8)
                    .wrapping_sub(16);
            }
            Spread::Area(x, y, mask) => {
                h.session_mut().enemy_projectiles.spawn.point.x =
                    ({ h.next_random() } & mask).wrapping_add(x);
                h.session_mut().enemy_projectiles.spawn.point.y =
                    ((h.inputs().random & 0x3f00) >> 8).wrapping_add(y);
            }
        }
        {
            spawn(definition, index, h);
        }
    }
}

pub fn select(index: u16, h: &mut impl Host) -> u16 {
    let definition = enemy!(h, (index >> 1) as isize, drop_definition);
    if definition == 0 {
        return 6;
    }
    let address =
        { h.loaded_record(0xa0, definition) }.definition_word(DefinitionField::EnemyDropTable);
    if address == 0 {
        return 6;
    }
    let random = loop {
        let value = { h.next_random() } as u8;
        if value != 0 {
            break value;
        }
    };
    let health = h
        .session()
        .inventory
        .reserve_health
        .wrapping_add(h.session().inventory.health.current);
    if health < 30 {
        {
            let value = 1;
            h.session_mut().enemy_projectiles.drop_bias = value;
        };
    } else if health >= 50 {
        {
            let value = 0;
            h.session_mut().enemy_projectiles.drop_bias = value;
        };
    }
    let table = { h.loaded_record(0xb4, address) };
    let weights = Weights::new(
        h.session().enemy_projectiles.drop_bias as u8 != 0,
        [
            h.session().inventory.health.current != h.session().inventory.health.maximum
                || h.session().inventory.reserve_health != h.session().inventory.reserve_capacity,
            h.session().inventory.ammunition[0].current
                != h.session().inventory.ammunition[0].maximum,
            h.session().inventory.ammunition[1].current
                != h.session().inventory.ammunition[1].maximum,
            h.session().inventory.ammunition[2].current
                != h.session().inventory.ammunition[2].maximum,
        ],
        |offset| table.byte(offset as isize),
    );
    let mut cumulative = 0u16;
    for (slot, pickup) in PICKUPS.into_iter().enumerate() {
        if weights.enabled(slot) {
            // Each selected row diagnoses and captures the then-current ROM separately.
            let value = { h.loaded_record(0xb4, address.wrapping_add(slot as u16)) }.byte(0);
            cumulative = cumulative.wrapping_add(weights.contribution(slot, value));
            if cumulative >= u16::from(random) {
                return pickup;
            }
        }
    }
    6
}

fn award(slot: usize, h: &mut impl Host) {
    use crate::drop_rules::REWARDS;

    let (resource, amount, sound) = REWARDS
        [usize::from(enemy!(h, (slot) as isize, behavior_a) >> 1)]
    .expect("non-null function pointer");
    h.session_mut().inventory.restore(resource, amount);
    h.queue_sound(2, 1, u16::from(sound));
}

/// Advance expiry, then attempt grapple collection before body contact.
pub fn collect(index: u16, h: &mut impl Host) {
    use crate::contact_rules::axis;
    let slot = usize::from(index >> 1);
    let remaining = enemy!(h, slot as isize, behavior_b).wrapping_sub(1);
    enemy!(h, slot as isize, behavior_b = remaining);
    if remaining != 0 {
        // The query precedes the live lifetime and coordinate reads.
        let grapple = { h.grapple_pickup_query() } != 0
            && enemy!(h, (slot) as isize, behavior_b).wrapping_sub(384) & 0x8000 != 0
            && ((enemy!(h, (slot) as isize, x).wrapping_sub(h.session().player.grapple_endpoint.x)
                as i16)
                .wrapping_abs() as u16)
                < 16
            && ((enemy!(h, (slot) as isize, y).wrapping_sub(h.session().player.grapple_endpoint.y)
                as i16)
                .wrapping_abs() as u16)
                < 16;
        if !grapple {
            let x_radius = u16::from(enemy_byte!(h, slot as isize * 2, radius));
            h.session_mut().compatibility_enemy_scratch = x_radius;
            let y_radius = u16::from(enemy_byte!(h, slot as isize * 2 + 1, radius));
            if !axis(
                h.session().player.x,
                enemy!(h, (index >> 1) as isize, x),
                h.session().player.contact.x_radius,
                x_radius,
            ) || !axis(
                h.session().player.y,
                enemy!(h, (index >> 1) as isize, y),
                h.session().player.contact.radius,
                y_radius,
            ) {
                return;
            }
        }
        award(slot, h);
    }
    // Award and sound complete before the original slot changes to its exit script.
    dismiss(slot, h);
}

#[forbid(unsafe_code)]
fn dismiss(slot: usize, h: &mut impl Host) {
    enemy!(h, 0xeca3 => (slot) as isize, program);
    enemy!(h, 1 => (slot) as isize, delay);
    enemy!(h, 0x3000 => (slot) as isize, properties);
    enemy!(h, 0xefdf => (slot) as isize, pre_action);
}

#[forbid(unsafe_code)]
fn begin_pickup(slot: usize, kind: u16, h: &mut impl Host) {
    let code = kind.wrapping_mul(2);
    enemy!(h, code => (slot) as isize, behavior_a);
    let instruction = h
        .loaded_record(0x86, 0xef04)
        .native_word((code >> 1) as isize * 2);
    enemy!(h, instruction => (slot) as isize, program);
    enemy!(h, 1 => (slot) as isize, delay);
    enemy!(h, 400 => (slot) as isize, behavior_b);
}

pub fn initialize(index: u16, h: &mut impl Host) {
    let slot = usize::from(index >> 1);
    enemy!(h, h.session().enemy_projectiles.spawn.point.x => (slot) as isize, x);
    enemy!(h, h.session().enemy_projectiles.spawn.point.y => (slot) as isize, y);
    enemy!(h, 0 => (slot) as isize, graphics);
    enemy!(h, h.session().enemy_projectiles.spawn.drop_definition => (slot) as isize, drop_definition);
    let kind = { select(index, h) };
    if kind != 0 && kind.wrapping_sub(6) & 0x8000 != 0 {
        begin_pickup(slot, kind, h);
        enemy!(h, 0xffff => (slot) as isize, killed_enemy);
    } else {
        dismiss(slot, h);
    }
}

pub fn convert(index: u16, h: &mut impl Host) -> u16 {
    let kind = { select(index, h) };
    let slot = usize::from(index >> 1);
    // Conversion tests the incoming index, unlike initialization's drop-code test.
    if index != 0 && kind.wrapping_sub(6) & 0x8000 != 0 {
        begin_pickup(slot, kind, h);
        enemy!(h, 0xefe0 => (slot) as isize, pre_action);
        enemy!(h, enemy!(h, (slot) as isize, properties) & !0x4000 => (slot) as isize, properties);
        enemy!(h, (slot) as isize, program)
    } else {
        // Conversion writes its exit address last.
        enemy!(h, 1 => (slot) as isize, delay);
        enemy!(h, 0x3000 => (slot) as isize, properties);
        enemy!(h, 0xefdf => (slot) as isize, pre_action);
        enemy!(h, 0xeca3 => (slot) as isize, program);
        0xeca3
    }
}

/// Read the original marker before the population respawn effect can mutate slots.
pub fn respawn(index: u16, h: &mut impl Host) {
    let marker = enemy!(h, (index >> 1) as isize, killed_enemy);
    if (marker as i16) <= -2 {
        h.respawn_enemy(marker & 0x7fff);
    }
}

pub fn reset_room_metadata(h: &mut impl Host) {
    // The original 144-word span includes other columns and foreign bytes.
    // Preserve its descending order through the qualified compatibility edge.
    for i in (0..144).rev() {
        enemy!(h, i, drop_flags = 0);
    }
    for i in (0..18).rev() {
        enemy!(h, i, killed_enemy = 0xffff);
    }
}
