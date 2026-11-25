//! Spec: `HurtColours`. Her hurt colours: a fall in her energy or a push's start pulses her
//! colours, three frames in her hurt colours each followed by a frame in her suit's, then her
//! suit's colours hold while the hurt count runs out. The push's two poses, `Recoiling` and
//! `Leaping`, are ordinary figures with clips of their own, which start and play as every clip
//! does.

use types::RowColours;

use crate::colours::HurtCount;
use crate::inputs::{Activity, GrappleActivity, Inputs, MoveEvent, PulseColours, Suit};
use crate::sounds::{Sound, Spin};

/// The frame's hurt colours, after Health, with what their owners supply for them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HurtOpportunity {
    /// The frame has no hurt colours work: the hurt count stays as it is.
    Absent,
    /// Her hurt colours run.
    Observed {
        /// The suit selection made for this frame (equipment).
        suit: Suit,
        /// The grapple beam's activity (Grapple).
        grapple: GrappleActivity,
    },
}

/// The colours a count of the pulse shows in her row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PulseTint {
    /// Her hurt colours.
    Hurt,
    /// Her suit's colours, or the cinematic's grey while a cinematic is active.
    Suit,
}

/// Spec: `HurtColours`. The pulse, the one place that decides which colours she shows on each
/// count: on counts 1 to 6 in turn, her hurt colours then her suit's, three times. From count 7
/// on, her row is left as it is.
const PULSE: [PulseTint; 6] = [
    PulseTint::Hurt,
    PulseTint::Suit,
    PulseTint::Hurt,
    PulseTint::Suit,
    PulseTint::Hurt,
    PulseTint::Suit,
];

/// Spec: `HurtColours`. The count on which the pulse asks for the hurt sound: 2, its second
/// frame.
const HURT_SOUND_COUNT: u8 = 2;

/// Spec: `HurtColours`. The count whose reaching asks for her spin's sound again: 40.
const SPIN_AGAIN_COUNT: u8 = 40;

/// What the frame's hurt colours leave: the hurt count, the colours her row of the current
/// colours takes (none: left as it is), and the sound asked for, at most one.
pub type HurtFrame = (HurtCount, Option<RowColours>, Option<Sound>);

/// Spec: `HurtColours`. The frame's hurt colours, from `count`, her hurt count as the frame
/// starts, with `colours` the content's.
///
/// A push started this frame sets the count to 1; so does her energy falling with no pulse
/// under way, while a fall mid-pulse changes nothing. Then, with the count above 0, her row
/// takes the colours the pulse gives that count, the count 2 asks for `Hurt` unless a cinematic
/// is active, and the count goes up by 1. Reaching 40 while she spins or wall jumps with the
/// grapple dormant asks for her spin's sound again; reaching 60 it is 0. The target colours are
/// left as they are.
#[must_use]
pub fn hurt_colours(
    count: HurtCount,
    opportunity: HurtOpportunity,
    inputs: &Inputs,
    colours: &PulseColours,
) -> HurtFrame {
    let HurtOpportunity::Observed { suit, grapple } = opportunity else {
        return (count, None, None);
    };
    let count = struck(count, inputs);
    if !count.pulsing() {
        return (count, None, None);
    }
    let cinematic = inputs.activities[Activity::Cinematic];
    let row = tint(count).map(|tint| match tint {
        PulseTint::Hurt => colours.hurt,
        PulseTint::Suit if cinematic => colours.cinematic_grey,
        PulseTint::Suit => colours.suits[suit],
    });
    let next = count.next();
    let sound = if count.frames() == HURT_SOUND_COUNT {
        (!cinematic).then_some(Sound::Hurt)
    } else {
        spin_again(next, grapple, inputs)
    };
    (next, row, sound)
}

/// Spec: `HurtColours`. The count as a hit leaves it: 1 when a push started this frame, or when
/// her energy fell with no pulse under way; otherwise as it is.
fn struck(count: HurtCount, inputs: &Inputs) -> HurtCount {
    let fell_unpulsed = inputs.energy_fell() && !count.pulsing();
    if inputs.moves[MoveEvent::PushStarted] || fell_unpulsed {
        HurtCount::STRUCK
    } else {
        count
    }
}

/// The colours the pulse gives `count`, if it is one of its counts.
fn tint(count: HurtCount) -> Option<PulseTint> {
    let place = usize::from(count.frames().checked_sub(1)?);
    PULSE.get(place).copied()
}

/// Spec: `HurtColours`. Her spin's sound again, as her items choose it (`Spin::of`), where the
/// count `reached` is 40, the grapple is dormant and her figure as the frame ends spins or wall
/// jumps.
fn spin_again(reached: HurtCount, grapple: GrappleActivity, inputs: &Inputs) -> Option<Sound> {
    let again = reached.frames() == SPIN_AGAIN_COUNT
        && grapple == GrappleActivity::Dormant
        && inputs.figure_after.pose.action.spins();
    again.then(|| Spin::of(&inputs.items).sound())
}
