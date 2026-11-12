//! The blocks' reactions as the game's content gives them, in the blocks spec's format.
//!
//! The format is `docs/behaviours/world-blocks.md` §2: each reaction's family, whole look, first wait
//! and its three scripts; and what each reacting cell's mark is, by its block type, BTS and area.
//!
//! This is the importer the spec names: it is the only code that knows the PLMs' layouts.
//! The decoder's maps, which PLM runs which reaction's script and where in its list each step
//! is, are kept beside the reactions for the verification's record decoder.

use canonical::Canonical;
use types::{Family, Flip, Look, MetAs, Pair, Picture, Script};

use crate::level::{BlockKind, Bts};
use crate::rom::{AddressError, Lorom, RomAddress};
use crate::room::Area;

/// The PLMs' bank.
const PLM_BANK: u8 = 0x84;
/// The block collision tables' bank.
const COLLISION_BANK: u8 = 0x94;

/// A cell's ten bits of picture, then its two flips, then its block type.
const PICTURE: u16 = 0x03ff;
const FLIP_ACROSS: u16 = 0x0400;
const FLIP_DOWN: u16 = 0x0800;
const TYPE_SHIFT: u16 = 12;
/// A draw instruction's header: bit 15 a column, else a row; the low byte its count.
const COLUMN: u16 = 0x8000;
/// A BTS's high bit: the area's table rather than the common one.
const AREA_TABLE: u8 = 0x80;
/// The word that ends a draw instruction.
const DRAW_END: u16 = 0;
/// A word below this in a list is a draw's frames.
const INSTRUCTION: u16 = 0x8000;
/// The most frames a show may wait: the spec allows 1 to 32,767.
const MOST_FRAMES: u16 = 0x7fff;
/// A spawn's first wait (`$84:84C5`): one frame, unless the setup sets another.
const SPAWN_WAIT: u16 = 1;

/// Instructions of the lists (bank $84 log).
mod op {
    // Go to (`$84:8724`).
    pub const GO_TO: u16 = 0x8724;
    // Draw the PLM's block, its respawn block written back (`$84:8B17`, `$84:8B05`).
    pub const REGROW: [u16; 2] = [0x8b17, 0x8b05];
    // Delete (`$84:86BC`).
    pub const END: u16 = 0x86bc;
    // Queue a sound: its library and queue limit (`$84:8C07`.., `$84:8C7C`).
    pub const SOUNDS: [(u16, u8, u16); 5] = [
        (0x8c07, 1, 6),
        (0x8c10, 2, 6),
        (0x8c19, 3, 6),
        (0x8c46, 2, 3),
        (0x8c7c, 2, 1),
    ];
}

/// The PLMs that react to nothing and only hold their place until the next pass (`$84:B62F`,
/// `$B633`, `$B653`..`$B65B`).
const NOTHING: [u16; 5] = [0xb62f, 0xb633, 0xb653, 0xb657, 0xb65b];
/// The PLMs a cell she is inside holds a place with (`$94:9B16`'s area tables).
const INSIDE_NOTHING: [u16; 4] = [0xb62f, 0xb653, 0xb657, 0xb65b];
/// The scroll block touch PLM (`$84:B6FF`).
const SCROLL_TOUCH: u16 = 0xb6ff;

/// The setups that start this part's reactions, with their family, where the whole look is
/// in their code (the operand of the `ORA` that builds the respawn block) and where a first
/// wait other than a spawn's is (`LDA #n` before the timer's store).
struct Setup {
    family: Family,
    whole_look: Option<u16>,
    first_wait: Option<u16>,
}

/// The crumble blocks' setup (`$84:CE37`).
const CRUMBLE: Setup = Setup {
    family: Family::Crumble,
    whole_look: Some(0xce4d),
    first_wait: Some(0xce5b),
};
/// The speed blocks' (`$84:CDEA`).
const SPEED: Setup = Setup {
    family: Family::Speed,
    whole_look: Some(0xce29),
    first_wait: None,
};
/// The bomb blocks' hit setup (`$84:CEDA`), whose whole look its touched setup's (`$84:CE83`)
/// shares.
const BOMB_HIT: Setup = Setup {
    family: Family::Bomb,
    whole_look: Some(0xceff),
    first_wait: None,
};
/// The shot blocks that grow back (`$84:CE6B`) and those that keep their look (`$84:B3C1`).
const SHOT_REGROWS: Setup = Setup {
    family: Family::Shot,
    whole_look: Some(0xce76),
    first_wait: None,
};
const SHOT: Setup = Setup {
    family: Family::Shot,
    whole_look: None,
    first_wait: None,
};
/// The power bomb blocks' (`$84:CF2E`) and the super missile blocks' (`$84:CF67`).
const POWER_BOMB: Setup = Setup {
    family: Family::PowerBomb,
    whole_look: Some(0xcf53),
    first_wait: None,
};
const SUPER_MISSILE: Setup = Setup {
    family: Family::SuperMissile,
    whole_look: Some(0xcf8c),
    first_wait: None,
};

/// Each reaction: its name, its setup, the PLM that runs its `broken` script, and where its
/// `revealed` and `touched` scripts come from.
struct Entry {
    name: &'static str,
    setup: Setup,
    plm: u16,
    revealed: Revealed,
    /// A bomb block's: the PLM its touch spawns, whose list is its `touched` script; its
    /// `broken` script is then the hit's, `plm`.
    touched: Option<u16>,
}

/// Where a reaction's `revealed` script is.
#[derive(Clone, Copy)]
enum Revealed {
    /// It has none.
    None,
    /// The list of this bombed special block PLM, by its shape.
    Plm(u16),
    /// This list.
    List(u16),
}

/// Each bombed special block's PLM, by the shape it reveals.
const REVEAL_1X1: u16 = 0xcffc;
const REVEAL_2X1: u16 = 0xd000;
const REVEAL_1X2: u16 = 0xd004;
const REVEAL_2X2: u16 = 0xd008;
const REVEAL_SPEED: u16 = 0xd024;
/// The power bomb blocks' and the super missile blocks' revealed lists (`$84:C91C`, `$C922`).
const REVEAL_POWER_BOMB: u16 = 0xc91c;
const REVEAL_SUPER: u16 = 0xc922;

/// A bomb block's hit PLM lies this far past its touch PLM (`$94:A012` against `$94:936B`).
const BOMB_HIT_AFTER: u16 = 0x20;

/// A reaction entry.
const fn entry(name: &'static str, setup: Setup, plm: u16, revealed: Revealed) -> Entry {
    Entry {
        name,
        setup,
        plm,
        revealed,
        touched: None,
    }
}

/// A bomb block's entry: its touch PLM, and its hit PLM past it.
const fn bomb(name: &'static str, touch: u16) -> Entry {
    Entry {
        name,
        setup: BOMB_HIT,
        plm: touch.wrapping_add(BOMB_HIT_AFTER),
        revealed: Revealed::None,
        touched: Some(touch),
    }
}

/// This part's reactions, by their PLMs.
const ENTRIES: [Entry; 33] = [
    entry(
        "crumble-1x1-regrows",
        CRUMBLE,
        0xd044,
        Revealed::Plm(REVEAL_1X1),
    ),
    entry(
        "crumble-2x1-regrows",
        CRUMBLE,
        0xd048,
        Revealed::Plm(REVEAL_2X1),
    ),
    entry(
        "crumble-1x2-regrows",
        CRUMBLE,
        0xd04c,
        Revealed::Plm(REVEAL_1X2),
    ),
    entry(
        "crumble-2x2-regrows",
        CRUMBLE,
        0xd050,
        Revealed::Plm(REVEAL_2X2),
    ),
    entry("crumble-1x1", CRUMBLE, 0xd054, Revealed::Plm(REVEAL_1X1)),
    entry("crumble-2x1", CRUMBLE, 0xd058, Revealed::Plm(REVEAL_2X1)),
    entry("crumble-1x2", CRUMBLE, 0xd05c, Revealed::Plm(REVEAL_1X2)),
    entry("crumble-2x2", CRUMBLE, 0xd060, Revealed::Plm(REVEAL_2X2)),
    entry(
        "speed-regrows-slow",
        SPEED,
        0xd030,
        Revealed::Plm(REVEAL_SPEED),
    ),
    entry("speed-slow", SPEED, 0xd034, Revealed::Plm(REVEAL_SPEED)),
    entry("speed-regrows", SPEED, 0xd038, Revealed::Plm(REVEAL_SPEED)),
    entry(
        "speed-regrows-b",
        SPEED,
        0xd03c,
        Revealed::Plm(REVEAL_SPEED),
    ),
    entry("speed", SPEED, 0xd040, Revealed::Plm(REVEAL_SPEED)),
    entry("shot-1x1-regrows", SHOT_REGROWS, 0xd064, Revealed::None),
    entry("shot-2x1-regrows", SHOT_REGROWS, 0xd068, Revealed::None),
    entry("shot-1x2-regrows", SHOT_REGROWS, 0xd06c, Revealed::None),
    entry("shot-2x2-regrows", SHOT_REGROWS, 0xd070, Revealed::None),
    entry("shot-1x1", SHOT, 0xd074, Revealed::None),
    entry("shot-2x1", SHOT, 0xd078, Revealed::None),
    entry("shot-1x2", SHOT, 0xd07c, Revealed::None),
    entry("shot-2x2", SHOT, 0xd080, Revealed::None),
    entry(
        "power-bomb-regrows",
        POWER_BOMB,
        0xd084,
        Revealed::List(REVEAL_POWER_BOMB),
    ),
    entry(
        "power-bomb",
        POWER_BOMB,
        0xd088,
        Revealed::List(REVEAL_POWER_BOMB),
    ),
    entry(
        "super-regrows",
        SUPER_MISSILE,
        0xd08c,
        Revealed::List(REVEAL_SUPER),
    ),
    entry("super", SUPER_MISSILE, 0xd090, Revealed::List(REVEAL_SUPER)),
    bomb("bomb-1x1-regrows", 0xd098),
    bomb("bomb-2x1-regrows", 0xd09c),
    bomb("bomb-1x2-regrows", 0xd0a0),
    bomb("bomb-2x2-regrows", 0xd0a4),
    bomb("bomb-1x1", 0xd0a8),
    bomb("bomb-2x1", 0xd0ac),
    bomb("bomb-1x2", 0xd0b0),
    bomb("bomb-2x2", 0xd0b4),
];

/// The block collision tables (bank $94 log): the PLM each class of cell spawns, by its BTS
/// below 80h, and the area tables' pointers for the BTS from 80h.
mod table {
    // Special cells' touch (`$94:9139`) and its area tables (`$94:92E9`).
    pub const SPECIAL: u16 = 0x9139;
    pub const SPECIAL_AREAS: u16 = 0x92e9;
    // Special blocks' blast (`$94:9DA4`) and its area tables (`$94:9E44`).
    pub const SPECIAL_BOMBED: u16 = 0x9da4;
    pub const SPECIAL_BOMBED_AREAS: u16 = 0x9e44;
    // Shootable cells' (`$94:9EA6`) and their area tables (`$94:9FC6`).
    pub const SHOOTABLE: u16 = 0x9ea6;
    pub const SHOOTABLE_AREAS: u16 = 0x9fc6;
    // Bombable cells' touch (`$94:936B`), 16 entries.
    pub const BOMBABLE: u16 = 0x936b;
    pub const BOMBABLE_ENTRIES: u8 = 0x10;
    // Special air's inside area tables (`$94:9B06`).
    pub const INSIDE_AREAS: u16 = 0x9b06;
}

/// The cell a scroll cell's placement writes: special air whose mark is a scroll cell, its look
/// kept, as the scroll PLM's setup writes it.
pub const SCROLL_CELL: BlockKind = BlockKind::SpecialAir(Bts(0x46));

/// A reaction, by its place in [`Reactions::all`].
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReactionId(pub u16);

/// A sound a script asks for: its library and number, and its queue limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sound {
    /// The sound library.
    pub library: u8,
    /// The sound's number in it.
    pub number: u8,
    /// How many sounds the queue may hold for it to be queued.
    pub queue_limit: u16,
}

impl Sound {
    /// The sound's name: the blocks' two by what they sound like, the others by their library
    /// and number.
    #[must_use]
    pub fn name(self) -> String {
        match (self.library, self.number) {
            (2, 0x0a) => "BlockCrumbles".to_owned(),
            (2, 0x06) => "BlockBreaksOnContact".to_owned(),
            (library, number) => format!("library {library} sound {number:02X}"),
        }
    }
}

/// What a pattern cell becomes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Becomes {
    /// Air.
    Air,
    /// A reacting block of the family, whatever the cell was.
    Block,
    /// A stand-in along its row.
    StandInAcross,
    /// A stand-in along its column.
    StandInDown,
}

/// One cell of a show's pattern.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PatternCell {
    /// Where it is from the reaction's cell.
    pub offset: Pair<i16>,
    /// What it becomes.
    pub becomes: Becomes,
    /// The look it takes.
    pub look: Look,
}

/// A script's step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Asks for a sound and goes on.
    Sound(Sound),
    /// The pattern's cells change, and the reaction waits `frames`.
    Show {
        /// The cells, in order.
        pattern: Vec<PatternCell>,
        /// The frames it waits, 1 to 32,767.
        frames: u16,
    },
    /// The cell grows back with the whole look; the reaction waits a frame.
    Regrow,
    /// The reaction ends.
    End,
}

/// A reaction: what a reacting cell does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reaction {
    /// The importer's name for it.
    pub name: &'static str,
    /// Which rules start it.
    pub family: Family,
    /// The picture it has when whole, never flipped; none for the shot blocks that keep their
    /// own look.
    pub whole_look: Option<Picture>,
    /// The frames from its start to its first step.
    pub first_wait: u16,
    /// What it does when it breaks.
    pub broken: Vec<Step>,
    /// What a bomb block does when her touch breaks it; empty for the other families.
    pub touched: Vec<Step>,
    /// What a bomb that doesn't break it does; empty where it has none.
    pub revealed: Vec<Step>,
}

impl Reaction {
    /// Its script `script`.
    #[must_use]
    pub fn script(&self, script: Script) -> &[Step] {
        match script {
            Script::Broken => &self.broken,
            Script::Revealed => &self.revealed,
            Script::Touched => &self.touched,
        }
    }
}

/// A reacting cell's mark (the spec's §1), as the room's tables give it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    /// One of this part's reactions.
    Reaction(ReactionId),
    /// A scroll cell.
    Scroll,
    /// Reacts to nothing but holds a place, met as this family's blocks.
    Nothing(MetAs),
    /// Reacts to nothing and holds no place.
    Inert,
    /// Another part's cell, with what a blast meets it as.
    Other(OtherMetAs),
}

/// What a blast meets another part's cell as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OtherMetAs {
    /// No family's blocks: only its part answers it.
    NoFamily,
    /// Crumble blocks, as a station's access cells are.
    Crumble,
}

/// Why the importer refuses a cell or a reaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    /// A read outside the image.
    Address(AddressError),
    /// An asker of a cell of this type and BTS reads past its table's end, so the original's
    /// answer is a neighbouring table's word or code (LEGACY, "Reacting cells whose tables end
    /// before their mark").
    PastTable {
        /// The block type.
        kind: u8,
        /// The BTS.
        bts: u8,
    },
    /// A list holds an instruction the format has no step for.
    Instruction(RomAddress),
    /// A draw writes a block type the format has no pattern cell for.
    Pattern(RomAddress),
    /// A list runs back on itself.
    Loop(RomAddress),
    /// A show of 0 frames, or more than 32,767.
    ShowFrames(RomAddress),
    /// A whole look with a flip, which the spec's whole look (a picture) can't hold.
    FlippedWholeLook(RomAddress),
    /// A reacting cell of this type and BTS whose mark names a reaction this part doesn't hold.
    NoReaction {
        /// The block type.
        kind: u8,
        /// The BTS.
        bts: u8,
    },
    /// A bomb block's `broken` script that doesn't begin with a sound, which a bomb's start
    /// skips three bytes of (LEGACY).
    BombWithoutSound(&'static str),
}

impl From<AddressError> for Refused {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// Which script of which reaction a PLM runs, for the record decoder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Runs {
    /// This reaction's script.
    Script(ReactionId, Script),
    /// The `revealed` script of the reaction the cell's mark names (the bombed special blocks
    /// name only a shape).
    RevealedOfMark,
    /// A hold until the next pass: a touch of a scroll cell or of a cell marked `Nothing`.
    Hold,
}

/// The reactions, and the maps a record decoder reads a reaction's place by.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reactions {
    all: Vec<Reaction>,
    /// Each PLM this part's places hold, and what it runs.
    plms: Vec<(u16, Runs)>,
    /// For each reaction and script, the list positions each step is at, in step order: the
    /// step's own instruction and the go-tos that lead to it.
    positions: Vec<[Vec<Vec<u16>>; 3]>,
    /// The PLM each class of cell spawns, by its BTS, from its tables.
    tables: Tables,
}

/// The collision tables' PLMs, read once.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Tables {
    /// By area, then BTS: the special cells' touch, the special blocks' blast, the shootable
    /// cells' and the special air's inside PLM; `None` past the table.
    special: Vec<[Option<u16>; 256]>,
    special_bombed: Vec<[Option<u16>; 256]>,
    shootable: Vec<[Option<u16>; 256]>,
    inside: Vec<[Option<u16>; 256]>,
    /// The bombable cells', 16 entries.
    bombable: Vec<u16>,
}

/// The areas the tables have a row for, the debug area last.
const AREAS: usize = 8;

/// A word of bank `bank` at `offset`.
fn word(rom: Lorom<'_>, bank: u8, offset: u16) -> Result<u16, Refused> {
    let bytes = rom.read(RomAddress::new(bank, offset), 2)?;
    match *bytes {
        [low, high] => Ok(u16::from_le_bytes([low, high])),
        _ => Err(Refused::Address(AddressError::PastEnd(RomAddress::new(
            bank, offset,
        )))),
    }
}

/// A byte of bank `bank` at `offset`.
fn byte(rom: Lorom<'_>, bank: u8, offset: u16) -> Result<u8, Refused> {
    let bytes = rom.read(RomAddress::new(bank, offset), 1)?;
    bytes
        .first()
        .copied()
        .ok_or_else(|| Refused::Address(AddressError::PastEnd(RomAddress::new(bank, offset))))
}

/// The look of a cell's word: its ten bits of picture, which always name one, and its flips.
fn look(word: u16) -> Option<Look> {
    let flip = match (word & FLIP_ACROSS != 0, word & FLIP_DOWN != 0) {
        (false, false) => Flip::None,
        (true, false) => Flip::Across,
        (false, true) => Flip::Down,
        (true, true) => Flip::Both,
    };
    Some(Look {
        picture: Picture::new(word & PICTURE)?,
        flip,
    })
}

/// The whole look of a setup's respawn word: its picture, which no setup flips.
fn whole_look(word: u16) -> Option<Picture> {
    look(word)
        .filter(|look| look.flip == Flip::None)
        .map(|look| look.picture)
}

/// The draw instruction at `at` (`$84:861E`): rows and columns of cells, each a block word
/// whose type says what the cell becomes.
fn pattern(rom: Lorom<'_>, at: u16) -> Result<Vec<PatternCell>, Refused> {
    let mut cells = Vec::new();
    let mut origin = Pair { x: 0_i16, y: 0_i16 };
    let mut cursor = at;
    loop {
        let header = word(rom, PLM_BANK, cursor)?;
        let column = header & COLUMN != 0;
        let [count, _] = header.to_le_bytes();
        cursor = cursor.wrapping_add(2);
        for index in 0..i16::from(count) {
            let block = word(rom, PLM_BANK, cursor)?;
            let becomes = match block >> TYPE_SHIFT {
                0x0 => Becomes::Air,
                0xb | 0xc | 0xf => Becomes::Block,
                0x5 => Becomes::StandInAcross,
                0xd => Becomes::StandInDown,
                _ => return Err(Refused::Pattern(RomAddress::new(PLM_BANK, cursor))),
            };
            let offset = if column {
                Pair {
                    x: origin.x,
                    y: origin.y.wrapping_add(index),
                }
            } else {
                Pair {
                    x: origin.x.wrapping_add(index),
                    y: origin.y,
                }
            };
            let look =
                look(block).ok_or_else(|| Refused::Pattern(RomAddress::new(PLM_BANK, cursor)))?;
            cells.push(PatternCell {
                offset,
                becomes,
                look,
            });
            cursor = cursor.wrapping_add(2);
        }
        if word(rom, PLM_BANK, cursor)? == DRAW_END {
            return Ok(cells);
        }
        let [x, y] = word(rom, PLM_BANK, cursor)?.to_le_bytes();
        origin = Pair {
            x: i16::from(i8::from_le_bytes([x])),
            y: i16::from(i8::from_le_bytes([y])),
        };
        cursor = cursor.wrapping_add(2);
    }
}

/// The script of the list at `at`, its go-tos followed, with each step's list positions.
fn script(rom: Lorom<'_>, at: u16) -> Result<(Vec<Step>, Vec<Vec<u16>>), Refused> {
    let mut steps = Vec::new();
    let mut positions = Vec::new();
    let mut leading = Vec::new();
    let mut seen = Vec::new();
    let mut cursor = at;
    loop {
        let place = RomAddress::new(PLM_BANK, cursor);
        if seen.contains(&cursor) {
            return Err(Refused::Loop(place));
        }
        seen.push(cursor);
        leading.push(cursor);
        let instruction = word(rom, PLM_BANK, cursor)?;
        let (step, length) = if instruction == op::GO_TO {
            cursor = word(rom, PLM_BANK, cursor.wrapping_add(2))?;
            continue;
        } else if let Some(&(_, library, queue_limit)) = op::SOUNDS
            .iter()
            .find(|&&(sound, _, _)| sound == instruction)
        {
            let number = byte(rom, PLM_BANK, cursor.wrapping_add(2))?;
            let sound = Sound {
                library,
                number,
                queue_limit,
            };
            (Step::Sound(sound), 3)
        } else if op::REGROW.contains(&instruction) {
            (Step::Regrow, 2)
        } else if instruction == op::END {
            (Step::End, 2)
        } else if instruction < INSTRUCTION {
            if !(1..=MOST_FRAMES).contains(&instruction) {
                return Err(Refused::ShowFrames(place));
            }
            let draw = word(rom, PLM_BANK, cursor.wrapping_add(2))?;
            let step = Step::Show {
                pattern: pattern(rom, draw)?,
                frames: instruction,
            };
            (step, 4)
        } else {
            return Err(Refused::Instruction(place));
        };
        let end = step == Step::End;
        steps.push(step);
        positions.push(core::mem::take(&mut leading));
        if end {
            return Ok((steps, positions));
        }
        cursor = cursor.wrapping_add(length);
    }
}

/// A PLM's instruction list: its header's second word.
fn list(rom: Lorom<'_>, plm: u16) -> Result<u16, Refused> {
    word(rom, PLM_BANK, plm.wrapping_add(2))
}

/// A table of PLMs by BTS: the common table for BTS below 80h (`common` entries), the area's
/// for 80h on (`per_area` entries).
fn by_bts(
    rom: Lorom<'_>,
    common: (u16, u8),
    areas: u16,
    per_area: u8,
    area: u16,
) -> Result<[Option<u16>; 256], Refused> {
    let mut plms = [None; 256];
    let area_table = word(
        rom,
        COLLISION_BANK,
        areas.wrapping_add(area.wrapping_mul(2)),
    )?;
    for (bts, plm) in (0_u8..=255).zip(plms.iter_mut()) {
        let (table, index, entries) = if bts & AREA_TABLE == 0 {
            (common.0, bts, common.1)
        } else {
            (area_table, bts & !AREA_TABLE, per_area)
        };
        if index < entries {
            *plm = Some(word(
                rom,
                COLLISION_BANK,
                table.wrapping_add(u16::from(index).wrapping_mul(2)),
            )?);
        }
    }
    Ok(plms)
}

impl Tables {
    /// The tables of the image, past each one's end `None`. A BTS past a table's end is read
    /// only by the refusals, which name it first.
    fn load(rom: Lorom<'_>) -> Result<Self, Refused> {
        let mut tables = Self::default();
        for area in (0_u16..).take(AREAS) {
            tables.special.push(by_bts(
                rom,
                (table::SPECIAL, 0x50),
                table::SPECIAL_AREAS,
                0x10,
                area,
            )?);
            tables.special_bombed.push(by_bts(
                rom,
                (table::SPECIAL_BOMBED, 0x50),
                table::SPECIAL_BOMBED_AREAS,
                0x08,
                area,
            )?);
            tables.shootable.push(by_bts(
                rom,
                (table::SHOOTABLE, 0x50),
                table::SHOOTABLE_AREAS,
                0x08,
                area,
            )?);
            tables
                .inside
                .push(by_bts(rom, (0, 0), table::INSIDE_AREAS, 0x10, area)?);
        }
        for index in 0..u16::from(table::BOMBABLE_ENTRIES) {
            tables.bombable.push(word(
                rom,
                COLLISION_BANK,
                table::BOMBABLE.wrapping_add(index.wrapping_mul(2)),
            )?);
        }
        Ok(tables)
    }
}

/// The table row of `area`.
const fn row(area: Area) -> usize {
    match area {
        Area::Crateria => 0,
        Area::Brinstar => 1,
        Area::Norfair => 2,
        Area::WreckedShip => 3,
        Area::Maridia => 4,
        Area::Tourian => 5,
        Area::Ceres => 6,
        Area::Debug => 7,
    }
}

/// Whether a cell of type `kind` and BTS `bts` has an asker that reads past its table's end.
/// A special block's blast past its 16 entries reads `$B62F` at 10h..19h and 1Eh..4Fh, which the
/// importer types; at 1Ah..1Dh it reads a speed block's reveal.
fn past_table(kind: u8, bts: u8) -> bool {
    let pointers = (0x50..0x80).contains(&bts);
    match kind {
        0x3 => pointers || bts >= 0x90,
        0xb => pointers || bts >= 0x88 || (0x1a..=0x1d).contains(&bts),
        0x4 => pointers,
        0xc => pointers || bts >= 0x88,
        0x7 | 0xf => (0x10..0x80).contains(&bts),
        _ => false,
    }
}

impl Reactions {
    /// The reactions of the image, and its collision tables.
    ///
    /// # Errors
    ///
    /// [`Refused`] for a list or a table outside the image, or a reaction the format can't
    /// hold.
    pub fn load(rom: Lorom<'_>) -> Result<Self, Refused> {
        let mut reactions = Self {
            tables: Tables::load(rom)?,
            ..Self::default()
        };
        for (entry, index) in ENTRIES.iter().zip(0_u16..) {
            let id = ReactionId(index);
            let (broken, broken_at) = script(rom, list(rom, entry.plm)?)?;
            let (touched, touched_at) = match entry.touched {
                Some(plm) => {
                    reactions
                        .plms
                        .push((plm, Runs::Script(id, Script::Touched)));
                    script(rom, list(rom, plm)?)?
                }
                None => (Vec::new(), Vec::new()),
            };
            let (revealed, revealed_at) = match entry.revealed {
                Revealed::None => (Vec::new(), Vec::new()),
                Revealed::Plm(plm) => script(rom, list(rom, plm)?)?,
                Revealed::List(at) => script(rom, at)?,
            };
            if entry.setup.family == Family::Bomb && !matches!(broken.first(), Some(Step::Sound(_)))
            {
                return Err(Refused::BombWithoutSound(entry.name));
            }
            reactions
                .plms
                .push((entry.plm, Runs::Script(id, Script::Broken)));
            let whole_look = match entry.setup.whole_look {
                Some(at) => Some(
                    whole_look(word(rom, PLM_BANK, at)?)
                        .ok_or_else(|| Refused::FlippedWholeLook(RomAddress::new(PLM_BANK, at)))?,
                ),
                None => None,
            };
            let first_wait = match entry.setup.first_wait {
                Some(at) => word(rom, PLM_BANK, at)?,
                None => SPAWN_WAIT,
            };
            reactions.all.push(Reaction {
                name: entry.name,
                family: entry.setup.family,
                whole_look,
                first_wait,
                broken,
                touched,
                revealed,
            });
            reactions
                .positions
                .push([broken_at, revealed_at, touched_at]);
        }
        for plm in [REVEAL_1X1, REVEAL_2X1, REVEAL_1X2, REVEAL_2X2, REVEAL_SPEED] {
            reactions.plms.push((plm, Runs::RevealedOfMark));
        }
        for plm in NOTHING.into_iter().chain([SCROLL_TOUCH]) {
            reactions.plms.push((plm, Runs::Hold));
        }
        Ok(reactions)
    }

    /// Every reaction, by its id's place.
    #[must_use]
    pub fn all(&self) -> &[Reaction] {
        &self.all
    }

    /// The reaction `id` names.
    #[must_use]
    pub fn get(&self, id: ReactionId) -> Option<&Reaction> {
        self.all.get(usize::from(id.0))
    }

    /// The reaction named `name`.
    #[must_use]
    pub fn named(&self, name: &str) -> Option<ReactionId> {
        self.all
            .iter()
            .position(|reaction| reaction.name == name)
            .and_then(|index| u16::try_from(index).ok())
            .map(ReactionId)
    }

    /// What the PLM `plm` runs, if it is one of this part's.
    #[must_use]
    pub fn runs(&self, plm: u16) -> Option<Runs> {
        self.plms
            .iter()
            .find_map(|&(known, runs)| (known == plm).then_some(runs))
    }

    /// The PLM that runs `script` of reaction `id`: the PLM a place holding it has.
    #[must_use]
    pub fn plm_of(&self, id: ReactionId, script: Script) -> Option<u16> {
        self.plms
            .iter()
            .find_map(|&(plm, runs)| (runs == Runs::Script(id, script)).then_some(plm))
    }

    /// The step of `script` of reaction `id` a place whose list is at `position` runs next.
    #[must_use]
    pub fn step_at(&self, id: ReactionId, script: Script, position: u16) -> Option<usize> {
        let steps = self.script_positions(id, script)?;
        steps.iter().position(|leading| leading.contains(&position))
    }

    /// The list position of step `step` of `script` of reaction `id`: where a place that runs
    /// it next holds its list.
    #[must_use]
    pub fn position_of(&self, id: ReactionId, script: Script, step: usize) -> Option<u16> {
        self.script_positions(id, script)?
            .get(step)?
            .last()
            .copied()
    }

    fn script_positions(&self, id: ReactionId, script: Script) -> Option<&Vec<Vec<u16>>> {
        let scripts = self.positions.get(usize::from(id.0))?;
        Some(match script {
            Script::Broken => &scripts[0],
            Script::Revealed => &scripts[1],
            Script::Touched => &scripts[2],
        })
    }

    /// The mark of a cell of `kind` in `area`: `None` for a kind that isn't reacting.
    ///
    /// # Errors
    ///
    /// [`Refused::PastTable`] for a cell an asker of which reads past its table's end, and
    /// [`Refused::NoReaction`] for a mark that names a reaction this part doesn't hold, which the
    /// room's loading refuses (spec §8).
    pub fn mark(&self, kind: BlockKind, area: Area) -> Result<Option<Mark>, Refused> {
        let (nybble, bts) = (kind.nybble(), kind.bts());
        if past_table(nybble, bts) {
            return Err(Refused::PastTable { kind: nybble, bts });
        }
        let at = |tables: &[[Option<u16>; 256]]| {
            tables
                .get(row(area))
                .and_then(|plms| plms.get(usize::from(bts)))
                .copied()
                .flatten()
        };
        let known = |plm: Option<u16>| -> Option<Mark> {
            let plm = plm?;
            self.runs(plm).and_then(|runs| match runs {
                Runs::Script(id, Script::Broken | Script::Touched) => Some(Mark::Reaction(id)),
                Runs::Script(_, Script::Revealed) | Runs::RevealedOfMark | Runs::Hold => None,
            })
        };
        let mark = self.typed(nybble, bts, at, known);
        match mark {
            Some(Mark::Reaction(id)) if self.get(id).is_none() => {
                Err(Refused::NoReaction { kind: nybble, bts })
            }
            Some(_) | None => Ok(mark),
        }
    }

    /// The mark of a cell of type `nybble` and BTS `bts` by its tables (`at` reads one of them
    /// by the cell's area and BTS), `known` the mark of a PLM that runs a reaction's script.
    fn typed(
        &self,
        nybble: u8,
        bts: u8,
        at: impl Fn(&[[Option<u16>; 256]]) -> Option<u16>,
        known: impl Fn(Option<u16>) -> Option<Mark>,
    ) -> Option<Mark> {
        Some(match nybble {
            0x3 | 0xb => {
                let plm = at(&self.tables.special);
                if plm == Some(SCROLL_TOUCH) {
                    Mark::Scroll
                } else if plm.is_some_and(|plm| NOTHING.contains(&plm)) {
                    Mark::Nothing(MetAs::Crumble)
                } else if let Some(mark) = known(plm) {
                    mark
                } else if nybble == 0xb
                    && at(&self.tables.special_bombed).is_some_and(|plm| NOTHING.contains(&plm))
                {
                    Mark::Other(OtherMetAs::Crumble)
                } else {
                    Mark::Other(OtherMetAs::NoFamily)
                }
            }
            0x4 | 0xc => {
                if nybble == 0x4 && bts & AREA_TABLE != 0 {
                    Mark::Inert
                } else {
                    let plm = at(&self.tables.shootable);
                    if plm.is_some_and(|plm| NOTHING.contains(&plm)) {
                        Mark::Nothing(MetAs::Shot)
                    } else {
                        known(plm).unwrap_or(Mark::Other(OtherMetAs::NoFamily))
                    }
                }
            }
            0x7 | 0xf => {
                if bts & AREA_TABLE != 0 {
                    Mark::Inert
                } else {
                    let plm = self.tables.bombable.get(usize::from(bts)).copied();
                    if plm.is_some_and(|plm| NOTHING.contains(&plm)) {
                        Mark::Nothing(MetAs::Bomb)
                    } else {
                        known(plm).unwrap_or(Mark::Other(OtherMetAs::NoFamily))
                    }
                }
            }
            _ => return None,
        })
    }

    /// Whether she holds a place when she is inside a cell of `kind` in `area`: special air
    /// whose area's inside table names a PLM that only holds its place.
    #[must_use]
    pub fn holds_place_inside(&self, kind: BlockKind, area: Area) -> bool {
        let bts = kind.bts();
        kind.nybble() == 0x3
            && bts & AREA_TABLE != 0
            && self
                .tables
                .inside
                .get(row(area))
                .and_then(|plms| plms.get(usize::from(bts)))
                .copied()
                .flatten()
                .is_some_and(|plm| INSIDE_NOTHING.contains(&plm))
    }

    /// The block type a cell of `family`'s reactions has as a block, and as air to her.
    #[must_use]
    pub const fn class(family: Family) -> (u8, u8) {
        match family {
            Family::Crumble | Family::Speed => (0xb, 0x3),
            Family::PowerBomb | Family::Shot | Family::SuperMissile => (0xc, 0x4),
            Family::Bomb => (0xf, 0x7),
        }
    }

    /// The block type a cell met as `met` has as a block, and as air to her.
    #[must_use]
    pub const fn met_class(met: MetAs) -> (u8, u8) {
        match met {
            MetAs::Crumble => (0xb, 0x3),
            MetAs::Shot => (0xc, 0x4),
            MetAs::Bomb => (0xf, 0x7),
        }
    }
}

#[cfg(test)]
mod tests;

// ROM sources (bank $84 and $94 logs): the setups `$84:CE37` (crumble, the first wait at
// `$84:CE5A`), `$84:CDEA` (speed), `$84:CE83` and `$84:CEDA` (bomb), `$84:CE6B` and `$84:B3C1`
// (shot), `$84:CF2E` (power bomb), `$84:CF67` (super missile); each builds its respawn block as
// `[block] & F000h | tile` (`$84:CE49`, `$84:CE25`, `$84:CEFB`, `$84:CE72`,
// `$84:CF4F`, `$84:CF88`), whose `ORA` operand is the whole look. The bombed special blocks
// `$84:CFFC`..`$84:D008` and `$84:D024`, the revealed lists `$84:C91C` and `$84:C922`.
