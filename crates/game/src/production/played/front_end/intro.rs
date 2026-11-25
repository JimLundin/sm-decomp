//! The intro (screens-intro) from the frame end the options hand on to it at: its opening and
//! page 1, through the intro's own adapter.

use super::{Answers, Content, Fault, Screen, Sound, Stepped, Work};
use crate::production::played::intro::{self, FrameEnd, State};
use screens_intro::{IntroEvent, Rules};

/// One frame end of the intro. Each finished frame of work drew the random number as it began.
pub(super) fn step(
    mut state: State,
    work: Work,
    rules: Rules,
    content: &Content,
    answers: &impl Answers,
) -> Result<Stepped, Fault> {
    let end = match work {
        Work::Unfinished => FrameEnd::Lagged,
        Work::Finished(pressed) => FrameEnd::Finished {
            pressed: pressed.values().any(|&down| down),
            track: answers.track(),
        },
        Work::FinishedTwice(_) => return Err(Fault::TwoFinished),
    };
    let report = intro::played(&mut state, end, &content.intro, rules).map_err(Fault::Intro)?;
    let sounds = report.map_or_else(Vec::new, |report| {
        let clicks = report
            .events
            .iter()
            .filter(|&&event| event == IntroEvent::Click)
            .map(|_| Sound::IntroClick);
        report
            .music
            .iter()
            .map(|&request| Sound::IntroMusic(request))
            .chain(clicks)
            .collect()
    });
    let finished = work.finished();
    Ok(Stepped {
        screen: Screen::Intro(Box::new(state)),
        finished,
        drawn: finished,
        sounds,
    })
}
