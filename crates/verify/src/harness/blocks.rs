//! Blocks as a harness domain: the blocks model (`clean_model_blocks`) against the oracle.
//!
//! It is checked frame by frame by the decode notes' harness note
//! (`docs/behaviours/blocks.decode.md`, "The harness"). A frame runs from one commit to the next
//! (records where the frame counter moved); a lag record between them folds into the frame, its
//! cell changes already in the next commit's work RAM and its hits carried to it. Record N's
//! State is decoded with the game's reactions, imported from the ROM (`data::blocks`): the
//! room's cells, every place, the screens. The model's output is compared with record N+1 on
//! each place, the screens and the cells within the frame's reach: the cells of this part's
//! places at either record, those their reactions' patterns cover, and the cells the frame's
//! requests name, chosen by position, never by value. Outside the reach the model's cells must
//! be record N's, since nothing of this part writes there.
//!
//! The requests are heard from the frame's watched instruction hits, in the order the CPU ran
//! them: each reaction's setup is matched, by family, to the place record N+1 shows a new
//! reaction of this part in, which gives its cell (a crumble's touch from above; a shot block's
//! hit by a beam, a power bomb block's by a power bomb's blast, a super missile block's by a
//! super missile, a bomb block's by the weapon its branch names, a reveal's by a bomb's blast);
//! a setup with no new place is a start refused, which asks nothing. Each scroll touch is a
//! touch of the scroll cell nearest her not yet touched in the frame, at her centre where the
//! centre's inside spawn ran before it; a repeat touches the first one touched. A start no hit
//! decides is stood in for from record N+1, and every field it reaches is unverified. How she
//! moves is read from record N+1, and so is whether the run is paused.
//!
//! The other parts' places are another part's: a place one of them takes or frees between the
//! records is copied from record N+1 before the requests, so the free place a start takes is the
//! one the original chose; the places it copies are unverified.
//!
//! The null model is the blocks left as record N has them.

use core::cell::RefCell;
use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use clean_model_blocks::{
    BehaviourId, CATALOGUE, CellKind, Frames, Grid, Mark, Place, Places, Reacting, ReactionName,
    Request, Run, Running, ScreenChange, ScrollReaction, State, Step, Tile, Touched,
};
use clean_types::{Cell, Family, Heading, HitBy, Look, Motion, Pair, Point, ScreenKind, Script};
use clean_types::{Set, Touch, Way};
use data::blocks::{ReactionId, Reactions as Imported, Runs};
use data::level::{Block, ScrollArea};
use data::rom::{Lorom, RomAddress};
use data::room::Area;
use game::blocks_world::{BlockContent, Covering};
use oracle_record::FrameRecord;

use super::{Domain, Entry, Field, FieldSet, Interval, Uncompared};
use crate::decode::{DecodeError, LevelSources};
use crate::domain_field;
use crate::source::{self, WramWord};

/// A work RAM word the harness reads that `source` doesn't name.
const fn wram(name: &'static str, offset: u16) -> WramWord {
    WramWord { name, offset }
}

/// Bank $84 log, `$84:85BA`: the PLM handler runs while bit 15 is set.
const PLMS_ENABLED: WramWord = wram("PLM_ENABLE_FLAG", 0x1C23);
const PLMS_RUN: u16 = 0x8000;
/// Bank $82 log, `$82:DED0`: the room's door list, which a room's load sets.
const DOOR_LIST: WramWord = wram("DOOR_LIST_POINTER", 0x07B5);
/// Bank $90 log: the speed booster's state; `0400h` in its high nybbles is full speed.
const SPEED_BOOST: WramWord = wram("SPEED_BOOST_COUNTER", 0x0B3E);
const SPEED_BITS: u16 = 0x0F00;
const FULL_SPEED: u16 = 0x0400;
/// Her shinesparking poses and the screw attack's spins (bank $91 log, the pose table).
const SHINESPARKING: core::ops::RangeInclusive<u16> = 0xC9..=0xCE;
const SCREW_ATTACKING: [u16; 2] = [0x81, 0x82];
/// Main gameplay (`$0998`).
const GAMEPLAY: u16 = 8;
/// The PLM slots.
const PLACES: u16 = 40;
/// The scroll PLM (`$84:B703`); a respawn word's type bits; a scroll PLM's touched bit.
const SCROLL_PLM: u16 = 0xB703;
const TYPE_SHIFT: u16 = 12;
const TOUCHED: u16 = 0x8000;
/// The bank a scroll PLM's argument points into.
const ROOM_BANK: u8 = 0x8F;
/// A cell's pixels, as a shift; a screen's cells.
const CELL_SHIFT: u16 = 4;
const SCREEN_CELLS: u16 = 16;

/// The watched setups (bank $84 log) and what each starts.
const SETUPS: [(u32, Started); 9] = [
    (0x84_CE37, Started::Family(Family::Crumble)),
    (0x84_CDEA, Started::Family(Family::Speed)),
    (0x84_CE83, Started::BombTouched),
    (0x84_CEDA, Started::BombHit),
    (0x84_B3C1, Started::Family(Family::Shot)),
    (0x84_CE6B, Started::Family(Family::Shot)),
    (0x84_CF67, Started::Family(Family::SuperMissile)),
    (0x84_CF2E, Started::Family(Family::PowerBomb)),
    (0x84_CFA0, Started::Reveal),
];
/// The bomb block hit setup's branches: a bomb's (`$84:CF0C`) and a power bomb's (`$84:CEF4`).
const BOMB_BRANCHES: [(u32, HitBy); 2] = [(0x84_CF0C, HitBy::Bomb), (0x84_CEF4, HitBy::PowerBomb)];
/// The scroll block touch's setup (`$84:B393`), its trigger (`$84:B3AD`), the centre's inside
/// spawn (`$94:995E`) and a spawn refused for want of a free slot (`$84:84F9`).
const SCROLL_TOUCH: u32 = 0x84_B393;
const SCROLL_TRIGGER: u32 = 0x84_B3AD;
const CENTRE: u32 = 0x94_995E;
const REFUSED: u32 = 0x84_84F9;

/// What a setup starts, as the record shows its place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Started {
    /// A family's `broken` script.
    Family(Family),
    /// A bomb block's `touched` script.
    BombTouched,
    /// A bomb block's `broken` script.
    BombHit,
    /// A `revealed` script.
    Reveal,
}

impl Started {
    /// What the place `running` of `family` started as.
    const fn of(family: Family, script: Script) -> Self {
        match (family, script) {
            (Family::Bomb, Script::Touched) => Self::BombTouched,
            (Family::Bomb, Script::Broken) => Self::BombHit,
            (_, Script::Revealed) => Self::Reveal,
            (family, Script::Broken | Script::Touched) => Self::Family(family),
        }
    }
}

/// The declared fields by their places in [`BlocksDomain`]'s `FIELDS`.
const PLACES_FIELD: FieldSet = FieldSet::single(0);
const CELLS: FieldSet = FieldSet::single(1);
const SCREENS: FieldSet = FieldSet::single(2);
const OUTSIDE: FieldSet = FieldSet::single(3);

/// The declared fields a catalogue entry's output reaches. The answers to her motion, the
/// redraws and the sounds are events no declared field holds: credited nothing.
const fn shown_by(id: BehaviourId) -> FieldSet {
    use BehaviourId as B;
    match id {
        B::Places | B::BlocksMarkedNothing => PLACES_FIELD,
        B::PatternCells => CELLS.union(OUTSIDE),
        B::ScrollCells => SCREENS.union(PLACES_FIELD),
        B::TheScriptsRun
        | B::Regrowing
        | B::RoomStart
        | B::CrumbleBlocks
        | B::SpeedBlocks
        | B::BombBlocks
        | B::ShotBlocks
        | B::SuperMissileBlocks
        | B::PowerBombBlocks
        | B::BombsReveal => PLACES_FIELD.union(CELLS).union(OUTSIDE),
        B::AnswersToHerMotion
        | B::FullPlacesLetHerThrough
        | B::WhatTheScreenRedraws
        | B::Sounds => FieldSet::EMPTY,
    }
}

/// Why a frame isn't compared.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Skip {
    /// Not main gameplay at either record.
    NotGameplay,
    /// The frame counter moved by other than one between the commits.
    Frames(u16),
    /// The room changed, or its level reloaded.
    RoomChanged,
    /// A source couldn't be read, or holds a value the spec names nothing for.
    Unreadable(DecodeError),
    /// A reacting cell the importer refuses.
    Refused(Cell),
    /// A place the decoder can't read: its step, its wait or its reaction.
    Place(u16, &'static str),
    /// A scroll PLM's change list isn't in the ROM.
    ScrollList(u16),
}

impl fmt::Display for Skip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotGameplay => write!(f, "not main gameplay at either record"),
            Self::Frames(moved) => write!(f, "the frame counter moved by {moved}"),
            Self::RoomChanged => write!(f, "the room changed or its level reloaded"),
            Self::Unreadable(error) => write!(f, "unreadable: {error:?}"),
            Self::Refused(cell) => write!(f, "a cell the importer refuses at {cell:?}"),
            Self::Place(slot, what) => write!(f, "place {slot}: {what}"),
            Self::ScrollList(argument) => write!(f, "scroll list {argument:#06X}"),
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

/// Where a request comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// Heard from the frame's watched instruction hits.
    Heard,
    /// The stand-in's, decoded from what record N+1 shows: copied.
    Copied,
}

/// What the frame gives the blocks besides their State.
#[derive(Clone)]
pub struct Upstream {
    /// The requests in the frame's order, each with where it comes from.
    requests: Vec<(Request, Source)>,
    /// The places another part took or freed, copied from record N+1.
    copied: Vec<(usize, Place)>,
    /// How she moves.
    moving: Set<Motion>,
    /// Whether the run runs.
    run: Run,
    /// The catalogue entries the step exercised; filled by the step.
    did: Vec<BehaviourId>,
}

/// A place as it is compared: the scroll reaction's changes are the room's, and another part's
/// place is only taken.
#[derive(Clone, Debug, PartialEq, Eq)]
enum PlaceView {
    Free,
    OtherPart,
    HeldUntilRun,
    Scroll(Cell, Touched),
    Running(ReactionName, Script, Cell, usize, Frames, Option<Reacting>),
}

/// A cell as it is compared: its mark where its kind reads one.
#[derive(Clone, Debug, PartialEq, Eq)]
struct CellView {
    kind: CellKind,
    look: Look,
    mark: Option<Mark>,
}

/// The cells of the frame's reach, by cell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cells(BTreeMap<(u16, u16), Option<CellView>>);

/// The declared fields: every place, the cells within the frame's reach, the screens, and the
/// cells outside the reach that differ from record N's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declared {
    places: Vec<PlaceView>,
    cells: Cells,
    screens: Vec<Vec<ScreenKind>>,
    /// The model writes no cell outside the frame's reach, so the oracle's is always empty: a
    /// stray write shows here on every frame it happens.
    outside: Vec<(u16, u16)>,
}

fn place_view(place: &Place) -> PlaceView {
    match place {
        Place::Free => PlaceView::Free,
        Place::OtherPart => PlaceView::OtherPart,
        Place::HeldUntilRun => PlaceView::HeldUntilRun,
        Place::Scroll(scroll) => PlaceView::Scroll(scroll.cell, scroll.touched),
        Place::Running(running) => PlaceView::Running(
            running.reaction.clone(),
            running.script,
            running.cell,
            running.next,
            running.wait,
            running.grows_back_as,
        ),
    }
}

fn cell_view(tile: &Tile) -> CellView {
    let marked = matches!(
        tile.kind,
        CellKind::Block | CellKind::ReactingAir | CellKind::StandInAcross | CellKind::StandInDown
    );
    CellView {
        kind: tile.kind,
        look: tile.look,
        mark: marked.then(|| tile.mark.clone()),
    }
}

/// A record's blocks: its State, and its places' cells and the cells their patterns cover.
struct Decoded {
    state: State,
    reach: BTreeSet<(u16, u16)>,
}

/// The blocks domain over the ROM, which holds the reactions and the scroll lists.
pub struct BlocksDomain {
    image: Vec<u8>,
    content: BlockContent,
    /// The scroll lists read, by argument and the room's screens across.
    lists: RefCell<BTreeMap<(u16, u16), Vec<ScreenChange>>>,
    /// The cells compared on the frame in hand: chosen in `world` from both records' places and
    /// the requests, by position.
    reach: RefCell<BTreeSet<(u16, u16)>>,
    /// Record N's cells on the frame in hand, which the model's cells outside the reach must
    /// equal.
    start: RefCell<Vec<Vec<Tile>>>,
}

/// The level's layer 1 sources.
const LEVEL: LevelSources = LevelSources {
    width: source::ROOM_WIDTH_IN_BLOCKS,
    height: source::ROOM_HEIGHT_IN_BLOCKS,
    blocks: source::LEVEL_DATA,
    bts: source::BTS,
    scrolls: source::SCROLLS,
    state: source::ROOMDEFROOMSTATE_PTR,
    layer2: source::CUSTOM_BACKGROUND,
};

/// The record's area.
fn area(record: &FrameRecord) -> Result<Area, Skip> {
    let index = word(record, source::AREA_INDEX)?;
    u8::try_from(index)
        .ok()
        .and_then(|index| Area::try_from(index).ok())
        .ok_or(Skip::Unreadable(DecodeError::Unrepresentable {
            source: source::AREA_INDEX.name,
            value: index,
        }))
}

/// The cell at level index `index` of a room `width` across.
fn cell(index: u16, width: u16) -> Cell {
    Cell {
        x: index.checked_rem(width).unwrap_or(index),
        y: index.checked_div(width).unwrap_or(0),
    }
}

/// `cell` moved by `by`.
fn moved(cell: Cell, by: Pair<i16>) -> Option<Cell> {
    Some(Cell {
        x: cell.x.checked_add_signed(by.x)?,
        y: cell.y.checked_add_signed(by.y)?,
    })
}

impl BlocksDomain {
    /// The domain over the ROM `image`.
    ///
    /// # Errors
    ///
    /// The importer's refusal of the image's reactions.
    pub fn load(image: Vec<u8>) -> Result<Self, String> {
        let imported = Imported::load(Lorom::new(&image))
            .map_err(|error| format!("the reactions: {error:?}"))?;
        let content =
            BlockContent::import(&imported).map_err(|id| format!("reaction {id:?}: a wait"))?;
        Ok(Self {
            image,
            content,
            lists: RefCell::new(BTreeMap::new()),
            reach: RefCell::new(BTreeSet::new()),
            start: RefCell::new(Vec::new()),
        })
    }

    /// The changes of the scroll list at `argument` in a room `across` screens wide.
    fn changes(&self, argument: u16, across: u16) -> Result<Vec<ScreenChange>, Skip> {
        if let Some(changes) = self.lists.borrow().get(&(argument, across)) {
            return Ok(changes.clone());
        }
        let list = data::plm::scroll_changes(
            Lorom::new(&self.image),
            RomAddress::new(ROOM_BANK, argument),
        )
        .map_err(|_| Skip::ScrollList(argument))?;
        let changes: Vec<ScreenChange> = list
            .iter()
            .map(|change| {
                let screen = u16::from(change.screen);
                ScreenChange {
                    screen: Pair {
                        x: screen.checked_rem(across).unwrap_or(screen),
                        y: screen.checked_div(across).unwrap_or(0),
                    },
                    becomes: kind(change.area),
                }
            })
            .collect();
        self.lists
            .borrow_mut()
            .insert((argument, across), changes.clone());
        Ok(changes)
    }

    /// The places `record` holds, each with its cell's level index where it sits on one.
    fn places(&self, record: &FrameRecord, blocks: &[Block], area: Area) -> Result<Places, Skip> {
        let width = word(record, source::ROOM_WIDTH_IN_BLOCKS)?;
        let across = width / SCREEN_CELLS;
        let imported = self.content.imported();
        let mut places = Places::default();
        for (slot, place) in (0..PLACES).zip(places.0.iter_mut()) {
            let read = |array: source::WramArray| -> Result<u16, Skip> {
                Ok(array.word(slot)?.read(&record.wram)?)
            };
            let id = read(source::PLM_HEADER_PTR)?;
            let at = cell(read(source::PLM_BLOCK_INDICES)? / 2, width);
            *place = match id {
                0 => Place::Free,
                SCROLL_PLM => Place::Scroll(ScrollReaction {
                    cell: at,
                    changes: self.changes(read(source::PLM_ROOM_ARGUMENTS)?, across)?,
                    touched: if read(source::PLM_VARIABLE)? & TOUCHED == 0 {
                        Touched::No
                    } else {
                        Touched::Yes
                    },
                }),
                id => match imported.runs(id) {
                    Some(Runs::Hold) => Place::HeldUntilRun,
                    Some(Runs::Script(reaction, script)) => {
                        self.running(record, slot, at, reaction, script)?
                    }
                    Some(Runs::RevealedOfMark) => {
                        let index = usize::from(at.y.wrapping_mul(width).wrapping_add(at.x));
                        let block = blocks
                            .get(index)
                            .copied()
                            .ok_or(Skip::Place(slot, "a reveal past the level"))?;
                        let mark = self.content.tile(block, area, None).map(|tile| tile.mark);
                        let Some(Mark::Reaction(name)) = mark else {
                            return Err(Skip::Place(slot, "a reveal on a cell of no reaction"));
                        };
                        let reaction = self
                            .content
                            .id(&name)
                            .ok_or(Skip::Place(slot, "a reveal's reaction"))?;
                        self.running(record, slot, at, reaction, Script::Revealed)?
                    }
                    None => Place::OtherPart,
                },
            };
        }
        Ok(places)
    }

    /// The reaction running `script` of `reaction` in slot `slot`, on `at`.
    fn running(
        &self,
        record: &FrameRecord,
        slot: u16,
        at: Cell,
        reaction: ReactionId,
        script: Script,
    ) -> Result<Place, Skip> {
        let read = |array: source::WramArray| -> Result<u16, Skip> {
            Ok(array.word(slot)?.read(&record.wram)?)
        };
        let imported = self.content.imported();
        let pointer = read(source::PLM_INSTR_LIST_PTRS)?;
        let next = imported
            .step_at(reaction, script, pointer)
            .ok_or(Skip::Place(slot, "a list pointer at no step"))?;
        let regrows = imported
            .get(reaction)
            .and_then(|imported| imported.script(script).get(next..))
            .is_some_and(|ahead| ahead.contains(&data::blocks::Step::Regrow));
        let grows_back_as = if regrows {
            Some(match read(source::PLM_VARIABLE)? >> TYPE_SHIFT {
                0xB | 0xC | 0xF => Reacting::Block,
                0x3 | 0x4 | 0x7 => Reacting::Air,
                _ => return Err(Skip::Place(slot, "a respawn block of no reacting kind")),
            })
        } else {
            None
        };
        let (name, _) = self
            .content
            .named(reaction)
            .ok_or(Skip::Place(slot, "a reaction the content doesn't hold"))?;
        Ok(Place::Running(Running {
            reaction: name.clone(),
            script,
            cell: at,
            next,
            wait: Frames::new(read(source::PLM_INSTRUCTION_TIMER)?)
                .ok_or(Skip::Place(slot, "a wait past the model's"))?,
            grows_back_as,
        }))
    }

    /// The cells the running reactions of `places` sit on and cover, each with what covers it.
    fn coverings(&self, places: &Places) -> Vec<(Cell, Covering)> {
        let mut own = Vec::new();
        let mut others = Vec::new();
        for place in &places.0 {
            let Place::Running(running) = place else {
                continue;
            };
            let Some(id) = self.content.id(&running.reaction) else {
                continue;
            };
            if let Some((_, family)) = self.content.named(id) {
                own.push((running.cell, Covering::Own(family)));
            }
            others.extend(
                self.content
                    .covered(id)
                    .iter()
                    .filter_map(|&by| moved(running.cell, by))
                    .map(|at| (at, Covering::Other)),
            );
        }
        own.extend(others);
        own
    }

    /// The blocks `record` holds.
    fn decode_record(&self, record: &FrameRecord) -> Result<Decoded, Skip> {
        let width = word(record, source::ROOM_WIDTH_IN_BLOCKS)?;
        let height = word(record, source::ROOM_HEIGHT_IN_BLOCKS)?;
        let area = area(record)?;
        let blocks = LEVEL.layer1(&record.wram)?;
        let places = self.places(record, &blocks, area)?;
        let coverings = self.coverings(&places);
        let mut reach = BTreeSet::new();
        for (at, _) in &coverings {
            reach.insert((at.x, at.y));
        }
        for place in &places.0 {
            if let Place::Scroll(scroll) = place {
                reach.insert((scroll.cell.x, scroll.cell.y));
            }
        }
        let mut rows = Vec::new();
        for (y, row) in (0..height).zip(blocks.chunks(usize::from(width.max(1)))) {
            let mut tiles = Vec::new();
            for (x, &block) in (0..width).zip(row) {
                let at = Cell { x, y };
                let covering = coverings
                    .iter()
                    .find_map(|&(covered, covering)| (covered == at).then_some(covering));
                let tile = self
                    .content
                    .tile(block, area, covering)
                    .ok_or(Skip::Refused(at))?;
                tiles.push(tile);
            }
            rows.push(tiles);
        }
        let across = usize::from(width / SCREEN_CELLS);
        let down = usize::from(height / SCREEN_CELLS);
        let scrolls = source::SCROLLS.read(&record.wram)?;
        let screens = scrolls
            .chunks(across.max(1))
            .take(down)
            .map(|row| {
                row.iter()
                    .map(|&area| {
                        ScrollArea::try_from(area).map(kind).map_err(|value| {
                            Skip::Unreadable(DecodeError::Unrepresentable {
                                source: source::SCROLLS.name,
                                value: u16::from(value),
                            })
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Decoded {
            state: State {
                tiles: Grid { rows },
                places,
                screens: Grid { rows: screens },
            },
            reach,
        })
    }

    /// The fields of `state` over the frame's reach; `outside` the cells outside it that differ
    /// from record N's, which only the model's projections compute.
    fn declared(&self, state: &State, outside: Vec<(u16, u16)>) -> Declared {
        let reach = self.reach.borrow();
        Declared {
            outside,
            places: state.places.0.iter().map(place_view).collect(),
            cells: Cells(
                reach
                    .iter()
                    .map(|&(x, y)| ((x, y), state.tiles.get(Cell { x, y }).map(cell_view)))
                    .collect(),
            ),
            screens: state.screens.rows.clone(),
        }
    }
}

/// A screen's kind by its scroll area: red is closed, blue short, green open.
const fn kind(area: ScrollArea) -> ScreenKind {
    match area {
        ScrollArea::Red => ScreenKind::Closed,
        ScrollArea::Blue => ScreenKind::Short,
        ScrollArea::Green => ScreenKind::Open,
    }
}

/// How she moves at `record`.
fn moving(record: &FrameRecord) -> Result<Set<Motion>, Skip> {
    let pose = word(record, source::SAMUS_POSE)?;
    let mut moving = Set::default();
    moving[Motion::Dashing] = word(record, SPEED_BOOST)? & SPEED_BITS == FULL_SPEED;
    moving[Motion::Shinesparking] = SHINESPARKING.contains(&pose);
    moving[Motion::ScrewAttacking] = SCREW_ATTACKING.contains(&pose);
    Ok(moving)
}

/// The starts record N+1 shows: each place of this part's that holds a reaction record N's
/// doesn't, or one gone back in its script, highest place first.
fn fresh(from: &Places, next: &Places, content: &BlockContent) -> Vec<(usize, Running, Started)> {
    let mut starts = Vec::new();
    for (slot, (before, after)) in from.0.iter().zip(&next.0).enumerate().rev() {
        let Place::Running(after) = after else {
            continue;
        };
        let is_new = match before {
            Place::Running(before) => {
                before.reaction != after.reaction
                    || before.cell != after.cell
                    || before.script != after.script
                    || before.next > after.next
            }
            Place::Free | Place::Scroll(_) | Place::OtherPart | Place::HeldUntilRun => true,
        };
        let family = content
            .id(&after.reaction)
            .and_then(|id| content.named(id))
            .map(|(_, family)| family);
        if let (true, Some(family)) = (is_new, family) {
            starts.push((slot, after.clone(), Started::of(family, after.script)));
        }
    }
    starts
}

/// The request a start of `started` on `at` makes, for a hit or a touch the hits don't name:
/// the bomb block's weapon by the step it starts from.
fn stand_in(started: Started, at: Cell, next: usize) -> Request {
    let hit = |by| Request::Hit { cell: at, by };
    match started {
        Started::Reveal => hit(HitBy::Bomb),
        Started::Family(Family::Crumble) => Request::Touch(Touch {
            cell: at,
            way: Way::Move(Heading::Down),
        }),
        Started::Family(Family::Speed) | Started::BombTouched => Request::Touch(Touch {
            cell: at,
            way: Way::Move(Heading::Right),
        }),
        Started::BombHit if next == 1 => hit(HitBy::Bomb),
        Started::BombHit | Started::Family(Family::PowerBomb | Family::Bomb) => {
            hit(HitBy::PowerBomb)
        }
        Started::Family(Family::Shot) => hit(HitBy::Beam),
        Started::Family(Family::SuperMissile) => hit(HitBy::SuperMissile),
    }
}

/// The frame's requests, from its watched instruction hits and the starts record N+1 shows; a
/// start no hit decides is stood in for.
fn requests(
    next: &FrameRecord,
    from: &Places,
    starts: Vec<(usize, Running, Started)>,
    her: Cell,
) -> Vec<(Request, Source)> {
    let mut pending = starts;
    let mut requests = Vec::new();
    let scrolls: Vec<(usize, Cell)> = from
        .0
        .iter()
        .enumerate()
        .filter_map(|(slot, place)| match place {
            Place::Scroll(scroll) => Some((slot, scroll.cell)),
            Place::Free | Place::Running(_) | Place::OtherPart | Place::HeldUntilRun => None,
        })
        .collect();
    let mut touched: Vec<(usize, Cell)> = Vec::new();
    let pcs: Vec<u32> = next.hits.iter().map(|hit| hit.pc).collect();
    let mut at = 0;
    while let Some(&pc) = pcs.get(at) {
        let after = pcs.get(at.wrapping_add(1)).copied();
        if let Some(&(_, started)) = SETUPS.iter().find(|&&(setup, _)| setup == pc) {
            let branch = BOMB_BRANCHES
                .iter()
                .find(|&&(branch, _)| Some(branch) == after)
                .map(|&(_, by)| by);
            if started == Started::BombHit && branch.is_some() {
                at = at.wrapping_add(1);
            }
            if let Some(place) = pending.iter().position(|(_, _, of)| *of == started) {
                let (_, running, _) = pending.remove(place);
                let cell = running.cell;
                let request = match (started, branch) {
                    (Started::BombHit, Some(by)) => Request::Hit { cell, by },
                    (Started::BombHit, None) => Request::Hit {
                        cell,
                        by: HitBy::Bomb,
                    },
                    _ => stand_in(started, cell, running.next),
                };
                requests.push((request, Source::Heard));
            }
        } else if pc == SCROLL_TOUCH || pc == CENTRE {
            let centre = pc == CENTRE;
            if centre {
                at = at.wrapping_add(1);
                if pcs.get(at) != Some(&SCROLL_TOUCH) {
                    continue;
                }
            }
            let after = pcs.get(at.wrapping_add(1)).copied();
            let pick = if after == Some(SCROLL_TRIGGER) {
                at = at.wrapping_add(1);
                let nearest = scrolls
                    .iter()
                    .filter(|(slot, _)| !touched.iter().any(|(done, _)| done == slot))
                    .min_by_key(|&&(slot, cell)| {
                        let distance = cell.x.abs_diff(her.x).wrapping_add(cell.y.abs_diff(her.y));
                        (distance, core::cmp::Reverse(slot))
                    })
                    .copied();
                if let Some(nearest) = nearest {
                    touched.push(nearest);
                }
                nearest
            } else {
                touched.first().copied()
            };
            if let Some((_, cell)) = pick {
                let request = if centre {
                    Request::Inside {
                        cell,
                        point: Point::Centre,
                    }
                } else {
                    Request::Touch(Touch {
                        cell,
                        way: Way::Move(Heading::Down),
                    })
                };
                requests.push((request, Source::Heard));
            }
        } else if pc == REFUSED {
            // A spawn with every slot taken: the request it would be takes no place.
        }
        at = at.wrapping_add(1);
    }
    requests.extend(pending.into_iter().map(|(_, running, started)| {
        (
            stand_in(started, running.cell, running.next),
            Source::Copied,
        )
    }));
    requests
}

/// The entry a start of `family`'s `script` is.
const fn started(family: Family, script: Script) -> BehaviourId {
    use BehaviourId as B;
    match (family, script) {
        (Family::Crumble | Family::Speed, Script::Revealed) => B::BombsReveal,
        (Family::Crumble, _) => B::CrumbleBlocks,
        (Family::Speed, _) => B::SpeedBlocks,
        (Family::Bomb, _) => B::BombBlocks,
        (Family::Shot, _) => B::ShotBlocks,
        (Family::SuperMissile, _) => B::SuperMissileBlocks,
        (Family::PowerBomb, _) => B::PowerBombBlocks,
    }
}

/// The cells `requests` name.
fn named(requests: &[(Request, Source)]) -> impl Iterator<Item = Cell> + '_ {
    requests.iter().filter_map(|(request, _)| match *request {
        Request::Inside { cell, .. } | Request::Hit { cell, .. } => Some(cell),
        Request::Touch(touch) => Some(touch.cell),
        Request::OtherPart => None,
    })
}

impl Domain for BlocksDomain {
    const NAME: &'static str = "blocks";
    type State = State;
    type Inputs = ();
    type World = Upstream;
    type Fields = Declared;
    type Skip = Skip;
    const FIELDS: &'static [Field<Declared>] = &[
        domain_field!("places", places),
        domain_field!("cells within the frame's reach", cells),
        domain_field!("screens", screens),
        domain_field!("cells outside the frame's reach, as record N's", outside),
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

    fn folds(&self, from: &FrameRecord, next: &FrameRecord) -> bool {
        let counter = |record: &FrameRecord| source::NMI_FRAME_COUNTER_WORD.read(&record.wram).ok();
        counter(from).is_some() && counter(from) == counter(next)
    }

    fn decode(&self, from: &FrameRecord, _interval: Interval) -> Result<(State, ()), Skip> {
        if word(from, source::GAME_STATE)? != GAMEPLAY {
            return Err(Skip::NotGameplay);
        }
        Ok((self.decode_record(from)?.state, ()))
    }

    fn world(&self, from: &FrameRecord, next: &FrameRecord) -> Result<Upstream, Skip> {
        if word(next, source::GAME_STATE)? != GAMEPLAY {
            return Err(Skip::NotGameplay);
        }
        let moved = word(next, source::NMI_FRAME_COUNTER_WORD)?
            .wrapping_sub(word(from, source::NMI_FRAME_COUNTER_WORD)?);
        if moved != 1 {
            return Err(Skip::Frames(moved));
        }
        if word(from, source::ROOM_PTR)? != word(next, source::ROOM_PTR)?
            || word(from, DOOR_LIST)? != word(next, DOOR_LIST)?
        {
            return Err(Skip::RoomChanged);
        }
        let before = self.decode_record(from)?;
        let after = self.decode_record(next)?;
        let starts = fresh(&before.state.places, &after.state.places, &self.content);
        let her = Cell {
            x: word(from, source::SAMUS_X_POS)? >> CELL_SHIFT,
            y: word(from, source::SAMUS_Y_POS)? >> CELL_SHIFT,
        };
        let requests = requests(next, &before.state.places, starts, her);
        let copied = before
            .state
            .places
            .0
            .iter()
            .zip(&after.state.places.0)
            .enumerate()
            .filter_map(|(slot, (was, now))| match (was, now) {
                (Place::OtherPart, Place::OtherPart) => None,
                (Place::OtherPart, _) => Some((slot, Place::Free)),
                (_, Place::OtherPart) => Some((slot, Place::OtherPart)),
                _ => None,
            })
            .collect();
        *self.start.borrow_mut() = before.state.tiles.rows;
        let mut reach = before.reach;
        reach.extend(after.reach);
        reach.extend(named(&requests).map(|at| (at.x, at.y)));
        *self.reach.borrow_mut() = reach;
        let run = if word(next, PLMS_ENABLED)? & PLMS_RUN == 0 {
            Run::Paused
        } else {
            Run::Runs
        };
        Ok(Upstream {
            requests,
            copied,
            moving: moving(next)?,
            run,
            did: Vec::new(),
        })
    }

    fn neutral(&self, upstream: &Upstream) -> Option<Upstream> {
        let copies = upstream
            .requests
            .iter()
            .any(|&(_, source)| source == Source::Copied);
        (copies || !upstream.copied.is_empty()).then(|| Upstream {
            requests: upstream
                .requests
                .iter()
                .filter(|&&(_, source)| source == Source::Heard)
                .copied()
                .collect(),
            copied: Vec::new(),
            ..upstream.clone()
        })
    }

    fn step(&self, state: &mut State, (): &(), upstream: &mut Upstream) {
        use BehaviourId as B;
        for (slot, place) in &upstream.copied {
            if let Some(held) = state.places.0.get_mut(*slot) {
                *held = place.clone();
            }
        }
        let before = state.places.clone();
        let requests: Vec<Request> = upstream
            .requests
            .iter()
            .map(|&(request, _)| request)
            .collect();
        let mut did = Vec::new();
        if before
            .0
            .iter()
            .any(|place| matches!(place, Place::Running(_)))
        {
            did.push(B::TheScriptsRun);
        }
        let touched = |place: &Place| matches!(place, Place::Scroll(scroll) if scroll.touched == Touched::Yes);
        if before.0.iter().any(touched) {
            did.push(B::ScrollCells);
        }
        if requests
            .iter()
            .any(|request| matches!(request, Request::Touch(_)))
        {
            did.push(B::AnswersToHerMotion);
            if before.full() {
                did.push(B::FullPlacesLetHerThrough);
            }
        }
        for request in &requests {
            let at = match *request {
                Request::Inside { cell, .. } | Request::Hit { cell, .. } => Some(cell),
                Request::Touch(touch) => Some(touch.cell),
                Request::OtherPart => None,
            };
            match at.and_then(|at| state.tiles.get(at)).map(|tile| &tile.mark) {
                Some(Mark::Scroll) => did.push(B::ScrollCells),
                Some(Mark::Nothing(_) | Mark::Other(Some(_))) => did.push(B::BlocksMarkedNothing),
                Some(
                    Mark::Unmarked
                    | Mark::Reaction(_)
                    | Mark::Inert
                    | Mark::Other(None)
                    | Mark::Reach(_),
                )
                | None => {}
            }
        }
        let blocks = self.content.blocks();
        let report = blocks.step(state, &[], &requests, upstream.moving, upstream.run);
        for (was, now) in before.0.iter().zip(&state.places.0) {
            if let (Place::Free, Place::Running(running)) = (was, now)
                && let Some((_, family)) = self
                    .content
                    .id(&running.reaction)
                    .and_then(|id| self.content.named(id))
            {
                did.extend([B::Places, B::TheScriptsRun, started(family, running.script)]);
            }
            let Place::Running(running) = was else {
                continue;
            };
            let Some(reaction) = blocks.reactions.get(&running.reaction) else {
                continue;
            };
            let end = match now {
                Place::Running(after) if after.reaction == running.reaction => after.next,
                Place::Free
                | Place::Running(_)
                | Place::Scroll(_)
                | Place::OtherPart
                | Place::HeldUntilRun => usize::MAX,
            };
            let steps = &reaction.scripts[running.script];
            let passed = steps
                .get(running.next..end.min(steps.len()))
                .unwrap_or_default();
            if passed.contains(&Step::Regrow) {
                did.push(B::Regrowing);
            }
        }
        if !report.redrawn.is_empty() {
            did.extend([B::PatternCells, B::WhatTheScreenRedraws]);
        }
        if !report.sounds.is_empty() {
            did.push(B::Sounds);
        }
        upstream.did = did;
    }

    fn exercised(&self, _state: &State, (): &(), upstream: &Upstream) -> Vec<usize> {
        let mut places: Vec<usize> = upstream
            .did
            .iter()
            .filter_map(|&id| CATALOGUE.iter().position(|entry| entry.id == id))
            .collect();
        places.sort_unstable();
        places.dedup();
        places
    }

    fn fields(&self, state: &State) -> Declared {
        let reach = self.reach.borrow();
        let start = self.start.borrow();
        let mut outside = Vec::new();
        for (y, (rows, was)) in (0_u16..).zip(state.tiles.rows.iter().zip(start.iter())) {
            for (x, (tile, was)) in (0_u16..).zip(rows.iter().zip(was)) {
                if tile != was && !reach.contains(&(x, y)) {
                    outside.push((x, y));
                }
            }
        }
        drop((reach, start));
        self.declared(state, outside)
    }

    fn expected(&self, next: &FrameRecord) -> Result<Declared, Skip> {
        Ok(self.declared(&self.decode_record(next)?.state, Vec::new()))
    }

    fn uncompared(&self, skip: &Skip) -> Uncompared {
        match skip {
            Skip::NotGameplay | Skip::Frames(_) | Skip::RoomChanged => {
                Uncompared::Skip(skip.to_string())
            }
            Skip::Unreadable(_) | Skip::Refused(_) | Skip::Place(..) | Skip::ScrollList(_) => {
                Uncompared::Defect(skip.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BlocksDomain, CELLS, Domain, FieldSet, OUTSIDE, PLACES_FIELD, SCREENS};

    #[test]
    fn the_field_sets_name_the_declared_fields_at_their_places() {
        for (set, name) in [
            (PLACES_FIELD, "places"),
            (CELLS, "cells within the frame's reach"),
            (SCREENS, "screens"),
            (OUTSIDE, "cells outside the frame's reach, as record N's"),
        ] {
            let place = BlocksDomain::FIELDS
                .iter()
                .position(|field| field.name == name);
            assert_eq!(place.map(FieldSet::single), Some(set), "{name}");
        }
    }
}
