//! The harness cases (MODEL §1 decision 13): recorded frames the spec's vectors can't carry, since
//! they hold the game's own content, the HUD's pictures and her area's map.
//!
//! Each case is the decode notes' ("The harness cases"): a record pair the harness runs with the
//! importer's pictures and the area's map, as its frames run, compared with record N+1 on every
//! declared field. A case is named, so it can't hide: one the harness skips or masks fails, as a
//! harness defect. Each has a constructed twin in the vectors, `C-` and its name, that carries the
//! same rules over invented content.

use oracle_record::FrameRecord;

use super::HudDomain;
use crate::harness::{Domain, Interval, Uncompared};

/// A recorded frame the harness checks by name.
pub struct Case {
    /// The decode notes' name.
    pub id: &'static str,
    /// The row.
    pub row: &'static str,
    /// The record the checked frame ends at (N+1).
    pub checked: u64,
}

/// A case by its name, row and checked record.
const fn case(id: &'static str, row: &'static str, checked: u64) -> Case {
    Case { id, row, checked }
}

/// The decode notes' eighteen cases.
pub const CASES: [Case; 18] = [
    case("R1", "chain-0059-A293", 228),
    case("R2", "chain-0100-B07A", 134),
    case("R3", "chain-0224-D0B9", 13),
    case("R4", "samus-grapple-release", 11),
    case("R5", "area-brinstar", 1),
    case("R6", "area-maridia", 2),
    case("R7", "samus-grapple-release", 231),
    case("R8", "chain-0059-A293", 185),
    case("R9", "area-maridia", 393),
    case("R10", "chain-0097-ACF0", 662),
    case("R11", "area-maridia", 396),
    case("R12", "chain-0220-D017", 92),
    case("R13", "chain-0057-9FBA", 265),
    case("R14", "chain-0059-A293", 6),
    case("R15", "chain-0232-D5A7", 269),
    case("R16", "area-brinstar", 170),
    case("R17", "chain-0012-91F8", 55),
    case("R18", "samus-ceres-push", 2),
];

impl HudDomain {
    /// `case` over its row's `records`: the pair ending at its checked record, decoded and
    /// stepped as the harness's frames are, compared with record N+1 on every declared field.
    ///
    /// # Errors
    ///
    /// Each difference, naming the field with the model's value and the record's; or why the
    /// pair wasn't compared: a record missing, a skip, a defect or a mask.
    pub fn case(&self, case: &Case, records: &[FrameRecord]) -> Result<(), Vec<String>> {
        let record = |index: u64| {
            records
                .iter()
                .find(|record| record.index == index)
                .ok_or_else(|| vec![format!("record {index} isn't in the row")])
        };
        let next = record(case.checked)?;
        let from = record(case.checked.saturating_sub(1))?;
        let uncompared = |skip| {
            let why = match self.uncompared(&skip) {
                Uncompared::Skip(why) => format!("skipped: {why}"),
                Uncompared::Defect(why) => format!("a defect: {why}"),
            };
            vec![why]
        };
        let (mut state, pad) = self
            .decode(from, Interval::ending_at(next))
            .map_err(uncompared)?;
        let mut world = self.world(from, next).map_err(uncompared)?;
        let masks = self.masks(from, next, &pad).map_err(uncompared)?;
        if !masks.is_empty() {
            return Err(masks
                .iter()
                .map(|mask| format!("masked: {}", mask.reason))
                .collect());
        }
        let oracle = self.expected(next).map_err(uncompared)?;
        self.step(&mut state, &pad, &mut world);
        let model = self.fields(&state);
        let differences: Vec<String> = Self::FIELDS
            .iter()
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
        if differences.is_empty() {
            Ok(())
        } else {
            Err(differences)
        }
    }
}
