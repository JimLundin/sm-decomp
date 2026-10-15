//! Authoritative binary Offline families. See docs/FIXTURE_FORMAT.md.
//! All byte extents are validated before shared, allocation-free span access.
use crate::family_model::*;
use anyhow::{Context, Result, ensure};
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tool_execution::bounded;

const MAGIC: &[u8; 8] = b"SMFIXT01";
const HEADER: usize = 64;
const WRAM: usize = 0x20000;
pub const EXTENSION: &str = "smf";
pub const RUN_MANIFEST: &str = "manifest.smf";

// Metadata is typed and little endian. Its schema changes with the magic/version;
// there is deliberately no runtime legacy TOML-fixture decoder.
trait Wire: Sized {
    const MIN: usize;
    fn put(&self, out: &mut Vec<u8>) -> Result<()>;
    fn get(input: &mut Reader<'_>) -> Result<Self>;
}
struct Reader<'a> {
    bytes: &'a [u8],
    items: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, len: usize) -> Result<&'a [u8]> {
        ensure!(len <= self.bytes.len(), "truncated fixture metadata");
        let (value, rest) = self.bytes.split_at(len);
        self.bytes = rest;
        Ok(value)
    }
    fn count<T: Wire>(&mut self) -> Result<usize> {
        let count = u32::get(self)? as usize;
        ensure!(count <= self.items && count <= self.bytes.len() / T::MIN.max(1), "fixture metadata count exceeded");
        self.items -= count;
        bounded::check()?;
        Ok(count)
    }
}
macro_rules! integer {
    ($($ty:ty),*) => {$ (
        impl Wire for $ty {
            const MIN: usize = std::mem::size_of::<Self>();
            fn put(&self, out: &mut Vec<u8>) -> Result<()> { out.extend_from_slice(&self.to_le_bytes()); Ok(()) }
            fn get(input: &mut Reader<'_>) -> Result<Self> { Ok(Self::from_le_bytes(input.take(<Self as Wire>::MIN)?.try_into()?)) }
        }
    )*};
}
integer!(u8, u16, u32, u64, i64);
impl Wire for usize {
    const MIN: usize = 8;
    fn put(&self, out: &mut Vec<u8>) -> Result<()> {
        (*self as u64).put(out)
    }
    fn get(input: &mut Reader<'_>) -> Result<Self> {
        Ok(Self::try_from(u64::get(input)?)?)
    }
}
impl Wire for bool {
    const MIN: usize = 1;
    fn put(&self, out: &mut Vec<u8>) -> Result<()> {
        u8::from(*self).put(out)
    }
    fn get(input: &mut Reader<'_>) -> Result<Self> {
        let byte = u8::get(input)?;
        ensure!(byte <= 1, "invalid fixture boolean");
        Ok(byte == 1)
    }
}
impl Wire for String {
    const MIN: usize = 4;
    fn put(&self, out: &mut Vec<u8>) -> Result<()> {
        u32::try_from(self.len())?.put(out)?;
        out.extend_from_slice(self.as_bytes());
        Ok(())
    }
    fn get(input: &mut Reader<'_>) -> Result<Self> {
        let len = u32::get(input)? as usize;
        Ok(std::str::from_utf8(input.take(len)?)?.into())
    }
}
impl Wire for PathBuf {
    const MIN: usize = String::MIN;
    fn put(&self, out: &mut Vec<u8>) -> Result<()> {
        self.to_str().context("fixture path is not UTF-8")?.to_owned().put(out)
    }
    fn get(input: &mut Reader<'_>) -> Result<Self> {
        Ok(String::get(input)?.into())
    }
}
impl<T: Wire> Wire for Option<T> {
    const MIN: usize = 1;
    fn put(&self, out: &mut Vec<u8>) -> Result<()> {
        self.is_some().put(out)?;
        if let Some(value) = self {
            value.put(out)?;
        }
        Ok(())
    }
    fn get(input: &mut Reader<'_>) -> Result<Self> {
        if bool::get(input)? { Ok(Some(T::get(input)?)) } else { Ok(None) }
    }
}
impl<T: Wire> Wire for Vec<T> {
    const MIN: usize = 4;
    fn put(&self, out: &mut Vec<u8>) -> Result<()> {
        u32::try_from(self.len())?.put(out)?;
        for value in self {
            value.put(out)?;
        }
        Ok(())
    }
    fn get(input: &mut Reader<'_>) -> Result<Self> {
        let count = input.count::<T>()?;
        (0..count).map(|_| T::get(input)).collect()
    }
}
impl Wire for BTreeMap<String, String> {
    const MIN: usize = 4;
    fn put(&self, out: &mut Vec<u8>) -> Result<()> {
        u32::try_from(self.len())?.put(out)?;
        for (key, value) in self {
            key.put(out)?;
            value.put(out)?;
        }
        Ok(())
    }
    fn get(input: &mut Reader<'_>) -> Result<Self> {
        let count = input.count::<String>()?;
        let mut out = Self::new();
        for _ in 0..count {
            let key = String::get(input)?;
            ensure!(out.insert(key, String::get(input)?).is_none(), "duplicate fixture metadata key");
        }
        Ok(out)
    }
}
macro_rules! record {
    ($ty:ident { $($field:ident : $kind:ty),* $(,)? }) => {
        impl Wire for $ty {
            const MIN: usize = 0 $(+ <$kind as Wire>::MIN)*;
            fn put(&self, out: &mut Vec<u8>) -> Result<()> { $(self.$field.put(out)?;)* Ok(()) }
            fn get(input: &mut Reader<'_>) -> Result<Self> {
                Ok(Self { $($field: <$kind as Wire>::get(input)?,)* ..Default::default() })
            }
        }
    };
}
record!(Asset { file: PathBuf, sha256: String });
record!(Case {
    name: String, a: i64, b: i64, fill: Option<u8>, host_words: Option<Vec<u16>>,
    continue_from: Option<String>, expected_return: Option<u64>, expected_amount: Option<i64>,
    expected_flags: Option<i64>, minimum_callbacks: u64, allocator: Option<String>,
    before_file: Option<PathBuf>, before_sha256: Option<String>, expected_after_file: Option<PathBuf>,
    expected_after_sha256: Option<String>,
});
record!(Routine {
    symbol: String, args: usize, arg_types: Option<Vec<String>>, returns: String, return_type: Option<String>,
    domain: Option<String>, cases: Vec<Case>, live_eligible: Option<bool>, live_limitation: Option<String>,
    pointer_bank: Option<u64>, packed_u16_args: Option<u64>, packed_args: Option<String>,
    pair_u16_return: bool, original_address: Option<u64>,
});
record!(Manifest {
    family: String, domain: Option<String>, rust_module: String, bank: String, c_support: Vec<String>,
    rust_imports: Vec<String>, rust_dependencies: Vec<PathBuf>, rom: Option<Asset>, routines: Vec<Routine>,
    cases: Vec<Case>, projectile_host: bool, projectile_rendering: bool, case_order: Option<Vec<String>>,
    infrastructure_probe: bool, callback_adapter: Option<String>, plm_amount_context: Option<String>,
    admitted_domains: BTreeMap<String, String>, source: Option<String>, license: Option<PathBuf>, scope: Option<String>,
});

fn number(bytes: &[u8], at: usize) -> Result<usize> {
    let raw =
        bytes.get(at..at.checked_add(8).context("fixture offset overflow")?).context("truncated fixture header")?;
    Ok(usize::try_from(u64::from_le_bytes(raw.try_into()?))?)
}
fn offset(base: usize, count: usize, stride: usize) -> Result<usize> {
    base.checked_add(count.checked_mul(stride).context("fixture count overflow")?).context("fixture extent overflow")
}

pub fn read(path: &Path) -> Result<Manifest> {
    open(path, None)
}

pub fn read_authenticated(path: &Path, expected: &str) -> Result<Manifest> {
    open(path, Some(expected))
}

fn open(path: &Path, expected: Option<&str>) -> Result<Manifest> {
    let path = if path.is_absolute() { path.to_path_buf() } else { std::env::current_dir()?.join(path) };
    decomp_source::context::limited(|| {
        if let Some(declarations) = crate::declarations::current() {
            return Ok((*declarations.read(&path, expected)?.manifest).clone());
        }
        let bytes = bounded::read(&path)?;
        if let Some(expected) = expected {
            use sha2::{Digest, Sha256};
            ensure!(format!("{:x}", Sha256::digest(&bytes)) == expected, "binary fixture identity differs");
        }
        decode(bytes)
    })
    .with_context(|| format!("binary fixture {}", path.display()))
}

pub fn decode(bytes: Vec<u8>) -> Result<Manifest> {
    if let Some(declarations) = crate::declarations::current() {
        let hash = translation_execution::model::sha256(&bytes);
        return Ok((*declarations.decode(bytes, hash)?.manifest).clone());
    }
    decode_shared(Arc::new(bytes))
}

/// Retain the caller's immutable container in every packed span. Declaration
/// caches must not copy the payload merely to keep its authenticated bytes.
pub(crate) fn decode_shared(bytes: Arc<Vec<u8>>) -> Result<Manifest> {
    ensure!(bytes.len() >= HEADER && bytes.get(..8) == Some(MAGIC), "expected SMFIXT01 binary fixture");
    ensure!(bytes[40..HEADER].iter().all(|&v| v == 0), "unsupported fixture format flags");
    let metadata_len = number(&bytes, 8)?;
    let cases = number(&bytes, 16)?;
    let records = number(&bytes, 24)?;
    let payload_len = number(&bytes, 32)?;
    let index = offset(HEADER, metadata_len, 1)?;
    let record_start = offset(index, cases, 16)?;
    let payload = offset(record_start, records, 16)?;
    ensure!(offset(payload, payload_len, 1)? == bytes.len(), "fixture section extents differ");
    ensure!(cases <= 1_000_000 && records <= 8_000_000, "fixture record inventory exceeded");
    let mut reader = Reader { bytes: &bytes[HEADER..index], items: 4_000_000 };
    let mut manifest = Manifest::get(&mut reader)?;
    ensure!(reader.bytes.is_empty(), "trailing fixture metadata");
    ensure!(
        manifest.cases.len() + manifest.routines.iter().map(|r| r.cases.len()).sum::<usize>() == cases,
        "fixture case index differs from metadata"
    );
    let mut next = 0;
    for i in 0..cases {
        bounded::check()?;
        let first = number(&bytes, index + i * 16)?;
        let count = number(&bytes, index + i * 16 + 8)?;
        ensure!(first == next && count <= records - next, "fixture write index differs");
        let mut end = 0;
        for record in first..first + count {
            let at = record_start + record * 16;
            let address = u32::from_le_bytes(bytes[at..at + 4].try_into()?) as usize;
            let len = u32::from_le_bytes(bytes[at + 4..at + 8].try_into()?) as usize;
            let blob = number(&bytes, at + 8)?;
            ensure!(
                len > 0 && address >= end && address <= WRAM && len <= WRAM - address,
                "overlapping or out-of-range fixture span"
            );
            ensure!(blob <= payload_len && len <= payload_len - blob, "fixture span outside payload");
            end = address + len;
        }
        next += count;
    }
    ensure!(next == records, "unreferenced fixture write records");
    for (i, case) in
        manifest.cases.iter_mut().chain(manifest.routines.iter_mut().flat_map(|r| &mut r.cases)).enumerate()
    {
        let first = number(&bytes, index + i * 16)?;
        let count = number(&bytes, index + i * 16 + 8)?;
        case.writes = Writes::packed(bytes.clone(), record_start + first * 16, count, payload);
        validate_input(case)?;
    }
    Ok(manifest)
}

/// A conservative allocation reservation made before decoding untrusted
/// metadata. Packed payloads are shared; strings, records and vector capacity
/// are bounded by their wire representation plus fixed container overhead.
pub(crate) fn retained_bound(bytes: &[u8]) -> Result<u64> {
    ensure!(bytes.len() >= HEADER, "truncated fixture header");
    let metadata = u64::try_from(number(bytes, 8)?)?;
    ensure!(metadata <= bytes.len() as u64 - HEADER as u64, "fixture metadata extent differs");
    (bytes.len() as u64)
        .checked_add(metadata.checked_mul(24).context("fixture memory reservation overflow")?)
        .and_then(|amount| amount.checked_add(4096))
        .context("fixture memory reservation overflow")
}

fn validate_input(case: &Case) -> Result<()> {
    ensure!(
        case.continue_from.is_none() || case.before_file.is_none() && case.fill.is_none(),
        "continuation cannot replace inherited state"
    );
    ensure!(
        case.before_file.is_none() || case.writes.is_empty() && case.fill.is_none(),
        "captured before_file cannot be patched"
    );
    ensure!(case.before_file.is_some() == case.before_sha256.is_some(), "captured before identity incomplete");
    ensure!(
        case.expected_after_file.is_some() == case.expected_after_sha256.is_some(),
        "captured after identity incomplete"
    );
    Ok(())
}

fn spans(case: &Case, state: &mut [u8], touched: &mut [bool]) -> Result<Vec<Write>> {
    validate_input(case)?;
    let fill = case.fill.unwrap_or(0);
    state.fill(fill);
    touched.fill(false);
    for write in &case.writes {
        ensure!(write.offset <= WRAM && write.data.len() <= WRAM - write.offset, "fixture write outside WRAM");
        let end = write.offset + write.data.len();
        state[write.offset..end].copy_from_slice(write.data);
        touched[write.offset..end].fill(true);
    }
    let continuation = case.continue_from.is_some();
    let keep = |at: usize| if continuation { touched[at] } else { state[at] != fill };
    let maximum_gap = if continuation { 0 } else { 16 };
    let mut writes = Vec::new();
    let mut at = 0;
    while at < WRAM {
        if !keep(at) {
            at += 1;
            continue;
        }
        let start = at;
        let mut end = at + 1;
        at += 1;
        while at < WRAM {
            if keep(at) {
                end = at + 1;
            } else if at - end >= maximum_gap {
                break;
            }
            at += 1;
        }
        writes.push(Write { offset: start, data: state[start..end].to_vec() });
    }
    Ok(writes)
}

pub fn encode(manifest: &Manifest) -> Result<Vec<u8>> {
    let mut metadata = Vec::new();
    manifest.put(&mut metadata)?;
    let mut index = Vec::new();
    let mut records = Vec::new();
    let mut payload = Vec::new();
    let mut blobs = HashMap::<Vec<u8>, usize>::new();
    let mut state = vec![0; WRAM];
    let mut touched = vec![false; WRAM];
    for case in manifest.cases.iter().chain(manifest.routines.iter().flat_map(|r| &r.cases)) {
        bounded::check()?;
        let writes = spans(case, &mut state, &mut touched)?;
        (records.len() / 16).put(&mut index)?;
        writes.len().put(&mut index)?;
        for write in writes {
            let blob = *blobs.entry(write.data.clone()).or_insert_with(|| {
                let offset = payload.len();
                payload.extend_from_slice(&write.data);
                offset
            });
            u32::try_from(write.offset)?.put(&mut records)?;
            u32::try_from(write.data.len())?.put(&mut records)?;
            blob.put(&mut records)?;
        }
    }
    let mut out = Vec::with_capacity(HEADER + metadata.len() + index.len() + records.len() + payload.len());
    out.extend_from_slice(MAGIC);
    metadata.len().put(&mut out)?;
    (index.len() / 16).put(&mut out)?;
    (records.len() / 16).put(&mut out)?;
    payload.len().put(&mut out)?;
    out.resize(HEADER, 0);
    for bytes in [metadata, index, records, payload] {
        out.extend_from_slice(&bytes);
    }
    Ok(out)
}

pub fn write(path: &Path, manifest: &Manifest) -> Result<()> {
    let bytes = encode(manifest)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!("smf-{}-{}", std::process::id(), tool_execution::unique_id()));
    let result = (|| -> Result<()> {
        use std::io::Write as _;
        let mut file = fs::OpenOptions::new().create_new(true).write(true).open(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

/// Compact resume identity: payloads already reside in manifest.smf and before.wram.
pub fn case_identity(case: &Case) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut metadata = Vec::new();
    case.put(&mut metadata)?;
    let mut hash = Sha256::new();
    hash.update(metadata);
    for write in &case.writes {
        hash.update((write.offset as u64).to_le_bytes());
        hash.update((write.data.len() as u64).to_le_bytes());
        hash.update(write.data);
    }
    Ok(format!("{:x}", hash.finalize()))
}

pub fn identity(manifest: &Manifest) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut metadata = Vec::new();
    manifest.put(&mut metadata)?;
    let mut hash = Sha256::new();
    hash.update(metadata);
    for case in manifest.cases.iter().chain(manifest.routines.iter().flat_map(|r| &r.cases)) {
        hash.update((case.writes.len() as u64).to_le_bytes());
        for write in &case.writes {
            hash.update((write.offset as u64).to_le_bytes());
            hash.update((write.data.len() as u64).to_le_bytes());
            hash.update(write.data);
        }
    }
    Ok(format!("{:x}", hash.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authenticated_reads_bind_decoded_bytes_with_one_input_budget() -> Result<()> {
        use sha2::{Digest, Sha256};
        let directory = tool_execution::testing::Directory::new()?;
        let path = directory.0.join("family.smf");
        let mut original =
            Manifest { cases: vec![Case { name: "root".into(), a: 5, ..Default::default() }], ..Default::default() };
        let bytes = encode(&original)?;
        let hash = format!("{:x}", Sha256::digest(&bytes));
        fs::write(&path, &bytes)?;
        decomp_source::context::until(bounded::now_unix_ms()? + 10_000, bytes.len() as u64, || {
            assert_eq!(read_authenticated(&path, &hash)?, original);
            assert_eq!(bounded::charged_bytes(), bytes.len() as u64);
            Ok(())
        })?;
        assert!(read_authenticated(&path, &"0".repeat(64)).is_err());
        original.cases[0].a = 9;
        write(&path, &original)?;
        assert!(read_authenticated(&path, &hash).is_err(), "a later operation must reopen the current input");
        assert_eq!(read(&path)?, original);
        Ok(())
    }

    fn prepared(case: &Case, inherited: u8) -> Vec<u8> {
        let mut state = vec![if case.continue_from.is_some() { inherited } else { case.fill.unwrap_or(0) }; WRAM];
        for write in &case.writes {
            state[write.offset..write.offset + write.data.len()].copy_from_slice(write.data);
        }
        state
    }

    #[test]
    fn binary_roundtrip_preserves_overlaps_root_fill_and_inherited_holes() -> Result<()> {
        let patches = vec![
            Write { offset: 19, data: vec![8, 9, 10] },
            Write { offset: 20, data: vec![0] },
            Write { offset: 4, data: vec![0, 3] },
            Write { offset: WRAM - 1, data: vec![255] },
        ];
        let original = Manifest {
            family: "edge-fixtures".into(),
            routines: vec![Routine {
                symbol: "Step".into(),
                cases: vec![
                    Case { name: "root".into(), fill: Some(3), writes: patches.clone().into(), ..Default::default() },
                    Case {
                        name: "child".into(),
                        continue_from: Some("root".into()),
                        writes: patches.into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            case_order: Some(vec!["root".into(), "child".into()]),
            ..Default::default()
        };
        let binary = encode(&original)?;
        let decoded = decode(binary.clone())?;
        assert_eq!(encode(&decoded)?, binary);
        for (old, new) in original.routines[0].cases.iter().zip(&decoded.routines[0].cases) {
            for inherited in [0, 11, 255] {
                assert_eq!(prepared(old, inherited), prepared(new, inherited));
            }
        }
        assert_eq!(decoded.case_order, original.case_order);
        assert_eq!(decoded.routines[0].cases[1].writes.iter().find(|w| w.offset == 19).unwrap().data, &[8, 0, 10]);
        assert_eq!(prepared(&decoded.routines[0].cases[1], 77)[18], 77);
        Ok(())
    }

    #[test]
    fn captured_assets_and_optional_metadata_survive_without_payloads() -> Result<()> {
        let original = Manifest {
            family: "capture".into(),
            admitted_domains: BTreeMap::from([("gameplay".into(), "bounded".into())]),
            cases: vec![Case {
                name: "observed".into(),
                a: i64::MIN,
                b: i64::MAX,
                expected_return: Some(u64::MAX),
                expected_amount: Some(-7),
                expected_flags: Some(3),
                minimum_callbacks: 2,
                allocator: Some("room".into()),
                host_words: Some(vec![0, 65535]),
                before_file: Some("capture/before.wram".into()),
                before_sha256: Some("a".repeat(64)),
                expected_after_file: Some("capture/after.wram".into()),
                expected_after_sha256: Some("b".repeat(64)),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(decode(encode(&original)?)?, original);
        let mut invalid = original.clone();
        invalid.cases[0].writes.push(Write { offset: 0, data: vec![1] });
        assert!(encode(&invalid).is_err());
        invalid.cases[0].writes.clear();
        invalid.cases[0].continue_from = Some("earlier".into());
        assert!(encode(&invalid).is_err());
        Ok(())
    }

    #[test]
    fn malformed_binary_extents_and_text_are_rejected_without_panics() -> Result<()> {
        let original = Manifest {
            cases: vec![Case { writes: vec![Write { offset: 3, data: vec![1, 2] }].into(), ..Default::default() }],
            ..Default::default()
        };
        let binary = encode(&original)?;
        for len in 0..binary.len() {
            assert!(decode(binary[..len].to_vec()).is_err());
        }
        for at in [0, 8, 16, 24, 32, 40] {
            let mut changed = binary.clone();
            changed[at..at + 8].fill(255);
            assert!(decode(changed).is_err());
        }
        let index = HEADER + number(&binary, 8)?;
        let record = index + 16;
        for at in [index, index + 8, record, record + 4, record + 8] {
            let mut changed = binary.clone();
            changed[at..at + 4].fill(255);
            assert!(decode(changed).is_err());
        }
        let mut changed = binary.clone();
        changed.extend([0]);
        assert!(decode(changed).is_err());
        assert!(decode(b"family = 'text'\n".to_vec()).is_err());
        Ok(())
    }

    #[test]
    fn editing_shared_spans_does_not_modify_another_fixture() -> Result<()> {
        let case = Case { writes: vec![Write { offset: 7, data: vec![9] }].into(), ..Default::default() };
        let mut decoded = decode(encode(&Manifest { cases: vec![case.clone(), case], ..Default::default() })?)?;
        decoded.cases[0].writes.iter_mut().next().unwrap().data[0] = 2;
        assert_eq!(prepared(&decoded.cases[0], 0)[7], 2);
        assert_eq!(prepared(&decoded.cases[1], 0)[7], 9);
        Ok(())
    }
}
