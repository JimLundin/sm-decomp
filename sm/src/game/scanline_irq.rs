//! Native access for the shared scanline interrupt decisions and effects.
use crate::game_state::Game;
use crate::ram_map::{checked_word as at, word};
use crate::rtl::{BG3SC, CGADSUB, CGWSEL, HTIMEL, NMITIMEN, TM, VTIMEL, write_reg};
use std::convert::Infallible;
use {
    crate::game::{room_entry, session_runtime},
    sm_game::scene::irq,
};

struct NativeControls<'a>(&'a mut Game);
impl irq::Controls for NativeControls<'_> {
    fn read_control(&mut self, field: irq::Control) -> u16 {
        let game = &mut *self.0;
        match field {
            irq::Control::Current => word!(game.CUR_IRQ_HANDLER),
            irq::Control::Pending => word!(game.IRQHANDLER_NEXT_HANDLER),
            irq::Control::Enable => at!(game, 0x84;),
            irq::Control::PreviousCre => word!(game.PREVIOUS_CRE_BITSET),
            irq::Control::CurrentCre => word!(game.CRE_BITSET),
            irq::Control::TransitionFlag => word!(game.DOOR_TRANSITION_FLAG),
        }
    }
    fn write_control(&mut self, field: irq::Store, value: u16) {
        let game = &mut *self.0;
        match field {
            irq::Store::Current => word!(game.CUR_IRQ_HANDLER = value),
            irq::Store::Pending => word!(game.IRQHANDLER_NEXT_HANDLER = value),
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
        #[cfg(sm_verify_dll)]
        table_check::record(field, value);
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
        unsafe { write_reg(register as u16, value, self.0.0) };
        Ok(())
    }
    fn follow_transition(&mut self) -> Result<(), Infallible> {
        #[cfg(sm_verify_dll)]
        table_check::resumed();
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
    #[cfg(sm_verify_dll)]
    table_check::begin_pass(unsafe { (*game.g_snes).vTimer });
    let mut host = unsafe { NativeIrqHost::new(game) };
    match irq::advance(&mut host) {
        Ok(()) => {}
        Err(never) => match never {},
    }
}

/// Scanline interrupts as per-frame tables (native renderer step 4). With
/// `SM_IRQ_TABLE` set, each native frame's interrupts are planned as it begins
/// settling with `scene::irq::frame_table`, from a copy of the interrupt
/// controls, mirrors, vertical timer and its enable, and compared with the
/// interrupts the beam then runs. `SM_IRQ_TABLE_LOG` receives each frame that
/// differs (frame, planned and actual interrupt lines, with `r` marking a
/// door transition resume poll).
#[cfg(sm_verify_dll)]
pub(crate) mod table_check {
    use super::{Game, irq};
    use crate::ram_map::{checked_word as at, word};
    use std::cell::RefCell;
    use std::io::Write as _;
    use std::sync::OnceLock;

    #[derive(Default)]
    struct Frame {
        number: u32,
        planned: Option<Vec<irq::IrqPass>>,
        actual: Vec<irq::IrqPass>,
    }

    thread_local! {
        static FRAME: RefCell<Frame> = RefCell::new(Frame::default());
    }

    fn enabled() -> bool {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        *ENABLED.get_or_init(|| std::env::var_os("SM_IRQ_TABLE").is_some())
    }

    /// A copy of the interrupt controls and display mirrors as the frame settles.
    struct Scratch {
        controls: [u16; 6],
        mirrors: [u8; 5],
    }

    fn control_index(field: irq::Control) -> usize {
        match field {
            irq::Control::Current => 0,
            irq::Control::Pending => 1,
            irq::Control::Enable => 2,
            irq::Control::PreviousCre => 3,
            irq::Control::CurrentCre => 4,
            irq::Control::TransitionFlag => 5,
        }
    }

    impl irq::Controls for Scratch {
        fn read_control(&mut self, field: irq::Control) -> u16 {
            self.controls[control_index(field)]
        }
        fn write_control(&mut self, field: irq::Store, value: u16) {
            let index = match field {
                irq::Store::Current => 0,
                irq::Store::Pending => 1,
                irq::Store::Enable => 2,
            };
            self.controls[index] = value;
        }
    }

    impl irq::Host for Scratch {
        type Error = std::convert::Infallible;
        fn read_mirror(&mut self, field: irq::Mirror) -> u8 {
            self.mirrors[match field {
                irq::Mirror::Background3Map => 0,
                irq::Mirror::ColorWindow => 1,
                irq::Mirror::ColorArithmetic => 2,
                irq::Mirror::MainLayers => 3,
                irq::Mirror::InterruptEnable => 4,
            }]
        }
        fn write_display(&mut self, _: irq::Output, _: u8) -> Result<(), Self::Error> {
            unreachable!("the frame table records its writes itself")
        }
        fn follow_transition(&mut self) -> Result<(), Self::Error> {
            unreachable!("the frame table records resumes itself")
        }
    }

    fn lines(passes: &[irq::IrqPass]) -> String {
        passes
            .iter()
            .map(|pass| format!("{}{}", pass.line, if pass.resumed { "r" } else { "" }))
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub(crate) fn begin_frame(game: &mut Game) {
        if !enabled() {
            return;
        }
        let mut scratch = Scratch {
            controls: [
                word!(game.CUR_IRQ_HANDLER),
                word!(game.IRQHANDLER_NEXT_HANDLER),
                at!(game, 0x84;),
                word!(game.PREVIOUS_CRE_BITSET),
                word!(game.CRE_BITSET),
                word!(game.DOOR_TRANSITION_FLAG),
            ],
            mirrors: [0x5b, 0x70, 0x73, 0x6a, 0x84].map(|address| game.ram().read_byte(address)),
        };
        let (timer, enable) = unsafe { ((*game.g_snes).vTimer, (*game.g_snes).vIrqEnabled) };
        let planned = match irq::frame_table(&mut scratch, timer, enable) {
            Ok(planned) => planned,
            Err(never) => match never {},
        };
        FRAME.with_borrow_mut(|frame| {
            frame.number += 1;
            frame.planned = Some(planned);
            frame.actual.clear();
        });
    }

    pub(crate) fn begin_pass(line: u16) {
        if enabled() {
            FRAME.with_borrow_mut(|frame| {
                frame.actual.push(irq::IrqPass {
                    line,
                    writes: Vec::new(),
                    resumed: false,
                })
            });
        }
    }

    pub(crate) fn record(field: irq::Output, value: u8) {
        if enabled() {
            FRAME.with_borrow_mut(|frame| {
                if let Some(pass) = frame.actual.last_mut() {
                    pass.writes.push((field, value));
                }
            });
        }
    }

    pub(crate) fn resumed() {
        if enabled() {
            FRAME.with_borrow_mut(|frame| {
                if let Some(pass) = frame.actual.last_mut() {
                    pass.resumed = true;
                }
            });
        }
    }

    pub(crate) fn end_frame() {
        if !enabled() {
            return;
        }
        FRAME.with_borrow_mut(|frame| {
            let Some(planned) = frame.planned.take() else {
                return;
            };
            if planned == frame.actual {
                return;
            }
            let row = format!(
                "{},planned {},actual {}",
                frame.number,
                lines(&planned),
                lines(&frame.actual)
            );
            if let Some(path) = std::env::var_os("SM_IRQ_TABLE_LOG") {
                let mut log = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .unwrap();
                writeln!(log, "{row}").unwrap();
            }
        });
    }
}
