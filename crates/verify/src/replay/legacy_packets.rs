//! Packet identities and source descriptions from published replay indices.

use super::borrowed::Selection;
use super::index::{
    Borrowed, Case, ConnectedPacket, FixturePair, InputProvider, NativeProjection, Qualification,
    SenderOfferControl,
};
use super::intro_sources::{Join, RecordCut};
use super::native_search::Coordinates;
use serde::Deserialize;
use std::{collections::BTreeMap, sync::LazyLock};

#[derive(Deserialize)]
struct Search {
    #[serde(flatten)]
    coordinates: Coordinates,
    projection: NativeProjection,
    borrowed: PublishedSources,
}

#[derive(Deserialize)]
struct Packets {
    movement_sources: Vec<PublishedSources>,
    source_migration: BTreeMap<InputProvider, Vec<PublishedSources>>,
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

#[derive(Deserialize)]
struct PublishedSources {
    sources: Vec<Borrowed>,
    operations: Vec<String>,
    #[serde(default)]
    selection: Selection,
    handoff: Option<RecordCut>,
}

fn same_source(source: &Borrowed, input: &Borrowed) -> bool {
    source.provider == input.provider
        && input.describes(&source.dependency, &source.fields, &source.phase)
}

impl PublishedSources {
    fn complete(&self, rows: &[Borrowed]) -> bool {
        self.sources
            .iter()
            .all(|source| rows.iter().filter(|row| same_source(source, row)).count() == 1)
    }

    fn matches(&self, rows: &[Borrowed]) -> bool {
        rows.len() == self.sources.len() && self.complete(rows)
    }
}

fn expand(case: &mut Case, recipes: &[&PublishedSources]) -> Result<(), &'static str> {
    if recipes.is_empty() {
        return Ok(());
    }
    for recipe in recipes {
        if !recipe.complete(&case.borrowed) {
            return Err("incomplete or duplicate published source");
        }
        if recipe.handoff.as_ref().is_some_and(|pin| {
            !matches!(
                &case.intro_join, Some(Join::Flashback { handoff, .. }) if handoff == pin
            )
        }) {
            return Err("published flashback handoff");
        }
    }
    for input in std::mem::take(&mut case.borrowed) {
        let Some(recipe) = recipes
            .iter()
            .find(|r| r.sources.iter().any(|source| same_source(source, &input)))
        else {
            case.borrowed.push(input);
            continue;
        };
        if recipe
            .sources
            .first()
            .is_some_and(|first| same_source(first, &input))
        {
            for name in &recipe.operations {
                if let Some(operation) = super::borrowed::published_operation(case, name)? {
                    case.borrowed.push(operation);
                }
            }
        }
        case.qualifications.push(Qualification::Published(input));
    }
    Ok(())
}

pub(super) fn read_sources(case: &mut Case) -> Result<(), &'static str> {
    let packets = PACKETS.as_ref().ok_or("published source catalogue")?;
    if let Some(migrations) = packets.source_migration.get(&case.inputs) {
        let selected: Vec<_> = migrations
            .iter()
            .filter(|r| r.selection.applies(case, None))
            .collect();
        if selected.iter().any(|r| {
            r.sources
                .iter()
                .any(|s| case.borrowed.iter().any(|input| same_source(s, input)))
        }) {
            expand(case, &selected)?;
        }
    }
    let recipe = if case.inputs == InputProvider::PrecedingCutMovementInputs {
        packets
            .movement_sources
            .iter()
            .find(|r| r.matches(&case.borrowed))
    } else if case.inputs == InputProvider::NativeSearchInputs {
        case.start
            .native_opportunity()
            .and_then(|(_, pin, at)| {
                packets
                    .native_search
                    .get(pin)
                    .filter(|search| search.coordinates == at)
            })
            .map(|search| &search.borrowed)
            .filter(|r| r.matches(&case.borrowed))
    } else {
        None
    };
    if let Some(recipe) = recipe {
        expand(case, &[recipe])?;
    }
    Ok(())
}

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
