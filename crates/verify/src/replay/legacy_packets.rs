//! Packet identities for format-1 cases written before the index declared their fixtures.

use super::index::{ConnectedPacket, NativeProjection};
use super::native_search::Coordinates;
use serde::Deserialize;
use std::{collections::BTreeMap, sync::LazyLock};

#[derive(Deserialize)]
struct Search {
    #[serde(flatten)]
    coordinates: Coordinates,
    projection: NativeProjection,
}

#[derive(Deserialize)]
struct Packets {
    connected: BTreeMap<String, ConnectedPacket>,
    native_search: BTreeMap<String, Search>,
}

static PACKETS: LazyLock<Option<Packets>> = LazyLock::new(|| {
    toml::from_str(include_str!(
        "../../../../scenarios/replay/legacy-packets.toml"
    ))
    .ok()
});

pub(super) fn connected(id: &str) -> Option<&'static ConnectedPacket> {
    PACKETS.as_ref()?.connected.get(id)
}

pub(super) fn projection(pin: &str, coordinates: Coordinates) -> Option<&'static NativeProjection> {
    let search = PACKETS.as_ref()?.native_search.get(pin)?;
    (search.coordinates == coordinates).then_some(&search.projection)
}
