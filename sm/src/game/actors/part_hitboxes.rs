//! Boss part hitboxes against Samus: a count word at bank $A9 followed by
//! eight-byte boxes of left, top, right and bottom extents.
#![forbid(unsafe_code)]
use crate::game::constants::bank_a9::ADDR_K_DEAD_TORIZO_HITBOX_D77C;
use crate::game::session_runtime::native_record;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use sm_game::actors::layouts::EnemyLayout40 as DeadTorizo;
use sm_game::actors::layouts::EnemyLayout72 as MotherBrain;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Contact {
    None,
    Pushed,
    /// Pushed, and Samus is right of the body; the caller hurts her last.
    Hurt,
}

/// Each extent is chosen by Samus's side of the centre (`x`, `y`); the first
/// overlapping box sets the push words, clamped to at least 4, and stops.
fn push_out(boxes: u16, x: u16, y: u16, game: &mut Game) -> bool {
    let table = native_record(0xa9, boxes, game);
    let abs = |value: u16| i32::from((value as i16).unsigned_abs());
    for part in 0..usize::from(table.word(0)) {
        let at = 2 + 8 * part;
        let samus_y = word!(game.SAMUS_Y_POS);
        let (distance, extent) = if (samus_y as i16).wrapping_sub(y as i16) >= 0 {
            (samus_y.wrapping_sub(y), table.word(at + 6))
        } else {
            (y.wrapping_sub(samus_y), table.word(at + 2))
        };
        if ((i32::from(word!(game.SAMUS_Y_RADIUS)) + abs(extent) - i32::from(distance)) as i16) < 0
        {
            continue;
        }
        let samus_x = word!(game.SAMUS_X_POS);
        let (distance, extent) = if (samus_x as i16).wrapping_sub(x as i16) >= 0 {
            (samus_x.wrapping_sub(x), table.word(at + 4))
        } else {
            (x.wrapping_sub(samus_x), table.word(at))
        };
        let mut overlap =
            (i32::from(word!(game.SAMUS_X_RADIUS)) + abs(extent) - i32::from(distance)) as i16;
        if overlap < 0 {
            continue;
        }
        if (i32::from(overlap) - 4) & 0x8000 != 0 {
            overlap = 4;
        }
        word!(game.EXTRA_SAMUS_X_DISPLACEMENT = overlap as u16);
        word!(game.EXTRA_SAMUS_Y_DISPLACEMENT = 4);
        word! { game, 0; EXTRA_SAMUS_X_SUBDISPLACEMENT, EXTRA_SAMUS_Y_SUBDISPLACEMENT }
        return true;
    }
    false
}

/// Mother Brain's parts also knock Samus back; right of the body she is hurt.
pub(crate) fn samus_contact(boxes: u16, x: u16, y: u16, game: &mut Game) -> Contact {
    if !push_out(boxes, x, y, game) {
        return Contact::None;
    }
    word! { game; SAMUS_INVINCIBILITY_TIMER = 96; SAMUS_KNOCKBACK_TIMER = 5; KNOCKBACK_X_DIR = 1; }
    if (i32::from(word!(game.SAMUS_Y_POS)) - 192) & 0x8000 != 0 {
        word!(game.SAMUS_Y_DIR = 2);
    }
    let body = i32::from(slot!(game, MotherBrain[0].base.x_pos)) + 24;
    if ((body - i32::from(word!(game.SAMUS_X_POS))) as i16) < 0 {
        Contact::Hurt
    } else {
        Contact::Pushed
    }
}

/// The dead Torizo's fixed box list pushes Samus out without knockback.
pub(crate) fn dead_torizo_contact(game: &mut Game) -> bool {
    let (x, y) = (
        slot!(game, DeadTorizo[0].base.x_pos),
        slot!(game, DeadTorizo[0].base.y_pos),
    );
    push_out(ADDR_K_DEAD_TORIZO_HITBOX_D77C as u16, x, y, game)
}
