use crate::canonical::Reader;
use crate::{BLOB, Id, RUN, Store, verify_tree};
use anyhow::{Result, ensure};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Run {
    pub observations: Id,
    pub provenance: serde_json::Value,
    pub files: u64,
    pub bytes: u64,
}

/// Authenticate the observation closure before constructing the completed-run
/// object. Acceptance verdicts remain a separate concern for pipeline consumers.
/// The observation root stays stable when producer provenance changes.
pub fn put_run(store: &mut Store, observations: Id, provenance: &serde_json::Value) -> Result<Id> {
    let (files, bytes) = verify_tree(store, &observations, None)?;
    let canonical = serde_json::to_vec(provenance)?;
    let producer = store.put(BLOB, &canonical)?;
    let mut out = vec![1]; // completed, never an implicit partial run
    out.extend_from_slice(&observations);
    out.extend_from_slice(&producer);
    crate::canonical::varint(&mut out, files);
    crate::canonical::varint(&mut out, bytes);
    store.put(RUN, &out)
}

pub fn read_run(store: &Store, id: &Id) -> Result<Run> {
    let bytes = store.get_kind(id, RUN)?;
    let mut r = Reader::new(&bytes);
    ensure!(r.byte()? == 1, "run is incomplete or unknown status");
    let observations = r.id()?;
    let provenance_bytes = store.get_kind(&r.id()?, BLOB)?;
    let provenance: serde_json::Value = serde_json::from_slice(&provenance_bytes)?;
    ensure!(serde_json::to_vec(&provenance)? == provenance_bytes, "noncanonical provenance JSON");
    let files = r.varint()?;
    let count = r.varint()?;
    r.done()?;
    Ok(Run { observations, provenance, files, bytes: count })
}

pub fn verify_run(store: &Store, id: &Id, source: Option<&Path>) -> Result<Run> {
    let run = read_run(store, id)?;
    let (files, bytes) = verify_tree(store, &run.observations, source)?;
    ensure!(files == run.files && bytes == run.bytes, "run closure counts differ");
    Ok(run)
}
