//! Evir, Maridia's shrimp: a body bobbing towards Samus, its arms in the next slot and its
//! projectile in the one after, spat at Samus and regrown (bank $A8:8687-$A8:8F8B).
use crate::actors::access::{AccessMode::Native, ActorKey};
use crate::actors::api::{self, Axis, DeathHandling::Finalize, ShotHandling};
use crate::actors::combat::{self, current_slot};
use crate::actors::host::{ActorLease, BossEffects as _, LeaseActors};
use crate::actors::layouts::part_view;
use crate::enemy_projectiles::host::Effects as _;
use crate::enemy_script::EnemyScript;
use crate::session_slot;
use sm_data::enemy_speeds::SpeedTable::Linear;

part_view! {
    /// An evir part's slot and extended RAM; a non-zero first parameter makes the arms.
    Evir: EnemyLayout40 {
        words: [var_a, facing, function, var_d, movement_timer, regeneration_x_offset, parameter_1, parameter_2],
        pad1: [u8; 26696],
        words: [
            movement_direction, instruction_list, new_instruction_list, down_subvelocity, down_velocity,
            up_subvelocity, up_velocity, x_velocity, x_subvelocity, y_velocity, y_subvelocity, moving,
            regenerating,
        ],
        _: [u16; 19],
    }
}

/// Bank $A8 evir functions, stored in `function`.
pub mod function {
    pub const HANDLE_BODY_OR_ARMS: u16 = 0x8922;
    pub const PROJECTILE_IDLE: u16 = 0x8a34;
    pub const PROJECTILE_MOVING: u16 = 0x8a3b;
    pub const PROJECTILE_REGENERATING: u16 = 0x8a78;
}

/// Bank $A8 evir instructions and reactions; the shot runs the normal touch reaction and the
/// touch the normal shot reaction, as the source does.
pub mod routine {
    pub const QUEUE_SPIT_SOUND: u32 = 0xa8_878f;
    pub const SET_INITIAL_REGENERATION_X_OFFSET: u32 = 0xa8_879b;
    pub const ADVANCE_REGENERATION_X_OFFSET: u32 = 0xa8_87b6;
    pub const FINISH_REGENERATION: u32 = 0xa8_87cb;
    pub const SHOT: u32 = 0xa8_8b06;
    pub const POWER_BOMB: u32 = 0xa8_8b0c;
    pub const TOUCH: u32 = 0xa8_8b12;
}

/// Bank $A8 evir instruction lists: body and arms facing left and right, and the projectile.
const BODY_LISTS: [u16; 2] = [0x86a7, 0x870b];
const ARMS_LISTS: [u16; 2] = [0x86c3, 0x8727];
const PROJECTILE_LIST: u16 = 0x876f;
const REGENERATING_LIST: u16 = 0x8775;
/// The arms are the next slot after the body, the projectile the one after.
const SLOT: i32 = 0x40;
/// The arms hang 4 px either side of the body and Ah below, the projectile 12h below.
const SIDE_OFFSETS: [u16; 2] = [0xfffc, 4];
const ARMS_DROP: u16 = 10;
const PROJECTILE_DROP: u16 = 18;
const ARMS_LAYER: u16 = 4;
/// The projectile flies at speed 4 at Samus within 80h px columns, regrows once over 100h px
/// off screen, and starts regrowing 8 px out.
const PROJECTILE_SPEED: u16 = 4;
const SHOOTING_COLUMNS: u16 = 0x80;
const OFF_SCREEN_PADDING: u16 = 0x100;
const INITIAL_REGENERATION_X_OFFSETS: [u16; 2] = [8, 0xfff8];
const REGENERATION_X_STEPS: [u16; 2] = [0xffff, 1];
const QUARTER_TURN: u8 = 64;
const SPIT_SOUND: u16 = 0x5e;
/// Enemy properties bit 9: delete; AI handler bit 2: frozen AI.
const DELETE: u16 = 0x200;
const FROZEN_AI: u16 = 4;
const SIGN: u16 = 0x8000;

/// Set evir instruction list ($A8:8AE8) to `list`, if it changed.
fn set_instruction_list(runtime: &mut impl ActorLease, evir: i32, list: u16) {
    session_slot!(runtime, Evir[evir].new_instruction_list = list);
    if list != session_slot!(runtime, Evir[evir].instruction_list) {
        session_slot!(runtime, Evir[evir].instruction_list = list);
        let key = combat::current(runtime);
        runtime.edit_actor(key, Native, |actor| actor.restart_animation(list));
    }
}
/// Set evir facing direction ($A8:8838): towards Samus.
fn face_samus(runtime: &mut impl ActorLease, evir: i32) {
    let key = combat::current(runtime);
    let right = api::player_delta(runtime, key, Axis::X) & SIGN == 0;
    session_slot!(runtime, Evir[evir].facing = u16::from(right));
    set_instruction_list(runtime, evir, BODY_LISTS[usize::from(right)]);
}
/// Face the way of the body `slots` back, `drop` px below it and 4 px to that side.
fn follow_body(runtime: &mut impl ActorLease, evir: i32, slots: i32, drop: u16) -> usize {
    let body = evir - slots * SLOT;
    let facing = session_slot!(runtime, Evir[body].facing);
    session_slot!(runtime, Evir[evir].facing = facing);
    let right = usize::from(facing != 0);
    let x = session_slot!(runtime, Evir[body].base.x_pos).wrapping_add(SIDE_OFFSETS[right]);
    session_slot!(runtime, Evir[evir].base.x_pos = x);
    let y = session_slot!(runtime, Evir[body].base.y_pos).wrapping_add(drop);
    session_slot!(runtime, Evir[evir].base.y_pos = y);
    right
}
/// Handle evir arms ($A8:8866): hang from the body.
fn handle_arms(runtime: &mut impl ActorLease, evir: i32) {
    let right = follow_body(runtime, evir, 1, ARMS_DROP);
    set_instruction_list(runtime, evir, ARMS_LISTS[right]);
}
/// Initialisation AI - evir ($A8:87E0): the body or its arms.
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let runtime = &mut game.lease();
    let evir = current_slot(runtime);
    if session_slot!(runtime, Evir[evir].parameter_1) != 0 {
        handle_arms(runtime, evir);
        session_slot!(runtime, Evir[evir].base.layer = ARMS_LAYER);
    } else {
        face_samus(runtime, evir);
        // The second parameter's low byte picks the linear speed table entry.
        let entry = 4 * (session_slot!(runtime, Evir[evir].parameter_2) & 0xff);
        let speeds =
            [0, 1, 2, 3].map(|word| runtime.lease_enemy_speed(Linear, (entry + word) as isize));
        session_slot!(runtime, Evir[evir]; down_velocity = speeds[0], down_subvelocity = speeds[1],
            up_velocity = speeds[2], up_subvelocity = speeds[3]);
        // As in the source, the movement timer is half the second parameter's high byte of
        // the slot at the speed entry's byte offset, not this one's.
        let timer = session_slot!(byte runtime, Evir[i32::from(2 * entry)].parameter_2[1]) >> 1;
        session_slot!(runtime, Evir[evir].movement_timer = u16::from(timer));
    }
    session_slot!(runtime, Evir[evir]; movement_direction = 0, instruction_list = 0,
        function = function::HANDLE_BODY_OR_ARMS);
}
/// Initialisation AI - evir projectile ($A8:88B0).
pub fn initialize_projectile<H: LeaseActors + ?Sized>(game: &mut H) {
    let runtime = &mut game.lease();
    let evir = current_slot(runtime);
    set_instruction_list(runtime, evir, PROJECTILE_LIST);
    let palette = session_slot!(runtime, Evir[evir - 2 * SLOT].base.palette_index);
    let tiles = session_slot!(runtime, Evir[evir - 2 * SLOT].base.vram_tiles_index);
    session_slot!(runtime, Evir[evir].base; palette_index = palette, vram_tiles_index = tiles);
    follow_body(runtime, evir, 2, PROJECTILE_DROP);
    session_slot!(runtime, Evir[evir]; instruction_list = 0, regenerating = 0, moving = 0,
        regeneration_x_offset = 0, function = function::PROJECTILE_IDLE);
}
/// Handle evir body ($A8:8933): face Samus unless the projectile is regrowing, and bob.
fn handle_body(runtime: &mut impl ActorLease, evir: i32) {
    if session_slot!(runtime, Evir[evir + 2 * SLOT].regenerating) == 0 {
        face_samus(runtime, evir);
    }
    let speed = match session_slot!(runtime, Evir[evir].movement_direction) {
        0 => session_slot!(runtime, Evir[evir](up_velocity, up_subvelocity)),
        _ => session_slot!(runtime, Evir[evir](down_velocity, down_subvelocity)),
    };
    session_slot!(runtime, Evir[evir].base(y_pos, y_subpos) += speed);
    let timer = session_slot!(runtime, Evir[evir].movement_timer).wrapping_sub(1);
    session_slot!(runtime, Evir[evir].movement_timer = timer);
    if timer & SIGN != 0 {
        let timer = session_slot!(byte runtime, Evir[evir].parameter_2[1]);
        session_slot!(runtime, Evir[evir].movement_timer = u16::from(timer));
        let direction = session_slot!(runtime, Evir[evir].movement_direction) ^ 1;
        session_slot!(runtime, Evir[evir].movement_direction = direction);
    }
}
/// Shoot evir projectile at Samus ($A8:89D4), once she is near the body.
fn shoot(runtime: &mut impl ActorLease, evir: i32) {
    let body = ActorKey::from_legacy((evir - 2 * SLOT) as u16);
    if !api::player_within(runtime, body, Axis::X, SHOOTING_COLUMNS) {
        return;
    }
    let angle = api::angle_to_player(runtime, combat::current(runtime)) as u8;
    let angle = u16::from(angle.wrapping_sub(QUARTER_TURN)).wrapping_neg();
    let x = runtime.foreign(|game| game.actor_cosine_full(angle, PROJECTILE_SPEED));
    session_slot!(runtime, Evir[evir]; x_subvelocity = x as u16, x_velocity = (x >> 16) as u16);
    let y = runtime.foreign(|game| game.actor_sine_full(angle, PROJECTILE_SPEED));
    session_slot!(runtime, Evir[evir]; y_subvelocity = y as u16, y_velocity = (y >> 16) as u16);
    set_instruction_list(runtime, evir, PROJECTILE_LIST);
    session_slot!(runtime, Evir[evir]; moving = 1, function = function::PROJECTILE_MOVING);
}
fn body_frozen(runtime: &impl ActorLease, evir: i32) -> bool {
    session_slot!(runtime, Evir[evir - 2 * SLOT].base.frozen_timer) != 0
}
fn body_facing_right(runtime: &impl ActorLease, evir: i32) -> usize {
    usize::from(session_slot!(runtime, Evir[evir - 2 * SLOT].instruction_list) == BODY_LISTS[1])
}
/// Run the evir's function.
fn run_function(runtime: &mut impl ActorLease, evir: i32) {
    match session_slot!(runtime, Evir[evir].function) {
        // Evir function - handle body/arms ($A8:8922).
        function::HANDLE_BODY_OR_ARMS => match session_slot!(runtime, Evir[evir].parameter_1) {
            0 => handle_body(runtime, evir),
            _ => handle_arms(runtime, evir),
        },
        // Evir projectile function - idle ($A8:8A34): reset evir projectile position ($A8:88E5).
        function::PROJECTILE_IDLE => {
            follow_body(runtime, evir, 2, PROJECTILE_DROP);
        }
        // Evir projectile function - moving ($A8:8A3B), first starting evir projectile
        // regenerating if far off-screen ($A8:8AB1) unless the body is frozen.
        function::PROJECTILE_MOVING => {
            if api::current_outside_padded_screen(runtime, OFF_SCREEN_PADDING)
                && !body_frozen(runtime, evir)
            {
                session_slot!(runtime, Evir[evir]; moving = 0, regenerating = 1,
                    function = function::PROJECTILE_REGENERATING, regenerating = 1);
                set_instruction_list(runtime, evir, REGENERATING_LIST);
            }
            let x = session_slot!(runtime, Evir[evir](x_velocity, x_subvelocity));
            session_slot!(runtime, Evir[evir].base(x_pos, x_subpos) += x);
            let y = session_slot!(runtime, Evir[evir](y_velocity, y_subvelocity));
            session_slot!(runtime, Evir[evir].base(y_pos, y_subpos) += y);
        }
        // Evir projectile function - regenerating ($A8:8A78), unless the body is frozen.
        function::PROJECTILE_REGENERATING if body_frozen(runtime, evir) => {}
        function::PROJECTILE_REGENERATING
            if session_slot!(runtime, Evir[evir].regenerating) != 0 =>
        {
            follow_body(runtime, evir, 2, PROJECTILE_DROP);
            let offset = session_slot!(runtime, Evir[evir].regeneration_x_offset);
            session_slot!(runtime, Evir[evir].base.x_pos, |x| x.wrapping_add(offset));
        }
        function::PROJECTILE_REGENERATING => {
            set_instruction_list(runtime, evir, PROJECTILE_LIST);
            session_slot!(runtime, Evir[evir]; regenerating = 0, moving = 0, function = function::PROJECTILE_IDLE);
        }
        _ => runtime.unreachable(),
    }
}
/// Main AI - evir ($A8:891B).
pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let evir = current_slot(&lease);
    run_function(&mut lease, evir);
}
/// Main AI - evir projectile ($A8:899E): unless frozen, keep its moving or regrowing list,
/// or shoot; then run its function.
pub fn projectile_main_ai<H: LeaseActors + ?Sized>(game: &mut H) {
    let runtime = &mut game.lease();
    let evir = current_slot(runtime);
    if session_slot!(runtime, Evir[evir].base.frozen_timer) == 0 {
        if session_slot!(runtime, Evir[evir].moving) != 0 {
            set_instruction_list(runtime, evir, PROJECTILE_LIST);
        } else if session_slot!(runtime, Evir[evir].regenerating) != 0 {
            set_instruction_list(runtime, evir, REGENERATING_LIST);
        } else {
            shoot(runtime, evir);
        }
    }
    run_function(runtime, evir);
}
/// Enemy shot, power bomb reaction or enemy touch - evir, then the evir shared contact
/// reaction ($A8:8B16): a dead body takes its parts with it, and a frozen one freezes its
/// arms and its projectile unless that is in flight.
pub fn react<H: LeaseActors + ?Sized>(address: u32, game: &mut H) {
    let runtime = &mut game.lease();
    match address {
        routine::SHOT => combat::contact_reaction(runtime, Finalize),
        routine::POWER_BOMB => combat::power_bomb_reaction(runtime, Finalize),
        routine::TOUCH => combat::shot_reaction(runtime, ShotHandling::Finalize),
        _ => return runtime.unreachable(),
    }
    let evir = current_slot(runtime);
    let (arms, projectile) = (evir + SLOT, evir + 2 * SLOT);
    if session_slot!(runtime, Evir[evir].base.health) == 0 {
        for part in [arms, projectile] {
            session_slot!(runtime, Evir[part].base.properties, |p| p | DELETE);
        }
    }
    let frozen = session_slot!(runtime, Evir[evir].base.frozen_timer);
    if frozen != 0 {
        freeze(runtime, arms, frozen);
        if session_slot!(runtime, Evir[projectile].function) != function::PROJECTILE_MOVING {
            freeze(runtime, projectile, frozen);
        }
    }
}
fn freeze(runtime: &mut impl ActorLease, part: i32, frozen: u16) {
    session_slot!(runtime, Evir[part].base.frozen_timer = frozen);
    session_slot!(runtime, Evir[part].base.ai_handler_bits, |b| b | FROZEN_AI);
}
/// The evir instructions: queue the spit sound, set or advance the regeneration X offset
/// away from the body's facing, and finish regrowing.
pub fn instruction<H: LeaseActors + ?Sized>(
    address: u32,
    script: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    let runtime = &mut game.lease();
    let evir = current_slot(runtime);
    match address {
        routine::QUEUE_SPIT_SOUND => runtime.queue_sound(2, 6, SPIT_SOUND),
        routine::SET_INITIAL_REGENERATION_X_OFFSET => {
            let offset = INITIAL_REGENERATION_X_OFFSETS[body_facing_right(runtime, evir)];
            session_slot!(runtime, Evir[evir].regeneration_x_offset = offset);
        }
        routine::ADVANCE_REGENERATION_X_OFFSET => {
            let step = REGENERATION_X_STEPS[body_facing_right(runtime, evir)];
            let offset =
                session_slot!(runtime, Evir[evir].regeneration_x_offset).wrapping_add(step);
            session_slot!(runtime, Evir[evir].regeneration_x_offset = offset);
        }
        routine::FINISH_REGENERATION => {
            session_slot!(runtime, Evir[evir]; regenerating = 0, moving = 0, function = function::PROJECTILE_IDLE);
        }
        _ => {
            runtime.unreachable();
            return EnemyScript::Stop;
        }
    }
    script
}
