//! Packet identities for format-1 cases written before the index declared their fixtures.

use super::index::{
    ConnectedPacket, FixturePair, InputProvider, NativeProjection, SenderOfferControl,
};
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
    sender_control: BTreeMap<String, SenderOfferControl>,
    fixture: Vec<FixturePair>,
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

pub(super) fn fixture(provider: InputProvider) -> Option<&'static FixturePair> {
    PACKETS
        .as_ref()?
        .fixture
        .iter()
        .find(|pair| pair.provider == provider)
}

pub(super) fn sender_control<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<SenderOfferControl, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Control {
        Window(SenderOfferControl),
        Published(String),
    }

    match Control::deserialize(deserializer)? {
        Control::Window(window) => Ok(window),
        Control::Published(id) => PACKETS
            .as_ref()
            .and_then(|packets| packets.sender_control.get(&id))
            .cloned()
            .ok_or_else(|| serde::de::Error::custom("unknown sender offer control")),
    }
}
