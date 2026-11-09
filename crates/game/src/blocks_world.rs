//! The blocks as the engine meets them: the blocks model (`clean_model_blocks`).
//!
//! The engine keeps the room's cells as the game keeps them, in `RoomScoped::blocks` (each cell's
//! type, BTS and tile), and the reactions' places in its PLM slots beside the items'. Each call
//! builds the model's State from those, steps it, and writes back what it changed
//! (`docs/behaviours/blocks.decode.md`, "State"):
//! - the room's start: its scroll cells and stand-ins ([`start_room`]);
//! - each request, in the frame's order: the cells she is inside ([`inside`]), her beam's hits
//!   ([`hit`]) and her moves' touches ([`touch`]);
//! - the run, in the PLM pass, between the items' places ([`run`]).
//!
//! A call hands the model the rows of cells it can reach, a request's own row and the rows a
//! running reaction's patterns cover; the other rows are empty, which the model reads as past the
//! room, and which no request or pattern of the call reaches.
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
//! - the sounds the scripts ask for have no sound queue to go to (U1);
//! - another part's start is an item's, which `plm` keeps in its own places.

use core::ops::Range;

use clean_model_blocks::{
    Becomes, Blocks, CellKind, Frames, Grid, Inside, Mark, PatternCell, Place, Placement, Reaction,
    ReactionName, Reactions, Request, Rules, Run, Running, ScreenChange, ScrollReaction,
    SoundRequest, StandIn, State, Step, Tile, Touched,
};
use clean_types::{Cell, Family, Flip, HitBy, Look, MetAs, Pair, Picture, Point, ScreenKind};
use clean_types::{Script, Set, Touch};
use data::blocks::{self as asset, ReactionId, SCROLL_CELL};
use data::level::{Block, BlockKind, Bts, Offset, ScrollArea, TileNumber};
use data::plm::{Extension, PlmPopulation, ScrollChangesId};
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
    /// A reacting cell some asker of which reads past its table's end: the importer refuses it
    /// (LEGACY, "Reacting cells whose tables end before their mark").
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
}

/// The model, its rules the game's and its reactions the content's.
struct Model(Blocks);

impl Model {
    const fn new(reactions: Reactions) -> Self {
        Self(Blocks {
            rules: Rules::REFERENCE,
            reactions,
        })
    }
}

impl Clone for Model {
    fn clone(&self) -> Self {
        Self::new(self.0.reactions.clone())
    }
}

impl core::fmt::Debug for Model {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Model").field(&self.0.reactions).finish()
    }
}

impl PartialEq for Model {
    fn eq(&self, other: &Self) -> bool {
        self.0.reactions == other.0.reactions
    }
}

impl Eq for Model {}

impl Default for Model {
    fn default() -> Self {
        Self::new(Reactions::new())
    }
}

/// A reacting cell's mark as the content types it, or the importer's refusal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Typed {
    Mark(asset::Mark),
    Refused,
}

/// An imported reaction as the engine keeps it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Imported {
    /// The model's name for it.
    name: ReactionName,
    /// Its family, which names the block type of its cells.
    family: Family,
    /// The rows its patterns cover, from its cell's.
    rows: Vec<i16>,
    /// The cells its patterns cover other than its own.
    others: Vec<Pair<i16>>,
}

/// The blocks' content: the model over the imported reactions, and what each reacting cell's
/// mark is, by its area, block type and BTS.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlockContent {
    /// The reactions as the importer gave them.
    imported: asset::Reactions,
    model: Model,
    /// By [`ReactionId`].
    reactions: Vec<Imported>,
    /// By area, then the reacting types' order (`REACTING`, blocks then air), then BTS.
    marks: Vec<Vec<Typed>>,
    /// By area, then BTS: whether she holds a place inside special air.
    inside: Vec<Vec<bool>>,
}

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

/// The model's step of an imported one.
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
                        asset::Becomes::Itself => Becomes::Itself,
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

/// The cells a reaction's patterns cover, past its own.
fn covered(reaction: &Reaction) -> Vec<Pair<i16>> {
    let mut cells: Vec<Pair<i16>> = reaction
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
    cells.sort_unstable_by_key(|offset| (offset.y, offset.x));
    cells.dedup();
    cells
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

/// The place of a reacting type in `REACTING`'s order, blocks first.
fn class_place(nybble: u8) -> Option<usize> {
    let blocks = REACTING.iter().position(|&(block, _)| block == nybble);
    let air = REACTING.iter().position(|&(_, air)| air == nybble);
    blocks.or_else(|| air.map(|place| place.wrapping_add(REACTING.len())))
}

impl BlockContent {
    /// The content of the imported `reactions` and their mark tables.
    ///
    /// # Errors
    ///
    /// The id of a reaction whose shows wait longer than the model's frames, which the importer
    /// refuses first.
    pub fn import(reactions: &asset::Reactions) -> Result<Self, ReactionId> {
        let mut model = Reactions::new();
        let mut imported = Vec::new();
        for (reaction, index) in reactions.all().iter().zip(0_u16..) {
            let id = ReactionId(index);
            let converted = convert(reaction).ok_or(id)?;
            let others = covered(&converted);
            let mut rows: Vec<i16> = others.iter().map(|offset| offset.y).collect();
            rows.push(0);
            rows.sort_unstable();
            rows.dedup();
            let name = ReactionName(reaction.name.to_owned());
            imported.push(Imported {
                name: name.clone(),
                family: reaction.family,
                rows,
                others,
            });
            model.insert(name, converted);
        }
        let marks = AREAS
            .iter()
            .map(|&area| {
                let blocks = REACTING.iter().map(|&(block, _)| block);
                let air = REACTING.iter().map(|&(_, air)| air);
                blocks
                    .chain(air)
                    .flat_map(|nybble| (0..=u8::MAX).map(move |bts| (nybble, bts)))
                    .map(
                        |(nybble, bts)| match reactions.mark(BlockKind::new(nybble, bts), area) {
                            Ok(Some(mark)) => Typed::Mark(mark),
                            Ok(None) | Err(_) => Typed::Refused,
                        },
                    )
                    .collect()
            })
            .collect();
        let inside = AREAS
            .iter()
            .map(|&area| {
                (0..=u8::MAX)
                    .map(|bts| reactions.holds_place_inside(BlockKind::SpecialAir(Bts(bts)), area))
                    .collect()
            })
            .collect();
        Ok(Self {
            imported: reactions.clone(),
            model: Model::new(model),
            reactions: imported,
            marks,
            inside,
        })
    }

    /// Content of invented reactions, each with its family, and the marks of the cells listed,
    /// in every area; another reacting cell is another part's.
    #[must_use]
    pub fn invented(reactions: Vec<(&str, Reaction)>, marks: &[(BlockKind, asset::Mark)]) -> Self {
        let mut model = Reactions::new();
        let mut imported = Vec::new();
        for (name, reaction) in reactions {
            let others = covered(&reaction);
            let mut rows: Vec<i16> = others.iter().map(|offset| offset.y).collect();
            rows.push(0);
            rows.sort_unstable();
            rows.dedup();
            let name = ReactionName(name.to_owned());
            imported.push(Imported {
                name: name.clone(),
                family: reaction.family,
                rows,
                others,
            });
            model.insert(name, reaction);
        }
        let mut row = vec![Typed::Mark(asset::Mark::Other(None)); MARK_ROW];
        for &(kind, mark) in marks {
            if let Some(entry) = class_place(kind.nybble()).and_then(|place| {
                row.get_mut(
                    place
                        .wrapping_mul(BTS_VALUES)
                        .wrapping_add(usize::from(kind.bts())),
                )
            }) {
                *entry = Typed::Mark(mark);
            }
        }
        Self {
            imported: asset::Reactions::default(),
            model: Model::new(model),
            reactions: imported,
            marks: vec![row; AREAS.len()],
            inside: vec![vec![false; BTS_VALUES]; AREAS.len()],
        }
    }

    /// The reactions as the importer gave them, which a record decoder reads places by.
    #[must_use]
    pub const fn imported(&self) -> &asset::Reactions {
        &self.imported
    }

    /// The model, over the content's reactions with the game's rules.
    #[must_use]
    pub const fn blocks(&self) -> &Blocks {
        &self.model.0
    }

    /// The reaction `id` names.
    fn reaction(&self, id: ReactionId) -> Option<&Imported> {
        self.reactions.get(usize::from(id.0))
    }

    /// The model's name of the reaction `id`, and its family.
    #[must_use]
    pub fn named(&self, id: ReactionId) -> Option<(&ReactionName, Family)> {
        self.reaction(id)
            .map(|reaction| (&reaction.name, reaction.family))
    }

    /// The cells the patterns of reaction `id` cover other than its own, from its cell.
    #[must_use]
    pub fn covered(&self, id: ReactionId) -> &[Pair<i16>] {
        self.reaction(id)
            .map_or(&[][..], |reaction| reaction.others.as_slice())
    }

    /// The id of the reaction the model names `name`.
    #[must_use]
    pub fn id(&self, name: &ReactionName) -> Option<ReactionId> {
        self.reactions
            .iter()
            .position(|reaction| reaction.name == *name)
            .and_then(|index| u16::try_from(index).ok())
            .map(ReactionId)
    }

    /// The typed mark of a reacting cell of `kind` in `area`; `None` for another kind.
    fn typed(&self, kind: BlockKind, area: Area) -> Option<Typed> {
        let place = class_place(kind.nybble())?;
        let row = AREAS.iter().position(|&known| known == area)?;
        self.marks
            .get(row)?
            .get(
                place
                    .wrapping_mul(BTS_VALUES)
                    .wrapping_add(usize::from(kind.bts())),
            )
            .copied()
    }

    /// The model's mark of an imported one.
    fn mark(&self, mark: asset::Mark) -> Mark {
        match mark {
            asset::Mark::Reaction(id) => self.reaction(id).map_or(Mark::Other(None), |reaction| {
                Mark::Reaction(reaction.name.clone())
            }),
            asset::Mark::Scroll => Mark::Scroll,
            asset::Mark::Nothing(met) => Mark::Nothing(met),
            asset::Mark::Inert => Mark::Inert,
            asset::Mark::Other(met) => Mark::Other(met),
        }
    }

    /// Whether she holds a place inside special air with `bts` in `area`.
    fn holds_inside(&self, bts: u8, area: Area) -> bool {
        AREAS
            .iter()
            .position(|&known| known == area)
            .and_then(|row| self.inside.get(row))
            .and_then(|row| row.get(usize::from(bts)))
            .copied()
            .unwrap_or(false)
    }

    /// The model's tile of a cell of `block` in `area`, `covering` naming the reaction running on
    /// an air or solid cell: `None` for a cell the importer refuses.
    #[must_use]
    pub fn tile(&self, block: Block, area: Area, covering: Option<Covering>) -> Option<Tile> {
        let kind = block.kind;
        let (cell, mark) = match kind {
            BlockKind::Air(_) => (CellKind::Air, self.covering_mark(kind, area, covering)?),
            BlockKind::Solid(_) => (CellKind::Solid, self.covering_mark(kind, area, covering)?),
            BlockKind::HorizontalExtension(Offset(by)) => {
                (CellKind::StandInAcross, Mark::Reach(i16::from(by)))
            }
            BlockKind::VerticalExtension(Offset(by)) => {
                (CellKind::StandInDown, Mark::Reach(i16::from(by)))
            }
            BlockKind::Special(_) | BlockKind::Shootable(_) | BlockKind::Bombable(_) => {
                (CellKind::Block, self.reacting_mark(kind, area)?)
            }
            BlockKind::SpecialAir(_) | BlockKind::ShootableAir(_) | BlockKind::BombableAir(_) => {
                (CellKind::ReactingAir, self.reacting_mark(kind, area)?)
            }
            // The other domains' kinds, by how they meet her: none reacts here.
            BlockKind::Slope(_)
            | BlockKind::Door(_)
            | BlockKind::Spike(_)
            | BlockKind::Grapple(_) => (CellKind::Solid, Mark::Unmarked),
            BlockKind::SpikeAir(_) | BlockKind::UnusedAir(_) => (CellKind::Air, Mark::Unmarked),
        };
        let holds = matches!(kind, BlockKind::SpecialAir(Bts(bts)) if self.holds_inside(bts, area));
        let inside = if holds {
            Inside::HoldsPlace
        } else {
            Inside::Passes
        };
        Some(Tile {
            kind: cell,
            mark,
            look: look(block.tile)?,
            inside,
        })
    }

    /// A reacting cell's mark; `None` where the importer refuses it.
    fn reacting_mark(&self, kind: BlockKind, area: Area) -> Option<Mark> {
        match self.typed(kind, area)? {
            Typed::Mark(mark) => Some(self.mark(mark)),
            Typed::Refused => None,
        }
    }

    /// The mark of an air or solid cell: none, unless a reaction runs on it, whose own cell's mark
    /// is its BTS as the reaction's family's block reads it, and whose other cells' a reach.
    fn covering_mark(
        &self,
        kind: BlockKind,
        area: Area,
        covering: Option<Covering>,
    ) -> Option<Mark> {
        Some(match covering {
            None => Mark::Unmarked,
            Some(Covering::Own(family)) => {
                let (block, _) = asset::Reactions::class(family);
                self.reacting_mark(BlockKind::new(block, kind.bts()), area)?
            }
            Some(Covering::Other) => Mark::Reach(i16::from(i8::from_le_bytes([kind.bts()]))),
        })
    }
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
        family: reaction.family,
        whole_look: reaction.whole_look,
        first_wait: Frames::new(reaction.first_wait)?,
        scripts,
    })
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

/// What a call is, for what its places' changes write back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    /// The room's start, with the change list a scroll cell placed names.
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

/// The model's State over the rows a call reaches, with what was built.
struct View {
    state: State,
    before: State,
    rows: Vec<u16>,
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
            reaction: content.reaction(running.reaction)?.name.clone(),
            script: script(running.script),
            cell: at,
            next: usize::from(running.next),
            wait: Frames::new(slot.common.cursor.delay.0)?,
            grows_back_as: running.grows_back_as.map(|kind| match kind {
                GrowsBackAs::Block => clean_model_blocks::Reacting::Block,
                GrowsBackAs::Air => clean_model_blocks::Reacting::Air,
            }),
        }),
    })
}

/// The screens a scroll PLM's change list `id` changes, each at its place in a room `across`
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

/// The rows `cells` and the running reactions in `places` reach.
fn reached_rows(
    room: &RoomScoped,
    content: &BlockContent,
    places: Range<usize>,
    cells: &[Cell],
) -> Vec<u16> {
    let width = room.width.0;
    let mut rows: Vec<u16> = cells.iter().map(|cell| cell.y).collect();
    for slot in room.plms.slots.get(places).into_iter().flatten() {
        if let Occupancy::Live(PlmBody::Reaction(running)) = slot.occupancy
            && let Some(reaction) = content.reaction(running.reaction)
        {
            let at = cell(slot.common.block, width);
            rows.extend(
                reaction
                    .rows
                    .iter()
                    .filter_map(|&dy| at.y.checked_add_signed(dy)),
            );
        }
    }
    rows.sort_unstable();
    rows.dedup();
    rows
}

/// What runs on each air or solid cell of the room: each running reaction's own cell, then the
/// other cells its patterns cover.
fn coverings(room: &RoomScoped, content: &BlockContent) -> Vec<(Cell, Covering)> {
    let width = room.width.0;
    let running = room
        .plms
        .slots
        .iter()
        .filter_map(|slot| match slot.occupancy {
            Occupancy::Live(PlmBody::Reaction(running)) => Some((
                cell(slot.common.block, width),
                content.reaction(running.reaction)?,
            )),
            Occupancy::Live(_) | Occupancy::Free | Occupancy::Deleting(_) => None,
        });
    let mut own = Vec::new();
    let mut others = Vec::new();
    for (at, reaction) in running {
        own.push((at, Covering::Own(reaction.family)));
        others.extend(reaction.others.iter().filter_map(|offset| {
            Some((
                Cell {
                    x: at.x.checked_add_signed(offset.x)?,
                    y: at.y.checked_add_signed(offset.y)?,
                },
                Covering::Other,
            ))
        }));
    }
    own.extend(others);
    own
}

/// The model's State of `room` over `rows`, the places outside `places` another part's.
fn view(
    room: &RoomScoped,
    content: &BlockContent,
    population: &PlmPopulation,
    rows: Vec<u16>,
    places: Range<usize>,
) -> Result<View, BlocksFault> {
    let width = room.width.0;
    let height = u16::try_from(
        room.blocks
            .len()
            .checked_div(usize::from(width))
            .unwrap_or(0),
    )
    .unwrap_or(u16::MAX);
    let coverings = if rows.is_empty() {
        Vec::new()
    } else {
        coverings(room, content)
    };
    let mut grid: Vec<Vec<Tile>> = vec![Vec::new(); usize::from(height)];
    for &y in &rows {
        let Some(row) = grid.get_mut(usize::from(y)) else {
            continue;
        };
        for x in 0..width {
            let at = Cell { x, y };
            let block_index = index(at, width);
            let Some(&block) = room.blocks.get(usize::from(block_index.0)) else {
                continue;
            };
            let covering = coverings
                .iter()
                .find_map(|&(covered, covering)| (covered == at).then_some(covering));
            let tile = content
                .tile(block, room.area, covering)
                .ok_or(BlocksFault::Refused(block_index))?;
            row.push(tile);
        }
    }
    let across = usize::from(screens_across(room));
    let screens = Grid {
        rows: room
            .scroll_areas
            .chunks(across.max(1))
            .filter(|_| across != 0)
            .map(|row| row.iter().map(|&area| screen_kind(area)).collect())
            .collect(),
    };
    let mut model = clean_model_blocks::Places::default();
    for ((index, slot), place_of) in room.plms.slots.iter().enumerate().zip(&mut model.0) {
        *place_of = if places.contains(&index) {
            place(slot, content, population, room)
                .ok_or_else(|| BlocksFault::Place(u8::try_from(index).unwrap_or(u8::MAX)))?
        } else {
            Place::OtherPart
        };
    }
    let state = State {
        tiles: Grid { rows: grid },
        places: model,
        screens,
    };
    Ok(View {
        before: state.clone(),
        state,
        rows,
    })
}

/// The engine's block of the model's `tile`, from what the cell was.
fn block(
    was: Block,
    tile: &Tile,
    content: &BlockContent,
    at: BlockIndex,
) -> Result<Block, BlocksFault> {
    let bts = match tile.mark {
        Mark::Scroll if tile.kind == CellKind::ReactingAir => SCROLL_CELL.bts(),
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
    let reacting = |as_block: bool| -> Result<u8, BlocksFault> {
        let pick = |(block, air): (u8, u8)| if as_block { block } else { air };
        let named = match &tile.mark {
            Mark::Reaction(name) => content
                .id(name)
                .and_then(|id| content.reaction(id))
                .map(|reaction| asset::Reactions::class(reaction.family)),
            Mark::Nothing(met) => Some(asset::Reactions::met_class(*met)),
            Mark::Scroll => Some(asset::Reactions::met_class(MetAs::Crumble)),
            Mark::Unmarked | Mark::Inert | Mark::Other(_) | Mark::Reach(_) => None,
        };
        let kept = REACTING
            .iter()
            .copied()
            .find(|&(block, air)| was.kind.nybble() == block || was.kind.nybble() == air);
        named.or(kept).map(pick).ok_or(BlocksFault::Class(at))
    };
    let nybble = match tile.kind {
        CellKind::Air => 0x0,
        CellKind::Solid => 0x8,
        CellKind::StandInAcross => 0x5,
        CellKind::StandInDown => 0xd,
        CellKind::Block => reacting(true)?,
        CellKind::ReactingAir => reacting(false)?,
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
        grows_back_as: running.grows_back_as.map(|kind| match kind {
            clean_model_blocks::Reacting::Block => GrowsBackAs::Block,
            clean_model_blocks::Reacting::Air => GrowsBackAs::Air,
        }),
    })
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

/// Write the view's changes back into `room`: its cells, screens and places, as `phase` makes
/// them. Returns the places another part's placement took.
fn write_back(
    room: &mut RoomScoped,
    view: &View,
    content: &BlockContent,
    phase: Phase,
) -> Result<Vec<usize>, BlocksFault> {
    write_cells(room, view, content)?;
    write_screens(room, view);
    let width = room.width.0;
    let mut taken = Vec::new();
    let places = view.state.places.0.iter().zip(&view.before.places.0);
    for (number, ((after, before), slot)) in places.zip(room.plms.slots.iter_mut()).enumerate() {
        let fault = BlocksFault::Place(u8::try_from(number).unwrap_or(u8::MAX));
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
        let change = Change {
            content,
            width,
            phase,
        };
        change.place(slot, before, after).ok_or(fault)?;
    }
    Ok(taken)
}

/// The cells the model changed, written back with their types and BTS.
fn write_cells(
    room: &mut RoomScoped,
    view: &View,
    content: &BlockContent,
) -> Result<(), BlocksFault> {
    let width = room.width.0;
    for &y in &view.rows {
        let (Some(after), Some(before)) = (
            view.state.tiles.rows.get(usize::from(y)),
            view.before.tiles.rows.get(usize::from(y)),
        ) else {
            continue;
        };
        for (x, (after, before)) in (0_u16..).zip(after.iter().zip(before)) {
            if after == before {
                continue;
            }
            let at = index(Cell { x, y }, width);
            let Some(cell) = room.blocks.get_mut(usize::from(at.0)) else {
                continue;
            };
            *cell = block(*cell, after, content, at)?;
        }
    }
    Ok(())
}

/// The screens the model changed, written back as scroll areas.
fn write_screens(room: &mut RoomScoped, view: &View) {
    let across = screens_across(room);
    let rows = view
        .state
        .screens
        .rows
        .iter()
        .zip(&view.before.screens.rows);
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
}

/// What a place's change writes into its slot.
#[derive(Clone, Copy)]
struct Change<'a> {
    content: &'a BlockContent,
    width: u16,
    phase: Phase,
}

impl Change<'_> {
    /// Write the place's change from `before` to `after` into `slot`; `None` for a change no call
    /// of this phase makes.
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

/// One request on `cell`, in the frame's order.
fn request(
    room: &mut RoomScoped,
    content: &BlockContent,
    population: &PlmPopulation,
    at: Cell,
    request: Request,
) -> Result<(), BlocksFault> {
    let width = room.width.0;
    let block_index = index(at, width);
    let rows = vec![at.y];
    let mut view = view(room, content, population, rows, 0..room.plms.slots.len())?;
    let hold = match view.state.tiles.get(at).map(|tile| &tile.mark) {
        Some(Mark::Scroll) => {
            let placed = view
                .state
                .places
                .0
                .iter()
                .any(|place| matches!(place, Place::Scroll(scroll) if scroll.cell == at));
            if !placed {
                return Err(BlocksFault::NoScrollReaction(block_index));
            }
            HoldBody::ScrollTouch
        }
        _ => HoldBody::Held,
    };
    content.blocks().step(
        &mut view.state,
        &[],
        &[request],
        Set::default(),
        Run::Paused,
    );
    write_back(room, &view, content, Phase::Request(block_index, hold))?;
    Ok(())
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
    let Reads {
        content,
        population,
    } = reads;
    let width = room.width.0;
    for &(block, point) in points {
        let at = cell(block, width);
        request(
            room,
            content,
            population,
            at,
            Request::Inside { cell: at, point },
        )?;
    }
    Ok(())
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
    let Reads {
        content,
        population,
    } = reads;
    let at = cell(block, room.width.0);
    request(room, content, population, at, Request::Hit { cell: at, by })
}

/// `AnswersToHerMotion`: a cell her move or look reached, its stand-in already followed.
///
/// # Errors
///
/// A [`BlocksFault`] where the cell is one the room's loading would refuse.
pub fn touch(room: &mut RoomScoped, reads: Reads<'_>, touch: Touch) -> Result<(), BlocksFault> {
    let Reads {
        content,
        population,
    } = reads;
    request(room, content, population, touch.cell, Request::Touch(touch))
}

/// `TheScriptsRun`: the run of the places in `places`, from the highest down, its redraws queued
/// in `drawing`.
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
    let Reads {
        content,
        population,
    } = reads;
    let rows = reached_rows(room, content, places.clone(), &[]);
    let mut view = view(room, content, population, rows, places)?;
    let report = content
        .blocks()
        .step(&mut view.state, &[], &[], Set::default(), Run::Runs);
    write_back(room, &view, content, Phase::Run)?;
    let width = room.width.0;
    for &at in &report.redrawn {
        let block = index(at, width);
        if let Some(cell) = room.blocks.get(usize::from(block.0)) {
            drawing.block(room, block, cell.tile);
        }
    }
    Ok(())
}

/// `RoomStart`: the room's placements in its order, the scroll cells and stand-ins the model's,
/// and the places another part's placement takes returned in order for its part to fill.
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
    placement: &data::plm::PlmEntry,
) -> Result<Option<usize>, BlocksFault> {
    let Reads {
        content,
        population,
    } = reads;
    let width = room.width.0;
    let at = Cell {
        x: u16::from(placement.x),
        y: u16::from(placement.y),
    };
    let model = match placement.kind {
        data::plm::PlmKind::Scroll(id) => {
            let changes = changes(population, id, screens_across(room)).unwrap_or_default();
            Placement::Scroll { cell: at, changes }
        }
        data::plm::PlmKind::Extension(extension) => {
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
        data::plm::PlmKind::Item(..) => Placement::OtherPart,
        // open: the PLMs no slice 0 room state that loads places; each joins with its part.
        data::plm::PlmKind::Unmodelled(_) => return Ok(None),
    };
    let free = room
        .plms
        .slots
        .iter()
        .rposition(|slot| matches!(slot.occupancy, Occupancy::Free));
    let mut view = view(
        room,
        content,
        population,
        vec![at.y],
        0..room.plms.slots.len(),
    )?;
    view.state.start_room(core::slice::from_ref(&model));
    let named = match placement.kind {
        data::plm::PlmKind::Scroll(id) => Some(id),
        data::plm::PlmKind::Extension(_)
        | data::plm::PlmKind::Item(..)
        | data::plm::PlmKind::Unmodelled(_) => None,
    };
    let taken = write_back(room, &view, content, Phase::RoomStart(named))?;
    if let (Placement::StandIn { .. }, Some(slot)) =
        (&model, free.and_then(|free| room.plms.slots.get_mut(free)))
    {
        fill(slot, index(at, width), Occupancy::Free);
    }
    Ok(taken.first().copied())
}

/// Invented content for the engine's tests: a scroll cell's mark, a shot reaction that keeps its
/// look and a crumble reaction that grows back.
#[cfg(test)]
pub(crate) mod fixture {
    use super::{Becomes, BlockContent, Frames, PatternCell, Reaction, SoundRequest, Step};
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

    fn reaction(family: Family, whole_look: Option<u16>, wait: u16, broken: Vec<Step>) -> Reaction {
        let mut scripts: EnumMap<Script, Vec<Step>> = EnumMap::default();
        scripts[Script::Broken] = broken;
        Reaction {
            family,
            whole_look: whole_look.map(look),
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
                (
                    "shot",
                    reaction(
                        Family::Shot,
                        None,
                        1,
                        vec![sound, show(0x12, 2), show(0x13, 1), Step::End],
                    ),
                ),
                (
                    "crumble",
                    reaction(
                        Family::Crumble,
                        Some(0x20),
                        3,
                        vec![show(0x21, 2), Step::Regrow, Step::End],
                    ),
                ),
            ],
            &[
                (SCROLL_CELL, Mark::Scroll),
                (SHOT, Mark::Reaction(ReactionId(0))),
                (CRUMBLE, Mark::Reaction(ReactionId(1))),
            ],
        )
    }
}

#[cfg(test)]
mod tests;
