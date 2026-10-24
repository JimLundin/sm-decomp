//! Player-projectile lifecycle, producers, trails and charge feedback.
#[macro_use]
pub(crate) mod access;
mod animation;
mod bomb_creation;
mod bomb_terrain;
mod bombs;
mod collision;
pub(crate) mod content;
use crate::game::session_encoding as encoding;
pub(crate) mod feedback;
pub(crate) mod host;
mod lifecycle;
mod missiles;
pub(crate) mod motion;
mod placement;
mod preparation;
mod presentation;

mod specials;
use crate::game::session as state;
mod weapons;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use host::Host;
#[path = "../../verification/player_projectiles.rs"]
pub(crate) mod checks;

pub(crate) fn prepare(game: &mut Game) {
    let address = 0x900000
        | u32::from(encoding::read(
            &game.g_ram,
            crate::ram_map::words::frame_handler_alfa,
        ));
    prepare_entry(address, game);
}
pub(crate) fn prepare_entry(address: u32, game: &mut Game) {
    use state::PrepareEntry;
    let entry = match address {
        0x90e695 => PrepareEntry::Normal,
        0x90e6c9 => PrepareEntry::Demo,
        0x90e713 => PrepareEntry::ProjectilesOnly,
        0x90e8cd => PrepareEntry::Inactive,
        _ => {
            Runtime::new(game).unknown();
            return;
        }
    };
    preparation::prepare(&mut Runtime::new(game), entry);
}
pub(crate) fn update(game: &mut Game) {
    motion::advance_all(&mut Runtime::new(game));
}
pub(crate) fn reset(game: &mut Game) {
    motion::reset(&mut Runtime::new(game));
}
pub(crate) fn clear(slot: u16, game: &mut Game) {
    motion::clear(&mut Runtime::new(game), slot);
}
pub(crate) fn impact(slot: u16, game: &mut Game) {
    lifecycle::impact(&mut Runtime::new(game), slot);
}
pub(crate) fn retire(slot: u16, game: &mut Game) {
    lifecycle::retire(&mut Runtime::new(game), slot);
}
pub(crate) fn reflect(slot: u16, game: &mut Game) {
    motion::reflect(&mut Runtime::new(game), slot);
}
pub(crate) fn draw_projectiles(game: &mut Game) {
    presentation::draw_projectiles(&mut Runtime::new(game));
}
pub(crate) fn draw_explosions(game: &mut Game) {
    presentation::draw_explosions(&mut Runtime::new(game));
}
pub(crate) fn update_trails(game: &mut Game) {
    presentation::update_trails(&mut Runtime::new(game));
}
pub(crate) fn flare(game: &mut Game) {
    feedback::flare(&mut Runtime::new(game));
}
pub(crate) fn grapple_flare(game: &mut Game) {
    feedback::grapple_flare(&mut Runtime::new(game));
}
pub(crate) fn draw_flare(slot: u16, game: &mut Game) {
    feedback::draw_flare(&mut Runtime::new(game), slot);
}
pub(crate) fn movement_sound(game: &mut Game) {
    feedback::movement_sound(&mut Runtime::new(game));
}
pub(crate) fn restore_sound(game: &mut Game) {
    feedback::restore_sound(&mut Runtime::new(game));
}
pub(crate) fn charge_palette(game: &mut Game) -> u8 {
    feedback::palette(&mut Runtime::new(game))
}
pub(crate) fn deselect(game: &mut Game) -> u8 {
    preparation::deselect(&mut Runtime::new(game));
    0
}
pub(crate) fn initialize(
    slot: u16,
    kind: crate::game::projectile_initialization::Initialization,
    game: &mut Game,
) {
    animation::initialize(&mut Runtime::new(game), slot, kind);
}
pub(crate) fn call_motion(address: u32, slot: u16, game: &mut Game) {
    let mut h = Runtime::new(game);
    if address >> 16 != 0x90 {
        h.unknown();
        return;
    }
    motion::run(&mut h, address as u16, slot);
}
pub(crate) fn clear_animation(game: &mut Game) {
    weapons::clear_animation(&mut Runtime::new(game));
}
