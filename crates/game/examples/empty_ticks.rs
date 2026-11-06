//! Empty Ticks per second: the Session's fixed cost with no gameplay, against SLICE0's target
//! of at least 50 000. Run with `cargo run --release -p game --example empty_ticks`.

use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;

use game::session::{AutoRepeat, Rules, Schedule, Session, TickInput, TimingSource};
use game::state::slots::Frames;
use game::state::{State, Tick};

const TICKS: u64 = 2_000_000;

fn main() -> ExitCode {
    let rules = Rules {
        auto_repeat: AutoRepeat {
            initial: Frames(12),
            subsequent: Frames(2),
        },
    };
    let Ok(mut session) = Session::open(State::default(), rules, std::sync::Arc::default()) else {
        return ExitCode::FAILURE;
    };
    // One overrun in every hundred Ticks, so the lag path is in the measurement.
    let overruns = (0..TICKS).step_by(100).map(Tick).collect();
    let source = TimingSource::Recorded(Schedule::new(overruns, Vec::new()));
    let started = Instant::now();
    for _ in 0..TICKS {
        let timing = source.at(session.state().clock.tick);
        let input = TickInput {
            timing,
            ..TickInput::default()
        };
        if session.step(input).is_err() {
            return ExitCode::FAILURE;
        }
    }
    let nanos = started.elapsed().as_nanos().max(1);
    let per_second = u128::from(TICKS)
        .checked_mul(1_000_000_000)
        .and_then(|scaled| scaled.checked_div(nanos))
        .unwrap_or(u128::MAX);
    let report = writeln!(
        io::stdout().lock(),
        "{TICKS} empty Ticks in {nanos} ns: {per_second} Ticks/s; counters {:?}",
        session.state().clock.counters
    );
    if report.is_err() {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
