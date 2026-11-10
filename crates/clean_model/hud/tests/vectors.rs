//! The HUD's vectors (`scenarios/vectors/hud.toml`) run through the model, and the catalogue
//! checked against them.
//!
//! The file is read with `toml` and `serde` into the shape declared once below, every table of
//! it closed to keys the file's header doesn't define and every name an enum of the names it
//! does; that shape is then turned into the model's own types.

#[cfg(test)]
mod shape {
    //! The vector file's shape, as its header defines it.

    use std::collections::BTreeMap;

    use clean_model_hud::{ColourSet, Event, Flip, Picture, SelectableItem, Tile};
    use serde::Deserialize;

    /// The whole file.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct VectorFile {
        pub pictures: PicturesIn,
        pub vector: Vec<VectorIn>,
    }

    /// `[pictures]`: the HUD's pictures by name.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct PicturesIn {
        pub blank: PictureIn,
        pub digits: [PictureIn; 10],
        pub full_tank: PictureIn,
        pub empty_tank: PictureIn,
        pub reserve_sign: [[PictureIn; 2]; 3],
        pub icons: IconsIn,
        pub map_blank: PictureIn,
        pub diagonal: PictureIn,
    }

    /// `[pictures.icons]`: each item's two rows.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields, rename_all = "PascalCase")]
    pub struct IconsIn {
        pub missiles: [[PictureIn; 3]; 2],
        pub super_missiles: [[PictureIn; 2]; 2],
        pub power_bombs: [[PictureIn; 2]; 2],
        pub grapple_beam: [[PictureIn; 2]; 2],
        pub x_ray_scope: [[PictureIn; 2]; 2],
    }

    /// One `[[vector]]`.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct VectorIn {
        pub id: String,
        /// Read only to refuse a source the header doesn't name.
        #[serde(rename = "source")]
        pub _source: Source,
        pub entries: Vec<String>,
        #[serde(default)]
        pub step: Step,
        pub frame: Option<u8>,
        #[serde(default)]
        pub before: BeforeIn,
        pub inputs: InputsIn,
        pub after: AfterIn,
    }

    /// Where a vector comes from.
    #[derive(Deserialize)]
    pub enum Source {
        #[serde(rename = "constructed")]
        Constructed,
    }

    /// The step a vector takes.
    #[derive(Deserialize, Default, Clone, Copy)]
    pub enum Step {
        #[default]
        Frame,
        BossEvent,
    }

    /// A vector's before.
    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields)]
    pub struct BeforeIn {
        pub hud: Option<[HudRowIn; 3]>,
        #[serde(default)]
        pub shown: ShownIn,
        #[serde(default)]
        pub selection: ItemName,
        #[serde(default)]
        pub one_shot: bool,
        #[serde(default)]
        pub mini_map_off: bool,
        #[serde(default)]
        pub explored: Vec<[i32; 2]>,
    }

    /// A vector's after: a field left out is as before.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct AfterIn {
        #[serde(default)]
        pub changed: BTreeMap<String, TileIn>,
        pub shown: Option<ShownIn>,
        pub selection: Option<ItemName>,
        pub one_shot: Option<bool>,
        pub mini_map_off: Option<bool>,
        pub explored: Option<Vec<[i32; 2]>>,
        #[serde(default)]
        pub events: Vec<EventIn>,
    }

    /// `shown`: each value left out is 0 or none.
    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields, default)]
    pub struct ShownIn {
        pub energy: u16,
        pub missiles: u16,
        pub super_missiles: u16,
        pub power_bombs: u16,
        pub selection: ItemName,
    }

    /// A vector's inputs; which are allowed depends on the step.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct InputsIn {
        pub energy: Option<[u16; 2]>,
        pub reserve: Option<u16>,
        pub auto_reserve: Option<bool>,
        pub missiles: Option<[u16; 2]>,
        pub super_missiles: Option<[u16; 2]>,
        pub power_bombs: Option<[u16; 2]>,
        pub equipped: Option<Vec<Equipment>>,
        pub select: Option<SelectIn>,
        pub cancel: Option<CancelIn>,
        pub can_switch: Option<bool>,
        pub shot_cancels: Option<bool>,
        pub muted_by: Option<MuteIn>,
        pub collected: Option<ItemName>,
        pub mini_map: Option<MiniMapIn>,
    }

    /// `mini_map`; which keys are allowed depends on the step.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct MiniMapIn {
        pub runs: Option<bool>,
        pub turned_on: Option<bool>,
        pub mother_brain_falls: Option<bool>,
        pub position: Option<[u16; 2]>,
        pub room: RoomIn,
        pub map: Option<MapIn>,
        pub boss: Option<BossName>,
    }

    /// `room`.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct RoomIn {
        pub cell: [i32; 2],
        pub blocks: Option<[u16; 2]>,
    }

    /// `map`.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct MapIn {
        #[serde(default)]
        pub collected: bool,
        #[serde(default)]
        pub pictures: BTreeMap<String, PictureIn>,
        #[serde(default)]
        pub charted: Vec<[i32; 2]>,
        #[serde(default)]
        pub beyond: BTreeMap<String, PictureIn>,
    }

    /// An item's name, or `"None"`.
    #[derive(Deserialize, Default, Clone, Copy)]
    pub enum ItemName {
        #[default]
        None,
        Missiles,
        SuperMissiles,
        PowerBombs,
        GrappleBeam,
        XRayScope,
    }

    /// What `equipped` names.
    #[derive(Deserialize, Clone, Copy)]
    pub enum Equipment {
        GrappleBeam,
        XRayScope,
    }

    /// Item select this frame.
    #[derive(Deserialize, Clone, Copy)]
    pub enum SelectIn {
        Pressed,
    }

    /// Item cancel this frame.
    #[derive(Deserialize, Clone, Copy)]
    pub enum CancelIn {
        Pressed,
        Held,
    }

    /// Why a switch makes no click.
    #[derive(Deserialize, Clone, Copy)]
    pub enum MuteIn {
        Grapple,
        SpinJump,
        TimeStopped,
        WallJump,
    }

    /// The boss whose room a `BossEvent` enters.
    #[derive(Deserialize, Clone, Copy)]
    pub enum BossName {
        Kraid,
        Draygon,
        Crocomire,
        Phantoon,
        RidleyInNorfair,
        RidleyInCeres,
    }

    /// A picture, `PPP` then `h` and `v` for its flips.
    #[derive(Deserialize, Clone, Copy)]
    #[serde(try_from = "String")]
    pub struct PictureIn(pub Picture);

    impl TryFrom<String> for PictureIn {
        type Error = String;
        fn try_from(text: String) -> Result<Self, String> {
            let split = text
                .find(|c: char| !c.is_ascii_hexdigit())
                .unwrap_or(text.len());
            let (number, flips) = text.split_at(split);
            let number =
                u16::from_str_radix(number, 16).map_err(|_| format!("bad picture {text}"))?;
            Ok(Self(Picture::new(number, flip(flips)?)))
        }
    }

    /// A HUD cell, `PPP:c` then `h` and `v` for its flips.
    #[derive(Deserialize, Clone, Copy)]
    #[serde(try_from = "String")]
    pub struct TileIn(pub Tile);

    impl TryFrom<String> for TileIn {
        type Error = String;
        fn try_from(text: String) -> Result<Self, String> {
            let bad = || format!("bad cell {text}");
            let (number, rest) = text.split_once(':').ok_or_else(bad)?;
            let number = u16::from_str_radix(number, 16).map_err(|_| bad())?;
            let mut chars = rest.chars();
            let colours = chars
                .next()
                .and_then(|c| c.to_digit(10))
                .and_then(|c| u8::try_from(c).ok())
                .and_then(ColourSet::new)
                .ok_or_else(bad)?;
            let picture = Picture::new(number, flip(chars.as_str())?);
            Ok(Self(picture.in_colours(colours)))
        }
    }

    /// A HUD row: its 32 cells, left to right, separated by spaces.
    #[derive(Deserialize, Clone, Copy)]
    #[serde(try_from = "String")]
    pub struct HudRowIn(pub [Tile; 32]);

    impl TryFrom<String> for HudRowIn {
        type Error = String;
        fn try_from(text: String) -> Result<Self, String> {
            let cells = text
                .split_whitespace()
                .map(|cell| TileIn::try_from(cell.to_owned()).map(|tile| tile.0))
                .collect::<Result<Vec<_>, _>>()?;
            let count = cells.len();
            cells
                .try_into()
                .map(Self)
                .map_err(|_| format!("a HUD row of {count} cells"))
        }
    }

    /// An event, `Switched:<item>` or `Click`.
    #[derive(Deserialize, Clone, Copy)]
    #[serde(try_from = "String")]
    pub struct EventIn(pub Event);

    impl TryFrom<String> for EventIn {
        type Error = String;
        fn try_from(text: String) -> Result<Self, String> {
            if text == "Click" {
                return Ok(Self(Event::Click));
            }
            let item = text
                .strip_prefix("Switched:")
                .ok_or_else(|| format!("unknown event {text}"))?;
            let item: ItemName = toml::Value::String(item.to_owned())
                .try_into()
                .map_err(|_| format!("unknown item in {text}"))?;
            Ok(Self(Event::Switched(item.item())))
        }
    }

    impl ItemName {
        /// The item it names, or none.
        pub const fn item(self) -> Option<SelectableItem> {
            use clean_model_hud::Ammo;
            match self {
                Self::None => None,
                Self::Missiles => Some(SelectableItem::Ammo(Ammo::Missiles)),
                Self::SuperMissiles => Some(SelectableItem::Ammo(Ammo::SuperMissiles)),
                Self::PowerBombs => Some(SelectableItem::Ammo(Ammo::PowerBombs)),
                Self::GrappleBeam => Some(SelectableItem::GrappleBeam),
                Self::XRayScope => Some(SelectableItem::XRayScope),
            }
        }
    }

    /// The flip letters.
    fn flip(letters: &str) -> Result<Flip, String> {
        match letters {
            "" => Ok(Flip::None),
            "h" => Ok(Flip::Across),
            "v" => Ok(Flip::Down),
            "hv" => Ok(Flip::Both),
            other => Err(format!("bad flips {other}")),
        }
    }
}

#[cfg(test)]
mod convert {
    //! The vector file's shape turned into the model's types, and a vector run.

    use std::collections::BTreeMap;

    use clean_model_hud::{
        Ammo, Ammunition, CancelButton, Controls, Energy, Equipped, Event, FrameCount, FrameInputs,
        HerMap, HudGrid, HudPictures, HudState, Icons, MapCellSet, MapPictures, MiniMapInputs,
        MiniMapPower, Mute, Pair, Picture, Point, Reserve, ReserveMode, Room, RoomBoss, Rules,
        SelectButton, SelectableItem, Selection, Shown, Status, Stock,
    };
    use enum_map::enum_map;

    use super::shape::{
        AfterIn, BeforeIn, BossName, CancelIn, Equipment, IconsIn, InputsIn, ItemName, MapIn,
        MiniMapIn, MuteIn, PictureIn, PicturesIn, RoomIn, SelectIn, ShownIn, Step, VectorFile,
        VectorIn,
    };

    /// The vector file.
    const VECTOR_TEXT: &str = include_str!("../../../../scenarios/vectors/hud.toml");
    /// The HUD's rows (Notation).
    const HUD_ROWS: i32 = 3;
    /// The HUD's columns.
    const HUD_COLUMNS: i32 = 32;
    /// Her map's columns: a beyond picture's column counts on from the last (Notation).
    const MAP_COLUMNS: i32 = 64;
    /// Her map's rows.
    const MAP_ROWS: i32 = 32;
    /// The beyond pictures' columns (spec: `LastColumnsShift`).
    const BEYOND_COLUMNS: i32 = 2;

    /// The file, read.
    pub fn load() -> Result<VectorFile, String> {
        load_text(VECTOR_TEXT)
    }

    /// The file's text with its first `from` replaced by `to`, read.
    pub fn load_changed(from: &str, to: &str) -> Result<VectorFile, String> {
        assert!(VECTOR_TEXT.contains(from), "the file holds no {from}");
        load_text(&VECTOR_TEXT.replacen(from, to, 1))
    }

    /// A vector file's text, read.
    fn load_text(text: &str) -> Result<VectorFile, String> {
        toml::from_str(text).map_err(|error| error.to_string())
    }

    /// The HUD's pictures.
    pub fn pictures(pictures: &PicturesIn) -> HudPictures {
        let IconsIn {
            missiles,
            super_missiles,
            power_bombs,
            grapple_beam,
            x_ray_scope,
        } = &pictures.icons;
        HudPictures {
            blank: pictures.blank.0,
            digits: pictures.digits.map(|digit| digit.0),
            full_tank: pictures.full_tank.0,
            empty_tank: pictures.empty_tank.0,
            reserve_sign: rows(&pictures.reserve_sign),
            icons: Icons {
                missiles: rows(missiles),
                super_missiles: rows(super_missiles),
                power_bombs: rows(power_bombs),
                grapple_beam: rows(grapple_beam),
                x_ray_scope: rows(x_ray_scope),
            },
            map_blank: pictures.map_blank.0,
            diagonal: pictures.diagonal.0,
        }
    }

    /// Rows of pictures, as the model holds them.
    fn rows<const ACROSS: usize, const DOWN: usize>(
        rows: &[[PictureIn; ACROSS]; DOWN],
    ) -> [[Picture; ACROSS]; DOWN] {
        rows.map(|row| row.map(|picture| picture.0))
    }

    impl VectorIn {
        /// Runs the vector: `Ok` with each way the model's after differs from the vector's, or
        /// `Err` when the vector breaks the file's format.
        pub fn run(&self, pictures: &HudPictures) -> Result<Vec<String>, String> {
            let before = state(&self.before, pictures)?;
            let rules = Rules::reference();
            let (actual, events) = match self.step {
                Step::Frame => {
                    let frame = self.frame.ok_or("a Frame step has a frame")?;
                    let map = her_map(self.inputs.mini_map.as_ref(), pictures)?;
                    let inputs = frame_inputs(FrameCount(frame), &self.inputs, &map)?;
                    let outcome = before.clone().frame(&rules, pictures, &inputs);
                    (outcome.state, outcome.events)
                }
                Step::BossEvent => {
                    if self.frame.is_some() {
                        return Err("a BossEvent step has no frame".to_owned());
                    }
                    let (boss, room) = boss_event(&self.inputs)?;
                    let state = before
                        .clone()
                        .boss_encountered(&rules, pictures, boss, room);
                    (state, Vec::new())
                }
            };
            let expected = expected_state(&self.after, &before)?;
            let expected_events: Vec<Event> = self.after.events.iter().map(|e| e.0).collect();
            Ok(differences(&expected, &actual, &expected_events, &events))
        }
    }

    /// A `BossEvent`'s inputs: her room's cell and the boss, and nothing else.
    fn boss_event(inputs: &InputsIn) -> Result<(RoomBoss, Point), String> {
        let InputsIn {
            energy: None,
            reserve: None,
            auto_reserve: None,
            missiles: None,
            super_missiles: None,
            power_bombs: None,
            equipped: None,
            select: None,
            cancel: None,
            can_switch: None,
            shot_cancels: None,
            muted_by: None,
            collected: None,
            mini_map:
                Some(MiniMapIn {
                    runs: None,
                    turned_on: None,
                    mother_brain_falls: None,
                    position: None,
                    room: RoomIn { cell, blocks: None },
                    map: None,
                    boss: Some(boss),
                }),
        } = inputs
        else {
            return Err("a BossEvent's only inputs are mini_map's room cell and boss".to_owned());
        };
        let boss = match boss {
            BossName::Kraid => RoomBoss::Kraid,
            BossName::Draygon => RoomBoss::Draygon,
            BossName::Crocomire => RoomBoss::Crocomire,
            BossName::Phantoon => RoomBoss::Phantoon,
            BossName::RidleyInNorfair => RoomBoss::RidleyInNorfair,
            BossName::RidleyInCeres => RoomBoss::RidleyInCeres,
        };
        Ok((boss, point(*cell)))
    }

    /// A `Frame` step's inputs.
    fn frame_inputs<'map>(
        frame: FrameCount,
        inputs: &InputsIn,
        map: &'map HerMap,
    ) -> Result<FrameInputs<'map>, String> {
        let [now, capacity] = inputs.energy.ok_or("a Frame step has an energy")?;
        let stock = |stock: Option<[u16; 2]>| {
            let [count, capacity] = stock.unwrap_or([0, 0]);
            Stock { count, capacity }
        };
        let ammunition = Ammunition::new(enum_map! {
            Ammo::Missiles => stock(inputs.missiles),
            Ammo::SuperMissiles => stock(inputs.super_missiles),
            Ammo::PowerBombs => stock(inputs.power_bombs),
        })
        .ok_or("ammunition outside the HUD's domain")?;
        let mut equipped = Equipped::default();
        for equipment in inputs.equipped.iter().flatten() {
            match equipment {
                Equipment::GrappleBeam => equipped.grapple_beam = true,
                Equipment::XRayScope => equipped.x_ray_scope = true,
            }
        }
        let mode = if inputs.auto_reserve.unwrap_or(false) {
            ReserveMode::Auto
        } else {
            ReserveMode::Manual
        };
        let (mini_map, mother_brain_falls) = mini_map_inputs(inputs.mini_map.as_ref(), map)?;
        Ok(FrameInputs {
            frame,
            status: Status {
                energy: Energy { now, capacity },
                reserve: Reserve {
                    energy: inputs.reserve.unwrap_or(0),
                    mode,
                },
                ammunition,
                equipped,
            },
            controls: controls(inputs),
            collected: inputs.collected.and_then(ItemName::item),
            mother_brain_falls,
            mini_map,
        })
    }

    /// The buttons.
    fn controls(inputs: &InputsIn) -> Controls {
        Controls {
            select: match inputs.select {
                None => SelectButton::NotPressed,
                Some(SelectIn::Pressed) => SelectButton::Pressed,
            },
            cancel: match inputs.cancel {
                None => CancelButton::NotPressed,
                Some(CancelIn::Pressed) => CancelButton::Pressed,
                Some(CancelIn::Held) => CancelButton::Held,
            },
            can_switch: inputs.can_switch.unwrap_or(true),
            shot_cancels: inputs.shot_cancels.unwrap_or(false),
            muted_by: inputs.muted_by.map(|mute| match mute {
                MuteIn::Grapple => Mute::Grapple,
                MuteIn::SpinJump => Mute::SpinJump,
                MuteIn::TimeStopped => Mute::TimeStopped,
                MuteIn::WallJump => Mute::WallJump,
            }),
        }
    }

    /// The mini-map's inputs and whether Mother Brain falls; with no `mini_map`, it has nothing
    /// to do: it doesn't run.
    fn mini_map_inputs<'map>(
        mini_map: Option<&MiniMapIn>,
        map: &'map HerMap,
    ) -> Result<(MiniMapInputs<'map>, bool), String> {
        let Some(mini_map) = mini_map else {
            let idle = MiniMapInputs {
                runs: false,
                turned_on: false,
                position: Pair { x: 0, y: 0 },
                room: Room {
                    cell: Pair { x: 0, y: 0 },
                    blocks: Pair { x: 0, y: 0 },
                },
                map,
            };
            return Ok((idle, false));
        };
        if mini_map.boss.is_some() {
            return Err("a Frame step names no boss".to_owned());
        }
        let [x, y] = mini_map.position.ok_or("mini_map has a position")?;
        let [across, down] = mini_map.room.blocks.ok_or("room has blocks")?;
        let inputs = MiniMapInputs {
            runs: mini_map.runs.unwrap_or(true),
            turned_on: mini_map.turned_on.unwrap_or(false),
            position: Pair { x, y },
            room: Room {
                cell: point(mini_map.room.cell),
                blocks: Pair { x: across, y: down },
            },
            map,
        };
        Ok((inputs, mini_map.mother_brain_falls.unwrap_or(false)))
    }

    /// Her map: a cell not listed shows the map's blank, as does a beyond picture not listed.
    fn her_map(mini_map: Option<&MiniMapIn>, pictures: &HudPictures) -> Result<HerMap, String> {
        let mut map = HerMap {
            pictures: MapPictures::filled(pictures.map_blank),
            charted: MapCellSet::default(),
            collected: false,
        };
        let Some(MapIn {
            collected,
            pictures: cells,
            charted,
            beyond,
        }) = mini_map.and_then(|mini_map| mini_map.map.as_ref())
        else {
            return match mini_map {
                Some(_) => Err("mini_map has a map".to_owned()),
                None => Ok(map),
            };
        };
        map.collected = *collected;
        lay_pictures(&mut map.pictures, cells, 0..MAP_COLUMNS, 0)?;
        lay_pictures(&mut map.pictures, beyond, 0..BEYOND_COLUMNS, MAP_COLUMNS)?;
        map.charted = cell_set(charted)?;
        Ok(map)
    }

    /// Lays `"column,row" = picture` pictures, each in `columns`, into her map's pictures
    /// `shift` columns along: the beyond pictures' columns 0 and 1 are its columns 64 and 65.
    fn lay_pictures(
        grid: &mut MapPictures,
        pictures: &BTreeMap<String, PictureIn>,
        columns: std::ops::Range<i32>,
        shift: i32,
    ) -> Result<(), String> {
        for (key, picture) in pictures {
            let at = coordinates(key)?;
            if !columns.contains(&at.x) || !(0..MAP_ROWS).contains(&at.y) {
                return Err(format!("no map cell {key}"));
            }
            grid.set(
                Pair {
                    x: at.x.saturating_add(shift),
                    y: at.y,
                },
                picture.0,
            );
        }
        Ok(())
    }

    /// The state from a before; its HUD left out, every cell is the blank in its colours.
    fn state(before: &BeforeIn, pictures: &HudPictures) -> Result<HudState, String> {
        let mut grid = HudGrid::filled(pictures.blank_cell());
        for (y, row) in (0..).zip(before.hud.iter().flatten()) {
            for (x, &tile) in (0..).zip(&row.0) {
                grid.set(Pair { x, y }, tile);
            }
        }
        Ok(HudState {
            grid,
            shown: shown(&before.shown),
            selection: selection(before.selection.item(), before.one_shot)?,
            explored: cell_set(&before.explored)?,
            mini_map: power(before.mini_map_off),
        })
    }

    /// The state an after expects: the before's, with the after's changes.
    fn expected_state(after: &AfterIn, before: &HudState) -> Result<HudState, String> {
        let mut grid = before.grid;
        for (key, tile) in &after.changed {
            let Pair { x: row, y: column } = coordinates(key)?;
            let at = Pair { x: column, y: row };
            if grid.get(at).is_none() {
                return Err(format!("no HUD cell {key}"));
            }
            grid.set(at, tile.0);
        }
        let item = after
            .selection
            .map_or_else(|| before.selection.item(), ItemName::item);
        let one_shot = after
            .one_shot
            .unwrap_or_else(|| before.selection.one_shot().is_some());
        Ok(HudState {
            grid,
            shown: after.shown.as_ref().map_or(before.shown, shown),
            selection: selection(item, one_shot)?,
            explored: after
                .explored
                .as_ref()
                .map_or(Ok(before.explored), |cells| cell_set(cells))?,
            mini_map: after.mini_map_off.map_or(before.mini_map, power),
        })
    }

    /// The selection from the selected item and whether it is the one-shot item.
    fn selection(item: Option<SelectableItem>, one_shot: bool) -> Result<Selection, String> {
        match (item, one_shot) {
            (None, false) => Ok(Selection::None),
            (None, true) => Err("one_shot with none selected".to_owned()),
            (Some(item), false) => Ok(Selection::Item(item)),
            (Some(item), true) => Ok(Selection::OneShot(item)),
        }
    }

    /// Whether the mini-map is off.
    const fn power(off: bool) -> MiniMapPower {
        if off {
            MiniMapPower::Off
        } else {
            MiniMapPower::On
        }
    }

    /// The values last shown.
    fn shown(shown: &ShownIn) -> Shown {
        Shown {
            energy: shown.energy,
            ammo: enum_map! {
                Ammo::Missiles => shown.missiles,
                Ammo::SuperMissiles => shown.super_missiles,
                Ammo::PowerBombs => shown.power_bombs,
            },
            highlighted: shown.selection.item(),
        }
    }

    /// A set of `[column, row]` cells, each on her map.
    fn cell_set(cells: &[[i32; 2]]) -> Result<MapCellSet, String> {
        let mut set = MapCellSet::default();
        for &cell in cells {
            let at = point(cell);
            if set.get(at).is_none() {
                return Err(format!("no map cell {cell:?}"));
            }
            set.insert(at);
        }
        Ok(set)
    }

    /// A `[x, y]` pair as a place.
    const fn point([x, y]: [i32; 2]) -> Point {
        Pair { x, y }
    }

    /// A `"a,b"` key, as `x` and `y` in the order written.
    fn coordinates(key: &str) -> Result<Point, String> {
        let bad = || format!("bad key {key}");
        let (a, b) = key.split_once(',').ok_or_else(bad)?;
        Ok(Pair {
            x: a.trim().parse().map_err(|_| bad())?,
            y: b.trim().parse().map_err(|_| bad())?,
        })
    }

    /// Every way the model's after differs from the vector's.
    fn differences(
        expected: &HudState,
        actual: &HudState,
        expected_events: &[Event],
        actual_events: &[Event],
    ) -> Vec<String> {
        let mut problems = grid_differences(&expected.grid, &actual.grid);
        if expected.shown != actual.shown {
            problems.push(format!(
                "shown: expected {:?}, got {:?}",
                expected.shown, actual.shown
            ));
        }
        if expected.selection != actual.selection {
            problems.push(format!(
                "selection: expected {:?}, got {:?}",
                expected.selection, actual.selection
            ));
        }
        if expected.mini_map != actual.mini_map {
            problems.push(format!(
                "mini-map: expected {:?}, got {:?}",
                expected.mini_map, actual.mini_map
            ));
        }
        if expected.explored != actual.explored {
            problems.push(format!(
                "explored: expected {:?}, got {:?}",
                cell_list(&expected.explored),
                cell_list(&actual.explored)
            ));
        }
        if expected_events != actual_events {
            problems.push(format!(
                "events: expected {expected_events:?}, got {actual_events:?}"
            ));
        }
        problems
    }

    /// The cells that differ, each named "row,column" as the vector file names them.
    fn grid_differences(expected: &HudGrid, actual: &HudGrid) -> Vec<String> {
        let mut problems = Vec::new();
        for y in 0..HUD_ROWS {
            for x in 0..HUD_COLUMNS {
                let at = Pair { x, y };
                if expected.get(at) != actual.get(at) {
                    problems.push(format!(
                        "cell {y},{x}: expected {:?}, got {:?}",
                        expected.get(at),
                        actual.get(at)
                    ));
                }
            }
        }
        problems
    }

    /// A set's cells as `[column, row]` pairs.
    fn cell_list(cells: &MapCellSet) -> Vec<[i32; 2]> {
        (0..MAP_ROWS)
            .flat_map(|y| (0..MAP_COLUMNS).map(move |x| [x, y]))
            .filter(|&cell| cells.contains(point(cell)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use clean_model_hud::{BehaviourId, CATALOGUE, Checked, Event};

    use crate::convert::{load, load_changed, pictures};
    use crate::shape::VectorIn;

    /// Changes to the file that each add a key or a name its header doesn't define, one in each
    /// kind of table and each kind of name.
    const UNKNOWN: [(&str, &str); 16] = [
        ("[pictures]", "speed = 1\n[pictures]"),
        ("diagonal = \"0E4\"", "diagonal = \"0E4\"\nspeed = \"000\""),
        ("[pictures.icons]", "[pictures.icons]\nSpeedBooster = []"),
        ("id = \"C-R1\"", "id = \"C-R1\"\nspeed = 1"),
        (
            "shown = { energy = 333,",
            "shown = { speed = 1, energy = 333,",
        ),
        ("can_switch = true", "can_switch = true\nspeed = 1"),
        ("runs = true", "runs = true\nspeed = 1"),
        ("blocks = [128, 32] }", "blocks = [128, 32], speed = 1 }"),
        ("collected = false", "collected = false\nspeed = 1"),
        (
            "changed = { \"2,7\" = \"055:3\" }",
            "changed = { \"2,7\" = \"055:3\" }\nspeed = 1",
        ),
        ("boss = \"Kraid\"", "boss = \"Mario\""),
        ("muted_by = \"SpinJump\"", "muted_by = \"Somersault\""),
        ("source = \"constructed\"", "source = \"recorded\""),
        ("step = \"BossEvent\"", "step = \"Pause\""),
        (
            "\"Switched:Missiles\", \"Click\"",
            "\"Switched:Missiles\", \"Clack\"",
        ),
        ("selection = \"None\"", "selection = \"Nothing\""),
    ];

    #[test]
    fn an_unknown_key_or_name_fails_the_file() {
        for (from, to) in UNKNOWN {
            assert!(load_changed(from, to).is_err(), "{to} was read");
        }
    }

    /// The entry's name in the spec: its variant's.
    fn name(id: BehaviourId) -> String {
        format!("{id:?}")
    }

    /// The entries a vector exercises: those it lists, and the outputs whose events it expects
    /// (spec: `Switching`, `SwitchClick`).
    fn exercised(vector: &VectorIn) -> impl Iterator<Item = String> + '_ {
        let outputs = vector.after.events.iter().map(|event| match event.0 {
            Event::Switched(_) => "Switching".to_owned(),
            Event::Click => "SwitchClick".to_owned(),
        });
        vector.entries.iter().cloned().chain(outputs)
    }

    #[test]
    fn every_vector_passes() {
        let file = load().unwrap();
        let pictures = pictures(&file.pictures);
        assert!(!file.vector.is_empty(), "the vector file holds no vectors");
        let failures: Vec<String> = file
            .vector
            .iter()
            .filter_map(|vector| match vector.run(&pictures) {
                Ok(problems) if problems.is_empty() => None,
                Ok(problems) => Some(format!("{}: {}", vector.id, problems.join("; "))),
                Err(error) => Some(format!("{}: unreadable: {error}", vector.id)),
            })
            .collect();
        assert!(
            failures.is_empty(),
            "{} of {} vectors fail:\n{}",
            failures.len(),
            file.vector.len(),
            failures.join("\n")
        );
    }

    #[test]
    fn vector_ids_are_unique() {
        let file = load().unwrap();
        for (at, vector) in file.vector.iter().enumerate() {
            let again = file
                .vector
                .iter()
                .skip(at)
                .skip(1)
                .any(|v| v.id == vector.id);
            assert!(!again, "vector {} appears twice", vector.id);
        }
    }

    #[test]
    fn every_listed_entry_is_in_the_catalogue() {
        for vector in load().unwrap().vector {
            for entry in &vector.entries {
                assert!(
                    CATALOGUE.iter().any(|b| name(b.id) == *entry),
                    "vector {} lists {entry}, which the catalogue lacks",
                    vector.id
                );
            }
        }
    }

    #[test]
    fn every_entry_checked_by_vectors_has_one() {
        let file = load().unwrap();
        let checked = CATALOGUE.iter().filter(|b| b.checked == Checked::ByVectors);
        for behaviour in checked {
            let name = name(behaviour.id);
            assert!(
                file.vector.iter().any(|v| exercised(v).any(|e| e == name)),
                "no vector exercises {name}"
            );
        }
    }
}
