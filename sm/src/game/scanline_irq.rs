//! Native access for the shared scanline interrupt's per-line handler.
use crate::game_state::Game;
use std::convert::Infallible;
use {
    crate::game::{room_entry, session_runtime},
    sm_game::scene::irq,
};

/// The projected controls and registers, plus the door resume a handler polls.
struct NativeIrqHost<'a>(sm_game::display_refresh::ScanlineIrq<'a, Game>);
impl irq::Controls for NativeIrqHost<'_> {
    fn read_control(&mut self, field: irq::Control) -> u16 {
        irq::Controls::read_control(&mut self.0, field)
    }
    fn write_control(&mut self, field: irq::Store, value: u16) {
        irq::Controls::write_control(&mut self.0, field, value);
    }
}
impl irq::Display for NativeIrqHost<'_> {
    type Error = Infallible;

    fn read_mirror(&mut self, field: irq::Mirror) -> u8 {
        irq::Display::read_mirror(&mut self.0, field)
    }
    fn write_display(&mut self, field: irq::Output, value: u8) -> Result<(), Infallible> {
        irq::Display::write_display(&mut self.0, field, value)
    }
}
impl irq::Host for NativeIrqHost<'_> {
    fn follow_transition(&mut self) -> Result<(), Infallible> {
        // One resume poll, with the same live native capability as the caller.
        unsafe { follow_transition(self.0.0) };
        Ok(())
    }
}

/// Resuming a door may update the display; its backend must remain live and exclusive.
pub(crate) unsafe fn follow_transition(game: &mut Game) {
    room_entry::protocol::resume(
        &mut session_runtime::Runtime::new(game),
        room_entry::EntrySignal::ScrollIrq,
    );
}

/// Requires a live, exclusively accessed SNES backend for the entire interrupt.
pub(crate) unsafe fn advance(game: &mut Game) {
    let Ok(()) = irq::advance(&mut NativeIrqHost(sm_game::display_refresh::ScanlineIrq(
        game,
    )));
}
