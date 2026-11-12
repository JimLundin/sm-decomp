//! How Samus looks, part 1, as a harness domain: the samus-appearance model against the oracle.
//!
//! The model (`samus_appearance`) is checked frame by frame on the spec's State
//! (`docs/behaviours/samus-appearance.md` §1), by the decode notes' harness note
//! (`docs/behaviours/samus-appearance.decode.md`, "The harness").
//!
//! Her clips and the run's paces are the engine's importer's (`game::samus_appearance_bridge`), from
//! the ROM. A frame is a pair of quiescent records: the State is decoded from record N, her step
//! through the step map of her pose's clip; the inputs from records N and N+1, at the
//! animation's turn, as the decode notes give them; the State the model leaves is compared with
//! record N+1's, her step through the step map of the clip of the pose she ends the frame in. A
//! wall jump readied, the standing restart, the dash, a dash lost, the pose an end handed
//! over, heading down and her vertical speed are samus-movement's outputs, from its model over record
//! N where that model runs the frame, through the engine's own mapping
//! (`game::samus_appearance_bridge::Moved`); elsewhere they are read off the records, and a wall jump
//! readied off record N+1's own step, so the fields such a frame's readied changes are unverified,
//! not matched. The null model is her State left as record N has it.

pub mod cases;

use core::cell::Cell;
use core::fmt;

use core::num::Wrapping;
use data::pose::animation::AnimationFrame;
use data::pose::{PoseId, Poses};
use game::samus_appearance_bridge::{Clips, Imported, Moved};
use oracle_record::FrameRecord;
use samus_appearance::{
    BehaviourId, CATALOGUE, Content, Control, Dash, Frames, Inputs, Instruction, Laps, LapsLeft,
    Liquid, MoveEvent, Playhead, Rules, RunDown, Slowdown, SoundsWaiting, State, Step, update,
};
use types::{Action, BoostStage, Item, Parts as _, Set};

use super::movement::Movement;
use super::{Domain, Entry, Field, FieldSet, Hides, Interval, Mask, Uncompared};
use crate::bundle::ContentError;
use crate::cache::CutKind;
use crate::decode::DecodeError;
use crate::domain_field;
use crate::source::{self, WramByte, WramWord};

/// Why the domain can't load: the engine's content, which holds her clips and which
/// samus-movement's model reads.
pub type LoadError = ContentError;

/// A work RAM word the harness reads that `source` doesn't name.
const fn wram(name: &'static str, offset: u16) -> WramWord {
    WramWord { name, offset }
}

/// The game state the main loop runs gameplay in.
const MAIN_GAMEPLAY: u16 = 8;
/// Bank $90 log: the handler gameplay runs for Samus once a frame ($90:E722).
const SAMUS_NEW_STATE_HANDLER: WramWord = wram("FRAME_HANDLER_BETA", 0x0A44);
/// The game state a door's start is in at the frame's end: her frame runs before the door's.
const DOOR_STARTS: u16 = 9;
/// The handlers that run her animation: the normal one, the demos' two, the elevator's and the
/// Ceres escape's timer, which runs the normal one first (`$90:E725`, `$90:E7F5`, `$90:E833`,
/// `$90:E8EC`, `$90:E8AA`).
const ANIMATING: [u16; 5] = [0xE725, 0xE7F5, 0xE833, 0xE8EC, 0xE8AA];
/// Bank $90 log: the speed booster's counter, the stage in its high byte and the laps in its low.
const SPEED_BOOST_COUNTER: WramWord = source::SPEED_BOOSTER_LEVEL;
/// Bank $90 log: whether she is dashing, 0 when not.
const DASHING: WramWord = source::SAMUS_HAS_MOMENTUM_FLAG;
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
/// Her vertical directions up and down (`0B36`).
const UP: u16 = 1;
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
const WATER_START: u16 = 3;
const LAVA_ACID_START: u16 = 2;

/// The declared fields by their places in [`Appearance`]'s `FIELDS`.
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
    pub time_left: Frames,
    /// The boost stage.
    pub stage: BoostStage,
    /// The laps left.
    pub laps_left: LapsLeft,
}

impl From<&State> for Declared {
    fn from(state: &State) -> Self {
        Self {
            step: state.playhead.step,
            time_left: state.playhead.time_left,
            stage: state.stage,
            laps_left: state.laps_left,
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

/// The samus-appearance domain, over the importer's clips, with the samus-movement model that supplies
/// her moves' outputs.
pub struct Appearance {
    motion: Movement,
    /// The frame count at the record before the current frame's, and its record's index; then the
    /// current frame's. The frame a message box ends is told by the stall into record N.
    before: Cell<Option<(u64, u16)>>,
    last: Cell<Option<(u64, u16)>>,
}

impl Appearance {
    /// The domain over the ROM `image`.
    ///
    /// # Errors
    ///
    /// [`LoadError`] when the engine's content, her clips and paces among it, doesn't import.
    pub fn load(image: &[u8]) -> Result<Self, LoadError> {
        Ok(Self {
            motion: Movement::load(image)?,
            before: Cell::new(None),
            last: Cell::new(None),
        })
    }

    /// The imported clips.
    #[must_use]
    pub const fn clips(&self) -> &Clips {
        &self.motion.content().samus_clips
    }

    /// The pose definitions, which give her radius in each pose.
    const fn poses(&self) -> &Poses {
        &self.motion.content().poses
    }

    /// Her pose in `record`, with its clip.
    fn imported(&self, record: &FrameRecord) -> Result<&Imported, Skip> {
        let id = word(record, source::SAMUS_POSE)?;
        self.clips().of(PoseId(id)).ok_or(Skip::Pose(id))
    }

    /// Her y radius in `imported`'s pose.
    fn radius(&self, imported: &Imported) -> u8 {
        self.poses()
            .get(imported.id)
            .map_or(0, |definition| definition.y_radius)
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
                time_left: Frames(left),
            },
            stage,
            // The counter's 0 laps are her run without the speed booster.
            laps_left: Laps::new(laps).map_or(LapsLeft::WithoutBooster, LapsLeft::WithBooster),
        })
    }

    /// The frame's inputs at the animation's turn, from records `from` and `next`.
    fn inputs(&self, from: &FrameRecord, next: &FrameRecord) -> Result<Frame, Skip> {
        let before = self.imported(from)?;
        let after = self.imported(next)?;
        let held = word(from, source::JOYPAD1_LASTKEYS)?;
        let pressing = |binding| Ok::<_, Skip>(held & word(from, binding)? != 0);
        let restarts = byte(from, source::SAMUS_MOVEMENT_TYPE)? == STANDING
            && STANDING_UNAIMED.contains(&before.id.0)
            && pressing(BINDING_SHOOT)?;
        // Where samus-movement's model runs the frame, all it gives is the bridge's (`Moved`), the
        // standing restart included; elsewhere the records give it, the restart by `$90:A3AD`'s test.
        let (moved, from_the_record) =
            if let Ok(outcome) = self.motion.outcome(from, Interval::ending_at(next)) {
                (Moved::from(&outcome), false)
            } else {
                let (mut moved, readied) = from_the_records(before, after, from, next)?;
                moved.moves[MoveEvent::StandingRestart] = restarts;
                (moved, readied)
            };
        let radius = (self.radius(before), self.radius(after));
        let equipped = word(next, source::EQUIPPED_ITEMS)?;
        let mut items = Set::<Item>::default();
        for (item, bit) in ITEM_BITS {
            items[item] = equipped & bit != 0;
        }
        let waiting = byte(from, SOUND_3_WRITE)?.wrapping_sub(byte(from, SOUND_3_READ)?);
        let slowdown = word(next, FRAME_BUFFER)?;
        Ok(Frame {
            inputs: Inputs {
                figure: before.figure,
                figure_after: after.figure,
                moves: moved.moves,
                dash: moved.dash,
                run_held: pressing(source::BINDING_RUN)?,
                heading_down: moved.heading_down,
                vertical_speed: moved.vertical_speed,
                energy: word(from, source::SAMUS_HEALTH)?,
                items,
                liquid: Liquid {
                    slowdown: slowdown_of("SAMUS_ANIM_FRAME_BUFFER", slowdown)?,
                    start_slowdown: start_slowdown(next, radius.1, equipped)?,
                    submerged: submerged(next, radius.0, equipped)?,
                },
                sounds_waiting: SoundsWaiting::new(waiting & QUEUE_PLACES)
                    .ok_or_else(|| Skip::Undecodable("SOUND_QUEUE_3", u16::from(waiting)))?,
            },
            readied_from_the_record: from_the_record,
        })
    }
}

/// Her motion's outputs on a frame samus-movement's model doesn't run, from records `from` and
/// `next`, all but the standing restart, and whether a wall jump was readied, which is read off
/// record N+1's own step: a dash begun or lost is the flag `0B3C`'s change from N to N+1, and
/// otherwise the dash is the flag at N, handed over the super special command at N+1, heading down her direction at N+1 and
/// her vertical speed record N+1's, read as signed and turned upward when she rises.
fn from_the_records(
    before: &Imported,
    after: &Imported,
    from: &FrameRecord,
    next: &FrameRecord,
) -> Result<(Moved, bool), Skip> {
    let dash = [word(from, DASHING)? != 0, word(next, DASHING)? != 0];
    let readied = readied(before, after, from, next)?;
    let mut moves = Set::<MoveEvent>::default();
    moves[MoveEvent::WallJumpReadied] = readied;
    moves[MoveEvent::DashLost] = dash == [true, false];
    moves[MoveEvent::HandedOver] = word(next, POSE_CHANGE_COMMAND)? == ANIMATION_FINISHED
        && word(next, SUPER_SPECIAL_POSE)? & NO_POSE == 0;
    let direction = word(next, source::SAMUS_Y_DIR)?;
    let speed = Wrapping(
        (u32::from(word(next, source::SAMUS_Y_SPEED)?) << 16)
            | u32::from(word(next, source::SAMUS_Y_SUBSPEED)?),
    );
    let motion = Moved {
        moves,
        dash: match dash {
            [false, true] => Dash::Began,
            [true, _] => Dash::On,
            [false, false] => Dash::Off,
        },
        heading_down: direction == DOWN,
        vertical_speed: if direction == UP { -speed } else { speed },
    };
    Ok((motion, readied))
}

/// A slowdown word as the model's slowdown: a harness defect past the most a liquid gives.
fn slowdown_of(name: &'static str, word: u16) -> Result<Slowdown, Skip> {
    u8::try_from(word)
        .ok()
        .and_then(Slowdown::new)
        .ok_or(Skip::Undecodable(name, word))
}

/// Whether samus-movement's near-wall test readied a wall jump on a frame its model doesn't run in,
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

/// The start's slowdown for the pose she takes, of y radius `radius`: with the gravity suit the
/// X speed divisor; otherwise 3 with the water's surface above the feet of the pose she takes, 2
/// with the lava's or acid's, else the divisor (`$91:FB08`).
fn start_slowdown(next: &FrameRecord, radius: u8, equipped: u16) -> Result<Slowdown, Skip> {
    let divisor = word(next, X_SPEED_DIVISOR)?;
    if equipped & GRAVITY_SUIT != 0 {
        return slowdown_of("X_SPEED_DIVISOR", divisor);
    }
    let (_, bottom) = boundaries(next, radius)?;
    let (water, lava) = surfaces(next)?;
    if water.is_some_and(|surface| surface < bottom) {
        slowdown_of("the water's start", WATER_START)
    } else if lava.is_some_and(|surface| surface < bottom) {
        slowdown_of("the lava's or acid's start", LAVA_ACID_START)
    } else {
        slowdown_of("X_SPEED_DIVISOR", divisor)
    }
}

/// Whether she is submerged as a wall jump's branch tests it, in a pose of y radius `radius`:
/// no gravity suit, and a liquid's surface above her top (`$90:841D`).
fn submerged(next: &FrameRecord, radius: u8, equipped: u16) -> Result<bool, Skip> {
    if equipped & GRAVITY_SUIT != 0 {
        return Ok(false);
    }
    let (top, _) = boundaries(next, radius)?;
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

/// Gameplay, or a door's start, at the frame's end: her frame ran before the door's.
fn gameplay_or_door(record: &FrameRecord) -> Result<(), Skip> {
    match word(record, source::GAME_STATE)? {
        MAIN_GAMEPLAY | DOOR_STARTS => Ok(()),
        state => Err(Skip::GameState(state)),
    }
}

impl Domain for Appearance {
    const NAME: &'static str = "samus-appearance";
    type State = State;
    type Inputs = ();
    type World = Frame;
    type Fields = Declared;
    type Skip = Skip;
    const FIELDS: &'static [Field<Declared>] = &[
        domain_field!("step", step),
        domain_field!("time_left", time_left),
        domain_field!("stage", stage),
        domain_field!("laps_left", laps_left),
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

    /// A stall from N to N+1 is the message box's or the fanfare's wait, unless the count moved
    /// into N: then it is the frame a message box opens, in which her frame ran before the box's
    /// wait. A row's first record has no count before it, so its stall is taken as a wait.
    fn world(&self, from: &FrameRecord, next: &FrameRecord) -> Result<Frame, Skip> {
        gameplay_or_door(next)?;
        let count = word(from, source::NMI_FRAME_COUNTER_WORD)?;
        let stalled_into = self
            .before
            .get()
            .filter(|&(index, _)| index < from.index)
            .map(|(_, before)| before == count);
        let stalls = word(next, source::NMI_FRAME_COUNTER_WORD)? == count;
        match (stalled_into, stalls) {
            (Some(true) | None, true) => return Err(Skip::Stalled),
            (Some(true), false) => return Err(Skip::MessageBoxEnded),
            (Some(false), true | false) | (None, false) => {}
        }
        self.inputs(from, next)
    }

    /// The boost on a frame it changes with no dash at N, none beginning and none lost: no rule
    /// of the animation changes it then, and the shinespark's (S3) clears it so as the shine
    /// ends (terrain-quicksand 156 to 157, at the morph). Record N+1's boost is read only to find
    /// that writer's frames.
    fn masks(&self, from: &FrameRecord, next: &FrameRecord, (): &()) -> Result<Vec<Mask>, Skip> {
        let dash = [word(from, DASHING)?, word(next, DASHING)?];
        let boost = [
            word(from, SPEED_BOOST_COUNTER)?,
            word(next, SPEED_BOOST_COUNTER)?,
        ];
        Ok(if dash == [0, 0] && boost[0] != boost[1] {
            vec![Mask {
                reason: "another domain: the boost changes with no dash, the shinespark's as its shine ends (S3)",
                fields: BOOST,
                hides: Hides::AnotherWriter,
            }]
        } else {
            Vec::new()
        })
    }

    fn neutral(&self, frame: &Frame) -> Option<Frame> {
        frame.readied_from_the_record.then(|| {
            let mut inputs = frame.inputs;
            inputs.moves[MoveEvent::WallJumpReadied] = false;
            Frame {
                inputs,
                readied_from_the_record: false,
            }
        })
    }

    fn step(&self, state: &mut State, (): &(), frame: &mut Frame) {
        *state = update(*state, &frame.inputs, self.clips(), &Rules::VANILLA).0;
    }

    fn exercised(&self, state: &State, (): &(), frame: &Frame) -> Vec<usize> {
        exercised(*state, &frame.inputs, self.clips())
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
    let clip = content.clip(&inputs.figure);
    let mut hit = vec![B::ClipPlays];
    let moves = &inputs.moves;
    if moves[MoveEvent::StandingRestart] {
        hit.push(B::StandingRestart);
    }
    if moves[MoveEvent::WallJumpReadied] && clip.kick.is_some() {
        hit.push(B::WallJumpReady);
    }
    let fast = inputs.vertical_speed.size() >= Rules::VANILLA.fast_fall_speed;
    if clip
        .fast_fall
        .is_some_and(|step| fast && state.playhead.step < step)
    {
        hit.push(B::FastFall);
    }
    let rising = clip.rising == Some(state.playhead.step)
        && state.playhead.time_left == Frames(1)
        && !inputs.heading_down;
    if rising {
        hit.push(B::RisingHold);
    }
    let running = inputs.dash != Dash::Off && inputs.figure.pose.action == Action::Running;
    let played = !rising && !moves[MoveEvent::StandingRestart];
    if let (true, RunDown::MovesOn(_, next)) = (played, state.playhead.run_down(clip)) {
        hit.extend(moved_on(next, running, inputs, state));
    }
    if inputs.dash == Dash::Began || moves[MoveEvent::DashLost] {
        hit.push(B::BoostStages);
    }
    if inputs.figure != inputs.figure_after {
        hit.push(B::StartingClip);
        if moves[MoveEvent::HandedOver] && inputs.liquid.slowdown != Slowdown::NONE {
            hit.push(B::HandOverSlowdown);
        }
    }
    if content.clip(&inputs.figure_after).kick.is_some() {
        hit.push(B::SpinTurnedFar);
    }
    hit.sort_by_key(|id| CATALOGUE.iter().position(|entry| entry.id == *id));
    hit.dedup();
    hit
}

/// The entries the clip moving on to `next` brings into play.
fn moved_on(next: Instruction, running: bool, inputs: &Inputs, state: State) -> Vec<BehaviourId> {
    use BehaviourId as B;
    let booster = inputs.items[Item::SpeedBooster];
    let control = match next {
        Instruction::Show(_) if running => return vec![B::RunPace],
        Instruction::Show(_) => return Vec::new(),
        Instruction::Control(control) => control,
    };
    match control {
        _ if running && inputs.run_held && !booster => vec![B::RunPace],
        _ if running && inputs.run_held => {
            let up_to_boosting =
                state.laps_left.after_lap().is_none() && state.stage == BoostStage::Stage3;
            if up_to_boosting {
                vec![B::RunPace, B::BoostStages, B::BoostingLap]
            } else {
                vec![B::RunPace, B::BoostStages]
            }
        }
        Control::Again(_) | Control::Rest => vec![B::ClipControls],
        Control::Breathe => vec![B::Breathing],
        Control::End | Control::EndUnlessJump => vec![B::ClipEnds],
        Control::Branch(_) => vec![B::WallJumpBranch],
    }
}

#[cfg(test)]
mod tests {
    use super::{Appearance, BOOST, PLAYHEAD};
    use crate::harness::Domain;

    #[test]
    fn the_field_sets_name_the_declared_fields_at_their_places() {
        let names: Vec<&str> = <Appearance as Domain>::FIELDS
            .iter()
            .map(|field| field.name)
            .collect();
        assert_eq!(names, ["step", "time_left", "stage", "laps_left"]);
        assert!(PLAYHEAD.contains(0) && PLAYHEAD.contains(1) && !PLAYHEAD.contains(2));
        assert!(BOOST.contains(2) && BOOST.contains(3) && !BOOST.contains(1));
    }
}
