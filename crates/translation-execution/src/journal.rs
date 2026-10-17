use crate::model::{FrozenPolicy, sha256};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    cell::RefCell,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

const MAX_EVENT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_JOURNAL_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: String,
    run_id: String,
    policy_sha256: String,
    policy: FrozenPolicy,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Event<T> {
    schema: String,
    sequence: u64,
    previous_sha256: String,
    state: T,
}

/// Protected, hash-chained intent/result snapshots. Locks survive for the entire
/// operation, including adapter reconciliation, rather than only file writes.
pub(crate) struct Journal {
    root: PathBuf,
    pub run_id: String,
    pub policy_sha256: String,
    max_events: u64,
    head: RefCell<Option<VerifiedHead>>,
}

struct VerifiedHead {
    sequence: u64,
    sha256: String,
    bytes: u64,
    state: Option<serde_json::Value>,
}

fn safe_directory(path: &Path) -> Result<()> {
    ensure!(path.is_absolute(), "controller journal must have an absolute path");
    for ancestor in path.ancestors() {
        ensure!(!ancestor.is_symlink(), "controller journal crosses a symlink");
    }
    Ok(())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    ensure!(!path.is_symlink(), "journal output cannot be a symlink");
    let pending = path.with_extension("pending");
    ensure!(!pending.is_symlink(), "pending journal output cannot be a symlink");
    // The exclusive journal lock makes an orphaned pending file safe to retire.
    if pending.exists() {
        fs::remove_file(&pending)?;
    }
    let mut out = OpenOptions::new().create_new(true).write(true).open(&pending)?;
    out.write_all(bytes)?;
    out.sync_all()?;
    fs::rename(&pending, path)?;
    File::open(path.parent().context("journal parent missing")?)?.sync_all()?;
    Ok(())
}

fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    crate::retention::read_bounded(path, MAX_EVENT_BYTES)
}

impl Journal {
    /// Authenticate an existing history without creating a directory, lock or
    /// policy record. Retained proof inspection uses this after retirement.
    pub fn read_only(root: &Path, policy: &FrozenPolicy) -> Result<(Self, File)> {
        safe_directory(root)?;
        ensure!(root.is_dir(), "retained journal is missing");
        let lock_path = root.join("controller.lock");
        ensure!(!lock_path.is_symlink(), "retained journal lock is aliased");
        let lock = File::open(lock_path)?;
        lock.try_lock_shared().context("retained journal has an active controller")?;
        Ok((Self::existing(root, policy)?, lock))
    }

    /// Read immutable state without taking a second lease. Only a caller
    /// already authenticating a retained history or holding the lifecycle lease
    /// can use this; no directory, lock or journal record is created.
    pub(crate) fn existing(root: &Path, policy: &FrozenPolicy) -> Result<Self> {
        safe_directory(root)?;
        ensure!(root.is_dir(), "retained journal is missing");
        let policy_sha256 = crate::model::sealed(policy)?;
        let run_id = sha256(format!("translation-execution-run-v1:{policy_sha256}"));
        let max_events = u64::from(policy.budgets.max_attempts)
            .checked_mul(24 + policy.verification_stages.len() as u64 * 8)
            .and_then(|count| count.checked_add(4))
            .context("journal event budget overflow")?;
        let manifest: Manifest = serde_json::from_slice(&read_bytes(&root.join("policy.json"))?)?;
        ensure!(
            manifest.schema == "translation-execution-journal-v1"
                && manifest.run_id == run_id
                && manifest.policy_sha256 == policy_sha256
                && manifest.policy == *policy,
            "retained journal policy changed"
        );
        let journal = Self { root: root.into(), run_id, policy_sha256, max_events, head: RefCell::new(None) };
        journal.verify_head()?;
        Ok(journal)
    }

    pub fn open(root: &Path, policy: &FrozenPolicy) -> Result<Self> {
        safe_directory(root)?;
        fs::create_dir_all(root)?;
        let policy_sha256 = crate::model::sealed(policy)?;
        let run_id = sha256(format!("translation-execution-run-v1:{policy_sha256}"));
        let max_events = u64::from(policy.budgets.max_attempts)
            .checked_mul(24 + policy.verification_stages.len() as u64 * 8)
            .and_then(|count| count.checked_add(4))
            .context("journal event budget overflow")?;
        let journal = Self { root: root.to_path_buf(), run_id, policy_sha256, max_events, head: RefCell::new(None) };
        let _lock = journal.lock()?;
        let path = root.join("policy.json");
        if path.exists() {
            let saved: Manifest = serde_json::from_slice(&read_bytes(&path)?)?;
            ensure!(
                saved.schema == "translation-execution-journal-v1"
                    && saved.run_id == journal.run_id
                    && saved.policy_sha256 == journal.policy_sha256
                    && saved.policy == *policy,
                "journal frozen policy changed"
            );
        } else {
            let manifest = Manifest {
                schema: "translation-execution-journal-v1".into(),
                run_id: journal.run_id.clone(),
                policy_sha256: journal.policy_sha256.clone(),
                policy: policy.clone(),
            };
            write_atomic(&path, &serde_json::to_vec(&manifest)?)?;
        }
        Ok(journal)
    }

    pub fn lock(&self) -> Result<File> {
        safe_directory(&self.root)?;
        let path = self.root.join("controller.lock");
        ensure!(!path.is_symlink(), "controller lock cannot be a symlink");
        let lock = OpenOptions::new().create(true).append(true).open(path)?;
        lock.lock()?;
        // Cache reuse is valid only within this exclusive operation. Independent
        // resume/status authenticates the full persisted history again.
        *self.head.borrow_mut() = None;
        Ok(lock)
    }

    pub fn load<T: DeserializeOwned>(&self) -> Result<Option<T>> {
        self.verify_head()?;
        self.head
            .borrow()
            .as_ref()
            .context("verified journal head missing")?
            .state
            .clone()
            .map(serde_json::from_value)
            .transpose()
            .map_err(Into::into)
    }

    /// Walk the authenticated history one bounded state at a time. A caller can
    /// validate its domain transitions and historical acceptance without
    /// collecting every source capsule into memory. The caller holds the
    /// journal's existing shared or exclusive lease throughout this walk.
    pub fn visit_states<T: DeserializeOwned>(&self, mut visit: impl FnMut(u64, T) -> Result<()>) -> Result<()> {
        self.visit_events(|sequence, state, _| visit(sequence, state))
    }

    /// Includes exact authenticated event bytes for a pure historical-tail
    /// comparison. The bytes stay bounded to one event throughout the walk.
    pub(crate) fn visit_events<T: DeserializeOwned>(
        &self,
        mut visit: impl FnMut(u64, T, &[u8]) -> Result<()>,
    ) -> Result<()> {
        self.verify_head()?;
        let (sequence, expected_head) = {
            let head = self.head.borrow();
            let head = head.as_ref().context("verified journal head missing")?;
            (head.sequence, head.sha256.clone())
        };
        let mut previous = self.policy_sha256.clone();
        for sequence in 1..=sequence {
            let bytes = read_bytes(&self.root.join(format!("event-{sequence:08}.json")))?;
            let event: Event<T> = serde_json::from_slice(&bytes)?;
            ensure!(
                event.schema == "translation-execution-event-v1"
                    && event.sequence == sequence
                    && event.previous_sha256 == previous,
                "journal changed during retained history inspection"
            );
            previous = sha256(&bytes);
            visit(sequence, event.state, &bytes)?;
        }
        ensure!(previous == expected_head, "journal head changed during retained history inspection");
        Ok(())
    }

    pub fn retained_bytes(&self) -> Result<u64> {
        self.verify_head()?;
        Ok(self.head.borrow().as_ref().context("verified journal head missing")?.bytes)
    }

    fn append_bytes<T: Serialize>(&self, state: &T) -> Result<Vec<u8>> {
        self.verify_head()?;
        let head = self.head.borrow();
        let head = head.as_ref().context("verified journal head missing")?;
        let sequence = head.sequence.checked_add(1).context("journal sequence overflow")?;
        let bytes = self.event_bytes(sequence, &head.sha256, state)?;
        ensure!(
            head.bytes.checked_add(bytes.len() as u64).is_some_and(|total| total <= MAX_JOURNAL_BYTES),
            "controller journal exceeds fixed metadata storage budget"
        );
        Ok(bytes)
    }

    /// Shared exact encoding for real appends, prospective appends and an
    /// authenticated historical successor. Callers supply a verified prefix.
    pub(crate) fn event_bytes<T: Serialize>(&self, sequence: u64, previous_sha256: &str, state: &T) -> Result<Vec<u8>> {
        ensure!((1..=self.max_events).contains(&sequence), "journal event budget exhausted");
        let bytes = serde_json::to_vec(&Event {
            schema: "translation-execution-event-v1".into(),
            sequence,
            previous_sha256: previous_sha256.into(),
            state,
        })?;
        ensure!(bytes.len() as u64 <= MAX_EVENT_BYTES, "journal state exceeds bounded metadata size");
        Ok(bytes)
    }

    /// Serialize the exact next event under the existing lease without writing
    /// it. Publication can reserve its eventual terminal tail before accepting
    /// the result whose decimal byte count is contained in that same event.
    pub fn projected_append_bytes<T: Serialize>(&self, state: &T) -> Result<u64> {
        Ok(self.append_bytes(state)?.len() as u64)
    }

    fn verify_head(&self) -> Result<()> {
        if self.head.borrow().is_some() {
            return Ok(());
        }
        let mut paths = Vec::new();
        let mut total_bytes = 0u64;
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            total_bytes = total_bytes
                .checked_add(if metadata.is_file() { metadata.len() } else { 0 })
                .context("journal byte count overflow")?;
            let path = entry.path();
            let name = path.file_name().and_then(|name| name.to_str()).context("non-UTF8 journal path")?;
            if name.starts_with("event-") && name.ends_with(".json") {
                paths.push(path);
            }
        }
        ensure!(paths.len() as u64 <= self.max_events, "journal event budget exhausted");
        ensure!(total_bytes <= MAX_JOURNAL_BYTES, "controller journal exceeds fixed metadata storage budget");
        paths.sort();
        let mut sequence = 0;
        let mut previous = self.policy_sha256.clone();
        let mut state = None;
        for path in paths {
            let bytes = read_bytes(&path)?;
            let event: Event<serde_json::Value> = serde_json::from_slice(&bytes)?;
            sequence += 1;
            ensure!(
                event.schema == "translation-execution-event-v1"
                    && event.sequence == sequence
                    && event.previous_sha256 == previous
                    && path.file_name().and_then(|name| name.to_str()) == Some(&format!("event-{sequence:08}.json")),
                "journal sequence or predecessor seal changed"
            );
            previous = sha256(&bytes);
            state = Some(event.state);
        }
        *self.head.borrow_mut() = Some(VerifiedHead { sequence, sha256: previous, bytes: total_bytes, state });
        Ok(())
    }

    pub fn append<T: Serialize + DeserializeOwned>(&self, state: &T) -> Result<()> {
        self.verify_head()?;
        let (sequence, current_bytes) = {
            let head = self.head.borrow();
            let head = head.as_ref().context("verified journal head missing")?;
            (head.sequence.checked_add(1).context("journal sequence overflow")?, head.bytes)
        };
        ensure!(sequence <= self.max_events, "journal event budget exhausted");
        let path = self.root.join(format!("event-{sequence:08}.json"));
        ensure!(!path.exists(), "journal event already exists");
        let bytes = self.append_bytes(state)?;
        write_atomic(&path, &bytes)?;
        *self.head.borrow_mut() = Some(VerifiedHead {
            sequence,
            sha256: sha256(&bytes),
            bytes: current_bytes + bytes.len() as u64,
            state: Some(serde_json::to_value(state)?),
        });
        Ok(())
    }
}
