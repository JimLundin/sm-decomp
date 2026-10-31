//! The fire arc (bank $A6:9470-$A6:998B): a chain of fireballs swinging along
//! an arc around its spawn point, and the spike platform's bottom half.
//!
//! Was `NuclearWaffle_*` and `SpikeyPlatform2ndEnemy_Init` in the original
//! decompilation. The arc's body is five enemy projectiles and three sprite
//! objects; the active swing runs with them in the enemy projectile code.
use crate::actors::layouts::{EnemyBase, EnemyLayout8, same_words};
use crate::constants::enemy_properties::K_ENEMY_PROPS_PROCESSED_OFFSCREEN;
use crate::ram::RawRam as _;
use crate::ram::word;
use crate::session_slot;
use sm_data::enemy_speeds::SpeedTable;

/// What the fire arc needs beyond the shared boss effects.
pub trait Host: crate::actors::host::BossEffects {
    /// Fire arc function - active ($A6:9682): advance the orbit, whose
    /// projectiles run in the host. Was `NuclearWaffle_Func_2`.
    fn advance_orbit(&mut self);
}

/// The fire arc's view of its slot and two RAM blocks. Was
/// `Enemy_NuclearWaffle`.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct FireArc {
    pub base: EnemyBase,
    pub function: u16,
    pub inactive_timer: u16,
    pub speed_index: u16,
    pub radius: u16,
    pub direction: u16,
    pub inactive_reset: u16,
    pub parameter_1: u16,
    pub parameter_2: u16,
    pad1: [u8; 26696],
    pad_7800: [u16; 8],
    pub projectile_explosions: [u16; 5],
    unread_781a: [u16; 3],
    /// The first body part's sprite object; the others follow at the body
    /// index's byte offset.
    pub sprite_object: u16,
    pad_7822: [u16; 7],
    pub sprite_explosions: [u16; 4],
    unread_7838: [u16; 4],
    pad2: [u8; 1984],
    pad_8000: [u16; 1],
    pub angle: u16,
    pad_8004: [u16; 1],
    pub start_angle: u16,
    pub angle_delta_subpixel: u16,
    pub angle_delta: u16,
    pub origin_x: u16,
    pub origin_y: u16,
    pub spawn_x: u16,
    pub spawn_y: u16,
    pub body_index: u16,
    pub part_angle_2x: u16,
    pub part_angle: u16,
    pad_801a: [u16; 1],
    pub finish_angle: u16,
    pub falling_explosion_angle: u16,
    pub rising_explosion_angle: u16,
    pad_8022: [u16; 2],
    pub head_explosion: u16,
    pub graphics: u16,
    pad_802a: [u16; 11],
}

same_words!(FireArc: EnemyLayout72 {
    function = var_a, inactive_timer = var_b, speed_index = var_c, radius = var_d,
    direction = var_e, inactive_reset = var_f, parameter_1 = parameter_1,
    parameter_2 = parameter_2, projectile_explosions = var_08, sprite_object = var_10,
    sprite_explosions = var_18, angle = var_21, start_angle = var_23,
    angle_delta_subpixel = var_24, angle_delta = var_25, origin_x = var_26, origin_y = var_27,
    spawn_x = var_28, spawn_y = var_29, body_index = var_2a, part_angle_2x = var_2b,
    part_angle = var_2c, finish_angle = var_2e, falling_explosion_angle = var_2f,
    rising_explosion_angle = var_30, head_explosion = var_33, graphics = var_34,
});

/// Bank $A6 fire arc functions, stored in `function`.
pub mod function {
    pub const INACTIVE: u16 = 0x9615;
    pub const ACTIVE: u16 = 0x9682;
}

/// The fire arc's instruction list.
pub const INSTRUCTION_LIST: u16 = 0x9490;
/// Bank $86 fire arc body enemy projectile.
pub const BODY_PROJECTILE: u16 = 0xbbc7;
/// The fire arc body sprite object.
const BODY_SPRITE: u16 = 43;

/// The arc angles ($A6:95F6), clockwise then anticlockwise: start and finish,
/// twice and once the angle between body parts, rising and falling explosion.
const START_AND_FINISH: [u16; 4] = [0x190, 0xf0, 0xf0, 0x190];
const PART_ANGLES: [u16; 4] = [0xffe8, 0xfff4, 0x18, 0xc];
const EXPLOSION_ANGLES: [u16; 4] = [0x180, 0x100, 0x100, 0x180];

fn current<H: Host + ?Sized>(game: &H) -> i32 {
    word!(game.CUR_ENEMY_INDEX) as i32
}

/// Initialisation AI - enemy $E0BF (fire arc) ($A6:94C4): the parameters'
/// bytes are the speed, radius, direction and inactive time; the body's
/// projectiles and sprite objects spawn at the arc's start. Was
/// `NuclearWaffle_Init`.
pub fn initialize<H: Host + ?Sized>(game: &mut H) {
    crate::probe::hit("fire_arc.initialize");
    let e = current(game);
    session_slot!(game, FireArc[e].base.current_instruction = INSTRUCTION_LIST);
    session_slot!(
        game,
        FireArc[e].speed_index = session_slot!(byte game, FireArc[e].parameter_1[0]) as u16
    );
    session_slot!(
        game,
        FireArc[e].radius = session_slot!(byte game, FireArc[e].parameter_1[1]) as u16
    );
    session_slot!(
        game,
        FireArc[e].direction = session_slot!(byte game, FireArc[e].parameter_2[0]) as u16
    );
    let inactive = session_slot!(byte game, FireArc[e].parameter_2[1]) as u16;
    session_slot!(game, FireArc[e]; inactive_reset = inactive, inactive_timer = inactive,
        function = function::INACTIVE);
    let pair = 2 * usize::from(session_slot!(game, FireArc[e].direction));
    let start = START_AND_FINISH[pair];
    session_slot!(game, FireArc[e]; start_angle = start, angle = start,
        finish_angle = START_AND_FINISH[pair + 1], part_angle_2x = PART_ANGLES[pair],
        part_angle = PART_ANGLES[pair + 1], rising_explosion_angle = EXPLOSION_ANGLES[pair],
        falling_explosion_angle = EXPLOSION_ANGLES[pair + 1], );
    // Linear speed table rows are 8 bytes; the second half negates the speed.
    let mut speed = 8u16.wrapping_mul(session_slot!(game, FireArc[e].speed_index));
    if session_slot!(game, FireArc[e].direction) == 0 {
        speed = speed.wrapping_add(4);
    }
    let speed = (speed >> 1) as isize;
    session_slot!(
        game,
        FireArc[e].angle_delta = game.enemy_speed(SpeedTable::Linear, speed)
    );
    session_slot!(
        game,
        FireArc[e].angle_delta_subpixel = game.enemy_speed(SpeedTable::Linear, speed + 1)
    );
    session_slot!(
        game,
        FireArc[e].origin_x = session_slot!(game, FireArc[e].base.x_pos)
    );
    session_slot!(
        game,
        FireArc[e].origin_y = session_slot!(game, FireArc[e].base.y_pos)
    );
    let x = session_slot!(game, FireArc[e].origin_x).wrapping_add(game.actor_cosine(
        session_slot!(game, FireArc[e].start_angle),
        session_slot!(game, FireArc[e].radius),
    ));
    session_slot!(game, FireArc[e].spawn_x = x);
    session_slot!(game, FireArc[e].base.x_pos = x);
    let y = session_slot!(game, FireArc[e].origin_y).wrapping_add(game.actor_sine(
        session_slot!(game, FireArc[e].start_angle),
        session_slot!(game, FireArc[e].radius),
    ));
    session_slot!(game, FireArc[e].spawn_y = y);
    session_slot!(game, FireArc[e].base.y_pos = y);
    // Body indices 8 down to 2 are projectiles; 6 down to 2 also sprite objects.
    let mut index = 8;
    session_slot!(game, FireArc[e].body_index = 8);
    loop {
        game.spawn_projectile(index, word!(game.CUR_ENEMY_INDEX), BODY_PROJECTILE);
        index = session_slot!(game, FireArc[e].body_index).wrapping_sub(2);
        session_slot!(game, FireArc[e].body_index = index);
        if index == 0 {
            break;
        }
    }
    session_slot!(game, FireArc[e].body_index = 6);
    loop {
        let graphics = session_slot!(game, FireArc[e].base.vram_tiles_index)
            | session_slot!(game, FireArc[e].base.palette_index);
        session_slot!(game, FireArc[e].graphics = graphics);
        let x = session_slot!(game, FireArc[e].base.x_pos);
        let y = session_slot!(game, FireArc[e].base.y_pos);
        let sprite = game.create_sprite(x, y, BODY_SPRITE, graphics);
        // The sprite object words are indexed by the body index's bytes.
        let part =
            word!(game.CUR_ENEMY_INDEX) as i32 + session_slot!(game, FireArc[e].body_index) as i32;
        session_slot!(game, FireArc[part].sprite_object = sprite);
        let index = session_slot!(game, FireArc[e].body_index).wrapping_sub(2);
        session_slot!(game, FireArc[e].body_index = index);
        if index == 0 {
            break;
        }
    }
}

/// Main AI - enemy $E0BF (fire arc) ($A6:960E). Was `NuclearWaffle_Main`.
pub fn main_ai<H: Host + ?Sized>(game: &mut H) {
    let e = current(game);
    match session_slot!(game, FireArc[e].function) {
        function::INACTIVE => inactive(game),
        function::ACTIVE => game.advance_orbit(),
        _ => game.unreachable(),
    }
}

/// Fire arc function - inactive ($A6:9615): when the inactive timer runs out,
/// swing from the start angle with every explosion cleared, and keep running
/// off-screen. Was `NuclearWaffle_Func_1`.
pub fn inactive<H: Host + ?Sized>(game: &mut H) {
    let e = current(game);
    session_slot!(game, FireArc[e].inactive_timer, |old| old.wrapping_sub(1));
    if (session_slot!(game, FireArc[e].inactive_timer) & 0x8000) == 0 {
        return;
    }
    session_slot!(
        game,
        FireArc[e].inactive_timer = session_slot!(game, FireArc[e].inactive_reset)
    );
    session_slot!(game, FireArc[e]; angle = session_slot!(game, FireArc[e].start_angle),
        function = function::ACTIVE, head_explosion = 0);
    clear_words(
        game,
        crate::ram_slot!(@at FireArc, e, projectile_explosions),
        8,
    );
    clear_words(game, crate::ram_slot!(@at FireArc, e, sprite_explosions), 8);
    session_slot!(game, FireArc[e].base.properties, |old| old
        | K_ENEMY_PROPS_PROCESSED_OFFSCREEN as u16);
}

/// Zero `count` words from `at`, in order.
fn clear_words<H: Host + ?Sized>(game: &mut H, at: usize, count: usize) {
    for n in 0..count {
        game.ram().write_byte(at + 2 * n, 0);
        game.ram().write_byte(at + 2 * n + 1, 0);
    }
}

/// Initialisation AI - enemy $E03F (spike platform bottom) ($A6:8B85): 12px
/// below the top half, the enemy before it. Was `SpikeyPlatform2ndEnemy_Init`.
pub fn initialize_spike_platform_bottom<H: Host + ?Sized>(game: &mut H) {
    let e = current(game);
    session_slot!(
        game,
        EnemyLayout8[e].base.x_pos = session_slot!(game, EnemyLayout8[e - 0x40].base.x_pos)
    );
    session_slot!(
        game,
        EnemyLayout8[e].base.y_pos =
            session_slot!(game, EnemyLayout8[e - 0x40].base.y_pos).wrapping_add(12)
    );
}

// The fire arc's functions and data are its dispatch IDs and the
// decompilation's addresses.
const _: () = {
    use crate::actors::fire_arc::{self as a, function as f};
    use crate::constants::{bank_86::*, bank_a6::*};
    assert!((0xa6_0000 | f::INACTIVE as u32) == 10917397);
    assert!((0xa6_0000 | f::ACTIVE as u32) == 10917506);
    assert!(a::INSTRUCTION_LIST == ADDR_K_FIRE_ARC_ILIST_9490 as u16);
    assert!(a::BODY_PROJECTILE == ADDR_K_EPROJ_FIRE_ARC_BODY as u16);
};
