//! Bomb and Golden Torizo: walking, the low-health drool and Golden Torizo's
//! shot reactions.
#![forbid(unsafe_code)]
use crate::actors::layouts::part_view;
use crate::constants::enemy_properties::kEnemyProps_Tangible;
use crate::enemy_script::EnemyScript;
use crate::ram::{CompatibilityRam, RawRam as _, WorkRamAccess, slot, word};

/// What the Torizos need beyond work RAM: actor movement, projectiles and the
/// shared shot reaction, which the compatibility runtime still implements.
pub trait Host: WorkRamAccess + CompatibilityRam + crate::player_position::PlayerPosition {
    /// Move actor `k` by `amount` (16.16) horizontally, ignoring slopes; true when blocked.
    fn move_horizontal_ignoring_slopes(&mut self, k: u16, amount: i32) -> bool;
    fn snap_to_slopes(&mut self, k: u16);
    /// Spawn projectile `definition` from actor `k` with `parameter`.
    fn spawn_projectile(&mut self, parameter: u16, k: u16, definition: u16);
    /// The common shot reaction, without death handling.
    fn shot_reaction_only(&mut self);
}

part_view! {
    /// A Torizo's view of its slot.
    TorizoSlot: EnemyLayout40 {
        words: [x_velocity, var_B, var_C, var_D, function, var_F, direction, damage],
        pad1: [u8; 26696],
        words: [
            var_00, var_01, var_02, turn_around_timer, var_04, var_05, var_06, var_07, var_08,
            var_09, var_0A, var_0B, var_0C, var_0D, var_0E, var_0F, var_10, var_11, var_12, var_13,
            var_14, var_15, var_16, var_17, var_18, var_19, var_1A, var_1B, var_1C, var_1D, var_1E,
            var_1F,
        ],
    }
}

/// Bank $AA instruction lists the routines select.
pub mod instruction_list {
    /// $AA:B1C8: the death sequence.
    pub const DEATH_SEQUENCE: u16 = 0xb1c8;
    /// Caught super missile, by facing and leg forward.
    pub const CAUGHT_SUPER_MISSILE_LEFT_LEFT_LEG: u16 = 0xcde1;
    pub const CAUGHT_SUPER_MISSILE_LEFT_RIGHT_LEG: u16 = 0xce43;
    pub const CAUGHT_SUPER_MISSILE_RIGHT_RIGHT_LEG: u16 = 0xcea5;
    pub const CAUGHT_SUPER_MISSILE_RIGHT_LEFT_LEG: u16 = 0xceff;
    /// Golden Torizo's dodge.
    pub const DODGE_TURNING_LEFT: u16 = 0xd1f1;
    pub const DODGE_TURNING_RIGHT: u16 = 0xd2ad;
}

/// Bank $86: the Bomb Torizo's low-health initial drool projectile.
pub const LOW_HEALTH_DROOL: u16 = 0xa969;
/// $AA:C6AB, an RTS: the function of a Torizo that has caught or dodged a shot.
const NO_FUNCTION: u16 = 0xc6ab;

/// `direction` bit 15: facing right; bit 13: the background leg is forward.
const FACING_RIGHT: u16 = 0x8000;
const BACKGROUND_LEG_FORWARD: u16 = 0x2000;
/// `damage` bits 14-15: gut and face blown up; bit 12: caught a super missile.
const GUT_AND_FACE_BLOWN_UP: u16 = 0xc000;
const CAUGHT_SUPER_MISSILE: u16 = 0x1000;
/// A sign test on the low 16 bits of a facing and offset XOR.
const SIGN: i32 = 0x8000;
/// Projectile direction bit 4 deletes the projectile.
const DELETE_PROJECTILE: u16 = 0x10;

/// The drool appears at random draws with none of these bits, in Crateria,
/// while health is below `LOW_HEALTH`.
const DROOL_CHANCE: u16 = 0x8142;
const CRATERIA: u16 = 0;
const LOW_HEALTH: u16 = 350;

/// $AA:C4BD: X velocities by step index; the first ten move left, the rest right.
const WALK_X_VELOCITIES: [i16; 20] = [
    -5, 0, -5, -19, -16, -7, 0, -7, -17, -18, 5, 0, 5, 19, 16, 7, 0, 7, 17, 18,
];

/// Instruction - Bomb Torizo walking movement ($AA:C470 normal, $AA:C4E5
/// faceless): one step at the X velocity for the step index in `jp`. A wall
/// turns the Torizo round; otherwise it aligns with slopes and, facing away
/// from Samus, starts the `pause` turn around timer. Was `torizo_stride`.
pub fn walk(
    k: u16,
    jp: EnemyScript,
    blocked_left: u16,
    blocked_right: u16,
    pause: u16,
    game: &mut impl Host,
) -> EnemyScript {
    let e = i32::from(k);
    slot!(
        game,
        TorizoSlot[e].x_velocity = WALK_X_VELOCITIES[(jp.word(0) >> 1) as usize] as u16
    );
    let amount = (slot!(game, TorizoSlot[e].x_velocity) as i16 as i32) << 16;
    let blocked = game.move_horizontal_ignoring_slopes(k, amount);
    if blocked {
        slot!(game, TorizoSlot[e].turn_around_timer = 0);
        if slot!(game, TorizoSlot[e].direction) & FACING_RIGHT != 0 {
            EnemyScript::jump(blocked_left)
        } else {
            EnemyScript::jump(blocked_right)
        }
    } else {
        game.snap_to_slopes(k);
        if (slot!(game, TorizoSlot[e].direction) as i32
            ^ game.player_x() as i32 - slot!(game, TorizoSlot[e].base.x_pos) as i32)
            & SIGN
            == 0
            && slot!(game, TorizoSlot[e].turn_around_timer) == 0
        {
            slot!(game, TorizoSlot[e].turn_around_timer = pause);
        }
        jp.offset(1)
    }
}

/// Handle low-health initial drool ($AA:C620): in Crateria, now and then
/// drool while health is low. Was `Torizo_C620`.
pub fn drool_at_low_health(k: u16, game: &mut impl Host) {
    if word!(game.area_index) == CRATERIA && word!(game.random_number) & DROOL_CHANCE == 0 {
        let health: u16 = slot!(game, TorizoSlot[k as i32].base.health);
        if health != 0 && health < LOW_HEALTH {
            game.spawn_projectile(health, k, LOW_HEALTH_DROOL);
        }
    }
}

/// Check if torizo is facing Samus ($AA:D3A7): the direction word XOR the
/// Torizo's offset from Samus, whose bit 15 is clear when it faces her. Was
/// `Torizo_Func_12`.
pub fn facing_against_samus(k: u16, game: &mut impl Host) -> u16 {
    let e = i32::from(k);
    (slot!(game, TorizoSlot[e].direction) as i32
        ^ slot!(game, TorizoSlot[e].base.x_pos) as i32 - game.player_x() as i32) as u16
}

/// Golden Torizo shot reaction - damaged ($AA:D6A6): on death, start the death
/// sequence. Was `Torizo_D6A6`.
pub fn golden_damaged_shot_reaction(game: &mut impl Host) {
    game.shot_reaction_only();
    let e = word!(game.cur_enemy_index) as i32;
    if slot!(game, TorizoSlot[e].base.health) == 0 {
        slot!(
            game,
            TorizoSlot[e].base.current_instruction = instruction_list::DEATH_SEQUENCE
        );
        slot!(game, TorizoSlot[e].base.instruction_timer = 1);
        slot!(
            game,
            TorizoSlot[e].damage = slot!(game, TorizoSlot[e].damage) | GUT_AND_FACE_BLOWN_UP
        );
        slot!(
            game,
            TorizoSlot[e].base.properties =
                slot!(game, TorizoSlot[e].base.properties) | kEnemyProps_Tangible as u16
        );
    }
}

/// Golden Torizo shot reaction - normal - missile ($AA:D6D1): dodge it. Was
/// `Torizo_D6D1`.
pub fn golden_missile_reaction(k: u16, j: u16, game: &mut impl Host) {
    word!(game.projectile_dir[(j >> 1)] &= !DELETE_PROJECTILE);
    let e = i32::from(k);
    slot!(game, TorizoSlot[e].function = NO_FUNCTION);
    slot!(game, TorizoSlot[e].base.instruction_timer = 1);
    let list = if slot!(game, TorizoSlot[e].direction) & FACING_RIGHT != 0 {
        instruction_list::DODGE_TURNING_RIGHT
    } else {
        instruction_list::DODGE_TURNING_LEFT
    };
    slot!(game, TorizoSlot[e].base.current_instruction = list);
}

/// Golden Torizo shot reaction - normal - super missile ($AA:D6F7): catch it
/// when facing Samus, otherwise take damage. Was `Torizo_D6F7`.
pub fn golden_super_missile_reaction(k: u16, j: u16, game: &mut impl Host) {
    let e = word!(game.cur_enemy_index) as i32;
    if facing_against_samus(k, game) & SIGN as u16 == 0 {
        slot!(
            game,
            TorizoSlot[e].damage = slot!(game, TorizoSlot[e].damage) | CAUGHT_SUPER_MISSILE
        );
        slot!(game, TorizoSlot[e].function = NO_FUNCTION);
        word!(game.projectile_dir[(j >> 1)] |= DELETE_PROJECTILE);
        slot!(game, TorizoSlot[e].base.instruction_timer = 1);
        let parameter = slot!(game, TorizoSlot[e].direction);
        let list = match (
            parameter & BACKGROUND_LEG_FORWARD != 0,
            parameter & FACING_RIGHT != 0,
        ) {
            (true, true) => instruction_list::CAUGHT_SUPER_MISSILE_RIGHT_LEFT_LEG,
            (true, false) => instruction_list::CAUGHT_SUPER_MISSILE_LEFT_RIGHT_LEG,
            (false, true) => instruction_list::CAUGHT_SUPER_MISSILE_RIGHT_RIGHT_LEG,
            (false, false) => instruction_list::CAUGHT_SUPER_MISSILE_LEFT_LEFT_LEG,
        };
        slot!(game, TorizoSlot[e].base.current_instruction = list);
    } else {
        golden_damaged_shot_reaction(game);
    }
}
