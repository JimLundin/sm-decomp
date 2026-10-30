//! Ceres steam vents and doors: visibility/contact script steps and escape rumble.
//! Actor words stay RAM-authoritative.
#![forbid(unsafe_code)]
use crate::game::actors::{api, combat};
use crate::game::gameplay_flag_runtime::boss_word_at;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::{word, words as fields};
use crate::rtl::abs16;
use sm_game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use sm_game::enemy_script::EnemyScript;
use sm_game::gameplay_flag_host::BossWordOrder;

pub(crate) const INVISIBLE: u16 = 0x0100;
pub(crate) const TANGIBLE: u16 = 0x0400;

/// Script step: sets then clears actor property bits.
pub(crate) fn change_properties(
    k: u16,
    set: u16,
    clear: u16,
    script: EnemyScript,
    game: &mut Game,
) -> EnemyScript {
    Runtime::new(game).edit_actor(Key::from_legacy(k), Native, |actor| {
        actor.properties = actor.properties & !clear | set;
    });
    script
}

/// Script step: a door records whether it is open (and shows itself when closing).
pub(crate) fn mark_open(k: u16, open: bool, script: EnemyScript, game: &mut Game) -> EnemyScript {
    Runtime::new(game).edit_actor(Key::from_legacy(k), Native, |actor| {
        actor.variables[1] = u16::from(open);
    });
    if open {
        script
    } else {
        change_properties(k, 0, INVISIBLE, script, game)
    }
}

fn branch(taken: bool, script: EnemyScript) -> EnemyScript {
    if taken {
        EnemyScript::jump(script.word(0))
    } else {
        script.offset(1)
    }
}

pub(crate) fn play_open_sound(script: EnemyScript, game: &mut Game) -> EnemyScript {
    crate::game::music::queue_sfx3_max6(0x2c, game);
    script
}

/// Branches while Samus is at least 0x30 pixels away on either axis.
pub(crate) fn branch_unless_samus_near(
    k: u16,
    script: EnemyScript,
    game: &mut Game,
) -> EnemyScript {
    let runtime = Runtime::new(game);
    let actor = runtime.resolve_actor(Key::from_legacy(k), Native);
    let far = abs16(
        actor
            .x
            .whole
            .wrapping_sub(runtime.word(fields::SAMUS_X_POS)),
    ) >= 0x30
        || abs16(
            actor
                .y
                .whole
                .wrapping_sub(runtime.word(fields::SAMUS_Y_POS)),
        ) >= 0x30;
    branch(far, script)
}

/// Branches until the area boss (Ceres Ridley) is marked defeated.
pub(crate) fn branch_unless_boss_defeated(script: EnemyScript, game: &mut Game) -> EnemyScript {
    let area = word!(game.AREA_INDEX);
    branch(
        boss_word_at(area, BossWordOrder::Native, game) & 1 == 0,
        script,
    )
}

/// Branches until the Ceres escape has started.
pub(crate) fn branch_unless_escape(script: EnemyScript, game: &mut Game) -> EnemyScript {
    branch(word!(game.CERES_STATUS) == 0, script)
}

/// Steam contact keeps the vent at full health before the shared reaction.
pub(crate) fn steam_touch(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(runtime.word(fields::CUR_ENEMY_INDEX));
    runtime.edit_actor(key, Native, |actor| actor.health = 0x7fff);
    combat::contact_reaction(&mut runtime, api::DeathHandling::Finalize);
}
