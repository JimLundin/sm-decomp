use crate::canonical::{Id, MAX_OBJECT, digest, hex, identity};
use anyhow::{Context, Result, ensure};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

type BlockKey = (PathBuf, u64, Id, u32, u32);
type BlockCache = Vec<(BlockKey, Vec<u8>)>;

const HEADER: &[u8; 8] = b"SMEPACK1";
const INDEX_MAGIC: &[u8; 8] = b"SMEINDEX";
const FOOTER: &[u8; 8] = b"SMEEND01";
const BLOCK_TARGET: usize = 1 << 20;
const MAX_BLOCK: usize = 4 << 20;
const MAX_INDEX: usize = 256 << 20;

#[derive(Clone)]
struct Location {
    pack: PathBuf,
    offset: u64,
    packed_len: u32,
    raw_len: u32,
    block_digest: Id,
    position: u32,
    length: u32,
    kind: u8,
}

#[derive(Clone, Default, Debug)]
pub struct StoreStats {
    pub objects: u64,
    pub packs: u64,
    pub stored_bytes: u64,
    pub new_objects: u64,
    pub shared_objects: u64,
}

struct Pending {
    id: Id,
    kind: u8,
    logical: Vec<u8>,
    physical: Vec<u8>,
    compact: bool,
}

/// One writer per store; read-only opens never create or mutate files.
/// An interrupted writer leaves its lock and unsealed `.pending` file. Remove
/// these only after confirming that writer has exited; sealed packs remain usable.
pub struct Store {
    root: PathBuf,
    owns_lock: bool,
    authenticated: RefCell<HashSet<Id>>,
    objects: HashMap<Id, Location>,
    ordinals: HashMap<Id, u32>,
    ids: Vec<Id>,
    pending: Vec<Pending>,
    pending_bytes: usize,
    pending_logical: HashMap<Id, (u8, Vec<u8>)>,
    writer: Option<File>,
    pending_path: Option<PathBuf>,
    entries: Vec<(Id, Location)>,
    cache: RefCell<BlockCache>,
    stats: StoreStats,
}

impl Store {
    /// Re-encode canonical objects in dependency order, remapping compact ordinals.
    /// Independent sealed packs must never be copied between store directories.
    pub fn import_store(&mut self, source: &Store) -> Result<()> {
        for id in &source.ids {
            let (kind, logical) = source.get(id)?;
            ensure!(self.put(kind, &logical)? == *id, "imported canonical identity differs");
        }
        Ok(())
    }

    pub fn create(root: &Path) -> Result<Self> {
        if let Ok(meta) = root.symlink_metadata() {
            ensure!(!meta.file_type().is_symlink(), "store symlink rejected");
        }
        fs::create_dir_all(root)?;
        let lock = root.join("writer.lock");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock)
            .with_context(|| format!("store already has a writer: {}", lock.display()))?;
        let result = (|| {
            writeln!(file, "{}", std::process::id())?;
            file.sync_all()?;
            Self::load(root, true)
        })();
        if result.is_err() {
            let _ = fs::remove_file(lock);
        }
        result
    }

    pub fn open(root: &Path) -> Result<Self> {
        Self::load(root, false)
    }

    fn load(root: &Path, writable: bool) -> Result<Self> {
        ensure!(root.is_dir() && !root.symlink_metadata()?.file_type().is_symlink(), "invalid store directory");
        let mut paths: Vec<_> = fs::read_dir(root)?.map(|r| r.map(|e| e.path())).collect::<std::io::Result<_>>()?;
        paths.retain(|p| p.extension().is_some_and(|e| e == "pack"));
        paths.sort();
        let mut store = Self {
            root: root.to_owned(),
            owns_lock: false,
            authenticated: RefCell::new(HashSet::new()),
            objects: HashMap::new(),
            ordinals: HashMap::new(),
            ids: Vec::new(),
            pending: Vec::new(),
            pending_bytes: 0,
            pending_logical: HashMap::new(),
            writer: None,
            pending_path: None,
            entries: Vec::new(),
            cache: RefCell::new(Vec::new()),
            stats: StoreStats::default(),
        };
        for path in paths {
            store.load_index(&path)?;
        }
        store.stats.objects = store.objects.len() as u64;
        if writable {
            let name = format!("{:016x}-{}.pending", store.stats.packs, std::process::id());
            let path = root.join(name);
            let mut file = OpenOptions::new().read(true).write(true).create_new(true).open(&path)?;
            file.write_all(HEADER)?;
            store.writer = Some(file);
            store.pending_path = Some(path);
            store.owns_lock = true;
        }
        Ok(store)
    }

    fn load_index(&mut self, path: &Path) -> Result<()> {
        ensure!(!path.symlink_metadata()?.file_type().is_symlink(), "pack symlink rejected");
        let mut file = File::open(path)?;
        let length = file.metadata()?.len();
        ensure!(length >= 8 + 80, "truncated pack");
        let mut magic = [0; 8];
        file.read_exact(&mut magic)?;
        ensure!(&magic == HEADER, "unknown pack version");
        file.seek(SeekFrom::End(-80))?;
        let mut footer = [0; 80];
        file.read_exact(&mut footer)?;
        ensure!(&footer[72..] == FOOTER, "missing pack completion footer");
        let offset = u64::from_le_bytes(footer[..8].try_into()?);
        ensure!(offset >= 8 && offset <= length - 80, "invalid index offset");
        let index_len = usize::try_from(length - 80 - offset)?;
        ensure!(index_len <= MAX_INDEX, "pack index exceeds limit");
        file.seek(SeekFrom::Start(offset))?;
        let mut index = vec![0; index_len];
        file.read_exact(&mut index)?;
        ensure!(digest(&index) == footer[8..40], "pack index seal differs");
        ensure!(index.starts_with(INDEX_MAGIC) && index.len() >= 24, "invalid pack index");
        let start = u64::from_le_bytes(index[8..16].try_into()?);
        let count = u64::from_le_bytes(index[16..24].try_into()?);
        ensure!(
            count <= u32::MAX as u64
                && index.len()
                    == 24usize
                        .checked_add(usize::try_from(count)?.checked_mul(89).context("index overflow")?)
                        .context("index overflow")?,
            "invalid index length"
        );
        ensure!(start == self.ids.len() as u64, "pack ordinal sequence differs");
        for chunk in index[24..].as_chunks::<89>().0 {
            let id: Id = chunk[..32].try_into()?;
            let kind = chunk[32];
            let location = Location {
                pack: path.to_owned(),
                kind,
                offset: u64::from_le_bytes(chunk[33..41].try_into()?),
                packed_len: u32::from_le_bytes(chunk[41..45].try_into()?),
                raw_len: u32::from_le_bytes(chunk[45..49].try_into()?),
                block_digest: chunk[49..81].try_into()?,
                position: u32::from_le_bytes(chunk[81..85].try_into()?),
                length: u32::from_le_bytes(chunk[85..89].try_into()?),
            };
            ensure!(
                location.offset >= 8
                    && location.offset.checked_add(location.packed_len as u64).is_some_and(|end| end <= offset),
                "block outside pack body"
            );
            ensure!(
                location.packed_len as usize <= MAX_BLOCK + 65536
                    && location.raw_len as usize <= MAX_BLOCK
                    && location.length > 0
                    && location.length as usize <= MAX_OBJECT + 1
                    && location.position.checked_add(location.length).is_some_and(|end| end <= location.raw_len),
                "invalid object bounds"
            );
            ensure!(!self.objects.contains_key(&id), "duplicate object index");
            self.ordinals.insert(id, u32::try_from(self.ids.len())?);
            self.ids.push(id);
            self.objects.insert(id, location);
        }
        self.stats.packs += 1;
        self.stats.stored_bytes += length;
        Ok(())
    }

    /// Identities use canonical kind/version/payload, regardless of compression.
    pub fn put(&mut self, kind: u8, payload: &[u8]) -> Result<Id> {
        ensure!(self.writer.is_some(), "read-only store");
        ensure!(payload.len() <= MAX_OBJECT, "object exceeds size limit; split large blobs");
        let id = identity(kind, payload);
        if self.objects.contains_key(&id) || self.pending_logical.contains_key(&id) {
            if !self.authenticated.borrow().contains(&id) {
                let (stored_kind, stored) = self.get(&id)?;
                ensure!(stored_kind == kind && stored == payload, "existing object differs");
            }
            self.stats.shared_objects += 1;
            return Ok(id);
        }
        let (physical, compact) = self.compact(kind, payload)?;
        if self.pending_bytes + physical.len() + 1 > BLOCK_TARGET && !self.pending.is_empty() {
            self.flush()?;
        }
        self.ordinals.insert(id, u32::try_from(self.ids.len())?);
        self.ids.push(id);
        self.pending_bytes += physical.len() + 1;
        self.pending_logical.insert(id, (kind, payload.to_vec()));
        self.pending.push(Pending { id, kind, logical: payload.to_vec(), physical, compact });
        self.authenticated.borrow_mut().insert(id);
        self.stats.new_objects += 1;
        Ok(id)
    }

    fn compact(&self, kind: u8, payload: &[u8]) -> Result<(Vec<u8>, bool)> {
        if kind == crate::NODE {
            ensure!(payload.len().is_multiple_of(32), "invalid node");
            let mut bytes = Vec::with_capacity(payload.len() / 8);
            for chunk in payload.as_chunks::<32>().0 {
                let id: Id = *chunk;
                let ordinal = self.ordinals.get(&id).context("node references missing object")?;
                bytes.extend_from_slice(&ordinal.to_le_bytes());
            }
            Ok((bytes, true))
        } else {
            Ok((payload.to_vec(), false))
        }
    }

    fn flush(&mut self) -> Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let mut raw = Vec::with_capacity(self.pending_bytes);
        let mut positions = Vec::new();
        for object in &self.pending {
            let position = raw.len();
            raw.push(u8::from(object.compact));
            raw.extend_from_slice(&object.physical);
            positions.push((position as u32, (object.physical.len() + 1) as u32));
        }
        let compressed = zstd::bulk::compress(&raw, 3)?;
        let file = self.writer.as_mut().context("read-only store")?;
        let offset = file.stream_position()?;
        file.write_all(&compressed)?;
        let block_digest = digest(&raw);
        for (object, (position, length)) in self.pending.drain(..).zip(positions) {
            let location = Location {
                pack: self.pending_path.clone().unwrap(),
                offset,
                packed_len: compressed.len() as u32,
                raw_len: raw.len() as u32,
                block_digest,
                position,
                length,
                kind: object.kind,
            };
            debug_assert_eq!(identity(object.kind, &object.logical), object.id);
            self.objects.insert(object.id, location.clone());
            self.entries.push((object.id, location));
        }
        self.pending_bytes = 0;
        self.pending_logical.clear();
        Ok(())
    }

    pub fn get(&self, id: &Id) -> Result<(u8, Vec<u8>)> {
        if let Some(object) = self.pending_logical.get(id) {
            return Ok(object.clone());
        }
        let location = self.objects.get(id).with_context(|| format!("missing object {}", hex(id)))?;
        let key =
            (location.pack.clone(), location.offset, location.block_digest, location.raw_len, location.packed_len);
        let mut cache = self.cache.borrow_mut();
        let index = cache.iter().position(|(k, _)| k == &key);
        let raw = if let Some(index) = index {
            cache.remove(index).1
        } else {
            let mut file = File::open(&location.pack)?;
            file.seek(SeekFrom::Start(location.offset))?;
            let mut compressed = vec![0; location.packed_len as usize];
            file.read_exact(&mut compressed)?;
            let raw = zstd::bulk::decompress(&compressed, location.raw_len as usize)?;
            ensure!(
                raw.len() == location.raw_len as usize && digest(&raw) == location.block_digest,
                "pack block corrupt"
            );
            raw
        };
        let bytes = &raw[location.position as usize..(location.position + location.length) as usize];
        let compact = bytes[0];
        let payload = if compact == 0 {
            bytes[1..].to_vec()
        } else {
            ensure!(
                compact == 1 && location.kind == crate::NODE && (bytes.len() - 1) % 4 == 0,
                "unknown physical object encoding"
            );
            ensure!((bytes.len() - 1) / 4 <= MAX_OBJECT / 32, "expanded object exceeds limit");
            let mut payload = Vec::new();
            for ordinal in bytes[1..].as_chunks::<4>().0 {
                payload.extend_from_slice(
                    self.ids.get(u32::from_le_bytes(*ordinal) as usize).context("invalid object ordinal")?,
                );
            }
            payload
        };
        ensure!(identity(location.kind, &payload) == *id, "object identity differs");
        self.authenticated.borrow_mut().insert(*id);
        cache.push((key, raw));
        if cache.len() > 8 {
            cache.remove(0);
        }
        Ok((location.kind, payload))
    }

    pub(crate) fn get_kind(&self, id: &Id, kind: u8) -> Result<Vec<u8>> {
        let (actual, payload) = self.get(id)?;
        ensure!(actual == kind, "object kind differs");
        Ok(payload)
    }

    /// Publishes one immutable pack after its body/index/footer are durable.
    /// Returns the SHA-256 of the stored pack bytes; content ids stay unchanged.
    pub fn commit(mut self) -> Result<Option<Id>> {
        self.flush()?;
        let mut file = self.writer.take().context("read-only store")?;
        if self.entries.is_empty() {
            drop(file);
            fs::remove_file(self.pending_path.take().unwrap())?;
            return Ok(None);
        }
        let offset = file.stream_position()?;
        let mut index = INDEX_MAGIC.to_vec();
        index.extend_from_slice(&(self.ids.len() as u64 - self.entries.len() as u64).to_le_bytes());
        index.extend_from_slice(&(self.entries.len() as u64).to_le_bytes());
        for (id, location) in &self.entries {
            index.extend_from_slice(id);
            index.push(location.kind);
            index.extend_from_slice(&location.offset.to_le_bytes());
            index.extend_from_slice(&location.packed_len.to_le_bytes());
            index.extend_from_slice(&location.raw_len.to_le_bytes());
            index.extend_from_slice(&location.block_digest);
            index.extend_from_slice(&location.position.to_le_bytes());
            index.extend_from_slice(&location.length.to_le_bytes());
        }
        ensure!(index.len() <= MAX_INDEX, "index exceeds pack limit");
        file.write_all(&index)?;
        file.flush()?;
        file.seek(SeekFrom::Start(0))?;
        let mut hash = sha2::Sha256::new();
        use sha2::Digest;
        let mut buffer = [0; 65536];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hash.update(&buffer[..n]);
        }
        let body: Id = hash.finalize().into();
        file.seek(SeekFrom::End(0))?;
        file.write_all(&offset.to_le_bytes())?;
        file.write_all(&digest(&index))?;
        file.write_all(&body)?;
        file.write_all(FOOTER)?;
        file.sync_all()?;
        file.seek(SeekFrom::Start(0))?;
        let mut hash = sha2::Sha256::new();
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hash.update(&buffer[..n]);
        }
        let seal: Id = hash.finalize().into();
        drop(file);
        let target = self.root.join(format!("{:016x}-{}.pack", self.stats.packs, hex(&seal)));
        fs::rename(self.pending_path.as_ref().context("missing pending path")?, target)?;
        self.pending_path.take();
        File::open(&self.root)?.sync_all()?;
        Ok(Some(seal))
    }

    pub fn stats(&self) -> StoreStats {
        let mut stats = self.stats.clone();
        stats.objects = self.ids.len() as u64;
        stats
    }

    pub fn verify_packs(&self) -> Result<()> {
        let mut paths: Vec<_> = self.objects.values().map(|l| l.pack.clone()).collect();
        paths.sort();
        paths.dedup();
        use sha2::Digest;
        for path in paths {
            let mut file = File::open(&path)?;
            let size = file.metadata()?.len();
            let mut hash = sha2::Sha256::new();
            let mut remaining = size - 80;
            let mut buffer = [0; 65536];
            while remaining > 0 {
                let limit = remaining.min(buffer.len() as u64) as usize;
                let n = file.read(&mut buffer[..limit])?;
                ensure!(n > 0, "truncated pack");
                hash.update(&buffer[..n]);
                remaining -= n as u64;
            }
            let mut footer = [0; 80];
            file.read_exact(&mut footer)?;
            let actual: Id = hash.clone().finalize().into();
            ensure!(actual == footer[40..72], "physical pack seal differs");
            hash.update(footer);
            let full: Id = hash.finalize().into();
            let name = path.file_stem().and_then(|n| n.to_str()).context("invalid pack name")?;
            ensure!(name.get(17..) == Some(hex(&full).as_str()), "pack filename seal differs");
        }
        Ok(())
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        if self.writer.is_some() || self.pending_path.is_some() {
            self.writer.take();
            if let Some(path) = self.pending_path.take() {
                let _ = fs::remove_file(path);
            }
        }
        if self.owns_lock {
            let _ = fs::remove_file(self.root.join("writer.lock"));
        }
    }
}
