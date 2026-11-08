//! The catalogue: one behaviour per entry of the HUD's spec, with its kind, its switch and how
//! the spec checks it.

use clean_types::{Checked, Kind, Switch};

/// An entry of the HUD's spec.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BehaviourId {
    /// Spec: EnergyGauge.
    EnergyGauge,
    /// Spec: AmmoCounts.
    AmmoCounts,
    /// Spec: AutoReserveSign.
    AutoReserveSign,
    /// Spec: SelectionHighlight.
    SelectionHighlight,
    /// Spec: SwitchClick.
    SwitchClick,
    /// Spec: OneShotFlash.
    OneShotFlash,
    /// Spec: IconsAppear.
    IconsAppear,
    /// Spec: CyclingItems.
    CyclingItems,
    /// Spec: CancellingTheItem.
    CancellingTheItem,
    /// Spec: OneShotItem.
    OneShotItem,
    /// Spec: Switching.
    Switching,
    /// Spec: Exploring.
    Exploring,
    /// Spec: MiniMapWindow.
    MiniMapWindow,
    /// Spec: HerBlink.
    HerBlink,
    /// Spec: MiniMapStill.
    MiniMapStill,
    /// Spec: BossBlanksTheMap.
    BossBlanksTheMap,
    /// Spec: DiagonalPicture.
    DiagonalPicture,
    /// Spec: FirstColumnsShift.
    FirstColumnsShift,
    /// Spec: LastColumnsShift.
    LastColumnsShift,
}

impl BehaviourId {
    /// The entry's name in the spec.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::EnergyGauge => "EnergyGauge",
            Self::AmmoCounts => "AmmoCounts",
            Self::AutoReserveSign => "AutoReserveSign",
            Self::SelectionHighlight => "SelectionHighlight",
            Self::SwitchClick => "SwitchClick",
            Self::OneShotFlash => "OneShotFlash",
            Self::IconsAppear => "IconsAppear",
            Self::CyclingItems => "CyclingItems",
            Self::CancellingTheItem => "CancellingTheItem",
            Self::OneShotItem => "OneShotItem",
            Self::Switching => "Switching",
            Self::Exploring => "Exploring",
            Self::MiniMapWindow => "MiniMapWindow",
            Self::HerBlink => "HerBlink",
            Self::MiniMapStill => "MiniMapStill",
            Self::BossBlanksTheMap => "BossBlanksTheMap",
            Self::DiagonalPicture => "DiagonalPicture",
            Self::FirstColumnsShift => "FirstColumnsShift",
            Self::LastColumnsShift => "LastColumnsShift",
        }
    }
}

/// A behaviour of the HUD, as the catalogue lists it.
pub struct Behaviour {
    /// The spec entry.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// What kind of behaviour it is.
    pub kind: Kind,
    /// How it can be adjusted: `Parameter` where `Rules` holds its values.
    pub switch: Switch,
    /// How the spec checks it.
    pub checked: Checked,
}

/// A behaviour whose values are fixed and which vectors check.
const fn fixed(id: BehaviourId, kind: Kind, summary: &'static str) -> Behaviour {
    Behaviour {
        id,
        summary,
        kind,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    }
}

/// A behaviour whose values `Rules` holds and which vectors check.
const fn parameter(id: BehaviourId, kind: Kind, summary: &'static str) -> Behaviour {
    Behaviour {
        id,
        summary,
        kind,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    }
}

/// Every behaviour of the HUD's spec, in the spec's order.
pub const CATALOGUE: [Behaviour; 19] = [
    fixed(
        BehaviourId::EnergyGauge,
        Kind::Mechanic,
        "Her energy's last two digits, and one tank per 100 of her capacity, full while whole \
         hundreds remain; drawn again only when her energy changes.",
    ),
    fixed(
        BehaviourId::AmmoCounts,
        Kind::Mechanic,
        "The missiles' count in three digits, the super missiles' and the power bombs' in two, \
         each drawn again only when it changes, and only once she can carry some.",
    ),
    fixed(
        BehaviourId::AutoReserveSign,
        Kind::Mechanic,
        "While the reserve tanks refill on their own, the reserve sign shows, lit with energy in \
         them and dim when they are empty.",
    ),
    fixed(
        BehaviourId::SelectionHighlight,
        Kind::Mechanic,
        "The selected item's icon shows in the highlighted colours, every other in the plain \
         ones; when the selection changes the new icon lights and the old one goes plain.",
    ),
    fixed(
        BehaviourId::SwitchClick,
        Kind::Output,
        "A click sounds when the highlight moves, unless she spin jumps, wall jumps, the grapple \
         beam is out or time is stopped.",
    ),
    parameter(
        BehaviourId::OneShotFlash,
        Kind::Mechanic,
        "The icon of the one-shot item flashes, 16 frames highlighted and 16 plain.",
    ),
    fixed(
        BehaviourId::IconsAppear,
        Kind::Mechanic,
        "A pickup puts its icon on the HUD in the plain colours, unless the icon already shows.",
    ),
    fixed(
        BehaviourId::CyclingItems,
        Kind::Mechanic,
        "Select moves the selection to the next item she has, in the HUD's order, then back to \
         none.",
    ),
    fixed(
        BehaviourId::CancellingTheItem,
        Kind::Mechanic,
        "Item cancel puts the selection back to none; so does a shot the weapons cancel.",
    ),
    fixed(
        BehaviourId::OneShotItem,
        Kind::Mechanic,
        "Selecting an item with item cancel held makes it the one-shot item, the one the weapons \
         drop after a shot; selecting without it clears that.",
    ),
    fixed(
        BehaviourId::Switching,
        Kind::Output,
        "Each switch tells the weapons and her look which item she switched to.",
    ),
    fixed(
        BehaviourId::Exploring,
        Kind::Mechanic,
        "The cell she is in becomes explored.",
    ),
    fixed(
        BehaviourId::MiniMapWindow,
        Kind::Mechanic,
        "The mini-map shows the 5 by 3 cells around her: the explored ones with their pictures, \
         the ones the map charts once she has it, the rest blank.",
    ),
    parameter(
        BehaviourId::HerBlink,
        Kind::Mechanic,
        "Her cell blinks, 8 frames in the blink colours and 8 as drawn.",
    ),
    fixed(
        BehaviourId::MiniMapStill,
        Kind::Mechanic,
        "Some frames the mini-map doesn't update: nothing is explored and its cells keep what \
         they show, blink included.",
    ),
    parameter(
        BehaviourId::BossBlanksTheMap,
        Kind::Mechanic,
        "On entering a boss's room while it lives, and when Mother Brain's first form falls, the \
         mini-map goes blank and the cells the boss names count as explored.",
    ),
    parameter(
        BehaviourId::DiagonalPicture,
        Kind::Quirk,
        "Where her cell shows the diagonal picture, the cell over hers becomes explored too.",
    ),
    fixed(
        BehaviourId::FirstColumnsShift,
        Kind::Quirk,
        "With her cell in the map's columns 0 and 1, the mini-map shows the cells 32 columns \
         along and a row higher.",
    ),
    fixed(
        BehaviourId::LastColumnsShift,
        Kind::Quirk,
        "With her cell in the map's columns 62 and 63, the mini-map's pictures are its own \
         cells', but whether each is explored or charted is read 32 columns back and a row \
         lower, and past column 63 the pictures are another map's.",
    ),
];
