//! Shared pre-action behavior and numeric dispatch. No legacy projectile callbacks.
#![forbid(unsafe_code)]
use super::access::{enemy, enemy_add, enemy_byte, shot_word};
use super::addresses;
use super::host::{EnemyMember as SlotField, Host};
use super::pre_actions::*;
use super::pre_helpers::*;
use super::shared::spawn_with_room_gfx;
use super::*;
use crate::actors::access::{AccessMode::LittleEndian as Le, ActorKey as Key};
use crate::geometry::Point;
use crate::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;

#[derive(Clone, Copy)]
pub struct CollisionRect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

pub use super::pre_actions::kraid_rocks_use_palette_0_pre_instruction as use_palette_0;
pub use super::pre_actions::mini_crocomire_projectile_move_left as magdollite_lava_move_left;
pub use super::pre_actions::polyp_rock_move_x as mini_crocomire_projectile_move_right;
pub use super::pre_actions::polyp_rock_move_x as magdollite_lava_move_right;

pub fn plant_pre<H: Host>(index: usize, k: u16, ctx: &mut H) {
    let actions: [fn(u16, &mut H); 10] = [
        cacatac_spike_left,
        cacatac_spike_up,
        cacatac_spike_right,
        cacatac_spike_left,
        cacatac_spike_down,
        cacatac_spike_right,
        cacatac_spike_up_left,
        cacatac_spike_up_right,
        cacatac_spike_down_left,
        cacatac_spike_down_right,
    ];
    actions[index](k, ctx);
}

pub fn x_velocity_at_angle(a: u16, angle: u16) -> u16 {
    multiply_by_sine(a, angle)
}
pub fn y_velocity_at_angle(a: u16, angle: u16) -> u16 {
    multiply_by_sine(a, angle.wrapping_add(64))
}

pub fn dispatch_polyp_rock_function(ea: u32, k: u16, ctx: &mut impl Host) {
    use addresses::functions::*;
    match ea {
        POLYP_ROCK_RISE => polyp_rock_rise(k, ctx),
        POLYP_ROCK_FALL => polyp_rock_fall(k, ctx),
        _ => ctx.unreachable(),
    }
}

pub fn dispatch_mother_brain_tube_falling_function(ea: u32, k: u16, ctx: &mut impl Host) {
    use addresses::functions::*;
    match ea {
        FALLING_TUBE_BURST => falling_tube_burst(k, ctx),
        FALLING_TUBE_FALL => falling_tube_fall(k, ctx),
        _ => ctx.unreachable(),
    }
}

pub fn dispatch_namihe_fune_fireball_function(ea: u32, k: u16, ctx: &mut impl Host) {
    use addresses::functions::*;
    match ea {
        NAMIHE_FUNE_FIREBALL_MOVE_X1 => namihe_fune_fireball_move_left(k, ctx),
        NAMIHE_FUNE_FIREBALL_MOVE_X2 => namihe_fune_fireball_move_right(k, ctx),
        _ => ctx.unreachable(),
    }
}

pub fn dispatch_magdollite_lava_function(ea: u32, k: u16, ctx: &mut impl Host) {
    use addresses::functions::*;
    match ea {
        LAVA_THROWN_BY_MAGDOLLITE_MOVE_X1 => magdollite_lava_move_left(k, ctx),
        LAVA_THROWN_BY_MAGDOLLITE_MOVE_X2 => magdollite_lava_move_right(k, ctx),
        _ => ctx.unreachable(),
    }
}

pub fn dispatch_botwoon_body_function(ea: u32, k: u16, ctx: &mut impl Host) {
    use addresses::functions::*;
    match ea {
        BOTWONS_BODY_FUNCTION_DYING => botwoon_body_dying_set_delay(k, ctx),
        BOTWONS_BODY_FUNCTION_DYING2 => botwoon_body_dying_waiting(k, ctx),
        BOTWONS_BODY_FUNCTION_DYING_FALLING => botwoon_body_dying_falling(k, ctx),
        BOTWOONS_BODY_MAIN => botwoon_body_main(k, ctx),
        NONE_EB93 => (),
        _ => ctx.unreachable(),
    }
}

/// Whether the projectile is outside the 256-pixel camera view.
/// Was `EprojPreInstr_NorfairLavaquakeRocks_Inner2` (also `EprojPreInstrHelper_DBF2_Func2`).
pub fn outside_camera_view(k: u16, ctx: &mut impl Host) -> u16 {
    let v1: i32 = k as i32 >> 1;
    (((enemy!(ctx, (v1) as isize, x) as i32 - ctx.inputs().camera_x as i32) as i16) < 0
        || ((ctx.inputs().camera_x as i32 + 256 - enemy!(ctx, (v1) as isize, x) as i32) as i16) < 0
        || ((enemy!(ctx, (v1) as isize, y) as i32 - ctx.inputs().camera_y as i32) as i16) < 0
        || ((ctx.inputs().camera_y as i32 + 256 - enemy!(ctx, (v1) as isize, y) as i32) as i16) < 0)
        as u16
}

/// Steps a turret bullet's direction. Was `Eproj_MotherBrainRoomTurretBullets_Func2`.
pub fn turret_bullet_step_direction(v0: u16, ctx: &mut impl Host) {
    let r20: u8 = ((enemy_byte!(ctx, v0 as isize + 1, y_fraction) as i32
        + enemy_byte!(ctx, v0 as isize, y_fraction) as i32)
        & 7) as u8;
    let r18: u16 = enemy!(ctx, (v0 >> 1) as isize, x_fraction);
    if turret_direction(r18, (r20 as i32 & 7) as isize, ctx) != 0 {
        enemy_byte!(ctx, v0 as isize, y_fraction = r20);
    } else {
        let v1: i8 = -(enemy_byte!(ctx, v0 as isize + 1, y_fraction) as i32) as i8;
        enemy_byte!(ctx, v0 as isize + 1, y_fraction = v1 as u8);
        {
            let old = enemy_byte!(ctx, v0 as isize, y_fraction);
            enemy_byte!(
                ctx,
                v0 as isize,
                y_fraction = (old as i32 + v1 as i32) as u8
            );
        }
    };
}

pub fn is_turret_on_screen(k: u16, ctx: &mut impl Host) -> u8 {
    let v1: i32 = k as i32 >> 1;
    let mut v2: i16 = enemy!(ctx, (v1) as isize, y) as i16;
    let result: u8 = 1;
    if v2 >= 0 {
        let mut v3: i16 = (v2 as i32 + 16 - ctx.inputs().camera_y as i32) as i16;
        if v3 >= 0 && (v3 as i32 - 256i32) & 0x8000 != 0 {
            let mut v4: i16 = enemy!(ctx, (v1) as isize, x) as i16;
            if v4 >= 0 {
                let mut v5: i16 = (v4 as i32 + 4 - ctx.inputs().camera_x as i32) as i16;
                if v5 >= 0 && (v5 as i32 - 264i32) & 0x8000 != 0 {
                    return 0;
                }
            }
        }
    }
    result
}

pub fn check_for_bomb_collision_with_rectangle(rect: CollisionRect, ctx: &mut impl Host) -> u8 {
    if ctx.session().projectiles.bomb_count == 0 {
        return 0;
    }
    for v1 in (10i32..20).step_by(2) {
        let v2: i32 = v1 >> 1;
        if (shot_word(ctx, super::host::ShotMember::Kind, (v2 as u16) as isize) & 0xf00) == 1280
            && shot_word(ctx, super::host::ShotMember::Variable, (v2) as isize) == 0
        {
            let x: u16 = (((shot_word(ctx, super::host::ShotMember::X, (v2) as isize) as i32
                - rect.x as i32) as u16) as i16)
                .unsigned_abs();
            if x < shot_word(ctx, super::host::ShotMember::XRadius, (v2) as isize)
                || ((x as i32
                    - shot_word(ctx, super::host::ShotMember::XRadius, (v2) as isize) as i32)
                    as u16)
                    < rect.w
            {
                let y: u16 = (((shot_word(ctx, super::host::ShotMember::Y, (v2) as isize) as i32
                    - rect.y as i32) as u16) as i16)
                    .unsigned_abs();
                if y < shot_word(ctx, super::host::ShotMember::YRadius, (v2) as isize)
                    || ((y as i32
                        - shot_word(ctx, super::host::ShotMember::YRadius, (v2) as isize) as i32)
                        as u16)
                        < rect.h
                {
                    return 1;
                }
            }
        }
    }
    0
}

pub fn check_for_enemy_collision_with_rectangle(
    k: u16,
    rect: CollisionRect,
    ctx: &mut impl Host,
) -> u8 {
    let v1 = k;
    (((((ctx.resolve_actor(Key::from_legacy(v1), Le).x.whole as i32 - rect.x as i32) as u16) as i16)
        .unsigned_abs() as i32
        - ctx.resolve_actor(Key::from_legacy(v1), Le).radius.x as i32)
        < rect.w as i32
        && ((((ctx.resolve_actor(Key::from_legacy(v1), Le).y.whole as i32 - rect.y as i32) as u16)
            as i16)
            .unsigned_abs() as i32
            - ctx.resolve_actor(Key::from_legacy(v1), Le).radius.y as i32)
            < rect.h as i32) as u8
}

pub fn check_for_collision_with_samus(v0: u16, ctx: &mut impl Host) -> u8 {
    let rect: CollisionRect = set_up_collision_detection(v0, ctx);
    let v1: u16 = (((ctx.session().player.x as i32 - rect.x as i32) as u16) as i16).unsigned_abs();
    let mut v2: bool = v1 < ctx.session().player.contact.x_radius;
    let v3: u16 = (v1 as i32 - ctx.session().player.contact.x_radius as i32) as u16;
    let result: u8 = 0;
    if v2 || v3 < rect.w {
        let v4: u16 =
            (((ctx.session().player.y as i32 - rect.y as i32) as u16) as i16).unsigned_abs();
        v2 = v4 < ctx.session().player.contact.radius;
        let v5: u16 = (v4 as i32 - ctx.session().player.contact.radius as i32) as u16;
        if v2 || v5 < rect.h {
            return 1;
        }
    }
    result
}

pub fn multiply_by_sine(r38: u16, a: u16) -> u16 {
    let r46: u16 = kSinCosTable8bit_Sext[((a as i32 & 0xff) + 64) as usize] as u16;
    let prod: u16 = ((r38 as u32).wrapping_mul(((r46) as i16).unsigned_abs() as u32) >> 8) as u16;
    (if r46 & 0x8000 != 0 {
        -(prod as i32)
    } else {
        prod as i32
    }) as u16
}

pub fn check_for_collision_with_baby_metroid(v0: u16, ctx: &mut impl Host) -> u8 {
    if ctx.mother_brain_target_enemy() == 0 {
        return 0;
    }
    if ctx
        .resolve_actor(Key::from_legacy(ctx.mother_brain_target_enemy()), Le)
        .health
        != 0
    {
        let rect: CollisionRect = set_up_collision_detection(v0, ctx);
        return check_for_enemy_collision_with_rectangle(
            ctx.mother_brain_target_enemy(),
            rect,
            ctx,
        );
    }
    enemy!(ctx, 0 => (v0 >> 1) as isize, definition);
    0xff
}

pub fn check_for_blue_ring_collision_with_room(k: u16, ctx: &mut impl Host) -> u8 {
    let v1: i32 = k as i32 >> 1;
    let result: u8 = 1;
    if (enemy!(ctx, (v1) as isize, y) as i32 - 32i32) & 0x8000 == 0
        && enemy!(ctx, (v1) as isize, y) < 0xd8
    {
        let mut v2: i16 = enemy!(ctx, (v1) as isize, x) as i16;
        if v2 >= 0 {
            let mut v3: i16 = (v2 as i32 - ctx.inputs().camera_x as i32) as i16;
            if v3 >= 0 && (v3 as i32 - 248i32) & 0x8000 != 0 {
                return 0;
            }
        }
    }
    result
}

pub fn set_up_collision_detection(k: u16, ctx: &mut impl Host) -> CollisionRect {
    let v1: i32 = k as i32 >> 1;
    CollisionRect {
        x: enemy!(ctx, (v1) as isize, x),
        y: enemy!(ctx, (v1) as isize, y),
        w: enemy_byte!(ctx, v1 as isize * 2, radius) as u16,
        h: enemy_byte!(ctx, v1 as isize * 2 + 1, radius) as u16,
    }
}

pub fn handle_blue_ring_collision_with_room(k: u16, ctx: &mut impl Host) {
    ctx.session_mut().feedback.earthquake_timer = 10;
    ctx.session_mut().feedback.earthquake_kind = 5;
    blue_ring_contact_explosion(k, ctx);
}

pub fn mother_brain_bomb_collides_with_bomb(k: u16, ctx: &mut impl Host) -> u8 {
    let v1: i32 = k as i32 >> 1;
    let rect = set_up_collision_detection(k, ctx);
    if check_for_bomb_collision_with_rectangle(rect, ctx) == 0 {
        return 0;
    }
    let value = ctx.mother_brain_active_bombs().wrapping_sub(1);
    ctx.set_mother_brain_active_bombs(value);
    enemy!(ctx, 0 => (v1) as isize, velocity_x);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);
    enemy!(ctx, 0 => (v1) as isize, definition);
    ctx.session_mut().enemy_projectiles.spawn.point = Point {
        x: enemy!(ctx, (v1) as isize, x),
        y: enemy!(ctx, (v1) as isize, y),
    };
    spawn_with_room_gfx(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 9, ctx);
    ctx.session_mut().enemy_projectiles.spawn.point = Point {
        x: enemy!(ctx, (v1) as isize, x),
        y: enemy!(ctx, (v1) as isize, y),
    };
    super::drops::spawn(ADDR_K_ENEMY_DEF_EC3F as u16, k, ctx);
    1
}

pub fn move_mother_brain_bomb(k: u16, a: u16, ctx: &mut impl Host) -> u8 {
    let v2: i32 = k as i32 >> 1;
    enemy_add!(ctx, (v2) as isize, velocity_y, a);
    move_with_velocity(k, ctx);
    if (enemy!(ctx, (v2) as isize, x) as i32 - 240i32) & 0x8000 == 0 {
        enemy!(ctx, -(enemy!(ctx, (v2) as isize, velocity_x) as i32) as u16 => (v2) as isize, velocity_x);
    }
    if (enemy!(ctx, (v2) as isize, y) as i32 - 208i32) & 0x8000 != 0 {
        return 0;
    }
    enemy!(ctx, 208 => (v2) as isize, y);
    enemy!(ctx, (if enemy!(ctx, (v2) as isize, velocity_x) & 0x8000 != 0 {
            -(enemy!(ctx, (v2) as isize, behavior_a) as i32)
        } else {
            enemy!(ctx, (v2) as isize, behavior_a) as i32
        }) as u16 => (v2) as isize, velocity_x);
    enemy!(ctx, -512i32 as u16 => (v2) as isize, velocity_y);
    1
}

pub fn spawn_mother_brain_hand_beam(x: u16, ctx: &mut impl Host) {
    spawn_with_room_gfx(
        ADDR_K_EPROJ_MOTHER_BRAIN_DEATH_BEAM_FIRED as u16,
        enemy!(ctx, (x >> 1) as isize, behavior_a),
        ctx,
    );
}

pub use super::pre_actions::despawn_off_screen as delete_namihe_fune_fireball_if_off_screen;

pub use super::pre_actions::despawn_off_screen as delete_magdollite_lava_if_off_screen;

pub fn is_off_screen(k: u16, ctx: &mut impl Host) -> u16 {
    if (enemy!(ctx, (k >> 1) as isize, x).wrapping_sub(ctx.inputs().camera_x) as i16) < 0 {
        return 1;
    }
    ctx.session_mut().compatibility_enemy_scratch = ctx.inputs().camera_x.wrapping_add(256);
    if (enemy!(ctx, (k >> 1) as isize, x).wrapping_sub(ctx.session().compatibility_enemy_scratch)
        as i16)
        >= 0
        || (enemy!(ctx, (k >> 1) as isize, y).wrapping_sub(ctx.inputs().camera_y) as i16) < 0
    {
        return 1;
    }
    ctx.session_mut().compatibility_enemy_scratch = ctx.inputs().camera_y.wrapping_add(256);
    u16::from(
        (enemy!(ctx, (k >> 1) as isize, y).wrapping_sub(ctx.session().compatibility_enemy_scratch)
            as i16)
            >= 0,
    )
}

/// Whether the projectile is outside the room's top-left 512-pixel square.
/// Was `Eproj_FuncE722`.
pub fn outside_512_pixel_square(k: u16, ctx: &mut impl Host) -> u16 {
    let v1: i32 = k as i32 >> 1;
    let mut v2: i16 = enemy!(ctx, (v1) as isize, x) as i16;
    let result: u16 = 1;
    if v2 >= 0 && (v2 as i32 - 512i32) & 0x8000 != 0 {
        let mut v3: i16 = enemy!(ctx, (v1) as isize, y) as i16;
        if v3 >= 0 && (v3 as i32 - 512i32) & 0x8000 != 0 {
            return 0;
        }
    }
    result
}

pub use super::pre_actions::botwoon_body_hurt_flash as botwoon_body_hurt_flash_alternate;

pub fn queue_small_explosion_sound(ctx: &mut impl Host) {
    ctx.queue_sound(2, 6, 0x24);
}

pub use super::pre_actions::despawn_off_screen as delete_botwoon_spit_if_off_screen;

pub use super::pre_actions::move_cacatac_spike_left as namihe_fune_fireball_move_left;

pub use super::pre_actions::move_cacatac_spike_right as namihe_fune_fireball_move_right;

pub fn botwoon_body_dying_set_delay(v0: u16, ctx: &mut impl Host) {
    let v1: i32 = v0 as i32 >> 1;
    enemy!(ctx, (4 * v0 as i32 + 96) as u16 => (v1) as isize, behavior_a);
    enemy!(ctx, 0xeb04 => (v1) as isize, velocity_x);
    botwoon_body_dying_waiting(v0, ctx);
}

pub fn botwoon_body_dying_waiting(v0: u16, ctx: &mut impl Host) {
    let v1: i32 = v0 as i32 >> 1;
    if (enemy_add!(ctx, (v1) as isize, behavior_a, 1) as i32 - 256i32) & 0x8000 == 0 {
        enemy!(ctx, (0x86eb1f & 0xffff) as u16 => (v1) as isize, velocity_x);
    }
    enemy!(ctx, 0 => (v1) as isize, delay);
    botwoon_body_hurt_flash(v0, ctx);
}

pub fn botwoon_body_dying_falling(v0: u16, ctx: &mut impl Host) {
    let v1: i32 = v0 as i32 >> 1;
    let v3: i32 = (8 * ((enemy!(ctx, (v1) as isize, velocity_y) as i32 & 0xff00) >> 8)) >> 1;
    {
        let index = v1 as isize;
        let amount = polyp_rock_delta((v3 >> 1) as isize, ctx);
        add_split(SlotField::Y, SlotField::YFraction, index, amount, ctx);
    };
    if (enemy!(ctx, (v1) as isize, y) as i32 - 200i32) & 0x8000 != 0 {
        let v7: i32 = v0 as i32 >> 1;
        enemy_add!(ctx, (v7) as isize, velocity_y, 192);
        enemy!(ctx, 0 => (v7) as isize, delay);
        botwoon_body_hurt_flash_alternate(v0, ctx);
    } else {
        enemy!(ctx, 200 => (v1) as isize, y);
        enemy!(ctx, (0x86eb93 & 0xffff) as u16 => (v1) as isize, velocity_x);
        enemy!(ctx, ADDR_WORD_86E208 as u16 => (v1) as isize, program);
        enemy!(ctx, 2560 => (v1) as isize, graphics);
        enemy!(ctx, 1 => (v1) as isize, delay);
        queue_small_explosion_sound(ctx);
        if v0 == 10 {
            ctx.set_botwoon_body_landed(1);
        }
    };
}
