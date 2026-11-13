//! The Ceres doors (boss-ceres-ridley, part 1) as a harness domain: the model against the oracle.
//!
//! The model (`boss_ceres_ridley`) is checked frame by frame by the decode notes' harness note
//! (`docs/behaviours/boss-ceres-ridley.decode.md`, "The harness"). The State is every door of
//! record N this piece models (parameters 0, 1 and 4; the station's other doors are later
//! pieces), decoded by `crate::enemy::ceres_door`'s tables, with the original's "new figure"
//! bit beside each. The inputs are record N's: her position (the doors run before she moves),
//! the Ceres status, the random number stepped once (the main loop steps it before the
//! enemies run) and the screen's shake.
//!
//! Which doors the framework ran on the frame (C1's: the frame counter `+2C` advanced by one
//! from N to N+1) is the neighbours' answer, from record N+1, as other harnesses hear the
//! frame's writers from its hits. A door that ran steps once, in slot order, a shake one starts
//! threaded on to the next; a door that didn't run is compared unchanged.
//!
//! Compared per door: the phase with its wait, the figure, whether it is shown and solid, and
//! the "new figure" bit, which the harness derives from the step (set on a run whose wait ran
//! out, cleared on any other run). Per frame, in every stage: the shake's type and timer, the
//! shake a door's run started or the one running, counted down by R3's room shaking when it
//! ran (`Ran::room_shaking`, `Screen::counted`), so a shake one frame short or one started
//! while calm differs. Masked: the shake on a door-passage frame with one running, which the
//! new room's set-up clears. The opening sound is an event the harness doesn't compare (U1's
//! queue). Skipped: frames with no modelled door, and the frames a room changes (a room's
//! doors are placed by the loader, R2 and C1).
//!
//! The null model is the doors left as record N has them.

use core::fmt;

use boss_ceres_ridley::{
    BehaviourId, CeresDoor, ColourSet, DoorKind, Figure, Inputs, Phase, Rules, Shake, Stage,
    catalogue,
};
use data::palette::{CERES_DOOR_SHADES, CeresDoorColours, DoorShades};
use data::rom::{AddressError, Lorom};
use game::state::Rng;
use game::state::room::ENEMY_SLOTS;
use oracle_record::FrameRecord;
use types::{Pair, Px, Radius, Side};

use super::{Domain, Entry, Field, FieldSet, Hides, Interval, Mask, Uncompared};
use crate::decode::DecodeError;
use crate::domain_field;
use crate::enemy::EnemySources;
use crate::enemy::ceres_door::{self, Unknown};
use crate::source::{self, SlotField, WramWord};

/// The Ceres door species' header (`$A0:E23F`), the slot's species word.
const CERES_DOOR: u16 = 0xE23F;
/// The "new figure" bit of the extra properties (`0F88` bit 15).
const NEW_FIGURE: u16 = 0x8000;
/// The properties' invisible and intangible bits (`0F86`).
const INVISIBLE: u16 = 0x0100;
const INTANGIBLE: u16 = 0x0400;
/// Bank $A0 log: the earthquake's type and timer, which a door's shake sets (`$A6:F773`).
const EARTHQUAKE_TYPE: WramWord = WramWord {
    name: "EARTHQUAKE_TYPE",
    offset: 0x183E,
};
const EARTHQUAKE_TIMER: WramWord = WramWord {
    name: "EARTHQUAKE_TIMER",
    offset: 0x1840,
};
/// Time frozen (`0A78`), when R3's room shaking holds the timer.
const TIME_IS_FROZEN: WramWord = WramWord {
    name: "TIME_IS_FROZEN",
    offset: 0x0A78,
};
/// The game states that run main gameplay (`$82:8B44`), and with it R3's room shaking: main
/// gameplay itself, and Ceres's elevator, blackout and time-up states (`$82:8367`, `$82:8388`,
/// `$82:8411`).
const RUNS_GAMEPLAY: [u16; 4] = [0x08, 0x20, 0x21, 0x23];
/// The door passage's game states (`$82:E169`, `$82:E1B7`, `$82:E288`): the new room's set-up
/// (`$88:8350`) clears the shake's timer.
const DOOR_PASSAGE: [u16; 3] = [0x09, 0x0A, 0x0B];
/// The first earthquake type R3's room shaking doesn't count down (`$A0:8699`, a signed
/// compare).
const UNCOUNTED_TYPE: u16 = 0x24;
/// The earthquake types of the small and the strong shake (BG1, BG2 and enemies, diagonal, 1
/// px and 3 px).
const SMALL_TYPE: u16 = 0x14;
const STRONG_TYPE: u16 = 0x1A;

/// The declared fields by their places in `FIELDS`.
const PHASE: FieldSet = FieldSet::single(0);
const FIGURE: FieldSet = FieldSet::single(1);
const SHOWN: FieldSet = FieldSet::single(2);
const SOLID: FieldSet = FieldSet::single(3);
const NEW_FIGURE_FIELD: FieldSet = FieldSet::single(4);
const SHAKE: FieldSet = FieldSet::single(5);
const COLOURS: FieldSet = FieldSet::single(6);
const BOX: FieldSet = FieldSet::single(7);

/// The doors' target colours, sprite palette 7's colours 1 to 15 (`7EC3E2`), which a door's
/// placing sets and the door passage's fade brings the palette to.
const TARGET_COLOURS: u16 = 0xC3E2;

/// The declared fields a catalogue entry's output reaches.
const fn shown_by(id: BehaviourId) -> FieldSet {
    use BehaviourId as B;
    match id {
        B::Waiting => PHASE.union(NEW_FIGURE_FIELD),
        B::SheIsNear | B::RoomStart | B::Polling | B::Opening | B::Closing | B::ShaftWall => {
            PHASE.union(FIGURE)
        }
        B::Solid => SOLID.union(BOX),
        B::Shown => SHOWN,
        B::EscapeShakes => SHAKE,
        B::TheDoorsColours => COLOURS,
    }
}

/// Why a frame isn't compared.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Skip {
    /// Record N holds no door this piece models.
    NoDoors,
    /// The room's doors change from N to N+1: a door passage, whose placing is the loader's.
    RoomChanges,
    /// A record's words couldn't be read.
    Unreadable(DecodeError),
    /// A door's instruction pointer or timer is no phase's.
    UnknownDoor(u16, Unknown),
    /// A door's spritemap pointer is no figure's.
    UnknownFigure(u16, u16),
    /// A door's frame counter moved by more than one run.
    Counter(u16, u16),
}

impl fmt::Display for Skip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDoors => write!(f, "no modelled Ceres door"),
            Self::RoomChanges => write!(f, "the room's doors change (the loader places them)"),
            Self::Unreadable(error) => write!(f, "unreadable: {error:?}"),
            Self::UnknownDoor(slot, unknown) => write!(f, "slot {slot}: {unknown:?}"),
            Self::UnknownFigure(slot, pointer) => {
                write!(f, "slot {slot}: spritemap {pointer:04X} is no figure")
            }
            Self::Counter(slot, moved) => write!(f, "slot {slot}: frame counter moved {moved}"),
        }
    }
}

impl From<DecodeError> for Skip {
    fn from(error: DecodeError) -> Self {
        Self::Unreadable(error)
    }
}

/// One door in its slot, with the original's "new figure" bit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slotted {
    /// The slot.
    pub slot: u16,
    /// The door.
    pub door: CeresDoor,
    /// The "new figure" bit.
    pub new_figure: bool,
}

/// The screen's shake: its type and frames left.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Screen {
    /// The earthquake type.
    pub kind: u16,
    /// Its frames left.
    pub timer: u16,
}

/// The doors of the room, and the shake as the frame leaves it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Doors {
    /// Every modelled door, in slot order.
    pub doors: Vec<Slotted>,
    /// The shake as the frame leaves it: as the doors leave it, counted by R3.
    pub screen: Screen,
    /// The doors' target colours.
    pub colours: Colours,
}

/// The doors' target colours, by the content's set they equal (the colours themselves are the
/// ROM's, so the report names them only).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Colours {
    /// One of the doors' two colour sets.
    Set(ColourSet),
    /// All black: no room has placed its doors' colours.
    Blank,
    /// Neither set.
    Other,
}

/// What the doors read from record N.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Read {
    /// Her position.
    pub her: Pair<Px>,
    /// The station's stage.
    pub stage: Stage,
    /// The random number as the doors read it: N's stepped once.
    pub random: u16,
    /// The screen's shake as the frame starts.
    pub screen: Screen,
}

/// A door the room's loader placed: its slot, and its kind, facing and position from the
/// room's population.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed {
    /// The slot.
    pub slot: u16,
    /// The kind.
    pub kind: DoorKind,
    /// The facing.
    pub facing: Side,
    /// The position.
    pub position: Pair<Px>,
}

/// What the frame's other writers did.
///
/// The doors the room's loader placed (a room with no door at N starting with doors at N+1),
/// which doors the framework ran, in the State's order, and whether R3's room shaking counted
/// the shake after them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ran {
    /// The doors the loader placed, in slot order; none on a frame no room starts.
    pub placed: Vec<Placed>,
    /// Whether each door ran.
    pub doors: Vec<bool>,
    /// R3's room shaking ran with time running: a game state at N that runs main gameplay (the
    /// door passage's frames run the doors but not the room shaking) and time not frozen at N+1
    /// (`0A78`).
    pub room_shaking: bool,
}

impl Ran {
    /// Whether any door ran.
    fn any(&self) -> bool {
        self.doors.contains(&true)
    }
}

/// The declared fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fields {
    /// Each door's slot and phase.
    pub phase: Vec<(u16, Phase)>,
    /// Each door's figure.
    pub figure: Vec<Option<Figure>>,
    /// Whether each is drawn.
    pub shown: Vec<bool>,
    /// Whether each is solid.
    pub solid: Vec<bool>,
    /// Each door's "new figure" bit.
    pub new_figure: Vec<bool>,
    /// The shake as the frame leaves it.
    pub screen: Screen,
    /// The doors' target colours.
    pub colours: Colours,
    /// Each door's box: its centre and radius.
    pub hitbox: Vec<(Pair<Px>, Radius)>,
}

/// The shake's field: the same type and timer.
const SHAKE_FIELD: Field<Fields> = Field {
    name: "shake",
    same: |model, oracle| model.screen == oracle.screen,
    show: |fields| format!("{:?}", fields.screen),
};

/// The Ceres doors' domain, with the doors' two colour sets from the importer.
#[derive(Clone, Copy, Debug, Default)]
pub struct CeresDomain {
    colours: CeresDoorColours,
}

impl CeresDomain {
    /// The domain with its content from `image`.
    ///
    /// # Errors
    ///
    /// [`AddressError`] when the doors' colour sets are outside the image.
    pub fn load(image: &[u8]) -> Result<Self, AddressError> {
        Ok(Self {
            colours: CeresDoorColours::load(Lorom::new(image))?,
        })
    }

    /// The doors' target colours in `record`, named.
    fn target_colours(&self, record: &FrameRecord) -> Result<Colours, DecodeError> {
        let mut shades: DoorShades = [0; CERES_DOOR_SHADES];
        for (colour, offset) in shades.iter_mut().zip((TARGET_COLOURS..).step_by(2)) {
            *colour = WramWord {
                name: "TARGET_PALETTE",
                offset,
            }
            .read(&record.wram)?;
        }
        Ok(if shades == self.colours.calm {
            Colours::Set(ColourSet::Calm)
        } else if shades == self.colours.escape {
            Colours::Set(ColourSet::Escape)
        } else if shades.iter().all(|&colour| colour == 0) {
            Colours::Blank
        } else {
            Colours::Other
        })
    }
}

/// The enemy slots' words.
const SLOTS: EnemySources = EnemySources::ORIGINAL;

fn word(record: &FrameRecord, field: SlotField, slot: u16) -> Result<u16, DecodeError> {
    SLOTS.table.word(field, slot)?.read(&record.wram)
}

/// The modelled doors of `record`, in slot order; none is [`Skip::NoDoors`].
fn doors(record: &FrameRecord) -> Result<Vec<Slotted>, Skip> {
    let doors = any_doors(record)?;
    if doors.is_empty() {
        return Err(Skip::NoDoors);
    }
    Ok(doors)
}

/// The modelled doors of `record`, in slot order, if any.
fn any_doors(record: &FrameRecord) -> Result<Vec<Slotted>, Skip> {
    let mut doors = Vec::new();
    for slot in (0_u16..).take(ENEMY_SLOTS) {
        if word(record, SLOTS.species, slot)? != CERES_DOOR {
            continue;
        }
        let position =
            ceres_door::position(word(record, SLOTS.x, slot)?, word(record, SLOTS.y, slot)?);
        let door = ceres_door::door(
            word(record, SLOTS.parameter_1, slot)?,
            position,
            word(record, SLOTS.instruction, slot)?,
            word(record, SLOTS.instruction_timer, slot)?,
        )
        .map_err(|unknown| Skip::UnknownDoor(slot, unknown))?;
        if let Some(door) = door {
            doors.push(Slotted {
                slot,
                door: door.0,
                new_figure: word(record, SLOTS.extra_properties, slot)? & NEW_FIGURE != 0,
            });
        }
    }
    Ok(doors)
}

fn screen(record: &FrameRecord) -> Result<Screen, DecodeError> {
    Ok(Screen {
        kind: EARTHQUAKE_TYPE.read(&record.wram)?,
        timer: EARTHQUAKE_TIMER.read(&record.wram)?,
    })
}

/// The earthquake type of a door's shake.
const fn shake_type(shake: Shake) -> u16 {
    if shake.strength > 1 {
        STRONG_TYPE
    } else {
        SMALL_TYPE
    }
}

impl Screen {
    /// The shake after R3's room shaking ran (`$A0:8687`): it counts a running shake down by
    /// one, unless the type is one it doesn't count.
    const fn counted(self) -> Self {
        let counts = self
            .kind
            .wrapping_sub(UNCOUNTED_TYPE)
            .cast_signed()
            .is_negative();
        match self.timer.checked_sub(1) {
            Some(timer) if counts => Self { timer, ..self },
            Some(_) | None => self,
        }
    }
}

/// Whether a door of `door`'s kind, facing and position stands in the same slot.
fn same_door(before: &Slotted, after: &Slotted) -> bool {
    before.slot == after.slot
        && before.door.kind == after.door.kind
        && before.door.facing == after.door.facing
        && before.door.position == after.door.position
}

/// The frame a room's loader places its doors: none at N, doors at N+1, each `Placed`. Any
/// other frame from a room without doors is [`Skip::NoDoors`].
fn placing(from: &FrameRecord, next: &FrameRecord) -> Result<Ran, Skip> {
    let after = any_doors(next)?;
    if after.is_empty() {
        return Err(Skip::NoDoors);
    }
    if after
        .iter()
        .any(|slotted| slotted.door.phase != Phase::Placed)
    {
        return Err(Skip::RoomChanges);
    }
    Ok(Ran {
        placed: after
            .iter()
            .map(|slotted| Placed {
                slot: slotted.slot,
                kind: slotted.door.kind,
                facing: slotted.door.facing,
                position: slotted.door.position,
            })
            .collect(),
        doors: Vec::new(),
        room_shaking: RUNS_GAMEPLAY.contains(&source::GAME_STATE.read(&from.wram)?)
            && TIME_IS_FROZEN.read(&next.wram)? == 0,
    })
}

impl Domain for CeresDomain {
    const NAME: &'static str = "boss-ceres-ridley";
    type State = Doors;
    type Inputs = Read;
    type World = Ran;
    type Fields = Fields;
    type Skip = Skip;
    const FIELDS: &'static [Field<Fields>] = &[
        domain_field!("phase", phase),
        domain_field!("figure", figure),
        domain_field!("shown", shown),
        domain_field!("solid", solid),
        domain_field!("new figure", new_figure),
        SHAKE_FIELD,
        domain_field!("colours", colours),
        domain_field!("box", hitbox),
    ];

    fn behaviours(&self) -> Vec<Entry> {
        catalogue()
            .values()
            .map(|entry| Entry {
                name: format!("{:?}", entry.id),
                fields: shown_by(entry.id),
            })
            .collect()
    }

    fn decode(&self, from: &FrameRecord, _interval: Interval) -> Result<(Doors, Read), Skip> {
        let doors = any_doors(from)?;
        let wram = &from.wram;
        let status = source::CERES_STATUS.read(wram)?;
        let mut rng = Rng {
            word: source::RANDOM_NUMBER.read(wram)?,
            ..Rng::default()
        };
        let screen = screen(from)?;
        let read = Read {
            her: ceres_door::position(
                source::SAMUS_X_POS.read(wram)?,
                source::SAMUS_Y_POS.read(wram)?,
            ),
            stage: game::boss_ceres_ridley_bridge::stage(crate::decode::ceres_stage(status)),
            random: rng.advance(),
            screen,
        };
        Ok((
            Doors {
                doors,
                screen,
                colours: self.target_colours(from)?,
            },
            read,
        ))
    }

    fn world(&self, from: &FrameRecord, next: &FrameRecord) -> Result<Ran, Skip> {
        let before = any_doors(from)?;
        if before.is_empty() {
            return placing(from, next);
        }
        let after = doors(next).map_err(|skip| match skip {
            Skip::NoDoors => Skip::RoomChanges,
            skip @ (Skip::RoomChanges
            | Skip::Unreadable(_)
            | Skip::UnknownDoor(..)
            | Skip::UnknownFigure(..)
            | Skip::Counter(..)) => skip,
        })?;
        if before.len() != after.len()
            || before
                .iter()
                .zip(&after)
                .any(|(before, after)| !same_door(before, after))
        {
            return Err(Skip::RoomChanges);
        }
        let doors = before
            .iter()
            .map(|door| {
                let count = |record| word(record, SLOTS.frame_counter, door.slot);
                match count(next)?.wrapping_sub(count(from)?) {
                    0 => Ok(false),
                    1 => Ok(true),
                    moved => Err(Skip::Counter(door.slot, moved)),
                }
            })
            .collect::<Result<_, Skip>>()?;
        Ok(Ran {
            placed: Vec::new(),
            doors,
            room_shaking: RUNS_GAMEPLAY.contains(&source::GAME_STATE.read(&from.wram)?)
                && TIME_IS_FROZEN.read(&next.wram)? == 0,
        })
    }

    fn masks(
        &self,
        from: &FrameRecord,
        next: &FrameRecord,
        read: &Read,
    ) -> Result<Vec<Mask>, Skip> {
        // The shake is compared on every frame: the doors start it, R3's room shaking counts it
        // down, and only the door passage's room set-up writes it besides them in the station.
        // The target colours are the doors' on the frame they are placed (their placing writes
        // them last), the door passage's palette loading's on its other frames. Neither mask
        // breaks a free run: the doors read the shake from record N each frame, and the frame
        // that next compares the colours after a passage is the placing, which sets them again.
        if !DOOR_PASSAGE.contains(&source::GAME_STATE.read(&from.wram)?) {
            return Ok(Vec::new());
        }
        let mut masks = Vec::new();
        if read.screen.timer != 0 {
            masks.push(Mask {
                reason: "the door passage's room set-up clears the shake (R2)",
                fields: SHAKE,
                hides: Hides::NoWriter,
            });
        }
        if self.world(from, next)?.placed.is_empty() {
            masks.push(Mask {
                reason: "the door passage's palette loading sets the target colours (R2)",
                fields: COLOURS,
                hides: Hides::NoWriter,
            });
        }
        Ok(masks)
    }

    fn step(&self, state: &mut Doors, read: &Read, ran: &mut Ran) {
        if !ran.placed.is_empty() {
            state.doors = ran
                .placed
                .iter()
                .map(|placed| {
                    let placement =
                        CeresDoor::place(placed.kind, placed.facing, placed.position, read.stage);
                    state.colours = Colours::Set(placement.colours);
                    Slotted {
                        slot: placed.slot,
                        door: placement.door,
                        new_figure: false,
                    }
                })
                .collect();
            return;
        }
        let rules = Rules::default();
        let mut screen = read.screen;
        for (slotted, &ran) in state.doors.iter_mut().zip(&ran.doors) {
            if !ran {
                continue;
            }
            let inputs = Inputs {
                her: read.her,
                stage: read.stage,
                random: read.random,
                shake_frames_left: screen.timer,
            };
            slotted.new_figure = slotted.door.phase.counted_down().is_none();
            let run = slotted.door.run(&inputs, &rules);
            slotted.door = run.door;
            if let Some(shake) = run.shake {
                screen = Screen {
                    kind: shake_type(shake),
                    timer: shake.length,
                };
            }
        }
        state.screen = if ran.room_shaking {
            screen.counted()
        } else {
            screen
        };
    }

    fn exercised(&self, state: &Doors, read: &Read, ran: &Ran) -> Vec<usize> {
        use BehaviourId as B;
        let mut ids = Vec::new();
        if !ran.placed.is_empty() {
            ids.extend([B::RoomStart, B::TheDoorsColours, B::Solid, B::Shown]);
        }
        for (slotted, _) in state.doors.iter().zip(&ran.doors).filter(|&(_, &ran)| ran) {
            ids.extend([B::Waiting, B::Solid, B::Shown]);
            if slotted.door.phase.counted_down().is_some() {
                continue;
            }
            let near = Inputs {
                her: read.her,
                stage: read.stage,
                random: read.random,
                shake_frames_left: 0,
            }
            .she_is_near(slotted.door.position, Rules::default().reach);
            ids.extend(match slotted.door.phase {
                Phase::Placed if slotted.door.kind.becomes_wall(read.stage) => {
                    vec![B::RoomStart, B::ShaftWall]
                }
                Phase::Placed => vec![B::RoomStart],
                Phase::Starting { .. } => vec![B::RoomStart, B::SheIsNear],
                Phase::Closed { .. } if near => vec![B::Polling, B::SheIsNear, B::Opening],
                Phase::Open { .. } if !near => vec![B::Polling, B::SheIsNear, B::Closing],
                Phase::Closed { .. } | Phase::Open { .. } => vec![B::Polling, B::SheIsNear],
                Phase::Opening { .. } => vec![B::Opening],
                Phase::Closing { .. } => vec![B::Closing],
                Phase::Wall => vec![B::ShaftWall],
            });
        }
        // A frame a door ran checks the shakes' stage arm too: no shake outside the escape.
        if ran.any() {
            ids.push(B::EscapeShakes);
        }
        let entries: Vec<BehaviourId> = catalogue().values().map(|entry| entry.id).collect();
        let mut places: Vec<usize> = ids
            .into_iter()
            .filter_map(|id| entries.iter().position(|&entry| entry == id))
            .collect();
        places.sort_unstable();
        places.dedup();
        places
    }

    fn fields(&self, state: &Doors) -> Fields {
        let doors = &state.doors;
        Fields {
            phase: doors
                .iter()
                .map(|slotted| (slotted.slot, slotted.door.phase))
                .collect(),
            figure: doors.iter().map(|slotted| slotted.door.figure()).collect(),
            shown: doors
                .iter()
                .map(|slotted| slotted.door.phase.shown())
                .collect(),
            solid: doors
                .iter()
                .map(|slotted| slotted.door.phase.solid())
                .collect(),
            new_figure: doors.iter().map(|slotted| slotted.new_figure).collect(),
            screen: state.screen,
            colours: state.colours,
            hitbox: doors
                .iter()
                .map(|slotted| {
                    let hitbox = slotted.door.hitbox();
                    (hitbox.centre, hitbox.radius)
                })
                .collect(),
        }
    }

    fn expected(&self, next: &FrameRecord) -> Result<Fields, Skip> {
        let doors = doors(next)?;
        let mut fields = Fields {
            phase: Vec::new(),
            figure: Vec::new(),
            shown: Vec::new(),
            solid: Vec::new(),
            new_figure: Vec::new(),
            screen: screen(next)?,
            colours: self.target_colours(next)?,
            hitbox: Vec::new(),
        };
        for slotted in doors {
            let slot = slotted.slot;
            let properties = word(next, SLOTS.properties, slot)?;
            let pointer = word(next, SLOTS.spritemap, slot)?;
            fields.phase.push((slot, slotted.door.phase));
            fields.figure.push(
                ceres_door::figure(pointer)
                    .map_err(|pointer| Skip::UnknownFigure(slot, pointer))?,
            );
            fields.shown.push(properties & INVISIBLE == 0);
            fields.solid.push(properties & INTANGIBLE == 0);
            fields.new_figure.push(slotted.new_figure);
            fields.hitbox.push((
                slotted.door.position,
                ceres_door::position(
                    word(next, SLOTS.radius_x, slot)?,
                    word(next, SLOTS.radius_y, slot)?,
                ),
            ));
        }
        Ok(fields)
    }

    fn uncompared(&self, skip: &Skip) -> Uncompared {
        match skip {
            Skip::NoDoors | Skip::RoomChanges => Uncompared::OutOfScope(skip.to_string()),
            Skip::Unreadable(_)
            | Skip::UnknownDoor(..)
            | Skip::UnknownFigure(..)
            | Skip::Counter(..) => Uncompared::Defect(skip.to_string()),
        }
    }
}

#[cfg(test)]
mod tests;
