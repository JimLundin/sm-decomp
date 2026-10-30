//! Owned enemy-projectile pre-actions; compatibility and effects stay at the execution edge.
#![forbid(unsafe_code)]
use super::access::{enemy, enemy_add, enemy_byte, enemy_decrement, enemy_update, read_member};
use super::addresses;
use super::host::{EnemyMember as SlotField, Host, LoadedRecord};
use super::pre_helpers::*;
use super::shared::spawn_with_room_gfx;
use super::*;
use crate::actors::access::{AccessMode::LittleEndian as Le, ActorKey as Key};
use crate::geometry::Point;
use crate::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;
use crate::{
    sprite_effect_state::{LegacySpriteDefinition, SpriteSpawn},
    sprite_objects,
};

pub fn dispatch(ea: u32, k: u16, ctx: &mut impl Host) {
    use addresses::pre::*;
    match ea {
        SKREE_PARTICLE => skree_particle_pre_instruction(k, ctx),
        CACATAC_SPIKES_MOVE_Y1 => move_cacatac_spike_up(k, ctx),
        CACATAC_SPIKES_MOVE_X1 => move_cacatac_spike_left(k, ctx),
        CACATAC_SPIKES_MOVE_X2 => move_cacatac_spike_right(k, ctx),
        CACATAC_SPIKES_6_MOVE_X1_Y1 => cacatac_spike_up_left(k, ctx),
        CACATAC_SPIKES_7_MOVE_X2_Y1 => cacatac_spike_up_right(k, ctx),
        DUST_CLOUD_OR_EXPLOSION => dust_cloud_explosion_pre_instruction(k, ctx),
        AT_8DCA => draygon_goop_stuck_to_samus_pre_instruction(k, ctx),
        DRAYGONS_TURRET_8_DFF => draygon_turret_projectile_fired_pre_instruction(k, ctx),
        DRAYGON_GOOP_8_E0_F => draygon_goop_pre_instruction(k, ctx),
        CROCOMIRE_PROJECTILE => crocomire_projectile_setup_pre_instruction(k, ctx),
        SUB_8690_B3 => crocomire_projectile_fired_pre_instruction(k, ctx),
        CROCOMIRE_BRIDGE_CRUMBLING => crocomire_bridge_crumbling_pre_instruction(k, ctx),
        RIDLEYS_FIREBALL => ridleys_fireball_pre_instruction(k, ctx),
        AT_966C => horizontal_afterburn_pre_instruction(k, ctx),
        AT_9688 => vertical_afterburn_pre_instruction(k, ctx),
        AT_96A4 => proto_horizontal_afterburn_pre_instruction(k, ctx),
        AT_96C0 => proto_vertical_afterburn_up_pre_instruction(k, ctx),
        CERES_FALLING_DEBRIS => ceres_falling_debris_pre_instruction(k, ctx),
        PHANTOON_DESTROYABLE_FIREBALLS => phantoon_enraged_flame_pre_instruction(k, ctx),
        PHANTOON_DESTROYABLE_FIREBALLS_3 => phantoon_flame_spiral_pre_instruction(k, ctx),
        PHANTOON_STARTING_FIREBALLS_B => phantoon_starting_flames_pre_instruction(k, ctx),
        KRAID_ROCKS => kraid_rocks_pre_instruction(k, ctx),
        ROCKS_FALLING_KRAID_CEILING => kraid_ceiling_rocks_pre_instruction(k, ctx),
        SUB_869_DA5 => kraid_rocks_use_palette_0_pre_instruction(k, ctx),
        MINI_KRAID_SPIT => mini_kraid_spit_pre_instruction(k, ctx),
        MINI_KRAID_SPIKES => mini_kraid_spikes_pre_instruction(k, ctx),
        ALCOON_FIREBALL => alcoon_fireball_pre_instruction(k, ctx),
        CERES_ELEVATOR_PAD => ceres_elevator_pad_pre_instruction(k, ctx),
        CERES_ELEVATOR_PLATFORM => ceres_elevator_pad_concealer_pre_instruction(k, ctx),
        AT_A977 => bomb_torizo_statue_breaking_falling_pre_instruction(k, ctx),
        BOMB_TORIZOS_CHOZO_ORBS => bomb_torizos_chozo_orbs_pre_instruction(k, ctx),
        GOLDEN_TORIZOS_CHOZO_ORBS => golden_torizos_chozo_orbs_pre_instruction(k, ctx),
        TORIZO_SONIC_BOOM => torizo_sonic_boom_pre_instruction(k, ctx),
        SUB_86_B0_B9 => golden_torizo_egg_hatched_pre_instruction(k, ctx),
        SUB_86_B0_DD => golden_torizo_egg_hit_wall_pre_instruction(k, ctx),
        AT_B237 => golden_torizo_super_missile_thrown_pre_instruction(k, ctx),
        GOLDEN_TORIZO_EYE_BEAM => golden_torizo_eye_beam_pre_instruction(k, ctx),
        EYE_DOOR_PROJECTILE => eye_door_projectile_moving_pre_instruction(k, ctx),
        EYE_DOOR_SWEAT => eye_door_sweat_pre_instruction(k, ctx),
        TOURIAN_STATUE_UNLOCKING_PARTICLE_WATER_SPLASH => {
            tourian_statue_particle_splash_pre_instruction(k, ctx)
        }
        TOURIAN_STATUE_STUFF => tourian_statue_base_decoration_pre_instruction(k, ctx),
        DESPAWN_OFF_SCREEN_BD1E => despawn_off_screen(k, ctx),
        POLYP_ROCKS => polyp_rock_pre_instruction(k, ctx),
        MOTHER_BRAIN_ROOM_TURRETS => mother_brain_turrets_pre_instruction(k, ctx),
        MOTHER_BRAIN_BOMB => mother_brain_bomb_pre_instruction(k, ctx),
        MOTHER_BRAIN_RAINBOW_BEAM => mother_brain_rainbow_beam_charging_pre_instruction(k, ctx),
        MOTHER_BRAINS_DEATH_EXPLOSION_0 => mother_brain_death_explosion_pre_instruction(k, ctx),
        MOTHER_BRAIN_TUBE_FALLING => mother_brain_tube_falling_pre_instruction(k, ctx),
        MOTHER_BRAIN_GLASS_SHATTERING_SHARD => mother_brain_glass_shard_pre_instruction(k, ctx),
        KAGOS_BUGS => kago_bug_idling_pre_instruction(k, ctx),
        AT_D0EC => kago_bug_jumping_pre_instruction(k, ctx),
        N00B_TUBE_SHARDS => n00b_tube_shard_flying_pre_instruction(k, ctx),
        SUB_86_D83_D => n00b_tube_shard_falling_pre_instruction(k, ctx),
        SUB_86_D89_F => n00b_tube_air_bubbles_falling_pre_instruction(k, ctx),
        SUB_86_D8_DF => n00b_tube_air_bubbles_flying_pre_instruction(k, ctx),
        CACATAC_SPIKES_4_MOVE_Y2 => cacatac_spike_down(k, ctx),
        CACATAC_SPIKES_8_MOVE_X1_Y2 => cacatac_spike_down_left(k, ctx),
        CACATAC_SPIKES_9_MOVE_X2_Y2 => cacatac_spike_down_right(k, ctx),
        CACATAC_SPIKES => cacatac_spike_pre_instruction(k, ctx),
        DESPAWN_OFF_SCREEN_DBB6 => despawn_off_screen(k, ctx),
        AT_DBF2 => mini_crocomire_projectile_pre_instruction(k, ctx),
        SPORE_SPAWNERS => spore_spawner_pre_instruction(k, ctx),
        BOTWOONS_BODY => botwoons_body_pre_instruction(k, ctx),
        BOTWOONS_SPIT => botwoons_spit_pre_instruction(k, ctx),
        SUB_86_A887 => bomb_torizo_drool_falling_pre_instruction(k, ctx),
        AT_D128 => kago_bug_falling_pre_instruction(k, ctx),
        NAMIHE_FUNE_FIREBALL => namihe_fune_fireball_pre_instruction(k, ctx),
        SUB_86_E049 => lava_thrown_by_magdollite_pre_instruction(k, ctx),
        AT_BA42 => tourian_statue_decoration_pre_instruction(k, ctx),
        MOTHER_BRAINS_DROOL_FALLING => mother_brain_drool_falling_pre_instruction(k, ctx),
        SHAKTOOL_ATTACK_FRONT_CIRCLE => shaktool_attack_front_circle_pre_instruction(k, ctx),
        PRE_INIT_0X8AAF => projectile_8aaf_pre_instruction(k, ctx),
        CROCOMIRE_SPIKE_WALL_PIECES => crocomire_spike_wall_pieces_pre_instruction(k, ctx),
        AT_9634 => unused_9634_pre_instruction(k, ctx),
        PHANTOON_STARTING_FIREBALLS => phantoon_casual_flame_falling_pre_instruction(k, ctx),
        PHANTOON_STARTING_FIREBALLS2 => phantoon_casual_flame_hit_ground_pre_instruction(k, ctx),
        PHANTOON_STARTING_FIREBALLS3 => phantoon_casual_flame_bouncing_pre_instruction(k, ctx),
        PHANTOON_DESTROYABLE_FIREBALLS_2 => phantoon_flame_rain_pre_instruction(k, ctx),
        PHANTOON_STARTING_FIREBALLS_B_2 => {
            phantoon_starting_flames_activated_pre_instruction(k, ctx)
        }
        PIRATE_MOTHER_BRAIN_LASER_MOVE_LEFT => pirate_laser_move_left_pre_instruction(k, ctx),
        PIRATE_MOTHER_BRAIN_LASER_MOVE_RIGHT => pirate_laser_move_right_pre_instruction(k, ctx),
        PIRATE_CLAW_THROWN_LEFT => pirate_claw_thrown_left_pre_instruction(k, ctx),
        PIRATE_CLAW_THROWN_RIGHT => pirate_claw_thrown_right_pre_instruction(k, ctx),
        AT_AB07 => projectile_ab07_pre_instruction(k, ctx),
        GOLDEN_TORIZO_EGG => golden_torizo_egg_bouncing_pre_instruction(k, ctx),
        GOLDEN_TORIZO_SUPER_MISSILE => golden_torizo_super_missile_held_pre_instruction(k, ctx),
        SUB_86_B535 => dragon_fireball_pre_instruction(k, ctx),
        TOURIAN_STATUE_UNLOCKING_PARTICLE => {
            tourian_statue_unlocking_particle_pre_instruction(k, ctx)
        }
        TOURIAN_STATUE_SOUL => tourian_statue_soul_pre_instruction(k, ctx),
        AT_BE12 => shaktool_attack_back_circle_moving_pre_instruction(k, ctx),
        MOTHER_BRAIN_ROOM_TURRET_BULLETS => mother_brain_turret_bullets_pre_instruction(k, ctx),
        MOVE_TO_BLUE_RING_SPAWN_POSITION => mother_brain_blue_ring_lasers_pre_instruction(k, ctx),
        AT_C84D => mother_brain_drool_attached_pre_instruction(k, ctx),
        MOTHER_BRAINS_RAINBOW_BEAM_EXPLOSION => {
            mother_brain_rainbow_beam_explosion_pre_instruction(k, ctx)
        }
        MOTHER_BRAINS_EXPLODED_DOOR_PARTICLES => {
            mother_brain_escape_door_particles_pre_instruction(k, ctx)
        }
        TIME_BOMB_SET_JAPANESE_TEXT => time_bomb_set_japanese_text_pre_instruction(k, ctx),
        SUB_86_CFD5 => kihunter_acid_spit_left_start_pre_instruction(k, ctx),
        SUB_86_CFE6 => kihunter_acid_spit_right_start_pre_instruction(k, ctx),
        SUB_86_CFF8 => kihunter_acid_spit_moving_pre_instruction(k, ctx),
        KAGO_BUG_CRY_COUNTDOWN => kago_bug_cry_countdown(k, ctx),
        KAGO_BUG_FLAG_WHEN_FAR => kago_bug_flag_when_far(k, ctx),
        POWAMP_SPIKES => powamp_spike_pre_instruction(k, ctx),
        WORK_ROBOT_LASER => work_robot_laser_pre_instruction(k, ctx),
        SUB_86_D7_BF => n00b_tube_crack_flickering_pre_instruction(k, ctx),
        SUB_86_D7_DE => n00b_tube_crack_falling_pre_instruction(k, ctx),
        CACATAC_SPIKES_MOVE_Y2 => move_cacatac_spike_down(k, ctx),
        MOVE_X1 => mini_crocomire_projectile_move_left(k, ctx),
        SPORES => spores_pre_instruction(k, ctx),
        AT_E605 => dust_cloud_moving_pre_instruction(k, ctx),
        SPARKS => falling_spark_pre_instruction(k, ctx),
        CACATAC_SPIKES_0_MOVE_X1 => cacatac_spike_left(k, ctx),
        CACATAC_SPIKES_2_MOVE_X2 => cacatac_spike_right(k, ctx),
        CACATAC_SPIKES_1_MOVE_Y1 => cacatac_spike_up(k, ctx),
        COLLECT_DROP => collect_drop(k, ctx),
        NONE_A918 => (),
        NONE_8170 => (),
        NONE_84FB => (),
        NONE_8D54 => (),
        NONE_950C => (),
        AT_96CE => proto_vertical_afterburn_down_pre_instruction(k, ctx),
        NONE_9A44 => (),
        NONE_A05B => (),
        NONE_A327 => (),
        PRE_PHANTOM_ROOM => pre_phantoon_room_pre_instruction(k, ctx),
        NONE_A919 => (),
        NONE_B4B0 => (),
        NONE_BB4F => (),
        NONE_BBC6 => (),
        OUTSIDE_CAMERA_VIEW_CHECK => {
            outside_camera_view(k, ctx);
        }
        NONE_CAA3 => (),
        NONE_CFF7 => (),
        NONE_D0EB => (),
        MOVE_X2 => mini_crocomire_projectile_move_right(k, ctx),
        NONE_DD44 => (),
        NONE_E508 => (),
        NONE_E604 => (),
        NONE_E6D1 => (),
        NONE_EC94 => (),
        NONE_EFDF => (),
        NONE_C76D => (),
        _ => ctx.unreachable(),
    }
}

pub fn skree_particle_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    integrate_shard(v1 as isize, SlotField::X, SlotField::XFraction, ctx);
    enemy_add!(ctx, (v1 as usize) as isize, velocity_y, 80);
    if is_off_screen(k, ctx) != 0 {
        enemy!(ctx, (v1 as usize) as isize, definition = 0);
    }
}

fn move_spike_axis(k: u16, x: bool, velocity: SlotField, ctx: &mut impl Host) {
    let index = (k >> 1) as isize;
    let amount = ((read_member(ctx, velocity, index) as i16 as i32) << 8) as u32;
    let (high, low) = if x {
        (SlotField::X, SlotField::XFraction)
    } else {
        (SlotField::Y, SlotField::YFraction)
    };
    add_split(high, low, index, amount, ctx);
}

pub fn move_cacatac_spike_up(k: u16, ctx: &mut impl Host) {
    move_spike_axis(k, false, SlotField::VelocityY, ctx);
}

pub fn move_cacatac_spike_left(k: u16, ctx: &mut impl Host) {
    move_spike_axis(k, true, SlotField::VelocityY, ctx);
}

pub fn move_cacatac_spike_right(k: u16, ctx: &mut impl Host) {
    move_spike_axis(k, true, SlotField::VelocityX, ctx);
}

pub fn cacatac_spike_up_left(k: u16, ctx: &mut impl Host) {
    move_cacatac_spike_left(k, ctx);
    move_cacatac_spike_up(k, ctx);
}

pub fn cacatac_spike_up_right(k: u16, ctx: &mut impl Host) {
    move_cacatac_spike_right(k, ctx);
    move_cacatac_spike_up(k, ctx);
}

pub fn dust_cloud_explosion_pre_instruction(k: u16, ctx: &mut impl Host) {
    if is_off_screen(k, ctx) != 0 {
        enemy!(ctx, (usize::from(k >> 1)) as isize, definition = 0);
    }
}

pub fn draygon_goop_stuck_to_samus_pre_instruction(k: u16, ctx: &mut impl Host) {
    EprojPowerBombCollision(k, ctx);
    if ctx.session().player.contact.contact_attack != 0
        || {
            let v1: i32 = k as i32 >> 1;
            enemy!(ctx, ctx.session().player.x => (v1) as isize, x);
            enemy!(ctx, (ctx.session().player.y as i32 + 4 * enemy!(ctx, (v1) as isize, behavior_b) as i32
                - 12i32) as u16 => (v1) as isize, y);
            enemy_decrement!(ctx, (v1) as isize, behavior_a);
            enemy!(ctx, (v1) as isize, behavior_a) == 0
        }
    {
        enemy!(ctx, 0 => (k >> 1) as isize, definition);
        if ({
            let old = ctx.session().player.speed_divisor;
            let value = old.wrapping_sub(1);
            ctx.session_mut().player.speed_divisor = value;
            ctx.session().player.speed_divisor
        } & 0x8000)
            != 0
        {
            ctx.session_mut().player.speed_divisor = 0;
        }
    }
}

pub fn draygon_turret_projectile_fired_pre_instruction(k: u16, ctx: &mut impl Host) {
    EprojPowerBombCollision(k, ctx);
    move_along_angle(k, ctx);
    let v1: i16 = outside_512_pixel_square(k, ctx) as i16;
    if v1 != 0 {
        enemy!(ctx, 0 => (k >> 1) as isize, definition);
    }
}

pub fn draygon_goop_pre_instruction(k: u16, ctx: &mut impl Host) {
    EprojPowerBombCollision(k, ctx);
    move_along_angle(k, ctx);
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = (((ctx.session().player.x as i32 - enemy!(ctx, (v1) as isize, x) as i32) as u16)
        as i16)
        .unsigned_abs();
    if (v2 as i32 - 16i32) & 0x8000 != 0 && {
        let v3: u16 = (((ctx.session().player.y as i32 - enemy!(ctx, (v1) as isize, y) as i32)
            as u16) as i16)
            .unsigned_abs();
        (v3 as i32 - 20i32) & 0x8000 != 0
    } {
        enemy!(ctx, 0x8c38 => (v1) as isize, program);
        enemy!(ctx, 1 => (v1) as isize, delay);
    } else if outside_512_pixel_square(k, ctx) != 0 {
        enemy!(ctx, 0 => (v1) as isize, definition);
    }
}

pub fn crocomire_projectile_setup_pre_instruction(k: u16, ctx: &mut impl Host) {
    move_horizontally_with_block_collision(k, ctx);
    enemy!(ctx, 2560 => 0_isize, graphics);
    enemy_update!(ctx, (k >> 1) as isize, counter, |old| ((old as i32)
        + (enemy!(ctx, (k >> 1) as isize, velocity_x) as i32))
        as u16);
    let x: u16 = -64i32 as u16;
    let y: u16 = {
        let record = ctx.loaded_record(0x86, 0x9059);
        record.native_word(
            ((ctx.resolve_actor(Key::from_legacy(0), Le).pre_instruction >> 1) as isize)
                .checked_mul(2)
                .unwrap(),
        )
    };
    let v1: i32 = ctx.calculate_angle_from_xy(x, y) as i32;
    let v2: i32 = k as i32 >> 1;
    enemy!(ctx, (4 * kSinCosTable8bit_Sext[(v1 + 64) as usize] as i32) as u16 => (v2) as isize, velocity_x);
    enemy!(ctx, (4 * kSinCosTable8bit_Sext[v1 as usize] as i32) as u16 => (v2) as isize, velocity_y);
    enemy!(ctx, (0x8690b3 & 0xffff) as u16 => (v2) as isize, pre_action);
}

pub fn crocomire_projectile_fired_pre_instruction(k: u16, ctx: &mut impl Host) {
    if move_horizontally_with_block_collision(k, ctx) & 1 != 0
        || move_vertically_with_block_collision(k, ctx) & 1 != 0
    {
        enemy!(ctx, 0 => (k >> 1) as isize, definition);
    }
}

pub fn crocomire_bridge_crumbling_pre_instruction(k: u16, ctx: &mut impl Host) {
    if move_vertically_with_block_collision(k, ctx) & 1 != 0 {
        enemy!(ctx, 0 => (k >> 1) as isize, definition);
    } else {
        enemy!(ctx, ((enemy!(ctx, (k >> 1) as isize, velocity_y) as i32 + 24) & 0x3fff) as u16 => (k >> 1) as isize, velocity_y);
    };
}

pub fn ridleys_fireball_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: u16 = if move_horizontally_with_block_collision(k, ctx) & 1 != 0 {
        -27042i32 as u16
    } else {
        if move_vertically_with_block_collision(k, ctx) & 1 == 0 {
            return;
        }
        -27056i32 as u16
    };
    let v2: i32 = k as i32 >> 1;
    enemy!(ctx, (v2 as usize) as isize, definition = 0);
    if enemy!(ctx, (v2 as usize) as isize, behavior_b) == 0 {
        set_spawn_point_to_projectile(k, ctx);
        spawn_with_room_gfx(v1, 3, ctx);
        ctx.queue_sound(2, 6, 0x2b);
    }
}

pub fn horizontal_afterburn_pre_instruction(k: u16, ctx: &mut impl Host) {
    move_with_x_velocity(k, ctx);
    if move_vertically_with_block_collision(k, ctx) & 1 != 0 {
        let slot = usize::from(k >> 1);
        enemy!(ctx, (slot) as isize, program = ADDR_OFF_869574 as u16);
        enemy!(ctx, (slot) as isize, delay = 1);
    }
}

pub fn vertical_afterburn_pre_instruction(k: u16, ctx: &mut impl Host) {
    move_with_y_velocity(k, ctx);
    if move_horizontally_with_block_collision(k, ctx) & 1 != 0 {
        let slot = usize::from(k >> 1);
        enemy!(ctx, (slot) as isize, program = ADDR_OFF_869574 as u16);
        enemy!(ctx, (slot) as isize, delay = 1);
    }
}

pub fn proto_horizontal_afterburn_pre_instruction(k: u16, ctx: &mut impl Host) {
    if move_horizontally_with_block_collision(k, ctx) & 1 != 0 {
        enemy!(ctx, 0 => (k >> 1) as isize, definition);
    }
}

pub fn proto_vertical_afterburn_up_pre_instruction(k: u16, ctx: &mut impl Host) {
    if move_vertically_with_block_collision(k, ctx) & 1 != 0 {
        enemy!(ctx, 0 => (k >> 1) as isize, definition);
    }
}

pub fn ceres_falling_debris_pre_instruction(k: u16, ctx: &mut impl Host) {
    let slot = usize::from(k >> 1);
    enemy_add!(ctx, (slot) as isize, velocity_y, 16);
    if move_vertically_with_block_collision(k, ctx) & 1 != 0 {
        enemy!(ctx, (slot) as isize, definition = 0);
        set_spawn_point_to_projectile(k, ctx);
        spawn_with_room_gfx(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 9, ctx);
        ctx.queue_sound(2, 6, 0x6d);
    }
}

pub fn phantoon_enraged_flame_pre_instruction(k: u16, ctx: &mut impl Host) {
    let mut v5: u16 = 0;
    let v1: i32 = k as i32 >> 1;
    enemy_add!(ctx, (v1) as isize, velocity_y, 4);
    let v2: u16 = (enemy_byte!(ctx, v1 as isize * 2, velocity_x) as i32
        + enemy!(ctx, (v1) as isize, behavior_a) as i32) as u8 as u16;
    move_phantoon_orbit(v1 as isize, v2, &mut v5, ctx);
}

pub fn phantoon_flame_spiral_pre_instruction(k: u16, ctx: &mut impl Host) {
    let mut v5: u16 = 0;
    let v1: i32 = k as i32 >> 1;
    enemy_add!(ctx, (v1) as isize, velocity_y, 2);
    let v2: u16 = (enemy!(ctx, (v1) as isize, behavior_a) as i32 + 2) as u8 as u16;
    move_phantoon_orbit(v1 as isize, v2, &mut v5, ctx);
}

pub fn phantoon_starting_flames_pre_instruction(k: u16, ctx: &mut impl Host) {
    if ctx.resolve_actor(Key::from_legacy(0), Le).variables[1] != 0 {
        let v1: i32 = k as i32 >> 1;
        enemy!(ctx, (0x869b41 & 0xffff) as u16 => (v1) as isize, pre_action);
        enemy!(ctx, 180 => (v1) as isize, velocity_x);
        enemy!(ctx, 48 => (v1) as isize, velocity_y);
    }
}

pub fn kraid_rocks_pre_instruction(k: u16, ctx: &mut impl Host) {
    if move_horizontally_with_block_collision(k, ctx) & 1 != 0
        || move_vertically_with_block_collision(k, ctx) & 1 != 0
    {
        enemy!(ctx, (usize::from(k >> 1)) as isize, definition = 0);
    } else {
        let v1: i32 = k as i32 >> 1;
        let mut v2: u16 = enemy_add!(ctx, (v1 as usize) as isize, velocity_x, 8);
        if (v2 as i32 - 256i32) & 0x8000 == 0 {
            v2 = -256i32 as u16;
        }
        enemy!(ctx, v2 => (v1) as isize, velocity_x);
        enemy_add!(ctx, (v1 as usize) as isize, velocity_y, 64);
    };
}

pub use self::crocomire_bridge_crumbling_pre_instruction as kraid_ceiling_rocks_pre_instruction;

pub fn kraid_rocks_use_palette_0_pre_instruction(k: u16, ctx: &mut impl Host) {
    enemy!(ctx, 0 => (k >> 1) as isize, graphics);
}

pub fn mini_kraid_spit_pre_instruction(k: u16, ctx: &mut impl Host) {
    if move_horizontally_with_block_collision(k, ctx) & 1 != 0
        || move_vertically_with_block_collision(k, ctx) & 1 != 0
    {
        enemy!(ctx, (usize::from(k >> 1)) as isize, definition = 0);
    } else {
        let v1: i32 = k as i32 >> 1;
        let mut v2: i16 = ((enemy!(ctx, (v1) as isize, velocity_y)) as i16).wrapping_add(64);
        if v2 >= 0 && (enemy!(ctx, (v1) as isize, velocity_y) as i32 - 960i32) & 0x8000 == 0 {
            v2 = 1024;
        }
        enemy!(ctx, v2 as u16 => (v1) as isize, velocity_y);
    };
}

pub fn mini_kraid_spikes_pre_instruction(k: u16, ctx: &mut impl Host) {
    if move_horizontally_with_block_collision(k, ctx) & 1 != 0 {
        enemy!(ctx, (usize::from(k >> 1)) as isize, definition = 0);
    }
}

pub fn alcoon_fireball_pre_instruction(k: u16, ctx: &mut impl Host) {
    let mut v2: u16;
    if move_vertically_with_block_collision(k, ctx) & 1 != 0
        || move_horizontally_with_block_collision(k, ctx) & 1 != 0
    {
        enemy!(ctx, 0 => (k >> 1) as isize, definition);
    } else {
        let v1: i32 = k as i32 >> 1;
        if (enemy!(ctx, (v1) as isize, velocity_x) & 0x8000) == 0 {
            v2 = (enemy!(ctx, (v1) as isize, velocity_x) as i32 - 64i32) as u16;
            enemy!(ctx, v2 => (v1) as isize, velocity_x);
            if (v2 as i32 - 512i32) & 0x8000 != 0 {
                v2 = 512;
            }
        } else {
            v2 = (enemy!(ctx, (v1) as isize, velocity_x) as i32 + 64) as u16;
            enemy!(ctx, v2 => (v1) as isize, velocity_x);
            if (v2 as i32 + 512) & 0x8000 == 0 {
                v2 = -512i32 as u16;
            }
        }
        enemy!(ctx, v2 => (v1) as isize, velocity_x);
    };
}

pub fn ceres_elevator_pad_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v3: bool;
    let v1: i32 = k as i32 >> 1;
    if enemy!(ctx, (v1) as isize, behavior_a) == 0
        || {
            let v2: bool = enemy!(ctx, (v1) as isize, behavior_a) == 1;
            v3 = ((enemy!(ctx, (v1) as isize, behavior_a)) as i16).wrapping_sub(1) < 0;
            enemy_decrement!(ctx, (v1) as isize, behavior_a);
            v2 as i32 != 0
        }
        || v3
    {
        enemy!(ctx, ctx.session().player.y.wrapping_add(28) => (v1) as isize, y);
        let fresh113 = {
            let value = ctx.session().player.y.wrapping_add(1);
            ctx.session_mut().player.y = value;
            value
        };
        if (fresh113 as i32 - 73i32) & 0x8000 == 0 {
            ctx.session_mut().player.y = 72;
            enemy!(ctx, 1 => (v1) as isize, delay);
            enemy!(ctx, ADDR_OFF_86A28B as u16 => (v1) as isize, program);
            ctx.resume_player_after_elevator();
        }
    }
}

pub fn ceres_elevator_pad_concealer_pre_instruction(k: u16, ctx: &mut impl Host) {
    if ctx.session().player.y == 72 {
        let v1: i32 = k as i32 >> 1;
        enemy!(ctx, 1 => (v1) as isize, delay);
        enemy!(ctx, ADDR_OFF_86A28B as u16 => (v1) as isize, program);
    }
}

pub fn bomb_torizo_statue_breaking_falling_pre_instruction(k: u16, ctx: &mut impl Host) {
    let carry: u8 = move_vertically_with_block_collision(k, ctx);
    let v1: i32 = k as i32 >> 1;
    if (enemy!(ctx, (v1) as isize, velocity_y) & 0x8000) != 0 || carry == 0 {
        let v2: u16 =
            (enemy!(ctx, 0_isize, behavior_b)).wrapping_add(enemy!(ctx, (v1) as isize, velocity_y));
        enemy!(ctx, v2 => (v1) as isize, velocity_y);
        if (v2 & 0xf000) == 4096 {
            enemy!(ctx, 4096 => (v1) as isize, velocity_y);
        }
    } else {
        enemy!(ctx, 0xa918 => (v1) as isize, pre_action);
    };
}

pub fn bomb_torizos_chozo_orbs_pre_instruction(k: u16, ctx: &mut impl Host) {
    if move_horizontally_with_block_collision(k, ctx) != 0 {
        let v3: i32 = k as i32 >> 1;
        start_instruction((v3 as usize) as isize, ADDR_OFF_86AB25 as u16, ctx);
    } else {
        let carry: u8 = move_vertically_with_block_collision(k, ctx);
        let v1: i32 = k as i32 >> 1;
        if (enemy!(ctx, (v1) as isize, velocity_y) & 0x8000) != 0 || carry == 0 {
            let v2: u16 = enemy_add!(ctx, (v1 as usize) as isize, velocity_y, 18);
            if (v2 & 0xf000) == 4096 {
                enemy!(ctx, 0 => (v1) as isize, definition);
            }
        } else {
            let v4: i32 = k as i32 >> 1;
            enemy!(ctx, (enemy!(ctx, (k >> 1) as isize, y) & 0xfff0 | 8).wrapping_sub(2) => (v4) as isize, y);
            start_instruction((v4 as usize) as isize, ADDR_OFF_86AB41 as u16, ctx);
        }
    };
}

pub fn golden_torizos_chozo_orbs_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32;
    if move_horizontally_with_block_collision(k, ctx) & 1 != 0 {
        enemy!(ctx, -(enemy!(ctx, (k >> 1) as isize, velocity_x) as i32) as u16 => (k >> 1) as isize, velocity_x);
    }
    if move_vertically_with_block_collision(k, ctx) & 1 != 0
        && {
            v1 = k as i32 >> 1;
            (enemy!(ctx, (v1) as isize, velocity_y) & 0x8000) == 0
        }
        && {
            let v2: i16 = enemy!(ctx, (v1) as isize, velocity_x) as i16;
            let v3 = (v2 as i32 + if v2 >= 0 { -64 } else { 64 }) as u16;
            enemy!(ctx, v3 => (v1) as isize, velocity_x);
            let v4: u16 = -(enemy!(ctx, (v1) as isize, velocity_y) as i32 >> 1i32) as u16;
            enemy!(ctx, v4 => (v1) as isize, velocity_y);
            (v4 & 0xff80) == 0xff80
        }
    {
        let v5: i32 = k as i32 >> 1;
        enemy!(ctx, ((enemy!(ctx, (v5) as isize, y) as i32 & 0xfff0 | 8) - 2i32) as u16 => (v5) as isize, y);
        enemy!(ctx, ADDR_OFF_86AB41 as u16 => (v5) as isize, program);
        enemy!(ctx, 1 => (v5) as isize, delay);
    } else {
        enemy_add!(ctx, (k >> 1) as isize, velocity_y, 24);
    };
}

pub fn torizo_sonic_boom_pre_instruction(k: u16, ctx: &mut impl Host) {
    if move_horizontally_with_block_collision(k, ctx) & 1 != 0 {
        let v4: i32 = k as i32 >> 1;
        enemy!(ctx, ADDR_OFF_86ADE5 as u16 => (v4) as isize, program);
        enemy!(ctx, 1 => (v4) as isize, delay);
        enemy!(ctx, enemy!(ctx, (v4) as isize, x) => (v4) as isize, behavior_a);
        enemy!(ctx, enemy!(ctx, (v4) as isize, y) => (v4) as isize, behavior_b);
    } else {
        let v1: i32 = k as i32 >> 1;
        enemy_update!(ctx, (v1) as isize, velocity_x, |old| ((old as i32)
            + (if enemy!(ctx, (v1) as isize, velocity_x) & 0x8000 != 0 {
                -16i32
            } else {
                16
            })) as u16);
        if (enemy!(ctx, (v1) as isize, velocity_x) & 0xf000) == 4096 {
            enemy!(ctx, 0 => (v1) as isize, definition);
        }
    };
}

pub fn golden_torizo_egg_hatched_pre_instruction(k: u16, ctx: &mut impl Host) {
    if move_horizontally_with_block_collision(k, ctx) & 1 != 0 {
        let v3: i32 = k as i32 >> 1;
        enemy!(ctx, 0xb0dd => (v3) as isize, pre_action);
        enemy!(ctx, 0 => (v3) as isize, velocity_y);
    } else {
        let v1: i32 = k as i32 >> 1;
        let v2: i16 = if (enemy!(ctx, (v1) as isize, behavior_a) & 0x8000) != 0 {
            48
        } else {
            -48
        };
        enemy_add!(ctx, (v1) as isize, velocity_x, v2 as u16);
    };
}

pub fn golden_torizo_egg_hit_wall_pre_instruction(k: u16, ctx: &mut impl Host) {
    if move_vertically_with_block_collision(k, ctx) & 1 != 0 {
        let v1: i32 = k as i32 >> 1;
        enemy!(ctx, (if (enemy!(ctx, (v1) as isize, behavior_a) & 0x8000) != 0 {
                ADDR_OFF_86B1A8 as i32
            } else {
                ADDR_OFF_86B190 as i32
            }) as u16 => (v1) as isize, program);
        enemy!(ctx, 1 => (v1) as isize, delay);
    } else {
        enemy_add!(ctx, (k >> 1) as isize, velocity_y, 48);
    };
}

pub fn golden_torizo_super_missile_thrown_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32;
    if move_horizontally_with_block_collision(k, ctx) != 0 || {
        let carry: u8 = move_vertically_with_block_collision(k, ctx);
        v1 = k as i32 >> 1;
        (enemy!(ctx, (v1) as isize, velocity_y) & 0x8000) == 0 && carry != 0
    } {
        let v3: i32 = k as i32 >> 1;
        enemy!(ctx, ADDR_OFF_86B2EF as u16 => (v3) as isize, program);
        enemy!(ctx, 1 => (v3) as isize, delay);
    } else {
        let v2: u16 = (enemy!(ctx, (v1) as isize, velocity_y) as i32 + 16) as u16;
        enemy!(ctx, v2 => (v1) as isize, velocity_y);
        if (v2 & 0xf000) == 4096 {
            enemy!(ctx, 0 => (v1) as isize, definition);
        }
    };
}

pub fn golden_torizo_eye_beam_pre_instruction(k: u16, ctx: &mut impl Host) {
    if move_horizontally_with_block_collision(k, ctx) & 1 != 0 {
        enemy!(ctx, ADDR_OFF_86B3CD as u16 => (k >> 1) as isize, program);
        enemy!(ctx, 1 => (k >> 1) as isize, delay);
        return;
    }
    if move_vertically_with_block_collision(k, ctx) & 1 != 0 {
        let v1: i32 = k as i32 >> 1;
        enemy!(ctx, ((enemy!(ctx, (v1) as isize, y) as i32 & 0xfff0 | 8) - 2i32) as u16 => (v1) as isize, y);
        enemy!(ctx, ADDR_OFF_86B3E5 as u16 => (v1) as isize, program);
        enemy!(ctx, 1 => (v1) as isize, delay);
    }
}

pub fn eye_door_projectile_moving_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v3: i32 = k as i32 >> 1;
    if !(move_horizontally_with_block_collision(k, ctx) & 1 != 0
        || move_vertically_with_block_collision(k, ctx) & 1 != 0)
    {
        let v1: i32 = enemy!(ctx, (k >> 1) as isize, behavior_a) as i32 >> 1;
        enemy_update!(ctx, (v3) as isize, velocity_x, |old| old
            .wrapping_add((kSinCosTable8bit_Sext[(v1 + 64) as usize] >> 4) as u16));
        enemy_update!(ctx, (v3) as isize, velocity_y, |old| ((old as i32)
            + (kSinCosTable8bit_Sext[v1 as usize] as i32 >> 4))
            as u16);
        let door = enemy!(ctx, (v3) as isize, behavior_b);
        if !crate::progress_flags::door_opened(ctx, door) {
            return;
        }
    }
    start_instruction((v3 as usize) as isize, ADDR_OFF_86B5F3 as u16, ctx);
}

pub fn eye_door_sweat_pre_instruction(k: u16, ctx: &mut impl Host) {
    move_horizontally_with_block_collision(k, ctx);
    let carry: u8 = move_vertically_with_block_collision(k, ctx);
    let v1: i32 = k as i32 >> 1;
    if (enemy!(ctx, (v1) as isize, velocity_y) & 0x8000) != 0 || carry == 0 {
        enemy_add!(ctx, (v1 as usize) as isize, velocity_y, 12);
    } else {
        enemy_add!(ctx, (v1 as usize) as isize, y, (-4i16) as u16);
        start_instruction((v1 as usize) as isize, ADDR_OFF_86B61D as u16, ctx);
    };
}

pub fn tourian_statue_particle_splash_pre_instruction(k: u16, ctx: &mut impl Host) {
    enemy!(ctx, (ctx.inputs().fx_y as i32 - 4i32) as u16 => (k >> 1) as isize, y);
}

pub fn tourian_statue_base_decoration_pre_instruction(k: u16, ctx: &mut impl Host) {
    if ctx.session().mechanisms.statue_animation == 0 {
        ctx.session_mut().mechanisms.statue_finished |= 0x8000;
    }
    tourian_statue_decoration_pre_instruction(k, ctx);
}

/// Despawns the projectile once it leaves the camera view.
/// Was `EprojPreInstr_NorfairLavaquakeRocks_Inner` (also `EprojPreInstr_DBF2_Func1`).
pub fn despawn_off_screen(k: u16, ctx: &mut impl Host) {
    if outside_camera_view(k, ctx) != 0 {
        enemy!(ctx, 0 => (k >> 1) as isize, definition);
    }
}

pub fn polyp_rock_pre_instruction(k: u16, ctx: &mut impl Host) {
    dispatch_polyp_rock_function(
        (enemy!(ctx, (k >> 1) as isize, behavior_a) as i32 | 0x860000) as u32,
        k,
        ctx,
    );
    despawn_off_screen(k, ctx);
}

pub fn mother_brain_turrets_pre_instruction(k: u16, ctx: &mut impl Host) {
    if is_turret_on_screen(k, ctx) & 1 != 0 {
        if ctx.mother_brain_turret_destroyed() != 0 {
            clear_unrounded_definition(k, ctx);
        }
    } else if ctx.mother_brain_turret_destroyed() != 0 {
        clear_unrounded_definition(k, ctx);
        let v5: i32 = k as i32 >> 1;
        ctx.session_mut().enemy_projectiles.spawn.point = Point {
            x: enemy!(ctx, (v5) as isize, x),
            y: enemy!(ctx, (v5) as isize, y),
        };
        spawn_with_room_gfx(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 0xc, ctx);
    } else {
        let v1: i32 = k as i32 >> 1;
        let fresh89 = enemy_decrement!(ctx, v1 as isize, velocity_x);
        let mut v2: bool = fresh89 == 1;
        if v2 {
            set_random_rotation_timer(k, ctx);
            turret_bullet_step_direction(k, ctx);
            let v3: i32 = k as i32 >> 1;
            enemy!(ctx, turret_instruction(enemy_byte!(ctx, v3 as isize * 2, y_fraction) as isize, ctx) => (v3) as isize, program);
            enemy!(ctx, 1 => (v3) as isize, delay);
        }
        let v4: i32 = k as i32 >> 1;
        let fresh93 = enemy_decrement!(ctx, v4 as isize, velocity_y);
        v2 = fresh93 == 1;
        if v2 {
            set_random_cooldown_timer(k, ctx);
            spawn_with_room_gfx(ADDR_K_EPROJ_MOTHER_BRAIN_ROOM_TURRET_BULLETS as u16, k, ctx);
        }
    };
}

// Keep signed-word interpretation and narrowing before the zero clamp.
fn decelerated_magnitude(velocity: u16, decrement: u16) -> i16 {
    (((velocity as i16).unsigned_abs() as i32 - i32::from(decrement)) as i16).max(0)
}

pub fn mother_brain_bomb_pre_instruction(k: u16, ctx: &mut impl Host) {
    if mother_brain_bomb_collides_with_bomb(k, ctx) != 0 {
        return;
    }
    let v1: i32 = k as i32 >> 1;

    let advance_bomb_phase: bool = if enemy!(ctx, (v1) as isize, behavior_b) != 0 {
        let v4: i32 = k as i32 >> 1;
        let v5: u16 = K_MOTHER_BRAINS_BOMB_YACCEL
            [(enemy!(ctx, (v4) as isize, behavior_b) >> 1) as usize] as u16;
        if v5 == 0 {
            enemy!(ctx, 0 => (v4) as isize, velocity_x);
            enemy!(ctx, 0 => (v4) as isize, velocity_y);
            {
                let value = ctx.mother_brain_active_bombs().wrapping_sub(1);
                ctx.set_mother_brain_active_bombs(value);
            }
            enemy!(ctx, 0 => (v4) as isize, definition);
            ctx.session_mut().enemy_projectiles.spawn.point = Point {
                x: enemy!(ctx, (v4) as isize, x),
                y: enemy!(ctx, (v4) as isize, y),
            };
            spawn_with_room_gfx(
                ADDR_STRU_869650 as u16,
                enemy_byte!(ctx, v4 as isize * 2, x_fraction) as u16,
                ctx,
            );
            ctx.session_mut().enemy_projectiles.spawn.point = Point {
                x: enemy!(ctx, (v4) as isize, x),
                y: enemy!(ctx, (v4) as isize, y),
            };
            spawn_with_room_gfx(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 3, ctx);
            ctx.queue_sound(3, 6, 0x13);
            return;
        }
        move_mother_brain_bomb(k, v5, ctx) & 1 != 0
    } else {
        let v2 = decelerated_magnitude(enemy!(ctx, (v1) as isize, velocity_x), 2);
        enemy!(ctx, (if enemy!(ctx, (v1) as isize, velocity_x) & 0x8000 != 0 {
                -(v2 as i32)
            } else {
                v2 as i32
            }) as u16 => (v1) as isize, velocity_x);
        move_mother_brain_bomb(k, 7, ctx) & 1 != 0
    };
    if advance_bomb_phase {
        let v3: i32 = k as i32 >> 1;
        enemy_add!(ctx, (v3) as isize, behavior_b, 2);
    };
}

pub fn mother_brain_rainbow_beam_charging_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v2: i32 = k as i32 >> 1;
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(64), Le).x.whole => (v2) as isize, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(64), Le).y.whole => (v2) as isize, y);
}

pub fn mother_brain_death_explosion_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v2: i32 = k as i32 >> 1;
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(0), Le).x.whole as i32 + enemy!(ctx, (v2) as isize, velocity_x) as i32) as u16 => (v2) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32 + enemy!(ctx, (v2) as isize, velocity_y) as i32) as u16 => (v2) as isize, y);
}

pub fn mother_brain_tube_falling_pre_instruction(k: u16, ctx: &mut impl Host) {
    dispatch_mother_brain_tube_falling_function(
        (enemy!(ctx, (k >> 1) as isize, behavior_a) as i32 | 0x860000) as u32,
        k,
        ctx,
    );
}

pub fn mother_brain_glass_shard_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1 = (k >> 1) as i32;
    integrate_shard(v1 as isize, SlotField::X, SlotField::XFraction, ctx);
    let _v6: u16 = enemy!(ctx, (v1) as isize, y_fraction);
    if (enemy!(ctx, (v1) as isize, y) & 0xff00) != 0 {
        enemy!(ctx, 0 => (v1) as isize, definition);
    } else {
        enemy_add!(ctx, (v1) as isize, velocity_y, 32);
        if (ctx.next_random() & 0x420) == 0 {
            spawn_with_room_gfx(
                ADDR_K_EPROJ_MOTHER_BRAIN_GLASS_SHATTERING_SPARKLE as u16,
                k,
                ctx,
            );
        }
    };
}

pub fn kago_bug_idling_pre_instruction(k: u16, ctx: &mut impl Host) {
    kago_bug_cry_countdown(k, ctx);
    kago_bug_flag_when_far(k, ctx);
    let v1: i32 = k as i32 >> 1;
    if enemy!(ctx, (v1) as isize, behavior_c) != 0 {
        let _fresh71 = enemy_add!(ctx, (v1 as usize) as isize, behavior_c, u16::MAX);
    } else {
        start_instruction((v1 as usize) as isize, ADDR_WORD_86D052 as u16, ctx);
        enemy!(ctx, (0x86d0ebi32) as u16 => (v1) as isize, pre_action);
    };
}

pub fn kago_bug_jumping_pre_instruction(k: u16, ctx: &mut impl Host) {
    kago_bug_cry_countdown(k, ctx);
    kago_bug_flag_when_far(k, ctx);
    let reset_vertical_speed: bool = if move_horizontally_with_block_collision(k, ctx) != 0 {
        enemy!(ctx, 0 => (k >> 1) as isize, velocity_x);
        true
    } else if move_vertically_with_block_collision(k, ctx) != 0 {
        true
    } else {
        let v2: u16 = enemy!(ctx, (k >> 1) as isize, velocity_y);
        let v3: bool = ((WORD_86_D082) as i16).wrapping_add((v2) as i16) < 0;
        enemy!(ctx, (WORD_86_D082).wrapping_add(v2) => (k >> 1) as isize, velocity_y);
        if !v3 {
            false
        } else {
            return;
        }
    };
    if reset_vertical_speed {
        enemy!(ctx, 256 => (k >> 1) as isize, velocity_y);
    }
    enemy!(ctx, (0x86d128i32) as u16 => (k >> 1) as isize, pre_action);
    start_instruction((usize::from(k >> 1)) as isize, ADDR_WORD_86D04A as u16, ctx);
}

pub fn n00b_tube_shard_flying_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1 = (k >> 1) as i32;
    integrate_shard(v1 as isize, SlotField::BehaviorB, SlotField::BehaviorA, ctx);
    delete_if_vertically_off_screen(k, ctx);
}

pub fn n00b_tube_shard_falling_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v2: i32 = (enemy!(ctx, (k >> 1) as isize, velocity_x) as i32 & 0x17e | 0x80) >> 1;
    let v6: i32 = ctx.session().enemy_projectiles.selected as i32 >> 1;
    let mut amt: i32 = ((kSinCosTable8bit_Sext[(v2 + 64) as usize] >> 2) as i32) << 8;
    add_split(
        SlotField::BehaviorB,
        SlotField::BehaviorA,
        v6 as isize,
        amt as u32,
        ctx,
    );
    enemy_add!(ctx, (v6) as isize, velocity_x, 2);
    amt = (enemy!(ctx, (v6) as isize, velocity_y) as i16 as i32) << 8;
    add_split(
        SlotField::Y,
        SlotField::YFraction,
        v6 as isize,
        amt as u32,
        ctx,
    );
    delete_if_vertically_off_screen(ctx.session().enemy_projectiles.selected, ctx);
}

pub fn n00b_tube_air_bubbles_falling_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: u16 = (enemy!(ctx, (k >> 1) as isize, velocity_x) & 0x17e | 0x80);
    let v2: i32 = v1 as i32 >> 1;
    let amt: i32 = ((kSinCosTable8bit_Sext[(v2 + 64) as usize] >> 2) as i32) << 8;
    let v5: i32 = ctx.session().enemy_projectiles.selected as i32 >> 1;
    add_split(
        SlotField::BehaviorB,
        SlotField::BehaviorA,
        v5 as isize,
        amt as u32,
        ctx,
    );
    enemy_add!(ctx, (v5) as isize, velocity_x, 4);
    n00b_tube_air_bubbles_flying_pre_instruction(ctx.session().enemy_projectiles.selected, ctx);
}

pub fn n00b_tube_air_bubbles_flying_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v5: i32 = k as i32 >> 1;
    move_spike_axis(k, false, SlotField::VelocityY, ctx);
    enemy!(ctx, enemy!(ctx, (v5) as isize, behavior_b) => (v5) as isize, x);
}

pub fn cacatac_spike_down(k: u16, ctx: &mut impl Host) {
    move_cacatac_spike_down(k, ctx);
}

pub fn cacatac_spike_down_left(k: u16, ctx: &mut impl Host) {
    move_cacatac_spike_left(k, ctx);
    move_cacatac_spike_down(k, ctx);
}

pub fn cacatac_spike_down_right(k: u16, ctx: &mut impl Host) {
    move_cacatac_spike_right(k, ctx);
    move_cacatac_spike_down(k, ctx);
}

pub fn cacatac_spike_pre_instruction(k: u16, ctx: &mut impl Host) {
    plant_pre(
        (enemy!(ctx, (k >> 1) as isize, behavior_a) >> 1) as usize,
        k,
        ctx,
    );
    despawn_off_screen(k, ctx);
}

pub fn mini_crocomire_projectile_pre_instruction(k: u16, ctx: &mut impl Host) {
    dispatch(
        (enemy!(ctx, (k >> 1) as isize, behavior_a) as i32 | 0x860000) as u32,
        k,
        ctx,
    );
    despawn_off_screen(k, ctx);
}

pub fn spore_spawner_pre_instruction(k: u16, ctx: &mut impl Host) {
    if ctx.boss_death_quake_sound_timer() == 0 {
        let v1: i32 = k as i32 >> 1;
        if enemy!(ctx, (v1) as isize, behavior_b) == 0 {
            enemy!(ctx, ADDR_WORD_86DC06 as u16 => (v1) as isize, program);
            enemy!(ctx, 1 => (v1) as isize, delay);
            enemy!(ctx, ctx.next_random() & 0x1ff => (v1) as isize, behavior_b);
        }
        enemy_decrement!(ctx, (v1) as isize, behavior_b);
    }
}

pub fn botwoons_body_pre_instruction(k: u16, ctx: &mut impl Host) {
    if ctx.botwoon_death_phase() != 0 {
        let v1: i32 = k as i32 >> 1;
        if enemy!(ctx, (v1) as isize, velocity_x) as i32 == 0x86ea98 & 0xffff {
            enemy!(ctx, (0x86eaf4 & 0xffff) as u16 => (v1) as isize, velocity_x);
        }
    }
    dispatch_botwoon_body_function(
        (enemy!(ctx, (k >> 1) as isize, velocity_x) as i32 | 0x860000) as u32,
        k,
        ctx,
    );
}

pub fn botwoons_spit_pre_instruction(k: u16, ctx: &mut impl Host) {
    move_along_angle(k, ctx);
    delete_botwoon_spit_if_off_screen(k, ctx);
}

pub fn bomb_torizo_drool_falling_pre_instruction(v0: u16, ctx: &mut impl Host) {
    if move_horizontally_with_block_collision(v0, ctx) & 1 != 0 {
        let v6: i32 = v0 as i32 >> 1;
        enemy!(ctx, 0xa48a => (v6) as isize, program);
        enemy!(ctx, 1 => (v6) as isize, delay);
    } else {
        let v1: i32 = v0 as i32 >> 1;
        let v2: i16 = enemy!(ctx, (v1) as isize, velocity_x) as i16;
        if v2 >= 0 {
            let mut v4: i16 = (v2).wrapping_sub(4);
            if v4 < 0 {
                v4 = 3;
            }
            enemy!(ctx, v4 as u16 => (v1) as isize, velocity_x);
        } else {
            let mut v3: i16 = (v2 as i32 + 4) as i16;
            if v3 >= 0 {
                v3 = 3;
            }
            enemy!(ctx, v3 as u16 => (v1) as isize, velocity_x);
        }
        let carry: u8 = move_vertically_with_block_collision(v0, ctx);
        if (enemy!(ctx, (v1) as isize, velocity_y) & 0x8000) != 0 || carry == 0 {
            let v5: u16 = enemy_add!(ctx, (v1 as usize) as isize, velocity_y, 16);
            if (v5 & 0xf000) == 4096 {
                enemy!(ctx, 0 => (v1) as isize, definition);
            }
        } else {
            let v7: i32 = v0 as i32 >> 1;
            enemy_add!(ctx, (v7 as usize) as isize, y, (-3i16) as u16);
            start_instruction((v7 as usize) as isize, 0xa48e, ctx);
        }
    };
}

pub fn kago_bug_falling_pre_instruction(v0: u16, ctx: &mut impl Host) {
    kago_bug_cry_countdown(v0, ctx);
    kago_bug_flag_when_far(v0, ctx);
    if move_horizontally_with_block_collision(v0, ctx) & 1 != 0 {
        enemy!(ctx, 0 => (v0 >> 1) as isize, velocity_x);
    } else if move_vertically_with_block_collision(v0, ctx) & 1 != 0 {
        let v1: i32 = v0 as i32 >> 1;
        enemy!(ctx, (0x86d0ebi32) as u16 => (v1) as isize, pre_action);
        start_instruction((v1 as usize) as isize, ADDR_WORD_86D03C as u16, ctx);
    } else {
        enemy_add!(ctx, (v0 >> 1) as isize, velocity_y, WORD_86_D082);
    };
}

pub fn namihe_fune_fireball_pre_instruction(v0: u16, ctx: &mut impl Host) {
    dispatch_namihe_fune_fireball_function(
        (enemy!(ctx, (v0 >> 1) as isize, behavior_a) as i32 | 0x860000) as u32,
        v0,
        ctx,
    );
    delete_namihe_fune_fireball_if_off_screen(v0, ctx);
}

pub fn lava_thrown_by_magdollite_pre_instruction(v0: u16, ctx: &mut impl Host) {
    dispatch_magdollite_lava_function(
        (enemy!(ctx, (v0 >> 1) as isize, behavior_a) as i32 | 0x860000) as u32,
        v0,
        ctx,
    );
    delete_magdollite_lava_if_off_screen(v0, ctx);
}

pub fn tourian_statue_decoration_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy!(ctx, enemy!(ctx, (v1) as isize, behavior_a) => (v1) as isize, x);
    enemy!(ctx, (enemy!(ctx, (v1) as isize, behavior_b) as i32 + ctx.inputs().camera_y as i32
            - ctx.expanding_square_origin() as i32) as u16 => (v1) as isize, y);
}

pub fn mother_brain_drool_falling_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy_add!(ctx, (v1) as isize, velocity_y, 12);
    if move_with_y_velocity(k, ctx) >= 0xd7 {
        enemy_add!(ctx, (v1) as isize, y, (-4i16) as u16);
        enemy!(ctx, ADDR_OFF_86C8E1 as u16 => (v1) as isize, program);
        enemy!(ctx, 1 => (v1) as isize, delay);
    }
}

pub fn shaktool_attack_front_circle_pre_instruction(v1: u16, ctx: &mut impl Host) {
    if move_horizontally_with_block_collision(v1, ctx) & 1 != 0
        || move_vertically_with_block_collision(v1, ctx) & 1 != 0
    {
        enemy!(ctx, 0 => (v1 >> 1) as isize, definition);
    }
}

pub fn projectile_8aaf_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    move_spike_axis(k, false, SlotField::VelocityY, ctx);
    if enemy!(ctx, (v1) as isize, y) >= enemy!(ctx, (v1) as isize, counter) {
        enemy!(ctx, 0 => (v1) as isize, definition);
    }
}

pub fn crocomire_spike_wall_pieces_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let mut v2: u16 = enemy!(ctx, (v1) as isize, behavior_a);
    if v2 != CROCOMIRE_SPIKE_WALL_PIECES_TAB1[v1 as usize] {
        v2 = (v2 as i32 + CROCOMIRE_SPIKE_WALL_PIECES_TAB2[v1 as usize] as i32) as u16;
        if v2 >= CROCOMIRE_SPIKE_WALL_PIECES_TAB1[v1 as usize] {
            v2 = CROCOMIRE_SPIKE_WALL_PIECES_TAB1[v1 as usize];
        }
    }
    enemy!(ctx, v2 => (v1) as isize, behavior_a);
    let v3: i8 = enemy_byte!(ctx, k as isize + 1, behavior_a) as i8;
    let mut v4: bool = u8::overflowing_add(enemy_byte!(ctx, k as isize, velocity_x), v3 as u8).1;
    {
        let index = k as isize;
        let old = enemy_byte!(ctx, index, velocity_x);
        let value = (old as i32 + v3 as i32) as u8;
        enemy_byte!(ctx, index, velocity_x = value);
    }
    let mut v5: u8 = (v4 as i32 + enemy_byte!(ctx, k as isize + 1, velocity_x) as i32) as u8;
    if ((v5 as i32 - CROCOMIRE_SPIKE_WALL_PIECES_TAB3[k as usize] as i32) as i8) >= 0 {
        v5 = CROCOMIRE_SPIKE_WALL_PIECES_TAB3[k as usize] as u8;
    }
    enemy_byte!(ctx, k as isize + 1, velocity_x = v5);
    let v6: i8 = enemy_byte!(ctx, k as isize, velocity_x) as i8;
    v4 = u8::overflowing_add(enemy_byte!(ctx, k as isize + 1, x_fraction), v6 as u8).1;
    {
        let index = k as isize + 1;
        let old = enemy_byte!(ctx, index, x_fraction);
        let value = (old as i32 + v6 as i32) as u8;
        enemy_byte!(ctx, index, x_fraction = value);
    }
    let v7: i8 = enemy_byte!(ctx, k as isize + 1, velocity_x) as i8;
    let v9: bool = v4;
    v4 = u8::overflowing_add(v4 as u8, v7 as u8).1;
    let v8: i8 = (v9 as i32 + v7 as i32) as i8;
    v4 = (v4 as i32 | u8::overflowing_add(enemy_byte!(ctx, k as isize, x), v8 as u8).1 as i32) != 0;
    {
        let index = k as isize;
        let old = enemy_byte!(ctx, index, x);
        let value = (old as i32 + v8 as i32) as u8;
        enemy_byte!(ctx, index, x = value);
    }
    {
        let index = k as isize + 1;
        let old = enemy_byte!(ctx, index, x);
        let value = (old as i32 + v4 as i32) as u8;
        enemy_byte!(ctx, index, x = value);
    }
    add_split(
        SlotField::VelocityY,
        SlotField::BehaviorB,
        v1 as isize,
        0x3000,
        ctx,
    );
    add_split(
        SlotField::Y,
        SlotField::YFraction,
        v1 as isize,
        (((enemy!(ctx, (v1) as isize, velocity_y)) as u32) << 16)
            | ((enemy!(ctx, (v1) as isize, behavior_b)) as u32),
        ctx,
    );
    if enemy!(ctx, (v1) as isize, y) >= 0xa8 {
        enemy!(ctx, 0 => (v1) as isize, definition);
        if (k & 2) == 0 {
            ctx.queue_sound(2, 6, 0x29);
        }
        let v12: i32 = k as i32 >> 1;
        ctx.session_mut().enemy_projectiles.spawn.point = Point {
            x: enemy!(ctx, (v12) as isize, x),
            y: enemy!(ctx, (v12) as isize, y),
        };
        spawn_with_room_gfx(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 0x15, ctx);
        ctx.queue_sound(2, 6, 0x25);
    }
}

pub fn unused_9634_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    if enemy!(ctx, (v1) as isize, behavior_a) >= 8 {
        move_with_velocity(k, ctx);
        if move_vertically_with_block_collision(k, ctx) & 1 != 0 {
            enemy!(ctx, ADDR_OFF_869574 as u16 => (v1) as isize, program);
            enemy_add!(ctx, (v1) as isize, behavior_a, 1);
            enemy!(ctx, 1 => (v1) as isize, delay);
            enemy!(ctx, 0 => (v1) as isize, velocity_x);
            enemy!(ctx, 0 => (v1) as isize, velocity_y);
            enemy!(ctx, 2560 => (v1) as isize, graphics);
            ctx.queue_sound(2, 6, 0x2b);
        }
    } else {
        enemy_add!(ctx, (v1) as isize, behavior_a, 1);
    };
}

pub fn phantoon_casual_flame_falling_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy_add!(ctx, (v1) as isize, velocity_y, 16);
    if move_vertically_with_block_collision(k, ctx) & 1 != 0 {
        enemy!(ctx, (enemy!(ctx, (v1) as isize, properties) & 0xfff | 0x8000) => (v1) as isize, properties);
        enemy!(ctx, (0x8699bf & 0xffff) as u16 => (v1) as isize, pre_action);
        enemy!(ctx, ADDR_WORD_86976C as u16 => (v1) as isize, program);
        enemy!(ctx, 1 => (v1) as isize, delay);
        enemy!(ctx, 8 => (v1) as isize, behavior_a);
        enemy_add!(ctx, (v1) as isize, y, 8);
    }
}

pub fn phantoon_casual_flame_hit_ground_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2: bool = enemy!(ctx, (v1) as isize, behavior_a) == 1;
    let v3: bool = (enemy_add!(ctx, (v1) as isize, behavior_a, (-1i16) as u16) & 0x8000) != 0;
    if v2 || v3 {
        enemy!(ctx, (0x869a01 & 0xffff) as u16 => (v1) as isize, pre_action);
        enemy!(ctx, ADDR_WORD_869772 as u16 => (v1) as isize, program);
        enemy!(ctx, 1 => (v1) as isize, delay);
        enemy_add!(ctx, (v1) as isize, y, (-8i16) as u16);
        enemy!(ctx, -768i32 as u16 => (v1) as isize, velocity_y);
        enemy!(ctx, 0 => (v1) as isize, behavior_a);
        if (ctx.inputs().frame & 1) != 0 {
            enemy!(ctx, -128i32 as u16 => (v1) as isize, velocity_x);
        } else {
            enemy!(ctx, 128 => (v1) as isize, velocity_x);
        }
    }
}

pub fn phantoon_casual_flame_bouncing_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy_add!(ctx, (v1) as isize, velocity_y, 16);
    let switch_fireball_phase: bool = if move_vertically_with_block_collision(k, ctx) & 1 != 0 {
        let v2: u16 = (enemy!(ctx, (v1) as isize, behavior_a) as i32 + 1) as u16;
        enemy!(ctx, v2 => (v1) as isize, behavior_a);
        if (v2 as i32 - 3i32) & 0x8000 != 0 {
            enemy!(ctx, WORD_869_A3_E[v2 as usize] => (v1) as isize, velocity_y);
            return;
        }
        true
    } else {
        move_horizontally_with_block_collision(k, ctx) & 1 != 0
    };
    if switch_fireball_phase {
        enemy!(ctx, ADDR_WORD_869782 as u16 => (v1) as isize, program);
        enemy!(ctx, 1 => (v1) as isize, delay);
        enemy!(ctx, (0x869a44 & 0xffff) as u16 => (v1) as isize, pre_action);
    };
}

pub fn phantoon_flame_rain_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let apply_gravity: bool = (enemy!(ctx, (v1) as isize, velocity_x) == 0)
        || ({
            let v2: bool = enemy!(ctx, (v1) as isize, velocity_x) == 1;
            let v3: bool =
                (enemy_add!(ctx, (v1) as isize, velocity_x, (-1i16) as u16) & 0x8000) != 0;
            (v2 || v3)
                && ({
                    ctx.queue_sound(3, 6, 0x1d);
                    true
                })
        });
    if apply_gravity {
        enemy_add!(ctx, (v1) as isize, velocity_y, 16);
        if move_vertically_with_block_collision(k, ctx) & 1 != 0 {
            enemy!(ctx, ADDR_WORD_8697AC as u16 => (v1) as isize, program);
            enemy!(ctx, 1 => (v1) as isize, delay);
            enemy_add!(ctx, (v1) as isize, y, 8);
            enemy!(ctx, (0x869a44 & 0xffff) as u16 => (v1) as isize, pre_action);
            ctx.queue_sound(3, 6, 0x1d);
        }
    };
}

pub fn phantoon_starting_flames_activated_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = enemy!(ctx, (v1) as isize, velocity_x);
    if v2 != 0 {
        enemy!(ctx, (v2 as i32 - 1i32) as u16 => (v1) as isize, velocity_x);
    } else if (ctx.inputs().frame & 1) != 0 {
        let v3: u16 = (enemy!(ctx, (v1) as isize, velocity_y) as i32 - 1i32) as u16;
        enemy!(ctx, v3 => (v1) as isize, velocity_y);
        if v3 == 0 {
            enemy!(ctx, ctx.resolve_actor(Key::from_legacy(0), Le).x.whole => (v1) as isize, x);
            enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32 + 16) as u16 => (v1) as isize, y);
            enemy!(ctx, 1 => (v1) as isize, delay);
            enemy!(ctx, ADDR_OFF_8697F8 as u16 => (v1) as isize, program);
            return;
        }
    }
    let v4: u8 = (enemy!(ctx, (v1) as isize, behavior_a) as i32 + 1) as u8;
    enemy!(ctx, v4 as u16 => (v1) as isize, behavior_a);
    let pt = sine_offset::<false>(v4 as u16, enemy!(ctx, (v1) as isize, velocity_y));
    enemy!(ctx, (pt.x as i32 + ctx.resolve_actor(Key::from_legacy(0), Le).x.whole as i32) as u16 => (v1) as isize, x);
    enemy!(ctx, (pt.y as i32 + ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32 + 16) as u16 => (v1) as isize, y);
}

pub fn pirate_laser_move_left_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy_add!(ctx, (v1 as usize) as isize, x, (-2i16) as u16);
    if (enemy!(ctx, (v1) as isize, behavior_a) & 0x8000) == 0 {
        enemy_add!(ctx, (v1) as isize, x, (-2i16) as u16);
    }
    if is_off_screen(k, ctx) != 0 {
        enemy!(ctx, (v1 as usize) as isize, definition = 0);
    }
}

pub fn pirate_laser_move_right_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy_add!(ctx, (v1 as usize) as isize, x, 2);
    if (enemy!(ctx, (v1) as isize, behavior_a) & 0x8000) == 0 {
        enemy_add!(ctx, (v1) as isize, x, 2);
    }
    if is_off_screen(k, ctx) != 0 {
        enemy!(ctx, (v1 as usize) as isize, definition = 0);
    }
}

pub fn pirate_claw_thrown_left_pre_instruction(k: u16, ctx: &mut impl Host) {
    advance_thrown_claw(k, false, ctx);
}

pub fn pirate_claw_thrown_right_pre_instruction(k: u16, ctx: &mut impl Host) {
    advance_thrown_claw(k, true, ctx);
}

pub fn projectile_ab07_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let mut amt: i32 = ((8 * (ctx.session().player.x as i32 - enemy!(ctx, (v1) as isize, x) as i32))
        as i16 as i32)
        << 8;
    add_split(
        SlotField::X,
        SlotField::XFraction,
        v1 as isize,
        amt as u32,
        ctx,
    );
    amt = ((8 * (ctx.session().player.y as i32 - 36i32 - enemy!(ctx, (v1) as isize, y) as i32))
        as i16 as i32)
        << 8;
    add_split(
        SlotField::Y,
        SlotField::YFraction,
        v1 as isize,
        amt as u32,
        ctx,
    );
}

pub fn golden_torizo_egg_bouncing_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    if (enemy_add!(ctx, (v1) as isize, behavior_b, (-1i16) as u16) & 0x8000) != 0 {
        enemy_add!(ctx, (v1) as isize, program, 2);
        enemy!(ctx, 1 => (v1) as isize, delay);
        enemy!(ctx, (if (enemy!(ctx, (v1) as isize, behavior_a) & 0x8000) != 0 {
                256
            } else {
                -256i32
            }) as u16 => (v1) as isize, velocity_x);
    } else {
        if move_horizontally_with_block_collision(k, ctx) & 1 != 0 {
            enemy!(ctx, -(enemy!(ctx, (v1) as isize, velocity_x) as i32) as u16 => (v1) as isize, velocity_x);
            enemy_update!(ctx, (v1) as isize, behavior_a, |old| old ^ (0x8000));
        }
        if move_vertically_with_block_collision(k, ctx) & 1 != 0
            && (enemy!(ctx, (v1) as isize, velocity_y) & 0x8000) == 0
        {
            enemy_update!(ctx, (v1) as isize, velocity_x, |old| ((old as i32)
                + (if enemy!(ctx, (v1) as isize, velocity_x) & 0x8000 != 0 {
                    32
                } else {
                    -32i32
                }))
                as u16);
            enemy!(ctx, -(enemy!(ctx, (v1) as isize, velocity_y) as i32) as u16 => (v1) as isize, velocity_y);
        }
        enemy_add!(ctx, (v1) as isize, velocity_y, 48);
        if (enemy!(ctx, (v1) as isize, velocity_y) & 0xf000) == 4096 {
            enemy!(ctx, 0 => (v1) as isize, definition);
        }
    };
}

pub fn golden_torizo_super_missile_held_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2 = enemy!(ctx, (v1) as isize, behavior_a);
    let v3: i32 = if (ctx.resolve_actor(Key::from_legacy(v2), Le).parameters[0] & 0x8000) != 0 {
        32
    } else {
        -32i32
    };
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v2), Le).x.whole as i32 + v3) as u16 => (v1) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v2), Le).y.whole as i32 - 52i32) as u16 => (v1) as isize, y);
}

pub fn dragon_fireball_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    move_spike_axis(k, true, SlotField::VelocityX, ctx);
    move_spike_axis(k, false, SlotField::VelocityY, ctx);
    let v6: i16 = enemy!(ctx, (v1 as usize) as isize, velocity_y) as i16;
    if v6 >= 0 {
        enemy!(
            ctx,
            (v1 as usize) as isize,
            velocity_y = (v6 as u16).wrapping_add(32)
        );
        delete_if_vertically_off_screen(k, ctx);
    } else {
        enemy!(
            ctx,
            (v1 as usize) as isize,
            velocity_y = (v6 as u16).wrapping_add(32)
        );
        if (v6).wrapping_add(32) >= 0 {
            {
                let instruction: u16 = (if (enemy!(ctx, (v1) as isize, velocity_x) & 0x8000) == 0 {
                    ADDR_WORD_86B4E3 as i32
                } else {
                    ADDR_WORD_86B4D7 as i32
                }) as u16;
                start_instruction((v1 as usize) as isize, instruction, ctx);
            }
        }
    };
}

pub fn delete_if_vertically_off_screen(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    if ((enemy!(ctx, (v1) as isize, y)) as i16).wrapping_sub((ctx.inputs().camera_y) as i16) >= 288
    {
        enemy!(ctx, (v1 as usize) as isize, definition = 0);
    }
}

pub fn tourian_statue_unlocking_particle_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    move_spike_axis(k, true, SlotField::VelocityX, ctx);
    let v7: u16 = (ctx.inputs().fx_y as i32 - enemy!(ctx, (v1) as isize, y) as i32) as u16;
    move_spike_axis(k, false, SlotField::VelocityY, ctx);
    if ((v7 as i32 ^ (ctx.inputs().fx_y as i32 - enemy!(ctx, (v1) as isize, y) as i32)) & 0x8000)
        != 0
    {
        spawn_with_room_gfx(ADDR_STRU_86BA5C as u16, k, ctx);
    }
    if (enemy!(ctx, (v1) as isize, y) & 0xff00) == 256 {
        enemy!(ctx, ADDR_OFF_86B79F as u16 => (v1) as isize, program);
        enemy!(ctx, 1 => (v1) as isize, delay);
    } else {
        enemy_add!(ctx, (v1) as isize, velocity_y, 16);
    };
}

pub fn tourian_statue_soul_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    move_cacatac_spike_up(k, ctx);
    if (enemy!(ctx, (v1) as isize, y) & 0x100) != 0 {
        enemy!(ctx, ADDR_OFF_86B79F as u16 => (v1) as isize, program);
        enemy!(ctx, 1 => (v1) as isize, delay);
    }
    enemy_add!(ctx, (v1) as isize, velocity_y, (-128i16) as u16);
}

/// A lavaquake rock rising and slowing. Was `Eproj_NorfairLavaquakeRocks_Func1`.
pub fn polyp_rock_rise(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2: i16 = (enemy!(ctx, (v1) as isize, velocity_y) as i32 - 2i32) as i16;
    enemy!(ctx, v2 as u16 => (v1) as isize, velocity_y);
    if v2 >= 0 {
        for n in (1i32..=2).rev() {
            let mut v3: i16 = (n + enemy!(ctx, (v1) as isize, velocity_y) as i32 - 1i32) as i16;
            if v3 < 0 {
                v3 = 0;
            }
            let t: i32 = (8 * v3 as i32 + 4) >> 1;
            apply_polyp_rock_sample(v1 as isize, t, ctx);
        }
        polyp_rock_move_x(k, ctx);
    } else {
        enemy!(ctx, 0 => (v1) as isize, velocity_y);
        enemy!(ctx, (0x86bc8f & 0xffff) as u16 => (v1) as isize, behavior_a);
    };
}

/// A lavaquake rock falling, capped at speed 64. Was `Eproj_NorfairLavaquakeRocks_Func2`.
pub fn polyp_rock_fall(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = (enemy!(ctx, (v1) as isize, velocity_y) as i32 + 2) as u16;
    enemy!(ctx, v2 => (v1) as isize, velocity_y);
    if (v2 as i32 - 64i32) & 0x8000 == 0 {
        enemy!(ctx, 64 => (v1) as isize, velocity_y);
    }
    for n in (1i32..=2).rev() {
        let t: i32 = (8 * (enemy!(ctx, (v1) as isize, velocity_y) as i32 - n + 1)) >> 1;
        apply_polyp_rock_sample(v1 as isize, t, ctx);
    }
    polyp_rock_move_x(k, ctx);
}

/// A lavaquake rock's horizontal move. Was `Eproj_NorfairLavaquakeRocks_Func3`.
pub use self::move_cacatac_spike_right as polyp_rock_move_x;

pub fn shaktool_attack_back_circle_moving_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    if enemy!(
        ctx,
        (enemy!(ctx, v1 as isize, behavior_a) >> 1) as isize,
        definition
    ) != 0
    {
        move_horizontally_with_block_collision(k, ctx);
        move_vertically_with_block_collision(k, ctx);
    } else {
        enemy!(ctx, 0 => (v1) as isize, definition);
    };
}

pub fn mother_brain_turret_bullets_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy_update!(ctx, (v1) as isize, properties, |old| old ^ (0x8000));
    move_with_velocity(k, ctx);
    if ctx.ridley_offscreen(enemy!(ctx, (v1) as isize, x), enemy!(ctx, (v1) as isize, y)) & 1 != 0 {
        {
            let index = (ctx.session().enemy_projectiles.selected >> 1) as isize;
            enemy!(ctx, index, definition = 0);
        }
    }
}

pub fn move_to_blue_ring_spawn_position(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).x.whole as i32 + 10) as u16 => (v1) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).y.whole as i32 + 16) as u16 => (v1) as isize, y);
}

pub fn mother_brain_blue_ring_lasers_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    if enemy!(ctx, (v1) as isize, behavior_a) != 0 {
        enemy_decrement!(ctx, (v1) as isize, behavior_a);
        move_to_blue_ring_spawn_position(k, ctx);
    } else {
        move_with_velocity(k, ctx);
        let t: u8 = check_for_collision_with_baby_metroid(k, ctx);
        if t & 0x80 != 0 {
            return;
        }
        if t != 0 {
            {
                let value = ctx.mother_brain_blue_ring_contacts().wrapping_add(1);
                ctx.set_mother_brain_blue_ring_contacts(value);
            }
            blue_ring_contact_explosion(k, ctx);
            let v3: u16 = ctx.mother_brain_target_enemy();
            {
                let target = ctx.mother_brain_target_enemy();
                ctx.set_enemy_extra_health_fraction(target, 0, 16);
            }
            let v4 = v3;
            ctx.edit_actor(Key::from_legacy(v4), Le, |actor| {
                let health = (i32::from(actor.health) - 80) as i16;
                actor.health = if health < 0 { 0 } else { health as u16 };
            });
        } else if check_for_collision_with_samus(k, ctx) & 1 != 0 {
            blue_ring_contact_explosion(k, ctx);
            {
                let damage = crate::combat_rules::suit_damage(
                    0x50,
                    ctx.session().inventory.equipment.active,
                );
                ctx.damage_player(damage);
            }
            ctx.session_mut().player.invincibility = 96;
            ctx.session_mut().player.knockback_timer = 5;
            ctx.session_mut().player.knockback_x_direction = (((ctx.session().player.x as i32
                - enemy!(ctx, (k >> 1) as isize, x) as i32)
                as i16)
                >= 0) as u16;
        } else if check_for_blue_ring_collision_with_room(k, ctx) & 1 != 0 {
            handle_blue_ring_collision_with_room(k, ctx);
        }
    };
}

pub fn blue_ring_contact_explosion(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy!(ctx, 0 => (v1) as isize, definition);
    ctx.session_mut().enemy_projectiles.spawn.point = Point {
        x: enemy!(ctx, (v1) as isize, x),
        y: enemy!(ctx, (v1) as isize, y),
    };
    spawn_with_room_gfx(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 3, ctx);
    ctx.queue_sound(3, 6, 0x13);
    use_palette_0(k, ctx);
}

pub fn mother_brain_drool_attached_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2: i32 = (4 * enemy!(ctx, (v1) as isize, behavior_a) as i32) as u16 as i32 >> 1;
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).x.whole as i32 + K_EPROJ_INIT_MOTHER_BRAINS_DROOL[v2 as usize] as i32) as u16 => (v1) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).y.whole as i32 + K_EPROJ_INIT_MOTHER_BRAINS_DROOL[(v2 + 1) as usize] as i32)
            as u16 => (v1) as isize, y);
    enemy!(ctx, 0 => (v1) as isize, velocity_x);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);
}

pub fn mother_brain_rainbow_beam_explosion_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy!(ctx, (ctx.session().player.x as i32 + enemy!(ctx, (v1) as isize, velocity_x) as i32) as u16 => (v1) as isize, x);
    enemy!(ctx, (ctx.session().player.y as i32 + enemy!(ctx, (v1) as isize, velocity_y) as i32) as u16 => (v1) as isize, y);
}

pub fn mother_brain_escape_door_particles_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2 = decelerated_magnitude(enemy!(ctx, (v1) as isize, velocity_x), 16);
    enemy!(ctx, (if enemy!(ctx, (v1) as isize, velocity_x) & 0x8000 != 0 {
            -(v2 as i32)
        } else {
            v2 as i32
        }) as u16 => (v1) as isize, velocity_x);
    enemy_add!(ctx, (v1) as isize, velocity_y, 32);
    move_with_velocity(k, ctx);
    if (enemy_add!(ctx, (v1) as isize, behavior_a, (-1i16) as u16) & 0x8000) != 0 {
        enemy!(ctx, 0 => (v1) as isize, definition);
        enemy_add!(ctx, (v1) as isize, y, (-4i16) as u16);
        ctx.session_mut().enemy_projectiles.spawn.point = Point {
            x: enemy!(ctx, (v1) as isize, x),
            y: enemy!(ctx, (v1) as isize, y),
        };
        spawn_with_room_gfx(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 9, ctx);
    }
}

pub fn time_bomb_set_japanese_text_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy!(ctx, 0 => (v1) as isize, velocity_x);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);
    enemy!(ctx, 128 => (v1) as isize, x);
    enemy!(ctx, 192 => (v1) as isize, y);
}

/// A falling tube bursting into dust, then falling.
/// Was `MotherBrainTubeFallingFunc_GenerateExplosion`.
pub fn falling_tube_burst(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    ctx.session_mut().enemy_projectiles.spawn.point = Point {
        x: enemy!(ctx, (v1) as isize, x),
        y: (enemy!(ctx, (v1) as isize, y) as i32 + 8) as u16,
    };
    spawn_with_room_gfx(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 9, ctx);
    enemy!(ctx, (0x86cc08 & 0xffff) as u16 => (v1) as isize, behavior_a);
    falling_tube_fall(k, ctx);
}

/// A falling tube's fall, ending in dust below the floor.
/// Was `MotherBrainTubeFallingFunc_Falling`.
pub fn falling_tube_fall(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy_add!(ctx, (v1) as isize, velocity_y, 6);
    let v2: i16 = move_with_y_velocity(k, ctx) as i16;
    if (v2 as i32 - 208i32) & 0x8000 == 0 {
        enemy!(ctx, 0 => (v1) as isize, definition);
        ctx.session_mut().enemy_projectiles.spawn.point = Point {
            x: enemy!(ctx, (v1) as isize, x),
            y: enemy!(ctx, (v1) as isize, y),
        };
        spawn_with_room_gfx(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 0xc, ctx);
    }
}

pub fn kihunter_acid_spit_left_start_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy!(ctx, (0x86cff8 & 0xffff) as u16 => (v1) as isize, pre_action);
    enemy_add!(ctx, (v1) as isize, x, (-19i16) as u16);
}

pub fn kihunter_acid_spit_right_start_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy!(ctx, (0x86cff8 & 0xffff) as u16 => (v1) as isize, pre_action);
    enemy_add!(ctx, (v1) as isize, x, 19);
}

pub fn kihunter_acid_spit_moving_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    if move_vertically_with_block_collision(k, ctx) & 1 != 0 {
        enemy!(ctx, ADDR_OFF_86CF56 as u16 => (v1) as isize, program);
        enemy!(ctx, 1 => (v1) as isize, delay);
    } else if move_horizontally_with_block_collision(k, ctx) & 1 != 0 {
        enemy!(ctx, 0 => (k >> 1) as isize, velocity_x);
    } else {
        let mut v2: u16 = (enemy!(ctx, (v1) as isize, velocity_y) as i32 + 16) as u16;
        enemy!(ctx, v2 => (v1) as isize, velocity_y);
        if (v2 as i32 - 512i32) & 0x8000 == 0 {
            v2 = 512;
        }
        enemy!(ctx, v2 => (v1) as isize, velocity_y);
    };
}

/// Counts down to the Kago bug's cry. Was `EprojPreInstr_KagosBugs_Func1`.
pub fn kago_bug_cry_countdown(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = enemy!(ctx, (v1 as usize) as isize, behavior_a);
    if v2 != 0 {
        let v3: u16 = v2.wrapping_sub(1);
        enemy!(ctx, (v1 as usize) as isize, behavior_a = v3);
        if v3 == 0 {
            ctx.queue_sound(2, 6, 0x6c);
        }
    }
}

/// Flags the Kago bug once it is 23 pixels or more from its Kago.
/// Was `EprojPreInstr_KagosBugs_Func2`.
pub fn kago_bug_flag_when_far(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let enemy_index = enemy!(ctx, (v1) as isize, behavior_b);
    let v3: u16 = (((ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole)
        .wrapping_sub(enemy!(ctx, (k >> 1) as isize, x))) as i16)
        .unsigned_abs();
    if (v3 as i32 - 23i32) & 0x8000 == 0 {
        set_property_bits((v1 as usize) as isize, 0x8000, ctx);
    }
}

pub fn powamp_spike_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy_update!(ctx, (v1) as isize, velocity_x, |old| ((old as i32)
        + (WORD_86_D21_A[enemy!(ctx, (v1) as isize, behavior_a) as usize] as i32))
        as u16);
    if move_horizontally_with_block_collision(k, ctx) & 1 != 0 || {
        enemy_update!(ctx, (v1) as isize, velocity_y, |old| ((old as i32)
            + (WORD_86_D22_A[enemy!(ctx, (v1) as isize, behavior_a) as usize] as i32))
            as u16);
        move_vertically_with_block_collision(k, ctx) & 1 != 0
    } {
        enemy!(ctx, ADDR_OFF_86D218 as u16 => (v1) as isize, program);
        enemy!(ctx, 1 => (v1) as isize, delay);
    }
}

pub fn work_robot_laser_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy!(ctx, 0 => (v1) as isize, graphics);
    if move_horizontally_with_block_collision(k, ctx) & 1 != 0
        || move_vertically_with_block_collision(k, ctx) & 1 != 0
    {
        enemy!(ctx, 0 => (v1) as isize, definition);
    }
}

pub fn n00b_tube_crack_flickering_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    if enemy!(ctx, (v1) as isize, x) != 0xee00 {
        enemy!(ctx, enemy!(ctx, (v1) as isize, x) => (v1) as isize, behavior_a);
    }
    if ctx.inputs().frame & 1 != 0 {
        enemy!(ctx, -4608i32 as u16 => (v1) as isize, x);
    } else {
        enemy!(ctx, enemy!(ctx, (v1) as isize, behavior_a) => (v1) as isize, x);
    };
}

pub fn n00b_tube_crack_falling_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let amt: i32 = (192i16 as i32) << 8;
    add_split(
        SlotField::Y,
        SlotField::YFraction,
        v1 as isize,
        amt as u32,
        ctx,
    );
}

pub fn move_cacatac_spike_down(k: u16, ctx: &mut impl Host) {
    move_spike_axis(k, false, SlotField::VelocityX, ctx);
}

pub use self::move_cacatac_spike_left as mini_crocomire_projectile_move_left;

pub fn spores_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = enemy_byte!(ctx, v1 as isize * 2, behavior_a) as u16;
    let mut r18: u16 = (spore_delta(enemy_byte!(ctx, v1 as isize * 2, behavior_a) as isize, ctx)
        as u16) as u8 as i8 as i16 as u16;
    if (enemy!(ctx, (v1) as isize, behavior_b) & 0x80) != 0 {
        r18 = r18.wrapping_neg();
    }
    enemy_add!(ctx, (v1) as isize, x, r18);
    r18 = (spore_delta((v2 as i32 + 1) as isize, ctx) as u16) as u8 as i8 as i16 as u16;
    let v3: u16 = (r18 as i32 + enemy!(ctx, (v1) as isize, y) as i32 + r18 as i32) as u16;
    enemy!(ctx, v3 => (v1) as isize, y);
    if (v3 as i32 - 768i32) & 0x8000 == 0 {
        enemy!(ctx, 0 => (v1) as isize, definition);
    }
    enemy!(ctx, (enemy_byte!(ctx, v1 as isize * 2, behavior_a) as i32 + 2) as u8 as u16 => (v1) as isize, behavior_a);
}

pub fn dust_cloud_moving_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let mut v2: u16 = (enemy!(ctx, (v1) as isize, counter))
        .wrapping_add(((enemy!(ctx, (v1) as isize, velocity_y)) as i16).unsigned_abs());
    if v2 >= 0x1000 {
        enemy!(ctx, 1 => (v1) as isize, delay);
        let _fresh60 = enemy_add!(ctx, (v1 as usize) as isize, program, 1);
        let _fresh61 = enemy_add!(ctx, (v1 as usize) as isize, program, 1);
        v2 = 0;
    }
    enemy!(ctx, v2 => (v1) as isize, counter);
    {
        let index = (v1) as isize;
        let old = enemy!(ctx, index, y_fraction);
        let value =
            ((old as i32) + ((enemy_byte!(ctx, v1 as isize * 2, velocity_y) as i32) << 8)) as u16;
        enemy!(ctx, index, y_fraction = value);
    };
    {
        let index = (v1) as isize;
        let old = enemy!(ctx, index, y);
        let value =
            old.wrapping_add((enemy_byte!(ctx, v1 as isize * 2 + 1, velocity_y) as i8) as u16);
        enemy!(ctx, index, y = value);
    };
}

/// Moves along the projectile's angle on both axes. Was `Eproj_FuncE73E_MoveXY`.
pub fn move_along_angle(k: u16, ctx: &mut impl Host) {
    let index = (k >> 1) as isize;
    integrate_angle_axis(
        index,
        SlotField::X,
        SlotField::XFraction,
        SlotField::VelocityX,
        SlotField::BehaviorA,
        64,
        ctx,
    );
    integrate_angle_axis(
        index,
        SlotField::Y,
        SlotField::YFraction,
        SlotField::VelocityY,
        SlotField::BehaviorB,
        128,
        ctx,
    );
}

pub fn botwoon_body_main(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = {
        let record = ctx.loaded_record(0x86, 0xe9f1);
        record.native_word(
            ((enemy!(ctx, (v1) as isize, behavior_a) >> 1) as isize)
                .checked_mul(2)
                .unwrap(),
        )
    };
    if v2 != enemy!(ctx, (v1) as isize, behavior_b) {
        enemy!(ctx, v2 => (v1) as isize, program);
        enemy!(ctx, v2 => (v1) as isize, behavior_b);
        enemy!(ctx, 1 => (v1) as isize, delay);
    }
    botwoon_body_hurt_flash(k, ctx);
}

pub fn botwoon_body_hurt_flash(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy_update!(ctx, (v1) as isize, graphics, |old| old | (0xe00));
    if ctx.resolve_actor(Key::from_legacy(0), Le).flash != 0
        && (ctx.inputs().random_enemy_counter & 2) != 0
    {
        enemy_update!(ctx, (v1) as isize, graphics, |old| old & (0xf1ff));
    }
}

pub fn falling_spark_pre_instruction(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    if (enemy!(ctx, (v1) as isize, velocity_y) & 0x8000) == 0 {
        if move_vertically_with_block_collision(k, ctx) & 1 != 0 {
            enemy!(ctx, 0xf363 => (v1) as isize, program);
            enemy!(ctx, 1 => (v1) as isize, delay);
            for _ in 0..2 {
                let high = (enemy!(ctx, v1 as isize, behavior_b) as i32 * 2
                    + (enemy!(ctx, v1 as isize, behavior_a) as i32 >> 15))
                    as u16;
                enemy!(ctx, v1 as isize, behavior_b = high);
                let low = enemy!(ctx, v1 as isize, behavior_a).wrapping_mul(2);
                enemy!(ctx, v1 as isize, behavior_a = low);
            }
            enemy!(ctx, 0x8000 => (v1) as isize, velocity_x);
            enemy!(ctx, -1i32 as u16 => (v1) as isize, velocity_y);
            enemy_add!(ctx, (v1) as isize, y, (-2i16) as u16);
            return;
        }
        let v2: u16 = enemy!(ctx, (v1) as isize, velocity_x);
        enemy!(ctx, (v2 as i32 + 0x4000) as u16 => (v1) as isize, velocity_x);
        let v3: u16 = (u16::overflowing_add(v2, 0x4000u16).1 as i32
            + enemy!(ctx, (v1) as isize, velocity_y) as i32) as u16;
        if v3 < 4 {
            enemy!(ctx, v3 => (v1) as isize, velocity_y);
        }
    }
    add_split(
        SlotField::Y,
        SlotField::YFraction,
        v1 as isize,
        (((enemy!(ctx, (v1) as isize, velocity_y)) as u32) << 16)
            | ((enemy!(ctx, (v1) as isize, velocity_x)) as u32),
        ctx,
    );
    add_split(
        SlotField::X,
        SlotField::XFraction,
        v1 as isize,
        (((enemy!(ctx, (v1) as isize, behavior_b)) as u32) << 16)
            | ((enemy!(ctx, (v1) as isize, behavior_a)) as u32),
        ctx,
    );
    if (ctx.inputs().nmi_counter_low & 3) == 0 {
        let request = SpriteSpawn {
            position: Point {
                x: enemy!(ctx, (v1) as isize, x),
                y: enemy!(ctx, (v1) as isize, y),
            },
            definition: LegacySpriteDefinition(48),
            appearance: enemy!(ctx, (v1) as isize, graphics),
        };
        sprite_objects::spawn(ctx, request);
    }
}

pub fn cacatac_spike_left(k: u16, ctx: &mut impl Host) {
    move_cacatac_spike_left(k, ctx)
}

pub fn cacatac_spike_right(k: u16, ctx: &mut impl Host) {
    move_cacatac_spike_right(k, ctx)
}

pub fn cacatac_spike_up(k: u16, ctx: &mut impl Host) {
    move_cacatac_spike_up(k, ctx)
}

pub use self::proto_vertical_afterburn_up_pre_instruction as proto_vertical_afterburn_down_pre_instruction;

fn advance_thrown_claw(k: u16, initially_right: bool, ctx: &mut impl Host) {
    let index = (k >> 1) as isize;
    let outbound = enemy!(ctx, index, behavior_b) != 0;
    let x = enemy!(ctx, index, x);
    let speed = u16::from(enemy_byte!(ctx, index * 2 + 1, behavior_a));
    let right = if outbound {
        initially_right
    } else {
        !initially_right
    };
    enemy!(
        ctx,
        index,
        x = if right {
            x.wrapping_add(speed)
        } else {
            x.wrapping_sub(speed)
        }
    );
    if outbound {
        let speed = enemy!(ctx, index, behavior_a).wrapping_sub(32);
        enemy!(ctx, index, behavior_a = speed);
        if speed == 0 {
            enemy!(ctx, index, behavior_b = 0);
        }
    } else {
        enemy_add!(ctx, index, behavior_a, 32);
    }
    enemy_add!(ctx, index, y, 1);
    if is_off_screen(k, ctx) != 0 {
        enemy!(ctx, index, definition = 0);
    }
}

fn integrate_angle_axis(
    index: isize,
    high: SlotField,
    low: SlotField,
    velocity: SlotField,
    fraction: SlotField,
    phase: i32,
    ctx: &mut impl Host,
) {
    // Y must reread Angle after the complete X integration.
    let negative = ((enemy!(ctx, index, angle) as i32 + phase) & 0x80) != 0;
    let amount = (((read_member(ctx, velocity, index)) as u32) << 16)
        | ((read_member(ctx, fraction, index)) as u32);
    // Preserve the original signed-min negation panic in checked builds.
    let amount = if negative {
        -(amount as i32) as u32
    } else {
        amount
    };
    add_split(high, low, index, amount, ctx);
}

fn apply_polyp_rock_sample(index: isize, table_index: i32, ctx: &mut impl Host) {
    let amount = polyp_rock_delta((table_index >> 1) as isize, ctx);
    add_split(SlotField::Y, SlotField::YFraction, index, amount, ctx);
    let record = ctx.loaded_record(0xa0, 0xcbc7);
    let value = record.native_word(((table_index + 1) as isize).checked_mul(2).unwrap());
    enemy!(ctx, index, behavior_b = value);
}
