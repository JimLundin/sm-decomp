//! The escape etecoons and escape dachora, the critters Samus can free during
//! the escape (bank $B3:E525-$B3:EB19).
//!
//! Was `escape_animals`. Each critter waits or runs around the escape room
//! until the critters-escaped event, then runs for the exit.
use crate::actors::boss_lease::BossState;
use crate::actors::host::Slopes;
use crate::actors::slot_view::{SlotState, slot_view};
use crate::constants::enemy_properties::*;
use crate::enemy_script::EnemyScript;
use sm_data::escape_etecoons::EscapeEtecoon;

/// What the escape critters need beyond a boss's state; their slots are the
/// Session's.
pub trait Host: BossState + SlotState {
    /// Whether game event `event` has happened.
    fn event_happened(&mut self, event: u16) -> bool;
    /// Etecoon `index`'s initial state at $B3:E718.
    fn escape_etecoon(&self, index: isize) -> EscapeEtecoon;
    /// The original failure for a function no routine answers.
    fn unknown_function(&mut self);
}

slot_view! {
    /// A critter's view of its slot.
    Critter, CritterEdit {
        record: [x_velocity = var_a, function = var_f, etecoon_index = parameter_1],
        extended: [],
    }
}

/// Bank $B3 instruction lists.
pub mod instruction_list {
    pub const ETECOON_RUNNING_LEFT: u16 = 0xe556;
    pub const ETECOON_RUNNING_RIGHT: u16 = 0xe582;
    pub const ETECOON_RUNNING_FOR_ESCAPE: u16 = 0xe5ae;
    pub const ETECOON_EXPRESS_GRATITUDE_THEN_ESCAPE: u16 = 0xe5da;
    pub const DACHORA_RUNNING_AROUND_AIMLESSLY: u16 = 0xe964;
}

/// Bank $B3 escape etecoon functions, stored in `function`.
pub mod function {
    pub const RUNNING_FOR_ESCAPE: u16 = 0xe65c;
    pub const STATIONARY: u16 = 0xe670;
    pub const RUNNING_AROUND_AIMLESSLY: u16 = 0xe680;
}

/// Bank $B3 instructions the critters' scripts call.
pub mod instruction {
    pub const ETECOON_GO_TO_IF_LOW_TIDE: u32 = 0xb3_e545;
    pub const ETECOON_X_POSITION_PLUS: u32 = 0xb3_e610;
    pub const DACHORA_GO_TO_IF_LOW_TIDE: u32 = 0xb3_eaa8;
    pub const DACHORA_GO_TO_IF_CRITTERS_ESCAPED: u32 = 0xb3_eab8;
    pub const DACHORA_X_POSITION_MINUS_6: u32 = 0xb3_eac9;
    pub const DACHORA_X_POSITION_PLUS_6: u32 = 0xb3_ead7;
}

/// Game event 0Fh: the critters escaped.
const CRITTERS_ESCAPED: u16 = 0xf;
/// Properties bit 13, process instructions (the decompiler's
/// `K_ENEMY_PROPS_DISABLE_SAMUS_COLL`); the etecoons also set bit 10, intangible
/// (`K_ENEMY_PROPS_TANGIBLE`), and bit 15.
const PROCESS_INSTRUCTIONS: u16 = K_ENEMY_PROPS_DISABLE_SAMUS_COLL as u16;
const ETECOON_PROPERTIES: u16 =
    (K_ENEMY_PROPS_DISABLE_SAMUS_COLL | K_ENEMY_PROPS_TANGIBLE) as u16 | 0x8000;
/// $B3:804D, the dachora's placeholder spritemap.
const NO_SPRITEMAP: u16 = 0x804d;
/// Running for escape moves 3.8000h px right a frame.
const ESCAPE_SPEED: u32 = 0x3_8000;
/// The tide is low while the acid is below Y CEh.
const LOW_TIDE_ACID_Y: u16 = 0xce;
const SIGN: i32 = 0x8000;

/// Initialisation AI (escape etecoon $B3:E6CB, escape dachora $B3:EAE5):
/// once the critters have escaped, delete the critter; otherwise start it.
pub fn initialize(dachora: bool, game: &mut (impl Host + ?Sized)) {
    let e = game.current_enemy();
    if game.event_happened(CRITTERS_ESCAPED) {
        CritterEdit::of(game, e).actor().properties |= K_ENEMY_PROPS_DELETED as u16;
        return;
    }
    let flags = if dachora {
        PROCESS_INSTRUCTIONS
    } else {
        ETECOON_PROPERTIES
    };
    CritterEdit::of(game, e).actor().properties |= flags;
    if dachora {
        let mut edit = CritterEdit::of(game, e);
        edit.actor().spritemap = NO_SPRITEMAP;
        edit.actor().instruction_delay = 1;
        edit.actor().timer = 0;
        edit.actor().instruction = instruction_list::DACHORA_RUNNING_AROUND_AIMLESSLY;
    } else {
        {
            let mut edit = CritterEdit::of(game, e);
            edit.actor().instruction_delay = 1;
            edit.actor().timer = 0;
            edit.actor().palette = 0;
        }
        let index = (Critter::of(game, e).etecoon_index() >> 1) as isize;
        let etecoon = game.escape_etecoon(index);
        {
            let mut edit = CritterEdit::of(game, e);
            edit.actor().x.whole = etecoon.x_position;
            edit.actor().y.whole = etecoon.y_position;
        }
        *CritterEdit::of(game, e).function() = etecoon.function;
        CritterEdit::of(game, e).actor().instruction = etecoon.instruction_list;
        *CritterEdit::of(game, e).x_velocity() = etecoon.x_velocity;
    }
}

/// Main AI - escape etecoon ($B3:E655): run the etecoon's function.
pub fn main_ai(game: &mut (impl Host + ?Sized)) {
    let k = game.current_enemy();
    match Critter::of(game, k).function() {
        function::RUNNING_FOR_ESCAPE => run_for_escape(k, game),
        function::STATIONARY => wait_to_express_gratitude(k, game),
        function::RUNNING_AROUND_AIMLESSLY => run_around_aimlessly(k, game),
        _ => game.unknown_function(),
    }
}

/// Run one of the critters' script instructions (one of [`instruction`]), for
/// critter `k`.
pub fn instruction(
    address: u32,
    k: u16,
    script: EnemyScript,
    game: &mut (impl Host + ?Sized),
) -> EnemyScript {
    match address {
        instruction::ETECOON_GO_TO_IF_LOW_TIDE | instruction::DACHORA_GO_TO_IF_LOW_TIDE => {
            go_to_if(false, script, game)
        }
        instruction::DACHORA_GO_TO_IF_CRITTERS_ESCAPED => go_to_if(true, script, game),
        instruction::ETECOON_X_POSITION_PLUS => move_x(k, script, game),
        instruction::DACHORA_X_POSITION_MINUS_6 => shift_dachora(0xfffa, script, game),
        instruction::DACHORA_X_POSITION_PLUS_6 => shift_dachora(6, script, game),
        _ => unreachable!("unrecognized escape critter instruction"),
    }
}

/// Escape etecoon function - running for escape ($B3:E65C). Was `run_right`.
pub fn run_for_escape(k: u16, game: &mut (impl Host + ?Sized)) {
    let e = k;
    CritterEdit::of(game, e)
        .actor()
        .x
        .advance(ESCAPE_SPEED.cast_signed());
}

/// Escape etecoon function - stationary ($B3:E670): wait to express gratitude
/// once the critters have escaped. Was `await_rescue`.
pub fn wait_to_express_gratitude(k: u16, game: &mut (impl Host + ?Sized)) {
    if game.event_happened(CRITTERS_ESCAPED) {
        CritterEdit::of(game, k).actor().instruction =
            instruction_list::ETECOON_EXPRESS_GRATITUDE_THEN_ESCAPE;
    }
}

/// Escape etecoon function - running around aimlessly ($B3:E680): run, turn at
/// walls, run for the escape once the critters have escaped, and fall. Was
/// `walk`.
pub fn run_around_aimlessly(k: u16, game: &mut (impl Host + ?Sized)) {
    let e = k;
    let amount = i32::from(Critter::of(game, e).x_velocity() as i16) << 8;
    let blocked = game.move_horizontal(k, amount, Slopes::Ignore);
    if blocked {
        CritterEdit::of(game, e).actor().instruction_delay = 1;
        let left = (-i32::from(Critter::of(game, e).x_velocity())) & SIGN != 0;
        let reversed = (-i32::from(Critter::of(game, e).x_velocity())) as u16;
        *CritterEdit::of(game, e).x_velocity() = reversed;
        let instruction = if left {
            instruction_list::ETECOON_RUNNING_LEFT
        } else {
            instruction_list::ETECOON_RUNNING_RIGHT
        };
        CritterEdit::of(game, e).actor().instruction = instruction;
        if game.event_happened(CRITTERS_ESCAPED) {
            CritterEdit::of(game, e).actor().instruction =
                instruction_list::ETECOON_RUNNING_FOR_ESCAPE;
        }
    }
    game.move_vertical(k, 1 << 16);
}

/// Instruction - go to [[Y]] if the tide is low ($B3:E545, $B3:EAA8), or, with
/// `critters_escaped`, if the critters escaped ($B3:EAB8). Was `branch`.
pub fn go_to_if(
    critters_escaped: bool,
    script: EnemyScript,
    game: &mut (impl Host + ?Sized),
) -> EnemyScript {
    let jump = if critters_escaped {
        game.event_happened(CRITTERS_ESCAPED)
    } else {
        game.session().room.liquid.lava_acid.whole < LOW_TIDE_ACID_Y
    };
    if jump {
        EnemyScript::jump(script.word(0))
    } else {
        script.offset(1)
    }
}

/// Instruction - enemy X position += [[Y]] ($B3:E610). Was `move_scripted`.
pub fn move_x(k: u16, script: EnemyScript, game: &mut (impl Host + ?Sized)) -> EnemyScript {
    let e = k;
    {
        let mut edit = CritterEdit::of(game, e);
        let old = edit.actor().x.whole;
        edit.actor().x.whole = old.wrapping_add(script.word(0));
    }
    script.offset(1)
}

/// Instruction - enemy X position -= 6 or += 6 ($B3:EAC9, $B3:EAD7), as
/// `amount`. Was `step_dachora`.
pub fn shift_dachora(
    amount: u16,
    script: EnemyScript,
    game: &mut (impl Host + ?Sized),
) -> EnemyScript {
    let e = game.current_enemy();
    {
        let mut edit = CritterEdit::of(game, e);
        let old = edit.actor().x.whole;
        edit.actor().x.whole = old.wrapping_add(amount);
    }
    script
}
