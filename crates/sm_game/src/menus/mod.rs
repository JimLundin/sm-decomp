//! The pause, map, equipment, file and options menus, over a `Host`.
pub mod canvas;
pub mod controller_input;
pub mod equipment_menu;
pub mod file_menu;
pub mod game_over;
pub mod host;
pub mod map_menu;
pub mod menu_tilemap;
pub mod menu_window;
pub mod options_objects;
pub mod options_settings;
pub mod pause_controls;
pub mod prelude;
pub mod reserve_tanks;
pub mod save_slots;
pub mod state;

/// Whether the current menu state resumes after its forced-blank wait.
pub fn resuming<H: host::Host + ?Sized>(game: &H) -> bool {
    game.screen_primary() != 0
}

/// Set or clear forced blank and wait for NMI ($80:836F, $80:8382). True
/// while the state must return and yield; the next frame's call of the same
/// state resumes here, after the NMI.
pub fn blank_and_wait<H: host::Host + ?Sized>(blank: bool, game: &mut H) -> bool {
    if !resuming(game) {
        game.set_blank(blank);
    }
    let waiting = crate::display_refresh::wait(true, game) != 0;
    game.screen_set_primary(u8::from(waiting));
    waiting
}

/// A menu dispatcher's yield: nonzero while a state waits for the NMI.
pub fn pending_wait<H: host::Host + ?Sized>(game: &H) -> u8 {
    game.screen_primary()
}
