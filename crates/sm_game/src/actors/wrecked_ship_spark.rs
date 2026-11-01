//! The Wrecked Ship spark: a live wire that, once the area's boss is dead, sparks on and
//! off or drips falling sparks on random delays (bank $A8:E587-$A8:E7AB).
#![forbid(unsafe_code)]
use crate::actors::boss_lease::BossLease;
use crate::actors::host::BossEffects;
use crate::actors::layouts::EnemyLayout8 as Spark;
use crate::enemy_lists::{spark_function, spark_instruction_list};
use crate::enemy_script::EnemyScript;
use crate::session_slot;

/// Spark functions, in `var_b`; `var_f` is the function timer and `var_e` its base time.
const ALWAYS_ACTIVE: u16 = 0xe694;
const INACTIVE: u16 = 0xe695;
const ACTIVE: u16 = 0xe6b7;
const EMIT_FALLING_SPARKS: u16 = 0xe6dc;

/// A lease on a game with the sparks' effects.
pub trait Lease: BossLease<Game: BossEffects> {}
impl<T: BossLease<Game: BossEffects> + ?Sized> Lease for T {}

/// Set Wrecked Ship spark function timer ($A8:E6F6): its base time, or a random 4..43h.
fn reload_timer(game: &mut impl Lease, spark: i32, bias: u16) {
    let time = match session_slot!(game, Spark[spark].var_e) {
        time if time & 0x8000 != 0 => (game.roll_random() & 0x3f) + 4,
        time => time,
    };
    session_slot!(game, Spark[spark].var_f = bias.wrapping_add(time));
}
/// Initialisation AI - Wrecked Ship spark ($A8:E637).
pub fn initialize(game: &mut impl Lease) {
    let spark = i32::from(game.current_enemy());
    let kind = (session_slot!(game, Spark[spark].parameter_1) & 3) as isize;
    let function = spark_function(game, kind);
    let time = session_slot!(game, Spark[spark].parameter_2);
    session_slot!(game, Spark[spark]; var_b = function, var_e = time);
    reload_timer(game, spark, 0);
    let list = spark_instruction_list(game, kind);
    session_slot!(game, Spark[spark].base; instruction_timer = 1, current_instruction = list, timer = 0);
    if game.area_boss_byte() & 1 == 0 {
        // As the source does: properties |= [$7E:0100], inside the VRAM write queue.
        session_slot!(game, Spark[spark].base.properties, |old| old
            | game.slot_word(0x100));
    }
}
/// Enemy shot - Wrecked Ship spark ($A8:E70E), as the blue Brinstar face block's ($A8:E91D):
/// the collided projectile is not deleted.
pub fn pass_shot<H: crate::actors::host::LeaseActors + ?Sized>(game: &mut H) {
    use crate::actors::host::ActorLease as _;
    use crate::ram::{word_arrays::PROJECTILE_DIR, words::COLLISION_DETECTION_INDEX};
    let mut runtime = game.lease();
    let at = PROJECTILE_DIR + 2 * usize::from(runtime.word(COLLISION_DETECTION_INDEX));
    let direction = runtime.word(at) & !0x10;
    runtime.write_slice_le(at, direction);
}
/// Main AI - Wrecked Ship spark ($A8:E68E): its function when its timer runs out.
pub fn main_ai(game: &mut impl Lease) {
    let k = game.current_enemy();
    let spark = i32::from(k);
    let function = session_slot!(game, Spark[spark].var_b);
    match function {
        ALWAYS_ACTIVE => return,
        INACTIVE | ACTIVE | EMIT_FALLING_SPARKS => (),
        _ => return game.foreign(|game| game.unreachable()),
    }
    if session_slot!(game, Spark[spark].var_f) != 1 {
        session_slot!(game, Spark[spark].var_f, |old| old.wrapping_sub(1));
    } else if function == EMIT_FALLING_SPARKS {
        game.spawn_projectile(0, k, 0xf498);
        reload_timer(game, spark, 0);
    } else {
        let active = function == INACTIVE;
        let (next, list) = [(INACTIVE, 0xe5e5), (ACTIVE, 0xe5a7)][usize::from(active)];
        session_slot!(game, Spark[spark].var_b = next);
        session_slot!(game, Spark[spark].base; current_instruction = list, instruction_timer = 1);
        reload_timer(game, spark, if active { 0 } else { 8 });
    }
}
/// Instruction - set enemy as intangible ($A8:E61D) or tangible ($A8:E62A).
pub fn set_intangible(game: &mut impl Lease, intangible: bool, script: EnemyScript) -> EnemyScript {
    let spark = i32::from(game.current_enemy());
    let bit = if intangible { 0x400 } else { 0 };
    session_slot!(game, Spark[spark].base.properties, |old| old & !0x400 | bit);
    script
}
