//! Kihunter, the winged insect that swoops at Samus and, once shot down to its wingless
//! form, hops and spits acid; its wings, in the next slot, flutter down
//! (bank $A8:E99A-$A8:F9BD). Was `ki_hunter`.
#![forbid(unsafe_code)]
use crate::actors::access::ActorKey as Key;
use crate::actors::api::{self, Axis, MovementResult::Blocked, SlopePolicy};
use crate::actors::host::{ActorLease as _, EnemyHost, LeaseActors, Shot, abs16};
use crate::actors::layouts::part_view;
use crate::enemy_lists::kihunter_wing_instruction_list;
use crate::enemy_script::EnemyScript;
use crate::session_slot;

part_view! {
    /// The Kihunter's slot and extended RAM. Its velocities and arc angular velocity and
    /// acceleration are whole words over subwords.
    Kihunter: EnemyLayout40 {
        words: [function, arc_centre_x, arc_centre_y, var_d, var_e, arc_angle, parameter_1, parameter_2],
        pad1: [u8; 26696],
        words: [angular_velocity_cap, var_01, angular_velocity, angular_subvelocity, angular_acceleration,
            angular_subacceleration, x_subvelocity, x_velocity, y_subvelocity, y_velocity, min_y, max_y, spawn_x,
            spawn_y, swipe_trigger_angle, thinking_timer, swipe, arc_radius_x, arc_radius_y, var_13, wingless],
        _: [u16; 11],
    }
}
part_view! {
    /// The Kihunter wings' slot and extended RAM.
    Wings: EnemyLayout40 {
        words: [function, speed_table_index, var_c, fall_phase, var_e, arc_angle, parameter_1, parameter_2],
        pad1: [u8; 26696],
        words: [falling_function, right_offset_x, right_offset_y, left_offset_x, left_offset_y, arc_start_x,
            arc_start_y, fall_start_y, fall_start_x, var_09, speed_table_reset, arc_top_offset],
        _: [u16; 20],
    }
}

const IDLE_FLYING: u16 = 0xf268;
const SWOOP: u16 = 0xf3b8;
const BACK_OFF: u16 = 0xf4ed;
const INITIAL_FALLING: u16 = 0xf55a;
const PREPARE_TO_HOP: u16 = 0xf58b;
const IDLE: u16 = 0xf5e3; // An RTS, while its instruction list runs.
const HOP: u16 = 0xf5f0;
const THINKING: u16 = 0xf68b;
const FIRE_ACID_SPIT: u16 = 0xf6b3;
const ATTACHED: u16 = 0xf6f3;
const FALLING: u16 = 0xf7cf;
const DRIFTING_LEFT: u16 = 0xf7db;
const DRIFTING_RIGHT: u16 = 0xf8ad;
/// $A2:838F: the quadratic speed table, 8 bytes an entry.
const QUADRATIC_SPEEDS: u32 = 0xa2_838f;
/// The wings are the next slot after the body.
const WINGS: i32 = 64;

/// Set up falling Kihunter wings drifting left ($A8:F947) or right ($A8:F96A).
fn begin_drift(game: &mut impl EnemyHost, wings: i32, left: bool) {
    let falling = if left { DRIFTING_LEFT } else { DRIFTING_RIGHT };
    let index = session_slot!(game, Wings[wings].speed_table_reset);
    let angle = if left { 0xe000 } else { 0xa000 };
    let x = session_slot!(game, Wings[wings].base.x_pos);
    let y = session_slot!(game, Wings[wings].base.y_pos);
    session_slot!(game, Wings[wings]; falling_function = falling, speed_table_index = index, arc_angle = angle,
        arc_start_x = x, arc_start_y = y);
}
/// Kihunter wings falling function - drifting left ($A8:F7DB) or right ($A8:F8AD).
fn drift(game: &mut (impl EnemyHost + LeaseActors), left: bool) {
    let wings = i32::from(game.current_enemy());
    let record = game
        .lease()
        .lease_native_record(0xa2, (QUADRATIC_SPEEDS & 0xffff) as u16);
    let index = session_slot!(byte game, Wings[wings].speed_table_index[1]);
    let speed = record.word(8 * usize::from(index) + if left { 5 } else { 1 });
    session_slot!(game, Wings[wings].arc_angle, |angle| angle
        .wrapping_add(speed));
    let y = wing_coordinate(game, wings, false, left);
    if left {
        session_slot!(game, Wings[wings].base.y_pos = y);
    } else {
        let amount =
            (y.wrapping_sub(session_slot!(game, Wings[wings].base.y_pos)) as i16 as i32) << 16;
        let key = Key::from_legacy(game.current_enemy());
        if api::move_vertical(&mut game.lease(), key, amount) == Blocked {
            session_slot!(game, Wings[wings].base.properties, |old| old | 0x200);
            let x = session_slot!(game, Wings[wings].fall_start_x);
            session_slot!(game, Wings[wings].base.x_pos = x);
            let y = session_slot!(game, Wings[wings].fall_start_y);
            session_slot!(game, Wings[wings].base.y_pos = y);
            let current = i32::from(game.current_enemy());
            session_slot!(game, Wings[current].speed_table_index = y);
            return;
        }
    }
    let x = wing_coordinate(game, wings, true, left);
    session_slot!(game, Wings[wings].base.x_pos = x);
    if ((session_slot!(game, Wings[wings].arc_angle).wrapping_add(0x4000) & 0x8000) != 0) == left {
        begin_drift(game, i32::from(game.current_enemy()), !left);
    } else {
        let speed = (session_slot!(game, Wings[wings].speed_table_index) as i16).wrapping_sub(384);
        let target = if left {
            wings
        } else {
            i32::from(game.current_enemy())
        };
        let index = if speed < 0 { 256 } else { speed as u16 };
        session_slot!(game, Wings[target].speed_table_index = index);
    }
}
/// Initialisation AI - Kihunter ($A8:F188).
pub fn initialize(game: &mut impl EnemyHost) {
    let hunter = i32::from(game.current_enemy());
    session_slot!(game, Kihunter[hunter].base.properties, |old| old | 0x2000);
    session_slot!(game, Kihunter[hunter].wingless = 0);
    session_slot!(game, Kihunter[hunter].base; instruction_timer = 1, timer = 0, current_instruction = 0xe9fa);
    session_slot!(game, Kihunter[hunter]; arc_angle = 0, function = IDLE_FLYING, y_subvelocity = 0, y_velocity = 1,
        x_subvelocity = 0, x_velocity = u16::MAX);
    let min_y = session_slot!(game, Kihunter[hunter].base.y_pos).wrapping_sub(16);
    session_slot!(game, Kihunter[hunter]; min_y = min_y, max_y = min_y.wrapping_add(32));
    let x = session_slot!(game, Kihunter[hunter].base.x_pos);
    let y = session_slot!(game, Kihunter[hunter].base.y_pos);
    session_slot!(game, Kihunter[hunter]; spawn_x = x, spawn_y = y);
    if session_slot!(game, Kihunter[hunter].parameter_1) & 0x8000 != 0 {
        session_slot!(game, Kihunter[hunter]; wingless = 1, function = INITIAL_FALLING, y_subvelocity = 0,
            y_velocity = 1);
    }
}
/// Initialisation AI - Kihunter wings ($A8:F214): take the body's position and palette.
pub fn initialize_wings(game: &mut (impl EnemyHost + LeaseActors)) {
    let wings = i32::from(game.current_enemy());
    session_slot!(game, Wings[wings].base.properties, |old| old | 0x2000);
    session_slot!(game, Wings[wings].base; instruction_timer = 1, timer = 0, current_instruction = 0xea4e);
    follow_body(game, wings, false);
    session_slot!(game, Wings[wings].function = ATTACHED);
    let palette = session_slot!(game, Kihunter[wings - WINGS].base.palette_index);
    let tiles = session_slot!(game, Kihunter[wings - WINGS].base.vram_tiles_index);
    for (field, value) in [(0x1e, palette), (0x20, tiles)] {
        let at = crate::ram_slot!(@at Wings, wings, base) + field;
        game.lease().write_slice_native(at, value);
    }
    if session_slot!(game, Kihunter[wings - WINGS].parameter_1) & 0x8000 != 0 {
        session_slot!(game, Wings[wings].base.properties, |old| old | 0x200);
    }
}
/// Copy the body's Y then X position to the wings (or X then Y), each through its own
/// compatibility Runtime, as the source does.
fn follow_body(game: &mut (impl EnemyHost + LeaseActors), wings: i32, x_first: bool) {
    for x in [x_first, !x_first] {
        let (field, value) = match x {
            true => (2, session_slot!(game, Kihunter[wings - WINGS].base.x_pos)),
            false => (6, session_slot!(game, Kihunter[wings - WINGS].base.y_pos)),
        };
        let at = crate::ram_slot!(@at Wings, wings, base) + field;
        game.lease().write_slice_native(at, value);
    }
}
/// Determine falling Kihunter wings speed table index reset value ($A8:F98D).
fn prepare_fall_speed(game: &mut (impl EnemyHost + LeaseActors)) {
    let wings = i32::from(game.current_enemy());
    session_slot!(game, Wings[wings]; speed_table_reset = 0, speed_table_index = 0);
    loop {
        let speed = session_slot!(game, Wings[wings].speed_table_reset).wrapping_add(384);
        session_slot!(game, Wings[wings].speed_table_reset = speed);
        let record = game
            .lease()
            .lease_native_record(0xa2, (QUADRATIC_SPEEDS & 0xffff) as u16);
        let phase = record
            .word(8 * usize::from(speed >> 8) + 1)
            .wrapping_add(session_slot!(game, Wings[wings].fall_phase));
        session_slot!(game, Wings[wings].fall_phase = phase);
        if phase.wrapping_sub(0x2000) & 0x8000 == 0 {
            break;
        }
    }
}
/// Enemy shot - Kihunter ($A8:F701): below its wings' health it loses them, which fall;
/// above it, the wings share its frozen state and flash; dead, the wings go too.
pub fn shot(game: &mut (impl EnemyHost + LeaseActors)) {
    game.shot_reaction(Shot::Finalize);
    let hunter = i32::from(game.current_enemy());
    let wings = hunter + WINGS;
    let health = session_slot!(game, Kihunter[hunter].base.health);
    if health == 0 {
        return session_slot!(game, Wings[wings].base.properties = 512);
    }
    let threshold = session_slot!(game, Wings[wings].parameter_1);
    if health != threshold && (health as i16).wrapping_sub(threshold as i16) >= 0 {
        for field in [0x12, 0x26, 0x28, 0x24] {
            let at = crate::ram_slot!(@at Kihunter, hunter, base) + field;
            let value = game.lease().lease_native_word(at);
            game.lease().write_slice_native(at + 64, value);
        }
        return;
    }
    if session_slot!(game, Kihunter[hunter].wingless) != 0 {
        return;
    }
    session_slot!(game, Kihunter[hunter]; wingless = 1, function = INITIAL_FALLING, y_subvelocity = 0, y_velocity = 1);
    let body = game.current_enemy();
    game.set_current_enemy(body.wrapping_add(64));
    if session_slot!(game, Wings[wings].function) != FALLING {
        let x = session_slot!(game, Wings[wings].base.x_pos);
        let y = session_slot!(game, Wings[wings].base.y_pos);
        session_slot!(game, Wings[wings]; fall_start_y = y, fall_start_x = x);
        prepare_fall_speed(game);
        wing_origins(game);
        let arc_y = session_slot!(game, Wings[wings].fall_start_y)
            .wrapping_sub(session_slot!(game, Wings[wings].arc_top_offset));
        let x = session_slot!(game, Wings[wings].base.x_pos);
        let index = session_slot!(game, Wings[wings].speed_table_reset);
        session_slot!(game, Wings[wings]; arc_angle = 0xe000, function = FALLING, falling_function = DRIFTING_LEFT,
            arc_start_y = arc_y, arc_start_x = x, speed_table_index = index);
        session_slot!(game, Wings[wings].base; current_instruction = 0xea7e, spritemap_pointer = 0x804d,
            instruction_timer = 1, timer = 0);
        session_slot!(game, Wings[wings].base.properties, |old| old | 0x800);
    }
    game.set_current_enemy(body);
}
/// Fire acid spit ($A8:F6DC).
fn spit(game: &mut impl EnemyHost, projectile: u16) {
    game.queue_sound(1, 0x4c, 6);
    game.spawn_projectile(0, game.current_enemy(), projectile);
    let current = i32::from(game.current_enemy());
    session_slot!(game, Kihunter[current].thinking_timer = 24);
}
/// Determine falling Kihunter wings left ($A8:F851) and right ($A8:F87F) arc centre offsets.
fn wing_origins(game: &mut (impl EnemyHost + LeaseActors)) {
    for (phase, x_field, y_field) in [(0xe0, 3, 4), (0xa0, 1, 2)] {
        let x = game.actor_cosine(phase, 0x30);
        let at = 0x7800 + usize::from(game.current_enemy());
        game.lease().write_slice_native(at + x_field * 2, x);
        let y = game.actor_sine(phase, 0x30);
        game.lease().write_slice_native(at + y_field * 2, y);
    }
}
/// Set the body's and the wings' instruction lists to the pair at `index`.
fn set_paired_lists(game: &mut (impl EnemyHost + LeaseActors), k: u16, index: u16) {
    for (offset, entry) in [(0, index), (64, index + 1)] {
        let value = kihunter_wing_instruction_list(game, entry as isize);
        let at = crate::ram_slot!(@at Kihunter, i32::from(k) + offset, base.current_instruction);
        game.lease().write_slice_native(at, value);
    }
    for offset in [0, 64] {
        let at = crate::ram_slot!(@at Kihunter, i32::from(k) + offset, base.instruction_timer);
        game.lease().write_slice_native(at, 1);
    }
}
/// Main AI - Kihunter ($A8:F25C) or Kihunter wings ($A8:F262): run its function.
pub fn main_ai(game: &mut (impl EnemyHost + LeaseActors)) {
    let k = game.current_enemy();
    let hunter = i32::from(k);
    match session_slot!(game, Kihunter[hunter].function) {
        IDLE_FLYING => idle_flying(game, k),
        SWOOP => swoop(game, k),
        BACK_OFF => {
            if move_velocity(game, k, Axis::X)
                || {
                    snap(game, k);
                    move_velocity(game, k, Axis::Y)
                }
                || (session_slot!(game, Kihunter[hunter].base.y_pos) as i16)
                    .wrapping_sub(session_slot!(game, Kihunter[hunter].spawn_y) as i16)
                    < 0
            {
                session_slot!(game, Kihunter[hunter].function = IDLE_FLYING);
            }
        }
        INITIAL_FALLING => {
            if move_velocity(game, k, Axis::Y) {
                session_slot!(game, Kihunter[hunter].function = PREPARE_TO_HOP);
            } else {
                session_slot!(game, Kihunter[hunter](y_velocity, y_subvelocity) += 0xe000);
            }
        }
        PREPARE_TO_HOP => {
            session_slot!(game, Kihunter[hunter]; function = IDLE, y_subvelocity = 0);
            let hop = ((game.random_number() as i32 & 1) - 8) as u16;
            session_slot!(game, Kihunter[hunter].y_velocity = hop);
            let left = (session_slot!(game, Kihunter[hunter].base.x_pos) as i16)
                .wrapping_sub(game.player_x() as i16)
                >= 0;
            session_slot!(game, Kihunter[hunter]; x_subvelocity = 0, x_velocity = if left { (-2i16) as u16 } else { 2 });
            let list = if left { 0xea8a } else { 0xeaa6 };
            session_slot!(game, Kihunter[hunter].base; current_instruction = list, instruction_timer = 1);
        }
        HOP => {
            if move_velocity(game, k, Axis::Y) {
                if session_slot!(game, Kihunter[hunter].y_velocity) & 0x8000 != 0 {
                    session_slot!(game, Kihunter[hunter].y_velocity = 1);
                } else {
                    session_slot!(game, Kihunter[hunter]; y_subvelocity = 0, y_velocity = (-4i16) as u16, function = IDLE,
                        thinking_timer = 12);
                    let list = session_slot!(game, Kihunter[hunter].base.current_instruction);
                    let list = if (list as i32 + 0x155a) & 0x8000 == 0 {
                        0xeada
                    } else {
                        0xeac2
                    };
                    session_slot!(game, Kihunter[hunter].base; current_instruction = list, instruction_timer = 1);
                }
            } else if move_velocity(game, k, Axis::X) {
                session_slot!(game, Kihunter[hunter].x_velocity, |v| (-(v as i32)) as u16);
            } else {
                snap(game, k);
                session_slot!(game, Kihunter[hunter](y_velocity, y_subvelocity) += 0xe000);
            }
        }
        THINKING => {
            let timer = session_slot!(game, Kihunter[hunter].thinking_timer).wrapping_sub(1);
            session_slot!(game, Kihunter[hunter].thinking_timer = timer);
            if timer == 0 {
                let x = session_slot!(game, Kihunter[hunter].base.x_pos);
                let distance = abs16(x.wrapping_sub(game.player_x()));
                let next = if (distance as i32 - 96) & 0x8000 != 0 {
                    FIRE_ACID_SPIT
                } else {
                    PREPARE_TO_HOP
                };
                session_slot!(game, Kihunter[hunter].function = next);
            }
        }
        FIRE_ACID_SPIT => {
            let right = (session_slot!(game, Kihunter[hunter].base.x_pos) as i16)
                .wrapping_sub(game.player_x() as i16)
                < 0;
            let list = if right { 0xeb10 } else { 0xeaf2 };
            session_slot!(game, Kihunter[hunter].base; current_instruction = list, instruction_timer = 1);
            session_slot!(game, Kihunter[hunter].function = IDLE);
        }
        ATTACHED => follow_body(game, hunter, true),
        // Kihunter wings function - falling ($A8:F7CF).
        FALLING => match session_slot!(game, Wings[hunter].falling_function) {
            DRIFTING_LEFT => drift(game, true),
            DRIFTING_RIGHT => drift(game, false),
            _ => game.unknown_routine(),
        },
        IDLE => (),
        _ => game.unknown_routine(),
    }
}
/// Ki-hunter function - winged - idle flying ($A8:F268): bob between its bounds and pace,
/// and swoop at Samus when she comes near and below.
fn idle_flying(game: &mut (impl EnemyHost + LeaseActors), k: u16) {
    let hunter = i32::from(k);
    let vertical = if move_velocity(game, k, Axis::Y) {
        Some(session_slot!(game, Kihunter[hunter].y_velocity).wrapping_neg())
    } else {
        let y = session_slot!(game, Kihunter[hunter].base.y_pos) as i16;
        if y.wrapping_sub(session_slot!(game, Kihunter[hunter].min_y) as i16) < 0 {
            Some(1)
        } else if y.wrapping_sub(session_slot!(game, Kihunter[hunter].max_y) as i16) >= 0 {
            Some(u16::MAX)
        } else {
            None
        }
    };
    if let Some(speed) = vertical {
        session_slot!(game, Kihunter[hunter].y_velocity = speed);
    }
    if move_velocity(game, k, Axis::X) {
        let speed = session_slot!(game, Kihunter[hunter].x_velocity).wrapping_neg();
        session_slot!(game, Kihunter[hunter].x_velocity = speed);
        set_paired_lists(game, k, if speed & 0x8000 != 0 { 0 } else { 2 });
    }
    snap(game, k);
    let dx = game
        .player_x()
        .wrapping_sub(session_slot!(game, Kihunter[hunter].base.x_pos));
    let distance = abs16(dx);
    let y = i32::from(session_slot!(game, Kihunter[hunter].base.y_pos));
    let below = (i32::from(game.player_y()) - y - 32) & 0x8000 == 0;
    if (distance as i16).wrapping_sub(0x60) >= 0 || !below {
        return;
    }
    let dy = game
        .player_y()
        .wrapping_sub(session_slot!(game, Kihunter[hunter].base.y_pos));
    let pair = if dx & 0x8000 != 0 {
        session_slot!(game, Kihunter[hunter]; angular_velocity_cap = (-2i16) as u16, angular_velocity = 0,
            angular_subvelocity = 0, angular_acceleration = u16::MAX, x_velocity = u16::MAX,
            angular_subacceleration = 0xe000, arc_angle = 255, swipe_trigger_angle = 240);
        0
    } else {
        session_slot!(game, Kihunter[hunter]; angular_velocity_cap = 2, angular_velocity = 0, angular_subvelocity = 0,
            angular_acceleration = 0, angular_subacceleration = 0x2000, arc_angle = 128, x_velocity = 1,
            swipe_trigger_angle = 144);
        4
    };
    set_paired_lists(game, k, pair >> 1);
    let x = session_slot!(game, Kihunter[hunter].base.x_pos);
    let y = session_slot!(game, Kihunter[hunter].base.y_pos);
    session_slot!(game, Kihunter[hunter]; arc_centre_x = dx.wrapping_add(x), arc_centre_y = y, function = SWOOP,
        arc_radius_y = dy, arc_radius_x = distance, swipe = 0);
}
/// One frame of the swoop's swing ($A8:F3B8): the 16.16 angular `velocity` gains the
/// `acceleration`, its whole part stopping at `cap`, and moves the arc `angle`. Forwards
/// (a positive acceleration) the arc ends at angle 100h, backwards below 80h. The new
/// velocity and angle, and whether the arc ended.
pub fn swing(velocity: u32, acceleration: u32, cap: u16, angle: u16) -> (u32, u16, bool) {
    let forward = acceleration & 0x8000_0000 == 0;
    let mut velocity = velocity.wrapping_add(acceleration);
    let whole = (velocity >> 16) as u16;
    if forward == (whole.wrapping_sub(cap) & 0x8000 == 0) {
        velocity = (u32::from(cap) << 16) | (velocity & 0xffff);
    }
    let angle = angle.wrapping_add((velocity >> 16) as u16);
    let finished = match forward {
        true => angle.wrapping_sub(0x100) & 0x8000 == 0,
        false => angle.wrapping_sub(0x80) & 0x8000 != 0,
    };
    (velocity, angle, finished)
}
/// Ki-hunter function - winged - swoop ($A8:F3B8): swing along the arc to Samus, swiping
/// past the trigger angle, then back off at a block.
fn swoop(game: &mut (impl EnemyHost + LeaseActors), k: u16) {
    let hunter = i32::from(k);
    let forward = session_slot!(game, Kihunter[hunter].angular_acceleration) & 0x8000 == 0;
    let past_turn = (session_slot!(game, Kihunter[hunter].arc_angle) as i16)
        .wrapping_sub(session_slot!(game, Kihunter[hunter].swipe_trigger_angle) as i16)
        >= 0;
    if forward == past_turn && session_slot!(game, Kihunter[hunter].swipe) == 0 {
        session_slot!(game, Kihunter[hunter].swipe = 1);
        let list = if forward { 0xea32 } else { 0xea08 };
        session_slot!(game, Kihunter[hunter].base; current_instruction = list, instruction_timer = 1);
    }
    let acceleration = session_slot!(
        game,
        Kihunter[hunter](angular_acceleration, angular_subacceleration)
    );
    let velocity = session_slot!(
        game,
        Kihunter[hunter](angular_velocity, angular_subvelocity)
    );
    let cap = session_slot!(game, Kihunter[hunter].angular_velocity_cap);
    let angle = session_slot!(game, Kihunter[hunter].arc_angle);
    let (velocity, angle, finished_arc) = swing(velocity, acceleration, cap, angle);
    session_slot!(
        game,
        Kihunter[hunter](angular_velocity, angular_subvelocity) = velocity
    );
    session_slot!(game, Kihunter[hunter].arc_angle = angle);
    if finished_arc {
        return session_slot!(game, Kihunter[hunter].function = IDLE_FLYING);
    }
    let radius = session_slot!(game, Kihunter[hunter].arc_radius_x);
    let x = session_slot!(game, Kihunter[hunter].arc_centre_x) as i32
        + game.actor_cosine(angle, radius) as i32;
    let dx = (x - session_slot!(game, Kihunter[hunter].base.x_pos) as i32) as u16;
    let collided = if move_by(game, k, Axis::X, (dx as i16 as i32) << 16) {
        let forward = session_slot!(game, Kihunter[hunter].angular_acceleration) & 0x8000 == 0;
        session_slot!(game, Kihunter[hunter]; x_subvelocity = 0, x_velocity = if forward { u16::MAX } else { 1 });
        true
    } else {
        snap(game, k);
        let angle = session_slot!(game, Kihunter[hunter].arc_angle);
        let radius = session_slot!(game, Kihunter[hunter].arc_radius_y);
        let y = session_slot!(game, Kihunter[hunter].arc_centre_y) as i32
            + game.actor_sine(angle, radius) as i32;
        let dy = (y - session_slot!(game, Kihunter[hunter].base.y_pos) as i32) as u16;
        move_by(game, k, Axis::Y, (dy as i16 as i32) << 16)
    };
    if collided {
        session_slot!(game, Kihunter[hunter]; function = BACK_OFF, y_subvelocity = 0, y_velocity = u16::MAX);
    }
}
/// The Kihunter instructions: set idling instruction lists facing forwards ($A8:F526),
/// enemy function = hop ($A8:F5E4) or thinking ($A8:F67F), and fire acid spit left
/// ($A8:F6D2) or right ($A8:F6D8).
pub fn instruction(
    game: &mut (impl EnemyHost + LeaseActors),
    ea: u32,
    k: u16,
    j: EnemyScript,
) -> EnemyScript {
    let hunter = i32::from(k);
    match ea {
        0xa8_f526 => {
            let left = session_slot!(game, Kihunter[hunter].x_velocity) & 0x8000 != 0;
            let (next, attached) = if left {
                (0xe9fa, 0xea4e)
            } else {
                (0xea24, 0xea5e)
            };
            session_slot!(game, Kihunter[hunter].base; current_instruction = attached, instruction_timer = 1);
            if session_slot!(game, Wings[hunter + WINGS].function) == ATTACHED {
                session_slot!(game, Wings[hunter + WINGS].base; current_instruction = attached, instruction_timer = 1);
            }
            return EnemyScript::jump(next);
        }
        0xa8_f5e4 | 0xa8_f67f => {
            let function = if ea == 0xa8_f5e4 { HOP } else { THINKING };
            let current = i32::from(game.current_enemy());
            session_slot!(game, Kihunter[current].function = function);
        }
        0xa8_f6d2 | 0xa8_f6d8 => spit(game, if ea == 0xa8_f6d2 { 0xcf18 } else { 0xcf26 }),
        _ => {
            game.unknown_routine();
            return EnemyScript::Stop;
        }
    }
    j
}
fn move_by(game: &mut impl LeaseActors, k: u16, axis: Axis, amount: i32) -> bool {
    let mut lease = game.lease();
    let key = Key::from_legacy(k);
    let result = match axis {
        Axis::X => api::move_horizontal(&mut lease, key, amount, SlopePolicy::Ignore),
        Axis::Y => api::move_vertical(&mut lease, key, amount),
    };
    result == Blocked
}
fn move_velocity(game: &mut (impl EnemyHost + LeaseActors), k: u16, axis: Axis) -> bool {
    let hunter = i32::from(k);
    let amount = match axis {
        Axis::X => session_slot!(game, Kihunter[hunter](x_velocity, x_subvelocity)),
        Axis::Y => session_slot!(game, Kihunter[hunter](y_velocity, y_subvelocity)),
    };
    move_by(game, k, axis, amount as i32)
}
fn snap(game: &mut impl LeaseActors, k: u16) {
    api::snap_to_slopes(&mut game.lease(), Key::from_legacy(k));
}
/// A falling wing's X or Y: the arc start plus the arc's offset at its angle, less the
/// arc centre offset for its side.
fn wing_coordinate(
    game: &mut (impl EnemyHost + LeaseActors),
    wings: i32,
    x: bool,
    left: bool,
) -> u16 {
    let at = 0x7800 + wings as usize;
    let start = game.lease().lease_native_word(at + if x { 10 } else { 12 });
    let phase = u16::from(session_slot!(byte game, Wings[wings].arc_angle[1]));
    let displacement = if x {
        game.actor_cosine(phase, 0x30)
    } else {
        game.actor_sine(phase, 0x30)
    };
    let field = if left { 6 } else { 2 };
    let correction = game
        .lease()
        .lease_native_word(at + field + usize::from(!x) * 2);
    start.wrapping_add(displacement).wrapping_sub(correction)
}
