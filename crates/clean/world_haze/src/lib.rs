//! The single-colour Ceres backdrop haze during ordinary room time.

mod catalogue;

pub use catalogue::{Behaviour, BehaviourId, Kind, Switch, catalogue};
use types::Colour;
pub use types::haze::{
    HAZE_BAND_COUNT, HazeBands, HazeColour, HazeFade, HazeLevel, HazePeak, HazeState,
};

/// What display preparation should do with the band backing for this opportunity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BandIntent {
    /// Spec: `HazeBrightens`. Prepare the gradient using the resulting colour and bands,
    /// even if the levels have not changed.
    Prepare,
    /// Spec: `HazeSettles`, `HazeHolds`. Keep the current backing available to display;
    /// the retained colour and bands are in the resulting state.
    Retain,
}

/// The haze's instruction for the third background in the colour combination.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThirdBackground {
    /// Leave the third background out of the combination.
    Excluded,
}

/// One backdrop decision, not a composed display or a display transfer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayIntent {
    /// Prepare or retain the gradient described by the resulting haze state.
    pub bands: BandIntent,
    /// Black base backdrop under the haze.
    pub backdrop: Colour,
    /// Which treatment the haze requests for the third background.
    pub third_background: ThirdBackground,
}

/// Completed state and display intent from one ordinary opportunity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HazeStep {
    /// Carry this complete state into the next opportunity or into a save.
    pub state: HazeState,
    /// Hand this decision and the resulting gradient to display preparation before publication.
    pub display: DisplayIntent,
    /// The specification entry applied in this opportunity.
    pub behaviour: BehaviourId,
}

/// Spec: `HazeBrightens`, `HazeSettles`, `HazeHolds`. Advance by one ordinary room opportunity.
///
/// The room owns placement and creation. With no opportunity, preserve the saved haze and
/// display intent. Departure stays with the room. Display preparation combines the backgrounds.
#[must_use]
pub fn ordinary_opportunity(mut state: HazeState) -> HazeStep {
    let (behaviour, bands) = match state.fade {
        HazeFade::Brightening(peak) => {
            if let Some(level) = peak.level_to_prepare() {
                state.bands = gradient(level);
                state.fade = HazeFade::Brightening(level.following_peak());
                (BehaviourId::HazeBrightens, BandIntent::Prepare)
            } else {
                state.fade = HazeFade::Steady;
                (BehaviourId::HazeSettles, BandIntent::Retain)
            }
        }
        HazeFade::Steady => (BehaviourId::HazeHolds, BandIntent::Retain),
    };
    HazeStep {
        state,
        display: DisplayIntent {
            bands,
            backdrop: Colour::BLACK,
            third_background: ThirdBackground::Excluded,
        },
        behaviour,
    }
}

/// Spec: `HazeBrightens`. Start at the bright edge and drop one level per band toward
/// the dim end, stopping at zero. At index `i` this gives `max(0, peak + i - 15)`.
fn gradient(mut level: HazeLevel) -> HazeBands {
    let mut bands = [level; HAZE_BAND_COUNT];
    for band in bands.iter_mut().rev() {
        *band = level;
        level = level.dimmed();
    }
    HazeBands(bands)
}
