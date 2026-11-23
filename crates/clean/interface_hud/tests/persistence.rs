//! Saving and restoring the HUD's State between frames, using invented cells.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::drop_non_drop,
    reason = "tests assert known invented ranges and serialized shapes, and explicitly discard the original State"
)]

use enum_map::enum_map;
use interface_hud::{
    Ammo, Ammunition, CancelButton, ColourSet, Controls, CountdownRuns, Dropped, Energy, Equipped,
    Flip, FrameCount, FrameInputs, Grid, HerMap, HudEvent, HudGrid, HudPictures, HudState, Icons,
    Look, MapCellSet, MiniMapInputs, MiniMapPower, Pair, Picture, Reserve, ReserveMode, Room,
    Rules, SelectButton, SelectableItem, Selection, Shown, Status, Stock, StockAtStart, Tile,
    UnsupportedFrame,
};

const fn at(x: i32, y: i32) -> Pair<i32> {
    Pair { x, y }
}

const fn look(index: u16, flip: Flip) -> Look {
    Look {
        picture: Picture::new(index).unwrap(),
        flip,
    }
}

fn state() -> HudState {
    let mut grid = HudGrid::filled(Tile::new(look(0, Flip::None), ColourSet::new(0).unwrap()));
    for y in 0..3 {
        for x in 0..32 {
            let index = u16::try_from(y * 32 + x).unwrap();
            let flip = [Flip::None, Flip::Across, Flip::Down, Flip::Both][usize::from(index % 4)];
            grid.set(
                at(x, y),
                Tile::new(
                    look(index, flip),
                    ColourSet::new(u8::try_from(index % 8).unwrap()).unwrap(),
                ),
            );
        }
    }
    grid.set(
        at(31, 2),
        Tile::new(look(1023, Flip::Both), ColourSet::new(7).unwrap()),
    );
    let mut explored = MapCellSet::default();
    for y in 0..32 {
        for x in 0..64 {
            explored.set(at(x, y), (x * 3 + y) % 5 == 0);
        }
    }
    explored.insert(at(31, 30));
    explored.insert(at(32, 31));
    explored.insert(at(63, 31));
    HudState {
        grid,
        shown: Shown {
            energy: u16::MAX,
            ammo: enum_map! { Ammo::Missiles => 0, Ammo::SuperMissiles => 999, Ammo::PowerBombs => u16::MAX },
            highlighted: Some(SelectableItem::XRayScope),
        },
        selection: Selection::OneShot(SelectableItem::Ammo(Ammo::Missiles)),
        explored,
        mini_map: MiniMapPower::Off,
        countdown: interface_hud::Countdown::Dormant,
    }
}

fn restore(original: HudState) -> HudState {
    let text = toml::to_string(&original).unwrap();
    drop(original);
    toml::from_str(&text).unwrap()
}

#[test]
fn complete_state_round_trips_all_selection_and_power_cases() {
    let mut cases = vec![Selection::None];
    for item in SelectableItem::ORDER {
        cases.push(Selection::Item(item));
        cases.push(Selection::OneShot(item));
    }
    for selection in cases {
        for mini_map in [MiniMapPower::On, MiniMapPower::Off] {
            let mut control = state();
            control.selection = selection;
            control.mini_map = mini_map;
            control.shown.highlighted = selection.item();
            assert_eq!(restore(control.clone()), control);
        }
    }
}

fn pictures() -> HudPictures {
    HudPictures {
        blank: Picture::new(701).unwrap(),
        digits: std::array::from_fn(|i| look(710 + u16::try_from(i).unwrap(), Flip::Across)),
        full_tank: look(730, Flip::Both),
        empty_tank: look(731, Flip::Down),
        reserve_sign: [[look(740, Flip::Across); 2]; 3],
        icons: Icons {
            missiles: [[look(750, Flip::Down); 3]; 2],
            super_missiles: [[look(751, Flip::Both); 2]; 2],
            power_bombs: [[look(752, Flip::Across); 2]; 2],
            grapple_beam: [[look(753, Flip::None); 2]; 2],
            x_ray_scope: [[look(754, Flip::Down); 2]; 2],
        },
        map_blank: look(760, Flip::Across),
        diagonal: Picture::new(761).unwrap(),
    }
}

#[test]
fn fresh_restoration_continues_with_equal_complete_state_and_events() {
    let mut control = state();
    let mut restored = restore(control.clone());
    let pictures = pictures();
    let mut map = HerMap::default();
    for y in 0..32 {
        for x in 0..66 {
            map.pictures.set(
                at(x, y),
                look(u16::try_from((y * 66 + x) % 1024).unwrap(), Flip::Both),
            );
            map.charted.set(at(x, y), (x + y) % 3 == 0);
        }
    }
    let mut inputs = FrameInputs {
        frame: FrameCount(17),
        stock_at_start: StockAtStart {
            counts: enum_map! { Ammo::Missiles => 999, Ammo::SuperMissiles => 99, Ammo::PowerBombs => 0 },
            equipped: Equipped {
                grapple_beam: true,
                x_ray_scope: true,
            },
        },
        controls: Controls {
            select: SelectButton::Pressed,
            cancel: CancelButton::Held,
            can_switch: true,
        },
        mini_map: MiniMapInputs {
            runs: true,
            turned_on: true,
            position: Pair { x: 0, y: 0 },
            room: Room {
                cell: at(32, 30),
                blocks: Pair { x: 16, y: 16 },
            },
            map: &map,
            map_collected: true,
        },
        mother_brain_falls: false,
        status: Status {
            energy: Energy {
                now: 349,
                capacity: 1499,
            },
            reserve: Reserve {
                energy: 1,
                mode: ReserveMode::Auto,
            },
            ammunition: Ammunition::new(enum_map! {
                Ammo::Missiles => Stock { count: 999, capacity: 999 },
                Ammo::SuperMissiles => Stock { count: 99, capacity: 99 },
                Ammo::PowerBombs => Stock { count: 0, capacity: 99 },
            })
            .unwrap(),
        },
        collected: None,
        muted_by: None,
        dropped_before_status_bar: Dropped::Nothing,
        dropped_after_status_bar: Dropped::Nothing,
        countdown: CountdownRuns::MaintainCountdown,
    };
    let rules = Rules::reference();
    let outputs = control.frame_of_play(&rules, &pictures, &inputs).unwrap();
    assert_eq!(
        restored.frame_of_play(&rules, &pictures, &inputs),
        Ok(outputs.clone())
    );
    let super_missiles = SelectableItem::Ammo(Ammo::SuperMissiles);
    assert_eq!(
        outputs.events,
        vec![
            HudEvent::Switched {
                item: Some(super_missiles),
            },
            HudEvent::Click
        ]
    );
    assert_eq!(
        outputs.selection_after_controls,
        Selection::OneShot(super_missiles)
    );
    assert_eq!(restored, control);
    // A second frame, from the restored State and the original alike.
    inputs.frame = FrameCount(0);
    inputs.controls.select = SelectButton::NotPressed;
    inputs.mini_map.room.cell = at(63, 30);
    let restored = &mut restore(restored);
    assert_eq!(
        restored.frame_of_play(&rules, &pictures, &inputs),
        control.frame_of_play(&rules, &pictures, &inputs)
    );
    assert_eq!(*restored, control);
}

#[test]
fn a_frame_that_only_shows_the_countdown_is_refused_before_any_effect() {
    let map = HerMap::default();
    let mut hud = state();
    let before = hud.clone();
    let inputs = FrameInputs {
        frame: FrameCount(0),
        stock_at_start: StockAtStart {
            counts: enum_map! { Ammo::Missiles => 5, Ammo::SuperMissiles => 5, Ammo::PowerBombs => 5 },
            equipped: Equipped::default(),
        },
        controls: Controls {
            select: SelectButton::Pressed,
            cancel: CancelButton::NotPressed,
            can_switch: true,
        },
        mini_map: MiniMapInputs {
            runs: true,
            turned_on: true,
            position: Pair { x: 0, y: 0 },
            room: Room {
                cell: at(10, 10),
                blocks: Pair { x: 16, y: 16 },
            },
            map: &map,
            map_collected: false,
        },
        mother_brain_falls: false,
        status: Status {
            energy: Energy {
                now: 7,
                capacity: 99,
            },
            reserve: Reserve {
                energy: 0,
                mode: ReserveMode::Auto,
            },
            ammunition: Ammunition::new(enum_map! {
                Ammo::Missiles => Stock { count: 5, capacity: 5 },
                Ammo::SuperMissiles => Stock { count: 5, capacity: 5 },
                Ammo::PowerBombs => Stock { count: 5, capacity: 5 },
            })
            .unwrap(),
        },
        collected: None,
        muted_by: None,
        dropped_before_status_bar: Dropped::Selection,
        dropped_after_status_bar: Dropped::Nothing,
        countdown: CountdownRuns::PresentationOnly,
    };
    assert_eq!(
        hud.frame_of_play(&Rules::reference(), &pictures(), &inputs),
        Err(UnsupportedFrame::CountdownOnlyShown)
    );
    assert_eq!(hud, before);
}

#[test]
fn malformed_colour_and_picture_values_are_rejected_in_complete_state() {
    let value = toml::Value::try_from(state()).unwrap();
    for invalid in [-1, 8, 255, 256] {
        let mut malformed = value.clone();
        malformed["grid"]["rows"][2][31]["colours"] = toml::Value::Integer(invalid);
        assert!(
            malformed.try_into::<HudState>().is_err(),
            "colour {invalid}"
        );
    }
    for invalid in [-1, 1024, 65535, 65536] {
        let mut malformed = value.clone();
        malformed["grid"]["rows"][2][31]["look"]["picture"] = toml::Value::Integer(invalid);
        assert!(
            malformed.try_into::<HudState>().is_err(),
            "picture {invalid}"
        );
    }
    for number in 0..8 {
        let decoded: ColourSet = toml::Value::Integer(number).try_into().unwrap();
        assert_eq!(Some(decoded), ColourSet::new(u8::try_from(number).unwrap()));
        assert_eq!(
            toml::Value::try_from(decoded).unwrap(),
            toml::Value::Integer(number)
        );
    }
}

#[test]
fn complete_state_rejects_short_and_long_rows_and_columns() {
    let value = toml::Value::try_from(state()).unwrap();
    for field in ["grid", "explored"] {
        for extra in [false, true] {
            let mut malformed = value.clone();
            change_length(malformed[field]["rows"].as_array_mut().unwrap(), extra);
            assert!(
                malformed.try_into::<HudState>().is_err(),
                "{field} row extent"
            );
            let mut malformed = value.clone();
            change_length(malformed[field]["rows"][0].as_array_mut().unwrap(), extra);
            assert!(
                malformed.try_into::<HudState>().is_err(),
                "{field} column extent"
            );
        }
    }
}

fn change_length(array: &mut Vec<toml::Value>, extra: bool) {
    if extra {
        array.push(array[0].clone());
    } else {
        array.pop();
    }
}

#[test]
fn generic_grid_retains_copy_full_extents_and_outside_behavior() {
    let mut grid = Grid::<u16, 35, 34>::filled(0);
    for y in 0..34 {
        for x in 0..35 {
            grid.set(at(x, y), u16::try_from(y * 35 + x).unwrap());
        }
    }
    let copied = grid;
    let text = toml::to_string(&grid).unwrap();
    let mut restored: Grid<u16, 35, 34> = toml::from_str(&text).unwrap();
    assert_eq!(restored, copied);
    assert_eq!(restored.get(at(34, 33)), Some(1189));
    for place in [at(-1, 0), at(0, -1), at(35, 0), at(0, 34)] {
        assert_eq!(restored.get(place), None);
        restored.set(place, 42);
    }
    assert_eq!(restored, grid);
    let empty = Grid::<bool, 0, 0>::default();
    let decoded: Grid<bool, 0, 0> = toml::from_str(&toml::to_string(&empty).unwrap()).unwrap();
    assert_eq!(decoded, empty);
}

#[test]
fn complete_state_requires_its_owned_admitted_countdown() {
    let value = toml::Value::try_from(state()).unwrap();
    assert_eq!(value["countdown"].as_str(), Some("Dormant"));
    let mut missing = value.clone();
    missing.as_table_mut().unwrap().remove("countdown");
    assert!(missing.try_into::<HudState>().is_err()); // no inferred value or serde default
    for unsupported in ["Active", "Expired"] {
        let mut invalid = value.clone();
        invalid["countdown"] = unsupported.into();
        assert!(invalid.try_into::<HudState>().is_err()); // outside admitted State
    }
}
