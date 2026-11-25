//! Spec: `EncounterBeats`, `EncounterStarts`, `BackgroundBobs`. The encounter's clock: the start
//! admitted during scene setup, then one beat per scene work frame.

use enum_map::EnumMap;
use types::Parity;
use types::flashback::{
    BrainPhase, BurstPlace, EncounterState, PendingStart, PriorHits, RoomBackgroundY,
};

use crate::brain::{self, HurtColours, IntroRequest, MissileImpact, Toll};
use crate::bursts;
use crate::contacts::{ReturnDissolve, RoomPlay, ShotId};
use crate::rings::{self, RingKnockback};
use crate::rules::{Rules, ShakeTravel};

/// What a beat reports, each to its owner. Video also reads the State's background Y and the
/// encounter's [`pictures`](crate::pictures).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Effects {
    /// To weapons: the missile to turn into its impact at the same place.
    pub missile_impact: Option<ShotId>,
    /// To Samus's owners, after movement and before drawing: a ring's push, with its protection
    /// and push holds.
    pub ring_knockback: Option<RingKnockback>,
    /// To video: the colours of the hit, the flash's write of image palette 7.
    pub palette: Option<HurtColours>,
    /// To the intro: the cue for page 2, or the request to hide Samus and her shots.
    pub intro: Option<IntroRequest>,
}

/// Spec: `EncounterStarts`. A new encounter start during scene setup.
///
/// It remembers the pending start and the inherited background Y. Neither the boss nor the
/// ring schedule steps until the encounter's first beat; the start and its Y carry across a
/// cut before then.
pub const fn admit_start(state: &mut EncounterState, background_y: RoomBackgroundY) {
    state.start = PendingStart::Pending;
    state.background_y = background_y;
}

/// Spec: `EncounterBeats`. One encounter beat, once per scene work frame.
///
/// The scene calls it the first time it reaches its encounter phase in a work frame, a frame
/// that spans several video cuts included. A frame resumed after a cut that already ran its
/// beat, or a cut before the encounter phase, doesn't call it: the State stays put and no
/// events are made.
///
/// `parity` is the intro work frame's beat, `dissolve` where the return dissolve stands, and
/// `play` what room play supplies after Samus moves and weapons step.
pub fn beat(
    state: &mut EncounterState,
    rules: Rules,
    parity: Parity,
    dissolve: ReturnDissolve,
    play: &RoomPlay,
) -> Effects {
    match state.start {
        PendingStart::Pending => first_beat(state, rules, play),
        PendingStart::Clear => later_beat(state, rules, parity, dissolve, play),
    }
}

/// Spec: `EncounterStarts`. The first beat consumes the pending start: Mother Brain shows her
/// first breathing picture, the first pair is scheduled and the first missile is checked with
/// no prior hits and no flash. The inherited background Y stays.
fn first_beat(state: &mut EncounterState, rules: Rules, play: &RoomPlay) -> Effects {
    let mut brain = brain::AT_START;
    let impact = brain::endure(&mut brain, PriorHits::NONE, rules, play);
    state.start = PendingStart::Clear;
    state.brain = Some(brain);
    state.next_pair = Some(rings::FIRST_PAIR_DUE);
    land(state, impact);
    Effects {
        missile_impact: impact.map(|impact| impact.shot),
        ring_knockback: None,
        palette: None,
        intro: None,
    }
}

/// A beat of the running encounter. Rings in motion and the background read Mother Brain's
/// phase as the beat began; bursts take their ranks after the rings' departures.
fn later_beat(
    state: &mut EncounterState,
    rules: Rules,
    parity: Parity,
    dissolve: ReturnDissolve,
    play: &RoomPlay,
) -> Effects {
    let phase_at_start = state.brain.map(|brain| brain.phase);
    let brain_beat = brain::step(&mut state.brain, rules, dissolve, play);
    let shaking_at_start = matches!(phase_at_start, Some(BrainPhase::Shaking { .. }));
    let ring_knockback = rings::advance(state, shaking_at_start, play.samus_x);
    bursts::pulse(&mut state.bursts);
    if brain_beat.intro == Some(IntroRequest::HideSamus) {
        state.bursts = EnumMap::<BurstPlace, _>::default();
    }
    land(state, brain_beat.impact);
    if let Some(BrainPhase::Shaking { .. } | BrainPhase::Returning) = phase_at_start {
        state.background_y = bobbed(state.background_y, parity, rules.shake_travel);
    }
    Effects {
        missile_impact: brain_beat.impact.map(|impact| impact.shot),
        ring_knockback,
        palette: brain_beat.palette,
        intro: brain_beat.intro,
    }
}

/// Spec: `BrainEndures`, `BurstsPulse`. The decisive impact begins the delayed bursts.
fn land(state: &mut EncounterState, impact: Option<MissileImpact>) {
    if let Some(MissileImpact {
        toll: Toll::Decisive,
        ..
    }) = impact
    {
        bursts::reserve(state);
    }
}

/// Spec: `BackgroundBobs`. While Mother Brain was already shaking or returning as the beat
/// began, the background moves down by the travel on even beats and up on odd ones, wrapping
/// at signed 16 bits.
fn bobbed(y: RoomBackgroundY, parity: Parity, travel: ShakeTravel) -> RoomBackgroundY {
    let travel = i16::from(travel.get());
    RoomBackgroundY(match parity {
        Parity::Even => y.0.wrapping_add(travel),
        Parity::Odd => y.0.wrapping_sub(travel),
    })
}
