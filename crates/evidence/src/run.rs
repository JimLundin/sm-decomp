use crate::canonical::Reader;
use crate::{BLOB, Id, RUN, Store, verify_tree};
use anyhow::{Result, ensure};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Run {
    pub observations: Id,
    pub provenance: crate::Provenance,
    pub files: u64,
    pub bytes: u64,
}

/// Authenticate the observation closure before constructing the completed-run
/// object. Acceptance verdicts remain a separate concern for pipeline consumers.
/// The observation root stays stable when producer provenance changes.
pub fn put_run(store: &mut Store, observations: Id, provenance: &crate::Provenance) -> Result<Id> {
    let (files, bytes) = verify_tree(store, &observations, None)?;
    provenance.validate()?;
    let canonical = toml::to_string(provenance)?.into_bytes();
    let producer = store.put(BLOB, &canonical)?;
    let mut out = vec![2]; // v2 completed run; producer metadata is canonical typed TOML
    out.extend_from_slice(&observations);
    out.extend_from_slice(&producer);
    crate::canonical::varint(&mut out, files);
    crate::canonical::varint(&mut out, bytes);
    store.put(RUN, &out)
}

pub fn read_run(store: &Store, id: &Id) -> Result<Run> {
    let bytes = store.get_kind(id, RUN)?;
    let mut r = Reader::new(&bytes);
    ensure!(r.byte()? == 2, "unknown completed run version");
    let observations = r.id()?;
    let provenance_bytes = store.get_kind(&r.id()?, BLOB)?;
    let provenance: crate::Provenance = toml::from_str(std::str::from_utf8(&provenance_bytes)?)?;
    provenance.validate()?;
    ensure!(toml::to_string(&provenance)?.as_bytes() == provenance_bytes, "noncanonical provenance TOML");
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

/// Restore each member once, returning its authenticated digest and run counts.
pub fn verified_inventory(
    store: &Store,
    id: &Id,
) -> Result<(Run, std::collections::BTreeMap<String, crate::MemberInfo>)> {
    let run = read_run(store, id)?;
    let inventory = crate::tree_inventory(store, &run.observations)?;
    let bytes = inventory.values().try_fold(0u64, |total, member| {
        total.checked_add(member.bytes).ok_or_else(|| anyhow::anyhow!("inventory byte count overflow"))
    })?;
    ensure!(inventory.len() as u64 == run.files && bytes == run.bytes, "run closure counts differ");
    Ok((run, inventory))
}
