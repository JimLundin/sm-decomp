//! Native access for the shared scanline interrupt's per-line handler, on Game
//! and on the frame's lease.
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
        let value = irq::Controls::read_control(&mut self.0, field);
        #[cfg(sm_verify_dll)]
        publish_check::read_control(field, value);
        value
    }
    fn write_control(&mut self, field: irq::Store, value: u16) {
        #[cfg(sm_verify_dll)]
        table_check::store(field, value);
        irq::Controls::write_control(&mut self.0, field, value);
        #[cfg(sm_verify_dll)]
        publish_check::store(field, value);
    }
}
impl irq::Display for NativeIrqHost<'_> {
    type Error = Infallible;

    fn read_mirror(&mut self, field: irq::Mirror) -> u8 {
        let value = irq::Display::read_mirror(&mut self.0, field);
        #[cfg(sm_verify_dll)]
        publish_check::read_mirror(field, value);
        value
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
        #[cfg(sm_verify_dll)]
        publish_check::resumed(self.0.0);
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

/// One interrupt pass on Game: settlement after an emulated frame (the
/// oracle's) or a frame stepped by hand. The engine's frames settle on their
/// lease (`advance_on_lease`).
/// Requires a live, exclusively accessed SNES backend for the entire interrupt.
pub(crate) unsafe fn advance(game: &mut Game) {
    #[cfg(sm_verify_dll)]
    table_check::begin_pass(unsafe { (*game.g_snes).vTimer });
    #[cfg(sm_verify_dll)]
    publish_check::begin_pass(game, unsafe { (*game.g_snes).vTimer });
    let Ok(()) = irq::advance(&mut NativeIrqHost(sm_game::display_refresh::ScanlineIrq(
        game,
    )));
    #[cfg(sm_verify_dll)]
    publish_check::end_pass(game);
}

/// The scanline interrupt on the frame's lease: the controls and mirrors are
/// the lease's, the outputs device writes, and a door resume poll runs on the
/// lease itself, with the observations refreshed as the fresh lease Game's
/// poll opened left them. `stored` records a control store or a poll, after
/// which the lease publishes before the beam's next event reads work RAM.
struct LeaseIrqHost<'a, 'g> {
    irq: sm_game::display_refresh::ScanlineIrq<'a, session_runtime::Runtime<'g>>,
    stored: bool,
}
impl irq::Controls for LeaseIrqHost<'_, '_> {
    fn read_control(&mut self, field: irq::Control) -> u16 {
        irq::Controls::read_control(&mut self.irq, field)
    }
    fn write_control(&mut self, field: irq::Store, value: u16) {
        #[cfg(sm_verify_dll)]
        table_check::store(field, value);
        irq::Controls::write_control(&mut self.irq, field, value);
        self.stored = true;
    }
}
impl irq::Display for LeaseIrqHost<'_, '_> {
    type Error = Infallible;

    fn read_mirror(&mut self, field: irq::Mirror) -> u8 {
        irq::Display::read_mirror(&mut self.irq, field)
    }
    fn write_display(&mut self, field: irq::Output, value: u8) -> Result<(), Infallible> {
        #[cfg(sm_verify_dll)]
        table_check::record(field, value);
        irq::Display::write_display(&mut self.irq, field, value)
    }
}
impl irq::Host for LeaseIrqHost<'_, '_> {
    fn follow_transition(&mut self) -> Result<(), Infallible> {
        #[cfg(sm_verify_dll)]
        table_check::resumed();
        self.irq.0.owned_player_call(|lease| {
            room_entry::protocol::resume(lease, room_entry::EntrySignal::ScrollIrq);
        });
        self.stored = true;
        Ok(())
    }
}

/// One interrupt pass on the frame's lease, at the line the beam reached. A
/// pass that stored publishes the lease, so HDMA on the following lines reads
/// work RAM as Game's stores and its poll's lease release left it.
pub(crate) fn advance_on_lease(lease: &mut session_runtime::Runtime<'_>, line: u16) {
    #[cfg(sm_verify_dll)]
    table_check::begin_pass(line);
    #[cfg(not(sm_verify_dll))]
    let _ = line;
    let mut host = LeaseIrqHost {
        irq: sm_game::display_refresh::ScanlineIrq(lease),
        stored: false,
    };
    let Ok(()) = irq::advance(&mut host);
    if host.stored {
        lease.publish();
    }
}

/// The frame's settlement on its lease (`PassLease::settle_frame`): the lease
/// publishes first, so HDMA's first table read finds the bytes the lease's
/// deferred export would have written there, then the beam runs as on Game.
pub(crate) fn settle_on_lease(lease: &mut session_runtime::Runtime<'_>) {
    lease.publish();
    #[cfg(sm_verify_dll)]
    {
        use sm_game::scene::settle::SettlementHost as _;
        let (timer, enable) = (lease.vertical_timer(), lease.vertical_irq_enabled());
        table_check::begin_frame_on(lease, timer, enable);
    }
    match sm_game::scene::settle::settle(lease) {
        Ok(()) => (),
        Err(never) => match never {},
    }
    #[cfg(sm_verify_dll)]
    table_check::end_frame();
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

    /// The HDMA table check plans with the interrupts' control stores too.
    fn planning() -> bool {
        static PLANNING: OnceLock<bool> = OnceLock::new();
        *PLANNING.get_or_init(|| enabled() || std::env::var_os("SM_HDMA_TABLE").is_some())
    }

    /// The planned interrupts' lines and control stores for the frame that is
    /// settling, if it has a plan.
    pub(crate) fn planned_stores() -> Vec<(u16, Vec<(irq::Store, u16)>)> {
        FRAME.with_borrow(|frame| {
            frame
                .planned
                .iter()
                .flatten()
                .map(|pass| (pass.line, pass.stores.clone()))
                .collect()
        })
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
        let (timer, enable) = unsafe { ((*game.g_snes).vTimer, (*game.g_snes).vIrqEnabled) };
        begin_frame_on(game, timer, enable);
    }

    /// `begin_frame` from `host`'s controls and mirrors (Game's, or the frame
    /// lease's), with the device's vertical timer and its enable.
    pub(crate) fn begin_frame_on<
        H: sm_game::display::preparation::Access
            + sm_game::ram::CompatibilityRam
            + sm_game::display::video::Video
            + ?Sized,
    >(
        host: &mut H,
        timer: u16,
        enable: bool,
    ) {
        if !planning() {
            return;
        }
        let mut live = ScanlineIrq(host);
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
        if planning() {
            FRAME.with_borrow_mut(|frame| {
                frame.actual.push(irq::IrqPass {
                    line,
                    writes: Vec::new(),
                    stores: Vec::new(),
                    resumed: false,
                })
            });
        }
    }

    pub(crate) fn record(field: irq::Output, value: u8) {
        if planning() {
            FRAME.with_borrow_mut(|frame| {
                if let Some(pass) = frame.actual.last_mut() {
                    pass.writes.push((field, value));
                }
            });
        }
    }

    pub(crate) fn store(field: irq::Store, value: u16) {
        if planning() {
            FRAME.with_borrow_mut(|frame| {
                if let Some(pass) = frame.actual.last_mut() {
                    pass.stores.push((field, value));
                }
            });
        }
    }

    pub(crate) fn resumed() {
        if planning() {
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

/// The frame lease's IRQ publish, checked against today's Game path (stage 10,
/// step 1). With `SM_IRQ_PUBLISH_CHECK` set, each interrupt pass is replayed on
/// a copy of the Session a lease opened at the pass would hold
/// (`session_runtime::preview_lease_session`): its control stores ($A7, $AB,
/// $84/$85) go in as the lease's `ScanlineIrq` would make them (the owned
/// display state, a projected byte, or a RAM byte), and its control and mirror
/// reads are compared with the lease's. After a pass that stored or polled a
/// door resume, RAM as the lease would leave it (the pass's RAM, the lease's
/// direct RAM stores, then `encode_spans` of the copy: a publish) is compared
/// with RAM as Game's path left it, on the stored lanes and on every other
/// byte the copy did not already differ on when the pass began. A door resume
/// poll still opens its own lease; the copy is taken again after it.
///
/// Report-only: nothing is written to the game, no lease opens, no epoch
/// moves and no deferred export is written or served. `SM_IRQ_PUBLISH_LOG`
/// receives each pass that differs (frame, line, source, the differing lanes'
/// addresses and counts; no byte values). `SM_IRQ_PUBLISH_COUNTS.<pid>` holds
/// each verifier process's running totals, so a run that checked nothing
/// shows it.
///
/// The check hooks Game's interrupt pass (`advance`). Once the engine's frames
/// settle on their lease (`settle_on_lease`), only the oracle's frames and the
/// frames verification steps by hand pass there, so its engine numbers are the
/// ones a run of the commit before that move reports.
#[cfg(sm_verify_dll)]
pub(crate) mod publish_check {
    use super::{Game, irq};
    use crate::game::session_runtime::{LeaseSource, preview_lease_session};
    use crate::game_state::{frame_field_address, frame_word_address};
    use sm_game::display::preparation::{self as preparation, Field, Word};
    use sm_game::session::Session;
    use sm_game::session_encoding as encoding;
    use std::cell::RefCell;
    use std::io::Write as _;
    use std::sync::OnceLock;

    /// A control's lanes, as the lease's display access names them.
    #[derive(Clone, Copy)]
    enum Lane {
        History(Word),
        Controls(Field, Field),
    }

    impl Lane {
        fn of_store(field: irq::Store) -> Self {
            match field {
                irq::Store::Current => Self::History(Word::CurrentInterrupt),
                irq::Store::Pending => Self::History(Word::PendingInterrupt),
                irq::Store::Enable => Self::Controls(Field::NmiEnable, Field::HdmaEnable),
            }
        }
        fn of_control(field: irq::Control) -> Option<Self> {
            match field {
                irq::Control::Current => Some(Self::History(Word::CurrentInterrupt)),
                irq::Control::Pending => Some(Self::History(Word::PendingInterrupt)),
                irq::Control::Enable => Some(Self::Controls(Field::NmiEnable, Field::HdmaEnable)),
                // The room's layer words and the door flag are Session fields no
                // interrupt stores; the lease reads them as Game's projection does.
                irq::Control::PreviousCre
                | irq::Control::CurrentCre
                | irq::Control::TransitionFlag => None,
            }
        }
        fn addresses(self) -> [usize; 2] {
            match self {
                Self::History(word) => {
                    let at = frame_word_address(word);
                    [at, at + 1]
                }
                Self::Controls(low, high) => [frame_field_address(low), frame_field_address(high)],
            }
        }
    }

    fn mirror_field(field: irq::Mirror) -> Field {
        match field {
            irq::Mirror::Background3Map => Field::Background3Map,
            irq::Mirror::ColorWindow => Field::PreviousColorSelection,
            irq::Mirror::ColorArithmetic => Field::PreviousColorMath,
            irq::Mirror::MainLayers => Field::PreviousMainLayers,
            irq::Mirror::InterruptEnable => Field::NmiEnable,
        }
    }

    /// The lease a pass would run on, modelled on a copy.
    struct Lease {
        session: Session,
        source: LeaseSource,
        /// RAM at the pass's start (or after its door resume poll), with the
        /// lease's direct RAM stores.
        ram: Vec<u8>,
        /// The bytes where a publish of the copy already differed from RAM.
        stale: Vec<bool>,
    }

    impl Lease {
        fn open(game: &Game) -> Self {
            let ram = game.ram().peek_image();
            let (session, source) = preview_lease_session(game, &ram);
            let published = publish(&session, &ram);
            let stale = ram.iter().zip(&published).map(|(a, b)| a != b).collect();
            Self {
                session,
                source,
                ram,
                stale,
            }
        }
        fn owned(&self) -> bool {
            self.session.presentation.is_owned()
        }
        /// `Runtime::byte`: the Session's projection, or RAM.
        fn byte(&self, at: usize) -> u8 {
            encoding::projected_byte(&self.session, at).unwrap_or(self.ram[at])
        }
        /// `Runtime::set_byte` for a lane outside the scroll bytes.
        fn set_byte(&mut self, at: usize, value: u8) {
            if !encoding::write_projected_byte(&mut self.session, at, value) {
                self.ram[at] = value;
            }
        }
        fn display(&self) -> &sm_game::scene::state::DisplayState {
            &self
                .session
                .presentation
                .owner()
                .expect("owned Display")
                .state
        }
        fn display_mut(&mut self) -> &mut sm_game::scene::state::DisplayState {
            &mut self
                .session
                .presentation
                .owner_mut()
                .expect("owned Display")
                .state
        }
        /// The lease's `ScanlineIrq` read of `lane`.
        fn read(&self, lane: Lane) -> u16 {
            if self.owned() {
                let display = self.display();
                return match lane {
                    Lane::History(word) => preparation::display_word(display, word),
                    Lane::Controls(low, high) => u16::from_le_bytes([
                        preparation::display_byte(display, low),
                        preparation::display_byte(display, high),
                    ]),
                };
            }
            let [low, high] = lane.addresses();
            u16::from_le_bytes([self.byte(low), self.byte(high)])
        }
        fn read_field(&self, field: Field) -> u8 {
            if self.owned() {
                preparation::display_byte(self.display(), field)
            } else {
                self.byte(frame_field_address(field))
            }
        }
        /// The lease's `ScanlineIrq` store of `value` in `lane`, byte by byte as
        /// `store_display_history_word` and `Place::write` make it.
        fn write(&mut self, lane: Lane, value: u16) {
            let bytes = value.to_le_bytes();
            match lane {
                Lane::History(word) => {
                    for (index, (byte, at)) in bytes.into_iter().zip(lane.addresses()).enumerate() {
                        if self.owned() {
                            let display = self.display_mut();
                            let mut lanes = preparation::display_word(display, word).to_le_bytes();
                            lanes[index] = byte;
                            preparation::set_display_word(display, word, u16::from_le_bytes(lanes));
                        } else {
                            self.set_byte(at, byte);
                        }
                    }
                }
                Lane::Controls(low, high) => {
                    for (byte, field) in bytes.into_iter().zip([low, high]) {
                        if self.owned() {
                            preparation::set_display_byte(self.display_mut(), field, byte);
                        } else {
                            self.set_byte(frame_field_address(field), byte);
                        }
                    }
                }
            }
        }
    }

    /// `ram` with `session`'s spans written over it, as a publish writes them.
    fn publish(session: &Session, ram: &[u8]) -> Vec<u8> {
        let mut image = ram.to_vec();
        encoding::encode_spans(session, |at: usize, bytes: &[u8]| {
            image[at..at + bytes.len()].copy_from_slice(bytes);
        });
        image
    }

    struct Pass {
        frame: i32,
        line: u16,
        lease: Lease,
        stored: Vec<usize>,
        resumed: bool,
        read_mismatches: usize,
    }

    #[derive(Default)]
    struct Counts {
        passes: u64,
        checked: u64,
        owned: u64,
        persistent: u64,
        kept: u64,
        import: u64,
        resumed: u64,
        reads: u64,
        lane_mismatches: u64,
        read_mismatches: u64,
        other_mismatches: u64,
    }

    thread_local! {
        static PASS: RefCell<Option<Pass>> = const { RefCell::new(None) };
        static COUNTS: RefCell<Counts> = RefCell::new(Counts::default());
    }

    fn enabled() -> bool {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        *ENABLED.get_or_init(|| std::env::var_os("SM_IRQ_PUBLISH_CHECK").is_some())
    }

    pub(crate) fn begin_pass(game: &Game, line: u16) {
        if !enabled() {
            return;
        }
        let pass = Pass {
            frame: game.snes_frame_counter,
            line,
            lease: Lease::open(game),
            stored: Vec::new(),
            resumed: false,
            read_mismatches: 0,
        };
        PASS.with_borrow_mut(|slot| *slot = Some(pass));
    }

    fn with_pass(action: impl FnOnce(&mut Pass)) {
        if enabled() {
            PASS.with_borrow_mut(|slot| {
                if let Some(pass) = slot.as_mut() {
                    action(pass);
                }
            });
        }
    }

    pub(crate) fn read_control(field: irq::Control, value: u16) {
        with_pass(|pass| {
            if let Some(lane) = Lane::of_control(field) {
                COUNTS.with_borrow_mut(|counts| counts.reads += 1);
                if pass.lease.read(lane) != value {
                    pass.read_mismatches += 1;
                }
            }
        });
    }

    pub(crate) fn read_mirror(field: irq::Mirror, value: u8) {
        with_pass(|pass| {
            COUNTS.with_borrow_mut(|counts| counts.reads += 1);
            if pass.lease.read_field(mirror_field(field)) != value {
                pass.read_mismatches += 1;
            }
        });
    }

    pub(crate) fn store(field: irq::Store, value: u16) {
        with_pass(|pass| {
            let lane = Lane::of_store(field);
            pass.stored.extend(lane.addresses());
            pass.lease.write(lane, value);
        });
    }

    /// After a door resume poll's own lease: the frame lease would hold the
    /// Session that poll released, so the copy is taken again. The poll comes
    /// before its pass's only store (`scene::irq::advance`, handlers 9 and 12).
    pub(crate) fn resumed(game: &Game) {
        with_pass(|pass| {
            pass.resumed = true;
            pass.lease = Lease::open(game);
        });
    }

    pub(crate) fn end_pass(game: &Game) {
        if !enabled() {
            return;
        }
        let Some(pass) = PASS.with_borrow_mut(Option::take) else {
            return;
        };
        let compared = !pass.stored.is_empty() || pass.resumed;
        let mut lanes = Vec::new();
        let mut other = Vec::new();
        if compared {
            let leased = publish(&pass.lease.session, &pass.lease.ram);
            let live = game.ram().peek_image();
            for (at, ((leased, live), &stale)) in
                leased.iter().zip(&live).zip(&pass.lease.stale).enumerate()
            {
                if leased == live {
                    continue;
                }
                if pass.stored.contains(&at) {
                    lanes.push(at);
                } else if !stale {
                    other.push(at);
                }
            }
        }
        let counts = COUNTS.with_borrow_mut(|counts| {
            counts.passes += 1;
            counts.checked += u64::from(compared);
            counts.owned += u64::from(pass.lease.owned());
            match pass.lease.source {
                LeaseSource::Persistent => counts.persistent += 1,
                LeaseSource::Kept => counts.kept += 1,
                LeaseSource::Import => counts.import += 1,
            }
            counts.resumed += u64::from(pass.resumed);
            counts.lane_mismatches += u64::from(!lanes.is_empty());
            counts.read_mismatches += u64::from(pass.read_mismatches != 0);
            counts.other_mismatches += u64::from(!other.is_empty());
            format!(
                "passes={} checked={} owned={} persistent={} kept={} import={} resumed={} \
                 reads={} lane_mismatches={} read_mismatches={} other_mismatches={}",
                counts.passes,
                counts.checked,
                counts.owned,
                counts.persistent,
                counts.kept,
                counts.import,
                counts.resumed,
                counts.reads,
                counts.lane_mismatches,
                counts.read_mismatches,
                counts.other_mismatches,
            )
        });
        if let Some(mut path) = std::env::var_os("SM_IRQ_PUBLISH_COUNTS") {
            path.push(format!(".{}", std::process::id()));
            std::fs::write(path, format!("{counts}\n")).unwrap();
        }
        if lanes.is_empty() && other.is_empty() && pass.read_mismatches == 0 {
            return;
        }
        let addresses = |list: &[usize]| {
            list.iter()
                .take(8)
                .map(|at| format!("{at:05x}"))
                .collect::<Vec<_>>()
                .join(" ")
        };
        let row = format!(
            "{},{},{:?}{},lanes {} [{}],reads {},other {} [{}]",
            pass.frame,
            pass.line,
            pass.lease.source,
            if pass.resumed { " resumed" } else { "" },
            lanes.len(),
            addresses(&lanes),
            pass.read_mismatches,
            other.len(),
            addresses(&other),
        );
        if let Some(path) = std::env::var_os("SM_IRQ_PUBLISH_LOG") {
            let mut log = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .unwrap();
            writeln!(log, "{row}").unwrap();
        }
    }
}
