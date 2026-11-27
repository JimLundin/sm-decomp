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
    ordinary_witness: BTreeMap<String, super::recorded::Witness>,
    save_schedule: BTreeMap<String, super::index::SaveSchedule>,
    intro_join: BTreeMap<String, PublishedJoin>,
    played_witnesses: super::played::witnesses::Witnesses,
    played: BTreeMap<String, super::index::PlayedWindow>,
    native_cases: std::collections::BTreeSet<String>,
    sender_control: BTreeMap<String, SenderOfferControl>,
    fixture: Vec<FixturePair>,
    native_packet: Vec<super::index::NativePacket>,
    connected: BTreeMap<String, ConnectedPacket>,
    native_search: BTreeMap<String, Search>,
}

static PACKETS: LazyLock<Option<Packets>> = LazyLock::new(|| {
    toml::from_str(include_str!(
        "../../../../scenarios/replay/legacy-packets.toml"
    ))
    .ok()
});

pub(super) fn ordinary_witness(id: &str) -> Option<&'static super::recorded::Witness> {
    PACKETS.as_ref()?.ordinary_witness.get(id)
}

pub(super) fn native(case: &super::index::Case) -> Option<&'static super::index::NativePacket> {
    let packets = PACKETS.as_ref()?;
    if !packets.native_cases.contains(&case.id.0) {
        return None;
    }
    packets
        .native_packet
        .iter()
        .find(|packet| packet.applies_to(case))
}

pub(super) fn save_schedule(id: &str) -> Option<&'static super::index::SaveSchedule> {
    PACKETS.as_ref()?.save_schedule.get(id)
}

#[derive(Deserialize)]
struct PublishedJoin {
    row: String,
    origin: u64,
    end: u64,
    join: super::intro_sources::Join,
}

pub(super) fn read_intro_join(case: &mut super::index::Case) -> Result<(), &'static str> {
    case.inputs = if case.inputs == InputProvider::PublishedDiscoveryInputs {
        InputProvider::DiscoveryInputs
    } else if case.inputs == InputProvider::PublishedFlashbackInputs {
        InputProvider::FlashbackInputs
    } else if case.inputs == InputProvider::PublishedIntroRoomJoinInputs {
        InputProvider::IntroRoomJoinInputs
    } else if case.inputs == InputProvider::PublishedCinematicActorOpportunities {
        InputProvider::CinematicActorOpportunities
    } else {
        return Ok(());
    };
    let published = PACKETS
        .as_ref()
        .and_then(|p| p.intro_join.get(&case.id.0))
        .ok_or("unknown published intro source")?;
    if case.intro_join.is_some()
        || case.row != published.row
        || case.start
            != (super::index::Start::CacheCut {
                frame: published.origin,
            })
        || case.end.cut_inclusive != published.end
        || !published.join.valid(case)
    {
        return Err("changed published intro source");
    }
    case.intro_join = Some(published.join.clone());
    Ok(())
}

pub(super) fn played_witnesses() -> Option<&'static super::played::witnesses::Witnesses> {
    PACKETS.as_ref().map(|packets| &packets.played_witnesses)
}

pub(super) fn played(id: &str) -> Option<&'static super::index::PlayedWindow> {
    PACKETS.as_ref()?.played.get(id)
}

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
