//! The recorded pad and control authority at the intro's actor opportunity.

use data::assets::playback as asset;
pub use screens_playback::{CrossfadeCount, DemoPad, Handoff, SceneStart, WholeX};
use screens_playback::{PadHold, ScenePad, TurnReadings};
use types::playback::PadAction;
pub use types::playback::ScenePhase;
use types::{Buttons, Set, button};

/// Imported holds in the model's input format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content(ScenePad);

impl From<asset::ScenePad> for Content {
    fn from(pad: asset::ScenePad) -> Self {
        let holds = |timeline: Vec<asset::PadHold>| {
            timeline
                .into_iter()
                .map(|hold| PadHold {
                    turns: hold.turns,
                    sample: hold.sample,
                })
                .collect()
        };
        Self(ScenePad {
            battle: holds(pad.battle),
            approach: holds(pad.approach),
            watch: holds(pad.watch),
        })
    }
}

/// Imported pad content and the sole live playback phase.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Playback {
    content: Content,
    phase: ScenePhase,
}

/// What work Samus and weapons can perform on this room actor turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Authority {
    /// The played scene does not own this opportunity.
    Outside,
    /// Controlled pose, weapons and subsequent intro movement/appearance.
    Controlled,
    /// Existing projectile work and subsequent intro movement/appearance.
    BattleLocked,
    /// Controlled pose and weapons complete on the release turn, with no subsequent
    /// movement/appearance.
    Releasing,
}

/// An actor turn's pad, immediate finish handoff and work authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TurnReport {
    /// Current and previous samples for the controlled pose and weapons update.
    pub pad: Option<DemoPad>,
    /// Immediate finish, applied before the turn's ordinary pose choice.
    pub handoff: Option<Handoff>,
    /// The work admitted on this turn.
    pub authority: Authority,
}

impl Playback {
    /// Admit imported content with no active scene.
    #[must_use]
    pub fn new(content: asset::ScenePad) -> Self {
        Self {
            content: content.into(),
            phase: ScenePhase::Off,
        }
    }

    /// Current scene memory, including the last issued pad.
    #[must_use]
    pub const fn phase(&self) -> ScenePhase {
        self.phase
    }

    /// Begin a scene before its first room actor opportunity.
    #[must_use]
    pub fn start(&mut self, scene: SceneStart) -> Option<Handoff> {
        start(&mut self.phase, &self.content, scene)
    }

    /// Supply controls after the intro stage and before Samus's motion.
    #[must_use]
    pub fn turn(&mut self, x: WholeX, fade: CrossfadeCount) -> TurnReport {
        actor_turn(&mut self.phase, &self.content, x, fade)
    }
}

fn start(phase: &mut ScenePhase, content: &Content, scene: SceneStart) -> Option<Handoff> {
    let report = screens_playback::play_turn(phase, &content.0, Some(scene), None);
    report.handoffs.first().copied()
}

/// Scene activity that reaches a display cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cut {
    /// The intro may install a scene independently from its first actor opportunity.
    pub start: Option<SceneStart>,
    /// The pre-motion X and updated crossfade at the actual actor pad opportunity.
    pub actor: Option<(WholeX, CrossfadeCount)>,
}

/// Fresh playback publications on a display cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CutReport {
    /// The scene entry event.
    pub start: Option<Handoff>,
    /// The work authority and pad at the reached actor opportunity.
    pub actor: Option<TurnReport>,
}

pub(super) fn advance(phase: &mut ScenePhase, content: &Content, cut: Cut) -> CutReport {
    CutReport {
        start: cut.start.and_then(|scene| start(phase, content, scene)),
        actor: cut
            .actor
            .map(|(x, fade)| actor_turn(phase, content, x, fade)),
    }
}

pub(super) fn valid_phase(phase: ScenePhase, content: &Content) -> bool {
    let (place, timeline) = match phase {
        ScenePhase::Battle(place) => (place, &content.0.battle),
        ScenePhase::Approach(place) => (place, &content.0.approach),
        ScenePhase::Watch(place) => (place, &content.0.watch),
        ScenePhase::Off | ScenePhase::BattleLocked | ScenePhase::Released => return true,
    };
    let turns = timeline.iter().try_fold(0_u32, |total, hold| {
        total.checked_add(u32::from(hold.turns.get()))
    });
    turns.is_some_and(|turns| place.progress.0 <= turns)
        && place
            .last
            .pressed
            .iter()
            .all(|(action, pressed)| !*pressed || place.last.held[action])
}

pub(super) fn actor_turn(
    phase: &mut ScenePhase,
    content: &Content,
    x: WholeX,
    fade: CrossfadeCount,
) -> TurnReport {
    let before = *phase;
    let report = screens_playback::play_turn(phase, &content.0, None, Some(&Readings(x, fade)));
    let authority = match (before, *phase) {
        (ScenePhase::Off | ScenePhase::Released, _) => Authority::Outside,
        (ScenePhase::BattleLocked, _) => Authority::BattleLocked,
        (_, ScenePhase::Released) => Authority::Releasing,
        _ => Authority::Controlled,
    };
    TurnReport {
        pad: report.pad,
        handoff: report.handoffs.first().copied(),
        authority,
    }
}

struct Readings(WholeX, CrossfadeCount);

impl TurnReadings for Readings {
    fn samus_x(&self) -> WholeX {
        self.0
    }

    fn crossfade(&self) -> CrossfadeCount {
        self.1
    }
}

/// The standard intro bindings expressed as the gameplay owner's buttons.
#[must_use]
pub fn buttons(actions: Set<PadAction>) -> Buttons {
    let bindings = [
        (PadAction::Left, button::LEFT),
        (PadAction::Jump, button::JUMP),
        (PadAction::Fire, button::SHOOT),
        (PadAction::AimUp, button::AIM_UP),
    ];
    Buttons(
        bindings
            .into_iter()
            .filter(|(action, _)| actions[*action])
            .fold(0, |bits, (_, button)| bits | button.0),
    )
}
