//! The blocks as the engine meets them: the blocks model (`clean_model_blocks`).
//!
//! The engine keeps the room's cells as the game keeps them, in `RoomScoped::blocks` (each cell's
//! type, BTS and tile), and the reactions' places in its PLM slots beside the items'. Each call
//! builds the model's State of the whole room from those once, steps it through the call's
//! requests in order, and writes back what it changed (`docs/behaviours/blocks.decode.md`,
//! "State"):
//! - the room's start: its scroll cells and stand-ins, beside the items' placements
//!   ([`start_room`]);
//! - the frame's requests, where the engine asks them: the cells she is inside ([`inside`]), her
//!   beam's hits ([`hit`]) and her moves' touches ([`touches`]);
//! - the run, in the PLM pass, between the items' places ([`run`]).
//!
//! The State isn't held from one call to the next: the engine's other parts read and write the
//! cells and the PLM slots between the calls (her moves, the items' touches, the items' turns in
//! the pass), so the engine's room stays the one copy. A call of requests therefore steps the
//! model with the run paused, and the PLM pass steps it with the run on the places between two
//! items'.
//!
//! Where an asker doesn't exist in the engine yet, the model is asked nothing:
//! - her moves meet each cell by its body (`samus_world`), and the touches reach the blocks after
//!   them, so a touch's answer (`AnswersToHerMotion`) changes nothing she does;
//! - how she moves: the engine keeps no speed booster's charge, so to the blocks she is never
//!   dashing, shinesparking or screw attacking;
//! - her weapons are the uncharged beam alone, and only on shootable blocks (`projectile`); the
//!   bombs', power bombs' and the grapple's hits join with their domains (W1, S6);
//! - the run is never paused: the engine has no room loading or door transition that pauses it
//!   (R1, R2);
//! - the sounds the scripts ask for have no sound queue to go to (U1).

use core::ops::Range;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use clean_model_blocks::{
    Becomes, CellKind, Frames, Grid, Inside, Mark, OutsideTheRoom, PatternCell, Place, Placement,
    Places, ReactingBody, Reaction, ReactionName, Request, Rules, Run, Running, ScreenChange,
    ScrollReaction, SoundRequest, StandIn, State, Step, Tile, Touched,
};
use clean_types::{Cell, Family, Flip, HitBy, Look, MetAs, Pair, Picture, Point, ScreenKind};
use clean_types::{Script, Set, Touch};
use data::blocks::{self as asset, ReactionId, SCROLL_CELL};
use data::level::{Block, BlockKind, Bts, Offset, ScrollArea, TileNumber};
use data::plm::{Extension, PlmEntry, PlmKind, PlmPopulation, ScrollChangesId};
use data::room::Area;
use enum_map::EnumMap;

use crate::plm::draw::Drawing;
use crate::state::room::{
    BlockIndex, GrowsBackAs, PlmBody, PlmCommon, ReactionPlace, ReactionScript, RoomScoped,
    ScrollPlm, ScrollStep,
};
use crate::state::slots::{Frames as Timer, Occupancy, Slot};

/// The cells a screen spans across and down.
const SCREEN_CELLS: u16 = 16;
/// The block types of the reacting cells, as blocks and as air to her: special, shootable,
/// bombable.
const REACTING: [(u8, u8); 3] = [(0xb, 0x3), (0xc, 0x4), (0xf, 0x7)];
/// Every BTS.
const BTS_VALUES: usize = 256;
/// A mark table's row: every BTS of every reacting type, blocks then air.
const MARK_ROW: usize = 2 * REACTING.len() * BTS_VALUES;
/// The areas the mark tables have a row for.
const AREAS: [Area; 8] = [
    Area::Crateria,
    Area::Brinstar,
    Area::Norfair,
    Area::WreckedShip,
    Area::Maridia,
    Area::Tourian,
    Area::Ceres,
    Area::Debug,
];

/// The game's rules, built once.
fn rules() -> &'static Rules {
    static RULES: OnceLock<Rules> = OnceLock::new();
    RULES.get_or_init(Rules::reference)
}

/// What a call reads besides the room: the blocks' content and the room's PLM population, whose
/// scroll cells' change lists the scroll reactions name.
#[derive(Clone, Copy, Debug)]
pub struct Reads<'a> {
    /// The blocks' content.
    pub content: &'a BlockContent,
    /// The room's population.
    pub population: &'a PlmPopulation,
}

/// Why a call can't go on: a cell the room's loading would have refused, or a place the model
/// isn't defined on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlocksFault {
    /// A reacting cell some asker of which reads past its table's end, or whose mark names no
    /// reaction: the importer refuses it (LEGACY, "Reacting cells whose tables end before their
    /// mark"; spec §8).
    Refused(BlockIndex),
    /// A scroll cell with no scroll reaction on it, where the original hangs (LEGACY, "A scroll
    /// cell with no scroll reaction").
    NoScrollReaction(BlockIndex),
    /// A place whose reaction the content doesn't hold, or whose step or wait its script
    /// doesn't have.
    Place(u8),
    /// A cell the model made a reacting kind whose block type nothing names: its mark is no
    /// family's and the cell had none.
    Class(BlockIndex),
    /// A request or a placement past the room's last row, which every asker keeps inside it.
    Outside(Cell),
    /// The room's screens aren't whole rows of the screens it is across.
    Screens,
}

impl From<OutsideTheRoom> for BlocksFault {
    fn from(OutsideTheRoom(cell): OutsideTheRoom) -> Self {
        Self::Outside(cell)
    }
}

/// An imported reaction as the engine keeps it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Imported {
    /// The model's reaction.
    reaction: Arc<Reaction>,
    /// The cells its patterns cover other than its own.
    others: Vec<Pair<i16>>,
}

/// The blocks' content: the imported reactions, and what each reacting cell's mark is, by its
/// area, block type and BTS.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlockContent {
    /// The reactions as the importer gave them.
    imported: asset::Reactions,
    /// By [`ReactionId`].
    reactions: Vec<Imported>,
    /// By area, then the reacting types' order (`REACTING`, blocks then air), then BTS; `None`
    /// where the importer refuses the cell.
    marks: Vec<Vec<Option<Mark>>>,
    /// By area, then BTS: whether she holds a place inside special air.
    inside: Vec<Vec<bool>>,
    /// The last room's cells as imported, which the next call starts from.
    imports: Imports,
}

/// A room's cells as a call last imported them, with the level they were imported from.
#[derive(Debug)]
struct ImportedRoom {
    area: Area,
    width: u16,
    blocks: Vec<Block>,
    /// Row by row, with no running reaction's covering.
    tiles: Vec<Vec<Tile>>,
}

/// The cells the last call imported, kept so that the next imports only the cells whose blocks
/// differ from the ones they came from. Every cell is compared, so whoever wrote a block since,
/// the cell is imported again from what it is now: no writer is trusted to say what it wrote.
#[derive(Debug, Default)]
struct Imports(Mutex<Option<ImportedRoom>>);

impl Clone for Imports {
    /// A copy starts with nothing imported, which only costs its first call a full import.
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PartialEq for Imports {
    /// What is kept changes no call's result, so any two are alike.
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for Imports {}

/// The model's look of a tile.
fn look(tile: data::level::Tile) -> Option<Look> {
    Some(Look {
        picture: Picture::new(tile.number.0)?,
        flip: match (tile.x_flip, tile.y_flip) {
            (false, false) => Flip::None,
            (true, false) => Flip::Across,
            (false, true) => Flip::Down,
            (true, true) => Flip::Both,
        },
    })
}

/// The tile of the model's look.
const fn tile(look: Look) -> data::level::Tile {
    data::level::Tile {
        number: TileNumber(look.picture.index()),
        x_flip: matches!(look.flip, Flip::Across | Flip::Both),
        y_flip: matches!(look.flip, Flip::Down | Flip::Both),
    }
}

/// The model's step of an imported one; `None` for a wait the model can't hold.
fn step(step: &asset::Step) -> Option<Step> {
    Some(match step {
        asset::Step::Sound(sound) => Step::Sound(SoundRequest {
            sound: sound.name(),
            queue_limit: sound.queue_limit,
        }),
        asset::Step::Show { pattern, frames } => Step::Show {
            pattern: pattern
                .iter()
                .map(|cell| PatternCell {
                    offset: cell.offset,
                    becomes: match cell.becomes {
                        asset::Becomes::Air => Becomes::Air,
                        asset::Becomes::Block => Becomes::Block,
                        asset::Becomes::StandInAcross => Becomes::StandInAcross,
                        asset::Becomes::StandInDown => Becomes::StandInDown,
                    },
                    look: cell.look,
                })
                .collect(),
            frames: Frames::new(*frames)?,
        },
        asset::Step::Regrow => Step::Regrow,
        asset::Step::End => Step::End,
    })
}

/// The model's reaction of an imported one; `None` for a wait the model can't hold.
fn convert(reaction: &asset::Reaction) -> Option<Reaction> {
    let mut scripts: EnumMap<Script, Vec<Step>> = EnumMap::default();
    for (script, steps) in &mut scripts {
        *steps = reaction
            .script(script)
            .iter()
            .map(step)
            .collect::<Option<_>>()?;
    }
    Some(Reaction {
        name: ReactionName(reaction.name.to_owned()),
        family: reaction.family,
        whole_look: reaction.whole_look,
        first_wait: Frames::new(reaction.first_wait)?,
        scripts,
    })
}

/// The reaction as the engine keeps it, with the cells its patterns cover past its own.
fn imported(reaction: Reaction) -> Imported {
    let mut others: Vec<Pair<i16>> = reaction
        .scripts
        .values()
        .flatten()
        .filter_map(|step| match step {
            Step::Show { pattern, .. } => Some(pattern.iter().map(|cell| cell.offset)),
            Step::Sound(_) | Step::Regrow | Step::End => None,
        })
        .flatten()
        .filter(|&offset| offset != Pair { x: 0, y: 0 })
        .collect();
    others.sort_unstable_by_key(|offset| (offset.y, offset.x));
    others.dedup();
    Imported {
        reaction: Arc::new(reaction),
        others,
    }
}

/// The model's script of the engine's.
const fn script(script: ReactionScript) -> Script {
    match script {
        ReactionScript::Broken => Script::Broken,
        ReactionScript::Revealed => Script::Revealed,
        ReactionScript::Touched => Script::Touched,
    }
}

/// The engine's script of the model's.
const fn engine_script(script: Script) -> ReactionScript {
    match script {
        Script::Broken => ReactionScript::Broken,
        Script::Revealed => ReactionScript::Revealed,
        Script::Touched => ReactionScript::Touched,
    }
}

/// The model's body of what the engine's cell grows back as.
const fn reacting_body(grows: GrowsBackAs) -> ReactingBody {
    match grows {
        GrowsBackAs::Block => ReactingBody::Block,
        GrowsBackAs::Air => ReactingBody::Air,
    }
}

/// What the engine's cell grows back as, of the model's body.
const fn grows_back_as(body: ReactingBody) -> GrowsBackAs {
    match body {
        ReactingBody::Block => GrowsBackAs::Block,
        ReactingBody::Air => GrowsBackAs::Air,
    }
}

/// A mark table row's entry for a cell of `kind`: its type's place in `REACTING`'s order,
/// blocks first, then its BTS.
fn mark_entry(kind: BlockKind) -> Option<usize> {
    let nybble = kind.nybble();
    let blocks = REACTING.iter().position(|&(block, _)| block == nybble);
    let air = REACTING.iter().position(|&(_, air)| air == nybble);
    let place = blocks.or_else(|| air.map(|place| place.wrapping_add(REACTING.len())))?;
    Some(
        place
            .wrapping_mul(BTS_VALUES)
            .wrapping_add(usize::from(kind.bts())),
    )
}

/// Every reacting kind, in a mark table row's order.
fn reacting_kinds() -> impl Iterator<Item = BlockKind> {
    let blocks = REACTING.iter().map(|&(block, _)| block);
    let air = REACTING.iter().map(|&(_, air)| air);
    blocks
        .chain(air)
        .flat_map(|nybble| (0..=u8::MAX).map(move |bts| BlockKind::new(nybble, bts)))
}

impl BlockContent {
    /// The content of the imported `reactions` and their mark tables. A cell the importer
    /// refuses, its tables ending before its mark or its mark naming no reaction this part
    /// holds, is kept as refused, and a call that meets it stops.
    ///
    /// # Errors
    ///
    /// The id of a reaction whose shows wait longer than the model's frames, which the importer
    /// refuses first.
    pub fn import(reactions: &asset::Reactions) -> Result<Self, ReactionId> {
        let mut held = Vec::new();
        for (reaction, index) in reactions.all().iter().zip(0_u16..) {
            held.push(imported(convert(reaction).ok_or(ReactionId(index))?));
        }
        let mut content = Self {
            imported: reactions.clone(),
            reactions: held,
            ..Self::default()
        };
        content.marks = AREAS
            .iter()
            .map(|&area| {
                reacting_kinds()
                    .map(|kind| match reactions.mark(kind, area) {
                        Ok(Some(mark)) => content.mark(mark),
                        Ok(None) | Err(_) => None,
                    })
                    .collect()
            })
            .collect();
        content.inside = AREAS
            .iter()
            .map(|&area| {
                (0..=u8::MAX)
                    .map(|bts| reactions.holds_place_inside(BlockKind::SpecialAir(Bts(bts)), area))
                    .collect()
            })
            .collect();
        Ok(content)
    }

    /// Content of invented reactions, by their ids' order, and the marks of the cells listed in
    /// every area; another reacting cell is another part's. `None` where a listed mark names no
    /// reaction listed.
    #[must_use]
    pub fn invented(reactions: Vec<Reaction>, marks: &[(BlockKind, asset::Mark)]) -> Option<Self> {
        let mut content = Self {
            reactions: reactions.into_iter().map(imported).collect(),
            ..Self::default()
        };
        let mut row = vec![Some(Mark::Other(None)); MARK_ROW];
        for &(kind, mark) in marks {
            if let Some(entry) = mark_entry(kind).and_then(|entry| row.get_mut(entry)) {
                *entry = Some(content.mark(mark)?);
            }
        }
        content.marks = vec![row; AREAS.len()];
        content.inside = vec![vec![false; BTS_VALUES]; AREAS.len()];
        Some(content)
    }

    /// The reactions as the importer gave them, which a record decoder reads places by.
    #[must_use]
    pub const fn imported(&self) -> &asset::Reactions {
        &self.imported
    }

    /// The reaction `id` names.
    #[must_use]
    pub fn reaction(&self, id: ReactionId) -> Option<&Arc<Reaction>> {
        self.reactions
            .get(usize::from(id.0))
            .map(|held| &held.reaction)
    }

    /// The id of `reaction`, if it is one of the content's.
    #[must_use]
    pub fn id(&self, reaction: &Arc<Reaction>) -> Option<ReactionId> {
        self.reactions
            .iter()
            .position(|held| Arc::ptr_eq(&held.reaction, reaction))
            .and_then(|index| u16::try_from(index).ok())
            .map(ReactionId)
    }

    /// The cells the patterns of `reaction` cover other than its own, from its cell.
    fn covered(&self, reaction: &Arc<Reaction>) -> &[Pair<i16>] {
        self.reactions
            .iter()
            .find(|held| Arc::ptr_eq(&held.reaction, reaction))
            .map_or(&[][..], |held| held.others.as_slice())
    }

    /// The model's mark of an imported one; `None` for a reaction the content doesn't hold.
    fn mark(&self, mark: asset::Mark) -> Option<Mark> {
        Some(match mark {
            asset::Mark::Reaction(id) => Mark::Reaction(Arc::clone(self.reaction(id)?)),
            asset::Mark::Scroll => Mark::Scroll,
            asset::Mark::Nothing(met) => Mark::Nothing(met),
            asset::Mark::Inert => Mark::Inert,
            asset::Mark::Other(met) => Mark::Other(met),
        })
    }

    /// The mark tables of `area`; none for an area they have no row for.
    fn area(&self, area: Area) -> AreaTables<'_> {
        fn pick<T>(rows: &[Vec<T>], row: Option<usize>) -> &[T] {
            row.and_then(|row| rows.get(row)).map_or(&[], Vec::as_slice)
        }
        let row = AREAS.iter().position(|&known| known == area);
        AreaTables {
            marks: pick(&self.marks, row),
            inside: pick(&self.inside, row),
        }
    }

    /// The model's tile of a cell of `block` in `area`, `covering` naming the reaction running on
    /// an air or solid cell: `None` for a cell the importer refuses.
    #[must_use]
    pub fn tile(&self, block: Block, area: Area, covering: Option<Covering>) -> Option<Tile> {
        self.area(area).tile(block, covering)
    }

    /// What runs on the air and solid cells under the running reactions of `places`: each one's
    /// own cell, then the other cells its patterns cover.
    fn coverings(&self, places: &Places) -> Vec<(Cell, Covering)> {
        let mut own = Vec::new();
        let mut others = Vec::new();
        for place in &places.0 {
            let Place::Running(running) = place else {
                continue;
            };
            let reaction = &running.reaction;
            own.push((running.cell, Covering::Own(reaction.family)));
            others.extend(self.covered(reaction).iter().filter_map(|offset| {
                Some((
                    Cell {
                        x: running.cell.x.checked_add_signed(offset.x)?,
                        y: running.cell.y.checked_add_signed(offset.y)?,
                    },
                    Covering::Other,
                ))
            }));
        }
        own.extend(others);
        own
    }

    /// The model's cells of the level `blocks`, `width` cells across, in `area`, under the
    /// running reactions of `places`, row by row.
    ///
    /// # Errors
    ///
    /// The level index of the first cell the importer refuses.
    pub fn tiles(
        &self,
        blocks: &[Block],
        width: u16,
        area: Area,
        places: &Places,
    ) -> Result<Vec<Vec<Tile>>, u16> {
        let tables = self.area(area);
        let mut rows = tables.rows(blocks, width)?;
        self.cover(&mut rows, blocks, width, tables, places)?;
        Ok(rows)
    }

    /// [`Self::tiles`], starting from the last call's import: only the cells whose blocks
    /// differ from the ones they were imported from are imported again.
    fn kept_tiles(
        &self,
        blocks: &[Block],
        width: u16,
        area: Area,
        places: &Places,
    ) -> Result<Vec<Vec<Tile>>, u16> {
        let tables = self.area(area);
        let mut kept = self
            .imports
            .0
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let room = match kept.take() {
            Some(mut room)
                if room.area == area
                    && room.width == width
                    && room.blocks.len() == blocks.len() =>
            {
                let pairs = blocks.iter().zip(room.blocks.iter_mut());
                for (index, (&block, was)) in (0_u16..).zip(pairs) {
                    if block == *was {
                        continue;
                    }
                    let at = Cell {
                        x: index.checked_rem(width).unwrap_or(index),
                        y: index.checked_div(width).unwrap_or(0),
                    };
                    let cell = room
                        .tiles
                        .get_mut(usize::from(at.y))
                        .and_then(|row| row.get_mut(usize::from(at.x)));
                    if let Some(cell) = cell {
                        *cell = tables.tile(block, None).ok_or(index)?;
                    }
                    *was = block;
                }
                room
            }
            Some(_) | None => ImportedRoom {
                area,
                width,
                blocks: blocks.to_vec(),
                tiles: tables.rows(blocks, width)?,
            },
        };
        let mut rows = room.tiles.clone();
        *kept = Some(room);
        drop(kept);
        self.cover(&mut rows, blocks, width, tables, places)?;
        Ok(rows)
    }

    /// Lay the coverings of the running reactions of `places` over `rows`.
    fn cover(
        &self,
        rows: &mut [Vec<Tile>],
        blocks: &[Block],
        width: u16,
        tables: AreaTables<'_>,
        places: &Places,
    ) -> Result<(), u16> {
        let index = |at: Cell| at.y.wrapping_mul(width).wrapping_add(at.x);
        // In reverse, so a reaction's own cell wins over another's covering of the same cell.
        for &(at, covering) in self.coverings(places).iter().rev() {
            let block = blocks.get(usize::from(index(at)));
            let cell = rows
                .get_mut(usize::from(at.y))
                .and_then(|row| row.get_mut(usize::from(at.x)));
            if let (Some(&block), Some(cell)) = (block, cell) {
                *cell = tables
                    .tile(block, Some(covering))
                    .ok_or_else(|| index(at))?;
            }
        }
        Ok(())
    }
}

/// The content's mark tables for one area.
#[derive(Clone, Copy)]
struct AreaTables<'a> {
    /// By the reacting types' order, then BTS.
    marks: &'a [Option<Mark>],
    /// By BTS: whether she holds a place inside special air.
    inside: &'a [bool],
}

impl AreaTables<'_> {
    /// The model's cells of the level `blocks`, `width` cells across, with no covering, row by
    /// row: `Err` with the level index of the first cell the importer refuses.
    fn rows(self, blocks: &[Block], width: u16) -> Result<Vec<Vec<Tile>>, u16> {
        (0_u16..)
            .zip(blocks.chunks_exact(usize::from(width.max(1))))
            .map(|(y, row)| {
                (0_u16..)
                    .zip(row)
                    .map(|(x, &block)| {
                        self.tile(block, None)
                            .ok_or_else(|| y.wrapping_mul(width).wrapping_add(x))
                    })
                    .collect()
            })
            .collect()
    }

    /// A reacting cell's mark; `None` where the importer refuses it.
    fn reacting_mark(self, kind: BlockKind) -> Option<Mark> {
        self.marks.get(mark_entry(kind)?)?.clone()
    }

    /// The model's tile of a cell of `block`, `covering` naming the reaction running on an air or
    /// solid cell: `None` for a cell the importer refuses.
    fn tile(self, block: Block, covering: Option<Covering>) -> Option<Tile> {
        let kind = block.kind;
        let (cell, mark) = match kind {
            BlockKind::Air(_) => (CellKind::Air, self.covering_mark(kind, covering)?),
            BlockKind::Solid(_) => (CellKind::Solid, self.covering_mark(kind, covering)?),
            BlockKind::HorizontalExtension(Offset(by)) => (
                CellKind::StandIn(StandIn::Across),
                Mark::Reach(i16::from(by)),
            ),
            BlockKind::VerticalExtension(Offset(by)) => {
                (CellKind::StandIn(StandIn::Down), Mark::Reach(i16::from(by)))
            }
            BlockKind::Special(_) | BlockKind::Shootable(_) | BlockKind::Bombable(_) => (
                CellKind::Reacting(ReactingBody::Block),
                self.reacting_mark(kind)?,
            ),
            BlockKind::SpecialAir(_) | BlockKind::ShootableAir(_) | BlockKind::BombableAir(_) => (
                CellKind::Reacting(ReactingBody::Air),
                self.reacting_mark(kind)?,
            ),
            // The other domains' kinds, by how they meet her: none reacts here.
            BlockKind::Slope(_)
            | BlockKind::Door(_)
            | BlockKind::Spike(_)
            | BlockKind::Grapple(_) => (CellKind::Solid, Mark::Unmarked),
            BlockKind::SpikeAir(_) | BlockKind::UnusedAir(_) => (CellKind::Air, Mark::Unmarked),
        };
        let holds = matches!(kind, BlockKind::SpecialAir(Bts(bts))
            if self.inside.get(usize::from(bts)) == Some(&true));
        Some(Tile {
            kind: cell,
            mark,
            look: look(block.tile)?,
            inside: if holds {
                Inside::HoldsPlace
            } else {
                Inside::HoldsNoPlace
            },
        })
    }

    /// The mark of an air or solid cell: none, unless a reaction runs on it, whose own cell's mark
    /// is its BTS as the reaction's family's block reads it, and whose other cells' a reach.
    fn covering_mark(self, kind: BlockKind, covering: Option<Covering>) -> Option<Mark> {
        Some(match covering {
            None => Mark::Unmarked,
            Some(Covering::Own(family)) => {
                let (block, _) = asset::Reactions::class(family);
                self.reacting_mark(BlockKind::new(block, kind.bts()))?
            }
            Some(Covering::Other) => Mark::Reach(i16::from(i8::from_le_bytes([kind.bts()]))),
        })
    }
}

/// What runs on an air or solid cell, which names its mark: a reaction of this family on the
/// cell itself, or another cell's reaction whose patterns cover it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Covering {
    /// The reaction on the cell itself, of this family: the cell's BTS is its mark as the
    /// family's blocks read it.
    Own(Family),
    /// Another cell's reaction: the cell's BTS is a stand-in's reach.
    Other,
}

/// What a step is, for what its places' changes write back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    /// One placement of the room's start, with the change list a scroll cell placed names.
    RoomStart(Option<ScrollChangesId>),
    /// A request on a cell: a hold it takes is a scroll cell's touch or a cell marked
    /// `Nothing`'s.
    Request(BlockIndex, HoldBody),
    /// The run over a range of places.
    Run,
}

/// What a hold until the run is in the engine's slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HoldBody {
    /// The scroll block touch PLM, on block 0.
    ScrollTouch,
    /// A PLM that only deletes itself, on the cell.
    Held,
}

/// The cell at level index `index` of a room `width` cells across.
fn cell(index: BlockIndex, width: u16) -> Cell {
    Cell {
        x: index.0.checked_rem(width).unwrap_or(index.0),
        y: index.0.checked_div(width).unwrap_or(0),
    }
}

/// The level index of `cell`.
const fn index(cell: Cell, width: u16) -> BlockIndex {
    BlockIndex(cell.y.wrapping_mul(width).wrapping_add(cell.x))
}

/// A screen's kind by its scroll area: red is closed, blue short, green open.
const fn screen_kind(area: ScrollArea) -> ScreenKind {
    match area {
        ScrollArea::Red => ScreenKind::Closed,
        ScrollArea::Blue => ScreenKind::Short,
        ScrollArea::Green => ScreenKind::Open,
    }
}

/// The scroll area of a screen's kind.
const fn scroll_area(kind: ScreenKind) -> ScrollArea {
    match kind {
        ScreenKind::Closed => ScrollArea::Red,
        ScreenKind::Short => ScrollArea::Blue,
        ScreenKind::Open => ScrollArea::Green,
    }
}

/// The screens a room is across.
const fn screens_across(room: &RoomScoped) -> u16 {
    room.width.0 / SCREEN_CELLS
}

/// The screens the scroll change list `id` changes, each at its place in a room `across`
/// screens wide.
fn changes(
    population: &PlmPopulation,
    id: ScrollChangesId,
    across: u16,
) -> Option<Vec<ScreenChange>> {
    Some(
        population
            .changes(id)?
            .iter()
            .map(|change| {
                let screen = u16::from(change.screen);
                ScreenChange {
                    screen: Pair {
                        x: screen.checked_rem(across).unwrap_or(screen),
                        y: screen.checked_div(across).unwrap_or(0),
                    },
                    becomes: screen_kind(change.area),
                }
            })
            .collect(),
    )
}

/// The model's place of an engine slot; `None` where the model isn't defined on it.
fn place(
    slot: &Slot<PlmCommon, PlmBody>,
    content: &BlockContent,
    population: &PlmPopulation,
    room: &RoomScoped,
) -> Option<Place> {
    let body = match slot.occupancy {
        Occupancy::Free => return Some(Place::Free),
        Occupancy::Live(body) => body,
        Occupancy::Deleting(_) => return None,
    };
    let at = cell(slot.common.block, room.width.0);
    Some(match body {
        PlmBody::Scroll(scroll) => Place::Scroll(ScrollReaction {
            cell: at,
            changes: changes(population, scroll.changes, screens_across(room))?,
            touched: if scroll.triggered {
                Touched::Yes
            } else {
                Touched::No
            },
        }),
        PlmBody::ScrollTouch | PlmBody::Held => Place::HeldUntilRun,
        PlmBody::Item { .. } | PlmBody::ItemCollision => Place::OtherPart,
        PlmBody::Reaction(running) => Place::Running(Running {
            reaction: Arc::clone(content.reaction(running.reaction)?),
            script: script(running.script),
            grows_back_as: running.grows_back_as.map(reacting_body),
            cell: at,
            next: usize::from(running.next),
            wait: Frames::new(slot.common.cursor.delay.0)?,
        }),
    })
}

/// The model's State of the whole of `room`, the places outside `places` another part's.
fn state(
    room: &RoomScoped,
    content: &BlockContent,
    population: &PlmPopulation,
    places: &Range<usize>,
) -> Result<State, BlocksFault> {
    // Every running reaction names the marks of the cells it covers, a place outside `places`
    // too; the model sees those as another part's.
    let mut model = Places::default();
    for ((index, slot), place_of) in room.plms.slots.iter().enumerate().zip(&mut model.0) {
        let running = matches!(slot.occupancy, Occupancy::Live(PlmBody::Reaction(_)));
        if places.contains(&index) || running {
            *place_of = place(slot, content, population, room)
                .ok_or_else(|| BlocksFault::Place(u8::try_from(index).unwrap_or(u8::MAX)))?;
        } else {
            *place_of = Place::OtherPart;
        }
    }
    let rows = content
        .kept_tiles(&room.blocks, room.width.0, room.area, &model)
        .map_err(|at| BlocksFault::Refused(BlockIndex(at)))?;
    for (index, place_of) in model.0.iter_mut().enumerate() {
        if !places.contains(&index) {
            *place_of = Place::OtherPart;
        }
    }
    let across = usize::from(screens_across(room));
    let screens = room
        .scroll_areas
        .chunks(across.max(1))
        .filter(|_| across != 0)
        .map(|row| row.iter().map(|&area| screen_kind(area)).collect())
        .collect();
    Ok(State {
        // Rows of the level's width, each as long as the others.
        tiles: Grid::new(rows).ok_or(BlocksFault::Screens)?,
        places: model,
        screens: Grid::new(screens).ok_or(BlocksFault::Screens)?,
    })
}

/// The engine's block of the model's `tile`, from what the cell was.
fn block(was: Block, tile: &Tile, at: BlockIndex) -> Result<Block, BlocksFault> {
    let bts = match tile.mark {
        Mark::Scroll if tile.kind == CellKind::Reacting(ReactingBody::Air) => SCROLL_CELL.bts(),
        Mark::Reach(reach) => {
            let [byte] = i8::try_from(reach).unwrap_or(0).to_le_bytes();
            byte
        }
        Mark::Unmarked
        | Mark::Reaction(_)
        | Mark::Scroll
        | Mark::Nothing(_)
        | Mark::Inert
        | Mark::Other(_) => was.kind.bts(),
    };
    let reacting = |body: ReactingBody| -> Result<u8, BlocksFault> {
        let named = match &tile.mark {
            Mark::Reaction(reaction) => Some(asset::Reactions::class(reaction.family)),
            Mark::Nothing(met) => Some(asset::Reactions::met_class(*met)),
            Mark::Scroll => Some(asset::Reactions::met_class(MetAs::Crumble)),
            Mark::Unmarked | Mark::Inert | Mark::Other(_) | Mark::Reach(_) => None,
        };
        let kept = REACTING
            .iter()
            .copied()
            .find(|&(block, air)| was.kind.nybble() == block || was.kind.nybble() == air);
        let (block, air) = named.or(kept).ok_or(BlocksFault::Class(at))?;
        Ok(match body {
            ReactingBody::Block => block,
            ReactingBody::Air => air,
        })
    };
    let nybble = match tile.kind {
        CellKind::Air => 0x0,
        CellKind::Solid => 0x8,
        CellKind::StandIn(StandIn::Across) => 0x5,
        CellKind::StandIn(StandIn::Down) => 0xd,
        CellKind::Reacting(body) => reacting(body)?,
    };
    Ok(Block {
        kind: BlockKind::new(nybble, bts),
        tile: self::tile(tile.look),
    })
}

/// The engine's reaction place of a running one.
fn reaction_place(running: &Running, content: &BlockContent) -> Option<ReactionPlace> {
    Some(ReactionPlace {
        reaction: content.id(&running.reaction)?,
        script: engine_script(running.script),
        next: u16::try_from(running.next).ok()?,
        grows_back_as: running.grows_back_as.map(grows_back_as),
    })
}

/// The cell `request` names, if it names one.
const fn request_cell(request: Request) -> Option<Cell> {
    match request {
        Request::Inside { cell, .. }
        | Request::Hit { cell, .. }
        | Request::Touch(Touch { cell, .. }) => Some(cell),
        Request::OtherPart => None,
    }
}

/// The cell `placement` places, if it places one.
const fn placement_cell(placement: &Placement) -> Option<Cell> {
    match placement {
        Placement::Scroll { cell, .. } | Placement::StandIn { cell, .. } => Some(*cell),
        Placement::OtherPart => None,
    }
}

/// A spawn into `slot` on `block` (`$84:84C5`): its timer 1, its body `occupancy`; a field the
/// spawn doesn't write keeps the last occupant's value.
const fn fill(
    slot: &mut Slot<PlmCommon, PlmBody>,
    block: BlockIndex,
    occupancy: Occupancy<PlmBody>,
) {
    slot.generation.0 = slot.generation.0.wrapping_add(1);
    slot.common.block = block;
    slot.common.cursor.delay = Timer(1);
    slot.occupancy = occupancy;
}

/// The fault of place `number`.
fn place_fault(number: usize) -> BlocksFault {
    BlocksFault::Place(u8::try_from(number).unwrap_or(u8::MAX))
}

/// One call's model: the State of the room as the call found it, stepped through the call.
///
/// A step writes only the cells it names: a request's or a placement's own, and those a show or
/// a regrow writes, which its report lists (`WhatTheScreenRedraws`). The call keeps those cells
/// as they were and writes back the ones that changed; the harness compares the model's whole
/// room with the original's, so a write the model doesn't name is a mismatch there.
struct Call<'a> {
    content: &'a BlockContent,
    state: State,
    /// The cells a step may write, as the call found them; a cell a run wrote that isn't here
    /// is written back whatever it was.
    watched: BTreeMap<(u16, u16), Tile>,
    /// The cells the call's runs wrote.
    written: Vec<Cell>,
    /// The screens as the call found them.
    screens: Grid<ScreenKind>,
}

impl<'a> Call<'a> {
    /// The call's State of `room`, the places outside `places` another part's.
    fn new(
        room: &RoomScoped,
        reads: Reads<'a>,
        places: &Range<usize>,
    ) -> Result<Self, BlocksFault> {
        let state = state(room, reads.content, reads.population, places)?;
        Ok(Self {
            content: reads.content,
            watched: BTreeMap::new(),
            written: Vec::new(),
            screens: state.screens.clone(),
            state,
        })
    }

    /// Keep `cells` as they are now, before a step that may write them.
    fn watch(&mut self, cells: impl IntoIterator<Item = Cell>) {
        for at in cells {
            if let (Some(tile), std::collections::btree_map::Entry::Vacant(entry)) =
                (self.state.tiles.get(at), self.watched.entry((at.x, at.y)))
            {
                entry.insert(tile.clone());
            }
        }
    }

    /// One step of the model: `step` changes the State, and each place it changes is written
    /// into `room`'s slot as `phase` makes it. Returns the places another part's start took, and
    /// what the step returns.
    fn step<T>(
        &mut self,
        room: &mut RoomScoped,
        phase: Phase,
        step: impl FnOnce(&mut State) -> Result<T, OutsideTheRoom>,
    ) -> Result<(Vec<usize>, T), BlocksFault> {
        let before = self.state.places.clone();
        let stepped = step(&mut self.state)?;
        let change = Change {
            content: self.content,
            width: room.width.0,
            phase,
        };
        let mut taken = Vec::new();
        let places = self.state.places.0.iter().zip(&before.0);
        for (number, ((after, before), slot)) in places.zip(room.plms.slots.iter_mut()).enumerate()
        {
            if phase == Phase::Run && !matches!(before, Place::Free | Place::OtherPart) {
                // The pass counts every live place's timer down, and a sleeping scroll reaction's
                // wraps (`$84:85E0`); a place still running waits its reaction's wait.
                slot.common.cursor.delay = match after {
                    Place::Running(running) => Timer(running.wait.count()),
                    Place::Free | Place::Scroll(_) | Place::OtherPart | Place::HeldUntilRun => {
                        Timer(slot.common.cursor.delay.0.wrapping_sub(1))
                    }
                };
            }
            if after == before {
                continue;
            }
            if matches!((before, after), (Place::Free, Place::OtherPart)) {
                taken.push(number);
                continue;
            }
            change
                .place(slot, before, after)
                .ok_or_else(|| place_fault(number))?;
        }
        Ok((taken, stepped))
    }

    /// One of the frame's requests, its run paused: the run is the PLM pass's. Its cell is one of
    /// the room's, or the call stops.
    fn request(&mut self, room: &mut RoomScoped, request: Request) -> Result<(), BlocksFault> {
        let Some(at) = request_cell(request) else {
            return Ok(());
        };
        let block_index = index(at, room.width.0);
        let hold =
            match self.state.tiles.get(at).map(|tile| &tile.mark) {
                Some(Mark::Scroll) => {
                    let placed =
                        self.state.places.0.iter().any(
                            |place| matches!(place, Place::Scroll(scroll) if scroll.cell == at),
                        );
                    if !placed {
                        return Err(BlocksFault::NoScrollReaction(block_index));
                    }
                    HoldBody::ScrollTouch
                }
                Some(_) => HoldBody::Held,
                None => return Err(BlocksFault::Outside(at)),
            };
        self.watch([at]);
        self.step(room, Phase::Request(block_index, hold), |state| {
            state.step(rules(), &[request], Set::default(), Run::Paused)
        })?;
        Ok(())
    }

    /// Write the cells and the screens the call changed into `room`.
    fn write_back(&self, room: &mut RoomScoped) -> Result<(), BlocksFault> {
        let width = room.width.0;
        let watched = self.watched.keys().map(|&(x, y)| Cell { x, y });
        for at in watched.chain(self.written.iter().copied()) {
            let Some(after) = self.state.tiles.get(at) else {
                continue;
            };
            if self.watched.get(&(at.x, at.y)) == Some(after) {
                continue;
            }
            let index = index(at, width);
            if let Some(cell) = room.blocks.get_mut(usize::from(index.0)) {
                *cell = block(*cell, after, index)?;
            }
        }
        let across = screens_across(room);
        let rows = self.state.screens.rows().iter().zip(self.screens.rows());
        for (y, (after, before)) in (0_u16..).zip(rows) {
            for (x, (&after, &before)) in (0_u16..).zip(after.iter().zip(before)) {
                if after == before {
                    continue;
                }
                let at = usize::from(y.wrapping_mul(across).wrapping_add(x));
                if let Some(area) = room.scroll_areas.get_mut(at) {
                    *area = scroll_area(after);
                }
            }
        }
        Ok(())
    }
}

/// What a place's change writes into its slot.
#[derive(Clone, Copy)]
struct Change<'a> {
    content: &'a BlockContent,
    width: u16,
    phase: Phase,
}

impl Change<'_> {
    /// Write the place's change from `before` to `after` into `slot`; `None` for a change no
    /// step of this phase makes.
    fn place(
        self,
        slot: &mut Slot<PlmCommon, PlmBody>,
        before: &Place,
        after: &Place,
    ) -> Option<()> {
        let block_of = |at: Cell| index(at, self.width);
        match (before, after) {
            (_, Place::Free) => slot.occupancy = Occupancy::Free,
            (Place::Free, Place::Running(running)) => {
                let body = reaction_place(running, self.content)?;
                fill(
                    slot,
                    block_of(running.cell),
                    Occupancy::Live(PlmBody::Reaction(body)),
                );
                slot.common.cursor.delay = Timer(running.wait.count());
            }
            (Place::Running(_), Place::Running(running)) => {
                let body = reaction_place(running, self.content)?;
                slot.occupancy = Occupancy::Live(PlmBody::Reaction(body));
            }
            (Place::Free, Place::HeldUntilRun) => match self.phase {
                Phase::Request(_, HoldBody::ScrollTouch) => {
                    fill(slot, BlockIndex(0), Occupancy::Live(PlmBody::ScrollTouch));
                }
                Phase::Request(at, HoldBody::Held) => {
                    fill(slot, at, Occupancy::Live(PlmBody::Held));
                }
                Phase::RoomStart(_) | Phase::Run => return None,
            },
            (Place::Free, Place::Scroll(scroll)) => {
                let Phase::RoomStart(Some(changes)) = self.phase else {
                    return None;
                };
                let body = ScrollPlm {
                    changes,
                    triggered: false,
                    step: ScrollStep::Sleep,
                };
                fill(
                    slot,
                    block_of(scroll.cell),
                    Occupancy::Live(PlmBody::Scroll(body)),
                );
            }
            (Place::Scroll(_), Place::Scroll(scroll)) => {
                let Occupancy::Live(PlmBody::Scroll(plm)) = &mut slot.occupancy else {
                    return None;
                };
                let touched = scroll.touched == Touched::Yes;
                plm.triggered = touched;
                plm.step = if touched {
                    ScrollStep::Update
                } else {
                    ScrollStep::Sleep
                };
                if touched {
                    // The touch's setup moves the scroll PLM past its sleep with a timer of 1.
                    slot.common.cursor.delay = Timer(1);
                }
            }
            (
                Place::Free
                | Place::Running(_)
                | Place::Scroll(_)
                | Place::OtherPart
                | Place::HeldUntilRun,
                Place::Running(_) | Place::Scroll(_) | Place::OtherPart | Place::HeldUntilRun,
            ) => return None,
        }
        Some(())
    }
}

/// The requests `asked`, in order, on one State of the room, then the cells written back.
fn requests(
    room: &mut RoomScoped,
    reads: Reads<'_>,
    asked: impl IntoIterator<Item = Request>,
) -> Result<(), BlocksFault> {
    let mut call = Call::new(room, reads, &(0..room.plms.slots.len()))?;
    for request in asked {
        call.request(room, request)?;
    }
    call.write_back(room)
}

/// `ScrollCells`, `BlocksMarkedNothing`: the cells she is inside, bottom first, each with its
/// point, a stand-in already followed to the cell it stands for.
///
/// # Errors
///
/// A [`BlocksFault`] where a cell is one the room's loading would refuse.
pub fn inside(
    room: &mut RoomScoped,
    reads: Reads<'_>,
    points: &[(BlockIndex, Point)],
) -> Result<(), BlocksFault> {
    if points.is_empty() {
        return Ok(());
    }
    let width = room.width.0;
    requests(
        room,
        reads,
        points.iter().map(|&(block, point)| Request::Inside {
            cell: cell(block, width),
            point,
        }),
    )
}

/// Spec §5: a weapon `by` reaches the cell at `block`.
///
/// # Errors
///
/// A [`BlocksFault`] where the cell is one the room's loading would refuse.
pub fn hit(
    room: &mut RoomScoped,
    reads: Reads<'_>,
    block: BlockIndex,
    by: HitBy,
) -> Result<(), BlocksFault> {
    let at = cell(block, room.width.0);
    requests(room, reads, [Request::Hit { cell: at, by }])
}

/// `AnswersToHerMotion`: the cells her moves or looks reached, in order, each stand-in already
/// followed.
///
/// # Errors
///
/// A [`BlocksFault`] where a cell is one the room's loading would refuse.
pub fn touches(
    room: &mut RoomScoped,
    reads: Reads<'_>,
    touches: &[Touch],
) -> Result<(), BlocksFault> {
    requests(room, reads, touches.iter().copied().map(Request::Touch))
}

/// `TheScriptsRun`: the run of the places in `places`, from the highest down, the cells it
/// writes queued in `drawing`.
///
/// # Errors
///
/// A [`BlocksFault`] where a place or a cell is one the model isn't defined on.
pub fn run(
    drawing: &mut Drawing,
    room: &mut RoomScoped,
    reads: Reads<'_>,
    places: Range<usize>,
) -> Result<(), BlocksFault> {
    let free = room
        .plms
        .slots
        .get(places.clone())
        .is_none_or(|slots| slots.iter().all(|slot| slot.occupancy == Occupancy::Free));
    if free {
        // Free places hold nothing to run.
        return Ok(());
    }
    let mut call = Call::new(room, reads, &places)?;
    let covered: Vec<Cell> = reads
        .content
        .coverings(&call.state.places)
        .into_iter()
        .map(|(at, _)| at)
        .collect();
    call.watch(covered);
    let (_, report) = call.step(room, Phase::Run, |state| {
        state.step(rules(), &[], Set::default(), Run::Runs)
    })?;
    call.written.extend(report.written.iter().copied());
    call.write_back(room)?;
    let width = room.width.0;
    for &at in &report.written {
        let block = index(at, width);
        if let Some(cell) = room.blocks.get(usize::from(block.0)) {
            drawing.block(room, block, cell.tile);
        }
    }
    Ok(())
}

/// The model's placement of a population entry, in a room `across` screens wide; `None` for one
/// of a part no model runs yet.
fn placement(entry: PlmEntry, population: &PlmPopulation, across: u16) -> Option<Placement> {
    let at = Cell {
        x: u16::from(entry.x),
        y: u16::from(entry.y),
    };
    Some(match entry.kind {
        PlmKind::Scroll(id) => Placement::Scroll {
            cell: at,
            changes: changes(population, id, across).unwrap_or_default(),
        },
        PlmKind::Extension(extension) => {
            let (way, reach) = match extension {
                Extension::Right => (StandIn::Across, -1),
                Extension::Left => (StandIn::Across, 1),
                Extension::Down => (StandIn::Down, -1),
                Extension::Up => (StandIn::Down, 1),
            };
            Placement::StandIn {
                cell: at,
                way,
                reach,
            }
        }
        PlmKind::Item(..) => Placement::OtherPart,
        // open: the PLMs no slice 0 room state that loads places; each joins with its part.
        PlmKind::Unmodelled(..) => return None,
    })
}

/// `RoomStart`: the room's population in its order on one State of the room.
///
/// The scroll cells and stand-ins are the model's. Returns, for each entry in order, the place
/// another part's placement took, for its part to fill.
///
/// A stand-in's place is free again at once, but the spawn leaves its cell and timer in the
/// slot, as every spawn does.
///
/// # Errors
///
/// A [`BlocksFault`] where a placement's cell is one the room's loading would refuse.
pub fn start_room(
    room: &mut RoomScoped,
    reads: Reads<'_>,
) -> Result<Vec<Option<usize>>, BlocksFault> {
    let mut call = Call::new(room, reads, &(0..room.plms.slots.len()))?;
    let across = screens_across(room);
    let mut taken = Vec::new();
    for entry in &reads.population.entries {
        let Some(placement) = placement(*entry, reads.population, across) else {
            taken.push(None);
            continue;
        };
        let free = call
            .state
            .places
            .0
            .iter()
            .rposition(|place| *place == Place::Free);
        let named = match entry.kind {
            PlmKind::Scroll(id) => Some(id),
            PlmKind::Extension(_) | PlmKind::Item(..) | PlmKind::Unmodelled(..) => None,
        };
        call.watch(placement_cell(&placement));
        let (took, ()) = call.step(room, Phase::RoomStart(named), |state| {
            state.start_room(core::slice::from_ref(&placement))
        })?;
        if let (Placement::StandIn { cell: at, .. }, Some(slot)) = (
            &placement,
            free.and_then(|free| room.plms.slots.get_mut(free)),
        ) {
            fill(slot, index(*at, room.width.0), Occupancy::Free);
        }
        taken.push(took.first().copied());
    }
    call.write_back(room)?;
    Ok(taken)
}

/// Invented content for the engine's tests: a scroll cell's mark, a shot reaction that keeps its
/// look and a crumble reaction that grows back.
#[cfg(test)]
pub(crate) mod fixture {
    use super::{
        Becomes, BlockContent, Frames, PatternCell, Reaction, ReactionName, SoundRequest, Step,
    };
    use clean_types::{Family, Flip, Look, Pair, Picture, Script};
    use data::blocks::{Mark, ReactionId, SCROLL_CELL};
    use data::level::{BlockKind, PlmChoice};
    use enum_map::EnumMap;

    /// A shootable block with BTS 4, the invented shot block.
    pub const SHOT: BlockKind = BlockKind::Shootable(PlmChoice::Common(4));
    /// A special block with BTS 1, the invented crumble block.
    pub const CRUMBLE: BlockKind = BlockKind::Special(PlmChoice::Common(1));

    /// Picture `index`, unflipped.
    pub fn look(index: u16) -> Look {
        Look {
            picture: Picture::new(index).unwrap(),
            flip: Flip::None,
        }
    }

    /// The cell itself becoming air with picture `index` for `frames`.
    fn show(index: u16, frames: u16) -> Step {
        Step::Show {
            pattern: vec![PatternCell {
                offset: Pair { x: 0, y: 0 },
                becomes: Becomes::Air,
                look: look(index),
            }],
            frames: Frames::new(frames).unwrap(),
        }
    }

    fn reaction(
        name: &str,
        family: Family,
        whole_look: Option<u16>,
        wait: u16,
        broken: Vec<Step>,
    ) -> Reaction {
        let mut scripts: EnumMap<Script, Vec<Step>> = EnumMap::default();
        scripts[Script::Broken] = broken;
        Reaction {
            name: ReactionName(name.to_owned()),
            family,
            whole_look: whole_look.map(|index| Picture::new(index).unwrap()),
            first_wait: Frames::new(wait).unwrap(),
            scripts,
        }
    }

    /// The content: `shot` asks for a sound, shows 12h for 2 frames and 13h for 1, and ends;
    /// `crumble`, whole as 20h, waits 3, shows 21h for 2, grows back and ends.
    pub fn content() -> BlockContent {
        let sound = Step::Sound(SoundRequest {
            sound: "a sound".to_owned(),
            queue_limit: 1,
        });
        BlockContent::invented(
            vec![
                reaction(
                    "shot",
                    Family::Shot,
                    None,
                    1,
                    vec![sound, show(0x12, 2), show(0x13, 1), Step::End],
                ),
                reaction(
                    "crumble",
                    Family::Crumble,
                    Some(0x20),
                    3,
                    vec![show(0x21, 2), Step::Regrow, Step::End],
                ),
            ],
            &[
                (SCROLL_CELL, Mark::Scroll),
                (SHOT, Mark::Reaction(ReactionId(0))),
                (CRUMBLE, Mark::Reaction(ReactionId(1))),
            ],
        )
        .unwrap()
    }
}

#[cfg(test)]
mod tests;
