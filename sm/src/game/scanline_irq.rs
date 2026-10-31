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
        #[cfg(sm_verify_dll)]
        table_check::record(field, value);
        irq::Display::write_display(&mut self.0, field, value)
    }
}
impl irq::Host for NativeIrqHost<'_> {
    fn follow_transition(&mut self) -> Result<(), Infallible> {
        #[cfg(sm_verify_dll)]
        table_check::resumed();
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
    #[cfg(sm_verify_dll)]
    table_check::begin_pass(unsafe { (*game.g_snes).vTimer });
    let Ok(()) = irq::advance(&mut NativeIrqHost(sm_game::display_refresh::ScanlineIrq(
        game,
    )));
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
    use sm_game::display_refresh::ScanlineIrq;
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

    impl irq::Display for Scratch {
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
    }

    impl irq::Host for Scratch {
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
        let mut live = ScanlineIrq(&mut *game);
        let mut scratch = Scratch {
            controls: [
                irq::Control::Current,
                irq::Control::Pending,
                irq::Control::Enable,
                irq::Control::PreviousCre,
                irq::Control::CurrentCre,
                irq::Control::TransitionFlag,
            ]
            .map(|field| irq::Controls::read_control(&mut live, field)),
            mirrors: [
                irq::Mirror::Background3Map,
                irq::Mirror::ColorWindow,
                irq::Mirror::ColorArithmetic,
                irq::Mirror::MainLayers,
                irq::Mirror::InterruptEnable,
            ]
            .map(|field| irq::Display::read_mirror(&mut live, field)),
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
