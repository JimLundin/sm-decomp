//! The flashback to Zebes (`BattleBegins`, `PageTwoBegins`): the press after page 1 asks for the
//! encounter and starts the text dissolving into the fight; once the encounter signals its
//! return, page 2 begins and the fight starts dissolving back to the text. The dissolves
//! themselves are `dissolve`'s (`BattleDissolves`); page 2's end is a page's signal
//! (`PageTwoEnds`).

use core::num::Wrapping;

use types::intro::{FightVisibility, IntroStage, IntroState, PageScript};
use types::{Pair, Px};

use crate::content::{Dissolve, IntroContent};
use crate::dissolve::dissolve_begins;
use crate::pages::begin_page;
use crate::report::{FlashbackRequest, IntroEvent, PictureLayout, Report, TextTiles};

/// `BattleBegins`: the caret's battle position, px.
const BATTLE_CARET_PLACE: Pair<Px> = Pair {
    x: Wrapping(8),
    y: Wrapping(248),
};

/// The encounter's signal: whether the fight is ready to dissolve back to the text, its
/// explosions around Mother Brain having held for their closing interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FightReply {
    /// Not yet: the fight goes on.
    Fighting,
    /// The fight is ready to return to the text.
    ReadyToReturn,
}

/// `BattleBegins`: the press after page 1 asks for the encounter, shows Samus and her shots in
/// front of the fight's sprites, moves the caret to its battle position without restarting its
/// script and asks for the split blend; the dissolve into the fight begins, the colours that brighten
/// for it black.
pub fn battle_begins(state: &mut IntroState, content: &IntroContent, report: &mut Report) {
    state.stage = IntroStage::IntoFight;
    report
        .events
        .push(IntroEvent::Flashback(FlashbackRequest::StartZebesFight));
    state.fight = FightVisibility::InFront;
    if let Some(caret) = &mut state.caret {
        caret.place = BATTLE_CARET_PLACE;
    }
    report
        .events
        .push(IntroEvent::Layout(PictureLayout::SplitBlend));
    let black = &content.sets[Dissolve::IntoFight].brightens;
    dissolve_begins(state, content, black, report);
}

/// `BattleDissolves`: a hide of Samus and her shots asked for in this frame's stage step takes
/// effect after room play, so they take their last room step still shown.
pub fn hide_takes_effect(state: &mut IntroState, report: &Report) {
    let hide = IntroEvent::Flashback(FlashbackRequest::HideFightSamusAndShots);
    if report.events.contains(&hide) {
        state.fight = FightVisibility::Hidden;
    }
}

/// `PageTwoBegins`: the encounter's signal, taken after room play. While the fight
/// shows, its return makes the stage `PageTwoPending` on this same frame and pauses the text's
/// tiles; without it the fight holds and the scripts run. In any other stage it is not read.
pub fn fight_signals(state: &mut IntroState, reply: FightReply, report: &mut Report) {
    if state.stage == IntroStage::Fight && reply == FightReply::ReadyToReturn {
        state.stage = IntroStage::PageTwoPending;
        report
            .events
            .push(IntroEvent::TextUpdates(TextTiles::Paused));
    }
}

/// `PageTwoBegins`: the frame after the encounter's return, page 2 is placed with the click turn
/// off and the caret at its home on its steady script; the split blend is asked for and the
/// text's tiles resume; the dissolve back to the text begins, the colours that brighten for it
/// black.
pub fn page_two_begins(state: &mut IntroState, content: &IntroContent, report: &mut Report) {
    state.stage = IntroStage::OutOfFight;
    begin_page(state, PageScript::Two, content);
    report.events.extend([
        IntroEvent::Layout(PictureLayout::SplitBlend),
        IntroEvent::TextUpdates(TextTiles::Resumed),
    ]);
    let black = &content.sets[Dissolve::OutOfFight].brightens;
    dissolve_begins(state, content, black, report);
}
