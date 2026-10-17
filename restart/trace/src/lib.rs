//! Compact, authenticated reference observations and strict ordered comparison.
//!
//! The caller owns the observation schema and reference identity. This crate
//! hashes exactly the provided WRAM, external state and ordered effect bytes;
//! it does not claim those bytes cover every relevant game/hardware state.

use sha2::{Digest, Sha256};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::Path;

pub const WRAM_BYTES: usize = 128 * 1024;
pub const MAX_TRACE_BYTES: usize = 64 * 1024 * 1024;
const MAX_IDENTITY_BYTES: usize = 16 * 1024;
const MAGIC: &[u8; 8] = b"SMTRACE2";
const FOOTER: &[u8; 8] = b"SMEND001";
const RECORD_BYTES: usize = 8 + 3 * 32;
const FOOTER_BYTES: usize = 8 + 8 + 32;

/// Exact semantic reference identity, independent of candidate source/build.
///
/// Supply immutable source/ROM/host/input identities and the explicit observed
/// byte schema. Canonicalize composite values before constructing these fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    pub reference: String,
    pub rom: String,
    pub host: String,
    pub input: String,
    pub observation: String,
    pub scenario: String,
}

impl Identity {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        self.validate()?;
        let mut bytes = Vec::new();
        for (_, field) in self.fields() {
            if field.len() > MAX_IDENTITY_BYTES || bytes.len() + 4 + field.len() > MAX_IDENTITY_BYTES {
                return Err(Error::InvalidReference("identity exceeds 16 KiB".into()));
            }
            bytes.extend_from_slice(&(field.len() as u32).to_le_bytes());
            bytes.extend_from_slice(field.as_bytes());
        }
        Ok(bytes)
    }

    fn decode(mut bytes: &[u8]) -> Result<Self, Error> {
        let invalid = || Error::InvalidReference("invalid binary identity".into());
        let mut fields = Vec::with_capacity(6);
        for _ in 0..6 {
            if bytes.len() < 4 {
                return Err(invalid());
            }
            let count = u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize;
            bytes = &bytes[4..];
            if count == 0 || count > bytes.len() || count > MAX_IDENTITY_BYTES {
                return Err(invalid());
            }
            fields.push(std::str::from_utf8(&bytes[..count]).map_err(|_| invalid())?.to_owned());
            bytes = &bytes[count..];
        }
        if !bytes.is_empty() {
            return Err(invalid());
        }
        let [reference, rom, host, input, observation, scenario]: [String; 6] =
            fields.try_into().map_err(|_| invalid())?;
        Ok(Self { reference, rom, host, input, observation, scenario })
    }
    fn fields(&self) -> [(&'static str, &str); 6] {
        [
            ("reference", &self.reference),
            ("rom", &self.rom),
            ("host", &self.host),
            ("input", &self.input),
            ("observation", &self.observation),
            ("scenario", &self.scenario),
        ]
    }

    fn validate(&self) -> Result<(), Error> {
        for (field, value) in self.fields() {
            if value.is_empty() {
                return Err(Error::InvalidReference(format!("empty identity field {field}")));
            }
        }
        Ok(())
    }

    fn matches(&self, expected: &Self) -> Result<(), Error> {
        for ((field, actual), (_, expected)) in self.fields().into_iter().zip(expected.fields()) {
            if actual != expected {
                return Err(Error::IdentityMismatch { field, expected: expected.into(), actual: actual.into() });
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordHashes {
    pub ordinal: u64,
    pub wram: [u8; 32],
    pub external: [u8; 32],
    pub effects: [u8; 32],
}

/// An observation validated by an ordered comparison against a reference.
/// Its digests cannot be supplied or changed by callers. A writer can consume
/// it without hashing the same observation bytes a second time.
pub struct ObservedRecord {
    hashes: RecordHashes,
}

impl RecordHashes {
    fn observe(ordinal: u64, wram: &[u8], external: &[u8], effects: &[u8]) -> Result<Self, Error> {
        if wram.len() != WRAM_BYTES {
            return Err(Error::InvalidObservation(format!(
                "step {ordinal}: expected {WRAM_BYTES} WRAM bytes, received {}",
                wram.len()
            )));
        }
        Ok(Self {
            ordinal,
            wram: Sha256::digest(wram).into(),
            external: Sha256::digest(external).into(),
            effects: Sha256::digest(effects).into(),
        })
    }

    fn bytes(self) -> [u8; RECORD_BYTES] {
        let mut bytes = [0; RECORD_BYTES];
        bytes[..8].copy_from_slice(&self.ordinal.to_le_bytes());
        bytes[8..40].copy_from_slice(&self.wram);
        bytes[40..72].copy_from_slice(&self.external);
        bytes[72..104].copy_from_slice(&self.effects);
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            ordinal: u64::from_le_bytes(bytes[..8].try_into().unwrap()),
            wram: bytes[8..40].try_into().unwrap(),
            external: bytes[40..72].try_into().unwrap(),
            effects: bytes[72..104].try_into().unwrap(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Summary {
    pub records: u64,
    /// SHA-256 of the complete file, including its internal checksum.
    pub sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Component {
    Ordinal,
    Wram,
    External,
    Effects,
    MissingRecord,
    ExtraRecord,
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Ordinal => "ordinal",
            Self::Wram => "wram",
            Self::External => "external",
            Self::Effects => "effects",
            Self::MissingRecord => "missing-record",
            Self::ExtraRecord => "extra-record",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Difference {
    /// First expected position, starting at 1 (also for a missing/extra record).
    pub ordinal: u64,
    pub component: Component,
    pub expected: Option<[u8; 32]>,
    pub actual: Option<[u8; 32]>,
    pub expected_ordinal: Option<u64>,
    pub actual_ordinal: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Io(String),
    InvalidReference(String),
    InvalidObservation(String),
    IdentityMismatch { field: &'static str, expected: String, actual: String },
    Mismatch(Box<Difference>),
}

impl Error {
    pub fn difference(&self) -> Option<&Difference> {
        match self {
            Self::Mismatch(difference) => Some(difference),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(detail) => write!(f, "trace I/O: {detail}"),
            Self::InvalidReference(detail) => write!(f, "invalid reference: {detail}"),
            Self::InvalidObservation(detail) => write!(f, "invalid observation: {detail}"),
            Self::IdentityMismatch { field, expected, actual } => {
                write!(f, "reference identity {field}: expected {expected:?}, received {actual:?}")
            }
            Self::Mismatch(difference) => {
                write!(f, "first divergence at step {}: {}", difference.ordinal, difference.component)
            }
        }
    }
}

impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

/// Create a fresh reference; existing paths are never overwritten.
///
/// Errors poison the writer. An unfinished file remains for diagnosis and is
/// rejected by Reference::load. Only finish writes the authenticated footer.
#[must_use = "finish the writer to seal a valid reference"]
pub struct Writer {
    file: BufWriter<File>,
    hash: Sha256,
    records: u64,
    bytes: usize,
    failure: Option<Error>,
}

impl Writer {
    pub fn create(path: impl AsRef<Path>, identity: &Identity) -> Result<Self, Error> {
        identity.validate()?;
        let encoded = identity.encode()?;
        if encoded.len() > MAX_IDENTITY_BYTES {
            return Err(Error::InvalidReference("identity exceeds 16 KiB".into()));
        }
        let file = OpenOptions::new().write(true).create_new(true).open(path)?;
        let mut this = Self { file: BufWriter::new(file), hash: Sha256::new(), records: 0, bytes: 0, failure: None };
        this.write(MAGIC)?;
        this.write(&(encoded.len() as u32).to_le_bytes())?;
        this.write(&encoded)?;
        Ok(this)
    }

    pub fn record(&mut self, ordinal: u64, wram: &[u8], external: &[u8], effects: &[u8]) -> Result<(), Error> {
        self.record_with(ordinal, || RecordHashes::observe(ordinal, wram, external, effects))
    }

    /// Append an already validated observation, retaining writer sequence and
    /// extent checks. Public expected RecordHashes cannot enter this path.
    pub fn record_observed(&mut self, observed: ObservedRecord) -> Result<(), Error> {
        self.record_with(observed.hashes.ordinal, || Ok(observed.hashes))
    }

    fn record_with(
        &mut self,
        ordinal: u64,
        observe: impl FnOnce() -> Result<RecordHashes, Error>,
    ) -> Result<(), Error> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        let result = (|| {
            if ordinal != self.records + 1 {
                return Err(Error::InvalidObservation(format!(
                    "expected ordinal {}, received {ordinal}",
                    self.records + 1
                )));
            }
            if self.bytes + RECORD_BYTES + FOOTER_BYTES > MAX_TRACE_BYTES {
                return Err(Error::InvalidReference("reference exceeds 64 MiB".into()));
            }
            let hashes = observe()?;
            self.write(&hashes.bytes())?;
            self.records += 1;
            Ok(())
        })();
        if let Err(error) = &result {
            self.failure = Some(error.clone());
        }
        result
    }

    pub fn finish(mut self) -> Result<Summary, Error> {
        if let Some(error) = self.failure.take() {
            return Err(error);
        }
        if self.records == 0 {
            return Err(Error::InvalidReference("empty reference".into()));
        }
        self.write(FOOTER)?;
        self.write(&self.records.to_le_bytes())?;
        let checksum: [u8; 32] = self.hash.clone().finalize().into();
        self.write(&checksum)?;
        self.file.flush()?;
        self.file.get_ref().sync_all()?;
        Ok(Summary { records: self.records, sha256: self.hash.finalize().into() })
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.file.write_all(bytes)?;
        self.hash.update(bytes);
        self.bytes += bytes.len();
        Ok(())
    }
}

/// Fully authenticated immutable expected hashes. No later comparison reads
/// the file, so post-load file changes cannot replace expected observations.
pub struct Reference {
    identity: Identity,
    records: Vec<RecordHashes>,
    summary: Summary,
}

impl Reference {
    pub fn load(path: impl AsRef<Path>, expected: &Identity) -> Result<Self, Error> {
        expected.validate()?;
        let file = File::open(path)?;
        if file.metadata()?.len() > MAX_TRACE_BYTES as u64 {
            return Err(Error::InvalidReference("reference exceeds 64 MiB".into()));
        }
        let mut bytes = Vec::new();
        file.take(MAX_TRACE_BYTES as u64 + 1).read_to_end(&mut bytes)?;
        Self::decode(&bytes, expected)
    }

    fn decode(bytes: &[u8], expected: &Identity) -> Result<Self, Error> {
        let invalid = |detail: &str| Error::InvalidReference(detail.into());
        if bytes.len() > MAX_TRACE_BYTES {
            return Err(invalid("reference exceeds 64 MiB"));
        }
        if bytes.len() < 12 + RECORD_BYTES + FOOTER_BYTES {
            return Err(invalid("truncated or empty reference"));
        }
        if &bytes[..8] != MAGIC {
            return Err(invalid("unsupported format magic/version"));
        }
        let footer_at = bytes.len() - FOOTER_BYTES;
        if &bytes[footer_at..footer_at + 8] != FOOTER {
            return Err(invalid("missing footer or trailing data"));
        }
        let checksum_at = bytes.len() - 32;
        let checksum: [u8; 32] = Sha256::digest(&bytes[..checksum_at]).into();
        if bytes[checksum_at..] != checksum {
            return Err(invalid("checksum mismatch"));
        }
        let identity_len = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        if identity_len == 0 || identity_len > MAX_IDENTITY_BYTES || 12 + identity_len > footer_at {
            return Err(invalid("invalid identity length"));
        }
        let identity = Identity::decode(&bytes[12..12 + identity_len])?;
        identity.validate()?;
        identity.matches(expected)?;
        let records_bytes = &bytes[12 + identity_len..footer_at];
        if records_bytes.is_empty() || !records_bytes.len().is_multiple_of(RECORD_BYTES) {
            return Err(invalid("incomplete or empty records"));
        }
        let count = u64::from_le_bytes(bytes[footer_at + 8..footer_at + 16].try_into().unwrap());
        if count != (records_bytes.len() / RECORD_BYTES) as u64 {
            return Err(invalid("record count mismatch"));
        }
        let mut records = Vec::with_capacity(count as usize);
        for (index, record) in records_bytes.as_chunks::<RECORD_BYTES>().0.iter().enumerate() {
            let record = RecordHashes::from_bytes(record);
            if record.ordinal != index as u64 + 1 {
                return Err(invalid("nonconsecutive record ordinal"));
            }
            records.push(record);
        }
        Ok(Self { identity, records, summary: Summary { records: count, sha256: Sha256::digest(bytes).into() } })
    }

    pub fn identity(&self) -> &Identity {
        &self.identity
    }
    pub fn summary(&self) -> Summary {
        self.summary
    }

    /// Inspect an authenticated expected record by its 1-based ordinal.
    pub fn expected(&self, ordinal: u64) -> Option<&RecordHashes> {
        let index = usize::try_from(ordinal.checked_sub(1)?).ok()?;
        self.records.get(index)
    }

    pub fn comparison(&self) -> Comparison<'_> {
        Comparison { reference: self, position: 0, failure: None }
    }
}

/// Candidate observations are checked one at a time in strict sequence.
/// Any failure sticks: callers cannot ignore a difference and later pass.
#[must_use = "finish the comparison to reject missing trailing records"]
pub struct Comparison<'a> {
    reference: &'a Reference,
    position: usize,
    failure: Option<Error>,
}

impl Comparison<'_> {
    pub fn check(&mut self, ordinal: u64, wram: &[u8], external: &[u8], effects: &[u8]) -> Result<(), Error> {
        self.check_observed(ordinal, wram, external, effects).map(|_| ())
    }

    /// Check raw bytes in sequence and return their validated digests for
    /// recording. Sequence errors precede shape validation, as in check.
    pub fn check_observed(
        &mut self,
        ordinal: u64,
        wram: &[u8],
        external: &[u8],
        effects: &[u8],
    ) -> Result<ObservedRecord, Error> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        let result = self.check_next(ordinal, wram, external, effects);
        if let Err(error) = &result {
            self.failure = Some(error.clone());
        }
        result
    }

    fn check_next(
        &mut self,
        ordinal: u64,
        wram: &[u8],
        external: &[u8],
        effects: &[u8],
    ) -> Result<ObservedRecord, Error> {
        let expected = self.reference.records.get(self.position);
        let mut difference = Difference {
            ordinal: self.position as u64 + 1,
            component: Component::ExtraRecord,
            expected: None,
            actual: None,
            expected_ordinal: expected.map(|r| r.ordinal),
            actual_ordinal: Some(ordinal),
        };
        let Some(expected) = expected else {
            return Err(Error::Mismatch(Box::new(difference)));
        };
        if ordinal != expected.ordinal {
            difference.component = Component::Ordinal;
            return Err(Error::Mismatch(Box::new(difference)));
        }
        let actual = RecordHashes::observe(ordinal, wram, external, effects)?;
        for (component, expected, actual) in [
            (Component::Wram, expected.wram, actual.wram),
            (Component::External, expected.external, actual.external),
            (Component::Effects, expected.effects, actual.effects),
        ] {
            if expected != actual {
                difference.component = component;
                difference.expected = Some(expected);
                difference.actual = Some(actual);
                return Err(Error::Mismatch(Box::new(difference)));
            }
        }
        self.position += 1;
        Ok(ObservedRecord { hashes: actual })
    }

    pub fn finish(self) -> Result<Summary, Error> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if let Some(expected) = self.reference.records.get(self.position) {
            return Err(Error::Mismatch(Box::new(Difference {
                ordinal: expected.ordinal,
                component: Component::MissingRecord,
                expected: None,
                actual: None,
                expected_ordinal: Some(expected.ordinal),
                actual_ordinal: None,
            })));
        }
        Ok(self.reference.summary)
    }
}

#[cfg(any(test, feature = "private-checks"))]
mod tests;

#[cfg(feature = "private-checks")]
pub fn private_assertions() -> Vec<(&'static str, fn())> {
    vec![
        (
            "observed_record_preserves_literal_wire_and_full_digests",
            tests::observed_record_preserves_literal_wire_and_full_digests as fn(),
        ),
        (
            "observed_sequence_shape_and_component_failures_stick",
            tests::observed_sequence_shape_and_component_failures_stick as fn(),
        ),
        (
            "observed_writer_retains_sequence_extent_and_io_poisoning",
            tests::observed_writer_retains_sequence_extent_and_io_poisoning as fn(),
        ),
        (
            "binary_identity_has_independent_ordered_length_prefixed_layout",
            tests::binary_identity_has_independent_ordered_length_prefixed_layout as fn(),
        ),
        (
            "compact_roundtrip_preserves_expected_hashes_and_identity",
            tests::compact_roundtrip_preserves_expected_hashes_and_identity as fn(),
        ),
        ("every_identity_field_is_required_and_bound", tests::every_identity_field_is_required_and_bound as fn()),
        (
            "all_byte_corruption_truncation_and_appends_are_rejected",
            tests::all_byte_corruption_truncation_and_appends_are_rejected as fn(),
        ),
        (
            "malformed_records_are_rejected_even_with_a_recomputed_checksum",
            tests::malformed_records_are_rejected_even_with_a_recomputed_checksum as fn(),
        ),
        (
            "candidate_state_and_effect_changes_report_first_component_and_stick",
            tests::candidate_state_and_effect_changes_report_first_component_and_stick as fn(),
        ),
        (
            "candidate_order_missing_and_extra_records_fail_without_searching_ahead",
            tests::candidate_order_missing_and_extra_records_fail_without_searching_ahead as fn(),
        ),
        (
            "empty_partial_and_invalid_producers_never_seal_and_existing_files_are_preserved",
            tests::empty_partial_and_invalid_producers_never_seal_and_existing_files_are_preserved as fn(),
        ),
        ("invalid_candidate_shape_poisons_comparison", tests::invalid_candidate_shape_poisons_comparison as fn()),
        (
            "authenticated_hashes_are_immutable_after_loading",
            tests::authenticated_hashes_are_immutable_after_loading as fn(),
        ),
        (
            "oversized_reference_is_rejected_before_reading_its_payload",
            tests::oversized_reference_is_rejected_before_reading_its_payload as fn(),
        ),
        (
            "header_length_endian_and_total_extent_are_independent",
            tests::header_length_endian_and_total_extent_are_independent as fn(),
        ),
        (
            "each_empty_header_field_refuses_without_other_corruption",
            tests::each_empty_header_field_refuses_without_other_corruption as fn(),
        ),
        (
            "full_trace_header_refusals_are_not_checksum_failures",
            tests::full_trace_header_refusals_are_not_checksum_failures as fn(),
        ),
    ]
}
