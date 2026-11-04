//! The fire arc (bank $A6:9470-$A6:998B): a chain of fireballs swinging along
//! an arc around its spawn point, and the spike platform's bottom half.
//!
//! Was `NuclearWaffle_*` and `SpikeyPlatform2ndEnemy_Init` in the original
//! decompilation. The arc's body is five enemy projectiles and three sprite
//! objects; the active swing runs with them in the enemy projectile code.
use crate::actors::boss_lease::BossLease;
use crate::actors::layouts::EnemyLayout104;
use crate::actors::slot_view::{SlotState, extended_var, slot_view};
use crate::constants::enemy_properties::K_ENEMY_PROPS_PROCESSED_OFFSCREEN;
use core::mem::offset_of;
use sm_data::enemy_speeds::SpeedTable;

/// A lease on a game with the fire arc's effects.
pub trait Lease: BossLease + SlotState + crate::sprite_effect_host::Host {
    /// Fire arc function - active ($A6:9682): advance the orbit, whose
    /// projectiles run in the host. Was `NuclearWaffle_Func_2`.
    fn advance_orbit(&mut self);
}

slot_view! {
    /// The fire arc's slot. Was `Enemy_NuclearWaffle`. The eight words from
    /// `var_08` hold the projectiles' explosions and the eight from `var_18`
    /// the sprite objects'; `var_10` is the first body part's sprite object,
    /// the others following at the body index's byte offset. The falling and
    /// rising explosion angles are `var_2f` and `var_30`
    /// ([`set_explosion_angles`]).
    FireArc, FireArcEdit {
        record: [
            function = var_a, inactive_timer = var_b, speed_index = var_c, radius = var_d,
            direction = var_e, inactive_reset = var_f, parameter_1 = parameter_1,
            parameter_2 = parameter_2,
        ],
        extended: [
            angle = var_21, start_angle = var_23, angle_delta_subpixel = var_24,
            angle_delta = var_25, origin_x = var_26, origin_y = var_27, spawn_x = var_28,
            spawn_y = var_29, body_index = var_2a, part_angle_2x = var_2b, part_angle = var_2c,
            finish_angle = var_2e, falling_explosion_angle = var_2f,
            rising_explosion_angle = var_30, head_explosion = var_33, graphics = var_34,
        ],
    }
}

slot_view! {
    /// The spike platform halves' slots.
    Platform, PlatformEdit {
        record: [],
        extended: [],
    }
}

impl<H: SlotState + ?Sized> FireArcEdit<'_, H> {
    /// Zero the eight extended words from `var_xx` = `first`, in order.
    fn clear_eight(&mut self, first: usize) {
        for var in first..first + 8 {
            *self.0.extended(var) = 0;
        }
    }

    /// The sprite object of the body part at byte offset `index` from
    /// `var_10`, to store to.
    fn sprite_object_at(&mut self, index: u16) -> &mut u16 {
        debug_assert!(index.is_multiple_of(2));
        self.0
            .extended(extended_var(offset_of!(EnemyLayout104, var_10)) + usize::from(index / 2))
    }
}

/// Store fire arc `k`'s rising then falling explosion angle. Slot 0's
/// `var_30` and `var_2f` ($7E:8020 and $7E:801E) are the Session's Botwoon
/// death words, which the codec projects those bytes to.
fn set_explosion_angles<L: Lease>(game: &mut L, k: u16, rising: u16, falling: u16) {
    if k == 0 {
        let death = &mut game.session_mut().botwoon_death;
        death.active = rising;
        death.pending_hit = falling;
        game.record_stored();
    } else {
        let mut arc = FireArcEdit::of(game, k);
        *arc.rising_explosion_angle() = rising;
        *arc.falling_explosion_angle() = falling;
    }
}

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

/// Initialisation AI - enemy $E0BF (fire arc) ($A6:94C4): the parameters'
/// bytes are the speed, radius, direction and inactive time; the body's
/// projectiles and sprite objects spawn at the arc's start. Was
/// `NuclearWaffle_Init`.
pub fn initialize<L: Lease>(game: &mut L) {
    crate::probe::hit("fire_arc.initialize");
    let k = game.current_enemy();
    FireArcEdit::of(game, k).actor().instruction = INSTRUCTION_LIST;
    *FireArcEdit::of(game, k).speed_index() =
        FireArc::of(game, k).parameter_1().to_le_bytes()[0] as u16;
    *FireArcEdit::of(game, k).radius() = FireArc::of(game, k).parameter_1().to_le_bytes()[1] as u16;
    *FireArcEdit::of(game, k).direction() =
        FireArc::of(game, k).parameter_2().to_le_bytes()[0] as u16;
    let inactive = FireArc::of(game, k).parameter_2().to_le_bytes()[1] as u16;
    *FireArcEdit::of(game, k).inactive_reset() = inactive;
    *FireArcEdit::of(game, k).inactive_timer() = inactive;
    *FireArcEdit::of(game, k).function() = function::INACTIVE;
    let pair = 2 * usize::from(FireArc::of(game, k).direction());
    let start = START_AND_FINISH[pair];
    {
        let mut arc = FireArcEdit::of(game, k);
        *arc.start_angle() = start;
        *arc.angle() = start;
        *arc.finish_angle() = START_AND_FINISH[pair + 1];
        *arc.part_angle_2x() = PART_ANGLES[pair];
        *arc.part_angle() = PART_ANGLES[pair + 1];
    }
    set_explosion_angles(game, k, EXPLOSION_ANGLES[pair], EXPLOSION_ANGLES[pair + 1]);
    // Linear speed table rows are 8 bytes; the second half negates the speed.
    let mut speed = 8u16.wrapping_mul(FireArc::of(game, k).speed_index());
    if FireArc::of(game, k).direction() == 0 {
        speed = speed.wrapping_add(4);
    }
    let speed = (speed >> 1) as isize;
    *FireArcEdit::of(game, k).angle_delta() = game.enemy_speed(SpeedTable::Linear, speed);
    *FireArcEdit::of(game, k).angle_delta_subpixel() =
        game.enemy_speed(SpeedTable::Linear, speed + 1);
    *FireArcEdit::of(game, k).origin_x() = FireArc::of(game, k).actor().x.whole;
    *FireArcEdit::of(game, k).origin_y() = FireArc::of(game, k).actor().y.whole;
    let x = FireArc::of(game, k)
        .origin_x()
        .wrapping_add(game.scaled_cosine(
            FireArc::of(game, k).start_angle(),
            FireArc::of(game, k).radius(),
        ));
    *FireArcEdit::of(game, k).spawn_x() = x;
    FireArcEdit::of(game, k).actor().x.whole = x;
    let y = FireArc::of(game, k)
        .origin_y()
        .wrapping_add(game.scaled_sine(
            FireArc::of(game, k).start_angle(),
            FireArc::of(game, k).radius(),
        ));
    *FireArcEdit::of(game, k).spawn_y() = y;
    FireArcEdit::of(game, k).actor().y.whole = y;
    // Body indices 8 down to 2 are projectiles; 6 down to 2 also sprite objects.
    let mut index = 8;
    *FireArcEdit::of(game, k).body_index() = 8;
    loop {
        game.spawn_actor_eproj(index, game.current_enemy(), BODY_PROJECTILE);
        index = FireArc::of(game, k).body_index().wrapping_sub(2);
        *FireArcEdit::of(game, k).body_index() = index;
        if index == 0 {
            break;
        }
    }
    *FireArcEdit::of(game, k).body_index() = 6;
    loop {
        let graphics = FireArc::of(game, k).actor().tile | FireArc::of(game, k).actor().palette;
        *FireArcEdit::of(game, k).graphics() = graphics;
        let x = FireArc::of(game, k).actor().x.whole;
        let y = FireArc::of(game, k).actor().y.whole;
        let sprite = crate::actors::boss_lease::steps::create_sprite_appearing(
            game,
            x,
            y,
            BODY_SPRITE,
            graphics,
        );
        // The sprite object words are indexed by the body index's bytes.
        let index = FireArc::of(game, k).body_index();
        *FireArcEdit::of(game, k).sprite_object_at(index) = sprite;
        let index = FireArc::of(game, k).body_index().wrapping_sub(2);
        *FireArcEdit::of(game, k).body_index() = index;
        if index == 0 {
            break;
        }
    }
}

/// Main AI - enemy $E0BF (fire arc) ($A6:960E). Was `NuclearWaffle_Main`.
pub fn main_ai<L: Lease>(game: &mut L) {
    let k = game.current_enemy();
    match FireArc::of(game, k).function() {
        function::INACTIVE => inactive(game),
        function::ACTIVE => game.advance_orbit(),
        _ => game.unreachable(),
    }
}

/// Fire arc function - inactive ($A6:9615): when the inactive timer runs out,
/// swing from the start angle with every explosion cleared, and keep running
/// off-screen. Was `NuclearWaffle_Func_1`.
pub fn inactive<L: Lease>(game: &mut L) {
    let k = game.current_enemy();
    *FireArcEdit::of(game, k).inactive_timer() =
        FireArc::of(game, k).inactive_timer().wrapping_sub(1);
    if (FireArc::of(game, k).inactive_timer() & 0x8000) == 0 {
        return;
    }
    *FireArcEdit::of(game, k).inactive_timer() = FireArc::of(game, k).inactive_reset();
    *FireArcEdit::of(game, k).angle() = FireArc::of(game, k).start_angle();
    *FireArcEdit::of(game, k).function() = function::ACTIVE;
    *FireArcEdit::of(game, k).head_explosion() = 0;
    {
        let mut arc = FireArcEdit::of(game, k);
        arc.clear_eight(extended_var(offset_of!(EnemyLayout104, var_08)));
        arc.clear_eight(extended_var(offset_of!(EnemyLayout104, var_18)));
    }
    FireArcEdit::of(game, k).actor().properties |= K_ENEMY_PROPS_PROCESSED_OFFSCREEN as u16;
}

/// Initialisation AI - enemy $E03F (spike platform bottom) ($A6:8B85): 12px
/// below the top half, the enemy before it. Was `SpikeyPlatform2ndEnemy_Init`.
pub fn initialize_spike_platform_bottom<L: Lease>(game: &mut L) {
    let k = game.current_enemy();
    PlatformEdit::of(game, k).actor().x.whole = Platform::of(game, k - 0x40).actor().x.whole;
    PlatformEdit::of(game, k).actor().y.whole = Platform::of(game, k - 0x40)
        .actor()
        .y
        .whole
        .wrapping_add(12);
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
