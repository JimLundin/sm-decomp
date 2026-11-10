//! The harness cases (MODEL §1 decision 13): recorded frames whose room is the game's own,
//! which the spec's vectors can't hold.
//!
//! Each case is the decode notes' ("The harness cases"): the frame from record N to record N+1,
//! over the room's screens as the importer loads them. The importer's screens must be the ones
//! record N+1's scroll table holds, but for the screens the case names as changed before the
//! frame and, with preset scrolls, the run after them, which the last room left: those two come
//! from the record. The frame must be one the harness compares, with neither axis masked, and the
//! model over the importer's screens must match record N+1 on the camera's position. The
//! constructed vectors carry the same rules over invented rooms.

use clean_model_camera::Room;
use clean_types::Pair;
use data::level::{ScrollArea, names_scroll_table};
use data::rom::Lorom;
use data::room::{RoomKey, decode};
use game::camera_world;
use game::state::room::Scrollers;
use oracle_record::FrameRecord;

use super::{CAMERA, CameraDomain, Screens, Skip, Y, word};
use crate::harness::{Domain, FieldSet, Interval};
use crate::source;

/// Where a case's screens after the room's own come from.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Scrolls {
    /// The room's scroll table: the room's screens and the run after them, the importer's.
    Table,
    /// Preset scrolls, which fill the room's screens only; the frame reads nothing after them.
    Preset,
    /// Preset scrolls, and the frame reads the run after them, which the last room left: the
    /// record's.
    AfterLastRoom,
}

/// How the harness compares a case's frame.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Compared {
    /// On both axes.
    Both,
    /// With these axes under one of the harness note's masks, though the notes list the frame
    /// as compared: a finding for the observer. The model must still match on both axes, and the
    /// case fails once the mask lifts, so that it is counted.
    Masked(FieldSet),
}

/// A recorded frame over the game's own room.
pub struct Case {
    /// The decode notes' name.
    pub id: &'static str,
    /// The row.
    pub row: &'static str,
    /// The record the frame ends at (N+1).
    pub record: u64,
    /// The room's header, in bank $8F.
    room: u16,
    /// Where its screens after the room's come from.
    scrolls: Scrolls,
    /// The screens the room's scroll cells or its own code changed before the frame, which the
    /// record holds.
    changed: &'static [usize],
    /// How the harness compares it.
    pub compared: Compared,
}

/// The case `id` at `record` of `row`, in `room`, compared on both axes.
const fn case(
    id: &'static str,
    (row, record): (&'static str, u64),
    room: u16,
    scrolls: Scrolls,
    changed: &'static [usize],
) -> Case {
    Case {
        id,
        row,
        record,
        room,
        scrolls,
        changed,
        compared: Compared::Both,
    }
}

impl Case {
    /// The case with `axes` under the harness note's masks.
    const fn masked(self, axes: FieldSet) -> Self {
        Self {
            compared: Compared::Masked(axes),
            ..self
        }
    }
}

use Scrolls::{AfterLastRoom, Preset, Table};

/// The rows the cases come from.
const BRINSTAR: &str = "area-brinstar";
const MARIDIA: &str = "area-maridia";
const NORFAIR: &str = "area-norfair";
const R01: &str = "chain-0012-91F8";
const R02: &str = "chain-0013-92FD";
const R03: &str = "chain-0014-96BA";
const R04: &str = "chain-0057-9FBA";
const R05: &str = "chain-0059-A293";
const R06: &str = "chain-0097-ACF0";
const R08: &str = "chain-0106-AD5E";
const R11: &str = "chain-0232-D5A7";
const MORPH_BALL: &str = "item-morph-ball";

/// The decode notes' 35 cases. R24's y lies under the knockback's start, which places her y
/// where samus-motion's model doesn't; its rule is on x, which is compared.
pub const CASES: [Case; 35] = [
    case("R1", (BRINSTAR, 72), 0x9E9F, Table, &[]),
    case("R2", (BRINSTAR, 74), 0x9E9F, Table, &[]),
    case("R3", (BRINSTAR, 414), 0x9E9F, Table, &[]),
    case("R4", (MARIDIA, 6), 0xCEFB, Table, &[0, 1, 2]),
    case("R5", (MARIDIA, 7), 0xCEFB, Table, &[0, 1, 2]),
    case("R6", (MARIDIA, 208), 0xCF54, Table, &[]),
    case("R7", (MARIDIA, 419), 0xCF54, Table, &[]),
    case("R8", (NORFAIR, 17), 0xA9E5, Preset, &[]),
    case("R9", (NORFAIR, 18), 0xA9E5, Preset, &[]),
    case("R10", (NORFAIR, 55), 0xA9E5, AfterLastRoom, &[]),
    case("R11", (NORFAIR, 286), 0xAA41, Table, &[0, 2]),
    case("R12", (NORFAIR, 304), 0xAA41, Table, &[0, 2]),
    case("R13", (R01, 418), 0x91F8, Table, &[]),
    case("R14", (R01, 419), 0x91F8, Table, &[]),
    case("R15", (R01, 620), 0x91F8, Table, &[]),
    case("R16", (R01, 655), 0x91F8, Table, &[]),
    case("R17", (R01, 994), 0x91F8, Table, &[]),
    case("R18", (R01, 996), 0x91F8, Table, &[]),
    case("R19", (R02, 7), 0x92FD, Table, &[]),
    case("R20", (R03, 5), 0x96BA, Table, &[]),
    case("R21", (R03, 31), 0x96BA, Table, &[]),
    case("R22", (R03, 210), 0x96BA, Table, &[]),
    case("R23", (R04, 19), 0x9FBA, AfterLastRoom, &[]),
    case("R24", (R04, 297), 0x9FBA, AfterLastRoom, &[]).masked(Y),
    case("R25", (R04, 314), 0x9FBA, AfterLastRoom, &[]),
    case("R26", (R05, 1), 0xA293, Table, &[]),
    case("R27", (R05, 66), 0xA293, Table, &[]),
    case("R28", (R06, 662), 0xACF0, AfterLastRoom, &[]),
    case("R29", (R08, 656), 0xACB3, Table, &[]),
    case("R30", (R08, 714), 0xACB3, Table, &[]),
    case("R31", (R11, 475), 0xD5A7, Table, &[]),
    case("R32", (MORPH_BALL, 709), 0x9E9F, Table, &[]),
    case("R33", (MORPH_BALL, 1228), 0x9E9F, Table, &[]),
    case(
        "R34",
        ("samus-bomb-jump-vertical", 1030),
        0x9804,
        AfterLastRoom,
        &[],
    ),
    case("R35", ("samus-shinespark-left", 214), 0x91F8, Table, &[]),
];

/// Why a case fails: each difference, or why its frame couldn't run.
type Failures = Vec<String>;

/// The room as the importer loads it for a case: its screens, with what the case takes from the
/// record, its size in screens and its scrollers.
struct Imported {
    areas: Vec<ScrollArea>,
    size: Pair<usize>,
    scrollers: Scrollers,
}

/// The room as the importer loads it for `case` at `next`; each way it differs from the record
/// but where the case takes the record's goes into `failures`.
fn imported(
    case: &Case,
    rom: Lorom<'_>,
    next: &FrameRecord,
    failures: &mut Failures,
) -> Result<Imported, Failures> {
    let defect = |skip: Skip| vec![skip.to_string()];
    let held_room = word(next, source::ROOM_PTR).map_err(defect)?;
    if held_room != case.room {
        return Err(vec![format!(
            "room {held_room:#06X}, the case {:#06X}",
            case.room
        )]);
    }
    let header = decode(rom, RoomKey::from_room_pointer(case.room))
        .map_err(|error| vec![format!("the importer: {error:?}")])?;
    let held = word(next, source::ROOMDEFROOMSTATE_PTR).map_err(defect)?;
    let (_, state) = header
        .states()
        .find(|(_, state)| state.at.offset == held)
        .ok_or_else(|| vec![format!("the room has no state at {held:#06X}")])?;
    if names_scroll_table(state.scroll) != (case.scrolls == Table) {
        failures.push(format!("the state's scroll pointer is {:?}", state.scroll));
    }
    let size = [header.size.width, header.size.height].map(u16::from);
    let recorded_size = [
        word(next, super::ROOM_WIDTH_SCROLLS).map_err(defect)?,
        word(next, super::ROOM_HEIGHT_SCROLLS).map_err(defect)?,
    ];
    if size != recorded_size {
        failures.push(format!("size {size:?}, the record {recorded_size:?}"));
    }
    let scrollers = Scrollers {
        up: u16::from(header.scrollers.up),
        down: u16::from(header.scrollers.down),
    };
    let recorded_scrollers = Scrollers {
        up: word(next, source::UP_SCROLLER).map_err(defect)?,
        down: word(next, source::DOWN_SCROLLER).map_err(defect)?,
    };
    if scrollers != recorded_scrollers {
        failures.push(format!(
            "heights {scrollers:?}, the record {recorded_scrollers:?}"
        ));
    }
    let recorded: Vec<ScrollArea> = source::SCROLLS
        .read(&next.wram)
        .map_err(|error| defect(Skip::from(error)))?
        .iter()
        .copied()
        .map(ScrollArea::from)
        .collect();
    let mut areas = state.scroll_areas.clone();
    for (screen, area) in areas.iter_mut().enumerate() {
        let held = recorded.get(screen).copied();
        if case.changed.contains(&screen) {
            match held {
                Some(held) if held != *area => *area = held,
                _ => failures.push(format!(
                    "screen {screen} is listed as changed: the importer {area:?}, the record {held:?}"
                )),
            }
        } else if held != Some(*area) {
            failures.push(format!(
                "screen {screen}: the importer {area:?}, the record {held:?}"
            ));
        }
    }
    if let Some(&past) = case.changed.iter().find(|&&screen| screen >= areas.len()) {
        failures.push(format!(
            "screen {past} is listed as changed, past the table"
        ));
    }
    if case.scrolls == AfterLastRoom {
        areas.extend(recorded.get(areas.len()..).unwrap_or_default());
    }
    let [x, y] = size.map(usize::from);
    Ok(Imported {
        areas,
        size: Pair { x, y },
        scrollers,
    })
}

impl CameraDomain {
    /// `case` over the frame from `from` (record N) to `next` (record N+1): the importer's
    /// screens equal to the record's but where the case takes them from it, the frame compared on
    /// the axes the case says, and the model over the importer's screens equal to `next` on the
    /// camera's position.
    ///
    /// # Errors
    ///
    /// Each screen, size or height the importer and the record differ on, masks other than the
    /// case's, each field that differs; or why the frame couldn't run.
    pub fn case(
        &self,
        case: &Case,
        rom: Lorom<'_>,
        from: &FrameRecord,
        next: &FrameRecord,
    ) -> Result<(), Failures> {
        let defect = |skip: Skip| vec![skip.to_string()];
        let mut failures = Vec::new();
        let content = imported(case, rom, next, &mut failures)?;
        let (mut camera, placement) = self
            .decode(from, Interval::ending_at(next))
            .map_err(defect)?;
        let mut upstream = self.world(from, next).map_err(defect)?;
        let masks = self.masks(from, next, &placement).map_err(defect)?;
        let masked = masks
            .iter()
            .filter(|mask| mask.fields.overlaps(CAMERA))
            .fold(FieldSet::EMPTY, |axes, mask| axes.union(mask.fields));
        let expected = match case.compared {
            Compared::Both => FieldSet::EMPTY,
            Compared::Masked(axes) => axes,
        };
        if masked != expected {
            let reasons: Vec<_> = masks
                .iter()
                .filter(|mask| mask.fields.overlaps(CAMERA))
                .map(|mask| mask.reason)
                .collect();
            failures.push(format!(
                "masked {masked:?}, the case {expected:?}: {}",
                reasons.join("; ")
            ));
        }
        let screens = Screens {
            run: camera_world::run(&content.areas, &[]),
            size: content.size,
        };
        upstream.room = Room::new(
            screens.run.clone(),
            screens.size,
            camera_world::heights(content.scrollers),
        )
        .map_err(|error| defect(Skip::Room(error)))?;
        upstream.screens = screens;
        self.step(&mut camera, &placement, &mut upstream);
        let (model, oracle) = (self.fields(&camera), self.expected(next).map_err(defect)?);
        let differing = Self::FIELDS
            .iter()
            .enumerate()
            .filter(|&(place, field)| CAMERA.contains(place) && !(field.same)(&model, &oracle));
        for (_, field) in differing {
            failures.push(format!(
                "{}: model {}, record {}",
                field.name,
                (field.show)(&model),
                (field.show)(&oracle)
            ));
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}
