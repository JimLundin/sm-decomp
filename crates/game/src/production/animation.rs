//! Checked immutable animation content and the clean appearance entry.

use samus_appearance::{
    AnimationContent, AppearanceState, Clip, Figure, Frame, FrameOutputs, FramePart, Inputs,
    ItemControls, MissingClip, Paces, PaletteOpportunity, Rules,
};
use types::Set;

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

impl AnimationContent for AppearanceContent {
    fn clip(&self, figure: &Figure) -> Option<&Clip> {
        self.get(*figure)
    }

    fn paces(&self) -> &Paces {
        &self.paces
    }
}

/// The parts of a frame of play in these loops: her work is prepared, she moves, and she is
/// drawn. A passage's frame doesn't reach her appearance.
const PLAYED: [FramePart; 3] = [
    FramePart::Preparation,
    FramePart::Movement,
    FramePart::Drawing,
];

/// A frame of play that brings each of `inputs`' facts once.
///
/// Her item controls and the palette opportunity have no producer in these loops: the frame
/// reports the controls skipped and no palette opportunity, so the cannon delay and the colour
/// sequences carry in her State as they came.
#[must_use]
pub(super) fn frame_of_play(inputs: Inputs) -> Frame {
    let mut parts = Set::default();
    for part in PLAYED {
        parts[part] = true;
    }
    Frame {
        inputs,
        parts,
        item_controls: ItemControls::Skipped,
        palette: PaletteOpportunity::Absent,
    }
}

/// Play one frame of her appearance through the clean entry, from her whole State.
///
/// # Errors
/// A figure the frame shows has no admitted clip; `state` is unchanged.
pub(super) fn play(
    state: &mut AppearanceState,
    content: &AppearanceContent,
    rules: &Rules,
    frame: &Frame,
) -> Result<FrameOutputs, ClipFault> {
    let (after, outputs) = samus_appearance::update(*state, frame, content, rules)
        .map_err(|MissingClip(figure)| ClipFault::Missing(figure))?;
    *state = after;
    Ok(outputs)
}
