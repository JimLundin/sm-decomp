//! Spec: `PerformedAnimationDecisions`, `SuspendedAppearance`.
//! Answers are chosen locally, retained before effects, and consumed exactly once.

use crate::clip::{Clip, Frames, Paces, Step};
use crate::inputs::{Content, Figure, Inputs};
use crate::moves::{fast_fall, rising_hold, standing_restart, wall_jump_ready};
use crate::opportunity::{
    Charge, CinematicActivity, ContactSuppressor, GraphicalContact, Opportunity, OpportunityOutput,
    SoundRequest,
};
use crate::play::play;
use crate::rules::Rules;
use crate::run::{dash_began, dash_lost};
use crate::starting::start;
use crate::state::State;
use serde::{Deserialize, Serialize};
use types::{Action, AnimationSound, BoostStage, Item, Set};

/// Spec: `SuspendedAppearance`. Owned opportunity inputs, including the prepared action.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppearanceRead {
    /// Stride inputs, before animation; suppressors remain mutable until read.
    Contact {
        /// Actual stride figure.
        figure: Figure,
        /// Last showing playhead before animation.
        playhead: crate::state::Playhead,
        /// Stride boost stage.
        stage: BoostStage,
        /// Stride instructions and contact marks.
        clip: Clip,
        /// Sound-only suppressors.
        suppressors: Set<ContactSuppressor>,
    },
    /// Actual accepted selection.
    Selection {
        /// Figure immediately before selection.
        from: Figure,
        /// Actually accepted figure.
        selected: Figure,
        /// Selection-time items.
        items: Set<Item>,
        /// Selection-time liquid predicate.
        submerged: bool,
        /// Selection-time cinematic activity.
        cinematic: CinematicActivity,
    },
    /// Drawing reads the independently retained preparation snapshot.
    Draw {
        /// Independently retained prepared action.
        previous: Action,
        /// Current drawing action.
        current: Action,
    },
    /// First passage entry, separate from drawing.
    Passage {
        /// Current action at first entry.
        current: Action,
        /// Shoot held at entry.
        shoot_held: bool,
        /// Weapons charge at entry.
        charge: Charge,
    },
}
impl From<Opportunity<'_>> for AppearanceRead {
    fn from(value: Opportunity<'_>) -> Self {
        match value {
            Opportunity::Contact {
                figure,
                playhead,
                stage,
                clip,
                suppressors,
            } => Self::Contact {
                figure,
                playhead,
                stage,
                clip: clip.clone(),
                suppressors,
            },
            Opportunity::PoseSelected {
                from,
                selected,
                items,
                submerged,
                cinematic,
            } => Self::Selection {
                from,
                selected,
                items,
                submerged,
                cinematic,
            },
            Opportunity::NormalDraw { previous, current } => Self::Draw { previous, current },
            Opportunity::PassageEntry {
                current,
                shoot_held,
                charge,
            } => Self::Passage {
                current,
                shoot_held,
                charge,
            },
        }
    }
}
impl AppearanceRead {
    /// Spec: `SuspendedAppearance`. The same read-only implementation as ordinary reads.
    #[must_use]
    pub fn read(&self) -> OpportunityOutput {
        match self {
            Self::Contact {
                figure,
                playhead,
                stage,
                clip,
                suppressors,
            } => Opportunity::Contact {
                figure: *figure,
                playhead: *playhead,
                stage: *stage,
                clip,
                suppressors: *suppressors,
            }
            .read(),
            Self::Selection {
                from,
                selected,
                items,
                submerged,
                cinematic,
            } => Opportunity::PoseSelected {
                from: *from,
                selected: *selected,
                items: *items,
                submerged: *submerged,
                cinematic: *cinematic,
            }
            .read(),
            Self::Draw { previous, current } => Opportunity::NormalDraw {
                previous: *previous,
                current: *current,
            }
            .read(),
            Self::Passage {
                current,
                shoot_held,
                charge,
            } => Opportunity::PassageEntry {
                current: *current,
                shoot_held: *shoot_held,
                charge: *charge,
            }
            .read(),
        }
    }
}

/// Spec: `PerformedAnimationDecisions`. Rules in their required execution order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationDecision {
    /// Begin a dash before playing.
    DashBegan,
    /// Restart standing before countdown.
    StandingRestart,
    /// Ready the kick step before countdown.
    WallJumpReady,
    /// Set the fast fall picture.
    FastFall,
    /// Hold the rising picture.
    RisingHold,
    /// Play the actual current clip.
    ClipPlaying,
    /// Initialize the actually accepted selection.
    SelectedClip,
    /// Clear a lost dash after initialization.
    DashLoss,
}
const ANIMATION_ORDER: [AnimationDecision; 8] = [
    AnimationDecision::DashBegan,
    AnimationDecision::StandingRestart,
    AnimationDecision::WallJumpReady,
    AnimationDecision::FastFall,
    AnimationDecision::RisingHold,
    AnimationDecision::ClipPlaying,
    AnimationDecision::SelectedClip,
    AnimationDecision::DashLoss,
];

/// Spec: `PerformedAnimationDecisions`. Local result, not a timing-assigned animation value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Answer {
    /// Apply the move's retained State.
    Change,
    /// Count down on the current picture.
    CountDown,
    /// Advance through the reached instruction.
    Advance,
    /// Keep the already played result.
    KeepPlayhead,
    /// Start at step zero.
    StartFirst,
    /// Start at step one.
    StartSecond,
}
/// Spec: `SuspendedAppearance`. Animation and appearance sounds share opportunity order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublishedSound {
    /// Current-clip control request.
    Animation(AnimationSound),
    /// Independent read-only opportunity request.
    Appearance(SoundRequest),
}
/// Spec: `PerformedAnimationDecisions`. Complete actual context and retained chosen result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfferedDecision {
    /// Rule reached by actual execution.
    pub rule: AnimationDecision,
    /// All four quantities immediately before its effect.
    pub before: State,
    /// Actual inputs retained with this answer.
    pub inputs: Inputs,
    /// Current clip for playing, accepted clip for initialization.
    pub clip: Clip,
    /// Actual pace content.
    pub paces: Paces,
    /// Actual tuning.
    pub rules: Rules,
    /// Chosen local answer.
    pub answer: Answer,
    /// Complete State that its effect will leave.
    pub after: State,
    /// Retained semantic control request.
    pub sound: Option<AnimationSound>,
}
/// Spec: `SuspendedAppearance`. Completed receipts include silent reads and complete State.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Receipt {
    /// Completed answer, with actual before and after quantities.
    Animation(Box<OfferedDecision>),
    /// Completed read, including silent publication.
    Appearance {
        /// Retained actual opportunity inputs.
        input: AppearanceRead,
        /// Complete unchanged animation State.
        state: State,
        /// Sound and graphics published by this read.
        output: OpportunityOutput,
    },
}
/// Spec: `SuspendedAppearance`. Snapshot of actual current and accepted clips and owner inputs.
///
/// Inputs for unmade decisions may change at their actual opportunities. Its `Content` lookup
/// serves only those admitted figures, not a general figure-content map.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnimationInputs {
    /// Mutable owner inputs for rules not yet decided.
    pub inputs: Inputs,
    /// Actual current-figure clip.
    pub current_clip: Clip,
    /// Figure immediately before the actual selection, independently of current-clip playing.
    pub selected_from: Figure,
    /// Actual accepted-figure clip, supplied before initialization is decided.
    pub selected_clip: Clip,
    /// Importer-supplied running paces.
    pub paces: Paces,
    /// Spec parameters.
    pub rules: Rules,
}
impl Content for AnimationInputs {
    fn clip(&self, figure: &Figure) -> &Clip {
        if *figure == self.inputs.figure {
            &self.current_clip
        } else {
            &self.selected_clip
        }
    }
    fn paces(&self) -> &Paces {
        &self.paces
    }
}
/// Spec: `SuspendedAppearance`. An ordered work plan; repeated drawing entries are distinct reads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkStep {
    /// One ordinary animation rule.
    Animation(AnimationDecision),
    /// One independent read at its actual opportunity.
    Appearance(AppearanceRead),
}
/// Spec: `SuspendedAppearance`. Complete saveable work, not additional inter-frame animation State.
///
/// Retains produced receipts, sounds, graphics and consumed work progress. The caller owns
/// external delivered positions and pending publications and saves them with this work and
/// the rest of its continuation. Serialize with serde; saving is a read, and deserialization
/// needs no original object.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuspendedAppearance {
    /// Caller-retained completed frame-start query, independent of later playhead changes.
    pub ahead: Option<crate::ahead::Ahead>,
    /// All four actual animation quantities.
    pub state: State,
    /// Still-needed owner inputs and content; absent for frozen appearance-only work.
    pub animation: Option<AnimationInputs>,
    /// Caller opportunity plan and mutable unmade read inputs.
    pub steps: Vec<WorkStep>,
    /// Consumed opportunity/input position.
    pub consumed: usize,
    /// Already chosen answer; future input changes cannot replace it.
    pub pending: Option<OfferedDecision>,
    /// Ordered completed receipts, including the playing result and actual selection.
    pub completed: Vec<Receipt>,
    /// Ordered completed requests, without deduplication.
    pub sounds: Vec<PublishedSound>,
    /// Completed graphical contacts, even when sound was suppressed.
    pub graphics: Vec<GraphicalContact>,
}
impl SuspendedAppearance {
    /// Spec: `SuspendedAppearance`. Start an explicit work plan; no opportunity runs here.
    #[must_use]
    pub const fn new(
        state: State,
        animation: Option<AnimationInputs>,
        steps: Vec<WorkStep>,
    ) -> Self {
        Self {
            ahead: None,
            state,
            animation,
            steps,
            consumed: 0,
            pending: None,
            completed: Vec::new(),
            sounds: Vec::new(),
            graphics: Vec::new(),
        }
    }
    /// Spec: `PerformedAnimationDecisions`. Ordinary animation plan in the required order.
    #[must_use]
    pub fn animation_steps() -> Vec<WorkStep> {
        ANIMATION_ORDER
            .into_iter()
            .map(WorkStep::Animation)
            .collect()
    }
    /// Spec: `PerformedAnimationDecisions`. Choose and retain an applicable animation answer
    /// without effects. An existing answer never changes; skipped rules are consumed.
    /// `None` means the next work is an appearance read or there is no unfinished work.
    /// `complete` distinguishes those cases and completes the read.
    pub fn offer(&mut self) -> Option<&OfferedDecision> {
        if self.pending.is_some() {
            return self.pending.as_ref();
        }
        while let Some(WorkStep::Animation(rule)) = self.steps.get(self.consumed) {
            let Some(data) = &self.animation else {
                self.consumed = self.consumed.saturating_add(1);
                continue;
            };
            if let Some(answer) = decide(*rule, self.state, data) {
                self.pending = Some(answer);
                break;
            }
            self.consumed = self.consumed.saturating_add(1);
        }
        self.pending.as_ref()
    }
    /// Spec: `SuspendedAppearance`. Complete one offered effect or one actual read.
    /// Distinguishes an appearance read from finished work when `offer` returns `None`.
    /// Returns false once there is no unfinished work. Completed reads are not repeated;
    /// delivery of retained outputs to external consumers remains caller-owned.
    pub fn complete(&mut self) -> bool {
        let _ = self.offer();
        if let Some(decision) = self.pending.take() {
            self.state = decision.after;
            self.sounds
                .extend(decision.sound.map(PublishedSound::Animation));
            self.completed.push(Receipt::Animation(Box::new(decision)));
        } else if let Some(WorkStep::Appearance(input)) = self.steps.get(self.consumed) {
            let output = input.read();
            self.sounds
                .extend(output.sound.map(PublishedSound::Appearance));
            self.graphics.extend(output.contact);
            self.completed.push(Receipt::Appearance {
                input: input.clone(),
                state: self.state,
                output,
            });
        } else {
            return false;
        }
        self.consumed = self.consumed.saturating_add(1);
        true
    }
    /// Spec: `SuspendedAppearance`. Resume only unfinished work, with no pauses.
    /// Appropriate only once all remaining inputs are available at their actual opportunities.
    pub fn finish(&mut self) {
        while self.complete() {}
    }
}

/// Spec: `StartingClip`, `HandOverSlowdown`. Initialization consumes the completed playing result.
fn initialize(before: State, data: &AnimationInputs) -> (State, Answer) {
    let selection_inputs = Inputs {
        figure: data.selected_from,
        ..data.inputs
    };
    let initialized = start(&selection_inputs, &data.selected_clip);
    let after = State {
        playhead: initialized.unwrap_or(before.playhead),
        ..before
    };
    let answer = match initialized {
        None => Answer::KeepPlayhead,
        Some(p) if p.step == Step::FIRST => Answer::StartFirst,
        Some(_) => Answer::StartSecond,
    };
    (after, answer)
}

fn decide(
    rule: AnimationDecision,
    before: State,
    data: &AnimationInputs,
) -> Option<OfferedDecision> {
    let inputs = &data.inputs;
    let clip = &data.current_clip;
    let rules = &data.rules;
    let mut after = before;
    let mut sound = None;
    let answer = match rule {
        AnimationDecision::DashBegan => {
            after = dash_began(before, inputs, rules);
            Answer::Change
        }
        AnimationDecision::StandingRestart => {
            after.playhead = standing_restart(inputs, rules)?;
            Answer::Change
        }
        AnimationDecision::WallJumpReady => {
            after.playhead = wall_jump_ready(inputs, clip)?;
            Answer::Change
        }
        AnimationDecision::FastFall => {
            after.playhead = fast_fall(before.playhead.step, inputs, clip, rules)?;
            Answer::Change
        }
        AnimationDecision::RisingHold => {
            after.playhead = rising_hold(before.playhead, inputs, clip, rules)?;
            Answer::Change
        }
        AnimationDecision::ClipPlaying => {
            (after, sound) = play(before, inputs, data, rules);
            if before.playhead.time_left > Frames::LAST {
                Answer::CountDown
            } else {
                Answer::Advance
            }
        }
        AnimationDecision::SelectedClip => {
            let (initialized, answer) = initialize(before, data);
            after = initialized;
            answer
        }
        AnimationDecision::DashLoss => {
            after = dash_lost(before, inputs);
            Answer::Change
        }
    };
    let needed = match rule {
        AnimationDecision::DashBegan => inputs.dash == crate::inputs::Dash::Began,
        AnimationDecision::DashLoss => inputs.moves[crate::inputs::MoveEvent::DashLost],
        AnimationDecision::StandingRestart
        | AnimationDecision::WallJumpReady
        | AnimationDecision::FastFall
        | AnimationDecision::RisingHold
        | AnimationDecision::ClipPlaying
        | AnimationDecision::SelectedClip => true,
    };
    if !needed {
        return None;
    }
    Some(OfferedDecision {
        rule,
        before,
        inputs: if rule == AnimationDecision::SelectedClip {
            Inputs {
                figure: data.selected_from,
                ..*inputs
            }
        } else {
            *inputs
        },
        clip: if rule == AnimationDecision::SelectedClip {
            data.selected_clip.clone()
        } else {
            clip.clone()
        },
        paces: data.paces.clone(),
        rules: rules.clone(),
        answer,
        after,
        sound,
    })
}
