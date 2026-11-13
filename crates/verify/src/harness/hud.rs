//! The HUD as a harness domain: the HUD model (`interface_hud`) against the oracle.
//!
//! It is checked frame by frame by the decode notes' harness note
//! (`docs/behaviours/interface-hud.decode.md`, "The harness"). The HUD is downstream of everything
//! it shows and nothing of the same frame reads it back, so it is checked on its own: its State
//! (the HUD's cells, the values last shown, the selection and the one-shot item, the explored
//! cells, whether the mini-map is off) and the frame count and pad are decoded from record N,
//! every other input from record N+1 at the HUD's turn, and the model's output is compared with
//! record N+1: the whole tilemap, not values, since a stale cell is behaviour. The HUD's pictures
//! and her map's are the importer's.
//!
//! A frame runs from cut to cut: from a record whose NMI returned to the main loop's wait (cut
//! `R`) to the next such record. A record with any other cut is mid-frame (an overrun, vblank
//! with the NMI disabled, or vblank ended with the NMI still running) and folds into its frame,
//! its hits carried along, so a frame whose work straddles an overrun is paired whole.
//!
//! Record N's game state picks the step (spec §1.3): a frame of play where its routine runs main
//! gameplay, skipped by name where the frame counter didn't advance by one (a message box); the
//! load flow's start-up of the HUD and the pause screen's frames, skipped by name, so the next
//! pair seeds afresh; and a step outside play for every other state (a door's passage, a load's
//! other frames, the cinematics), which carries the HUD across a door. Before the HUD's first
//! start-up no word holds its priority bit: no HUD yet, skipped by name.
//!
//! The inputs other domains' writers make are named by their instruction hits in frame N+1: the
//! weapons' drop of the selection before the status bar (their stores `$90:BF25`, `$90:BF41`,
//! `$90:C091`, `$90:C0A6`), the item whose pickup draws its icon (`$80:99CF`, `$80:9A0E`,
//! `$80:9A1E`, `$80:9A2E`, `$80:9A3E`) and Mother Brain's fall (`$90:A7E2` with her boss id). A
//! boss met on entering its room calls `$90:A7E2` outside play: that pair is the model's boss
//! step. The other drops of the selection are decoded from the game state (her death's first
//! frame, the station exploding), and a clear no decoded writer explains is a defect. A new
//! area's explored cells are the copy Progress keeps for the area she enters in record N
//! (`$7E:CD52`), which the door's transition loads (`$80:858C`): never the compared field.
//!
//! The mini-map's turn-on is its stores' hits, but no row watches them yet, so it is also decoded
//! from `05F7`'s edge, which is the compared "mini-map off". A turn-on decoded from the edge alone
//! is the stand-in's answer from record N+1: [`HudDomain`]'s neutral world drops it, so every
//! field it reaches is unverified on that pair, never matched.
//!
//! One interim mask stands in for a writer the records don't yet decode, keyed as near it as the
//! records allow and reported as masked, never passed: her position at the mini-map's turn. Some
//! writer after the mini-map's read can move her across a screen (the pose change's placement,
//! `$91:EB88`, is one; samus-movement's step diverges on the same frames, so it isn't the only
//! one), so the window and the explored cells are masked where her pose changed, the mini-map
//! read her, and her position lies within 32 px of a screen's edge. No hit records her position
//! where the mini-map read her, so nothing decodes it yet, and the frames it hides may only fall
//! ([`POSITION_MASK_CEILING`]).
//!
//! The click and the switch aren't compared until the sound and the weapons' request streams
//! exist, so their entries are credited nothing. The null model is the HUD left as record N has
//! it.

pub mod cases;

use core::fmt;

use data::map::{MAP_BIT_BYTES, MapCells};
use data::room::Area;
use enum_map::enum_map;
use game::interface_hud_bridge::{self, HudContent};
use game::state::video::TilemapEntry;
use interface_hud::{
    Ammo, Ammunition, BehaviourId, Boss, CATALOGUE, CancelButton, Controls, Dropped, Energy,
    Equipped, FrameCount, FrameInputs, HerMap, HudEvent, HudGrid, HudState, MapCellSet,
    MiniMapInputs, MiniMapPower, Mute, NewArea, OutOfPlayInputs, Place, Reserve, ReserveMode, Room,
    Rules, SelectButton, SelectableItem, Selection, Shown, Status, Stock, Tile,
};
use oracle_record::FrameRecord;
use types::Pair;

use super::{Domain, Entry, Field, FieldSet, Hides, Interval, Mask, Uncompared};
use crate::bundle::ContentError;
use crate::cache::CutKind;
use crate::decode::DecodeError;
use crate::domain_field;
use crate::source::{self, WramBlock, WramWord};

/// What frame N+1 runs, by record N's game state (`0998`): the routine the main loop calls
/// (the decode notes' harness note).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Routine {
    /// Main gameplay (`$82:8B44`), whose `$82:8BA0` runs the HUD and whose Samus handlers run
    /// the switch and the mini-map: a frame of play (spec §1.3).
    Play,
    /// The load flow's start-up of the HUD (`$80:9A79`, from `$82:8000`): the load flow's
    /// frame, which sets the HUD's every field.
    HudStartUp,
    /// The pause screen's frames, whose `$82:90F6` runs the HUD routine without the switch or
    /// the mini-map: the pause screen's.
    Pause,
    /// Every other routine: a door's passage, a load's other frames, the cinematics: a step
    /// outside play (spec `OutOfPlay`).
    OutsidePlay,
}

impl Routine {
    /// The routine of game state `code`.
    const fn of(code: u16) -> Self {
        match code {
            // The fade in after a load (7), gameplay (8), the pause's fades (Ch, 12h), her
            // death's first frames (13h, 14h), the reserve tanks' refill (1Bh), Ceres's last
            // frames at its elevator (20h, 21h), the time running out (23h), the escape from
            // Zebes (26h) and the demos (29h, 2Ah). 24h runs only the fade.
            0x07 | 0x08 | 0x0C | 0x12..=0x14 | 0x1B | 0x20 | 0x21 | 0x23 | 0x26 | 0x29 | 0x2A => {
                Self::Play
            }
            0x06 | 0x1F | 0x28 => Self::HudStartUp,
            0x0D..=0x11 => Self::Pause,
            _ => Self::OutsidePlay,
        }
    }
}

/// Her death's first frame (game state 13h): `$82:DCCF` drops the selection after main
/// gameplay's status bar.
const DEATHS_FIRST_FRAME: u16 = 0x13;
/// The station exploding with her aboard (game state 25h): `$8B:C655` drops the selection.
const STATION_EXPLODES: u16 = 0x25;

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
/// Bank $80 log, `$80:858C` and `$80:85C6`: Progress's copy of each area's explored cells, an
/// area's 100h bytes laid out as the current area's, by the area's index.
const EXPLORED_BY_AREA: WramBlock = WramBlock {
    name: "MAP_TILES_EXPLORED_BY_AREA",
    offset: 0xCD52,
    len: 0x0800,
};

/// The current state handlers under which she can switch: the normal one (`$90:E695`) and the
/// demo's (`$90:E6C9`, the title's demos), both calling the switch (`$90:DCDD`).
const SWITCHING_CURRENT: [u16; 2] = [0xE695, 0xE6C9];
/// The poses facing the screen, in which she can't switch (`$90:DCE0`).
const FACING_FORWARD: [u16; 2] = [0x00, 0x9B];
/// The grapple beam's function while it is put away.
const GRAPPLE_INACTIVE: u16 = 0xC4F0;
/// The movement types that mute the click: spin jumping, wall jumping.
const SPIN_JUMPING: u8 = 0x03;
const WALL_JUMPING: u8 = 0x14;
/// The new state handlers that run the mini-map (normal, title demo, appearance, locked, elevator,
/// Ceres) and those that don't (debug, intro demo, station, drained, gunship, her death).
const MINI_MAP_RUNS: [u16; 6] = [0xE725, 0xE7F5, 0xE86A, 0xE8DC, 0xE8EC, 0xE8AA];
const MINI_MAP_STILL: [u16; 6] = [0xE7D2, 0xE833, 0xE8D6, 0xE8D9, 0xE902, 0xE8CD];
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
/// The weapons' stores that drop the selection before the status bar, after a shot of the
/// one-shot item or of the last of the selected ammunition: `$90:BF25` and `$90:C091` clear the
/// selected and the one-shot item, `$90:BF41` and `$90:C0A6` the selected item when there is no
/// one-shot item. The decode notes list six more stores no row watches yet.
const WEAPONS_DROP: [u32; 4] = [0x90_BF25, 0x90_BF41, 0x90_C091, 0x90_C0A6];
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
const ROOM_BOSSES: [(u16, Boss); 6] = [
    (0x01, Boss::RidleyInCeres),
    (0x03, Boss::Kraid),
    (0x05, Boss::RidleyInNorfair),
    (0x06, Boss::Crocomire),
    (0x07, Boss::Phantoon),
    (0x08, Boss::Draygon),
];
/// The interim mask's reason, by which its frames are counted.
pub const POSITION_MASK: &str = "interim: her pose changed within 32 px of a screen's edge and the mini-map read her, so a writer after its read (the pose change's placement $91:EB88 among them) may have moved her across; no hit records where the mini-map read her";
/// The most frames the interim mask may hide over every cached row: the count may only fall,
/// with the change that lowers it, until a recording of her position at the mini-map's read
/// retires the mask.
pub const POSITION_MASK_CEILING: u64 = 1_045;

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
        B::OutOfPlay => STATUS_BAR
            .union(SHOWN)
            .union(SELECTED)
            .union(ONE_SHOT)
            .union(WINDOW_TILES)
            .union(EXPLORED)
            .union(MINI_MAP_OFF),
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
    /// The selection or the one-shot item cleared with no writer decoded for it: a drop whose
    /// store no row watches yet (the rainbow beam, the icon cancel option's), a defect.
    UndecodedDrop,
    /// The load flow's start-up of the HUD (`$80:9A79`) in this game state: the load flow sets
    /// every field.
    HudStartUp(u16),
    /// The pause screen's frame in this game state: the pause screen sets the HUD.
    Paused(u16),
    /// No HUD yet: no word of it holds its priority bit, before the HUD's first start-up
    /// (power-on, the title, the opening, the flight to Ceres).
    NoHudYet,
    /// The frame counter didn't advance by one in a frame of play: no main processing ran (a
    /// message box), or more than one frame.
    Frames(u16),
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
            Self::UndecodedDrop => f.write_str("the selection cleared by no decoded writer"),
            Self::HudStartUp(code) => {
                write!(
                    f,
                    "game state {code:#04X}: the load flow's start-up of the HUD"
                )
            }
            Self::Paused(code) => write!(f, "game state {code:#04X}: the pause screen's"),
            Self::NoHudYet => f.write_str("no HUD yet"),
            Self::Frames(count) => write!(f, "{count} frames ran, not one"),
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

/// The HUD's cells `record` holds, or [`Skip::NoHudYet`] when no word holds its priority bit.
fn grid(record: &FrameRecord) -> Result<HudGrid, Skip> {
    let words = source::HUD_TILEMAP.read(&record.wram)?.as_chunks::<2>().0;
    if words
        .iter()
        .all(|&bytes| u16::from_le_bytes(bytes) & PRIORITY == 0)
    {
        return Err(Skip::NoHudYet);
    }
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

/// The explored cells in `bytes`, laid out as the current area's, from `source`.
fn cells(bytes: Option<&[u8]>, source: &'static str) -> Result<MapCellSet, Skip> {
    let bitmap: &[u8; MAP_BIT_BYTES] = bytes
        .and_then(<[u8]>::first_chunk)
        .ok_or(Skip::Unreadable(DecodeError::WramTooShort { source }))?;
    Ok(interface_hud_bridge::explored(&MapCells::from_bitmap(
        bitmap,
    )))
}

/// The explored cells of her area `record` holds.
fn explored(record: &FrameRecord) -> Result<MapCellSet, Skip> {
    let bytes = source::MAP_TILES_EXPLORED.read(&record.wram)?;
    cells(Some(bytes), source::MAP_TILES_EXPLORED.name)
}

/// The explored cells Progress keeps for `area` in `record`.
fn explored_kept(record: &FrameRecord, area: Area) -> Result<MapCellSet, Skip> {
    let start = usize::from(u8::from(area)).saturating_mul(MAP_BIT_BYTES);
    let bytes = EXPLORED_BY_AREA.read(&record.wram)?.get(start..);
    cells(bytes, EXPLORED_BY_AREA.name)
}

/// The HUD's State `record` holds (the decode notes' State).
fn hud(record: &FrameRecord) -> Result<HudState, Skip> {
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
        explored: explored(record)?,
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
    dropped_before: Dropped,
    dropped_after: Dropped,
    collected: Option<SelectableItem>,
    power: PowerChanges,
    position: Pair<u16>,
    room: Room,
    map: MapKey,
}

/// Whether the mini-map turned on in a frame, and how the harness knows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnOn {
    /// It didn't.
    No,
    /// One of its stores was hit.
    Watched,
    /// No store was hit, but `05F7` went from nonzero at N to 0 at N+1: read from the compared
    /// field, so the fields it reaches are unverified.
    AtTheEdge,
}

impl TurnOn {
    /// Whether the mini-map turned on.
    const fn on(self) -> bool {
        !matches!(self, Self::No)
    }
}

/// What turns the mini-map on or off in a frame, before its turn: other domains' writers.
#[derive(Clone, Copy, Debug)]
pub struct PowerChanges {
    /// One of the stores of 0 to `05F7` ran ([`turned_on`]).
    turned_on: TurnOn,
    /// Mother Brain's first form fell.
    mother_brain_falls: bool,
}

/// What a step outside play reads from its records (spec `OutOfPlay`).
#[derive(Clone, Debug)]
pub struct Outside {
    /// The mini-map turned on in the step.
    turned_on: TurnOn,
    /// The explored cells Progress keeps for the area the step leads into, or `None` when she
    /// stays in hers.
    new_area: Option<Box<MapCellSet>>,
    /// What another domain dropped in the step.
    dropped: Dropped,
}

/// What a record pair is to the HUD.
#[derive(Clone, Debug)]
pub enum Turn {
    /// A frame of play.
    Frame(Upstream),
    /// A step outside play.
    OutOfPlay(Outside),
    /// A step outside play in which a boss was met on entering its room.
    BossMet {
        /// The boss.
        boss: Boss,
        /// Her room's top-left cell on her map.
        room: Place,
        /// Her area, whose map the HUD draws.
        area: Area,
    },
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
    InItsRoom(Boss),
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

/// Whether the mini-map turned on in the frame from `from` to `next`: one of the stores of 0 to
/// `05F7` ran. No row watches them yet, so it is decoded from `05F7` too: nonzero at N and 0 at
/// N+1, which only those stores make (only `$90:A7E2` writes it nonzero). That misses a turn-on
/// a boss turns off again in the same frame, which no vanilla frame holds.
fn turned_on(from: &FrameRecord, next: &FrameRecord) -> Result<TurnOn, Skip> {
    let off = |record| Ok::<_, Skip>(word(record, source::DEBUG_DISABLE_MINIMAP)? != 0);
    Ok(if TURNS_THE_MINI_MAP_ON.iter().any(|&pc| ran(next, pc)) {
        TurnOn::Watched
    } else if off(from)? && !off(next)? {
        TurnOn::AtTheEdge
    } else {
        TurnOn::No
    })
}

/// What another domain drops after the status bar in a frame of play run from game state
/// `code`: her death's first frame drops the selection. The other drops in play (the rainbow
/// beam, an elevator's start with the icon cancel option on) aren't watched, and their clears are
/// defects ([`Skip::UndecodedDrop`]).
const fn dropped_after_status_bar(code: u16) -> Dropped {
    if code == DEATHS_FIRST_FRAME {
        Dropped::Selection
    } else {
        Dropped::Nothing
    }
}

/// Every input but the pad of the frame of play from `from` to `next`, at the HUD's turn in
/// `next`; Mother Brain falls when `mother_brain_falls`.
fn upstream(
    from: &FrameRecord,
    next: &FrameRecord,
    mother_brain_falls: bool,
) -> Result<Upstream, Skip> {
    Ok(Upstream {
        status: status(next)?,
        select: word(next, ITEM_SELECT_BINDING)?,
        cancel: word(next, ITEM_CANCEL_BINDING)?,
        can_switch: SWITCHING_CURRENT.contains(&word(next, CURRENT_STATE_HANDLER)?)
            && !FACING_FORWARD.contains(&word(next, source::SAMUS_POSE)?),
        muted_by: muted_by(next)?,
        runs: runs(next)?,
        dropped_before: if WEAPONS_DROP.iter().any(|&pc| ran(next, pc)) {
            Dropped::Selection
        } else {
            Dropped::Nothing
        },
        dropped_after: dropped_after_status_bar(game_state(from)?),
        collected: next.hits.iter().find_map(|hit| {
            PICKUPS
                .iter()
                .find(|&&(pc, _)| pc == hit.pc)
                .map(|&(_, item)| item)
        }),
        power: PowerChanges {
            turned_on: turned_on(from, next)?,
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
    /// The buttons under the bindings.
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
            muted_by: self.muted_by,
        }
    }

    /// The model's inputs over `map`.
    const fn model<'map>(&self, pad: Pad, map: &'map HerMap) -> FrameInputs<'map> {
        FrameInputs {
            frame: pad.frame,
            status: self.status,
            controls: self.controls(pad),
            dropped_before_status_bar: self.dropped_before,
            dropped_after_status_bar: self.dropped_after,
            collected: self.collected,
            mother_brain_falls: self.power.mother_brain_falls,
            mini_map: MiniMapInputs {
                runs: self.runs,
                turned_on: self.power.turned_on.on(),
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
            || (matches!(before, MiniMapPower::Off) && !self.power.turned_on.on())
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

/// Whether the selection or the one-shot item cleared from `from` to `next` with none of the
/// writers the harness decodes behind it (`explained`): a drop whose store isn't watched.
fn undecoded_drop(from: &FrameRecord, next: &FrameRecord, explained: bool) -> Result<bool, Skip> {
    let (before, after) = (selection(from)?, selection(next)?);
    let cleared = (before.item().is_some() && after.item().is_none())
        || (before.one_shot().is_some() && after.one_shot().is_none());
    Ok(cleared && !explained)
}

/// The frame of play from `from` to `next`, in whose frame `called` blanked the mini-map.
fn frame_of_play(
    from: &FrameRecord,
    next: &FrameRecord,
    called: Option<BossCalled>,
) -> Result<Upstream, Skip> {
    let frames = word(next, source::NMI_FRAME_COUNTER_WORD)?
        .wrapping_sub(word(from, source::NMI_FRAME_COUNTER_WORD)?);
    if frames != 1 {
        return Err(Skip::Frames(frames));
    }
    let mother_brain_falls = match called {
        None => false,
        Some(BossCalled::MotherBrain) => true,
        Some(BossCalled::InItsRoom(_)) => return Err(Skip::BossInPlay(word(next, BOSS_ID)?)),
    };
    let upstream = upstream(from, next, mother_brain_falls)?;
    let controls = upstream.controls(pad(from)?);
    let switched = controls.can_switch
        && (controls.select == SelectButton::Pressed
            || controls.cancel != CancelButton::NotPressed);
    let explained = switched
        || upstream.dropped_before != Dropped::Nothing
        || upstream.dropped_after != Dropped::Nothing;
    if undecoded_drop(from, next, explained)? {
        return Err(Skip::UndecodedDrop);
    }
    Ok(upstream)
}

/// The step outside play from `from`, in game state `code`, to `next`, in whose frame `called`
/// blanked the mini-map.
fn outside_play(
    from: &FrameRecord,
    next: &FrameRecord,
    code: u16,
    called: Option<BossCalled>,
) -> Result<Turn, Skip> {
    match called {
        Some(BossCalled::InItsRoom(boss)) => {
            return Ok(Turn::BossMet {
                boss,
                room: room_cell(next)?,
                area: area(next)?,
            });
        }
        // Mother Brain's fall is her first form's, in gameplay.
        Some(BossCalled::MotherBrain) => return Err(unnamed(BOSS_ID.name, MOTHER_BRAIN)),
        None => {}
    }
    let dropped = if code == STATION_EXPLODES {
        Dropped::Selection
    } else {
        Dropped::Nothing
    };
    if undecoded_drop(from, next, dropped != Dropped::Nothing)? {
        return Err(Skip::UndecodedDrop);
    }
    let entered = area(next)?;
    let new_area = if area(from)? == entered {
        None
    } else {
        Some(Box::new(explored_kept(from, entered)?))
    };
    Ok(Turn::OutOfPlay(Outside {
        turned_on: turned_on(from, next)?,
        new_area,
        dropped,
    }))
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

    /// Record N's game state picks the routine: the load flow's start-up of the HUD and the
    /// pause screen's frames are other domains', skipped by name, and the next frame seeds
    /// afresh.
    fn decode(&self, from: &FrameRecord, _interval: Interval) -> Result<(HudState, Pad), Skip> {
        let code = game_state(from)?;
        match Routine::of(code) {
            Routine::HudStartUp => Err(Skip::HudStartUp(code)),
            Routine::Pause => Err(Skip::Paused(code)),
            Routine::Play | Routine::OutsidePlay => Ok((hud(from)?, pad(from)?)),
        }
    }

    fn world(&self, from: &FrameRecord, next: &FrameRecord) -> Result<World, Skip> {
        let code = game_state(from)?;
        let called = boss_called(next)?;
        let turn = match Routine::of(code) {
            Routine::HudStartUp => return Err(Skip::HudStartUp(code)),
            Routine::Pause => return Err(Skip::Paused(code)),
            Routine::Play => Turn::Frame(frame_of_play(from, next, called)?),
            Routine::OutsidePlay => outside_play(from, next, code, called)?,
        };
        Ok(World {
            turn,
            events: Vec::new(),
            her_tile: None,
        })
    }

    fn masks(&self, from: &FrameRecord, next: &FrameRecord, _pad: &Pad) -> Result<Vec<Mask>, Skip> {
        let mut masks = Vec::new();
        if Routine::of(game_state(from)?) != Routine::Play {
            return Ok(masks);
        }
        let falls = boss_called(next)? == Some(BossCalled::MotherBrain);
        let upstream = upstream(from, next, falls)?;
        let read_her = upstream.runs && !upstream.off(hud(from)?.mini_map);
        let near_edge =
            near_screen_edge(upstream.position.x) || near_screen_edge(upstream.position.y);
        if read_her
            && near_edge
            && word(from, source::SAMUS_POSE)? != word(next, source::SAMUS_POSE)?
        {
            masks.push(Mask {
                reason: POSITION_MASK,
                fields: WINDOW_TILES.union(EXPLORED),
                hides: Hides::AnotherWriter,
            });
        }
        Ok(masks)
    }

    /// The world without a turn-on decoded from `05F7`'s edge alone, which is the compared
    /// "mini-map off" of record N+1: every field it reaches is unverified on the pair.
    fn neutral(&self, world: &World) -> Option<World> {
        let mut neutral = world.clone();
        let turned_on = match &mut neutral.turn {
            Turn::Frame(upstream) => &mut upstream.power.turned_on,
            Turn::OutOfPlay(outside) => &mut outside.turned_on,
            Turn::BossMet { .. } => return None,
        };
        if *turned_on != TurnOn::AtTheEdge {
            return None;
        }
        *turned_on = TurnOn::No;
        Some(neutral)
    }

    fn step(&self, state: &mut HudState, pad: &Pad, world: &mut World) {
        world.events = match &world.turn {
            Turn::Frame(upstream) => {
                let (map, pictures) = self.content.of(upstream.map.area);
                state.frame_of_play(&self.rules, pictures, &upstream.model(*pad, map))
            }
            Turn::OutOfPlay(outside) => {
                state.out_of_play(&OutOfPlayInputs {
                    mini_map_turned_on: outside.turned_on.on(),
                    new_area: outside
                        .new_area
                        .as_deref()
                        .map_or(NewArea::SameArea, NewArea::Entered),
                    dropped: outside.dropped,
                });
                Vec::new()
            }
            &Turn::BossMet { boss, room, area } => {
                let (_, pictures) = self.content.of(area);
                state.boss_blanks_the_map(&self.rules, pictures, boss, room);
                Vec::new()
            }
        };
        world.her_tile = state.grid.get(HER_TILE);
    }

    fn exercised(&self, state: &HudState, pad: &Pad, world: &World) -> Vec<usize> {
        let diagonal = self.content.pictures().diagonal.index();
        exercised(state, *pad, world, diagonal)
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
            | Skip::BossInPlay(_)
            | Skip::UndecodedDrop => Uncompared::Defect(skip.to_string()),
            Skip::HudStartUp(_) | Skip::Paused(_) | Skip::NoHudYet | Skip::Frames(_) => {
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
    let upstream = match &world.turn {
        Turn::Frame(upstream) => *upstream,
        Turn::OutOfPlay(_) => return places([B::OutOfPlay]),
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
    // Another domain's drop; the highlight shows it only when it lands before the status bar.
    if upstream.dropped_before != Dropped::Nothing || upstream.dropped_after != Dropped::Nothing {
        entries.push(B::CancellingTheItem);
    }
    if upstream.dropped_before == Dropped::Selection {
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
    if upstream.power.turned_on.on() {
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
