//! The Wrecked Ship spark: a live wire that, once the area's boss is dead, sparks on and
//! off or drips falling sparks on random delays (bank $A8:E587-$A8:E7AB).
#![forbid(unsafe_code)]
use crate::actors::boss_lease::BossLease;
use crate::actors::slot_view::{SlotState, slot_view};
use crate::enemy_lists::{spark_function, spark_instruction_list};
use crate::enemy_script::EnemyScript;

slot_view! {
    /// The spark's slot: its `function`, its `function_timer` and the timer's
    /// `base_time` (negative: random).
    Spark, SparkEdit {
        record: [function = var_b, base_time = var_e, function_timer = var_f],
        extended: [],
    }
}

/// Spark functions, in `function`.
const ALWAYS_ACTIVE: u16 = 0xe694;
const INACTIVE: u16 = 0xe695;
const ACTIVE: u16 = 0xe6b7;
const EMIT_FALLING_SPARKS: u16 = 0xe6dc;
/// $7E:0100, the word the spark ORs into its properties: 30h bytes into the
/// VRAM write queue ($7E:00D0).
const QUEUED_WORD: usize = 0x30;

/// A lease on a game with the sparks' effects.
pub trait Lease: BossLease + SlotState {}
impl<T: BossLease + SlotState + ?Sized> Lease for T {}

/// Set Wrecked Ship spark function timer ($A8:E6F6): its base time, or a random 4..43h.
fn reload_timer(game: &mut impl Lease, k: u16, bias: u16) {
    let time = match Spark::of(game, k).base_time() {
        time if time & 0x8000 != 0 => (game.roll_random() & 0x3f) + 4,
        time => time,
    };
    *SparkEdit::of(game, k).function_timer() = bias.wrapping_add(time);
}
/// Initialisation AI - Wrecked Ship spark ($A8:E637).
pub fn initialize(game: &mut impl Lease) {
    let k = game.current_enemy();
    let kind = (Spark::of(game, k).actor().parameters[0] & 3) as isize;
    let function = spark_function(game, kind);
    let time = Spark::of(game, k).actor().parameters[1];
    {
        let mut spark = SparkEdit::of(game, k);
        *spark.function() = function;
        *spark.base_time() = time;
    }
    reload_timer(game, k, 0);
    let list = spark_instruction_list(game, kind);
    {
        let mut spark = SparkEdit::of(game, k);
        let actor = spark.actor();
        actor.instruction_delay = 1;
        actor.instruction = list;
        actor.timer = 0;
    }
    if game.area_boss_byte() & 1 == 0 {
        // As the source does: properties |= [$7E:0100], inside the VRAM write queue.
        let queue = &game.session().queued_tilemaps.queue;
        let queued = u16::from_le_bytes([queue[QUEUED_WORD], queue[QUEUED_WORD + 1]]);
        SparkEdit::of(game, k).actor().properties |= queued;
    }
}
/// Enemy shot - Wrecked Ship spark ($A8:E70E), as the blue Brinstar face block's ($A8:E91D):
/// the collided projectile is not deleted.
pub fn pass_shot(runtime: &mut impl crate::actors::species::TableLease) {
    use crate::player_projectiles::access::shot;
    let raw = runtime
        .session()
        .room
        .collision_detection_index
        .wrapping_mul(2);
    let direction = shot!(runtime, raw, direction) & !0x10;
    shot!(runtime, raw, direction = direction; SliceLe);
}
/// Main AI - Wrecked Ship spark ($A8:E68E): its function when its timer runs out.
pub fn main_ai(game: &mut impl Lease) {
    let k = game.current_enemy();
    let function = Spark::of(game, k).function();
    match function {
        ALWAYS_ACTIVE => return,
        INACTIVE | ACTIVE | EMIT_FALLING_SPARKS => (),
        _ => game.unreachable(),
    }
    if Spark::of(game, k).function_timer() != 1 {
        let mut spark = SparkEdit::of(game, k);
        let timer = spark.function_timer();
        *timer = timer.wrapping_sub(1);
    } else if function == EMIT_FALLING_SPARKS {
        game.spawn_projectile(0, k, 0xf498);
        reload_timer(game, k, 0);
    } else {
        let active = function == INACTIVE;
        let (next, list) = [(INACTIVE, 0xe5e5), (ACTIVE, 0xe5a7)][usize::from(active)];
        {
            let mut spark = SparkEdit::of(game, k);
            *spark.function() = next;
            let actor = spark.actor();
            actor.instruction = list;
            actor.instruction_delay = 1;
        }
        reload_timer(game, k, if active { 0 } else { 8 });
    }
}
/// Instruction - set enemy as intangible ($A8:E61D) or tangible ($A8:E62A).
pub fn set_intangible(game: &mut impl Lease, intangible: bool, script: EnemyScript) -> EnemyScript {
    let k = game.current_enemy();
    let bit = if intangible { 0x400 } else { 0 };
    let mut spark = SparkEdit::of(game, k);
    let actor = spark.actor();
    actor.properties = (actor.properties & !0x400) | bit;
    script
}
