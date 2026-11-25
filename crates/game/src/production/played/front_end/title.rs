//! The title (screens-title), from the game's settling until file select begins.

use super::{Fault, Screen, Sound, Stepped, Work};
use screens_title::{FrameEnd, Rules};
use types::title::TitleState;

/// One frame end of the title. Each finished frame of work drew the random number as it began.
pub(super) fn step(title: TitleState, work: Work, rules: Rules) -> Result<Stepped, Fault> {
    let end = match work {
        Work::Unfinished => FrameEnd::NoneFinished,
        Work::Finished(pressed) => FrameEnd::OneFinished(pressed),
        Work::FinishedTwice(pressed) => FrameEnd::TwoFinished(pressed),
    };
    let report = screens_title::end_frame(title, rules, &end).map_err(|_| Fault::TitleLeft)?;
    let finished = work.finished();
    Ok(Stepped {
        screen: Screen::Title(report.state),
        finished,
        drawn: finished,
        sounds: report.tunes.into_iter().map(Sound::Title).collect(),
    })
}
