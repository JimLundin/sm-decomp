//! Spec: `PerformedAnimationDecisions`. Ordinary updating completes the same rule engine.
use crate::inputs::{Content, Inputs};
use crate::rules::Rules;
use crate::state::State;
use crate::suspended::{AnimationInputs, PublishedSound, SuspendedAppearance};
use types::AnimationSound;

/// Complete an ordinary animation update without pauses, in the spec's required order.
#[must_use]
pub fn update(
    state: State,
    inputs: &Inputs,
    content: &impl Content,
    rules: &Rules,
) -> (State, Option<AnimationSound>) {
    let data = AnimationInputs {
        inputs: *inputs,
        selected_from: inputs.figure,
        current_clip: content.clip(&inputs.figure).clone(),
        selected_clip: content.clip(&inputs.figure_after).clone(),
        paces: content.paces().clone(),
        rules: rules.clone(),
    };
    let mut work =
        SuspendedAppearance::new(state, Some(data), SuspendedAppearance::animation_steps());
    work.finish();
    let sound = work.sounds.into_iter().find_map(|sound| match sound {
        PublishedSound::Animation(sound) => Some(sound),
        PublishedSound::Appearance(_) => None,
    });
    (work.state, sound)
}
