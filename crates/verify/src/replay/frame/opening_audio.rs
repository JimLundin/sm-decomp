//! Independent opening CPU audio port comparison in physical bus order.

use super::{Observation, data_failure, read_expected};
use crate::{lock::Sha256, replay::result::Outcome};
use canonical::Canonical;
use data::timing::MasterTicks;
use game::production::timing::{Event, EventKind};
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub(in crate::replay) struct Source(PathBuf);

#[derive(Deserialize)]
struct Capture {
    format: u16,
    cpu_port_reads: usize,
    cpu_port_writes: usize,
    cpu_port: Vec<Recorded>,
}

#[derive(Deserialize)]
struct Recorded {
    cut: u64,
    order: usize,
    master_ticks: u64,
    kind: Access,
    library: u8,
    value: u8,
}

#[derive(Canonical, Copy, Clone, PartialEq, Eq, Deserialize)]
enum Access {
    Read,
    Write,
}

#[derive(Canonical)]
struct Callback {
    at: MasterTicks,
    access: Access,
    library: u8,
    value: u8,
}

impl Source {
    pub(in crate::replay) fn new(root: &Path) -> Self {
        Self(root.join(
            "opening-clock-movie-timing-expected-v1/opening-movie-timing-expected.private.toml",
        ))
    }

    pub(super) fn read(&self, cut: u64) -> Result<(Observation, usize), Outcome> {
        let fault = || data_failure(Some(cut), "locked opening CPU audio port stream");
        let pin =
            Sha256::try_from("0f5edeeb257e065b1f4e200656b8c778e0e319a2e55809e7e1668110f2e772cc")
                .map_err(|_| fault())?;
        let bytes = read_expected(&self.0, pin, cut)?;
        let capture: Capture = toml::from_str(std::str::from_utf8(&bytes).map_err(|_| fault())?)
            .map_err(|_| fault())?;
        if capture.format != 1
            || capture.cpu_port_reads.checked_add(capture.cpu_port_writes)
                != Some(capture.cpu_port.len())
            || capture
                .cpu_port
                .iter()
                .enumerate()
                .any(|(order, row)| row.order != order || row.library > 3)
            || !capture
                .cpu_port
                .is_sorted_by(|left, right| left.master_ticks < right.master_ticks)
        {
            return Err(fault());
        }
        let native_cut = cut.checked_sub(1).ok_or_else(fault)?;
        let ports: Vec<_> = capture
            .cpu_port
            .into_iter()
            .filter(|row| row.cut == native_cut)
            .map(|row| Callback {
                at: MasterTicks(row.master_ticks),
                access: row.kind,
                library: row.library,
                value: row.value,
            })
            .collect();
        let replies = ports
            .iter()
            .filter(|event| event.access == Access::Read)
            .count();
        Ok((encode(canonical::to_bytes(&ports)), replies))
    }
}

pub(in crate::replay) fn actual(events: &[Event]) -> Observation {
    let observed: Vec<_> = events
        .iter()
        .filter_map(|event| {
            let (port, access, value) = match &event.kind {
                EventKind::Publication(write) => (write.port, Access::Write, write.value),
                EventKind::AudioPortRead { port, value } => (*port, Access::Read, *value),
                EventKind::Payload(_)
                | EventKind::BulkService(_)
                | EventKind::LineService(_)
                | EventKind::Interrupt(_)
                | EventKind::Sample(..)
                | EventKind::ControllerStart
                | EventKind::ControllerEnd
                | EventKind::ControllerCommitted => return None,
            };
            let library = port
                .0
                .checked_sub(0x2140)
                .and_then(|value| u8::try_from(value).ok())?;
            (library <= 3).then_some(Callback {
                at: event.at,
                access,
                library,
                value,
            })
        })
        .collect();
    encode(canonical::to_bytes(&observed))
}

fn encode(bytes: Vec<u8>) -> Observation {
    Observation {
        field: "frame.audio.ports".to_owned(),
        bytes,
    }
}
