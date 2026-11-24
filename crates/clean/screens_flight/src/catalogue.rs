//! The catalogue: one behaviour for each entry of the flight's spec.

use enum_map::{Enum, EnumMap};
use types::{Checked, Kind, Switch, by_member};

/// The flight's behaviours, one for each spec entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum BehaviourId {
    /// The flight's first frame makes it ready as the scene says.
    FlightIsMadeReady,
    /// The picture appears once no track waits.
    PictureAppears,
    /// The gunship grows as it rushes at the camera.
    GunshipRushesIn,
    /// The stars streak past ever faster.
    StarsRushPast,
    /// The view cuts to the gunship from behind, with a white flash.
    ViewTurnsAway,
    /// The gunship shrinks and drifts left as it flies away.
    GunshipFliesAway,
    /// The white flash fades to black.
    FlashFades,
    /// Each placed sprite drifts sideways.
    SpritesDrift,
    /// The picture changes to the station's view.
    StationComesIntoView,
    /// The label is typed one step at a time.
    LabelIsTyped,
    /// The picture darkens one level every `d + 1` frames.
    PictureDarkens,
    /// Once black and blanked, the flight is over.
    FlightEnds,
    /// The plane is drawn turned by its angle and scaled by its zoom.
    PlaneTransforms,
}

/// A behaviour of the flight, as its spec entry gives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// Its spec entry.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// What kind of behaviour it is.
    pub kind: Kind,
    /// How it can be adjusted.
    pub switch: Switch,
    /// How the spec checks it.
    pub checked: Checked,
}

/// The catalogue's rows, one per spec entry, in the spec's order.
const ROWS: [Behaviour; 13] = [
    Behaviour {
        id: BehaviourId::FlightIsMadeReady,
        summary: "the flight's first frame makes it ready: the plane shows the gunship from the \
                  front, turned and small, the stars are placed to rush, the screen is dark and \
                  the flash black",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PictureAppears,
        summary: "the flight waits in the dark while its music is still queued; then the \
                  picture comes on at full brightness and the gunship starts to rush in",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::GunshipRushesIn,
        summary: "the gunship grows as it rushes at the camera, its zoom falling by 16 each \
                  frame until it would be too large, and then the view turns",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::StarsRushPast,
        summary: "the stars streak past ever faster, their speed rising by half a pixel per \
                  frame, the stars and the plane moving by it down and to the right alike",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ViewTurnsAway,
        summary: "the view cuts to the gunship from behind with a white flash; the rushing \
                  stars go and the sprites of the scene's turn appear",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::GunshipFliesAway,
        summary: "the gunship shrinks as it flies away, slowly then twice as fast, drifting \
                  left an eighth of a pixel each frame; once small enough the view changes",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::FlashFades,
        summary: "the white flash fades one level per frame on each of its colours, to black",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SpritesDrift,
        summary: "each placed sprite drifts sideways by its own speed every frame, those that \
                  wrap coming back in on the other side of 512 px",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::StationComesIntoView,
        summary: "the picture changes to the station's view, on which the label is typed; the \
                  plane is put back square and at its size",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::LabelIsTyped,
        summary: "the label is typed one step at a time, each showing its letter as it begins \
                  and holding for its frames; after the last the picture starts to fade out",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PictureDarkens,
        summary: "the picture darkens one level every d + 1 frames; at level 1 the next step \
                  blanks it",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::FlightEnds,
        summary: "once the picture is black and blanked the flight is over: its plane and \
                  sprites are cleared, and the next frame starts the new game",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PlaneTransforms,
        summary: "every frame the plane is drawn turned by its angle and scaled by its zoom, \
                  scrolled to its position",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
];

/// The catalogue, by entry.
#[must_use]
pub fn catalogue() -> EnumMap<BehaviourId, Behaviour> {
    by_member(ROWS, |row| row.id)
}

#[cfg(test)]
mod tests {
    use super::catalogue;

    #[test]
    fn each_entry_has_its_own_row() {
        for (id, row) in catalogue() {
            assert_eq!(row.id, id);
        }
    }
}
