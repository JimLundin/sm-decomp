//! The power-on (screens-boot), until the game settles and the title begins.

use super::{Answers, Content, Fault, FrameEnd, Rules, Screens, Sound, State, Work};
use screens_boot::{FrameWork, Outcome, PowerOnState};

/// One frame end of the power-on. The first frame that finishes once the game has settled is
/// the title's, which then plays it.
pub(super) fn step(
    power_on: PowerOnState,
    end: FrameEnd,
    rules: &Rules,
    content: &Content,
    answers: &impl Answers,
) -> Result<(State, Vec<Sound>), Fault> {
    let work = match end.work {
        Work::Unfinished => FrameWork::Unfinished,
        Work::Finished(_) => FrameWork::Finished,
        Work::FinishedTwice(_) => return Err(Fault::TwoFinished),
    };
    let boot_end = screens_boot::FrameEnd {
        work,
        upload: end.upload,
    };
    match screens_boot::step(rules.power_on, power_on, boot_end) {
        Outcome::PowerOn(next) => Ok((State::PowerOn(next), Vec::new())),
        Outcome::TitleBegins => {
            let (screens, sounds) =
                Screens::settled(power_on, answers).step(end.work, rules, content, answers)?;
            Ok((State::Screens(screens), sounds))
        }
    }
}
