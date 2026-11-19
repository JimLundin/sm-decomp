//! Complete unselected stimuli retained at an ordinary opportunity.

use canonical::model::Pinned;
use serde::{Deserialize, Serialize};
use types::{Buttons, Item, Parity, Set};

/// Camera room/settings, excluding selected movement and camera quantities.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CameraInputs {
    /// Complete screen permission run.
    pub screens: Vec<Screen>,
    /// Width/height in screens.
    pub size: [u16; 2],
    /// Normal/rising camera aim heights.
    pub heights: [u16; 2],
    /// Actual current lead selection.
    pub lead: Lead,
}

/// Canonical current screen permission, independent of the clean room's content codec.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Screen {
    /// Whole screen is permitted.
    Open,
    /// Following is pushed out of this screen.
    Closed,
    /// The view is restricted to the screen's top.
    Short,
}

/// Current ordinary lead selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lead {
    /// Ordinary directional lead.
    Ahead,
    /// Left-edge lead.
    Left,
    /// Right-edge lead.
    Right,
    /// Near lead.
    Near,
}

/// Unsupported activity cannot be accepted as unchanged constructor inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Activity {
    /// Independently qualified inactive family.
    Inactive,
    /// Requires another current producer/constructor contract.
    Unqualified,
}

/// One independently admitted ordinary opportunity's complete external inputs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inputs {
    /// Actual current held actions, under independently checked bindings.
    pub held: Buttons,
    /// Newly pressed actions.
    pub pressed: Buttons,
    /// Current equipped items.
    pub items: Set<Item>,
    /// Current semantic frame parity.
    pub parity: Parity,
    /// Current pre-Animation energy; health is unselected.
    pub energy: u16,
    /// Borrowed current liquid/slowdown inputs.
    pub liquid: samus_appearance::Liquid,
    /// Actual borrowed destination occupancy; no live Sender is invented.
    pub sounds_waiting: samus_appearance::SoundsWaiting,
    /// Complete current Contact guard meanings.
    pub contact: Set<samus_appearance::ContactSuppressor>,
    /// Current selection-time cinematic meaning.
    pub cinematic: samus_appearance::CinematicActivity,
    /// Current room/query geometry.
    pub geometry: crate::production::world::MovementWorld,
    /// Current camera room/settings.
    pub camera: CameraInputs,
    /// Unsupported clock, interruption, AI/script or foreign writer activity.
    pub activity: Activity,
}

/// Complete captured external-input schema, including geometry and constructor fields.
pub struct InputsPin;
impl Pinned for InputsPin {
    type State = Inputs;
    const NAME: &'static str = "production-connected-inputs";
    const SHAPE: u64 = 0xf64e_9370_17ae_47af;
}
