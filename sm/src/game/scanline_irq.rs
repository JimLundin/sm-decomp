//! Native access for the shared scanline interrupt decisions and effects.
use crate::game::{room_entry, scene::irq, session_runtime};
use crate::game_state::Game;
use crate::ram_map::{checked_word as at, word};
use crate::rtl::{BG3SC, CGADSUB, CGWSEL, HTIMEL, NMITIMEN, TM, VTIMEL, WriteReg};
use std::convert::Infallible;

struct NativeControls<'a>(&'a mut Game);
impl irq::Controls for NativeControls<'_> {
    fn read_control(&mut self, field: irq::Control) -> u16 {
        let game = &mut *self.0;
        match field {
            irq::Control::Current => word!(game.cur_irq_handler),
            irq::Control::Pending => word!(game.irqhandler_next_handler),
            irq::Control::Enable => at!(game, 0x84;),
            irq::Control::PreviousCre => word!(game.previous_cre_bitset),
            irq::Control::CurrentCre => word!(game.cre_bitset),
            irq::Control::TransitionFlag => word!(game.door_transition_flag),
        }
    }
    fn write_control(&mut self, field: irq::Store, value: u16) {
        let game = &mut *self.0;
        match field {
            irq::Store::Current => word!(game.cur_irq_handler = value),
            irq::Store::Pending => word!(game.irqhandler_next_handler = value),
            irq::Store::Enable => at!(game, 0x84; = value),
        }
    }
}

/// Constructed only while the caller holds a live, exclusive native backend.
struct NativeIrqHost<'a>(NativeControls<'a>);
impl<'a> NativeIrqHost<'a> {
    /// The native backend must remain live and exclusive for this host's lifetime.
    unsafe fn new(game: &'a mut Game) -> Self {
        Self(NativeControls(game))
    }
}
impl irq::Controls for NativeIrqHost<'_> {
    fn read_control(&mut self, field: irq::Control) -> u16 {
        irq::Controls::read_control(&mut self.0, field)
    }
    fn write_control(&mut self, field: irq::Store, value: u16) {
        irq::Controls::write_control(&mut self.0, field, value);
    }
}
impl irq::Host for NativeIrqHost<'_> {
    type Error = Infallible;

    fn read_mirror(&mut self, field: irq::Mirror) -> u8 {
        self.0.0.ram().read_byte(match field {
            irq::Mirror::Background3Map => 0x5b,
            irq::Mirror::ColorWindow => 0x70,
            irq::Mirror::ColorArithmetic => 0x73,
            irq::Mirror::MainLayers => 0x6a,
            irq::Mirror::InterruptEnable => 0x84,
        })
    }
    fn write_display(&mut self, field: irq::Output, value: u8) -> Result<(), Infallible> {
        let register = match field {
            irq::Output::Background3Map => BG3SC,
            irq::Output::ColorWindow => CGWSEL,
            irq::Output::ColorArithmetic => CGADSUB,
            irq::Output::MainLayers => TM,
            irq::Output::VerticalLow => VTIMEL,
            irq::Output::VerticalHigh => VTIMEL + 1,
            irq::Output::HorizontalLow => HTIMEL,
            irq::Output::HorizontalHigh => HTIMEL + 1,
            irq::Output::InterruptEnable => NMITIMEN,
        };
        // The private host keeps the construction invariant across ordered effects.
        unsafe { WriteReg(register as u16, value, self.0.0) };
        Ok(())
    }
    fn follow_transition(&mut self) -> Result<(), Infallible> {
        // One resume poll, with the same live native capability as the caller.
        unsafe { follow_transition(self.0.0) };
        Ok(())
    }
}

pub(crate) fn disable(game: &mut Game) {
    irq::disable(&mut NativeControls(game));
}

/// Requires a live, exclusively accessed SNES backend.
pub(crate) unsafe fn enable(immediate: bool, game: &mut Game) {
    let mut host = unsafe { NativeIrqHost::new(game) };
    match irq::enable(&mut host, immediate) {
        Ok(()) => {}
        Err(never) => match never {},
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
    let mut host = unsafe { NativeIrqHost::new(game) };
    match irq::advance(&mut host) {
        Ok(()) => {}
        Err(never) => match never {},
    }
}
