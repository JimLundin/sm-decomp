//! Typed interchange for native pipeline tools. TOML is the native text format.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{fs, path::Path};

pub mod metrics;
pub mod readiness;
pub mod timed_corpus;
pub mod timed_pattern;

pub fn read<T: DeserializeOwned>(path: &Path) -> Result<T> {
    ensure!(path.extension().is_some_and(|s| s == "toml"), "pipeline metadata must use .toml");
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&text).context("invalid native TOML artifact")
}
pub fn write<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    ensure!(path.extension().is_some_and(|s| s == "toml"), "new pipeline metadata must use .toml");
    fs::write(path, toml::to_string_pretty(value)?)?;
    Ok(())
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstructionCount {
    pub address: u32,
    pub mx: u8,
    pub opcode: u8,
    pub count: u64,
    pub emulation: bool,
    pub pattern: String,
    #[serde(default)]
    pub supported: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstructionProfile {
    pub schema_version: u32,
    pub compiled: Vec<InstructionCount>,
    pub interpreted: Vec<InstructionCount>,
    pub generated_instructions: u64,
    pub interpreted_instructions: u64,
    pub generated_dispatches: u64,
    pub interpreted_dispatches: u64,
    pub noninstruction_dispatches: u64,
}
impl InstructionProfile {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.schema_version == 1, "unsupported instruction profile version");
        for (rows, total) in
            [(&self.compiled, self.generated_instructions), (&self.interpreted, self.interpreted_instructions)]
        {
            let mut sum = 0u64;
            let mut keys = std::collections::BTreeSet::new();
            for row in rows {
                ensure!(
                    row.address <= 0xffffff
                        && row.mx < 4
                        && !row.pattern.is_empty()
                        && keys.insert((row.address, row.mx, row.emulation, row.opcode)),
                    "invalid or duplicate instruction profile key"
                );
                sum = sum.checked_add(row.count).context("instruction count overflow")?;
            }
            ensure!(sum == total, "profile total differs from individual counts");
        }
        let dispatches =
            self.generated_dispatches.checked_add(self.interpreted_dispatches).context("dispatch overflow")?;
        let instructions = self
            .generated_instructions
            .checked_add(self.interpreted_instructions)
            .context("instruction total overflow")?;
        ensure!(
            dispatches.checked_sub(instructions) == Some(self.noninstruction_dispatches),
            "profile dispatch accounting differs"
        );
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneratedKey {
    pub address: u32,
    pub mx: u8,
    pub opcode: u8,
    pub bytes: Vec<u8>,
    pub body_sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyInventory {
    pub schema_version: u32,
    pub keys: Vec<GeneratedKey>,
}
impl KeyInventory {
    pub fn validate(&self) -> Result<()> {
        let mut keys = std::collections::BTreeSet::new();
        ensure!(self.schema_version == 1, "unsupported key inventory version");
        for row in &self.keys {
            ensure!(
                row.address <= 0xffffff
                    && row.mx < 4
                    && row.bytes.first() == Some(&row.opcode)
                    && keys.insert((row.address, row.mx))
                    && row.body_sha256.len() == 64
                    && row.body_sha256.bytes().all(|b| b.is_ascii_hexdigit()),
                "invalid or duplicate generated inventory key"
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SelectionOutcome {
    SupportedRoutine,
    NeedsTimedPattern,
    NoCoveredCandidate,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SelectionAction {
    AddRoutineWidth,
    EnableDeferredKeys,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SelectedKey {
    pub address: u32,
    pub mx: u8,
    pub opcode: u8,
    pub bytes: Vec<u8>,
    pub pattern: String,
    pub count: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Complexity {
    pub blocks: usize,
    pub instruction_keys: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Candidate {
    pub address: u32,
    pub mx: u8,
    pub variant: String,
    pub action: SelectionAction,
    pub estimated_compiled_step_gain: u64,
    pub complexity: Complexity,
    pub new_executed_keys: Vec<SelectedKey>,
    pub deferred_keys: Vec<SelectedKey>,
    pub overlapping_keys: Vec<SelectedKey>,
}
impl Candidate {
    pub fn order(&self) -> (std::cmp::Reverse<u64>, usize, usize, u32, u8) {
        (
            std::cmp::Reverse(self.estimated_compiled_step_gain),
            self.complexity.instruction_keys,
            self.complexity.blocks,
            self.address,
            self.mx,
        )
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.address <= 0xffffff && self.mx < 4 && self.variant == format!("m{}x{}", self.mx >> 1, self.mx & 1),
            "invalid selection address/width"
        );
        ensure!(
            !self.new_executed_keys.is_empty()
                && self.complexity.blocks > 0
                && self.complexity.blocks <= 128
                && self.complexity.instruction_keys > 0
                && self.complexity.instruction_keys <= 1024,
            "selection lacks bounded observed work"
        );
        let mut keys = std::collections::BTreeSet::new();
        let mut gain = 0u64;
        for rows in [&self.new_executed_keys, &self.deferred_keys, &self.overlapping_keys] {
            for row in rows {
                ensure!(
                    row.address <= 0xffffff
                        && row.mx < 4
                        && row.bytes.first() == Some(&row.opcode)
                        && keys.insert((row.address, row.mx)),
                    "selection duplicate key/invalid bytes"
                );
            }
        }
        for row in &self.new_executed_keys {
            ensure!(row.count > 0, "selection key was not observed");
            gain = gain.checked_add(row.count).context("selection gain overflow")?;
        }
        ensure!(gain == self.estimated_compiled_step_gain, "selection gain differs from observed counts");
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RejectedCandidate {
    pub address: u32,
    pub mx: u8,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnsupportedPattern {
    pub pattern: String,
    pub count: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SelectionReport {
    pub schema_version: u32,
    pub outcome: SelectionOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chosen: Option<Candidate>,
    pub ranked: Vec<Candidate>,
    pub rejected: Vec<RejectedCandidate>,
    pub observed_unsupported_patterns: Vec<UnsupportedPattern>,
    pub ranking_policy: String,
    pub scope_policy: String,
}

/// The input domain must agree with the accepted owner's sealed identities.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputDomain {
    pub rom_sha256: String,
    pub movie_sha256: String,
    pub input_sha256: String,
    pub seed: u64,
    pub profile: String,
    pub region: String,
    pub controller: String,
    pub reset: String,
    pub frames: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AcceptedBaseline {
    pub run_id: String,
    pub run_path: std::path::PathBuf,
    pub source_sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileIdentities {
    pub rom_sha256: String,
    pub movie_sha256: String,
    pub input_sha256: String,
    pub binary_sha256: String,
    pub compiled_rom_sha256: String,
    pub selected_input_sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileProvenance {
    pub schema_version: u32,
    pub baseline: AcceptedBaseline,
    pub profile_sha256: String,
    pub original_profile_sha256: String,
    pub report_sha256: String,
    pub expected_frames: u64,
    pub identities: ProfileIdentities,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoundSelection {
    pub schema_version: u32,
    pub baseline: AcceptedBaseline,
    pub source_sha256: String,
    pub profile_sha256: String,
    pub baseline_inventory_sha256: String,
    pub original_inventory_sha256: String,
    pub discovery_sha256: String,
    pub identities: InputDomain,
    pub ranking: SelectionReport,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckpointSettings {
    pub seed: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_sha256: Option<String>,
    #[serde(default)]
    pub disabled_keys: Vec<(u32, u8)>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckpointIdentities {
    pub rom_sha256: String,
    pub movie_sha256: String,
    pub input_sha256: String,
    pub source_sha256: String,
    pub hardware_sha256: String,
    pub profile: String,
    pub serializer: String,
    pub compiler: String,
    pub features: Vec<String>,
    pub settings: CheckpointSettings,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dispatch_sha256: Option<String>,
}
impl CheckpointIdentities {
    pub fn validate(&self) -> Result<()> {
        for digest in
            [&self.rom_sha256, &self.movie_sha256, &self.input_sha256, &self.source_sha256, &self.hardware_sha256]
        {
            ensure!(
                digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit()),
                "invalid checkpoint identity digest"
            );
        }
        ensure!(
            self.profile == "lsnes-profile"
                && self.serializer == "paired-v2"
                && !self.compiler.is_empty()
                && !self.features.is_empty(),
            "incompatible checkpoint profile or serializer"
        );
        if let Some(digest) = &self.dispatch_sha256 {
            ensure!(digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit()), "invalid dispatch digest");
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PairMetadata {
    pub schema_version: u32,
    pub identities: CheckpointIdentities,
    pub input_index: usize,
    pub pad: u16,
    pub history_sha256: String,
    pub reference_sha256: String,
    pub subject_sha256: String,
}
/// Version 2 stores state envelopes as bytes, without decimal JSON expansion.
pub struct PairData {
    pub metadata: PairMetadata,
    pub reference: Vec<u8>,
    pub subject: Vec<u8>,
    pub pair_sha256: String,
}
impl PairData {
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let metadata = toml::to_string(&self.metadata)?.into_bytes();
        let mut bytes = b"SMP2\x02\x00".to_vec();
        for data in [&metadata, &self.reference, &self.subject, &self.pair_sha256.as_bytes().to_vec()] {
            let length = u32::try_from(data.len()).context("paired payload too large")?;
            bytes.extend_from_slice(&length.to_le_bytes());
            bytes.extend_from_slice(data);
        }
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.starts_with(b"SMP2\x02\x00"), "unsupported paired binary format");
        let mut rest = &bytes[6..];
        fn payload<'a>(rest: &mut &'a [u8], maximum: usize) -> Result<&'a [u8]> {
            ensure!(rest.len() >= 4, "truncated paired length");
            let length = u32::from_le_bytes(rest[..4].try_into()?) as usize;
            *rest = &rest[4..];
            ensure!(length <= maximum && length <= rest.len(), "invalid paired payload length");
            let data = &rest[..length];
            *rest = &rest[length..];
            Ok(data)
        }
        let metadata = toml::from_str(std::str::from_utf8(payload(&mut rest, 1024 * 1024)?)?)?;
        let reference = payload(&mut rest, 64 * 1024 * 1024)?.to_vec();
        let subject = payload(&mut rest, 64 * 1024 * 1024)?.to_vec();
        let pair_sha256 = std::str::from_utf8(payload(&mut rest, 64)?)?.into();
        ensure!(rest.is_empty(), "trailing paired binary data");
        let pair = Self { metadata, reference, subject, pair_sha256 };
        pair.validate()?;
        Ok(pair)
    }
    pub fn validate(&self) -> Result<()> {
        use sha2::{Digest, Sha256};
        ensure!(self.metadata.schema_version == 2, "unknown paired metadata version");
        self.metadata.identities.validate()?;
        ensure!(
            self.metadata.history_sha256.len() == 64
                && self.metadata.history_sha256.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid paired history digest"
        );
        for (state, tag, hash) in [
            (&self.reference, b"RCP2", &self.metadata.reference_sha256),
            (&self.subject, b"SCP2", &self.metadata.subject_sha256),
        ] {
            ensure!(
                state.len() >= 6
                    && state.starts_with(tag)
                    && state[4..6] == [2, 0]
                    && format!("{:x}", Sha256::digest(state)) == *hash,
                "paired state format or digest differs"
            );
        }
        ensure!(
            format!("{:x}", Sha256::digest(toml::to_string(&self.metadata)?.as_bytes())) == self.pair_sha256,
            "paired metadata digest differs"
        );
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheRecord {
    pub schema_version: u32,
    pub format: String,
    pub pair_file_sha256: String,
    pub validation_file_sha256: String,
    pub start_input_index: usize,
    pub start_instruction: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SideValidation {
    pub instructions: u64,
    pub observed_rollovers: u64,
    pub ordered_trace_sha256: String,
    pub ending_sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PairValidation {
    pub schema_version: u32,
    pub status: String,
    pub complete: bool,
    pub completed_frames: u64,
    pub observed_rollovers: u64,
    pub candidate_reference_comparison: String,
    pub reference: SideValidation,
    pub subject: SideValidation,
    pub elapsed_seconds: f64,
    pub pair_sha256: String,
    pub identities: CheckpointIdentities,
}
impl SelectionReport {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.schema_version == 1, "unsupported selection report version");
        match self.outcome {
            SelectionOutcome::SupportedRoutine => ensure!(
                !self.ranked.is_empty() && self.chosen.as_ref() == self.ranked.first(),
                "selection chosen candidate differs from first ranked candidate"
            ),
            _ => ensure!(self.ranked.is_empty() && self.chosen.is_none(), "no-work selection includes a candidate"),
        };
        for row in &self.ranked {
            row.validate()?;
        }
        ensure!(
            self.ranked.windows(2).all(|r| r[0].order() <= r[1].order()),
            "selection order differs from observed gain/complexity"
        );
        if self.outcome == SelectionOutcome::NeedsTimedPattern {
            ensure!(
                !self.observed_unsupported_patterns.is_empty()
                    && self.observed_unsupported_patterns.iter().all(|r| r.count > 0 && !r.pattern.is_empty()),
                "timed pattern selection lacks observed unsupported work"
            );
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn toml_roundtrip_preserves_no_work_and_rejects_forged_gain() {
        let report = SelectionReport {
            schema_version: 1,
            outcome: SelectionOutcome::NoCoveredCandidate,
            chosen: None,
            ranked: vec![],
            rejected: vec![],
            observed_unsupported_patterns: vec![],
            ranking_policy: "measured gain".into(),
            scope_policy: "bounded ROM CFG".into(),
        };
        let text = toml::to_string_pretty(&report).unwrap();
        assert!(!text.contains("null"));
        let restored: SelectionReport = toml::from_str(&text).unwrap();
        assert_eq!(restored, report);
        restored.validate().unwrap();
        let candidate = Candidate {
            address: 0x8000,
            mx: 3,
            variant: "m1x1".into(),
            action: SelectionAction::AddRoutineWidth,
            estimated_compiled_step_gain: 99,
            complexity: Complexity { blocks: 1, instruction_keys: 1 },
            new_executed_keys: vec![SelectedKey {
                address: 0x8000,
                mx: 3,
                opcode: 234,
                bytes: vec![234],
                pattern: "NOP/Imp".into(),
                count: 1,
            }],
            deferred_keys: vec![],
            overlapping_keys: vec![],
        };
        assert!(candidate.validate().is_err());
    }
    #[test]
    fn profiles_reject_duplicate_counts_and_inconsistent_dispatches() {
        let row = InstructionCount {
            address: 0x8000,
            mx: 3,
            opcode: 234,
            count: 1,
            emulation: false,
            pattern: "NOP/Imp".into(),
            supported: true,
        };
        let mut profile = InstructionProfile {
            schema_version: 1,
            compiled: vec![row.clone()],
            interpreted: vec![],
            generated_instructions: 1,
            interpreted_instructions: 0,
            generated_dispatches: 1,
            interpreted_dispatches: 0,
            noninstruction_dispatches: 0,
        };
        profile.validate().unwrap();
        profile.compiled.push(row);
        profile.generated_instructions = 2;
        assert!(profile.validate().is_err());
    }
}
