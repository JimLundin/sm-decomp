//! The camera as a harness domain: the camera model (`clean_model_camera`) against the oracle.
//!
//! It is checked frame by frame on the spec's declared fields (`docs/behaviours/camera.md` §1),
//! by the decode notes' harness note (`docs/behaviours/camera.decode.md`, "The harness").
//! The camera is downstream of her motion and nothing upstream reads it, so it is checked on its
//! own: its State (the camera position and where it last saw her) is decoded from record N, the
//! inputs from record N+1 as the scroll read them (her position after everything moved her, her
//! facing and state, the room's screens), and `camera_from` from samus-motion's model run on the
//! frame, never from record N+1's word, which is her y at the scroll by construction. The
//! compared fields are the camera's position, pixel and subpixel parts. Where it last saw her is
//! decoded from the same words as her position, so it is reported and masked, never counted.
//! Nothing answers from the camera's own fields of record N+1, so no field is unverified; the null
//! model is the camera left where record N has it.

use core::fmt;
use core::num::Wrapping;

use clean_model_camera::{
    BehaviourId, CATALOGUE, Camera, Frame, Heights, LeadSetting, Room, RoomError, Rules,
    Samus as Her, ScreenKind,
};
use clean_types::{Facing, Fixed, Pair, Parts, Px, below, fixed};
use game::camera_world;
use game::model::num::Fixed as Exact;
use game::state::room::{Scrollers, StaleScroll};
use oracle_record::FrameRecord;

use super::samus_motion::{self, Ps, SamusMotion, Skip as Motion};
use super::{Domain, Entry, Field, FieldSet, Interval, Mask, Uncompared};
use crate::bundle::ContentError;
use crate::decode::DecodeError;
use crate::domain_field;
use crate::source::{self, WramByte, WramWord};

/// The game state the main loop runs gameplay in.
const MAIN_GAMEPLAY: u16 = 8;

/// A work RAM word the harness reads that `source` doesn't name.
const fn wram(name: &'static str, offset: u16) -> WramWord {
    WramWord { name, offset }
}

/// Bank $80 log: the room's size in screens across and down.
const ROOM_WIDTH_SCROLLS: WramWord = wram("ROOM_WIDTH_SCROLLS", 0x07A9);
const ROOM_HEIGHT_SCROLLS: WramWord = wram("ROOM_HEIGHT_SCROLLS", 0x07AB);
/// Bank $90 log: the knockback's direction, 0 when she takes none.
const KNOCKBACK_DIRECTION: WramWord = wram("KNOCKBACK_DIR", 0x0A52);
/// Bank $90 log: time frozen (the x-ray scope, the reserve tanks' refill).
const TIME_IS_FROZEN: WramWord = wram("TIME_IS_FROZEN", 0x0A78);
/// Bank $90 log, `$90:94F7`: the grapple beam's slow scrolling.
const SLOW_GRAPPLE_SCROLLING: WramWord = wram("SLOW_GRAPPLE_SCROLLING", 0x0CF8);
/// Bank $90 log, `$90:9589`: the scrolling-finished hook, Spore Spawn's floor.
const SCROLLING_FINISHED_HOOK: WramWord = wram("SCROLLING_FINISHED_HOOK", 0x07E9);
/// Bank $90 log: the handler gameplay runs for Samus once a frame ($90:E722).
const SAMUS_NEW_STATE_HANDLER: WramWord = wram("FRAME_HANDLER_BETA", 0x0A44);
/// Bank $91 log, `$91:EB88`: the super special prospective pose change command.
const POSE_CHANGE_COMMAND: WramWord = source::SAMUS_HURT_SWITCH_INDEX;
/// Bank $A0 log: the enemy slots, 40h bytes each, and the extra words beside them at $7E:8000.
const ENEMY_SLOTS: u16 = 0x0F78;
const ENEMY_EXTRA: u16 = 0x8000;
const ENEMY_SLOT_SIZE: u16 = 0x40;
const ENEMY_SLOT_COUNT: u16 = 32;
/// An enemy's function pointer, in its slot.
const ENEMY_FUNCTION: u16 = 0x3A;

/// The facing byte that faces left (the scroll tests only it) and the one that faces right.
const FACING_LEFT: u8 = 4;
const FACING_RIGHT: u8 = 8;
/// The movement type of the moonwalk.
const MOONWALKING: u8 = 0x10;
/// The movement types whose own domains move where the camera saw her, every frame: the grapple
/// beam, Draygon holding her, the shinespark.
const GRAPPLING: u8 = 0x16;
const HELD_BY_DRAYGON: u8 = 0x1A;
const SHINESPARKING: u8 = 0x1B;
/// The X acceleration mode of turning round: the skid.
const TURNING_AROUND: u16 = 1;
/// The vertical direction of the rise state.
const UP: u16 = 1;
/// Samus's new state handler of her appearance when a game loads ($90:E86A).
const APPEARANCE: u16 = 0xE86A;
/// Bank $90 log: her movement handler, and the knockback's, which a hit's start sets
/// ($91:EE1F, $91:EE40).
const SAMUS_MOVEMENT_HANDLER: WramWord = wram("SAMUS_MOVEMENT_HANDLER", 0x0A58);
const KNOCKBACK: u16 = 0xDF38;
/// The super special pose change command that sets her position to the view's centre
/// ($91:F37C).
const CENTRE_ON_VIEW: u16 = 4;
/// The yapping maw's species, and its extra word that says it holds her ($A8:A634).
const YAPPING_MAW: u16 = 0xE7BF;
const MAW_GRABBING: u16 = 0x0020;
/// The equipment bits of the two suits, whose pick-ups place her at the view's centre, across
/// ($91:D57D, $91:D580) and down ($91:D58A, $91:D58D).
const SUITS: u16 = 0x0021;

/// An enemy whose code moves where the camera saw her, or the view, while it is live.
struct Writer {
    /// Its species, any of these.
    species: &'static [u16],
    /// The mask's reason.
    reason: &'static str,
    /// The fields it moves.
    fields: FieldSet,
}

/// The bosses that move her or the view while they are live: their collisions, their holds and
/// their rooms' code.
const WRITERS: [Writer; 4] = [
    Writer {
        species: &[0xDDBF],
        reason: "another domain: Crocomire moves her ($A4:8C95), and his room shakes the view across ($8F:E8CD)",
        fields: CAMERA,
    },
    Writer {
        species: &[0xE2BF],
        reason: "another domain: Kraid moves her ($A7:B0F3) and holds her x on the first screen ($A7:C9EE)",
        fields: CAMERA,
    },
    // Ceres Ridley (E13Fh) has no feet.
    Writer {
        species: &[0xE17F],
        reason: "another domain: Ridley's feet set where it saw her to her position ($A6:E501)",
        fields: CAMERA,
    },
    Writer {
        species: &[0xEC3F, 0xEC7F, 0xECBF, 0xEEBF],
        reason: "another domain: Mother Brain moves her ($A9:BBFD, $A9:BC3F), and the Shitroid waits for the view ($A9:EFE6)",
        fields: CAMERA,
    },
];

/// The gunship's top, and its idle function, the one that only waits for her to enter: every
/// other function holds her and moves her, across ($A2:A925, $A2:AA1D, where she enters) and
/// down ($A2:A8DC landing, $A2:AC2F liftoff, lowering and raising her).
const GUNSHIP: u16 = 0xD07F;
const GUNSHIP_IDLE: u16 = 0xA9BD;
/// The Dachora room, whose main code scrolls the view right ($8F:C1E6) while its scroll Bh is
/// green.
const DACHORA_ROOM: u16 = 0x9CB3;
const DACHORA_SCREEN: usize = 0x0B;
const GREEN: u8 = 2;

/// The declared fields by their places in [`CameraDomain`]'s `FIELDS`.
const X: FieldSet = FieldSet::single(0);
const Y: FieldSet = FieldSet::single(1);
const CAMERA: FieldSet = X.union(Y);
const SEEN: FieldSet = FieldSet::single(2);

/// The declared fields a catalogue entry's output reaches: the axis it moves the view on, or
/// both. Where it last saw her is reported only, so no entry is credited by it.
const fn shown_by(id: BehaviourId) -> FieldSet {
    use BehaviourId as B;
    match id {
        B::LeadingHer | B::ScreensRunOn => X,
        B::AimingHeight | B::ShortScreens | B::DownwardLooksAhead | B::BelowTheRoom => Y,
        B::FollowingHer
        | B::Remembering
        | B::CreepReadsRightward
        | B::ClosedScreens
        | B::RoomEdges
        | B::Settling
        | B::TimeStopped
        | B::PushCarries => CAMERA,
    }
}

/// Why a frame isn't one the camera is compared on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Skip {
    /// A source couldn't be read, or holds a code the model has no value for: a harness defect.
    Unreadable(DecodeError),
    /// The room's screens aren't a room the model takes: a harness defect.
    Room(RoomError),
    /// The main loop isn't in gameplay at record N or N+1: no scroll runs.
    GameState(u16),
    /// The requested-NMI count advanced by other than one: a lag frame (no scroll) or two
    /// frames (two scrolls).
    Frames(u16),
    /// The room changed: the door's domain places the view.
    RoomChange,
}

impl fmt::Display for Skip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(error) => write!(f, "unreadable: {error:?}"),
            Self::Room(error) => write!(f, "the room's screens aren't a room: {error:?}"),
            Self::GameState(code) => write!(f, "game state {code:#04X}, not gameplay"),
            Self::Frames(count) => write!(f, "{count} frames ran, not one"),
            Self::RoomChange => f.write_str("the room changed (the door's domain)"),
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

/// The model's quantity from the pixel and subpixel words.
fn quantity(record: &FrameRecord, pixel: WramWord, sub: WramWord) -> Result<Fixed, Skip> {
    Ok(fixed(Wrapping(word(record, pixel)?), word(record, sub)?))
}

/// The camera position in `record`: layer 1's.
fn position(record: &FrameRecord) -> Result<Pair<Fixed>, Skip> {
    Ok(Pair {
        x: quantity(record, source::LAYER1_X_POS, source::LAYER1_X_SUBPOS)?,
        y: quantity(record, source::LAYER1_Y_POS, source::LAYER1_Y_SUBPOS)?,
    })
}

/// Samus's previous position in `record`: where the camera last saw her.
fn seen(record: &FrameRecord) -> Result<Pair<Fixed>, Skip> {
    Ok(Pair {
        x: quantity(
            record,
            source::SAMUS_PREV_X_POS,
            source::SAMUS_PREV_X_SUBPOS,
        )?,
        y: quantity(
            record,
            source::SAMUS_PREV_Y_POS,
            source::SAMUS_PREV_Y_SUBPOS,
        )?,
    })
}

/// The room's screens as the scroll read them in `record`: the whole scroll table, one run
/// whose first `w × h` bytes are the room's and the rest what its loading left after them.
fn screens(record: &FrameRecord) -> Result<Screens, Skip> {
    // Only red and blue are ever tested: any other byte reads as green.
    let run = source::SCROLLS
        .read(&record.wram)?
        .iter()
        .map(|&area| camera_world::screen(StaleScroll::from(area).area()))
        .collect();
    Ok(Screens {
        run,
        width: usize::from(word(record, ROOM_WIDTH_SCROLLS)?),
        height: usize::from(word(record, ROOM_HEIGHT_SCROLLS)?),
    })
}

/// The room's screens as one run, and its size in screens, which the attribution reads.
struct Screens {
    run: Vec<ScreenKind>,
    width: usize,
    height: usize,
}

impl Screens {
    /// The room's own screens: the run's first `w × h`.
    fn room(&self) -> &[ScreenKind] {
        let screens = self.width.saturating_mul(self.height);
        self.run.get(..screens).unwrap_or(&self.run)
    }

    /// Whether the run goes on below the room.
    const fn below(&self) -> bool {
        self.run.len() > self.width.saturating_mul(self.height)
    }

    /// Whether the room's last screen in the row holding `y` is closed.
    fn edge_closed(&self, y: Px) -> bool {
        let [row, _] = y.0.to_be_bytes();
        usize::from(row)
            .checked_mul(self.width)
            .and_then(|start| start.checked_add(self.width.checked_sub(1)?))
            .and_then(|screen| self.room().get(screen))
            .is_some_and(|&kind| matches!(kind, ScreenKind::Closed))
    }
}

/// The lead setting by the camera distance index.
fn lead(record: &FrameRecord) -> Result<LeadSetting, Skip> {
    Ok(match word(record, source::CAMERA_DISTANCE_INDEX)? {
        0 => LeadSetting::Ahead,
        2 => LeadSetting::Near,
        4 => LeadSetting::KeepLeft,
        6 => LeadSetting::KeepRight,
        value => {
            return Err(Skip::Unreadable(DecodeError::Unrepresentable {
                source: source::CAMERA_DISTANCE_INDEX.name,
                value,
            }));
        }
    })
}

/// What the camera read of her and the room at the scroll of the frame ending at `next`, but
/// `camera_from`, which is samus-motion's.
fn upstream(next: &FrameRecord, camera_from: Fixed) -> Result<Upstream, Skip> {
    let facing = match byte(next, source::SAMUS_POSE_X_DIR)? {
        FACING_LEFT => Facing::Left,
        FACING_RIGHT => Facing::Right,
        _ => Facing::Screen,
    };
    let backwards = word(next, KNOCKBACK_DIRECTION)? != 0
        || byte(next, source::SAMUS_MOVEMENT_TYPE)? == MOONWALKING
        || word(next, source::SAMUS_X_ACCEL_MODE)? == TURNING_AROUND;
    let screens = screens(next)?;
    let scrollers = Scrollers {
        up: word(next, source::UP_SCROLLER)?,
        down: word(next, source::DOWN_SCROLLER)?,
    };
    let heights = camera_world::heights(scrollers);
    let room = Room::new(screens.run.clone(), screens.width, screens.height, heights)
        .map_err(Skip::Room)?;
    Ok(Upstream {
        samus: Her {
            // The scroll ends by setting her previous position to her position: exactly what
            // it read.
            position: seen(next)?,
            camera_from,
            facing,
            backwards,
            rising: word(next, source::SAMUS_Y_DIR)? == UP,
        },
        room,
        screens,
        heights,
        lead: lead(next)?,
        time_stopped: word(next, TIME_IS_FROZEN)? != 0,
    })
}

/// The camera's inputs as the scroll read them, from record N+1 and samus-motion's model.
pub struct Upstream {
    samus: Her,
    room: Room,
    /// The run of screens and the size `room` was built from.
    screens: Screens,
    /// The heights `room` was built with, from which the attribution finds the aim down.
    heights: Heights,
    lead: LeadSetting,
    time_stopped: bool,
}

impl Upstream {
    /// The model's input for the frame.
    const fn frame(&self) -> Frame<'_> {
        Frame {
            samus: self.samus,
            room: &self.room,
            lead: self.lead,
            time_stopped: self.time_stopped,
        }
    }
}

/// What record N gives besides the State: where samus-motion's model placed the y the camera
/// follows her from.
#[derive(Clone, Copy, Debug)]
pub enum Placement {
    /// Her motion's frame ran through samus-motion's model: `camera_from`, where it isn't her
    /// start y.
    Model(Option<Fixed>),
    /// Her motion doesn't run in the frame (no gameplay pass, lag, a frame handler that runs no
    /// motion, the message box): nothing places her.
    NoMotion,
    /// Her motion runs, but samus-motion's model doesn't cover the frame (a pose outside its
    /// domain, a deferred movement): its placement is that model's to come.
    Uncovered,
}

impl TryFrom<Result<Option<Fixed>, Motion>> for Placement {
    type Error = Skip;

    /// The frame's placement by samus-motion's answer; a decode failure is a harness defect.
    fn try_from(answer: Result<Option<Fixed>, Motion>) -> Result<Self, Skip> {
        Ok(match answer {
            Ok(placed) => Self::Model(placed),
            Err(Motion::Unreadable(error)) => return Err(Skip::Unreadable(error)),
            Err(
                Motion::GameState(_)
                | Motion::NotAtFrameEnd
                | Motion::Waiting(_)
                | Motion::NotRunning(_),
            ) => Self::NoMotion,
            Err(Motion::Pose(_) | Motion::MovementHandler(_) | Motion::Deferred(_)) => {
                Self::Uncovered
            }
        })
    }
}

impl Placement {
    /// `camera_from`, where it isn't her start y.
    const fn placed(self) -> Option<Fixed> {
        match self {
            Self::Model(placed) => placed,
            Self::NoMotion | Self::Uncovered => None,
        }
    }
}

/// The declared fields: the camera position, and where it last saw her, reported only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Declared {
    /// The camera position across.
    pub x: Ps,
    /// The camera position down.
    pub y: Ps,
    /// Where it last saw her, across and down.
    pub seen: (Ps, Ps),
}

/// The engine's quantity as the motion harness shows it.
const fn ps(value: Fixed) -> Ps {
    Ps(Exact(value.0))
}

impl From<&Camera> for Declared {
    fn from(camera: &Camera) -> Self {
        Self {
            x: ps(camera.position().x),
            y: ps(camera.position().y),
            seen: (ps(camera.seen().x), ps(camera.seen().y)),
        }
    }
}

/// The camera domain, with the samus-motion model that supplies `camera_from`.
pub struct CameraDomain {
    motion: SamusMotion,
}

impl CameraDomain {
    /// The domain over the ROM `image`, which samus-motion's model reads.
    ///
    /// # Errors
    ///
    /// [`ContentError`] when the content can't be read.
    pub fn load(image: &[u8]) -> Result<Self, ContentError> {
        Ok(Self {
            motion: SamusMotion::load(image)?,
        })
    }
}

/// Whether the main loop runs gameplay in `record`.
fn gameplay(record: &FrameRecord) -> Result<(), Skip> {
    match word(record, source::GAME_STATE)? {
        MAIN_GAMEPLAY => Ok(()),
        code => Err(Skip::GameState(code)),
    }
}

/// An enemy slot as the masks read it.
#[derive(Clone, Copy, Debug)]
struct Slot {
    /// Its species, 0 when free.
    species: u16,
    /// Its function pointer.
    function: u16,
    /// Its extra word at 20h: the yapping maw's hold.
    grabbing: u16,
}

/// The enemy slots of `record`.
fn slots(record: &FrameRecord) -> Result<Vec<Slot>, Skip> {
    (0..ENEMY_SLOT_COUNT)
        .map(|slot| {
            let base = ENEMY_SLOTS.wrapping_add(slot.wrapping_mul(ENEMY_SLOT_SIZE));
            let extra = ENEMY_EXTRA.wrapping_add(slot.wrapping_mul(ENEMY_SLOT_SIZE));
            Ok(Slot {
                species: word(record, wram("ENEMY_ID", base))?,
                function: word(
                    record,
                    wram("ENEMY_FUNCTION", base.wrapping_add(ENEMY_FUNCTION)),
                )?,
                grabbing: word(
                    record,
                    wram("ENEMY_RAM_7E8020", extra.wrapping_add(MAW_GRABBING)),
                )?,
            })
        })
        .collect()
}

/// The masks of the enemies that move her or the view in the frame from `from` to `next`,
/// live at either record: a yapping maw holding her, the bosses, the gunship holding her.
fn enemies(from: &FrameRecord, next: &FrameRecord) -> Result<Vec<Mask>, Skip> {
    let mut masks = Vec::new();
    let mut mask = |reason, fields| masks.push(Mask { reason, fields });
    let (before, after) = (slots(from)?, slots(next)?);
    let live = |species: u16| {
        before
            .iter()
            .chain(&after)
            .any(|slot| slot.species == species)
    };
    if before
        .iter()
        .chain(&after)
        .any(|slot| slot.species == YAPPING_MAW && slot.grabbing != 0)
    {
        mask(
            "another domain: a yapping maw holds her and caps the scroll's speed",
            CAMERA,
        );
    }
    for writer in &WRITERS {
        if writer.species.iter().any(|&species| live(species)) {
            mask(writer.reason, writer.fields);
        }
    }
    if before
        .iter()
        .chain(&after)
        .any(|slot| slot.species == GUNSHIP && slot.function != GUNSHIP_IDLE)
    {
        mask(
            "another domain: the gunship holds her and moves her",
            CAMERA,
        );
    }
    Ok(masks)
}

/// The masks of the frames on which samus-motion's harness defers her y to another domain
/// (adversary 42, C2 and M4, by its own predicates) and that deferral reaches what the camera
/// reads (adversary 49, F1). The camera takes her position from record N+1; from samus-motion it
/// takes only `camera_from`, which differs from the seen y through a placement alone: a pose
/// change (camera.gaps C2) or a placement samus-motion's step reports (C3). On a deferred frame
/// with neither, nothing the camera reads is unverified, so y stays compared. x stays compared
/// throughout: the knockback's turn of the lead is the camera's own `LeadingHer`.
fn deferred(
    from: &FrameRecord,
    next: &FrameRecord,
    placement: Placement,
) -> Result<Vec<Mask>, Skip> {
    let mut masks = Vec::new();
    let places = word(from, source::SAMUS_POSE)? != word(next, source::SAMUS_POSE)?
        || placement.placed().is_some();
    if !places {
        return Ok(masks);
    }
    if samus_motion::pushed(next)? {
        masks.push(Mask {
            reason: "another domain: a block-inside push (quicksand, treadmill) moves her y, which samus-motion defers, and her pose change or its step places the y the camera follows from",
            fields: Y,
        });
    }
    if samus_motion::knockback_tail(from)? {
        masks.push(Mask {
            reason: "another domain: the knockback's tail (the knockback on under the normal handler) decides her y, which samus-motion defers, and her pose change or its step places the y the camera follows from",
            fields: Y,
        });
    }
    Ok(masks)
}

impl Domain for CameraDomain {
    const NAME: &'static str = "camera";
    type State = Camera;
    type Inputs = Placement;
    type World = Upstream;
    type Fields = Declared;
    type Skip = Skip;
    const FIELDS: &'static [Field<Declared>] = &[
        domain_field!("position.x", x),
        domain_field!("position.y", y),
        domain_field!("seen", seen),
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

    fn decode(&self, from: &FrameRecord, interval: Interval) -> Result<(Camera, Placement), Skip> {
        gameplay(from)?;
        let camera = Camera::new(position(from)?, seen(from)?);
        let placement = Placement::try_from(self.motion.placement(from, interval))?;
        Ok((camera, placement))
    }

    fn world(&self, from: &FrameRecord, next: &FrameRecord) -> Result<Upstream, Skip> {
        gameplay(next)?;
        let frames = word(next, source::NMI_FRAME_COUNTER_WORD)?
            .wrapping_sub(word(from, source::NMI_FRAME_COUNTER_WORD)?);
        if frames != 1 {
            return Err(Skip::Frames(frames));
        }
        if word(next, source::ROOM_PTR)? != word(from, source::ROOM_PTR)? {
            return Err(Skip::RoomChange);
        }
        // Filled in by `step` from the frame's placement; the State's seen y until then.
        upstream(next, seen(from)?.y)
    }

    fn masks(
        &self,
        from: &FrameRecord,
        next: &FrameRecord,
        placement: &Placement,
    ) -> Result<Vec<Mask>, Skip> {
        let mut masks = vec![Mask {
            reason: "reported, not counted: where it last saw her is her position's own words",
            fields: SEEN,
        }];
        masks.extend(enemies(from, next)?);
        masks.extend(deferred(from, next, *placement)?);
        let mut mask = |reason, fields| masks.push(Mask { reason, fields });
        let movement = [
            byte(from, source::SAMUS_MOVEMENT_TYPE)?,
            byte(next, source::SAMUS_MOVEMENT_TYPE)?,
        ];
        if movement.contains(&GRAPPLING) {
            mask(
                "another domain: the grapple beam's clamps move where it saw her",
                CAMERA,
            );
        }
        if movement.contains(&SHINESPARKING) {
            mask(
                "another domain: the shinespark's clamp moves where it saw her",
                CAMERA,
            );
        }
        if movement.contains(&HELD_BY_DRAYGON) {
            mask(
                "another domain: Draygon holds her and caps the scroll's speed",
                CAMERA,
            );
        }
        if word(from, SLOW_GRAPPLE_SCROLLING)? != 0 || word(next, SLOW_GRAPPLE_SCROLLING)? != 0 {
            mask(
                "another domain: the grapple beam's slow scrolling moves the view",
                CAMERA,
            );
        }
        if word(next, SCROLLING_FINISHED_HOOK)? != 0 {
            mask(
                "the scrolling-finished hook moves the view down after it follows",
                Y,
            );
        }
        let green = source::SCROLLS
            .read(&next.wram)?
            .get(DACHORA_SCREEN)
            .is_some_and(|&area| area == GREEN);
        if word(from, source::ROOM_PTR)? == DACHORA_ROOM && green {
            mask("the room's code scrolls the view right after it follows", X);
        }
        let handlers = [
            word(from, SAMUS_NEW_STATE_HANDLER)?,
            word(next, SAMUS_NEW_STATE_HANDLER)?,
        ];
        if handlers.contains(&APPEARANCE) {
            mask(
                "another domain: her appearance when a game loads places her",
                Y,
            );
        }
        if word(from, POSE_CHANGE_COMMAND)? == CENTRE_ON_VIEW {
            mask(
                "another domain: a pose change command centres her on the view",
                CAMERA,
            );
        }
        let equipped = |record| word(record, source::EQUIPPED_ITEMS);
        if (equipped(next)? & !equipped(from)?) & SUITS != 0 {
            mask("another domain: a suit's pick-up places her", CAMERA);
        }
        let handler = |record| word(record, SAMUS_MOVEMENT_HANDLER);
        if handler(next)? == KNOCKBACK && handler(from)? != KNOCKBACK {
            mask(
                "another domain: a hit's knockback starts in the pose change and fits its body, placing her",
                Y,
            );
        }
        let pose = |record| word(record, source::SAMUS_POSE);
        if pose(from)? != pose(next)? {
            match placement {
                Placement::Model(_) => {}
                Placement::NoMotion => mask(
                    "camera_from: her motion doesn't run in the frame, yet her pose changes and may place her",
                    Y,
                ),
                Placement::Uncovered => mask(
                    "camera_from: samus-motion's model doesn't cover the frame (a pose outside its domain, a deferred movement), so where her pose change placed her is that model's to come",
                    Y,
                ),
            }
        }
        Ok(masks)
    }

    fn step(&self, camera: &mut Camera, placement: &Placement, upstream: &mut Upstream) {
        // Remembering: where her motion's step placed her, else the y it last saw her at.
        let seen = camera.seen();
        upstream.samus.camera_from = placement.placed().unwrap_or(seen.y);
        *camera = camera.step(&Rules::REFERENCE, &upstream.frame());
    }

    fn exercised(&self, camera: &Camera, placement: &Placement, upstream: &Upstream) -> Vec<usize> {
        exercised(camera, *placement, upstream)
    }

    fn fields(&self, camera: &Camera) -> Declared {
        Declared::from(camera)
    }

    fn expected(&self, next: &FrameRecord) -> Result<Declared, Skip> {
        Ok(Declared::from(&Camera::new(position(next)?, seen(next)?)))
    }

    fn uncompared(&self, skip: &Skip) -> Uncompared {
        match skip {
            Skip::Unreadable(_) | Skip::Room(_) => Uncompared::Defect(skip.to_string()),
            Skip::GameState(_) | Skip::Frames(_) | Skip::RoomChange => {
                Uncompared::Skip(skip.to_string())
            }
        }
    }
}

/// The catalogue entries a frame's situation brings into play, from the camera as record N has
/// it and the frame's inputs: on each axis she moved or it settles, and the room's screens and
/// edges it is at.
fn exercised(camera: &Camera, placement: Placement, upstream: &Upstream) -> Vec<usize> {
    use BehaviourId as B;
    let samus = &upstream.samus;
    let screens = &upstream.screens;
    let last_seen = camera.seen();
    let seen = Pair {
        y: placement
            .placed()
            .map_or(last_seen.y, |placed| placed.with_sub(last_seen.y.sub())),
        ..last_seen
    };
    let moved = |her: Fixed, seen: Fixed| her.pixel() != seen.pixel();
    let creeps = |her: Fixed, seen: Fixed| !moved(her, seen) && her.sub() < seen.sub();
    let (across, down) = (
        moved(samus.position.x, seen.x),
        moved(samus.position.y, seen.y),
    );
    let kinds = || screens.room().iter();
    let last = |screens: usize| -> Px {
        Wrapping(u16::try_from(screens.saturating_sub(1)).unwrap_or(u16::MAX) << 8)
    };
    let (w, h) = (screens.width, screens.height);
    let (x, y) = (camera.position().x.pixel(), camera.position().y.pixel());
    let in_last_row = y >= last(h);
    [
        (B::FollowingHer, across || down),
        (B::LeadingHer, across),
        (B::AimingHeight, down),
        (B::Remembering, true),
        (
            B::CreepReadsRightward,
            creeps(samus.position.x, seen.x) || creeps(samus.position.y, seen.y),
        ),
        (
            B::ClosedScreens,
            kinds().any(|&kind| matches!(kind, ScreenKind::Closed)),
        ),
        (
            B::RoomEdges,
            x == Wrapping(0) || y == Wrapping(0) || x >= last(w) || in_last_row,
        ),
        (
            B::ShortScreens,
            kinds().any(|&kind| matches!(kind, ScreenKind::Short)),
        ),
        (B::Settling, !across || !down),
        (B::TimeStopped, upstream.time_stopped && (!across || !down)),
        (
            B::DownwardLooksAhead,
            down && looks_ahead_decides(camera, seen.y, upstream),
        ),
        // Only from the room's last column, with that screen closed, does the screen after it
        // (the next row's first) decide anything.
        (
            B::ScreensRunOn,
            x >= last(w) && screens.edge_closed(y + Wrapping(MIDDLE)),
        ),
        (B::BelowTheRoom, down && in_last_row && screens.below()),
    ]
    .into_iter()
    .filter(|&(_, exercised)| exercised)
    .filter_map(|(id, _)| CATALOGUE.iter().position(|entry| entry.id == id))
    .collect()
}

/// Whether `DownwardLooksAhead` decides the frame: her move down went past its aim into
/// another screen than the one the aim stops it in, and the room's limits on the move stop it
/// elsewhere when they read that screen than when they read the one it stopped in. The column
/// read is the camera's after it moved across (spec §4), so the model's step gives it.
fn looks_ahead_decides(camera: &Camera, seen: Fixed, upstream: &Upstream) -> bool {
    let Some(went) = down_past_aim(camera.position().y, seen, upstream) else {
        return false;
    };
    let across = camera
        .step(&Rules::REFERENCE, &upstream.frame())
        .position()
        .x;
    let column = (across.pixel() + Wrapping(MIDDLE)).0 / SCREEN;
    let limited = |read: Px| upstream.screens.limit_down(&went, column, read);
    edge(went.reached) != edge(went.stopped) && limited(went.reached) != limited(went.stopped)
}

/// A move down that the aim stopped short of where it reached (spec `FollowingHer`).
struct DownMove {
    /// Where the aim stopped it.
    stopped: Px,
    /// Where the move took it before the aim stopped it.
    reached: Px,
    /// The pixel part of its speed.
    speed: Px,
}

/// The camera's move down on the frame, if it goes down and its aim stops it.
fn down_past_aim(camera: Fixed, seen: Fixed, upstream: &Upstream) -> Option<DownMove> {
    let samus = &upstream.samus;
    let height = if samus.rising {
        upstream.heights.rising
    } else {
        upstream.heights.level
    };
    let her = samus.position.y;
    let aim = her.pixel() - height;
    let at = camera.pixel();
    let travel = if below(her.pixel().0, seen.pixel().0) {
        seen - her
    } else {
        her - seen
    };
    let speed = travel + Rules::REFERENCE.extra;
    let reached = (camera + speed).pixel();
    let goes_down = aim != at && !below(aim.0, at.0);
    (goes_down && below(aim.0, reached.0)).then_some(DownMove {
        stopped: aim,
        reached,
        speed: speed.pixel(),
    })
}

impl Screens {
    /// The screen `column` across and `row` down, in the run as one strip (spec `ScreensRunOn`,
    /// `BelowTheRoom`): past its end, closed.
    fn kind(&self, column: u16, row: u16) -> ScreenKind {
        usize::from(row)
            .checked_mul(self.width)
            .and_then(|start| start.checked_add(usize::from(column)))
            .and_then(|screen| self.run.get(screen))
            .copied()
            .unwrap_or(ScreenKind::Closed)
    }

    /// Where the room's bottom and closed screens stop the move down, reading the screen in
    /// `column` that holds `read` and the one under it (spec `ClosedScreens`, `RoomEdges`,
    /// `ShortScreens`).
    fn limit_down(&self, went: &DownMove, column: u16, read: Px) -> Px {
        let row = read.0 / SCREEN;
        let rows = match self.kind(column, row) {
            ScreenKind::Short => Wrapping(0),
            ScreenKind::Open | ScreenKind::Closed => Rules::REFERENCE.rows_not_short,
        };
        let next_closed = matches!(self.kind(column, row.wrapping_add(1)), ScreenKind::Closed);
        let back = went.reached - went.speed - Rules::REFERENCE.push;
        let stop = |at: Px| if below(at.0, back.0) { back } else { at };
        let height = u16::try_from(self.height.saturating_sub(1)).unwrap_or(u16::MAX);
        let bottom = Wrapping(height.wrapping_mul(SCREEN)) + rows;
        let c = went.stopped;
        if c > bottom {
            stop(bottom)
        } else if next_closed && c > edge(c) + rows {
            stop(edge(c) + rows)
        } else {
            c
        }
    }
}

/// The near edge of the screen holding `p`.
const fn edge(p: Px) -> Px {
    Wrapping(p.0.wrapping_sub(p.0 % SCREEN))
}

/// A screen's size on each axis.
const SCREEN: u16 = 256;

/// Half the view: its middle is this far from its near edge.
const MIDDLE: u16 = 128;

#[cfg(test)]
mod tests {
    use super::{CAMERA, CameraDomain, Domain, SEEN, X, Y};

    #[test]
    fn the_field_sets_name_the_declared_fields_at_their_places() {
        let named = [(X, "position.x"), (Y, "position.y"), (SEEN, "seen")];
        for (set, name) in named {
            let place = CameraDomain::FIELDS
                .iter()
                .position(|field| field.name == name);
            assert_eq!(place.map(super::FieldSet::single), Some(set), "{name}");
        }
        assert_eq!(CAMERA, X.union(Y));
    }
}
