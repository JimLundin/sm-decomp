//! The options menu (screens-options), until it hands on.

use super::{Button, Fault, Options, Pressed, Screen, Sound, Stepped, Work};
use crate::production::played::intro;
use screens_options::{FrameEnd, MenuAction, NextPiece};
use types::options::{Language, WayOut};

/// One frame end of the options menu. Each finished frame of work drew as it began.
///
/// Once the menu has handed on to the intro, the next frame end is the hand-on's: the game
/// turns to the intro, whose first frame is the one after (screens-intro `IntroBegins`).
pub(super) fn step(options: Options, work: Work) -> Result<Stepped, Fault> {
    let end = match work {
        Work::Unfinished => FrameEnd::Busy,
        Work::Finished(pressed) => FrameEnd::Finished(actions(&pressed)),
        Work::FinishedTwice(_) => return Err(Fault::TwoFinished),
    };
    let mut menu = options.menu;
    let finished = work.finished();
    let sounds = match screens_options::step(&mut menu, end, options.begins) {
        Ok(sounds) => sounds,
        Err(NextPiece(WayOut::IntroBegins)) => {
            let language = match menu.settings.language {
                Language::English => screens_intro::Language::English,
                Language::Japanese => screens_intro::Language::Japanese,
            };
            let begun = intro::begins(language).map_err(|_| Fault::IntroLanguage)?;
            return Ok(Stepped {
                screen: Screen::Intro(Box::new(super::PlayingIntro::begin(
                    begun,
                    options.actor,
                    menu.settings,
                ))),
                finished,
                drawn: finished,
                sounds: Vec::new(),
            });
        }
        Err(NextPiece(way)) => return Err(Fault::HandedOn(way)),
    };
    Ok(Stepped {
        screen: Screen::Options(Options { menu, ..options }),
        finished,
        drawn: finished,
        sounds: sounds.into_iter().map(Sound::Options).collect(),
    })
}

/// What the buttons just pressed ask of the menu.
fn actions(pressed: &Pressed) -> screens_options::Pressed {
    pressed
        .iter()
        .filter(|&(_, &down)| down)
        .filter_map(|(button, _)| action(button))
        .map(|action| (action, true))
        .collect()
}

/// The menu's binding: A and Start choose, B goes back, Left and Right go sideways.
const fn action(button: Button) -> Option<MenuAction> {
    match button {
        Button::Up => Some(MenuAction::Up),
        Button::Down => Some(MenuAction::Down),
        Button::B => Some(MenuAction::Back),
        Button::A | Button::Start => Some(MenuAction::Choose),
        Button::Left | Button::Right => Some(MenuAction::Sideways),
        Button::X | Button::Y | Button::L | Button::R | Button::Select => None,
    }
}
