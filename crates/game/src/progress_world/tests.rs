use clean_types::Event;
use data::level::Level;
use data::plm::PlmPopulation;
use data::room::{
    ConditionalState, LevelData, MapPosition, Music, RoomKey, RoomState, Screens, Scrollers,
};

use super::*;

/// A state header at `$8F:at` holding nothing.
fn state_at(at: u16) -> RoomState {
    let nowhere = RomAddress::new(0x8f, 0);
    RoomState {
        at: RomAddress::new(0x8f, at),
        level: LevelData {
            at: nowhere,
            bytes: Vec::new(),
        },
        layers: Level::default(),
        tileset: 0,
        music: Music { data: 0, track: 0 },
        fx: nowhere,
        enemy_population: nowhere,
        enemy_set: nowhere,
        layer2_scroll: [0, 0],
        scroll: nowhere,
        scroll_areas: Vec::new(),
        special_xray_blocks: nowhere,
        main_code: nowhere,
        plm_population: nowhere,
        plms: PlmPopulation::default(),
        library_background: nowhere,
        setup_code: nowhere,
    }
}

/// A room in `area` whose states load on `conditions`, in order, then the default.
fn room(area: Area, conditions: &[room::Condition]) -> Room {
    Room {
        key: RoomKey::from_room_pointer(0x91f8),
        index: 0,
        area,
        map_position: MapPosition::default(),
        size: Screens {
            width: 1,
            height: 1,
        },
        scrollers: Scrollers { up: 0, down: 0 },
        cre: 0,
        doors: RomAddress::new(0x8f, 0),
        conditional: conditions
            .iter()
            .zip(0x9000..)
            .map(|(&condition, at)| ConditionalState {
                condition,
                state: state_at(at),
            })
            .collect(),
        default: state_at(0x8000),
    }
}

#[test]
fn taking_an_item_makes_it_hers_and_switched_on_and_empties_its_place() {
    let mut state = State::default();
    take(&mut state, Item::MorphBall, 0x1a);
    let equipment = state.samus.equipment;
    assert!(equipment.collected.contains(Item::MorphBall));
    assert!(equipment.equipped.contains(Item::MorphBall));
    assert!(is_emptied(&state.collected_items, 0x1a));
    assert!(!is_emptied(&state.collected_items, 0x1b));
    assert_eq!(state.collected_items.0[1], 1 << 0xa);
}

#[test]
fn a_place_with_bit_15_is_never_emptied() {
    let mut state = State::default();
    take(&mut state, Item::MorphBall, 0x801a);
    assert!(state.samus.equipment.collected.contains(Item::MorphBall));
    assert_eq!(state.collected_items, CollectedItems::default());
    assert!(!is_emptied(&state.collected_items, 0x801a));
}

#[test]
fn the_engine_keeps_what_the_model_gave_back() {
    let mut equipment = Equipment {
        energy: Gauge {
            current: 30,
            max: 199,
        },
        missiles: Gauge {
            current: 3,
            max: 20,
        },
        reserve_mode: samus::ReserveMode::Manual,
        ..Equipment::default()
    };
    equipment.collected.insert(Item::SpeedBooster);
    equipment.collected.insert(Item::XRayScope);
    equipment.equipped.insert(Item::XRayScope);
    let mut written = Equipment::default();
    store_inventory(&inventory(&equipment), &mut written);
    assert_eq!(written, equipment);
}

#[test]
fn her_motion_reads_the_items_switched_on() {
    let mut equipment = Equipment::default();
    equipment.collected.insert(Item::HiJumpBoots);
    equipment.collected.insert(Item::GravitySuit);
    equipment.equipped.insert(Item::GravitySuit);
    let motion = super::equipment(&equipment);
    assert!(!motion.hi_jump);
    assert!(motion.other_suit);
}

#[test]
fn the_first_state_whose_condition_holds_loads() {
    let landing = room(
        Area::Crateria,
        &[
            room::Condition::Event(Event::TimeBombSet),
            room::Condition::PowerBombs,
            room::Condition::Event(Event::PlanetAwake),
        ],
    );
    let mut progress = Progress::default();
    assert_eq!(room_state(&landing, &progress, None), Some(StateIndex(3)));
    progress.events[Event::PlanetAwake] = true;
    assert_eq!(room_state(&landing, &progress, None), Some(StateIndex(2)));
    progress.inventory.capacity[Supply::PowerBombs] = Wrapping(5);
    assert_eq!(room_state(&landing, &progress, None), Some(StateIndex(1)));
}

#[test]
fn the_room_s_own_area_and_door_decide() {
    let door = RomAddress::new(0x83, 0x8a4e);
    let torizo = room(
        Area::Crateria,
        &[
            room::Condition::BossDead(BossBits(4)),
            room::Condition::Door(door),
            room::Condition::MainAreaBossDead,
        ],
    );
    let mut progress = Progress::default();
    assert_eq!(
        room_state(&torizo, &progress, Some(door)),
        Some(StateIndex(1))
    );
    progress.defeated[Region::Norfair][BossKind::MainBoss] = true;
    assert_eq!(room_state(&torizo, &progress, None), Some(StateIndex(3)));
    progress.defeated[Region::Crateria][BossKind::Torizo] = true;
    assert_eq!(room_state(&torizo, &progress, None), Some(StateIndex(0)));
}

#[test]
fn a_boss_bit_no_kind_has_reads_no_states() {
    let odd = room(Area::Brinstar, &[room::Condition::BossDead(BossBits(8))]);
    assert_eq!(conditions(&odd), None);
    assert_eq!(room_state(&odd, &Progress::default(), None), None);
}
