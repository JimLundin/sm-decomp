//! The Wrecked Ship spark: a live wire that, once the area's boss is dead, sparks on and
//! off or drips falling sparks on random delays (bank $A8:E587-$A8:E7AB).
#![forbid(unsafe_code)]
use crate::actors::host::EnemyHost;
use crate::actors::layouts::EnemyLayout8 as Spark;
use crate::enemy_lists::{spark_function, spark_instruction_list};
use crate::enemy_script::EnemyScript;
use crate::session_slot;

/// Spark functions, in `var_b`; `var_f` is the function timer and `var_e` its base time.
const ALWAYS_ACTIVE: u16 = 0xe694;
const INACTIVE: u16 = 0xe695;
const ACTIVE: u16 = 0xe6b7;
const EMIT_FALLING_SPARKS: u16 = 0xe6dc;

/// Set Wrecked Ship spark function timer ($A8:E6F6): its base time, or a random 4..43h.
fn reload_timer(game: &mut impl EnemyHost, spark: i32, bias: u16) {
    let time = match session_slot!(game, Spark[spark].var_e) {
        time if time & 0x8000 != 0 => (game.next_random() & 0x3f) + 4,
        time => time,
    };
    session_slot!(game, Spark[spark].var_f = bias.wrapping_add(time));
}
/// Initialisation AI - Wrecked Ship spark ($A8:E637).
pub fn initialize(game: &mut impl EnemyHost) {
    let spark = i32::from(game.current_enemy());
    let kind = (session_slot!(game, Spark[spark].parameter_1) & 3) as isize;
    let function = spark_function(game, kind);
    let time = session_slot!(game, Spark[spark].parameter_2);
    session_slot!(game, Spark[spark]; var_b = function, var_e = time);
    reload_timer(game, spark, 0);
    let list = spark_instruction_list(game, kind);
    session_slot!(game, Spark[spark].base; instruction_timer = 1, current_instruction = list, timer = 0);
    if game.area_boss_flags() & 1 == 0 {
        // As the source does: properties |= [$7E:0100], inside the VRAM write queue.
        session_slot!(game, Spark[spark].base.properties, |old| old
            | game.slot_word(0x100));
    }
}
/// Main AI - Wrecked Ship spark ($A8:E68E): its function when its timer runs out.
pub fn main_ai(game: &mut impl EnemyHost) {
    let k = game.current_enemy();
    let spark = i32::from(k);
    let function = session_slot!(game, Spark[spark].var_b);
    match function {
        ALWAYS_ACTIVE => return,
        INACTIVE | ACTIVE | EMIT_FALLING_SPARKS => (),
        _ => return game.unknown_routine(),
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
pub fn set_intangible(
    game: &mut impl EnemyHost,
    intangible: bool,
    script: EnemyScript,
) -> EnemyScript {
    let spark = i32::from(game.current_enemy());
    let bit = if intangible { 0x400 } else { 0 };
    session_slot!(game, Spark[spark].base.properties, |old| old & !0x400 | bit);
    script
}
