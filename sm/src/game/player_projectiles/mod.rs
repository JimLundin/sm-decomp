//! Complete owned player-projectile subsystem.
mod state;
#[path = "../ownership_transport.rs"]
mod transport;
#[macro_use]
mod encoded;
pub(crate) mod checks;
mod animation;
mod lifecycle;
mod motion;
mod weapons;
mod missiles;
mod placement;
mod bombs;
mod bomb_creation;
mod specials;
mod collision;
mod bomb_terrain;
mod presentation;
mod feedback;
mod native_content;
mod preparation;
use encoded::Transaction;
#[derive(Default)]
struct Context {
    slot: usize,
    image: Option<crate::rom::RomImage>,
    frozen: bool,
}
impl Context {
    fn record<'g>(
        owner: Transaction<'g>,
        bank: u8,
        cursor: u16,
    ) -> (Transaction<'g>, crate::rom::RomRecord) {
        if cursor & 0x8000 == 0 {
            // The real low-window diagnostic observes already-published state.
            return owner.foreign(|game| unsafe { crate::rtl::rom_record(bank, cursor, game) });
        }
        let image = owner.rom();
        let base = crate::rtl::lorom_offset((u32::from(bank) << 16) | u32::from(cursor)) as usize;
        (owner, crate::rom::RomRecord { image, base })
    }
}

// Game-facing integration entries. Domain state and shared cores never expose
// Game; these complete transactions consume ownership before returning it.
use crate::game_state::Game;
fn transaction<'g>(game: &'g mut Game, action: impl FnOnce(Transaction<'g>) -> Transaction<'g>) {
    action(Transaction::acquire(game)).release();
}
pub(crate) fn prepare(game: &mut Game) {
    let address = 0x900000
        | u32::from(u16::from_le_bytes([
            game.g_ram[crate::ram_map::words::frame_handler_alfa],
            game.g_ram[crate::ram_map::words::frame_handler_alfa + 1],
        ]));
    prepare_entry(address, game);
}
pub(crate) fn prepare_entry(address: u32, game: &mut Game) {
    use crate::game::player_preparation::Entry;
    let entry = match address {
        0x90e695 => Entry::Normal,
        0x90e6c9 => Entry::Demo,
        0x90e713 => Entry::ProjectilesOnly,
        0x90e8cd => Entry::Inactive,
        _ => {
            unsafe {
                crate::rtl::Unreachable(game);
            }
            return;
        }
    };
    transaction(game, |owner| preparation::prepare(entry, owner));
}
pub(crate) fn update(game: &mut Game) {
    transaction(game, motion::advance_all);
}
pub(crate) fn reset(game: &mut Game) {
    transaction(game, motion::reset);
}
pub(crate) fn clear(slot: u16, game: &mut Game) {
    transaction(game, |o| motion::clear(slot, o));
}
pub(crate) fn impact(slot: u16, game: &mut Game) {
    transaction(game, |o| lifecycle::impact(slot, o));
}
pub(crate) fn retire(slot: u16, game: &mut Game) {
    transaction(game, |o| lifecycle::retire(slot, o));
}
pub(crate) fn reflect(slot: u16, game: &mut Game) {
    transaction(game, |o| motion::reflect(slot, o));
}
pub(crate) fn draw_projectiles(game: &mut Game) {
    transaction(game, presentation::draw_projectiles);
}
pub(crate) fn draw_explosions(game: &mut Game) {
    transaction(game, presentation::draw_explosions);
}
pub(crate) fn update_trails(game: &mut Game) {
    transaction(game, presentation::update_trails);
}
pub(crate) fn flare(game: &mut Game) {
    transaction(game, feedback::flare);
}
pub(crate) fn grapple_flare(game: &mut Game) {
    transaction(game, feedback::grapple_flare);
}
pub(crate) fn movement_sound(game: &mut Game) {
    transaction(game, feedback::movement_sound);
}
pub(crate) fn restore_sound(game: &mut Game) {
    transaction(game, feedback::restore_sound);
}
pub(crate) fn charge_palette(game: &mut Game) -> u8 {
    let (owner, result) = feedback::palette(Transaction::acquire(game));
    owner.release();
    result
}
pub(crate) fn deselect(game: &mut Game) -> u8 {
    transaction(game, preparation::deselect);
    0
}
pub(crate) fn initialize(
    slot: u16,
    kind: crate::game::projectile_initialization::Initialization,
    game: &mut Game,
) {
    transaction(game, |o| animation::initialize(slot, kind, o));
}
pub(crate) fn call_motion(address: u32, slot: u16, game: &mut Game) {
    if address >> 16 != 0x90 {
        unsafe {
            crate::rtl::Unreachable(game);
        }
        return;
    }
    transaction(game, |o| motion::run(address as u16, slot, o));
}
pub(crate) fn clear_animation(game: &mut Game) {
    transaction(game, |mut owner| {
        weapons::clear_animation(&mut owner);
        owner
    });
}
