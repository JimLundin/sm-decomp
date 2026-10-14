use crate::canonical::{MAX_OBJECT, Reader, varint};
use crate::{BLOB, FILE, Id, Store, TREE, ingest_trace, put_image, write_image, write_stream};
use anyhow::{Context, Result, bail, ensure};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const CHUNK: usize = 1 << 20;
const MAX_CHUNKS: usize = 60000;

/// Bounded blob chunks for arbitrary file bytes (up to 60 GiB per file).
pub fn put_file(store: &mut Store, input: &mut dyn Read) -> Result<Id> {
    let mut buffer = vec![0; CHUNK];
    let mut ids = Vec::new();
    let mut total = 0u64;
    loop {
        let mut n = 0;
        while n < CHUNK {
            let got = input.read(&mut buffer[n..])?;
            if got == 0 {
                break;
            }
            n += got;
        }
        if n == 0 {
            break;
        }
        ensure!(ids.len() < MAX_CHUNKS, "file exceeds chunk limit");
        ids.push(store.put(BLOB, &buffer[..n])?);
        total += n as u64;
    }
    let mut out = Vec::new();
    varint(&mut out, total);
    varint(&mut out, ids.len() as u64);
    for id in ids {
        out.extend_from_slice(&id);
    }
    store.put(FILE, &out)
}

pub fn write_file(store: &Store, id: &Id, out: &mut dyn Write) -> Result<()> {
    let bytes = store.get_kind(id, FILE)?;
    let mut r = Reader::new(&bytes);
    let total = r.varint()?;
    let count = r.len(MAX_CHUNKS)?;
    ensure!(count as u64 == total.div_ceil(CHUNK as u64), "file chunk count differs");
    for i in 0..count {
        let chunk = store.get_kind(&r.id()?, BLOB)?;
        let expected = (total - i as u64 * CHUNK as u64).min(CHUNK as u64);
        ensure!(chunk.len() as u64 == expected, "file chunk length differs");
        out.write_all(&chunk)?;
    }
    r.done()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum EntryKind {
    File = b'f',
    Image = b'i',
    Stream = b's',
    Tree = b't',
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub name: String,
    pub kind: EntryKind,
    pub id: Id,
}

fn valid_name(name: &str) -> Result<()> {
    ensure!(!name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\\', '\0']), "unsafe tree name");
    Ok(())
}

/// Construct a sorted directory from already recorded observations.
pub fn put_tree(store: &mut Store, mut entries: Vec<Entry>) -> Result<Id> {
    entries.sort_by(|a, b| a.name.as_bytes().cmp(b.name.as_bytes()));
    let mut out = Vec::new();
    let mut previous: Option<&str> = None;
    for entry in &entries {
        valid_name(&entry.name)?;
        ensure!(entry.name.len() <= 65535 && previous != Some(&entry.name), "duplicate or oversized tree name");
        previous = Some(&entry.name);
        let expected = match entry.kind {
            EntryKind::File => FILE,
            EntryKind::Image => crate::IMAGE,
            EntryKind::Stream => crate::STREAM,
            EntryKind::Tree => TREE,
        };
        store.get_kind(&entry.id, expected)?;
        varint(&mut out, entry.name.len() as u64);
        out.extend_from_slice(entry.name.as_bytes());
        out.push(entry.kind as u8);
        out.extend_from_slice(&entry.id);
        ensure!(out.len() <= MAX_OBJECT, "directory exceeds entry limit");
    }
    store.put(TREE, &out)
}

#[derive(Debug, Clone)]
pub struct MemberInfo {
    pub bytes: u64,
    pub sha256: Id,
}

/// Restore and hash every member without materialising it on disk.
pub fn tree_inventory(store: &Store, id: &Id) -> Result<std::collections::BTreeMap<String, MemberInfo>> {
    fn walk(
        store: &Store,
        id: &Id,
        prefix: &str,
        depth: usize,
        result: &mut std::collections::BTreeMap<String, MemberInfo>,
    ) -> Result<()> {
        ensure!(depth <= 64, "tree nesting exceeds limit");
        for entry in read_tree(store, id)? {
            let path = format!("{prefix}{}", entry.name);
            if entry.kind == EntryKind::Tree {
                walk(store, &entry.id, &format!("{path}/"), depth + 1, result)?;
            } else {
                result.insert(path, authenticate_entry(store, entry.kind, &entry.id)?);
            }
        }
        Ok(())
    }
    let mut result = std::collections::BTreeMap::new();
    walk(store, id, "", 0, &mut result)?;
    Ok(result)
}

/// Ingest a directory without following symlinks. Names are UTF-8 and sorted.
/// Captures observations exactly; does not mark them as accepted evidence.
pub fn ingest_tree(store: &mut Store, directory: &Path, progress: &mut dyn FnMut(&Path)) -> Result<Id> {
    ingest(store, directory, progress, 0)
}

fn ingest(store: &mut Store, dir: &Path, progress: &mut dyn FnMut(&Path), depth: usize) -> Result<Id> {
    ensure!(depth <= 64 && fs::symlink_metadata(dir)?.is_dir(), "invalid or too deep directory");
    let mut names = fs::read_dir(dir)?.map(|e| e.map(|e| e.file_name())).collect::<std::io::Result<Vec<_>>>()?;
    names.sort();
    let mut out = Vec::new();
    for name in names {
        let text = name.to_str().context("non-UTF-8 tree name")?;
        valid_name(text)?;
        let path = dir.join(&name);
        let meta = fs::symlink_metadata(&path)?;
        let (kind, id) = if meta.is_dir() {
            (EntryKind::Tree, ingest(store, &path, progress, depth + 1)?)
        } else {
            ensure!(meta.is_file(), "symlink or special file rejected: {}", path.display());
            progress(&path);
            if text == "hardware.log" || text == "routine-calls.log" {
                (
                    EntryKind::Stream,
                    ingest_trace(store, &mut std::io::BufReader::with_capacity(CHUNK, File::open(&path)?))?,
                )
            } else if [".wram", ".rom", ".bin", ".srm", ".sram"].iter().any(|s| text.ends_with(s)) {
                ensure!(meta.len() <= 1 << 30, "image exceeds limit");
                (EntryKind::Image, put_image(store, &fs::read(&path)?)?)
            } else {
                (EntryKind::File, put_file(store, &mut File::open(&path)?)?)
            }
        };
        varint(&mut out, text.len() as u64);
        out.extend_from_slice(text.as_bytes());
        out.push(kind as u8);
        out.extend_from_slice(&id);
        ensure!(out.len() <= MAX_OBJECT, "directory exceeds entry limit");
    }
    store.put(TREE, &out)
}

pub fn read_tree(store: &Store, id: &Id) -> Result<Vec<Entry>> {
    let bytes = store.get_kind(id, TREE)?;
    let mut r = Reader::new(&bytes);
    let mut entries: Vec<Entry> = Vec::new();
    while r.at < bytes.len() {
        let len = r.len(65535)?;
        let name = std::str::from_utf8(r.take(len)?)?.to_owned();
        valid_name(&name)?;
        if let Some(prev) = entries.last() {
            ensure!(prev.name.as_bytes() < name.as_bytes(), "tree names not sorted or duplicate");
        }
        let kind = match r.byte()? {
            b'f' => EntryKind::File,
            b'i' => EntryKind::Image,
            b's' => EntryKind::Stream,
            b't' => EntryKind::Tree,
            _ => bail!("unknown tree entry kind"),
        };
        entries.push(Entry { name, kind, id: r.id()? });
    }
    Ok(entries)
}

pub fn write_entry(store: &Store, kind: EntryKind, id: &Id, out: &mut dyn Write) -> Result<()> {
    match kind {
        EntryKind::File => write_file(store, id, out),
        EntryKind::Image => write_image(store, id, out),
        EntryKind::Stream => write_stream(store, id, out),
        EntryKind::Tree => bail!("tree is a directory"),
    }
}

// Full stream restoration already computes the exact-byte digest to check its
// seal. Reuse that computed result within this traversal, never the declared
// digest or a result cached from an earlier authentication.
fn authenticate_entry(store: &Store, kind: EntryKind, id: &Id) -> Result<MemberInfo> {
    if kind == EntryKind::Stream {
        crate::trace::restore_stream(store, id, &mut std::io::sink())
    } else {
        let mut hash = HashWriter::default();
        write_entry(store, kind, id, &mut hash)?;
        Ok(hash.finish())
    }
}

pub(crate) struct HashWriter(Sha256, u64);
impl HashWriter {
    pub(crate) fn finish(self) -> MemberInfo {
        MemberInfo { bytes: self.1, sha256: self.0.finalize().into() }
    }
}
impl Default for HashWriter {
    fn default() -> Self {
        Self(Sha256::new(), 0)
    }
}
impl Write for HashWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        self.1 += bytes.len() as u64;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Authenticate all referenced bytes, optionally compare to a source directory.
/// Extra or missing source members also fail verification.
pub fn verify_tree(store: &Store, id: &Id, source: Option<&Path>) -> Result<(u64, u64)> {
    fn walk(store: &Store, id: &Id, source: Option<&Path>, depth: usize) -> Result<(u64, u64)> {
        ensure!(depth <= 64, "tree nesting exceeds limit");
        let entries = read_tree(store, id)?;
        if let Some(dir) = source {
            ensure!(fs::symlink_metadata(dir)?.is_dir(), "source is not directory");
            let mut names =
                fs::read_dir(dir)?.map(|e| e.map(|e| e.file_name())).collect::<std::io::Result<Vec<_>>>()?;
            names.sort();
            ensure!(
                names.len() == entries.len() && names.iter().zip(&entries).all(|(n, e)| n.to_str() == Some(&e.name)),
                "source directory members differ"
            );
        }
        let mut totals = (0, 0);
        for entry in entries {
            let path = source.map(|p| p.join(&entry.name));
            if entry.kind == EntryKind::Tree {
                let n = walk(store, &entry.id, path.as_deref(), depth + 1)?;
                totals.0 += n.0;
                totals.1 += n.1;
            } else {
                let member = authenticate_entry(store, entry.kind, &entry.id)?;
                if let Some(path) = path {
                    ensure!(fs::symlink_metadata(&path)?.is_file(), "source is not regular file");
                    let mut original = HashWriter::default();
                    std::io::copy(&mut File::open(path)?, &mut original)?;
                    let original = original.finish();
                    ensure!(
                        member.bytes == original.bytes && member.sha256 == original.sha256,
                        "source bytes differ: {}",
                        entry.name
                    );
                }
                totals.0 += 1;
                totals.1 += member.bytes;
            }
        }
        Ok(totals)
    }
    walk(store, id, source, 0)
}
