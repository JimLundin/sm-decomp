//! One frame of the intro, in the order its parts read each other's results: first the stage's
//! step (the opening's, page 1's, a passage's, a dissolve's, a page's turn or the fade's),
//! then, after room play, the hide of Samus and her shots and the encounters' cues, then the
//! baby, the caret's script, the page's script, the eyes' script and the glows. So a script
//! placed by the stage's step runs its first step in the same frame, while one placed by a
//! script's signal (the caret's blinking script) runs from the next; a baby's signal for page 4
//! or 5 takes effect at the next frame's stage step; and an encounter's return cue makes the
//! stage `PageTwoPending` or `PageThreePending` in the frame it comes, the page beginning at the
//! next frame's stage step and its dissolve counting down from the frame after.
//!
//! A finished frame comes in two parts, with room play between them, each run exactly once and
//! in this order:
//! 1. `begin_frame` runs the stage's step and hands back a `FrameAwaitingReply`. From it the
//!    caller reads the encounters' requests (`FrameAwaitingReply::requests`,
//!    `FrameAwaitingReply::discovery_requests`): a start (`BattleBegins`, `DiscoveryBegins`),
//!    applied before room play, and a close (`BattleDissolves`, `DiscoveryReturns`), applied
//!    after room play and before drawing.
//! 2. The caller plays the room. Samus and her shots are still shown in the intro's state, so
//!    they take their last room step before the hide takes effect on the picture.
//! 3. `FrameAwaitingReply::finish` consumes the encounters' replies and completes the frame: the
//!    hide takes effect, the cues are read where `PageTwoBegins` and `DiscoveryReturns` read
//!    them, and the scripts run.
//!
//! A cut may fall between the parts, once the start is applied and before the room's turn
//! (`DiscoveryBegins`): `FrameAwaitingReply::interrupt` hands back what the stage's step
//! reported and a `RoomTurnAwaited`, which the caller holds with the state; `resume` takes the
//! room's turn and the scripts' exactly once, from the holds the stage's step left, and reports
//! only what they do. To save the cut, the caller stores the turn's `checkpoint`
//! (`types::intro::RoomTurnCheckpoint`) with the state, and later `restore`s the turn from it.
//! `step` is the two parts run back to back, for a caller that already has
//! the replies.

use types::intro::{Baby, Caret, IntroStage, IntroState, RoomTurnCheckpoint, ScriptPlace};
use types::{Pair, Px};

use crate::baby::{baby_crosses_the_picture, baby_leaves, baby_rises};
use crate::content::IntroContent;
use crate::delivery::{delivery_begins, examination_begins, page_four_begins};
use crate::discovery::{DiscoveryReply, discovery_begins, discovery_signals, page_three_begins};
use crate::dissolve::{Crossing, dissolve_frame};
use crate::fade::{intro_ends, picture_darkens};
use crate::flashback::{FightReply, battle_begins, fight_signals, hide_asked_in, page_two_begins};
use crate::music::TrackReply;
use crate::opening::{
    first_tune, intro_begins, opening_brightens, opening_darkens, opening_loads, opening_pause,
    second_tune, second_tune_loads,
};
use crate::page_one::{page_one_brightens, page_one_is_made_ready, page_one_loads};
use crate::pages::{
    letter_is_typed, letters_glow, page_five_begins, press_turns_the_page, take_signal,
};
use crate::report::{DiscoveryRequest, FlashbackRequest, IntroEvent, Report, ShownPicture};
use crate::rules::Rules;
use crate::script::{PictureName, Ran, Shows, Step, run, showing};

/// What the intro reads each frame besides its content and rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameInputs {
    /// Whether any button was pressed anew this frame (the movie's pad).
    pub pressed: bool,
    /// Audio's reply: whether a tune asked for still waits to load and start.
    pub track: TrackReply,
    /// The flashback encounter's signal, received after room play: whether the fight is ready
    /// to return to the text.
    pub fight: FightReply,
    /// The discovery encounter's cue, received after room play: whether the discovery is ready
    /// to return to the text.
    pub discovery: DiscoveryReply,
}

/// One whole frame of the intro, for a caller that already has the encounters' replies: the
/// same two parts as `begin_frame` and `FrameAwaitingReply::finish`, run back to back.
pub fn step(
    state: &mut IntroState,
    inputs: FrameInputs,
    content: &IntroContent,
    rules: Rules,
) -> Report {
    begin_frame(state, inputs.pressed, inputs.track, content, rules)
        .finish(inputs.fight, inputs.discovery)
}

/// The frame's first part: the stage's step, its requests handed back before room play.
pub fn begin_frame<'frame>(
    state: &'frame mut IntroState,
    pressed: bool,
    track: TrackReply,
    content: &'frame IntroContent,
    rules: Rules,
) -> FrameAwaitingReply<'frame> {
    let mut report = Report::empty();
    stage_step(state, pressed, track, content, rules, &mut report);
    FrameAwaitingReply {
        state,
        content,
        rules,
        report,
    }
}

/// A frame whose stage has stepped, waiting for the encounters' replies from room play before
/// its scripts run. It holds the intro's state until it is finished or interrupted.
#[must_use = "a frame ends only when it is finished with the encounters' replies"]
#[derive(Debug)]
pub struct FrameAwaitingReply<'frame> {
    /// The intro's state, its stage stepped.
    state: &'frame mut IntroState,
    /// The scene's content.
    content: &'frame IntroContent,
    /// The rules.
    rules: Rules,
    /// What the frame has reported so far.
    report: Report,
}

impl FrameAwaitingReply<'_> {
    /// The intro's state after the stage's step, as room play sees it: a hide asked for this
    /// frame has not yet taken effect.
    #[must_use]
    pub const fn state(&self) -> &IntroState {
        self.state
    }

    /// The flashback's requests the stage's step made, in order: the encounter's start, for
    /// production to apply before room play, and the hiding of Samus and her shots, to apply
    /// after room play and before drawing.
    pub fn requests(&self) -> impl Iterator<Item = FlashbackRequest> {
        self.report.events.iter().filter_map(|event| {
            if let IntroEvent::Flashback(request) = event {
                Some(*request)
            } else {
                None
            }
        })
    }

    /// The discovery's requests the stage's step made, in order: its start, for production to
    /// apply before room play, and its finish, to apply after Samus's last room step and before
    /// drawing.
    pub fn discovery_requests(&self) -> impl Iterator<Item = DiscoveryRequest> {
        self.report.events.iter().filter_map(|event| {
            if let IntroEvent::Discovery(request) = event {
                Some(*request)
            } else {
                None
            }
        })
    }

    /// The second part of the frame, once room play has run (`RoomTurnAwaited::resume`'s
    /// steps), reported after the stage's step: the frame's whole report.
    #[must_use]
    pub fn finish(self, fight: FightReply, discovery: DiscoveryReply) -> Report {
        let Self {
            state,
            content,
            rules,
            mut report,
        } = self;
        RoomTurnAwaited::after(&report).complete(
            state,
            content,
            rules,
            fight,
            discovery,
            &mut report,
        );
        report
    }

    /// A cut between the parts: what the stage's step reported, and the room's turn still
    /// awaited, which `RoomTurnAwaited::resume` takes once.
    pub fn interrupt(self) -> (Report, RoomTurnAwaited) {
        let turn = RoomTurnAwaited::after(&self.report);
        (self.report, turn)
    }
}

/// A frame cut after its stage's step and before its room's turn (`DiscoveryBegins`).
///
/// The stage step's effects are in the state the caller holds; this keeps what the turn still
/// needs, its `RoomTurnCheckpoint`. It is taken once, by `resume`. A caller that saves the cut
/// stores `checkpoint` with the intro's state and later continues from `restore`: the restored
/// turn finishes the same frame, reporting only what the room's turn and the scripts do.
#[must_use = "the frame ends only when its room's turn is resumed"]
#[derive(Debug, PartialEq, Eq)]
pub struct RoomTurnAwaited {
    /// What the turn still needs, as it is saved.
    saved: RoomTurnCheckpoint,
}

impl RoomTurnAwaited {
    /// The room's turn awaited after a stage step that reported this.
    fn after(stage_step: &Report) -> Self {
        Self::restore(RoomTurnCheckpoint {
            hide: hide_asked_in(stage_step),
        })
    }

    /// The turn as the caller saves it, with the intro's state.
    #[must_use]
    pub const fn checkpoint(&self) -> RoomTurnCheckpoint {
        self.saved
    }

    /// The turn a saved cut awaits, to be resumed once.
    pub const fn restore(saved: RoomTurnCheckpoint) -> Self {
        Self { saved }
    }

    /// The frame's second part, once room play has run: the hide asked for this frame takes
    /// effect, the encounters' cues are read (`PageTwoBegins`, `DiscoveryReturns`), then the
    /// baby, the caret's script, the page's script, the eyes' script and the glows. Only what
    /// this part does is reported: the stage step's requests are not asked again.
    #[must_use]
    pub fn resume(
        self,
        state: &mut IntroState,
        content: &IntroContent,
        rules: Rules,
        fight: FightReply,
        discovery: DiscoveryReply,
    ) -> Report {
        let mut report = Report::empty();
        self.complete(state, content, rules, fight, discovery, &mut report);
        report
    }

    /// The second part's steps, reported into `report`.
    fn complete(
        self,
        state: &mut IntroState,
        content: &IntroContent,
        rules: Rules,
        fight: FightReply,
        discovery: DiscoveryReply,
        report: &mut Report,
    ) {
        self.saved.hide.takes_effect(state);
        fight_signals(state, fight, report);
        discovery_signals(state, discovery, report);
        baby_frame(state, content, rules, report);
        caret_frame(state, content, rules, report);
        page_frame(state, content, rules, report);
        eyes_frame(state, content, rules, report);
        letters_glow(state, report);
        show_pictures(state, content, report);
    }
}

/// The stage's step: the opening's, page 1's, a passage's, a dissolve's frame, a page's turn, or
/// the fade's. A waiting stage moves on here only on a new press (`PageOneWaits`,
/// `PageTwoWaits`, `PageThreeWaits`, `PageFourWaits`, `WaitingForPress`); the fight and the
/// discovery wait for their encounter's cue, taken after room play, and the delivery for its
/// baby's signal; the stages left out hold while the scripts run.
fn stage_step(
    state: &mut IntroState,
    pressed: bool,
    track: TrackReply,
    content: &IntroContent,
    rules: Rules,
    report: &mut Report,
) {
    match state.stage {
        IntroStage::Begins => intro_begins(state, report),
        IntroStage::OpeningLoads => opening_loads(state, track),
        IntroStage::OpeningBrightens => opening_brightens(state),
        IntroStage::OpeningPause => opening_pause(state, report),
        IntroStage::FirstTune => first_tune(state, report),
        IntroStage::SecondTuneLoads => second_tune_loads(state, track),
        IntroStage::SecondTune => second_tune(state),
        IntroStage::OpeningDarkens => opening_darkens(state),
        IntroStage::PageOneMadeReady => page_one_is_made_ready(state, content, report),
        IntroStage::PageOneLoads => page_one_loads(state, track),
        IntroStage::PageOneBrightens => page_one_brightens(state),
        IntroStage::PageOneWaits => {
            if pressed {
                battle_begins(state, content, report);
            }
        }
        IntroStage::IntoFight => dissolve_frame(state, Crossing::IntoFight, content, report),
        IntroStage::PageTwoPending => page_two_begins(state, content, report),
        IntroStage::OutOfFight => dissolve_frame(state, Crossing::OutOfFight, content, report),
        IntroStage::PageTwoWaits => {
            if pressed {
                discovery_begins(state, content, report);
            }
        }
        IntroStage::IntoDiscovery => {
            dissolve_frame(state, Crossing::IntoDiscovery, content, report);
        }
        IntroStage::PageThreePending => page_three_begins(state, content, report),
        IntroStage::OutOfDiscovery => {
            dissolve_frame(state, Crossing::OutOfDiscovery, content, report);
        }
        IntroStage::PageThreeWaits => {
            if pressed {
                delivery_begins(state, content, report);
            }
        }
        IntroStage::IntoDelivery => {
            dissolve_frame(state, Crossing::IntoDelivery, content, report);
        }
        IntroStage::PageFourPending => page_four_begins(state, content, report),
        IntroStage::OutOfDelivery => {
            dissolve_frame(state, Crossing::OutOfDelivery, content, report);
        }
        IntroStage::PageFourWaits => {
            if pressed {
                examination_begins(state, content, report);
            }
        }
        IntroStage::ToScene => dissolve_frame(state, Crossing::ToScene, content, report),
        IntroStage::ToText => dissolve_frame(state, Crossing::ToText, content, report),
        IntroStage::ToPageFive => page_five_begins(state, content, report),
        IntroStage::WaitingForPress => {
            if pressed {
                press_turns_the_page(state, content, report);
            }
        }
        IntroStage::FadingOut => {
            picture_darkens(state, rules);
            intro_ends(state, report);
        }
        IntroStage::Fight
        | IntroStage::Discovery
        | IntroStage::Delivery
        | IntroStage::Scene
        | IntroStage::Text
        | IntroStage::Done => {}
    }
}

/// The signals a script handed the scene, taken in order.
fn take_signals(state: &mut IntroState, ran: &Ran, rules: Rules, report: &mut Report) {
    for &signal in &ran.signals {
        take_signal(state, signal, rules, report);
    }
}

/// The baby: it leaves, rises or is carried across, then its script runs.
fn baby_frame(state: &mut IntroState, content: &IntroContent, rules: Rules, report: &mut Report) {
    baby_leaves(state);
    baby_rises(state);
    baby_crosses_the_picture(state);
    let Some(baby) = state.baby else {
        return;
    };
    let ran = run(baby.script.progress, &content.baby[baby.script.script]);
    state.baby = ran.progress.map(|progress| Baby {
        script: ScriptPlace {
            progress,
            ..baby.script
        },
        ..baby
    });
    take_signals(state, &ran, rules, report);
}

/// The caret's script runs (`CaretBlinks`).
fn caret_frame(state: &mut IntroState, content: &IntroContent, rules: Rules, report: &mut Report) {
    let Some(caret) = state.caret else {
        return;
    };
    let ran = run(caret.script.progress, &content.caret[caret.script.script]);
    state.caret = ran.progress.map(|progress| Caret {
        script: ScriptPlace {
            progress,
            ..caret.script
        },
        ..caret
    });
    take_signals(state, &ran, rules, report);
}

/// The page's script runs: its signals, and the letter its new step types (`LetterIsTyped`).
fn page_frame(state: &mut IntroState, content: &IntroContent, rules: Rules, report: &mut Report) {
    let Some(page) = state.page else {
        return;
    };
    let script = &content.pages[page.script];
    let ran = run(page.progress, script);
    state.page = ran
        .progress
        .map(|progress| ScriptPlace { progress, ..page });
    take_signals(state, &ran, rules, report);
    let Some(began) = ran.began else {
        return;
    };
    if let Some(Step::Show {
        shows: Shows::Text(letter),
        ..
    }) = script.get(began)
    {
        letter_is_typed(state, *letter, script.get(began.wrapping_add(1)), report);
    }
}

/// Samus's eyes' script runs.
fn eyes_frame(state: &mut IntroState, content: &IntroContent, rules: Rules, report: &mut Report) {
    let Some(eyes) = state.eyes else {
        return;
    };
    let ran = run(eyes.progress, &content.eyes[eyes.script]);
    state.eyes = ran
        .progress
        .map(|progress| ScriptPlace { progress, ..eyes });
    take_signals(state, &ran, rules, report);
}

/// The pictures the eyes', the baby's and the caret's scripts show as the frame ends; the caret
/// shows whenever its script's step shows its picture (`CaretBlinks`).
fn show_pictures(state: &IntroState, content: &IntroContent, report: &mut Report) {
    report.eyes = state
        .eyes
        .and_then(|eyes| image(showing(eyes.progress, &content.eyes[eyes.script])))
        .cloned();
    report.baby = state.baby.and_then(|baby| {
        let script = &content.baby[baby.script.script];
        placed(showing(baby.script.progress, script), baby.place)
    });
    report.caret = state.caret.and_then(|caret| {
        let script = &content.caret[caret.script.script];
        placed(showing(caret.script.progress, script), caret.place)
    });
}

/// The picture a step shows, if it shows one.
const fn image(shows: &Shows) -> Option<&PictureName> {
    match shows {
        Shows::Image(picture) => Some(picture),
        Shows::Nothing | Shows::Text(_) => None,
    }
}

/// The picture a step shows at a place, if it shows one.
fn placed(shows: &Shows, place: Pair<Px>) -> Option<ShownPicture> {
    image(shows).map(|picture| ShownPicture {
        picture: picture.clone(),
        place,
    })
}
