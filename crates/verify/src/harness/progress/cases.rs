//! The harness cases (MODEL §1 decision 13): recorded frames whose report or room carries the
//! game's own content, which the spec's vectors can't hold.
//!
//! Each case is the decode notes' ("The harness cases"): a report case is the frame from record
//! N-1 to record N, whose report must be heard and carry the content the case names, and whose
//! progress after must equal record N's; a room case is the state a room holds at a record,
//! which must be the case's and the model's choice. A case whose report the harness can only
//! stand in for matches by construction, so it is marked stood in and not counted, and fails
//! once the harness hears it, so that it is credited. The constructed vectors carry the same
//! rules over invented content.

use core::num::Wrapping;

use oracle_record::FrameRecord;
use run_progress::{Pickup, Report, Supply};
use types::{Beam, DoorId, Item, LocationId};

use super::{Held, ProgressDomain, Source, Standing};
use crate::harness::Domain;
use crate::source;

/// What a report case's frame must hear.
#[derive(Clone, Copy)]
pub enum Heard {
    /// A report heard from the frame's hits, record N-1 and the ROM.
    Report(Report),
    /// The items the room's state places, from the ROM, when its PLMs spawn.
    Placed(u16),
}

/// Whether a report case counts.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Counted {
    /// Its report is heard, so its frame checks the model.
    Heard,
    /// The harness stands its report in from what record N changed: it matches by construction,
    /// so it doesn't count.
    StoodIn,
}

/// A recorded frame whose report carries the room's content.
pub struct ReportCase {
    /// The decode notes' name.
    pub id: &'static str,
    /// The row.
    pub row: &'static str,
    /// The record the frame ends at (N).
    pub record: u64,
    /// What the frame must hear.
    heard: Heard,
    /// Whether the case counts.
    pub counted: Counted,
}

/// How a room case's state was chosen.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Chosen {
    /// By a load before the row: the state the row's first record holds.
    BeforeTheRow,
    /// By the load in the frame that ends at the record.
    InTheFrame,
}

/// A recorded room whose state its list and the progress choose.
pub struct RoomCase {
    /// The decode notes' name.
    pub id: &'static str,
    /// The row.
    pub row: &'static str,
    /// The record that holds the state.
    pub record: u64,
    /// The room's header, in bank $8F.
    room: u16,
    /// The state, by its place in the room's list (the default last).
    state: usize,
    /// How the state was chosen.
    pub chosen: Chosen,
}

/// `pickup` taken at location `location`.
const fn taken(pickup: Pickup, location: u16) -> Heard {
    Heard::Report(Report::Taken {
        pickup,
        location: Some(LocationId(location)),
    })
}

/// A tank of `supply` that gives `amount`.
const fn tank(supply: Supply, amount: u16) -> Pickup {
    Pickup::Tank {
        supply,
        amount: Wrapping(amount),
    }
}

/// The door `door` opened.
const fn opened(door: u16) -> Heard {
    Heard::Report(Report::DoorOpened(Some(DoorId(door))))
}

/// The decode notes' fifteen report cases.
pub const REPORTS: [ReportCase; 15] = [
    ReportCase {
        id: "R-morph-ball",
        row: "item-morph-ball",
        record: 817,
        heard: taken(Pickup::Item(Item::MorphBall), 26),
        counted: Counted::Heard,
    },
    ReportCase {
        id: "R-bombs",
        row: "samus-bomb-jump-vertical",
        record: 55,
        heard: taken(Pickup::Item(Item::Bombs), 7),
        counted: Counted::Heard,
    },
    ReportCase {
        id: "R-missile-tank",
        row: "samus-square-slope-reserve",
        record: 37,
        heard: taken(tank(Supply::Missiles, 5), 63),
        counted: Counted::Heard,
    },
    ReportCase {
        id: "R-door-a",
        row: "chain-0057-9FBA",
        record: 280,
        heard: opened(51),
        counted: Counted::Heard,
    },
    ReportCase {
        id: "R-door-b",
        row: "chain-0097-ACF0",
        record: 519,
        heard: opened(85),
        counted: Counted::StoodIn,
    },
    ReportCase {
        id: "R-placed-two",
        row: "area-norfair",
        record: 200,
        heard: Heard::Placed(2),
        counted: Counted::Heard,
    },
    ReportCase {
        id: "R-placed-a",
        row: "chain-0097-ACF0",
        record: 757,
        heard: Heard::Placed(1),
        counted: Counted::Heard,
    },
    ReportCase {
        id: "R-placed-b",
        row: "chain-0106-AD5E",
        record: 567,
        heard: Heard::Placed(1),
        counted: Counted::Heard,
    },
    ReportCase {
        id: "R-plasma-beam",
        row: "item-plasma-beam",
        record: 146,
        heard: taken(Pickup::Beam(Beam::Plasma), 143),
        counted: Counted::Heard,
    },
    ReportCase {
        id: "R-ice-beam",
        row: "chain-0163-A890",
        record: 28,
        heard: taken(Pickup::Beam(Beam::Ice), 50),
        counted: Counted::Heard,
    },
    ReportCase {
        id: "R-charge-beam",
        row: "item-charge-beam",
        record: 902,
        heard: taken(Pickup::Beam(Beam::Charge), 23),
        counted: Counted::Heard,
    },
    ReportCase {
        id: "R-wave-beam",
        row: "item-wave-beam",
        record: 37,
        heard: taken(Pickup::Beam(Beam::Wave), 68),
        counted: Counted::Heard,
    },
    ReportCase {
        id: "R-energy-tank",
        row: "item-energy-tank",
        record: 240,
        heard: taken(tank(Supply::Energy, 100), 8),
        counted: Counted::Heard,
    },
    ReportCase {
        id: "R-reserve-tank",
        row: "item-reserve-tank",
        record: 1212,
        heard: taken(tank(Supply::Reserve, 100), 17),
        counted: Counted::Heard,
    },
    ReportCase {
        id: "R-power-bomb-tank",
        row: "chain-0043-9AD9",
        record: 417,
        heard: taken(tank(Supply::PowerBombs, 5), 13),
        counted: Counted::Heard,
    },
];

/// The decode notes' seven room cases.
pub const ROOMS: [RoomCase; 7] = [
    RoomCase {
        id: "R-landing-power-bombs",
        row: "samus-mockball",
        record: 0,
        room: 0x91F8,
        state: 1,
        chosen: Chosen::BeforeTheRow,
    },
    RoomCase {
        id: "R-landing-default",
        row: "chain-0012-91F8",
        record: 0,
        room: 0x91F8,
        state: 3,
        chosen: Chosen::BeforeTheRow,
    },
    RoomCase {
        id: "R-morph-room-default",
        row: "item-morph-ball",
        record: 0,
        room: 0x9E9F,
        state: 1,
        chosen: Chosen::BeforeTheRow,
    },
    RoomCase {
        id: "R-no-morph-ball",
        row: "chain-0014-96BA",
        record: 267,
        room: 0x975C,
        state: 1,
        chosen: Chosen::InTheFrame,
    },
    RoomCase {
        id: "R-ceres-alive",
        row: "samus-ceres-push",
        record: 0,
        room: 0xE0B5,
        state: 1,
        chosen: Chosen::BeforeTheRow,
    },
    RoomCase {
        id: "R-torizo-alive",
        row: "samus-bomb-jump-vertical",
        record: 0,
        room: 0x9804,
        state: 2,
        chosen: Chosen::BeforeTheRow,
    },
    RoomCase {
        id: "R-tunnel-whole",
        row: "area-maridia",
        record: 0,
        room: 0xCEFB,
        state: 1,
        chosen: Chosen::BeforeTheRow,
    },
];

/// Why a case fails: each difference, or why its frame couldn't run.
type Failures = Vec<String>;

impl ProgressDomain {
    /// `case` over the frame from `from` (record N-1) to `next` (record N): its report made with
    /// the case's content, heard (or still stood in, for a case marked so), and the progress
    /// after equal to `next`'s on every field the frame compares.
    ///
    /// # Errors
    ///
    /// The report not heard, or heard with other content; each field that differs; or why the
    /// frame couldn't run.
    pub fn report_case(
        &self,
        case: &ReportCase,
        from: &FrameRecord,
        next: &FrameRecord,
    ) -> Result<(), Failures> {
        let defect = |skip: super::Skip| vec![skip.to_string()];
        let (mut held, ()) = self
            .decode(from, crate::harness::Interval::ending_at(next))
            .map_err(defect)?;
        let mut upstream = self.world(from, next).map_err(defect)?;
        let mut failures = Vec::new();
        match case.heard {
            Heard::Report(report) => {
                let source = upstream
                    .reports
                    .iter()
                    .find(|&&(made, _)| made == report)
                    .map(|&(_, source)| source);
                match (source, case.counted) {
                    (Some(Source::Heard), Counted::Heard)
                    | (Some(Source::Copied), Counted::StoodIn) => {}
                    (Some(Source::Copied), Counted::Heard) => {
                        failures.push(format!("{report:?} is stood in, not heard"));
                    }
                    (Some(Source::Heard), Counted::StoodIn) => {
                        failures.push(format!("{report:?} is heard now: count the case"));
                    }
                    (None, _) => failures.push(format!(
                        "{report:?} isn't made; the frame makes {:?}",
                        upstream.reports
                    )),
                }
            }
            Heard::Placed(count) => {
                if upstream.placed != Some(Wrapping(count)) {
                    failures.push(format!(
                        "items placed: {:?}, the case {count}",
                        upstream.placed
                    ));
                }
            }
        }
        self.step(&mut held, &(), &mut upstream);
        failures.extend(self.differences(&held, from, next).map_err(defect)?);
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }

    /// The fields the frame from `from` to `next` compares on which `held` differs from `next`.
    fn differences(
        &self,
        held: &Held,
        from: &FrameRecord,
        next: &FrameRecord,
    ) -> Result<Vec<String>, super::Skip> {
        let masked = self
            .masks(from, next, &())?
            .iter()
            .fold(crate::harness::FieldSet::EMPTY, |set, mask| {
                set.union(mask.fields)
            });
        let (model, oracle) = (self.fields(held), self.expected(next)?);
        Ok(Self::FIELDS
            .iter()
            .enumerate()
            .filter(|&(place, field)| !masked.contains(place) && !(field.same)(&model, &oracle))
            .map(|(_, field)| {
                format!(
                    "{}: model {}, record {}",
                    field.name,
                    (field.show)(&model),
                    (field.show)(&oracle)
                )
            })
            .collect())
    }

    /// `case`, chosen in the frame from `from` to `next`: a room loads there, it is the case's
    /// room, and the model chooses the case's state, which `next` holds.
    ///
    /// # Errors
    ///
    /// No load, another room, or another state; or why the frame couldn't run.
    pub fn loaded_case(
        &self,
        case: &RoomCase,
        from: &FrameRecord,
        next: &FrameRecord,
    ) -> Result<(), Failures> {
        let defect = |skip: super::Skip| vec![skip.to_string()];
        let (mut held, ()) = self
            .decode(from, crate::harness::Interval::ending_at(next))
            .map_err(defect)?;
        let mut upstream = self.world(from, next).map_err(defect)?;
        if upstream.load.is_none() {
            return Err(vec!["no room loads in the frame".to_owned()]);
        }
        let mut failures = room_is(case, next);
        self.step(&mut held, &(), &mut upstream);
        let recorded = self.expected(next).map_err(defect)?.room_state;
        for (whose, state) in [("model", held.room_state), ("record", recorded)] {
            if state != Some(case.state) {
                failures.push(format!("{whose} state {state:?}, the case {}", case.state));
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

/// `case`, chosen before its row: `record` is in the case's room and holds the case's state,
/// which the model chooses, as `standing` gives them.
///
/// # Errors
///
/// Another room, or another state held or chosen.
pub fn standing_case(
    case: &RoomCase,
    record: &FrameRecord,
    standing: Standing,
) -> Result<(), Failures> {
    let mut failures = room_is(case, record);
    for (whose, state) in [("held", standing.held), ("chosen", standing.chosen)] {
        if state != case.state {
            failures.push(format!("{whose} state {state}, the case {}", case.state));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}

/// Whether `record` is in `case`'s room: none, or the difference.
fn room_is(case: &RoomCase, record: &FrameRecord) -> Failures {
    match source::ROOM_PTR.read(&record.wram) {
        Ok(room) if room == case.room => Vec::new(),
        Ok(room) => vec![format!("room {room:#06X}, the case {:#06X}", case.room)],
        Err(error) => vec![format!("room: {error:?}")],
    }
}
