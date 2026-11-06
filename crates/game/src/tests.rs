//! The Tick skeleton's proofs: the counters under a recorded schedule, the latch, the strand's
//! resume order and a Checkpoint taken mid-run.

use std::marker::PhantomData;

use canonical::{from_bytes, to_bytes};

use crate::nmi::NmiKind;
use crate::session::{AutoRepeat, Rules, Schedule, Session, SessionFault, TickInput, TimingSource};
use crate::state::slots::{FlagSet, Frames, SlotIndex};
use crate::state::strand::{Frame, GameplayAt, MainLoopAt, MessageBoxAt, Wait};
use crate::state::{
    Buttons, Checkpoint, Counters, Fingerprint, FrameCount, NmiTiming, State, Tick,
};
use crate::tick::{self, Next};

const RULES: Rules = Rules {
    auto_repeat: AutoRepeat {
        initial: Frames(3),
        subsequent: Frames(2),
    },
};

fn session(state: State) -> Result<Session, String> {
    Session::open(state, RULES).map_err(|error| format!("{error:?}"))
}

fn ticks(numbers: &[u64]) -> Vec<Tick> {
    numbers.iter().copied().map(Tick).collect()
}

/// Run `count` Ticks with an empty pad under `source`, returning each NMI's path.
fn run(
    session: &mut Session,
    source: &TimingSource,
    count: u64,
) -> Result<Vec<NmiKind>, SessionFault> {
    (0..count)
        .map(|_| {
            let timing = source.at(session.state().clock.tick);
            session
                .step(TickInput {
                    pad: Buttons::default(),
                    timing,
                })
                .map(|output| output.nmi)
        })
        .collect()
}

const fn counters(nmi: u16, completed: u16, lag: u16, max_lag: u16) -> Counters {
    Counters {
        nmi: FrameCount(nmi),
        completed: FrameCount(completed),
        lag: FrameCount(lag),
        max_lag: FrameCount(max_lag),
    }
}

/// Worked by hand from the bank `$80` NMI handler's counter rules. Ticks 0-2 commit. Tick 3's
/// iteration finishes but its NMI overruns, so it lags with the request pending; Tick 4's NMI
/// overruns too, and the unanswered strand doesn't run. Tick 5's on-time NMI commits the
/// iteration Tick 3 ran, and Tick 6 runs and commits a new one. Tick 7 runs an iteration but its
/// vblank has NMIs off, so no counter moves; Tick 8's strand stays waiting and its NMI commits
/// Tick 7's iteration.
#[test]
fn recorded_schedule_counts_as_the_nmi_handler_does() -> Result<(), String> {
    use NmiKind::{Committed as C, Disabled as D, Lag as L};
    let mut session = session(State::default())?;
    let source = TimingSource::Recorded(Schedule::new(ticks(&[4, 3]), ticks(&[7])));
    let kinds = run(&mut session, &source, 9).map_err(|fault| format!("{fault:?}"))?;
    assert_eq!(kinds, [C, C, C, L, L, C, C, D, C]);
    // Nine Ticks, eight NMIs; six requested; the two-Tick lag run is the longest.
    assert_eq!(session.state().clock.counters, counters(8, 6, 0, 2));
    assert_eq!(session.state().clock.tick, Tick(9));
    assert_eq!(session.state().main.waiting, Some(Wait::Nmi));
    Ok(())
}

#[test]
fn an_unanswered_strand_does_not_run() -> Result<(), String> {
    let mut session = session(State::default())?;
    let source = TimingSource::Recorded(Schedule::new(ticks(&[1, 2]), Vec::new()));
    run(&mut session, &source, 2).map_err(|fault| format!("{fault:?}"))?;
    // Tick 1 ran an iteration and overran; Tick 2 must not run another.
    let before = session.state().clone();
    run(&mut session, &source, 1).map_err(|fault| format!("{fault:?}"))?;
    assert_eq!(session.state().main, before.main);
    assert!(session.state().video.nmi_requested);
    assert_eq!(session.state().timing.last, NmiTiming::Overrun);
    Ok(())
}

#[test]
fn the_lag_counter_is_a_byte() -> Result<(), String> {
    let mut state = State::default();
    state.clock.counters = counters(0, 0, 255, 255);
    let mut session = session(state)?;
    let source = TimingSource::Recorded(Schedule::new(ticks(&[0]), Vec::new()));
    run(&mut session, &source, 1).map_err(|fault| format!("{fault:?}"))?;
    assert_eq!(session.state().clock.counters, counters(1, 0, 0, 255));
    Ok(())
}

fn buttons(bits: u32) -> Buttons {
    FlagSet {
        bits,
        members: PhantomData,
    }
}

#[test]
fn a_requested_nmi_latches_the_pad_with_auto_repeat() -> Result<(), String> {
    let mut session = session(State::default())?;
    let held = buttons(0b1001);
    let mut seen = Vec::new();
    for _ in 0..6 {
        session
            .step(TickInput {
                pad: held,
                timing: NmiTiming::OnTime,
            })
            .map_err(|fault| format!("{fault:?}"))?;
        let controller = session.state().controller;
        assert_eq!((controller.held, controller.previous), (held, held));
        assert_eq!(controller.auto_read, held);
        seen.push((
            controller.pressed.bits,
            controller.repeat.bits,
            controller.repeat_delay,
        ));
    }
    // Pressed once; the timer reloads with 3, counts 2, 1, repeats at 0 and reloads with 2.
    assert_eq!(
        seen,
        [
            (0b1001, 0b1001, Frames(3)),
            (0, 0, Frames(2)),
            (0, 0, Frames(1)),
            (0, 0b1001, Frames(2)),
            (0, 0, Frames(1)),
            (0, 0b1001, Frames(2)),
        ]
    );
    Ok(())
}

#[test]
fn a_lag_nmi_auto_reads_but_does_not_latch() -> Result<(), String> {
    let mut session = session(State::default())?;
    let pad = buttons(0b10);
    session
        .step(TickInput {
            pad,
            timing: NmiTiming::Overrun,
        })
        .map_err(|fault| format!("{fault:?}"))?;
    let controller = session.state().controller;
    assert_eq!(controller.auto_read, pad);
    assert!(controller.held.is_empty() && controller.pressed.is_empty());
    Ok(())
}

#[test]
fn the_main_loop_resumes_in_its_order() -> Result<(), String> {
    let mut state = State::default();
    let main_loop = |at| Frame::MainLoop(at);
    let steps = (0..5)
        .map(|_| tick::step_once(&mut state, RULES).map(|next| (next, state.main.frames.clone())))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|fault| format!("{fault:?}"))?;
    assert_eq!(
        steps,
        [
            (Next::Sync, vec![main_loop(MainLoopAt::AfterHdmaObjects)]),
            (
                Next::Call(Frame::Gameplay(GameplayAt::Start)),
                vec![
                    main_loop(MainLoopAt::AfterGameState),
                    Frame::Gameplay(GameplayAt::Start),
                ]
            ),
            (
                Next::Continue,
                vec![
                    main_loop(MainLoopAt::AfterGameState),
                    Frame::Gameplay(GameplayAt::AfterPlms),
                ]
            ),
            (Next::Return, vec![main_loop(MainLoopAt::AfterGameState)]),
            (Next::Yield(Wait::Nmi), vec![main_loop(MainLoopAt::Start)]),
        ]
    );
    assert_eq!(state.main.waiting, Some(Wait::Nmi));
    assert!(state.video.nmi_requested);
    Ok(())
}

#[test]
fn a_nested_strand_resumes_innermost_first_after_the_scan_out() -> Result<(), String> {
    let mut state = State::default();
    state.main.frames = vec![
        Frame::MainLoop(MainLoopAt::AfterGameState),
        Frame::Gameplay(GameplayAt::AfterPlms),
        Frame::Plms {
            slot: SlotIndex(3, PhantomData),
        },
        Frame::MessageBox(MessageBoxAt::Shown),
    ];
    state.main.waiting = Some(Wait::LagFrame);
    let mut session = session(state)?;
    let begun = session.begin().map_err(|fault| format!("{fault:?}"))?;
    // The message box's beam-sync point is before its work: nothing ran before the scan-out.
    let output = begun
        .end(TickInput::default())
        .map_err(|fault| format!("{fault:?}"))?;
    assert_eq!(output.nmi, NmiKind::Committed);
    assert_eq!(
        session.state().main.frames,
        [Frame::MainLoop(MainLoopAt::Start)]
    );
    Ok(())
}

#[test]
fn begin_stops_at_the_main_loops_beam_sync_point() -> Result<(), String> {
    let mut session = session(State::default())?;
    session.begin().map_err(|fault| format!("{fault:?}"))?;
    assert_eq!(
        session.state().main.frames,
        [Frame::MainLoop(MainLoopAt::AfterHdmaObjects)]
    );
    // The dropped Tick never ended, so the Session refuses to start another.
    assert_eq!(session.begin().err(), Some(SessionFault::TickAbandoned));
    assert_eq!(
        session.step(TickInput::default()).err(),
        Some(SessionFault::TickAbandoned)
    );
    Ok(())
}

#[test]
fn a_checkpoint_mid_run_round_trips_and_continues_alike() -> Result<(), String> {
    let source = TimingSource::Recorded(Schedule::new(ticks(&[3, 4, 9]), ticks(&[6])));
    let mut original = session(State::default())?;
    run(&mut original, &source, 5).map_err(|fault| format!("{fault:?}"))?;
    let checkpoint = Checkpoint::new(Fingerprint(7), original.state().clone());
    let restored =
        from_bytes::<Checkpoint>(&to_bytes(&checkpoint)).map_err(|error| format!("{error:?}"))?;
    assert_eq!(restored, checkpoint);
    let mut resumed = session(restored.state)?;
    let ahead = run(&mut original, &source, 8).map_err(|fault| format!("{fault:?}"))?;
    let again = run(&mut resumed, &source, 8).map_err(|fault| format!("{fault:?}"))?;
    assert_eq!(ahead, again);
    assert_eq!(original.state(), resumed.state());
    Ok(())
}

#[test]
fn an_empty_strand_does_not_open() {
    let mut state = State::default();
    state.main.frames.clear();
    assert!(Session::open(state, RULES).is_err());
}

/// One main-loop iteration with nothing drawn: the RNG steps once (0 becomes 0111h, bank `$80`
/// log `$80:8111`), the game time counts a frame, and the committed OAM has every sprite
/// hidden with the high table cleared.
#[test]
fn an_iteration_steps_the_rng_and_the_clock_and_commits_an_empty_oam() -> Result<(), String> {
    let mut state = State::default();
    state.video.batch.oam.high = [0xFF; _];
    let mut session = session(state)?;
    run(&mut session, &TimingSource::Modeled, 1).map_err(|fault| format!("{fault:?}"))?;
    let after = session.state();
    assert_eq!((after.rng.word, after.rng.calls), (0x0111, 1));
    assert_eq!(after.clock.game_time.frames, 1);
    let oam = &after.video.memory.oam;
    assert!(oam.low.iter().all(|entry| entry.y == crate::oam::HIDDEN_Y));
    assert_eq!(oam.high, [0; _]);
    assert_eq!(after.video.batch.sprites.0, 0);
    Ok(())
}
