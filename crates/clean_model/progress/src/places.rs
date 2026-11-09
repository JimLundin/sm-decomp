//! What the places remember: the item locations emptied and the doors that stay open, by id, and
//! how she has each area's map.

use std::collections::BTreeSet;

/// Places of one sort she has changed for good, by the id a room's population gives them.
///
/// They are the item locations emptied and the doors that stay open. A population may place one
/// without an id; then nothing of it is remembered, and asked about, it reads as unchanged: an
/// item still in its place, a door closed. Spec: EmptiedLocations, Spec: DoorsStayOpen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Remembered<Id>(BTreeSet<Id>);

impl<Id: Ord> Remembered<Id> {
    /// The places of `ids`, as a saved file holds them.
    pub fn of(ids: impl IntoIterator<Item = Id>) -> Self {
        Self(ids.into_iter().collect())
    }

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

/// How she has an area's map.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MapStatus {
    /// She doesn't have it.
    #[default]
    None,
    /// A map station gave it, or she has left the area since the room that gives it on arrival.
    Downloaded,
    /// The room that gives it on arrival gave it, and she hasn't left the area since: the quirk
    /// of Tourian, which has no map station. Play reads it as `Downloaded`; the saved file is what
    /// reads it, which keeps the two apart. Spec: MapOnArrival.
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
