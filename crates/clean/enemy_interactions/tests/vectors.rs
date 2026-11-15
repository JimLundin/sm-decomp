//! Strict vector decoding and contact checks for every catalogue entry.

use enemy_interactions::{
    BehaviourId, Centre, ContactBox, ContactConditions, DoorContact, Kind, ProtectionFrames, Radii,
    SHOT_SLOTS, ShotBody, ShotCount, ShotSlot, Shots, Switch, behaviour, consumption,
};
use enum_map::EnumMap;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Suite {
    vector: Vec<Vector>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Vector {
    id: String,
    entries: Vec<Entry>,
    source: Source,
    count: u16,
    door: Door,
    shots: [Shot; 5],
    conditions: Conditions,
    after: After,
    repeat: Option<u16>,
    frame: Option<u16>,
    actor: Option<u16>,
    observation: Option<String>,
}

#[derive(Deserialize)]
enum Entry {
    ContactAllowed,
    BoxOverlap,
    FirstPowerShot,
}

impl Entry {
    const fn id(&self) -> BehaviourId {
        match self {
            Self::ContactAllowed => BehaviourId::ContactAllowed,
            Self::BoxOverlap => BehaviourId::BoxOverlap,
            Self::FirstPowerShot => BehaviourId::FirstPowerShot,
        }
    }
}

#[derive(Deserialize)]
enum Source {
    #[serde(rename = "constructed")]
    Constructed,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct After {
    slot: i16,
}

#[derive(Deserialize)]
enum DoorKind {
    Passable,
    WithoutFigure,
    Box,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Door {
    kind: DoorKind,
    centre: [u16; 2],
    radius: [u16; 2],
}

impl Door {
    const fn contact(&self) -> DoorContact {
        match self.kind {
            DoorKind::Passable => DoorContact::Passable,
            DoorKind::WithoutFigure => DoorContact::WithoutFigure,
            DoorKind::Box => DoorContact::Box(figure(self.centre, self.radius)),
        }
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Shot {
    Free {},
    Impact {},
    Power {
        centre: [u16; 2],
        radius: [u16; 2],
        consumed: bool,
    },
}

impl Shot {
    const fn body(self) -> ShotBody {
        match self {
            Self::Free {} => ShotBody::Free,
            Self::Impact {} => ShotBody::Impact,
            Self::Power {
                centre,
                radius,
                consumed,
            } => ShotBody::Power {
                figure: figure(centre, radius),
                consumed,
            },
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Vector mirrors five independent facts"
)]
struct Conditions {
    actor_updates: bool,
    interactions: bool,
    #[serde(rename = "Samus_stopped")]
    samus_stopped: bool,
    enemies_stopped: bool,
    processing_member: bool,
    protection: u16,
}

impl Conditions {
    const fn facts(&self) -> ContactConditions {
        ContactConditions {
            actor_updates_enabled: self.actor_updates,
            interactions_enabled: self.interactions,
            samus_time_stopped: self.samus_stopped,
            enemies_stopped_in_time: self.enemies_stopped,
            processing_membership: self.processing_member,
            protection_remaining: ProtectionFrames(self.protection),
        }
    }
}

const fn figure(centre: [u16; 2], radius: [u16; 2]) -> ContactBox {
    let [x, y] = centre;
    let [rx, ry] = radius;
    ContactBox {
        centre: Centre { x, y },
        radii: Radii { x: rx, y: ry },
    }
}

const fn slot_number(slot: Option<ShotSlot>) -> i16 {
    match slot {
        None => -1,
        Some(ShotSlot::Slot0) => 0,
        Some(ShotSlot::Slot1) => 1,
        Some(ShotSlot::Slot2) => 2,
        Some(ShotSlot::Slot3) => 3,
        Some(ShotSlot::Slot4) => 4,
    }
}

fn run_vector(vector: &Vector) {
    let mut shots = Shots::from_array(vector.shots.map(Shot::body));
    let door = vector.door.contact();
    let facts = vector.conditions.facts();
    for _ in 0..vector.repeat.unwrap_or(1) {
        let before = shots;
        let slot = consumption(facts, door, ShotCount(vector.count), &shots);
        assert_eq!(
            slot_number(slot),
            vector.after.slot,
            "{} consumption",
            vector.id
        );
        assert_eq!(shots, before, "{} must not mutate bodies", vector.id);
        if let Some(slot) = slot {
            let ShotBody::Power { consumed, .. } = &mut shots[slot] else {
                assert!(
                    matches!(shots[slot], ShotBody::Power { .. }),
                    "returned slot must hold Power"
                );
                return;
            };
            *consumed = true;
        }
    }
    // These are provenance only, never permission or expected-future inputs.
    assert!(matches!(vector.source, Source::Constructed));
    if let Some(observation) = &vector.observation {
        assert_eq!(observation, "translated geometry");
        assert!(vector.frame.is_some());
        assert!(vector.actor.is_some());
    }
}

#[test]
fn every_vector_and_catalogue_entry() -> Result<(), Box<dyn std::error::Error>> {
    let suite: Suite = toml::from_str(include_str!(
        "../../../../scenarios/vectors/enemy-interactions.toml"
    ))?;
    assert_eq!(suite.vector.len(), 30);
    let mut covered: EnumMap<BehaviourId, bool> = EnumMap::default();
    for vector in &suite.vector {
        run_vector(vector);
        for entry in &vector.entries {
            covered[entry.id()] = true;
        }
    }
    for (id, checked) in covered {
        assert!(checked, "{id:?} must have a vector");
        let record = behaviour(id);
        assert_eq!(record.id, id);
        assert_eq!(record.switch, Switch::Fixed);
        assert_eq!(
            record.kind,
            if id == BehaviourId::BoxOverlap {
                Kind::Quirk
            } else {
                Kind::Mechanic
            }
        );
        assert_ne!(record.checked, "");
    }
    assert_eq!(SHOT_SLOTS.len(), 5);
    Ok(())
}

#[test]
fn full_word_domains_and_wrapped_large_radii() {
    let door = figure([0, 0], [u16::MAX, u16::MAX]);
    let shot = figure([0x8000, 0x8000], [u16::MAX, u16::MAX]);
    assert!(shot.overlaps_door(door));
    let facts = ContactConditions {
        actor_updates_enabled: true,
        interactions_enabled: true,
        samus_time_stopped: false,
        enemies_stopped_in_time: false,
        processing_membership: true,
        protection_remaining: ProtectionFrames(0),
    };
    assert_eq!(
        facts.allowed_box(DoorContact::Box(door), ShotCount(u16::MAX)),
        Some(door)
    );
    assert_eq!(
        ContactConditions {
            protection_remaining: ProtectionFrames(u16::MAX),
            ..facts
        }
        .allowed_box(DoorContact::Box(door), ShotCount(u16::MAX)),
        None
    );
    // Signed midpoint negation stays 0x8000, and subtraction wraps exactly.
    assert!(figure([0x8000, 0x8000], [0x8001, 0x8001]).overlaps_door(figure([0, 0], [0, 0])));
    assert!(!figure([0, 0], [0, 0]).overlaps_door(figure([0, 0], [0, 0])));
}

#[test]
fn vector_schema_rejects_unknown_keys_and_capabilities() {
    assert!(toml::from_str::<Suite>("vector = []\nunknown = true").is_err());
    assert!(toml::from_str::<Shot>("kind = 'Missile'").is_err());
    assert!(toml::from_str::<Shot>("kind = 'Free'\nconsumed = false").is_err());
}
