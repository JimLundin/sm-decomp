//! What the places remember (spec §3): the item locations emptied and the doors that stay open,
//! by id, and how she has each area's map.

use std::collections::BTreeSet;

/// Places of one sort she has changed for good, by the id a room's population gives them.
///
/// They are the item locations emptied and the doors that stay open. A population may place one
/// without an id; then nothing of it is remembered, and asked about, it reads as unchanged: an
/// item still in its place, a door closed. Spec: EmptiedLocations, Spec: DoorsStayOpen.
///
/// Its set is every id remembered, which a caller reads back whole and sets, as a saved file
/// holds them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Remembered<Id>(pub BTreeSet<Id>);

impl<Id: Ord> Remembered<Id> {
    /// Remembers the place of `id`; a place without one is not remembered.
    pub(crate) fn remember(&mut self, id: Option<Id>) {
        self.0.extend(id);
    }

    /// Whether the place of `id` is remembered; a place without one never is.
    #[must_use]
    pub fn contains(&self, id: Option<Id>) -> bool {
        id.is_some_and(|id| self.0.contains(&id))
    }
}

/// None remembered, as a new file starts.
impl<Id> Default for Remembered<Id> {
    fn default() -> Self {
        Self(BTreeSet::new())
    }
}

/// How she has an area's map.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MapStatus {
    /// She doesn't have it.
    #[default]
    Unmapped,
    /// A map station gave it, or a door has taken her into another area since the room that
    /// gives it on arrival.
    Downloaded,
    /// The map a room gives on arrival: Tourian has no map station, so its first room gives its
    /// map as she enters it. It stays `OnArrival` until a door takes her into another area,
    /// when it becomes `Downloaded`. Play reads the two the same; only the saved file
    /// tells them apart, when she saves before leaving. Spec: MapOnArrival.
    OnArrival,
}

impl MapStatus {
    /// Whether the map is known: the mini-map and the pause map show the area's rooms, however
    /// she got it. Spec: MapsByArea.
    #[must_use]
    pub const fn is_known(self) -> bool {
        matches!(self, Self::Downloaded | Self::OnArrival)
    }
}
