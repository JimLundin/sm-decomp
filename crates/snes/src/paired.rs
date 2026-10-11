//! Distinct complete checkpoint wire envelopes and identity-bound cache validation.
use crate::{Reference, Subject};
use anyhow::{Result, ensure};
use pipeline_model::{CacheRecord, CheckpointIdentities, PairData, PairMetadata, PairValidation};
use rustysnes_core::{SaveReader, SaveWriter};
use serde::{Deserialize, Serialize};

use sha2::{Digest, Sha256};

pub(crate) fn raw_cpu(cpu: &rustysnes_core::cpu::Cpu) -> Vec<u8> {
    let mut writer = SaveWriter::new();
    cpu.save_state(&mut writer);
    writer.write_u64(cpu.interrupts_taken);
    writer.write_bool(cpu.last_interrupt_was_nmi);
    writer.into_bytes()
}
pub(crate) fn load_cpu(cpu: &mut rustysnes_core::cpu::Cpu, bytes: &[u8]) -> Result<()> {
    let mut reader = SaveReader::new(bytes);
    cpu.load_state(&mut reader)?;
    cpu.interrupts_taken = reader.read_u64()?;
    cpu.last_interrupt_was_nmi = reader.read_bool()?;
    ensure!(reader.remaining() == 0, "trailing raw CPU checkpoint data");
    Ok(())
}
pub(crate) fn envelope(tag: &[u8; 4], cpu: &[u8], hardware: &[u8], metadata: &[u8]) -> Vec<u8> {
    let mut writer = SaveWriter::new();
    writer.write_bytes(tag);
    writer.write_u16(2);
    writer.write_len_prefixed(cpu);
    writer.write_len_prefixed(hardware);
    writer.write_len_prefixed(metadata);
    writer.into_bytes()
}
pub(crate) fn open<'a>(bytes: &'a [u8], tag: &[u8; 4]) -> Result<(&'a [u8], &'a [u8], &'a [u8])> {
    ensure!(bytes.len() >= 6 && &bytes[..4] == tag, "checkpoint side format mismatch");
    let mut reader = SaveReader::new(&bytes[4..]);
    ensure!(reader.read_u16()? == 2, "unknown checkpoint version");
    let cpu = reader.read_len_prefixed()?;
    let hardware = reader.read_len_prefixed()?;
    let metadata = reader.read_len_prefixed()?;
    ensure!(reader.remaining() == 0, "trailing checkpoint state");
    Ok((cpu, hardware, metadata))
}
pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PairedCheckpoint {
    pub schema_version: u32,
    pub identities: CheckpointIdentities,
    pub input_index: usize,
    pub pad: u16,
    pub history_sha256: String,
    pub reference: Vec<u8>,
    pub subject: Vec<u8>,
    pub reference_sha256: String,
    pub subject_sha256: String,
    pub pair_sha256: String,
}
fn metadata(pair: &PairedCheckpoint) -> Result<PairMetadata> {
    Ok(PairMetadata {
        schema_version: pair.schema_version,
        identities: pair.identities.clone(),
        input_index: pair.input_index,
        pad: pair.pad,
        history_sha256: pair.history_sha256.clone(),
        reference_sha256: pair.reference_sha256.clone(),
        subject_sha256: pair.subject_sha256.clone(),
    })
}
fn pair_digest(pair: &PairedCheckpoint) -> Result<String> {
    Ok(sha256(toml::to_string(&metadata(pair)?)?.as_bytes()))
}
pub fn encode_pair(pair: &PairedCheckpoint) -> Result<Vec<u8>> {
    PairData {
        metadata: metadata(pair)?,
        reference: pair.reference.clone(),
        subject: pair.subject.clone(),
        pair_sha256: pair.pair_sha256.clone(),
    }
    .encode()
}
pub fn decode_pair(bytes: &[u8]) -> Result<PairedCheckpoint> {
    let pair = PairData::decode(bytes)?;
    Ok(PairedCheckpoint {
        schema_version: pair.metadata.schema_version,
        identities: pair.metadata.identities,
        input_index: pair.metadata.input_index,
        pad: pair.metadata.pad,
        history_sha256: pair.metadata.history_sha256,
        reference: pair.reference,
        subject: pair.subject,
        reference_sha256: pair.metadata.reference_sha256,
        subject_sha256: pair.metadata.subject_sha256,
        pair_sha256: pair.pair_sha256,
    })
}
fn validate_identities(identities: &CheckpointIdentities) -> Result<()> {
    identities.validate()
}
pub fn capture(
    reference: &Reference,
    subject: &Subject,
    identities: CheckpointIdentities,
    input_index: usize,
    pad: u16,
    history_sha256: String,
) -> Result<PairedCheckpoint> {
    validate_identities(&identities)?;
    ensure!(
        history_sha256.len() == 64 && history_sha256.bytes().all(|v| v.is_ascii_hexdigit()),
        "history must be a computed digest"
    );
    ensure!(reference.instruction_snapshot() == subject.instruction_snapshot(), "pair boundary is divergent");
    let r = reference.checkpoint_state();
    let s = subject.checkpoint_state();
    let mut pair = PairedCheckpoint {
        schema_version: 2,
        identities,
        input_index,
        pad,
        history_sha256,
        reference_sha256: sha256(&r),
        subject_sha256: sha256(&s),
        reference: r,
        subject: s,
        pair_sha256: String::new(),
    };
    pair.pair_sha256 = pair_digest(&pair)?;
    Ok(pair)
}
pub fn restore(
    pair: &PairedCheckpoint,
    identities: &CheckpointIdentities,
    reference: &mut Reference,
    subject: &mut Subject,
) -> Result<()> {
    validate_identities(identities)?;
    ensure!(
        pair.schema_version == 2 && pair.identities == *identities,
        "incompatible pair identity or serializer version"
    );
    ensure!(
        sha256(&pair.reference) == pair.reference_sha256
            && sha256(&pair.subject) == pair.subject_sha256
            && pair_digest(pair)? == pair.pair_sha256,
        "corrupt paired checkpoint digest"
    );
    reference.restore_checkpoint_state(&pair.reference)?;
    subject.restore_checkpoint_state(&pair.subject)?;
    ensure!(reference.instruction_snapshot() == subject.instruction_snapshot(), "restored pair diverges");
    Ok(())
}
fn trace_hash(hash: &mut Sha256, trace: &crate::diagnostic::InstructionTrace) -> Result<()> {
    hash.update(toml::to_string(&trace.before.cpu)?.as_bytes());
    hash.update(&trace.before.pending_cpu);
    hash.update(&trace.before.hardware);
    for event in &trace.events {
        hash.update(toml::to_string(event)?.as_bytes());
    }
    hash.update(toml::to_string(&trace.after.cpu)?.as_bytes());
    hash.update(&trace.after.pending_cpu);
    hash.update(&trace.after.hardware);
    Ok(())
}
trait CompleteMachine {
    fn observed(&mut self) -> Result<crate::diagnostic::InstructionTrace>;
    fn snapshot(&self) -> crate::diagnostic::InstructionSnapshot;
    fn checkpoint(&self) -> Vec<u8>;
    fn input(&mut self, pad: u16);
}
impl CompleteMachine for Reference {
    fn observed(&mut self) -> Result<crate::diagnostic::InstructionTrace> {
        self.step_instruction_observed()
    }
    fn snapshot(&self) -> crate::diagnostic::InstructionSnapshot {
        self.instruction_snapshot()
    }
    fn checkpoint(&self) -> Vec<u8> {
        self.checkpoint_state()
    }
    fn input(&mut self, pad: u16) {
        self.set_input(pad)
    }
}
impl CompleteMachine for Subject {
    fn observed(&mut self) -> Result<crate::diagnostic::InstructionTrace> {
        self.step_instruction_observed()
    }
    fn snapshot(&self) -> crate::diagnostic::InstructionSnapshot {
        self.instruction_snapshot()
    }
    fn checkpoint(&self) -> Vec<u8> {
        self.checkpoint_state()
    }
    fn input(&mut self, pad: u16) {
        self.set_input(pad)
    }
}
fn validate_side<M: CompleteMachine>(
    cold: &mut M,
    warm: &mut M,
    next_pad: u16,
) -> Result<pipeline_model::SideValidation> {
    let frame = cold.snapshot().cpu.frame;
    ensure!(cold.snapshot() == warm.snapshot(), "restored starting boundary differs");
    let mut hash = Sha256::new();
    for count in 1..=2_000_000 {
        let a = cold.observed()?;
        let b = warm.observed()?;
        ensure!(a.before == b.before && a.after == b.after && a.events == b.events, "restored ordered trace diverged");
        trace_hash(&mut hash, &a)?;
        if a.after.cpu.frame != a.before.cpu.frame {
            ensure!(a.after.cpu.frame == a.before.cpu.frame + 1, "validation skipped frame");
            if a.after.cpu.frame == frame + 1 {
                cold.input(next_pad);
                warm.input(next_pad);
                continue;
            }
            ensure!(a.after.cpu.frame == frame + 2, "validation skipped complete frame");
            ensure!(cold.checkpoint() == warm.checkpoint(), "restored complete ending state differs");
            return Ok(pipeline_model::SideValidation {
                instructions: count,
                observed_rollovers: 2,
                ordered_trace_sha256: format!("{:x}", hash.finalize()),
                ending_sha256: sha256(&cold.checkpoint()),
            });
        }
    }
    anyhow::bail!("checkpoint validation exceeded instruction budget")
}
/// Check each restored side against its own cold execution through a complete frame.
/// Candidate/reference divergence remains diagnostic evidence, not a serializer failure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FrameValidation {
    pub schema_version: u32,
    pub status: String,
    pub complete: bool,
    pub completed_frames: u64,
    pub observed_rollovers: u64,
    pub candidate_reference_comparison: String,
    pub reference: pipeline_model::SideValidation,
    pub subject: pipeline_model::SideValidation,
    pub elapsed_seconds: f64,
}
pub fn validate_next_frame(
    cold_r: &mut Reference,
    cold_s: &mut Subject,
    warm_r: &mut Reference,
    warm_s: &mut Subject,
    next_pad: u16,
) -> Result<FrameValidation> {
    let started = std::time::Instant::now();
    let reference = validate_side(cold_r, warm_r, next_pad)?;
    let subject = validate_side(cold_s, warm_s, next_pad)?;
    Ok(FrameValidation {
        schema_version: 1,
        status: "pass".into(),
        complete: true,
        completed_frames: 1,
        observed_rollovers: 2,
        candidate_reference_comparison: if reference.ordered_trace_sha256 == subject.ordered_trace_sha256 {
            "pass"
        } else {
            "divergence"
        }
        .into(),
        reference,
        subject,
        elapsed_seconds: started.elapsed().as_secs_f64(),
    })
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CacheReport {
    pub schema_version: u32,
    pub status: String,
    pub complete: bool,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejection: Option<String>,
    pub validation: PairValidation,
    pub cache_path: std::path::PathBuf,
    pub cache_creation_seconds: f64,
}

/// Obtain an exact-candidate cache. Unknown, corrupt and cross-candidate caches are
/// preserved as rejected evidence and rebuilt from clean power-on.
pub fn prepare_cache(
    rom: &[u8],
    seed: u64,
    table: &[runtime::step::StepEntry],
    pads: &[u16],
    boundary: (usize, usize),
    mut identities: CheckpointIdentities,
    path: &std::path::Path,
) -> Result<(PairedCheckpoint, CacheReport)> {
    let (start, skip) = boundary;
    validate_identities(&identities)?;
    ensure!(identities.rom_sha256 == sha256(rom), "checkpoint ROM identity differs from bytes");
    let canonical = pads.iter().map(|pad| format!("{pad:04X}\n")).collect::<String>();
    ensure!(identities.input_sha256 == sha256(canonical.as_bytes()), "checkpoint input identity differs from sequence");
    ensure!(identities.settings.seed == seed, "checkpoint seed differs from settings");
    let mut dispatch = table.iter().map(|entry| (entry.0, entry.1, entry.2)).collect::<Vec<_>>();
    dispatch.sort_by_key(|entry| (entry.0, entry.1));
    let dispatch_bytes = dispatch
        .iter()
        .flat_map(|(address, mx, bytes)| {
            let mut row = address.to_le_bytes().to_vec();
            row.push(*mx);
            row.extend_from_slice(bytes);
            row
        })
        .collect::<Vec<_>>();
    identities.dispatch_sha256 = Some(sha256(&dispatch_bytes));
    ensure!(start < pads.len(), "checkpoint input boundary outside movie");
    let began = std::time::Instant::now();
    let mut rejection = None;
    if path.exists() {
        let loaded = (|| -> Result<(PairedCheckpoint, PairValidation)> {
            let pair_bytes = std::fs::read(path.join("pair.bin"))?;
            let validation_bytes = std::fs::read(path.join("validation.toml"))?;
            let record: CacheRecord = pipeline_model::read(&path.join("cache.toml"))?;
            ensure!(
                record.schema_version == 2
                    && record.format == "sm-paired-cache-v2"
                    && record.start_input_index == start
                    && record.start_instruction == skip,
                "cache boundary incompatible"
            );
            ensure!(
                record.pair_file_sha256 == sha256(&pair_bytes)
                    && record.validation_file_sha256 == sha256(&validation_bytes),
                "cache evidence digest mismatch"
            );
            let pair = decode_pair(&pair_bytes)?;
            let validation: PairValidation = toml::from_str(std::str::from_utf8(&validation_bytes)?)?;
            ensure!(
                validation.status == "pass"
                    && validation.complete
                    && validation.completed_frames == 1
                    && validation.observed_rollovers == 2
                    && validation.pair_sha256 == pair.pair_sha256,
                "cache lacks complete cold validation"
            );
            let mut r = Reference::new(rom, seed)?;
            let mut s = Subject::with_steps(rom, seed, table)?;
            restore(&pair, &identities, &mut r, &mut s)?;
            Ok((pair, validation))
        })();
        match loaded {
            Ok((pair, validation)) => {
                return Ok((
                    pair,
                    CacheReport {
                        schema_version: 1,
                        status: "pass".into(),
                        complete: true,
                        action: "reused".into(),
                        rejection: None,
                        validation,
                        cache_path: path.to_path_buf(),
                        cache_creation_seconds: began.elapsed().as_secs_f64(),
                    },
                ));
            }
            Err(error) => {
                rejection = Some(error.to_string());
                let rejected = path.with_extension(format!(
                    "rejected-{}",
                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos()
                ));
                std::fs::rename(path, &rejected)?;
            }
        }
    }
    let mut r = Reference::new(rom, seed)?;
    let mut s = Subject::with_steps(rom, seed, table)?;
    let mut history = Sha256::new();
    for (index, pad) in pads[..start].iter().enumerate() {
        r.run_frame(*pad)?;
        s.run_frame(*pad)?;
        let a = r.instruction_snapshot();
        let b = s.instruction_snapshot();
        ensure!(a == b, "cold checkpoint prefix diverged at input {index}");
        history.update(index.to_le_bytes());
        history.update(pad.to_le_bytes());
        history.update(toml::to_string(&a.cpu)?.as_bytes());
        history.update(a.pending_cpu);
        history.update(a.hardware);
    }
    let mut input = start;
    r.set_input(pads[input]);
    s.set_input(pads[input]);
    for _ in 0..skip {
        let a = r.step_instruction_observed()?;
        let b = s.step_instruction_observed()?;
        ensure!(
            a.before == b.before && a.after == b.after && a.events == b.events,
            "cold checkpoint instruction prefix diverged"
        );
        trace_hash(&mut history, &a)?;
        if a.before.cpu.frame != a.after.cpu.frame {
            input += 1;
            ensure!(input < pads.len(), "checkpoint prefix exhausted inputs");
            r.set_input(pads[input]);
            s.set_input(pads[input]);
        }
    }
    ensure!(input + 1 < pads.len(), "checkpoint validation requires next complete cold frame input");
    let pair = capture(&r, &s, identities.clone(), input, pads[input], format!("{:x}", history.finalize()))?;
    let mut warm_r = Reference::new(rom, seed)?;
    let mut warm_s = Subject::with_steps(rom, seed, table)?;
    restore(&pair, &identities, &mut warm_r, &mut warm_s)?;
    let frame = validate_next_frame(&mut r, &mut s, &mut warm_r, &mut warm_s, pads[input + 1])?;
    let validation = PairValidation {
        schema_version: frame.schema_version,
        status: frame.status,
        complete: frame.complete,
        completed_frames: frame.completed_frames,
        observed_rollovers: frame.observed_rollovers,
        candidate_reference_comparison: frame.candidate_reference_comparison,
        reference: frame.reference,
        subject: frame.subject,
        elapsed_seconds: frame.elapsed_seconds,
        pair_sha256: pair.pair_sha256.clone(),
        identities,
    };
    std::fs::create_dir_all(path)?;
    let pair_bytes = encode_pair(&pair)?;
    let validation_bytes = toml::to_string_pretty(&validation)?.into_bytes();
    std::fs::write(path.join("pair.bin"), &pair_bytes)?;
    std::fs::write(path.join("validation.toml"), &validation_bytes)?;
    pipeline_model::write(
        &path.join("cache.toml"),
        &CacheRecord {
            schema_version: 2,
            format: "sm-paired-cache-v2".into(),
            pair_file_sha256: sha256(&pair_bytes),
            validation_file_sha256: sha256(&validation_bytes),
            start_input_index: start,
            start_instruction: skip,
        },
    )?;
    Ok((
        pair,
        CacheReport {
            schema_version: 1,
            status: "pass".into(),
            complete: true,
            action: if rejection.is_some() { "rebuilt" } else { "created" }.into(),
            rejection,
            validation,
            cache_path: path.to_path_buf(),
            cache_creation_seconds: began.elapsed().as_secs_f64(),
        },
    ))
}
