#[allow(
    unused_imports,
    reason = "Detached projectile subset omits shot and shot_lane facade consumers."
)]
pub(crate) use sm_game::player_projectiles::access;
pub(crate) use sm_game::player_projectiles::animation;
pub(crate) use sm_game::player_projectiles::host;
pub(crate) use sm_game::player_projectiles::missiles;
pub(crate) use sm_game::player_projectiles::motion;
pub(crate) use sm_game::player_projectiles::preparation;
pub(crate) use sm_game::player_projectiles::presentation;
use crate::game::session as state;
pub(crate) use sm_game::player_projectiles::weapons;
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
