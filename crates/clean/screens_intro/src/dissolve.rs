//! The dissolves (`PicturesDissolve`, `DissolveEnds`, `BattleDissolves`): one set of colours dims
//! to black while another brightens from black, in 32 even steps, one every 4 frames. The
//! battle's, the discovery's, the delivery's and the examination's crossings, each way, share
//! this arithmetic; each supplies only its colour sets, the order it moves them in and what
//! happens as it ends.

use types::intro::{FadingColour, FadingColours, IntroStage, IntroState};
use types::{Colour, Palette};

use crate::content::{Dissolve, IntroContent};
use crate::report::{DiscoveryRequest, FlashbackRequest, IntroEvent, PictureLayout, Report};

/// `PageFiveBegins`, `PageOneIsMadeReady`, `BattleBegins`, `PageTwoBegins`, `DiscoveryBegins`,
/// `DiscoveryReturns`, `DeliveryBegins`, `PageFourBegins`, `ExaminationBegins`: a dissolve runs
/// 128 frames, its count from 127.
pub const DISSOLVE_FROM: u16 = 127;
/// `BattleDissolves`, `DiscoveryReturns`: the count the dissolve back to the text steps from as
/// the encounter is closed, the frame before it ends.
const ENCOUNTER_CLOSES_FROM: u16 = 1;
/// `PicturesDissolve`: the dissolve steps on every fourth count.
const BEAT: u16 = 4;
/// A channel's level is in 256ths of a whole level.
const LEVEL: u16 = 256;
/// `PicturesDissolve`: a brightening channel's level wraps at 8,192, 32 whole levels.
const BRIGHTEN_SPAN: u16 = 8192;
/// The colour shown takes a channel's whole levels modulo 32.
const SHOWN_LEVELS: u16 = 32;
/// A colour's step is its full level × 8, a 32nd of its full level in 256ths.
const STEP_PER_LEVEL: u16 = 8;

/// A crossing between the text and a picture: the dissolve one stage runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crossing {
    /// The text into the battle on Zebes (`IntoFight`).
    IntoFight,
    /// The battle back to the text, page 2 typing (`OutOfFight`).
    OutOfFight,
    /// The text into the discovery room (`IntoDiscovery`).
    IntoDiscovery,
    /// The discovery back to the text, page 3 typing (`OutOfDiscovery`).
    OutOfDiscovery,
    /// The text into the delivery (`IntoDelivery`).
    IntoDelivery,
    /// The delivery back to the text, page 4 typing (`OutOfDelivery`).
    OutOfDelivery,
    /// The text into the examination (`ToScene`).
    ToScene,
    /// The examination back to the text, page 5 typing (`ToText`).
    ToText,
}

impl Crossing {
    /// The stage that runs this crossing.
    #[must_use]
    pub const fn stage(self) -> IntroStage {
        match self {
            Self::IntoFight => IntroStage::IntoFight,
            Self::OutOfFight => IntroStage::OutOfFight,
            Self::IntoDiscovery => IntroStage::IntoDiscovery,
            Self::OutOfDiscovery => IntroStage::OutOfDiscovery,
            Self::IntoDelivery => IntroStage::IntoDelivery,
            Self::OutOfDelivery => IntroStage::OutOfDelivery,
            Self::ToScene => IntroStage::ToScene,
            Self::ToText => IntroStage::ToText,
        }
    }

    /// The colour sets it moves: the discovery takes the battle's (`DiscoveryBegins`), the
    /// delivery the examination's (`DeliveryBegins`).
    #[must_use]
    pub const fn sets(self) -> Dissolve {
        match self {
            Self::IntoFight | Self::IntoDiscovery => Dissolve::IntoFight,
            Self::OutOfFight | Self::OutOfDiscovery => Dissolve::OutOfFight,
            Self::IntoDelivery | Self::ToScene => Dissolve::ToScene,
            Self::OutOfDelivery | Self::ToText => Dissolve::ToText,
        }
    }

    /// The stage it settles on as it ends (`DissolveEnds`, `BattleDissolves`).
    const fn settles_on(self) -> IntroStage {
        match self {
            Self::IntoFight => IntroStage::Fight,
            Self::IntoDiscovery => IntroStage::Discovery,
            Self::IntoDelivery => IntroStage::Delivery,
            Self::ToScene => IntroStage::Scene,
            Self::OutOfFight | Self::OutOfDiscovery | Self::OutOfDelivery | Self::ToText => {
                IntroStage::Text
            }
        }
    }

    /// The request that closes its encounter, asked as the count steps from 1 and applied after
    /// room play: Samus, her shots and Mother Brain hidden (`BattleDissolves`), or the
    /// discovery's actors removed (`DiscoveryReturns`).
    const fn closes_encounter(self) -> Option<IntroEvent> {
        match self {
            Self::OutOfFight => Some(IntroEvent::Flashback(
                FlashbackRequest::HideFightSamusAndShots,
            )),
            Self::OutOfDiscovery => Some(IntroEvent::Discovery(DiscoveryRequest::FinishDiscovery)),
            Self::IntoFight
            | Self::IntoDiscovery
            | Self::IntoDelivery
            | Self::OutOfDelivery
            | Self::ToScene
            | Self::ToText => None,
        }
    }
}

/// How a dissolve moves a colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Move {
    /// Each channel's level less its step, wrapping at 16 bits.
    Dim,
    /// Each channel's level plus its step, wrapping at 8,192.
    Brighten,
}

/// `PicturesDissolve`: the order each set of colours is moved in. To the scene, the dimming set
/// goes first; to the text, the brightening set. It tells only for a colour in both sets. The
/// spec gives no order for the battle's dissolves; they take the order of the dissolve that
/// leaves the text (into the fight) or returns to it (out of the fight).
const fn order(dissolve: Dissolve) -> [Move; 2] {
    match dissolve {
        Dissolve::ToScene | Dissolve::IntoFight => [Move::Dim, Move::Brighten],
        Dissolve::ToText | Dissolve::OutOfFight => [Move::Brighten, Move::Dim],
    }
}

/// A dissolve begins (`PageFiveBegins`, `BattleBegins`, `PageTwoBegins`, `DiscoveryBegins`,
/// `DiscoveryReturns`, `DeliveryBegins`, `PageFourBegins`, `ExaminationBegins`): every colour at
/// its full level, its step its full level × 8; then the colours of `black` at 0; the colours
/// shown are all taken, and the count starts from 127.
pub fn dissolve_begins(
    state: &mut IntroState,
    content: &IntroContent,
    black: &[u8],
    report: &mut Report,
) {
    state.colours = FadingColours(content.full.0.map(at_full));
    for &index in black {
        if let Some(colour) = state.colours.colour_mut(index) {
            colour.levels = FadingColour::STILL_BLACK.levels;
        }
    }
    report.colours = Some(shown(&state.colours));
    state.dissolve = DISSOLVE_FROM;
}

/// `PicturesDissolve`, `BabyRises`: whether a dissolve's count is on the beat, a multiple of 4.
#[must_use]
pub const fn on_beat(count: u16) -> bool {
    count.wrapping_rem(BEAT) == 0
}

/// One frame of a dissolve (`PicturesDissolve`, `BattleDissolves`): the count goes down by 1,
/// wrapping past 0 to 65,535. On the beat, each set moves and the colours shown are all taken;
/// when the count was 0 the step is the last, its colours taken in the frame the dissolve ends
/// (`DissolveEnds`, `BattleDissolves`). Back out of an encounter, the step from count 1 asks for
/// it to be closed (`Crossing::closes_encounter`), applied after room play.
pub fn dissolve_frame(
    state: &mut IntroState,
    crossing: Crossing,
    content: &IntroContent,
    report: &mut Report,
) {
    let count = state.dissolve;
    state.dissolve = count.wrapping_sub(1);
    if count == ENCOUNTER_CLOSES_FROM
        && let Some(request) = crossing.closes_encounter()
    {
        report.events.push(request);
    }
    if !on_beat(count) {
        return;
    }
    let dissolve = crossing.sets();
    let sets = &content.sets[dissolve];
    for way in order(dissolve) {
        let members = match way {
            Move::Dim => &sets.dims,
            Move::Brighten => &sets.brightens,
        };
        for &index in members {
            if let Some(colour) = state.colours.colour_mut(index) {
                *colour = moved(*colour, way);
            }
        }
    }
    let mut shown = shown(&state.colours);
    if count == 0 {
        dissolve_ends(state, crossing, content, &mut shown, report);
    }
    report.colours = Some(shown);
}

/// `PicturesDissolve`: a colour dimmed or brightened by its steps.
fn moved(colour: FadingColour, way: Move) -> FadingColour {
    let mut levels = colour.levels;
    for (level, step) in levels.iter_mut().zip(colour.steps) {
        *level = match way {
            Move::Dim => level.wrapping_sub(step),
            Move::Brighten => level.wrapping_add(step).wrapping_rem(BRIGHTEN_SPAN),
        };
    }
    FadingColour { levels, ..colour }
}

/// A dissolve ends, the frame's last step taken; the stage settles (`Crossing::settles_on`) and
/// the count has wrapped to 65,535. `BattleDissolves`: into the fight, the text is cleared, the
/// fight's picture is shown plain and its flashing lights begin. `DissolveEnds`: into the
/// discovery, the delivery or the examination, the text is cleared and the scene's picture is
/// shown plain. Every return to the text shows the text's picture plain and the colours the last
/// step took. Returning from the delivery or the examination, the caret's colours are shown over
/// those, their dissolve levels left unchanged. Back from the discovery (or, by
/// `BattleDissolves`, the battle), no caret colours are applied and the faded colours stay.
fn dissolve_ends(
    state: &mut IntroState,
    crossing: Crossing,
    content: &IntroContent,
    shown: &mut Palette,
    report: &mut Report,
) {
    state.stage = crossing.settles_on();
    match crossing {
        Crossing::IntoFight => report.events.extend([
            IntroEvent::TextCleared,
            IntroEvent::Layout(PictureLayout::PlainFight),
            IntroEvent::FlashingLightsBegin,
        ]),
        Crossing::IntoDiscovery | Crossing::IntoDelivery | Crossing::ToScene => {
            report.events.extend([
                IntroEvent::TextCleared,
                IntroEvent::Layout(PictureLayout::PlainScene),
            ]);
        }
        Crossing::OutOfFight | Crossing::OutOfDiscovery => report
            .events
            .push(IntroEvent::Layout(PictureLayout::PlainText)),
        Crossing::OutOfDelivery | Crossing::ToText => {
            report
                .events
                .push(IntroEvent::Layout(PictureLayout::PlainText));
            caret_colours_shown(content, shown);
        }
    }
}

/// `DissolveEnds`: the caret's colours shown over the colours the last step took, on returning
/// to the text from the delivery or the examination only.
fn caret_colours_shown(content: &IntroContent, shown: &mut Palette) {
    for caret in &content.caret_colours {
        if let Some(colour) = shown.0.get_mut(usize::from(caret.index)) {
            *colour = caret.colour;
        }
    }
}

/// The colours shown: each channel's whole levels, `level / 256` modulo 32.
fn shown(colours: &FadingColours) -> Palette {
    Palette(colours.0.map(|colour| {
        Colour::from_channels(
            colour
                .levels
                .map(|level| level.wrapping_div(LEVEL).wrapping_rem(SHOWN_LEVELS)),
        )
    }))
}

/// As a dissolve begins, a colour at its full level, its step its full level × 8.
fn at_full(full: Colour) -> FadingColour {
    let channels = full.channels();
    FadingColour {
        levels: channels.map(|channel| channel.wrapping_mul(LEVEL)),
        steps: channels.map(|channel| channel.wrapping_mul(STEP_PER_LEVEL)),
    }
}
