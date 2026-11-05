//! `sm_game`: the Session, its state schema, the Tick and the shared machinery
//! (slot store, program interpreter, strands).
use crate::sm_data::display::{
    Change, CommitPoint, DisplayFrame, Line, LineChange, Registers, Target, Upload, VideoMemory,
};
use crate::sm_data::ids::{DoorwayId, RoomId};
use crate::sm_data::profile::{GameplayProfile, ReferenceProfile};
use crate::sm_data::program::{CoreOp, Family, Frames, Label, Programs, Step};
use crate::sm_data::sound::{Port, PortFrame, SoundFrame};
use std::marker::PhantomData;
use std::sync::Arc;

// ---------------------------------------------------------------- slot store

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnOrder {
    Ascending,
    Descending,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunOrder {
    /// A live loop: a spawn into a slot the pass has not reached runs this Tick.
    LiveAscending,
    LiveDescending,
    /// The enemies' active list, built once at the start of the pass ($A0:8EB6).
    ListedAtPassStart,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeleteRule {
    Immediate,
    /// The slot stays `Deleting` until the next determine pass frees it.
    NextPass,
}
#[derive(Clone, Copy, Debug)]
pub struct SlotPolicy {
    pub spawn: SpawnOrder,
    pub run: RunOrder,
    pub delete: DeleteRule,
}

/// A slot number the original's behaviour can see (processing order, contact
/// tie-break, boss part arithmetic).
#[derive(Debug)]
pub struct SlotIndex<F>(pub u8, PhantomData<F>);
impl<F> Clone for SlotIndex<F> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<F> Copy for SlotIndex<F> {}
impl<F> SlotIndex<F> {
    pub const fn new(index: u8) -> Self {
        Self(index, PhantomData)
    }
}

/// A reference the original keeps as a raw slot number: it sees whatever
/// occupies the slot now, as the original does.
pub type SlotRef<F> = SlotIndex<F>;

/// A reference whose behaviour must not follow a reused slot.
#[derive(Debug)]
pub struct Handle<F> {
    pub slot: SlotIndex<F>,
    pub generation: u32,
}

#[derive(Clone, Debug)]
pub enum Occupancy<B> {
    Free,
    Live(B),
    Deleting(B),
}

/// A slot: the common fields persist across occupants (the store never clears
/// them; only spawn code writes what the original's init writes), the body is
/// the occupant's typed state.
#[derive(Clone, Debug)]
pub struct Slot<C, B> {
    pub common: C,
    pub body: Occupancy<B>,
    pub generation: u32,
}

#[derive(Clone, Debug)]
pub struct SlotStore<F, C, B, const N: usize> {
    pub slots: [Slot<C, B>; N],
    pub policy: SlotPolicyOf<F>,
}
#[derive(Debug)]
pub struct SlotPolicyOf<F>(pub SlotPolicy, PhantomData<F>);
impl<F> Clone for SlotPolicyOf<F> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnResult<F> {
    Spawned(SlotIndexEq<F>),
    /// The table is full: an original limit, a defined gameplay result.
    Full,
}
#[derive(Debug)]
pub struct SlotIndexEq<F>(pub u8, PhantomData<F>);
impl<F> Clone for SlotIndexEq<F> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<F> Copy for SlotIndexEq<F> {}
impl<F> PartialEq for SlotIndexEq<F> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<F> Eq for SlotIndexEq<F> {}

impl<F, C, B, const N: usize> SlotStore<F, C, B, N> {
    pub fn spawn(&mut self, body: B, init: impl FnOnce(&mut C)) -> SpawnResult<F> {
        let free = |slot: &Slot<C, B>| matches!(slot.body, Occupancy::Free);
        let found = match self.policy.0.spawn {
            SpawnOrder::Ascending => self.slots.iter().position(free),
            SpawnOrder::Descending => self.slots.iter().rposition(free),
        };
        let Some(index) = found else {
            return SpawnResult::Full;
        };
        let slot = &mut self.slots[index];
        init(&mut slot.common);
        slot.body = Occupancy::Live(body);
        slot.generation = slot.generation.wrapping_add(1);
        SpawnResult::Spawned(SlotIndexEq(
            u8::try_from(index).expect("slot tables hold at most 255 slots"),
            PhantomData,
        ))
    }

    /// This slot mutably and every other slot read-only.
    pub fn split(&mut self, index: usize) -> (&mut Slot<C, B>, Others<'_, C, B>) {
        let (before, rest) = self.slots.split_at_mut(index);
        let (current, after) = rest.split_first_mut().expect("slot index in range");
        (current, Others { before, after })
    }

    /// The pass order. A live order re-reads occupancy at each step, so a slot
    /// filled ahead of the cursor during the pass runs in the same pass.
    pub fn pass(&self) -> Pass {
        match self.policy.0.run {
            RunOrder::LiveAscending => Pass::Live { next: Some(0), ascending: true },
            RunOrder::LiveDescending => Pass::Live { next: N.checked_sub(1), ascending: false },
            RunOrder::ListedAtPassStart => Pass::Listed {
                list: self
                    .slots
                    .iter()
                    .enumerate()
                    .filter(|(_, slot)| matches!(slot.body, Occupancy::Live(_)))
                    .map(|(index, _)| index)
                    .collect(),
                at: 0,
            },
        }
    }
}

#[derive(Debug)]
pub struct Others<'a, C, B> {
    before: &'a [Slot<C, B>],
    after: &'a [Slot<C, B>],
}
impl<C, B> Others<'_, C, B> {
    pub fn get(&self, index: usize) -> Option<&Slot<C, B>> {
        let split = self.before.len();
        match index.cmp(&split) {
            core::cmp::Ordering::Less => self.before.get(index),
            core::cmp::Ordering::Equal => None,
            core::cmp::Ordering::Greater => self.after.get(index - split - 1),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Pass {
    Live { next: Option<usize>, ascending: bool },
    Listed { list: Vec<usize>, at: usize },
}
impl Pass {
    pub fn next<C, B, F, const N: usize>(&mut self, store: &SlotStore<F, C, B, N>) -> Option<usize> {
        match self {
            Self::Live { next, ascending } => loop {
                let index = (*next)?;
                *next = if *ascending {
                    index.checked_add(1).filter(|&n| n < N)
                } else {
                    index.checked_sub(1)
                };
                if matches!(store.slots[index].body, Occupancy::Live(_)) {
                    return Some(index);
                }
            },
            Self::Listed { list, at } => {
                let index = *list.get(*at)?;
                *at += 1;
                Some(index)
            }
        }
    }
}

// --------------------------------------------------------- program interpreter

/// A program cursor, in the slot's typed state.
#[derive(Debug)]
pub struct Cursor<F: Family> {
    pub at: Label<F>,
    pub delay: Frames,
    pub link: Option<Label<F>>,
    pub pre_op: F::PreOp,
    pub timer: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    Continue,
    Yield,
    Delete,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifetime {
    Alive,
    Deleted,
}

/// What a family supplies to the shared loop.
pub trait Interpret<F: Family> {
    fn pre_op(&mut self, pre_op: F::PreOp, cursor: &mut Cursor<F>) -> Flow;
    fn show(&mut self, payload: F::Payload);
    fn op(&mut self, op: F::Op, cursor: &mut Cursor<F>) -> Flow;
    fn sound(&mut self, request: crate::sm_data::sound::SoundRequest);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterpretFault {
    /// A label past its program: the decoder's validation failed to catch it.
    NoStep,
    /// More steps without a yield than the content's validated bound.
    Runaway,
}

/// The original's loop: pre-instruction, timer countdown, then steps until a
/// `Show`, a sleep or a delete.
pub fn run<F: Family, I: Interpret<F>>(
    programs: &Programs<F>,
    cursor: &mut Cursor<F>,
    host: &mut I,
    budget: u16,
) -> Result<Lifetime, InterpretFault> {
    match host.pre_op(cursor.pre_op, cursor) {
        Flow::Delete => return Ok(Lifetime::Deleted),
        Flow::Yield | Flow::Continue => {}
    }
    cursor.delay = Frames(cursor.delay.0.saturating_sub(1));
    if cursor.delay.0 != 0 {
        return Ok(Lifetime::Alive);
    }
    for _ in 0..budget {
        let step = programs.step(cursor.at).ok_or(InterpretFault::NoStep)?;
        cursor.at.at.0 += 1;
        match step {
            Step::Show { frames, payload } => {
                host.show(payload);
                cursor.delay = frames;
                return Ok(Lifetime::Alive);
            }
            Step::Core(op) => match op {
                CoreOp::Delete => return Ok(Lifetime::Deleted),
                CoreOp::Sleep => {
                    cursor.at.at.0 -= 1;
                    cursor.delay = Frames(1);
                    return Ok(Lifetime::Alive);
                }
                CoreOp::Goto(label) => cursor.at = label,
                CoreOp::Link(label) => {
                    cursor.link = Some(cursor.at);
                    cursor.at = label;
                }
                CoreOp::Return => cursor.at = cursor.link.take().ok_or(InterpretFault::NoStep)?,
                CoreOp::SetPreOp(pre_op) => cursor.pre_op = pre_op,
                CoreOp::ClearPreOp => cursor.pre_op = F::PreOp::default(),
                CoreOp::SetTimer(timer) => cursor.timer = timer,
                CoreOp::DecrementTimerAndBranch(label) => {
                    cursor.timer = cursor.timer.wrapping_sub(1);
                    if cursor.timer != 0 {
                        cursor.at = label;
                    }
                }
                CoreOp::QueueSound(request) => host.sound(request),
            },
            Step::Family(op) => match host.op(op, cursor) {
                Flow::Continue => {}
                Flow::Yield => return Ok(Lifetime::Alive),
                Flow::Delete => return Ok(Lifetime::Deleted),
            },
        }
    }
    Err(InterpretFault::Runaway)
}

// ----------------------------------------------------------------- strands

/// Why the main strand stopped this Tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wait {
    /// Wait for NMI with the request set ($80:8338): the NMI commits the batch.
    Nmi,
    /// Wait for the next NMI without requesting it ($85:8136): a structural lag Tick.
    LagFrame,
    /// Wait for NMI, then test a condition other strands set; repeat until true.
    Until(Condition),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    DoorScrollDone,
    DoorVramUpdated,
    MusicQueueEmpty,
}

/// One resumable routine on the main strand. The set is closed: only these can
/// span a Tick, and the oracle decoder maps each yield site to one of them.
#[derive(Clone, Debug)]
pub enum Frame {
    MainLoop(MainLoopAt),
    Gameplay(PassAt),
    Plms(PlmPassAt),
    MessageBox(MessageBoxAt),
    Door(DoorAt),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MainLoopAt {
    Top,
    AfterGameState,
}
/// The $82:8B44 main gameplay pass, phase by phase, in the original's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassAt {
    EnemySelection,
    PaletteFx,
    SamusCurrentState,
    SamusProjectileInteraction,
    Enemies,
    SamusNewState,
    EnemyProjectiles,
    Plms,
    AnimatedTiles,
    Collisions,
    PowerBomb,
    Scrolling,
    Drawing,
    Hud,
    BgScroll,
    RoomMain,
    HealthAndTime,
    RoomShaking,
    HurtTimers,
    Done,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlmPassAt {
    pub slot: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageBoxAt {
    Opening { step: u8 },
    Shown { frames_left: u16 },
    Closing { step: u8 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorAt {
    FadeOut,
    LoadDoor,
    AlignScreen,
    LoadRoom,
    SetUpScrolling,
    PlaceSamus,
    LoadObjects,
    AwaitVramUpdate,
    AwaitScroll,
    FirstPlmPass,
    AnimatedTiles,
    AwaitMusic,
    FadeIn,
}

/// What one step of a frame asks the strand to do next.
#[derive(Clone, Debug)]
pub enum Next {
    Continue,
    Call(Frame),
    Return,
    Yield(Wait),
}

#[derive(Clone, Debug, Default)]
pub struct MainStrand {
    /// Innermost last; bounded by the original's deepest nesting.
    pub frames: Vec<Frame>,
    pub waiting: Option<Wait>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrandFault {
    Empty,
    TooDeep,
    Runaway,
}

impl MainStrand {
    pub const DEPTH: usize = 8;

    /// Run until the strand waits. `step` runs one step of the innermost frame.
    pub fn resume(
        &mut self,
        mut step: impl FnMut(&mut Frame) -> Next,
        mut condition: impl FnMut(Condition) -> bool,
    ) -> Result<Wait, StrandFault> {
        if let Some(Wait::Until(test)) = self.waiting
            && !condition(test)
        {
            return Ok(Wait::Until(test));
        }
        self.waiting = None;
        for _ in 0..10_000 {
            let frame = self.frames.last_mut().ok_or(StrandFault::Empty)?;
            match step(frame) {
                Next::Continue => {}
                Next::Call(inner) => {
                    if self.frames.len() == Self::DEPTH {
                        return Err(StrandFault::TooDeep);
                    }
                    self.frames.push(inner);
                }
                Next::Return => {
                    self.frames.pop();
                }
                Next::Yield(wait) => {
                    self.waiting = Some(wait);
                    return Ok(wait);
                }
            }
        }
        Err(StrandFault::Runaway)
    }
}

/// The interrupt strand: the `$AB` command chain at its lines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IrqCommand {
    Off,
    GameplayHud,
    GameplayAfterHud,
    DoorStartHud,
    DoorStartAfterHud,
    VerticalDoor(DoorIrqAt),
    HorizontalDoor(DoorIrqAt),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorIrqAt {
    Hud,
    Scroll,
    Update,
}

// ------------------------------------------------------------------ state

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Tick(pub u64);

/// Buttons in the original's bit layout, typed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Buttons(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Counters {
    /// $05B5 (8-bit) and $05B6: completed frames.
    pub completed8: u8,
    pub completed16: u16,
    /// $05B8: every NMI.
    pub nmi: u16,
    /// $05BA and $05BB: lag.
    pub lag: u16,
    pub max_lag: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Controller {
    /// The auto-joypad result of the last vblank: what direct reads see.
    pub auto_read: Buttons,
    /// $8B, $8F, $93, $97: latched by a requested NMI only.
    pub held: Buttons,
    pub pressed: Buttons,
    pub repeat: Buttons,
    pub previous: Buttons,
}

/// What the main strand prepares for the NMI ($0370 OAM, $7E:C000 palette,
/// register mirrors, the VRAM write and read tables, Samus tile transfers).
#[derive(Clone, Debug, Default)]
pub struct NmiBatch {
    pub uploads: Vec<(Target, Box<[u16]>)>,
    pub registers: Registers,
    pub readbacks: Vec<Readback>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Readback {
    pub from_word: u16,
    pub words: u16,
    pub into: ReadbackTarget,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadbackTarget {
    PauseBg2Backup,
    XrayBackground,
}

#[derive(Clone, Debug, Default)]
pub struct Video {
    pub memory: VideoMemory,
    pub committed: Registers,
    pub batch: NmiBatch,
    pub nmi_requested: bool,
}

/// The CPU side of the sound protocol plus the Tick-time model of the driver's
/// acknowledgements.
#[derive(Clone, Debug, Default)]
pub struct Sound {
    pub written: PortFrame,
    /// What the CPU reads back from each port this Tick.
    pub echo: [u8; 4],
}
impl Sound {
    /// The driver takes the Tick's port frame after the Tick, as the oracle's
    /// port queue hands it over; its echo is visible from the next Tick.
    pub fn deliver(&mut self) -> PortFrame {
        let ports = core::mem::take(&mut self.written);
        for port in Port::ALL {
            if let Some(value) = ports.0[port.index()] {
                self.echo[port.index()] = value;
            }
        }
        ports
    }
}

/// State reset by a door transition; everything outside it carries.
#[derive(Clone, Debug, Default)]
pub struct RoomScoped {
    pub room: Option<RoomId>,
    pub entered_by: Option<DoorwayId>,
}

/// The whole Session state as plain data: the Checkpoint's schema, what the
/// oracle decoder builds, what the projections read.
#[derive(Clone, Debug, Default)]
pub struct State {
    pub tick: Tick,
    pub counters: Counters,
    pub rng: u16,
    pub controller: Controller,
    pub main: MainStrand,
    pub irq: Option<(Line, IrqCommandSlot)>,
    pub video: Video,
    pub sound: Sound,
    pub room: RoomScoped,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IrqCommandSlot(pub IrqCommand);

// -------------------------------------------------------------- session

#[derive(Debug)]
pub struct Content {
    pub reference: ReferenceProfile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fingerprint(pub [u8; 32]);

#[derive(Clone, Debug)]
pub struct Checkpoint {
    pub fingerprint: Fingerprint,
    pub state: State,
}

#[derive(Clone, Debug)]
pub enum Start {
    Boot,
    /// Ours, or decoded from an oracle snapshot by the verification decoder.
    Checkpoint(Box<Checkpoint>),
    /// A doorway and a typed carried state, for joins no vanilla neighbour gives.
    Arrival { doorway: DoorwayId, carried: Box<State> },
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TickInput {
    /// The pad as sampled at this Tick's vblank.
    pub pad: Buttons,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NmiKind {
    Requested,
    Lag,
}

#[derive(Debug)]
pub struct TickOutput {
    pub tick: Tick,
    pub nmi: NmiKind,
    pub display: DisplayFrame,
    pub sound: SoundFrame,
}

#[derive(Debug)]
pub enum SessionFault {
    Poisoned,
    Strand(StrandFault),
}

#[derive(Debug)]
pub struct Session {
    content: Arc<Content>,
    profile: GameplayProfile,
    state: State,
    poisoned: bool,
}

/// The only route to video memory outside a Tick's NMI: forced blank.
struct VideoWriter<'a> {
    memory: &'a mut VideoMemory,
    at: CommitPoint,
    log: &'a mut Vec<Upload>,
}
impl VideoWriter<'_> {
    fn upload(&mut self, target: Target, words: Box<[u16]>) {
        let (memory, start) = match target {
            Target::Vram { word } => (&mut self.memory.vram, word),
            Target::Cgram { color } => (&mut self.memory.cgram, u16::from(color)),
            Target::Oam { entry } => (&mut self.memory.oam, u16::from(entry) * 2),
            Target::OamHigh { .. } => return,
        };
        for (offset, &word) in words.iter().enumerate() {
            let at = (usize::from(start) + offset) % memory.len();
            memory[at] = word;
        }
        self.log.push(Upload { at: self.at, target, words });
    }
}

impl Session {
    pub fn open(content: Arc<Content>, start: Start) -> Result<Self, SessionFault> {
        let state = match start {
            Start::Boot => State::default(),
            Start::Checkpoint(checkpoint) => checkpoint.state,
            Start::Arrival { carried, .. } => *carried,
        };
        Ok(Self { content, profile: GameplayProfile::VANILLA, state, poisoned: false })
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint { fingerprint: Fingerprint([0; 32]), state: self.state.clone() }
    }

    /// The committed state, read-only, for projections and inspection.
    pub fn state(&self) -> &State {
        &self.state
    }

    /// One video frame: the main strand until it waits, the NMI, the lines.
    pub fn step(&mut self, input: TickInput) -> Result<TickOutput, SessionFault> {
        if self.poisoned {
            return Err(SessionFault::Poisoned);
        }
        let result = self.tick(input);
        self.poisoned = result.is_err();
        result
    }

    fn tick(&mut self, input: TickInput) -> Result<TickOutput, SessionFault> {
        let state = &mut self.state;
        let wait = state
            .main
            .resume(|_frame| Next::Yield(Wait::Nmi), |_condition| true)
            .map_err(SessionFault::Strand)?;
        // `Until` is the oracle's "wait for NMI, then test" loop: it requests the NMI.
        if wait != Wait::LagFrame {
            state.video.nmi_requested = true;
        }
        let mut display = DisplayFrame::default();
        let nmi = if state.video.nmi_requested {
            let batch = core::mem::take(&mut state.video.batch);
            let mut writer = VideoWriter {
                memory: &mut state.video.memory,
                at: CommitPoint::Nmi,
                log: &mut display.uploads,
            };
            for (target, words) in batch.uploads {
                writer.upload(target, words);
            }
            state.video.committed = batch.registers;
            let controller = &mut state.controller;
            controller.previous = controller.held;
            controller.held = input.pad;
            controller.pressed = Buttons(input.pad.0 & !controller.previous.0);
            state.video.nmi_requested = false;
            state.counters.lag = 0;
            state.counters.completed8 = state.counters.completed8.wrapping_add(1);
            state.counters.completed16 = state.counters.completed16.wrapping_add(1);
            NmiKind::Requested
        } else {
            state.counters.lag = state.counters.lag.wrapping_add(1);
            state.counters.max_lag = state.counters.max_lag.max(state.counters.lag);
            NmiKind::Lag
        };
        state.counters.nmi = state.counters.nmi.wrapping_add(1);
        state.controller.auto_read = input.pad;
        display.start = state.video.committed;
        Self::lines(state, &mut display);
        let ports = state.sound.deliver();
        state.tick.0 += 1;
        Ok(TickOutput {
            tick: state.tick,
            nmi,
            display,
            sound: SoundFrame { ports, uploads: Vec::new() },
        })
    }

    /// Each line: raster with the registers in effect, then the HDMA writes for
    /// the next line, then the IRQ command due at this line (oracle line rule).
    fn lines(state: &mut State, display: &mut DisplayFrame) {
        for line in (0..Line::VISIBLE).map(Line) {
            if let Some((due, command)) = state.irq
                && due == line
            {
                if let IrqCommand::GameplayHud = command.0 {
                    display.changes.push(LineChange {
                        line: Line(line.0 + 1),
                        change: Change::Screens(Default::default()),
                    });
                }
                state.irq = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    struct Projectiles;

    fn store() -> SlotStore<Projectiles, u16, u8, 18> {
        SlotStore {
            slots: core::array::from_fn(|_| Slot { common: 0, body: Occupancy::Free, generation: 0 }),
            policy: SlotPolicyOf(
                SlotPolicy {
                    spawn: SpawnOrder::Descending,
                    run: RunOrder::LiveDescending,
                    delete: DeleteRule::Immediate,
                },
                PhantomData,
            ),
        }
    }

    /// A projectile spawned during the descending pass into a lower slot runs in
    /// the same pass; a full table refuses the spawn (an original limit).
    #[test]
    fn a_live_descending_pass_runs_spawns_below_the_cursor() {
        let mut store = store();
        assert!(matches!(store.spawn(1, |_| ()), SpawnResult::Spawned(SlotIndexEq(17, _))));
        let mut pass = store.pass();
        let mut ran = Vec::new();
        while let Some(index) = pass.next(&store) {
            ran.push(index);
            if index == 17 {
                store.spawn(2, |common| *common = 7);
            }
        }
        assert_eq!(ran, [17, 16]);
        for _ in 0..16 {
            store.spawn(3, |_| ());
        }
        assert!(matches!(store.spawn(4, |_| ()), SpawnResult::Full));
    }

    #[test]
    fn a_requested_nmi_latches_this_ticks_pad() {
        let content = Arc::new(Content {
            reference: crate::sm_data::profile::ReferenceProfile {
                cadence: (21_477_272, 357_368),
                lag: crate::sm_data::profile::LagModel::Structural,
            },
        });
        let mut state = State::default();
        state.main.frames.push(Frame::MainLoop(MainLoopAt::Top));
        let checkpoint = Checkpoint { fingerprint: Fingerprint([0; 32]), state };
        let start = Start::Checkpoint(Box::new(checkpoint));
        let mut session = Session::open(content, start).expect("open");
        let output = session.step(TickInput { pad: Buttons(0x80) }).expect("step");
        assert_eq!(output.nmi, NmiKind::Requested);
        assert_eq!(session.state().controller.held, Buttons(0x80));
        assert_eq!(session.state().counters.nmi, 1);
    }
}
