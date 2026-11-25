//! Mother Brain: she breathes, takes missile hits and flashes, shakes once defeated, cues page 2
//! and leaves as the return dissolve ends.

use core::num::Wrapping;

use types::Parity;
use types::flashback::{
    Animation, Brain, BrainPhase, BreathStep, FlashBeats, HoldBeats, PriorHits, ShakeBeats,
};

use crate::clip;
use crate::contacts::{ReturnDissolve, RoomPlay, ShotCandidate, ShotId};
use crate::rules::Rules;

/// Spec: `BrainBreathes`. The beats each breathing picture holds, 16.
const BREATH_HOLD_BEATS: u16 = 16;
/// Spec: `BrainBreathes`. Each breathing picture's full hold.
const BREATH_HOLD: HoldBeats = HoldBeats::of::<BREATH_HOLD_BEATS>();

/// Spec: `BrainBreathes`, `ReturnToPage`. The breathing clip at its first picture, full hold.
const BREATH_FROM_START: Animation<BreathStep> = Animation {
    step: BreathStep::First,
    hold: BREATH_HOLD,
};

/// Spec: `EncounterStarts`. Mother Brain on the encounter's first beat: enduring with no hits,
/// no flash, her first breathing picture for its full hold.
pub const AT_START: Brain = Brain {
    phase: BrainPhase::Endures {
        hits: PriorHits::NONE,
    },
    flash: FlashBeats::NONE,
    breath: BREATH_FROM_START,
};

/// Spec: `BrainFlashes`. The colours of the hit: what a flash beat writes to all 16 colours of
/// image palette 7, the palette Mother Brain and the rings share.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HurtColours {
    /// Every colour white.
    White,
    /// The imported normal boss palette, in full.
    Normal,
}

/// What the encounter asks of the intro.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntroRequest {
    /// Spec: `ReturnToPage`. The cue for page 2: the shaking has run its hold.
    CuePageTwo,
    /// Spec: `ReturnToPage`. The encounter has ended with the return dissolve: hide Samus and
    /// her shots for drawing.
    HideSamus,
}

/// Spec: `BrainEndures`. What a missile impact does to Mother Brain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toll {
    /// She endures it.
    Endured,
    /// It reaches the hit threshold: she is defeated and the bursts begin.
    Decisive,
}

/// Spec: `BrainEndures`. A missile impact: the shot weapons turns into its impact, and its toll.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissileImpact {
    /// The shot that hit.
    pub shot: ShotId,
    /// What it did.
    pub toll: Toll,
}

/// What Mother Brain did on a beat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrainBeat {
    /// The colours of the hit the flash wrote, if one ran.
    pub palette: Option<HurtColours>,
    /// The missile impact, if one hit.
    pub impact: Option<MissileImpact>,
    /// The request to the intro, if one is made.
    pub intro: Option<IntroRequest>,
}

impl BrainBeat {
    /// A beat with nothing to report.
    const QUIET: Self = Self {
        palette: None,
        impact: None,
        intro: None,
    };
}

/// One beat of Mother Brain, by the phase she began it in. With her absent nothing happens.
pub fn step(
    slot: &mut Option<Brain>,
    rules: Rules,
    dissolve: ReturnDissolve,
    play: &RoomPlay,
) -> BrainBeat {
    let Some(brain) = slot else {
        return BrainBeat::QUIET;
    };
    match brain.phase {
        BrainPhase::Endures { hits } => {
            let palette = flash(brain);
            breathe(brain);
            BrainBeat {
                palette,
                impact: endure(brain, hits, rules, play),
                intro: None,
            }
        }
        BrainPhase::Shaking { elapsed } => BrainBeat {
            palette: flash(brain),
            impact: None,
            intro: shake(brain, elapsed, rules),
        },
        BrainPhase::Returning => match dissolve {
            ReturnDissolve::Continuing => {
                breathe(brain);
                BrainBeat::QUIET
            }
            ReturnDissolve::ReachesZero => {
                *slot = None;
                BrainBeat {
                    intro: Some(IntroRequest::HideSamus),
                    ..BrainBeat::QUIET
                }
            }
        },
    }
}

/// Spec: `BrainBreathes`. The picture after each, the clip wrapping from the fourth to the
/// first.
const fn next_breath(step: BreathStep) -> BreathStep {
    match step {
        BreathStep::First => BreathStep::Second,
        BreathStep::Second => BreathStep::Third,
        BreathStep::Third => BreathStep::Fourth,
        BreathStep::Fourth => BreathStep::First,
    }
}

/// Spec: `BrainBreathes`. One beat of breathing.
fn breathe(brain: &mut Brain) {
    brain.breath = clip::advanced(brain.breath, |step| Animation {
        step: next_breath(step),
        hold: BREATH_HOLD,
    });
}

/// Spec: `BrainFlashes`. The colours of the hit a flash beat writes: white on an even count,
/// the normal palette on an odd one.
const fn hurt_colours(count: Parity) -> HurtColours {
    match count {
        Parity::Even => HurtColours::White,
        Parity::Odd => HurtColours::Normal,
    }
}

/// Spec: `BrainFlashes`. A positive flash count at the start of the beat writes the colours of
/// the hit and spends a beat of flash. It runs before a new hit refreshes the count.
fn flash(brain: &mut Brain) -> Option<HurtColours> {
    if brain.flash == FlashBeats::NONE {
        return None;
    }
    let colours = hurt_colours(brain.flash.parity());
    brain.flash = brain.flash.spent();
    Some(colours)
}

/// Spec: `BrainEndures`. Whether a missile's centre is strictly left of the contact line.
fn crosses(shot: ShotCandidate, rules: Rules) -> bool {
    shot.x < Wrapping(u16::from(rules.contact.limit.0))
}

/// Spec: `BrainEndures`. The first ordinary missile hits if it has crossed the contact line:
/// it counts a hit and sets a whole flash, and the hit that reaches the threshold defeats her,
/// starting her shaking at no beats elapsed.
pub fn endure(
    brain: &mut Brain,
    hits: PriorHits,
    rules: Rules,
    play: &RoomPlay,
) -> Option<MissileImpact> {
    let shot = play.first_missile().filter(|&&shot| crosses(shot, rules))?;
    let counted = PriorHits(hits.0.saturating_add(1));
    brain.flash = FlashBeats::FULL;
    let toll = if counted.0 >= rules.contact.hits.get() {
        brain.phase = BrainPhase::Shaking {
            elapsed: ShakeBeats::NONE,
        };
        Toll::Decisive
    } else {
        brain.phase = BrainPhase::Endures { hits: counted };
        Toll::Endured
    };
    Some(MissileImpact {
        shot: shot.id,
        toll,
    })
}

/// Spec: `ReturnToPage`. A shaking beat: count it, and when it reaches the shake hold cue page
/// 2, start returning and restart the breathing clip in place of this beat's breath.
fn shake(brain: &mut Brain, elapsed: ShakeBeats, rules: Rules) -> Option<IntroRequest> {
    let elapsed = ShakeBeats(elapsed.0.saturating_add(1));
    if elapsed.0 >= rules.shake_hold.get() {
        brain.phase = BrainPhase::Returning;
        brain.breath = BREATH_FROM_START;
        Some(IntroRequest::CuePageTwo)
    } else {
        brain.phase = BrainPhase::Shaking { elapsed };
        breathe(brain);
        None
    }
}
