//! Ridley behavior, scripts, tail, collisions and player control over owned state.
#![forbid(unsafe_code)]
use super::{
    DEATH_SEQUENCE, DUST_CLOUD, Deceleration, ENEMY_DEFINITION_BANK, ENEMY_DEFINITION_DAMAGE,
    Explosion, FACING_FORWARD, FACING_LEFT, FACING_RIGHT, FEET_CLENCHED_TILES,
    FEET_UNCLENCHED_TILES, FIREBALL, GRABBED_HURT_LAG, INERTIA, LOCK_SAMUS, MISSILE,
    MOVEMENT_TYPE_FLAGS, NO_WHIP_TARGET, NORFAIR, RIBS_ANIMATION, RIDLEY, Rect, Ridley,
    SPIN_JUMPING, SUPER_MISSILE, UNLOCK_SAMUS, actions, baby_metroid_function,
    baby_metroid_instruction, explosion_instruction_list, function, instruction as instruction_id,
    instruction_list, tail, tail_address, tail_data,
};
use crate::actors::api::angle_from_delta;
use crate::actors::host::{Death, Shot, abs16, below};
use crate::actors::session_slots::session_slot;
use crate::carry::{adc, sbc};
use crate::constants::enemy_properties::{
    K_ENEMY_PROPS_DELETED, K_ENEMY_PROPS_INVISIBLE, K_ENEMY_PROPS_TANGIBLE,
};
use crate::display_refresh::mult8x8;
use crate::enemy_projectiles::drops::DropSource;
use crate::enemy_projectiles::shared_pre::{x_velocity_at_angle, y_velocity_at_angle};
use crate::enemy_script::EnemyScript;
use crate::geometry::Point;
use sm_data::ridley_explosion::{
    HAND_ADDRESS, LEGS_ADDRESS, MOUTH_ADDRESS, TORSO_ADDRESS, WINGS_ADDRESS,
};

#[derive(Clone, Copy)]
pub enum ProjectileField {
    X,
    Y,
    XRadius,
    YRadius,
    Kind,
    Direction,
}
pub trait Host:
    crate::actors::access::ActorAccess
    + crate::actors::session_slots::SessionSlots
    + crate::room_mechanisms::host::ImportedAliases
    + crate::effects::SoundEffects
{
    fn actor_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord;
    fn run_ceres_ridley_function(&mut self, function: u16);
    fn frame(&self) -> u16;
    fn roll_random(&mut self) -> u16;
    fn place_projectile_spawn(&mut self, x: u16, y: u16);
    fn spawn_room_projectile(&mut self, definition: u16, parameter: u16) -> u16;
    fn spawn_population(&mut self, bank: u8, population: u16) -> u16;
    fn set_boss_bit(&mut self, mask: u16);
    fn hardware_sine_cosine(&mut self, amplitude: u16, angle: u16) -> (u16, u16);
    fn move_liquid(&mut self, target: u16, velocity: u16, delay: u16);
    fn command_player(&mut self, command: u16) -> u16;
    fn normal_death(&mut self, animation: u16);
    fn scatter_enemy_drop(&mut self, source: DropSource, key: u16);
    fn queue_music_delayed(&mut self, track: u16);
    fn suit_damage(&mut self, amount: u16) -> u16;
    fn damage_player(&mut self, amount: u16);
    fn contact_reaction(&mut self, death: Death);
    fn shot_reaction(&mut self, shot: Shot);
    fn power_bomb_reaction(&mut self, death: Death);
    fn unreachable(&mut self) -> !;
    fn imported_tail_byte(&self, relative: usize) -> u8;
    fn set_imported_tail_byte(&mut self, relative: usize, value: u8);
    fn imported_projectile_field(&self, index: usize, field: ProjectileField) -> u16;
    fn set_imported_projectile_field(&mut self, index: usize, field: ProjectileField, value: u16);
    fn ridley_hover_behind(&mut self, index: isize) -> u16;
    fn ridley_hover_in_front(&mut self, index: isize) -> u16;
    fn ridley_pogo_target(&mut self, index: isize) -> u16;
    fn ridley_feet_x(&mut self, index: isize) -> u16;
    fn ridley_feet_y(&mut self, index: isize) -> u16;
    fn ridley_lunge_side(&mut self, index: isize) -> u16;
    fn ridley_lunge_acceleration(&mut self, index: isize) -> u16;
    fn ridley_tail_angles(&mut self, index: isize) -> sm_data::ridley::TailAngleLimits;
    fn explosion_lifetime(&mut self, index: isize) -> u16;
    fn explosion_placement(
        &mut self,
        part: u32,
        side: isize,
    ) -> sm_data::ridley_explosion::PartPlacement;
    fn tail_tip_explosion_list(&mut self, index: isize) -> u16;
    fn pogo_row(&mut self, horizontal: bool, index: isize) -> u16;
    fn pogo_acceleration(&mut self, upwards: bool, index: isize) -> u16;
    fn set_imported_tail_destination_byte(&mut self, destination: u16, offset: usize, value: u8);
    fn install_baby_graphics_hook(&mut self);
    fn draw_baby_metroid_and_door(&mut self);
    fn draw_baby_metroid(&mut self);
    fn draw_tail(&mut self);
    fn draw_wings(&mut self);
    fn load_health_palette(&mut self, index: u16);
    fn hurt_flash(&mut self);
    fn baby_metroid_load_palette(&mut self, arguments: u16) -> u16;
    fn queue_tile_pair(&mut self, sources: [u16; 2], destinations: [u16; 2], size: u16);
    fn tail_byte(&self, relative: usize) -> u8 {
        if relative < 4096 {
            (*self
                .session()
                .actors
                .extended
                .word(4 + relative / 2048, (relative % 2048) / 2))
            .to_le_bytes()[relative & 1]
        } else {
            self.imported_tail_byte(relative)
        }
    }
    fn set_tail_byte(&mut self, relative: usize, value: u8) {
        if relative < 4096 {
            let word = &mut *self
                .session_mut()
                .actors
                .extended
                .word_mut(4 + relative / 2048, (relative % 2048) / 2);
            let mut bytes = word.to_le_bytes();
            bytes[relative & 1] = value;
            *word = u16::from_le_bytes(bytes);
        } else {
            self.set_imported_tail_byte(relative, value);
        }
    }
    fn projectile_field(&self, index: usize, field: ProjectileField) -> u16 {
        let Some(shot) = self.session().projectiles.shots.get(index) else {
            return self.imported_projectile_field(index, field);
        };
        match field {
            ProjectileField::X => shot.x,
            ProjectileField::Y => shot.y,
            ProjectileField::XRadius => shot.x_radius,
            ProjectileField::YRadius => shot.y_radius,
            ProjectileField::Kind => shot.kind,
            ProjectileField::Direction => shot.direction,
        }
    }
    fn set_projectile_field(&mut self, index: usize, field: ProjectileField, value: u16) {
        if let Some(shot) = self.session_mut().projectiles.shots.get_mut(index) {
            *match field {
                ProjectileField::X => &mut shot.x,
                ProjectileField::Y => &mut shot.y,
                ProjectileField::XRadius => &mut shot.x_radius,
                ProjectileField::YRadius => &mut shot.y_radius,
                ProjectileField::Kind => &mut shot.kind,
                ProjectileField::Direction => &mut shot.direction,
            } = value;
        } else {
            self.set_imported_projectile_field(index, field, value);
        }
    }
    fn set_tail_destination_byte(&mut self, destination: u16, offset: usize, value: u8) {
        if let Some(relative) = usize::from(destination)
            .checked_sub(0x2000)
            .and_then(|r| r.checked_add(offset))
            .filter(|&r| r < 4096)
        {
            self.set_tail_byte(relative, value);
        } else {
            self.set_imported_tail_destination_byte(destination, offset, value);
        }
    }
}
mod bg2_staging {
    use super::Host;
    pub fn get<H: Host + ?Sized>(h: &H, index: usize) -> u16 {
        u16::from_le_bytes([h.tail_byte(2 * index), h.tail_byte(2 * index + 1)])
    }
    pub fn set<H: Host + ?Sized>(h: &mut H, index: usize, value: u16) {
        let [low, high] = value.to_le_bytes();
        h.set_tail_byte(2 * index, low);
        h.set_tail_byte(2 * index + 1, high);
    }
}
mod interaction {
    use super::{Host, ProjectileField};
    pub fn get_at<H: Host + ?Sized>(h: &H, field: ProjectileField, index: usize) -> u16 {
        h.projectile_field(index, field)
    }
    pub fn set_at<H: Host + ?Sized>(h: &mut H, field: ProjectileField, index: usize, value: u16) {
        h.set_projectile_field(index, field, value);
    }
}

pub(crate) fn separated_from_rect<L: Host + ?Sized>(k: u16, rect: Rect, game: &mut L) -> bool {
    let e = i32::from(k);
    let width = (session_slot!(game, Ridley[e].base.x_width) as i32 + rect.w as i32 + 1) as u16;
    let dx = abs16(
        rect.x
            .wrapping_sub(session_slot!(game, Ridley[e].base.x_pos)),
    );
    if dx < width {
        let height =
            (session_slot!(game, Ridley[e].base.y_height) as i32 + rect.h as i32 + 1) as u16;
        return abs16(
            rect.y
                .wrapping_sub(session_slot!(game, Ridley[e].base.y_pos)),
        ) >= height;
    }
    true
}

pub(super) fn current<L: Host + ?Sized>(game: &L) -> i32 {
    game.session().actors.controls.selected as i32
}

pub(super) fn timer_expired<L: Host + ?Sized>(game: &mut L) -> bool {
    session_slot!(game, Ridley[RIDLEY].function_timer, |function_timer| {
        function_timer.wrapping_sub(1)
    });
    (session_slot!(game, Ridley[RIDLEY].function_timer) & 0x8000) != 0
}

pub(super) fn tail_low_byte<L: Host + ?Sized>(game: &L, index: usize) -> u16 {
    u16::from(game.tail_byte(index * 2))
}

pub(super) fn snes_divide(a: u16, b: u8) -> u16 {
    (if b as i32 == 0 {
        0xffff
    } else {
        a as i32 / b as i32
    }) as u16
}

pub(super) fn tail_word(game: &(impl Host + ?Sized), index: usize) -> u16 {
    let at = index * 2;
    u16::from_le_bytes([game.tail_byte(at), game.tail_byte(at + 1)])
}

pub(super) fn set_tail_word(game: &mut (impl Host + ?Sized), index: usize, value: u16) {
    let at = index * 2;
    let [low, high] = value.to_le_bytes();
    game.set_tail_byte(at, low);
    game.set_tail_byte(at + 1, high);
}

pub(super) fn chained_homing_velocity(
    position: u16,
    target: u16,
    velocity: u16,
    divisor: u8,
    deceleration: Deceleration,
    limit: u16,
) -> u16 {
    let (distance, mut carry) = sbc(position, target, true);
    let distance = distance as i16;
    let step = snes_divide(distance.unsigned_abs(), divisor).max(1);
    let reversing = (velocity as i16 >= 0) == (distance > 0);
    let mut velocity = velocity;
    if distance > 0 {
        if reversing {
            match deceleration {
                Deceleration::Force(force) => {
                    velocity = velocity.wrapping_sub(force);
                    (velocity, carry) = sbc(velocity, 8, true);
                    (velocity, carry) = sbc(velocity, step, carry);
                }
                Deceleration::DoubleStep => {
                    (velocity, carry) = sbc(velocity, step, carry);
                    (velocity, carry) = sbc(velocity, step, carry);
                }
            }
        }
        velocity = sbc(velocity, step, carry).0;
        if velocity.wrapping_add(limit) & 0x8000 != 0 {
            velocity = 0u16.wrapping_sub(limit);
        }
    } else {
        if reversing {
            match deceleration {
                Deceleration::Force(force) => {
                    velocity = velocity.wrapping_add(force);
                    (velocity, carry) = adc(velocity, 8, false);
                    (velocity, carry) = adc(velocity, step, carry);
                }
                Deceleration::DoubleStep => {
                    (velocity, carry) = adc(velocity, step, carry);
                    (velocity, carry) = adc(velocity, step, carry);
                }
            }
        }
        velocity = adc(velocity, step, carry).0;
        if velocity.wrapping_sub(limit) & 0x8000 == 0 {
            velocity = limit;
        }
    }
    velocity
}

pub(super) fn move_with_velocity<L: Host + ?Sized>(game: &mut L) {
    let e = game.session().actors.controls.selected as i32;
    let x_velocity = session_slot!(game, Explosion[e].x_velocity);
    let carry = session_slot!(byte game, Explosion[e].base.x_subpos[1]) as i32
        + x_velocity.to_ne_bytes()[0] as i32;
    session_slot!(byte game, Explosion[e].base.x_subpos[1] = carry as u8);
    session_slot!(game, Explosion[e].base.x_pos, |old| (old as i32
        + ((x_velocity >> 8) as i8 as i32 + (carry >> 8)))
        as u16);
    let y_velocity = session_slot!(game, Explosion[e].y_velocity);
    let carry = session_slot!(byte game, Explosion[e].base.y_subpos[1]) as i32
        + y_velocity.to_ne_bytes()[0] as i32;
    session_slot!(byte game, Explosion[e].base.y_subpos[1] = carry as u8);
    session_slot!(game, Explosion[e].base.y_pos, |old| (old as i32
        + ((y_velocity >> 8) as i8 as i32 + (carry >> 8)))
        as u16);
}

pub fn handle_baby_metroid<L: Host + ?Sized>(game: &mut L) {
    run_baby_metroid_function(game);
    if session_slot!(game, Ridley[RIDLEY].movement_enabled) != 0 {
        // The hook is the long pointer $A6:A2F2, `draw_baby_metroid_and_door`.
        game.install_baby_graphics_hook();
    } else {
        game.draw_baby_metroid_and_door();
    }
}

pub fn main<L: Host + ?Sized>(lease: &mut L) {
    // The state steps, Ridley's function, the tail, drawing and ribs run on
    // the lease.
    let mut lag = (session_slot!(lease, Ridley[RIDLEY].grabbed_lag_timer) as i16).wrapping_sub(4);
    if lag < 0 {
        lag = 0;
    }
    session_slot!(lease, Ridley[RIDLEY].grabbed_lag_timer = lag as u16);
    set_tangibility(lease);
    lunge_if_power_bomb(lease);
    let function = session_slot!(lease, Ridley[RIDLEY].function);
    lease.run_ceres_ridley_function(function);
    if session_slot!(lease, Ridley[RIDLEY].movement_enabled) != 0 {
        lease.hurt_flash();
        move_ridley(lease);
        update_wings(lease);
        handle_tail(lease);
        tail_projectile_collision(lease);
        lease.draw_tail();
        lease.draw_wings();
        handle_ribs(lease);
        if session_slot!(lease, Ridley[RIDLEY].holding_samus) != 0 {
            move_samus_with_feet(lease);
        }
    }
    health_palette(lease);
}

pub(crate) fn run_ridley_function<L: Host + ?Sized>(game: &mut L, function: u16) -> bool {
    match function {
        function::LIFTOFF => liftoff(game),
        function::DECIDE_ACTION => decide_action(game),
        function::LUNGE_MISSED => lunge_missed(game),
        function::LUNGE_MISSED_MOVE_TO_POSITION => lunge_missed_move_to_position(game),
        function::SWOOP => swoop(game),
        function::SWOOP_MOVE_TO_POSITION => swoop_move_to_position(game),
        function::SWOOP_AIMING_DOWN => swoop_aiming_down(game),
        function::SWOOP_AIMING_HORIZONTAL => swoop_aiming_horizontal(game),
        function::SWOOP_AIMING_UP => swoop_aiming_up(game),
        function::SWOOP_AIMING_UP_FASTER => swoop_aiming_up_faster(game),
        function::SWOOP_DECELERATE => swoop_decelerate(game),
        function::HOVER => hover(game),
        function::HOVER_TO_WALL_BEHIND => hover_to_wall_behind(game),
        function::HOVER_TO_WALL_IN_FRONT => hover_to_wall_in_front(game),
        function::POGO_FLY_TO_POSITION => pogo_fly_to_position(game),
        function::POGO_WAIT_AT_POSITION => pogo_wait_at_position(game),
        function::POGO_DESCENDING => pogo_descending(game),
        function::POGO_ASCENDING => pogo_ascending(game),
        function::LUNGE => lunge(game),
        function::GRABBED_SAMUS => grabbed_samus(game),
        function::GRABBED_SAMUS_RISING => grabbed_samus_rising(game),
        function::GRABBED_SAMUS_RISING_TO_MIDDLE => grabbed_samus_rising_to_middle(game),
        function::GRABBED_SAMUS_RELEASE => grabbed_samus_release(game),
        function::DODGE_POWER_BOMB => dodge_power_bomb(game),
        function::FAKE_RETREAT_MOVE_TO_POSITION => fake_retreat_move_to_position(game),
        function::FAKE_RETREAT_RISING => fake_retreat_rising(game),
        function::DROP_BABY_METROID => drop_baby_metroid(game),
        function::RETRIEVE_BABY_METROID => retrieve_baby_metroid(game),
        function::DYING_GRABBED_SAMUS => dying_grabbed_samus(game),
        function::DEATH_ROAR => death_roar(game),
        function::DEATH_MOVE_TO_DEATH_SPOT => death_move_to_death_spot(game),
        function::LOWER_ACID_AND_RELEASE_SAMUS => lower_acid_and_release_samus(game),
        function::START_FALLING_APART => start_falling_apart(game),
        function::FALLING_APART => falling_apart(game),
        function::STILL_FALLING_APART => still_falling_apart(game),
        function::DEAD => {}
        _ => return false,
    }
    true
}

pub fn time_frozen<L: Host + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY].grabbed_lag_timer = 0);
    session_slot!(game, Ridley[RIDLEY].base.frame_counter = 1);
    hurt(game);
}

pub fn hurt<L: Host + ?Sized>(game: &mut L) {
    let animate = if session_slot!(game, Ridley[RIDLEY].base.frame_counter) & 1 != 0 {
        true
    } else {
        set_tangibility(game);
        lunge_if_power_bomb(game);
        let function = session_slot!(game, Ridley[RIDLEY].function);
        game.run_ceres_ridley_function(function);
        if session_slot!(game, Ridley[RIDLEY].movement_enabled) != 0 {
            move_ridley(game);
            update_wings(game);
            handle_tail(game);
            true
        } else {
            false
        }
    };
    if animate {
        tail_projectile_collision(game);
        game.hurt_flash();
        game.draw_tail();
        game.draw_wings();
        health_palette(game);
        handle_ribs(game);
        let mut lag = session_slot!(game, Ridley[RIDLEY].grabbed_lag_timer).wrapping_add(1);
        if (lag as i16).wrapping_sub(GRABBED_HURT_LAG as i16) >= 0 {
            lag = GRABBED_HURT_LAG;
        }
        session_slot!(game, Ridley[RIDLEY].grabbed_lag_timer = lag);
        if (session_slot!(game, Ridley[RIDLEY].grabbed_lag_timer) as i16)
            .wrapping_sub(GRABBED_HURT_LAG as i16)
            < 0
        {
            return;
        }
    }
    if session_slot!(game, Ridley[RIDLEY].holding_samus) != 0 {
        move_samus_with_feet(game);
    }
}

pub fn liftoff<L: Host + ?Sized>(game: &mut L) {
    let rect = Rect {
        x: 64,
        y: 256,
        w: 8,
        h: 8,
    };
    accelerate(0, 14, rect.x, rect.y, game);
    if !separated_from_rect(0, rect, game) {
        session_slot!(game, Ridley[RIDLEY]; fight_mode = 1, function = function::DECIDE_ACTION);
        decide_action(game);
    }
}

pub fn decide_action<L: Host + ?Sized>(game: &mut L) {
    let choices = fight_action_choices(game);
    let choice = 2 * (game.roll_random() & 7) as usize;
    let next = game.actor_record(0xa6, choices).word(choice);
    session_slot!(game, Ridley[RIDLEY].function = next);
    game.run_ceres_ridley_function(next);
}

pub fn fight_action_choices<L: Host + ?Sized>(game: &mut L) -> u16 {
    if game.session().player.movement_and_facing.to_le_bytes()[1] == SPIN_JUMPING {
        return actions::HOVER;
    }
    let health = session_slot!(game, Ridley[RIDLEY].base.health);
    if health == 0 {
        session_slot!(game, Ridley[RIDLEY].death_lunges, |death_lunges| {
            death_lunges.wrapping_add(1)
        });
        actions::LUNGE
    } else if below(health, 14400) {
        actions::POGO_OR_SWOOP
    } else if samus_in_pogo_zone(game) {
        actions::POGO
    } else if samus_not_spinning(game) {
        actions::LUNGE_OR_POGO
    } else if below(session_slot!(game, Ridley[RIDLEY].base.health), 9000) {
        actions::SWOOP_OR_POGO
    } else {
        actions::POGO_OR_SWOOP
    }
}

pub fn lunge_missed<L: Host + ?Sized>(game: &mut L) {
    session_slot!(
        game,
        Ridley[RIDLEY].function = function::LUNGE_MISSED_MOVE_TO_POSITION
    );
    session_slot!(game, Ridley[RIDLEY].function_timer = 128);
    lunge_missed_move_to_position(game);
}

pub fn lunge_missed_move_to_position<L: Host + ?Sized>(game: &mut L) {
    if timer_expired(game) || {
        let x = if session_slot!(game, Ridley[RIDLEY].facing) != 0 {
            96
        } else {
            192
        };
        let rect = Rect {
            x,
            y: 256,
            w: 8,
            h: 8,
        };
        accelerate(0, acceleration_factor(game), rect.x, rect.y, game);
        !separated_from_rect(0, rect, game)
    } {
        session_slot!(game, Ridley[RIDLEY].function = function::DECIDE_ACTION);
    }
}

pub fn acceleration_factor<L: Host + ?Sized>(game: &mut L) -> u16 {
    const FACTORS: [u16; 4] = [4, 8, 0xa, 0xc];
    FACTORS[session_slot!(game, Ridley[RIDLEY].acceleration_index) as usize]
}

pub fn swoop<L: Host + ?Sized>(game: &mut L) {
    session_slot!(
        game,
        Ridley[RIDLEY].function = function::SWOOP_MOVE_TO_POSITION
    );
    session_slot!(game, Ridley[RIDLEY]; timer = 10, swoop_angle = 0);
    swoop_move_to_position(game);
}

pub fn swoop_move_to_position<L: Host + ?Sized>(game: &mut L) {
    let x = if session_slot!(game, Ridley[RIDLEY].facing) != 0 {
        64
    } else {
        192
    };
    let rect = Rect {
        x,
        y: 128,
        w: 8,
        h: 8,
    };
    accelerate(0, 1, rect.x, rect.y, game);
    if !separated_from_rect(0, rect, game) {
        session_slot!(game, Ridley[RIDLEY]; function = function::SWOOP_AIMING_DOWN, timer = 32,
            swoop_angle = 0);
    }
}

pub(super) fn swoop_stage<L: Host + ?Sized>(
    game: &mut L,
    angle_step: u16,
    target_angle: u16,
    target_speed: u16,
) -> bool {
    if session_slot!(game, Ridley[RIDLEY].facing) != 0 {
        calculate_swoop_velocities(angle_step, target_angle, target_speed, game);
    } else {
        calculate_swoop_velocities(
            angle_step.wrapping_neg(),
            target_angle.wrapping_neg(),
            target_speed,
            game,
        );
    }
    let timer = session_slot!(game, Ridley[RIDLEY].timer);
    if timer != 0 {
        session_slot!(game, Ridley[RIDLEY].timer = timer.wrapping_sub(1));
        false
    } else {
        true
    }
}

pub fn swoop_aiming_down<L: Host + ?Sized>(game: &mut L) {
    if swoop_stage(game, 32, 512, 1152) {
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::SWOOP_AIMING_HORIZONTAL
        );
        session_slot!(game, Ridley[RIDLEY].timer = 20);
    }
}

pub fn swoop_aiming_horizontal<L: Host + ?Sized>(game: &mut L) {
    if swoop_stage(game, 320, 0x4000, 1280) {
        session_slot!(game, Ridley[RIDLEY]; function = function::SWOOP_AIMING_UP, timer = 16);
        bg2_staging::set(game, tail::WHIP_REQUEST, 1);
    }
}

pub fn swoop_aiming_up<L: Host + ?Sized>(game: &mut L) {
    if swoop_stage(game, 512, 30720, 768) {
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::SWOOP_AIMING_UP_FASTER
        );
        session_slot!(game, Ridley[RIDLEY].timer = 32);
    }
}

pub fn swoop_aiming_up_faster<L: Host + ?Sized>(game: &mut L) {
    if swoop_stage(game, 1024, 30720, 768) {
        session_slot!(game, Ridley[RIDLEY]; function = function::SWOOP_DECELERATE, timer = 32);
        face_room_middle(game);
    }
}

pub fn swoop_decelerate<L: Host + ?Sized>(game: &mut L) {
    calculate_swoop_velocities(0, 0x8000, 448, game);
    let timer = session_slot!(game, Ridley[RIDLEY].timer);
    if timer != 0 {
        session_slot!(game, Ridley[RIDLEY].timer = timer.wrapping_sub(1));
    } else {
        let next = if samus_not_spinning(game) {
            function::LUNGE
        } else {
            function::DECIDE_ACTION
        };
        session_slot!(game, Ridley[RIDLEY].function = next);
    }
}

pub fn hover<L: Host + ?Sized>(game: &mut L) {
    bg2_staging::set(game, tail::IDEAL_SEGMENT_ANGLE, 11);
    bg2_staging::set(game, tail::EXTENSION_SPEED, 384);
    session_slot!(
        game,
        Ridley[RIDLEY].function = function::HOVER_TO_WALL_BEHIND
    );
    session_slot!(
        game,
        Ridley[RIDLEY].function_timer = ((game.session().timing.random_number & 0x1f) + 32)
    );
    hover_to_wall_behind(game);
}

pub fn hover_to_wall_behind<L: Host + ?Sized>(game: &mut L) {
    let x = game.ridley_hover_behind(session_slot!(game, Ridley[RIDLEY].facing) as isize);
    hover_movement(game, x, function::HOVER_TO_WALL_IN_FRONT);
}

pub fn hover_to_wall_in_front<L: Host + ?Sized>(game: &mut L) {
    let x = game.ridley_hover_in_front(session_slot!(game, Ridley[RIDLEY].facing) as isize);
    hover_movement(game, x, function::HOVER_TO_WALL_BEHIND);
}

pub(super) fn hover_movement<L: Host + ?Sized>(game: &mut L, x: u16, timeout: u16) {
    if hover_towards(x, game) {
        set_up_pogo_from_hover(game);
    } else if timer_expired(game) {
        session_slot!(game, Ridley[RIDLEY]; function = timeout, function_timer = 128);
        face_room_middle(game);
    }
}

pub fn hover_towards<L: Host + ?Sized>(x: u16, game: &mut L) -> bool {
    let mut y = game.session().player.y;
    if !below(game.session().player.y, 352) {
        y = 352;
    }
    accelerate(0, acceleration_factor(game), x, y, game);
    bg2_staging::set(game, tail::WHIP_REQUEST, 1);
    if game.session().player.movement_and_facing.to_le_bytes()[1] != SPIN_JUMPING {
        return true;
    }
    if (game.session().timing.random_number as u8) >= 0x80
        && session_slot!(game, Ridley[RIDLEY].roaring) == 0
        && session_slot!(game, Ridley[RIDLEY].facing) != FACING_FORWARD
    {
        set_instruction_list(instruction_list::SHOOT_FIREBALLS, game);
    }
    false
}

pub fn set_up_pogo_from_hover<L: Host + ?Sized>(game: &mut L) {
    bg2_staging::set(game, tail::EXTENSION_SPEED, 240);
    bg2_staging::set(game, tail::IDEAL_SEGMENT_ANGLE, 16);
    bg2_staging::set(game, tail::FUNCTION, 1);
    session_slot!(
        game,
        Ridley[RIDLEY].function = function::POGO_FLY_TO_POSITION
    );
}

pub fn pogo_fly_to_position<L: Host + ?Sized>(game: &mut L) {
    if below(session_slot!(game, Ridley[RIDLEY].base.y_pos), 288) {
        face_room_middle(game);
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::POGO_WAIT_AT_POSITION
        );
        session_slot!(game, Ridley[RIDLEY].function_timer = 32);
        pogo_wait_at_position(game);
    } else {
        let x = game.ridley_pogo_target(session_slot!(game, Ridley[RIDLEY].facing) as isize);
        accelerate(0, 0, x, 288, game);
    }
}

pub fn pogo_wait_at_position<L: Host + ?Sized>(game: &mut L) {
    accelerate(
        0,
        0,
        session_slot!(game, Ridley[RIDLEY].base.x_pos),
        288,
        game,
    );
    if timer_expired(game) {
        set_up_pogo_tail(game);
        crate::actors::ridley_tables::owned::set_pogo_speeds(game);
        session_slot!(game, Ridley[RIDLEY].function = function::POGO_DESCENDING);
        session_slot!(
            game,
            Ridley[RIDLEY].function_timer = ((game.session().timing.random_number & 0x3f) + 128)
        );
    }
}

pub fn pogo_descending<L: Host + ?Sized>(game: &mut L) {
    if samus_grabbable(game) {
        set_neutral_tail(game);
        grab_samus_from_pogo(game);
        return;
    }
    let mut y_velocity = (bg2_staging::get(game, tail::POGO_DOWN_ACCELERATION) as i16)
        .wrapping_add(session_slot!(game, Ridley[RIDLEY].y_velocity) as i16);
    if !below(y_velocity as u16, 1536) {
        y_velocity = 1536;
    }
    session_slot!(game, Ridley[RIDLEY].y_velocity = y_velocity as u16);
    if tail_hits_floor(game) {
        let x = bg2_staging::get(game, tail::x(6));
        let y = bg2_staging::get(game, tail::y(6)).wrapping_add(12);
        game.place_projectile_spawn(x, y);
        game.spawn_room_projectile(DUST_CLOUD, 9);
        game.queue_sound(1, 0x76, 6);
        {
            game.session_mut().feedback.earthquake_kind = 13;
            game.session_mut().feedback.earthquake_timer = 4;
        };
        set_pogo_x_direction(game);
        crate::actors::ridley_tables::owned::set_pogo_speeds(game);
        crate::actors::ridley_tables::owned::set_tail_segments_from_list(
            tail_address(tail::distance(0)),
            tail_data::DISTANCES,
            game,
        );
        for segment in 0..7 {
            bg2_staging::set(game, tail::target_distance(segment), 3072);
        }
        bg2_staging::set(game, tail::FUNCTION, 4);
        let mut bounces = session_slot!(game, Ridley[RIDLEY].pogo_bounces).wrapping_add(1);
        if !below(session_slot!(game, Ridley[RIDLEY].pogo_bounces), 1) {
            if session_slot!(game, Ridley[RIDLEY].facing) != FACING_FORWARD {
                set_instruction_list(instruction_list::SHOOT_FIREBALLS, game);
            }
            bounces = 0;
        }
        session_slot!(game, Ridley[RIDLEY]; pogo_bounces = bounces,
            function = function::POGO_ASCENDING);
    }
}

pub fn pogo_ascending<L: Host + ?Sized>(game: &mut L) {
    if !samus_in_pogo_zone(game) || {
        let expired =
            (session_slot!(game, Ridley[RIDLEY].function_timer) as i16).wrapping_sub(1) < 0;
        session_slot!(game, Ridley[RIDLEY].function_timer, |function_timer| {
            function_timer.wrapping_sub(1)
        });
        expired
    } {
        set_neutral_tail(game);
        session_slot!(game, Ridley[RIDLEY].function = function::DECIDE_ACTION);
    } else {
        let y_velocity = session_slot!(game, Ridley[RIDLEY].y_velocity);
        let rising = (bg2_staging::get(game, tail::POGO_UP_ACCELERATION) as i16)
            .wrapping_add(y_velocity as i16)
            < 0;
        session_slot!(
            game,
            Ridley[RIDLEY].y_velocity =
                bg2_staging::get(game, tail::POGO_UP_ACCELERATION).wrapping_add(y_velocity)
        );
        if !rising {
            session_slot!(game, Ridley[RIDLEY]; y_velocity = 0,
                function = function::POGO_DESCENDING);
        }
    }
}

pub fn tail_hits_floor<L: Host + ?Sized>(game: &mut L) -> bool {
    [(6, 16), (5, 18), (4, 18), (3, 18), (2, 18)]
        .into_iter()
        .any(|(segment, below)| {
            let x = bg2_staging::get(game, tail::x(segment));
            let y = bg2_staging::get(game, tail::y(segment)).wrapping_add(below);
            solid_block_at(x, y, game)
        })
}

pub fn set_neutral_tail<S: Host + ?Sized>(game: &mut S) {
    bg2_staging::set(game, tail::FUNCTION, 1);
    bg2_staging::set(game, tail::ANGLE_DELTA, 1);
}

pub fn samus_grabbable<L: Host + ?Sized>(game: &mut L) -> bool {
    samus_not_spinning(game) && samus_touches_feet(4, 4, game)
}

pub fn samus_touches_feet<L: Host + ?Sized>(x_radius: u16, y_radius: u16, game: &mut L) -> bool {
    let rect = Rect {
        x: session_slot!(game, Ridley[RIDLEY].base.x_pos)
            .wrapping_add(game.ridley_feet_x(session_slot!(game, Ridley[RIDLEY].facing) as isize)),
        y: session_slot!(game, Ridley[RIDLEY].base.y_pos).wrapping_add(game.ridley_feet_y(
            (session_slot!(game, Ridley[RIDLEY].feet_distance_index) >> 1) as isize,
        )),
        w: x_radius,
        h: y_radius,
    };
    samus_in_rect(rect, game)
}

pub fn grab_samus_from_pogo<L: Host + ?Sized>(game: &mut L) {
    let mut y_velocity = session_slot!(game, Ridley[RIDLEY].y_velocity) as i16;
    if y_velocity >= 0 {
        y_velocity = -(y_velocity as i32) as i16;
    }
    if (y_velocity as i32 + 512) & 0x8000 == 0 {
        y_velocity = -512;
    }
    session_slot!(game, Ridley[RIDLEY].y_velocity = y_velocity as u16);
    set_neutral_tail(game);
    session_slot!(game, Ridley[RIDLEY].function = function::GRABBED_SAMUS);
    grabbed_samus(game);
}

pub fn set_pogo_x_direction<L: Host + ?Sized>(game: &mut L) {
    if session_slot!(game, Ridley[RIDLEY].x_velocity) == 0 {
        let x_velocity = if session_slot!(game, Ridley[RIDLEY].base.x_pos) & 0x80 != 0 {
            -0xc0i32
        } else {
            0xc0
        };
        session_slot!(game, Ridley[RIDLEY].x_velocity = x_velocity as u16);
    }
    let facing_away = ((session_slot!(game, Ridley[RIDLEY].x_velocity) as i32
        ^ (session_slot!(game, Ridley[RIDLEY].base.x_pos) as i32
            - game.session().player.x as i32))
        & 0x8000)
        == 0;
    let x = session_slot!(game, Ridley[RIDLEY].base.x_pos);
    if (x as i16).wrapping_sub(session_slot!(game, Ridley[RIDLEY].min_x) as i16) < 0 {
        if (session_slot!(game, Ridley[RIDLEY].x_velocity) & 0x8000) == 0 {
            return;
        }
    } else if (x as i16).wrapping_sub(session_slot!(game, Ridley[RIDLEY].max_x) as i16) >= 0 {
        if (session_slot!(game, Ridley[RIDLEY].x_velocity) & 0x8000) != 0 {
            return;
        }
    } else if facing_away == (game.session().timing.random_number < 0x555) {
        return;
    }
    session_slot!(game, Ridley[RIDLEY].x_velocity, |old| -(old as i32) as u16);
}

pub fn move_samus_with_feet<L: Host + ?Sized>(game: &mut L) {
    if let Some(offset) = settle_offset(session_slot!(game, Ridley[RIDLEY].grabbed_x_offset)) {
        session_slot!(game, Ridley[RIDLEY].grabbed_x_offset = offset);
    }
    if let Some(offset) = settle_offset(session_slot!(game, Ridley[RIDLEY].grabbed_y_offset)) {
        session_slot!(game, Ridley[RIDLEY].grabbed_y_offset = offset);
    }
    {
        game.session_mut().player.x = (session_slot!(game, Ridley[RIDLEY].grabbed_x_offset) as i32
            + session_slot!(game, Ridley[RIDLEY].base.x_pos) as i32
            + game.ridley_feet_x(session_slot!(game, Ridley[RIDLEY].facing) as isize) as i32)
            as u16;
        game.session_mut().player.y = (session_slot!(game, Ridley[RIDLEY].grabbed_y_offset) as i32
            + session_slot!(game, Ridley[RIDLEY].base.y_pos) as i32
            + game.ridley_feet_y(
                (session_slot!(game, Ridley[RIDLEY].feet_distance_index) >> 1) as isize,
            ) as i32) as u16;
    };
}

pub(super) fn settle_offset(offset: u16) -> Option<u16> {
    if offset == 0 {
        return None;
    }
    let magnitude = (abs16(offset) as i16).wrapping_sub(4);
    Some(if magnitude < 0 {
        0
    } else if offset & 0x8000 != 0 {
        -(magnitude as i32) as u16
    } else {
        magnitude as u16
    })
}

pub fn set_grabbed_samus_offset<L: Host + ?Sized>(game: &mut L) {
    session_slot!(
        game,
        Ridley[RIDLEY].grabbed_x_offset = (game.session().player.x as i32
            - (session_slot!(game, Ridley[RIDLEY].base.x_pos) as i32
                + game.ridley_feet_x(session_slot!(game, Ridley[RIDLEY].facing) as isize) as i32))
            as u16
    );
    session_slot!(
        game,
        Ridley[RIDLEY].grabbed_y_offset = (game.session().player.y as i32
            - (session_slot!(game, Ridley[RIDLEY].base.y_pos) as i32
                + game.ridley_feet_y(
                    (session_slot!(game, Ridley[RIDLEY].feet_distance_index) >> 1) as isize
                ) as i32)) as u16
    );
}

pub fn lunge_missed_samus<L: Host + ?Sized>(game: &mut L) {
    if below(session_slot!(game, Ridley[RIDLEY].death_lunges), 10) {
        if game.session().projectiles.power_bomb.flag != 0 {
            session_slot!(game, Ridley[RIDLEY].function = function::DODGE_POWER_BOMB);
            dodge_power_bomb(game);
        } else {
            session_slot!(game, Ridley[RIDLEY].function = function::LUNGE_MISSED);
            bg2_staging::set(game, tail::WHIP_REQUEST, 1);
        }
    } else {
        start_death_sequence(game);
        session_slot!(game, Ridley[RIDLEY].function = function::DEATH_ROAR);
        death_roar(game);
    }
}

pub fn lunge<L: Host + ?Sized>(game: &mut L) {
    if !(samus_not_spinning(game)
        && session_slot!(game, Ridley[RIDLEY].hit_boundary) == 0
        && {
            let side = game.ridley_lunge_side(session_slot!(game, Ridley[RIDLEY].facing) as isize);
            let dx = session_slot!(game, Ridley[RIDLEY].base.x_pos)
                .wrapping_sub(game.session().player.x);
            ((side ^ dx) & 0x8000) == 0 || below(abs16(dx), 32)
        }
        && ((session_slot!(game, Ridley[RIDLEY].base.y_pos) as i32 + 35
            - game.session().player.y as i32) as i16)
            < 0)
    {
        lunge_missed_samus(game);
        return;
    }
    let lead: i16 = if session_slot!(game, Ridley[RIDLEY].facing) != 0 {
        16
    } else {
        -16
    };
    accelerate(
        0,
        game.ridley_lunge_acceleration(
            session_slot!(game, Ridley[RIDLEY].acceleration_index) as isize
        ),
        game.session().player.x.wrapping_add(lead as u16),
        game.session().player.y.wrapping_sub(4),
        game,
    );
    let rect = Rect {
        x: session_slot!(game, Ridley[RIDLEY].base.x_pos)
            .wrapping_add(game.ridley_feet_x(session_slot!(game, Ridley[RIDLEY].facing) as isize)),
        y: session_slot!(game, Ridley[RIDLEY].base.y_pos).wrapping_add(35),
        w: 8,
        h: 12,
    };
    if samus_in_rect(rect, game) {
        session_slot!(game, Ridley[RIDLEY].y_velocity, |old| -(old as i32) as u16);
        if session_slot!(game, Ridley[RIDLEY].base.health) != 0 {
            if game.session().projectiles.power_bomb.flag != 0 {
                grab_samus(game);
                session_slot!(game, Ridley[RIDLEY].function = function::DODGE_POWER_BOMB);
                dodge_power_bomb(game);
            } else {
                grabbed_samus(game);
            }
        } else {
            if session_slot!(game, Ridley[RIDLEY].holding_samus) == 0 {
                grab_samus(game);
            }
            start_death_sequence(game);
            session_slot!(
                game,
                Ridley[RIDLEY].function = function::DYING_GRABBED_SAMUS
            );
            dying_grabbed_samus(game);
        }
    }
}

pub fn grabbed_samus<L: Host + ?Sized>(game: &mut L) {
    /// Ridley lunge target X position ($A6:BBEB), by facing direction.
    const TARGET_X: [u16; 3] = [0x40, 0, 0xd0];
    session_slot!(
        game,
        Ridley[RIDLEY].target_x = TARGET_X[session_slot!(game, Ridley[RIDLEY].facing) as usize]
    );
    let mut y = (session_slot!(game, Ridley[RIDLEY].base.y_pos) as i16).wrapping_sub(64);
    if below(session_slot!(game, Ridley[RIDLEY].base.y_pos), 320) {
        y = 256;
    }
    session_slot!(game, Ridley[RIDLEY].target_y = y as u16);
    if session_slot!(game, Ridley[RIDLEY].holding_samus) == 0 {
        grab_samus(game);
    }
    session_slot!(
        game,
        Ridley[RIDLEY].function = function::GRABBED_SAMUS_RISING
    );
    session_slot!(game, Ridley[RIDLEY].function_timer = 32);
    grabbed_samus_rising(game);
}

pub fn grabbed_samus_rising<L: Host + ?Sized>(game: &mut L) {
    let x = session_slot!(game, Ridley[RIDLEY].target_x);
    let y = session_slot!(game, Ridley[RIDLEY].target_y);
    accelerate(0, 0, x, y, game);
    if timer_expired(game) {
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::GRABBED_SAMUS_RISING_TO_MIDDLE
        );
        session_slot!(game, Ridley[RIDLEY].function_timer = 32);
    }
}

pub fn grabbed_samus_rising_to_middle<L: Host + ?Sized>(game: &mut L) {
    if timer_expired(game) {
        bg2_staging::set(game, tail::IDEAL_SEGMENT_ANGLE, 8);
        bg2_staging::set(game, tail::EXTENSION_SPEED, 240);
        release_samus(game);
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::GRABBED_SAMUS_RELEASE
        );
        session_slot!(game, Ridley[RIDLEY].function_timer = 64);
    } else {
        accelerate(
            0,
            0,
            session_slot!(game, Ridley[RIDLEY].target_x),
            256,
            game,
        );
    }
}

pub fn grabbed_samus_release<L: Host + ?Sized>(game: &mut L) {
    /// The X position Ridley flies to after releasing Samus ($A6:BC62), by
    /// facing direction.
    const TARGET_X: [u16; 3] = [0xb0, 0, 0x50];
    if timer_expired(game) {
        bg2_staging::set(game, tail::IDEAL_SEGMENT_ANGLE, 16);
        bg2_staging::set(game, tail::EXTENSION_SPEED, 240);
        session_slot!(game, Ridley[RIDLEY].function = function::DECIDE_ACTION);
    } else {
        let x = TARGET_X[session_slot!(game, Ridley[RIDLEY].facing) as usize];
        accelerate(0, 0, x, 224, game);
    }
}

pub fn draw_feet<L: Host + ?Sized>(clenched: bool, game: &mut L) {
    let tiles = if clenched {
        FEET_CLENCHED_TILES
    } else {
        FEET_UNCLENCHED_TILES
    };
    let tiles = game.actor_record(0xa6, tiles);
    game.queue_tile_pair([tiles.word(0), tiles.word(2)], [31424, 31680], 128);
}

pub fn grab_samus<L: Host + ?Sized>(game: &mut L) {
    set_grabbed_samus_offset(game);
    session_slot!(game, Ridley[RIDLEY].base.properties, |old| old
        | K_ENEMY_PROPS_TANGIBLE as u16);
    session_slot!(game, Ridley[RIDLEY].holding_samus = 1);
    game.command_player(LOCK_SAMUS);
    draw_feet(true, game);
}

pub fn release_samus<L: Host + ?Sized>(game: &mut L) {
    bg2_staging::set(game, tail::WHIP_REQUEST, 1);
    bg2_staging::set(game, tail::FUNCTION, 1);
    if (session_slot!(game, Ridley[RIDLEY].fight_mode) & 0x8000) == 0 {
        let timer = if samus_morphed(game) { 6 } else { 10 };
        session_slot!(game, Ridley[RIDLEY].intangibility_timer = timer);
    }
    session_slot!(game, Ridley[RIDLEY].holding_samus = 0);
    game.command_player(UNLOCK_SAMUS);
    draw_feet(false, game);
}

pub fn set_tangibility<S: Host + ?Sized>(game: &mut S) {
    let mode = session_slot!(game, Ridley[RIDLEY].fight_mode) as i16;
    if mode < 0 {
        return;
    }
    if mode != 0 {
        if off_screen(game) {
            set_tangible(game, false);
            return;
        }
        set_tangible(game, true);
    }
    let timer = session_slot!(game, Ridley[RIDLEY].intangibility_timer);
    if timer != 0 {
        let timer = timer.wrapping_sub(1);
        session_slot!(game, Ridley[RIDLEY].intangibility_timer = timer);
        if timer == 0 && (session_slot!(game, Ridley[RIDLEY].fight_mode) & 0x8000) == 0 {
            set_tangible(game, true);
        }
    }
}

pub(super) fn set_tangible<S: Host + ?Sized>(game: &mut S, tangible: bool) {
    let properties = session_slot!(game, Ridley[RIDLEY].base.properties);
    let properties = if tangible {
        (properties as i32 & !(K_ENEMY_PROPS_TANGIBLE as i32)) as u16
    } else {
        properties | K_ENEMY_PROPS_TANGIBLE as u16
    };
    session_slot!(game, Ridley[RIDLEY].base.properties = properties);
}

pub fn samus_not_spinning<L: Host + ?Sized>(game: &mut L) -> bool {
    MOVEMENT_TYPE_FLAGS[game.session().player.movement_and_facing.to_le_bytes()[1] as usize] & 0x80
        != 0
}

pub fn samus_morphed<L: Host + ?Sized>(game: &mut L) -> bool {
    MOVEMENT_TYPE_FLAGS[game.session().player.movement_and_facing.to_le_bytes()[1] as usize] & 0x40
        != 0
}

pub fn samus_in_pogo_zone<L: Host + ?Sized>(game: &mut L) -> bool {
    ((game.session().player.y as i32 - 352) & 0x8000) == 0
}

pub fn lunge_if_power_bomb<S: Host + ?Sized>(game: &mut S) {
    let mode = session_slot!(game, Ridley[RIDLEY].fight_mode) as i16;
    if mode > 0
        && mode != 2
        && game.session().projectiles.power_bomb.flag != 0
        && session_slot!(game, Ridley[RIDLEY].holding_samus) == 0
    {
        set_neutral_tail(game);
        session_slot!(game, Ridley[RIDLEY].function = function::LUNGE);
    }
}

pub fn dodge_power_bomb<L: Host + ?Sized>(game: &mut L) {
    if game.session().projectiles.power_bomb.flag != 0 {
        session_slot!(game, Ridley[RIDLEY].fight_mode = 2);
        let x = if below(game.session().projectiles.power_bomb.x, 128) {
            192
        } else {
            80
        };
        let y = if below(game.session().projectiles.power_bomb.y, 256) {
            384
        } else {
            192
        };
        accelerate(0, acceleration_factor(game), x, y, game);
    } else {
        session_slot!(game, Ridley[RIDLEY].fight_mode = 1);
        let next = if session_slot!(game, Ridley[RIDLEY].holding_samus) != 0 {
            function::GRABBED_SAMUS
        } else {
            function::DECIDE_ACTION
        };
        session_slot!(game, Ridley[RIDLEY].function = next);
    }
}

pub fn fake_retreat_move_to_position<L: Host + ?Sized>(game: &mut L) {
    ceres_accelerate(0, 1, 192, 128, game);
    if !below(session_slot!(game, Ridley[RIDLEY].base.x_pos), 192) {
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::FAKE_RETREAT_RISING
        );
    }
}

pub fn fake_retreat_rising<L: Host + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY].min_y = -192i32 as u16);
    ceres_accelerate(0, 1, 192, -128i32 as u16, game);
    if below(session_slot!(game, Ridley[RIDLEY].base.y_pos), 32) {
        session_slot!(
            game,
            Ridley[RIDLEY].baby_metroid_function = baby_metroid_function::START_FALLING
        );
        session_slot!(game, Ridley[RIDLEY]; function = function::DROP_BABY_METROID,
            function_timer = 21);
    }
}

pub fn drop_baby_metroid<L: Host + ?Sized>(game: &mut L) {
    if timer_expired(game) {
        set_instruction_list(instruction_list::RETRIEVE_BABY_METROID, game);
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::RETRIEVE_BABY_METROID
        );
        retrieve_baby_metroid(game);
    }
}

pub fn retrieve_baby_metroid<L: Host + ?Sized>(game: &mut L) {
    let x = session_slot!(game, Ridley[RIDLEY].baby_metroid_x).wrapping_sub(10);
    let y = session_slot!(game, Ridley[RIDLEY].baby_metroid_y).wrapping_sub(56);
    accelerate(0, 12, x, y, game);
    let rect = Rect {
        x: session_slot!(game, Ridley[RIDLEY].base.x_pos).wrapping_add(14),
        y: session_slot!(game, Ridley[RIDLEY].base.y_pos).wrapping_add(66),
        w: 4,
        h: 4,
    };
    if baby_metroid_in_rect(rect, game) {
        session_slot!(game, Ridley[RIDLEY].holding_samus = 1);
        draw_feet(true, game);
        session_slot!(game, Ridley[RIDLEY].y_velocity = -512i32 as u16);
        session_slot!(
            game,
            Ridley[RIDLEY].baby_metroid_function = baby_metroid_function::HELD_BY_FEET
        );
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::CERES_REAL_RETREAT_RISING
        );
    }
}

pub fn baby_metroid_in_rect<L: Host + ?Sized>(rect: Rect, game: &mut L) -> bool {
    let dx = abs16(session_slot!(game, Ridley[RIDLEY].baby_metroid_x).wrapping_sub(rect.x));
    if dx < 4 || dx.wrapping_sub(4) < rect.w {
        let dy = abs16(session_slot!(game, Ridley[RIDLEY].baby_metroid_y).wrapping_sub(rect.y));
        if dy < 4 || dy.wrapping_sub(4) < rect.h {
            return true;
        }
    }
    false
}

pub(super) fn run_baby_metroid_instruction<L: Host + ?Sized>(
    game: &mut L,
    instruction: u16,
    arguments: u16,
) -> u16 {
    match instruction {
        baby_metroid_instruction::GOTO_IF_STILL => baby_metroid_goto_if_still(arguments, game),
        baby_metroid_instruction::LOAD_PALETTE => game.baby_metroid_load_palette(arguments),
        baby_metroid_instruction::GOTO_IF_MOVING => baby_metroid_goto_if_moving(arguments, game),
        baby_metroid_instruction::GOTO => baby_metroid_goto(arguments, game),
        _ => game.unreachable(),
    }
}

pub fn instruction<L: Host + ?Sized>(
    ea: u32,
    script: EnemyScript,
    game: &mut L,
) -> Result<EnemyScript, EnemyScript> {
    if ea >> 16 != 0xa6 {
        return Err(script);
    }
    Ok(match ea as u16 {
        instruction_id::ROAR => roar(script, game),
        instruction_id::STOP_ROARING => stop_roaring(script, game),
        instruction_id::GOTO_IF_SAMUS_HEALTH_LOW => goto_if_samus_health_low(script, game),
        instruction_id::GOTO_BY_HOLDING_SAMUS => goto_by_holding_samus(script, game),
        instruction_id::GOTO_IF_NOT_HOLDING_SAMUS => goto_if_not_holding_samus(script, game),
        instruction_id::SET_FEET_DISTANCE => set_feet_distance(script, game),
        instruction_id::GOTO_IF_NOT_FACING_LEFT => goto_if_not_facing_left(script, game),
        instruction_id::MOVE_BY => move_by(script, game),
        instruction_id::FLIP_LEFT => flip_left(script, game),
        instruction_id::FACE_FORWARD => face_forward(script, game),
        instruction_id::FLIP_RIGHT => flip_right(script, game),
        instruction_id::CALCULATE_FIREBALL_VELOCITIES => {
            calculate_fireball_velocities(script, game)
        }
        instruction_id::FIREBALL_WITH_AFTERBURN => fireball_with_afterburn(script, game),
        instruction_id::FIREBALL_WITHOUT_AFTERBURN => fireball_without_afterburn(script, game),
        instruction_id::START_CERES_LIFTOFF => start_ceres_liftoff(script, game),
        instruction_id::START_NORFAIR_LIFTOFF => start_norfair_liftoff(script, game),
        _ => return Err(script),
    })
}

pub fn run_baby_metroid_function<L: Host + ?Sized>(game: &mut L) {
    match session_slot!(game, Ridley[RIDLEY].baby_metroid_function) {
        baby_metroid_function::HELD => baby_metroid_held(game),
        baby_metroid_function::HELD_BY_FEET => baby_metroid_held_by_feet(game),
        baby_metroid_function::START_FALLING => baby_metroid_start_falling(game),
        baby_metroid_function::FALLING => baby_metroid_falling(game),
        baby_metroid_function::NOTHING => {}
        _ => game.unreachable(),
    }
}

pub fn baby_metroid_held<L: Host + ?Sized>(game: &mut L) {
    session_slot!(
        game,
        Ridley[RIDLEY].baby_metroid_x =
            session_slot!(game, Ridley[RIDLEY].base.x_pos).wrapping_sub(16)
    );
    session_slot!(
        game,
        Ridley[RIDLEY].baby_metroid_y =
            session_slot!(game, Ridley[RIDLEY].base.y_pos).wrapping_add(22)
    );
}

pub fn baby_metroid_held_by_feet<L: Host + ?Sized>(game: &mut L) {
    session_slot!(
        game,
        Ridley[RIDLEY].baby_metroid_x =
            session_slot!(game, Ridley[RIDLEY].base.x_pos).wrapping_add(14)
    );
    session_slot!(
        game,
        Ridley[RIDLEY].baby_metroid_y =
            session_slot!(game, Ridley[RIDLEY].base.y_pos).wrapping_add(66)
    );
}

pub fn baby_metroid_start_falling<L: Host + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY]; baby_metroid_unread = 0, baby_metroid_y_velocity = 0);
    session_slot!(
        game,
        Ridley[RIDLEY].baby_metroid_function = baby_metroid_function::FALLING
    );
    baby_metroid_falling(game);
}

pub fn baby_metroid_falling<L: Host + ?Sized>(game: &mut L) {
    let velocity = session_slot!(game, Ridley[RIDLEY].baby_metroid_y_velocity).wrapping_add(8);
    session_slot!(game, Ridley[RIDLEY].baby_metroid_y_velocity = velocity);
    let subpixel = session_slot!(byte game, Ridley[RIDLEY].baby_metroid_y_subpixel[1]) as i32
        + (velocity as i32 & 0xff);
    session_slot!(byte game, Ridley[RIDLEY].baby_metroid_y_subpixel[1] = subpixel as u8);
    session_slot!(game, Ridley[RIDLEY].baby_metroid_y, |old| (old as i32
        + ((velocity >> 8) as i8 as i32 + (subpixel >> 8)))
        as u16);
    if !below(session_slot!(game, Ridley[RIDLEY].baby_metroid_y), 192) {
        session_slot!(game, Ridley[RIDLEY].baby_metroid_y = 192);
        session_slot!(
            game,
            Ridley[RIDLEY].baby_metroid_function = baby_metroid_function::NOTHING
        );
    }
}

pub fn baby_metroid_goto_if_still<L: Host + ?Sized>(k: u16, game: &mut L) -> u16 {
    if session_slot!(game, Ridley[RIDLEY].baby_metroid_y_velocity) == 0
        && (game.session().timing.random_number & 1) != 0
    {
        return baby_metroid_goto(k, game);
    }
    game.queue_sound(2, 0x24, 6);
    k.wrapping_add(2)
}

pub fn baby_metroid_goto_if_moving<L: Host + ?Sized>(k: u16, game: &mut L) -> u16 {
    if session_slot!(game, Ridley[RIDLEY].baby_metroid_y_velocity) != 0 {
        baby_metroid_goto(k, game)
    } else {
        k.wrapping_add(2)
    }
}

pub fn baby_metroid_goto<L: Host + ?Sized>(k: u16, game: &mut L) -> u16 {
    game.actor_record(0xa6, k).word(0)
}

pub fn dying_grabbed_samus<L: Host + ?Sized>(game: &mut L) {
    if move_to_death_spot(game) {
        death_roar(game);
    }
}

pub fn death_roar<L: Host + ?Sized>(game: &mut L) {
    set_instruction_list(instruction_list::DEATH_ROAR, game);
    session_slot!(
        game,
        Ridley[RIDLEY].function = function::DEATH_MOVE_TO_DEATH_SPOT
    );
    session_slot!(game, Ridley[RIDLEY].function_timer = 32);
}

pub fn death_move_to_death_spot<L: Host + ?Sized>(game: &mut L) {
    move_to_death_spot(game);
    if timer_expired(game) {
        session_slot!(game, Ridley[RIDLEY]; x_velocity = 0, y_velocity = 0);
        game.move_liquid(528, 64, 1);
        session_slot!(game, Ridley[RIDLEY]; small_explosion_timer = 0, small_explosion_index = 0);
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::LOWER_ACID_AND_RELEASE_SAMUS
        );
        session_slot!(game, Ridley[RIDLEY].function_timer = 160);
        lower_acid_and_release_samus(game);
    }
}

pub fn lower_acid_and_release_samus<L: Host + ?Sized>(game: &mut L) {
    spawn_small_explosion(game);
    if timer_expired(game) {
        if session_slot!(game, Ridley[RIDLEY].holding_samus) != 0 {
            release_samus(game);
        }
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::START_FALLING_APART
        );
        session_slot!(game, Ridley[RIDLEY]; x_velocity = 0, y_velocity = 0);
        spawn_explosions(game);
    }
}

pub fn start_falling_apart<L: Host + ?Sized>(game: &mut L) {
    spawn_small_explosion(game);
    session_slot!(game, Ridley[RIDLEY].movement_enabled = 0);
    session_slot!(game, Ridley[RIDLEY].base.properties, |old| old
        | K_ENEMY_PROPS_INVISIBLE as u16);
    session_slot!(game, Ridley[RIDLEY]; function = function::FALLING_APART, function_timer = 32);
}

pub fn falling_apart<L: Host + ?Sized>(game: &mut L) {
    if timer_expired(game) {
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::STILL_FALLING_APART
        );
        session_slot!(game, Ridley[RIDLEY].function_timer = 256);
    }
}

pub fn still_falling_apart<L: Host + ?Sized>(game: &mut L) {
    if timer_expired(game) {
        // The drop routine keeps but never reads the X it is passed.
        game.set_boss_bit(1);
        game.scatter_enemy_drop(DropSource::Ridley, 0);
        game.queue_music_delayed(3);
        session_slot!(game, Ridley[RIDLEY].base.properties, |old| old
            | K_ENEMY_PROPS_DELETED as u16);
        session_slot!(game, Ridley[RIDLEY].function = function::DEAD);
    }
}

pub fn move_to_death_spot<L: Host + ?Sized>(game: &mut L) -> bool {
    let rect = Rect {
        x: 128,
        y: 328,
        w: 4,
        h: 4,
    };
    accelerate_with_force(0, 0, 0x10, rect.x, rect.y, game);
    !separated_from_rect(0, rect, game)
}

pub fn spawn_small_explosion<L: Host + ?Sized>(game: &mut L) {
    /// The explosion offsets ($A6:C66E): X and Y pairs.
    const OFFSETS: [i16; 20] = [
        -24, -24, -20, 20, 16, -30, 30, -3, 14, -13, -2, 18, -2, -32, -31, 8, -4, -10, 19, 19,
    ];
    let timer = (session_slot!(game, Ridley[RIDLEY].small_explosion_timer) as i16).wrapping_sub(1);
    if timer >= 0 {
        session_slot!(game, Ridley[RIDLEY].small_explosion_timer = timer as u16);
        return;
    }
    session_slot!(game, Ridley[RIDLEY].small_explosion_timer = 4);
    let mut index = session_slot!(game, Ridley[RIDLEY].small_explosion_index).wrapping_add(1);
    if !below(session_slot!(game, Ridley[RIDLEY].small_explosion_index), 9) {
        index = 0;
    }
    session_slot!(game, Ridley[RIDLEY].small_explosion_index = index);
    let pair = 2 * usize::from(index);
    let x = session_slot!(game, Ridley[RIDLEY].base.x_pos).wrapping_add(OFFSETS[pair] as u16);
    let y = session_slot!(game, Ridley[RIDLEY].base.y_pos).wrapping_add(OFFSETS[pair + 1] as u16);
    game.place_projectile_spawn(x, y);
    game.spawn_room_projectile(DUST_CLOUD, 3);
    game.queue_sound(1, 0x24, 3);
}

pub fn explosion_init<L: Host + ?Sized>(game: &mut L) {
    let e = current(game);
    session_slot!(game, Explosion[e].base.instruction_timer = 1);
    session_slot!(game, Explosion[e].base; timer = 0, vram_tiles_index = 0);
    session_slot!(game, Explosion[e].base.palette_index = 3584);
    let part = session_slot!(game, Explosion[e].part);
    session_slot!(
        game,
        Explosion[e].lifetime = game.explosion_lifetime((part >> 1) as isize)
    );
    let random = game.roll_random();
    let drift = if random & 0x8000 != 0 {
        -(random as i32 & 0x130)
    } else {
        random as i32 & 0x130
    };
    session_slot!(game, Explosion[e]; x_velocity = drift as u16, y_velocity = 0);
    match part {
        0 | 2 | 4 | 6 | 8 | 10 => {
            // Ridley explosion initialisation - index = 0..Ah (tail segments 0..5).
            let segment = usize::from(part / 2);
            const LISTS: [u16; 3] = [
                explosion_instruction_list::TAIL_SEGMENTS_0_1,
                explosion_instruction_list::TAIL_SEGMENTS_2_3,
                explosion_instruction_list::TAIL_SEGMENTS_4_5,
            ];
            session_slot!(
                game,
                Explosion[e].base.x_pos = bg2_staging::get(game, tail::x(segment))
            );
            session_slot!(
                game,
                Explosion[e].base.y_pos = bg2_staging::get(game, tail::y(segment))
            );
            session_slot!(
                game,
                Explosion[e].base.current_instruction = LISTS[segment / 2]
            );
        }
        12 => {
            // Ridley explosion initialisation - index = Ch (tail tip).
            session_slot!(
                game,
                Explosion[e].base.x_pos = bg2_staging::get(game, tail::x(6))
            );
            session_slot!(
                game,
                Explosion[e].base.y_pos = bg2_staging::get(game, tail::y(6))
            );
            let list = game.tail_tip_explosion_list(tail_tip_sector(game));
            session_slot!(game, Explosion[e].base.current_instruction = list);
        }
        14 => place_explosion(game, e, WINGS_ADDRESS, 0),
        16 => place_explosion(game, e, LEGS_ADDRESS, 22),
        18 => place_explosion(game, e, MOUTH_ADDRESS, -24),
        20 => place_explosion(game, e, TORSO_ADDRESS, 0),
        22 => place_explosion(game, e, HAND_ADDRESS, 7),
        _ => game.unreachable(),
    }
}

pub(super) fn place_explosion<L: Host + ?Sized>(game: &mut L, e: i32, part: u32, y_offset: i16) {
    let side: isize = if session_slot!(game, Ridley[RIDLEY].facing) != 0 {
        1
    } else {
        0
    };
    let x = session_slot!(game, Ridley[RIDLEY].base.x_pos)
        .wrapping_add(game.explosion_placement(part, side).x_offset);
    session_slot!(game, Explosion[e].base.x_pos = x);
    session_slot!(
        game,
        Explosion[e].base.y_pos =
            session_slot!(game, Ridley[RIDLEY].base.y_pos).wrapping_add(y_offset as u16)
    );
    session_slot!(
        game,
        Explosion[e].base.current_instruction =
            game.explosion_placement(part, side).instruction_list
    );
}

pub fn explosion_main<L: Host + ?Sized>(game: &mut L) {
    explosion_flicker(game.session().actors.controls.selected, game);
    let e = current(game);
    let mut speed = (abs16(session_slot!(game, Explosion[e].x_velocity)) as i16).wrapping_sub(4);
    if speed < 0 {
        speed = 0;
    }
    let x_velocity = if session_slot!(game, Explosion[e].x_velocity) & 0x8000 != 0 {
        -(speed as i32)
    } else {
        speed as i32
    };
    session_slot!(game, Explosion[e].x_velocity = x_velocity as u16);
    session_slot!(game, Explosion[e].y_velocity, |old| old.wrapping_add(4));
    move_with_velocity(game);
    session_slot!(game, Explosion[e].lifetime, |old| old.wrapping_sub(1));
    if (session_slot!(game, Explosion[e].lifetime) & 0x8000) != 0 {
        game.normal_death(0);
    }
}

pub fn explosion_flicker<L: Host + ?Sized>(k: u16, game: &mut L) {
    let e = i32::from(k);
    let odd = session_slot!(game, Explosion[e].base.frame_counter) & 1;
    session_slot!(game, Explosion[e].base.frame_counter = odd);
    let properties = if odd != 0 {
        session_slot!(game, Explosion[e].base.properties) | K_ENEMY_PROPS_INVISIBLE as u16
    } else {
        (session_slot!(game, Explosion[e].base.properties) as i32
            & !(K_ENEMY_PROPS_INVISIBLE as i32)) as u16
    };
    session_slot!(game, Explosion[e].base.properties = properties);
}

pub fn spawn_explosions<L: Host + ?Sized>(game: &mut L) {
    /// The explosion enemy populations ($A6:C987..CA37), in spawn order.
    const POPULATIONS: [u16; 12] = [
        0xc9e7, 0xc9d7, 0xc9c7, 0xc9b7, 0xc9a7, 0xc997, 0xc987, 0xc9f7, 0xca07, 0xca27, 0xca17,
        0xca37,
    ];
    for population in POPULATIONS {
        game.spawn_population(0xa6, population);
    }
}

pub fn handle_tail<L: Host + ?Sized>(game: &mut L) {
    tail_whip_sound(game);
    // Ridley tail function pointers ($A6:CB21), by tail function index.
    let functions: [fn(&mut L); 9] = [
        tail_inactive,
        neutral_tail,
        set_up_pogo_tail,
        point_tail_down,
        pogo_tail,
        pogo_stab_setup_tail,
        pogo_descending_stab,
        unused_pogo_descending_stab,
        spinny_tail_whips,
    ];
    functions[bg2_staging::get(game, tail::FUNCTION) as usize](&mut *game);
    calculate_tail_segment_positions(game);
    update_tail_segment_distances(game);
    if session_slot!(game, Ridley[RIDLEY].holding_samus)
        | (game.session().player.invincibility
            | session_slot!(game, Ridley[RIDLEY].base.properties) & K_ENEMY_PROPS_TANGIBLE as u16)
        == 0
    {
        tail_samus_collision(game);
    }
}

pub(super) fn tail_inactive<L: Host + ?Sized>(_game: &mut L) {}

pub fn set_up_pogo_tail<L: Host + ?Sized>(game: &mut L) {
    bg2_staging::set(game, tail::ANGLE_DELTA, 8);
    pogo_tail_pointed_down(game);
    bg2_staging::set(game, tail::FUNCTION, 3);
}

pub fn point_tail_down<L: Host + ?Sized>(game: &mut L) {
    bg2_staging::set(game, tail::ANGLE_DELTA, 8);
    pogo_tail_pointed_down(game);
}

pub fn pogo_descending_stab<L: Host + ?Sized>(game: &mut L) {
    bg2_staging::set(game, tail::ANGLE_DELTA, 3);
    pogo_tail_pointed_down(game);
    bg2_staging::set(game, tail::FUNCTION, 6);
}

pub fn unused_pogo_descending_stab<L: Host + ?Sized>(game: &mut L) {
    bg2_staging::set(game, tail::ANGLE_DELTA, 2);
    pogo_tail_pointed_down(game);
    bg2_staging::set(game, tail::FUNCTION, 7);
}

pub fn pogo_tail_pointed_down<L: Host + ?Sized>(game: &mut L) {
    if session_slot!(game, Ridley[RIDLEY].facing) != 0 {
        bg2_staging::set(game, tail::WHIP_TARGET_CLOCKWISE, 0x4000);
        bg2_staging::set(game, tail::WHIP_TARGET_ANTICLOCKWISE, NO_WHIP_TARGET);
    } else {
        bg2_staging::set(game, tail::WHIP_TARGET_ANTICLOCKWISE, 0x4000);
        bg2_staging::set(game, tail::WHIP_TARGET_CLOCKWISE, NO_WHIP_TARGET);
    }
    pogo_tail(game);
    if !any_tail_segment_active(game) {
        bg2_staging::set(game, tail::FUNCTION, 4);
    }
}

pub fn neutral_tail<L: Host + ?Sized>(game: &mut L) {
    handle_tail_whip(game);
    update_tail_segment_angles(game);
}

pub fn pogo_tail<L: Host + ?Sized>(game: &mut L) {
    handle_normal_pogo_tail(game);
    update_tail_segment_angles(game);
}

pub fn pogo_stab_setup_tail<L: Host + ?Sized>(game: &mut L) {
    handle_pogo_tail_stab(game);
    update_tail_segment_angles(game);
}

pub fn spinny_tail_whips<L: Host + ?Sized>(game: &mut L) {
    handle_spinning_tail_whip(game);
    update_tail_segment_angles(game);
}

pub fn update_tail_segment_angles<L: Host + ?Sized>(game: &mut L) {
    for segment in 0..7 {
        update_tail_segment_angle(segment, game);
    }
}

pub fn set_tail_angle_extrema<L: Host + ?Sized>(game: &mut L) {
    let limits = game.ridley_tail_angles(session_slot!(game, Ridley[RIDLEY].facing) as isize);
    bg2_staging::set(game, tail::MIN_CLOCKWISE_ANGLE, limits.min_clockwise);
    bg2_staging::set(
        game,
        tail::MAX_ANTICLOCKWISE_ANGLE,
        limits.max_anticlockwise,
    );
}

pub fn all_tail_segments_active<L: Host + ?Sized>(game: &mut L) -> bool {
    let mut flags = bg2_staging::get(game, tail::active(6));
    for segment in (0..6).rev() {
        flags &= bg2_staging::get(game, tail::active(segment));
    }
    flags != 0
}

pub(super) fn any_tail_segment_active<L: Host + ?Sized>(game: &mut L) -> bool {
    let mut flags = bg2_staging::get(game, tail::active(6));
    for segment in (0..6).rev() {
        flags |= bg2_staging::get(game, tail::active(segment));
    }
    flags != 0
}

pub fn handle_tail_whip<L: Host + ?Sized>(game: &mut L) {
    set_tail_angle_extrema(game);
    if all_tail_segments_active(game) && configure_tail_whip(game) {
        return;
    }
    swing_idle_tail(game);
}

pub(super) fn swing_idle_tail<L: Host + ?Sized>(game: &mut L) {
    if !any_tail_segment_active(game) {
        bg2_staging::set(game, tail::active(0), 0x8000);
        bg2_staging::set(game, tail::WHIP_TARGET_CLOCKWISE, NO_WHIP_TARGET);
        bg2_staging::set(game, tail::WHIP_TARGET_ANTICLOCKWISE, NO_WHIP_TARGET);
        let delta = if game.session().player.x < 0x70 { 1 } else { 2 };
        bg2_staging::set(game, tail::ANGLE_DELTA, delta);
    }
}

pub fn configure_tail_whip<L: Host + ?Sized>(game: &mut L) -> bool {
    let attack = if bg2_staging::get(game, tail::WHIP_REQUEST) != 0
        && (bg2_staging::get(game, tail::WHIP_TARGET_CLOCKWISE)
            & bg2_staging::get(game, tail::WHIP_TARGET_ANTICLOCKWISE)
            & 0x8000)
            != 0
    {
        Some(bg2_staging::get(game, tail::WHIP_REQUEST).wrapping_sub(1))
    } else if bg2_staging::get(game, tail::IDLE_WHIP) == 0 {
        None
    } else if (game.session().timing.random_number as u8) >= 0xf0 {
        Some(bg2_staging::get(game, tail::WHIP_REQUEST).wrapping_sub(1))
    } else if abs16(
        game.session()
            .player
            .x
            .wrapping_sub(session_slot!(game, Ridley[RIDLEY].base.x_pos)),
    ) < 0x80
    {
        Some(0)
    } else {
        None
    };
    let started = if let Some(attack) = attack {
        set_tail_whip_target(attack, game);
        true
    } else {
        false
    };
    bg2_staging::set(game, tail::WHIP_REQUEST, 0);
    started
}

pub fn handle_spinning_tail_whip<L: Host + ?Sized>(game: &mut L) {
    set_tail_angle_extrema(game);
    if all_tail_segments_active(game) && configure_spinning_tail_whip(game) {
        return;
    }
    swing_idle_tail(game);
}

pub fn configure_spinning_tail_whip<L: Host + ?Sized>(game: &mut L) -> bool {
    if bg2_staging::get(game, tail::WHIP_REQUEST) != 0
        && (bg2_staging::get(game, tail::WHIP_TARGET_CLOCKWISE)
            & bg2_staging::get(game, tail::WHIP_TARGET_ANTICLOCKWISE)
            & 0x8000)
            != 0
    {
        set_tail_whip_target(1, game);
        bg2_staging::set(game, tail::WHIP_REQUEST, 0);
        return true;
    }
    false
}

pub(super) fn pogo_tail_whips<L: Host + ?Sized>(game: &mut L) -> bool {
    set_tail_angle_extrema(game);
    if all_tail_segments_active(game)
        && ((game.session().timing.random_number as u8) >= 0xf0
            || abs16(
                game.session()
                    .player
                    .x
                    .wrapping_sub(session_slot!(game, Ridley[RIDLEY].base.x_pos)),
            ) < 0x80)
        && (bg2_staging::get(game, tail::WHIP_TARGET_CLOCKWISE)
            & bg2_staging::get(game, tail::WHIP_TARGET_ANTICLOCKWISE)
            & 0x8000)
            != 0
    {
        bg2_staging::set(game, tail::WHIP_TARGET_CLOCKWISE, 16128);
        bg2_staging::set(game, tail::ANGLE_DELTA, 8);
        return true;
    }
    false
}

pub fn handle_normal_pogo_tail<L: Host + ?Sized>(game: &mut L) {
    if pogo_tail_whips(game) || any_tail_segment_active(game) {
        return;
    }
    if (session_slot!(game, Ridley[RIDLEY].y_velocity) & 0x8000) == 0 {
        bg2_staging::set(game, tail::FUNCTION, 5);
    }
    bg2_staging::set(game, tail::WHIP_TARGET_CLOCKWISE, NO_WHIP_TARGET);
    bg2_staging::set(game, tail::WHIP_TARGET_ANTICLOCKWISE, NO_WHIP_TARGET);
    let mut timer = bg2_staging::get(game, tail::POGO_TAIL_TIMER);
    if bg2_staging::get(game, tail::POGO_TAIL_TIMER) == 0 || {
        timer = bg2_staging::get(game, tail::POGO_TAIL_TIMER).wrapping_sub(1);
        bg2_staging::set(game, tail::POGO_TAIL_TIMER, timer);
        bg2_staging::get(game, tail::POGO_TAIL_TIMER) != 0
    } {
        bg2_staging::set(game, tail::ANGLE_DELTA, timer);
    } else {
        bg2_staging::set(game, tail::active(0), 0x8000);
        set_tail_angle_high_bytes(game);
        bg2_staging::set(game, tail::ANGLE_DELTA, 8);
    }
}

pub fn handle_pogo_tail_stab<L: Host + ?Sized>(game: &mut L) {
    if pogo_tail_whips(game) || any_tail_segment_active(game) {
        return;
    }
    if (session_slot!(game, Ridley[RIDLEY].y_velocity) & 0x8000) == 0 {
        bg2_staging::set(game, tail::FUNCTION, 6);
        for segment in 0..7 {
            bg2_staging::set(game, tail::target_distance(segment), 2560);
        }
        for segment in 0..7 {
            bg2_staging::set(game, tail::angle(segment), 0x4000);
        }
        bg2_staging::set(game, tail::active(0), 0x8000);
    }
    bg2_staging::set(game, tail::WHIP_TARGET_CLOCKWISE, NO_WHIP_TARGET);
    bg2_staging::set(game, tail::WHIP_TARGET_ANTICLOCKWISE, NO_WHIP_TARGET);
    let mut timer = bg2_staging::get(game, tail::POGO_TAIL_TIMER);
    if bg2_staging::get(game, tail::POGO_TAIL_TIMER) != 0 {
        timer = bg2_staging::get(game, tail::POGO_TAIL_TIMER).wrapping_sub(1);
        bg2_staging::set(game, tail::POGO_TAIL_TIMER, timer);
        if timer == 0 {
            bg2_staging::set(game, tail::active(0), 0x8000);
            set_tail_angle_high_bytes(game);
            timer = 8;
        }
    }
    bg2_staging::set(game, tail::ANGLE_DELTA, timer);
}

pub fn set_tail_angle_high_bytes<L: Host + ?Sized>(game: &mut L) {
    for segment in 0..4 {
        let value = tail_low_byte(game, tail::angle(segment)) | 0x4000;
        set_tail_word(game, tail::angle(segment), value);
    }
    // Segment 4 deliberately takes segment 0's word, including its bit 10.
    let value = tail_word(game, tail::angle(0)) & 0x4ff | 0x4000;
    set_tail_word(game, tail::angle(4), value);
    for segment in 5..7 {
        let value = tail_low_byte(game, tail::angle(segment)) | 0x4000;
        set_tail_word(game, tail::angle(segment), value);
    }
}

pub fn calculate_tail_segment_positions<L: Host + ?Sized>(game: &mut L) {
    /// The tail base X offsets ($A6:CF54), by facing direction.
    const BASE_X_OFFSETS: [i16; 3] = [0x20, 0, -0x20];
    bg2_staging::set(
        game,
        tail::y(0),
        (bg2_staging::get(game, tail::y_offset(0)) as i32
            + session_slot!(game, Ridley[RIDLEY].base.y_pos) as i32
            + 16) as u16,
    );
    chain_tail_positions(game, tail::y(0));
    let mut x_offset = (session_slot!(game, Ridley[RIDLEY].facing) as i16).wrapping_sub(1);
    if session_slot!(game, Ridley[RIDLEY].facing) != FACING_FORWARD {
        x_offset = (bg2_staging::get(game, tail::x_offset(0)) as i16)
            .wrapping_add(BASE_X_OFFSETS[session_slot!(game, Ridley[RIDLEY].facing) as usize]);
    }
    bg2_staging::set(
        game,
        tail::x(0),
        session_slot!(game, Ridley[RIDLEY].base.x_pos).wrapping_add(x_offset as u16),
    );
    if session_slot!(game, Ridley[RIDLEY].facing) == FACING_FORWARD {
        bg2_staging::set(
            game,
            tail::x(1),
            session_slot!(game, Ridley[RIDLEY].base.x_pos),
        );
        for segment in 2..7 {
            let x = tail_word(game, tail::x(1));
            set_tail_word(game, tail::x(segment), x);
        }
    } else {
        chain_tail_positions(game, tail::x(0));
    }
}

pub(super) fn chain_tail_positions<L: Host + ?Sized>(game: &mut L, first: usize) {
    for index in (first + tail::SEGMENT..=first + 5 * tail::SEGMENT).step_by(2 * tail::SEGMENT) {
        let value = tail_word(game, index + 2).wrapping_add(tail_word(game, index - tail::SEGMENT));
        set_tail_word(game, index, value);
        let value = tail_word(game, index + tail::SEGMENT + 2)
            .wrapping_add(tail_word(game, index + 2))
            .wrapping_add(tail_word(game, index - tail::SEGMENT));
        set_tail_word(game, index + tail::SEGMENT, value);
    }
}

pub fn update_tail_segment_distances<L: Host + ?Sized>(game: &mut L) {
    const MAXIMA: [u16; 6] = [6144, 6144, 5632, 5632, 4608, 1280];
    const RESTING: [u16; 6] = [2048, 2048, 2048, 2048, 2048, 1280];
    for segment in 1..7 {
        update_tail_segment_distance(game, segment, MAXIMA[segment - 1], RESTING[segment - 1]);
    }
}

pub(super) fn update_tail_segment_distance<L: Host + ?Sized>(
    game: &mut L,
    segment: usize,
    maximum: u16,
    resting: u16,
) {
    let (distance, target) = (tail::distance(segment), tail::target_distance(segment));
    if tail_word(game, target) != 0 {
        if (tail_word(game, target) as i16).wrapping_sub(tail_word(game, distance) as i16) < 0 {
            set_tail_word(game, target, 0);
        }
        let speed = tail_word(game, tail::EXTENSION_SPEED);
        let mut value = speed.wrapping_add(tail_word(game, distance));
        if (tail_word(game, tail::EXTENSION_SPEED) as i32 + tail_word(game, distance) as i32
            - maximum as i32)
            & 0x8000
            == 0
        {
            value = maximum;
        }
        set_tail_word(game, distance, value);
    } else if (resting as i16).wrapping_sub(tail_word(game, distance) as i16) < 0 {
        let value = tail_word(game, distance).wrapping_sub(128);
        set_tail_word(game, distance, value);
    }
}

pub fn update_tail_segment_angle<L: Host + ?Sized>(segment: usize, game: &mut L) {
    let angle = if (bg2_staging::get(game, tail::active(segment)) & 0x8000) != 0 {
        let stagger = bg2_staging::get(game, tail::stagger_angle(segment));
        if stagger < bg2_staging::get(game, tail::IDEAL_SEGMENT_ANGLE) {
            bg2_staging::set(
                game,
                tail::stagger_angle(segment),
                bg2_staging::get(game, tail::stagger_angle(segment))
                    .wrapping_add(bg2_staging::get(game, tail::ANGLE_DELTA)),
            );
            return;
        }
        if stagger != 0xffff {
            // The next segment starts moving the same way.
            bg2_staging::set(game, tail::stagger_angle(segment), 0xffff);
            bg2_staging::set(game, tail::active(segment + 1), 0x8000);
            bg2_staging::set(
                game,
                tail::direction(segment + 1),
                bg2_staging::get(game, tail::direction(segment)),
            );
        }
        let next = 'advance: {
            if (bg2_staging::get(game, tail::direction(segment)) & 0x8000) == 0 {
                let (turn, angle) =
                    if (bg2_staging::get(game, tail::WHIP_TARGET_ANTICLOCKWISE) & 0x8000) != 0 {
                        let angle = bg2_staging::get(game, tail::ANGLE_DELTA)
                            .wrapping_add(bg2_staging::get(game, tail::angle(segment)));
                        let limit = bg2_staging::get(game, tail::MAX_ANTICLOCKWISE_ANGLE);
                        ((angle as i16).wrapping_sub(limit as i16) >= 0, angle)
                    } else {
                        bg2_staging::set(game, tail::target_distance(segment), 3072);
                        let angle = bg2_staging::get(game, tail::ANGLE_DELTA)
                            .wrapping_add(bg2_staging::get(game, tail::angle(segment)));
                        let target = bg2_staging::get(game, tail::WHIP_TARGET_ANTICLOCKWISE);
                        if (angle as i16).wrapping_sub(target as i16) >= 0 {
                            if stops_at_whip_target(game, segment) {
                                bg2_staging::set(
                                    game,
                                    tail::angle(segment),
                                    bg2_staging::get(game, tail::WHIP_TARGET_ANTICLOCKWISE),
                                );
                                break 'advance None;
                            }
                            (true, angle)
                        } else {
                            (false, angle)
                        }
                    };
                if turn {
                    bg2_staging::set(game, tail::direction(segment), 0x8000);
                    Some(bg2_staging::get(game, tail::MAX_ANTICLOCKWISE_ANGLE))
                } else {
                    Some(angle)
                }
            } else {
                let (turn, decreased) =
                    if (bg2_staging::get(game, tail::WHIP_TARGET_CLOCKWISE) & 0x8000) != 0 {
                        let decreased = (bg2_staging::get(game, tail::angle(segment)) as i32
                            - bg2_staging::get(game, tail::ANGLE_DELTA) as i32
                            - 1) as i16;
                        let limit = bg2_staging::get(game, tail::MIN_CLOCKWISE_ANGLE);
                        (decreased.wrapping_sub(limit as i16) < 0, decreased)
                    } else {
                        bg2_staging::set(game, tail::target_distance(segment), 3072);
                        let decreased = (bg2_staging::get(game, tail::angle(segment)) as i32
                            - bg2_staging::get(game, tail::ANGLE_DELTA) as i32
                            - 1) as i16;
                        let target = bg2_staging::get(game, tail::WHIP_TARGET_CLOCKWISE);
                        if decreased.wrapping_sub(target as i16) < 0 {
                            if stops_at_whip_target(game, segment) {
                                bg2_staging::set(
                                    game,
                                    tail::angle(segment),
                                    bg2_staging::get(game, tail::WHIP_TARGET_CLOCKWISE),
                                );
                                break 'advance None;
                            }
                            (true, decreased)
                        } else {
                            (false, decreased)
                        }
                    };
                if turn {
                    bg2_staging::set(game, tail::direction(segment), 0);
                    Some(bg2_staging::get(game, tail::MIN_CLOCKWISE_ANGLE))
                } else {
                    Some((decreased as u16).wrapping_add(1))
                }
            }
        };
        let Some(angle) = next else {
            bg2_staging::set(game, tail::active(segment), 0);
            bg2_staging::set(game, tail::stagger_angle(segment), 0);
            bg2_staging::set(
                game,
                tail::direction(segment),
                bg2_staging::get(game, tail::direction(segment)) ^ 0x8000u16,
            );
            return;
        };
        bg2_staging::set(game, tail::angle(segment), angle);
        angle
    } else {
        bg2_staging::get(game, tail::angle(segment))
    };
    // Segment angles add up along the tail, in their low bytes.
    let mut total = angle as u8 as u16;
    if segment != 0 {
        total = game
            .tail_byte(2 * tail::angle(segment - 1))
            .wrapping_add(total as u8) as u16;
    }
    let length = u16::from(game.tail_byte(2 * tail::distance(segment) + 1));
    let (x, y) = game.hardware_sine_cosine(length, total);
    bg2_staging::set(game, tail::x_offset(segment), x);
    bg2_staging::set(game, tail::y_offset(segment), y);
}

pub(super) fn stops_at_whip_target<L: Host + ?Sized>(game: &L, segment: usize) -> bool {
    segment == 0 || bg2_staging::get(game, tail::active(segment - 1)) == 0
}

pub fn set_tail_whip_target<L: Host + ?Sized>(attack: u16, game: &mut L) {
    // The attack's low byte becomes the target angle's high byte.
    bg2_staging::set(game, tail::WHIP_ANGLE_OFFSET, u16::from(attack as u8) << 8);
    if session_slot!(game, Ridley[RIDLEY].facing) != FACING_FORWARD
        && session_slot!(game, Ridley[RIDLEY].fight_mode) != 0
    {
        let (dx, dy) = if let Some(missile) = missile_near_tail(game) {
            (
                missile.x.wrapping_sub(bg2_staging::get(game, tail::x(0))),
                missile.y.wrapping_sub(bg2_staging::get(game, tail::y(0))),
            )
        } else {
            (
                game.session()
                    .player
                    .x
                    .wrapping_sub(bg2_staging::get(game, tail::x(0))),
                (game.session().player.y as i32 + 24 - bg2_staging::get(game, tail::y(0)) as i32)
                    as u16,
            )
        };
        let direction = -(angle_from_delta(dx, dy) as i32 + 0x80) as u8 as u16;
        if session_slot!(game, Ridley[RIDLEY].facing) != 0 {
            let mut reach = direction;
            if (0x18..0xe8).contains(&direction) {
                reach = 24;
            }
            let target = (bg2_staging::get(game, tail::WHIP_ANGLE_OFFSET) as i32
                + reach as i32
                + 0x4000) as u16;
            if target >= bg2_staging::get(game, tail::angle(0)) {
                bg2_staging::set(game, tail::WHIP_TARGET_ANTICLOCKWISE, target);
                bg2_staging::set(game, tail::ANGLE_DELTA, 8);
            }
        } else {
            let mut reach = direction;
            if (0x18..0xe8).contains(&direction) {
                reach = 232;
            }
            let target = (reach as i32 + 16128
                - bg2_staging::get(game, tail::WHIP_ANGLE_OFFSET) as i32)
                as u16;
            if target < bg2_staging::get(game, tail::angle(0)) {
                bg2_staging::set(game, tail::WHIP_TARGET_CLOCKWISE, target);
                bg2_staging::set(game, tail::ANGLE_DELTA, 8);
            }
        }
    }
}

pub fn missile_near_tail<L: Host + ?Sized>(game: &mut L) -> Option<Point> {
    let rect = Rect {
        x: bg2_staging::get(game, tail::x(6)),
        y: bg2_staging::get(game, tail::y(6)),
        w: 64,
        h: 64,
    };
    if game.session().projectiles.count == 0 {
        return None;
    }
    let found = (0..5).find(|&n| {
        let kind = projectile_kind(game, n);
        (kind == MISSILE || kind == SUPER_MISSILE) && projectile_overlaps(game, n, rect)
    })?;
    Some(Point {
        x: interaction::get_at(game, ProjectileField::X, found),
        y: interaction::get_at(game, ProjectileField::Y, found),
    })
}

pub(super) fn projectile_kind<L: Host + ?Sized>(game: &L, n: usize) -> u8 {
    game.projectile_field(n, ProjectileField::Kind)
        .to_le_bytes()[1]
        & 0xf
}

pub(super) fn projectile_overlaps<L: Host + ?Sized>(game: &L, n: usize, rect: Rect) -> bool {
    let dx = abs16(interaction::get_at(game, ProjectileField::X, n).wrapping_sub(rect.x));
    if dx < interaction::get_at(game, ProjectileField::XRadius, n)
        || dx.wrapping_sub(interaction::get_at(game, ProjectileField::XRadius, n)) < rect.w
    {
        let dy = abs16(interaction::get_at(game, ProjectileField::Y, n).wrapping_sub(rect.y));
        if dy < interaction::get_at(game, ProjectileField::YRadius, n)
            || dy.wrapping_sub(interaction::get_at(game, ProjectileField::YRadius, n)) < rect.h
        {
            return true;
        }
    }
    false
}

pub fn tail_whip_sound<L: Host + ?Sized>(game: &mut L) {
    if !below(bg2_staging::get(game, tail::ANGLE_DELTA), 8) {
        let counter = session_slot!(game, Ridley[RIDLEY].whip_sound_counter).wrapping_add(1);
        session_slot!(game, Ridley[RIDLEY].whip_sound_counter = counter);
        if below(counter, 16) {
            return;
        }
        if session_slot!(game, Ridley[RIDLEY].roaring) == 0 {
            game.queue_sound(2, 0x21, 6);
        }
    }
    session_slot!(game, Ridley[RIDLEY].whip_sound_counter = 0);
}

pub fn initialise_tail<L: Host + ?Sized>(game: &mut L) {
    bg2_staging::set(game, tail::ANGLE_DELTA, 1);
    bg2_staging::set(game, tail::MIN_CLOCKWISE_ANGLE, 16368);
    bg2_staging::set(game, tail::MAX_ANTICLOCKWISE_ANGLE, 16448);
    bg2_staging::set(game, tail::WHIP_TARGET_CLOCKWISE, NO_WHIP_TARGET);
    bg2_staging::set(game, tail::WHIP_TARGET_ANTICLOCKWISE, NO_WHIP_TARGET);
    bg2_staging::set(game, tail::EXTENSION_SPEED, 240);
    bg2_staging::set(game, tail::IDEAL_SEGMENT_ANGLE, 16);
    for segment in 0..7 {
        bg2_staging::set(game, tail::stagger_angle(segment), 17);
    }
    crate::actors::ridley_tables::owned::set_tail_segments_from_list(
        tail_address(tail::direction(0)),
        tail_data::DIRECTIONS,
        game,
    );
    crate::actors::ridley_tables::owned::set_tail_segments_from_list(
        tail_address(tail::distance(0)),
        tail_data::DISTANCES,
        game,
    );
    crate::actors::ridley_tables::owned::set_tail_segments_from_list(
        tail_address(tail::angle(0)),
        tail_data::ANGLES,
        game,
    );
    crate::actors::ridley_tables::owned::set_tail_segments_from_list(
        tail_address(tail::x(0)),
        tail_data::X_POSITIONS,
        game,
    );
    crate::actors::ridley_tables::owned::set_tail_segments_from_list(
        tail_address(tail::y(0)),
        tail_data::Y_POSITIONS,
        game,
    );
    set_tail_segment_flags(0, game);
}

pub fn activate_tail_segments<L: Host + ?Sized>(game: &mut L) {
    set_tail_segment_flags(0x8000, game);
}

pub fn set_tail_segment_flags<L: Host + ?Sized>(flags: u16, game: &mut L) {
    for segment in 0..7 {
        bg2_staging::set(game, tail::active(segment), flags);
    }
}

pub fn flip_tail<L: Host + ?Sized>(game: &mut L) {
    crate::actors::ridley_tables::owned::set_tail_segments_from_list(
        tail_address(tail::distance(0)),
        tail_data::DISTANCES,
        game,
    );
    for segment in 0..7 {
        bg2_staging::set(
            game,
            tail::angle(segment),
            0x8000u16.wrapping_sub(bg2_staging::get(game, tail::angle(segment))),
        );
        bg2_staging::set(
            game,
            tail::direction(segment),
            bg2_staging::get(game, tail::direction(segment)) | 0x8000u16,
        );
    }
}

pub fn contact_damage(game: &mut (impl Host + ?Sized)) -> u16 {
    let definition = session_slot!(
        game,
        Ridley[i32::from(game.session().actors.controls.selected)]
            .base
            .enemy_ptr
    );
    definition_damage(game.actor_record(ENEMY_DEFINITION_BANK, definition))
}

pub fn set_instruction_list<L: Host + ?Sized>(list: u16, game: &mut L) {
    session_slot!(game, Ridley[RIDLEY].base; current_instruction = list, instruction_timer = 1);
    session_slot!(game, Ridley[RIDLEY].base.timer = 0);
}

pub fn health_palette<L: Host + ?Sized>(game: &mut L) {
    let health = session_slot!(game, Ridley[RIDLEY].base.health);
    let index: i16 = if health >= 0x2328 {
        0
    } else if health >= 0x1518 {
        1
    } else if health >= 0x708 {
        2
    } else {
        3
    };
    session_slot!(game, Ridley[RIDLEY].acceleration_index = index as u16);
    if index.wrapping_sub(1) >= 0 {
        game.load_health_palette((index as u16).wrapping_sub(1));
    }
}

pub fn solid_block_at<H: Host + ?Sized>(x: u16, y: u16, game: &mut H) -> bool {
    let row = mult8x8((y >> 4) as u8, game.session().room.width as u8);
    let offset = usize::try_from(2 * (i32::from(row) + (i32::from(x) >> 4))).expect("level offset");
    crate::room_terrain::level_word(
        game,
        offset,
        crate::room_mechanisms::host::ByteOrder::Native,
    ) & 0xf000
        != 0
}

pub fn accelerate<L: Host + ?Sized>(k: u16, inertia: u16, x: u16, y: u16, game: &mut L) {
    accelerate_with_force(k, inertia, 0, x, y, game);
}

pub fn accelerate_with_force<L: Host + ?Sized>(
    k: u16,
    inertia: u16,
    force: u16,
    x: u16,
    y: u16,
    game: &mut L,
) {
    crate::probe::hit("homing.ridley.extra");
    let divisor = INERTIA[inertia as usize];
    accelerate_x(k, x, u16::from(divisor), force, game);
    let e = i32::from(k);
    let position = session_slot!(game, Ridley[e].base.y_pos);
    if position != y {
        session_slot!(game, Ridley[e].y_velocity, |old| chained_homing_velocity(
            position,
            y,
            old,
            divisor,
            Deceleration::Force(force),
            1280,
        ));
    }
}

pub fn accelerate_x<L: Host + ?Sized>(k: u16, x: u16, divisor: u16, force: u16, game: &mut L) {
    let e = i32::from(k);
    let position = session_slot!(game, Ridley[e].base.x_pos);
    if position != x {
        session_slot!(game, Ridley[e].x_velocity, |old| chained_homing_velocity(
            position,
            x,
            old,
            divisor as u8,
            Deceleration::Force(force),
            1280,
        ));
    }
}

pub fn ceres_accelerate<L: Host + ?Sized>(k: u16, inertia: u16, x: u16, y: u16, game: &mut L) {
    /// Ceres Ridley inertia table ($A6:D712).
    const CERES_INERTIA: [u8; 16] = [
        0x10, 0xf, 0xe, 0xd, 0xc, 0xb, 0xa, 9, 8, 7, 6, 5, 4, 3, 2, 1,
    ];
    crate::probe::hit("homing.ridley.double");
    let divisor = CERES_INERTIA[inertia as usize];
    ceres_accelerate_x(k, x, u16::from(divisor), game);
    let e = i32::from(k);
    let position = session_slot!(game, Ridley[e].base.y_pos);
    if position != y {
        session_slot!(game, Ridley[e].y_velocity, |old| chained_homing_velocity(
            position,
            y,
            old,
            divisor,
            Deceleration::DoubleStep,
            1280,
        ));
    }
}

pub fn ceres_accelerate_x<L: Host + ?Sized>(k: u16, x: u16, divisor: u16, game: &mut L) {
    let e = i32::from(k);
    let position = session_slot!(game, Ridley[e].base.x_pos);
    if position != x {
        session_slot!(game, Ridley[e].x_velocity, |old| chained_homing_velocity(
            position,
            x,
            old,
            divisor as u8,
            Deceleration::DoubleStep,
            1280,
        ));
    }
}

pub fn calculate_swoop_velocities<L: Host + ?Sized>(
    angle_step: u16,
    target_angle: u16,
    target_speed: u16,
    game: &mut L,
) {
    crate::probe::hit("ridley.swoop_velocities");
    if target_speed != session_slot!(game, Ridley[RIDLEY].swoop_speed) {
        if (target_speed as i16)
            .wrapping_sub(session_slot!(game, Ridley[RIDLEY].swoop_speed) as i16)
            >= 0
        {
            let mut speed = session_slot!(game, Ridley[RIDLEY].swoop_speed).wrapping_add(32);
            if !below(speed, target_speed) {
                speed = target_speed;
            }
            session_slot!(game, Ridley[RIDLEY].swoop_speed = speed);
        } else {
            let mut speed = session_slot!(game, Ridley[RIDLEY].swoop_speed).wrapping_sub(32);
            if below(speed, target_speed) {
                speed = target_speed;
            }
            session_slot!(game, Ridley[RIDLEY].swoop_speed = speed);
        }
    }
    let mut angle = session_slot!(game, Ridley[RIDLEY].swoop_angle).wrapping_add(angle_step);
    let past_target = if (angle_step & 0x8000) != 0 {
        below(angle, target_angle)
    } else {
        !below(angle, target_angle)
    };
    if past_target {
        angle = target_angle;
    }
    session_slot!(game, Ridley[RIDLEY].swoop_angle = angle);
    let direction = angle.to_ne_bytes()[1] as u16;
    session_slot!(
        game,
        Ridley[RIDLEY].x_velocity =
            x_velocity_at_angle(session_slot!(game, Ridley[RIDLEY].swoop_speed), direction)
    );
    session_slot!(
        game,
        Ridley[RIDLEY].y_velocity =
            y_velocity_at_angle(session_slot!(game, Ridley[RIDLEY].swoop_speed), direction)
    );
}

pub fn move_ridley<S: Host + ?Sized>(game: &mut S) {
    session_slot!(game, Ridley[RIDLEY].hit_boundary = 0);
    let k = game.session().actors.controls.selected;
    let e = game.session().actors.controls.selected as i32;
    let carry = session_slot!(byte game, Ridley[e].base.x_subpos[1]) as i32
        + session_slot!(byte game, Ridley[e].x_velocity[0]) as i32;
    session_slot!(byte game, Ridley[e].base.x_subpos[1] = carry as u8);
    let mut x = (session_slot!(game, Ridley[e].base.x_pos) as i32
        + (session_slot!(game, Ridley[e].x_velocity) >> 8) as i8 as i32
        + (carry >> 8)) as u16;
    if (x as i16).wrapping_sub(session_slot!(game, Ridley[RIDLEY].min_x) as i16) >= 0 {
        if (x as i16).wrapping_sub(session_slot!(game, Ridley[RIDLEY].max_x) as i16) >= 0 {
            session_slot!(game, Ridley[e].x_velocity = 0);
            session_slot!(game, Ridley[RIDLEY].hit_boundary = 2);
            x = session_slot!(game, Ridley[RIDLEY].max_x);
        }
        session_slot!(game, Ridley[e].base.x_pos = x);
    } else {
        ceres_earthquake(k, game);
        let copied = session_slot!(game, Ridley[RIDLEY].min_x);
        session_slot!(game, Ridley[e].base.x_pos = copied);
        session_slot!(game, Ridley[e].x_velocity = 0);
        session_slot!(game, Ridley[RIDLEY].hit_boundary = 1);
    }
    let carry = session_slot!(byte game, Ridley[e].base.y_subpos[1]) as i32
        + session_slot!(byte game, Ridley[e].y_velocity[0]) as i32;
    session_slot!(byte game, Ridley[e].base.y_subpos[1] = carry as u8);
    let mut y = (session_slot!(game, Ridley[e].base.y_pos) as i32
        + (session_slot!(game, Ridley[e].y_velocity) >> 8) as i8 as i32
        + (carry >> 8)) as u16;
    if (y as i16).wrapping_sub(session_slot!(game, Ridley[RIDLEY].min_y) as i16) >= 0 {
        if (y as i16).wrapping_sub(session_slot!(game, Ridley[RIDLEY].max_y) as i16) >= 0 {
            session_slot!(game, Ridley[e].y_velocity = 0);
            session_slot!(game, Ridley[RIDLEY].hit_boundary = 8);
            y = session_slot!(game, Ridley[RIDLEY].max_y);
        }
        session_slot!(game, Ridley[e].base.y_pos = y);
    } else {
        let copied = session_slot!(game, Ridley[RIDLEY].min_y);
        session_slot!(game, Ridley[e].base.y_pos = copied);
        session_slot!(game, Ridley[e].y_velocity = 0);
        session_slot!(game, Ridley[RIDLEY].hit_boundary = 4);
    }
}

pub fn ceres_earthquake<S: Host + ?Sized>(k: u16, game: &mut S) {
    if game.session().room.area != NORFAIR {
        let e = i32::from(k);
        let speed = abs16(session_slot!(game, Ridley[e].x_velocity))
            .max(abs16(session_slot!(game, Ridley[e].y_velocity)));
        if speed >= 0x280 {
            let kind = if game.session().room.area == NORFAIR {
                24
            } else {
                33
            };
            {
                game.session_mut().feedback.earthquake_kind = kind;
                game.session_mut().feedback.earthquake_timer = 12;
            };
        }
    }
}

pub fn face_room_middle<L: Host + ?Sized>(game: &mut L) {
    let list = match session_slot!(game, Ridley[RIDLEY].facing) {
        FACING_LEFT if !in_right_half(game) => instruction_list::TURN_TO_FACE_RIGHT,
        FACING_LEFT | FACING_FORWARD => return,
        _ if in_right_half(game) => instruction_list::TURN_TO_FACE_LEFT,
        _ => return,
    };
    session_slot!(game, Ridley[RIDLEY].base; current_instruction = list, instruction_timer = 2);
    session_slot!(game, Ridley[RIDLEY].base.timer = 0);
}

pub(super) fn in_right_half<L: Host + ?Sized>(game: &L) -> bool {
    session_slot!(byte game, Ridley[RIDLEY].base.x_pos[0]) >= 0x80
}

pub fn update_wings<S: Host + ?Sized>(game: &mut S) {
    wings_animation_timer(game);
    let timer = (session_slot!(game, Ridley[RIDLEY].wings_timer) as i16)
        .wrapping_sub(session_slot!(game, Ridley[RIDLEY].wings_timer_delta) as i16);
    session_slot!(game, Ridley[RIDLEY].wings_timer = timer as u16);
    if timer < 0 {
        session_slot!(game, Ridley[RIDLEY].wings_timer = 32);
        let mut frame = session_slot!(game, Ridley[RIDLEY].wings_frame).wrapping_add(1);
        if frame >= 0xa {
            frame = 0;
        }
        session_slot!(game, Ridley[RIDLEY].wings_frame = frame);
    }
}

pub fn wings_animation_timer<S: Host + ?Sized>(game: &mut S) {
    /// The wings timer deltas ($A6:D9ED), by speed.
    const DELTAS: [u16; 8] = [0xc, 0xe, 0x10, 0x12, 0x1c, 0x20, 0x28, 0x30];
    let x_speed = abs16(session_slot!(game, Ridley[RIDLEY].x_velocity));
    let sum = x_speed.wrapping_add(abs16(session_slot!(game, Ridley[RIDLEY].y_velocity)));
    if sum == 0 {
        session_slot!(game, Ridley[RIDLEY].wings_timer_delta = 0);
        return;
    }
    let mut speed = sum.wrapping_sub(x_speed);
    if below(speed, x_speed) {
        speed = x_speed;
    }
    let index = (((4 * speed as i32) & 0xf00) >> 8).min(7);
    let mut delta = DELTAS[index as usize];
    if (session_slot!(game, Ridley[RIDLEY].y_velocity) & 0x8000) == 0 {
        delta >>= 1;
    }
    session_slot!(game, Ridley[RIDLEY].wings_timer_delta = delta);
}

pub fn initialise_ribs<L: Host + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY]; ribs_pointer = RIBS_ANIMATION, ribs_timer = 1);
}

pub fn handle_ribs<L: Host + ?Sized>(game: &mut L) {
    let timer = session_slot!(game, Ridley[RIDLEY].ribs_timer).wrapping_sub(1);
    session_slot!(game, Ridley[RIDLEY].ribs_timer = timer);
    if timer != 0 {
        return;
    }
    let mut cursor = session_slot!(game, Ridley[RIDLEY].ribs_pointer);
    let frame = loop {
        let frame = game.actor_record(0xa6, cursor);
        if (frame.word(0) & 0x8000) == 0 {
            break frame;
        }
        // A word with bit 15 set is the table's loop pointer.
        cursor = frame.word(0);
        session_slot!(game, Ridley[RIDLEY].ribs_pointer = cursor);
    };
    session_slot!(game, Ridley[RIDLEY].ribs_timer = frame.word(0));
    game.queue_tile_pair([frame.word(2), frame.word(4)], [29216, 29472], 64);
    session_slot!(game, Ridley[RIDLEY].ribs_pointer = cursor.wrapping_add(6));
}

pub(super) fn tail_tip_sector<L: Host + ?Sized>(game: &L) -> isize {
    (((game.tail_byte(2 * tail::angle(5)) as i32 + game.tail_byte(2 * tail::angle(6)) as i32 + 8)
        & 0xf0) as u8
        >> 4) as isize
}

pub fn baby_metroid_spritemap<L: Host + ?Sized>(game: &mut L) -> Option<u16> {
    if (session_slot!(game, Ridley[RIDLEY].baby_metroid_instruction) & 0x8000) == 0 {
        return None;
    }
    let mut cursor = session_slot!(game, Ridley[RIDLEY].baby_metroid_instruction);
    let mut entry = game.actor_record(0xa6, cursor);
    let mut instruction = entry.word(0) & 0x8000 != 0;
    if !instruction {
        if session_slot!(game, Ridley[RIDLEY].baby_metroid_instruction_timer) != entry.word(0) {
            session_slot!(game, Ridley[RIDLEY].baby_metroid_instruction_timer, |old| {
                old.wrapping_add(1)
            });
            return Some(entry.word(2));
        }
        cursor = cursor.wrapping_add(4);
    }
    loop {
        if instruction {
            cursor = run_baby_metroid_instruction(game, entry.word(0), cursor.wrapping_add(2));
            instruction = false;
        } else {
            entry = game.actor_record(0xa6, cursor);
            if entry.word(0) & 0x8000 == 0 {
                break;
            }
            instruction = true;
        }
    }
    session_slot!(game, Ridley[RIDLEY]; baby_metroid_instruction_timer = 1,
        baby_metroid_instruction = cursor);
    Some(entry.word(2))
}

pub fn off_screen<S: Host + ?Sized>(game: &mut S) -> bool {
    let y = session_slot!(game, Ridley[RIDLEY].base.y_pos) as i16;
    if y >= 0 {
        let layer1 = game.session().room.camera.layer1;
        let screen_y = (y as i32 + 32 - layer1[1] as i32) as i16;
        if screen_y >= 0 && below(screen_y as u16, 288) {
            let x = session_slot!(game, Ridley[RIDLEY].base.x_pos) as i16;
            if x >= 0 {
                let screen_x = (x as i32 + 32 - layer1[0] as i32) as i16;
                if screen_x >= 0 && below(screen_x as u16, 320) {
                    return false;
                }
            }
        }
    }
    true
}

pub fn projectile_in_rect<L: Host + ?Sized>(rect: Rect, game: &mut L) -> Option<usize> {
    if game.session().projectiles.count == 0 {
        return None;
    }
    let n = (0..5).find(|&n| {
        let kind = interaction::get_at(game, ProjectileField::Kind, n) as i16;
        kind < 0
            && ((kind.to_ne_bytes()[1] as i32 & 0xf) - 3) & 0x8000 != 0
            && projectile_overlaps(game, n, rect)
    })?;
    game.set_projectile_field(n, ProjectileField::X, rect.x);
    game.set_projectile_field(n, ProjectileField::Y, rect.y);
    interaction::set_at(
        game,
        ProjectileField::Direction,
        n,
        interaction::get_at(game, ProjectileField::Direction, n) | 0x10u16,
    );
    Some(n)
}

pub fn samus_in_rect<L: Host + ?Sized>(rect: Rect, game: &mut L) -> bool {
    let dx = abs16(game.session().player.x.wrapping_sub(rect.x));
    if dx < game.session().player.contact.x_radius
        || dx.wrapping_sub(game.session().player.contact.x_radius) < rect.w
    {
        let dy = abs16(game.session().player.y.wrapping_sub(rect.y));
        if dy < game.session().player.contact.radius
            || dy.wrapping_sub(game.session().player.contact.radius) < rect.h
        {
            return true;
        }
    }
    false
}

pub fn touch<L: Host + ?Sized>(game: &mut L) {
    game.contact_reaction(Death::LeavePending);
}

pub fn shot<L: Host + ?Sized>(game: &mut L) {
    if game.session().room.area == NORFAIR {
        game.shot_reaction(Shot::ReactionOnly);
        return;
    }
    let flash_timer = session_slot!(game, Ridley[RIDLEY].base.flash_timer);
    let flash = if flash_timer != 0 && flash_timer & 1 != 0 {
        14
    } else {
        13
    };
    session_slot!(game, Ridley[RIDLEY].base.flash_timer = flash);
    session_slot!(game, Ridley[RIDLEY].hit_counter, |hit_counter| hit_counter
        .wrapping_add(1));
}

pub fn power_bomb<L: Host + ?Sized>(game: &mut L) {
    game.power_bomb_reaction(Death::LeavePending);
}

pub fn start_death_sequence<L: Host + ?Sized>(game: &mut L) {
    if session_slot!(game, Ridley[RIDLEY].base.health) == 0
        && (session_slot!(game, Ridley[RIDLEY].fight_mode) & 0x8000) == 0
    {
        session_slot!(game, Ridley[RIDLEY].fight_mode = DEATH_SEQUENCE);
        set_tangible(game, false);
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::DYING_GRABBED_SAMUS
        );
    }
}

pub fn tail_samus_collision<L: Host + ?Sized>(game: &mut L) {
    let rect = Rect {
        x: bg2_staging::get(game, tail::x(6)),
        y: bg2_staging::get(game, tail::y(6)),
        w: 14,
        h: 14,
    };
    if samus_in_rect(rect, game) {
        let damage = session_slot!(game, Ridley[RIDLEY].tail_damage);
        let amount = game.suit_damage(damage);
        game.damage_player(amount);
        game.session_mut().player.invincibility = 96;
        game.session_mut().player.knockback_timer = 5;
        game.session_mut().player.knockback_x_direction = ((game.session().player.x as i16)
            .wrapping_sub(bg2_staging::get(game, tail::x(6)) as i16)
            >= 0) as u16;
    }
}

pub fn tail_projectile_collision<L: Host + ?Sized>(game: &mut L) {
    if (session_slot!(game, Ridley[RIDLEY].base.properties) & K_ENEMY_PROPS_TANGIBLE as u16) != 0 {
        return;
    }
    let tip = Rect {
        x: bg2_staging::get(game, tail::x(6)),
        y: bg2_staging::get(game, tail::y(6)),
        w: 14,
        h: 14,
    };
    let Some(n) = projectile_in_rect(tip, game).or_else(|| {
        let segment = Rect {
            x: bg2_staging::get(game, tail::x(5)),
            y: bg2_staging::get(game, tail::y(5)),
            w: 10,
            h: 10,
        };
        projectile_in_rect(segment, game)
    }) else {
        return;
    };
    let x = interaction::get_at(game, ProjectileField::X, n);
    let y = interaction::get_at(game, ProjectileField::Y, n);
    game.place_projectile_spawn(x, y);
    let mut kind = 12;
    if projectile_kind(game, n) == MISSILE {
        game.queue_sound(0, 0x3d, 6);
        kind = 6;
    }
    game.spawn_room_projectile(DUST_CLOUD, kind);
}

pub fn roar<L: Host + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, Ridley[RIDLEY].roaring = 89);
    game.queue_sound(1, 0x59, 6);
    script
}

pub fn stop_roaring<L: Host + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, Ridley[RIDLEY].roaring = 0);
    script
}

pub fn goto_if_samus_health_low<L: Host + ?Sized>(
    script: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    if game.session().room.area == NORFAIR || !below(game.session().inventory.health.current, 30) {
        return script.offset(1);
    }
    session_slot!(game, Ridley[RIDLEY].timer = 8);
    EnemyScript::jump(script.word(0))
}

pub fn goto_by_holding_samus<L: Host + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    if session_slot!(game, Ridley[RIDLEY].holding_samus) != 0 {
        EnemyScript::jump(script.word(0))
    } else {
        EnemyScript::jump(script.word(2))
    }
}

pub fn goto_if_not_holding_samus<L: Host + ?Sized>(
    script: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    if session_slot!(game, Ridley[RIDLEY].holding_samus) != 0 {
        script.offset(1)
    } else {
        EnemyScript::jump(script.word(0))
    }
}

pub fn set_feet_distance<L: Host + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    game.session_mut().player.previous_position.x.whole = game.session().player.x;
    game.session_mut().player.previous_position.y.whole = game.session().player.y;
    session_slot!(game, Ridley[RIDLEY].feet_distance_index = script.word(0));
    script.offset(1)
}

pub fn goto_if_not_facing_left<L: Host + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    if session_slot!(game, Ridley[RIDLEY].facing) != FACING_LEFT {
        EnemyScript::jump(script.word(0))
    } else {
        script.offset(1)
    }
}

pub fn move_by<L: Host + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, Ridley[RIDLEY].base.x_pos, |x_pos| x_pos
        .wrapping_add(script.word(0)));
    session_slot!(game, Ridley[RIDLEY].base.y_pos, |y_pos| y_pos
        .wrapping_add(script.word(2)));
    script.offset(2)
}

pub fn flip_left<L: Host + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, Ridley[RIDLEY].facing = FACING_LEFT);
    flip_tail(game);
    script
}

pub fn face_forward<L: Host + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, Ridley[RIDLEY].facing = FACING_FORWARD);
    script
}

pub fn flip_right<L: Host + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, Ridley[RIDLEY].facing = FACING_RIGHT);
    flip_tail(game);
    script
}

pub fn calculate_fireball_velocities<L: Host + ?Sized>(
    script: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    let angle = if session_slot!(game, Ridley[RIDLEY].facing) != 0 {
        let dx = (game.session().player.x as i32
            - (session_slot!(game, Ridley[RIDLEY].base.x_pos) as i32 + 25)) as u16;
        let dy = (game.session().player.y as i32
            - (session_slot!(game, Ridley[RIDLEY].base.y_pos) as i32 - 43)) as u16;
        let mut angle = -(angle_from_delta(dx, dy) as i32 + 0x80) as u8 as u16;
        if !(0x15..0xc0).contains(&angle) {
            angle = 0x15;
        } else if angle >= 0x50 {
            angle = 0x50;
        }
        angle
    } else {
        let dx = (game.session().player.x as i32
            - (session_slot!(game, Ridley[RIDLEY].base.x_pos) as i32 - 25)) as u16;
        let dy = (game.session().player.y as i32
            - (session_slot!(game, Ridley[RIDLEY].base.y_pos) as i32 - 43)) as u16;
        let mut angle = -(angle_from_delta(dx, dy) as i32 + 0x80) as u8 as u16;
        if !(0x40..0xeb).contains(&angle) {
            angle = 0xeb;
        } else if angle < 0xb0 {
            angle = 0xb0;
        }
        angle
    };
    session_slot!(
        game,
        Ridley[RIDLEY].fireball_x_velocity = x_velocity_at_angle(0x500, angle)
    );
    session_slot!(
        game,
        Ridley[RIDLEY].fireball_y_velocity = y_velocity_at_angle(0x500, angle)
    );
    script
}

pub fn fireball_with_afterburn<L: Host + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    spawn_fireball(script, 0, game)
}

pub fn fireball_without_afterburn<L: Host + ?Sized>(
    script: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    spawn_fireball(script, 0xe, game)
}

pub fn spawn_fireball<L: Host + ?Sized>(
    script: EnemyScript,
    afterburn: u16,
    game: &mut L,
) -> EnemyScript {
    game.session_mut().enemy_projectiles.spawn.secondary = afterburn;
    game.spawn_room_projectile(FIREBALL, session_slot!(game, Ridley[RIDLEY].facing));
    script
}

pub fn start_ceres_liftoff<L: Host + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(
        game,
        Ridley[RIDLEY].function = function::CERES_LIFTOFF_ACCELERATING
    );
    session_slot!(game, Ridley[RIDLEY].y_velocity = -352i32 as u16);
    script
}

pub fn start_norfair_liftoff<L: Host + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, Ridley[RIDLEY]; function = function::LIFTOFF, y_velocity = -352i32 as u16);
    script
}

pub(super) fn definition_damage(record: sm_data::cartridge::RomRecord) -> u16 {
    record.word(ENEMY_DEFINITION_DAMAGE)
}
