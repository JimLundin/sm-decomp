//! Kihunter, the winged insect that swoops at Samus and, once shot down to its wingless
//! form, hops and spits acid; its wings, in the next slot, flutter down (bank
//! $A8:E99A-$A8:F9BD).
#![forbid(unsafe_code)]
use crate::actors::api::Axis;
use crate::actors::boss_lease::BossLease;
use crate::actors::host::{Shot, Slopes, abs16};
use crate::actors::slot_view::{SlotState, slot_view};
use crate::enemy_lists::kihunter_wing_instruction_list;
use crate::enemy_script::EnemyScript;

slot_view! {
    /// The Kihunter's slot: its function, arc centre and angle, the swoop's angular
    /// velocity (cap, whole and sub words) and acceleration, its velocities (whole words
    /// over subwords), bobbing bounds, spawn position, swipe trigger angle and state,
    /// thinking timer, arc radii and whether it lost its wings.
    Kihunter, KihunterEdit {
        record: [
            function = var_a, arc_centre_x = var_b, arc_centre_y = var_c, arc_angle = var_f,
            parameter_1 = parameter_1,
        ],
        extended: [
            angular_velocity_cap = var_00, angular_velocity = var_02,
            angular_subvelocity = var_03, angular_acceleration = var_04,
            angular_subacceleration = var_05, x_subvelocity = var_06, x_velocity = var_07,
            y_subvelocity = var_08, y_velocity = var_09, min_y = var_0a, max_y = var_0b,
            spawn_x = var_0c, spawn_y = var_0d, swipe_trigger_angle = var_0e,
            thinking_timer = var_0f, swipe = var_10, arc_radius_x = var_11,
            arc_radius_y = var_12, wingless = var_14,
        ],
    }
}
slot_view! {
    /// The Kihunter wings' slot: their function and falling function, the fall's speed
    /// table index and its reset value, the fall phase and arc angle, each side's arc
    /// centre offsets, the arc and fall start positions and the arc's top offset.
    Wings, WingsEdit {
        record: [
            function = var_a, speed_table_index = var_b, fall_phase = var_d, arc_angle = var_f,
            parameter_1 = parameter_1,
        ],
        extended: [
            falling_function = var_00, right_offset_x = var_01, right_offset_y = var_02,
            left_offset_x = var_03, left_offset_y = var_04, arc_start_x = var_05,
            arc_start_y = var_06, fall_start_y = var_07, fall_start_x = var_08,
            speed_table_reset = var_0a, arc_top_offset = var_0b,
        ],
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
const WINGS: u16 = 64;

/// A lease on a game with the Kihunter's effects.
pub trait Lease: BossLease + SlotState {}
impl<T: BossLease + SlotState + ?Sized> Lease for T {}

/// Set up falling Kihunter wings drifting left ($A8:F947) or right ($A8:F96A).
fn begin_drift(game: &mut impl Lease, wings: u16, left: bool) {
    let falling = if left { DRIFTING_LEFT } else { DRIFTING_RIGHT };
    let view = Wings::of(game, wings);
    let index = view.speed_table_reset();
    let angle = if left { 0xe000 } else { 0xa000 };
    let (x, y) = (view.actor().x.whole, view.actor().y.whole);
    let mut edit = WingsEdit::of(game, wings);
    *edit.falling_function() = falling;
    *edit.speed_table_index() = index;
    *edit.arc_angle() = angle;
    *edit.arc_start_x() = x;
    *edit.arc_start_y() = y;
}
/// Kihunter wings falling function - drifting left ($A8:F7DB) or right ($A8:F8AD).
fn drift(game: &mut impl Lease, left: bool) {
    crate::probe::hit("kihunter.drift");
    let wings = game.current_enemy();
    let record = crate::cartridge::native_record(0xa2, (QUADRATIC_SPEEDS & 0xffff) as u16, game);
    let index = Wings::of(game, wings).speed_table_index() >> 8;
    let speed = record.word(8 * usize::from(index) + if left { 5 } else { 1 });
    {
        let mut edit = WingsEdit::of(game, wings);
        let angle = edit.arc_angle();
        *angle = angle.wrapping_add(speed);
    }
    let y = wing_coordinate(game, wings, false, left);
    if left {
        WingsEdit::of(game, wings).actor().y.whole = y;
    } else {
        let amount = (y.wrapping_sub(Wings::of(game, wings).actor().y.whole) as i16 as i32) << 16;
        let k = game.current_enemy();
        if game.move_vertical(k, amount) {
            let y = {
                let mut edit = WingsEdit::of(game, wings);
                edit.actor().properties |= 0x200;
                let x = *edit.fall_start_x();
                edit.actor().x.whole = x;
                let y = *edit.fall_start_y();
                edit.actor().y.whole = y;
                y
            };
            let current = game.current_enemy();
            *WingsEdit::of(game, current).speed_table_index() = y;
            return;
        }
    }
    let x = wing_coordinate(game, wings, true, left);
    WingsEdit::of(game, wings).actor().x.whole = x;
    if ((Wings::of(game, wings).arc_angle().wrapping_add(0x4000) & 0x8000) != 0) == left {
        let current = game.current_enemy();
        begin_drift(game, current, !left);
    } else {
        let speed = (Wings::of(game, wings).speed_table_index() as i16).wrapping_sub(384);
        let target = if left { wings } else { game.current_enemy() };
        let index = if speed < 0 { 256 } else { speed as u16 };
        *WingsEdit::of(game, target).speed_table_index() = index;
    }
}
/// Initialisation AI - Kihunter ($A8:F188).
pub fn initialize(game: &mut impl Lease) {
    let hunter = game.current_enemy();
    let mut edit = KihunterEdit::of(game, hunter);
    edit.actor().properties |= 0x2000;
    *edit.wingless() = 0;
    let actor = edit.actor();
    actor.instruction_delay = 1;
    actor.timer = 0;
    actor.instruction = 0xe9fa;
    *edit.arc_angle() = 0;
    *edit.function() = IDLE_FLYING;
    *edit.y_subvelocity() = 0;
    *edit.y_velocity() = 1;
    *edit.x_subvelocity() = 0;
    *edit.x_velocity() = u16::MAX;
    let min_y = edit.actor().y.whole.wrapping_sub(16);
    *edit.min_y() = min_y;
    *edit.max_y() = min_y.wrapping_add(32);
    let (x, y) = (edit.actor().x.whole, edit.actor().y.whole);
    *edit.spawn_x() = x;
    *edit.spawn_y() = y;
    if *edit.parameter_1() & 0x8000 != 0 {
        *edit.wingless() = 1;
        *edit.function() = INITIAL_FALLING;
        *edit.y_subvelocity() = 0;
        *edit.y_velocity() = 1;
    }
}
/// Initialisation AI - Kihunter wings ($A8:F214): take the body's position and palette.
pub fn initialize_wings(game: &mut impl Lease) {
    let wings = game.current_enemy();
    {
        let mut edit = WingsEdit::of(game, wings);
        let actor = edit.actor();
        actor.properties |= 0x2000;
        actor.instruction_delay = 1;
        actor.timer = 0;
        actor.instruction = 0xea4e;
    }
    follow_body(game, wings, false);
    *WingsEdit::of(game, wings).function() = ATTACHED;
    let body = Kihunter::of(game, wings - WINGS);
    let (palette, tile, parameter) = (body.actor().palette, body.actor().tile, body.parameter_1());
    let mut edit = WingsEdit::of(game, wings);
    edit.actor().palette = palette;
    edit.actor().tile = tile;
    if parameter & 0x8000 != 0 {
        edit.actor().properties |= 0x200;
    }
}
/// Copy the body's Y then X position to the wings (or X then Y).
fn follow_body(game: &mut impl Lease, wings: u16, x_first: bool) {
    for x in [x_first, !x_first] {
        let body = Kihunter::of(game, wings - WINGS).actor();
        let (x_pos, y_pos) = (body.x.whole, body.y.whole);
        let mut edit = WingsEdit::of(game, wings);
        match x {
            true => edit.actor().x.whole = x_pos,
            false => edit.actor().y.whole = y_pos,
        }
    }
}
/// Determine falling Kihunter wings speed table index reset value ($A8:F98D).
fn prepare_fall_speed(game: &mut impl Lease) {
    let wings = game.current_enemy();
    {
        let mut edit = WingsEdit::of(game, wings);
        *edit.speed_table_reset() = 0;
        *edit.speed_table_index() = 0;
    }
    loop {
        let speed = Wings::of(game, wings).speed_table_reset().wrapping_add(384);
        *WingsEdit::of(game, wings).speed_table_reset() = speed;
        let record =
            crate::cartridge::native_record(0xa2, (QUADRATIC_SPEEDS & 0xffff) as u16, game);
        let phase = record
            .word(8 * usize::from(speed >> 8) + 1)
            .wrapping_add(Wings::of(game, wings).fall_phase());
        *WingsEdit::of(game, wings).fall_phase() = phase;
        if phase.wrapping_sub(0x2000) & 0x8000 == 0 {
            break;
        }
    }
}
/// Enemy shot - Kihunter ($A8:F701): below its wings' health it loses them, which fall;
/// above it, the wings share its frozen state and flash; dead, the wings go too.
pub fn shot(game: &mut impl Lease) {
    game.shot_reaction(Shot::Finalize);
    let hunter = game.current_enemy();
    let wings = hunter + WINGS;
    let health = Kihunter::of(game, hunter).actor().health;
    if health == 0 {
        WingsEdit::of(game, wings).actor().properties = 512;
        return;
    }
    let threshold = Wings::of(game, wings).parameter_1();
    if health != threshold && (health as i16).wrapping_sub(threshold as i16) >= 0 {
        let body = Kihunter::of(game, hunter).actor();
        let (handlers, frozen, invincibility, flash) = (
            body.ai_handlers,
            body.frozen,
            body.invincibility,
            body.flash,
        );
        let mut edit = WingsEdit::of(game, wings);
        let actor = edit.actor();
        actor.ai_handlers = handlers;
        actor.frozen = frozen;
        actor.invincibility = invincibility;
        actor.flash = flash;
        return;
    }
    if Kihunter::of(game, hunter).wingless() != 0 {
        return;
    }
    {
        let mut edit = KihunterEdit::of(game, hunter);
        *edit.wingless() = 1;
        *edit.function() = INITIAL_FALLING;
        *edit.y_subvelocity() = 0;
        *edit.y_velocity() = 1;
    }
    let body = game.current_enemy();
    game.select_enemy(body.wrapping_add(64));
    if Wings::of(game, wings).function() != FALLING {
        let view = Wings::of(game, wings);
        let (x, y) = (view.actor().x.whole, view.actor().y.whole);
        {
            let mut edit = WingsEdit::of(game, wings);
            *edit.fall_start_y() = y;
            *edit.fall_start_x() = x;
        }
        prepare_fall_speed(game);
        wing_origins(game);
        let view = Wings::of(game, wings);
        let arc_y = view.fall_start_y().wrapping_sub(view.arc_top_offset());
        let x = view.actor().x.whole;
        let index = view.speed_table_reset();
        let mut edit = WingsEdit::of(game, wings);
        *edit.arc_angle() = 0xe000;
        *edit.function() = FALLING;
        *edit.falling_function() = DRIFTING_LEFT;
        *edit.arc_start_y() = arc_y;
        *edit.arc_start_x() = x;
        *edit.speed_table_index() = index;
        let actor = edit.actor();
        actor.instruction = 0xea7e;
        actor.spritemap = 0x804d;
        actor.instruction_delay = 1;
        actor.timer = 0;
        actor.properties |= 0x800;
    }
    game.select_enemy(body);
}
/// Fire acid spit ($A8:F6DC).
fn spit(game: &mut impl Lease, projectile: u16) {
    game.queue_sound(1, 0x4c, 6);
    game.spawn_actor_eproj(0, game.current_enemy(), projectile);
    let current = game.current_enemy();
    *KihunterEdit::of(game, current).thinking_timer() = 24;
}
/// Determine falling Kihunter wings left ($A8:F851) and right ($A8:F87F) arc centre offsets.
fn wing_origins(game: &mut impl Lease) {
    for (phase, left) in [(0xe0, true), (0xa0, false)] {
        let x = game.scaled_cosine(phase, 0x30);
        let wings = game.current_enemy();
        {
            let mut edit = WingsEdit::of(game, wings);
            *if left {
                edit.left_offset_x()
            } else {
                edit.right_offset_x()
            } = x;
        }
        let y = game.scaled_sine(phase, 0x30);
        let mut edit = WingsEdit::of(game, wings);
        *if left {
            edit.left_offset_y()
        } else {
            edit.right_offset_y()
        } = y;
    }
}
/// Set the body's and the wings' instruction lists to the pair at `index`.
fn set_paired_lists(game: &mut impl Lease, k: u16, index: u16) {
    for (offset, entry) in [(0, index), (WINGS, index + 1)] {
        let value = kihunter_wing_instruction_list(game, entry as isize);
        KihunterEdit::of(game, k + offset).actor().instruction = value;
    }
    for offset in [0, WINGS] {
        KihunterEdit::of(game, k + offset).actor().instruction_delay = 1;
    }
}
/// Add `amount` to the Kihunter's 16.16 Y velocity.
fn accelerate_y(game: &mut impl Lease, k: u16, amount: u32) {
    let mut edit = KihunterEdit::of(game, k);
    let velocity = (u32::from(*edit.y_velocity()) << 16) | u32::from(*edit.y_subvelocity());
    let sum = velocity.wrapping_add(amount);
    *edit.y_subvelocity() = sum as u16;
    *edit.y_velocity() = (sum >> 16) as u16;
}
/// Set the Kihunter's instruction list to `list`, to run next frame.
fn set_list(game: &mut impl Lease, k: u16, list: u16) {
    let mut edit = KihunterEdit::of(game, k);
    let actor = edit.actor();
    actor.instruction = list;
    actor.instruction_delay = 1;
}
/// Main AI - Kihunter ($A8:F25C) or Kihunter wings ($A8:F262): run its function.
pub fn main_ai(game: &mut impl Lease) {
    crate::probe::hit("kihunter.main_ai");
    let k = game.current_enemy();
    match Kihunter::of(game, k).function() {
        IDLE_FLYING => idle_flying(game, k),
        SWOOP => swoop(game, k),
        BACK_OFF => {
            if move_velocity(game, k, Axis::X)
                || {
                    snap(game, k);
                    move_velocity(game, k, Axis::Y)
                }
                || {
                    let hunter = Kihunter::of(game, k);
                    (hunter.actor().y.whole as i16).wrapping_sub(hunter.spawn_y() as i16) < 0
                }
            {
                *KihunterEdit::of(game, k).function() = IDLE_FLYING;
            }
        }
        INITIAL_FALLING => {
            if move_velocity(game, k, Axis::Y) {
                *KihunterEdit::of(game, k).function() = PREPARE_TO_HOP;
            } else {
                accelerate_y(game, k, 0xe000);
            }
        }
        PREPARE_TO_HOP => {
            {
                let mut edit = KihunterEdit::of(game, k);
                *edit.function() = IDLE;
                *edit.y_subvelocity() = 0;
            }
            let hop = ((game.current_random() as i32 & 1) - 8) as u16;
            *KihunterEdit::of(game, k).y_velocity() = hop;
            let x = Kihunter::of(game, k).actor().x.whole;
            let left = (x as i16).wrapping_sub(game.player_x() as i16) >= 0;
            {
                let mut edit = KihunterEdit::of(game, k);
                *edit.x_subvelocity() = 0;
                *edit.x_velocity() = if left { (-2i16) as u16 } else { 2 };
            }
            set_list(game, k, if left { 0xea8a } else { 0xeaa6 });
        }
        HOP => {
            if move_velocity(game, k, Axis::Y) {
                if Kihunter::of(game, k).y_velocity() & 0x8000 != 0 {
                    *KihunterEdit::of(game, k).y_velocity() = 1;
                } else {
                    let mut edit = KihunterEdit::of(game, k);
                    *edit.y_subvelocity() = 0;
                    *edit.y_velocity() = (-4i16) as u16;
                    *edit.function() = IDLE;
                    *edit.thinking_timer() = 12;
                    let list = edit.actor().instruction;
                    let list = if (list as i32 + 0x155a) & 0x8000 == 0 {
                        0xeada
                    } else {
                        0xeac2
                    };
                    let actor = edit.actor();
                    actor.instruction = list;
                    actor.instruction_delay = 1;
                }
            } else if move_velocity(game, k, Axis::X) {
                let mut edit = KihunterEdit::of(game, k);
                let velocity = edit.x_velocity();
                *velocity = velocity.wrapping_neg();
            } else {
                snap(game, k);
                accelerate_y(game, k, 0xe000);
            }
        }
        THINKING => {
            let timer = Kihunter::of(game, k).thinking_timer().wrapping_sub(1);
            *KihunterEdit::of(game, k).thinking_timer() = timer;
            if timer == 0 {
                let x = Kihunter::of(game, k).actor().x.whole;
                let distance = abs16(x.wrapping_sub(game.player_x()));
                let next = if (distance as i32 - 96) & 0x8000 != 0 {
                    FIRE_ACID_SPIT
                } else {
                    PREPARE_TO_HOP
                };
                *KihunterEdit::of(game, k).function() = next;
            }
        }
        FIRE_ACID_SPIT => {
            let x = Kihunter::of(game, k).actor().x.whole;
            let right = (x as i16).wrapping_sub(game.player_x() as i16) < 0;
            set_list(game, k, if right { 0xeb10 } else { 0xeaf2 });
            *KihunterEdit::of(game, k).function() = IDLE;
        }
        ATTACHED => follow_body(game, k, true),
        // Kihunter wings function - falling ($A8:F7CF).
        FALLING => match Wings::of(game, k).falling_function() {
            DRIFTING_LEFT => drift(game, true),
            DRIFTING_RIGHT => drift(game, false),
            _ => game.unreachable(),
        },
        IDLE => (),
        _ => game.unreachable(),
    }
}
/// Ki-hunter function - winged - idle flying ($A8:F268): bob between its bounds and pace,
/// and swoop at Samus when she comes near and below.
fn idle_flying(game: &mut impl Lease, k: u16) {
    let vertical = if move_velocity(game, k, Axis::Y) {
        Some(Kihunter::of(game, k).y_velocity().wrapping_neg())
    } else {
        let hunter = Kihunter::of(game, k);
        let y = hunter.actor().y.whole as i16;
        if y.wrapping_sub(hunter.min_y() as i16) < 0 {
            Some(1)
        } else if y.wrapping_sub(hunter.max_y() as i16) >= 0 {
            Some(u16::MAX)
        } else {
            None
        }
    };
    if let Some(speed) = vertical {
        *KihunterEdit::of(game, k).y_velocity() = speed;
    }
    if move_velocity(game, k, Axis::X) {
        let speed = Kihunter::of(game, k).x_velocity().wrapping_neg();
        *KihunterEdit::of(game, k).x_velocity() = speed;
        set_paired_lists(game, k, if speed & 0x8000 != 0 { 0 } else { 2 });
    }
    snap(game, k);
    let dx = game
        .player_x()
        .wrapping_sub(Kihunter::of(game, k).actor().x.whole);
    let distance = abs16(dx);
    let y = i32::from(Kihunter::of(game, k).actor().y.whole);
    let below = (i32::from(game.player_y()) - y - 32) & 0x8000 == 0;
    if (distance as i16).wrapping_sub(0x60) >= 0 || !below {
        return;
    }
    let dy = game
        .player_y()
        .wrapping_sub(Kihunter::of(game, k).actor().y.whole);
    let pair = {
        let mut edit = KihunterEdit::of(game, k);
        if dx & 0x8000 != 0 {
            *edit.angular_velocity_cap() = (-2i16) as u16;
            *edit.angular_velocity() = 0;
            *edit.angular_subvelocity() = 0;
            *edit.angular_acceleration() = u16::MAX;
            *edit.x_velocity() = u16::MAX;
            *edit.angular_subacceleration() = 0xe000;
            *edit.arc_angle() = 255;
            *edit.swipe_trigger_angle() = 240;
            0
        } else {
            *edit.angular_velocity_cap() = 2;
            *edit.angular_velocity() = 0;
            *edit.angular_subvelocity() = 0;
            *edit.angular_acceleration() = 0;
            *edit.angular_subacceleration() = 0x2000;
            *edit.arc_angle() = 128;
            *edit.x_velocity() = 1;
            *edit.swipe_trigger_angle() = 144;
            4
        }
    };
    set_paired_lists(game, k, pair >> 1);
    let mut edit = KihunterEdit::of(game, k);
    let (x, y) = (edit.actor().x.whole, edit.actor().y.whole);
    *edit.arc_centre_x() = dx.wrapping_add(x);
    *edit.arc_centre_y() = y;
    *edit.function() = SWOOP;
    *edit.arc_radius_y() = dy;
    *edit.arc_radius_x() = distance;
    *edit.swipe() = 0;
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
fn swoop(game: &mut impl Lease, k: u16) {
    crate::probe::hit("kihunter.swoop");
    let hunter = Kihunter::of(game, k);
    let forward = hunter.angular_acceleration() & 0x8000 == 0;
    let past_turn =
        (hunter.arc_angle() as i16).wrapping_sub(hunter.swipe_trigger_angle() as i16) >= 0;
    if forward == past_turn && hunter.swipe() == 0 {
        *KihunterEdit::of(game, k).swipe() = 1;
        set_list(game, k, if forward { 0xea32 } else { 0xea08 });
    }
    let hunter = Kihunter::of(game, k);
    let acceleration = (u32::from(hunter.angular_acceleration()) << 16)
        | u32::from(hunter.angular_subacceleration());
    let velocity =
        (u32::from(hunter.angular_velocity()) << 16) | u32::from(hunter.angular_subvelocity());
    let cap = hunter.angular_velocity_cap();
    let angle = hunter.arc_angle();
    let (velocity, angle, finished_arc) = swing(velocity, acceleration, cap, angle);
    {
        let mut edit = KihunterEdit::of(game, k);
        *edit.angular_subvelocity() = velocity as u16;
        *edit.angular_velocity() = (velocity >> 16) as u16;
        *edit.arc_angle() = angle;
        if finished_arc {
            *edit.function() = IDLE_FLYING;
            return;
        }
    }
    let radius = Kihunter::of(game, k).arc_radius_x();
    let centre = Kihunter::of(game, k).arc_centre_x();
    let x = centre as i32 + game.scaled_cosine(angle, radius) as i32;
    let dx = (x - Kihunter::of(game, k).actor().x.whole as i32) as u16;
    let collided = if move_by(game, k, Axis::X, (dx as i16 as i32) << 16) {
        let mut edit = KihunterEdit::of(game, k);
        let forward = *edit.angular_acceleration() & 0x8000 == 0;
        *edit.x_subvelocity() = 0;
        *edit.x_velocity() = if forward { u16::MAX } else { 1 };
        true
    } else {
        snap(game, k);
        let hunter = Kihunter::of(game, k);
        let (angle, radius, centre) = (
            hunter.arc_angle(),
            hunter.arc_radius_y(),
            hunter.arc_centre_y(),
        );
        let y = centre as i32 + game.scaled_sine(angle, radius) as i32;
        let dy = (y - Kihunter::of(game, k).actor().y.whole as i32) as u16;
        move_by(game, k, Axis::Y, (dy as i16 as i32) << 16)
    };
    if collided {
        let mut edit = KihunterEdit::of(game, k);
        *edit.function() = BACK_OFF;
        *edit.y_subvelocity() = 0;
        *edit.y_velocity() = u16::MAX;
    }
}
/// The Kihunter instructions: set idling instruction lists facing forwards ($A8:F526),
/// enemy function = hop ($A8:F5E4) or thinking ($A8:F67F), and fire acid spit left
/// ($A8:F6D2) or right ($A8:F6D8).
pub fn instruction(game: &mut impl Lease, ea: u32, k: u16, j: EnemyScript) -> EnemyScript {
    match ea {
        0xa8_f526 => {
            let left = Kihunter::of(game, k).x_velocity() & 0x8000 != 0;
            let (next, attached) = if left {
                (0xe9fa, 0xea4e)
            } else {
                (0xea24, 0xea5e)
            };
            set_list(game, k, attached);
            if Wings::of(game, k + WINGS).function() == ATTACHED {
                set_list(game, k + WINGS, attached);
            }
            return EnemyScript::jump(next);
        }
        0xa8_f5e4 | 0xa8_f67f => {
            let function = if ea == 0xa8_f5e4 { HOP } else { THINKING };
            let current = game.current_enemy();
            *KihunterEdit::of(game, current).function() = function;
        }
        0xa8_f6d2 | 0xa8_f6d8 => spit(game, if ea == 0xa8_f6d2 { 0xcf18 } else { 0xcf26 }),
        _ => game.unreachable(),
    }
    j
}
fn move_by(game: &mut impl Lease, k: u16, axis: Axis, amount: i32) -> bool {
    match axis {
        Axis::X => game.move_horizontal(k, amount, Slopes::Ignore),
        Axis::Y => game.move_vertical(k, amount),
    }
}
fn move_velocity(game: &mut impl Lease, k: u16, axis: Axis) -> bool {
    let hunter = Kihunter::of(game, k);
    let (whole, sub) = match axis {
        Axis::X => (hunter.x_velocity(), hunter.x_subvelocity()),
        Axis::Y => (hunter.y_velocity(), hunter.y_subvelocity()),
    };
    let amount = (u32::from(whole) << 16) | u32::from(sub);
    move_by(game, k, axis, amount as i32)
}
fn snap(game: &mut impl Lease, k: u16) {
    game.snap_to_slopes(k);
}
/// A falling wing's X or Y: the arc start plus the arc's offset at its angle, less the
/// arc centre offset for its side.
fn wing_coordinate(game: &mut impl Lease, wings: u16, x: bool, left: bool) -> u16 {
    let view = Wings::of(game, wings);
    let start = if x {
        view.arc_start_x()
    } else {
        view.arc_start_y()
    };
    let phase = view.arc_angle() >> 8;
    let displacement = if x {
        game.scaled_cosine(phase, 0x30)
    } else {
        game.scaled_sine(phase, 0x30)
    };
    let view = Wings::of(game, wings);
    let correction = match (left, x) {
        (true, true) => view.left_offset_x(),
        (true, false) => view.left_offset_y(),
        (false, true) => view.right_offset_x(),
        (false, false) => view.right_offset_y(),
    };
    start.wrapping_add(displacement).wrapping_sub(correction)
}
