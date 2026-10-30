//! Complete enemy-projectile setup dispatch, using owned fields and explicit effects.
#![forbid(unsafe_code)]
use super::access::{enemy, enemy_byte, enemy_update};
use super::addresses;
use super::coordinates::{calculate_coordinates, projectile_argument};
use super::host::*;
use super::pre_actions::{
    mother_brain_death_explosion_pre_instruction, mother_brain_drool_attached_pre_instruction,
    mother_brain_rainbow_beam_charging_pre_instruction, move_to_blue_ring_spawn_position,
    time_bomb_set_japanese_text_pre_instruction,
};
use super::setup_helpers::*;
use super::shared::*;
use super::shared_pre::*;
use super::tables::*;
use crate::actors::access::{AccessMode::LittleEndian as Le, ActorKey as Key};
use crate::constants::bank_86::*;
use crate::constants::bank_a0::*;
use crate::geometry::{Point, Vector32};
use crate::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;

fn native_setup_word(address: u16, index: isize, ctx: &mut impl Host) -> u16 {
    let record = ctx.loaded_record(0x86, address);
    record.native_word(index.checked_mul(2).expect("ROM table offset overflow"))
}

pub fn dispatch(ea: u32, j: u16, ctx: &mut impl Host) {
    use addresses::init::*;
    match ea {
        AT_8AAF => init_projectile_8aaf(j, ctx),
        DRAYGONS_WALL_TURRET_PROJS => init_draygon_turret_projectile(j, ctx),
        RIDLEYS_FIREBALL => init_ridleys_fireball(j, ctx),
        PHANTOON_DESTROYABLE_FIREBALLS => init_phantoon_destroyable_flames(j, ctx),
        ROCKS_KRAID_SPITS => init_kraid_spit_rocks(j, ctx),
        ROCKS_WHEN_KRAID_RISES => init_rocks_when_kraid_rises(j, ctx),
        ALCOON_FIREBALL => init_alcoon_fireball(j, ctx),
        PIRATE_CLAW => init_pirate_claw(j, ctx),
        BOMB_TORIZO_LOW_HEALTH_DROOL => init_bomb_torizo_continuous_drool(j, ctx),
        BOMB_TORIZO_EXPLOSIVE_SWIPE => init_bomb_torizo_explosive_swipe(j, ctx),
        BOMB_TORIZO_STATUE_BREAKING => init_bomb_torizo_statue_breaking(j, ctx),
        AT_AB07 => init_projectile_ab07(j, ctx),
        GOLDEN_TORIZOS_CHOZO_ORBS => init_golden_torizo_chozo_orbs(j, ctx),
        TORIZO_SONIC_BOOM => init_torizo_sonic_boom(j, ctx),
        WRECKED_SHIP_CHOZO_SPIKE_FOOTSTEPS => init_wrecked_ship_chozo_spike_footsteps(j, ctx),
        GOLDEN_TORIZO_EGG => init_golden_torizo_egg(j, ctx),
        GOLDEN_TORIZO_EYE_BEAM => init_golden_torizo_eye_beam(j, ctx),
        DRAGON_FIREBALL => init_dragon_fireball(j, ctx),
        EYE_DOOR_PROJECTILE => init_eye_door_projectile(j, ctx),
        EYE_DOOR_SWEAT => init_eye_door_sweat(j, ctx),
        TOURIAN_STATUE_EYE_GLOW => init_tourian_statue_eye_glow(j, ctx),
        TOURIAN_STATUE_UNLOCKING_PARTICLE => init_tourian_statue_unlocking_particle(j, ctx),
        TOURIAN_STATUE_SOUL => init_tourian_statue_soul(j, ctx),
        FIRE_ARC_BODY => init_fire_arc_body(j, ctx),
        SHAKTOOL_ATTACK_MIDDLE_BACK_CIRCLE => init_shaktool_attack_back_circle(j, ctx),
        MOTHER_BRAIN_BOMB => init_mother_brain_bomb(j, ctx),
        MOTHER_BRAIN_ESCAPE_DOOR_PARTICLES => init_mother_brain_escape_door_particles(j, ctx),
        TIME_BOMB_SET_JAPANESE_TEXT => init_time_bomb_set_japanese_text(j, ctx),
        MOTHER_BRAIN_GLASS_SHATTERING_SHARD => init_mother_brain_glass_shard(j, ctx),
        MOTHER_BRAIN_GLASS_SHATTERING_SPARKLE => init_mother_brain_glass_sparkle(j, ctx),
        N00B_TUBE_CRACK => init_n00b_tube_crack(j, ctx),
        N00B_TUBE_SHARDS => init_n00b_tube_shard(j, ctx),
        N00B_TUBE_RELEASED_AIR_BUBBLES => init_n00b_tube_released_air_bubbles(j, ctx),
        SAVE_STATION_ELECTRICITY => init_save_station_laser(j, ctx),
        BOTWOONS_SPIT => init_botwoon_spit(j, ctx),
        SPARKS => init_falling_spark(j, ctx),
        FIREBALL_EXPLOSION => init_afterburn_centre(j, ctx),
        CERES_FALLING_DEBRIS => init_ceres_falling_debris(j, ctx),
        CERES_ELEVATOR_PAD => init_ceres_elevator_pad(j, ctx),
        CERES_ELEVATOR_PLATFORM => init_ceres_elevator_pad_concealer(j, ctx),
        PRE_PHANTOM_ROOM => pre_phantoon_room_pre_instruction(j, ctx),
        SUB_86_BB30 => init_projectile_bb50(j, ctx),
        MOTHER_BRAINS_BLUE_RING_LASERS => init_mother_brain_blue_ring_lasers(j, ctx),
        SUB_86_C605 => init_mother_brain_hand_beam_charging(j, ctx),
        EPROJ_INI_TOURIAN_STATUE_UNLOCKING_PARTICLE_TAIL => {
            init_tourian_statue_particle_tail(j, ctx)
        }
        SUB_86_D992 => init_cacatac_spike(j, ctx),
        DUST_CLOUD_OR_EXPLOSION => init_dust_cloud_explosion(j, ctx),
        DRAYGON_GOOP => init_draygon_goop(j, ctx),
        CROCOMIRE_SPIKE_WALL_PIECES => init_crocomire_spike_wall_pieces(j, ctx),
        CROCOMIRE_BRIDGE_CRUMBLING => init_crocomire_bridge_crumbling(j, ctx),
        AT_9634 => init_unused_9634(j, ctx),
        PHANTOON_STARTING_FIREBALLS => init_phantoon_starting_flames(j, ctx),
        ROCKS_FALLING_KRAID_CEILING => init_kraid_ceiling_rocks(j, ctx),
        AT_A379 => init_gunship_liftoff_dust_clouds(j, ctx),
        BOMB_TORIZO_LOW_HEALTH_INITIAL_DROOL => init_bomb_torizo_initial_drool(j, ctx),
        AT_A977 => init_projectile_a977(j, ctx),
        TOURIAN_STATUE_DUST_CLOUDS => init_tourian_statue_dust_clouds(j, ctx),
        TOURIAN_ESCAPE_SHAFT_FAKE_WALL_EXPLODE => init_tourian_escape_shaft_wall_explosion(j, ctx),
        TOURIAN_STATUE_UNLOCKING_PARTICLE_WATER_SPLASH => {
            init_tourian_statue_particle_splash(j, ctx)
        }
        TOURIAN_STATUE_BASE_DECORATION => init_tourian_statue_base_decoration(j, ctx),
        TOURIAN_STATUE_RIDLEY => init_tourian_statue_ridley(j, ctx),
        TOURIAN_STATUE_PHANTOON => init_tourian_statue_phantoon(j, ctx),
        POLYP_ROCKS => init_polyp_rock(j, ctx),
        AT_BDA2 => init_shaktool_attack_front_circle(j, ctx),
        MOTHER_BRAIN_ROOM_TURRETS => init_mother_brain_turrets(j, ctx),
        MOTHER_BRAIN_ROOM_TURRET_BULLETS => init_mother_brain_turret_bullets(j, ctx),
        MOTHER_BRAIN_DEATH_BEEM_FIRED => init_mother_brain_hand_beam_fired(j, ctx),
        MOTHER_BRAIN_RAINBOW_BEAM => init_mother_brain_rainbow_beam_charging(j, ctx),
        MOTHER_BRAINS_DROOL => init_mother_brain_drool(j, ctx),
        MOTHER_BRAINS_DEATH_EXPLOSION => init_mother_brain_death_explosion(j, ctx),
        MOTHER_BRAINS_RAINBOW_BEAM_EXPLOSION => init_mother_brain_rainbow_beam_explosion(j, ctx),
        MOTHER_BRAIN_PURPLE_BREATH_BIG => init_mother_brain_purple_breath_big(j, ctx),
        MOTHER_BRAIN_PURPLE_BREATH_SMALL => init_mother_brain_purple_breath_small(j, ctx),
        MOTHER_BRAIN_TUBE_FALLING => init_mother_brain_tube_falling(j, ctx),
        POWAMP_SPIKES => init_powamp_spike(j, ctx),
        SPORE_SPAWN_STALK => init_spore_spawn_stalk(j, ctx),
        SPORE_SPAWNERS => init_spore_spawner(j, ctx),
        EYE_DOOR_SMOKE => init_plm_dust_cloud(j, ctx),
        BOTWOONS_BODY => init_botwoon_body(j, ctx),
        CROCOMIRE_PROJECTILE => init_crocomire_projectile(j, ctx),
        PIRATE_MOTHER_BRAIN_LASER => init_pirate_laser(j, ctx),
        GOLDEN_TORIZO_SUPER_MISSILE => init_golden_torizo_super_missile(j, ctx),
        KIHUNTER_ACID_SPIT_LEFT => init_kihunter_acid_spit_left(j, ctx),
        KIHUNTER_ACID_SPIT_RIGHT => init_kihunter_acid_spit_right(j, ctx),
        KAGOS_BUGS => init_kago_bug(j, ctx),
        WORK_ROBOT_LASER_DOWN => init_work_robot_laser_down(j, ctx),
        WORK_ROBOT_LASER_HORIZONTAL => init_work_robot_laser_horizontal(j, ctx),
        WORK_ROBOT_LASER_UP => init_work_robot_laser_up(j, ctx),
        AT_DBF2 => init_mini_crocomire_projectile(j, ctx),
        SPORES => init_spores(j, ctx),
        NAMIHE_FUNE_FIREBALL => init_namihe_fune_fireball(j, ctx),
        LAVA_THROWN_BY_MAGDOLLITE => init_lava_thrown_by_magdollite(j, ctx),
        MINI_KRAID_SPIT => init_mini_kraid_spit(j, ctx),
        BOMB_TORIZO_LOW_HEALTH_EXPLODE => init_bomb_torizo_low_health_explosion(j, ctx),
        BOMB_TORIZO_DEATH_EXPLOSION => init_torizo_death_explosion(j, ctx),
        BOMB_TORIZOS_CHOZO_ORBS => init_bomb_torizo_chozo_orbs(j, ctx),
        TOURIAN_LANDING_DUST_CLOUDS_RIGHT_FOOT => init_torizo_landing_dust_cloud_right_foot(j, ctx),
        TORIZO_LANDING_DUST_CLOUD_LEFT_FOOT => init_torizo_landing_dust_cloud_left_foot(j, ctx),
        YAPPING_MAWS_BODY => init_yapping_maw_body(j, ctx),
        ENEMY_DEATH_EXPLOSION => init_enemy_death_explosion(j, ctx),
        AT_9688 => init_vertical_afterburn_up(j, ctx),
        AT_9696 => init_vertical_afterburn_down(j, ctx),
        AT_966C => init_horizontal_afterburn_right(j, ctx),
        AT_967A => init_horizontal_afterburn_left(j, ctx),
        MINI_KRAID_SPIKES_LEFT => init_mini_kraid_spikes_left(j, ctx),
        MINI_KRAID_SPIKES_RIGHT => init_mini_kraid_spikes_right(j, ctx),
        SPAWNED_SHOT_GATE => init_spawned_shot_gate(j, ctx),
        CLOSED_DOWNWARDS_SHOT_GATE => init_closed_downwards_shot_gate(j, ctx),
        CLOSED_UPWARDS_SHOT_GATE => init_closed_upwards_shot_gate(j, ctx),
        INITIALIZE => super::drops::initialize(j, ctx),
        SKREE_PARTICLE_DOWN_RIGHT => {
            init_skree_particle(j, 6, 320, -769, ctx);
        }
        SKREE_PARTICLE_UP_RIGHT => {
            init_skree_particle(j, 6, 96, -1025, ctx);
        }
        SKREE_PARTICLE_DOWN_LEFT => {
            init_skree_particle(j, -6, -320, -769, ctx);
        }
        SKREE_PARTICLE_UP_LEFT => {
            init_skree_particle(j, -6, -96, -1025, ctx);
        }
        _ => {
            ctx.unreachable();
        }
    }
}
pub fn init_projectile_8aaf(j: u16, ctx: &mut impl Host) {
    ctx.next_random();
    let v1 = ctx.inputs().current_enemy as i32 as u16;
    let v2: u16 = (ctx.resolve_actor(Key::from_legacy(v1), Le).y.whole as i32 + 12) as u16;
    let v3: i32 = j as i32 >> 1;

    enemy!(ctx, v2 => (v3) as isize, y);
    enemy!(ctx, (v2 as i32 + 72) as u16 => (v3) as isize, counter);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(v1), Le).variables[3] => (v3) as isize, velocity_y);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v1), Le).x.whole as i32 + (ctx.inputs().random as i32 & 0x1f) - 16i32) as u16 => (v3) as isize, x);
    let value = native_setup_word(
        0x8a75,
        ((enemy_byte!(ctx, v3 as isize * 2 + 1, velocity_y) as u16 & 6) >> 1) as isize,
        ctx,
    );
    enemy!(ctx, value => (v3) as isize, program);
}

pub fn init_draygon_turret_projectile(j: u16, ctx: &mut impl Host) {
    place_and_aim_at_samus(
        j,
        ctx.session().enemy_projectiles.spawn.point.x,
        ctx.session().enemy_projectiles.spawn.point.y,
        ctx,
    );
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 2560 => (v1) as isize, graphics);
    enemy!(ctx, (0x868d54 & 0xffff) as u16 => (v1) as isize, pre_action);
}

pub fn init_ridleys_fireball(j: u16, ctx: &mut impl Host) {
    let slot = usize::from(j >> 1);
    let parameter = ctx.session().enemy_projectiles.spawn.secondary;
    enemy!(ctx, (slot) as isize, behavior_b = parameter);
    let v2: i16 = (if ctx.session().enemy_projectiles.spawn.primary != 0 {
        25
    } else {
        -25i32
    }) as i16;
    let x = (ctx.resolve_actor(Key::from_legacy(0), Le).x.whole).wrapping_add((v2) as u16);
    enemy!(ctx, (slot) as isize, x = x);
    let y = (ctx.resolve_actor(Key::from_legacy(0), Le).y.whole).wrapping_sub(43);
    enemy!(ctx, (slot) as isize, y = y);
    enemy!(ctx, (slot) as isize, graphics = 2560);
    let x_velocity = ctx.ridley_fireball_x_velocity();
    enemy!(ctx, (slot) as isize, velocity_x = x_velocity);
    let y_velocity = ctx.ridley_fireball_y_velocity();
    enemy!(ctx, (slot) as isize, velocity_y = y_velocity);
    set_area_dependent_damage(ADDR_K_RIDLEYS_FIREBALL_TAB0 as u16, j, ctx);
}

pub fn init_phantoon_destroyable_flames(j: u16, ctx: &mut impl Host) {
    let mut v1: i8 = ((ctx.session().enemy_projectiles.spawn.primary & 0xff00) >> 8) as i8;
    match v1 as i32 {
        0 => {
            let v2: i32 = j as i32 >> 1;

            enemy!(ctx, 0 => (v2) as isize, x_fraction);
            enemy!(ctx, 0 => (v2) as isize, y_fraction);
            enemy!(ctx, 0 => (v2) as isize, velocity_x);
            enemy!(ctx, 0 => (v2) as isize, velocity_y);

            enemy!(ctx, ctx.resolve_actor(Key::from_legacy(0), Le).x.whole => (v2) as isize, x);
            enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32 + 32) as u16 => (v2) as isize, y);
            enemy!(ctx, ADDR_OFF_8697B4 as u16 => (v2) as isize, program);
            enemy!(ctx, (enemy!(ctx, (v2) as isize, properties) & 0xfff | 0x2000) => (v2) as isize, properties);
        }
        2 => {
            let v3: i32 = j as i32 >> 1;

            enemy!(ctx, 0 => (v3) as isize, x_fraction);
            enemy!(ctx, 0 => (v3) as isize, y_fraction);
            enemy!(ctx, 0 => (v3) as isize, velocity_y);

            let v4: u16 = ctx.session().enemy_projectiles.spawn.primary as u8 as u16;
            if ((ctx.session().enemy_projectiles.spawn.primary as u8 as i32 - 8i32) as i16) >= 0 {
                enemy!(ctx, -2i32 as u16 => (v3) as isize, velocity_x);
            } else {
                enemy!(ctx, 2 => (v3) as isize, velocity_x);
            }
            enemy!(ctx, BYTE_8698_B4[v4 as usize] as u16 => (v3) as isize, behavior_a);
            enemy!(ctx, ctx.resolve_actor(Key::from_legacy(0), Le).x.whole => (v3) as isize, x);
            enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32 + 32) as u16 => (v3) as isize, y);
            enemy!(ctx, (0x869a45 & 0xffff) as u16 => (v3) as isize, pre_action);
        }
        4 => {
            let v5: i32 = j as i32 >> 1;

            enemy!(ctx, 0 => (v5) as isize, x_fraction);
            enemy!(ctx, 0 => (v5) as isize, y_fraction);
            enemy!(ctx, 0 => (v5) as isize, velocity_y);

            enemy!(ctx, ((ctx.session().enemy_projectiles.spawn.primary as i32 & 0xf0) >> 1) as u16 => (v5) as isize, velocity_x);
            enemy!(ctx, BYTE_8698_F7[(ctx.session().enemy_projectiles.spawn.primary as i32 & 0xf) as usize] as u16 => (v5) as isize, x);
            enemy!(ctx, 40 => (v5) as isize, y);
            enemy!(ctx, (0x869a94 & 0xffff) as u16 => (v5) as isize, pre_action);
        }
        6 => {
            let v7: i32 = j as i32 >> 1;

            enemy!(ctx, 0 => (v7) as isize, x_fraction);
            enemy!(ctx, 0 => (v7) as isize, y_fraction);
            enemy!(ctx, 0 => (v7) as isize, velocity_y);

            enemy!(ctx, 128 => (v7) as isize, velocity_x);
            enemy!(ctx, BYTE_869979[ctx.session().enemy_projectiles.spawn.primary as u8 as usize] as u16 => (v7) as isize, behavior_a);
            enemy!(ctx, ctx.resolve_actor(Key::from_legacy(0), Le).x.whole => (v7) as isize, x);
            enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32 + 16) as u16 => (v7) as isize, y);
            enemy!(ctx, (0x869ada & 0xffff) as u16 => (v7) as isize, pre_action);
        }
        _ => {
            ctx.unreachable();
        }
    };
}

pub fn init_kraid_spit_rocks(j: u16, ctx: &mut impl Host) {
    let enemy_index = ctx.inputs().current_enemy;
    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole).wrapping_add(16) => (v2) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(enemy_index), Le).y.whole).wrapping_sub(96) => (v2) as isize, y);

    enemy!(ctx, 0 => (v2) as isize, y_fraction);
    enemy!(ctx, 0 => (v2) as isize, x_fraction);

    enemy!(ctx, ctx.session().enemy_projectiles.spawn.primary => (v2) as isize, velocity_x);
    enemy!(ctx, -1024i32 as u16 => (v2) as isize, velocity_y);
    enemy!(ctx, 1536 => (v2) as isize, graphics);
}

pub fn init_rocks_when_kraid_rises(j: u16, ctx: &mut impl Host) {
    let mut v1: i16 = ((ctx.inputs().random) as i16) & (0x3fi16);
    if (ctx.inputs().random & 1) == 0 {
        v1 = !(v1 as i32) as i16;
    }
    let v2: i32 = j as i32 >> 1;
    {
        let value = ({
            let enemy_index = ctx.inputs().current_enemy;
            ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole
        })
        .wrapping_add((v1) as u16);
        enemy!(ctx, value => (v2) as isize, x);
    };
    enemy!(ctx, 432 => (v2) as isize, y);

    enemy!(ctx, 0 => (v2) as isize, y_fraction);
    enemy!(ctx, 0 => (v2) as isize, x_fraction);

    enemy!(ctx, ctx.session().enemy_projectiles.spawn.primary => (v2) as isize, velocity_x);
    enemy!(ctx, -1280i32 as u16 => (v2) as isize, velocity_y);
    enemy!(ctx, 1536 => (v2) as isize, graphics);

    ctx.queue_sound(3, 6, 0x1e);
}

pub fn init_alcoon_fireball(j: u16, ctx: &mut impl Host) {
    static FIREBALL_Y_VELOCITIES: [i16; 3] = [-0x100i16, 0, 0x100];
    let v1 = ctx.inputs().current_enemy as i32 as u16;
    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v1), Le).y.whole as i32 - 12i32) as u16 => (v2) as isize, y);
    enemy!(ctx, -1024i32 as u16 => (v2) as isize, velocity_x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v1), Le).x.whole as i32 - 16i32) as u16 => (v2) as isize, x);
    if (ctx.resolve_actor(Key::from_legacy(v1), Le).variables[3] & 0x8000) == 0 {
        enemy!(ctx, 1024 => (v2) as isize, velocity_x);
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v1), Le).x.whole as i32 + 16) as u16 => (v2) as isize, x);
    }
    enemy!(ctx, FIREBALL_Y_VELOCITIES[(ctx.session().enemy_projectiles.spawn.primary >> 1) as usize] as u16 => (v2) as isize, velocity_y);

    enemy!(ctx, 0 => (v2) as isize, y_fraction);
    enemy!(ctx, 0 => (v2) as isize, x_fraction);
}

pub fn init_pirate_claw(j: u16, ctx: &mut impl Host) {
    let r = ctx.session().enemy_projectiles.spawn.rectangle;
    let v1: i32 = j as i32 >> 1;
    enemy!(ctx, (r.origin.y as i32 + r.extent.y as i32) as u16 => (v1) as isize, y);
    enemy!(ctx, (r.origin.x as i32 + r.extent.x as i32) as u16 => (v1) as isize, x);
    {
        let value = (if ctx.session().enemy_projectiles.spawn.primary != 0 {
            ADDR_OFF_869FE1 as i32
        } else {
            ADDR_OFF_869FB9 as i32
        }) as u16;
        enemy!(ctx, value => (v1) as isize, program);
    };
    enemy!(ctx, (0x86a05b & 0xffff) as u16 => (v1) as isize, pre_action);
    enemy!(ctx, 2048 => (v1) as isize, behavior_a);
    enemy!(ctx, 1 => (v1) as isize, behavior_b);
}

pub fn init_bomb_torizo_continuous_drool(j: u16, ctx: &mut impl Host) {
    let mut v5: u16 = 0;
    let v1: i32 = j as i32 >> 1;
    enemy!(ctx, 0 => (v1) as isize, graphics);
    let value = native_setup_word(
        0xa64d,
        (((ctx.next_random() >> 1) & (0xeu16)) >> 1) as isize,
        ctx,
    );
    enemy!(ctx, value => (v1) as isize, program);
    ctx.next_random();
    let v2 = ctx.inputs().current_enemy as i32 as u16;
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v2), Le).y.whole).wrapping_sub(5) => (v1) as isize, y);
    let mut parameter_1: i16 = ctx.resolve_actor(Key::from_legacy(v2), Le).parameters[0] as i16;
    let mut v4: u16 = if (parameter_1 as u16 & 0x4000) != 0 {
        ctx.inputs().random & 0x1fe
    } else {
        if parameter_1 < 0 {
            v5 = 32;
        } else {
            v5 = 224;
        }
        (2 * (v5 as i32 + (ctx.inputs().random as i32 & 0xf) - 8i32)) as u16
    };
    let v6: i32 = v4 as i32 >> 1;
    enemy!(ctx, kSinCosTable8bit_Sext[(v6 + 64) as usize] as u16 => (v1) as isize, velocity_x);
    enemy!(ctx, kSinCosTable8bit_Sext[v6 as usize] as u16 => (v1) as isize, velocity_y);
    let v7 = ctx.inputs().current_enemy as i32 as u16;
    let mut v8: u16 = if (ctx.resolve_actor(Key::from_legacy(v7), Le).parameters[0] & 0x8000) != 0 {
        (ctx.resolve_actor(Key::from_legacy(v7), Le).x.whole).wrapping_add(8)
    } else {
        (ctx.resolve_actor(Key::from_legacy(v7), Le).x.whole as i32 - 8i32) as u16
    };
    enemy!(ctx, v8 => (v1) as isize, x);
}

pub fn init_bomb_torizo_explosive_swipe(j: u16, ctx: &mut impl Host) {
    let enemy = ctx.inputs().current_enemy;
    let mirrored = ctx.resolve_actor(Key::from_legacy(enemy), Le).parameters[0] & 0x8000 != 0;
    let variant = (ctx.session().enemy_projectiles.spawn.primary >> 1) as usize;
    let slot = (j >> 1) as isize;
    let x = ctx.resolve_actor(Key::from_legacy(enemy), Le).x.whole;
    let x_offset = K_EPROJ_INIT_BOMB_TORIZO_EXPLOSIVE_SWIPE_TAB0[variant] as u16;
    let x = if mirrored {
        x.wrapping_sub(x_offset)
    } else {
        x.wrapping_add(x_offset)
    };
    enemy!(ctx, x => slot, x);
    let y = ctx.resolve_actor(Key::from_legacy(enemy), Le).y.whole;
    let y_offset = K_EPROJ_INIT_BOMB_TORIZO_EXPLOSIVE_SWIPE_TAB1[variant] as u16;
    enemy!(ctx, y.wrapping_add(y_offset) => slot, y);
}

pub fn init_bomb_torizo_statue_breaking(j: u16, ctx: &mut impl Host) {
    calculate_coordinates(ctx.session().mechanisms.selected, ctx);
    let mut v1: i8 = ctx.session().enemy_projectiles.spawn.primary as i8;
    let v2: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    let v3: i32 = j as i32 >> 1;
    let value = native_setup_word(0xa7ab, v2 as isize, ctx);
    enemy!(ctx, value => (v3) as isize, program);
    {
        let value = (K_EPROJ_INIT_BOMB_TORIZO_STATUE_BREAKING_XPOS[v2 as usize] as i32
            + 16 * ctx.session().mechanisms.block_x as i32) as u16;
        enemy!(ctx, value => (v3) as isize, x);
    };
    let v4: i32 = (v1 as i32 & 0xf) >> 1;
    {
        let value = (K_EPROJ_INIT_BOMB_TORIZO_STATUE_BREAKING_YPOS[v4 as usize] as i32
            + 16 * ctx.session().mechanisms.block_y as i32) as u16;
        enemy!(ctx, value => (v3) as isize, y);
    };
    enemy!(ctx, K_EPROJ_INIT_BOMB_TORIZO_STATUE_BREAKING_YVEL[v4 as usize] as u16 => (v3) as isize, velocity_y);
    enemy!(ctx, K_EPROJ_INIT_BOMB_TORIZO_STATUE_BREAKING_F[v4 as usize] as u16 => (v3) as isize, behavior_b);
    enemy_update!(ctx, (v3 as usize) as isize, properties, |old| old
        | (0x1000));
}

pub fn init_projectile_ab07(j: u16, ctx: &mut impl Host) {
    let mut v1: u16 = ctx.vram_write_queue_tail();
    let v2 = ctx.vram_write_queue_tail();
    ctx.set_vram_queue_size(v2, 64);
    ctx.set_vram_queue_source(v2, ADDR_K_EPROJ_INIT_AB07_TILE0 as u16);
    ctx.set_vram_queue_bank_word(v2, 134);
    ctx.set_vram_queue_destination(v2, 28160);
    v1 = (v1 as i32 + 7) as u16;
    let v3 = v1;
    ctx.set_vram_queue_size(v3, 64);
    ctx.set_vram_queue_source(v3, ADDR_K_EPROJ_INIT_AB07_TILE1 as u16);
    ctx.set_vram_queue_bank_word(v3, 134);
    ctx.set_vram_queue_destination(v3, 28416);
    {
        let value = (v1 as i32 + 7) as u16;
        ctx.set_vram_write_queue_tail(value);
    };
    let v4: i32 = j as i32 >> 1;
    enemy!(ctx, ctx.session().player.x => (v4) as isize, x);
    enemy!(ctx, (ctx.session().player.y as i32 - 36i32) as u16 => (v4) as isize, y);
}

pub fn init_golden_torizo_chozo_orbs(j: u16, ctx: &mut impl Host) {
    let e = ctx.inputs().current_enemy as i32 as u16;
    let v2: u16 = (if (ctx.resolve_actor(Key::from_legacy(e), Le).parameters[0] & 0x8000) != 0 {
        ADDR_K_EPROJ_INIT_GOLDEN_TORIZOS_CHOZO_ORBS_INIT0 as i32
    } else {
        ADDR_K_EPROJ_INIT_GOLDEN_TORIZOS_CHOZO_ORBS_INIT1 as i32
    }) as u16;
    init_torizo_projectile(
        j,
        v2,
        Point {
            x: ctx.resolve_actor(Key::from_legacy(e), Le).x.whole,
            y: ctx.resolve_actor(Key::from_legacy(e), Le).y.whole,
        },
        ctx,
    );
}

pub fn init_torizo_sonic_boom(j: u16, ctx: &mut impl Host) {
    let v1: i32 = if (ctx.next_random() & 1) != 0 {
        -12i32
    } else {
        20
    };
    let v2 = ctx.inputs().current_enemy as i32 as u16;
    let v3: i32 = j as i32 >> 1;

    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v2), Le).y.whole as i32 + v1) as u16 => (v3) as isize, y);
    enemy!(ctx, 0 => (v3) as isize, velocity_y);

    if (ctx.resolve_actor(Key::from_legacy(v2), Le).parameters[0] & 0x8000) != 0 {
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v2), Le).x.whole as i32 + 32) as u16 => (v3) as isize, x);
        enemy!(ctx, 624 => (v3) as isize, velocity_x);
        enemy!(ctx, ADDR_OFF_86ADD2 as u16 => (v3) as isize, program);
    } else {
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v2), Le).x.whole as i32 - 32i32) as u16 => (v3) as isize, x);
        enemy!(ctx, -624i32 as u16 => (v3) as isize, velocity_x);
        enemy!(ctx, ADDR_OFF_86ADBF as u16 => (v3) as isize, program);
    };
}

pub fn init_wrecked_ship_chozo_spike_footsteps(j: u16, ctx: &mut impl Host) {
    let v1 = ctx.inputs().current_enemy as i32 as u16;
    let v2: i32 = j as i32 >> 1;
    {
        let value = (ctx.session().enemy_projectiles.spawn.primary as i32
            + ctx.resolve_actor(Key::from_legacy(v1), Le).x.whole as i32)
            as u16;
        enemy!(ctx, value => (v2) as isize, x);
    };
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v1), Le).y.whole as i32 + 28) as u16 => (v2) as isize, y);
}

pub fn init_golden_torizo_egg(j: u16, ctx: &mut impl Host) {
    let v1 = ctx.inputs().current_enemy as i32 as u16;
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, ((0xe2 & 0x1f) + 64) as u16 => (v2) as isize, behavior_b);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(v1), Le).parameters[0] => (v2) as isize, behavior_a);

    let v4: u16 = (if ctx.resolve_actor(Key::from_legacy(v1), Le).parameters[0] & 0x8000 != 0 {
        ADDR_K_EPROJ_INIT_GOLDEN_TORIZO_EGG0 as i32
    } else {
        ADDR_K_EPROJ_INIT_GOLDEN_TORIZO_EGG1 as i32
    }) as u16;
    init_torizo_projectile(
        j,
        v4,
        Point {
            x: ctx.resolve_actor(Key::from_legacy(v1), Le).x.whole,
            y: ctx.resolve_actor(Key::from_legacy(v1), Le).y.whole,
        },
        ctx,
    );
}

pub fn init_golden_torizo_eye_beam(j: u16, ctx: &mut impl Host) {
    let e = ctx.inputs().current_enemy as i32 as u16;
    let v2: u16 = (if (ctx.resolve_actor(Key::from_legacy(e), Le).parameters[0] & 0x8000) != 0 {
        ADDR_STRU_86B376 as i32
    } else {
        ADDR_STRU_86B380 as i32
    }) as u16;
    init_torizo_projectile(
        j,
        v2,
        Point {
            x: ctx.resolve_actor(Key::from_legacy(e), Le).x.whole,
            y: ctx.resolve_actor(Key::from_legacy(e), Le).y.whole,
        },
        ctx,
    );
    let mut v3: u16 = ((ctx.next_random() as i32 & 0x1e) - 16i32 + 192) as u16;
    if (ctx.resolve_actor(Key::from_legacy(e), Le).parameters[0] & 0x8000) == 0 {
        v3 = (v3 as i32 + 128) as u16;
    }
    let v4: i32 = v3 as i32 >> 1;
    let v5: i32 = j as i32 >> 1;
    enemy!(ctx, (8 * kSinCosTable8bit_Sext[(v4 + 64) as usize] as i32) as u16 => (v5) as isize, velocity_x);
    enemy!(ctx, (8 * kSinCosTable8bit_Sext[v4 as usize] as i32) as u16 => (v5) as isize, velocity_y);
}

pub fn init_dragon_fireball(j: u16, ctx: &mut impl Host) {
    let v1 = ctx.inputs().current_enemy as i32 as u16;
    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v1), Le).y.whole).wrapping_sub(28) => (v2) as isize, y);
    enemy!(ctx, -961i32 as u16 => (v2) as isize, velocity_y);
    if (ctx.resolve_actor(Key::from_legacy(v1), Le).variables[0] & 0x8000) == 0 {
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v1), Le).x.whole as i32 + 12) as u16 => (v2) as isize, x);
        enemy!(ctx, 704 => (v2) as isize, velocity_x);
        enemy!(ctx, ADDR_WORD_86B4CB as u16 => (v2) as isize, program);
    } else {
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v1), Le).x.whole).wrapping_sub(12) => (v2) as isize, x);
        enemy!(ctx, -704i32 as u16 => (v2) as isize, velocity_x);
        enemy!(ctx, ADDR_WORD_86B4BF as u16 => (v2) as isize, program);
    };
}

pub fn init_eye_door_projectile(j: u16, ctx: &mut impl Host) {
    let v1: u16 = ctx.session().mechanisms.selected;
    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, projectile_argument(ctx.session().mechanisms.selected, ctx) => (v2) as isize, behavior_b);
    calculate_coordinates(v1, ctx);
    let v3: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;

    {
        let value = (WORD_86_B65_B[v3 as usize] as i32
            + 8 * (2 * ctx.session().mechanisms.block_x as i32 + 1)) as u16;
        enemy!(ctx, value => (v2) as isize, x);
    };
    {
        let value = (WORD_86_B65_B[(v3 + 1) as usize] as i32
            + 16 * ctx.session().mechanisms.block_y as i32) as u16;
        enemy!(ctx, value => (v2) as isize, y);
    };
}

pub fn init_eye_door_sweat(j: u16, ctx: &mut impl Host) {
    calculate_coordinates(ctx.session().mechanisms.selected, ctx);
    let v1: i32 = j as i32 >> 1;
    enemy!(ctx, (8 * (2 * (ctx.session().mechanisms.block_x as i32 - 1i32) + 1)) as u16 => (v1) as isize, x);
    enemy!(ctx, (16 * (ctx.session().mechanisms.block_y as i32 + 1)) as u16 => (v1) as isize, y);
    let v2: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    enemy!(ctx, WORD_86_B6_B1[v2 as usize] as u16 => (v1) as isize, velocity_x);
    enemy!(ctx, WORD_86_B6_B1[(v2 + 1) as usize] as u16 => (v1) as isize, velocity_y);
}

pub fn init_tourian_statue_eye_glow(j: u16, ctx: &mut impl Host) {
    let v1: u16 = ctx.session().enemy_projectiles.spawn.primary;
    let v2: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    let v3: i32 = j as i32 >> 1;
    enemy!(ctx, K_EPROJ_INIT_TOURIAN_STATUE_EYE_GLOW_X[v2 as usize] => (v3) as isize, x);
    enemy!(ctx, K_EPROJ_INIT_TOURIAN_STATUE_EYE_GLOW_Y[v2 as usize] => (v3) as isize, y);
    let mut v4: u16 = (4 * v1 as i32) as u16;
    for i in (498i32..506).step_by(2) {
        {
            let value = K_EPROJ_INIT_TOURIAN_STATUE_EYE_GLOW_COLORS[(v4 >> 1) as usize];
            ctx.set_palette_buffer((i >> 1) as isize, value);
        };
        v4 = (v4 as i32 + 2) as u16;
    }
}

pub fn init_tourian_statue_unlocking_particle(j: u16, ctx: &mut impl Host) {
    let v1: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, enemy!(ctx, (v1) as isize, x) => (v2) as isize, x);
    enemy!(ctx, enemy!(ctx, (v1) as isize, y) => (v2) as isize, y);
    let v3: u16 = (2 * ((ctx.next_random() as i32 & 0x3f) - 32i32) as u8 as i32) as u16;
    enemy!(ctx, v3 => (v2) as isize, behavior_a);
    let v4: i32 = v3 as i32 >> 1;
    enemy!(ctx, kSinCosTable8bit_Sext[(v4 + 64) as usize] as u16 => (v2) as isize, velocity_x);
    enemy!(ctx, (4 * kSinCosTable8bit_Sext[v4 as usize] as i32) as u16 => (v2) as isize, velocity_y);
}

pub fn init_tourian_statue_soul(j: u16, ctx: &mut impl Host) {
    let v1: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, K_EPROJ_INIT_TOURIAN_STATUE_EYE_GLOW_X[v1 as usize] => (v2) as isize, x);
    enemy!(ctx, K_EPROJ_INIT_TOURIAN_STATUE_EYE_GLOW_Y[v1 as usize] => (v2) as isize, y);
    enemy!(ctx, -1024i32 as u16 => (v2) as isize, velocity_y);
}

pub fn init_fire_arc_body(j: u16, ctx: &mut impl Host) {
    let v2 = ctx.inputs().current_enemy as i32 as u16;
    let v3: i32 = j as i32 >> 1;

    copy_enemy_position(v3 as isize, v2, 0, ctx);

    let v4 = ctx.inputs().current_enemy;
    let linked = (ctx.inputs().current_enemy as i32 + ctx.fire_arc_part_offset(v4) as i32) as u16;
    ctx.set_enemy_linked_projectile(linked, j);
    enemy!(ctx, 1 => (v3) as isize, drop_flags);
}

pub fn init_shaktool_attack_back_circle(j: u16, ctx: &mut impl Host) {
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.primary => (j >> 1) as isize, behavior_a);
    init_shaktool_attack_front_circle(j, ctx);
}

pub fn init_mother_brain_bomb(j: u16, ctx: &mut impl Host) {
    enemy_byte!(
        ctx,
        j as isize,
        x_fraction = ctx.session().enemy_projectiles.spawn.primary as u8
    );
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 256 => (v1) as isize, velocity_y);
    enemy!(ctx, 224 => (v1) as isize, velocity_x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).x.whole as i32 + 12) as u16 => (v1) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).y.whole as i32 + 16) as u16 => (v1) as isize, y);
    enemy!(ctx, 1024 => (v1) as isize, graphics);
    enemy!(ctx, 112 => (v1) as isize, behavior_a);
    enemy!(ctx, 0 => (v1) as isize, behavior_b);

    let count = ctx.mother_brain_active_bombs();
    ctx.set_mother_brain_active_bombs(count.wrapping_add(1));
}

pub fn init_mother_brain_escape_door_particles(j: u16, ctx: &mut impl Host) {
    static ESCAPE_DOOR_PARTICLE_X: [i16; 16] = [
        0, -0x20i16, 0, -0x18i16, 0, -0x10i16, 0, -8i16, 0, 0, 0, 8, 0, 0x10, 0, 0x18,
    ];
    static ESCAPE_DOOR_PARTICLE_X_VELOCITIES: [i16; 16] = [
        0x500, -0x200i16, 0x500, -0x100i16, 0x500, -0x100i16, 0x500, -0x80i16, 0x500, -0x80i16,
        0x500, 0x80, 0x500, -0x100i16, 0x500, 0x200,
    ];
    let v1: i32 = j as i32 >> 1;
    enemy!(ctx, 0 => (v1) as isize, graphics);
    let v2: i32 = (4 * ctx.session().enemy_projectiles.spawn.primary as i32) >> 1;

    enemy!(ctx, (ESCAPE_DOOR_PARTICLE_X[v2 as usize] as i32 + 16) as u16 => (v1) as isize, x);
    enemy!(ctx, (ESCAPE_DOOR_PARTICLE_X[(v2 + 1) as usize] as i32 + 128) as u16 => (v1) as isize, y);
    enemy!(ctx, ESCAPE_DOOR_PARTICLE_X_VELOCITIES[v2 as usize] as u16 => (v1) as isize, velocity_x);
    enemy!(ctx, ESCAPE_DOOR_PARTICLE_X_VELOCITIES[(v2 + 1) as usize] as u16 => (v1) as isize, velocity_y);
    enemy!(ctx, 32 => (v1) as isize, behavior_a);
}

pub fn init_time_bomb_set_japanese_text(j: u16, ctx: &mut impl Host) {
    enemy!(ctx, 0 => (j >> 1) as isize, graphics);
    time_bomb_set_japanese_text_pre_instruction(j, ctx);
}

pub fn init_mother_brain_glass_shard(j: u16, ctx: &mut impl Host) {
    let v1: u16 = ((2 * ctx.next_random() as i32) & 0x1fe) as u16;
    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, v1 => (v2) as isize, behavior_a);
    let v3: i32 = v1 as i32 >> 1;
    enemy!(ctx, kSinCosTable8bit_Sext[(v3 + 64) as usize] as u16 => (v2) as isize, velocity_x);
    enemy!(ctx, (4 * kSinCosTable8bit_Sext[v3 as usize] as i32) as u16 => (v2) as isize, velocity_y);
    let value = native_setup_word(0xce41, ((v1 as i32 >> 4 & 0x1e) as u16 >> 1) as isize, ctx);
    enemy!(ctx, value => (v2) as isize, program);
    enemy!(ctx, 1600 => (v2) as isize, graphics);
    calculate_coordinates(ctx.session().mechanisms.selected, ctx);
    let v4: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;

    {
        let value = (K_EPROJ_INIT_MOTHER_BRAIN_GLASS_SHATTERING_SHARD_X[v4 as usize] as i32
            + 16 * ctx.session().mechanisms.block_x as i32) as u16;
        enemy!(ctx, value => (v2) as isize, x);
    };
    {
        let value = (K_EPROJ_INIT_MOTHER_BRAIN_GLASS_SHATTERING_SHARD_Y[v4 as usize] as i32
            + 16 * ctx.session().mechanisms.block_y as i32) as u16;
        enemy!(ctx, value => (v2) as isize, y);
    };

    enemy_update!(ctx, (v2) as isize, x, |old| ((old as i32)
        + ((ctx.next_random() as i32 & 0xf) - 8i32))
        as u16);
    enemy_update!(ctx, (v2) as isize, y, |old| ((old as i32)
        + ((ctx.next_random() as i32 & 0xf) - 8i32))
        as u16);
}

pub fn init_mother_brain_glass_sparkle(j: u16, ctx: &mut impl Host) {
    let v1: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, (enemy!(ctx, (v1) as isize, x) as i32 + (ctx.next_random() as i32 & 0x1f) - 16i32) as u16 => (v2) as isize, x);
    enemy!(ctx, (enemy!(ctx, (v1) as isize, y) as i32 + (ctx.next_random() as i32 & 0x1f) - 16i32) as u16 => (v2) as isize, y);
    enemy!(ctx, 1600 => (v2) as isize, graphics);
}

pub fn init_n00b_tube_crack(j: u16, ctx: &mut impl Host) {
    calculate_coordinates(ctx.session().mechanisms.selected, ctx);
    let v1: i32 = j as i32 >> 1;
    enemy!(ctx, (16 * ctx.session().mechanisms.block_x as i32 + 96) as u16 => (v1) as isize, x);
    enemy!(ctx, (16 * ctx.session().mechanisms.block_y as i32 + 48) as u16 => (v1) as isize, y);
}

pub fn init_n00b_tube_shard(j: u16, ctx: &mut impl Host) {
    calculate_coordinates(ctx.session().mechanisms.selected, ctx);
    let v1: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    let v2: i32 = j as i32 >> 1;

    {
        let value = (K_EPROJ_INIT_N00B_TUBE_SHARDS_X[v1 as usize] as i32
            + 16 * ctx.session().mechanisms.block_x as i32
            + 96) as u16;
        enemy!(ctx, value => (v2) as isize, behavior_b);
    };
    enemy!(ctx, 0 => (v2) as isize, behavior_a);
    {
        let value = (K_EPROJ_INIT_N00B_TUBE_SHARDS_Y[v1 as usize] as i32
            + 16 * ctx.session().mechanisms.block_y as i32
            + 48) as u16;
        enemy!(ctx, value => (v2) as isize, y);
    };
    let value = native_setup_word(0xd760, v1 as isize, ctx);
    enemy!(ctx, value => (v2) as isize, program);
    enemy!(ctx, K_EPROJ_INIT_N00B_TUBE_SHARDS_XVEL[v1 as usize] as u16 => (v2) as isize, velocity_x);
    enemy!(ctx, K_EPROJ_INIT_N00B_TUBE_SHARDS_YVEL[v1 as usize] as u16 => (v2) as isize, velocity_y);
}

pub fn init_n00b_tube_released_air_bubbles(j: u16, ctx: &mut impl Host) {
    calculate_coordinates(ctx.session().mechanisms.selected, ctx);
    let v1: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    let v2: i32 = j as i32 >> 1;

    {
        let value = (K_EPROJ_INIT_N00B_TUBE_RELEASED_AIR_BUBBLES_X[v1 as usize] as i32
            + 16 * ctx.session().mechanisms.block_x as i32) as u16;
        enemy!(ctx, value => (v2) as isize, behavior_b);
    };
    enemy!(ctx, 0 => (v2) as isize, behavior_a);
    {
        let value = (K_EPROJ_INIT_N00B_TUBE_RELEASED_AIR_BUBBLES_Y[v1 as usize] as i32
            + 16 * ctx.session().mechanisms.block_y as i32) as u16;
        enemy!(ctx, value => (v2) as isize, y);
    };
    enemy!(ctx, -1280i32 as u16 => (v2) as isize, velocity_y);
}

pub fn init_save_station_laser(j: u16, ctx: &mut impl Host) {
    calculate_coordinates(ctx.session().mechanisms.selected, ctx);
    let v1: i32 = j as i32 >> 1;
    enemy!(ctx, (16 * (ctx.session().mechanisms.block_x as i32 + 1)) as u16 => (v1) as isize, x);
    enemy!(ctx, (16 * (ctx.session().mechanisms.block_y as i32 - 2i32)) as u16 => (v1) as isize, y);
}

pub fn init_botwoon_spit(j: u16, ctx: &mut impl Host) {
    let v1 = ctx.inputs().current_enemy as i32 as u16;
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(v1), Le).x.whole => (v2) as isize, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(v1), Le).y.whole => (v2) as isize, y);
    enemy!(ctx, ADDR_WORD_86EBAE as u16 => (v2) as isize, program);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.angle as u16 => (v2) as isize, angle);

    let pt: Vector32 = convert_angle_to_xy(
        ctx.session().enemy_projectiles.spawn.angle as u16,
        ctx.session().enemy_projectiles.spawn.primary,
        ctx,
    );

    enemy!(ctx, (pt.x >> 16) as u16 => (v2) as isize, velocity_x);
    enemy!(ctx, pt.x as u16 => (v2) as isize, behavior_a);
    enemy!(ctx, (pt.y >> 16) as u16 => (v2) as isize, velocity_y);
    enemy!(ctx, pt.y as u16 => (v2) as isize, behavior_b);
}

pub fn init_falling_spark(j: u16, ctx: &mut impl Host) {
    static FALLING_SPARK_VELOCITIES: [u16; 14] = [
        0xffff, 0xb800, 0xffff, 0xc000, 0xffff, 0xe000, 0xffff, 0xff00, 0, 0x100, 0, 0x2000, 0,
        0x4000,
    ];
    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, ADDR_K_ENEMY_DEF_F353 as u16 => (v2) as isize, program);
    let v3 = ctx.inputs().current_enemy as i32 as u16;

    copy_enemy_position(v2 as isize, v3, 8, ctx);

    enemy!(ctx, 0 => (v2) as isize, velocity_x);
    enemy!(ctx, 0 => (v2) as isize, velocity_y);

    let v4: u16 = ((ctx.next_random() as i32 & 0x1c) >> 1) as u16;
    let value = native_setup_word(0xf3d4, (v4 as i32) as isize, ctx);
    enemy!(ctx, value => (v2) as isize, behavior_b);
    let value = native_setup_word(0xf3d4, (v4 as i32 + 1) as isize, ctx);
    enemy!(ctx, value => (v2) as isize, behavior_a);
}

pub fn init_afterburn_centre(j: u16, ctx: &mut impl Host) {
    let slot = usize::from(j >> 1);
    enemy!(ctx, (slot) as isize, graphics = 2560);
    let x = ctx.session().enemy_projectiles.spawn.point.x;
    enemy!(ctx, (slot) as isize, x = x);
    let y = ctx.session().enemy_projectiles.spawn.point.y;
    enemy!(ctx, (slot) as isize, y = y);
    let parameter = ctx.session().enemy_projectiles.spawn.primary;
    enemy!(ctx, (slot) as isize, behavior_a = parameter);
    enemy!(ctx, (slot) as isize, behavior_b = 0);
    enemy!(ctx, (slot) as isize, velocity_x = 0);
    enemy!(ctx, (slot) as isize, velocity_y = 0);
}

pub fn init_ceres_falling_debris(j: u16, ctx: &mut impl Host) {
    let slot = usize::from(j >> 1);
    enemy!(ctx, (slot) as isize, behavior_a = 0);
    enemy!(ctx, (slot) as isize, behavior_b = 0);
    enemy!(ctx, (slot) as isize, velocity_x = 0);
    enemy!(ctx, (slot) as isize, graphics = 3584);
    let x = ctx.session().enemy_projectiles.spawn.primary;
    enemy!(ctx, (slot) as isize, x = x);
    enemy!(ctx, (slot) as isize, y = 42);
    enemy!(ctx, (slot) as isize, velocity_y = 16);
}

pub fn init_ceres_elevator_pad(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;
    let eproj_y_pos: u16 = ctx.session().player.y.wrapping_add(28);
    enemy!(ctx, (v1 as usize) as isize, y = eproj_y_pos);
    enemy!(ctx, (v1 as usize) as isize, behavior_a = 60);
    init_ceres_elevator(j, ctx);
}

pub fn init_ceres_elevator_pad_concealer(j: u16, ctx: &mut impl Host) {
    let eproj_word_index = (j >> 1) as usize;
    enemy!(ctx, (eproj_word_index) as isize, y = 97);
    init_ceres_elevator(j, ctx);
}

pub fn pre_phantoon_room_pre_instruction(_j: u16, ctx: &mut impl Host) {
    {
        let value = 0;
        ctx.set_bg2_y_scroll(value);
    };
}

pub fn init_projectile_bb50(j: u16, ctx: &mut impl Host) {
    let v1 = ctx.inputs().current_enemy;
    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, ctx.enemy_projectile_origin_x(v1) => (v2) as isize, x);
    enemy!(ctx, ctx.enemy_projectile_origin_y(v1) => (v2) as isize, y);
    let value = native_setup_word(
        0xbb1e,
        ctx.session().enemy_projectiles.spawn.primary as isize,
        ctx,
    );
    enemy!(ctx, value => (v2) as isize, program);
}

pub fn init_mother_brain_blue_ring_lasers(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 8 => (v1) as isize, behavior_a);
    enemy!(ctx, 0 => (v1) as isize, behavior_b);
    enemy!(ctx, 1024 => (v1) as isize, graphics);

    let r18: u16 = ctx.session().enemy_projectiles.spawn.primary;

    enemy!(ctx, x_velocity_at_angle(0x450, r18) => (v1) as isize, velocity_x);
    enemy!(ctx, y_velocity_at_angle(0x450, r18) => (v1) as isize, velocity_y);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).x.whole as i32 + 10) as u16 => (v1) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).y.whole as i32 + 16) as u16 => (v1) as isize, y);

    move_to_blue_ring_spawn_position(j, ctx);
}

pub fn init_mother_brain_hand_beam_charging(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, behavior_a);
    enemy!(ctx, 0 => (v1) as isize, behavior_b);
    enemy!(ctx, 0 => (v1) as isize, velocity_x);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);
    enemy!(ctx, 0 => (v1) as isize, x_fraction);
    enemy!(ctx, 0 => (v1) as isize, y_fraction);

    ctx.set_mother_brain_beam_x_fraction(0);
    ctx.set_mother_brain_beam_y_fraction(0);
    enemy!(ctx, 1024 => (v1) as isize, graphics);
    enemy!(
        ctx,
        v1 as isize,
        x = (ctx.resolve_actor(Key::from_legacy(0), Le).x.whole as i32 + 64) as u16
    );
    ctx.set_mother_brain_beam_x(enemy!(ctx, v1 as isize, x));
    let x: u16 = (ctx.session().player.x as i32 - ctx.mother_brain_beam_x() as i32) as u16;
    enemy!(
        ctx,
        v1 as isize,
        y = (ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32 - 48i32) as u16
    );
    ctx.set_mother_brain_beam_y(enemy!(ctx, v1 as isize, y));
    let y: u16 = (ctx.session().player.y as i32 - ctx.mother_brain_beam_y() as i32) as u16;
    let r18: i32 = -(ctx.calculate_angle_from_xy(x, y) as i32 + 0x80i32) as u8 as i32;
    ctx.set_mother_brain_beam_angle(r18 as u16);
    ctx.set_mother_brain_beam_velocity_x(x_velocity_at_angle(0xc00, r18 as u16));
    ctx.set_mother_brain_beam_velocity_y(y_velocity_at_angle(0xc00, r18 as u16));
}

pub fn init_tourian_statue_particle_tail(v0: u16, ctx: &mut impl Host) {
    let v1: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    let v2: i32 = v0 as i32 >> 1;
    enemy!(ctx, enemy!(ctx, (v1) as isize, x) => (v2) as isize, x);
    enemy!(ctx, enemy!(ctx, (v1) as isize, y) => (v2) as isize, y);
}

pub fn init_cacatac_spike(v0: u16, ctx: &mut impl Host) {
    let v1: u16 = ctx.session().enemy_projectiles.spawn.primary;
    let v2: i32 = v0 as i32 >> 1;

    enemy!(ctx, ctx.session().enemy_projectiles.spawn.primary => (v2) as isize, behavior_a);
    let value = native_setup_word(0xd96a, (v1 >> 1) as isize, ctx);
    enemy!(ctx, value => (v2) as isize, program);

    let enemy_index = ctx.inputs().current_enemy;

    copy_enemy_position(v2 as isize, enemy_index, 0, ctx);
    enemy!(ctx, -512i32 as u16 => (v2) as isize, velocity_y);
    enemy!(ctx, 512 => (v2) as isize, velocity_x);

    if (ctx.session().enemy_projectiles.spawn.primary as i32 - 12i32) & 0x8000 == 0 {
        enemy!(ctx, -384i32 as u16 => (v2) as isize, velocity_y);
        enemy!(ctx, 384 => (v2) as isize, velocity_x);
    }
}

pub fn init_dust_cloud_explosion(v0: u16, ctx: &mut impl Host) {
    let v1: i32 = v0 as i32 >> 1;
    let value = native_setup_word(
        0xe42c,
        ctx.session().enemy_projectiles.spawn.primary as isize,
        ctx,
    );
    enemy!(ctx, value => (v1) as isize, program);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.x => (v1) as isize, x);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.y => (v1) as isize, y);
}

pub fn init_draygon_goop(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.x => (v1) as isize, x);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.y => (v1) as isize, y);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.secondary => (v1) as isize, angle);
    let pt: Vector32 = convert_angle_to_xy(
        ctx.session().enemy_projectiles.spawn.secondary,
        ctx.session().enemy_projectiles.spawn.primary,
        ctx,
    );

    enemy!(ctx, (pt.x >> 16) as u16 => (v1) as isize, velocity_x);
    enemy!(ctx, pt.x as u16 => (v1) as isize, behavior_a);
    enemy!(ctx, (pt.y >> 16) as u16 => (v1) as isize, velocity_y);
    enemy!(ctx, pt.y as u16 => (v1) as isize, behavior_b);
    enemy!(ctx, 1024 => (v1) as isize, graphics);
}

pub fn init_crocomire_spike_wall_pieces(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    let value = native_setup_word(0x9105, ((j as i32 - 20i32) as u16 >> 1) as isize, ctx);
    enemy!(ctx, value => (v1) as isize, y);
    enemy!(ctx, 528 => (v1) as isize, x);

    enemy!(ctx, 0 => (v1) as isize, velocity_x);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);
    enemy!(ctx, 0 => (v1) as isize, behavior_a);
    enemy!(ctx, 0 => (v1) as isize, behavior_b);
    enemy!(ctx, 0 => (v1) as isize, x_fraction);
    enemy!(ctx, 0 => (v1) as isize, y_fraction);

    enemy!(ctx, -5i32 as u16 => (v1) as isize, velocity_y);
    enemy!(ctx, -30720i32 as u16 => (v1) as isize, behavior_b);
}

pub fn init_crocomire_bridge_crumbling(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, ctx.session().enemy_projectiles.spawn.primary => (v1) as isize, x);
    enemy!(ctx, 187 => (v1) as isize, y);

    enemy!(ctx, 0 => (v1) as isize, y_fraction);
    enemy!(ctx, 0 => (v1) as isize, x_fraction);
    enemy!(ctx, 0 => (v1) as isize, velocity_x);

    enemy!(ctx, ((ctx.inputs().random as i32 & 0x3f) + 64) as u16 => (v1) as isize, velocity_y);
    enemy!(ctx, 1024 => (v1) as isize, graphics);
}

pub fn init_unused_9634(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, behavior_a);
    enemy!(ctx, 0 => (v1) as isize, behavior_b);

    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(0), Le).x.whole as i32 - 29i32) as u16 => (v1) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32 - 35i32) as u16 => (v1) as isize, y);
    enemy!(ctx, 2560 => (v1) as isize, graphics);

    let v2: i32 = ctx.resolve_actor(Key::from_legacy(0), Le).parameters[0] as i32;
    enemy!(ctx, K_EPROJ_INIT_9634_XVEL[v2 as usize] as u16 => (v1) as isize, velocity_x);
    enemy!(ctx, K_EPROJ_INIT_9634_YVEL[v2 as usize] as u16 => (v1) as isize, velocity_y);
}

pub fn init_phantoon_starting_flames(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, x_fraction);
    enemy!(ctx, 0 => (v1) as isize, y_fraction);
    enemy!(ctx, 0 => (v1) as isize, velocity_x);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);

    let v2: u16 = BYTE_869979[ctx.session().enemy_projectiles.spawn.primary as usize] as u16;
    enemy!(ctx, v2 => (v1) as isize, behavior_a);
    let pt: Point = sine_offset::<false>(v2, 0x30);

    enemy!(ctx, (pt.x as i32 + ctx.resolve_actor(Key::from_legacy(0), Le).x.whole as i32) as u16 => (v1) as isize, x);
    enemy!(ctx, (pt.y as i32 + ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32 + 16) as u16 => (v1) as isize, y);
}

pub fn init_kraid_ceiling_rocks(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, ctx.session().enemy_projectiles.spawn.primary => (v1) as isize, x);
    enemy!(ctx, 312 => (v1) as isize, y);

    enemy!(ctx, 0 => (v1) as isize, y_fraction);
    enemy!(ctx, 0 => (v1) as isize, x_fraction);
    enemy!(ctx, 0 => (v1) as isize, velocity_x);

    enemy!(ctx, (ctx.inputs().random & 0x3f).wrapping_add(64) => (v1) as isize, velocity_y);
    enemy!(ctx, 1536 => (v1) as isize, graphics);
}

pub fn init_gunship_liftoff_dust_clouds(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, x_fraction);
    enemy!(ctx, 0 => (v1) as isize, y_fraction);
    enemy!(ctx, 0 => (v1) as isize, velocity_x);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);

    let v2: u16 = ctx.session().enemy_projectiles.spawn.primary;
    let v3: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;

    enemy!(ctx, (WORD_86_A2_D6[v3 as usize] as i32 + ctx.session().player.x as i32) as u16 => (v1) as isize, x);
    enemy!(ctx, (ctx.session().player.y as i32 + 80) as u16 => (v1) as isize, y);
    let value = native_setup_word(0xa2e2, v3 as isize, ctx);
    enemy!(ctx, value => (v1) as isize, program);
    enemy!(ctx, v2 => (v1) as isize, behavior_a);
}

pub fn init_bomb_torizo_initial_drool(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, graphics);
    ctx.next_random();
    let v2 = ctx.inputs().current_enemy as i32 as u16;

    {
        let value = ((ctx.resolve_actor(Key::from_legacy(v2), Le).y.whole as i32
            + (ctx.inputs().random as i32 & 3)) as u16)
            .wrapping_sub(5);
        enemy!(ctx, value => (v1) as isize, y);
    };
    enemy!(ctx, (ctx.inputs().random & 0x1f).wrapping_add(48) => (v1) as isize, velocity_y);

    ctx.next_random();
    let mut v3: i16 = ((ctx.inputs().random) as i16) & (3i16);
    let mut parameter_1: i16 = ctx.resolve_actor(Key::from_legacy(v2), Le).parameters[0] as i16;
    if (parameter_1 as u16 & 0x4000) != 0 {
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v2), Le).x.whole as i32 + v3 as i32) as u16 => (v1) as isize, x);
        enemy!(ctx, 0 => (v1) as isize, velocity_x);
    } else {
        if parameter_1 < 0 {
            enemy!(ctx, ((ctx.resolve_actor(Key::from_legacy(v2), Le).x.whole as i32 + v3 as i32) as u16).wrapping_add(8) => (v1) as isize, x);
        } else {
            enemy!(ctx, ((ctx.resolve_actor(Key::from_legacy(v2), Le).x.whole as i32 + v3 as i32) as u16).wrapping_sub(8) => (v1) as isize, x);
        }
        enemy!(ctx, 0 => (v1) as isize, velocity_x);
    };
}

pub fn init_projectile_a977(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, graphics);
    calculate_coordinates(ctx.session().mechanisms.selected, ctx);
    enemy!(ctx, (16 * ctx.session().mechanisms.block_x as i32) as u16 => (v1) as isize, x);
    enemy!(ctx, (16 * ctx.session().mechanisms.block_y as i32 - 4i32) as u16 => (v1) as isize, y);

    enemy!(ctx, 0 => (v1) as isize, x_fraction);
    enemy!(ctx, 0 => (v1) as isize, y_fraction);
}

pub fn init_tourian_statue_dust_clouds(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 128 => (v1) as isize, behavior_a);
    enemy!(ctx, 188 => (v1) as isize, behavior_b);
}

pub fn init_tourian_escape_shaft_wall_explosion(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 272 => (v1) as isize, x);
    enemy!(ctx, 272 => (v1) as isize, behavior_a);

    enemy!(ctx, 2184 => (v1) as isize, y);
    enemy!(ctx, 2184 => (v1) as isize, behavior_b);
}

pub fn init_tourian_statue_particle_splash(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, enemy!(ctx, (ctx.session().enemy_projectiles.spawn.primary >> 1) as isize, x) => (v1) as isize, x);
    enemy!(ctx, (ctx.inputs().fx_y as i32 - 4i32) as u16 => (v1) as isize, y);
}

pub fn init_tourian_statue_base_decoration(j: u16, ctx: &mut impl Host) {
    init_statue_piece(j, 120, 184, ctx);
}

pub fn init_tourian_statue_ridley(j: u16, ctx: &mut impl Host) {
    init_statue_piece(j, 142, 85, ctx);
}

pub fn init_tourian_statue_phantoon(j: u16, ctx: &mut impl Host) {
    init_statue_piece(j, 132, 136, ctx);
}

pub fn init_polyp_rock(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, ADDR_WORD_86BBD5 as u16 => (v1) as isize, program);
    enemy!(ctx, (0x86bc16 & 0xffff) as u16 => (v1) as isize, behavior_a);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.primary => (v1) as isize, velocity_y);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.secondary => (v1) as isize, velocity_x);
    let v2 = ctx.inputs().current_enemy as i32 as u16;

    copy_enemy_position(v1 as isize, v2, 0, ctx);
}

pub fn init_shaktool_attack_front_circle(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    let v2 = ctx.inputs().current_enemy as i32 as u16;

    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(v2), Le).x.whole => (v1) as isize, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(v2), Le).y.whole => (v1) as isize, y);

    let v3: u16 =
        (2 * (ctx.resolve_actor(Key::from_legacy(v2), Le).variables[3] as u8) as i32) as u16;
    let v4: i32 = v3 as i32 >> 1;
    enemy!(ctx, kSinCosTable8bit_Sext[(v4 + 64) as usize] as u16 => (v1) as isize, velocity_x);
    enemy!(ctx, kSinCosTable8bit_Sext[v4 as usize] as u16 => (v1) as isize, velocity_y);
    let v5: i32 = (v3 >> 5) as i32 >> 1;
    enemy_update!(ctx, (v1) as isize, x, |old| ((old as i32)
        + (K_EPROJ_INIT_SHAKTOOL_ATTACK_MIDDLE_BACK_CIRCLE_X[v5 as usize] as i32))
        as u16);
    enemy_update!(ctx, (v1) as isize, y, |old| ((old as i32)
        + (K_EPROJ_INIT_SHAKTOOL_ATTACK_MIDDLE_BACK_CIRCLE_Y[v5 as usize] as i32))
        as u16);
}

pub fn init_mother_brain_turrets(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 1024 => (v1) as isize, graphics);
    let v2: u16 = (2 * ctx.session().enemy_projectiles.spawn.primary as i32) as u16;
    let primary = ctx.session().enemy_projectiles.spawn.primary;
    let v3: u16 = native_setup_word(0xbee1, primary as isize, ctx) | 0x100;
    enemy!(ctx, v3 => (v1) as isize, y_fraction);
    let value = native_setup_word(0xbeb9, ((2 * v3 as u8 as i32) as u16 >> 1) as isize, ctx);
    enemy!(ctx, value => (v1) as isize, program);
    let v4: i32 = v2 as i32 >> 1;
    enemy!(ctx, K_EPROJ_INIT_MOTHER_BRAIN_ROOM_TURRETS_X[v4 as usize] as u16 => (v1) as isize, x);
    enemy!(ctx, EPROJ_INIT_MOTHER_BRAIN_ROOM_TURRETS_Y[v4 as usize] as u16 => (v1) as isize, y);
    let value = native_setup_word(0xbec9, v4 as isize, ctx);
    enemy!(ctx, value => (v1) as isize, x_fraction);
    set_random_rotation_timer(j, ctx);
    set_random_cooldown_timer(j, ctx);
}

pub fn init_mother_brain_turret_bullets(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, behavior_b);
    enemy!(ctx, 1024 => (v1) as isize, graphics);

    let v2: u16 = (2 * enemy_byte!(
        ctx,
        (ctx.session().enemy_projectiles.spawn.primary >> 1) as isize * 2,
        y_fraction
    ) as i32) as u16;
    enemy!(ctx, v2 => (v1) as isize, behavior_a);
    let v3: i32 = v2 as i32 >> 1;
    let pt: Point = Point {
        x: K_EPROJ_MOTHER_BRAIN_ROOM_TURRET_BULLETS_X[v3 as usize] as u16,
        y: K_EPROJ_MOTHER_BRAIN_ROOM_TURRET_BULLETS_Y[v3 as usize] as u16,
    };
    enemy!(ctx, K_EPROJ_MOTHER_BRAIN_ROOM_TURRET_BULLETS_XVEL[v3 as usize] as u16 => (v1) as isize, velocity_x);
    enemy!(ctx, K_EPROJ_MOTHER_BRAIN_ROOM_TURRET_BULLETS_YVEL[v3 as usize] as u16 => (v1) as isize, velocity_y);
    let v4: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    enemy!(ctx, (pt.x as i32 + enemy!(ctx, (v4) as isize, x) as i32) as u16 => (v1) as isize, x);
    enemy!(ctx, (pt.y as i32 + enemy!(ctx, (v4) as isize, y) as i32) as u16 => (v1) as isize, y);
}

pub fn init_mother_brain_hand_beam_fired(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, ctx.mother_brain_beam_x() => (v1) as isize, x);
    enemy!(ctx, ctx.mother_brain_beam_x_fraction() => (v1) as isize, x_fraction);
    enemy!(ctx, ctx.mother_brain_beam_y() => (v1) as isize, y);
    enemy!(ctx, ctx.mother_brain_beam_y_fraction() => (v1) as isize, y_fraction);
    enemy!(ctx, ctx.mother_brain_beam_velocity_x() => (v1) as isize, velocity_x);
    enemy!(ctx, ctx.mother_brain_beam_velocity_y() => (v1) as isize, velocity_y);

    move_with_velocity(j, ctx);
    ctx.set_mother_brain_beam_x(enemy!(ctx, (v1) as isize, x));
    ctx.set_mother_brain_beam_x_fraction(enemy!(ctx, (v1) as isize, x_fraction));
    ctx.set_mother_brain_beam_y(enemy!(ctx, (v1) as isize, y));
    ctx.set_mother_brain_beam_y_fraction(enemy!(ctx, (v1) as isize, y_fraction));
    let r18: u16 =
        ((ctx.mother_brain_beam_angle() as u8) as i32 + ctx.next_random() as i32) as u8 as u16;
    let rv: u16 = ctx.next_random();
    enemy!(ctx, x_velocity_at_angle(rv & 0x700, r18) => (v1) as isize, velocity_x);
    enemy!(ctx, y_velocity_at_angle(ctx.inputs().random & 0x700, r18) => (v1) as isize, velocity_y);
    move_with_velocity(j, ctx);
    if (enemy!(ctx, (v1) as isize, y) as i32 - 34i32) & 0x8000 != 0
        || (enemy!(ctx, (v1) as isize, y) as i32 - 206i32) & 0x8000 == 0
        || (enemy!(ctx, (v1) as isize, x) as i32 - 2i32) & 0x8000 != 0
        || (enemy!(ctx, (v1) as isize, x) as i32 - 238i32) & 0x8000 == 0
    {
        enemy!(ctx, 0 => (v1) as isize, definition);
        ctx.session_mut().enemy_projectiles.spawn.point = Point {
            x: enemy!(ctx, (v1) as isize, x),
            y: enemy!(ctx, (v1) as isize, y),
        };
        spawn_with_room_gfx(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 0x1d, ctx);
        ctx.queue_sound(3, 6, 0x13);
        ctx.session_mut().feedback.earthquake_timer = 10;
        ctx.session_mut().feedback.earthquake_kind = 5;
    } else {
        enemy!(ctx, ((enemy!(ctx, (v1) as isize, behavior_a) as i32 + 1) & 3) as u16 => (v1) as isize, behavior_a);

        enemy!(ctx, 0 => (v1) as isize, behavior_b);
        enemy!(ctx, 0 => (v1) as isize, velocity_x);
        enemy!(ctx, 0 => (v1) as isize, velocity_y);
    };
}

pub fn init_mother_brain_rainbow_beam_charging(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, graphics);
    enemy!(ctx, 0 => (v1) as isize, velocity_x);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);

    mother_brain_rainbow_beam_charging_pre_instruction(j, ctx);
}

pub fn init_mother_brain_drool(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, graphics);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.primary => (v1) as isize, behavior_a);

    mother_brain_drool_attached_pre_instruction(j, ctx);
}

pub fn init_mother_brain_death_explosion(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    let value = native_setup_word(
        0xc929,
        ctx.session().enemy_projectiles.spawn.primary as isize,
        ctx,
    );
    enemy!(ctx, value => (v1) as isize, program);
    enemy!(ctx, 1 => (v1) as isize, delay);
    enemy!(ctx, 0 => (v1) as isize, graphics);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.x => (v1) as isize, velocity_x);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.y => (v1) as isize, velocity_y);
    mother_brain_death_explosion_pre_instruction(j, ctx);
}

pub fn init_mother_brain_rainbow_beam_explosion(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, graphics);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.x => (v1) as isize, velocity_x);
    {
        let value = (ctx.session().player.x as i32
            + ctx.session().enemy_projectiles.spawn.point.x as i32) as u16;
        enemy!(ctx, value => (v1) as isize, x);
    };
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.y => (v1) as isize, velocity_y);
    {
        let value = (ctx.session().player.y as i32
            + ctx.session().enemy_projectiles.spawn.point.y as i32) as u16;
        enemy!(ctx, value => (v1) as isize, y);
    };
}

pub fn init_mother_brain_purple_breath_big(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, graphics);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).x.whole as i32 + 6) as u16 => (v1) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).y.whole as i32 + 16) as u16 => (v1) as isize, y);
}

pub fn init_mother_brain_purple_breath_small(j: u16, ctx: &mut impl Host) {
    init_mother_brain_purple_breath_big(j, ctx);
    ctx.set_mother_brain_hurt_frame(1);
}

pub fn init_mother_brain_tube_falling(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 3584 => (v1) as isize, graphics);

    enemy!(ctx, 0 => (v1) as isize, velocity_x);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);

    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.x => (v1) as isize, x);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.y => (v1) as isize, y);
    enemy!(ctx, (0x86cbea & 0xffff) as u16 => (v1) as isize, behavior_a);
}

pub fn init_powamp_spike(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, x_fraction);
    enemy!(ctx, 0 => (v1) as isize, y_fraction);
    enemy!(ctx, 0 => (v1) as isize, velocity_x);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);

    let v2 = ctx.inputs().current_enemy as i32 as u16;

    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(v2), Le).x.whole => (v1) as isize, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(v2), Le).y.whole => (v1) as isize, y);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.primary => (v1) as isize, behavior_a);
}

pub fn init_spore_spawn_stalk(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    {
        let value = (ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32
            + WORD_86_DCB9[ctx.session().enemy_projectiles.spawn.primary as usize] as i32)
            as u16;
        enemy!(ctx, value => (v1) as isize, y);
    };
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(0), Le).x.whole => (v1) as isize, x);
}

pub fn init_spore_spawner(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, K_EPROJ_INIT_SPORE_SPAWNERS_X[ctx.session().enemy_projectiles.spawn.primary as usize] as u16 => (v1) as isize, x);
    enemy!(ctx, 520 => (v1) as isize, y);
}

pub fn init_plm_dust_cloud(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    let value = native_setup_word(
        0xe42c,
        ctx.session().enemy_projectiles.spawn.primary as u8 as isize,
        ctx,
    );
    enemy!(ctx, value => (v1) as isize, program);
    let v2: i32 = (8 * (ctx.session().enemy_projectiles.spawn.primary >> 8) as u8 as i32) >> 1;
    let x: u16 = (native_setup_word(0xe47e, (v2 + 2) as isize, ctx) as i32
        + ({
            let record = ctx.loaded_record(0x86, 0xe47e);
            let index = v2 as isize;
            record.native_word(index.checked_mul(2).expect("ROM table offset overflow"))
        } as i32
            & ctx.inputs().random as i32)) as u16;
    let y: u16 = (native_setup_word(0xe47e, (v2 + 3) as isize, ctx) as i32
        + (native_setup_word(0xe47e, (v2 + 1) as isize, ctx) as i32
            & ctx.inputs().random as i32 >> 8)) as u16;
    calculate_coordinates(ctx.session().mechanisms.selected, ctx);
    enemy!(ctx, (x as i32 + 8 * (2 * ctx.session().mechanisms.block_x as i32 + 1)) as u16 => (v1) as isize, x);
    enemy!(ctx, (y as i32 + 8 * (2 * ctx.session().mechanisms.block_y as i32 + 1)) as u16 => (v1) as isize, y);
    ctx.next_random();
}

pub fn init_botwoon_body(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(0), Le).x.whole => (v1) as isize, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(0), Le).y.whole => (v1) as isize, y);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);

    let v2: u16 = (if ctx.resolve_actor(Key::from_legacy(0), Le).variables[0] != 0 {
        16
    } else {
        48
    }) as u16;
    let v3: u16 = native_setup_word(0xe9f1, (v2 >> 1) as isize, ctx);

    enemy!(ctx, v3 => (v1) as isize, program);
    enemy!(ctx, v3 => (v1) as isize, behavior_b);
    enemy!(ctx, v2 => (v1) as isize, behavior_a);
    enemy!(ctx, (0x86ea98 & 0xffff) as u16 => (v1) as isize, velocity_x);

    let ex = ctx.resolve_actor(Key::from_legacy(0), Le).variables[0];
    ctx.set_enemy_linked_projectile(ex, j);
    enemy!(ctx, 2 => (v1) as isize, drop_flags);
    ctx.set_botwoon_body_active(ex, 1);
}

pub fn init_crocomire_projectile(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, -512i32 as u16 => (v2) as isize, velocity_x);
    enemy!(ctx, 1 => (v2) as isize, velocity_y);

    let v3 = ctx.selected_actor();
    enemy!(ctx, (ctx.resolve_actor(v3, Le).x.whole as i32 - 32i32) as u16 => (v2) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(v3, Le).y.whole as i32 - 16i32) as u16 => (v2) as isize, y);

    enemy!(ctx, 0 => (v2) as isize, counter);
    enemy!(ctx, 0 => (v2) as isize, x_fraction);
    enemy!(ctx, 0 => (v2) as isize, y_fraction);

    enemy!(ctx, 2560 => (v2) as isize, graphics);
}

pub fn init_pirate_laser(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    {
        let value = (if ctx.session().enemy_projectiles.spawn.variant != 0 {
            ADDR_WORD_869F7D as i32
        } else {
            ADDR_WORD_869F41 as i32
        }) as u16;
        enemy!(ctx, value => (v2) as isize, program);
    };
    enemy!(ctx, (0x86a05b & 0xffff) as u16 => (v2) as isize, pre_action);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.x => (v2) as isize, x);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.y => (v2) as isize, y);
    let enemy_index = ctx.inputs().current_enemy;
    enemy!(ctx, ctx.native_record(0xa0, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).definition).word_le(3) | 0x1000 => (v2) as isize, properties);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).parameters[0] => (v2) as isize, behavior_a);
    ctx.queue_sound(2, 6, 0x67);
}

pub fn init_golden_torizo_super_missile(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, ctx.inputs().current_enemy => (v2) as isize, behavior_a);
    let v3 = ctx.inputs().current_enemy as i32 as u16;
    let v4: i32 = if (ctx.resolve_actor(Key::from_legacy(v3), Le).parameters[0] & 0x8000) != 0 {
        1
    } else {
        0
    };
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v3), Le).x.whole as i32 + WORD_86_B205[v4 as usize] as i32) as u16 => (v2) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v3), Le).y.whole as i32 - 52i32) as u16 => (v2) as isize, y);
    let value = native_setup_word(0xb209, v4 as isize, ctx);
    enemy!(ctx, value => (v2) as isize, program);
}

pub fn init_kihunter_acid_spit_left(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, -768i32 as u16 => (v2) as isize, velocity_x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(ctx.inputs().current_enemy), Le).x.whole as i32 - 22i32) as u16 => (v2) as isize, x);
    init_kihunter_acid_spit(ctx.inputs().current_enemy, j, ctx);
}

pub fn init_kihunter_acid_spit_right(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, 768 => (v2) as isize, velocity_x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(ctx.inputs().current_enemy), Le).x.whole as i32 + 22) as u16 => (v2) as isize, x);
    init_kihunter_acid_spit(ctx.inputs().current_enemy, j, ctx);
}

pub fn init_kago_bug(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, ctx.inputs().current_enemy => (v2) as isize, behavior_b);
    let enemy_index = ctx.inputs().current_enemy;
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole => (v2) as isize, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).y.whole => (v2) as isize, y);
    let v4: u16 = ((ctx.inputs().random as i32 & 7) + 1) as u16;

    enemy!(ctx, v4 => (v2) as isize, behavior_c);
    enemy!(ctx, v4.wrapping_add(4) => (v2) as isize, behavior_a);
    enemy!(ctx, (0x86d0cai32) as u16 => (v2) as isize, pre_action);
}

pub fn init_work_robot_laser_down(j: u16, ctx: &mut impl Host) {
    init_robot_laser(j, 128, true, ctx);
}

pub fn init_work_robot_laser_horizontal(j: u16, ctx: &mut impl Host) {
    init_robot_laser(j, 0, false, ctx);
}

pub fn init_work_robot_laser_up(j: u16, ctx: &mut impl Host) {
    init_robot_laser(j, (-128i16) as u16, false, ctx);
}

pub fn init_mini_crocomire_projectile(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, ADDR_WORD_86DB0C as u16 => (v2) as isize, program);
    enemy!(ctx, (0x86db62 & 0xffff) as u16 => (v2) as isize, behavior_a);
    if ctx.session().enemy_projectiles.spawn.primary != 0 {
        enemy!(ctx, (0x86db8c & 0xffff) as u16 => (v2) as isize, behavior_a);
    }
    let v3 = ctx.inputs().current_enemy as i32 as u16;

    copy_enemy_position(v2 as isize, v3, 2, ctx);
    enemy!(ctx, -256i32 as u16 => (v2) as isize, velocity_y);
    enemy!(ctx, 256 => (v2) as isize, velocity_x);
}

pub fn init_spores(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    enemy!(
        ctx,
        v2 as isize,
        x = ctx.session().enemy_projectiles.spawn.point.x
    );

    enemy!(ctx, enemy!(ctx, v2 as isize, x) => (v2) as isize, behavior_b);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.y => (v2) as isize, y);
    enemy!(ctx, 512 => (v2) as isize, graphics);
}

pub fn init_namihe_fune_fireball(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, ADDR_WORD_86DE96 as u16 => (v2) as isize, program);
    enemy!(ctx, (0x86df40 & 0xffff) as u16 => (v2) as isize, behavior_a);
    if ctx.session().enemy_projectiles.spawn.primary != 0 {
        enemy!(ctx, ADDR_WORD_86DEA6 as u16 => (v2) as isize, program);
        enemy!(ctx, (0x86df6a & 0xffff) as u16 => (v2) as isize, behavior_a);
    }
    let v3 = ctx.inputs().current_enemy as i32 as u16;

    copy_enemy_position(v2 as isize, v3, 0, ctx);

    if (ctx.resolve_actor(Key::from_legacy(v3), Le).parameters[0] & 0xf) != 0 {
        enemy_update!(ctx, (v2) as isize, y, |old| ((old as i32) + (4)) as u16);
    }
    let v4: i32 = (4 * (ctx.resolve_actor(Key::from_legacy(v3), Le).parameters[1] as u8) as i32)
        as u16 as i32
        >> 1;
    let value = native_setup_word(0xdeb6, v4 as isize, ctx);
    enemy!(ctx, value => (v2) as isize, velocity_y);
    let value = native_setup_word(0xdeb6, (v4 + 1) as isize, ctx);
    enemy!(ctx, value => (v2) as isize, velocity_x);
}

pub fn init_lava_thrown_by_magdollite(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, ADDR_WORD_86DFD8 as u16 => (v2) as isize, program);
    enemy!(ctx, (0x86e050 & 0xffff) as u16 => (v2) as isize, behavior_a);
    if ctx.session().enemy_projectiles.spawn.primary != 0 {
        enemy!(ctx, ADDR_WORD_86DFDE as u16 => (v2) as isize, program);
        enemy!(ctx, (0x86e07a & 0xffff) as u16 => (v2) as isize, behavior_a);
    }
    let v3 = ctx.inputs().current_enemy as i32 as u16;

    copy_enemy_position(v2 as isize, v3, 2, ctx);
    enemy!(ctx, -768i32 as u16 => (v2) as isize, velocity_y);
    enemy!(ctx, 768 => (v2) as isize, velocity_x);
}

pub fn init_mini_kraid_spit(j: u16, ctx: &mut impl Host) {
    let enemy_index = ctx.inputs().current_enemy;

    let extra_index = ctx.inputs().current_enemy;
    let v3: i32 = j as i32 >> 1;
    {
        let value = (ctx.mini_kraid_spit_x_offset(extra_index) as i32
            + ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole as i32)
            as u16;
        enemy!(ctx, value => (v3) as isize, x);
    };
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(enemy_index), Le).y.whole as i32 - 16i32) as u16 => (v3) as isize, y);

    enemy!(ctx, 0 => (v3) as isize, y_fraction);
    enemy!(ctx, 0 => (v3) as isize, x_fraction);

    enemy!(ctx, ctx.mini_kraid_spit_x_velocity(extra_index) => (v3) as isize, velocity_x);
    enemy!(ctx, ctx.mini_kraid_spit_y_velocity(extra_index) => (v3) as isize, velocity_y);
}

pub fn init_bomb_torizo_low_health_explosion(j: u16, ctx: &mut impl Host) {
    let enemy_index = ctx.inputs().current_enemy;

    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole => (v2) as isize, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).y.whole => (v2) as isize, y);
    if (ctx
        .resolve_actor(Key::from_legacy(enemy_index), Le)
        .parameters[0]
        & 0x8000)
        == 0
    {
        let spawn = &mut ctx.session_mut().enemy_projectiles.spawn;
        spawn.primary = spawn.primary.wrapping_add(2);
    }
    let spawn = &mut ctx.session_mut().enemy_projectiles.spawn;
    spawn.primary = spawn.primary.wrapping_add(2);
    let v3: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    let v4: u16 = (K_EPROJ_INIT_BOMB_TORIZO_LOW_HEALTH_EXPLODE_X[v3 as usize] as i32
        + enemy!(ctx, (v2) as isize, x) as i32) as u16;

    enemy!(ctx, v4 => (v2) as isize, x);
    enemy!(ctx, v4 => (v2) as isize, behavior_a);

    let v5: u16 = (K_EPROJ_INIT_BOMB_TORIZO_LOW_HEALTH_EXPLODE_Y[v3 as usize] as i32
        + enemy!(ctx, (v2) as isize, y) as i32) as u16;

    enemy!(ctx, v5 => (v2) as isize, y);
    enemy!(ctx, v5 => (v2) as isize, behavior_b);
}

pub fn init_torizo_death_explosion(j: u16, ctx: &mut impl Host) {
    let enemy_index = ctx.inputs().current_enemy;

    let x_pos: u16 = ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole;
    let v3: i32 = j as i32 >> 1;

    enemy!(ctx, x_pos => (v3) as isize, x);
    enemy!(ctx, x_pos => (v3) as isize, behavior_a);

    let y_pos: u16 = ctx.resolve_actor(Key::from_legacy(enemy_index), Le).y.whole;

    enemy!(ctx, y_pos => (v3) as isize, y);
    enemy!(ctx, y_pos => (v3) as isize, behavior_b);
}

pub fn init_bomb_torizo_chozo_orbs(j: u16, ctx: &mut impl Host) {
    let enemy_index = ctx.inputs().current_enemy;

    let v2: u16 = (if (ctx
        .resolve_actor(Key::from_legacy(enemy_index), Le)
        .parameters[0]
        & 0x8000)
        != 0
    {
        ADDR_K_EPROJ_INIT_BOMB_TORIZOS_CHOZO_ORBS_INIT0 as i32
    } else {
        ADDR_K_EPROJ_INIT_BOMB_TORIZOS_CHOZO_ORBS_INIT1 as i32
    }) as u16;
    init_torizo_projectile(
        j,
        v2,
        Point {
            x: ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole,
            y: ctx.resolve_actor(Key::from_legacy(enemy_index), Le).y.whole,
        },
        ctx,
    );
}

pub fn init_torizo_landing_dust_cloud_right_foot(j: u16, ctx: &mut impl Host) {
    init_landing_dust(j, 24, ctx);
}

pub fn init_torizo_landing_dust_cloud_left_foot(j: u16, ctx: &mut impl Host) {
    init_landing_dust(j, -24, ctx);
}

pub fn init_yapping_maw_body(j: u16, ctx: &mut impl Host) {
    let enemy_index = ctx.inputs().current_enemy;

    let v3: i32 = j as i32 >> 1;
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole => (v3) as isize, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).y.whole => (v3) as isize, y);
    enemy!(ctx, ADDR_WORD_86EC5C as u16 => (v3) as isize, program);
    if ctx
        .resolve_actor(Key::from_legacy(enemy_index), Le)
        .parameters[1]
        == 0
    {
        enemy!(ctx, ADDR_WORD_86EC56 as u16 => (v3) as isize, program);
    }
    let base = ctx.inputs().current_enemy;
    let part = ctx.maw_body_part_index(ctx.inputs().current_enemy);
    ctx.set_maw_body_link((base as i32 + 2 * part as i32) as u16, j);
}

pub fn init_enemy_death_explosion(j: u16, ctx: &mut impl Host) {
    let enemy_index = ctx.inputs().current_enemy;

    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole => (v2) as isize, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).y.whole => (v2) as isize, y);
    enemy!(ctx, ctx.inputs().current_enemy => (v2) as isize, killed_enemy);
    if (ctx
        .resolve_actor(Key::from_legacy(enemy_index), Le)
        .properties
        & 0x4000)
        != 0
    {
        enemy!(ctx, (ctx.inputs().current_enemy) | (0x8000u16) => (v2) as isize, killed_enemy);
    }
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).definition => (v2) as isize, drop_definition);
    enemy!(ctx, 0 => (v2) as isize, graphics);
    {
        let instruction: u16 = native_setup_word(
            0xefd5,
            ctx.session().enemy_projectiles.spawn.primary as isize,
            ctx,
        );
        {
            let index = (v2 as usize) as isize;
            enemy!(ctx, index, program = instruction);
            enemy!(ctx, index, delay = 1);
        };
    }
}

pub fn init_vertical_afterburn_up(j: u16, ctx: &mut impl Host) {
    init_fireball_child(j, 0, (-3584i16) as u16, 0x9688, ctx)
}

pub fn init_vertical_afterburn_down(j: u16, ctx: &mut impl Host) {
    init_fireball_child(j, 0, 3584, 0x9696, ctx)
}

pub fn init_horizontal_afterburn_right(j: u16, ctx: &mut impl Host) {
    init_fireball_child(j, 3584, 0, 0x966c, ctx)
}

pub fn init_horizontal_afterburn_left(j: u16, ctx: &mut impl Host) {
    init_fireball_child(j, (-3584i16) as u16, 0, 0x967a, ctx)
}

pub fn init_mini_kraid_spikes_left(j: u16, ctx: &mut impl Host) {
    init_mini_kraid_spikes(j, -0x200i32 as u16, ctx)
}

pub fn init_mini_kraid_spikes_right(j: u16, ctx: &mut impl Host) {
    init_mini_kraid_spikes(j, 0x200, ctx)
}

pub fn init_spawned_shot_gate(j: u16, ctx: &mut impl Host) {
    init_shot_gate(j, 0, ctx)
}

pub fn init_closed_downwards_shot_gate(j: u16, ctx: &mut impl Host) {
    init_shot_gate(j, 0x40, ctx)
}

pub fn init_closed_upwards_shot_gate(j: u16, ctx: &mut impl Host) {
    init_shot_gate(j, -0x40i32 as u16, ctx)
}
