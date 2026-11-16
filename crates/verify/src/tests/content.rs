//! ROM content integrity checks, separate from recorded behaviour acceptance.

use core::num::Wrapping;
use data::plm::PlmKind;
use data::rom::{Lorom, Rom, RomAddress};
use data::room::{RoomKey, decode};
use run_progress::{Pickup, Supply};
use std::collections::{BTreeMap, BTreeSet};
use types::{Beam, Item, LocationId};

fn player_rom() -> Rom {
    let path = std::env::var_os("SM_ROM").expect("SM_ROM required for content checks");
    let image = std::fs::read(path).expect("player ROM readable");
    Rom::new(image).unwrap_or_else(|_| panic!("player ROM identity"))
}

#[test]
#[ignore = "requires the player's ROM"]
fn every_selectable_pose_has_a_unique_valid_clip() {
    use data::pose::animation::{AnimationDelays, RunPaces};
    use samus_appearance::{Control, Instruction};
    let rom = player_rom();
    let poses = data::pose::Poses::load(rom.lorom()).unwrap();
    let delays = AnimationDelays::load(rom.lorom(), &poses).unwrap();
    let paces = RunPaces::load(rom.lorom()).unwrap();
    let clips = game::samus_appearance_bridge::Clips::import(&delays, &paces).unwrap();
    let poses = clips.poses();
    assert_eq!(poses.len(), data::assets::samus::map::MAP.len());
    for (at, imported) in poses.iter().enumerate() {
        assert!(
            poses
                .iter()
                .skip(at.saturating_add(1))
                .all(|other| other.figure != imported.figure)
        );
        let shown = |step| matches!(imported.clip.at(step), Some(Instruction::Show(_)));
        for instruction in &imported.clip.instructions {
            assert!(match *instruction {
                Instruction::Control(Control::Again(to)) => shown(to),
                Instruction::Control(Control::Branch(branch)) =>
                    shown(branch.spin) && shown(branch.space_jump) && shown(branch.screw_attack),
                Instruction::Show(_)
                | Instruction::Control(
                    Control::Rest | Control::Breathe | Control::End | Control::EndUnlessJump,
                ) => true,
            });
        }
    }
}

fn word(rom: Lorom<'_>, bank: u8, offset: u16) -> u16 {
    let bytes = rom
        .read(RomAddress::new(bank, offset), 2)
        .unwrap_or_else(|_| panic!("content word unavailable"));
    u16::from_le_bytes(bytes.try_into().expect("content word width"))
}

/// Decode the pickup's asset operand, without executing the item script.
fn pickup(rom: Lorom<'_>, header: u16) -> Pickup {
    let list = word(rom, 0x84, header.wrapping_add(2));
    let bytes = rom
        .read(RomAddress::new(0x84, list), 0x100)
        .unwrap_or_else(|_| panic!("item content unavailable"));
    let start = bytes
        .windows(4)
        .position(|part| part == [0x99, 0x88, 0xDD, 0x8B])
        .expect("item collection content prefix")
        .saturating_add(5);
    let at = if bytes.get(start..start.saturating_add(2)) == Some(&[0x9D, 0xE2]) {
        start.saturating_add(2)
    } else {
        start
    };
    let [code_lo, code_hi, argument_lo, argument_hi]: [u8; 4] = bytes
        .get(at..at.saturating_add(4))
        .expect("item pickup operand")
        .try_into()
        .unwrap();
    let argument = u16::from_le_bytes([argument_lo, argument_hi]);
    let tank = |supply| Pickup::Tank {
        supply,
        amount: Wrapping(argument),
    };
    match u16::from_le_bytes([code_lo, code_hi]) {
        0x88B0 => Pickup::Beam(match argument {
            0x0001 => Beam::Wave,
            0x0002 => Beam::Ice,
            0x0004 => Beam::Spazer,
            0x0008 => Beam::Plasma,
            0x1000 => Beam::Charge,
            _ => panic!("single known beam pickup"),
        }),
        0x88F3 | 0x891A | 0x8941 => Pickup::Item(match argument {
            0x0001 => Item::VariaSuit,
            0x0002 => Item::SpringBall,
            0x0004 => Item::MorphBall,
            0x0008 => Item::ScrewAttack,
            0x0020 => Item::GravitySuit,
            0x0100 => Item::HiJumpBoots,
            0x0200 => Item::SpaceJump,
            0x1000 => Item::Bombs,
            0x2000 => Item::SpeedBooster,
            0x4000 => Item::GrappleBeam,
            0x8000 => Item::XRayScope,
            _ => panic!("single known item pickup"),
        }),
        0x8968 => tank(Supply::Energy),
        0x8986 => tank(Supply::Reserve),
        0x89A9 => tank(Supply::Missiles),
        0x89D2 => tank(Supply::SuperMissiles),
        0x89FB => tank(Supply::PowerBombs),
        _ => panic!("known pickup content"),
    }
}

struct Population {
    rooms: Vec<RoomKey>,
    items: BTreeMap<LocationId, Pickup>,
}

fn population(image: Lorom<'_>) -> Population {
    let mut seen = BTreeSet::new();
    let mut rooms = Vec::new();
    let mut items = BTreeMap::new();
    let mut todo = vec![0x91F8, 0xDF45];
    while let Some(pointer) = todo.pop() {
        if seen.contains(&pointer) || !(0x91F8..0xE82C).contains(&pointer) {
            continue;
        }
        let key = RoomKey::from_room_pointer(pointer);
        let room = decode(image, key).unwrap_or_else(|_| panic!("room content unavailable"));
        seen.insert(pointer);
        rooms.push(key);
        for (_, state) in room.states() {
            for entry in &state.plms.entries {
                let (PlmKind::Item(_, argument) | PlmKind::Unmodelled { argument, .. }) =
                    entry.kind
                else {
                    continue;
                };
                if !(0xEED7..=0xEFCF).contains(&entry.kind.id()) {
                    continue;
                }
                let item = pickup(image, entry.kind.id());
                if let Some(previous) = items.insert(LocationId(argument), item) {
                    assert!(previous == item, "one pickup kind per location");
                }
            }
        }
        for place in 0_u16..64 {
            let door = word(
                image,
                0x8F,
                room.doors.at.offset.wrapping_add(place.wrapping_mul(2)),
            );
            if !(0x88FC..0xAC00).contains(&door) {
                break;
            }
            let destination = word(image, 0x83, door);
            if destination != 0 {
                todo.push(destination);
            }
        }
    }
    Population { rooms, items }
}

#[test]
#[ignore = "requires the player's ROM"]
fn all_room_versions_decode() {
    let rom = player_rom();
    let population = population(rom.lorom());
    let states = |keys: &[RoomKey]| {
        keys.iter()
            .map(|&key| {
                decode(rom.lorom(), key)
                    .unwrap_or_else(|_| panic!("room version content"))
                    .states()
                    .count()
            })
            .sum::<usize>()
    };
    assert_eq!(population.rooms.len(), 261);
    assert_eq!(states(&population.rooms), 322);
    assert_eq!(states(&[0xB3E1, 0xE82C].map(RoomKey::from_room_pointer)), 2);
}

#[test]
#[ignore = "requires the player's ROM"]
fn all_placed_items_count_a_hundred_percent() {
    use game::production::{
        AdvanceInput, Production, ProductionContent, ProductionRules, ProductionSeed,
    };
    use game::state::owned::OwnedState;
    use run_progress::{Progress, Report, Rules};
    let rom = player_rom();
    let population = population(rom.lorom());
    let mut tanks = BTreeMap::<String, (usize, BTreeSet<u16>)>::new();
    let mut one_off = 0_usize;
    for item in population.items.values() {
        match *item {
            Pickup::Tank { supply, amount } => {
                let (count, amounts) = tanks.entry(format!("{supply:?}")).or_default();
                *count = count.saturating_add(1);
                amounts.insert(amount.0);
            }
            Pickup::Item(_) | Pickup::Beam(_) => one_off = one_off.saturating_add(1),
        }
    }
    let tank =
        |supply: Supply, count, amount| (format!("{supply:?}"), (count, BTreeSet::from([amount])));
    assert_eq!(population.items.len(), 100);
    assert_eq!(one_off, 16);
    assert_eq!(
        tanks,
        BTreeMap::from([
            tank(Supply::Energy, 14, 100),
            tank(Supply::Reserve, 4, 100),
            tank(Supply::Missiles, 46, 5),
            tank(Supply::SuperMissiles, 10, 5),
            tank(Supply::PowerBombs, 10, 5)
        ])
    );
    let rules = Rules::reference();
    let mut production = Production::admit(
        ProductionSeed::Progress(Box::default()),
        ProductionContent::Progress,
        ProductionRules::Progress(rules),
    )
    .unwrap();
    let reports: Vec<_> = population
        .items
        .iter()
        .map(|(&location, &pickup)| Report::Taken {
            pickup,
            location: Some(location),
        })
        .collect();
    production
        .advance(AdvanceInput::Progress(&reports))
        .unwrap();
    let OwnedState::Progress(progress) = production.state() else {
        panic!("progress owner")
    };
    assert_eq!(
        progress.inventory.item_percentage(&Rules::reference()),
        Wrapping(100)
    );
    assert_ne!(**progress, Progress::default());
}
