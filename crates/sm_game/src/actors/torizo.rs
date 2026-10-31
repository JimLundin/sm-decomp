//! Bomb and Golden Torizo, the chozo statues that wake and fight in Crateria
//! and Lower Norfair (bank $AA:B09C-$AA:D6F6): their functions, movement,
//! instructions, shots and palettes.
//!
//! Was `Torizo_*` and `GoldTorizo_*` in the original decompilation. The
//! movement functions are dispatched by the compat host, so their ids are
//! unchanged.
use crate::actors::boss_lease::BossLease;
use crate::actors::deaths::Deaths;
use crate::actors::host::{BossEffects, Shot, Slopes};
use crate::actors::layouts::part_view;
use crate::actors::session_slots::session_slot;
use crate::constants::enemy_properties::{
    K_ENEMY_PROPS_DELETED, K_ENEMY_PROPS_INVISIBLE, K_ENEMY_PROPS_TANGIBLE,
};
use crate::enemy_script::EnemyScript;
use crate::ram::word;
use crate::records::Records;

/// A lease on a game with the Torizos' effects.
pub trait Lease: BossLease<Game: Ai> {}
impl<T: BossLease<Game: Ai> + ?Sized> Lease for T {}

part_view! {
    /// A Torizo's view of its slot: its velocities and gravity (1/100h px a
    /// frame), the Torizo and movement functions, the facing and step state
    /// in `direction`, the damage marks, the link and gut explosion return
    /// lists, the animation lock, the last shot's kind, the eye beam's
    /// explosions, Samus's space jump frames and the steps since an attack.
    TorizoSlot: EnemyLayout40 {
        words: [x_velocity, y_velocity, gravity, var_d, function, movement, direction, damage],
        pad1: [u8; 26696],
        words: [
            link, saved_list, var_02, turn_around_timer, animation_lock, shot_kind, eye_beam,
            space_jump_frames, var_08, steps,
        ],
        _: [u16; 22],
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
pub fn walk<L: Lease>(
    k: u16,
    jp: EnemyScript,
    blocked_left: u16,
    blocked_right: u16,
    pause: u16,
    game: &mut L,
) -> EnemyScript {
    let e = i32::from(k);
    session_slot!(
        game,
        TorizoSlot[e].x_velocity = WALK_X_VELOCITIES[(jp.word(0) >> 1) as usize] as u16
    );
    let amount = (session_slot!(game, TorizoSlot[e].x_velocity) as i16 as i32) << 16;
    let blocked = game.move_horizontal(k, amount, Slopes::Ignore);
    if blocked {
        session_slot!(game, TorizoSlot[e].turn_around_timer = 0);
        if session_slot!(game, TorizoSlot[e].direction) & FACING_RIGHT != 0 {
            EnemyScript::jump(blocked_left)
        } else {
            EnemyScript::jump(blocked_right)
        }
    } else {
        game.snap_to_slopes(k);
        if (session_slot!(game, TorizoSlot[e].direction) as i32
            ^ (game.player_x() as i32 - session_slot!(game, TorizoSlot[e].base.x_pos) as i32))
            & SIGN
            == 0
            && session_slot!(game, TorizoSlot[e].turn_around_timer) == 0
        {
            session_slot!(game, TorizoSlot[e].turn_around_timer = pause);
        }
        jp.offset(1)
    }
}

/// Handle low-health initial drool ($AA:C620): in Crateria, now and then
/// drool while health is low. Was `Torizo_C620`.
pub fn drool_at_low_health<L: Lease>(k: u16, game: &mut L) {
    if game.area() == CRATERIA && word!(game.RANDOM_NUMBER) & DROOL_CHANCE == 0 {
        let health: u16 = session_slot!(game, TorizoSlot[k as i32].base.health);
        if health != 0 && health < LOW_HEALTH {
            game.spawn_projectile(health, k, LOW_HEALTH_DROOL);
        }
    }
}

/// Check if torizo is facing Samus ($AA:D3A7): the direction word XOR the
/// Torizo's offset from Samus, whose bit 15 is clear when it faces her. Was
/// `Torizo_Func_12`.
pub fn facing_against_samus<L: Lease>(k: u16, game: &mut L) -> u16 {
    let e = i32::from(k);
    (session_slot!(game, TorizoSlot[e].direction) as i32
        ^ (session_slot!(game, TorizoSlot[e].base.x_pos) as i32 - game.player_x() as i32))
        as u16
}

/// Golden Torizo shot reaction - damaged ($AA:D6A6): on death, start the death
/// sequence. Was `Torizo_D6A6`.
pub fn golden_damaged_shot_reaction<L: Lease>(game: &mut L) {
    game.shot_reaction(Shot::ReactionOnly);
    let e = game.current_enemy() as i32;
    if session_slot!(game, TorizoSlot[e].base.health) == 0 {
        session_slot!(
            game,
            TorizoSlot[e].base.current_instruction = instruction_list::DEATH_SEQUENCE
        );
        session_slot!(game, TorizoSlot[e].base.instruction_timer = 1);
        session_slot!(
            game,
            TorizoSlot[e].damage =
                session_slot!(game, TorizoSlot[e].damage) | GUT_AND_FACE_BLOWN_UP
        );
        session_slot!(
            game,
            TorizoSlot[e].base.properties =
                session_slot!(game, TorizoSlot[e].base.properties) | K_ENEMY_PROPS_TANGIBLE as u16
        );
    }
}

/// Golden Torizo shot reaction - normal - missile ($AA:D6D1): dodge it. Was
/// `Torizo_D6D1`.
pub fn golden_missile_reaction<L: Lease>(k: u16, j: u16, game: &mut L) {
    word!(game.PROJECTILE_DIR[(j >> 1)] &= !DELETE_PROJECTILE);
    let e = i32::from(k);
    session_slot!(game, TorizoSlot[e].function = NO_FUNCTION);
    session_slot!(game, TorizoSlot[e].base.instruction_timer = 1);
    let list = if session_slot!(game, TorizoSlot[e].direction) & FACING_RIGHT != 0 {
        instruction_list::DODGE_TURNING_RIGHT
    } else {
        instruction_list::DODGE_TURNING_LEFT
    };
    session_slot!(game, TorizoSlot[e].base.current_instruction = list);
}

/// Golden Torizo shot reaction - normal - super missile ($AA:D6F7): catch it
/// when facing Samus, otherwise take damage. Was `Torizo_D6F7`.
pub fn golden_super_missile_reaction<L: Lease>(k: u16, j: u16, game: &mut L) {
    let e = game.current_enemy() as i32;
    if facing_against_samus(k, game) & SIGN as u16 == 0 {
        session_slot!(game, TorizoSlot[e].damage, |old| old | CAUGHT_SUPER_MISSILE);
        session_slot!(game, TorizoSlot[e].function = NO_FUNCTION);
        word!(game.PROJECTILE_DIR[(j >> 1)] |= DELETE_PROJECTILE);
        session_slot!(game, TorizoSlot[e].base.instruction_timer = 1);
        let parameter = session_slot!(game, TorizoSlot[e].direction);
        let list = match (
            parameter & BACKGROUND_LEG_FORWARD != 0,
            parameter & FACING_RIGHT != 0,
        ) {
            (true, true) => instruction_list::CAUGHT_SUPER_MISSILE_RIGHT_LEFT_LEG,
            (true, false) => instruction_list::CAUGHT_SUPER_MISSILE_LEFT_RIGHT_LEG,
            (false, true) => instruction_list::CAUGHT_SUPER_MISSILE_RIGHT_RIGHT_LEG,
            (false, false) => instruction_list::CAUGHT_SUPER_MISSILE_LEFT_LEFT_LEG,
        };
        session_slot!(game, TorizoSlot[e].base.current_instruction = list);
    } else {
        golden_damaged_shot_reaction(game);
    }
}

/// What the Torizos' AI needs beyond the movement and drool host: the shared
/// boss effects, deaths, their palettes and the compat dispatch.
pub trait Ai: BossEffects + Deaths + Records + BossLease<Game = Self> {
    /// Run the bank $AA movement function at `address` for the current Torizo.
    fn run_function(&mut self, address: u32);
    /// Wake Torizo `k` once the Bomb Torizo's chozo statue finishes crumbling.
    fn wake_when_statue_crumbles(&mut self, k: u16);
    /// Boss bit `bit` of the current area.
    fn boss_bit(&mut self, bit: u16) -> u16;
    fn set_boss_bit(&mut self, bit: u16);
    /// Load Torizo target palettes `palettes`.
    fn set_palette_targets(&mut self, palettes: Palettes);
    /// Load the common Torizo target palettes.
    fn initialize_palette_targets(&mut self);
    /// Flash the Torizo's palettes white.
    fn flash_palette(&mut self);
    /// Restore the active Bomb Torizo palettes.
    fn restore_palette(&mut self);
    /// Golden Torizo's palettes for `health`.
    fn health_palette(&mut self, health: u16);
    /// Spawn the palette effect `definition` from bank $8D.
    fn spawn_palette_effect(&mut self, definition: u16);
    /// Spawn the Bomb Torizo room's haze.
    fn spawn_haze(&mut self);
}

/// The Torizo target palette pairs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Palettes {
    BombStart,
    BombActive,
    GoldenStart,
    GoldenActive,
}

/// Bank $AA Torizo functions, stored in `function`.
pub mod function {
    pub const NOTHING: u16 = 0xc6ab;
    pub const SIMPLE_MOVEMENT: u16 = 0xc6bf;
    pub const WAKE_WHEN_STATUE_CRUMBLES: u16 = 0xc6c6;
    pub const NORMAL_MOVEMENT: u16 = 0xc6ff;
    pub const GOLDEN_WAKE_WHEN_SAMUS_BELOW_RIGHT: u16 = 0xd5c2;
    pub const GOLDEN_SIMPLE_MOVEMENT: u16 = 0xd5df;
    pub const GOLDEN_NORMAL_MOVEMENT: u16 = 0xd5e6;
}

/// Bank $AA Torizo movement functions, stored in `movement`.
pub mod movement {
    pub const WALKING: u16 = 0xc752;
    pub const ATTACKING: u16 = 0xc828;
    pub const JUMPING_OR_FALLING: u16 = 0xc82c;
    pub const NOTHING: u16 = 0xc95e;
    pub const GOLDEN_ATTACKING: u16 = 0xd5ed;
    pub const GOLDEN_WALKING: u16 = 0xd5f1;
}

/// Bank $AA Torizo instruction lists and spritemaps the AI selects.
pub mod list {
    pub const BLOW_UP_GUT: u16 = 0xb0e5;
    pub const BLOW_UP_FACE: u16 = 0xb155;
    pub const TURNING_LEFT: u16 = 0xb962;
    pub const FALLING_FACING_LEFT: u16 = 0xbc78;
    pub const FACELESS_TURNING_LEFT: u16 = 0xbd0e;
    pub const TURNING_RIGHT: u16 = 0xbdd8;
    pub const FALLING_FACING_RIGHT: u16 = 0xc0f2;
    pub const FACELESS_TURNING_RIGHT: u16 = 0xc188;
    pub const GOLDEN_TURNING_LEFT: u16 = 0xd203;
    pub const GOLDEN_TURNING_RIGHT: u16 = 0xd2bf;
    pub const EXTENDED_SPRITEMAP: u16 = 0x87d0;
}

/// Bank $86 Torizo enemy projectiles.
pub mod projectile {
    pub const CONTINUOUS_DROOL: u16 = 0xa95b;
    pub const EXPLOSIVE_SWIPE: u16 = 0xa985;
    pub const LOW_HEALTH_EXPLOSION: u16 = 0xa9a1;
    pub const DEATH_EXPLOSION: u16 = 0xa9af;
    pub const BOMB_CHOZO_ORBS: u16 = 0xad5e;
    pub const GOLDEN_CHOZO_ORBS: u16 = 0xad7a;
    pub const BOMB_SONIC_BOOM: u16 = 0xaea8;
    pub const GOLDEN_SONIC_BOOM: u16 = 0xaeb6;
    pub const LANDING_DUST_RIGHT_FOOT: u16 = 0xafe5;
    pub const LANDING_DUST_LEFT_FOOT: u16 = 0xaff3;
    pub const GOLDEN_EGG: u16 = 0xb1c0;
    pub const GOLDEN_SUPER_MISSILE: u16 = 0xb31a;
    pub const GOLDEN_EYE_BEAM: u16 = 0xb428;
}

/// Bank $8D palette effects: the Bomb and Golden Torizo belly glows.
pub const BOMB_BELLY_GLOW: u16 = 0xf759;
pub const GOLDEN_BELLY_GLOW: u16 = 0xf75d;

const BANK: u32 = 0xaa;
/// `direction` bit 14: turning around; bits 13-15 are the step state.
const TURNING_AROUND: u16 = 0x4000;
const STEP_STATE: u16 = 0xe000;
/// `damage` bit 15: gut blown up; bit 14: face blown up; bit 13: stunned.
const GUT_BLOWN_UP: u16 = 0x8000;
const FACE_BLOWN_UP: u16 = 0x4000;
const STUNNED: u16 = 0x2000;
/// `eye_beam` bit 15: the eye beam explodes.
const EYE_BEAM_EXPLODES: u16 = 0x8000;
/// `animation_lock`'s locked value.
const ANIMATION_LOCKED: u16 = 0x7777;
/// The gut blows up below 350 health, the face below 100.
const GUT_HEALTH: u16 = 350;
const FACE_HEALTH: u16 = 100;
/// Golden Torizo attacks more at low health and when stunned at high health.
const GOLDEN_LOW_HEALTH: u16 = 0x788;
const GOLDEN_HIGH_HEALTH: u16 = 0x2a30;
/// Landing speed: Y velocity 1 px a frame; jumps add `gravity` a frame.
const LANDED_Y_VELOCITY: u16 = 256;
const GRAVITY: u16 = 40;
/// The fall caps at 15 px a frame.
const FALL_TOP_SPEED: u16 = 15;
/// The Golden Torizo stands at Y 375 once on the ground.
const GOLDEN_GROUND_Y: u16 = 375;
/// The Golden Torizo wakes when Samus is below and right of this point.
const GOLDEN_WAKE_X: u16 = 0x170;
const GOLDEN_WAKE_Y: u16 = 0x140;
/// Boss bit 2: the area's Torizo is dead.
const TORIZO_DEAD: u16 = 4;
/// Songs: fight music 5, the elevator music 3 after the fight.
const FIGHT_MUSIC: u16 = 5;
const ELEVATOR_MUSIC: u16 = 3;
const SHOT_SOUND: u8 = 0x27;
const FOOTSTEP_SOUND: u8 = 0x4b;
const EGG_RELEASED_SOUND: u8 = 0x34;
const LASER_SOUND: u8 = 0x67;
const SONIC_BOOM_SOUND: u8 = 0x48;
/// $7E:1997: enemy projectile ids, 18 words.
/// The Bomb Torizo's death explosion and landing dust, and the chozo orbs,
/// spawn with the instruction's own address as their parameter.
const DEATH_EXPLOSION_INSTRUCTION: u16 = 0xc32f;
const LANDING_DUST_INSTRUCTION: u16 = 0xc34a;
const BOMB_ORBS_INSTRUCTION: u16 = 0xc5cb;
const GOLDEN_ORBS_INSTRUCTION: u16 = 0xd4f3;
/// The Golden Torizo test cheat: holding A+X+L+R... as joypad 1's last keys.
const DEBUG_KEYS: u16 = 0xc0c0;
/// The Samus poses the Golden Torizo watches for.
const SPACE_JUMP_POSES: [u16; 2] = [0x1b, 0x1c];
const ROLLING_POSES: [u16; 7] = [0x1d, 0x1e, 0x1f, 0x79, 0x7a, 0x7b, 0x7c];

/// The Bomb (Crateria) and Golden (Norfair) Torizo's start position,
/// properties and hitbox, by area / 2 ($AA:C95F-$AA:C976).
const START_X: [u16; 2] = [0xdb, 0x1a8];
const START_Y: [u16; 2] = [0xb3, 0x90];
const START_PROPERTIES: [u16; 2] = [0x2800, 0x2800];
const START_X_RADIUS: [u16; 2] = [0x12, 0x12];
const START_Y_RADIUS: [u16; 2] = [0x30, 0x29];
/// The standing up and sitting down steps' X and Y offsets ($AA:C3EE-$AA:C46F).
const STAND_UP_X: [i16; 16] = [-9, -6, -7, 5, -16, -7, 0, 0, 9, 6, 7, -5, 16, 7, 0, 0];
const STAND_UP_Y: [i16; 8] = [0, -6, -6, -7, 0, 0, 0, 0];
const SIT_DOWN_X: [i16; 16] = [-9, -6, -7, 5, -16, -7, 0, 0, 9, 6, 7, -5, 16, 7, 0, 0];
const SIT_DOWN_Y: [i16; 8] = [0, -6, -6, -7, 0, 0, 0, 0];

fn quake<L: Lease>(game: &mut L) {
    word!(game.EARTHQUAKE_TYPE = 4);
    word!(game.EARTHQUAKE_TIMER = 32);
}

/// Whether Samus is at least `distance` px from Torizo `k` horizontally.
fn samus_at_least<L: Lease>(game: &mut L, k: u16, distance: u16) -> bool {
    let delta = game.player_delta(k, crate::actors::host::Axis::X);
    (delta as i16).wrapping_abs() as u16 >= distance
}

/// The instruction list address of `jp` in the Torizo's bank.
fn list_address<L: Lease>(game: &mut L, k: u16, jp: EnemyScript) -> u16 {
    let bank = session_slot!(byte game, TorizoSlot[k as i32].base.bank[0]);
    jp.offset_from(crate::enemy_script::bank_base(bank)) as i32 as u16
}

/// Runs Torizo `k`'s function. Was `CallTorizoFunc`.
pub fn run_torizo_function<L: Lease>(game: &mut L, function: u16, k: u16) {
    match function {
        function::NOTHING => {}
        function::SIMPLE_MOVEMENT | function::GOLDEN_SIMPLE_MOVEMENT => simple_movement(k, game),
        function::WAKE_WHEN_STATUE_CRUMBLES => {
            game.foreign(|game| game.wake_when_statue_crumbles(k))
        }
        function::NORMAL_MOVEMENT => normal_movement(k, game),
        function::GOLDEN_WAKE_WHEN_SAMUS_BELOW_RIGHT => golden_wake(game),
        function::GOLDEN_NORMAL_MOVEMENT => golden_normal_movement(k, game),
        _ => game.foreign(|game| game.unreachable()),
    }
}

fn run_movement<L: Lease>(game: &mut L, k: u16) {
    match session_slot!(game, TorizoSlot[k as i32].movement) {
        movement::WALKING => walking(k, game),
        movement::JUMPING_OR_FALLING => jumping_or_falling(k, game),
        movement::GOLDEN_WALKING => golden_walking(k, game),
        movement::ATTACKING | movement::GOLDEN_ATTACKING => attacking(k, game),
        movement::NOTHING => {}
        other => game.foreign(|game| game.run_function((BANK << 16) | u32::from(other))),
    }
}

/// Torizo function - simple movement ($AA:C6BF, Golden $AA:D5DF): drool and
/// fall. Was `Torizo_Func_3`.
pub fn simple_movement<L: Lease>(k: u16, game: &mut L) {
    drool_at_low_health(k, game);
    handle_falling(k, game);
}

/// Torizo function - normal movement ($AA:C6FF): blow up the gut, then the
/// face, as health falls; else the movement function. Was `Torizo_Func_1`.
fn normal_movement<L: Lease>(k: u16, game: &mut L) {
    drool_at_low_health(k, game);
    let e = i32::from(k);
    if (session_slot!(game, TorizoSlot[e].damage) & GUT_BLOWN_UP) != 0
        || session_slot!(game, TorizoSlot[e].base.health) >= GUT_HEALTH
    {
        if (session_slot!(game, TorizoSlot[e].damage) & FACE_BLOWN_UP) != 0
            || session_slot!(game, TorizoSlot[e].base.health) >= FACE_HEALTH
        {
            run_movement(game, k);
        } else {
            let turn = if (session_slot!(game, TorizoSlot[e].direction) & FACING_RIGHT) != 0 {
                list::FACELESS_TURNING_LEFT
            } else {
                list::FACELESS_TURNING_RIGHT
            };
            session_slot!(game, TorizoSlot[e].link = turn);
            session_slot!(
                game,
                TorizoSlot[e].base.current_instruction = list::BLOW_UP_FACE
            );
            session_slot!(game, TorizoSlot[e].base.instruction_timer = 1);
        }
    } else {
        let copied = session_slot!(game, TorizoSlot[e].base.current_instruction);
        session_slot!(game, TorizoSlot[e].saved_list = copied);
        session_slot!(
            game,
            TorizoSlot[e].base.current_instruction = list::BLOW_UP_GUT
        );
        session_slot!(game, TorizoSlot[e].base.instruction_timer = 1);
    }
}

/// Torizo function - Golden Torizo - wake enemy if Samus is below and right
/// of target position ($AA:D5C2). Was `Torizo_Func_9`.
fn golden_wake<L: Lease>(game: &mut L) {
    let e = game.current_enemy() as i32;
    if game.player_y() > GOLDEN_WAKE_Y && game.player_x() > GOLDEN_WAKE_X {
        session_slot!(
            game,
            TorizoSlot[e].base.current_instruction,
            |current_instruction| current_instruction.wrapping_add(1)
        );
        session_slot!(
            game,
            TorizoSlot[e].base.current_instruction,
            |current_instruction| current_instruction.wrapping_add(1)
        );
        session_slot!(game, TorizoSlot[e].base.instruction_timer = 1);
    }
}

/// Torizo function - Golden Torizo - normal movement ($AA:D5E6). Was
/// `Torizo_D5E6`.
fn golden_normal_movement<L: Lease>(k: u16, game: &mut L) {
    drool_at_low_health(k, game);
    run_movement(game, k);
}

/// Make torizo jump forwards ($AA:C20A). Was `Torizo_C20A`.
fn jump_forwards<L: Lease>(k: u16, game: &mut L) {
    let e = i32::from(k);
    let speed: i16 = if (session_slot!(game, TorizoSlot[e].direction) & FACING_RIGHT) != 0 {
        512
    } else {
        -512
    };
    session_slot!(game, TorizoSlot[e].x_velocity = speed as u16);
    session_slot!(game, TorizoSlot[e].y_velocity = -1472i32 as u16);
    session_slot!(game, TorizoSlot[e].gravity = GRAVITY);
    session_slot!(game, TorizoSlot[e].base.instruction_timer = 1);
}

/// Make torizo jump backwards ($AA:C22D). Was `Torizo_C22D`.
fn jump_backwards<L: Lease>(k: u16, game: &mut L) {
    let e = i32::from(k);
    let speed: i16 = if (session_slot!(game, TorizoSlot[e].direction) & FACING_RIGHT) != 0 {
        -768
    } else {
        768
    };
    session_slot!(game, TorizoSlot[e].x_velocity = speed as u16);
    session_slot!(game, TorizoSlot[e].y_velocity = -1152i32 as u16);
    session_slot!(game, TorizoSlot[e].gravity = GRAVITY);
    session_slot!(game, TorizoSlot[e].base.instruction_timer = 1);
}

/// Handle falling ($AA:C643): fall, shaking the room on landing. Was
/// `Torizo_C643`.
pub fn handle_falling<L: Lease>(k: u16, game: &mut L) {
    let e = i32::from(k);
    let amount = (session_slot!(game, TorizoSlot[e].y_velocity) as i16 as i32) << 8;
    if game.move_vertical(k, amount) {
        let speed: i16 = session_slot!(game, TorizoSlot[e].y_velocity) as i16;
        if speed >= 0 && speed as u16 != LANDED_Y_VELOCITY {
            quake(game);
            session_slot!(game, TorizoSlot[e].y_velocity = LANDED_Y_VELOCITY);
        }
    } else {
        session_slot!(
            game,
            TorizoSlot[e].y_velocity =
                (session_slot!(game, TorizoSlot[e].y_velocity)).wrapping_add(GRAVITY)
        );
    }
}

/// The walking movement: falls off ledges; at the turn timer's end it turns
/// to `turn_left` or `turn_right`.
fn walk_or_turn<L: Lease>(k: u16, game: &mut L, turn_left: u16, turn_right: u16) {
    let e = i32::from(k);
    let timer: u16 = session_slot!(game, TorizoSlot[e].turn_around_timer);
    if timer == 0 || {
        let left: u16 = timer.wrapping_sub(1);
        session_slot!(game, TorizoSlot[e].turn_around_timer = left);
        session_slot!(game, TorizoSlot[e].turn_around_timer) != 0
    } {
        let mut fall: u16 = (game.foreign(|game| {
            crate::actors::host::abs16(session_slot!(game, TorizoSlot[e].x_velocity))
        }))
        .wrapping_add(1);
        if fall >= 0x10 {
            fall = FALL_TOP_SPEED;
        }
        if !game.move_vertical(k, (fall as i16 as i32) << 16) {
            let falling = if (session_slot!(game, TorizoSlot[e].direction) & FACING_RIGHT) != 0 {
                list::FALLING_FACING_RIGHT
            } else {
                list::FALLING_FACING_LEFT
            };
            session_slot!(game, TorizoSlot[e].base.current_instruction = falling);
            session_slot!(game, TorizoSlot[e].base.instruction_timer = 1);
            session_slot!(game, TorizoSlot[e].y_velocity = LANDED_Y_VELOCITY);
            session_slot!(game, TorizoSlot[e].x_velocity = 0);
        }
    } else {
        session_slot!(game, TorizoSlot[e].turn_around_timer = 0);
        let turn = if (session_slot!(game, TorizoSlot[e].direction) & FACING_RIGHT) != 0 {
            turn_left
        } else {
            turn_right
        };
        session_slot!(game, TorizoSlot[e].base.current_instruction = turn);
        session_slot!(game, TorizoSlot[e].base.instruction_timer = 1);
    }
}

/// Torizo movement function - walking ($AA:C752). Was `Torizo_Func_5`.
pub fn walking<L: Lease>(k: u16, game: &mut L) {
    if (session_slot!(game, TorizoSlot[k as i32].damage) & FACE_BLOWN_UP) != 0 {
        walk_or_turn(
            k,
            game,
            list::FACELESS_TURNING_LEFT,
            list::FACELESS_TURNING_RIGHT,
        );
    } else {
        walk_or_turn(k, game, list::TURNING_LEFT, list::TURNING_RIGHT);
    }
}

/// Torizo movement function - Golden Torizo - walking ($AA:D5F1). Was
/// `Torizo_D5F1`.
pub fn golden_walking<L: Lease>(k: u16, game: &mut L) {
    walk_or_turn(
        k,
        game,
        list::GOLDEN_TURNING_LEFT,
        list::GOLDEN_TURNING_RIGHT,
    );
}

/// Torizo movement function - attacking ($AA:C828, Golden $AA:D5ED). Was
/// `Torizo_Func_6`.
pub fn attacking<L: Lease>(k: u16, game: &mut L) {
    handle_falling(k, game);
}

/// Torizo movement function - jumping/falling ($AA:C82C): until it lands,
/// then back to the link list with a quake. Was `Torizo_Func_7`.
pub fn jumping_or_falling<L: Lease>(k: u16, game: &mut L) {
    let e = i32::from(k);
    let amount = (session_slot!(game, TorizoSlot[e].x_velocity) as i16 as i32) << 8;
    game.move_horizontal(k, amount, Slopes::Ignore);
    game.snap_to_slopes(k);
    let amount = (session_slot!(game, TorizoSlot[e].y_velocity) as i16 as i32) << 8;
    if game.move_vertical(k, amount) {
        let copied = session_slot!(game, TorizoSlot[e].link);
        session_slot!(game, TorizoSlot[e].base.current_instruction = copied);
        session_slot!(game, TorizoSlot[e].base.instruction_timer = 1);
        session_slot!(game, TorizoSlot[e].y_velocity = LANDED_Y_VELOCITY);
        quake(game);
    } else {
        session_slot!(
            game,
            TorizoSlot[e].y_velocity = (session_slot!(game, TorizoSlot[e].y_velocity))
                .wrapping_add(session_slot!(game, TorizoSlot[e].gravity))
        );
    }
}

/// Hurt AI - Bomb Torizo ($AA:C67E). Was `Torizo_Hurt`.
pub fn hurt<L: Lease>(game: &mut L) {
    let k = game.current_enemy();
    drool_at_low_health(k, game);
    if session_slot!(
        game,
        TorizoSlot[game.current_enemy() as i32].base.flash_timer
    ) & 1
        != 0
    {
        game.foreign(|game| game.flash_palette());
    } else {
        game.foreign(|game| game.restore_palette());
    }
}

/// Hurt AI - Golden Torizo ($AA:D3BA). Was `GoldTorizo_Hurt`.
pub fn golden_hurt<L: Lease>(game: &mut L) {
    let k = game.current_enemy();
    drool_at_low_health(k, game);
    if session_slot!(
        game,
        TorizoSlot[game.current_enemy() as i32].base.flash_timer
    ) & 1
        != 0
    {
        game.foreign(|game| game.flash_palette());
    } else {
        health_palette(game.current_enemy(), game);
    }
}

/// Golden Torizo health-based palette handling ($AA:D3B2). Was
/// `Torizo_Func_11`.
fn health_palette<L: Lease>(k: u16, game: &mut L) {
    let health = session_slot!(game, TorizoSlot[k as i32].base.health);
    game.foreign(|game| game.health_palette(health));
}

/// Enemy touch - torizos ($AA:C977). Was `GoldTorizo_Touch`.
pub fn touch<L: Lease>(game: &mut L) {
    game.contact_reaction(crate::actors::host::Death::LeavePending);
}

/// Enemy shot - torizo - normal ($AA:C97C): the Golden Torizo's reaction in
/// Norfair; the Bomb Torizo is hurt when unlocked and not flashing, and dies
/// at no health. Was `Torizo_Shot`.
pub fn shot<L: Lease>(game: &mut L) {
    if game.area() != 0 {
        golden_shot(game);
    } else {
        let e = game.current_enemy() as i32;
        if session_slot!(game, TorizoSlot[e].animation_lock)
            | session_slot!(game, TorizoSlot[e].base.flash_timer)
            == 0
        {
            game.shot_reaction(crate::actors::host::Shot::ReactionOnly);
            if session_slot!(game, TorizoSlot[e].base.health) == 0 {
                session_slot!(game, TorizoSlot[e].function = function::NOTHING);
                session_slot!(
                    game,
                    TorizoSlot[e].base.current_instruction = instruction_list::DEATH_SEQUENCE
                );
                session_slot!(game, TorizoSlot[e].base.instruction_timer = 1);
                session_slot!(
                    game,
                    TorizoSlot[e].damage =
                        (session_slot!(game, TorizoSlot[e].damage)) | GUT_AND_FACE_BLOWN_UP
                );
                session_slot!(
                    game,
                    TorizoSlot[e].base.properties =
                        (session_slot!(game, TorizoSlot[e].base.properties))
                            | (K_ENEMY_PROPS_TANGIBLE as u16)
                );
            }
        }
    }
}

/// Enemy shot - torizo - stand up / sit down ($AA:C9C2). Was `Torizo_Func_8`.
pub fn standing_shot<L: Lease>(game: &mut L) {
    if game.area() != 0 {
        golden_standing_shot(game);
    }
}

/// Golden Torizo shot reaction - stand up / sit down ($AA:D658). Was
/// `Torizo_D658`.
fn golden_standing_shot<L: Lease>(game: &mut L) {
    let e = game.current_enemy() as i32;
    if session_slot!(game, TorizoSlot[e].base.flash_timer) == 0
        && session_slot!(game, TorizoSlot[e].animation_lock) == 0
    {
        golden_damaged_shot_reaction(game);
    }
}

/// Golden Torizo shot reaction - normal ($AA:D667): missiles are dodged,
/// super missiles caught; other shots stun and hurt it. Was
/// `GoldTorizo_Shot`.
pub fn golden_shot<L: Lease>(game: &mut L) {
    let e = game.current_enemy() as i32;
    if session_slot!(game, TorizoSlot[e].base.flash_timer) == 0 {
        if session_slot!(game, TorizoSlot[e].animation_lock) != 0 {
            golden_damaged_shot_reaction(game);
            return;
        }
        let stunned: bool =
            ((session_slot!(game, TorizoSlot[e].damage) & CAUGHT_SUPER_MISSILE) != 0) || {
                let j: u16 = 2u16.wrapping_mul(word!(game.COLLISION_DETECTION_INDEX));
                let kind: u16 =
                    (word!(game.PROJECTILE_TYPE[word!(game.COLLISION_DETECTION_INDEX)])) & 0xf00;
                session_slot!(game, TorizoSlot[e].shot_kind = kind);
                if kind == 0x100 {
                    golden_missile_reaction(game.current_enemy(), j, game);
                    return;
                }
                kind != 0x200 || {
                    golden_super_missile_reaction(game.current_enemy(), j, game);
                    false
                }
            };
        if stunned {
            session_slot!(game, TorizoSlot[e].damage, |old| old | STUNNED);
            golden_damaged_shot_reaction(game);
        }
    }
}

/// Main AI - Bomb Torizo ($AA:C6A4). Was `Torizo_Main`.
pub fn main_ai<L: Lease>(game: &mut L) {
    let e = game.current_enemy() as i32;
    let function = session_slot!(game, TorizoSlot[e].function);
    run_torizo_function(game, function, game.current_enemy());
}

/// Main AI - Golden Torizo ($AA:D369): counts Samus's space jump frames. Was
/// `GoldTorizo_Main`.
pub fn golden_main_ai<L: Lease>(game: &mut L) {
    let e = game.current_enemy() as i32;
    if SPACE_JUMP_POSES.contains(&word!(game.SAMUS_POSE)) {
        session_slot!(game, TorizoSlot[e].space_jump_frames, |old| old
            .wrapping_add(1));
    } else {
        session_slot!(game, TorizoSlot[e].space_jump_frames = 0);
    }
    let function = session_slot!(game, TorizoSlot[e].function);
    run_torizo_function(game, function, game.current_enemy());
}

/// Initialisation AI - torizos ($AA:C87F): gone once dead; else placed for
/// its area with its palettes, the Golden Torizo's debug equipment, or the
/// Bomb Torizo's haze. Was `Torizo_Init`.
pub fn initialize<L: Lease>(game: &mut L) {
    let e = game.current_enemy() as i32;
    if game.foreign(|game| game.boss_bit(TORIZO_DEAD)) & 1 != 0 {
        session_slot!(
            game,
            TorizoSlot[e].base.properties = (session_slot!(game, TorizoSlot[e].base.properties))
                | (K_ENEMY_PROPS_DELETED as u16)
        );
        return;
    }
    let torizo = (game.area() as i32 >> 1) as usize;
    session_slot!(
        game,
        TorizoSlot[e].base.properties =
            (session_slot!(game, TorizoSlot[e].base.properties)) | START_PROPERTIES[torizo]
    );
    session_slot!(
        game,
        TorizoSlot[e].base.extra_properties =
            (session_slot!(game, TorizoSlot[e].base.extra_properties)) | 4
    );
    session_slot!(game, TorizoSlot[e].base.x_width = START_X_RADIUS[torizo]);
    session_slot!(game, TorizoSlot[e].base.y_height = START_Y_RADIUS[torizo]);
    session_slot!(game, TorizoSlot[e].function = function::SIMPLE_MOVEMENT);
    session_slot!(game, TorizoSlot[e].base.instruction_timer = 1);
    session_slot!(game, TorizoSlot[e].base.timer = 0);
    session_slot!(game, TorizoSlot[e].base.palette_index = 0);
    session_slot!(game, TorizoSlot[e].movement = movement::NOTHING);
    session_slot!(
        game,
        TorizoSlot[e].base.current_instruction =
            crate::torizo_tables::torizo(game, torizo as isize).instruction_list
    );
    session_slot!(
        game,
        TorizoSlot[e].base.spritemap_pointer = list::EXTENDED_SPRITEMAP
    );
    session_slot!(game, TorizoSlot[e].base.x_pos = START_X[torizo]);
    session_slot!(game, TorizoSlot[e].base.y_pos = START_Y[torizo]);
    session_slot!(game, TorizoSlot[e].x_velocity = 0);
    session_slot!(game, TorizoSlot[e].y_velocity = LANDED_Y_VELOCITY);
    game.foreign(|game| game.initialize_palette_targets());
    if game.area() != 0 {
        game.foreign(|game| game.set_palette_targets(Palettes::GoldenStart));
        if word!(game.JOYPAD1_LASTKEYS) == DEBUG_KEYS {
            word!(game.SAMUS_HEALTH = 700);
            word!(game.SAMUS_MAX_HEALTH = 700);
            word!(game.SAMUS_MAX_RESERVE_HEALTH = 300);
            word!(game.SAMUS_RESERVE_HEALTH = 300);
            word!(game.SAMUS_MISSILES = 100);
            word!(game.SAMUS_MAX_MISSILES = 100);
            word!(game.SAMUS_SUPER_MISSILES = 20);
            word!(game.SAMUS_MAX_SUPER_MISSILES = 20);
            word!(game.SAMUS_POWER_BOMBS = 20);
            word!(game.SAMUS_MAX_POWER_BOMBS = 20);
            word!(game.EQUIPPED_ITEMS = -3273i32 as u16);
            word!(game.COLLECTED_ITEMS = -3273i32 as u16);
            word!(game.EQUIPPED_BEAMS = 4111);
            word!(game.COLLECTED_BEAMS = 4111);
        }
    } else {
        game.foreign(|game| {
            game.set_palette_targets(Palettes::BombStart);
            game.spawn_haze();
        });
    }
}

/// Instruction - enemy function = [[Y]] ($AA:B09C). Was `Torizo_Instr_3`.
pub fn set_function<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, TorizoSlot[k as i32].function = jp.word(0));
    jp.offset(1)
}

/// Instruction - mark Bomb Torizo gut blown up and spawn 6 Bomb Torizo
/// low-health continuous drool enemy projectiles ($AA:B11D). Was
/// `Torizo_Instr_31`.
pub fn blow_up_gut<L: Lease>(k: u16, game: &mut L) {
    let e = i32::from(k);
    session_slot!(game, TorizoSlot[e].damage, |old| old | GUT_BLOWN_UP);
    let mut parameter = game.spawn_projectile(
        session_slot!(game, TorizoSlot[e].damage),
        k,
        projectile::CONTINUOUS_DROOL,
    );
    for _ in 0..5 {
        parameter = game.spawn_projectile(parameter, k, projectile::CONTINUOUS_DROOL);
    }
}

/// Instruction - mark Bomb Torizo face blown up ($AA:B1BE). Was
/// `Torizo_Instr_33`.
pub fn blow_up_face<L: Lease>(k: u16, game: &mut L) {
    let e = i32::from(k);
    session_slot!(game, TorizoSlot[e].damage, |old| old | FACE_BLOWN_UP);
}

/// Instruction - set enemy as visible ($AA:B224). Was `Torizo_Instr_36`.
pub fn set_visible<L: Lease>(k: u16, game: &mut L) {
    let e = i32::from(k);
    session_slot!(
        game,
        TorizoSlot[e].base.properties = (session_slot!(game, TorizoSlot[e].base.properties) as i32
            & !(K_ENEMY_PROPS_INVISIBLE as i32)) as u16
    );
}

/// Instruction - set enemy as invisible ($AA:B22E). Was `Torizo_Instr_37`.
pub fn set_invisible<L: Lease>(k: u16, game: &mut L) {
    let e = i32::from(k);
    session_slot!(
        game,
        TorizoSlot[e].base.properties =
            (session_slot!(game, TorizoSlot[e].base.properties)) | (K_ENEMY_PROPS_INVISIBLE as u16)
    );
}

/// Instruction - set up palette transition to black ($AA:B238). Was
/// `Torizo_Instr_35`.
pub fn fade_to_black<L: Lease>(game: &mut L) {
    crate::word_transfers!(game, descending, 16;
        le(TARGET_PALETTES, 175) <- value(0),
        le(TARGET_PALETTES, 159) <- value(0),
    );
}

/// Instruction - set boss bit, queue elevator music, spawn item drops
/// ($AA:B24D). Was `Torizo_Instr_38`.
pub fn defeated<L: Lease>(k: u16, game: &mut L) {
    game.foreign(|game| {
        game.set_boss_bit(TORIZO_DEAD);
        game.queue_music_delayed(ELEVATOR_MUSIC);
    });
    if game.area() != 0 {
        game.foreign(|game| {
            game.scatter_enemy_drop(crate::enemy_projectiles::drops::DropSource::GoldenTorizo, k)
        });
    } else {
        game.foreign(|game| {
            game.scatter_enemy_drop(crate::enemy_projectiles::drops::DropSource::BombTorizo, k)
        });
    }
}

/// Instruction - advance gradual colour change ($AA:B271). Was
/// `Torizo_Instr_6`.
pub fn advance_colour_change<L: Lease>(game: &mut L) {
    game.foreign(|game| crate::palette_effects::fading::selected(0x600, game));
}

/// Instruction - set up palette transition to normal torizo ($AA:B94D). Was
/// `Torizo_Instr_5`.
pub fn fade_to_active<L: Lease>(game: &mut L) {
    game.foreign(|game| game.set_palette_targets(Palettes::BombActive));
}

/// Instruction - start fight music and Bomb Torizo belly palette FX
/// ($AA:B951). Was `Torizo_Instr_9`.
pub fn start_fight<L: Lease>(game: &mut L) {
    game.foreign(|game| {
        game.queue_music_delayed(FIGHT_MUSIC);
        game.spawn_palette_effect(BOMB_BELLY_GLOW);
    });
}

/// Instruction - set animation lock ($AA:C2C9). Was `Torizo_Instr_2`.
pub fn lock_animation<L: Lease>(k: u16, game: &mut L) {
    session_slot!(game, TorizoSlot[k as i32].animation_lock = ANIMATION_LOCKED);
}

/// Instruction - clear animation lock ($AA:C2D1). Was `Torizo_Instr_8`.
pub fn unlock_animation<L: Lease>(k: u16, game: &mut L) {
    session_slot!(game, TorizoSlot[k as i32].animation_lock = 0);
}

/// Instruction - go to [[Y]] if face blown up, else [[Y]+2] for the Golden
/// Torizo ($AA:C2D9). Was `Torizo_Instr_25`.
pub fn branch_if_faceless<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    if (session_slot!(game, TorizoSlot[k as i32].damage) & FACE_BLOWN_UP) != 0 {
        return EnemyScript::jump(jp.word(0));
    }
    if game.area() != 0 {
        EnemyScript::jump(jp.word(2))
    } else {
        jp.offset(2)
    }
}

/// Instruction - enemy link instruction = [[Y]] ($AA:C2ED). Was
/// `Torizo_Instr_22`.
pub fn set_link<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, TorizoSlot[k as i32].link = jp.word(0));
    jp.offset(1)
}

/// Instruction - return ($AA:C2F7). Was `Torizo_Instr_19`.
pub fn return_to_link<L: Lease>(k: u16, game: &mut L) -> EnemyScript {
    EnemyScript::jump(session_slot!(game, TorizoSlot[k as i32].link))
}

/// Instruction - go to enemy gut explosion link instruction ($AA:C2FD). Was
/// `Torizo_Instr_32`.
pub fn return_from_gut_explosion<L: Lease>(k: u16, game: &mut L) -> EnemyScript {
    EnemyScript::jump(session_slot!(game, TorizoSlot[k as i32].saved_list))
}

/// Instruction - spawn Bomb Torizo low-health explosions with parameter
/// [[Y]] and sleep for 28h i-frames ($AA:C303). Was `Torizo_Instr_30`.
pub fn low_health_explosions<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    let parameter: u16 = jp.word(0);
    // Six, though the bank log names five.
    for _ in 0..=5 {
        game.spawn_room_projectile(projectile::LOW_HEALTH_EXPLOSION, parameter);
    }
    let e = i32::from(k);
    session_slot!(
        game,
        TorizoSlot[e].base.current_instruction = list_address(game, k, jp.offset(1))
    );
    session_slot!(game, TorizoSlot[e].base.flash_timer = 40);
    session_slot!(game, TorizoSlot[e].base.instruction_timer = 40);
    EnemyScript::Stop
}

/// Instruction - spawn torizo death explosion and sleep for 1 i-frame
/// ($AA:C32F). Was `Torizo_Instr_34`.
pub fn death_explosion<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    game.spawn_room_projectile(projectile::DEATH_EXPLOSION, DEATH_EXPLOSION_INSTRUCTION);
    let e = i32::from(k);
    session_slot!(
        game,
        TorizoSlot[e].base.current_instruction = list_address(game, k, jp)
    );
    session_slot!(game, TorizoSlot[e].base.flash_timer = 1);
    session_slot!(game, TorizoSlot[e].base.instruction_timer = 1);
    EnemyScript::Stop
}

/// Instruction - spawn torizo landing dust clouds ($AA:C34A). Was
/// `Torizo_Instr_24`.
pub fn landing_dust<L: Lease>(game: &mut L) {
    let left = game.spawn_room_projectile(
        projectile::LANDING_DUST_RIGHT_FOOT,
        LANDING_DUST_INSTRUCTION,
    );
    game.spawn_room_projectile(projectile::LANDING_DUST_LEFT_FOOT, left);
}

/// Instruction - spawn low-health initial drool if health is low ($AA:C35B).
/// Was `Torizo_Instr_12`.
pub fn drool_if_low_health<L: Lease>(k: u16, game: &mut L) {
    let health: u16 = session_slot!(game, TorizoSlot[k as i32].base.health);
    if health < GUT_HEALTH {
        game.spawn_projectile(health, k, LOW_HEALTH_DROOL);
    }
}

/// Instruction - set torizo turning around flag ($AA:C36D). Was
/// `Torizo_Instr_10`.
pub fn start_turning<L: Lease>(k: u16, game: &mut L) {
    let e = i32::from(k);
    session_slot!(game, TorizoSlot[e].direction, |old| old | TURNING_AROUND);
}

/// Instructions - set stepped left/right with left/right foot state
/// ($AA:C377, $AA:C38A, $AA:C3A0, $AA:C3B6): the step state is `state`, and
/// the step counts. Was `Torizo_Instr_11`, `_29`, `_1` and `_28`.
pub fn stepped<L: Lease>(k: u16, game: &mut L, state: u16) {
    let e = i32::from(k);
    session_slot!(
        game,
        TorizoSlot[e].direction =
            session_slot!(game, TorizoSlot[e].direction) & !STEP_STATE | state
    );
    session_slot!(game, TorizoSlot[e].steps, |old| old.wrapping_add(1));
}

/// Instruction - standing up movement ($AA:C3CC). Was `Torizo_Instr_4`.
pub fn stand_up<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    let step: u16 = jp.word(0);
    let e = i32::from(k);
    session_slot!(
        game,
        TorizoSlot[e].base.x_pos = (session_slot!(game, TorizoSlot[e].base.x_pos))
            .wrapping_add((STAND_UP_X[(step >> 1) as usize]) as u16)
    );
    session_slot!(
        game,
        TorizoSlot[e].base.y_pos = (session_slot!(game, TorizoSlot[e].base.y_pos))
            .wrapping_add((STAND_UP_Y[((step as i32 & 0xf) >> 1) as usize]) as u16)
    );
    jp.offset(1)
}

/// Instruction - sitting down movement ($AA:C41E). Was `Torizo_Instr_40`.
pub fn sit_down<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    let step: u16 = jp.word(0);
    let e = i32::from(k);
    session_slot!(
        game,
        TorizoSlot[e].base.x_pos = (session_slot!(game, TorizoSlot[e].base.x_pos))
            .wrapping_sub((SIT_DOWN_X[(step >> 1) as usize]) as u16)
    );
    session_slot!(
        game,
        TorizoSlot[e].base.y_pos = (session_slot!(game, TorizoSlot[e].base.y_pos))
            .wrapping_sub((SIT_DOWN_Y[((step as i32 & 0xf) >> 1) as usize]) as u16)
    );
    jp.offset(1)
}

/// Instruction - go to [[Y]] if rising ($AA:C55A). Was `Torizo_Instr_23`.
pub fn branch_if_rising<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    if (session_slot!(game, TorizoSlot[k as i32].y_velocity) & 0x8000) != 0 {
        EnemyScript::jump(jp.word(0))
    } else {
        jp.offset(1)
    }
}

/// Instruction - call [[Y]] if Samus is less than 38h pixels in front
/// ($AA:C567). Was `Torizo_Instr_14`.
pub fn call_if_samus_close_in_front<L: Lease>(
    k: u16,
    jp: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    if samus_at_least(game, k, 0x38) {
        return jp.offset(1);
    }
    let e = i32::from(k);
    if ((session_slot!(game, TorizoSlot[e].direction) as i32
        ^ (game.player_x() as i32 - session_slot!(game, TorizoSlot[e].base.x_pos) as i32))
        & SIGN)
        == 0
    {
        return jp.offset(1);
    }
    session_slot!(
        game,
        TorizoSlot[e].link = list_address(game, k, jp.offset(1))
    );
    EnemyScript::jump(jp.word(0))
}

/// Instruction - go to [[Y]] and jump backwards if Samus is within 20h
/// pixels in front ($AA:C58B). Was `Torizo_Instr_15`.
pub fn jump_back_if_samus_close<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    if samus_at_least(game, k, 0x20) || (facing_against_samus(k, game) & 0x8000) != 0 {
        return jp.offset(1);
    }
    jump_backwards(k, game);
    EnemyScript::jump(jp.word(0))
}

/// The attack choice: [[Y]] when Samus has few missiles or on a frame/position
/// draw, else [[Y]+2], returning past both.
fn choose_attack<L: Lease>(k: u16, jp: EnemyScript, game: &mut L, missiles: u16) -> EnemyScript {
    session_slot!(
        game,
        TorizoSlot[k as i32].link = list_address(game, k, jp.offset(2))
    );
    if word!(game.SAMUS_MISSILES) < missiles
        || ((game.frame() as i32 + (game.player_x() as i32 & 1) + (game.player_x() as i32 >> 1))
            & 8)
            != 0
    {
        EnemyScript::jump(jp.word(0))
    } else {
        EnemyScript::jump(jp.word(2))
    }
}

/// Instruction - call [[Y]] or [[Y]]+2 for Bomb Torizo attack ($AA:C5A4).
/// Was `Torizo_Instr_26`.
pub fn bomb_attack<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    choose_attack(k, jp, game, 5)
}

/// Instruction - call [[Y]] or [[Y]]+2 for Golden Torizo attack ($AA:D526).
/// Was `Torizo_Instr_45`.
pub fn golden_attack<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    choose_attack(k, jp, game, 0x20)
}

/// Instruction - spawn Bomb Torizo's chozo orbs ($AA:C5CB). Was
/// `Torizo_Instr_18`.
pub fn bomb_chozo_orbs<L: Lease>(k: u16, game: &mut L) {
    let mut parameter =
        game.spawn_projectile(BOMB_ORBS_INSTRUCTION, k, projectile::BOMB_CHOZO_ORBS);
    parameter = game.spawn_projectile(parameter, k, projectile::BOMB_CHOZO_ORBS);
    game.spawn_projectile(parameter, k, projectile::BOMB_CHOZO_ORBS);
}

/// Instruction - spawn Bomb Torizo sonic boom ($AA:C5E3). Was
/// `Torizo_Instr_20`.
pub fn bomb_sonic_boom<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    game.spawn_projectile(jp.word(0), k, projectile::BOMB_SONIC_BOOM);
    jp.offset(1)
}

/// Instruction - spawn Golden Torizo sonic boom ($AA:C5F2). Was
/// `Torizo_Instr_44`.
pub fn golden_sonic_boom<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    game.spawn_projectile(jp.word(0), k, projectile::GOLDEN_SONIC_BOOM);
    jp.offset(1)
}

/// Instruction - spawn Bomb Torizo explosive swipe ($AA:C601). Was
/// `Torizo_Instr_21`.
pub fn explosive_swipe<L: Lease>(jp: EnemyScript, game: &mut L) -> EnemyScript {
    game.spawn_room_projectile(projectile::EXPLOSIVE_SWIPE, jp.word(0));
    jp.offset(1)
}

/// Instruction - queue shot torizo sound effect ($AA:C610). Was
/// `Torizo_Instr_17`.
pub fn shot_sound<L: Lease>(game: &mut L) {
    game.queue_sound(1, SHOT_SOUND, 6);
}

/// Instruction - queue torizo footsteps sound effect ($AA:C618). Was
/// `Torizo_Instr_13`.
pub fn footstep_sound<L: Lease>(game: &mut L) {
    game.queue_sound(1, FOOTSTEP_SOUND, 6);
}

/// Instruction - go to [[Y]] if not hit ground ($AA:CACE). Was
/// `Torizo_Instr_39`.
pub fn branch_if_airborne<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    if session_slot!(game, TorizoSlot[k as i32].base.y_pos) == GOLDEN_GROUND_Y {
        jp.offset(1)
    } else {
        EnemyScript::jump(jp.word(0))
    }
}

/// Instruction - load Golden Torizo palettes ($AA:CADE). Was
/// `Torizo_Instr_41`.
pub fn golden_palettes<L: Lease>(game: &mut L) {
    game.foreign(|game| game.set_palette_targets(Palettes::GoldenActive));
}

/// Instruction - start fight music and Golden Torizo belly palette FX
/// ($AA:CAE2). Was `Torizo_Instr_42`.
pub fn start_golden_fight<L: Lease>(k: u16, game: &mut L) {
    game.foreign(|game| game.queue_music_delayed(FIGHT_MUSIC));
    let e = i32::from(k);
    session_slot!(game, TorizoSlot[e].base.x_width = 18);
    session_slot!(game, TorizoSlot[e].base.y_height = 48);
    game.foreign(|game| game.spawn_palette_effect(GOLDEN_BELLY_GLOW));
}

/// Instruction - clear torizo caught super missile flag ($AA:CDD7). Was
/// `Torizo_Instr_48`.
pub fn clear_caught_super_missile<L: Lease>(k: u16, game: &mut L) {
    let e = i32::from(k);
    session_slot!(game, TorizoSlot[e].damage, |old| old
        & !CAUGHT_SUPER_MISSILE);
}

/// Instruction - spawn Golden Torizo egg ($AA:D0E9). Was `Torizo_Instr_57`.
pub fn spawn_egg<L: Lease>(game: &mut L) {
    game.spawn_room_projectile(projectile::GOLDEN_EGG, 0);
}

/// Instruction - go to [[Y]] if Golden Torizo egg is active ($AA:D0F3). Was
/// `Torizo_Instr_58`.
pub fn branch_if_egg_active<L: Lease>(jp: EnemyScript, game: &mut L) -> EnemyScript {
    let active = game.foreign(|game| {
        crate::enemy_projectiles::table::with_slots(game, |slots| {
            slots
                .iter()
                .any(|slot| slot.definition == projectile::GOLDEN_EGG)
        })
    });
    if active {
        EnemyScript::jump(jp.word(0))
    } else {
        jp.offset(1)
    }
}

/// Instruction - disable eye beam explosions ($AA:D17B). Was
/// `Torizo_Instr_59`.
pub fn disable_eye_beam_explosions<L: Lease>(k: u16, game: &mut L) {
    let e = i32::from(k);
    session_slot!(game, TorizoSlot[e].eye_beam, |old| old & !EYE_BEAM_EXPLODES);
}

/// Instruction - enable eye beam explosions ($AA:D187). Was
/// `Torizo_Instr_62`.
pub fn enable_eye_beam_explosions<L: Lease>(k: u16, game: &mut L) {
    let e = i32::from(k);
    session_slot!(game, TorizoSlot[e].eye_beam, |old| old | EYE_BEAM_EXPLODES);
}

/// Instruction - unmark Golden Torizo stunned ($AA:D1E7). Was
/// `Torizo_Instr_63`.
pub fn clear_stunned<L: Lease>(k: u16, game: &mut L) {
    let e = i32::from(k);
    session_slot!(game, TorizoSlot[e].damage, |old| old & !STUNNED);
}

/// Instruction - queue Golden Torizo egg released sound effect ($AA:D38F).
/// Was `Torizo_Instr_56`.
pub fn egg_released_sound<L: Lease>(game: &mut L) {
    game.queue_sound(1, EGG_RELEASED_SOUND, 6);
}

/// Instruction - queue laser sound effect ($AA:D397). Was `Torizo_Instr_60`.
pub fn laser_sound<L: Lease>(game: &mut L) {
    game.queue_sound(1, LASER_SOUND, 6);
}

/// Instruction - queue torizo sonic boom sound effect ($AA:D39F). Was
/// `Torizo_Instr_46`.
pub fn sonic_boom_sound<L: Lease>(game: &mut L) {
    game.queue_sound(1, SONIC_BOOM_SOUND, 6);
}

/// Instruction - spawn Golden Torizo super missile ($AA:D3E0). Was
/// `Torizo_Instr_47`.
pub fn golden_super_missile<L: Lease>(game: &mut L) {
    game.spawn_room_projectile(projectile::GOLDEN_SUPER_MISSILE, 0);
}

/// Instruction - go to [[Y]] if Samus is a medium distance behind torizo and
/// morphed ($AA:D3EA). Was `Torizo_Instr_49`.
pub fn branch_if_samus_rolling_behind<L: Lease>(
    k: u16,
    jp: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    if (facing_against_samus(k, game) & 0x8000) == 0
        || !samus_at_least(game, k, 4)
        || samus_at_least(game, k, 0x28)
        || !ROLLING_POSES.contains(&word!(game.SAMUS_POSE))
    {
        return jp.offset(1);
    }
    session_slot!(game, TorizoSlot[k as i32].steps = 0);
    EnemyScript::jump(jp.word(0))
}

/// Instruction - spawn Golden Torizo eye beam ($AA:D436). Was
/// `Torizo_Instr_61`.
pub fn eye_beam<L: Lease>(jp: EnemyScript, game: &mut L) -> EnemyScript {
    game.spawn_room_projectile(projectile::GOLDEN_EYE_BEAM, jp.word(0));
    jp.offset(1)
}

/// Instruction - call [[Y]] with 25% chance if Samus is a medium distance in
/// front of torizo ($AA:D445). Was `Torizo_Instr_53`.
pub fn call_sometimes_if_samus_in_front<L: Lease>(
    k: u16,
    jp: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    if (facing_against_samus(k, game) & 0x8000) != 0
        || !samus_at_least(game, k, 0x20)
        || samus_at_least(game, k, 0x60)
        || (game.roll_random() & 0x110) != 0
    {
        return jp.offset(1);
    }
    session_slot!(
        game,
        TorizoSlot[k as i32].link = list_address(game, k, jp.offset(1))
    );
    EnemyScript::jump(jp.word(0))
}

/// Instruction - call [[Y]] with 25% chance if health <= 1928 ($AA:D474).
/// Was `Torizo_Instr_55`.
pub fn call_sometimes_at_low_health<L: Lease>(
    k: u16,
    jp: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    let e = i32::from(k);
    if session_slot!(game, TorizoSlot[e].base.health) > GOLDEN_LOW_HEALTH
        || (game.roll_random() & 0x102) != 0
    {
        return jp.offset(1);
    }
    session_slot!(game, TorizoSlot[e].steps = 0);
    session_slot!(
        game,
        TorizoSlot[e].link = list_address(game, k, jp.offset(1))
    );
    EnemyScript::jump(jp.word(0))
}

/// Instruction - call [[Y]] if health >= 10800 and stunned ($AA:D49B). Was
/// `Torizo_Instr_52`.
pub fn call_if_stunned_at_high_health<L: Lease>(
    k: u16,
    jp: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    let e = i32::from(k);
    if session_slot!(game, TorizoSlot[e].base.health) <= GOLDEN_HIGH_HEALTH
        || (session_slot!(game, TorizoSlot[e].damage) & STUNNED) == 0
    {
        return jp.offset(1);
    }
    session_slot!(
        game,
        TorizoSlot[e].link = list_address(game, k, jp.offset(1))
    );
    EnemyScript::jump(jp.word(0))
}

/// Instruction - go to [[Y]] and jump forwards if Samus is at least 70h
/// pixels in front of Golden Torizo ($AA:D4BA): also while she space jumps
/// for 360 frames or holds a shoulder button, on a random draw. Was
/// `Torizo_Instr_50`.
pub fn jump_forwards_if_samus_far<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    if !samus_at_least(game, k, 0x70) {
        return jp.offset(1);
    }
    if (facing_against_samus(k, game) & 0x8000) != 0 {
        return jp.offset(1);
    }
    let e = i32::from(k);
    if session_slot!(game, TorizoSlot[e].space_jump_frames) <= 0x168
        && ((word!(game.JOYPAD1_LASTKEYS) & 0x300) == 0 || (game.roll_random() & 0x101) == 0)
    {
        return jp.offset(1);
    }
    session_slot!(game, TorizoSlot[e].steps = 0);
    jump_forwards(k, game);
    EnemyScript::jump(jp.word(0))
}

/// Instruction - spawn Golden Torizo's chozo orbs ($AA:D4F3). Was
/// `Torizo_Instr_43`.
pub fn golden_chozo_orbs<L: Lease>(k: u16, game: &mut L) {
    game.spawn_projectile(GOLDEN_ORBS_INSTRUCTION, k, projectile::GOLDEN_CHOZO_ORBS);
}

/// Instruction - go to [[Y]] and jump backwards if Samus is less than 20h
/// pixels in front of Golden Torizo ($AA:D4FD), or after eight steps. Was
/// `Torizo_Instr_51`.
pub fn golden_jump_back<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    let e = i32::from(k);
    if session_slot!(game, TorizoSlot[e].steps) < 8
        && (samus_at_least(game, k, 0x20) || (facing_against_samus(k, game) & 0x8000) != 0)
    {
        return jp.offset(1);
    }
    session_slot!(game, TorizoSlot[e].steps = 0);
    jump_backwards(k, game);
    EnemyScript::jump(jp.word(0))
}
