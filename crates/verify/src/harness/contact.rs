//! Stateless C1 output receipts. W1 is a preceding-input neighbour, not carried
//! C1 State. Marker proof does not establish uninterrupted W1 or engine play.

mod capability;

use super::weapons::{Defect, Weapons};
use super::{Domain, Entry, Field, FieldSet, Interval, Uncompared};
use enemy_interactions as contact;
use oracle_record::FrameRecord;
use weapons_projectiles as beam;

fn word(wram: &[u8], at: usize) -> Result<u16, Defect> {
    let bytes = wram
        .get(at..at.saturating_add(2))
        .ok_or(Defect("C1 short WRAM"))?;
    Ok(u16::from_le_bytes(
        bytes.try_into().map_err(|_| Defect("C1 short word"))?,
    ))
}

/// Reconstruct the normal gameplay selector receipt, retaining processing order.
/// The preceding override is valid on the source-proven ordinary caller route;
/// callers must establish that route before using a live receipt.
pub(super) fn processing_receipt(wram: &[u8]) -> Result<Vec<usize>, Defect> {
    let camera = (word(wram, 0x0911)?, word(wram, 0x0915)?);
    let global = crate::source::GLOBAL_OFF_SCREEN_ENEMY_PROCESSING
        .read(wram)
        .map_err(|_| Defect("C1 global selector input decode"))?
        != 0;
    let commons = crate::enemy::EnemySources::ORIGINAL
        .commons(wram)
        .map_err(|_| Defect("C1 selector input decode"))?;
    let mut receipt = Vec::new();
    for (index, common) in commons.iter().enumerate() {
        let Some(common) = common else {
            continue;
        };
        let offset = index.saturating_mul(64);
        // Commons intentionally defaults species; the respawn placeholder has no actor.
        if word(wram, 0x0f78_usize.saturating_add(offset))? == 0xdaff {
            continue;
        }
        if global || game::enemy::processes(common, camera.0, camera.1) {
            receipt.push(offset);
        }
    }
    Ok(receipt)
}

/// Source-backed supported-door temporal contract, applied in processing order.
/// Other selected actors might alter a later door and are explicitly rejected.
pub(super) fn apply(wram: &[u8], active: &[usize], state: &mut beam::State) -> Result<u64, Defect> {
    if !has_live_power(state) {
        return Ok(0);
    }
    if word(wram, 0x0998)? != 8 {
        return Err(Defect("C1 live shots outside gameplay caller"));
    }
    let mut hits = 0_u64;
    for &off in active {
        let w = |at: usize| word(wram, at.saturating_add(off));
        if w(0x0f78)? != 0xe23f {
            return Err(Defect("C1 unsupported processing actor species"));
        }
        if !matches!(w(0x0fb4)?, 0 | 1 | 4) {
            return Err(Defect("C1 unsupported Ceres door parameter"));
        }
        if w(0x0f88)? & 4 != 0 {
            return Err(Defect("C1 unsupported multibox actor"));
        }
        let door = if w(0x0f86)? & 0x0400 != 0 {
            contact::DoorContact::Passable
        } else if matches!(w(0x0f8e)?, 0 | 0x804d) {
            contact::DoorContact::WithoutFigure
        } else {
            contact::DoorContact::Box(contact::ContactBox {
                centre: contact::Centre {
                    x: w(0x0f7a)?,
                    y: w(0x0f7e)?,
                },
                radii: contact::Radii {
                    x: w(0x0f82)?,
                    y: w(0x0f84)?,
                },
            })
        };
        let conditions = contact::ContactConditions {
            actor_updates_enabled: word(wram, 0x0e4c)? != 0,
            interactions_enabled: word(wram, 0x0e12)? == 0,
            samus_time_stopped: word(wram, 0x0a78)? != 0,
            enemies_stopped_in_time: word(wram, 0x185e)? != 0,
            processing_membership: true,
            protection_remaining: contact::ProtectionFrames(w(0x0fa0)?),
        };
        if game::enemy_interactions_bridge::contact_door(state, conditions, door).is_some() {
            hits = hits.saturating_add(1);
        }
    }
    Ok(hits)
}

pub(super) fn has_live_power(state: &beam::State) -> bool {
    state.count.0 != 0
        && state
            .slots
            .values()
            .any(|shot| matches!(shot, beam::ShotState::Flying { .. }))
}

/// Reject restored encodings whose raw marker observable is not represented by C1.
/// Count gates the search; it does not establish absence of occupied flight slots.
fn marker_inputs(wram: &[u8], predicted: &beam::State) -> Result<(), Defect> {
    for offset in (0..10).step_by(2) {
        if word(wram, 0x0c40_usize.saturating_add(offset))? == 0 {
            continue;
        }
        let kind = word(wram, 0x0c18_usize.saturating_add(offset))?;
        if kind == 0x8700 && word(wram, 0x0c04_usize.saturating_add(offset))? & 0x0010 != 0 {
            return Err(Defect("C1 restored impact carries an unrepresented marker"));
        }
        if kind == 0x8000 && word(wram, 0x0cce)? == 0 {
            return Err(Defect("C1 restored zero count with flying shot"));
        }
    }
    if predicted.count.0 == 0
        && predicted
            .slots
            .values()
            .any(|shot| matches!(shot, beam::ShotState::Flying { .. }))
    {
        return Err(Defect("C1 predicted zero count with flying shot"));
    }
    Ok(())
}

/// Verification-only output receipt; no retained model fields exist.
#[derive(Clone)]
pub struct Receipt(pub [bool; 5]);
#[derive(Clone)]
/// Independently predicted W1 neighbour and preceding actor facts.
pub struct Inputs {
    predicted: beam::State,
    wram: Box<[u8]>,
    active: Vec<usize>,
}
#[derive(Clone)]
/// Per-call validation and observed hit accounting.
pub struct World {
    defect: Option<Defect>,
    hits: u64,
    completion: Result<(), Defect>,
}
impl Default for World {
    fn default() -> Self {
        Self {
            defect: None,
            hits: 0,
            completion: Ok(()),
        }
    }
}
/// Stateless ordinary Ceres-door marker proof.
pub struct Contact {
    weapons: Weapons,
    capability: capability::Content,
}
impl Contact {
    /// Load W1's content through its production importer.
    /// # Errors
    /// Unsupported or invalid imported content.
    pub fn load(image: &[u8]) -> Result<Self, Defect> {
        Ok(Self {
            weapons: Weapons::load(image)?,
            capability: capability::Content::load(image)?,
        })
    }
}
fn markers(state: &beam::State) -> Receipt {
    let mut result = [false; 5];
    for (value, shot) in result.iter_mut().zip(state.slots.values()) {
        *value = matches!(shot, beam::ShotState::Flying { consumed: true, .. });
    }
    Receipt(result)
}
/// Scope and decoding are separate: unsupported Ceres capabilities are defects.
#[derive(Clone, Debug)]
pub enum Skip {
    /// Another region owns these contact opportunities.
    OtherRegion,
    /// A missing or unsupported input in Ceres.
    Defect(Defect),
}
impl From<Defect> for Skip {
    fn from(error: Defect) -> Self {
        Self::Defect(error)
    }
}
impl std::fmt::Display for Skip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OtherRegion => f.write_str("C1 Ceres-door scope: another region"),
            Self::Defect(error) => error.fmt(f),
        }
    }
}
impl Domain for Contact {
    const NAME: &'static str = "enemy-interactions";
    type State = Receipt;
    type Inputs = Inputs;
    type World = World;
    type Fields = Receipt;
    type Skip = Skip;
    const FIELDS: &'static [Field<Receipt>] = &[Field {
        name: "consumed-markers",
        same: |a, b| a.0 == b.0,
        show: |value| format!("{:?}", value.0),
    }];
    fn behaviours(&self) -> Vec<Entry> {
        ["ContactAllowed", "BoxOverlap", "FirstPowerShot"]
            .into_iter()
            .map(|name| Entry {
                name: name.into(),
                fields: FieldSet::single(0),
            })
            .collect()
    }
    fn decode(&self, from: &FrameRecord, interval: Interval) -> Result<(Receipt, Inputs), Skip> {
        if word(&from.wram, 0x079f)? != u16::from(u8::from(data::room::Area::Ceres)) {
            return Err(Skip::OtherRegion);
        }
        let predicted = self.weapons.predict(from, interval)?;
        marker_inputs(&from.wram, &predicted)?;
        self.weapons.contact_start(from)?;
        self.weapons.marker_memory_inputs(from)?;
        self.capability.ordinary(&from.wram)?;
        let active = processing_receipt(&from.wram)?;
        self.capability
            .actors(&from.wram, &active, has_live_power(&predicted))?;
        Ok((
            markers(&predicted),
            Inputs {
                predicted,
                wram: from.wram.clone(),
                active,
            },
        ))
    }
    fn world(&self, from: &FrameRecord, next: &FrameRecord) -> Result<World, Skip> {
        Ok(World {
            defect: None,
            hits: 0,
            completion: super::gameplay_caller::completed(from, next).map_err(Defect::from),
        })
    }
    fn step(&self, receipt: &mut Receipt, inputs: &Inputs, world: &mut World) {
        let mut state = inputs.predicted.clone();
        if let Err(error) = &world.completion {
            world.defect = Some(error.clone());
            return;
        }
        match apply(&inputs.wram, &inputs.active, &mut state) {
            Ok(hits) => world.hits = hits,
            Err(error) => world.defect = Some(error),
        }
        *receipt = markers(&state);
    }
    fn checked_step(&self, world: &World) -> Result<(), Skip> {
        world
            .defect
            .clone()
            .map_or(Ok(()), |error| Err(error.into()))
    }
    fn exercised(&self, _: &Receipt, _: &Inputs, world: &World) -> Vec<usize> {
        if world.hits == 0 {
            Vec::new()
        } else {
            vec![0, 1, 2]
        }
    }
    fn fields(&self, state: &Receipt) -> Receipt {
        state.clone()
    }
    fn expected(&self, next: &FrameRecord) -> Result<Receipt, Skip> {
        let mut values = [false; 5];
        for (index, value) in values.iter_mut().enumerate() {
            let offset = index.saturating_mul(2);
            *value = word(&next.wram, 0x0c40_usize.saturating_add(offset))? != 0
                && word(&next.wram, 0x0c04_usize.saturating_add(offset))? & 0x0010 != 0;
        }
        Ok(Receipt(values))
    }
    fn uncompared(&self, skip: &Skip) -> Uncompared {
        match skip {
            Skip::OtherRegion => Uncompared::OutOfScope(skip.to_string()),
            Skip::Defect(_) => Uncompared::Defect(skip.to_string()),
        }
    }
    fn stateless_outputs(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game::enemy_interactions_bridge::contact_door;
    fn state() -> beam::State {
        let mut state = beam::State::reset(enum_map::EnumMap::from_fn(|_| beam::Axes {
            x: beam::PixelFraction(0),
            y: beam::PixelFraction(0),
        }));
        state.count = beam::ShotCount(1);
        state.slots[beam::ShotSlot::Slot4] = beam::ShotState::Flying {
            consumed: false,
            body: beam::ShotBody {
                position: beam::Axes {
                    x: beam::FixedPosition::from_parts(100, beam::PixelFraction(0)),
                    y: beam::FixedPosition::from_parts(200, beam::PixelFraction(0)),
                },
                heading: beam::Heading::Right,
                speed: beam::Axes {
                    x: beam::BeamSpeed(0),
                    y: beam::BeamSpeed(0),
                },
                radius: beam::Axes {
                    x: beam::Radius(4),
                    y: beam::Radius(4),
                },
                damage: beam::Damage::new(1).unwrap(),
                trail: beam::FrameWord(0),
                clip: beam::ClipId("invented".into()),
                next_frame: beam::ClipFrame::new(0, 1).unwrap(),
                remaining_hold: beam::FrameWord(1),
                picture: None,
                trail_appearance: None,
            },
        };
        state
    }
    fn conditions(protection: u16) -> contact::ContactConditions {
        contact::ContactConditions {
            actor_updates_enabled: true,
            interactions_enabled: true,
            samus_time_stopped: false,
            enemies_stopped_in_time: false,
            processing_membership: true,
            protection_remaining: contact::ProtectionFrames(protection),
        }
    }
    fn door(x: u16) -> contact::DoorContact {
        contact::DoorContact::Box(contact::ContactBox {
            centre: contact::Centre { x, y: 200 },
            radii: contact::Radii { x: 8, y: 32 },
        })
    }
    #[test]
    fn restored_marker_encodings_reject_before_receipt_projection() {
        let mut wram = vec![0; 0x20000];
        let mut predicted = state();
        predicted.count = beam::ShotCount(0);
        assert!(marker_inputs(&wram, &predicted).is_err());
        predicted.count = beam::ShotCount(1);
        let put = |wram: &mut [u8], at: usize, value: u16| {
            wram[at..at.saturating_add(2)].copy_from_slice(&value.to_le_bytes());
        };
        put(&mut wram, 0x0c40, 1);
        put(&mut wram, 0x0c18, 0x8000);
        assert!(marker_inputs(&wram, &predicted).is_err());
        put(&mut wram, 0x0cce, 1);
        assert!(marker_inputs(&wram, &predicted).is_ok());
        put(&mut wram, 0x0c18, 0x8700);
        put(&mut wram, 0x0c04, 0x0010);
        assert!(marker_inputs(&wram, &predicted).is_err());
        put(&mut wram, 0x0c04, 0);
        assert!(marker_inputs(&wram, &predicted).is_ok());
    }

    #[test]
    fn successive_current_doors_reuse_consumed_late_slot_and_initial_protection_suppresses() {
        let mut state = state();
        let original = state.slots[beam::ShotSlot::Slot4].body().unwrap().clone();
        assert_eq!(contact_door(&mut state, conditions(0), door(1000)), None);
        assert_eq!(contact_door(&mut state, conditions(1), door(100)), None);
        assert_eq!(
            contact_door(&mut state, conditions(0), door(100)),
            Some(beam::ShotSlot::Slot4)
        );
        assert_eq!(
            contact_door(&mut state, conditions(0), contact::DoorContact::Passable),
            None
        );
        assert_eq!(
            contact_door(&mut state, conditions(0), door(101)),
            Some(beam::ShotSlot::Slot4)
        );
        assert_eq!(state.count, beam::ShotCount(1));
        assert_eq!(state.slots[beam::ShotSlot::Slot4].body(), Some(&original));
    }
    #[test]
    fn source_boundary_rejects_unsupported_actor_instead_of_reporting_a_miss() {
        let mut state = state();
        let mut wram = vec![0; 0x20000];
        wram[0x0998..0x099a].copy_from_slice(&8_u16.to_le_bytes());
        wram[0x0f78..0x0f7a].copy_from_slice(&0xe13f_u16.to_le_bytes());
        assert!(apply(&wram, &[0], &mut state).is_err());
        assert_eq!(markers(&state).0, [false; 5]);
    }
    #[test]
    fn quiet_receipt_requires_valid_start_and_completed_endpoint() {
        let Some((cache, rom)) = crate::harness::tests::environment() else {
            return;
        };
        let domain = Contact::load(&std::fs::read(rom).unwrap()).unwrap();
        let lock = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../scenarios/locks/lsnes.toml");
        let cached = super::super::open_cached(&lock, &cache, "chain-any-0003-DFD7").unwrap();
        let records = cached
            .records_without_pictures()
            .take(2)
            .map(|record| record.unwrap().1)
            .collect::<Vec<_>>();
        let from = &records[0];
        let next = &records[1];
        let (mut receipt, input) = domain.decode(from, Interval::ending_at(next)).unwrap();
        assert!(!has_live_power(&input.predicted));
        let mut wrong_start = from.clone();
        wrong_start.cpu.pc = 0;
        assert!(matches!(
            domain.decode(&wrong_start, Interval::ending_at(next)),
            Err(Skip::Defect(_))
        ));
        let mut wrong_end = next.clone();
        wrong_end.cpu.pc = 0;
        let mut world = domain.world(from, &wrong_end).unwrap();
        domain.step(&mut receipt, &input, &mut world);
        assert!(matches!(domain.checked_step(&world), Err(Skip::Defect(_))));
    }

    #[test]
    fn recorded_m0_has_four_live_searches_and_one_hit_from_preceding_w1_execution() {
        let Some((cache, rom)) = crate::harness::tests::environment() else {
            return;
        };
        let domain = Contact::load(&std::fs::read(rom).unwrap()).unwrap();
        let lock = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../scenarios/locks/lsnes.toml");
        let cached = super::super::open_cached(&lock, &cache, "chain-any-0003-DFD7").unwrap();
        let records = cached
            .records_without_pictures()
            .map(|record| record.unwrap().1)
            .collect::<Vec<_>>();
        let mut searches = 0_u64;
        let mut hits = 0_u64;
        let mut compared = 0_u64;
        let mut unsupported = 0_u64;
        for pair in records.windows(2) {
            let from = &pair[0];
            let next = &pair[1];
            let (mut receipt, inputs) = match domain.decode(from, Interval::ending_at(next)) {
                Ok(value) => value,
                Err(Skip::Defect(_)) => {
                    unsupported = unsupported.saturating_add(1);
                    continue;
                }
                Err(Skip::OtherRegion) => panic!("Ceres witness left its declared region"),
            };
            let mut world = domain.world(from, next).unwrap();
            domain.step(&mut receipt, &inputs, &mut world);
            if domain.checked_step(&world).is_err() {
                unsupported = unsupported.saturating_add(1);
                continue;
            }
            compared = compared.saturating_add(1);
            if inputs.predicted.count.0 != 0
                && inputs
                    .predicted
                    .slots
                    .values()
                    .any(|shot| matches!(shot, beam::ShotState::Flying { .. }))
            {
                searches = searches.saturating_add(1);
            }
            hits = hits.saturating_add(world.hits);
        }
        assert_eq!((compared, unsupported, searches, hits), (136, 208, 4, 1));
    }
    #[test]
    fn global_selector_adds_offscreen_doors_but_never_placeholders_or_deleted_slots() {
        let mut wram = vec![0; 0x20000];
        let put = |wram: &mut [u8], at: usize, value: u16| {
            wram[at..at.saturating_add(2)].copy_from_slice(&value.to_le_bytes());
        };
        put(&mut wram, 0x0998, 8);
        put(&mut wram, 0x0e4c, 1);
        put(&mut wram, 0x0f78, 0xe23f);
        put(&mut wram, 0x0f82, 8);
        put(&mut wram, 0x0f84, 32);
        put(&mut wram, 0x0f8e, 1);
        put(&mut wram, 0x0f7a, 1000);
        put(&mut wram, 0x0f7e, 1000);
        put(&mut wram, 0x0f78 + 64, 0xdaff);
        put(&mut wram, 0x0f78 + 128, 0xe23f);
        put(&mut wram, 0x0f86 + 128, 0x0200);
        let mut shots = state();
        let beam::ShotState::Flying { body, .. } = &mut shots.slots[beam::ShotSlot::Slot4] else {
            panic!("fixture requires a flying shot")
        };
        body.position = beam::Axes {
            x: beam::FixedPosition::from_parts(1000, beam::PixelFraction(0)),
            y: beam::FixedPosition::from_parts(1000, beam::PixelFraction(0)),
        };
        let without_override = processing_receipt(&wram).unwrap();
        assert_eq!(without_override, Vec::<usize>::new());
        assert_eq!(apply(&wram, &without_override, &mut shots).unwrap(), 0);
        put(
            &mut wram,
            usize::from(crate::source::GLOBAL_OFF_SCREEN_ENEMY_PROCESSING.offset),
            1,
        );
        let overridden = processing_receipt(&wram).unwrap();
        assert_eq!(overridden, [0]);
        assert_eq!(apply(&wram, &overridden, &mut shots).unwrap(), 1);
        assert_eq!(markers(&shots).0, [false, false, false, false, true]);
    }
}
