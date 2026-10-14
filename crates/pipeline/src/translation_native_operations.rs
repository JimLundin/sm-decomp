//! Bounded storage for complete native producer attribution. Older receipts
//! retain their inline operation list; new receipts seal a small ordered index.
use super::{NativeOperation, NativeReceipt};
use crate::inputs::{self, bounded};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use translation_execution::{OperationContext, model::sha256, retention::Reference};

const MAX_OPERATIONS: usize = 32_770;
const MAX_CHUNKS: usize = 2_048;
const CHUNK_OPERATIONS: usize = 64;
const MAX_CHUNK_BYTES: u64 = 4 * 1024 * 1024;
const MAX_INDEX_BYTES: u64 = 1024 * 1024;
const MAX_STORED_BYTES: u64 = 256 * 1024 * 1024;

/// Reserve attribution before launching a producer. The fixed allowance covers
/// both compiler records and the index; fixture metadata has a finite per-slot
/// allowance. Storage checks this bound as well as each individual chunk.
pub(super) fn capacity(operations: usize) -> Result<u64> {
    ensure!(operations <= MAX_OPERATIONS, "native operation receipt inventory exceeded");
    let bytes = (operations as u64)
        .checked_mul(32 * 1024)
        .and_then(|bytes| bytes.checked_add(2 * MAX_CHUNK_BYTES + MAX_INDEX_BYTES))
        .context("native operation metadata reservation overflow")?;
    Ok(bytes.min(MAX_STORED_BYTES))
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Index {
    schema: String,
    context: OperationContext,
    prepared_sha256: String,
    candidate_sha256: String,
    operations: usize,
    chunks: Vec<Reference>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Chunk {
    schema: String,
    first: usize,
    operations: Vec<NativeOperation>,
}

fn output(receipt: &NativeReceipt) -> Result<&Path> {
    receipt.verdict.file.parent().and_then(Path::parent).context("native operation output parent missing")
}

fn index_path(receipt: &NativeReceipt) -> Result<PathBuf> {
    Ok(output(receipt)?.join("native-operations/index.json"))
}

fn chunk_path(receipt: &NativeReceipt, number: usize) -> Result<PathBuf> {
    Ok(output(receipt)?.join("native-operations").join(format!("{number:05}.json")))
}

fn read<T: DeserializeOwned>(reference: &Reference, maximum: u64) -> Result<T> {
    ensure!(reference.bytes > 0 && reference.bytes <= maximum, "native operation storage extent invalid");
    let bytes = bounded::read_limit(&reference.path, maximum)?;
    ensure!(
        bytes.len() as u64 == reference.bytes && sha256(&bytes) == reference.sha256,
        "native operation storage identity changed: {}",
        reference.path.display()
    );
    serde_json::from_slice(&bytes).context("native operation storage is invalid")
}

fn publish(path: &Path, bytes: &[u8], maximum: u64) -> Result<Reference> {
    ensure!(!bytes.is_empty() && bytes.len() as u64 <= maximum, "native operation storage bound exceeded");
    bounded::safe(path)?;
    fs::create_dir_all(path.parent().context("native operation storage parent missing")?)?;
    inputs::immutable_bytes(path, bytes)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o400))?;
    Ok(Reference { path: path.into(), sha256: sha256(bytes), bytes: bytes.len() as u64 })
}

pub(super) fn store(receipt: &NativeReceipt, directory: &Path) -> Result<Reference> {
    ensure!(directory == output(receipt)?, "native operation storage redirected outside its receipt");
    ensure!(receipt.operation_index.is_none(), "native operation storage is already indexed");
    ensure!(receipt.operations.len() <= MAX_OPERATIONS, "native operation receipt inventory exceeded");
    let mut chunks = Vec::new();
    let mut first = 0;
    let mut total = 0u64;
    let maximum = capacity(receipt.operations.len())?;
    while first < receipt.operations.len() {
        bounded::check()?;
        ensure!(chunks.len() < MAX_CHUNKS, "native operation chunk inventory exceeded");
        let mut end = (first + CHUNK_OPERATIONS).min(receipt.operations.len());
        let bytes = loop {
            #[derive(Serialize)]
            struct PublishedChunk<'a> {
                schema: &'static str,
                first: usize,
                operations: &'a [NativeOperation],
            }
            let bytes = serde_json::to_vec(&PublishedChunk {
                schema: "translation-native-operation-chunk-v1",
                first,
                operations: &receipt.operations[first..end],
            })?;
            if bytes.len() as u64 <= MAX_CHUNK_BYTES {
                break bytes;
            }
            ensure!(end - first > 1, "one native operation exceeds bounded storage");
            end = first + (end - first) / 2;
        };
        total = total.checked_add(bytes.len() as u64).context("native operation storage overflow")?;
        ensure!(total <= maximum - MAX_INDEX_BYTES, "native operation metadata aggregate exceeded");
        chunks.push(publish(&chunk_path(receipt, chunks.len())?, &bytes, MAX_CHUNK_BYTES)?);
        first = end;
    }
    let index = Index {
        schema: "translation-native-operation-index-v1".into(),
        context: receipt.context.clone(),
        prepared_sha256: receipt.prepared_sha256.clone(),
        candidate_sha256: receipt.candidate_sha256.clone(),
        operations: receipt.operations.len(),
        chunks,
    };
    publish(&index_path(receipt)?, &serde_json::to_vec(&index)?, MAX_INDEX_BYTES)
}

fn index(receipt: &NativeReceipt) -> Result<Index> {
    let reference = receipt.operation_index.as_ref().context("native operation index missing")?;
    ensure!(
        receipt.operations.is_empty() && reference.path == index_path(receipt)?,
        "native receipt has conflicting or redirected producer attribution"
    );
    let index: Index = read(reference, MAX_INDEX_BYTES)?;
    ensure!(
        index.schema == "translation-native-operation-index-v1"
            && index.context == receipt.context
            && index.prepared_sha256 == receipt.prepared_sha256
            && index.candidate_sha256 == receipt.candidate_sha256
            && index.operations <= MAX_OPERATIONS
            && index.chunks.len() <= MAX_CHUNKS
            && index.chunks.len() <= index.operations
            && index.chunks.is_empty() == (index.operations == 0),
        "native operation index changed its source, context or inventory"
    );
    let mut total = reference.bytes;
    let maximum = capacity(index.operations)?;
    for (number, chunk) in index.chunks.iter().enumerate() {
        ensure!(
            chunk.path == chunk_path(receipt, number)? && chunk.bytes > 0 && chunk.bytes <= MAX_CHUNK_BYTES,
            "native operation chunk redirected or oversized"
        );
        total = total.checked_add(chunk.bytes).context("native operation metadata extent overflow")?;
        ensure!(total <= maximum, "native operation metadata aggregate exceeded");
    }
    Ok(index)
}

pub(super) fn visit(receipt: &NativeReceipt, mut visit: impl FnMut(&NativeOperation) -> Result<()>) -> Result<()> {
    let index = index(receipt)?;
    let mut observed = 0usize;
    for reference in &index.chunks {
        bounded::check()?;
        let chunk: Chunk = read(reference, MAX_CHUNK_BYTES)?;
        ensure!(
            chunk.schema == "translation-native-operation-chunk-v1"
                && chunk.first == observed
                && !chunk.operations.is_empty()
                && chunk.operations.len() <= CHUNK_OPERATIONS,
            "native operation chunk changed ordering or extent"
        );
        for operation in &chunk.operations {
            observed = observed.checked_add(1).context("native operation inventory overflow")?;
            ensure!(observed <= index.operations, "native operation index omits producer records");
            visit(operation)?;
        }
    }
    ensure!(observed == index.operations, "native operation index is incomplete");
    Ok(())
}

pub(super) fn references(receipt: &NativeReceipt) -> Result<Vec<Reference>> {
    let index = index(receipt)?;
    // Authenticate every chunk before handing storage dependencies to retention.
    // Producer authentication remains the caller's separate responsibility.
    visit(receipt, |_| Ok(()))?;
    let mut references = vec![receipt.operation_index.as_ref().unwrap().clone()];
    references.extend(index.chunks);
    Ok(references)
}
