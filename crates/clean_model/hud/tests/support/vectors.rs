//! The HUD's vectors: each read from the vector file, run through the model and compared with
//! its after. Any key the file's header doesn't define fails the vector.

use clean_model_hud::{
    Ammunition, AreaMap, BeyondCell, Boss, CancelButton, ColourSet, Controls, Energy, Equipped,
    Event, Flip, FrameCount, FrameInputs, GridPos, HudGrid, HudState, MapCell, MapCellSet,
    MapPicture, MiniMapInputs, Mute, Pair, Picture, Reserve, ReserveMode, Room, Rules,
    SelectButton, SelectableItem, Selection, Shown, Status, Stock, Tile,
};

use super::toml::{Table, Value, parse};

/// The vector file.
const VECTOR_FILE: &str = include_str!("../../../../../scenarios/vectors/hud.toml");

/// The HUD's rows and columns (Notation).
const HUD_ROWS: u8 = 3;
/// The HUD's columns.
const HUD_COLUMNS: u8 = 32;

/// A vector's keys.
const VECTOR_KEYS: [&str; 8] = [
    "id", "source", "entries", "step", "frame", "before", "inputs", "after",
];
/// The keys of a before.
const BEFORE_KEYS: [&str; 5] = ["hud", "shown", "selection", "one_shot", "explored"];
/// The keys of an after.
const AFTER_KEYS: [&str; 6] = [
    "changed",
    "shown",
    "selection",
    "one_shot",
    "explored",
    "events",
];
/// The keys of `shown`.
const SHOWN_KEYS: [&str; 5] = [
    "energy",
    "missiles",
    "super_missiles",
    "power_bombs",
    "selection",
];
/// The keys of a frame's inputs.
const INPUT_KEYS: [&str; 14] = [
    "energy",
    "reserve",
    "auto_reserve",
    "missiles",
    "super_missiles",
    "power_bombs",
    "equipped",
    "select",
    "cancel",
    "can_switch",
    "shot_cancels",
    "muted_by",
    "collected",
    "mini_map",
];
/// The keys of `mini_map`.
const MINI_MAP_KEYS: [&str; 6] = ["runs", "off", "position", "room", "map", "boss_blanks"];
/// The keys of `room`.
const ROOM_KEYS: [&str; 2] = ["cell", "blocks"];
/// The keys of `map`.
const MAP_KEYS: [&str; 4] = ["collected", "pictures", "charted", "beyond"];

/// One vector as the file gives it.
pub struct Vector {
    /// Its id.
    pub id: String,
    /// The spec's entries it lists.
    pub entries: Vec<String>,
    /// The events its after expects.
    pub events: Vec<String>,
    /// The whole vector.
    table: Table,
}

/// Every vector in the file.
pub fn load() -> Result<Vec<Vector>, String> {
    let root = parse(VECTOR_FILE)?;
    known(&root, &["vector"], "the file")?;
    let Some(Value::Array(items)) = root.get("vector") else {
        return Err("no [[vector]] tables".to_owned());
    };
    items.iter().map(vector).collect()
}

/// A vector's id, entries and events, read up front.
fn vector(value: &Value) -> Result<Vector, String> {
    let Value::Table(table) = value else {
        return Err("a vector is not a table".to_owned());
    };
    let id = string(required(table, "id")?)?.to_owned();
    let entries = strings(required(table, "entries")?)?;
    let events = match sub_table(table, "after")?.and_then(|after| after.get("events")) {
        Some(events) => strings(events)?,
        None => Vec::new(),
    };
    Ok(Vector {
        id,
        entries,
        events,
        table: table.clone(),
    })
}

impl Vector {
    /// Runs the vector: `Ok` with each way the model's after differs from the vector's, or `Err`
    /// when the vector can't be read.
    pub fn run(&self) -> Result<Vec<String>, String> {
        let table = &self.table;
        known(table, &VECTOR_KEYS, "the vector")?;
        let empty = Table::default();
        let before = state(sub_table(table, "before")?.unwrap_or(&empty))?;
        let step = optional(table, "step").map_or(Ok("Frame"), string)?;
        let (actual, events) = match step {
            "Frame" => frame_step(table, before.clone())?,
            "BossEvent" => (boss_step(table, before.clone())?, Vec::new()),
            other => return Err(format!("unknown step {other}")),
        };
        let after = sub_table(table, "after")?.ok_or("no after")?;
        known(after, &AFTER_KEYS, "after")?;
        let expected = expected_state(after, &before)?;
        let events: Vec<String> = events.into_iter().map(event_name).collect();
        Ok(differences(&expected, &actual, &self.events, &events))
    }
}

/// The name a vector gives an event.
fn event_name(event: Event) -> String {
    match event {
        Event::Switched(item) => format!("Switched:{}", item_name(item)),
        Event::Click => "Click".to_owned(),
    }
}

/// The name a vector gives an item, or none.
fn item_name(item: Option<SelectableItem>) -> String {
    item.map_or_else(|| "None".to_owned(), |item| format!("{item:?}"))
}

/// Runs a "Frame" step.
fn frame_step(table: &Table, before: HudState) -> Result<(HudState, Vec<Event>), String> {
    let frame = FrameCount(int(required(table, "frame")?)?);
    let inputs = sub_table(table, "inputs")?.ok_or("no inputs")?;
    known(inputs, &INPUT_KEYS, "inputs")?;
    let read = MiniMapRead::from(sub_table(inputs, "mini_map")?)?;
    let frame_inputs = FrameInputs {
        frame,
        status: status(inputs)?,
        controls: controls(inputs)?,
        collected: optional(inputs, "collected").map_or(Ok(None), item)?,
        boss: read.boss,
        mini_map: MiniMapInputs {
            runs: read.runs,
            off: read.off,
            position: read.position,
            room: read.room,
            map: &read.map,
        },
    };
    let outcome = before.frame(&Rules::reference(), &frame_inputs);
    Ok((outcome.state, outcome.events))
}

/// Runs a `BossEvent` step: its only inputs are her room's cell and the boss's screens.
fn boss_step(table: &Table, before: HudState) -> Result<HudState, String> {
    if table.get("frame").is_some() {
        return Err("a BossEvent step has no frame".to_owned());
    }
    let inputs = sub_table(table, "inputs")?.ok_or("no inputs")?;
    known(inputs, &["mini_map"], "a BossEvent's inputs")?;
    let mini_map = sub_table(inputs, "mini_map")?.ok_or("no mini_map")?;
    known(mini_map, &["room", "boss_blanks"], "a BossEvent's mini_map")?;
    let room = sub_table(mini_map, "room")?.ok_or("no room")?;
    known(room, &["cell"], "a BossEvent's room")?;
    let cell = map_cell(required(room, "cell")?)?;
    let boss = boss(required(mini_map, "boss_blanks")?)?;
    Ok(before.boss_turns_off_mini_map(&Rules::reference(), boss, cell))
}

/// The mini-map's inputs as a vector gives them.
struct MiniMapRead {
    runs: bool,
    off: bool,
    position: Pair<u16>,
    room: Room,
    map: AreaMap,
    boss: Option<Boss>,
}

impl MiniMapRead {
    /// Reads `mini_map`; absent, the mini-map has nothing to do: it doesn't run.
    fn from(table: Option<&Table>) -> Result<Self, String> {
        let Some(table) = table else {
            return Ok(Self {
                runs: false,
                off: false,
                position: Pair { x: 0, y: 0 },
                room: Room {
                    cell: MapCell::new(0, 0).ok_or("no cell 0, 0")?,
                    blocks: Pair { x: 0, y: 0 },
                },
                map: AreaMap::new(),
                boss: None,
            });
        };
        known(table, &MINI_MAP_KEYS, "mini_map")?;
        let (x, y) = pair(required(table, "position")?)?;
        Ok(Self {
            runs: boolean(table, "runs", true)?,
            off: boolean(table, "off", false)?,
            position: Pair { x, y },
            room: room(sub_table(table, "room")?.ok_or("no room")?)?,
            map: area_map(sub_table(table, "map")?.ok_or("no map")?)?,
            boss: optional(table, "boss_blanks").map(boss).transpose()?,
        })
    }
}

/// Reads `room`.
fn room(table: &Table) -> Result<Room, String> {
    known(table, &ROOM_KEYS, "room")?;
    let (x, y) = pair(required(table, "blocks")?)?;
    Ok(Room {
        cell: map_cell(required(table, "cell")?)?,
        blocks: Pair { x, y },
    })
}

/// Reads `map`.
fn area_map(table: &Table) -> Result<AreaMap, String> {
    known(table, &MAP_KEYS, "map")?;
    let mut map = AreaMap::new();
    map.set_collected(boolean(table, "collected", false)?);
    for (key, value) in sub_table(table, "pictures")?.map_or(&[][..], |t| &t.0) {
        let (column, row) = coordinates(key)?;
        let cell = MapCell::new(column, row).ok_or_else(|| format!("no map cell {key}"))?;
        map.set_picture(cell, map_picture(string(value)?)?);
    }
    if let Some(charted) = optional(table, "charted") {
        for value in array(charted)? {
            map.chart(map_cell(value)?);
        }
    }
    for (key, value) in sub_table(table, "beyond")?.map_or(&[][..], |t| &t.0) {
        let (column, row) = coordinates(key)?;
        let cell = BeyondCell::new(column, row).ok_or_else(|| format!("no beyond cell {key}"))?;
        map.set_beyond(cell, map_picture(string(value)?)?);
    }
    Ok(map)
}

/// The boss whose screens the reference rules give as `screens`.
fn boss(screens: &Value) -> Result<Boss, String> {
    let mut wanted = array(screens)?
        .iter()
        .map(pair::<u8>)
        .collect::<Result<Vec<_>, _>>()?;
    wanted.sort_unstable();
    Rules::reference()
        .boss_screens
        .iter()
        .find(|row| {
            let mut screens: Vec<(u8, u8)> =
                row.screens.iter().map(|s| (s.across, s.down)).collect();
            screens.sort_unstable();
            screens == wanted
        })
        .map(|row| row.boss)
        .ok_or_else(|| format!("no boss names the screens {wanted:?}"))
}

/// Reads what the status bar shows from the inputs.
fn status(inputs: &Table) -> Result<Status, String> {
    let (now, capacity) = pair(required(inputs, "energy")?)?;
    let auto = boolean(inputs, "auto_reserve", false)?;
    let mut equipped = Equipped::default();
    if let Some(items) = optional(inputs, "equipped") {
        for name in strings(items)? {
            match name.as_str() {
                "GrappleBeam" => equipped.grapple_beam = true,
                "XRayScope" => equipped.x_ray_scope = true,
                other => return Err(format!("unknown equipment {other}")),
            }
        }
    }
    let ammunition = Ammunition::new(
        stock(inputs, "missiles")?,
        stock(inputs, "super_missiles")?,
        stock(inputs, "power_bombs")?,
    )
    .ok_or("ammunition outside the HUD's domain")?;
    Ok(Status {
        energy: Energy { now, capacity },
        reserve: Reserve {
            energy: optional(inputs, "reserve").map_or(Ok(0), int)?,
            mode: if auto {
                ReserveMode::Auto
            } else {
                ReserveMode::Manual
            },
        },
        ammunition,
        equipped,
    })
}

/// Reads one ammunition's `[count, capacity]`, by default `[0, 0]`.
fn stock(inputs: &Table, key: &str) -> Result<Stock, String> {
    let (count, capacity) = optional(inputs, key).map_or(Ok((0, 0)), pair)?;
    Ok(Stock { count, capacity })
}

/// Reads the buttons from the inputs.
fn controls(inputs: &Table) -> Result<Controls, String> {
    let select = match optional(inputs, "select").map(string).transpose()? {
        None => SelectButton::Up,
        Some("Pressed") => SelectButton::Pressed,
        Some(other) => return Err(format!("unknown select {other}")),
    };
    let cancel = match optional(inputs, "cancel").map(string).transpose()? {
        None => CancelButton::Up,
        Some("Pressed") => CancelButton::Pressed,
        Some("Held") => CancelButton::Held,
        Some(other) => return Err(format!("unknown cancel {other}")),
    };
    let muted_by = match optional(inputs, "muted_by").map(string).transpose()? {
        None => None,
        Some("SpinJump") => Some(Mute::SpinJump),
        Some("WallJump") => Some(Mute::WallJump),
        Some("Grapple") => Some(Mute::Grapple),
        Some("TimeStopped") => Some(Mute::TimeStopped),
        Some(other) => return Err(format!("unknown muted_by {other}")),
    };
    Ok(Controls {
        select,
        cancel,
        can_switch: boolean(inputs, "can_switch", true)?,
        shot_cancels: boolean(inputs, "shot_cancels", false)?,
        muted_by,
    })
}

/// The HUD's state from a before.
fn state(before: &Table) -> Result<HudState, String> {
    known(before, &BEFORE_KEYS, "before")?;
    let grid = match optional(before, "hud") {
        Some(rows) => hud_grid(rows)?,
        None => HudGrid::filled(tile("00F:3")?),
    };
    Ok(HudState {
        grid,
        shown: sub_table(before, "shown")?.map_or_else(|| Ok(Shown::default()), shown)?,
        selection: Selection {
            selected: optional(before, "selection").map_or(Ok(None), item)?,
            one_shot: optional(before, "one_shot").map_or(Ok(None), item)?,
        },
        explored: optional(before, "explored").map_or(Ok(MapCellSet::EMPTY), cells)?,
    })
}

/// The state an after expects: the before's, with the after's changes.
fn expected_state(after: &Table, before: &HudState) -> Result<HudState, String> {
    let mut grid = before.grid;
    for (key, value) in sub_table(after, "changed")?.map_or(&[][..], |t| &t.0) {
        let (row, column) = coordinates(key)?;
        let pos = GridPos::new(row, column).ok_or_else(|| format!("no HUD tile {key}"))?;
        grid.set(pos, tile(string(value)?)?);
    }
    Ok(HudState {
        grid,
        shown: sub_table(after, "shown")?.map_or(Ok(before.shown), shown)?,
        selection: Selection {
            selected: optional(after, "selection").map_or(Ok(before.selection.selected), item)?,
            one_shot: optional(after, "one_shot").map_or(Ok(before.selection.one_shot), item)?,
        },
        explored: optional(after, "explored").map_or(Ok(before.explored), cells)?,
    })
}

/// Reads `shown`: a value left out is 0 or none.
fn shown(table: &Table) -> Result<Shown, String> {
    known(table, &SHOWN_KEYS, "shown")?;
    let count = |key| optional(table, key).map_or(Ok(0), int);
    Ok(Shown {
        energy: count("energy")?,
        missiles: count("missiles")?,
        super_missiles: count("super_missiles")?,
        power_bombs: count("power_bombs")?,
        selection: optional(table, "selection").map_or(Ok(None), item)?,
    })
}

/// Every way the model's after differs from the vector's.
fn differences(
    expected: &HudState,
    actual: &HudState,
    expected_events: &[String],
    actual_events: &[String],
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

/// The tiles that differ.
fn grid_differences(expected: &HudGrid, actual: &HudGrid) -> Vec<String> {
    (0..HUD_ROWS)
        .flat_map(|row| (0..HUD_COLUMNS).filter_map(move |column| GridPos::new(row, column)))
        .filter(|&pos| expected.tile(pos) != actual.tile(pos))
        .map(|pos| {
            format!(
                "tile {},{}: expected {}, got {}",
                pos.row(),
                pos.column(),
                tile_name(expected.tile(pos)),
                tile_name(actual.tile(pos))
            )
        })
        .collect()
}

/// A tile as a vector writes it.
fn tile_name(tile: Tile) -> String {
    let h = if tile.flip.horizontal() { "h" } else { "" };
    let v = if tile.flip.vertical() { "v" } else { "" };
    format!(
        "{:03X}:{}{h}{v}",
        tile.picture.number(),
        tile.colours.number()
    )
}

/// A set's cells as `[column, row]` pairs.
fn cell_list(cells: &MapCellSet) -> Vec<[u8; 2]> {
    cells
        .iter()
        .map(|cell| [cell.column(), cell.row()])
        .collect()
}

/// Reads the HUD's three rows of 32 tiles.
fn hud_grid(rows: &Value) -> Result<HudGrid, String> {
    let rows = strings(rows)?;
    if rows.len() != usize::from(HUD_ROWS) {
        return Err(format!("hud has {} rows", rows.len()));
    }
    let mut grid = HudGrid::filled(tile("00F:3")?);
    for (row, text) in (0..HUD_ROWS).zip(&rows) {
        let tiles: Vec<&str> = text.split_whitespace().collect();
        if tiles.len() != usize::from(HUD_COLUMNS) {
            return Err(format!("hud row {row} has {} tiles", tiles.len()));
        }
        for (column, name) in (0..HUD_COLUMNS).zip(tiles) {
            let pos = GridPos::new(row, column).ok_or("no tile")?;
            grid.set(pos, tile(name)?);
        }
    }
    Ok(grid)
}

/// Reads a tile, `PPP:c` then `h` and `v` for its flips.
fn tile(text: &str) -> Result<Tile, String> {
    let bad = || format!("bad tile {text}");
    let (picture, rest) = text.split_once(':').ok_or_else(bad)?;
    let number = u16::from_str_radix(picture, 16).map_err(|_| bad())?;
    let mut chars = rest.chars();
    let colours = chars
        .next()
        .and_then(|c| c.to_digit(10))
        .and_then(|c| u8::try_from(c).ok())
        .and_then(ColourSet::new)
        .ok_or_else(bad)?;
    Ok(Tile::new(
        Picture::new(number),
        colours,
        flip(chars.as_str())?,
    ))
}

/// Reads a map picture, `PPP` then `h` and `v` for its flips.
fn map_picture(text: &str) -> Result<MapPicture, String> {
    let split = text
        .find(|c: char| !c.is_ascii_hexdigit())
        .unwrap_or(text.len());
    let (picture, flips) = text.split_at(split);
    let number = u16::from_str_radix(picture, 16).map_err(|_| format!("bad picture {text}"))?;
    Ok(MapPicture {
        picture: Picture::new(number),
        flip: flip(flips)?,
    })
}

/// Reads the flip letters.
fn flip(letters: &str) -> Result<Flip, String> {
    match letters {
        "" => Ok(Flip::None),
        "h" => Ok(Flip::Horizontal),
        "v" => Ok(Flip::Vertical),
        "hv" => Ok(Flip::Both),
        other => Err(format!("bad flips {other}")),
    }
}

/// Reads an item, or `"None"`.
fn item(value: &Value) -> Result<Option<SelectableItem>, String> {
    match string(value)? {
        "None" => Ok(None),
        "Missiles" => Ok(Some(SelectableItem::Missiles)),
        "SuperMissiles" => Ok(Some(SelectableItem::SuperMissiles)),
        "PowerBombs" => Ok(Some(SelectableItem::PowerBombs)),
        "GrappleBeam" => Ok(Some(SelectableItem::GrappleBeam)),
        "XRayScope" => Ok(Some(SelectableItem::XRayScope)),
        other => Err(format!("unknown item {other}")),
    }
}

/// Reads a set of `[column, row]` cells.
fn cells(value: &Value) -> Result<MapCellSet, String> {
    let mut set = MapCellSet::EMPTY;
    for cell in array(value)? {
        set.insert(map_cell(cell)?);
    }
    Ok(set)
}

/// Reads a `[column, row]` cell of the map.
fn map_cell(value: &Value) -> Result<MapCell, String> {
    let (column, row) = pair(value)?;
    MapCell::new(column, row).ok_or_else(|| format!("no map cell {column}, {row}"))
}

/// Reads a `"a,b"` key.
fn coordinates(key: &str) -> Result<(u8, u8), String> {
    let bad = || format!("bad key {key}");
    let (a, b) = key.split_once(',').ok_or_else(bad)?;
    let a = a.trim().parse().map_err(|_| bad())?;
    let b = b.trim().parse().map_err(|_| bad())?;
    Ok((a, b))
}

/// Fails on a key not in `keys`.
fn known(table: &Table, keys: &[&str], context: &str) -> Result<(), String> {
    match table
        .0
        .iter()
        .find(|(key, _)| !keys.contains(&key.as_str()))
    {
        Some((key, _)) => Err(format!("unknown key {key} in {context}")),
        None => Ok(()),
    }
}

/// The value at `key`, which must be there.
fn required<'t>(table: &'t Table, key: &str) -> Result<&'t Value, String> {
    table.get(key).ok_or_else(|| format!("missing {key}"))
}

/// The value at `key`, if there.
fn optional<'t>(table: &'t Table, key: &str) -> Option<&'t Value> {
    table.get(key)
}

/// The table at `key`, if there.
fn sub_table<'t>(table: &'t Table, key: &str) -> Result<Option<&'t Table>, String> {
    match table.get(key) {
        None => Ok(None),
        Some(Value::Table(inner)) => Ok(Some(inner)),
        Some(_) => Err(format!("{key} is not a table")),
    }
}

/// The boolean at `key`, or `default`.
fn boolean(table: &Table, key: &str, default: bool) -> Result<bool, String> {
    match table.get(key) {
        None => Ok(default),
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(format!("{key} is not a boolean")),
    }
}

/// Reads a string.
fn string(value: &Value) -> Result<&str, String> {
    if let Value::Str(text) = value {
        Ok(text)
    } else {
        Err(format!("expected a string, found {value:?}"))
    }
}

/// Reads an array of strings.
fn strings(value: &Value) -> Result<Vec<String>, String> {
    array(value)?
        .iter()
        .map(|v| string(v).map(str::to_owned))
        .collect()
}

/// Reads an array.
fn array(value: &Value) -> Result<&[Value], String> {
    if let Value::Array(items) = value {
        Ok(items)
    } else {
        Err(format!("expected an array, found {value:?}"))
    }
}

/// Reads an integer into the type it fills.
fn int<T: TryFrom<i64>>(value: &Value) -> Result<T, String> {
    if let Value::Int(number) = value {
        T::try_from(*number).map_err(|_| format!("{number} is out of range"))
    } else {
        Err(format!("expected an integer, found {value:?}"))
    }
}

/// Reads a two-integer array.
fn pair<T: TryFrom<i64>>(value: &Value) -> Result<(T, T), String> {
    match array(value)? {
        [a, b] => Ok((int(a)?, int(b)?)),
        other => Err(format!("expected two integers, found {other:?}")),
    }
}
