//! A stationary HUD witness with inert drop and turn-on alternatives while unselected and On.
use super::{compare::Observation, data_failure, result::Outcome};
use enum_map::enum_map;
use interface_hud::{
    Ammo, Ammunition, CancelButton, ColourSet, Controls, Dropped, Energy, Equipped, Flip,
    FrameCount, FrameInputs, HerMap, HudEvent, HudGrid, HudPictures, HudState, Icons, Look,
    MapCellSet, MapPictures, MiniMapInputs, MiniMapPower, Mute, Pair, Picture, Place, Reserve,
    ReserveMode, Room, SelectButton, SelectableItem, Selection, Shown, Status, Stock, Tile,
};
use std::path::Path;

pub struct Content {
    pub pictures: HudPictures,
    maps: Vec<(data::room::Area, HerMap)>,
}
impl Content {
    pub fn load(path: &Path) -> Result<Self, Outcome> {
        let bytes = std::fs::read(path).map_err(|_| data_failure(None, "HUD ROM"))?;
        let pinned =
            data::rom::Rom::new(bytes).map_err(|_| data_failure(None, "HUD player ROM pin"))?;
        let rom = pinned.lorom();
        let p =
            data::hud::HudPictures::load(rom).map_err(|_| data_failure(None, "HUD pictures"))?;
        let maps =
            data::map::AreaMaps::load(rom).map_err(|_| data_failure(None, "HUD area maps"))?;
        let pictures = HudPictures {
            blank: p.blank,
            digits: p.digits,
            full_tank: p.full_tank,
            empty_tank: p.empty_tank,
            reserve_sign: p.reserve_sign,
            icons: Icons {
                missiles: p.icons.missiles,
                super_missiles: p.icons.super_missiles,
                power_bombs: p.icons.power_bombs,
                grapple_beam: p.icons.grapple_beam,
                x_ray_scope: p.icons.x_ray_scope,
            },
            map_blank: p.map_blank,
            diagonal: p.diagonal,
        };
        let mut converted = Vec::new();
        for map in maps.0 {
            let mut pictures = MapPictures::default();
            for y in 0_usize..32 {
                for x in 0_usize..66 {
                    let index = (x / 32)
                        .saturating_mul(0x400)
                        .saturating_add(y.saturating_mul(32))
                        .saturating_add(x % 32);
                    let value = if index < 0x800 {
                        map.tiles.get(index)
                    } else {
                        map.beyond.get(index.saturating_sub(0x800))
                    }
                    .ok_or_else(|| data_failure(None, "HUD imported map cell"))?;
                    pictures.set(place(x, y), look(*value)?);
                }
            }
            let mut charted = MapCellSet::default();
            for (y, row) in map.station.0.iter().enumerate() {
                for x in 0..64 {
                    charted.set(place(x, y), (row & (1_u64 << x)) != 0);
                }
            }
            converted.push((map.area, HerMap { pictures, charted }));
        }
        Ok(Self {
            pictures,
            maps: converted,
        })
    }
    pub(super) fn map(&self, wram: &[u8]) -> Result<&HerMap, Outcome> {
        let area =
            u8::try_from(word(wram, 0x079f)?).map_err(|_| data_failure(None, "HUD area width"))?;
        self.map_area(area)
    }

    pub(super) fn map_area(&self, area: u8) -> Result<&HerMap, Outcome> {
        let area = data::room::Area::try_from(area).map_err(|_| data_failure(None, "HUD area"))?;
        self.maps
            .iter()
            .find(|(each, _)| *each == area)
            .map(|(_, map)| map)
            .ok_or_else(|| data_failure(None, "HUD mapped area"))
    }
}
fn word(wram: &[u8], at: usize) -> Result<u16, Outcome> {
    let bytes = wram
        .get(at..at.saturating_add(2))
        .ok_or_else(|| data_failure(None, "HUD record word"))?;
    Ok(u16::from_le_bytes(
        bytes
            .try_into()
            .map_err(|_| data_failure(None, "HUD record word"))?,
    ))
}
fn place(x: usize, y: usize) -> Place {
    Pair {
        x: i32::try_from(x).unwrap_or(i32::MAX),
        y: i32::try_from(y).unwrap_or(i32::MAX),
    }
}
fn look(value: u16) -> Result<Look, Outcome> {
    Ok(Look {
        picture: Picture::new(value & 0x03ff).ok_or_else(|| data_failure(None, "HUD picture"))?,
        flip: match (value & 0x4000 != 0, value & 0x8000 != 0) {
            (false, false) => Flip::None,
            (true, false) => Flip::Across,
            (false, true) => Flip::Down,
            (true, true) => Flip::Both,
        },
    })
}
fn item(value: u16) -> Result<Option<SelectableItem>, Outcome> {
    Ok(match value {
        0 => None,
        1 => Some(SelectableItem::Ammo(Ammo::Missiles)),
        2 => Some(SelectableItem::Ammo(Ammo::SuperMissiles)),
        3 => Some(SelectableItem::Ammo(Ammo::PowerBombs)),
        4 => Some(SelectableItem::GrappleBeam),
        5 => Some(SelectableItem::XRayScope),
        _ => return Err(data_failure(None, "HUD item code")),
    })
}
fn bitmap(wram: &[u8]) -> Result<MapCellSet, Outcome> {
    let bytes = wram
        .get(0x07f7..0x08f7)
        .ok_or_else(|| data_failure(None, "HUD explored bitmap"))?;
    let mut set = MapCellSet::default();
    for (index, byte) in bytes.iter().enumerate() {
        for bit in 0..8 {
            set.set(
                place(
                    (index / 128)
                        .saturating_mul(32)
                        .saturating_add((index % 4).saturating_mul(8))
                        .saturating_add(bit),
                    (index % 128) / 4,
                ),
                (byte & (0x80 >> bit)) != 0,
            );
        }
    }
    Ok(set)
}
pub fn seed(wram: &[u8], _content: &Content) -> Result<HudState, Outcome> {
    let state = decode_state(wram)?;
    if state.selection != Selection::None || state.mini_map != MiniMapPower::On {
        return Err(Outcome::Unsupported {
            cut: None,
            dependency: "HUD drop/turn-on equivalence requires unselected powered-on seed"
                .to_owned(),
        });
    }
    Ok(state)
}

/// Decode independently observed countdown status; active and unknown values stay unsupported.
pub fn countdown_origin(wram: &[u8]) -> Result<interface_hud::Countdown, Outcome> {
    if word(wram, 0x0943)? != 0 {
        return Err(data_failure(
            None,
            "HUD current countdown outside dormant scope",
        ));
    }
    Ok(interface_hud::Countdown::Dormant)
}

/// Decode every independently observed HUD field without choosing a replay scope.
pub fn decode_state(wram: &[u8]) -> Result<HudState, Outcome> {
    let first = word(wram, 0xc608)?;
    let mut grid = HudGrid::filled(Tile::new(
        look(first)?,
        ColourSet::new(0).ok_or_else(|| data_failure(None, "HUD colours"))?,
    ));
    for cell in 0_usize..96 {
        let value = word(wram, 0xc608_usize.saturating_add(cell.saturating_mul(2)))?;
        if value & 0x2000 == 0 {
            return Err(data_failure(None, "HUD priority"));
        }
        let colours = ColourSet::new(
            u8::try_from((value >> 10) & 7).map_err(|_| data_failure(None, "HUD colours"))?,
        )
        .ok_or_else(|| data_failure(None, "HUD colours"))?;
        grid.set(
            place(cell % 32, cell / 32),
            Tile::new(look(value)?, colours),
        );
    }
    let selected = item(word(wram, 0x09d2)?)?;
    let one_shot = item(word(wram, 0x0a04)?)?;
    let selection = match (selected, one_shot) {
        (None, None) => Selection::None,
        (Some(i), None) => Selection::Item(i),
        (Some(i), Some(j)) if i == j => Selection::OneShot(i),
        _ => return Err(data_failure(None, "HUD one-shot relation")),
    };
    Ok(HudState {
        countdown: countdown_origin(wram)?,
        grid,
        selection,
        explored: bitmap(wram)?,
        mini_map: if word(wram, 0x05f7)? == 0 {
            MiniMapPower::On
        } else {
            MiniMapPower::Off
        },
        shown: Shown {
            energy: word(wram, 0x0a06)?,
            ammo: enum_map! {
            Ammo::Missiles => word(wram, 0x0a08)?, Ammo::SuperMissiles => word(wram, 0x0a0a)?,
            Ammo::PowerBombs => word(wram, 0x0a0c)?, },
            highlighted: item(word(wram, 0x0a0e)?)?,
        },
    })
}

pub struct OwnedInput<'a> {
    frame: FrameInputs<'a>,
}
impl OwnedInput<'_> {
    #[cfg(test)]
    pub const fn frame(&self) -> FrameInputs<'_> {
        self.frame
    }
    pub const fn input(&self) -> game::production::HudInput<'_> {
        game::production::HudInput::Play(&self.frame)
    }
}
/// Only externally owned quantities participate in this stationary interval check.
/// Empty physical controls and no pickup publication preserve the seed's unselected invariant.
pub fn validate_step(prior: &[u8], current: &[u8]) -> Result<(), Outcome> {
    let unsupported = || Outcome::Unsupported {
        cut: None,
        dependency: "HUD stationary ordinary quiet-producer premise".to_owned(),
    };
    if word(prior, 0x0998)? != 8
        || word(current, 0x0998)? != 8
        || word(current, 0x05b6)?.wrapping_sub(word(prior, 0x05b6)?) != 1
    {
        return Err(unsupported());
    }
    for at in [
        0x079b, 0x079f, 0x0789, 0x07a1, 0x07a3, 0x07a5, 0x07a7, 0x09a2, 0x09b8, 0x09ba, 0x09c0,
        0x09c2, 0x09c4, 0x09c6, 0x09c8, 0x09ca, 0x09cc, 0x09ce, 0x09d0, 0x09d6, 0x0a1c, 0x0a1e,
        0x0a42, 0x0a44, 0x0a78, 0x0af6, 0x0afa, 0x0d32, 0x09ea, 0x179c,
    ] {
        if word(prior, at)? != word(current, at)? {
            return Err(unsupported());
        }
    }
    Ok(())
}
/// Reject recorded publications outside this bounded stationary witness.
/// Unrecorded drops are represented by Nothing only under the unselected invariant:
/// each permitted drop leaves selection None. A turn-on similarly preserves an already On map.
pub fn validate_events(record: &oracle_record::FrameRecord) -> Result<(), Outcome> {
    if record.hits.iter().any(|hit| {
        matches!(
            hit.pc,
            0x90_bf25
                | 0x90_bf41
                | 0x90_c091
                | 0x90_c0a6
                | 0x80_99cf
                | 0x80_9a0e
                | 0x80_9a1e
                | 0x80_9a2e
                | 0x80_9a3e
                | 0x90_a7e2
        )
    }) {
        return Err(Outcome::Unsupported {
            cut: None,
            dependency: "HUD quiet event source contradicted".to_owned(),
        });
    }
    Ok(())
}
fn validate_quiet_controls(prior: &[u8]) -> Result<(), Outcome> {
    if word(prior, 0x008b)? != 0
        || word(prior, 0x008f)? != 0
        || word(prior, 0x09a2)? & 0xc000 != 0
        || word(prior, 0x09ea)? != 0
        || word(prior, 0x179c)? != 0
        || word(prior, 0x0d32)? != 0xc4f0
        || word(prior, 0x0a78)? != 0
    {
        return Err(Outcome::Unsupported {
            cut: None,
            dependency: "HUD admitted quiet upstream producer interval".to_owned(),
        });
    }
    Ok(())
}
pub fn input<'a>(prior: &[u8], content: &'a Content) -> Result<OwnedInput<'a>, Outcome> {
    validate_quiet_controls(prior)?;
    let stock = |at: usize| {
        Ok::<_, Outcome>(Stock {
            count: word(prior, at)?,
            capacity: word(prior, at.saturating_add(2))?,
        })
    };
    let ammunition = Ammunition::new(enum_map! { Ammo::Missiles => stock(0x09c6)?,
    Ammo::SuperMissiles => stock(0x09ca)?, Ammo::PowerBombs => stock(0x09ce)? })
    .ok_or_else(|| data_failure(None, "HUD ammunition range"))?;
    let equipment = word(prior, 0x09a2)?;
    let select = word(prior, 0x008f)? & word(prior, 0x09ba)? != 0;
    let pressed_cancel = word(prior, 0x008f)? & word(prior, 0x09b8)? != 0;
    let held_cancel = word(prior, 0x008b)? & word(prior, 0x09b8)? != 0;
    let muted_by = if word(prior, 0x0a78)? != 0 {
        Some(Mute::TimeStopped)
    } else if word(prior, 0x0d32)? != 0xc4f0 {
        Some(Mute::Grapple)
    } else {
        match word(prior, 0x0a1e)?.to_le_bytes()[1] {
            3 => Some(Mute::SpinJump),
            0x14 => Some(Mute::WallJump),
            _ => None,
        }
    };
    let runs = match word(prior, 0x0a44)? {
        0xe725 | 0xe7f5 | 0xe86a | 0xe8dc | 0xe8ec | 0xe8aa => true,
        0xe7d2 | 0xe833 | 0xe8d6 | 0xe8d9 | 0xe902 | 0xe8cd => false,
        _ => return Err(data_failure(None, "HUD mini-map handler")),
    };
    Ok(OwnedInput {
        frame: FrameInputs {
            frame: FrameCount(word(prior, 0x05b5)?.to_le_bytes()[0]),
            status: Status {
                energy: Energy {
                    now: word(prior, 0x09c2)?,
                    capacity: word(prior, 0x09c4)?,
                },
                reserve: Reserve {
                    energy: word(prior, 0x09d6)?,
                    mode: match word(prior, 0x09c0)? {
                        1 => ReserveMode::Auto,
                        0 | 2 => ReserveMode::Manual,
                        _ => return Err(data_failure(None, "HUD reserve mode")),
                    },
                },
                ammunition,
                equipped: Equipped {
                    grapple_beam: equipment & 0x4000 != 0,
                    x_ray_scope: equipment & 0x8000 != 0,
                },
            },
            controls: Controls {
                select: if select {
                    SelectButton::Pressed
                } else {
                    SelectButton::NotPressed
                },
                cancel: if pressed_cancel {
                    CancelButton::Pressed
                } else if held_cancel {
                    CancelButton::Held
                } else {
                    CancelButton::NotPressed
                },
                can_switch: matches!(word(prior, 0x0a42)?, 0xe695 | 0xe6c9)
                    && !matches!(word(prior, 0x0a1c)?, 0 | 0x9b),
                muted_by,
            },
            dropped_before_status_bar: Dropped::Nothing,
            dropped_after_status_bar: Dropped::Nothing,
            collected: None,
            mother_brain_falls: false,
            mini_map: MiniMapInputs {
                runs,
                turned_on: false,
                position: Pair {
                    x: word(prior, 0x0af6)?,
                    y: word(prior, 0x0afa)?,
                },
                room: room_from_words(
                    [word(prior, 0x07a1)?, word(prior, 0x07a3)?],
                    [word(prior, 0x07a5)?, word(prior, 0x07a7)?],
                ),
                map: content.map(prior)?,
                map_collected: word(prior, 0x0789)? != 0,
            },
        },
    })
}
pub(super) fn room_from_words([x, y]: [u16; 2], [width, height]: [u16; 2]) -> Room {
    Room {
        // Stored room Y counts from the row above the first map cell.
        cell: Pair {
            x: i32::from(x),
            y: i32::from(y).saturating_add(1),
        },
        blocks: Pair {
            x: width,
            y: height,
        },
    }
}

pub fn expected(wram: &[u8], _content: &Content) -> Result<Vec<Observation>, Outcome> {
    let mut fields = vec![
        Observation {
            field: "hud.grid".to_owned(),
            bytes: wram
                .get(0xc608..0xc6c8)
                .ok_or_else(|| data_failure(None, "HUD raw tiles"))?
                .to_vec(),
        },
        Observation {
            field: "hud.explored".to_owned(),
            bytes: wram
                .get(0x07f7..0x08f7)
                .ok_or_else(|| data_failure(None, "HUD raw explored"))?
                .to_vec(),
        },
    ];
    for (name, at) in [
        ("energy", 0x0a06),
        ("missiles", 0x0a08),
        ("supers", 0x0a0a),
        ("bombs", 0x0a0c),
        ("highlighted", 0x0a0e),
        ("selected", 0x09d2),
        ("one-shot", 0x0a04),
    ] {
        fields.push(Observation::quantity(
            format!("hud.{name}"),
            i64::from(word(wram, at)?),
        ));
    }
    fields.push(Observation::quantity(
        "hud.map-off",
        i64::from(word(wram, 0x05f7)? != 0),
    ));
    Ok(fields)
}
const fn item_code(item: Option<SelectableItem>) -> u16 {
    match item {
        None => 0,
        Some(SelectableItem::Ammo(Ammo::Missiles)) => 1,
        Some(SelectableItem::Ammo(Ammo::SuperMissiles)) => 2,
        Some(SelectableItem::Ammo(Ammo::PowerBombs)) => 3,
        Some(SelectableItem::GrappleBeam) => 4,
        Some(SelectableItem::XRayScope) => 5,
    }
}
pub(super) fn explored_bitmap(state: &HudState) -> Vec<u8> {
    let mut explored = Vec::new();
    for index in 0_usize..256 {
        let byte = (0..8).fold(0_u8, |byte, bit| {
            (byte << 1)
                | u8::from(
                    state.explored.contains(place(
                        (index / 128)
                            .saturating_mul(32)
                            .saturating_add((index % 4).saturating_mul(8))
                            .saturating_add(bit),
                        (index % 128) / 4,
                    )),
                )
        });
        explored.push(byte);
    }
    explored
}

pub fn initial(state: &HudState, _content: &Content) -> Result<Vec<Observation>, Outcome> {
    let tiles = game::production::hud::tilemap(state)
        .map_err(|_| data_failure(None, "HUD complete tilemap"))?;
    let mut fields = vec![
        Observation {
            field: "hud.grid".to_owned(),
            bytes: tiles,
        },
        Observation {
            field: "hud.explored".to_owned(),
            bytes: explored_bitmap(state),
        },
    ];
    for (name, value) in [
        ("energy", state.shown.energy),
        ("missiles", state.shown.ammo[Ammo::Missiles]),
        ("supers", state.shown.ammo[Ammo::SuperMissiles]),
        ("bombs", state.shown.ammo[Ammo::PowerBombs]),
        ("highlighted", item_code(state.shown.highlighted)),
        ("selected", item_code(state.selection.item())),
        ("one-shot", item_code(state.selection.one_shot())),
    ] {
        fields.push(Observation::quantity(
            format!("hud.{name}"),
            i64::from(value),
        ));
    }
    fields.push(Observation::quantity(
        "hud.map-off",
        i64::from(state.mini_map == MiniMapPower::Off),
    ));
    Ok(fields)
}
pub fn actual(
    state: &HudState,
    _events: &[HudEvent],
    content: &Content,
) -> Result<Vec<Observation>, Outcome> {
    initial(state, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires independently qualified corrected-origin WRAM"]
    fn complete_origin_restore_and_actual_tilemap_preserve_observations() {
        let path = std::env::var_os("SM_FRAME_HUD_WRAM").expect("independent first-cut WRAM");
        let wram = std::fs::read(path).unwrap();
        let state = decode_state(&wram).unwrap();
        let owner = game::production::hud::Hud::new(state.clone());
        let restored: game::production::hud::Hud =
            canonical::from_bytes(&canonical::to_bytes(&owner)).unwrap();
        assert!(restored.eq(&owner));
        let actual = game::production::hud::tilemap(restored.state()).unwrap();
        assert_eq!(actual.len(), 192);
        assert!(actual.iter().eq(wram.get(0xc608..0xc6c8).unwrap()));
        let mut changed_health = wram.clone();
        changed_health
            .get_mut(0x09c2..0x09c4)
            .unwrap()
            .copy_from_slice(&state.shown.energy.wrapping_add(1).to_le_bytes());
        assert!(decode_state(&changed_health).unwrap().eq(&state));
        for (at, value) in [
            (0xc608_usize, word(&wram, 0xc608).unwrap() & !0x2000),
            (0x09d2, 0),
        ] {
            let mut invalid = wram.clone();
            invalid
                .get_mut(at..at.saturating_add(2))
                .unwrap()
                .copy_from_slice(&value.to_le_bytes());
            if at == 0x09d2 {
                invalid
                    .get_mut(0x0a04..0x0a06)
                    .unwrap()
                    .copy_from_slice(&1_u16.to_le_bytes());
            }
            assert!(decode_state(&invalid).is_err());
        }
        let mut selected = wram;
        selected
            .get_mut(0x09d2..0x09d4)
            .unwrap()
            .copy_from_slice(&4_u16.to_le_bytes());
        selected
            .get_mut(0x0a04..0x0a06)
            .unwrap()
            .copy_from_slice(&4_u16.to_le_bytes());
        selected
            .get_mut(0x05f7..0x05f9)
            .unwrap()
            .copy_from_slice(&1_u16.to_le_bytes());
        let decoded = decode_state(&selected).unwrap();
        assert_eq!(
            decoded.selection,
            Selection::OneShot(SelectableItem::GrappleBeam)
        );
        assert_eq!(decoded.mini_map, MiniMapPower::Off);
    }
}
