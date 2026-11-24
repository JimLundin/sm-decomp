//! The importer (MODEL §1 decision 13): her clips and the run's paces in the samus-appearance format.
//!
//! The format is the spec's (`docs/behaviours/samus-appearance.decode.md`, "The importer"), and the
//! importer reads the delay lists and the paces `data` decodes from the ROM. Each pose's delay list becomes a clip: a delay byte a shown step, each instruction a control,
//! its operand bytes no step of their own. The steps are numbered without the operands, so each
//! clip keeps its step map, the original's frame of each step: the RAM translation reads her step
//! through it, and her sprite, drawn by the original's frame, looks her picture up through it.

use data::assets::samus::key::{Dress, SpinSprite, Suit};
use data::assets::samus::map;
use data::pose::PoseId;
use data::pose::animation::{
    self, AnimationDelays, AnimationFrame, DelayItem, DelayList, FAST_FALL_FRAME, FAST_FALLS,
    RISING_FRAME, RISINGS, RunPaces, wall_jump_frame,
};
use enum_map::EnumMap;
use samus_appearance::{
    AnimationContent, Branch, Clip, Control, Figure, Hold, Instruction, Pace, Paces, Step,
};
use types::Sprite;

/// Why the decoded lists give no clips.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportError {
    /// A pose of hers has no delay list.
    NoList(PoseId),
    /// A list of one of her poses holds what the spec's format has no cell for: the drained fall,
    /// a delay of 0 or past the longest hold, or an `again`, a `branch` or a mark on an operand
    /// byte or past the list.
    Unwritten(PoseId),
    /// A pace holds a delay of 0 or past the longest hold.
    Pace,
}

/// How far on from a wall jump's `branch` its three sequences start: the plain spin's, the space
/// jump's and the screw attack's (bank $90 log, `$90:841D`).
const BRANCH_SPIN: usize = 0x01;
const BRANCH_SPACE_JUMP: usize = 0x0B;
const BRANCH_SCREW_ATTACK: usize = 0x15;

/// The spins whose kick step is their wall jump frame (`wall_jump_frame`): the plain spin, the
/// space jump's, the screw attack's and the wall jump's, facing right and left.
const KICKS: [PoseId; 8] = [
    PoseId(0x19),
    PoseId(0x1A),
    PoseId(0x1B),
    PoseId(0x1C),
    PoseId(0x81),
    PoseId(0x82),
    PoseId(0x83),
    PoseId(0x84),
];

/// One of her poses as the model sees it, with its clip and the clip's step map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Imported {
    /// The original's pose.
    pub id: PoseId,
    /// Her figure: the pose and the sprite it is drawn in.
    pub figure: Figure,
    /// Its clip.
    pub clip: Clip,
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
}

/// Her clips by pose, and the run's paces: the model's content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clips {
    poses: Vec<Imported>,
    paces: Paces,
}

/// No clips and no paces: content with nothing imported.
impl Default for Clips {
    fn default() -> Self {
        Self {
            poses: Vec::new(),
            paces: Paces {
                plain: Pace(Vec::new()),
                boosted: EnumMap::from_fn(|_| Pace(Vec::new())),
            },
        }
    }
}

impl Clips {
    /// The clips of every pose of hers from `delays`, and the paces from `paces`.
    ///
    /// # Errors
    ///
    /// An [`ImportError`] when a pose has no list, or a list or a pace has no cell in the
    /// format.
    pub fn import(delays: &AnimationDelays, paces: &RunPaces) -> Result<Self, ImportError> {
        let poses = map::MAP
            .iter()
            .map(|entry| {
                let list = delays.of(entry.id).ok_or(ImportError::NoList(entry.id))?;
                imported(entry.id, list, delays.contacts())
            })
            .collect::<Result<_, _>>()?;
        // By stage, from the first, as `BoostStage` orders them.
        let [first, second, third, fourth, boosting] =
            paces.boosted.each_ref().map(|holds| pace(holds));
        let clips = Self {
            poses,
            paces: Paces {
                plain: pace(&paces.plain)?,
                boosted: EnumMap::from_array([first?, second?, third?, fourth?, boosting?]),
            },
        };
        for imported in &clips.poses {
            if imported.figure.pose.action != types::Action::Running {
                continue;
            }
            for (index, instruction) in imported.clip.instructions.iter().enumerate() {
                if matches!(instruction, Instruction::Show(_))
                    && (clips.paces.plain.0.get(index).is_none()
                        || clips
                            .paces
                            .boosted
                            .values()
                            .any(|pace| pace.0.get(index).is_none()))
                {
                    return Err(ImportError::Pace);
                }
            }
        }
        Ok(clips)
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

impl AnimationContent for Clips {
    fn clip(&self, figure: &Figure) -> Option<&Clip> {
        self.poses
            .iter()
            .find(|imported| imported.figure == *figure)
            .map(|imported| &imported.clip)
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

/// The model's figure of the original's pose `id`; `None` outside her domain.
#[must_use]
pub fn figure_of(id: PoseId) -> Option<Figure> {
    map::shown(id).map(|(pose, dress)| Figure {
        pose,
        sprite: sprite(dress),
    })
}

/// A delay as the hold it is.
fn hold(id: PoseId, delay: u8) -> Result<Hold, ImportError> {
    Hold::new(delay).ok_or(ImportError::Unwritten(id))
}

/// Pose `id`'s clip from its delay list.
fn imported(
    id: PoseId,
    list: &DelayList,
    contacts: &[AnimationFrame],
) -> Result<Imported, ImportError> {
    use animation::Instruction as I;
    let (frames, items): (Vec<AnimationFrame>, Vec<DelayItem>) = list.steps().unzip();
    let step_at = |at: Option<usize>| -> Result<Step, ImportError> {
        let frame = at.and_then(|at| u16::try_from(at).ok());
        let step = frames.iter().position(|each| Some(each.0) == frame);
        step.and_then(|step| u8::try_from(step).ok())
            .map(Step)
            .ok_or(ImportError::Unwritten(id))
    };
    let instructions = frames
        .iter()
        .zip(&items)
        .map(|(&frame, &item)| {
            let at = usize::from(frame.0);
            let instruction = match item {
                DelayItem::Delay(delay) => return Ok(Instruction::Show(hold(id, delay)?)),
                DelayItem::Instruction(instruction) => instruction,
            };
            Ok(Instruction::Control(match instruction {
                I::NoOp(_) => Control::Rest,
                I::LoopIfHealthy => Control::Breathe,
                I::AutoJumpTransition(_) => Control::EndUnlessJump,
                I::Transition(_)
                | I::BySpeed(_)
                | I::ByItems { .. }
                | I::ByItemsAndSpeed { .. } => Control::End,
                I::WallJump => Control::Branch(Branch {
                    spin: step_at(at.checked_add(BRANCH_SPIN))?,
                    space_jump: step_at(at.checked_add(BRANCH_SPACE_JUMP))?,
                    screw_attack: step_at(at.checked_add(BRANCH_SCREW_ATTACK))?,
                }),
                I::GoBack(back) => Control::Again(step_at(at.checked_sub(usize::from(back)))?),
                I::Loop => Control::Again(Step::FIRST),
                I::DrainedFalling => return Err(ImportError::Unwritten(id)),
            }))
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_instructions(id, &instructions)?;
    let [kick, fast_fall, rising] = marks(id);
    let mark = |frame: Option<AnimationFrame>| {
        frame
            .map(|frame| step_at(Some(usize::from(frame.0))))
            .transpose()
    };
    let figure = figure_of(id).ok_or(ImportError::NoList(id))?;
    Ok(Imported {
        id,
        figure,
        clip: Clip {
            instructions,
            kick: mark(kick)?,
            fast_fall: mark(fast_fall)?,
            rising: mark(rising)?,
            contacts: if figure.pose.action == types::Action::Running {
                contacts
                    .iter()
                    .map(|frame| mark(Some(*frame)))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .flatten()
                    .collect()
            } else {
                std::collections::BTreeSet::new()
            },
        },
        frames,
    })
}

/// Imported targets must show a picture; clean total fallbacks never validate content.
fn validate_instructions(id: PoseId, instructions: &[Instruction]) -> Result<(), ImportError> {
    let shown = |step: Step| {
        matches!(
            instructions.get(usize::from(step.0)),
            Some(Instruction::Show(_))
        )
    };
    for (index, instruction) in instructions.iter().enumerate() {
        let valid = match instruction {
            Instruction::Show(_) => index.saturating_add(1) < instructions.len(),
            Instruction::Control(Control::Again(step)) => shown(*step),
            Instruction::Control(Control::Branch(branch)) => {
                [branch.spin, branch.space_jump, branch.screw_attack]
                    .into_iter()
                    .all(shown)
            }
            Instruction::Control(Control::Rest | Control::Breathe) => {
                matches!(
                    instructions.get(index.saturating_add(1)),
                    Some(Instruction::Show(_))
                )
            }
            Instruction::Control(Control::End | Control::EndUnlessJump) => true,
        };
        if !valid {
            return Err(ImportError::Unwritten(id));
        }
    }
    Ok(())
}

/// The frames pose `id`'s clip marks: its kick step's, its fast fall step's (`FastFall`) and its
/// rising step's (`RisingHold`), each a frame its list reaches (`data`'s `reached`).
fn marks(id: PoseId) -> [Option<AnimationFrame>; 3] {
    let at = |poses: &[PoseId], frame| poses.contains(&id).then_some(frame);
    [
        at(&KICKS, wall_jump_frame(id)),
        at(&FAST_FALLS, FAST_FALL_FRAME),
        at(&RISINGS, RISING_FRAME),
    ]
}

/// A pace from its delays.
fn pace(delays: &[u8]) -> Result<Pace, ImportError> {
    delays
        .iter()
        .map(|&delay| Hold::new(delay).ok_or(ImportError::Pace))
        .collect::<Result<_, _>>()
        .map(Pace)
}
