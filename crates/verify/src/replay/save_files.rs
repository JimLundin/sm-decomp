//! The battery witness's ordered cuts and independent live-save inputs.

use std::collections::BTreeMap;

use data::rom::Rom;
use game::exact::save_files::{BootProgress, Format, profile_from_bytes, progress_from_bytes};
use game::production::save_files::{self, Input};
use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules, ProductionSeed,
};
use save_files_model::{ChoiceStage, SaveCut, WriteStage};
use types::save_files::{
    BatteryImage, ExploredCells, MAP_BYTES, PlayerProfile, Progress, RememberedMaps, blank_maps,
};
use types::{Area, StationId};

use super::compare::Observation;
use super::readers::{
    ActualReader, AdmittedCase, ExpectedCut, ExpectedReader, InputReader, record,
};
use super::{Sources, data_failure, result::Outcome};
use crate::cache::Cache;
use crate::lock::{Lock, RowId};
use ::save_files as save_files_model;
use game::state::owned::OwnedState;

const POWER_ON: &str = "any-power-on";
const FLIGHT: &str = "any-fly-to-ceres";
const ARRIVAL: &str = "any-ceres-arrival";
const ESCAPE: &str = "chain-any-0011-DF45";
const LANDING: &str = "any-zebes-arrival";

#[derive(Clone, Copy)]
enum Request {
    Hold,
    Diagnostic(BootProgress),
    Boot,
    Menu,
    Choose,
    Start,
    Arrival,
    Finish,
    Escape,
}

use Request::{Arrival, Boot, Choose, Diagnostic, Escape, Finish, Hold, Menu, Start};

// Logical cut numbers are one-based positions in this contiguous schedule.
const CUTS: [(&str, u64, Request); 27] = [
    (POWER_ON, 262, Hold),
    (POWER_ON, 263, Diagnostic(BootProgress::Clearing(2347))),
    (POWER_ON, 264, Diagnostic(BootProgress::Testing(2463))),
    (POWER_ON, 265, Diagnostic(BootProgress::Testing(0))),
    (POWER_ON, 266, Diagnostic(BootProgress::Restoring(3880))),
    (POWER_ON, 267, Diagnostic(BootProgress::Restoring(1044))),
    (POWER_ON, 268, Diagnostic(BootProgress::ClearingFiles(151))),
    (POWER_ON, 269, Diagnostic(BootProgress::ClearingFiles(964))),
    (POWER_ON, 270, Diagnostic(BootProgress::ClearingFiles(1777))),
    (POWER_ON, 271, Boot),
    (POWER_ON, 396, Hold),
    (POWER_ON, 399, Hold),
    (POWER_ON, 400, Menu),
    (POWER_ON, 431, Hold),
    (POWER_ON, 432, Choose),
    (POWER_ON, 433, Start),
    (POWER_ON, 567, Hold),
    (FLIGHT, 802, Hold),
    (ARRIVAL, 0, Arrival),
    (ARRIVAL, 1, Finish),
    (ARRIVAL, 2, Hold),
    (ESCAPE, 239, Hold),
    (ESCAPE, 240, Escape),
    (ESCAPE, 241, Finish),
    (ESCAPE, 242, Hold),
    (ESCAPE, 2123, Hold),
    (LANDING, 0, Hold),
];

pub(super) const END: u64 = 27;

pub(super) struct Inputs {
    arrival: Capture,
    escape: Capture,
}

const AREAS: [Area; 7] = [
    Area::Crateria,
    Area::Brinstar,
    Area::Norfair,
    Area::WreckedShip,
    Area::Maridia,
    Area::Tourian,
    Area::Ceres,
];

pub(super) struct Capture {
    pub(super) profile: PlayerProfile,
    pub(super) progress: Progress,
    pub(super) maps: RememberedMaps,
    pub(super) live_map: ExploredCells,
    pub(super) area: Area,
    pub(super) station: StationId,
}

impl Capture {
    pub(super) fn from_record(record: &oracle_record::FrameRecord) -> Option<Self> {
        let read = |at: usize| {
            record
                .wram
                .get(at..at.checked_add(2)?)
                .and_then(|bytes| bytes.try_into().ok())
                .map(u16::from_le_bytes)
        };
        let area = *AREAS.get(usize::from(read(0x079f)?))?;
        let station = StationId(read(0x078b)?);
        let profile = profile_from_bytes(record.wram.get(0x09a2..0x0a02)?.try_into().ok()?);
        let progress = progress_from_bytes(
            record.wram.get(0xd820..0xd914)?.try_into().ok()?,
            read(0xd91a)?,
        );
        let mut maps = blank_maps();
        for (area, bytes) in AREAS
            .into_iter()
            .zip(record.wram.get(0xcd52..0xd452)?.as_chunks::<MAP_BYTES>().0)
        {
            maps[area].as_mut_slice().copy_from_slice(bytes);
        }
        let live_map = ExploredCells::new(record.wram.get(0x07f7..0x08f7)?.to_vec())?;
        Some(Self {
            profile,
            progress,
            maps,
            live_map,
            area,
            station,
        })
    }
}

impl Inputs {
    pub(super) fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        let (_, _, request) = cut
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| CUTS.get(index))
            .copied()
            .ok_or_else(|| data_failure(Some(cut), "cartridge input cut"))?;
        let simple = match request {
            Hold => Input::Hold,
            Diagnostic(progress) => Input::InspectBoot(progress),
            Boot => Input::Boot,
            Menu => Input::Menu,
            Choose => {
                let OwnedState::SaveFiles(state) = production.state() else {
                    return Err(data_failure(Some(cut), "cartridge selection scope"));
                };
                let slot = state
                    .menu_cursor
                    .ok_or_else(|| data_failure(Some(cut), "inspected menu cursor"))?;
                Input::Choose(slot, ChoiceStage::SelectionWritten)
            }
            Start => Input::FinishChoice,
            Finish => Input::FinishWrite,
            Arrival | Escape => return self.write(production, matches!(request, Arrival), cut),
        };
        advance(production, simple, cut)
    }

    fn write(
        &self,
        production: &mut Production,
        arrival: bool,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        let OwnedState::SaveFiles(state) = production.state() else {
            return Err(data_failure(Some(cut), "cartridge save scope"));
        };
        let start = state
            .started
            .as_ref()
            .ok_or_else(|| data_failure(Some(cut), "chosen file handoff"))?;
        let capture = if arrival { &self.arrival } else { &self.escape };
        let mut run = (**start).clone();
        run.progress = capture.progress.clone();
        run.maps = capture.maps.clone();
        let save = if arrival {
            // The fresh profile has no gameplay updates before the flight. Options can change it.
            run.profile.settings = capture.profile.settings;
            SaveCut::CeresArrival
        } else {
            run.profile = capture.profile.clone();
            SaveCut::CeresEscape {
                area: capture.area,
                station: capture.station,
            }
        };
        advance(
            production,
            Input::Write {
                cut: save,
                run: &run,
                live_map: &capture.live_map,
                stage: WriteStage::PayloadWritten,
            },
            cut,
        )
    }
}

fn advance(
    production: &mut Production,
    input: Input<'_>,
    cut: u64,
) -> Result<ProductionOutputs, Outcome> {
    production
        .advance(AdvanceInput::SaveFiles(input))
        .map_err(|fault| Outcome::Unsupported {
            cut: Some(cut),
            dependency: format!("cartridge production {fault:?}"),
        })
}

fn battery(bytes: Vec<u8>) -> Result<BatteryImage, Outcome> {
    BatteryImage::new(bytes).ok_or_else(|| data_failure(None, "complete battery image"))
}

pub(super) fn observed(image: BatteryImage) -> Observation {
    Observation {
        field: "save-files.battery".to_owned(),
        bytes: image.into(),
    }
}

pub(super) struct Expected(Vec<Observation>);

impl Expected {
    pub(super) fn read(&self, cut: u64) -> Result<ExpectedCut, Outcome> {
        let index = cut.checked_sub(1).and_then(|n| usize::try_from(n).ok());
        let field = index
            .and_then(|index| self.0.get(index))
            .cloned()
            .ok_or_else(|| data_failure(Some(cut), "battery expectation cut"))?;
        Ok(ExpectedCut {
            fields: vec![field.clone()],
            semantics: vec![field],
            semantic_count: 1,
            output_count: 0,
            replies: 0,
        })
    }
}

pub(super) fn admit(sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let raw =
        std::fs::read(&sources.rom).map_err(|_| data_failure(None, "save format ROM asset"))?;
    let rom = Rom::new(raw).map_err(|_| data_failure(None, "checked save format ROM"))?;
    let format = Format::load(rom.lorom()).map_err(|_| data_failure(None, "save format assets"))?;

    let text = std::fs::read_to_string(&sources.lock)
        .map_err(|_| data_failure(None, "battery source lock"))?;
    let lock = Lock::parse(&text).map_err(|_| data_failure(None, "battery lock schema"))?;
    let caches: BTreeMap<_, _> = [POWER_ON, FLIGHT, ARRIVAL, ESCAPE, LANDING]
        .into_iter()
        .map(|id| {
            let row = lock
                .row(&RowId(id.to_owned()))
                .ok_or_else(|| data_failure(None, "battery source row"))?;
            let cache = Cache::open(&sources.cache, row)
                .map_err(|_| data_failure(None, "pinned battery cache"))?;
            Ok((id, cache))
        })
        .collect::<Result<_, Outcome>>()?;
    let read = |row, frame| {
        let cache = caches
            .get(row)
            .ok_or_else(|| data_failure(Some(frame), "battery source cache"))?;
        record(cache, frame)
    };
    let input_image = battery(read(POWER_ON, 0)?.sram.into_vec())?;
    let capture = |row, frame| {
        Capture::from_record(&read(row, frame)?)
            .ok_or_else(|| data_failure(Some(frame), "preceding live save inputs"))
    };
    let inputs = Inputs {
        arrival: capture(FLIGHT, 802)?,
        escape: capture(ESCAPE, 239)?,
    };
    let expected = CUTS
        .into_iter()
        .map(|(row, frame, _)| battery(read(row, frame)?.sram.into_vec()).map(observed))
        .collect::<Result<_, Outcome>>()?;

    let state = save_files::State::new(save_files::power_on(&format, &input_image));
    let rules = ProductionRules::SaveFiles;
    let content = ProductionContent::SaveFiles(Box::new(format));
    let production = Production::admit(
        ProductionSeed::SaveFiles(Box::new(state)),
        content.clone(),
        rules.clone(),
    )
    .map_err(|_| data_failure(None, "cartridge production admission"))?;
    Ok(AdmittedCase {
        production,
        content,
        rules,
        inputs: InputReader::SaveFiles(Box::new(inputs)),
        expected: ExpectedReader::SaveFiles(Expected(expected)),
        actual: ActualReader::SaveFiles,
        schedule: (1..=END).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::save_files::SaveFormat;
    use data::rom::RomAddress;
    use types::Area;
    use types::save_files::EncodedRun;

    #[test]
    #[ignore = "requires the private player ROM"]
    fn cartridge_projection_imports_nonzero_selected_cells() {
        let path = std::env::var_os("SM_ROM").unwrap();
        let rom = Rom::new(std::fs::read(path).unwrap()).unwrap();
        let rom = rom.lorom();
        let format = Format::load(rom).unwrap();
        let [count] = *rom
            .read(RomAddress::new(0x81, 0x8131), 1)
            .unwrap()
            .first_chunk()
            .unwrap();
        let pointer = u16::from_le_bytes(
            *rom.read(RomAddress::new(0x81, 0x82d6), 2)
                .unwrap()
                .first_chunk()
                .unwrap(),
        );
        let list = rom
            .read(RomAddress::new(0x81, pointer), usize::from(count))
            .unwrap();
        let selected = usize::from(list[0]);
        let unselected = usize::from((0..=u8::MAX).find(|index| !list.contains(index)).unwrap());

        let mut run = format.decode(&EncodedRun::zeroed());
        run.maps[Area::Crateria].as_mut_slice()[selected] = 0xac;
        run.maps[Area::Crateria].as_mut_slice()[unselected] = 0x56;
        run.maps[Area::Ceres].as_mut_slice()[selected] = 0xfe;
        let payload = format.encode(&run);
        assert_eq!(payload.as_slice()[174].to_le_bytes()[0], 0xac);
        let decoded = format.decode(&payload);
        assert_eq!(decoded.maps[Area::Crateria].as_slice()[selected], 0xac);
        assert_eq!(decoded.maps[Area::Crateria].as_slice()[unselected], 0);
        assert_eq!(decoded.maps[Area::Ceres].as_slice()[selected], 0);
    }
}
