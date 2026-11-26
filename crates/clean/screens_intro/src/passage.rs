//! The passages between the text and a picture, each begun in one stage step. The text gives way
//! to a picture (`BattleBegins`, `DiscoveryBegins`, `DeliveryBegins`, `ExaminationBegins`): the
//! caret moves offscreen, its script kept, the split blend is asked for and the dissolve
//! begins. A page returns (`PageTwoBegins`, `DiscoveryReturns`, `PageFourBegins`): the page and
//! the caret are placed at their starts, the split blend is asked for, the text's tiles resume
//! and the dissolve back begins. Each passage supplies only its crossing and what it adds.

use core::num::Wrapping;

use types::intro::{IntroState, PageScript};
use types::{Pair, Px};

use crate::content::IntroContent;
use crate::dissolve::{Crossing, dissolve_begins};
use crate::pages::begin_page;
use crate::report::{IntroEvent, PictureLayout, Report, TextTiles};

/// `BattleBegins`, `DiscoveryBegins`, `DeliveryBegins`, `ExaminationBegins`: the caret's place
/// while a picture shows, offscreen, px.
const CARET_OFFSCREEN: Pair<Px> = Pair {
    x: Wrapping(8),
    y: Wrapping(248),
};

/// The text gives way to a picture: the stage runs `crossing`; the caret moves offscreen without
/// restarting its script; the split blend is asked for; every colour is restored to its full and
/// its step, the crossing's brightening colours black, the colours published, and the count
/// starts from 127.
pub fn text_gives_way(
    state: &mut IntroState,
    crossing: Crossing,
    content: &IntroContent,
    report: &mut Report,
) {
    state.stage = crossing.stage();
    if let Some(caret) = &mut state.caret {
        caret.place = CARET_OFFSCREEN;
    }
    report
        .events
        .push(IntroEvent::Layout(PictureLayout::SplitBlend));
    let black = &content.sets[crossing.sets()].brightens;
    dissolve_begins(state, content, black, report);
}

/// A page returns over a picture: the stage runs `crossing`; `page` is placed at its start with
/// the click turn off, and the caret at its home on its steady script; the split blend is asked
/// for and the text's tiles resume; every colour is restored to its full and its step, the
/// crossing's brightening colours black, the colours published, and the count starts from 127.
pub fn text_returns(
    state: &mut IntroState,
    crossing: Crossing,
    page: PageScript,
    content: &IntroContent,
    report: &mut Report,
) {
    state.stage = crossing.stage();
    begin_page(state, page, content);
    report.events.extend([
        IntroEvent::Layout(PictureLayout::SplitBlend),
        IntroEvent::TextUpdates(TextTiles::Resumed),
    ]);
    let black = &content.sets[crossing.sets()].brightens;
    dissolve_begins(state, content, black, report);
}
