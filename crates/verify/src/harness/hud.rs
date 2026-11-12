//! The HUD as a harness domain: the HUD model (`interface_hud`) against the oracle.
//!
//! It is checked frame by frame by the decode notes' harness note (`docs/behaviours/interface-hud.decode.md`,
//! "The harness"). The HUD is downstream of everything it shows and nothing of the same frame reads
//! it back, so it is checked on its own: its State (the HUD's cells, the values last shown, the
//! selection and the one-shot item, the explored cells, whether the mini-map is off) and the frame
//! count and pad are decoded from record N, every other input from record N+1 at the HUD's turn,
//! and the model's output is compared with record N+1: the whole tilemap, not values, since a
//! stale cell is behaviour. The HUD's pictures and her map's are the importer's.
//!
//! A frame runs from cut to cut: from a record whose NMI returned to the main loop's wait (cut
//! `R`) to the next such record. A record with any other cut is mid-frame (an overrun, vblank
//! with the NMI disabled, or vblank ended with the NMI still running) and folds into its frame,
//! its hits carried along, so a frame whose work straddles an overrun is paired whole.
//!
//! The inputs other domains' writers make are named by their instruction hits in frame N+1: the
//! shot's cancel (the weapons' stores `$90:BF25`, `$90:BF41`, `$90:C091`, `$90:C0A6`), the item
//! whose pickup draws its icon (`$80:99CF`, `$80:9A0E`, `$80:9A1E`, `$80:9A2E`, `$80:9A3E`), Mother
//! Brain's fall (`$90:A7E2` with her boss id) and the mini-map turned on (the six stores of 0 to
//! `05F7`). A boss met on entering its room calls `$90:A7E2` in a door's or a load's frame, outside
//! gameplay: that pair is the model's boss step, not a frame of play.
//!
//! One interim mask stands in for a writer the records don't yet decode, keyed as near it as the
//! records allow and reported as masked, never passed: her position at the mini-map's turn: the pose change's placement (`$91:EB88`) runs after the
//!   mini-map read her and can move her across a screen, so the window and the explored cells are
//!   masked where her pose changed, the mini-map read her, and her position lies within the
//!   placement's reach of a screen's edge. samus-movement's model, which the camera harness asks
//!   for the y the camera follows her from, doesn't report where her moves left her, so it can't
//!   yet decode her position at the mini-map's turn.
//!
//! The click and the switch aren't compared until the
//! sound and the weapons' request streams exist, so their entries are credited nothing. Nothing
//! answers from record N+1's HUD, so no field is unverified; the null model is the HUD left as
//! record N has it.

pub mod cases;

use core::fmt;

use data::map::{MAP_BIT_BYTES, MapCells};
use data::room::Area;
use enum_map::enum_map;
use game::interface_hud_bridge::{self, HudContent};
use game::state::video::TilemapEntry;
use interface_hud::{
    Ammo, Ammunition, BehaviourId, CATALOGUE, CancelButton, Controls, Energy, Equipped, FrameCount,
    FrameInputs, HerMap, HudEvent, HudGrid, HudState, MapCellSet, MiniMapInputs, MiniMapPower,
    Mute, Place, Reserve, ReserveMode, Room, RoomBoss, Rules, SelectButton, SelectableItem,
    Selection, Shown, Status, Stock, Tile,
};
use oracle_record::FrameRecord;
use types::Pair;

use super::{Domain, Entry, Field, FieldSet, Hides, Interval, Mask, Uncompared};
use crate::bundle::ContentError;
use crate::cache::CutKind;
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
/// Bank $A0 log: the boss id the room's boss enemy sets when it loads.
const BOSS_ID: WramWord = wram("BOSS_ID", 0x179C);

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
const WINDOW: core::ops::RangeInclusive<i32> = 26..=30;
/// The HUD's cells across a row, and its rows.
const HUD_COLUMNS: usize = 32;
const HUD_ROWS: usize = 3;
/// Her window tile, which the diagonal test reads.
const HER_TILE: Place = Pair { x: 28, y: 1 };
/// The HUD's item indices from 1, as the bank $90 log orders them: the harness's own table, so
/// a model that ordered its items otherwise would fail rather than decode the records its way.
const HUD_ITEMS: [SelectableItem; 5] = [
    SelectableItem::Ammo(Ammo::Missiles),
    SelectableItem::Ammo(Ammo::SuperMissiles),
    SelectableItem::Ammo(Ammo::PowerBombs),
    SelectableItem::GrappleBeam,
    SelectableItem::XRayScope,
];
/// A picture's number is read mod 200h by the diagonal test.
const DIAGONAL_READ: u16 = 0x1FF;
/// A screen is 256 px, a block 16.
const SCREEN_PX: u16 = 256;
const BLOCK_PX: u16 = 16;
/// How far the pose change's placement can move her: samus-movement's pose fit, the shrink, the
/// take-off's lift and the aim step each move her by less than this, and never together by more.
const PLACEMENT_REACH: u16 = 32;
/// The columns from which the window reads by the first or the last columns' rule.
const FIRST_COLUMNS_END: i32 = 1;
const LAST_COLUMNS_START: i32 = 62;
/// The map's columns.
const MAP_COLUMNS: i32 = 64;

/// Blanks the mini-map and turns it off (`$90:A7E2`), called by the bosses.
const BOSS_BLANKS_THE_MAP: u32 = 0x90_A7E2;
/// The stores of 0 to `05F7` that turn the mini-map on: a load at a station, the demos' load, a
/// door's start, the new room's effects setup, a new game's or a load's Samus command 8, and
/// Crocomire's skeleton flowing down the river.
const TURNS_THE_MINI_MAP_ON: [u32; 6] = [
    0x80_C4AF, 0x82_8767, 0x82_E1C4, 0x88_82F3, 0x90_F233, 0xA4_90C4,
];
/// The weapons' stores that cancel the selection after a shot: `$90:BF25` and `$90:C091` clear
/// the selected and the one-shot item, `$90:BF41` and `$90:C0A6` the selected item when there is
/// no one-shot item.
const SHOT_CANCELS: [u32; 4] = [0x90_BF25, 0x90_BF41, 0x90_C091, 0x90_C0A6];
/// The entries that draw a picked-up item's icon, by its item.
const PICKUPS: [(u32, SelectableItem); 5] = [
    (0x80_99CF, SelectableItem::Ammo(Ammo::Missiles)),
    (0x80_9A0E, SelectableItem::Ammo(Ammo::SuperMissiles)),
    (0x80_9A1E, SelectableItem::Ammo(Ammo::PowerBombs)),
    (0x80_9A2E, SelectableItem::GrappleBeam),
    (0x80_9A3E, SelectableItem::XRayScope),
];
/// Mother Brain's boss id, whose first form calls `$90:A7E2` during gameplay.
const MOTHER_BRAIN: u16 = 0x0A;
/// The boss ids of the bosses met on entering their rooms (`$90:A83A`; Ceres Ridley's id 1 isn't
/// in that table).
const ROOM_BOSSES: [(u16, RoomBoss); 6] = [
    (0x01, RoomBoss::RidleyInCeres),
    (0x03, RoomBoss::Kraid),
    (0x05, RoomBoss::RidleyInNorfair),
    (0x06, RoomBoss::Crocomire),
    (0x07, RoomBoss::Phantoon),
    (0x08, RoomBoss::Draygon),
];

/// The declared fields by their places in [`HudDomain`]'s `FIELDS`.
const STATUS_BAR: FieldSet = FieldSet::single(0);
const SHOWN: FieldSet = FieldSet::single(1);
const SELECTED: FieldSet = FieldSet::single(2);
const ONE_SHOT: FieldSet = FieldSet::single(3);
const WINDOW_TILES: FieldSet = FieldSet::single(4);
const EXPLORED: FieldSet = FieldSet::single(5);
const MINI_MAP_OFF: FieldSet = FieldSet::single(6);

/// The declared fields a catalogue entry's output reaches. The click and the switch are events,
/// which aren't compared until the sound and the weapons' request streams exist, so
/// `SwitchClick` and `Switching` are credited nothing.
const fn shown_by(id: BehaviourId) -> FieldSet {
    use BehaviourId as B;
    match id {
        B::EnergyGauge | B::AmmoCounts | B::SelectionHighlight => STATUS_BAR.union(SHOWN),
        B::AutoReserveSign | B::OneShotFlash | B::IconsAppear => STATUS_BAR,
        B::SwitchClick | B::Switching => FieldSet::EMPTY,
        B::CyclingItems => SELECTED,
        B::CancellingTheItem => SELECTED.union(ONE_SHOT),
        B::OneShotItem => ONE_SHOT,
        B::Exploring | B::DiagonalPicture => EXPLORED,
        B::MiniMapWindow | B::HerBlink => WINDOW_TILES,
        B::FirstColumnsShift | B::LastColumnsShift => WINDOW_TILES.union(EXPLORED),
        B::MiniMapStill | B::BossBlanksTheMap => WINDOW_TILES.union(EXPLORED).union(MINI_MAP_OFF),
    }
}

/// Why a frame isn't compared.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Skip {
    /// A source couldn't be read, or holds a value the spec names nothing for: a defect.
    Unreadable(DecodeError),
    /// A HUD word without its priority bit, which every word the HUD writes holds: a defect.
    NoPriority(u16),
    /// A one-shot item that is neither none nor the selected item, which the original never
    /// holds: a defect.
    OneShotNotSelected {
        /// The selected item's index.
        selected: u16,
        /// The one-shot item's.
        one_shot: u16,
    },
    /// A boss met on entering its room in a frame of play, which only a door or a load meets:
    /// a defect.
    BossInPlay(u16),
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
            Self::OneShotNotSelected { selected, one_shot } => write!(
                f,
                "a one-shot item {one_shot} that is neither none nor the selected {selected}"
            ),
            Self::BossInPlay(id) => write!(f, "boss {id:#04X} met in a frame of play"),
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

/// Whether `record` ran the instruction at `pc` in its frame.
fn ran(record: &FrameRecord, pc: u32) -> bool {
    record.hits.iter().any(|hit| hit.pc == pc)
}

/// Whether `at`, her position on one axis, is within the pose change's reach of a screen's edge,
/// so that where her moves left her, which the mini-map read, may lie in the next screen.
const fn near_screen_edge(at: u16) -> bool {
    let into = at % SCREEN_PX;
    into < PLACEMENT_REACH || into >= SCREEN_PX - PLACEMENT_REACH
}

/// Whether the HUD cell at `place` is the mini-map's window's.
fn in_window(place: Place) -> bool {
    WINDOW.contains(&place.x)
}

/// The place `column` across and `row` down the HUD.
fn place(column: usize, row: usize) -> Place {
    let coordinate = |value: usize| i32::try_from(value).unwrap_or(i32::MAX);
    Pair {
        x: coordinate(column),
        y: coordinate(row),
    }
}

/// A HUD item index: 0 for none, then the HUD's items left to right.
fn item(index: u16, source: WramWord) -> Result<Option<SelectableItem>, Skip> {
    match index {
        0 => Ok(None),
        index => HUD_ITEMS
            .get(usize::from(index).wrapping_sub(1))
            .copied()
            .map(Some)
            .ok_or_else(|| unnamed(source.name, index)),
    }
}

/// The selection `record` holds: the selected item, which is the one-shot item when the one-shot
/// index equals it.
fn selection(record: &FrameRecord) -> Result<Selection, Skip> {
    let selected = word(record, source::HUD_ITEM_INDEX)?;
    let one_shot = word(record, source::SAMUS_AUTO_CANCEL_HUD_ITEM_INDEX)?;
    match (item(selected, source::HUD_ITEM_INDEX)?, one_shot) {
        (None, 0) => Ok(Selection::None),
        (Some(item), 0) => Ok(Selection::Item(item)),
        (Some(item), _) if one_shot == selected => Ok(Selection::OneShot(item)),
        (None | Some(_), _) => Err(Skip::OneShotNotSelected { selected, one_shot }),
    }
}

/// The tile of a HUD word.
fn tile(bytes: [u8; 2]) -> Result<Tile, Skip> {
    let raw = u16::from_le_bytes(bytes);
    if raw & PRIORITY == 0 {
        return Err(Skip::NoPriority(raw));
    }
    interface_hud_bridge::tile(TilemapEntry::from_word(raw))
        .map_err(|_| unnamed(source::HUD_TILEMAP.name, raw))
}

/// The HUD's cells `record` holds.
fn grid(record: &FrameRecord) -> Result<HudGrid, Skip> {
    let words = source::HUD_TILEMAP.read(&record.wram)?.as_chunks::<2>().0;
    let first = words
        .first()
        .ok_or(Skip::Unreadable(DecodeError::WramTooShort {
            source: source::HUD_TILEMAP.name,
        }))?;
    // Every cell is set from the record below.
    let mut grid = HudGrid::filled(tile(*first)?);
    for (row, cells) in words.chunks(HUD_COLUMNS).enumerate() {
        for (column, &bytes) in cells.iter().enumerate() {
            grid.set(place(column, row), tile(bytes)?);
        }
    }
    Ok(grid)
}

/// The HUD's State `record` holds (the decode notes' State).
fn hud(record: &FrameRecord) -> Result<HudState, Skip> {
    let bitmap: &[u8; MAP_BIT_BYTES] = source::MAP_TILES_EXPLORED
        .read(&record.wram)?
        .first_chunk()
        .ok_or(Skip::Unreadable(DecodeError::WramTooShort {
            source: source::MAP_TILES_EXPLORED.name,
        }))?;
    Ok(HudState {
        grid: grid(record)?,
        shown: Shown {
            energy: word(record, source::SAMUS_PREV_HEALTH)?,
            ammo: enum_map! {
                Ammo::Missiles => word(record, source::SAMUS_PREV_MISSILES)?,
                Ammo::SuperMissiles => word(record, source::SAMUS_PREV_SUPER_MISSILES)?,
                Ammo::PowerBombs => word(record, source::SAMUS_PREV_POWER_BOMBS)?,
            },
            highlighted: item(
                word(record, source::SAMUS_PREV_HUD_ITEM_INDEX)?,
                source::SAMUS_PREV_HUD_ITEM_INDEX,
            )?,
        },
        selection: selection(record)?,
        explored: interface_hud_bridge::explored(&MapCells::from_bitmap(bitmap)),
        mini_map: if word(record, source::DEBUG_DISABLE_MINIMAP)? == 0 {
            MiniMapPower::On
        } else {
            MiniMapPower::Off
        },
    })
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

/// Her map: its area, and whether she has collected it. The map itself is lent from the domain's
/// content when the model steps.
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
    shot_cancels: bool,
    collected: Option<SelectableItem>,
    power: PowerChanges,
    position: Pair<u16>,
    room: Room,
    map: MapKey,
}

/// What turns the mini-map on or off in a frame, before its turn: other domains' writers.
#[derive(Clone, Copy, Debug)]
pub struct PowerChanges {
    /// One of the stores of 0 to `05F7` ran.
    turned_on: bool,
    /// Mother Brain's first form fell.
    mother_brain_falls: bool,
}

/// What a record pair is to the HUD.
#[derive(Clone, Copy, Debug)]
pub enum Turn {
    /// A frame of play.
    Frame(Upstream),
    /// A door's or a load's frame in which a boss was met on entering its room.
    BossMet {
        /// The boss.
        boss: RoomBoss,
        /// Her room's top-left cell on her map.
        room: Place,
        /// Her area, whose map the HUD draws.
        area: Area,
    },
}

impl Turn {
    /// Her area, whose map the HUD draws.
    const fn area(&self) -> Area {
        match self {
            Self::Frame(upstream) => upstream.map.area,
            Self::BossMet { area, .. } => *area,
        }
    }
}

/// The pair's turn and what the step reported, for the attribution.
#[derive(Clone, Debug)]
pub struct World {
    turn: Turn,
    events: Vec<HudEvent>,
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
    let ammunition = Ammunition::new(enum_map! {
        Ammo::Missiles => missiles,
        Ammo::SuperMissiles => stock(
            source::SAMUS_SUPER_MISSILES,
            source::SAMUS_MAX_SUPER_MISSILES,
        )?,
        Ammo::PowerBombs => stock(source::SAMUS_POWER_BOMBS, source::SAMUS_MAX_POWER_BOMBS)?,
    })
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

/// Her room's top-left cell on her map in `next`. Rooms start a row down: the map's row 0 is
/// padding.
fn room_cell(next: &FrameRecord) -> Result<Place, Skip> {
    let coordinate = |source: WramWord| -> Result<i32, Skip> {
        let value = word(next, source)?;
        u8::try_from(value)
            .map(i32::from)
            .map_err(|_| unnamed(source.name, value))
    };
    Ok(Pair {
        x: coordinate(source::ROOM_X_COORDINATE_ON_MAP)?,
        y: coordinate(source::ROOM_Y_COORDINATE_ON_MAP)?.wrapping_add(1),
    })
}

/// Her room on her map in `next`.
fn room(next: &FrameRecord) -> Result<Room, Skip> {
    Ok(Room {
        cell: room_cell(next)?,
        blocks: Pair {
            x: word(next, source::ROOM_WIDTH_IN_BLOCKS)?,
            y: word(next, source::ROOM_HEIGHT_IN_BLOCKS)?,
        },
    })
}

/// A boss that called `$90:A7E2` in a frame, by what the HUD does with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BossCalled {
    /// Mother Brain's first form fell.
    MotherBrain,
    /// A boss met on entering its room.
    InItsRoom(RoomBoss),
}

/// The boss that blanked the mini-map in `next`'s frame, or `None`.
fn boss_called(next: &FrameRecord) -> Result<Option<BossCalled>, Skip> {
    if !ran(next, BOSS_BLANKS_THE_MAP) {
        return Ok(None);
    }
    match word(next, BOSS_ID)? {
        MOTHER_BRAIN => Ok(Some(BossCalled::MotherBrain)),
        id => ROOM_BOSSES
            .iter()
            .find(|&&(each, _)| each == id)
            .map(|&(_, boss)| Some(BossCalled::InItsRoom(boss)))
            .ok_or_else(|| unnamed(BOSS_ID.name, id)),
    }
}

/// Her area in `record`.
fn area(record: &FrameRecord) -> Result<Area, Skip> {
    let index = word(record, source::AREA_INDEX)?;
    u8::try_from(index)
        .ok()
        .and_then(|index| Area::try_from(index).ok())
        .ok_or_else(|| unnamed(source::AREA_INDEX.name, index))
}

/// Every input but the pad, from `next` at the HUD's turn; Mother Brain falls when
/// `mother_brain_falls`.
fn upstream(next: &FrameRecord, mother_brain_falls: bool) -> Result<Upstream, Skip> {
    Ok(Upstream {
        status: status(next)?,
        select: word(next, ITEM_SELECT_BINDING)?,
        cancel: word(next, ITEM_CANCEL_BINDING)?,
        can_switch: word(next, CURRENT_STATE_HANDLER)? == NORMAL_CURRENT
            && !FACING_FORWARD.contains(&word(next, source::SAMUS_POSE)?),
        muted_by: muted_by(next)?,
        runs: runs(next)?,
        shot_cancels: SHOT_CANCELS.iter().any(|&pc| ran(next, pc)),
        collected: next.hits.iter().find_map(|hit| {
            PICKUPS
                .iter()
                .find(|&&(pc, _)| pc == hit.pc)
                .map(|&(_, item)| item)
        }),
        power: PowerChanges {
            turned_on: TURNS_THE_MINI_MAP_ON.iter().any(|&pc| ran(next, pc)),
            mother_brain_falls,
        },
        position: Pair {
            x: word(next, source::SAMUS_X_POS)?,
            y: word(next, source::SAMUS_Y_POS)?,
        },
        room: room(next)?,
        map: MapKey {
            area: area(next)?,
            collected: word(next, source::HAS_AREA_MAP)? != 0,
        },
    })
}

impl Upstream {
    /// The buttons under the bindings, and the shot's cancel.
    const fn controls(&self, pad: Pad) -> Controls {
        Controls {
            select: if has(pad.pressed, self.select) {
                SelectButton::Pressed
            } else {
                SelectButton::NotPressed
            },
            cancel: if has(pad.pressed, self.cancel) {
                CancelButton::Pressed
            } else if has(pad.held, self.cancel) {
                CancelButton::Held
            } else {
                CancelButton::NotPressed
            },
            can_switch: self.can_switch,
            shot_cancels: self.shot_cancels,
            muted_by: self.muted_by,
        }
    }

    /// The model's inputs over `map`.
    const fn model<'map>(&self, pad: Pad, map: &'map HerMap) -> FrameInputs<'map> {
        FrameInputs {
            frame: pad.frame,
            status: self.status,
            controls: self.controls(pad),
            collected: self.collected,
            mother_brain_falls: self.power.mother_brain_falls,
            mini_map: MiniMapInputs {
                runs: self.runs,
                turned_on: self.power.turned_on,
                position: self.position,
                room: self.room,
                map,
                map_collected: self.map.collected,
            },
        }
    }

    /// Whether the mini-map is off at its turn, `before` as record N holds it: turned on before
    /// the turn, and off after Mother Brain's fall (spec: `MiniMapStill`, `BossBlanksTheMap`).
    const fn off(&self, before: MiniMapPower) -> bool {
        self.power.mother_brain_falls
            || (matches!(before, MiniMapPower::Off) && !self.power.turned_on)
    }

    /// Whether the mini-map updates (spec: `MiniMapStill`).
    const fn updates(&self, before: MiniMapPower) -> bool {
        self.runs
            && !self.off(before)
            && self.position.x / BLOCK_PX < self.room.blocks.x
            && self.position.y / BLOCK_PX < self.room.blocks.y
    }

    /// Her cell's column on her map (spec: `Exploring`).
    fn her_column(&self) -> i32 {
        self.room
            .cell
            .x
            .wrapping_add(i32::from(self.position.x / SCREEN_PX))
            .rem_euclid(MAP_COLUMNS)
    }
}

/// The declared fields: the status bar's tiles and the values last shown, the selection and the
/// one-shot item, the window's tiles, the explored cells and whether the mini-map is off.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declared {
    status_bar: Vec<Tile>,
    shown: Shown,
    selected: Option<SelectableItem>,
    one_shot: Option<SelectableItem>,
    window: Vec<Tile>,
    explored: MapCellSet,
    mini_map_off: bool,
}

impl From<&HudState> for Declared {
    fn from(state: &HudState) -> Self {
        let tiles = |window: bool| -> Vec<Tile> {
            (0..HUD_ROWS)
                .flat_map(|row| (0..HUD_COLUMNS).map(move |column| place(column, row)))
                .filter(|&point| in_window(point) == window)
                .filter_map(|point| state.grid.get(point))
                .collect()
        };
        Self {
            status_bar: tiles(false),
            shown: state.shown,
            selected: state.selection.item(),
            one_shot: state.selection.one_shot(),
            window: tiles(true),
            explored: state.explored,
            mini_map_off: state.mini_map == MiniMapPower::Off,
        }
    }
}

/// The HUD domain over the importer's area maps and pictures, and the spec's rules.
pub struct HudDomain {
    content: HudContent,
    rules: Rules,
}

impl HudDomain {
    /// The domain over the ROM `image`'s area maps and HUD pictures.
    ///
    /// # Errors
    ///
    /// [`ContentError::AreaMaps`] or [`ContentError::HudPictures`] when they don't load.
    pub fn load(image: &[u8]) -> Result<Self, ContentError> {
        let rom = data::rom::Lorom::new(image);
        let maps = data::map::AreaMaps::load(rom).map_err(ContentError::AreaMaps)?;
        let pictures = data::hud::HudPictures::load(rom).map_err(ContentError::HudPictures)?;
        Ok(Self {
            content: HudContent::import(&maps, &pictures),
            rules: Rules::reference(),
        })
    }
}

/// The game state the main loop runs in `record`.
fn game_state(record: &FrameRecord) -> Result<u16, Skip> {
    word(record, source::GAME_STATE)
}

/// Whether the main loop runs gameplay in both records of a pair.
fn in_play(from: &FrameRecord, next: &FrameRecord) -> Result<bool, Skip> {
    Ok(game_state(from)? == MAIN_GAMEPLAY && game_state(next)? == MAIN_GAMEPLAY)
}

impl Domain for HudDomain {
    const NAME: &'static str = "interface-hud";
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
        domain_field!("mini-map off", mini_map_off),
    ];

    fn behaviours(&self) -> Vec<Entry> {
        CATALOGUE
            .iter()
            .map(|entry| Entry {
                name: format!("{:?}", entry.id),
                fields: shown_by(entry.id),
            })
            .collect()
    }

    /// Outside gameplay the HUD is decoded too, for the boss step a door or a load makes; a
    /// record there that doesn't decode is skipped for its game state, not a defect.
    fn decode(&self, from: &FrameRecord, _interval: Interval) -> Result<(HudState, Pad), Skip> {
        let state = game_state(from)?;
        hud(from)
            .and_then(|hud| Ok((hud, pad(from)?)))
            .map_err(|skip| {
                if state == MAIN_GAMEPLAY {
                    skip
                } else {
                    Skip::GameState(state)
                }
            })
    }

    fn world(&self, from: &FrameRecord, next: &FrameRecord) -> Result<World, Skip> {
        let called = boss_called(next)?;
        let world = |turn| World {
            turn,
            events: Vec::new(),
            her_tile: None,
        };
        for record in [from, next] {
            match (game_state(record)?, called) {
                (MAIN_GAMEPLAY, _) => {}
                (_, Some(BossCalled::InItsRoom(boss))) => {
                    return Ok(world(Turn::BossMet {
                        boss,
                        room: room_cell(next)?,
                        area: area(next)?,
                    }));
                }
                (state, _) => return Err(Skip::GameState(state)),
            }
        }
        let frames = word(next, source::NMI_FRAME_COUNTER_WORD)?
            .wrapping_sub(word(from, source::NMI_FRAME_COUNTER_WORD)?);
        if frames != 1 {
            return Err(Skip::Frames(frames));
        }
        if word(from, source::ROOM_PTR)? != word(next, source::ROOM_PTR)? {
            return Err(Skip::RoomChange);
        }
        let mother_brain_falls = match called {
            None => false,
            Some(BossCalled::MotherBrain) => true,
            Some(BossCalled::InItsRoom(_)) => return Err(Skip::BossInPlay(word(next, BOSS_ID)?)),
        };
        Ok(world(Turn::Frame(upstream(next, mother_brain_falls)?)))
    }

    fn masks(&self, from: &FrameRecord, next: &FrameRecord, _pad: &Pad) -> Result<Vec<Mask>, Skip> {
        let mut masks = Vec::new();
        if !in_play(from, next)? {
            return Ok(masks);
        }
        let falls = boss_called(next)? == Some(BossCalled::MotherBrain);
        let upstream = upstream(next, falls)?;
        let read_her = upstream.runs && !upstream.off(hud(from)?.mini_map);
        let near_edge =
            near_screen_edge(upstream.position.x) || near_screen_edge(upstream.position.y);
        if read_her
            && near_edge
            && word(from, source::SAMUS_POSE)? != word(next, source::SAMUS_POSE)?
        {
            masks.push(Mask {
                reason: "interim: her pose changed within its placement's reach of a screen's edge and the mini-map read her, so the placement ($91:EB88) may have moved her across after; no hit recorded, and samus-movement's model reports only the y the camera follows her from",
                fields: WINDOW_TILES.union(EXPLORED),
                hides: Hides::AnotherWriter,
            });
        }
        Ok(masks)
    }

    fn step(&self, state: &mut HudState, pad: &Pad, world: &mut World) {
        let (map, pictures) = self.content.of(world.turn.area());
        match world.turn {
            Turn::Frame(upstream) => {
                let outcome =
                    state
                        .clone()
                        .frame(&self.rules, pictures, &upstream.model(*pad, map));
                *state = outcome.state;
                world.events = outcome.events;
            }
            Turn::BossMet { boss, room, .. } => {
                *state = state
                    .clone()
                    .boss_encountered(&self.rules, pictures, boss, room);
            }
        }
        world.her_tile = state.grid.get(HER_TILE);
    }

    fn exercised(&self, state: &HudState, pad: &Pad, world: &World) -> Vec<usize> {
        let (_, pictures) = self.content.of(world.turn.area());
        exercised(state, *pad, world, pictures.diagonal.index())
    }

    fn fields(&self, state: &HudState) -> Declared {
        Declared::from(state)
    }

    fn expected(&self, next: &FrameRecord) -> Result<Declared, Skip> {
        Ok(Declared::from(&hud(next)?))
    }

    fn uncompared(&self, skip: &Skip) -> Uncompared {
        match skip {
            Skip::Unreadable(_)
            | Skip::NoPriority(_)
            | Skip::OneShotNotSelected { .. }
            | Skip::BossInPlay(_) => Uncompared::Defect(skip.to_string()),
            Skip::GameState(_) | Skip::Frames(_) | Skip::RoomChange => {
                Uncompared::Skip(skip.to_string())
            }
        }
    }

    /// A record whose cut isn't `R` is mid-frame (an overrun, vblank with the NMI disabled, or
    /// vblank ended with the NMI still running): the frame runs on to the next `R`.
    fn folds(&self, _from: &FrameRecord, next: &FrameRecord) -> bool {
        CutKind::of(next.cut) != CutKind::Requested
    }
}

/// The places in the catalogue of the entries `ids` names, in its order.
fn places(ids: impl IntoIterator<Item = BehaviourId>) -> Vec<usize> {
    let mut places: Vec<usize> = ids
        .into_iter()
        .filter_map(|id| CATALOGUE.iter().position(|entry| entry.id == id))
        .collect();
    places.sort_unstable();
    places.dedup();
    places
}

/// The catalogue entries a frame brings into play, from the HUD as record N has it, the inputs,
/// and what the step reported: the rule each part of the frame takes. `diagonal` is the
/// diagonal's picture number.
fn exercised(state: &HudState, pad: Pad, world: &World, diagonal: u16) -> Vec<usize> {
    use BehaviourId as B;
    let upstream = match world.turn {
        Turn::Frame(upstream) => upstream,
        Turn::BossMet { .. } => return places([B::BossBlanksTheMap]),
    };
    let status = &upstream.status;
    let controls = upstream.controls(pad);
    let shown = &state.shown;
    let mut entries = Vec::new();
    if status.energy.now != shown.energy {
        entries.push(B::EnergyGauge);
    }
    if shown.ammo.iter().any(|(ammo, &shown)| {
        let stock = status.ammunition.of(ammo);
        stock.capacity != 0 && stock.count != shown
    }) {
        entries.push(B::AmmoCounts);
    }
    if status.reserve.mode == ReserveMode::Auto {
        entries.push(B::AutoReserveSign);
    }
    let switched = world.events.iter().find_map(|event| match event {
        HudEvent::Switched(item) => Some(*item),
        HudEvent::Click => None,
    });
    let mut selected = state.selection.item();
    if controls.can_switch {
        if controls.cancel == CancelButton::Pressed {
            entries.push(B::CancellingTheItem);
        } else if controls.select == SelectButton::Pressed {
            entries.push(B::CyclingItems);
            entries.push(B::OneShotItem);
        }
        if let Some(item) = switched {
            entries.push(B::Switching);
            selected = item;
        }
    }
    // Each of the weapons' four stores clears the selected item.
    if controls.shot_cancels {
        entries.push(B::CancellingTheItem);
        selected = None;
    }
    if upstream.collected.is_some() {
        entries.push(B::IconsAppear);
    }
    if selected != shown.highlighted {
        entries.extend([B::SelectionHighlight, B::SwitchClick]);
    }
    if state.selection.one_shot().is_some() {
        entries.push(B::OneShotFlash);
    }
    if upstream.power.mother_brain_falls {
        entries.push(B::BossBlanksTheMap);
    }
    if upstream.power.turned_on {
        entries.push(B::MiniMapStill);
    }
    if upstream.updates(state.mini_map) {
        entries.extend([B::Exploring, B::MiniMapWindow, B::HerBlink]);
        let column = upstream.her_column();
        if column <= FIRST_COLUMNS_END {
            entries.push(B::FirstColumnsShift);
        } else if column >= LAST_COLUMNS_START {
            entries.push(B::LastColumnsShift);
        }
        // The diagonal test reads her window tile as the explored branch drew it.
        if world.her_tile.is_some_and(|tile| {
            (tile.look.picture.index() & DIAGONAL_READ) == (diagonal & DIAGONAL_READ)
                && tile.colours.number() != CHARTED
        }) {
            entries.push(B::DiagonalPicture);
        }
    } else {
        entries.push(B::MiniMapStill);
    }
    places(entries)
}

#[cfg(test)]
mod tests;
