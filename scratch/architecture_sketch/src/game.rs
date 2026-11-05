//! The `game` crate: the Session, its State, and the Tick from cut to cut.

use crate::data::{Change, Content, Cycles, DisplayFrame, Registers, Screens, SpriteEntry};

/// The controller word.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Buttons(pub u16);

impl Buttons {
    /// The sketch's stand-in for touching a door, which really comes from collision.
    pub const ENTER_DOOR: Self = Self(0x8000);
    /// Right on the d-pad.
    pub const RIGHT: Self = Self(0x0100);

    /// Whether every button in `other` is held.
    #[must_use]
    pub const fn holds(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

/// Whether an NMI arrived after the main strand yielded, or while it was still working (D8).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NmiTiming {
    /// After the main strand yielded.
    #[default]
    OnTime,
    /// While the main strand was still working.
    Overrun,
}

/// Which path an NMI took.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NmiKind {
    /// The commit path: the batch was applied and the pad latched.
    Requested,
    /// The lag path: counters only.
    Lag,
}

/// Where a Session's NMI timing comes from (D8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimingSource {
    /// From the input, which carries the oracle's schedule.
    Recorded,
    /// From the cost model.
    Modeled,
    /// Always on time.
    None,
}

/// One Tick's input.
#[derive(Clone, Copy, Debug, Default)]
pub struct TickInput {
    /// The pad as auto-read at this Tick's NMI.
    pub pad: Buttons,
    /// The NMI's timing, read only under `TimingSource::Recorded`.
    pub timing: NmiTiming,
}

/// The Tick counter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tick(pub u64);

/// The NMI counters, which wrap as the original's do.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counters {
    /// Every NMI.
    pub nmi: u16,
    /// Lag NMIs since the last requested one.
    pub lag: u16,
    /// Requested NMIs.
    pub completed: u16,
}

/// The RNG, with its call count for the `rng` stream.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rng {
    /// The original's RNG word.
    pub word: u16,
    /// Calls since the Session opened.
    pub calls: u32,
}

impl Rng {
    /// The original's step: `r * 5 + 0x111`.
    pub const fn next(&mut self) -> u16 {
        self.word = self.word.wrapping_mul(5).wrapping_add(0x111);
        self.calls = self.calls.wrapping_add(1);
        self.word
    }
}

/// What a yielded main strand waits for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wait {
    /// A requested NMI.
    Nmi,
}

/// Where the main loop is within an iteration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MainLoopAt {
    /// Before the HDMA object handler.
    Start,
    /// After it: the beam-sync point has passed (D7).
    AfterHdmaObjects,
    /// After the game state dispatch, before drawing.
    AfterGameplay,
}

/// A busy routine's place (D9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadingAt {
    /// Decompressing this many bytes of the next room.
    Decompress {
        /// Bytes the decompressor produces.
        bytes: u32,
    },
}

/// A resumable routine on the main strand (D10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frame {
    /// The main loop.
    MainLoop(MainLoopAt),
    /// A room load.
    Loading(LoadingAt),
}

/// The main strand: a short stack of typed bookmarks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MainStrand {
    /// Innermost last.
    pub frames: Vec<Frame>,
    /// What it waits for, if it has yielded.
    pub waiting: Option<Wait>,
}

/// An OAM entry in the original's encoding: 9-bit X, 8-bit Y.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OamEntry {
    /// X, two's complement in 9 bits.
    pub x: u16,
    /// Y; `PARKED` hides the entry.
    pub y: u8,
    /// The tile.
    pub tile: u16,
}

impl OamEntry {
    /// The Y at which the original parks unused entries.
    pub const PARKED: u8 = 0xF0;
    /// OAM's capacity.
    pub const CAPACITY: usize = 128;
}

/// What the requested NMI commits.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Batch {
    /// The register mirrors.
    pub registers: Registers,
    /// The OAM buffer.
    pub oam: Vec<OamEntry>,
}

/// The interrupt strand's command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IrqCommand {
    /// No interrupt work.
    Off,
    /// The gameplay HUD split.
    GameplayHud,
}

/// An enemy, reduced to what the sketch needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Enemy {
    /// Room X.
    pub x: i32,
    /// Room Y.
    pub y: i32,
    /// Animation frame, advanced only while processed.
    pub frame: u16,
}

/// The whole Session as plain data (D3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// The Tick counter.
    pub tick: Tick,
    /// NMI counters.
    pub counters: Counters,
    /// The RNG.
    pub rng: Rng,
    /// The latched pad.
    pub held: Buttons,
    /// The pad as last auto-read.
    pub auto_read: Buttons,
    /// The main strand.
    pub main: MainStrand,
    /// Whether the main strand has requested the next NMI.
    pub nmi_requested: bool,
    /// The committed registers.
    pub registers: Registers,
    /// The committed OAM.
    pub oam: Vec<OamEntry>,
    /// The batch waiting for a requested NMI.
    pub batch: Batch,
    /// The interrupt strand.
    pub irq: IrqCommand,
    /// Samus's room position.
    pub samus: (i32, i32),
    /// The camera's room X.
    pub camera_x: i32,
    /// The room's enemies.
    pub enemies: Vec<Enemy>,
    /// CPU time still owed by a busy routine, under the `Modeled` timing source.
    pub owed: Cycles,
}

impl State {
    /// A room with Samus, two enemies near the camera and one far off screen.
    #[must_use]
    pub fn sample() -> Self {
        Self {
            tick: Tick::default(),
            counters: Counters::default(),
            rng: Rng::default(),
            held: Buttons::default(),
            auto_read: Buttons::default(),
            main: MainStrand { frames: vec![Frame::MainLoop(MainLoopAt::Start)], waiting: None },
            nmi_requested: false,
            registers: Registers { screens: Screens { bg1: false, bg3: true }, bg1_scroll_x: 0 },
            oam: vec![OamEntry { y: OamEntry::PARKED, ..OamEntry::default() }; OamEntry::CAPACITY],
            batch: Batch::default(),
            irq: IrqCommand::GameplayHud,
            samus: (0x80, 0xB0),
            camera_x: 0,
            enemies: vec![
                Enemy { x: 0x40, y: 0x90, frame: 0 },
                Enemy { x: 0xF0, y: 0x90, frame: 0 },
                Enemy { x: 0x150, y: 0x90, frame: 0 },
            ],
            owed: Cycles::default(),
        }
    }
}

/// An engine fault. It poisons the Session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionFault {
    /// A strand ran past its step bound.
    StepBound,
    /// A strand's bookmark stack is empty or overfull.
    Strand,
}

/// One Tick's output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TickOutput {
    /// The Tick just completed.
    pub tick: Tick,
    /// The NMI that ended it.
    pub nmi: NmiKind,
    /// The picture scanned out during it, the one the previous NMI committed.
    pub picture: DisplayFrame,
}

/// Where the main strand stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stop {
    Synced,
    Yielded,
}

/// What a frame's step asks for.
enum Next {
    Continue,
    Sync,
    Call(Frame),
    Return,
    Yield(Wait),
}

/// The deterministic Session.
#[derive(Debug)]
pub struct Session {
    state: State,
    content: Content,
    timing: TimingSource,
}

/// A Tick between its scan-out and its NMI. It borrows the Session, so no Checkpoint can
/// be taken mid-Tick.
#[derive(Debug)]
pub struct Scan<'a> {
    session: &'a mut Session,
    picture: DisplayFrame,
}

impl Scan<'_> {
    /// The picture scanned out in this Tick.
    #[must_use]
    pub const fn picture(&self) -> &DisplayFrame {
        &self.picture
    }

    /// Finish the main strand, take the NMI, and reach the cut.
    ///
    /// # Errors
    ///
    /// A `SessionFault` if a strand breaks its bounds.
    pub fn end(self, input: TickInput) -> Result<TickOutput, SessionFault> {
        let Self { session, picture } = self;
        if session.state.main.waiting.is_none() {
            session.run_main(false)?;
        }
        let nmi = session.nmi(input);
        session.state.tick = Tick(session.state.tick.0.saturating_add(1));
        Ok(TickOutput { tick: session.state.tick, nmi, picture })
    }
}

impl Session {
    /// Open at a cut.
    #[must_use]
    pub const fn open(content: Content, state: State, timing: TimingSource) -> Self {
        Self { state, content, timing }
    }

    /// The State at the last cut.
    #[must_use]
    pub const fn state(&self) -> &State {
        &self.state
    }

    /// A Checkpoint. Only possible at a cut, because `Scan` borrows the Session.
    #[must_use]
    pub fn checkpoint(&self) -> State {
        self.state.clone()
    }

    /// One whole Tick.
    ///
    /// # Errors
    ///
    /// A `SessionFault` if a strand breaks its bounds.
    pub fn step(&mut self, input: TickInput) -> Result<TickOutput, SessionFault> {
        self.begin()?.end(input)
    }

    /// Run the main strand to its beam-sync point, then scan out the previous picture.
    ///
    /// # Errors
    ///
    /// A `SessionFault` if a strand breaks its bounds.
    pub fn begin(&mut self) -> Result<Scan<'_>, SessionFault> {
        if self.state.main.waiting.is_none() {
            self.run_main(true)?;
        }
        let picture = self.scan_out();
        Ok(Scan { session: self, picture })
    }

    fn run_main(&mut self, stop_at_sync: bool) -> Result<Stop, SessionFault> {
        for _ in 0..64 {
            let frame = *self.state.main.frames.last().ok_or(SessionFault::Strand)?;
            match self.step_frame(frame) {
                Next::Sync if stop_at_sync => return Ok(Stop::Synced),
                Next::Continue | Next::Sync => {}
                Next::Call(callee) => {
                    if self.state.main.frames.len() >= 8 {
                        return Err(SessionFault::Strand);
                    }
                    self.state.main.frames.push(callee);
                }
                Next::Return => {
                    self.state.main.frames.pop();
                }
                Next::Yield(wait) => {
                    self.state.main.waiting = Some(wait);
                    return Ok(Stop::Yielded);
                }
            }
        }
        Err(SessionFault::StepBound)
    }

    fn set_top(&mut self, frame: Frame) {
        if let Some(top) = self.state.main.frames.last_mut() {
            *top = frame;
        }
    }

    fn step_frame(&mut self, frame: Frame) -> Next {
        match frame {
            Frame::MainLoop(MainLoopAt::Start) => {
                self.set_top(Frame::MainLoop(MainLoopAt::AfterHdmaObjects));
                Next::Sync
            }
            Frame::MainLoop(MainLoopAt::AfterHdmaObjects) => {
                self.state.rng.next();
                self.gameplay();
                self.set_top(Frame::MainLoop(MainLoopAt::AfterGameplay));
                if self.state.held.holds(Buttons::ENTER_DOOR) {
                    Next::Call(Frame::Loading(LoadingAt::Decompress { bytes: 25_000 }))
                } else {
                    Next::Continue
                }
            }
            Frame::MainLoop(MainLoopAt::AfterGameplay) => {
                let registers = Registers {
                    screens: Screens { bg1: false, bg3: true },
                    bg1_scroll_x: u16::try_from(self.state.camera_x.rem_euclid(0x400)).unwrap_or(0),
                };
                self.state.batch = Batch { registers, oam: encode_oam(&draw(&self.state, &self.content)) };
                self.state.nmi_requested = true;
                self.set_top(Frame::MainLoop(MainLoopAt::Start));
                Next::Yield(Wait::Nmi)
            }
            Frame::Loading(LoadingAt::Decompress { bytes }) => {
                let cost = Cycles(self.content.costs.per_decompressed_byte.0.saturating_mul(bytes));
                self.state.owed = self.state.owed.plus(cost);
                self.state.camera_x = 0;
                self.state.samus.0 = 0x30;
                Next::Return
            }
        }
    }

    fn gameplay(&mut self) {
        if self.state.held.holds(Buttons::RIGHT) {
            self.state.samus.0 = self.state.samus.0.saturating_add(2);
            self.state.camera_x = self.state.samus.0.saturating_sub(0x80).max(0);
        }
        let viewport = self.content.viewport;
        let camera = self.state.camera_x;
        for enemy in &mut self.state.enemies {
            if viewport.overlaps(camera, enemy.x.saturating_sub(8), enemy.x.saturating_add(8)) {
                enemy.frame = enemy.frame.wrapping_add(1);
            }
        }
    }

    fn scan_out(&self) -> DisplayFrame {
        let mut changes = Vec::new();
        match self.state.irq {
            IrqCommand::GameplayHud => changes.push((
                self.content.oracle.hud_write,
                Change::Screens(Screens { bg1: true, bg3: false }),
            )),
            IrqCommand::Off => {}
        }
        DisplayFrame { start: self.state.registers, changes }
    }

    fn timing(&mut self, input: TickInput) -> NmiTiming {
        match self.timing {
            TimingSource::Recorded => input.timing,
            TimingSource::None => NmiTiming::OnTime,
            TimingSource::Modeled => {
                let budget = self.content.costs.frame_budget;
                match self.state.owed.minus(budget) {
                    Some(left) if left > Cycles(0) => {
                        self.state.owed = left;
                        NmiTiming::Overrun
                    }
                    Some(_) | None => {
                        self.state.owed = Cycles(0);
                        NmiTiming::OnTime
                    }
                }
            }
        }
    }

    fn nmi(&mut self, input: TickInput) -> NmiKind {
        let timing = self.timing(input);
        let state = &mut self.state;
        let requested = state.main.waiting == Some(Wait::Nmi) && state.nmi_requested;
        let kind = match (requested, timing) {
            (true, NmiTiming::OnTime) => {
                let batch = core::mem::take(&mut state.batch);
                state.registers = batch.registers;
                state.oam = batch.oam;
                state.counters.completed = state.counters.completed.wrapping_add(1);
                state.counters.lag = 0;
                state.held = input.pad;
                state.main.waiting = None;
                state.nmi_requested = false;
                NmiKind::Requested
            }
            (true, NmiTiming::Overrun) | (false, NmiTiming::OnTime | NmiTiming::Overrun) => {
                state.counters.lag = state.counters.lag.wrapping_add(1);
                NmiKind::Lag
            }
        };
        state.counters.nmi = state.counters.nmi.wrapping_add(1);
        state.auto_read = input.pad;
        kind
    }
}

/// The one drawing source: Samus and every processed enemy, at full precision.
#[must_use]
pub fn draw(state: &State, content: &Content) -> Vec<SpriteEntry> {
    let camera = state.camera_x;
    let samus = SpriteEntry {
        x: state.samus.0.saturating_sub(camera),
        y: state.samus.1,
        tile: 0,
    };
    core::iter::once(samus)
        .chain(
            state
                .enemies
                .iter()
                .filter(|enemy| processed(enemy, camera, content))
                .map(|enemy| enemy_entry(enemy, camera)),
        )
        .collect()
}

const fn processed(enemy: &Enemy, camera: i32, content: &Content) -> bool {
    content.viewport.overlaps(camera, enemy.x.saturating_sub(8), enemy.x.saturating_add(8))
}

const fn enemy_entry(enemy: &Enemy, camera: i32) -> SpriteEntry {
    SpriteEntry {
        x: enemy.x.saturating_sub(camera),
        y: enemy.y,
        tile: 0x10_u16.wrapping_add(enemy.frame & 3),
    }
}

/// The original's OAM encoding: 9-bit X, entries off the 9-bit range dropped, 128 entries,
/// the rest parked.
#[must_use]
pub fn encode_oam(entries: &[SpriteEntry]) -> Vec<OamEntry> {
    let parked = OamEntry { y: OamEntry::PARKED, ..OamEntry::default() };
    entries
        .iter()
        .filter(|entry| (-0x100..0x100).contains(&entry.x))
        .filter_map(|entry| {
            let x = u16::try_from(entry.x.rem_euclid(0x200)).ok()?;
            let y = u8::try_from(entry.y).ok()?;
            Some(OamEntry { x, y, tile: entry.tile })
        })
        .chain(core::iter::repeat(parked))
        .take(OamEntry::CAPACITY)
        .collect()
}

/// The intent view (D16): a pure function of the State.
pub mod view {
    use super::{State, draw, enemy_entry, processed};
    use crate::data::{Content, SpriteEntry};

    /// What presentation upgrades read.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct View {
        /// The camera's room X.
        pub camera_x: i32,
        /// The exact path's sprite entries, at full precision.
        pub sprites: Vec<SpriteEntry>,
        /// Live actors outside the processing window, drawn by the same function.
        pub margin: Vec<SpriteEntry>,
    }

    /// Describe the State for presentation. It takes `&State` and can't change it.
    #[must_use]
    pub fn describe(state: &State, content: &Content) -> View {
        let camera = state.camera_x;
        View {
            camera_x: camera,
            sprites: draw(state, content),
            margin: state
                .enemies
                .iter()
                .filter(|enemy| !processed(enemy, camera, content))
                .map(|enemy| enemy_entry(enemy, camera))
                .collect(),
        }
    }
}

/// The one instruction-list interpreter, with a concrete context per family (D13).
pub mod program {
    /// What a family op asks the interpreter to do.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Flow {
        /// Run the next step.
        Continue,
        /// Delete the object.
        Delete,
    }

    /// A slot family whose objects run instruction lists.
    pub trait Family {
        /// What a `Show` step shows.
        type Payload: Copy;
        /// The family's own instructions.
        type Op: Copy;
        /// The disjoint borrows this family's pass hands to its ops.
        type Context<'a>;
        /// Show a payload.
        fn show(cx: &mut Self::Context<'_>, payload: Self::Payload);
        /// Run a family op.
        fn op(cx: &mut Self::Context<'_>, op: Self::Op) -> Flow;
    }

    /// One step of a decoded instruction list.
    pub enum Step<F: Family> {
        /// Show a payload for some frames.
        Show {
            /// How many frames.
            frames: u16,
            /// What to show.
            payload: F::Payload,
        },
        /// Jump to a step.
        Goto(u16),
        /// Delete the object.
        Delete,
        /// A family op.
        Family(F::Op),
    }

    /// An object's place in its list.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Cursor {
        /// The next step.
        pub at: u16,
        /// Frames left on the current `Show`.
        pub delay: u16,
    }

    /// Whether the object lives on.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Lifetime {
        /// It lives on.
        Alive,
        /// It deleted itself.
        Deleted,
    }

    /// A list that ran off its end or past its bound.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct InterpretFault;

    /// Run one object for one frame.
    ///
    /// # Errors
    ///
    /// `InterpretFault` if the list has no step at the cursor or runs past `budget`.
    pub fn run<F: Family>(
        program: &[Step<F>],
        cursor: &mut Cursor,
        cx: &mut F::Context<'_>,
        budget: u16,
    ) -> Result<Lifetime, InterpretFault> {
        cursor.delay = cursor.delay.saturating_sub(1);
        if cursor.delay != 0 {
            return Ok(Lifetime::Alive);
        }
        for _ in 0..budget {
            let step = program.get(usize::from(cursor.at)).ok_or(InterpretFault)?;
            cursor.at = cursor.at.checked_add(1).ok_or(InterpretFault)?;
            match step {
                Step::Show { frames, payload } => {
                    F::show(cx, *payload);
                    cursor.delay = *frames;
                    return Ok(Lifetime::Alive);
                }
                Step::Goto(at) => cursor.at = *at,
                Step::Delete => return Ok(Lifetime::Deleted),
                Step::Family(op) => match F::op(cx, *op) {
                    Flow::Continue => {}
                    Flow::Delete => return Ok(Lifetime::Deleted),
                },
            }
        }
        Err(InterpretFault)
    }
}

#[cfg(test)]
mod tests {
    use super::program::{Cursor, Family, Flow, Lifetime, Step, run};
    use super::*;

    fn session(timing: TimingSource) -> Session {
        Session::open(Content::vanilla(), State::sample(), timing)
    }

    #[test]
    fn an_overrun_commits_one_frame_late_with_one_rng_call() {
        let mut session = session(TimingSource::Recorded);
        let on_time = TickInput::default();
        let overrun = TickInput { timing: NmiTiming::Overrun, ..TickInput::default() };
        assert_eq!(session.step(on_time).unwrap().nmi, NmiKind::Requested);
        let calls = session.state().rng.calls;
        assert_eq!(session.step(overrun).unwrap().nmi, NmiKind::Lag);
        assert_eq!(session.state().counters.lag, 1);
        assert_eq!(session.step(on_time).unwrap().nmi, NmiKind::Requested);
        assert_eq!(session.state().rng.calls, calls.wrapping_add(1), "the overrun iteration ran once");
        assert_eq!(session.state().counters.nmi, 3);
        assert_eq!(session.state().counters.completed, 2);
    }

    #[test]
    fn a_load_costs_the_frames_its_work_takes() {
        let mut session = session(TimingSource::Modeled);
        session.step(TickInput::default()).unwrap();
        let door = TickInput { pad: Buttons::ENTER_DOOR, ..TickInput::default() };
        session.step(door).unwrap();
        let kinds: Vec<NmiKind> =
            (0..5).map(|_| session.step(TickInput::default()).unwrap().nmi).collect();
        // 25,000 bytes at 40 cycles is 1,000,000 cycles: three frames over a 300,000 budget.
        assert_eq!(
            kinds,
            [NmiKind::Lag, NmiKind::Lag, NmiKind::Lag, NmiKind::Requested, NmiKind::Requested]
        );
    }

    #[test]
    fn the_hud_split_lands_where_the_oracle_rules_say() {
        let mut session = session(TimingSource::None);
        let output = session.step(TickInput::default()).unwrap();
        assert_eq!(output.picture.changes.first().map(|(at, _)| *at), Some(Content::vanilla().oracle.hud_write));
    }

    #[test]
    fn begin_returns_the_picture_before_the_input_is_known() {
        let mut session = session(TimingSource::None);
        let scan = session.begin().unwrap();
        let picture = scan.picture().clone();
        let output = scan.end(TickInput::default()).unwrap();
        assert_eq!(output.picture, picture);
    }

    struct Plm;
    struct PlmContext<'a> {
        level: &'a mut [u16],
        sounds: &'a mut Vec<u8>,
        block: usize,
    }
    impl Family for Plm {
        type Payload = u16;
        type Op = u8;
        type Context<'a> = PlmContext<'a>;
        fn show(cx: &mut PlmContext<'_>, block: u16) {
            if let Some(cell) = cx.level.get_mut(cx.block) {
                *cell = block;
            }
        }
        fn op(cx: &mut PlmContext<'_>, sound: u8) -> Flow {
            cx.sounds.push(sound);
            Flow::Continue
        }
    }

    #[test]
    fn one_interpreter_runs_a_family_through_its_concrete_context() {
        let program: [Step<Plm>; 4] = [
            Step::Show { frames: 1, payload: 5 },
            Step::Family(3),
            Step::Show { frames: 2, payload: 7 },
            Step::Delete,
        ];
        let (mut level, mut sounds) = ([0u16; 4], Vec::new());
        let mut cursor = Cursor::default();
        let mut cx = PlmContext { level: &mut level, sounds: &mut sounds, block: 2 };
        let lives: Vec<Lifetime> = (0..4).map(|_| run(&program, &mut cursor, &mut cx, 8).unwrap()).collect();
        assert_eq!(lives, [Lifetime::Alive, Lifetime::Alive, Lifetime::Alive, Lifetime::Deleted]);
        assert_eq!((level[2], sounds), (7, vec![3]));
    }
}
