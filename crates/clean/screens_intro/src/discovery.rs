//! The discovery (`DiscoveryBegins`, `DiscoveryReturns`): the press after page 2 brings up the
//! room where Samus finds the baby and asks for the played discovery; once the egg cues the
//! return, page 3 begins and the discovery starts dissolving back to the text. The dissolves are
//! `dissolve`'s, with the battle's colour sets; the encounter's close as the dissolve ends is
//! `Crossing::closes_encounter`; page 3's end is a page's signal
//! (`DiscoveryAndDeliveryPagesEnd`).

use types::intro::{IntroStage, IntroState, PageScript, Pan};

use crate::content::IntroContent;
use crate::dissolve::Crossing;
use crate::passage::{text_gives_way, text_returns};
use crate::report::{DiscoveryRequest, IntroEvent, Report, ScenePicture};
use crate::scene::return_cued;

/// The discovery encounter's cue, `ReadyForPageThree`: whether the egg has hatched and the open
/// shell has held for its closing interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscoveryReply {
    /// Not yet: the discovery goes on.
    Playing,
    /// The discovery is ready to return to the text.
    ReadyForPageThree,
}

/// `DiscoveryBegins`: the press after page 2 selects the discovery room's picture, its pan 0 and
/// its height the prior one; the text gives way to it (`text_gives_way`) with the battle's
/// colour sets, the split blend asked for before the played discovery's start.
pub fn discovery_begins(state: &mut IntroState, content: &IntroContent, report: &mut Report) {
    report
        .events
        .push(IntroEvent::PictureSelected(ScenePicture::Discovery));
    state.pan = Pan::AT_REST;
    text_gives_way(state, Crossing::IntoDiscovery, content, report);
    report
        .events
        .push(IntroEvent::Discovery(DiscoveryRequest::StartDiscovery));
}

/// `DiscoveryReturns`: the encounter's cue, taken after room play and before the text's scripts.
/// While the discovery shows, it makes the stage `PageThreePending` on this same frame and pauses
/// the text's tiles; without it the discovery holds and the scripts run.
pub fn discovery_signals(state: &mut IntroState, reply: DiscoveryReply, report: &mut Report) {
    if reply == DiscoveryReply::ReadyForPageThree {
        return_cued(
            state,
            IntroStage::Discovery,
            IntroStage::PageThreePending,
            report,
        );
    }
}

/// `DiscoveryReturns`: the frame after the cue, page 3 returns over the discovery
/// (`text_returns`), with the battle's colour sets.
pub fn page_three_begins(state: &mut IntroState, content: &IntroContent, report: &mut Report) {
    text_returns(
        state,
        Crossing::OutOfDiscovery,
        PageScript::Three,
        content,
        report,
    );
}
