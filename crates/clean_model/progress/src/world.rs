//! What has happened in the world, and what its places remember: the events, the zebetites, the
//! bosses defeated, the emptied item places, the open doors, the maps, the stations saved at and
//! the count of items placed.

use std::collections::BTreeSet;

use clean_types::{Area, BossKind, DoorId, Event, LocationId, StationId};

use crate::count::Count;
use crate::per_area::PerArea;

/// How she has an area's map.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AreaMap {
    /// She doesn't have it.
    #[default]
    None,
    /// A map station gave it, or she left the area after the room that gives it on arrival.
    Downloaded,
    /// The room that gives it on arrival gave it, and she hasn't left the area since. Play reads
    /// it as `Downloaded`; only the saved file tells the two apart.
    OnArrival,
}

impl AreaMap {
    /// Whether the map is known: the mini-map and the pause map show the area's rooms.
    /// Spec: MapsByArea.
    #[must_use]
    pub const fn is_known(self) -> bool {
        matches!(self, Self::Downloaded | Self::OnArrival)
    }
}

/// What has happened in the world, and what its places remember. A new file has none of it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorldMemory {
    /// The events that have happened. Spec: EventsHappen.
    pub events: BTreeSet<Event>,
    /// How many of Tourian's zebetites she has destroyed, 0 to 4. Spec: ZebetitesCounted.
    pub zebetites: Count,
    /// The kinds of boss defeated in each area. Spec: BossesDefeated.
    pub defeated: PerArea<BTreeSet<BossKind>>,
    /// The places whose item she has taken. Spec: EmptiedLocations.
    pub emptied: BTreeSet<LocationId>,
    /// The doors that stay open. Spec: DoorsStayOpen.
    pub opened: BTreeSet<DoorId>,
    /// How she has each area's map. Spec: MapsByArea, Spec: MapOnArrival.
    pub maps: PerArea<AreaMap>,
    /// The save stations she has saved at in each area. Spec: StationsSavedAt.
    pub stations: PerArea<BTreeSet<StationId>>,
    /// How many items rooms have placed as they loaded, over the whole run.
    /// Spec: ItemsPlacedCount.
    pub items_placed: Count,
}

impl WorldMemory {
    /// An event that happens has happened from then on; one that happens again changes nothing.
    /// Spec: EventsHappen.
    pub(crate) fn happen(&mut self, event: Event) {
        self.events.insert(event);
    }

    /// Whether `event` has happened. Spec: EventsHappen.
    #[must_use]
    pub fn has_happened(&self, event: Event) -> bool {
        self.events.contains(&event)
    }

    /// A zebetite destroyed counts one more. Spec: ZebetitesCounted.
    pub(crate) const fn destroy_zebetite(&mut self) {
        self.zebetites = self.zebetites.plus(Count::ONE);
    }

    /// A boss defeated is remembered by its kind, in the area it was defeated in.
    /// Spec: BossesDefeated.
    pub(crate) fn defeat(&mut self, area: Area, kind: BossKind) {
        self.defeated.get_mut(area).insert(kind);
    }

    /// Whether a boss of any of `kinds` is defeated in `area`; a boss defeated in another area
    /// says nothing of this one. Spec: BossesDefeated.
    #[must_use]
    pub fn is_any_defeated(&self, area: Area, kinds: &BTreeSet<BossKind>) -> bool {
        !self.defeated.get(area).is_disjoint(kinds)
    }

    /// The item taken from a place with an id leaves it emptied for good; one placed without an
    /// id leaves nothing remembered, so it appears again each time its room loads.
    /// Spec: EmptiedLocations.
    pub(crate) fn empty(&mut self, location: Option<LocationId>) {
        if let Some(location) = location {
            self.emptied.insert(location);
        }
    }

    /// Whether the place is emptied; a place without an id never is. Spec: EmptiedLocations.
    #[must_use]
    pub fn is_emptied(&self, location: Option<LocationId>) -> bool {
        location.is_some_and(|location| self.emptied.contains(&location))
    }

    /// A door with an id, once opened, stays open; one placed without an id is not remembered.
    /// Spec: DoorsStayOpen.
    pub(crate) fn open(&mut self, door: Option<DoorId>) {
        if let Some(door) = door {
            self.opened.insert(door);
        }
    }

    /// Whether the door is open for good; a door without an id reads as closed.
    /// Spec: DoorsStayOpen.
    #[must_use]
    pub fn is_open(&self, door: Option<DoorId>) -> bool {
        door.is_some_and(|door| self.opened.contains(&door))
    }

    /// A map station gives the map of its area, downloaded. Spec: MapsByArea.
    pub(crate) const fn download_map(&mut self, area: Area) {
        *self.maps.get_mut(area) = AreaMap::Downloaded;
    }

    /// Whether the map of `area` is known, however she got it. Spec: MapsByArea.
    #[must_use]
    pub const fn is_map_known(&self, area: Area) -> bool {
        self.maps.get(area).is_known()
    }

    /// The quirk of Tourian's map, which has no station: the room that gives an area's map on
    /// arrival leaves it `OnArrival`, unless it is already downloaded. Spec: MapOnArrival.
    pub(crate) const fn map_on_arrival(&mut self, area: Area) {
        let map = self.maps.get_mut(area);
        if !matches!(map, AreaMap::Downloaded) {
            *map = AreaMap::OnArrival;
        }
    }

    /// The other half of the quirk of Tourian's map: leaving an area by a door into another
    /// turns its `OnArrival` map into a downloaded one; any other map stays as it was.
    /// Spec: MapOnArrival.
    pub(crate) const fn leave_area(&mut self, area: Area) {
        let map = self.maps.get_mut(area);
        if matches!(map, AreaMap::OnArrival) {
            *map = AreaMap::Downloaded;
        }
    }

    /// A station she saves at, or the one the gunship first lands at, is one she has saved at in
    /// its area. Spec: StationsSavedAt.
    pub(crate) fn save(&mut self, area: Area, station: StationId) {
        self.stations.get_mut(area).insert(station);
    }

    /// The quirk of the items placed: every item a room places as it loads counts, taken or not,
    /// over the whole run, wrapping past 65535; nothing in play reads the count, but the saved
    /// file keeps it. Spec: ItemsPlacedCount.
    pub(crate) const fn place_items(&mut self, placed: Count) {
        self.items_placed = self.items_placed.plus(placed);
    }
}
