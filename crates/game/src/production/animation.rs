//! Checked immutable animation content and the clean appearance entry.

use samus_appearance::{
    AppearanceState, CannonDelay, CannonEvent, ChargeGlowPosition, Clip, ColourSequences, Content,
    Figure, Frame, FrameOutputs, FrameSound, Inputs, Paces, PaletteOpportunity, Rules, Selection,
    State, Stride, VisorDelay, VisorStep,
};
use types::AnimationSound;

/// One imported semantic figure and its complete clip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FigureClip {
    /// The pose and sprite selecting this clip.
    pub figure: Figure,
    /// Every shown step and control in its animation.
    pub clip: Clip,
}

/// Immutable clips and run paces selected at admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppearanceContent {
    figures: Vec<FigureClip>,
    paces: Paces,
}

/// A malformed or unavailable semantic clip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipFault {
    /// Two entries for one figure disagree.
    Conflicting(Figure),
    /// A required figure has no imported clip.
    Missing(Figure),
}

impl AppearanceContent {
    /// Admit an exact imported table; missing figures remain checked at the selected step.
    ///
    /// # Errors
    /// Conflicting duplicate entries are refused.
    pub fn new(figures: Vec<FigureClip>, paces: Paces) -> Result<Self, ClipFault> {
        for (index, entry) in figures.iter().enumerate() {
            if figures
                .iter()
                .take(index)
                .any(|prior| prior.figure == entry.figure && prior.clip != entry.clip)
            {
                return Err(ClipFault::Conflicting(entry.figure));
            }
        }
        Ok(Self { figures, paces })
    }

    /// The imported clip, without advancing animation or choosing a different figure.
    #[must_use]
    pub fn get(&self, figure: Figure) -> Option<&Clip> {
        self.figures
            .iter()
            .find(|entry| entry.figure == figure)
            .map(|entry| &entry.clip)
    }
}

struct ClipProvider<'a> {
    content: &'a AppearanceContent,
    missing: std::cell::Cell<Option<Figure>>,
    discarded: Clip,
}

impl Content for ClipProvider<'_> {
    fn clip(&self, figure: &Figure) -> &Clip {
        if let Some(clip) = self.content.get(*figure) {
            return clip;
        }
        self.missing.set(Some(*figure));
        // Finish only the private candidate so a failed lookup cannot publish a default clip.
        &self.discarded
    }

    fn paces(&self) -> &Paces {
        &self.content.paces
    }
}

/// The parts of a frame of her appearance these owners play. Her item controls and the palette
/// opportunity have no owner in these loops: the frame skips the controls and has no palette
/// opportunity, so neither changes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Parts {
    /// Her stride, before the animation update.
    pub stride: Option<Stride>,
    /// The animation update, with movement's results.
    pub animation: Option<Inputs>,
    /// Her pose selection.
    pub selection: Option<Selection>,
}

/// The cannon delay and colour sequences of a frame that neither runs her item controls nor
/// meets a palette opportunity: the update gives both back as they came, so any value does,
/// and neither is kept.
const UNOWNED: (CannonDelay, ColourSequences) = (
    CannonDelay::INITIAL,
    ColourSequences {
        visor_delay: VisorDelay(0),
        visor_step: VisorStep(0),
        charge_glow_position: ChargeGlowPosition(0),
    },
);

/// Play one frame of her animation through the clean appearance entry.
///
/// # Errors
/// A figure the frame shows has no admitted clip; `state` is unchanged.
pub(super) fn play(
    state: &mut State,
    content: &AppearanceContent,
    rules: &Rules,
    parts: Parts,
) -> Result<FrameOutputs, ClipFault> {
    let shown = parts
        .animation
        .iter()
        .flat_map(|inputs| [inputs.figure, inputs.figure_after])
        .chain(parts.stride.map(|stride| stride.figure));
    for figure in shown {
        if content.get(figure).is_none() {
            return Err(ClipFault::Missing(figure));
        }
    }
    let provider = ClipProvider {
        content,
        missing: std::cell::Cell::new(None),
        discarded: Clip {
            instructions: Vec::new(),
            kick: None,
            fast_fall: None,
            rising: None,
            contacts: std::collections::BTreeSet::new(),
        },
    };
    let (cannon, colours) = UNOWNED;
    let frame = Frame {
        cannon: CannonEvent::ControlsSkipped,
        stride: parts.stride,
        animation: parts.animation,
        selection: parts.selection,
        palette: PaletteOpportunity::Absent,
        passage: None,
        drawing: None,
    };
    let before = AppearanceState {
        animation: *state,
        cannon,
        colours,
    };
    let (after, outputs) = samus_appearance::update(before, &frame, &provider, rules);
    if let Some(figure) = provider.missing.get() {
        return Err(ClipFault::Missing(figure));
    }
    *state = after.animation;
    Ok(outputs)
}

/// Play one frame's animation update alone, giving the sound it asks for, if any.
///
/// # Errors
/// A figure the frame shows has no admitted clip; `state` is unchanged.
pub(super) fn advance(
    state: &mut State,
    content: &AppearanceContent,
    rules: &Rules,
    inputs: &Inputs,
) -> Result<Option<AnimationSound>, ClipFault> {
    let outputs = play(
        state,
        content,
        rules,
        Parts {
            animation: Some(*inputs),
            ..Parts::default()
        },
    )?;
    Ok(outputs.sounds.into_iter().find_map(|sound| match sound {
        FrameSound::Animation(sound) => Some(sound),
        FrameSound::Opportunity(_) => None,
    }))
}
