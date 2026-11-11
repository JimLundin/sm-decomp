//! The importer (MODEL §1 decision 13): her clips and the run's paces from the ROM, in the spec's
//! format (`docs/behaviours/samus-looks.decode.md`, "The importer").
//!
//! Each pose's delay list becomes a clip: a delay byte a shown step, each instruction a control,
//! its operand bytes no step of their own. The steps are numbered without the operands, so each
//! clip keeps its step map, the original's frame of each step, which the harness decodes her step
//! through. Nothing the importer reads is committed: the harness builds it from the ROM each run.

use clean_model_samus_looks::{
    Branch, Clip, Content, Frames, Instruction, Pace, Paces, Pose, Step,
};
use clean_types::{BoostStage, Sprite};
use data::assets::samus::import::map;
use data::assets::samus::key::{Dress, SpinSprite, Suit};
use data::pose::animation::{self, AnimationFrame, DelayItem, DelayList, wall_jump_frame};
use data::pose::{PoseError, PoseId, Poses};
use data::rom::{AddressError, Lorom, RomAddress};
use enum_map::EnumMap;

/// Why the ROM gives no clips.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportError {
    /// The pose definitions don't decode.
    Poses(PoseError),
    /// A delay list or a pace is outside the image.
    List(AddressError),
    /// A pose of hers has no delay list, or one with no end.
    NoList(PoseId),
    /// A list of one of her poses holds what the spec's format has no cell for: the drained fall,
    /// or an `again` or a `branch` to an operand byte.
    Unwritten(PoseId),
    /// A clip has more steps than a step can number.
    TooLong(PoseId),
}

/// The table of the poses' delay lists, a bank $91 pointer each (bank $91 log, `$91:B010`).
const DELAY_LISTS: RomAddress = RomAddress::new(0x91, 0xB010);
/// The pointer to the plain pace, and the table of the boosted paces' pointers, one per stage
/// (bank $91 log: `$91:B5D1`, `$91:B5DE`).
const PLAIN_PACE: RomAddress = RomAddress::new(0x91, 0xB5D1);
const BOOSTED_PACES: RomAddress = RomAddress::new(0x91, 0xB5DE);
/// The bank the paces are in.
const LIST_BANK: u8 = 0x91;
/// A pace's holds end at its first instruction byte, the first with its high bit set; none is
/// longer than the limit.
const FIRST_INSTRUCTION: u8 = 0x80;
const PACE_LIMIT: usize = 0x20;

/// How far on from a wall jump's `branch` its three sequences start: the plain spin's, the space
/// jump's and the screw attack's (bank $90 log, `$90:841D`).
const BRANCH_SPIN: usize = 0x01;
const BRANCH_SPACE_JUMP: usize = 0x0B;
const BRANCH_SCREW_ATTACK: usize = 0x15;

/// The spins whose kick step is their wall jump frame (`wall_jump_frame`): the plain spin, the
/// space jump's, the screw attack's and the wall jump's, facing right and left.
const KICKS: [u16; 8] = [0x19, 0x1A, 0x1B, 0x1C, 0x81, 0x82, 0x83, 0x84];
/// The falls whose fast fall step is frame 5 (`$90:A58D`).
const FAST_FALLS: [u16; 4] = [0x29, 0x2A, 0x67, 0x68];
const FAST_FALL_FRAME: AnimationFrame = AnimationFrame(5);
/// The straight jumps whose rising step is frame 1 (`$90:8011`).
const RISINGS: [u16; 2] = [0x4D, 0x4E];
const RISING_FRAME: AnimationFrame = AnimationFrame(1);

/// One of her poses as the model sees it, with its clip and the clip's step map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Imported {
    /// The original's pose.
    pub id: PoseId,
    /// The pose and the sprite it is drawn in.
    pub pose: Pose,
    /// Its clip.
    pub clip: Clip,
    /// Half her height in it, in pixels, which a liquid's surface is tested against.
    pub radius: u8,
    /// The original's frame of each step, by step.
    frames: Vec<AnimationFrame>,
}

impl Imported {
    /// The step at the original's `frame`; `None` for a frame on an operand byte or past the
    /// clip.
    #[must_use]
    pub fn step_of(&self, frame: AnimationFrame) -> Option<Step> {
        let at = self.frames.iter().position(|&each| each == frame)?;
        u8::try_from(at).ok().map(Step)
    }

    /// The original's frame of `step`.
    #[must_use]
    pub fn frame_of(&self, step: Step) -> Option<AnimationFrame> {
        self.frames.get(usize::from(step.0)).copied()
    }
}

/// Her clips by pose, and the run's paces: the model's content.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Clips {
    poses: Vec<Imported>,
    paces: Paces,
    none: Clip,
}

impl Clips {
    /// The clips and paces in `rom`.
    ///
    /// # Errors
    ///
    /// An [`ImportError`] when a table doesn't decode or a clip has no cell in the format.
    pub fn load(rom: Lorom<'_>) -> Result<Self, ImportError> {
        let poses = Poses::load(rom).map_err(ImportError::Poses)?;
        let poses = map::MAP
            .iter()
            .map(|entry| {
                let id = entry.id;
                let radius = poses.get(id).ok_or(ImportError::NoList(id))?.y_radius;
                let pointer = DELAY_LISTS
                    .advance(usize::from(id.0).wrapping_mul(2))
                    .ok_or(ImportError::List(AddressError::PastEnd(DELAY_LISTS)))?;
                let bytes = list_at(rom, pointer).map_err(ImportError::List)?;
                let list = DelayList::decode_through(bytes, marked(id).into_iter().max())
                    .ok_or(ImportError::NoList(id))?;
                imported(id, &list, radius)
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            poses,
            paces: paces(rom)?,
            none: Clip::default(),
        })
    }

    /// The pose the original's `id` is, with its clip; `None` outside her domain.
    #[must_use]
    pub fn of(&self, id: PoseId) -> Option<&Imported> {
        self.poses.iter().find(|imported| imported.id == id)
    }

    /// Every pose of hers, in the pose map's order.
    #[must_use]
    pub fn poses(&self) -> &[Imported] {
        &self.poses
    }
}

impl Content for Clips {
    /// A pose no entry holds has no steps: the model leaves her where she is.
    fn clip(&self, pose: &Pose) -> &Clip {
        self.poses
            .iter()
            .find(|imported| imported.pose == *pose)
            .map_or(&self.none, |imported| &imported.clip)
    }

    fn paces(&self) -> &Paces {
        &self.paces
    }
}

/// The sprite the pose map's dress draws her in.
const fn sprite(dress: Dress) -> Sprite {
    match (dress.spin, dress.suit) {
        (SpinSprite::ScrewAttack, _) => Sprite::ScrewAttack,
        (SpinSprite::SpaceJump, _) => Sprite::SpaceJump,
        (SpinSprite::Plain, Suit::VariaOrGravity) => Sprite::OtherSuit,
        (SpinSprite::Plain, Suit::First) => Sprite::Plain,
    }
}

/// The model's pose of the original's `id`; `None` outside her domain.
#[must_use]
pub fn pose_of(id: PoseId) -> Option<Pose> {
    map::shown(id).map(|(motion, dress)| Pose {
        motion,
        sprite: sprite(dress),
    })
}

/// The bytes an item takes: the instruction and its operands.
const fn size(item: DelayItem) -> usize {
    match item {
        DelayItem::Delay(_)
        | DelayItem::Instruction(
            animation::Instruction::NoOp(_)
            | animation::Instruction::LoopIfHealthy
            | animation::Instruction::DrainedFalling
            | animation::Instruction::WallJump
            | animation::Instruction::Loop,
        ) => 1,
        DelayItem::Instruction(
            animation::Instruction::AutoJumpTransition(_)
            | animation::Instruction::Transition(_)
            | animation::Instruction::GoBack(_),
        ) => 2,
        DelayItem::Instruction(animation::Instruction::BySpeed(_)) => 3,
        DelayItem::Instruction(animation::Instruction::ByItems { .. }) => 5,
        DelayItem::Instruction(animation::Instruction::ByItemsAndSpeed { .. }) => 7,
    }
}

/// Pose `id`'s clip from its delay list.
fn imported(id: PoseId, list: &DelayList, radius: u8) -> Result<Imported, ImportError> {
    let mut items = Vec::new();
    let mut at = 0_usize;
    while let Some(item) = u16::try_from(at)
        .ok()
        .and_then(|frame| list.at(AnimationFrame(frame)))
    {
        items.push((at, item));
        at = at.wrapping_add(size(item));
    }
    let frames: Vec<AnimationFrame> = items
        .iter()
        .map(|&(at, _)| u16::try_from(at).map(AnimationFrame))
        .collect::<Result<_, _>>()
        .map_err(|_| ImportError::TooLong(id))?;
    let step_at = |at: Option<usize>| -> Result<Step, ImportError> {
        let frame = at.and_then(|at| u16::try_from(at).ok());
        let step = frames.iter().position(|each| Some(each.0) == frame);
        step.and_then(|step| u8::try_from(step).ok())
            .map(Step)
            .ok_or(ImportError::Unwritten(id))
    };
    let instructions = items
        .iter()
        .map(|&(at, item)| {
            Ok(match item {
                DelayItem::Delay(hold) => Instruction::Show(Frames(hold)),
                DelayItem::Instruction(instruction) => match instruction {
                    animation::Instruction::NoOp(_) => Instruction::Rest,
                    animation::Instruction::LoopIfHealthy => Instruction::Breathe,
                    animation::Instruction::AutoJumpTransition(_) => Instruction::EndUnlessJump,
                    animation::Instruction::Transition(_)
                    | animation::Instruction::BySpeed(_)
                    | animation::Instruction::ByItems { .. }
                    | animation::Instruction::ByItemsAndSpeed { .. } => Instruction::End,
                    animation::Instruction::WallJump => Instruction::Branch(Branch {
                        spin: step_at(at.checked_add(BRANCH_SPIN))?,
                        space_jump: step_at(at.checked_add(BRANCH_SPACE_JUMP))?,
                        screw_attack: step_at(at.checked_add(BRANCH_SCREW_ATTACK))?,
                    }),
                    animation::Instruction::GoBack(back) => {
                        Instruction::Again(step_at(at.checked_sub(usize::from(back)))?)
                    }
                    animation::Instruction::Loop => Instruction::Again(Step::FIRST),
                    animation::Instruction::DrainedFalling => {
                        return Err(ImportError::Unwritten(id));
                    }
                },
            })
        })
        .collect::<Result<_, _>>()?;
    let [kick, fast_fall, rising] = marks(id);
    let mark = |frame: Option<AnimationFrame>| {
        frame
            .map(|frame| step_at(Some(usize::from(frame.0))))
            .transpose()
    };
    let pose = pose_of(id).ok_or(ImportError::NoList(id))?;
    Ok(Imported {
        id,
        pose,
        clip: Clip {
            instructions,
            kick: mark(kick)?,
            fast_fall: mark(fast_fall)?,
            rising: mark(rising)?,
        },
        radius,
        frames,
    })
}

/// The frames pose `id`'s clip marks: its kick step's, its fast fall step's and its rising
/// step's.
fn marks(id: PoseId) -> [Option<AnimationFrame>; 3] {
    let at = |poses: &[u16], frame| poses.contains(&id.0).then_some(frame);
    [
        at(&KICKS, wall_jump_frame(id)),
        at(&FAST_FALLS, FAST_FALL_FRAME),
        at(&RISINGS, RISING_FRAME),
    ]
}

/// The frames pose `id`'s clip marks, which its list reaches whatever its own end: the
/// original sets her frame to them, and reads on past the end as it would the list.
fn marked(id: PoseId) -> Vec<AnimationFrame> {
    marks(id).into_iter().flatten().collect()
}

/// The bytes from the list the bank $91 pointer at `pointer` points at, on to the bank's end.
fn list_at(rom: Lorom<'_>, pointer: RomAddress) -> Result<&[u8], AddressError> {
    let target = rom
        .read(pointer, 2)?
        .first_chunk::<2>()
        .copied()
        .ok_or(AddressError::PastEnd(pointer))?;
    rom.from(RomAddress::new(LIST_BANK, u16::from_le_bytes(target)))
}

/// The pace at the bank $91 pointer at `pointer`: its holds, up to its first instruction byte.
fn pace(rom: Lorom<'_>, pointer: RomAddress) -> Result<Pace, ImportError> {
    let list = list_at(rom, pointer).map_err(ImportError::List)?;
    Ok(Pace(
        list.iter()
            .take(PACE_LIMIT)
            .take_while(|&&byte| byte < FIRST_INSTRUCTION)
            .map(|&byte| Frames(byte))
            .collect(),
    ))
}

/// The run's paces: the plain pace, and each stage's boosted pace from its pointer.
fn paces(rom: Lorom<'_>) -> Result<Paces, ImportError> {
    let mut boosted = EnumMap::<BoostStage, Pace>::default();
    for (index, (_, slot)) in boosted.iter_mut().enumerate() {
        let pointer = BOOSTED_PACES
            .advance(index.wrapping_mul(2))
            .ok_or(ImportError::List(AddressError::PastEnd(BOOSTED_PACES)))?;
        *slot = pace(rom, pointer)?;
    }
    Ok(Paces {
        plain: pace(rom, PLAIN_PACE)?,
        boosted,
    })
}
