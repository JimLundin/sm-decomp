#[macro_use]
#[path = "../../game/player_projectiles/access.rs"]
pub(crate) mod access;
#[path = "../../game/player_projectiles/animation.rs"]
pub(crate) mod animation;
#[path = "../../game/player_projectiles/bomb_creation.rs"]
pub(crate) mod bomb_creation;
#[path = "../../game/player_projectiles/bomb_terrain.rs"]
pub(crate) mod bomb_terrain;
#[path = "../../game/player_projectiles/bombs.rs"]
pub(crate) mod bombs;
#[path = "../../game/player_projectiles/collision.rs"]
pub(crate) mod collision;
#[path = "../../game/player_projectiles/content.rs"]
pub(crate) mod content;
#[path = "../../game/player_projectiles/feedback.rs"]
pub(crate) mod feedback;
#[path = "../../game/player_projectiles/host.rs"]
pub(crate) mod host;
#[path = "../../game/player_projectiles/lifecycle.rs"]
pub(crate) mod lifecycle;
#[path = "../../game/player_projectiles/missiles.rs"]
pub(crate) mod missiles;
#[path = "../../game/player_projectiles/motion.rs"]
pub(crate) mod motion;
#[path = "../../game/player_projectiles/placement.rs"]
pub(crate) mod placement;
#[path = "../../game/player_projectiles/preparation.rs"]
pub(crate) mod preparation;
pub(crate) use preparation::deselect as deselect_owned;
#[path = "../../game/player_projectiles/presentation.rs"]
pub(crate) mod presentation;
#[path = "../../game/player_projectiles/specials.rs"]
pub(crate) mod specials;
use crate::game::session as state;
#[path = "../../game/player_projectiles/weapons.rs"]
pub(crate) mod weapons;
use host::Host;
pub(crate) fn fire(h: &mut impl Host) {
    weapons::update_input(h);
}
pub(crate) fn advance(h: &mut impl Host) {
    motion::advance_all(h);
}
pub(crate) fn draw_explosions(h: &mut impl Host) {
    presentation::draw_explosions(h);
}
pub(crate) fn reset(h: &mut impl Host) {
    motion::reset(h);
}
pub(crate) fn prepare(h: &mut impl Host) {
    preparation::prepare(h, state::PrepareEntry::Normal);
}
pub(crate) fn fire_missile(h: &mut impl Host) {
    missiles::fire(h);
}

pub(crate) fn call_motion(h: &mut impl Host, code: u16, slot: u16) {
    motion::run(h, code, slot);
}
