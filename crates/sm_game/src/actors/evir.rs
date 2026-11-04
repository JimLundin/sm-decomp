//! Evir, Maridia's shrimp: a body bobbing towards Samus, its arms in the next slot and its
//! projectile in the one after, spat at Samus and regrown (bank $A8:8687-$A8:8F8B).
use crate::actors::access::{AccessMode::Native, ActorKey};
use crate::actors::api::{self, Axis, DeathHandling::Finalize, ShotHandling};
use crate::actors::combat;
use crate::actors::host::ActorLease;
use crate::actors::layouts::EnemyLayout40;
use crate::actors::queries;
use crate::actors::slot_view::{SlotState, slot_view};
use crate::enemy_script::EnemyScript;
use crate::legacy_overflow;
use core::mem::offset_of;
use sm_data::enemy_speeds::SpeedTable::Linear;

/// A lease on a game whose slots are the Session's typed storage.
pub trait Lease: ActorLease + SlotState + crate::actors::boss_lease::BossLease {}
impl<T: ActorLease + SlotState + crate::actors::boss_lease::BossLease + ?Sized> Lease for T {}

/// Run `action` on the lease where a `foreign` call ran it on the game,
/// keeping that call's observation refresh on entry and return.
fn owned_step<L: Lease, T>(runtime: &mut L, action: impl FnOnce(&mut L) -> T) -> T {
    crate::actors::boss_lease::BossLease::owned_step(runtime, action)
}

slot_view! {
    /// An evir part's slot and extended RAM; a non-zero first parameter makes the arms.
    Evir, EvirEdit {
        record: [
            facing = var_b, function = var_c, movement_timer = var_e,
            regeneration_x_offset = var_f, parameter_1 = parameter_1, parameter_2 = parameter_2,
        ],
        extended: [
            movement_direction = var_00, instruction_list = var_01, new_instruction_list = var_02,
            down_subvelocity = var_03, down_velocity = var_04, up_subvelocity = var_05,
            up_velocity = var_06, x_velocity = var_07, x_subvelocity = var_08, y_velocity = var_09,
            y_subvelocity = var_0a, moving = var_0b, regenerating = var_0c,
        ],
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
const SLOT: u16 = 0x40;
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
fn set_instruction_list(runtime: &mut impl Lease, evir: u16, list: u16) {
    *EvirEdit::of(runtime, evir).new_instruction_list() = list;
    if list != Evir::of(runtime, evir).instruction_list() {
        *EvirEdit::of(runtime, evir).instruction_list() = list;
        let key = combat::current(runtime);
        runtime.edit_actor(key, Native, |actor| actor.restart_animation(list));
    }
}
/// Set evir facing direction ($A8:8838): towards Samus.
fn face_samus(runtime: &mut impl Lease, evir: u16) {
    let key = combat::current(runtime);
    let right = api::player_delta(runtime, key, Axis::X) & SIGN == 0;
    *EvirEdit::of(runtime, evir).facing() = u16::from(right);
    set_instruction_list(runtime, evir, BODY_LISTS[usize::from(right)]);
}
/// Face the way of the body `slots` back, `drop` px below it and 4 px to that side.
fn follow_body(runtime: &mut impl Lease, evir: u16, slots: u16, drop: u16) -> usize {
    let body = evir - slots * SLOT;
    let facing = Evir::of(runtime, body).facing();
    *EvirEdit::of(runtime, evir).facing() = facing;
    let right = usize::from(facing != 0);
    let x = Evir::of(runtime, body)
        .actor()
        .x
        .whole
        .wrapping_add(SIDE_OFFSETS[right]);
    EvirEdit::of(runtime, evir).actor().x.whole = x;
    let y = Evir::of(runtime, body).actor().y.whole.wrapping_add(drop);
    EvirEdit::of(runtime, evir).actor().y.whole = y;
    right
}
/// Handle evir arms ($A8:8866): hang from the body.
fn handle_arms(runtime: &mut impl Lease, evir: u16) {
    let right = follow_body(runtime, evir, 1, ARMS_DROP);
    set_instruction_list(runtime, evir, ARMS_LISTS[right]);
}
/// Initialisation AI - evir ($A8:87E0): the body or its arms.
pub fn initialize(runtime: &mut impl Lease) {
    let evir = combat::current(runtime).byte_index();
    if Evir::of(runtime, evir).parameter_1() != 0 {
        handle_arms(runtime, evir);
        EvirEdit::of(runtime, evir).actor().layer = ARMS_LAYER;
    } else {
        face_samus(runtime, evir);
        // The second parameter's low byte picks the linear speed table entry.
        let entry = 4 * (Evir::of(runtime, evir).parameter_2() & 0xff);
        let speeds =
            [0, 1, 2, 3].map(|word| runtime.lease_enemy_speed(Linear, (entry + word) as isize));
        {
            let mut part = EvirEdit::of(runtime, evir);
            *part.down_velocity() = speeds[0];
            *part.down_subvelocity() = speeds[1];
            *part.up_velocity() = speeds[2];
            *part.up_subvelocity() = speeds[3];
        }
        // As in the source, the movement timer is half the second parameter's high byte of
        // the slot at the speed entry's byte offset, not this one's.
        // LEGACY(original-overflow): that offset (8 * entry, up to 7F8h) is any
        // byte of the enemy table, not a slot's own parameter.
        let at = 0x0f78 + offset_of!(EnemyLayout40, parameter_2) + 1 + usize::from(2 * entry);
        let timer = legacy_overflow::slot_byte(runtime, at) >> 1;
        *EvirEdit::of(runtime, evir).movement_timer() = u16::from(timer);
    }
    let mut part = EvirEdit::of(runtime, evir);
    *part.movement_direction() = 0;
    *part.instruction_list() = 0;
    *part.function() = function::HANDLE_BODY_OR_ARMS;
}
/// Initialisation AI - evir projectile ($A8:88B0).
pub fn initialize_projectile(runtime: &mut impl Lease) {
    let evir = combat::current(runtime).byte_index();
    set_instruction_list(runtime, evir, PROJECTILE_LIST);
    let palette = Evir::of(runtime, evir - 2 * SLOT).actor().palette;
    let tiles = Evir::of(runtime, evir - 2 * SLOT).actor().tile;
    {
        let mut part = EvirEdit::of(runtime, evir);
        part.actor().palette = palette;
        part.actor().tile = tiles;
    }
    follow_body(runtime, evir, 2, PROJECTILE_DROP);
    let mut part = EvirEdit::of(runtime, evir);
    *part.instruction_list() = 0;
    *part.regenerating() = 0;
    *part.moving() = 0;
    *part.regeneration_x_offset() = 0;
    *part.function() = function::PROJECTILE_IDLE;
}
/// Handle evir body ($A8:8933): face Samus unless the projectile is regrowing, and bob.
fn handle_body(runtime: &mut impl Lease, evir: u16) {
    if Evir::of(runtime, evir + 2 * SLOT).regenerating() == 0 {
        face_samus(runtime, evir);
    }
    let body = Evir::of(runtime, evir);
    let (velocity, subvelocity) = match body.movement_direction() {
        0 => (body.up_velocity(), body.up_subvelocity()),
        _ => (body.down_velocity(), body.down_subvelocity()),
    };
    let speed = (u32::from(velocity) << 16) | u32::from(subvelocity);
    EvirEdit::of(runtime, evir)
        .actor()
        .y
        .advance(speed.cast_signed());
    let timer = Evir::of(runtime, evir).movement_timer().wrapping_sub(1);
    *EvirEdit::of(runtime, evir).movement_timer() = timer;
    if timer & SIGN != 0 {
        let timer = Evir::of(runtime, evir).parameter_2().to_le_bytes()[1];
        *EvirEdit::of(runtime, evir).movement_timer() = u16::from(timer);
        let direction = Evir::of(runtime, evir).movement_direction() ^ 1;
        *EvirEdit::of(runtime, evir).movement_direction() = direction;
    }
}
/// Shoot evir projectile at Samus ($A8:89D4), once she is near the body.
fn shoot(runtime: &mut impl Lease, evir: u16) {
    let body = ActorKey::from_legacy(evir - 2 * SLOT);
    if !api::player_within(runtime, body, Axis::X, SHOOTING_COLUMNS) {
        return;
    }
    let angle = api::angle_to_player(runtime, combat::current(runtime)) as u8;
    let angle = u16::from(angle.wrapping_sub(QUARTER_TURN)).wrapping_neg();
    let cosine = u16::from(angle.wrapping_add(64) as u8);
    let x = owned_step(runtime, |runtime| {
        queries::sine_product(runtime, PROJECTILE_SPEED, cosine)
    });
    {
        let mut part = EvirEdit::of(runtime, evir);
        *part.x_subvelocity() = x as u16;
        *part.x_velocity() = (x >> 16) as u16;
    }
    let sine = u16::from(angle.wrapping_add(0x80) as u8);
    let y = owned_step(runtime, |runtime| {
        queries::sine_product(runtime, PROJECTILE_SPEED, sine)
    });
    {
        let mut part = EvirEdit::of(runtime, evir);
        *part.y_subvelocity() = y as u16;
        *part.y_velocity() = (y >> 16) as u16;
    }
    set_instruction_list(runtime, evir, PROJECTILE_LIST);
    let mut part = EvirEdit::of(runtime, evir);
    *part.moving() = 1;
    *part.function() = function::PROJECTILE_MOVING;
}
fn body_frozen(runtime: &impl Lease, evir: u16) -> bool {
    Evir::of(runtime, evir - 2 * SLOT).actor().frozen != 0
}
fn body_facing_right(runtime: &impl Lease, evir: u16) -> usize {
    usize::from(Evir::of(runtime, evir - 2 * SLOT).instruction_list() == BODY_LISTS[1])
}
/// Run the evir's function.
fn run_function(runtime: &mut impl Lease, evir: u16) {
    match Evir::of(runtime, evir).function() {
        // Evir function - handle body/arms ($A8:8922).
        function::HANDLE_BODY_OR_ARMS => match Evir::of(runtime, evir).parameter_1() {
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
                {
                    let mut part = EvirEdit::of(runtime, evir);
                    *part.moving() = 0;
                    *part.regenerating() = 1;
                    *part.function() = function::PROJECTILE_REGENERATING;
                    *part.regenerating() = 1;
                }
                set_instruction_list(runtime, evir, REGENERATING_LIST);
            }
            let projectile = Evir::of(runtime, evir);
            let x =
                (u32::from(projectile.x_velocity()) << 16) | u32::from(projectile.x_subvelocity());
            EvirEdit::of(runtime, evir)
                .actor()
                .x
                .advance(x.cast_signed());
            let projectile = Evir::of(runtime, evir);
            let y =
                (u32::from(projectile.y_velocity()) << 16) | u32::from(projectile.y_subvelocity());
            EvirEdit::of(runtime, evir)
                .actor()
                .y
                .advance(y.cast_signed());
        }
        // Evir projectile function - regenerating ($A8:8A78), unless the body is frozen.
        function::PROJECTILE_REGENERATING if body_frozen(runtime, evir) => {}
        function::PROJECTILE_REGENERATING if Evir::of(runtime, evir).regenerating() != 0 => {
            follow_body(runtime, evir, 2, PROJECTILE_DROP);
            let offset = Evir::of(runtime, evir).regeneration_x_offset();
            let mut part = EvirEdit::of(runtime, evir);
            let old = part.actor().x.whole;
            part.actor().x.whole = old.wrapping_add(offset);
        }
        function::PROJECTILE_REGENERATING => {
            set_instruction_list(runtime, evir, PROJECTILE_LIST);
            let mut part = EvirEdit::of(runtime, evir);
            *part.regenerating() = 0;
            *part.moving() = 0;
            *part.function() = function::PROJECTILE_IDLE;
        }
        _ => runtime.unreachable(),
    }
}
/// Main AI - evir ($A8:891B).
pub fn main_ai(lease: &mut impl crate::actors::species::TableLease) {
    crate::probe::hit("evir.main_ai");
    let evir = combat::current(&*lease).byte_index();
    run_function(lease, evir);
}
/// Main AI - evir projectile ($A8:899E): unless frozen, keep its moving or regrowing list,
/// or shoot; then run its function.
pub fn projectile_main_ai(runtime: &mut impl Lease) {
    crate::probe::hit("evir.projectile");
    let evir = combat::current(runtime).byte_index();
    if Evir::of(runtime, evir).actor().frozen == 0 {
        if Evir::of(runtime, evir).moving() != 0 {
            set_instruction_list(runtime, evir, PROJECTILE_LIST);
        } else if Evir::of(runtime, evir).regenerating() != 0 {
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
pub fn react(address: u32, runtime: &mut impl Lease) {
    match address {
        routine::SHOT => combat::contact_reaction(runtime, Finalize),
        routine::POWER_BOMB => combat::power_bomb_reaction(runtime, Finalize),
        routine::TOUCH => combat::shot_reaction(runtime, ShotHandling::Finalize),
        _ => runtime.unreachable(),
    }
    let evir = combat::current(runtime).byte_index();
    let (arms, projectile) = (evir + SLOT, evir + 2 * SLOT);
    if Evir::of(runtime, evir).actor().health == 0 {
        for part in [arms, projectile] {
            EvirEdit::of(runtime, part).actor().properties |= DELETE;
        }
    }
    let frozen = Evir::of(runtime, evir).actor().frozen;
    if frozen != 0 {
        freeze(runtime, arms, frozen);
        if Evir::of(runtime, projectile).function() != function::PROJECTILE_MOVING {
            freeze(runtime, projectile, frozen);
        }
    }
}
fn freeze(runtime: &mut impl Lease, part: u16, frozen: u16) {
    EvirEdit::of(runtime, part).actor().frozen = frozen;
    EvirEdit::of(runtime, part).actor().ai_handlers |= FROZEN_AI;
}
/// The evir instructions: queue the spit sound, set or advance the regeneration X offset
/// away from the body's facing, and finish regrowing.
pub fn instruction(address: u32, script: EnemyScript, runtime: &mut impl Lease) -> EnemyScript {
    let evir = combat::current(runtime).byte_index();
    match address {
        routine::QUEUE_SPIT_SOUND => {
            crate::enemy_projectiles::host::Effects::queue_sound(runtime, 2, 6, SPIT_SOUND)
        }
        routine::SET_INITIAL_REGENERATION_X_OFFSET => {
            let offset = INITIAL_REGENERATION_X_OFFSETS[body_facing_right(runtime, evir)];
            *EvirEdit::of(runtime, evir).regeneration_x_offset() = offset;
        }
        routine::ADVANCE_REGENERATION_X_OFFSET => {
            let step = REGENERATION_X_STEPS[body_facing_right(runtime, evir)];
            let offset = Evir::of(runtime, evir)
                .regeneration_x_offset()
                .wrapping_add(step);
            *EvirEdit::of(runtime, evir).regeneration_x_offset() = offset;
        }
        routine::FINISH_REGENERATION => {
            let mut part = EvirEdit::of(runtime, evir);
            *part.regenerating() = 0;
            *part.moving() = 0;
            *part.function() = function::PROJECTILE_IDLE;
        }
        _ => runtime.unreachable(),
    }
    script
}
