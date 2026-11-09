//! The HUD as a harness domain: the HUD model (`clean_model_hud`) against the oracle.
//!
//! It is checked frame by frame by the decode notes' harness note (`docs/behaviours/hud.decode.md`,
//! "The harness"). The HUD is downstream of everything it shows and nothing of the same frame reads
//! it back, so it is checked on its own: its State (the HUD's cells, the values last shown, the
//! selection and the one-shot item, the explored cells) and the frame count and pad are decoded
//! from record N, every other input from record N+1 at the HUD's turn, and the model's output is
//! compared with record N+1: the whole tilemap, not values, since a stale cell is behaviour.
//!
//! The records carry none of the instruction hits the notes name, so two interim masks stand in,
//! each keyed as near its writer as the records allow and reported as masked, never passed:
//! - her position at the mini-map's turn: the pose change's placement (`$91:EB88`) runs after the
//!   mini-map read her and can move her across a screen, so the window and the explored cells are
//!   masked where her pose changed;
//! - the shot's cancel (the weapons' stores `$90:BF25`, `$90:BF41`, `$90:C091`, `$90:C0A6`): the
//!   selection and the one-shot item are masked where a count fell, and the status bar is
//!   compared with the cancel false.
//!
//! The pickups' icons (`collected`) and a boss turning the mini-map off in a frame (Mother Brain's
//! first form) have no hit either; no cached row reaches them, so both are none. The click isn't
//! compared until the sound domain's request stream exists. Nothing answers from record N+1's HUD,
//! so no field is unverified; the null model is the HUD left as record N has it.

use core::fmt;

use clean_model_hud::{
    Ammo, Ammunition, AreaMap, BehaviourId, CATALOGUE, CancelButton, Controls, Energy, Equipped,
    Event, FrameCount, FrameInputs, GridPos, HudState, MapCell, MapCellSet, MiniMapInputs, Mute,
    Reserve, ReserveMode, Room, Rules, SelectButton, SelectableItem, Selection, Shown, Status,
    Stock, Tile,
};
use clean_types::Pair;
use data::map::{MAP_BIT_BYTES, MapCells};
use data::room::Area;
use game::hud_world::{self, AreaMaps};
use game::state::video::TilemapEntry;
use oracle_record::FrameRecord;

use super::{Domain, Field, FieldSet, Interval, Mask, Uncompared};
use crate::bundle::ContentError;
use crate::decode::DecodeError;
use crate::domain_field;
use crate::source::{self, WramWord};

/// The game state the main loop runs gameplay in.
const MAIN_GAMEPLAY: u16 = 8;

/// A work RAM word the harness reads that `source` doesn't name.
const fn wram(name: &'static str, offset: u16) -> WramWord {
    WramWord { name, offset }
}

/// Bank $90 log: the item select and item cancel bindings.
const ITEM_SELECT_BINDING: WramWord = wram("BINDING_ITEM_SELECT", 0x09BA);
const ITEM_CANCEL_BINDING: WramWord = wram("BINDING_ITEM_CANCEL", 0x09B8);
/// Bank $90 log: Samus's current and new state handlers.
const CURRENT_STATE_HANDLER: WramWord = wram("FRAME_HANDLER_ALFA", 0x0A42);
const NEW_STATE_HANDLER: WramWord = wram("FRAME_HANDLER_BETA", 0x0A44);
/// Bank $90 log: the grapple beam's function, `$C4F0` while it is put away.
const GRAPPLE_FUNCTION: WramWord = wram("GRAPPLE_BEAM_FUNCTION", 0x0D32);
/// Bank $90 log: time frozen (the x-ray scope, the reserve tanks' refill).
const TIME_IS_FROZEN: WramWord = wram("TIME_IS_FROZEN", 0x0A78);

/// The normal current state handler, under which she can switch (`$90:E695`).
const NORMAL_CURRENT: u16 = 0xE695;
/// The poses facing the screen, in which she can't switch (`$90:DCE0`).
const FACING_FORWARD: [u16; 2] = [0x00, 0x9B];
/// The grapple beam's function while it is put away.
const GRAPPLE_INACTIVE: u16 = 0xC4F0;
/// The movement types that mute the click: spin jumping, wall jumping.
const SPIN_JUMPING: u8 = 0x03;
const WALL_JUMPING: u8 = 0x14;
/// The new state handlers that run the mini-map (normal, title demo, appearance, locked, elevator,
/// Ceres) and those that don't (debug, intro demo, station, drained, gunship).
const MINI_MAP_RUNS: [u16; 6] = [0xE725, 0xE7F5, 0xE86A, 0xE8DC, 0xE8EC, 0xE8AA];
const MINI_MAP_STILL: [u16; 5] = [0xE7D2, 0xE833, 0xE8D6, 0xE8D9, 0xE902];
/// The equipment bits the HUD can select.
const GRAPPLE_BEAM: u16 = 0x4000;
const X_RAY_SCOPE: u16 = 0x8000;
/// The reserve health mode in which the reserve tanks refill on their own.
const AUTO_RESERVE: u16 = 1;
/// The reserve health modes with none obtained, and refilled by hand.
const NO_RESERVE: u16 = 0;
const MANUAL_RESERVE: u16 = 2;
/// The charted cells' colour set, which the diagonal test never reads.
const CHARTED: u8 = 3;
/// The priority bit every HUD word holds.
const PRIORITY: u16 = 0x2000;
/// The mini-map's window: rows 0 to 2, columns 26 to 30 (spec §1.2).
const WINDOW: core::ops::RangeInclusive<u8> = 26..=30;
/// Her window tile, which the diagonal test reads.
const HER_TILE: (u8, u8) = (1, 28);
/// A picture's number is read mod 200h by the diagonal test, against 28h.
const DIAGONAL_READ: u16 = 0x1FF;
const DIAGONAL: u16 = 0x028;
/// A screen is 256 px, a block 16.
const SCREEN_PX: u16 = 256;
const BLOCK_PX: u16 = 16;
/// The columns from which the window reads by the first or the last columns' rule.
const FIRST_COLUMNS_END: u16 = 1;
const LAST_COLUMNS_START: u16 = 62;
/// The map's columns.
const MAP_COLUMNS: u16 = 64;

/// The declared fields by their places in [`HudDomain`]'s `FIELDS`.
const SELECTED: FieldSet = FieldSet::single(2);
const ONE_SHOT: FieldSet = FieldSet::single(3);
const WINDOW_TILES: FieldSet = FieldSet::single(4);
const EXPLORED: FieldSet = FieldSet::single(5);

/// Why a frame isn't compared.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Skip {
    /// A source couldn't be read, or holds a value the spec names nothing for: a defect.
    Unreadable(DecodeError),
    /// A HUD word without its priority bit, which every word the HUD writes holds: a defect.
    NoPriority(u16),
    /// The main loop runs another game state: the HUD runs only in gameplay (pause is its own
    /// domain).
    GameState(u16),
    /// The frame counter didn't advance by one: no main processing ran, or more than one frame.
    Frames(u16),
    /// The room changed: the door's domain.
    RoomChange,
}

impl fmt::Display for Skip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(error) => write!(f, "unreadable: {error:?}"),
            Self::NoPriority(word) => write!(f, "a HUD word without its priority bit: {word:#06X}"),
            Self::GameState(code) => write!(f, "game state {code:#04X}, not gameplay"),
            Self::Frames(count) => write!(f, "{count} frames ran, not one"),
            Self::RoomChange => f.write_str("the room changed (the door's domain)"),
        }
    }
}

impl From<DecodeError> for Skip {
    fn from(error: DecodeError) -> Self {
        Self::Unreadable(error)
    }
}

fn word(record: &FrameRecord, source: WramWord) -> Result<u16, Skip> {
    Ok(source.read(&record.wram)?)
}

/// A value the spec names nothing for.
const fn unnamed(source: &'static str, value: u16) -> Skip {
    Skip::Unreadable(DecodeError::Unrepresentable { source, value })
}

/// The HUD's tiles in the tilemap's order, row by row.
fn positions() -> impl Iterator<Item = GridPos> {
    (0..3).flat_map(|row| (0..32).filter_map(move |column| GridPos::new(row, column)))
}

/// Whether the tile at `pos` is the mini-map's window's.
fn in_window(pos: GridPos) -> bool {
    WINDOW.contains(&pos.column())
}

/// A HUD item index: 0 for none, then the HUD's items left to right.
fn item(record: &FrameRecord, source: WramWord) -> Result<Option<SelectableItem>, Skip> {
    match word(record, source)? {
        0 => Ok(None),
        index => SelectableItem::ORDER
            .get(usize::from(index).wrapping_sub(1))
            .copied()
            .map(Some)
            .ok_or_else(|| unnamed(source.name, index)),
    }
}

/// The HUD's State `record` holds (the decode notes' State).
fn hud(record: &FrameRecord) -> Result<HudState, Skip> {
    let mut state = HudState {
        shown: Shown {
            energy: word(record, source::SAMUS_PREV_HEALTH)?,
            missiles: word(record, source::SAMUS_PREV_MISSILES)?,
            super_missiles: word(record, source::SAMUS_PREV_SUPER_MISSILES)?,
            power_bombs: word(record, source::SAMUS_PREV_POWER_BOMBS)?,
            selection: item(record, source::SAMUS_PREV_HUD_ITEM_INDEX)?,
        },
        selection: Selection {
            selected: item(record, source::HUD_ITEM_INDEX)?,
            one_shot: item(record, source::SAMUS_AUTO_CANCEL_HUD_ITEM_INDEX)?,
        },
        ..HudState::default()
    };
    let words = source::HUD_TILEMAP.read(&record.wram)?.as_chunks::<2>().0;
    for (pos, &bytes) in positions().zip(words) {
        let raw = u16::from_le_bytes(bytes);
        if raw & PRIORITY == 0 {
            return Err(Skip::NoPriority(raw));
        }
        let tile = hud_world::tile(TilemapEntry::from_word(raw))
            .map_err(|_| unnamed(source::HUD_TILEMAP.name, raw))?;
        state.grid.set(pos, tile);
    }
    let bitmap: &[u8; MAP_BIT_BYTES] = source::MAP_TILES_EXPLORED
        .read(&record.wram)?
        .first_chunk()
        .ok_or(Skip::Unreadable(DecodeError::WramTooShort {
            source: source::MAP_TILES_EXPLORED.name,
        }))?;
    state.explored = hud_world::explored(&MapCells::from_bitmap(bitmap));
    Ok(state)
}

/// The button with `binding` in `buttons`.
const fn has(buttons: u16, binding: u16) -> bool {
    buttons & binding != 0
}

/// What the frame reads of record N: the frame count and the pad.
#[derive(Clone, Copy, Debug)]
pub struct Pad {
    frame: FrameCount,
    pressed: u16,
    held: u16,
}

/// Her map: its area, and whether she has it. The map itself is lent from the domain's maps when
/// the model steps.
#[derive(Clone, Copy, Debug)]
pub struct MapKey {
    area: Area,
    collected: bool,
}

/// What the frame reads of record N+1, at the HUD's turn, her map named by its [`MapKey`].
#[derive(Clone, Copy, Debug)]
pub struct Upstream {
    status: Status,
    select: u16,
    cancel: u16,
    can_switch: bool,
    muted_by: Option<Mute>,
    runs: bool,
    off: bool,
    position: Pair<u16>,
    room: Room,
    map: MapKey,
}

/// The frame's world and what the step reported, for the attribution.
#[derive(Clone, Debug)]
pub struct World {
    upstream: Upstream,
    events: Vec<Event>,
    her_tile: Option<Tile>,
}

/// The pad and the frame count of record `from`.
fn pad(from: &FrameRecord) -> Result<Pad, Skip> {
    Ok(Pad {
        frame: FrameCount(source::NMI_FRAME_COUNTER_BYTE.read(&from.wram)?),
        pressed: word(from, source::JOYPAD1_NEWKEYS)?,
        held: word(from, source::JOYPAD1_LASTKEYS)?,
    })
}

/// What the HUD shows of her, from `next`.
fn status(next: &FrameRecord) -> Result<Status, Skip> {
    let stock = |count, capacity| -> Result<Stock, Skip> {
        Ok(Stock {
            count: word(next, count)?,
            capacity: word(next, capacity)?,
        })
    };
    let missiles = stock(source::SAMUS_MISSILES, source::SAMUS_MAX_MISSILES)?;
    let ammunition = Ammunition::new(
        missiles,
        stock(
            source::SAMUS_SUPER_MISSILES,
            source::SAMUS_MAX_SUPER_MISSILES,
        )?,
        stock(source::SAMUS_POWER_BOMBS, source::SAMUS_MAX_POWER_BOMBS)?,
    )
    .ok_or_else(|| unnamed("a count past its cells", missiles.count))?;
    let equipped = word(next, source::EQUIPPED_ITEMS)?;
    Ok(Status {
        energy: Energy {
            now: word(next, source::SAMUS_HEALTH)?,
            capacity: word(next, source::SAMUS_MAX_HEALTH)?,
        },
        reserve: Reserve {
            energy: word(next, source::SAMUS_RESERVE_HEALTH)?,
            mode: match word(next, source::RESERVE_HEALTH_MODE)? {
                AUTO_RESERVE => ReserveMode::Auto,
                NO_RESERVE | MANUAL_RESERVE => ReserveMode::Manual,
                mode => return Err(unnamed(source::RESERVE_HEALTH_MODE.name, mode)),
            },
        },
        ammunition,
        equipped: Equipped {
            grapple_beam: has(equipped, GRAPPLE_BEAM),
            x_ray_scope: has(equipped, X_RAY_SCOPE),
        },
    })
}

/// Why a switch in `next` makes no click.
fn muted_by(next: &FrameRecord) -> Result<Option<Mute>, Skip> {
    let movement = source::SAMUS_MOVEMENT_TYPE.read(&next.wram)?;
    Ok(if movement == SPIN_JUMPING {
        Some(Mute::SpinJump)
    } else if movement == WALL_JUMPING {
        Some(Mute::WallJump)
    } else if word(next, GRAPPLE_FUNCTION)? != GRAPPLE_INACTIVE {
        Some(Mute::Grapple)
    } else if word(next, TIME_IS_FROZEN)? != 0 {
        Some(Mute::TimeStopped)
    } else {
        None
    })
}

/// Whether the new state handler of `next` runs the mini-map.
fn runs(next: &FrameRecord) -> Result<bool, Skip> {
    let handler = word(next, NEW_STATE_HANDLER)?;
    if MINI_MAP_RUNS.contains(&handler) {
        Ok(true)
    } else if MINI_MAP_STILL.contains(&handler) {
        Ok(false)
    } else {
        Err(unnamed(NEW_STATE_HANDLER.name, handler))
    }
}

/// Her room on her map in `next`.
fn room(next: &FrameRecord) -> Result<Room, Skip> {
    let coordinate = |source: WramWord| -> Result<u8, Skip> {
        let value = word(next, source)?;
        u8::try_from(value).map_err(|_| unnamed(source.name, value))
    };
    let (x, y) = (
        coordinate(source::ROOM_X_COORDINATE_ON_MAP)?,
        coordinate(source::ROOM_Y_COORDINATE_ON_MAP)?,
    );
    // Rooms start a row down: the map's row 0 is padding.
    let cell = MapCell::new(x, y.wrapping_add(1))
        .ok_or_else(|| unnamed(source::ROOM_Y_COORDINATE_ON_MAP.name, u16::from(y)))?;
    Ok(Room {
        cell,
        blocks: Pair {
            x: word(next, source::ROOM_WIDTH_IN_BLOCKS)?,
            y: word(next, source::ROOM_HEIGHT_IN_BLOCKS)?,
        },
    })
}

/// Every input but the pad, from `next` at the HUD's turn.
fn upstream(next: &FrameRecord) -> Result<Upstream, Skip> {
    let index = word(next, source::AREA_INDEX)?;
    let area = u8::try_from(index)
        .ok()
        .and_then(|index| Area::try_from(index).ok())
        .ok_or_else(|| unnamed(source::AREA_INDEX.name, index))?;
    Ok(Upstream {
        status: status(next)?,
        select: word(next, ITEM_SELECT_BINDING)?,
        cancel: word(next, ITEM_CANCEL_BINDING)?,
        can_switch: word(next, CURRENT_STATE_HANDLER)? == NORMAL_CURRENT
            && !FACING_FORWARD.contains(&word(next, source::SAMUS_POSE)?),
        muted_by: muted_by(next)?,
        runs: runs(next)?,
        off: word(next, source::DEBUG_DISABLE_MINIMAP)? != 0,
        position: Pair {
            x: word(next, source::SAMUS_X_POS)?,
            y: word(next, source::SAMUS_Y_POS)?,
        },
        room: room(next)?,
        map: MapKey {
            area,
            collected: word(next, source::HAS_AREA_MAP)? != 0,
        },
    })
}

impl Upstream {
    /// The buttons under the bindings, with the cancel by a shot false (the interim mask).
    const fn controls(&self, pad: Pad) -> Controls {
        Controls {
            select: if has(pad.pressed, self.select) {
                SelectButton::Pressed
            } else {
                SelectButton::Up
            },
            cancel: if has(pad.pressed, self.cancel) {
                CancelButton::Pressed
            } else if has(pad.held, self.cancel) {
                CancelButton::Held
            } else {
                CancelButton::Up
            },
            can_switch: self.can_switch,
            shot_cancels: false,
            muted_by: self.muted_by,
        }
    }

    /// The model's inputs over `map`. No cached row picks up an item with an icon or reaches
    /// Mother Brain's first form, so neither is reported.
    const fn model<'map>(&self, pad: Pad, map: &'map AreaMap) -> FrameInputs<'map> {
        FrameInputs {
            frame: pad.frame,
            status: self.status,
            controls: self.controls(pad),
            collected: None,
            boss: None,
            mini_map: MiniMapInputs {
                runs: self.runs,
                off: self.off,
                position: self.position,
                room: self.room,
                map,
            },
        }
    }

    /// Whether the mini-map updates (spec: `MiniMapStill`).
    const fn updates(&self) -> bool {
        self.runs
            && !self.off
            && self.position.x / BLOCK_PX < self.room.blocks.x
            && self.position.y / BLOCK_PX < self.room.blocks.y
    }

    /// Her cell's column on her map (spec: `Exploring`).
    fn her_column(&self) -> u16 {
        u16::from(self.room.cell.column()).wrapping_add(self.position.x / SCREEN_PX) % MAP_COLUMNS
    }
}

/// The declared fields: the status bar's tiles and the values last shown, the selection and the
/// one-shot item, the window's tiles and the explored cells.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declared {
    status_bar: Vec<Tile>,
    shown: Shown,
    selected: Option<SelectableItem>,
    one_shot: Option<SelectableItem>,
    window: Vec<Tile>,
    explored: MapCellSet,
}

impl From<&HudState> for Declared {
    fn from(state: &HudState) -> Self {
        let tiles = |window: bool| -> Vec<Tile> {
            positions()
                .filter(|&pos| in_window(pos) == window)
                .map(|pos| state.grid.tile(pos))
                .collect()
        };
        Self {
            status_bar: tiles(false),
            shown: state.shown,
            selected: state.selection.selected,
            one_shot: state.selection.one_shot,
            window: tiles(true),
            explored: state.explored,
        }
    }
}

/// The HUD domain over the ROM's area maps.
pub struct HudDomain {
    maps: AreaMaps,
}

impl HudDomain {
    /// The domain over the ROM `image`'s area maps.
    ///
    /// # Errors
    ///
    /// [`ContentError::AreaMaps`] when the maps don't load.
    pub fn load(image: &[u8]) -> Result<Self, ContentError> {
        let rom = data::rom::Lorom::new(image);
        let maps = data::map::AreaMaps::load(rom).map_err(ContentError::AreaMaps)?;
        Ok(Self {
            maps: AreaMaps::import(&maps),
        })
    }
}

/// Whether the main loop runs gameplay in `record`.
fn gameplay(record: &FrameRecord) -> Result<(), Skip> {
    match word(record, source::GAME_STATE)? {
        MAIN_GAMEPLAY => Ok(()),
        code => Err(Skip::GameState(code)),
    }
}

impl Domain for HudDomain {
    const NAME: &'static str = "hud";
    type State = HudState;
    type Inputs = Pad;
    type World = World;
    type Fields = Declared;
    type Skip = Skip;
    const FIELDS: &'static [Field<Declared>] = &[
        domain_field!("status bar", status_bar),
        domain_field!("values last shown", shown),
        domain_field!("selected item", selected),
        domain_field!("one-shot item", one_shot),
        domain_field!("mini-map window", window),
        domain_field!("explored cells", explored),
    ];

    fn behaviours(&self) -> Vec<String> {
        CATALOGUE
            .iter()
            .map(|entry| format!("{:?}", entry.id))
            .collect()
    }

    fn decode(&self, from: &FrameRecord, _interval: Interval) -> Result<(HudState, Pad), Skip> {
        gameplay(from)?;
        Ok((hud(from)?, pad(from)?))
    }

    fn world(&self, from: &FrameRecord, next: &FrameRecord) -> Result<World, Skip> {
        gameplay(next)?;
        let frames = word(next, source::NMI_FRAME_COUNTER_WORD)?
            .wrapping_sub(word(from, source::NMI_FRAME_COUNTER_WORD)?);
        if frames != 1 {
            return Err(Skip::Frames(frames));
        }
        if word(from, source::ROOM_PTR)? != word(next, source::ROOM_PTR)? {
            return Err(Skip::RoomChange);
        }
        Ok(World {
            upstream: upstream(next)?,
            events: Vec::new(),
            her_tile: None,
        })
    }

    fn masks(&self, from: &FrameRecord, next: &FrameRecord, _pad: &Pad) -> Result<Vec<Mask>, Skip> {
        let mut masks = Vec::new();
        if word(from, source::SAMUS_POSE)? != word(next, source::SAMUS_POSE)? {
            masks.push(Mask {
                reason: "interim: her pose changed, so its placement ($91:EB88) may have moved her after the mini-map read her; no hit recorded",
                fields: WINDOW_TILES.union(EXPLORED),
            });
        }
        let mut fell = false;
        for count in [
            source::SAMUS_MISSILES,
            source::SAMUS_SUPER_MISSILES,
            source::SAMUS_POWER_BOMBS,
        ] {
            fell |= word(next, count)? < word(from, count)?;
        }
        if fell {
            masks.push(Mask {
                reason: "interim: a count fell, so the weapons may have cancelled the selection ($90:BF25, $90:BF41, $90:C091, $90:C0A6); no hit recorded",
                fields: SELECTED.union(ONE_SHOT),
            });
        }
        Ok(masks)
    }

    fn step(&self, state: &mut HudState, pad: &Pad, world: &mut World) {
        let upstream = &world.upstream;
        let map = self.maps.of(upstream.map.area, upstream.map.collected);
        let outcome = state
            .clone()
            .frame(&Rules::reference(), &upstream.model(*pad, map));
        *state = outcome.state;
        world.her_tile = GridPos::new(HER_TILE.0, HER_TILE.1).map(|pos| state.grid.tile(pos));
        world.events = outcome.events;
    }

    fn exercised(&self, state: &HudState, pad: &Pad, world: &World) -> Vec<usize> {
        exercised(state, *pad, world)
    }

    fn fields(&self, state: &HudState) -> Declared {
        Declared::from(state)
    }

    fn expected(&self, next: &FrameRecord) -> Result<Declared, Skip> {
        Ok(Declared::from(&hud(next)?))
    }

    fn uncompared(&self, skip: &Skip) -> Uncompared {
        match skip {
            Skip::Unreadable(_) | Skip::NoPriority(_) => Uncompared::Defect(skip.to_string()),
            Skip::GameState(_) | Skip::Frames(_) | Skip::RoomChange => {
                Uncompared::Skip(skip.to_string())
            }
        }
    }
}

/// The catalogue entries a frame brings into play, from the HUD as record N has it, the inputs,
/// and what the step reported: the rule each part of the frame takes.
fn exercised(state: &HudState, pad: Pad, world: &World) -> Vec<usize> {
    use BehaviourId as B;
    let upstream = &world.upstream;
    let status = &upstream.status;
    let controls = upstream.controls(pad);
    let shown = &state.shown;
    let mut entries = Vec::new();
    if status.energy.now != shown.energy {
        entries.push(B::EnergyGauge);
    }
    let counts = [Ammo::Missiles, Ammo::SuperMissiles, Ammo::PowerBombs];
    if counts.iter().any(|&ammo| {
        let stock = status.ammunition.stock(ammo);
        stock.capacity != 0 && stock.count != shown.ammo(ammo)
    }) {
        entries.push(B::AmmoCounts);
    }
    if status.reserve.mode == ReserveMode::Auto {
        entries.push(B::AutoReserveSign);
    }
    let switched = world
        .events
        .iter()
        .any(|event| matches!(event, Event::Switched(_)));
    let mut selection = state.selection;
    if controls.can_switch {
        if controls.cancel == CancelButton::Pressed {
            entries.push(B::CancellingTheItem);
        } else if controls.select == SelectButton::Pressed {
            entries.push(B::CyclingItems);
            entries.push(B::OneShotItem);
        }
        if switched {
            entries.push(B::Switching);
            selection.selected = world
                .events
                .iter()
                .find_map(|event| match event {
                    Event::Switched(item) => Some(*item),
                    Event::Click => None,
                })
                .flatten();
        }
    }
    if selection.selected != shown.selection {
        entries.extend([B::SelectionHighlight, B::SwitchClick]);
    }
    if state.selection.one_shot.is_some() || selection.one_shot.is_some() {
        entries.push(B::OneShotFlash);
    }
    if upstream.updates() {
        entries.extend([B::Exploring, B::MiniMapWindow, B::HerBlink]);
        let column = upstream.her_column();
        if column <= FIRST_COLUMNS_END {
            entries.push(B::FirstColumnsShift);
        } else if column >= LAST_COLUMNS_START {
            entries.push(B::LastColumnsShift);
        }
        // The diagonal test reads her window tile as the explored branch drew it.
        if world.her_tile.is_some_and(|tile| {
            (tile.picture.number() & DIAGONAL_READ) == DIAGONAL && tile.colours.number() != CHARTED
        }) {
            entries.push(B::DiagonalPicture);
        }
    } else {
        entries.push(B::MiniMapStill);
    }
    let mut places: Vec<usize> = entries
        .into_iter()
        .filter_map(|id| CATALOGUE.iter().position(|entry| entry.id == id))
        .collect();
    places.sort_unstable();
    places.dedup();
    places
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_field_sets_name_the_declared_fields_at_their_places() {
        let place = |name| {
            HudDomain::FIELDS
                .iter()
                .position(|field| field.name == name)
                .map(FieldSet::single)
        };
        assert_eq!(place("selected item"), Some(SELECTED));
        assert_eq!(place("one-shot item"), Some(ONE_SHOT));
        assert_eq!(place("mini-map window"), Some(WINDOW_TILES));
        assert_eq!(place("explored cells"), Some(EXPLORED));
    }
}
