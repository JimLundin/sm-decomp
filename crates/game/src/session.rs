//! The Session (D6): the State and the rules it runs under, stepped one Tick at a time in the
//! console's order.
//!
//! A Tick runs from one commit cut (an NMI's return) to the next: the main strand resumes and
//! runs to its beam-sync point, the previous picture scans out, the main strand runs on until
//! it yields, and the NMI commits or lags. [`Session::begin`] and [`Begun::end`] split the Tick
//! at the scan-out so a player sees the picture before the pad is needed; [`Session::step`] is
//! both.

use crate::lines::{self, Picture};
use std::sync::Arc;

use crate::nmi::{self, NmiKind};
use crate::plm::ScrollTouchFault;
pub use crate::pose::Content;
use crate::state::slots::Frames;
use crate::state::strand::MAIN_STRAND_DEPTH;
use crate::state::{Buttons, NmiTiming, State, Tick};
use crate::tick::{self, STEP_BUDGET, Until};

/// The controller's auto-repeat delays, in frames.
///
/// The original keeps them in two work-RAM words that boot sets and nothing in slice 0's rows
/// changes, so they are rules here.
// open: if a row shows either word change, they move into `Controller` with their decode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AutoRepeat {
    /// Frames a pad must stay held before its first repeat.
    pub initial: Frames,
    /// Frames between later repeats.
    pub subsequent: Frames,
}

/// The rules a Session runs under, resolved from the profiles at open (D19).
// open: `GameplayProfile`'s sections join with the domains that read them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// The pad's auto-repeat.
    pub auto_repeat: AutoRepeat,
}

/// A recorded NMI schedule: the Ticks whose NMI arrived during the main strand's work, and
/// those whose vblank had NMIs disabled; every other Tick's NMI arrived on time.
///
/// A lock row's `overruns` and `nmi_off` frames, counted from the row's first, are its
/// Ticks counted from the Session's open at the row's seed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Schedule {
    overruns: Vec<Tick>,
    nmi_off: Vec<Tick>,
}

impl Schedule {
    /// The schedule with these overrun and NMI-off Ticks, in any order.
    #[must_use]
    pub fn new(mut overruns: Vec<Tick>, mut nmi_off: Vec<Tick>) -> Self {
        overruns.sort_unstable();
        overruns.dedup();
        nmi_off.sort_unstable();
        nmi_off.dedup();
        Self { overruns, nmi_off }
    }

    /// How Tick `tick`'s NMI arrived.
    #[must_use]
    pub fn at(&self, tick: Tick) -> NmiTiming {
        if self.nmi_off.binary_search(&tick).is_ok() {
            NmiTiming::Disabled
        } else if self.overruns.binary_search(&tick).is_ok() {
            NmiTiming::Overrun
        } else {
            NmiTiming::OnTime
        }
    }
}

/// Where a Tick's NMI timing comes from (D8). The app or tool holds it, not the Session: it
/// is an input, recorded in the replay.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TimingSource {
    /// The oracle's own schedule, from a lock row: the gate's source.
    Recorded(Schedule),
    /// The cost model's schedule (D9).
    // open: D9's cost table; until it exists every NMI is on time.
    Modeled,
}

impl TimingSource {
    /// How Tick `tick`'s NMI arrives.
    #[must_use]
    pub fn at(&self, tick: Tick) -> NmiTiming {
        match self {
            Self::Recorded(schedule) => schedule.at(tick),
            Self::Modeled => NmiTiming::OnTime,
        }
    }
}

/// What a Tick needs from outside: the pad as auto-read at this Tick's NMI and how that NMI
/// arrived.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TickInput {
    /// The pad at this Tick's vblank.
    pub pad: Buttons,
    /// How this Tick's NMI arrived, from the [`TimingSource`].
    pub timing: NmiTiming,
}

/// What a Tick produced, at its cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TickOutput {
    /// The Tick that ran, counted from the Session's open.
    pub tick: Tick,
    /// The path its NMI took.
    pub nmi: NmiKind,
    /// The previous picture, scanned out during this Tick.
    pub picture: Picture,
}

/// An engine fault: an invariant of the Session broke. The Session is poisoned and every
/// later Tick returns the same fault.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionFault {
    /// The main strand has no frame to run.
    StrandEmpty,
    /// A call would nest the main strand deeper than [`MAIN_STRAND_DEPTH`].
    StrandTooDeep,
    /// The main strand took more than [`STEP_BUDGET`] steps without yielding.
    StepBudgetExceeded,
    /// A Tick was begun and dropped before it ended.
    TickAbandoned,
    /// A scroll trigger's touch PLM found no scroll PLM on its block, where the original
    /// hangs.
    Plm(ScrollTouchFault),
}

/// Why a State can't open as a Session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenError {
    /// The main strand has no frame.
    StrandEmpty,
    /// The main strand nests deeper than [`MAIN_STRAND_DEPTH`].
    StrandTooDeep,
}

/// A running game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    state: State,
    rules: Rules,
    /// The decoded tables the rules read, shared between Sessions.
    content: Arc<Content>,
    /// Set by a fault; the Session runs no further Tick.
    fault: Option<SessionFault>,
    /// Between `begin` and `end`.
    begun: bool,
}

impl Session {
    /// A Session at `state`, which sits at a cut, under `rules`, reading `content`.
    ///
    /// # Errors
    ///
    /// An [`OpenError`] when the State's main strand is empty or too deep.
    pub fn open(state: State, rules: Rules, content: Arc<Content>) -> Result<Self, OpenError> {
        if state.main.frames.is_empty() {
            return Err(OpenError::StrandEmpty);
        }
        if state.main.frames.len() > MAIN_STRAND_DEPTH {
            return Err(OpenError::StrandTooDeep);
        }
        Ok(Self {
            state,
            rules,
            content,
            fault: None,
            begun: false,
        })
    }

    /// The State at the last cut.
    #[must_use]
    pub const fn state(&self) -> &State {
        &self.state
    }

    /// The rules.
    #[must_use]
    pub const fn rules(&self) -> &Rules {
        &self.rules
    }

    /// Run one Tick.
    ///
    /// # Errors
    ///
    /// The [`SessionFault`] that poisoned the Session, now or earlier.
    pub fn step(&mut self, input: TickInput) -> Result<TickOutput, SessionFault> {
        self.begin()?.end(input)
    }

    /// Run the Tick up to the scan-out of the previous picture (D6 steps 1 and 2).
    ///
    /// # Errors
    ///
    /// The [`SessionFault`] that poisoned the Session, now or earlier.
    pub fn begin(&mut self) -> Result<Begun<'_>, SessionFault> {
        if let Some(fault) = self.fault {
            return Err(fault);
        }
        if self.begun {
            return Err(self.poison(SessionFault::TickAbandoned));
        }
        self.begun = true;
        let mut budget = STEP_BUDGET;
        if tick::resume(&mut self.state) {
            let ran = tick::run(
                &mut self.state,
                self.rules,
                &self.content,
                Until::BeamSync,
                &mut budget,
            );
            if let Err(fault) = ran {
                return Err(self.poison(fault));
            }
        }
        let picture = lines::scan_out(&mut self.state);
        Ok(Begun {
            session: self,
            picture,
            budget,
        })
    }

    const fn poison(&mut self, fault: SessionFault) -> SessionFault {
        self.fault = Some(fault);
        fault
    }
}

/// A Tick past its scan-out. It borrows the Session, so no Checkpoint is taken mid-Tick (K4).
#[derive(Debug)]
pub struct Begun<'a> {
    session: &'a mut Session,
    picture: Picture,
    budget: u16,
}

impl Begun<'_> {
    /// The previous picture.
    #[must_use]
    pub const fn picture(&self) -> &Picture {
        &self.picture
    }

    /// Finish the Tick (D6 steps 3 to 5): the main strand runs until it yields, then the NMI.
    ///
    /// # Errors
    ///
    /// The [`SessionFault`] the main strand raised, which poisons the Session.
    pub fn end(mut self, input: TickInput) -> Result<TickOutput, SessionFault> {
        let session = &mut *self.session;
        let ran = tick::run(
            &mut session.state,
            session.rules,
            &session.content,
            Until::Yield,
            &mut self.budget,
        );
        if let Err(fault) = ran {
            return Err(session.poison(fault));
        }
        let kind = nmi::nmi(
            &mut session.state,
            session.rules,
            &session.content,
            input.timing,
            input.pad,
        );
        let clock = &mut session.state.clock;
        let tick = clock.tick;
        clock.tick = Tick(tick.0.wrapping_add(1));
        session.begun = false;
        Ok(TickOutput {
            tick,
            nmi: kind,
            picture: self.picture,
        })
    }
}
