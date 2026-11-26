//! The flashback to Zebes (`BattleBegins`, `PageTwoBegins`): the press after page 1 asks for the
//! encounter and starts the text dissolving into the fight; once the encounter signals its
//! return, page 2 begins and the fight starts dissolving back to the text. The passages are
//! `passage`'s, the dissolves themselves `dissolve`'s (`BattleDissolves`); page 2's end is a
//! page's signal (`PageTwoEnds`), and the press after it begins the discovery (`discovery`).

use types::intro::{FightHide, FightVisibility, IntroStage, IntroState, PageScript};

use crate::content::IntroContent;
use crate::dissolve::Crossing;
use crate::passage::{text_gives_way, text_returns};
use crate::report::{FlashbackRequest, IntroEvent, Report};
use crate::scene::return_cued;

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
    report
        .events
        .push(IntroEvent::Flashback(FlashbackRequest::StartZebesFight));
    state.fight = FightVisibility::InFront;
    text_gives_way(state, Crossing::IntoFight, content, report);
}

/// `BattleDissolves`: whether a stage step's report asks for Samus and her shots to be hidden,
/// the hide waiting for room play to have run (`FightHide::takes_effect`).
#[must_use]
pub fn hide_asked_in(stage_step: &Report) -> FightHide {
    let hide = IntroEvent::Flashback(FlashbackRequest::HideFightSamusAndShots);
    if stage_step.events.contains(&hide) {
        FightHide::Asked
    } else {
        FightHide::NotAsked
    }
}

/// `PageTwoBegins`: the encounter's signal, taken after room play. While the fight
/// shows, its return makes the stage `PageTwoPending` on this same frame and pauses the text's
/// tiles; without it the fight holds and the scripts run. In any other stage it is not read.
pub fn fight_signals(state: &mut IntroState, reply: FightReply, report: &mut Report) {
    if reply == FightReply::ReadyToReturn {
        return_cued(state, IntroStage::Fight, IntroStage::PageTwoPending, report);
    }
}

/// `PageTwoBegins`: the frame after the encounter's return, page 2 is placed with the click turn
/// off and the caret at its home on its steady script; the split blend is asked for and the
/// text's tiles resume; the dissolve back to the text begins, the colours that brighten for it
/// black.
pub fn page_two_begins(state: &mut IntroState, content: &IntroContent, report: &mut Report) {
    text_returns(
        state,
        Crossing::OutOfFight,
        PageScript::Two,
        content,
        report,
    );
}
