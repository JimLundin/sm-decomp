//! Storage for a finite observation interval; gameplay acceptance stays with its caller.
use crate::rio_observation::Observer;
use sha2::{Digest, Sha256};
use sm_native_host::State;
use sm_trace::{Identity, Writer};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const MAX_BYTES: usize = 512 * 1024 * 1024;

pub struct Capture {
    start: u64,
    count: u64,
    rows: u64,
    bytes: usize,
    failure: Option<String>,
    previous: [u64; 67],
    observer: Observer,
    directory: PathBuf,
    facts: BufWriter<File>,
    inventory: BufWriter<File>,
    trace: Option<Writer>,
}

impl Capture {
    pub fn create(
        out: &Path,
        start: u64,
        count: u64,
        previous: [u64; 67],
        identity: &Identity,
    ) -> Result<Self, String> {
        if start == 0 || count == 0 || count > 2048 || start.checked_add(count - 1).is_none() {
            return Err("invalid Rio capture interval".into());
        }
        let directory = out.join("rio-samples");
        fs::create_dir(&directory).map_err(|e| e.to_string())?;
        let mut facts = BufWriter::new(File::create(out.join("rio-facts.csv")).map_err(|e| e.to_string())?);
        writeln!(facts, "step,room,health,identity,x,x_fraction,y,y_fraction,state,event,animation,horizontal_velocity,vertical_velocity,sampled_event_consumption,waiting_with_positive_health,counter_delta")
            .map_err(|e| e.to_string())?;
        let trace = Writer::create(out.join("rio-trace.bin"), identity).map_err(|e| e.to_string())?;
        let inventory = BufWriter::new(File::create(out.join("rio-samples.sha256")).map_err(|e| e.to_string())?);
        Ok(Self {
            start,
            count,
            rows: 0,
            bytes: 0,
            failure: None,
            previous,
            observer: Observer::default(),
            directory,
            facts,
            inventory,
            trace: Some(trace),
        })
    }

    pub fn record(
        &mut self,
        state: &State,
        ram: &[u8],
        external: &[u8],
        effects: &[u8],
        counters: [u64; 67],
    ) -> Result<(), String> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        let result = self.record_step(state, ram, external, effects, counters);
        if let Err(error) = &result {
            self.failure = Some(error.clone());
        }
        result
    }

    fn record_step(
        &mut self,
        state: &State,
        ram: &[u8],
        external: &[u8],
        effects: &[u8],
        counters: [u64; 67],
    ) -> Result<(), String> {
        let mut delta = [0; 67];
        for (index, value) in delta.iter_mut().enumerate() {
            *value =
                counters[index].checked_sub(self.previous[index]).ok_or("Rio counters decreased during capture")?;
        }
        self.previous = counters;
        if state.host_steps < self.start || self.rows == self.count {
            return Ok(());
        }
        if state.host_steps != self.start + self.rows {
            return Err("Rio capture missed the configured host step".into());
        }
        let size = ram
            .len()
            .checked_add(external.len())
            .and_then(|n| n.checked_add(effects.len()))
            .and_then(|n| n.checked_add(2 * 67 * 8))
            .and_then(|n| self.bytes.checked_add(n))
            .filter(|&n| n <= MAX_BYTES)
            .ok_or("Rio capture exceeds its 512 MiB raw observation budget")?;
        // Persist raw evidence before validation so a rejected transaction remains inspectable.
        for (extension, bytes) in [("wram", ram), ("external", external), ("effects", effects)] {
            fs::write(self.directory.join(format!("step-{:06}.{extension}", state.host_steps)), bytes)
                .map_err(|e| e.to_string())?;
            writeln!(
                self.inventory,
                "{:x}  rio-samples/step-{:06}.{extension}",
                Sha256::digest(bytes),
                state.host_steps
            )
            .map_err(|e| e.to_string())?;
        }
        let counter_bytes: Vec<_> = counters.into_iter().chain(delta).flat_map(u64::to_le_bytes).collect();
        fs::write(self.directory.join(format!("step-{:06}.counters", state.host_steps)), &counter_bytes)
            .map_err(|e| e.to_string())?;
        writeln!(
            self.inventory,
            "{:x}  rio-samples/step-{:06}.counters",
            Sha256::digest(&counter_bytes),
            state.host_steps
        )
        .map_err(|e| e.to_string())?;
        self.inventory.flush().map_err(|e| e.to_string())?;
        let observation = self.observer.observe(state.host_steps, ram, effects, delta)?;
        let actor = observation.actor;
        let counts = delta.iter().map(u64::to_string).collect::<Vec<_>>().join(";");
        writeln!(
            self.facts,
            "{},{:04x},{},{},{},{},{},{},{:04x},{},{:04x},{},{},{},{},{}",
            observation.step,
            observation.room,
            observation.health,
            observation.expected_identity,
            actor.x,
            actor.x_fraction,
            actor.y,
            actor.y_fraction,
            actor.state,
            actor.animation_event,
            actor.selected_animation,
            actor.horizontal_velocity,
            actor.vertical_velocity,
            observation.sampled_event_consumption,
            observation.waiting_with_positive_health,
            counts
        )
        .map_err(|e| e.to_string())?;
        self.facts.flush().map_err(|e| e.to_string())?;
        self.trace
            .as_mut()
            .ok_or("Rio capture already finished")?
            .record(self.rows + 1, ram, external, effects)
            .map_err(|e| e.to_string())?;
        self.bytes = size;
        self.rows += 1;
        Ok(())
    }

    pub fn finish(mut self) -> Result<(u64, bool), String> {
        if let Some(error) = self.failure.take() {
            return Err(error);
        }
        self.facts.flush().map_err(|e| e.to_string())?;
        self.inventory.flush().map_err(|e| e.to_string())?;
        if self.rows > 0 {
            self.trace.take().ok_or("Rio capture already finished")?.finish().map_err(|e| e.to_string())?;
        }
        Ok((self.rows, self.rows == self.count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sm_trace::Reference;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "sm-rio-capture-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn identity() -> Identity {
        Identity {
            reference: "test-reference".into(),
            rom: "test-rom".into(),
            host: "test-host".into(),
            input: "test-input".into(),
            observation: "test-schema".into(),
            scenario: "interval-start-2".into(),
        }
    }
    #[test]
    fn skipped_prefix_updates_baseline_and_raw_records_match_authenticated_trace() {
        let directory = Directory::new();
        let identity = identity();
        let mut capture = Capture::create(&directory.0, 2, 2, [0; 67], &identity).unwrap();
        let ram = vec![0; crate::rio_observation::WRAM_SIZE];
        let mut counts = [0; 67];
        counts[0] = 1;
        counts[10] = 1;
        capture.record(&State { host_steps: 1, ..State::default() }, &ram, b"prefix", &[], counts).unwrap();
        let effects: Vec<_> =
            [[68u32, 1, 0, 0], [69, 1, 0, 0]].into_iter().flatten().flat_map(u32::to_le_bytes).collect();
        counts[1] = 1;
        counts[11] = 1;
        capture
            .record(&State { host_steps: 2, waits: 1, ..State::default() }, &ram, b"external", &effects, counts)
            .unwrap();
        capture.record(&State { host_steps: 3, ..State::default() }, &ram, b"last", &[], counts).unwrap();
        assert_eq!(capture.finish().unwrap(), (2, true));
        let reference = Reference::load(directory.0.join("rio-trace.bin"), &identity).unwrap();
        let mut comparison = reference.comparison();
        comparison.check(1, &ram, b"external", &effects).unwrap();
        comparison.check(2, &ram, b"last", &[]).unwrap();
        comparison.finish().unwrap();
        assert_eq!(fs::read(directory.0.join("rio-samples/step-000002.effects")).unwrap(), effects);
        assert!(!directory.0.join("rio-samples/step-000001.wram").exists());
    }
    #[test]
    fn invalid_interval_truncation_and_rejected_transactions_stay_distinct() {
        let directory = Directory::new();
        let identity = identity();
        assert!(Capture::create(&directory.0, u64::MAX, 2, [0; 67], &identity).is_err());
        let mut capture = Capture::create(&directory.0, 2, 2, [0; 67], &identity).unwrap();
        let ram = vec![0; crate::rio_observation::WRAM_SIZE];
        capture.record(&State { host_steps: 2, ..State::default() }, &ram, b"external", &[], [0; 67]).unwrap();
        assert_eq!(capture.finish().unwrap(), (1, false));
        let directory = Directory::new();
        let mut capture = Capture::create(&directory.0, 1, 1, [0; 67], &identity).unwrap();
        assert!(
            capture.record(&State { host_steps: 1, ..State::default() }, &ram, b"external", &[0], [0; 67]).is_err()
        );
        assert_eq!(fs::read(directory.0.join("rio-samples/step-000001.effects")).unwrap(), [0]);
        assert!(capture.record(&State { host_steps: 1, ..State::default() }, &ram, b"external", &[], [0; 67]).is_err());
        assert!(capture.finish().is_err());
        assert!(Reference::load(directory.0.join("rio-trace.bin"), &identity).is_err());
    }
}
