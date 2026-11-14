//! The harness cases (MODEL §1 decision 13): recorded frames whose rule needs the room's own
//! content.
//!
//! A sloped floor's shape and walk factor, or a reacting cell, is content the spec's vectors
//! can't carry. Each case runs over the record's level and the ROM's slopes, as the harness's
//! frames do, and compares what the decode notes' table ("Room content: harness cases") names
//! with record N+1. The constructed vectors carry the same rules over invented rooms.

use data::assets::samus::map;
use game::samus_movement_bridge::step::{Ran, settle};
use game::state::samus::PoseId;
use oracle_record::FrameRecord;
use samus_movement::WallTest;
use types::{DirectionalTravels, Heading, Way};

use super::{Hers, Movement, Quantity};
use crate::harness::{Domain, Interval};
use crate::source;

/// A cell her moves or looks reached, by its top-left corner in px, and the way.
type Reached = ([u16; 2], Way);

/// What the near-wall test and her moves met, as the rules give them over the record's room.
pub struct Touches {
    /// The near-wall test's outcome.
    wall_test: WallTest,
    /// The cells reached, in order.
    reached: &'static [Reached],
}

/// A recorded frame the harness checks by name.
pub struct Case {
    /// The decode notes' name.
    pub id: &'static str,
    /// The row.
    pub row: &'static str,
    /// The record the checked frame ends at (N+1).
    pub checked: u64,
    /// The frames before the checked one that lead in to it: each decoded from its own record,
    /// her motion carried from the frame before.
    lead_in: u64,
    /// The declared fields compared.
    compared: &'static [Quantity],
    /// Whether her pose after the frame is compared with record N+1's.
    pose: bool,
    /// The near-wall test and the cells reached, where the case compares them.
    touches: Option<Touches>,
    /// Compare all four signed receipts, with a zero-travel null contrast.
    pub distances: bool,
}

use Quantity as Q;

/// W15's cells, in the order her moves and looks reached them.
const W15_REACHED: [Reached; 10] = [
    ([1120, 640], Way::Move(Heading::Left)),
    ([1120, 656], Way::Move(Heading::Left)),
    ([1104, 640], Way::Look),
    ([1104, 656], Way::Look),
    ([1120, 656], Way::Move(Heading::Down)),
    ([1136, 656], Way::Move(Heading::Down)),
    ([1120, 624], Way::Look),
    ([1136, 624], Way::Look),
    ([1120, 672], Way::Look),
    ([1136, 672], Way::Look),
];

/// Room-content cases and the decode notes' seven directional-travel witnesses.
pub const CASES: [Case; 13] = [
    Case {
        id: "S1",
        row: "samus-input-poses",
        checked: 512,
        lead_in: 0,
        compared: &[Q::X, Q::Y, Q::BaseSpeed, Q::ExtraRun],
        pose: false,
        touches: None,
        distances: false,
    },
    Case {
        id: "S2",
        row: "chain-0012-91F8",
        checked: 717,
        lead_in: 0,
        compared: &[Q::X, Q::Y, Q::BaseSpeed, Q::VerticalSpeed],
        pose: false,
        touches: None,
        distances: false,
    },
    Case {
        id: "SL1",
        row: "samus-mockball",
        checked: 92,
        lead_in: 0,
        compared: &[Q::X, Q::Y, Q::BaseSpeed, Q::VerticalSpeed, Q::ExtraRun],
        pose: false,
        touches: None,
        distances: false,
    },
    Case {
        id: "SL2",
        row: "samus-mockball",
        checked: 93,
        lead_in: 0,
        compared: &[Q::X, Q::Y, Q::BaseSpeed, Q::VerticalSpeed, Q::ExtraRun],
        pose: false,
        touches: None,
        distances: false,
    },
    Case {
        id: "SL3",
        row: "chain-0220-D017",
        checked: 44,
        lead_in: 0,
        compared: &[Q::X, Q::Y, Q::BaseSpeed, Q::VerticalSpeed, Q::ExtraRun],
        pose: true,
        touches: None,
        distances: false,
    },
    Case {
        id: "W15",
        row: "item-morph-ball",
        checked: 817,
        lead_in: 2,
        compared: &[Q::X, Q::Y, Q::VerticalSpeed],
        pose: false,
        touches: Some(Touches {
            wall_test: WallTest::Neither,
            reached: &W15_REACHED,
        }),
        distances: false,
    },
    Case {
        id: "D-R2",
        row: "item-morph-ball",
        checked: 704,
        lead_in: 0,
        compared: &[],
        pose: false,
        touches: None,
        distances: true,
    },
    Case {
        id: "D-R61",
        row: "samus-grapple-release",
        checked: 2,
        lead_in: 0,
        compared: &[],
        pose: false,
        touches: None,
        distances: true,
    },
    Case {
        id: "D-R144",
        row: "area-norfair",
        checked: 2,
        lead_in: 0,
        compared: &[],
        pose: false,
        touches: None,
        distances: true,
    },
    Case {
        id: "D-W11",
        row: "item-morph-ball",
        checked: 683,
        lead_in: 0,
        compared: &[],
        pose: false,
        touches: None,
        distances: true,
    },
    Case {
        id: "D-W13",
        row: "item-morph-ball",
        checked: 1228,
        lead_in: 0,
        compared: &[],
        pose: false,
        touches: None,
        distances: true,
    },
    Case {
        id: "D-W20",
        row: "item-morph-ball",
        checked: 1230,
        lead_in: 0,
        compared: &[],
        pose: false,
        touches: None,
        distances: true,
    },
    Case {
        id: "D-inherited-cut",
        row: "chain-any-0003-DFD7",
        checked: 156,
        lead_in: 0,
        compared: &[],
        pose: false,
        touches: None,
        distances: true,
    },
];

/// Whether two ways are the same.
fn same_way(a: Way, b: Way) -> bool {
    match (a, b) {
        (Way::Look, Way::Look) => true,
        (Way::Move(a), Way::Move(b)) => a == b,
        (Way::Look, Way::Move(_)) | (Way::Move(_), Way::Look) => false,
    }
}

/// A reached cell as the case writes it.
fn shown(([x, y], way): Reached) -> String {
    let way = match way {
        Way::Move(Heading::Left) => "left",
        Way::Move(Heading::Right) => "right",
        Way::Move(Heading::Up) => "up",
        Way::Move(Heading::Down) => "down",
        Way::Look => "look",
    };
    format!("[{x}, {y}] {way}")
}

/// The same complete signed comparison for the model and the zero-output null.
fn travel_differences(model: &DirectionalTravels, oracle: &DirectionalTravels) -> Vec<String> {
    oracle
        .iter()
        .filter(|(direction, accepted)| model[*direction] != **accepted)
        .map(|(direction, _)| format!("travel {direction:?}: differs from record"))
        .collect()
}

impl Movement {
    /// `case` over its row's `records`: each frame from the first lead-in's start decoded from
    /// its own record (her pose, the pad, the room), her motion carried from the model's step
    /// before it, and the checked frame compared with record N+1 on what the case names.
    ///
    /// # Errors
    ///
    /// Each difference, naming the field with the model's value and the record's; or why a frame
    /// couldn't run (a record missing, a frame the domain doesn't run).
    pub fn case(&self, case: &Case, records: &[FrameRecord]) -> Result<(), Vec<String>> {
        let start = case.checked.saturating_sub(case.lead_in.saturating_add(1));
        let record = |index: u64| {
            records
                .iter()
                .find(|record| record.index == index)
                .ok_or_else(|| vec![format!("record {index} isn't in the row")])
        };
        let mut carried: Option<Hers> = None;
        for index in start..case.checked {
            let (from, next) = (record(index)?, record(index.saturating_add(1))?);
            let (mut hers, inputs) = self
                .decode(from, Interval::ending_at(next))
                .map_err(|skip| vec![format!("frame {index}: {skip}")])?;
            if let Some(before) = carried {
                hers.samus.motion = before.samus.motion;
                hers.samus.jump_held = before.samus.jump_held;
                hers.rising = before.rising;
            }
            let level = self.level(&inputs);
            let ran = self
                .ran(&mut hers.samus, &inputs, &level)
                .map_err(|skip| vec![format!("frame {index}: {skip}")])?;
            settle(&mut hers.samus, &ran);
            hers.rising = ran.outcome.rising;
            if index.saturating_add(1) == case.checked {
                return self.compare(case, &hers, &ran, next);
            }
            carried = Some(hers);
        }
        Err(vec!["no checked frame".to_owned()])
    }

    /// The checked frame's differences from record `next`.
    fn compare(
        &self,
        case: &Case,
        hers: &Hers,
        ran: &Ran,
        next: &FrameRecord,
    ) -> Result<(), Vec<String>> {
        let model = self.fields(hers);
        let oracle = self
            .expected(next)
            .map_err(|skip| vec![format!("record {}: {skip}", next.index)])?;
        let mut differences: Vec<String> = case
            .compared
            .iter()
            .filter_map(|quantity| Self::FIELDS.get(quantity.index()))
            .filter(|field| !(field.same)(&model, &oracle))
            .map(|field| {
                format!(
                    "{}: model {}, record {}",
                    field.name,
                    (field.show)(&model),
                    (field.show)(&oracle)
                )
            })
            .collect();
        if case.distances {
            let expected = crate::decode::directional_travel(&next.wram)
                .map_err(|error| vec![format!("record {}: {error:?}", next.index)])?;
            let zero = samus_movement::TravelledDistances::fresh().travels;
            differences.extend(
                travel_differences(&ran.outcome.travelled, &expected)
                    .into_iter()
                    .map(|difference| format!("record {}: {difference}", next.index)),
            );
            if travel_differences(&zero, &expected).is_empty() {
                differences.push("zero-travel null also passes this witness".to_owned());
            }
        }
        if case.pose {
            let pose = source::SAMUS_POSE
                .read(&next.wram)
                .map_err(|error| vec![format!("record {}: {error:?}", next.index)])?;
            if map::key_of(PoseId(pose)) != Some(ran.after.pose) {
                differences.push(format!("pose: record {pose:#04X}"));
            }
        }
        if let Some(touches) = &case.touches {
            if ran.outcome.wall_test != touches.wall_test {
                differences.push("the near-wall test's outcome".to_owned());
            }
            let reached: Vec<Reached> = ran
                .outcome
                .reached
                .iter()
                .map(|touch| {
                    (
                        [touch.cell.x.wrapping_mul(16), touch.cell.y.wrapping_mul(16)],
                        touch.way,
                    )
                })
                .collect();
            let same = reached.len() == touches.reached.len()
                && reached
                    .iter()
                    .zip(touches.reached)
                    .all(|(&(cell, way), &(want, wanted))| cell == want && same_way(way, wanted));
            if !same {
                let show = |cells: &[Reached]| {
                    cells
                        .iter()
                        .copied()
                        .map(shown)
                        .collect::<Vec<_>>()
                        .join("; ")
                };
                differences.push(format!(
                    "reached: model {}, case {}",
                    show(&reached),
                    show(touches.reached)
                ));
            }
        }
        if differences.is_empty() {
            Ok(())
        } else {
            Err(differences)
        }
    }
}
