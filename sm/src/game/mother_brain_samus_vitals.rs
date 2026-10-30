//! Samus's health and ammo during the Mother Brain fight: the rainbow beam drains both,
//! the baby Metroid drains health while latched on, then refills it.
#![forbid(unsafe_code)]
use crate::game::music::queue_sfx3_max3;
use crate::game_state::Game;
use crate::ram_map::word;

/// True when `value` is negative as a 16-bit signed word.
fn negative(value: u16) -> bool {
    value & 0x8000 != 0
}

/// Takes one unit of ammo; an emptied weapon deselects itself from the HUD.
fn spend_ammo(count: u16, hud_item: u16, game: &mut Game) -> u16 {
    if negative(count.wrapping_sub(2)) {
        if word!(game.hud_item_index) == hud_item {
            word!(game.hud_item_index = 0);
        }
        word!(game.samus_auto_cancel_hud_item_index = 0);
        0
    } else {
        count.wrapping_sub(1)
    }
}

/// The rainbow beam drains missiles and supers every fourth frame, power bombs every frame.
pub(crate) fn drain_ammo_by_rainbow_beam(game: &mut Game) {
    let missiles = word!(game.samus_missiles);
    if word!(game.random_enemy_counter) & 3 == 0 && missiles != 0 {
        let left = spend_ammo(missiles, 1, game);
        word!(game.samus_missiles = left);
    }
    let supers = word!(game.samus_super_missiles);
    if word!(game.random_enemy_counter) & 3 == 0 && supers != 0 {
        let left = spend_ammo(supers, 2, game);
        word!(game.samus_super_missiles = left);
    }
    let power_bombs = word!(game.samus_power_bombs);
    if power_bombs != 0 {
        let left = spend_ammo(power_bombs, 3, game);
        word!(game.samus_power_bombs = left);
    }
}

/// Every eighth frame above 80 health, the gaining/losing health sound plays.
fn health_change_sound(game: &mut Game) {
    if !negative(word!(game.samus_health).wrapping_sub(81))
        && word!(game.random_enemy_counter) & 7 == 0
    {
        queue_sfx3_max3(0x2d, game);
    }
}

/// Takes `loss` health, stopping at `floor` (signed 16-bit compare).
fn lose_health(loss: u16, floor: u16, game: &mut Game) {
    let health = word!(game.samus_health).wrapping_sub(loss);
    let floored = negative(health.wrapping_sub(floor + 1));
    word!(game.samus_health = if floored { floor } else { health });
}

/// The baby Metroid drains 4 health a frame (2 with Varia), never below 1.
pub(crate) fn drain_health_by_baby_metroid(game: &mut Game) {
    lose_health(4 - 2 * (word!(game.equipped_items) & 1), 1, game);
}

/// The rainbow beam drains 2 health a frame (1 with Varia), down to 0.
pub(crate) fn drain_health_by_rainbow_beam(game: &mut Game) {
    lose_health(2 - (word!(game.equipped_items) & 1), 0, game);
    health_change_sound(game);
}

/// The baby Metroid refills one health a frame; returns false once health is full.
pub(crate) fn heal_by_baby_metroid(game: &mut Game) -> bool {
    let health = word!(game.samus_health);
    let max = word!(game.samus_max_health);
    let healing = (health.wrapping_add(1).wrapping_sub(max) as i16) < 0;
    word!(game.samus_health = if healing { health.wrapping_add(1) } else { max });
    health_change_sound(game);
    healing
}
