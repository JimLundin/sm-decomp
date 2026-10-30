//! Botwoon motion, path programs, attack animation and combat over native fields.
//! Native-field compatibility retains independent body and callback epochs.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorKey as Key};
use crate::actors::api::{DeathHandling, ShotHandling};
use crate::actors::botwoon_death;
use crate::actors::host::{BossEffects as _, Random as _};
use crate::actors::layouts::EnemyLayout104 as Boss;
use crate::actors::{combat, queries};
use crate::cartridge::rom_table_word;
use crate::constants::{bank_86::*, enemy_properties::*};
use crate::ram::RawRam as _;
use crate::ram::{CompatibilityRam as _, WorkRamAccess as _};
use crate::ram::{slot, word};
use sm_data::botwoon_holes::Hole;
use sm_data::botwoon_speeds::BotwoonSpeed;
use sm_data::enemy_speeds::SpeedTable;

/// What Botwoon needs beyond the shared boss effects: its tables, the
/// player angle, the area boss flags, the passage it opens, its drop and
/// the bank $B3 pre-instruction and history hooks.
pub trait Host: crate::actors::host::BossEffects {
    /// The speed level whose two words start at word `index` of $B3:94BB.
    fn botwoon_speed_level(&self, index: isize) -> BotwoonSpeed;
    /// The hole whose four words start at word `index` of $B3:949B.
    fn botwoon_hole(&self, index: isize) -> Hole;
    /// The angle from actor `actor` to Samus.
    fn angle_to_player(&mut self, actor: u16) -> u16;
    /// Set boss bit `mask` for the current area.
    fn set_boss_bit(&mut self, mask: u16);
    /// The boss byte for `area`.
    fn boss_byte(&mut self, area: u16) -> u8;
    fn open_passage(&mut self);
    /// Botwoon's death drop for `area`.
    fn scatter_botwoon_drop(&mut self, area: u16);
    /// Run the bank $B3 pre-instruction at `address` for actor `k`.
    fn b3_pre_instruction(&mut self, address: u32, k: u16);
    fn record_botwoon_head(&mut self);
    fn follow_botwoon_history(&mut self);
    fn orient_botwoon_body(&mut self);
    fn advance_botwoon_history(&mut self);
}

/// The runtime lease Botwoon's death signals and shared reactions run in;
/// `foreign` runs the per-frame steps on the game it leases.
pub trait RuntimeHost:
    crate::actors::effects::ActorHost
    + crate::enemy_projectiles::host::Host
    + crate::room_mechanisms::host::Effects
    + botwoon_death::Host
{
    type Game: Host + ?Sized;
    fn foreign<T>(&mut self, action: impl FnOnce(&mut Self::Game) -> T) -> T;
}

fn initialize_living<H: Host + ?Sized>(e: i32, game: &mut H) {
    slot!(game, Boss[e].var_A = 24);
    loop {
        let k = word!(game.cur_enemy_index);
        game.spawn_projectile(24, k, addr_kEproj_BotwoonsBody as u16);
        slot!(game, Boss[e].var_A, |old| old.wrapping_sub(1));
        let finished = (slot!(game, Boss[e].var_A) as i16).wrapping_sub(1) < 0;
        slot!(game, Boss[e].var_A, |old| old.wrapping_sub(1));
        if finished {
            break;
        }
    }
    slot!(game, Boss[e].base.current_instruction = 0x9389);
    let initial = game.botwoon_speed_level(0);
    slot!(game, Boss[e]; var_3B = 0x9389, var_D = 0x9878, var_E = 0x9bb7, var_F = 0x9dc0, var_20 = 256,
        var_38 = initial.speed, var_C = initial.body_travel_time,
        var_33 = 1, var_34 = 1, var_59 = 1, var_36 = 0xffff, var_37 = 0);
    slot!(game, Boss[e].base.properties, |old| old
        | kEnemyProps_Tangible as u16);
    let health = slot!(game, Boss[e].base.health);
    slot!(game, Boss[e]; var_5B = health, var_5C = health >> 1, var_5D = (health >> 1) >> 1);
    let x = slot!(game, Boss[e].base.x_pos);
    slot!(game, Boss[e]; var_50 = x, var_52 = x, var_54 = x, var_56 = x);
    let y = slot!(game, Boss[e].base.y_pos);
    slot!(game, Boss[e]; var_51 = y, var_53 = y, var_55 = y, var_57 = y,
        var_4E = ((i32::from(slot!(game, Boss[e].base.palette_index)) >> 4) + 256) as u16);
}
pub fn health_palette<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    if slot!(game, Boss[e].var_4F) != 16
        && (slot!(game, Boss[e].base.health) as i16).wrapping_sub(rom_table_word!(
            game,
            0xb3981b,
            (slot!(game, Boss[e].var_4F) >> 1) as isize
        ) as i16)
            < 0
    {
        let mut source = 16u16.wrapping_mul(slot!(game, Boss[e].var_4F));
        let mut target = slot!(game, Boss[e].var_4E);
        loop {
            word!(
                game.palette_buffer[target >> 1] =
                    rom_table_word!(game, 0xb3971b, (source >> 1) as isize)
            );
            source = source.wrapping_add(2);
            target = target.wrapping_add(2);
            if target == 512 {
                break;
            }
        }
        slot!(game, Boss[e].var_4F, |old| old.wrapping_add(2));
    }
}
pub fn wait_entry<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    let delay = slot!(game, Boss[e].var_20).wrapping_sub(1);
    slot!(game, Boss[e].var_20 = delay);
    if delay == 0 {
        slot!(game, Boss[e].var_D = 0x989d);
    }
}
pub fn select_attack<H: Host + ?Sized>(e: i32, game: &mut H) -> bool {
    if slot!(game, Boss[e].var_3C) == 0 {
        return false;
    }
    slot!(game, Boss[e].var_3C = 0);
    let choice = if slot!(game, Boss[e].var_33) == 0
        && slot!(game, Boss[e].var_59) == 0
        && slot!(game, Boss[e].var_3F) == 0
    {
        game.next_random() as u8 & 0xe
    } else {
        0
    };
    slot!(game, Boss[e].var_59 = 0);
    // Masking above admits exactly 0,2,4,6,8,10,12,14.
    let current = i32::from(word!(game.cur_enemy_index));
    if choice <= 4 {
        slot!(game, Boss[current]; var_D = 0x99a4, var_E = 0xe250, var_21 = 0, var_F = 0x9dc0, var_4B = 0);
        select_path(word!(game.cur_enemy_index), game);
    } else {
        slot!(game, Boss[current]; var_D = 0x99e4, var_F = 0x9e7d, var_21 = 48);
        slot!(game, Boss[current].base.properties, |old| old
            & !(kEnemyProps_Tangible as u16));
    }
    true
}
pub fn resume_chase<H: Host + ?Sized>(e: i32, game: &mut H) -> bool {
    if slot!(game, Boss[e].var_3C) == 0 {
        return false;
    }
    slot!(game, Boss[e]; var_3C = 0, var_D = 0x989d, var_E = 0x9bb7);
    if slot!(game, Boss[e].var_33) != 0 {
        slot!(game, Boss[e].var_35 = 0);
    }
    true
}
fn await_spit<R: RuntimeHost>(e: i32, runtime: &mut R) -> bool {
    if runtime.foreign(|game| {
        if slot!(game, Boss[e].var_21) != 0 {
            slot!(game, Boss[e].var_2E = 0);
            true
        } else {
            slot!(game, Boss[e]; var_3C = 0, var_D = 0x99a4, var_E = 0xe250, var_F = 0x9dc0);
            false
        }
    }) {
        return true;
    }
    let direct = botwoon_death::pending(runtime, e as u16)
        || runtime.foreign(|game| {
            let choice = game.next_random() & 1;
            slot!(game, Boss[e]; var_33 = choice, var_34 = choice);
            slot!(game, Boss[e].var_34) == 0
        });
    runtime.foreign(|game| {
        if direct {
            slot!(game, Boss[e].var_4B = 0);
        } else {
            slot!(game, Boss[e]; var_35 = 0, var_36 = 0xffff);
        }
        select_path(word!(game.cur_enemy_index), game);
    });
    false
}
pub fn wait_collapse<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    let delay = slot!(game, Boss[e].var_22).wrapping_add(1);
    slot!(game, Boss[e].var_22 = delay);
    if (i32::from(delay) - 256) & 0x8000 == 0 {
        slot!(game, Boss[e].var_D = 0x9a5e);
    }
}
pub fn fall<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    let index = (8 * i32::from(slot!(byte game, Boss[e].var_28[1])) >> 1) as isize;
    let high = game.enemy_speed(SpeedTable::Quadratic, index + 1);
    let low = game.enemy_speed(SpeedTable::Quadratic, index);
    slot!(
        game,
        Boss[e].base(y_pos, y_subpos) += (u32::from(high) << 16) | u32::from(low)
    );
    if (i32::from(slot!(game, Boss[e].base.y_pos)) - 200) & 0x8000 != 0 {
        slot!(game, Boss[e].var_28, |old| old.wrapping_add(192));
    } else {
        slot!(game, Boss[e].base.y_pos = 200);
        slot!(game, Boss[e].var_D = 0x9aca);
        let x = slot!(game, Boss[e].base.x_pos);
        let y = slot!(game, Boss[e].base.y_pos);
        game.set_projectile_spawn_point(x, y);
        game.spawn_room_projectile(addr_kEproj_DustCloudExplosion as u16, 0x1d);
        game.queue_sound(1, 0x24, 6);
        slot!(game, Boss[e].base.properties, |old| old | 0x500);
    }
}
pub fn rubble<H: Host + ?Sized>(k: u16, game: &mut H) {
    let e = i32::from(k);
    if (i32::from(slot!(game, Boss[e].var_23)) - 192) & 0x8000 != 0 {
        if (i32::from(slot!(game, Boss[e].var_23)) - 64) & 0x8000 == 0 {
            let explosion = (slot!(game, Boss[e].var_24) as i16).wrapping_sub(1);
            slot!(game, Boss[e].var_24 = explosion as u16);
            if explosion < 0 {
                slot!(game, Boss[e].var_24 = 12);
                let x = ((game.next_random() & 0x1f) + 232) as u16;
                let y = (i32::from(slot!(game, Boss[e].var_23))
                    + i32::from(game.next_random() & 0x1f)
                    - 8) as u16;
                game.create_sprite(x, y, 29, 2560);
                game.queue_sound(1, 0x24, 6);
            }
            let dust = (slot!(game, Boss[e].var_25) as i16).wrapping_sub(1);
            slot!(game, Boss[e].var_25 = dust as u16);
            if dust < 0 {
                slot!(game, Boss[e].var_25 = 4);
                for _ in 0..2 {
                    let x = ((game.next_random() & 0x3f) + 224) as u16;
                    let y = (i32::from(slot!(game, Boss[e].var_23))
                        + (i32::from(game.next_random() & 0x1f) - 8))
                        as u16;
                    game.create_sprite(x, y, 9, 2560);
                }
            }
        }
        slot!(game, Boss[e].var_23, |old| old.wrapping_add(1));
    } else {
        slot!(game, Boss[e].base.properties, |old| old
            | kEnemyProps_Deleted as u16);
        game.set_boss_bit(2);
        game.queue_music_delayed(3);
    }
}
fn mark_hit<R: RuntimeHost>(h: &mut R, actor: Key) {
    if h.resolve_actor(actor, Native).health == 0 {
        h.set_native_death_signal(actor.byte_index(), botwoon_death::Signal::PendingHit, 1);
        h.edit_actor(Key::from_legacy(0), Native, |actor| {
            actor.properties |= kEnemyProps_Tangible as u16;
        });
    }
}
pub fn shot<R: RuntimeHost>(h: &mut R) {
    let actor = h.selected_actor();
    let health = h.resolve_actor(actor, Native).health;
    h.set_native_death_signal(
        actor.byte_index(),
        botwoon_death::Signal::PreviousHealth,
        health,
    );
    combat::shot_reaction(h, ShotHandling::ReactionOnly);
    mark_hit(h, actor);
}
pub fn contact<R: RuntimeHost>(power_bomb: bool, h: &mut R) {
    if power_bomb {
        combat::power_bomb_reaction(h, DeathHandling::LeavePending);
    } else {
        combat::contact_reaction(h, DeathHandling::LeavePending);
    }
    let actor = h.selected_actor();
    mark_hit(h, actor);
}
fn select_path<H: Host + ?Sized>(k: u16, game: &mut H) {
    let e = i32::from(k);
    if slot!(game, Boss[e].var_33) == 0 {
        slot!(game, Boss[e].var_3F = 0);
        let health = slot!(game, Boss[e].base.health) as i16;
        if health > 0 && health.wrapping_sub(slot!(game, Boss[e].var_5C) as i16) < 0 {
            let speed = if health.wrapping_sub(slot!(game, Boss[e].var_5D) as i16) < 0 {
                2
            } else {
                1
            };
            slot!(game, Boss[e].var_3F = speed);
            let index = (4u16.wrapping_mul(slot!(game, Boss[e].var_3F)) >> 1) as isize;
            let level = game.botwoon_speed_level(index);
            slot!(game, Boss[e]; var_38 = level.speed, var_C = level.body_travel_time);
        }
    }
    let bias = if slot!(game, Boss[e].var_33) != 0 {
        128
    } else {
        0
    };
    slot!(
        game,
        Boss[e].var_40 = (i32::from(game.next_random() & 0x18)
            + bias
            + 4 * i32::from(slot!(game, Boss[e].var_37))) as u16
    );
}
fn clamp_delta(value: u16) -> u16 {
    if (i32::from(value) - 256) & 0x8000 == 0 {
        255
    } else if (i32::from(value) + 256) & 0x8000 != 0 {
        (-255i32) as u16
    } else {
        value
    }
}
pub fn seek_hole<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    let target = game.botwoon_hole((slot!(game, Boss[e].var_37) >> 1) as isize);
    let x = clamp_delta(
        (i32::from(target.left) + 4 - i32::from(slot!(game, Boss[e].base.x_pos))) as u16,
    );
    let y = clamp_delta(
        (i32::from(target.top) + 4 - i32::from(slot!(game, Boss[e].base.y_pos))) as u16,
    );
    let angle = queries::angle_from_delta(x, y);
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Boss[e]; var_39 = angle, var_3A = u16::from(64u8.wrapping_sub(angle as u8)));
    if slot!(game, Boss[e].var_33) == slot!(game, Boss[e].var_34) {
        apply_direction(game);
    } else {
        slot!(game, Boss[e]; var_34 = slot!(game, Boss[e].var_33), var_3C = 1);
    }
}
fn apply_direction<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    let phase = slot!(game, Boss[e].var_3A);
    let amplitude = slot!(game, Boss[e].var_38);
    let vector = game.actor_direction_vector(phase, amplitude);
    let scratch = slot!(game, Boss[e].var_3A);
    game.set_enemy_scratch(scratch);
    let phase = slot!(game, Boss[e].var_3A);
    game.apply_vector(phase, vector);
}
pub fn load_path<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Boss[e]; var_E = 0xe28c, var_3C = 0);
    let record = crate::cartridge::native_record(
        0xb3,
        slot!(game, Boss[e].var_40).wrapping_add(0xe150),
        game,
    );
    slot!(game, Boss[e]; var_42 = record.word(0), var_37 = record.word(4));
    let direction = record.word(2) as i16;
    slot!(game, Boss[e].var_44 = direction as u16);
    if direction < 0 {
        slot!(game, Boss[e].var_42, |old| old.wrapping_sub(4));
    }
    advance_path(game);
}
pub fn advance_path<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    let mut delta = [0u16; 2];
    let mut remaining = i32::from(slot!(game, Boss[e].var_38));
    let step = if slot!(game, Boss[e].var_44) & 0x8000 == 0 {
        2
    } else {
        -2
    };
    loop {
        let record = crate::cartridge::native_record(0xb3, slot!(game, Boss[e].var_42), game);
        for (axis, total) in delta.iter_mut().enumerate() {
            let amount = (record.byte(axis) as i8) as i16 as u16;
            if amount == 0xff80 {
                slot!(game, Boss[e]; var_41 = 0, var_3C = 1);
                return;
            }
            *total = total.wrapping_add(amount);
        }
        slot!(
            game,
            Boss[e].var_42 = (i32::from(slot!(game, Boss[e].var_42)) + step) as u16
        );
        remaining -= 1;
        if remaining == 0 {
            break;
        }
    }
    if slot!(game, Boss[e].var_44) & 0x8000 != 0 {
        delta = delta.map(u16::wrapping_neg);
    }
    slot!(game, Boss[e].base.x_pos, |old| old.wrapping_add(delta[0]));
    slot!(game, Boss[e].base.y_pos, |old| old.wrapping_add(delta[1]));
}
fn animation<H: Host + ?Sized>(e: i32, instruction: u16, game: &mut H) {
    slot!(game, Boss[e].base.current_instruction = instruction);
    slot!(game, Boss[e].var_3B = instruction);
    slot!(game, Boss[e].base; instruction_timer = 1, timer = 0);
}
pub fn animate_motion<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    let x = slot!(game, Boss[e].base.x_pos).wrapping_sub(slot!(game, Boss[e].var_56));
    let y = slot!(game, Boss[e].base.y_pos).wrapping_sub(slot!(game, Boss[e].var_57));
    let angle = queries::angle_from_delta(x, y);
    if x != 0 || y != 0 {
        let instruction = if slot!(game, Boss[e].var_33) != 0 {
            slot!(game, Boss[e].base.layer = 7);
            slot!(game, Boss[e].base.properties, |old| old
                | kEnemyProps_Tangible as u16);
            0x9389
        } else {
            slot!(game, Boss[e].base.layer = 2);
            slot!(game, Boss[e].base.properties, |old| old
                & !(kEnemyProps_Tangible as u16));
            slot!(
                game,
                Boss[e].var_45 = (i32::from(slot!(game, Boss[e].var_45))
                    + i32::from((angle as i8).wrapping_sub(slot!(game, Boss[e].var_45) as i8)) * 3
                        / 4) as u8 as u16
            );
            rom_table_word!(game, 0xb3946b, (slot!(game, Boss[e].var_45) >> 5) as isize)
        };
        if instruction != slot!(game, Boss[e].var_3B) {
            animation(e, instruction, game);
        }
    }
    slot!(game, Boss[e]; var_56 = slot!(game, Boss[e].var_54), var_57 = slot!(game, Boss[e].var_55),
        var_54 = slot!(game, Boss[e].var_52), var_55 = slot!(game, Boss[e].var_53),
        var_52 = slot!(game, Boss[e].var_50), var_53 = slot!(game, Boss[e].var_51),
        var_50 = slot!(game, Boss[e].base.x_pos), var_51 = slot!(game, Boss[e].base.y_pos));
}
pub fn aim_spit<H: Host + ?Sized>(game: &mut H) -> u32 {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Boss[e].base.layer = 2);
    let actor = word!(game.cur_enemy_index);
    let angle = game.angle_to_player(actor);
    slot!(game, Boss[e].var_3D = angle);
    let instruction = rom_table_word!(
        game,
        0xb3948b,
        ((angle as u8).wrapping_add(16) >> 5) as isize
    );
    animation(e, instruction, game);
    slot!(game, Boss[e]; var_3D = u16::from(64u8.wrapping_sub(slot!(game, Boss[e].var_3D) as u8)), var_F = 0x9ee0);
    if slot!(game, Boss[e].var_D) != 0x99e4 {
        slot!(game, Boss[e].var_F = 0x9f34);
    }
    0xb30000 | u32::from(slot!(game, Boss[e].var_F))
}
pub fn spit<H: Host + ?Sized>(wide: bool, game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    if wide && slot!(game, Boss[e].var_5A) == 0 {
        return;
    }
    game.ram().write_byte(
        0x60b,
        (slot!(game, Boss[e].var_3D) as u8).wrapping_sub(if wide { 32 } else { 16 }),
    );
    let speed = [2u16, 3, 4][slot!(game, Boss[e].var_3F) as usize];
    for _ in 0..if wide { 5 } else { 3 } {
        let k = word!(game.cur_enemy_index);
        game.spawn_projectile(speed, k, addr_kEproj_BotwoonsSpit as u16);
        game.ram()
            .write_byte(0x60b, game.ram().read_byte(0x60b).wrapping_add(16));
    }
    let target = if wide {
        slot!(game, Boss[e].var_5A = 0);
        e
    } else {
        i32::from(word!(game.cur_enemy_index))
    };
    slot!(game, Boss[target].var_F = 0x9f7a);
}
pub fn wait_spit<H: Host + ?Sized>(k: u16, game: &mut H) {
    let e = i32::from(k);
    let delay = (slot!(game, Boss[e].var_21) as i16).wrapping_sub(1);
    slot!(game, Boss[e].var_21 = delay as u16);
    if delay < 0 {
        slot!(game, Boss[e]; var_21 = 0, var_F = 0x9dc0);
    }
}
pub fn detect_portal<H: Host + ?Sized>(k: u16, game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    if slot!(game, Boss[i32::from(k)].var_35) != 0 {
        return;
    }
    for index in [12, 8, 4, 0] {
        let hole = game.botwoon_hole(index);
        let bounds = [hole.left, hole.right, hole.top, hole.bottom];
        if (0..4).all(|offset| {
            let coordinate = if offset < 2 {
                slot!(game, Boss[e].base.x_pos)
            } else {
                slot!(game, Boss[e].base.y_pos)
            };
            ((coordinate as i16).wrapping_sub(bounds[offset] as i16) >= 0) == (offset % 2 == 0)
        }) {
            slot!(game, Boss[e].var_35 = 1);
            slot!(game, Boss[e].var_33, |old| old ^ 1);
            slot!(game, Boss[e].var_36 = slot!(game, Boss[e].var_B));
            slot!(game, Boss[e].var_26, |old| old.wrapping_sub(1));
            return;
        }
        slot!(game, Boss[e].var_35 = 0);
    }
}
pub fn set_hitbox<H: Host + ?Sized>(x: u16, y: u16, game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Boss[e].base; x_width = x, y_height = y);
}
pub fn set_spitting<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Boss[e].var_5A = 1);
}

pub fn initialize<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    if game.boss_byte(4) & 2 != 0 {
        game.spawn_plm(15, 4, 0xb797);
        game.open_passage();
        slot!(game, Boss[e].base.current_instruction = 0x9389);
        slot!(game, Boss[e].base.properties, |old| old
            | kEnemyProps_Deleted as u16);
    } else {
        initialize_living(e, game);
    }
}
pub fn update<R: RuntimeHost>(runtime: &mut R) {
    botwoon_death::arm(runtime);
    let actor = runtime.selected_actor();
    let state = 0xb30000 | u32::from(runtime.resolve_actor(actor, Native).variables[3]);
    let k = runtime.selected_actor().byte_index();
    match state {
        11769976 => runtime.foreign(|game| wait_entry(game)),
        11770013 | 11770276 => runtime.foreign(|game| {
            let e = i32::from(word!(game.cur_enemy_index));
            let changed = if state == 11770013 {
                select_attack(e, game)
            } else {
                resume_chase(e, game)
            };
            if !changed {
                step_body(e, game);
                if state == 11770013 {
                    detect_portal(word!(game.cur_enemy_index), game);
                }
            }
        }),
        11770340 => {
            let e = i32::from(runtime.selected_actor().byte_index());
            if await_spit(e, runtime) {
                runtime.foreign(|game| {
                    let address = 0xb30000 | u32::from(slot!(game, Boss[e].var_F));
                    let k = word!(game.cur_enemy_index);
                    game.b3_pre_instruction(address, k);
                });
            }
        }
        11770438 => runtime.foreign(|game| wait_collapse(game)),
        11770462 => runtime.foreign(|game| fall(game)),
        11770570 => {
            let actor = runtime.selected_actor();
            if botwoon_death::landed(runtime, actor.byte_index()) {
                runtime.edit_actor(actor, Native, |actor| actor.variables[3] = 0x9af9);
                let k = runtime.selected_actor().byte_index();
                runtime.foreign(|game| {
                    game.spawn_plm(15, 4, 0xb79b);
                    game.scatter_botwoon_drop(k);
                    slot!(game, Boss[i32::from(k)]; var_23 = 0, var_24 = 0, var_25 = 0);
                });
            }
        }
        11770617 => runtime.foreign(|game| rubble(k, game)),
        _ => {
            runtime.foreign(|game| game.unreachable());
        }
    }
    runtime.foreign(|game| health_palette(game));
}
fn step_body<H: Host + ?Sized>(e: i32, game: &mut H) {
    match 0xb30000 | u32::from(slot!(game, Boss[e].var_E)) {
        11770807 => seek_hole(game),
        11788940 => advance_path(game),
        11788880 => load_path(game),
        _ => {
            game.unreachable();
        }
    }
    game.record_botwoon_head();
    game.follow_botwoon_history();
    game.b3_pre_instruction(
        0xb30000 | u32::from(slot!(game, Boss[e].var_F)),
        word!(game.cur_enemy_index),
    );
    game.orient_botwoon_body();
    game.advance_botwoon_history();
}
