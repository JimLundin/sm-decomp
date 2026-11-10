//! How Samus looks, part 1, as a harness domain: the samus-looks model against the oracle.
//!
//! The model (`clean_model_samus_looks`) is checked frame by frame on the spec's State
//! (`docs/behaviours/samus-looks.md` §1), by the decode notes' harness note
//! (`docs/behaviours/samus-looks.decode.md`, "The harness").
//!
//! Her clips and the run's paces are the importer's, from the ROM. A frame is a pair of quiescent
//! records: the State is decoded from record N, her step through the step map of her pose's
//! clip; the inputs from records N and N+1, at the animation's turn, as the decode notes give
//! them; the State the model leaves is compared with record N+1's, her step through the step map
//! of the clip of the pose she ends the frame in. A wall jump readied is samus-motion's near-wall
//! test, from its model over record N where that model runs the frame; elsewhere it is read off
//! record N+1's own step, and the fields such a frame's readied changes are unverified, not
//! matched. The null model is her State left as record N has it.

pub mod cases;
pub mod import;

use core::cell::Cell;
use core::fmt;

use clean_model_samus_looks::{
    BehaviourId, Boost, CATALOGUE, ClipStep, Content, Frames, Inputs, Laps, Liquid, Move, Playhead,
    Rules, State, Step, Vertical, step,
};
use clean_model_samus_motion::WallTest;
use clean_types::{Action, BoostStage, Item, Set};
use data::pose::PoseId;
use data::pose::animation::AnimationFrame;
use data::rom::Lorom;
use oracle_record::FrameRecord;

use super::samus_motion::SamusMotion;
use super::{Domain, Entry, Field, FieldSet, Interval, Mask, Uncompared};
use crate::bundle::ContentError;
use crate::cache::CutKind;
use crate::decode::DecodeError;
use crate::domain_field;
use crate::source::{self, WramByte, WramWord};
use import::{Clips, ImportError, Imported};

/// Why the domain can't load.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadError {
    /// The importer's clips or paces.
    Clips(ImportError),
    /// Samus-motion's content, which its model reads.
    Motion(ContentError),
}

/// A work RAM word the harness reads that `source` doesn't name.
const fn wram(name: &'static str, offset: u16) -> WramWord {
    WramWord { name, offset }
}

/// The game state the main loop runs gameplay in.
const MAIN_GAMEPLAY: u16 = 8;
/// Bank $90 log: the handler gameplay runs for Samus once a frame ($90:E722).
const SAMUS_NEW_STATE_HANDLER: WramWord = wram("FRAME_HANDLER_BETA", 0x0A44);
/// The handlers that run her animation: the normal one, the demos' two and the elevator's
/// (`$90:E725`, `$90:E7F5`, `$90:E833`, `$90:E8EC`).
const ANIMATING: [u16; 4] = [0xE725, 0xE7F5, 0xE833, 0xE8EC];
/// Bank $90 log: the speed booster's counter, the stage in its high byte and the laps in its low.
const SPEED_BOOST_COUNTER: WramWord = wram("SPEED_BOOSTER_LEVEL", 0x0B3E);
/// Bank $90 log: her dash's momentum, the extra run speed, 0 without one.
const EXTRA_RUN_SPEED: WramWord = wram("SAMUS_X_EXTRA_RUN_SPEED", 0x0B3C);
/// Bank $90 log: the animation's frame buffer, the slowdown its clip moves on with (`$90:8078`).
const FRAME_BUFFER: WramWord = wram("SAMUS_ANIM_FRAME_BUFFER", 0x0A9C);
/// Bank $90 log: the X speed divisor, the start's slowdown with the gravity suit (`$91:FB3F`).
const X_SPEED_DIVISOR: WramWord = wram("SAMUS_X_SPEED_DIVISOR", 0x0A66);
/// Bank $91 log: the super special prospective pose change command and its pose (`$91:EB88`).
const POSE_CHANGE_COMMAND: WramWord = source::SAMUS_HURT_SWITCH_INDEX;
const SUPER_SPECIAL_POSE: WramWord = source::SAMUS_NEW_POSE_TRANSITIONAL;
/// The command "transition animation finished", and the bit that says no pose is asked.
const ANIMATION_FINISHED: u16 = 3;
const NO_POSE: u16 = 0x8000;
/// The pad's shoot binding.
const BINDING_SHOOT: WramWord = wram("BUTTON_CONFIG_SHOOT_X", 0x09B2);
/// The standing movement type, and the two unaimed standing poses (`$90:A3AD`).
const STANDING: u8 = 0;
const STANDING_UNAIMED: [u16; 2] = [0x01, 0x02];
/// Her vertical direction down (`0B36`).
const DOWN: u16 = 2;
/// The equipment bits the model reads, and the gravity suit's, which the liquids read.
const ITEM_BITS: [(Item, u16); 3] = [
    (Item::SpeedBooster, 0x2000),
    (Item::SpaceJump, 0x0200),
    (Item::ScrewAttack, 0x0008),
];
const GRAVITY_SUIT: u16 = 0x0020;
/// Bank $80 log: library 3's sound queue, its write index and its read index (`$80:9158`).
const SOUND_3_WRITE: WramByte = WramByte {
    name: "SOUND_QUEUE_3_NEXT",
    offset: 0x0648,
};
const SOUND_3_READ: WramByte = WramByte {
    name: "SOUND_QUEUE_3_START",
    offset: 0x0645,
};
/// The queue's 16 places.
const QUEUE_PLACES: u8 = 0x0F;
/// Bank $90 log: the liquids' surfaces, negative where the room has none, and the water's
/// physics switch.
const FX_Y_POS: WramWord = wram("FX_Y_POS", 0x195E);
const LAVA_ACID_Y_POS: WramWord = wram("LAVA_ACID_Y_POS", 0x1962);
const FX_LIQUID_OPTIONS: WramWord = wram("FX_LIQUID_OPTIONS", 0x197E);
const LIQUID_PHYSICS_OFF: u16 = 0x0004;
/// The start's slowdown under water and under lava or acid (`$90:9E93`, `$90:9E95`).
const WATER_START: Frames = Frames(3);
const LAVA_ACID_START: Frames = Frames(2);

/// The declared fields by their places in [`SamusLooks`]'s `FIELDS`.
const STEP: FieldSet = FieldSet::single(0);
const LEFT: FieldSet = FieldSet::single(1);
const STAGE: FieldSet = FieldSet::single(2);
const LAPS: FieldSet = FieldSet::single(3);
const PLAYHEAD: FieldSet = STEP.union(LEFT);
const BOOST: FieldSet = STAGE.union(LAPS);

/// The declared fields a catalogue entry's output reaches. Turned far and the sounds are
/// outputs no field holds, so they are never credited.
const fn shown_by(id: BehaviourId) -> FieldSet {
    use BehaviourId as B;
    match id {
        B::ClipPlays
        | B::ClipControls
        | B::Breathing
        | B::ClipEnds
        | B::WallJumpBranch
        | B::HandOverSlowdown
        | B::StartingClip
        | B::StandingRestart
        | B::WallJumpReady
        | B::FastFall
        | B::RisingHold
        | B::RunPace => PLAYHEAD,
        B::BoostStages | B::BoostingLap => PLAYHEAD.union(BOOST),
        B::SpinTurnedFar => FieldSet::EMPTY,
    }
}

/// Why a frame isn't one her animation is compared on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Skip {
    /// A source couldn't be read: a harness defect.
    Unreadable(DecodeError),
    /// A word holds what the State has no value for (a frame on an operand byte, a time left past
    /// 255, a stage past 4): a harness defect.
    Undecodable(&'static str, u16),
    /// Record N isn't a quiescent cut: the frame from it is the rest of a lag frame.
    NotQuiescent,
    /// The main loop isn't in gameplay at record N or N+1.
    GameState(u16),
    /// Her frame handler at record N runs no animation.
    NotAnimating(u16),
    /// The frame count stalled from N to N+1: a message box or the item fanfare holds the loop.
    Stalled,
    /// The frame count stalled into record N: a message box ended the frame.
    MessageBoxEnded,
    /// Her pose at N or N+1 is outside the domain (another domain's: the x-ray scope, the
    /// grapple, the shinespark, knockback, drained, the crystal flash, the Draygon grab, her
    /// appearance).
    Pose(u16),
}

impl fmt::Display for Skip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(error) => write!(f, "unreadable: {error:?}"),
            Self::Undecodable(word, value) => write!(f, "{word} {value:#06X} has no value"),
            Self::NotQuiescent => f.write_str("record N isn't a quiescent cut (lag)"),
            Self::GameState(code) => write!(f, "game state {code:#04X}, not gameplay"),
            Self::NotAnimating(code) => {
                write!(f, "her frame handler {code:#06X} runs no animation")
            }
            Self::Stalled => f.write_str("the frame count stalled (a message box, the fanfare)"),
            Self::MessageBoxEnded => f.write_str("a message box ended the frame"),
            Self::Pose(code) => write!(f, "pose {code:#04X} isn't in the domain"),
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

fn byte(record: &FrameRecord, source: WramByte) -> Result<u8, Skip> {
    Ok(source.read(&record.wram)?)
}

/// The spec's State as the model gives it and as the oracle's record holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Declared {
    /// Her step.
    pub step: Step,
    /// The time left.
    pub left: Frames,
    /// The boost stage.
    pub stage: BoostStage,
    /// The laps left.
    pub laps: Laps,
}

impl From<&State> for Declared {
    fn from(state: &State) -> Self {
        Self {
            step: state.playhead.step,
            left: state.playhead.left,
            stage: state.boost.stage,
            laps: state.boost.laps,
        }
    }
}

/// The frame's inputs, and what the harness read off record N+1's own step.
pub struct Frame {
    /// The model's inputs.
    pub inputs: Inputs,
    /// A wall jump was readied: read off record N+1's step, so the fields it changes are
    /// unverified.
    readied_from_the_record: bool,
}

/// The samus-looks domain, over the importer's clips, with the samus-motion model that supplies
/// the near-wall test.
pub struct SamusLooks {
    clips: Clips,
    motion: SamusMotion,
    /// The frame count at the record before the current frame's, and its record's index; then the
    /// current frame's. The frame a message box ends is told by the stall into record N.
    before: Cell<Option<(u64, u16)>>,
    last: Cell<Option<(u64, u16)>>,
}

impl SamusLooks {
    /// The domain over the ROM `image`.
    ///
    /// # Errors
    ///
    /// [`LoadError`] when the clips, the paces or samus-motion's content don't import.
    pub fn load(image: &[u8]) -> Result<Self, LoadError> {
        Ok(Self {
            clips: Clips::load(Lorom::new(image)).map_err(LoadError::Clips)?,
            motion: SamusMotion::load(image).map_err(LoadError::Motion)?,
            before: Cell::new(None),
            last: Cell::new(None),
        })
    }

    /// The imported clips.
    #[must_use]
    pub const fn clips(&self) -> &Clips {
        &self.clips
    }

    /// Her pose in `record`, with its clip.
    fn imported(&self, record: &FrameRecord) -> Result<&Imported, Skip> {
        let id = word(record, source::SAMUS_POSE)?;
        self.clips.of(PoseId(id)).ok_or(Skip::Pose(id))
    }

    /// The State `record` holds.
    fn state(&self, record: &FrameRecord) -> Result<State, Skip> {
        let imported = self.imported(record)?;
        let frame = word(record, source::SAMUS_ANIM_FRAME)?;
        let step = imported
            .step_of(AnimationFrame(frame))
            .ok_or(Skip::Undecodable("SAMUS_ANIM_FRAME", frame))?;
        let timer = word(record, source::SAMUS_ANIM_FRAME_TIMER)?;
        let left =
            u8::try_from(timer).map_err(|_| Skip::Undecodable("SAMUS_ANIM_FRAME_TIMER", timer))?;
        let counter = word(record, SPEED_BOOST_COUNTER)?;
        let [laps, stage] = counter.to_le_bytes();
        let stage = BoostStage::ALL
            .get(usize::from(stage))
            .copied()
            .ok_or(Skip::Undecodable("SPEED_BOOSTER_LEVEL", counter))?;
        Ok(State {
            playhead: Playhead {
                step,
                left: Frames(left),
            },
            boost: Boost {
                stage,
                laps: Laps(laps),
            },
        })
    }

    /// The frame's inputs at the animation's turn, from records `from` and `next`.
    fn inputs(&self, from: &FrameRecord, next: &FrameRecord) -> Result<Frame, Skip> {
        let before = self.imported(from)?;
        let after = self.imported(next)?;
        let held = word(from, source::JOYPAD1_LASTKEYS)?;
        let pressing = |binding| Ok::<_, Skip>(held & word(from, binding)? != 0);
        let dash = [
            word(from, EXTRA_RUN_SPEED)? != 0,
            word(next, EXTRA_RUN_SPEED)? != 0,
        ];
        let began = dash == [false, true];
        let restarts = byte(from, source::SAMUS_MOVEMENT_TYPE)? == STANDING
            && STANDING_UNAIMED.contains(&before.id.0)
            && pressing(BINDING_SHOOT)?;
        let motion = self.motion.wall_test(from, Interval::ending_at(next));
        let (readied, from_the_record) = if let Ok(test) = motion {
            (test == WallTest::Readied, false)
        } else {
            let readied = readied(before, after, from, next)?;
            (readied, readied)
        };
        let handed_over = word(next, POSE_CHANGE_COMMAND)? == ANIMATION_FINISHED
            && word(next, SUPER_SPECIAL_POSE)? & NO_POSE == 0;
        let mut moves = Set::<Move>::default();
        moves[Move::DashBegan] = began;
        moves[Move::StandingRestart] = restarts;
        moves[Move::WallJumpReadied] = readied;
        moves[Move::HandedOver] = handed_over;
        moves[Move::DashLost] = dash == [true, false];
        let equipped = word(next, source::EQUIPPED_ITEMS)?;
        let mut items = Set::<Item>::default();
        for (item, bit) in ITEM_BITS {
            items[item] = equipped & bit != 0;
        }
        let vertical = if word(next, source::SAMUS_Y_DIR)? == DOWN {
            Vertical::Down(word(next, source::SAMUS_Y_SPEED)?)
        } else {
            Vertical::NotDown
        };
        let waiting = byte(from, SOUND_3_WRITE)?.wrapping_sub(byte(from, SOUND_3_READ)?);
        let slowdown = word(next, FRAME_BUFFER)?;
        Ok(Frame {
            inputs: Inputs {
                pose: before.pose,
                pose_after: after.pose,
                moves,
                dashing: dash[0] || began,
                run_held: pressing(source::BINDING_RUN)?,
                vertical,
                energy: word(next, source::SAMUS_HEALTH)?,
                items,
                liquid: Liquid {
                    slowdown: Frames(
                        u8::try_from(slowdown)
                            .map_err(|_| Skip::Undecodable("SAMUS_ANIM_FRAME_BUFFER", slowdown))?,
                    ),
                    start_slowdown: start_slowdown(next, after, equipped)?,
                    submerged: submerged(next, before, equipped)?,
                },
                sounds_waiting: waiting & QUEUE_PLACES,
            },
            readied_from_the_record: from_the_record,
        })
    }
}

/// Whether samus-motion's near-wall test readied a wall jump on a frame its model doesn't run in,
/// read off the records: her pose
/// unchanged, a clip with a kick step, her frame at N+1 on it and at N neither on it nor on the
/// frame before it. The decode notes' reading: the kick step is reached no other way.
fn readied(
    before: &Imported,
    after: &Imported,
    from: &FrameRecord,
    next: &FrameRecord,
) -> Result<bool, Skip> {
    let Some(kick) = before.clip.kick.and_then(|kick| before.frame_of(kick)) else {
        return Ok(false);
    };
    let frame = |record| word(record, source::SAMUS_ANIM_FRAME);
    let was = frame(from)?;
    Ok(before.id == after.id
        && frame(next)? == kick.0
        && was != kick.0
        && was != kick.0.wrapping_sub(1))
}

/// Her top and bottom in `record` in a pose of y radius `radius` (`$90:EC58`).
fn boundaries(record: &FrameRecord, radius: u8) -> Result<(i16, i16), Skip> {
    let y = word(record, source::SAMUS_Y_POS)?.cast_signed();
    let radius = i16::from(radius);
    Ok((
        y.wrapping_sub(radius),
        y.wrapping_add(radius).wrapping_sub(1),
    ))
}

/// The liquids' surfaces in `record`: the water's, where its physics are on, and the lava's or
/// acid's, each `None` where the room has none.
fn surfaces(record: &FrameRecord) -> Result<(Option<i16>, Option<i16>), Skip> {
    let water = word(record, FX_Y_POS)?.cast_signed();
    let lava = word(record, LAVA_ACID_Y_POS)?.cast_signed();
    let physics = word(record, FX_LIQUID_OPTIONS)? & LIQUID_PHYSICS_OFF == 0;
    Ok((
        (water >= 0 && physics).then_some(water),
        (water < 0 && lava >= 0).then_some(lava),
    ))
}

/// The start's slowdown for the pose `taken`: with the gravity suit the X speed divisor;
/// otherwise 3 with the water's surface above the feet of the pose she takes, 2 with the lava's
/// or acid's, else the divisor (`$91:FB08`).
fn start_slowdown(next: &FrameRecord, taken: &Imported, equipped: u16) -> Result<Frames, Skip> {
    let divisor = word(next, X_SPEED_DIVISOR)?;
    let divisor =
        Frames(u8::try_from(divisor).map_err(|_| Skip::Undecodable("X_SPEED_DIVISOR", divisor))?);
    if equipped & GRAVITY_SUIT != 0 {
        return Ok(divisor);
    }
    let (_, bottom) = boundaries(next, taken.radius)?;
    let (water, lava) = surfaces(next)?;
    Ok(if water.is_some_and(|surface| surface < bottom) {
        WATER_START
    } else if lava.is_some_and(|surface| surface < bottom) {
        LAVA_ACID_START
    } else {
        divisor
    })
}

/// Whether she is submerged as a wall jump's branch tests it: no gravity suit, and a liquid's
/// surface above her top (`$90:841D`).
fn submerged(next: &FrameRecord, pose: &Imported, equipped: u16) -> Result<bool, Skip> {
    if equipped & GRAVITY_SUIT != 0 {
        return Ok(false);
    }
    let (top, _) = boundaries(next, pose.radius)?;
    let (water, lava) = surfaces(next)?;
    Ok(water.or(lava).is_some_and(|surface| surface < top))
}

/// The record is a quiescent cut: the NMI found the main thread at a wait site.
fn quiescent(record: &FrameRecord) -> bool {
    CutKind::of(record.cut) == CutKind::Requested
}

fn gameplay(record: &FrameRecord) -> Result<(), Skip> {
    match word(record, source::GAME_STATE)? {
        MAIN_GAMEPLAY => Ok(()),
        state => Err(Skip::GameState(state)),
    }
}

impl Domain for SamusLooks {
    const NAME: &'static str = "samus-looks";
    type State = State;
    type Inputs = ();
    type World = Frame;
    type Fields = Declared;
    type Skip = Skip;
    const FIELDS: &'static [Field<Declared>] = &[
        domain_field!("step", step),
        domain_field!("left", left),
        domain_field!("stage", stage),
        domain_field!("laps", laps),
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

    fn decode(&self, from: &FrameRecord, _interval: Interval) -> Result<(State, ()), Skip> {
        let count = word(from, source::NMI_FRAME_COUNTER_WORD)?;
        self.before
            .set(self.last.replace(Some((from.index, count))));
        if !quiescent(from) {
            return Err(Skip::NotQuiescent);
        }
        gameplay(from)?;
        let handler = word(from, SAMUS_NEW_STATE_HANDLER)?;
        if !ANIMATING.contains(&handler) {
            return Err(Skip::NotAnimating(handler));
        }
        Ok((self.state(from)?, ()))
    }

    fn world(&self, from: &FrameRecord, next: &FrameRecord) -> Result<Frame, Skip> {
        gameplay(next)?;
        let count = word(from, source::NMI_FRAME_COUNTER_WORD)?;
        if word(next, source::NMI_FRAME_COUNTER_WORD)? == count {
            return Err(Skip::Stalled);
        }
        let ended = self
            .before
            .get()
            .is_some_and(|(index, before)| index < from.index && before == count);
        if ended {
            return Err(Skip::MessageBoxEnded);
        }
        self.inputs(from, next)
    }

    /// The boost on a frame it changes with no dash at N, none beginning and none lost: no rule
    /// of the animation changes it then, and the shinespark's (S3) clears it so as the shine
    /// ends (terrain-quicksand 156 to 157, at the morph). Record N+1's boost is read only to find
    /// that writer's frames.
    fn masks(&self, from: &FrameRecord, next: &FrameRecord, (): &()) -> Result<Vec<Mask>, Skip> {
        let dash = [word(from, EXTRA_RUN_SPEED)?, word(next, EXTRA_RUN_SPEED)?];
        let boost = [
            word(from, SPEED_BOOST_COUNTER)?,
            word(next, SPEED_BOOST_COUNTER)?,
        ];
        Ok(if dash == [0, 0] && boost[0] != boost[1] {
            vec![Mask {
                reason: "another domain: the boost changes with no dash, the shinespark's as its shine ends (S3)",
                fields: BOOST,
            }]
        } else {
            Vec::new()
        })
    }

    fn neutral(&self, frame: &Frame) -> Option<Frame> {
        frame.readied_from_the_record.then(|| {
            let mut inputs = frame.inputs;
            inputs.moves[Move::WallJumpReadied] = false;
            Frame {
                inputs,
                readied_from_the_record: false,
            }
        })
    }

    fn step(&self, state: &mut State, (): &(), frame: &mut Frame) {
        *state = step(*state, &frame.inputs, &self.clips, &Rules::VANILLA).0;
    }

    fn exercised(&self, state: &State, (): &(), frame: &Frame) -> Vec<usize> {
        exercised(*state, &frame.inputs, &self.clips)
            .into_iter()
            .filter_map(|id| CATALOGUE.iter().position(|entry| entry.id == id))
            .collect()
    }

    fn fields(&self, state: &State) -> Declared {
        Declared::from(state)
    }

    fn expected(&self, next: &FrameRecord) -> Result<Declared, Skip> {
        Ok(Declared::from(&self.state(next)?))
    }

    fn uncompared(&self, skip: &Skip) -> Uncompared {
        match skip {
            Skip::Unreadable(_) | Skip::Undecodable(..) => Uncompared::Defect(skip.to_string()),
            Skip::NotQuiescent
            | Skip::GameState(_)
            | Skip::NotAnimating(_)
            | Skip::Stalled
            | Skip::MessageBoxEnded
            | Skip::Pose(_) => Uncompared::Skip(skip.to_string()),
        }
    }

    fn folds(&self, _from: &FrameRecord, next: &FrameRecord) -> bool {
        !quiescent(next)
    }

    fn row_notes(&self) -> String {
        self.before.set(None);
        self.last.set(None);
        String::new()
    }
}

/// The catalogue entries a frame brings into play, from the State record N holds and the
/// frame's inputs: what her moves change, then what the clip moves on to, then the pose she
/// takes.
fn exercised(state: State, inputs: &Inputs, content: &Clips) -> Vec<BehaviourId> {
    use BehaviourId as B;
    let clip = content.clip(&inputs.pose);
    let mut hit = vec![B::ClipPlays];
    let moves = &inputs.moves;
    if moves[Move::StandingRestart] {
        hit.push(B::StandingRestart);
    }
    if moves[Move::WallJumpReadied] && clip.kick.is_some() {
        hit.push(B::WallJumpReady);
    }
    let fast =
        matches!(inputs.vertical, Vertical::Down(speed) if speed >= Rules::VANILLA.fast_fall_speed);
    if clip
        .fast_fall
        .is_some_and(|step| fast && state.playhead.step < step)
    {
        hit.push(B::FastFall);
    }
    let rising = clip.rising == Some(state.playhead.step)
        && state.playhead.left == Frames(1)
        && inputs.vertical == Vertical::NotDown;
    if rising {
        hit.push(B::RisingHold);
    }
    let running = inputs.dashing && inputs.pose.motion.action == Action::Running;
    let moves_on =
        !rising && !moves[Move::StandingRestart] && state.playhead.counted_down().is_none();
    if let Some((_, next)) = moves_on.then(|| clip.after(state.playhead.step)).flatten() {
        hit.extend(moved_on(next, running, inputs, state.boost));
    }
    if moves[Move::DashBegan] || moves[Move::DashLost] {
        hit.push(B::BoostStages);
    }
    if inputs.pose != inputs.pose_after {
        hit.push(B::StartingClip);
        if moves[Move::HandedOver] && inputs.liquid.slowdown != Frames::NONE {
            hit.push(B::HandOverSlowdown);
        }
    }
    if content.clip(&inputs.pose_after).kick.is_some() {
        hit.push(B::SpinTurnedFar);
    }
    hit.sort_by_key(|id| CATALOGUE.iter().position(|entry| entry.id == *id));
    hit.dedup();
    hit
}

/// The entries the clip moving on to `next` brings into play.
fn moved_on(next: ClipStep, running: bool, inputs: &Inputs, boost: Boost) -> Vec<BehaviourId> {
    use BehaviourId as B;
    let booster = inputs.items[Item::SpeedBooster];
    match next {
        ClipStep::Show(_) if running => vec![B::RunPace],
        ClipStep::Show(_) => Vec::new(),
        _ if running && inputs.run_held && !booster => vec![B::RunPace],
        _ if running && inputs.run_held => {
            let up_to_boosting = boost.laps.0 <= 1 && boost.stage == BoostStage::Stage3;
            if up_to_boosting {
                vec![B::RunPace, B::BoostStages, B::BoostingLap]
            } else {
                vec![B::RunPace, B::BoostStages]
            }
        }
        ClipStep::Again(_) | ClipStep::Rest => vec![B::ClipControls],
        ClipStep::Breathe => vec![B::Breathing],
        ClipStep::End | ClipStep::EndUnlessJump => vec![B::ClipEnds],
        ClipStep::Branch(_) => vec![B::WallJumpBranch],
    }
}

#[cfg(test)]
mod tests {
    use super::{BOOST, PLAYHEAD, SamusLooks};
    use crate::harness::Domain;

    #[test]
    fn the_field_sets_name_the_declared_fields_at_their_places() {
        let names: Vec<&str> = <SamusLooks as Domain>::FIELDS
            .iter()
            .map(|field| field.name)
            .collect();
        assert_eq!(names, ["step", "left", "stage", "laps"]);
        assert!(PLAYHEAD.contains(0) && PLAYHEAD.contains(1) && !PLAYHEAD.contains(2));
        assert!(BOOST.contains(2) && BOOST.contains(3) && !BOOST.contains(1));
    }
}
