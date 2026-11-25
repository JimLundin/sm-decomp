//! One turn of the played scenes: a scene's start, the pad the room actors receive, and each
//! handoff to Samus and her weapons.

use types::playback::{PadSample, ScenePhase, TimelinePlace, TimelineProgress};

use crate::content::{PadHold, ScenePad, sample_at};

/// Samus's whole X, in pixels from the left edge of the discovery room.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WholeX(pub u16);

/// Spec: `SamusReachesEgg`. The stopping line, 178 px: Samus starting her turn left of it has
/// reached the baby.
const STOPPING_LINE: WholeX = WholeX(178);

/// The intro's remaining crossfade count, in turns, after its stage has advanced this turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrossfadeCount(pub u16);

impl CrossfadeCount {
    /// Spec: `DiscoveryReleasesSamus`. The count at which the return to the text crosses zero.
    pub const ZERO: Self = Self(0);
}

/// What a room actor turn reads from the other owners. Each is read only where a choice
/// depends on it: X on an Approach turn, the crossfade count on a Watch turn.
pub trait TurnReadings {
    /// Samus's whole X at the start of her turn, before this turn's motion.
    fn samus_x(&self) -> WholeX;
    /// The intro's crossfade count, from its updated stage.
    fn crossfade(&self) -> CrossfadeCount;
}

/// The scene the intro begins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneStart {
    /// The battle on Zebes.
    Battle,
    /// The discovery of the baby, starting with the approach.
    Discovery,
}

/// A one-turn output to Samus and her weapons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handoff {
    /// The battle has begun.
    StartBattle,
    /// The discovery has begun.
    StartDiscovery,
    /// The battle's finish: Samus stands looking left before her pose choice.
    StandLeft,
    /// The discovery's finish: Samus is released from the recorded controls.
    Release,
}

/// The pad a demo turn gives Samus and her weapons: this turn's sample and the one before it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DemoPad {
    /// This turn's sample.
    pub current: PadSample,
    /// The last sample issued before this turn.
    pub previous: PadSample,
}

/// What one turn reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TurnReport {
    /// The handoffs of this turn, in the order they happen: a start, then a finish.
    pub handoffs: Vec<Handoff>,
    /// The pad, when a demo turn issued one.
    pub pad: Option<DemoPad>,
}

impl SceneStart {
    /// Spec: `DemoReceivesScene`. The phase the scene enters, at the start of its timeline.
    fn entered(self) -> ScenePhase {
        match self {
            Self::Battle => ScenePhase::Battle(TimelinePlace::at_start()),
            Self::Discovery => ScenePhase::Approach(TimelinePlace::at_start()),
        }
    }

    /// Spec: `DemoReceivesScene`. The start handoff the scene publishes.
    const fn handoff(self) -> Handoff {
        match self {
            Self::Battle => Handoff::StartBattle,
            Self::Discovery => Handoff::StartDiscovery,
        }
    }
}

/// Spec: `DemoReceivesScene`. One turn of the recorded pad.
///
/// The intro may start a scene, which spends no room actor turn; then a room actor turn, if
/// there is one, plays the active phase. A turn with no start and no actor turn, or in Off,
/// Battle Locked or Released, keeps the State and publishes nothing.
pub fn play_turn(
    phase: &mut ScenePhase,
    scene_pad: &ScenePad,
    start: Option<SceneStart>,
    actor_turn: Option<&dyn TurnReadings>,
) -> TurnReport {
    let mut handoffs = Vec::new();
    if let Some(start) = start {
        *phase = start.entered();
        handoffs.push(start.handoff());
    }
    let issued = actor_turn.and_then(|readings| room_actor_turn(*phase, scene_pad, readings));
    let pad = issued.map(|issued| {
        *phase = issued.phase;
        handoffs.extend(issued.finish);
        issued.pad
    });
    TurnReport { handoffs, pad }
}

/// What an active phase's actor turn leaves: its next phase, the pad and any finish.
struct Issued {
    phase: ScenePhase,
    pad: DemoPad,
    finish: Option<Handoff>,
}

/// A room actor turn in each phase; the phases with no controls issue nothing.
fn room_actor_turn(
    phase: ScenePhase,
    scene_pad: &ScenePad,
    readings: &dyn TurnReadings,
) -> Option<Issued> {
    match phase {
        ScenePhase::Off | ScenePhase::BattleLocked | ScenePhase::Released => None,
        ScenePhase::Battle(place) => Some(battle_turn(place, &scene_pad.battle)),
        ScenePhase::Approach(place) => Some(approach_turn(place, scene_pad, readings.samus_x())),
        ScenePhase::Watch(place) => Some(watch_turn(place, &scene_pad.watch, readings.crossfade())),
    }
}

/// Spec: `DemoPlaysPad`, `BattleLocksOnLeft`. The battle plays its timeline; on the turn after
/// its last sample it finishes with Samus standing looking left.
fn battle_turn(place: TimelinePlace, battle: &[PadHold]) -> Issued {
    sample_at(battle, place.progress).map_or_else(
        || finish(place, Handoff::StandLeft, ScenePhase::BattleLocked),
        |sample| {
            issue(
                ScenePhase::Battle,
                place,
                sample,
                place.progress.after_turn(),
            )
        },
    )
}

/// Spec: `SamusReachesEgg`. Samus starting her turn left of the stopping line trades the walk
/// for the watch, whose first sample she takes at once, with the walk's last pad as the
/// previous one; otherwise the walk goes on.
fn approach_turn(place: TimelinePlace, scene_pad: &ScenePad, samus_x: WholeX) -> Issued {
    if samus_x < STOPPING_LINE {
        let watch_start = TimelinePlace {
            progress: TimelineProgress::START,
            last: place.last,
        };
        walk(ScenePhase::Watch, watch_start, &scene_pad.watch)
    } else {
        walk(ScenePhase::Approach, place, &scene_pad.approach)
    }
}

/// Spec: `DiscoveryReleasesSamus`. A Watch turn that begins with the crossfade count at zero
/// releases Samus; otherwise the watch goes on.
fn watch_turn(place: TimelinePlace, watch: &[PadHold], crossfade: CrossfadeCount) -> Issued {
    if crossfade == CrossfadeCount::ZERO {
        finish(place, Handoff::Release, ScenePhase::Released)
    } else {
        walk(ScenePhase::Watch, place, watch)
    }
}

/// Spec: `DemoPlaysPad`. A discovery timeline plays its samples; once it has run out it keeps
/// issuing Left with no fresh press, its progress held at the timeline's end.
fn walk(
    phase: fn(TimelinePlace) -> ScenePhase,
    place: TimelinePlace,
    timeline: &[PadHold],
) -> Issued {
    let (sample, progress) = sample_at(timeline, place.progress).map_or_else(
        || (PadSample::walking_left(), place.progress),
        |sample| (sample, place.progress.after_turn()),
    );
    issue(phase, place, sample, progress)
}

/// Spec: `DemoPlaysPad`. A sample issued with the last one before it; it becomes the last pad.
fn issue(
    phase: fn(TimelinePlace) -> ScenePhase,
    place: TimelinePlace,
    sample: PadSample,
    progress: TimelineProgress,
) -> Issued {
    Issued {
        phase: phase(TimelinePlace {
            progress,
            last: sample,
        }),
        pad: DemoPad {
            current: sample,
            previous: place.last,
        },
        finish: None,
    }
}

/// Spec: `BattleLocksOnLeft`, `DiscoveryReleasesSamus`. A finish issues the neutral sample with
/// the last one before it and the finish's handoff, and leaves the controls' phase.
fn finish(place: TimelinePlace, handoff: Handoff, ended: ScenePhase) -> Issued {
    Issued {
        phase: ended,
        pad: DemoPad {
            current: PadSample::neutral(),
            previous: place.last,
        },
        finish: Some(handoff),
    }
}
