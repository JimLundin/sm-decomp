//! The typed state layer's byte layout: every Session owner maps exactly its
//! RAM words, in both directions, and no two owners claim the same byte.
//!
//! Each owner test lists its fields against the RAM addresses written out
//! here, independently of `session_encoding`'s own tables, and checks every
//! path a field takes to and from RAM:
//!
//! - the field read back through `projected_byte` and through `encode`;
//! - the two bytes stored through `write_projected_byte`, touching no other
//!   field of the owner;
//! - the word decoded from an otherwise blank RAM image.
//!
//! A shifted address, two swapped fields or a dropped byte breaks at least one
//! of these. The whole-Session tests decode random RAM and require every
//! owned word to survive `visit`, `encode`, the projection and a second decode.
use std::collections::{BTreeMap, BTreeSet};

use sm_game::external_inputs::ExternalInputs;
use sm_game::ram::{CompatibilityRam, words as ram};
use sm_game::room_entry::world::WorldBinding;
use sm_game::session::Session;
use sm_game::session_encoding::{decode, encode, projected_byte, visit, write_projected_byte};

/// Work RAM, banks $7E and $7F.
const RAM: usize = 0x20000;

/// A field of an owner: `(address, index, accessor)`; the index selects an
/// array element for accessors that take one.
type Field = fn(&mut Session, usize) -> &mut u16;
type Lane = (usize, usize, Field);

fn session() -> Session {
    Session::with_world(WorldBinding::canonical_legacy())
}

fn read(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([ram[at], ram[at + 1]])
}

fn projected_word(s: &Session, at: usize) -> Option<u16> {
    Some(u16::from_le_bytes([
        projected_byte(s, at)?,
        projected_byte(s, at + 1)?,
    ]))
}

/// The bytes `encode` writes, by address; each address once.
fn encoded(s: &Session) -> BTreeMap<usize, u8> {
    let mut bytes = BTreeMap::new();
    encode(s, |at, span| {
        for (offset, &byte) in span.iter().enumerate() {
            bytes.insert(at + offset, byte);
        }
    });
    bytes
}

fn decoded(ram: &[u8]) -> Session {
    decode(&|at| ram[at], ram.len(), WorldBinding::canonical_legacy())
}

/// A deterministic xorshift byte stream.
struct Random(u64);
impl Random {
    fn byte(&mut self) -> u8 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 24) as u8
    }
    fn ram(seed: u64) -> Vec<u8> {
        let mut random = Self(seed);
        (0..RAM).map(|_| random.byte()).collect()
    }
}

/// A host over a Session's projection, as the compatibility game's is while
/// a Session is installed.
struct Projection(Session);
impl CompatibilityRam for Projection {
    fn compatibility_byte(&self, at: usize) -> u8 {
        projected_byte(&self.0, at).unwrap_or_else(|| panic!("{at:#x} is not projected"))
    }
    fn compatibility_write_byte(&mut self, at: usize, value: u8) {
        assert!(
            write_projected_byte(&mut self.0, at, value),
            "{at:#x} is not projected"
        );
    }
}

/// A value for lane `index` whose bytes differ from each other and from `base`.
fn value(index: usize, base: u16) -> u16 {
    let value = 0x1357u16.wrapping_mul(index as u16 + 1) ^ 0x8001;
    if value == base { value ^ 0x0101 } else { value }
}

/// Check every lane of `owner` along each path to and from RAM.
fn check_owner(owner: &str, lanes: &[Lane]) {
    let mut claimed = BTreeSet::new();
    for &(at, _, _) in lanes {
        assert!(
            claimed.insert(at) && claimed.insert(at + 1),
            "{owner}: two fields claim {at:#x}"
        );
    }
    let base = session();
    let base_values: Vec<u16> = lanes
        .iter()
        .map(|&(_, index, field)| *field(&mut base.clone(), index))
        .collect();
    let mut blank = vec![0u8; RAM];
    for (k, &(at, index, field)) in lanes.iter().enumerate() {
        let v = value(k, base_values[k]);

        // The whole-Session encode and decode paths are slow, so large
        // owners check them on a spread of lanes, always the first and last.
        let whole = lanes.len() <= 128 || k % 17 == 0 || k + 1 == lanes.len();

        let mut s = base.clone();
        *field(&mut s, index) = v;
        assert_eq!(
            projected_word(&s, at),
            Some(v),
            "{owner}: field -> projection {at:#x}"
        );
        if whole {
            let bytes = encoded(&s);
            assert_eq!(
                (bytes.get(&at).copied(), bytes.get(&(at + 1)).copied()),
                (Some(v as u8), Some((v >> 8) as u8)),
                "{owner}: field -> encode {at:#x}"
            );
        }

        let mut s = base.clone();
        let [low, high] = v.to_le_bytes();
        assert!(
            write_projected_byte(&mut s, at, low),
            "{owner}: store {at:#x}"
        );
        assert!(
            write_projected_byte(&mut s, at + 1, high),
            "{owner}: store {:#x}",
            at + 1
        );
        assert_eq!(
            *field(&mut s, index),
            v,
            "{owner}: projection -> field {at:#x}"
        );
        for (j, &(other, other_index, other_field)) in lanes.iter().enumerate() {
            if j != k {
                assert_eq!(
                    *other_field(&mut s, other_index),
                    base_values[j],
                    "{owner}: storing {at:#x} changed {other:#x}"
                );
            }
        }

        if whole {
            blank[at] = low;
            blank[at + 1] = high;
            let mut s = decoded(&blank);
            assert_eq!(*field(&mut s, index), v, "{owner}: RAM -> decode {at:#x}");
            blank[at] = 0;
            blank[at + 1] = 0;
        }
    }
}

/// `count` elements of an array field at `base`, `stride` bytes apart.
fn array(base: usize, stride: usize, count: usize, field: Field) -> impl Iterator<Item = Lane> {
    (0..count).map(move |index| (base + stride * index, index, field))
}

fn camera_lanes() -> Vec<Lane> {
    let mut lanes = Vec::new();
    lanes.extend(array(0x8f7, 2, 2, |s, i| {
        &mut s.room.camera.layer1_block[i]
    }));
    lanes.extend(array(0x8fb, 2, 2, |s, i| {
        &mut s.room.camera.layer2_block[i]
    }));
    lanes.extend(array(0x8ff, 2, 2, |s, i| {
        &mut s.room.camera.previous_layer1_block[i]
    }));
    lanes.extend(array(0x903, 2, 2, |s, i| {
        &mut s.room.camera.previous_layer2_block[i]
    }));
    lanes.extend(array(0x907, 2, 2, |s, i| &mut s.room.camera.bg1_block[i]));
    lanes.extend(array(0x90b, 2, 2, |s, i| &mut s.room.camera.bg2_block[i]));
    lanes.extend(array(0x90f, 4, 2, |s, i| {
        &mut s.room.camera.layer1_fraction[i]
    }));
    lanes.extend(array(0x911, 4, 2, |s, i| &mut s.room.camera.layer1[i]));
    lanes.extend(array(0x917, 2, 2, |s, i| &mut s.room.camera.layer2[i]));
    lanes.extend(array(0x91d, 2, 2, |s, i| &mut s.room.camera.bg1_offset[i]));
    lanes.extend(array(0x921, 2, 2, |s, i| &mut s.room.camera.bg2_scroll[i]));
    lanes.extend(array(0xb0a, 4, 2, |s, i| {
        &mut s.room.camera.ideal_layer1[i]
    }));
    lanes
}

#[test]
fn camera_round_trips_every_field() {
    check_owner("camera", &camera_lanes());
    let mut s = session();
    for (k, (_, index, field)) in camera_lanes().into_iter().enumerate() {
        *field(&mut s, index) = value(k, 0);
    }
    let expected = s.room.camera;
    assert_eq!(
        sm_game::camera::current(&Projection(s)),
        expected,
        "camera view"
    );
}

fn flow_lanes() -> Vec<Lane> {
    vec![
        (0x1f51, 0, |s, _| &mut s.flow.cinematic_function),
        (0x1f53, 0, |s, _| &mut s.flow.demo_timer),
        (0x1f55, 0, |s, _| &mut s.flow.demo_set),
        (0x1f57, 0, |s, _| &mut s.flow.demo_scene),
        (0x1f59, 0, |s, _| &mut s.flow.num_demo_sets),
    ]
}

#[test]
fn flow_round_trips_every_field() {
    check_owner("flow", &flow_lanes());
    use sm_game::flow::Word;
    let mut host = Projection(session());
    let words = [
        Word::CinematicFunction,
        Word::DemoTimer,
        Word::DemoSet,
        Word::DemoScene,
        Word::NumDemoSets,
    ];
    for (k, word) in words.into_iter().enumerate() {
        sm_game::flow::set(&mut host, word, value(k, 0));
    }
    let mut expected = session();
    for (k, (_, index, field)) in flow_lanes().into_iter().enumerate() {
        *field(&mut expected, index) = value(k, 0);
    }
    assert_eq!(host.0.flow, expected.flow, "flow view stores");
    assert_eq!(
        sm_game::flow::current(&host),
        expected.flow,
        "flow view reads"
    );
}

fn menu_lanes() -> Vec<Lane> {
    vec![
        (0x727, 0, |s, _| &mut s.menu.menu_index),
        (0x729, 0, |s, _| {
            &mut s.menu.pausemenu_start_lr_pressed_highlight_timer
        }),
        (0x72b, 0, |s, _| &mut s.menu.pausemenu_lr_animation_timer),
        (0x72d, 0, |s, _| {
            &mut s.menu.pausemenu_item_selector_animation_timer
        }),
        (0x72f, 0, |s, _| {
            &mut s.menu.pausemenu_reserve_tank_animation_timer
        }),
        (0x731, 0, |s, _| &mut s.menu.unused_word_7e0731),
        (0x733, 0, |s, _| {
            &mut s.menu.pausemenu_uparrow_animation_timer
        }),
        (0x735, 0, |s, _| {
            &mut s.menu.pausemenu_downarrow_animation_timer
        }),
        (0x737, 0, |s, _| {
            &mut s.menu.pausemenu_rightarrow_animation_timer
        }),
        (0x739, 0, |s, _| {
            &mut s.menu.pausemenu_leftarrow_animation_timer
        }),
        (0x73b, 0, |s, _| {
            &mut s.menu.pausemenu_palette_animation_timer
        }),
        (0x73d, 0, |s, _| &mut s.menu.unused_word_7e073d),
        (0x73f, 0, |s, _| &mut s.menu.pausemenu_lr_animation_frame),
        (0x741, 0, |s, _| {
            &mut s.menu.pausemenu_item_selector_animation_frame
        }),
        (0x743, 0, |s, _| {
            &mut s.menu.pausemenu_reserve_tank_animation_frame
        }),
        (0x745, 0, |s, _| &mut s.menu.unused_word_7e0745),
        (0x747, 0, |s, _| {
            &mut s.menu.pausemenu_uparrow_animation_frame
        }),
        (0x749, 0, |s, _| {
            &mut s.menu.pausemenu_downarrow_animation_frame
        }),
        (0x74b, 0, |s, _| {
            &mut s.menu.pausemenu_rightarrow_animation_frame
        }),
        (0x74d, 0, |s, _| {
            &mut s.menu.pausemenu_leftarrow_animation_frame
        }),
        (0x74f, 0, |s, _| {
            &mut s.menu.pausemenu_palette_animation_frame
        }),
        (0x751, 0, |s, _| {
            &mut s.menu.pausemenu_shoulder_button_highlight
        }),
        (0x753, 0, |s, _| &mut s.menu.pausemenu_button_label_mode),
        (0x755, 0, |s, _| {
            &mut s.menu.pausemenu_equipment_category_item
        }),
        (0x757, 0, |s, _| {
            &mut s.menu.pausemenu_reserve_tank_delay_ctr
        }),
        (0x759, 0, |s, _| &mut s.menu.unused_word_7e0759),
        (0x75b, 0, |s, _| &mut s.menu.unused_word_7e075b),
        (0x75d, 0, |s, _| &mut s.menu.unused_word_7e075d),
        (0x75f, 0, |s, _| &mut s.menu.unused_word_7e075f),
        (0x761, 0, |s, _| &mut s.menu.unused_word_7e0761),
        (0x763, 0, |s, _| &mut s.menu.pause_screen_mode),
    ]
}

#[test]
fn menu_state_round_trips_every_field() {
    check_owner("menu_state", &menu_lanes());
    let mut s = session();
    for (k, (_, index, field)) in menu_lanes().into_iter().enumerate() {
        *field(&mut s, index) = value(k, 0);
    }
    let expected = s.menu;
    let host = Projection(s);
    assert_eq!(sm_game::menu_state::current(&host), expected, "menu view");
    let words: Vec<u16> = (0..menu_lanes().len()).map(|k| value(k, 0)).collect();
    assert_eq!(
        expected.words().as_slice(),
        words.as_slice(),
        "menu words in RAM order"
    );
}

fn room_geometry_lanes() -> Vec<Lane> {
    vec![
        (ram::AREA_INDEX, 0, |s, _| &mut s.room.area),
        (ram::ROOM_WIDTH_IN_BLOCKS, 0, |s, _| &mut s.room.width),
        (ram::ROOM_HEIGHT_IN_BLOCKS, 0, |s, _| &mut s.room.height),
        (ram::ROOM_WIDTH_IN_SCROLLS, 0, |s, _| {
            &mut s.room.scroll_width
        }),
        (ram::ROOM_HEIGHT_IN_SCROLLS, 0, |s, _| {
            &mut s.room.scroll_height
        }),
    ]
}

fn room_effect_lanes() -> Vec<Lane> {
    vec![
        (ram::EARTHQUAKE_TYPE, 0, |s, _| {
            &mut s.feedback.earthquake_kind
        }),
        (ram::EARTHQUAKE_TIMER, 0, |s, _| {
            &mut s.feedback.earthquake_timer
        }),
        (ram::TOURIAN_ENTRANCE_STATUE_ANIMSTATE, 0, |s, _| {
            &mut s.mechanisms.statue_animation
        }),
        (ram::TOURIAN_ENTRANCE_STATUE_FINISHED, 0, |s, _| {
            &mut s.mechanisms.statue_finished
        }),
        (ram::ROOM_X_COORDINATE_ON_MAP, 0, |s, _| {
            &mut s.room.entry.map_x
        }),
        (ram::ROOM_Y_COORDINATE_ON_MAP, 0, |s, _| {
            &mut s.room.entry.map_y
        }),
        (ram::ELEVATOR_DOOR_PROPERTIES_ORIENTATION, 0, |s, _| {
            &mut s.room.entry.elevator_properties
        }),
    ]
}

/// The room's FX blending configurations and the Ceres status.
fn room_state_lanes() -> Vec<Lane> {
    vec![
        (0x1982, 0, |s, _| &mut s.room.fx_blending),
        (0x1984, 0, |s, _| &mut s.room.fx_layer3_blending),
        (0x093f, 0, |s, _| &mut s.room.ceres_status),
    ]
}

#[test]
fn room_state_words_round_trip() {
    check_owner("room state", &room_state_lanes());
}

#[test]
fn room_view_round_trips_every_field() {
    check_owner("room geometry", &room_geometry_lanes());
    check_owner("room effects", &room_effect_lanes());
    let mut s = session();
    for (k, (_, index, field)) in room_geometry_lanes()
        .into_iter()
        .chain(room_effect_lanes())
        .enumerate()
    {
        *field(&mut s, index) = value(k, 0);
    }
    let host = Projection(s);
    let geometry = sm_game::room_view::current(&host);
    let effects = sm_game::room_view::effects(&host);
    let values: Vec<u16> = (0..12).map(|k| value(k, 0)).collect();
    assert_eq!(
        [
            geometry.area,
            geometry.width,
            geometry.height,
            geometry.scroll_width,
            geometry.scroll_height
        ],
        values[..5],
        "room geometry view"
    );
    assert_eq!(
        [
            effects.earthquake_kind,
            effects.earthquake_timer,
            effects.statue_animation,
            effects.statue_finished,
            effects.map_x,
            effects.map_y,
            effects.elevator_properties,
        ],
        values[5..],
        "room effects view"
    );
}

/// The room entry's scroll, placement and transition words.
fn room_entry_lanes() -> Vec<Lane> {
    vec![
        (0x791, 0, |s, _| &mut s.room.entry.direction),
        (0x795, 0, |s, _| &mut s.room.entry.elevator_interlock),
        (0x797, 0, |s, _| &mut s.room.entry.enemy_interlock),
        (0x799, 0, |s, _| &mut s.room.entry.elevator_direction),
        (0x79d, 0, |s, _| &mut s.room.entry.room_index),
        (0x7ad, 0, |s, _| &mut s.room.entry.scroll_up),
        (0x7af, 0, |s, _| &mut s.room.entry.scroll_down),
        (0x7b1, 0, |s, _| &mut s.room.entry.previous_cre),
        (0x7b3, 0, |s, _| &mut s.room.entry.cre),
        (0x7c9, 0, |s, _| &mut s.room.entry.music_track),
        (0x7cb, 0, |s, _| &mut s.room.entry.music_data),
        (0x7cd, 0, |s, _| &mut s.room.entry.fx_key),
        (0x7cf, 0, |s, _| &mut s.room.entry.enemy_population_key),
        (0x7d1, 0, |s, _| &mut s.room.entry.enemy_tilesets_key),
        (0x7df, 0, |s, _| &mut s.room.entry.main_code_key),
        (0x7e9, 0, |s, _| &mut s.room.entry.scrolling_hook),
        (0x925, 0, |s, _| &mut s.room.entry.frame),
        (0x927, 0, |s, _| &mut s.room.entry.arrival_x),
        (0x929, 0, |s, _| &mut s.room.entry.arrival_y),
        (0x92b, 0, |s, _| &mut s.room.entry.speed_fraction),
        (0x92d, 0, |s, _| &mut s.room.entry.speed_whole),
        (0x92f, 0, |s, _| &mut s.room.entry.elevator_delay),
        (0x931, 0, |s, _| &mut s.room.entry.completion),
        (0x99c, 0, |s, _| &mut s.room.entry.transition_selector),
        (0xe16, 0, |s, _| &mut s.room.entry.elevator_flags),
        (0xe18, 0, |s, _| &mut s.room.entry.elevator_status),
        (0x0a9, 0, |s, _| &mut s.room.entry.irq_policy),
    ]
}

#[test]
fn room_entry_scroll_round_trips_every_field() {
    check_owner("room entry", &room_entry_lanes());
}

fn inventory_lanes() -> Vec<Lane> {
    vec![
        (0x9a2, 0, |s, _| &mut s.inventory.equipment.active),
        (0x9a4, 0, |s, _| &mut s.inventory.equipment.owned),
        (0x9a6, 0, |s, _| &mut s.inventory.beams.active),
        (0x9a8, 0, |s, _| &mut s.inventory.beams.owned),
        (0x9c0, 0, |s, _| &mut s.inventory.reserve_mode),
        (0x9c2, 0, |s, _| &mut s.inventory.health.current),
        (0x9c4, 0, |s, _| &mut s.inventory.health.maximum),
        (0x9c6, 0, |s, _| &mut s.inventory.ammunition[0].current),
        (0x9c8, 0, |s, _| &mut s.inventory.ammunition[0].maximum),
        (0x9ca, 0, |s, _| &mut s.inventory.ammunition[1].current),
        (0x9cc, 0, |s, _| &mut s.inventory.ammunition[1].maximum),
        (0x9ce, 0, |s, _| &mut s.inventory.ammunition[2].current),
        (0x9d0, 0, |s, _| &mut s.inventory.ammunition[2].maximum),
        (0x9d4, 0, |s, _| &mut s.inventory.reserve_capacity),
        (0x9d6, 0, |s, _| &mut s.inventory.reserve_health),
        (ram::SAMUS_RESERVE_MISSILES, 0, |s, _| {
            &mut s.inventory.reserve_missiles
        }),
        (0xa4c, 0, |s, _| &mut s.inventory.health_fraction),
        (0xa76, 0, |s, _| &mut s.inventory.hyper_beam),
    ]
}

#[test]
fn inventory_round_trips_every_field() {
    check_owner("inventory", &inventory_lanes());
}

fn liquid_lanes() -> Vec<Lane> {
    vec![
        (0x1978, 0, |s, _| &mut s.room.liquid_height),
        (0x197a, 0, |s, _| &mut s.room.liquid_target),
        (0x197c, 0, |s, _| &mut s.room.liquid_velocity),
        (0x197e, 0, |s, _| &mut s.room.liquid_options),
        (0x1980, 0, |s, _| &mut s.room.liquid_delay),
        (0x195c, 0, |s, _| &mut s.room.liquid.surface.fraction),
        (0x195e, 0, |s, _| &mut s.room.liquid.surface.whole),
        (0x1960, 0, |s, _| &mut s.room.liquid.lava_acid.fraction),
        (0x1962, 0, |s, _| &mut s.room.liquid.lava_acid.whole),
        (0x1964, 0, |s, _| &mut s.room.liquid.tilemap),
        (0x196c, 0, |s, _| &mut s.room.liquid.rising),
        (0x196e, 0, |s, _| &mut s.room.liquid.kind),
        (0x1970, 0, |s, _| &mut s.room.liquid.offset.fraction),
        (0x1972, 0, |s, _| &mut s.room.liquid.offset.whole),
        (0x1974, 0, |s, _| &mut s.room.liquid.tide_phase),
        (0x1976, 0, |s, _| &mut s.room.liquid.base_fraction),
    ]
}

#[test]
fn liquid_round_trips_every_field() {
    check_owner("liquid", &liquid_lanes());
    let mut s = session();
    for (k, (_, index, field)) in liquid_lanes().into_iter().enumerate() {
        *field(&mut s, index) = value(k, 0);
    }
    let expected = sm_game::liquid::LiquidWords::of(&s.room);
    let host = Projection(s);
    assert_eq!(
        sm_game::liquid::LiquidWords::read(&host),
        expected,
        "liquid view"
    );
}

fn enemy_projectile_lanes() -> Vec<Lane> {
    const SLOTS: usize = 18;
    let mut lanes: Vec<Lane> = vec![
        (0x198d, 0, |s, _| &mut s.enemy_projectiles.flags),
        (0x1991, 0, |s, _| &mut s.enemy_projectiles.selected),
        (0x1993, 0, |s, _| &mut s.enemy_projectiles.spawn.primary),
        (0x1995, 0, |s, _| &mut s.enemy_projectiles.spawn.secondary),
        (ram::HEALTH_DROP_BIAS_FLAG, 0, |s, _| {
            &mut s.enemy_projectiles.drop_bias
        }),
    ];
    lanes.extend(array(0x1997, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].definition
    }));
    lanes.extend(array(0x19bb, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].graphics
    }));
    lanes.extend(array(0x19df, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].counter
    }));
    lanes.extend(array(0x1a03, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].pre_action
    }));
    lanes.extend(array(0x1a27, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].x.fraction
    }));
    lanes.extend(array(0x1a4b, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].x.whole
    }));
    lanes.extend(array(0x1a6f, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].y.fraction
    }));
    lanes.extend(array(0x1a93, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].y.whole
    }));
    lanes.extend(array(0x1ab7, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].velocity.x
    }));
    lanes.extend(array(0x1adb, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].velocity.y
    }));
    lanes.extend(array(0x1aff, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].behavior_a
    }));
    lanes.extend(array(0x1b23, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].behavior_b
    }));
    lanes.extend(array(0x1b47, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].program
    }));
    lanes.extend(array(0x1b6b, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].spritemap
    }));
    lanes.extend(array(0x1b8f, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].delay
    }));
    lanes.extend(array(0x1bb3, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].radius
    }));
    lanes.extend(array(0x1bd7, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].properties
    }));
    lanes.extend(array(0x1bfb, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].behavior_c
    }));
    lanes.extend(array(0x97dc, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].angle
    }));
    lanes.extend(array(0xf380, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].drop.flags
    }));
    lanes.extend(array(0xf3c8, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].drop.enemy_definition
    }));
    lanes.extend(array(0xf410, 2, SLOTS, |s, i| {
        &mut s.enemy_projectiles.slots[i].drop.killed_enemy
    }));
    lanes
}

#[test]
fn enemy_projectile_table_round_trips_every_field() {
    check_owner("enemy projectile table", &enemy_projectile_lanes());
    // The spawn angle is the one byte-wide word of the table.
    let mut s = session();
    s.enemy_projectiles.spawn.angle = 0xa5;
    assert_eq!(
        projected_byte(&s, 0x60b),
        Some(0xa5),
        "spawn angle -> projection"
    );
    assert_eq!(
        encoded(&s).get(&0x60b),
        Some(&0xa5),
        "spawn angle -> encode"
    );
    let mut s = session();
    assert!(write_projected_byte(&mut s, 0x60b, 0x5a));
    assert_eq!(
        s.enemy_projectiles.spawn.angle, 0x5a,
        "projection -> spawn angle"
    );
    let mut blank = vec![0u8; RAM];
    blank[0x60b] = 0x3c;
    assert_eq!(
        decoded(&blank).enemy_projectiles.spawn.angle,
        0x3c,
        "RAM -> spawn angle"
    );
}

fn visual_fade_lanes() -> Vec<Lane> {
    let mut lanes: Vec<Lane> = vec![
        (0xc400, 0, |s, _| {
            &mut s.presentation.visual_mut().palette_fade.step
        }),
        (0xc402, 0, |s, _| {
            &mut s.presentation.visual_mut().palette_fade.duration
        }),
        (0xc404, 0, |s, _| {
            &mut s.presentation.visual_mut().palette_fade.cursor
        }),
        (0x723, 0, |s, _| {
            &mut s.presentation.visual_mut().screen_fade.delay
        }),
        (0x725, 0, |s, _| {
            &mut s.presentation.visual_mut().screen_fade.counter
        }),
    ];
    lanes.extend(array(0xc000, 2, 256, |s, i| {
        &mut s.presentation.visual_mut().colors[i]
    }));
    lanes.extend(array(0xc200, 2, 256, |s, i| {
        &mut s.presentation.visual_mut().targets[i]
    }));
    lanes
}

#[test]
fn visual_fades_round_trip_every_field() {
    check_owner("visual fades", &visual_fade_lanes());
}

fn hdma_object_lanes() -> Vec<Lane> {
    const OBJECTS: usize = sm_game::hdma::OBJECTS;
    let mut lanes: Vec<Lane> = vec![
        (0x18b0, 0, |s, _| &mut s.hdma.enabled),
        (0x18b2, 0, |s, _| &mut s.hdma.selected),
    ];
    lanes.extend(array(0x18b4, 2, OBJECTS, |s, i| {
        &mut s.hdma.objects[i].channels
    }));
    lanes.extend(array(0x18c0, 2, OBJECTS, |s, i| {
        &mut s.hdma.objects[i].bank_slot
    }));
    lanes.extend(array(0x18cc, 2, OBJECTS, |s, i| {
        &mut s.hdma.objects[i].instruction_list
    }));
    lanes.extend(array(0x18d8, 2, OBJECTS, |s, i| {
        &mut s.hdma.objects[i].table
    }));
    lanes.extend(array(0x18e4, 2, OBJECTS, |s, i| {
        &mut s.hdma.objects[i].instruction_timer
    }));
    lanes.extend(array(0x18f0, 2, OBJECTS, |s, i| {
        &mut s.hdma.objects[i].pre_instruction
    }));
    lanes.extend(array(0x18fc, 2, OBJECTS, |s, i| {
        &mut s.hdma.objects[i].pre_instruction_bank
    }));
    lanes.extend(array(0x1908, 2, OBJECTS, |s, i| {
        &mut s.hdma.objects[i].timer
    }));
    lanes.extend(array(0x1914, 2, OBJECTS, |s, i| &mut s.hdma.objects[i].a));
    lanes.extend(array(0x1920, 2, OBJECTS, |s, i| &mut s.hdma.objects[i].b));
    lanes.extend(array(0x192c, 2, OBJECTS, |s, i| &mut s.hdma.objects[i].c));
    lanes.extend(array(0x1938, 2, OBJECTS, |s, i| &mut s.hdma.objects[i].d));
    lanes.extend(array(0x1944, 2, OBJECTS, |s, i| &mut s.hdma.objects[i].e));
    lanes.extend(array(0x1950, 2, OBJECTS, |s, i| &mut s.hdma.objects[i].f));
    lanes
}

#[test]
fn hdma_objects_round_trip_every_field() {
    assert_eq!(sm_game::hdma::OBJECTS, 6, "six HDMA objects at $18B4");
    check_owner("HDMA objects", &hdma_object_lanes());
}

/// Extended actor RAM $7E:7800..$7E:9000 in 2 KiB blocks, without the five
/// Botwoon death words, which the Session holds separately.
fn actor_extended_lanes() -> Vec<Lane> {
    const BOTWOON: [usize; 5] = [0x801e, 0x8020, 0x803c, 0x8818, 0x883e];
    (0..0x1800 / 2)
        .map(|word| 0x7800 + 2 * word)
        .filter(|at| !BOTWOON.contains(at))
        .map(|at| -> Lane {
            (at, at - 0x7800, |s, offset| {
                &mut s.actors.extended.blocks[offset / 0x800][offset % 0x800 / 2]
            })
        })
        .collect()
}

#[test]
fn actor_extended_ram_round_trips_every_word() {
    check_owner("actor extended RAM", &actor_extended_lanes());
}

#[test]
fn actor_speed_scratch_round_trips_every_word() {
    check_owner("actor speed scratch", &actor_scratch_lanes());
}

fn actor_scratch_lanes() -> Vec<Lane> {
    array(0x0e22, 2, 5, |s, index| {
        &mut s.actors.compatibility.scratch[index]
    })
    .collect()
}

#[test]
fn crocomire_melt_round_trips_every_word() {
    check_owner("Crocomire melt", &crocomire_melt_lanes());
}

fn crocomire_melt_lanes() -> Vec<Lane> {
    array(0x688, 2, 10, |s, index| {
        &mut s.actors.crocomire_melt.words[index]
    })
    .collect()
}

#[test]
fn boss_ram_round_trips_every_word() {
    check_owner("boss RAM", &boss_ram_lanes());
}

fn boss_ram_lanes() -> Vec<Lane> {
    array(0x9000, 2, 0x3ee, |s, index| {
        &mut s.actors.extended.blocks[3][index]
    })
    .collect()
}

#[test]
fn botwoon_death_words_round_trip() {
    check_owner(
        "Botwoon death",
        &[
            (0x801e, 0, |s, _| &mut s.botwoon_death.pending_hit),
            (0x8020, 0, |s, _| &mut s.botwoon_death.active),
            (0x803c, 0, |s, _| &mut s.botwoon_death.tail_ready),
            (0x8818, 0, |s, _| &mut s.botwoon_death.previous_health),
            (0x883e, 0, |s, _| &mut s.botwoon_death.body_landed),
        ],
    );
}

#[test]
fn projection_words_are_little_endian_byte_pairs() {
    let mut host = Projection(session());
    sm_game::projection::set_word(&mut host, 0x1f53, 0xbeef);
    assert_eq!(
        host.0.flow.demo_timer, 0xbeef,
        "set_word stores low then high"
    );
    assert_eq!(
        projected_byte(&host.0, 0x1f53),
        Some(0xef),
        "low byte first"
    );
    assert_eq!(
        projected_byte(&host.0, 0x1f54),
        Some(0xbe),
        "high byte second"
    );
    assert_eq!(
        sm_game::projection::word(&host, 0x1f53),
        0xbeef,
        "word reads the pair"
    );
    sm_game::projection::set_byte(&mut host, 0x1f54, 0x12);
    assert_eq!(sm_game::projection::byte(&host, 0x1f54), 0x12, "byte store");
    assert_eq!(
        host.0.flow.demo_timer, 0x12ef,
        "a byte store changes one lane"
    );
}

type Observation = (&'static str, usize, fn(&ExternalInputs) -> u16);

#[test]
fn inputs_read_every_observation_from_its_address() {
    let observations: &[Observation] = &[
        (
            "debug_enemy_freeze",
            ram::DEBUG_TIME_FROZEN_FOR_ENEMIES,
            |i| i.debug_enemy_freeze,
        ),
        ("blending_config", 0x1982, |i| i.blending_config),
        ("lava_y", 0x1962, |i| i.lava_y),
        ("current_enemy", ram::CUR_ENEMY_INDEX, |i| i.current_enemy),
        ("random", ram::RANDOM_NUMBER, |i| i.random),
        ("random_enemy_counter", ram::RANDOM_ENEMY_COUNTER, |i| {
            i.random_enemy_counter
        }),
        ("fx_y", ram::FX_Y_POS, |i| i.fx_y),
        (
            "drop_origin_x",
            ram::SPECIAL_DEATH_ITEM_DROP_X_ORIGIN_POS,
            |i| i.drop_origin_x,
        ),
        (
            "drop_origin_y",
            ram::SPECIAL_DEATH_ITEM_DROP_Y_ORIGIN_POS,
            |i| i.drop_origin_y,
        ),
        ("shoot_button", ram::BUTTON_CONFIG_SHOOT_X, |i| {
            i.shoot_button
        }),
        ("jump_button", ram::BUTTON_CONFIG_JUMP_A, |i| i.jump_button),
        ("cancel_button", ram::BUTTON_CONFIG_ITEMCANCEL_Y, |i| {
            i.cancel_button
        }),
        ("switch_button", ram::BUTTON_CONFIG_ITEMSWITCH, |i| {
            i.switch_button
        }),
        ("run_button", ram::BUTTON_CONFIG_RUN_B, |i| i.run_button),
        ("aim_up_button", ram::BUTTON_CONFIG_AIM_UP_R, |i| {
            i.aim_up_button
        }),
        ("aim_down_button", ram::BUTTON_CONFIG_AIM_DOWN_L, |i| {
            i.aim_down_button
        }),
        ("held", ram::JOYPAD1_LASTKEYS, |i| i.held),
        ("pressed", ram::JOYPAD1_NEWKEYS, |i| i.pressed),
        ("filtered_held", ram::JOYPAD1_INPUT_SAMUSFILTER, |i| {
            i.filtered_held
        }),
        ("filtered_pressed", ram::JOYPAD1_NEWINPUT_SAMUSFILTER, |i| {
            i.filtered_pressed
        }),
        ("frozen", ram::TIME_IS_FROZEN_FLAG, |i| i.frozen),
        ("cinematic", 0x1f51, |i| i.cinematic),
        ("hud_auto_cancel", ram::HUD_AUTO_CANCEL_FLAG, |i| {
            i.hud_auto_cancel
        }),
        ("background_page_x", 0x91d, |i| i.background_page_x),
        ("power_bomb_active", 0x592, |i| i.power_bomb_active),
        ("camera_x", 0x911, |i| i.camera_x),
        ("camera_y", 0x915, |i| i.camera_y),
        ("frame", ram::NMI_FRAME_COUNTER_WORD, |i| i.frame),
        ("ceres", ram::CERES_STATUS, |i| i.ceres),
        (
            "grapple_x",
            ram::X_POS_OF_START_OF_GRAPPLE_BEAM_PREVFRAME,
            |i| i.grapple_x,
        ),
        (
            "grapple_y",
            ram::Y_POS_OF_START_OF_GRAPPLE_BEAM_PREVFRAME,
            |i| i.grapple_y,
        ),
        ("game_state", 0x998, |i| i.transitions.game_state),
        ("game_time_frames", 0x9da, |i| {
            i.transitions.game_time_frames
        }),
        ("debug_invincibility", 0xde0, |i| {
            i.transitions.debug_invincibility
        }),
        ("moonwalk_enabled", 0x9e4, |i| {
            i.transitions.moonwalk_enabled
        }),
        ("turn_request", 0xdf8, |i| i.transitions.turn_request),
        ("fx_kind", 0x196e, |i| i.transitions.fx_kind),
    ];
    for seed in [1, 0x5eed, 0xdead_beef] {
        let ram = Random::ram(seed);
        let inputs = sm_game::session_encoding::inputs(&|at| ram[at]);
        for &(name, at, observe) in observations {
            assert_eq!(observe(&inputs), read(&ram, at), "{name} reads {at:#x}");
        }
        assert_eq!(
            inputs.nmi_counter_low, ram[0x5b5],
            "nmi_counter_low reads 0x5b5"
        );
        assert_eq!(
            inputs.enemy_zero_health,
            u16::from_ne_bytes([ram[0xf8c], ram[0xf8d]]),
            "enemy_zero_health reads 0xf8c in native order"
        );
    }
}

/// Every owner's table, for the cross-owner overlap check.
fn all_owner_lanes() -> Vec<(&'static str, Vec<Lane>)> {
    vec![
        ("camera", camera_lanes()),
        ("flow", flow_lanes()),
        ("menu_state", menu_lanes()),
        ("room geometry", room_geometry_lanes()),
        ("room effects", room_effect_lanes()),
        ("room entry", room_entry_lanes()),
        ("inventory", inventory_lanes()),
        ("liquid", liquid_lanes()),
        ("enemy projectile table", enemy_projectile_lanes()),
        ("visual fades", visual_fade_lanes()),
        ("HDMA objects", hdma_object_lanes()),
        ("actor extended RAM", actor_extended_lanes()),
        ("actor speed scratch", actor_scratch_lanes()),
        ("Crocomire melt", crocomire_melt_lanes()),
        ("boss RAM", boss_ram_lanes()),
    ]
}

#[test]
fn no_two_owners_claim_the_same_byte() {
    let mut owners: BTreeMap<usize, &str> = BTreeMap::new();
    for (owner, lanes) in all_owner_lanes() {
        for (at, _, _) in lanes {
            for byte in [at, at + 1] {
                if let Some(other) = owners.insert(byte, owner) {
                    panic!("{owner} and {other} both claim {byte:#x}");
                }
            }
        }
    }
    // The encoder itself writes each byte of a decoded Session once.
    let s = decoded(&Random::ram(7));
    let mut writes: BTreeMap<usize, usize> = BTreeMap::new();
    encode(&s, |at, span| {
        for offset in 0..span.len() {
            *writes.entry(at + offset).or_default() += 1;
        }
    });
    let repeated: Vec<String> = writes
        .iter()
        .filter(|&(_, &count)| count > 1)
        .map(|(at, count)| format!("{at:#x} x{count}"))
        .collect();
    assert!(
        repeated.is_empty(),
        "bytes encoded more than once: {repeated:?}"
    );
    // And every byte an owner table claims is one the encoder writes.
    for (byte, owner) in owners {
        assert!(
            writes.contains_key(&byte),
            "{owner} claims {byte:#x}, which encode never writes"
        );
    }
}

#[test]
fn session_round_trips_random_ram() {
    for seed in [3, 0x0bad_cafe, 0x1234_5678_9abc] {
        let ram = Random::ram(seed);
        let s = decoded(&ram);

        // Every word the Session visits is the word it decoded.
        visit(&s, |at, word| {
            assert_eq!(word, read(&ram, at), "seed {seed:#x}: visit {at:#x}");
        });

        // encode writes the RAM bytes back, and the projection serves them.
        let bytes = encoded(&s);
        let mut image = vec![0u8; RAM];
        for (&at, &byte) in &bytes {
            assert_eq!(byte, ram[at], "seed {seed:#x}: encode {at:#x}");
            assert_eq!(
                projected_byte(&s, at),
                Some(byte),
                "seed {seed:#x}: projection {at:#x}"
            );
            image[at] = byte;
        }

        // Decoding only the encoded bytes gives the same Session: decode reads
        // no byte that encode does not write.
        assert!(
            decoded(&image) == s,
            "seed {seed:#x}: decode(encode(s)) != s"
        );

        // Each projected byte stores through the projection without moving
        // any other encoded byte.
        let mut s = s;
        for (&at, &byte) in &bytes {
            assert!(
                write_projected_byte(&mut s, at, byte ^ 0x5a),
                "seed {seed:#x}: store {at:#x}"
            );
            assert_eq!(
                projected_byte(&s, at),
                Some(byte ^ 0x5a),
                "seed {seed:#x}: stored {at:#x}"
            );
            assert!(write_projected_byte(&mut s, at, byte));
        }
        assert_eq!(
            encoded(&s),
            bytes,
            "seed {seed:#x}: restoring every byte restores the encoding"
        );
    }
}
