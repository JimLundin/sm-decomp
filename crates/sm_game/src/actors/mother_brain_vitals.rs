//! Samus's health and ammo during the Mother Brain fight: the rainbow beam
//! drains both, and the baby Metroid drains her health while latched on, then
//! refills it. Each runs on the fight's lease, over the Session's inventory.
use crate::actors::mother_brain_fight::Lease;
use crate::session::Session;

/// The gaining/losing health sound: 2Dh on sound library 3, at most three queued.
const HEALTH_SOUND: u8 = 0x2d;
const SOUND_LIBRARY_3: usize = 2;
const HEALTH_SOUND_QUEUED: u8 = 3;
/// The missiles, super missiles and power bombs: each one's HUD item index,
/// and whether the beam drains it every frame rather than every fourth.
const AMMUNITION: [(u16, bool); 3] = [(1, false), (2, false), (3, true)];

/// True when `value` is negative as a 16-bit signed word.
fn negative(value: u16) -> bool {
    value & 0x8000 != 0
}

/// The varia suit's bit of the equipped items.
fn varia(session: &Session) -> u16 {
    session.inventory.equipment.active & 1
}

/// Take one unit of the ammo `count`; an emptied weapon deselects itself from
/// the HUD and the auto-cancel item clears.
fn spend_ammo(session: &mut Session, count: u16, hud_item: u16) -> u16 {
    if negative(count.wrapping_sub(2)) {
        let controls = &mut session.projectiles.controls;
        if controls.selected_weapon == hud_item {
            controls.selected_weapon = 0;
        }
        controls.automatic_weapon = 0;
        0
    } else {
        count.wrapping_sub(1)
    }
}

/// The rainbow beam drains missiles and super missiles every fourth frame,
/// and power bombs every frame.
pub fn drain_ammo_by_rainbow_beam(game: &mut impl Lease) {
    let session = game.session_mut();
    for (weapon, (hud_item, every_frame)) in AMMUNITION.into_iter().enumerate() {
        let count = session.inventory.ammunition[weapon].current;
        let this_frame = every_frame || session.actors.controls.phase_counter & 3 == 0;
        if this_frame && count != 0 {
            let left = spend_ammo(session, count, hud_item);
            session.inventory.ammunition[weapon].current = left;
        }
    }
}

/// Every eighth frame above 80 health, the gaining/losing health sound plays.
fn health_change_sound(game: &mut impl Lease) {
    let session = game.session();
    if !negative(session.inventory.health.current.wrapping_sub(81))
        && session.actors.controls.phase_counter & 7 == 0
    {
        game.queue_sound_with_result(SOUND_LIBRARY_3, HEALTH_SOUND, HEALTH_SOUND_QUEUED);
    }
}

/// Take `loss` health, stopping at `floor` (a signed 16-bit compare).
fn lose_health(session: &mut Session, loss: u16, floor: u16) {
    let health = session.inventory.health.current.wrapping_sub(loss);
    let floored = negative(health.wrapping_sub(floor + 1));
    session.inventory.health.current = if floored { floor } else { health };
}

/// The baby Metroid drains 4 health a frame (2 with the varia suit), never below 1.
pub fn drain_health_by_baby_metroid(game: &mut impl Lease) {
    let session = game.session_mut();
    let loss = 4 - 2 * varia(session);
    lose_health(session, loss, 1);
}

/// The rainbow beam drains 2 health a frame (1 with the varia suit), down to 0.
pub fn drain_health_by_rainbow_beam(game: &mut impl Lease) {
    let session = game.session_mut();
    let loss = 2 - varia(session);
    lose_health(session, loss, 0);
    health_change_sound(game);
}

/// The baby Metroid refills one health a frame; false once health is full.
pub fn heal_by_baby_metroid(game: &mut impl Lease) -> bool {
    let health = &mut game.session_mut().inventory.health;
    let healing = negative(health.current.wrapping_add(1).wrapping_sub(health.maximum));
    health.current = if healing {
        health.current.wrapping_add(1)
    } else {
        health.maximum
    };
    health_change_sound(game);
    healing
}
