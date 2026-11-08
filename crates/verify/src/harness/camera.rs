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
    BehaviourId, CATALOGUE, Camera, Frame, LeadSetting, Room, Rules, Samus as Her, ScreenKind,
};
use clean_types::{Facing, Fixed, Pair, Parts, Px, fixed};
use game::camera_world;
use game::model::num::Fixed as Exact;
use game::state::room::{ScrollArea, Scrollers, StaleScroll};
use oracle_record::FrameRecord;

use super::samus_motion::{Ps, SamusMotion};
use super::{Domain, Field, FieldSet, Interval, Mask, Uncompared};
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
/// The equipment bits of the two suits, whose pick-ups place her ($91:D4E4, $91:D5BA).
const SUITS: u16 = 0x0021;

/// The rooms whose own code moves the camera, or where it last saw her, during the frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RoomCode {
    /// The Landing Site: the gunship moves her while she enters, leaves or lands in it
    /// ($A2:A925, $A2:A9BD, $A2:AC1B).
    LandingSite,
    /// The Dachora room's main code scrolls the view right ($8F:C1E6): x.
    Dachora,
    /// Crocomire's room: its collision moves her ($A4:8C95) and its main code shakes the view
    /// across ($8F:E8CD).
    Crocomire,
    /// Kraid's room: his collision moves her ($A7:B0F3) and he keeps her x on the first screen
    /// ($A7:C9EE).
    Kraid,
    /// Ridley's room in Norfair: his feet's instructions set where the camera saw her to her
    /// position ($A6:E501), so it doesn't follow her. Ceres Ridley has no feet.
    Ridley,
    /// Mother Brain's room: she moves her towards the ceiling, the floor and the wall
    /// ($A9:BBFD, $A9:BC3F), and the Shitroid waits for the view ($A9:EFE6).
    MotherBrain,
}

impl RoomCode {
    /// The room by its header pointer.
    const fn of(header: u16) -> Option<Self> {
        Some(match header {
            0x91F8 => Self::LandingSite,
            0x9CB3 => Self::Dachora,
            0xA98D => Self::Crocomire,
            0xA59F => Self::Kraid,
            0xB32E => Self::Ridley,
            0xDD58 => Self::MotherBrain,
            _ => return None,
        })
    }

    /// The mask's reason and the fields it takes, on a frame where the room's writer may act:
    /// the gunship only while it holds her (her state handler isn't the normal one).
    const fn mask(self) -> (&'static str, FieldSet) {
        match self {
            Self::LandingSite => (
                "another domain: the gunship moves her while it holds her",
                CAMERA,
            ),
            Self::Dachora => ("the room's code scrolls the view right after it follows", X),
            Self::Crocomire => (
                "another domain: Crocomire moves her, and his room shakes the view",
                CAMERA,
            ),
            Self::Kraid => ("another domain: Kraid moves her and holds her x", CAMERA),
            Self::Ridley => ("another domain: Ridley's feet set where it saw her", CAMERA),
            Self::MotherBrain => (
                "another domain: Mother Brain moves her, and the Shitroid waits for the view",
                CAMERA,
            ),
        }
    }
}

/// The declared fields by their places in [`CameraDomain`]'s `FIELDS`.
const X: FieldSet = FieldSet::single(0);
const Y: FieldSet = FieldSet::single(1);
const CAMERA: FieldSet = X.union(Y);
const SEEN: FieldSet = FieldSet::single(2);

/// Samus's normal new state handlers: the ones that run her motion ($90:E725, Ceres's, the
/// elevator's).
const fn normal(handler: u16) -> bool {
    matches!(handler, 0xE725 | 0xE8AA | 0xE8EC)
}

/// Why a frame isn't one the camera is compared on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Skip {
    /// A source couldn't be read, or holds a code the model has no value for: a harness defect.
    Unreadable(DecodeError),
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

/// The room's screens as the scroll read them in `record`: the scroll table's first `w × h`
/// bytes, then what earlier rooms left in it.
fn screens(record: &FrameRecord) -> Result<Room, Skip> {
    let width = usize::from(word(record, ROOM_WIDTH_SCROLLS)?);
    let height = usize::from(word(record, ROOM_HEIGHT_SCROLLS)?);
    let table = source::SCROLLS.read(&record.wram)?;
    let (areas, stale) =
        table
            .split_at_checked(width.saturating_mul(height))
            .ok_or(DecodeError::WramTooShort {
                source: source::SCROLLS.name,
            })?;
    // Only red and blue are ever tested: any other byte in the room reads as green.
    let areas: Vec<ScrollArea> = areas
        .iter()
        .map(|&area| StaleScroll::from(area).area())
        .collect();
    let stale: Vec<StaleScroll> = stale.iter().copied().map(StaleScroll::from).collect();
    let scrollers = Scrollers {
        up: word(record, source::UP_SCROLLER)?,
        down: word(record, source::DOWN_SCROLLER)?,
    };
    Ok(camera_world::room(&areas, &stale, width, scrollers))
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
        room: screens(next)?,
        lead: lead(next)?,
        time_stopped: word(next, TIME_IS_FROZEN)? != 0,
    })
}

/// The camera's inputs as the scroll read them, from record N+1 and samus-motion's model.
pub struct Upstream {
    samus: Her,
    room: Room,
    lead: LeadSetting,
    time_stopped: bool,
}

/// What record N gives besides the State: where samus-motion's model placed the y the camera
/// follows her from, if it ran.
#[derive(Clone, Copy, Debug)]
pub struct Placement {
    /// Whether her motion ran in the frame, so `camera_from` is the model's.
    ran: bool,
    /// `camera_from`, where it isn't her start y.
    placed: Option<Fixed>,
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
            x: ps(camera.position.x),
            y: ps(camera.position.y),
            seen: (ps(camera.seen.x), ps(camera.seen.y)),
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

/// The live enemy slots' words at `field` of the main table and of the extra one.
fn slots(record: &FrameRecord) -> Result<Vec<(u16, u16)>, Skip> {
    (0..ENEMY_SLOT_COUNT)
        .map(|slot| {
            let base = slot.wrapping_mul(ENEMY_SLOT_SIZE);
            Ok((
                word(record, wram("ENEMY_ID", ENEMY_SLOTS.wrapping_add(base)))?,
                word(
                    record,
                    wram(
                        "ENEMY_RAM_7E8020",
                        ENEMY_EXTRA.wrapping_add(MAW_GRABBING).wrapping_add(base),
                    ),
                )?,
            ))
        })
        .collect()
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

    fn behaviours(&self) -> Vec<String> {
        CATALOGUE
            .iter()
            .map(|entry| format!("{:?}", entry.id))
            .collect()
    }

    fn decode(&self, from: &FrameRecord, interval: Interval) -> Result<(Camera, Placement), Skip> {
        gameplay(from)?;
        let camera = Camera {
            position: position(from)?,
            seen: seen(from)?,
        };
        // Where samus-motion's harness can't decode the frame (a wait site it doesn't know),
        // her motion is taken as not running, as on its own skips: the camera's own words
        // decoded, and a pose change there masks y.
        let placement = self.motion.placement(from, interval).map_or(
            Placement {
                ran: false,
                placed: None,
            },
            |placed| Placement { ran: true, placed },
        );
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
        if slots(from)?
            .iter()
            .any(|&(species, grabbing)| species == YAPPING_MAW && grabbing != 0)
        {
            mask(
                "another domain: a yapping maw holds her and caps the scroll's speed",
                CAMERA,
            );
        }
        let handlers = [
            word(from, SAMUS_NEW_STATE_HANDLER)?,
            word(next, SAMUS_NEW_STATE_HANDLER)?,
        ];
        if let Some(room) = RoomCode::of(word(from, source::ROOM_PTR)?) {
            let holds = handlers.iter().any(|&handler| !normal(handler));
            if room != RoomCode::LandingSite || holds {
                let (reason, fields) = room.mask();
                mask(reason, fields);
            }
        }
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
        if !placement.ran && pose(from)? != pose(next)? {
            mask(
                "camera_from: her motion doesn't run in the frame, so where her pose change placed her is samus-motion's to come",
                Y,
            );
        }
        Ok(masks)
    }

    fn step(&self, camera: &mut Camera, placement: &Placement, upstream: &mut Upstream) {
        // Remembering: where her motion's step placed her, else the y it last saw her at.
        upstream.samus.camera_from = placement.placed.unwrap_or(camera.seen.y);
        let frame = Frame {
            samus: upstream.samus,
            room: &upstream.room,
            lead: upstream.lead,
            time_stopped: upstream.time_stopped,
        };
        *camera = camera.step(&Rules::REFERENCE, &frame);
    }

    fn exercised(&self, camera: &Camera, placement: &Placement, upstream: &Upstream) -> Vec<usize> {
        exercised(camera, placement, upstream)
    }

    fn fields(&self, camera: &Camera) -> Declared {
        Declared::from(camera)
    }

    fn expected(&self, next: &FrameRecord) -> Result<Declared, Skip> {
        Ok(Declared::from(&Camera {
            position: position(next)?,
            seen: seen(next)?,
        }))
    }

    fn uncompared(&self, skip: &Skip) -> Uncompared {
        match skip {
            Skip::Unreadable(_) => Uncompared::Defect(skip.to_string()),
            Skip::GameState(_) | Skip::Frames(_) | Skip::RoomChange => {
                Uncompared::Skip(skip.to_string())
            }
        }
    }
}

/// The catalogue entries a frame's situation brings into play, from the camera as record N has
/// it and the frame's inputs: on each axis she moved or it settles, and the room's screens and
/// edges it is at.
fn exercised(camera: &Camera, placement: &Placement, upstream: &Upstream) -> Vec<usize> {
    use BehaviourId as B;
    let samus = &upstream.samus;
    let room = &upstream.room;
    let seen = Pair {
        y: placement
            .placed
            .map_or(camera.seen.y, |placed| placed.with_sub(camera.seen.y.sub())),
        ..camera.seen
    };
    let moved = |her: Fixed, seen: Fixed| her.pixel() != seen.pixel();
    let creeps = |her: Fixed, seen: Fixed| !moved(her, seen) && her.sub() < seen.sub();
    let (across, down) = (
        moved(samus.position.x, seen.x),
        moved(samus.position.y, seen.y),
    );
    let kinds = || room.screens.iter().flatten();
    let last = |screens: usize| -> Px {
        Wrapping(u16::try_from(screens.saturating_sub(1)).unwrap_or(u16::MAX) << 8)
    };
    let (w, h) = (room.screens.first().map_or(0, Vec::len), room.screens.len());
    let (x, y) = (camera.position.x.pixel(), camera.position.y.pixel());
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
            kinds().any(|&kind| kind == ScreenKind::Closed),
        ),
        (
            B::RoomEdges,
            x == Wrapping(0) || y == Wrapping(0) || x >= last(w) || in_last_row,
        ),
        (
            B::ShortScreens,
            kinds().any(|&kind| kind == ScreenKind::Short),
        ),
        (B::Settling, !across || !down),
        (B::TimeStopped, upstream.time_stopped && (!across || !down)),
        (B::DownwardLooksAhead, down),
        // Only from the room's last column, with that screen closed, does the screen after it
        // (the next row's first) decide anything.
        (
            B::ScreensRunOn,
            x >= last(w) && edge_closed(room, w, y + Wrapping(MIDDLE)),
        ),
        (
            B::BelowTheRoom,
            down && in_last_row && !room.below.is_empty(),
        ),
    ]
    .into_iter()
    .filter(|&(_, exercised)| exercised)
    .filter_map(|(id, _)| CATALOGUE.iter().position(|entry| entry.id == id))
    .collect()
}

/// Half the view: its middle is this far from its near edge.
const MIDDLE: u16 = 128;

/// Whether the room's last screen in the row holding `y` is closed.
fn edge_closed(room: &Room, w: usize, y: Px) -> bool {
    let [row, _] = y.0.to_be_bytes();
    room.screens
        .get(usize::from(row))
        .and_then(|screens| screens.get(w.saturating_sub(1)))
        .is_some_and(|&kind| kind == ScreenKind::Closed)
}

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
