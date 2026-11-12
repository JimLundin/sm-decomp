//! The catalogue: one behaviour per entry of the HUD's spec, with its kind, its switch and how
//! the spec checks it.

use enum_map::Enum;
use types::{Checked, Kind, Switch};

/// An entry of the HUD's spec; each variant is named as the spec names its entry.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Enum)]
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

/// Every behaviour of the HUD's spec, one row per entry, each naming its entry; listed in the
/// spec's order, though the order means nothing.
pub const CATALOGUE: [Behaviour; 19] = [
    Behaviour {
        id: BehaviourId::EnergyGauge,
        summary: "Her energy's last two digits, and one tank per 100 of her capacity, full while \
                  whole hundreds remain; drawn again only when her energy changes.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::AmmoCounts,
        summary: "The missiles' count in three digits, the super missiles' and the power bombs' \
                  in two, each drawn again only when it changes, and only once she can carry \
                  some.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::AutoReserveSign,
        summary: "While the reserve tanks refill on their own, the reserve sign shows, lit with \
                  energy in them and dim when they are empty.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SelectionHighlight,
        summary: "The selected item's icon shows in the highlighted colours, every other in the \
                  plain ones; when the selection changes the new icon lights and the old one \
                  goes plain.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SwitchClick,
        summary: "A click sounds when the highlight moves, unless she spin jumps, wall jumps, the \
                  grapple beam is out or time is stopped.",
        kind: Kind::Output,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::OneShotFlash,
        summary: "The icon of the one-shot item flashes, 16 frames plain, then 16 highlighted.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::IconsAppear,
        summary: "A pickup puts its icon on the HUD in the plain colours, unless the icon \
                  already shows.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::CyclingItems,
        summary: "Select moves the selection to the next item she has, in the HUD's order, then \
                  back to none.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::CancellingTheItem,
        summary: "Item cancel puts the selection back to none; so does a shot the weapons \
                  cancel.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::OneShotItem,
        summary: "Selecting an item with item cancel held makes it the one-shot item, the one the \
                  weapons drop after a shot; selecting without it clears that.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Switching,
        summary: "Each switch tells the weapons and her look which item she switched to.",
        kind: Kind::Output,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Exploring,
        summary: "The cell she is in becomes explored.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::MiniMapWindow,
        summary: "The mini-map shows the 5 × 3 cells around her: the explored ones with their \
                  pictures, the ones the map charts once she has it, the rest blank.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::HerBlink,
        summary: "Her cell blinks, 8 frames in the blink colours and 8 as drawn.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::MiniMapStill,
        summary: "Some frames the mini-map doesn't update: nothing is explored and its cells \
                  keep what they show, blink included.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BossBlanksTheMap,
        summary: "On entering the room of Kraid, Draygon, Crocomire, Phantoon or Ridley (Ceres's \
                  or Norfair's) while it lives, and when Mother Brain's first form falls (her \
                  brain in the tank is destroyed), the mini-map goes blank and the cells the \
                  boss names count as explored. Entering the room is a step of its own, outside \
                  a frame of play; her fall comes during a frame.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::DiagonalPicture,
        summary: "Where her cell shows the diagonal picture (cells drawn as a diagonal), the cell \
                  over hers becomes explored too.",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::FirstColumnsShift,
        summary: "With her cell in the map's columns 0 and 1, the mini-map shows the cells 32 \
                  columns along and a row higher.",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::LastColumnsShift,
        summary: "With her cell in the map's columns 62 and 63, the mini-map's pictures are its \
                  own cells', but whether each is explored or charted is read 32 columns back \
                  and a row lower, and past column 63 the pictures are another map's.",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
];

#[cfg(test)]
mod tests {
    use super::{BehaviourId, CATALOGUE};
    use crate::tests::names_each_member_once;

    #[test]
    fn the_catalogue_names_each_entry_once() {
        assert!(names_each_member_once::<BehaviourId, _>(
            &CATALOGUE,
            |behaviour| behaviour.id
        ));
    }
}
