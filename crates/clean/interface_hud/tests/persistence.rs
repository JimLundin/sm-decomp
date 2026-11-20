//! Complete-State persistence, using invented cells rather than a saved projection.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::drop_non_drop,
    reason = "tests assert known invented ranges and serialized shapes, and explicitly discard the original State"
)]

use enum_map::enum_map;
use interface_hud::{
    Ammo, Ammunition, CancelButton, ColourSet, Controls, Dropped, Energy, Equipped, Flip,
    FrameCount, FrameInputs, Grid, HerMap, HudEvent, HudGrid, HudPictures, HudState, Icons, Look,
    MapCellSet, MiniMapInputs, MiniMapPower, Pair, Picture, Reserve, ReserveMode, Room, Rules,
    SelectButton, SelectableItem, Selection, Shown, Status, Stock, Tile,
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
            equipped: Equipped {
                grapple_beam: true,
                x_ray_scope: true,
            },
        },
        controls: Controls {
            select: SelectButton::Pressed,
            cancel: CancelButton::Held,
            can_switch: true,
            muted_by: None,
        },
        dropped_before_status_bar: Dropped::Nothing,
        dropped_after_status_bar: Dropped::Nothing,
        collected: None,
        mother_brain_falls: false,
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
    };
    let rules = Rules::reference();
    let events = control.frame_of_play(&rules, &pictures, &inputs);
    assert_eq!(restored.frame_of_play(&rules, &pictures, &inputs), events);
    assert_eq!(
        events,
        vec![
            HudEvent::Switched {
                item: Some(SelectableItem::Ammo(Ammo::SuperMissiles)),
            },
            HudEvent::Click
        ]
    );
    assert_eq!(restored, control);
    inputs.frame = FrameCount(0);
    inputs.controls.select = SelectButton::NotPressed;
    inputs.mini_map.room.cell = at(63, 30);
    assert_eq!(
        restored.frame_of_play(&rules, &pictures, &inputs),
        control.frame_of_play(&rules, &pictures, &inputs)
    );
    assert_eq!(restored, control);
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

fn restore_complete_work(original: interface_hud::HudWork) -> interface_hud::HudWork {
    let text = toml::to_string(&original).unwrap();
    drop(original);
    toml::from_str(&text).unwrap()
}

fn dense_refresh() -> interface_hud::DisplayInputs {
    let mut map = HerMap::default();
    for y in 0..32 {
        for x in 0..66 {
            map.pictures.set(
                at(x, y),
                look(u16::try_from((x + y * 66) % 1024).unwrap(), Flip::Both),
            );
            map.charted.set(at(x, y), (x + y) % 3 == 0);
        }
    }
    interface_hud::DisplayInputs {
        frame: FrameCount(17),
        status: Status {
            energy: Energy { now: u16::MAX, capacity: 1499 },
            reserve: Reserve { energy: 1, mode: ReserveMode::Manual },
            ammunition: Ammunition::new(enum_map! { Ammo::Missiles => Stock::default(), Ammo::SuperMissiles => Stock::default(), Ammo::PowerBombs => Stock::default() }).unwrap(),
            equipped: Equipped::default(),
        },
        muted_by: None,
        dropped_before_status_bar: Dropped::Nothing,
        dropped_after_status_bar: Dropped::Selection,
        collected: None,
        mother_brain_falls: false,
        runs: true,
        turned_on: true,
        position: Pair { x: 0, y: 0 },
        room: Room { cell: at(63, 30), blocks: Pair { x: 16, y: 16 } },
        map,
        map_collected: true,
    }
}

fn dense_boundaries() -> Vec<interface_hud::HudWork> {
    use interface_hud::{HudWork, ItemControlInputs};
    let refresh = dense_refresh();
    let controls = ItemControlInputs {
        select: SelectButton::Pressed,
        cancel: CancelButton::Pressed,
        can_switch: true,
        ammunition: refresh.status.ammunition,
        equipped: refresh.status.equipped,
    };
    let mut work = HudWork::new(state());
    let mut boundaries = vec![work.clone()];
    assert!(work.choose_controls(controls));
    boundaries.push(work.clone());
    assert!(work.apply());
    boundaries.push(work.clone());
    assert!(work.choose_display(&Rules::reference(), &pictures(), refresh));
    boundaries.push(work.clone());
    assert!(work.apply());
    boundaries.push(work);
    boundaries
}

#[test]
fn dense_work_restores_every_boundary_and_independent_initial_highlight() {
    use interface_hud::Observation;
    let boundaries = dense_boundaries();
    let complete = boundaries.last().unwrap();
    assert_eq!(
        complete.events(),
        [HudEvent::Switched { item: None }, HudEvent::Click]
    );
    assert_eq!(
        complete.initial_highlight(),
        Some(SelectableItem::XRayScope)
    );
    assert_eq!(complete.state().selection, Selection::None);
    for original in boundaries.clone() {
        let expected = original.clone();
        let mut fresh = restore_complete_work(original);
        assert_eq!(fresh, expected);
        if matches!(fresh.controls(), Observation::Unread) {
            let Observation::Completed(answer) = complete.controls() else {
                panic!("controls complete")
            };
            assert!(fresh.choose_controls(answer.inputs));
        }
        fresh.apply();
        if matches!(fresh.display(), Observation::Unread) {
            assert!(fresh.choose_display(&Rules::reference(), &pictures(), dense_refresh()));
        }
        fresh.apply();
        assert_eq!(&fresh, complete);
        assert!(!fresh.apply());
    }
    // Energy equals its cache despite independently stale digits/tanks: no forced redraw.
    assert_eq!(
        complete.state().grid.get(at(6, 2)),
        state().grid.get(at(6, 2))
    );
    assert_eq!(complete.state().shown.energy, u16::MAX);
}

/// Exercise derived Serde's external enum shape as a whole value, independently of text.
fn restore_work_value(original: interface_hud::HudWork) -> interface_hud::HudWork {
    let value = toml::Value::try_from(&original).unwrap();
    drop(original);
    value.try_into().unwrap()
}

const fn ordinary_inputs(
    controls: interface_hud::ItemControlInputs,
    display: &interface_hud::DisplayInputs,
) -> FrameInputs<'_> {
    FrameInputs {
        frame: display.frame,
        status: display.status,
        controls: Controls {
            select: controls.select,
            cancel: controls.cancel,
            can_switch: controls.can_switch,
            muted_by: display.muted_by,
        },
        dropped_before_status_bar: display.dropped_before_status_bar,
        dropped_after_status_bar: display.dropped_after_status_bar,
        collected: display.collected,
        mother_brain_falls: display.mother_brain_falls,
        mini_map: MiniMapInputs {
            runs: display.runs,
            turned_on: display.turned_on,
            position: display.position,
            room: display.room,
            map: &display.map,
            map_collected: display.map_collected,
        },
    }
}

#[test]
fn all_event_identities_survive_complete_work_at_every_boundary() {
    use interface_hud::{HudWork, ItemControlInputs, Observation};
    let selected = SelectableItem::Ammo(Ammo::SuperMissiles);
    let cases = [
        (
            SelectButton::Pressed,
            CancelButton::Pressed,
            Some(HudEvent::Switched { item: None }),
        ),
        (
            SelectButton::Pressed,
            CancelButton::Held,
            Some(HudEvent::Switched {
                item: Some(selected),
            }),
        ),
        (SelectButton::NotPressed, CancelButton::NotPressed, None),
    ];
    for (select, cancel, switched) in cases {
        let mut display = dense_refresh();
        display.status.ammunition = Ammunition::new(enum_map! {
            Ammo::Missiles => Stock { count: 1, capacity: 999 },
            Ammo::SuperMissiles => Stock { count: 1, capacity: 99 },
            Ammo::PowerBombs => Stock { count: 1, capacity: 99 },
        })
        .unwrap();
        let controls = ItemControlInputs {
            select,
            cancel,
            can_switch: true,
            ammunition: display.status.ammunition,
            equipped: display.status.equipped,
        };
        let rules = Rules::reference();
        let pictures = pictures();
        let mut ordinary = state();
        let ordinary_events =
            ordinary.frame_of_play(&rules, &pictures, &ordinary_inputs(controls, &display));
        let expected_events: Vec<_> = switched.into_iter().chain([HudEvent::Click]).collect();
        assert_eq!(ordinary_events, expected_events);
        let mut work = HudWork::new(state());
        let mut boundaries = vec![work.clone()];
        assert!(work.choose_controls(controls));
        let Observation::Chosen(answer) = work.controls() else {
            panic!("chosen controls")
        };
        assert_eq!(answer.switched, switched);
        if cancel == CancelButton::Pressed {
            assert_eq!(answer.selection, Selection::None); // valid cancellation, not a sentinel
        }
        boundaries.push(work.clone());
        assert!(work.apply());
        assert_eq!(work.events(), switched.as_slice());
        boundaries.push(work.clone());
        assert!(work.choose_display(&rules, &pictures, display.clone()));
        boundaries.push(work.clone());
        assert!(work.apply());
        assert_eq!(work.state(), &ordinary);
        assert_eq!(work.events(), expected_events);
        boundaries.push(work.clone());

        // External enum identity lives outside the optional payload, in both the answer and
        // retained event list. TOML represents a no-item switch as an empty Switched table.
        let value = toml::Value::try_from(&work).unwrap();
        assert_eq!(
            value["events"].as_array().unwrap().last().unwrap().as_str(),
            Some("Click")
        );
        if let Some(HudEvent::Switched { item }) = switched {
            let payload = &value["events"][0]["Switched"];
            assert!(payload.is_table());
            assert_eq!(payload.get("item").is_some(), item.is_some());
            assert_eq!(
                &value["controls"]["Completed"]["switched"]["Switched"],
                payload
            );
        }
        for original in boundaries {
            for restore in [restore_complete_work, restore_work_value] {
                let expected = original.clone();
                let fresh = restore(original.clone());
                assert_eq!(fresh, expected); // all state, answers, progress and ordered events
                let mut fresh = finish_work(fresh, controls, display.clone(), &rules, &pictures);
                assert_eq!(fresh, work);
                assert_eq!(
                    fresh.publication(),
                    Some((&ordinary, expected_events.as_slice()))
                );
                assert!(!fresh.apply());
                assert!(!fresh.choose_controls(controls));
                assert!(!fresh.choose_display(&rules, &pictures, display.clone()));
                assert_eq!(fresh, work); // no repeated controls, drawing, exploration or requests
            }
        }
    }
}

fn finish_work(
    mut work: interface_hud::HudWork,
    controls: interface_hud::ItemControlInputs,
    display: interface_hud::DisplayInputs,
    rules: &Rules,
    pictures: &HudPictures,
) -> interface_hud::HudWork {
    work.choose_controls(controls);
    work.apply();
    work.choose_display(rules, pictures, display);
    work.apply();
    work
}

#[test]
fn changed_buttons_and_display_content_are_fresh_only_until_their_own_choice() {
    use interface_hud::{HudWork, Observation};
    let boundaries = dense_boundaries();
    let complete = boundaries.last().unwrap();
    let Observation::Completed(old_controls) = complete.controls() else {
        panic!("completed controls")
    };
    let Observation::Completed(old_display) = complete.display() else {
        panic!("completed display")
    };
    let old_controls = old_controls.clone();
    let old_display = old_display.clone();
    let mut buttons = old_controls.inputs;
    buttons.cancel = CancelButton::Held;
    buttons.can_switch = false;
    let mut refresh = dense_refresh();
    refresh.frame = FrameCount(0);
    refresh.status.energy.now = 7;
    refresh.status.reserve.mode = ReserveMode::Auto;
    refresh.position = Pair { x: 256, y: 0 };
    refresh.room.blocks.x = 32;
    refresh.dropped_before_status_bar = Dropped::OneShotItemAlone;
    refresh.dropped_after_status_bar = Dropped::Nothing;
    refresh.collected = Some(SelectableItem::XRayScope);
    refresh.map.pictures.set(at(63, 30), look(900, Flip::Down));
    refresh.map.charted.insert(at(32, 31));
    let mut fresh_pictures = pictures();
    fresh_pictures.digits[7] = look(901, Flip::Down);
    let mut fresh_rules = Rules::reference();
    fresh_rules.her_blink.frames_each_way = 4;
    for original in boundaries {
        let controls = if matches!(original.controls(), Observation::Unread) {
            buttons
        } else {
            old_controls.inputs
        };
        let display_unread = matches!(original.display(), Observation::Unread);
        let (display, rules, pictures) = if display_unread {
            (refresh.clone(), &fresh_rules, &fresh_pictures)
        } else {
            (
                old_display.inputs.clone(),
                &old_display.rules,
                &old_display.pictures,
            )
        };
        let expected = finish_work(HudWork::new(state()), controls, display, rules, pictures);
        let fresh = restore_complete_work(original);
        let actual = finish_work(
            fresh,
            buttons,
            refresh.clone(),
            &fresh_rules,
            &fresh_pictures,
        );
        assert_eq!(
            actual, expected,
            "each unread observation stays fresh; each chosen answer stays retained"
        );
    }
}

#[test]
fn complete_work_checks_nested_content_and_all_fixed_extents() {
    let boundaries = dense_boundaries();
    let value = toml::Value::try_from(&boundaries[3]).unwrap();
    for extra in [false, true] {
        for field in ["grid", "explored"] {
            let mut malformed = value.clone();
            change_length(
                malformed["state"][field]["rows"].as_array_mut().unwrap(),
                extra,
            );
            assert!(malformed.try_into::<interface_hud::HudWork>().is_err());
            let mut malformed = value.clone();
            change_length(
                malformed["state"][field]["rows"][0].as_array_mut().unwrap(),
                extra,
            );
            assert!(malformed.try_into::<interface_hud::HudWork>().is_err());
        }
        for field in ["pictures", "charted"] {
            let mut malformed = value.clone();
            change_length(
                malformed["display"]["Chosen"]["inputs"]["map"][field]["rows"]
                    .as_array_mut()
                    .unwrap(),
                extra,
            );
            assert!(malformed.try_into::<interface_hud::HudWork>().is_err());
            let mut malformed = value.clone();
            change_length(
                malformed["display"]["Chosen"]["inputs"]["map"][field]["rows"][0]
                    .as_array_mut()
                    .unwrap(),
                extra,
            );
            assert!(malformed.try_into::<interface_hud::HudWork>().is_err());
        }
    }
    for invalid in [-1, 1024, 65536] {
        let mut malformed = value.clone();
        malformed["display"]["Chosen"]["inputs"]["map"]["pictures"]["rows"][31][65]["picture"] =
            invalid.into();
        assert!(malformed.try_into::<interface_hud::HudWork>().is_err());
        let mut malformed = value.clone();
        malformed["display"]["Chosen"]["pictures"]["digits"][9]["picture"] = invalid.into();
        assert!(malformed.try_into::<interface_hud::HudWork>().is_err());
    }
    let mut malformed = value.clone();
    malformed["state"]["grid"]["rows"][2][31]["colours"] = 8.into();
    assert!(malformed.try_into::<interface_hud::HudWork>().is_err());
    let mut malformed = value;
    malformed["controls"]["Completed"]["inputs"]["ammunition"]["Missiles"]["count"] = 1000.into();
    assert!(malformed.try_into::<interface_hud::HudWork>().is_err());
}

#[test]
fn countdown_is_independent_of_actual_health_and_ordinary_refresh() {
    use interface_hud::{
        Countdown, CountdownExpiry, CountdownOpportunity, CountdownPicture, CountdownWork, HudWork,
        ItemControlInputs,
    };
    for energy in [0, 47, 999] {
        let mut original = state();
        original.countdown = Countdown::Dormant; // independently set and read
        assert_eq!(original.countdown, Countdown::Dormant);
        let mut display = dense_refresh();
        display.status.energy.now = energy;
        let controls = ItemControlInputs {
            select: SelectButton::Pressed,
            cancel: CancelButton::Pressed,
            can_switch: true,
            ammunition: display.status.ammunition,
            equipped: display.status.equipped,
        };
        let mut hud = HudWork::new(original.clone());
        assert!(hud.choose_controls(controls));
        assert!(hud.apply());
        assert!(hud.choose_display(&Rules::reference(), &pictures(), display.clone()));
        assert!(hud.apply());
        let (refreshed, events) = hud.publication().unwrap();
        assert_eq!(refreshed.countdown, Countdown::Dormant);
        assert_eq!(refreshed.shown.energy, energy);
        assert_eq!(events, [HudEvent::Switched { item: None }, HudEvent::Click]);
        let mut ordinary = original;
        assert_eq!(
            ordinary.frame_of_play(
                &Rules::reference(),
                &pictures(),
                &ordinary_inputs(controls, &display),
            ),
            events
        );
        assert_eq!(&ordinary, refreshed);
        let mut countdown = CountdownWork::new(refreshed.clone());
        assert!(countdown.publication().is_none()); // HUD refresh never implies maintenance
        assert!(countdown.choose(CountdownOpportunity::MaintainCountdown));
        assert!(countdown.apply());
        let (after, answer) = countdown.publication().unwrap();
        assert_eq!(after, refreshed); // map, grid, selection and every cache also preserved
        assert_eq!(answer.expiry, CountdownExpiry::NoExpiry);
        assert_eq!(answer.picture, CountdownPicture::Absent);
        assert_eq!(hud.events(), events); // ordered HUD outputs remain separate and unchanged
    }
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

#[test]
fn dense_countdown_work_restores_unread_chosen_and_completed_once() {
    let mut cases = vec![Selection::None];
    for item in SelectableItem::ORDER {
        cases.push(Selection::Item(item));
        cases.push(Selection::OneShot(item));
    }
    for selection in cases {
        for power in [MiniMapPower::On, MiniMapPower::Off] {
            let mut current = state();
            current.selection = selection;
            current.mini_map = power;
            // Keep the independently stale highlighted cache; never derive it from selection.
            check_countdown_restoration(&current);
        }
    }
}

fn check_countdown_restoration(current: &HudState) {
    use interface_hud::{CountdownOpportunity, CountdownWork, Observation};
    let mut work = CountdownWork::new(current.clone());
    let mut boundaries = vec![work.clone()];
    assert!(work.choose(CountdownOpportunity::MaintainCountdown));
    boundaries.push(work.clone());
    assert!(work.apply());
    boundaries.push(work.clone());
    for original in boundaries {
        let expected = original.clone();
        let text = toml::to_string(&original).unwrap();
        drop(original);
        let mut fresh: CountdownWork = toml::from_str(&text).unwrap();
        assert_eq!(fresh, expected); // all owned State fields, chosen answer and progress
        if matches!(fresh.answer(), Observation::Unread) {
            assert!(fresh.choose(CountdownOpportunity::MaintainCountdown)); // freshly qualified
        }
        fresh.apply();
        assert_eq!(fresh, work);
        assert_eq!(fresh.publication(), work.publication());
        assert!(!fresh.apply());
        assert!(!fresh.choose(CountdownOpportunity::MaintainCountdown));
        assert_eq!(fresh, work); // no extra answer/picture contribution
        assert_eq!(fresh.state(), current);
        // Completion consumes only this opportunity, not the next one selected by the scene.
        let mut next = CountdownWork::new(fresh.state().clone());
        assert_eq!(next.answer(), &Observation::Unread);
        assert!(next.choose(CountdownOpportunity::MaintainCountdown));
        assert!(next.apply());
        assert_eq!(next, work);
        assert!(!next.apply());
    }
}
