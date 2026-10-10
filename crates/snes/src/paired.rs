//! Distinct complete checkpoint wire envelopes and identity-bound cache validation.
use crate::{Reference, Subject};
use anyhow::{Result, ensure};
use rustysnes_core::{SaveReader, SaveWriter};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
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
pub(crate) fn envelope(tag: &[u8; 4], cpu: &[u8], hardware: &[u8], metadata: &Value) -> Vec<u8> {
    let mut writer = SaveWriter::new();
    writer.write_bytes(tag);
    writer.write_u16(1);
    writer.write_len_prefixed(cpu);
    writer.write_len_prefixed(hardware);
    writer.write_len_prefixed(&serde_json::to_vec(metadata).unwrap());
    writer.into_bytes()
}
pub(crate) fn open<'a>(bytes: &'a [u8], tag: &[u8; 4]) -> Result<(&'a [u8], &'a [u8], Value)> {
    ensure!(bytes.len() >= 6 && &bytes[..4] == tag, "checkpoint side format mismatch");
    let mut reader = SaveReader::new(&bytes[4..]);
    ensure!(reader.read_u16()? == 1, "unknown checkpoint version");
    let cpu = reader.read_len_prefixed()?;
    let hardware = reader.read_len_prefixed()?;
    let metadata = serde_json::from_slice(reader.read_len_prefixed()?)?;
    ensure!(reader.remaining() == 0, "trailing checkpoint state");
    Ok((cpu, hardware, metadata))
}
pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PairedCheckpoint {
    pub schema_version: u32,
    pub identities: Value,
    pub input_index: usize,
    pub pad: u16,
    pub history_sha256: String,
    pub reference: Vec<u8>,
    pub subject: Vec<u8>,
    pub reference_sha256: String,
    pub subject_sha256: String,
    pub pair_sha256: String,
}
fn pair_digest(pair: &PairedCheckpoint) -> String {
    sha256(
        &serde_json::to_vec(&json!({"schema_version":pair.schema_version,"identities":pair.identities,
        "input_index":pair.input_index,"pad":pair.pad,"history_sha256":pair.history_sha256,
        "reference_sha256":pair.reference_sha256,"subject_sha256":pair.subject_sha256}))
        .unwrap(),
    )
}
fn validate_identities(identities: &Value) -> Result<()> {
    for name in ["rom_sha256", "movie_sha256", "input_sha256", "source_sha256", "hardware_sha256"] {
        let value = identities[name].as_str().ok_or_else(|| anyhow::anyhow!("missing checkpoint identity {name}"))?;
        ensure!(value.len() == 64 && value.bytes().all(|v| v.is_ascii_hexdigit()), "invalid checkpoint digest {name}");
    }
    ensure!(
        identities["profile"] == "lsnes-profile" && identities["serializer"] == "paired-v1",
        "incompatible profile or serializer"
    );
    ensure!(
        identities["compiler"].as_str().is_some_and(|s| !s.is_empty())
            && identities["features"].is_array()
            && identities["settings"].is_object(),
        "missing compiler/features/settings checkpoint identity"
    );
    Ok(())
}
pub fn capture(
    reference: &Reference,
    subject: &Subject,
    identities: Value,
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
        schema_version: 1,
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
    pair.pair_sha256 = pair_digest(&pair);
    Ok(pair)
}
pub fn restore(
    pair: &PairedCheckpoint,
    identities: &Value,
    reference: &mut Reference,
    subject: &mut Subject,
) -> Result<()> {
    validate_identities(identities)?;
    ensure!(
        pair.schema_version == 1 && pair.identities == *identities,
        "incompatible pair identity or serializer version"
    );
    ensure!(
        sha256(&pair.reference) == pair.reference_sha256
            && sha256(&pair.subject) == pair.subject_sha256
            && pair_digest(pair) == pair.pair_sha256,
        "corrupt paired checkpoint digest"
    );
    reference.restore_checkpoint_state(&pair.reference)?;
    subject.restore_checkpoint_state(&pair.subject)?;
    ensure!(reference.instruction_snapshot() == subject.instruction_snapshot(), "restored pair diverges");
    Ok(())
}
fn trace_hash(hash: &mut Sha256, trace: &crate::diagnostic::InstructionTrace) -> Result<()> {
    hash.update(serde_json::to_vec(&trace.before.cpu)?);
    hash.update(&trace.before.pending_cpu);
    hash.update(&trace.before.hardware);
    hash.update(serde_json::to_vec(&trace.events)?);
    hash.update(serde_json::to_vec(&trace.after.cpu)?);
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
fn validate_side<M: CompleteMachine>(cold: &mut M, warm: &mut M, next_pad: u16) -> Result<Value> {
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
            return Ok(
                json!({"instructions":count,"observed_rollovers":2,"ordered_trace_sha256":format!("{:x}",hash.finalize()),"ending_sha256":sha256(&cold.checkpoint())}),
            );
        }
    }
    anyhow::bail!("checkpoint validation exceeded instruction budget")
}
/// Check each restored side against its own cold execution through a complete frame.
/// Candidate/reference divergence remains diagnostic evidence, not a serializer failure.
pub fn validate_next_frame(
    cold_r: &mut Reference,
    cold_s: &mut Subject,
    warm_r: &mut Reference,
    warm_s: &mut Subject,
    next_pad: u16,
) -> Result<Value> {
    let started = std::time::Instant::now();
    let reference = validate_side(cold_r, warm_r, next_pad)?;
    let subject = validate_side(cold_s, warm_s, next_pad)?;
    Ok(json!({"schema_version":1,"status":"pass","complete":true,"completed_frames":1,"observed_rollovers":2,
        "candidate_reference_comparison":if reference["ordered_trace_sha256"]==subject["ordered_trace_sha256"]{"pass"}else{"divergence"},
        "reference":reference,"subject":subject,"elapsed_seconds":started.elapsed().as_secs_f64()}))
}

/// Obtain an exact-candidate cache. Unknown, corrupt and cross-candidate caches are
/// preserved as rejected evidence and rebuilt from clean power-on.
pub fn prepare_cache(
    rom: &[u8],
    seed: u64,
    table: &[runtime::step::StepEntry],
    pads: &[u16],
    boundary: (usize, usize),
    mut identities: Value,
    path: &std::path::Path,
) -> Result<(PairedCheckpoint, Value)> {
    let (start, skip) = boundary;
    validate_identities(&identities)?;
    ensure!(identities["rom_sha256"] == sha256(rom), "checkpoint ROM identity differs from bytes");
    let canonical = pads.iter().map(|pad| format!("{pad:04X}\n")).collect::<String>();
    ensure!(
        identities["input_sha256"] == sha256(canonical.as_bytes()),
        "checkpoint input identity differs from sequence"
    );
    ensure!(identities["settings"]["seed"] == seed, "checkpoint seed differs from settings");
    let mut dispatch = table.iter().map(|entry| (entry.0, entry.1, entry.2)).collect::<Vec<_>>();
    dispatch.sort_by_key(|entry| (entry.0, entry.1));
    identities["dispatch_sha256"] = sha256(&serde_json::to_vec(&dispatch)?).into();
    ensure!(start < pads.len(), "checkpoint input boundary outside movie");
    let began = std::time::Instant::now();
    let mut rejection = None;
    if path.exists() {
        let loaded = (|| -> Result<(PairedCheckpoint, Value)> {
            let pair_bytes = std::fs::read(path.join("pair.json"))?;
            let validation_bytes = std::fs::read(path.join("validation.json"))?;
            let record: Value = serde_json::from_slice(&std::fs::read(path.join("cache.json"))?)?;
            ensure!(
                record["schema_version"] == 1
                    && record["start_input_index"] == start
                    && record["start_instruction"] == skip,
                "cache boundary incompatible"
            );
            ensure!(
                record["pair_file_sha256"] == sha256(&pair_bytes)
                    && record["validation_file_sha256"] == sha256(&validation_bytes),
                "cache evidence digest mismatch"
            );
            let pair: PairedCheckpoint = serde_json::from_slice(&pair_bytes)?;
            let validation: Value = serde_json::from_slice(&validation_bytes)?;
            ensure!(
                validation["status"] == "pass"
                    && validation["complete"] == true
                    && validation["completed_frames"] == 1
                    && validation["observed_rollovers"] == 2
                    && validation["pair_sha256"] == pair.pair_sha256,
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
                    json!({"schema_version":1,"status":"pass","complete":true,"action":"reused","validation":validation,"cache_path":path,"cache_creation_seconds":began.elapsed().as_secs_f64()}),
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
        history.update(serde_json::to_vec(&a.cpu)?);
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
    let mut validation = validate_next_frame(&mut r, &mut s, &mut warm_r, &mut warm_s, pads[input + 1])?;
    validation["pair_sha256"] = pair.pair_sha256.clone().into();
    validation["identities"] = identities;
    std::fs::create_dir_all(path)?;
    let pair_bytes = serde_json::to_vec(&pair)?;
    let validation_bytes = serde_json::to_vec_pretty(&validation)?;
    std::fs::write(path.join("pair.json"), &pair_bytes)?;
    std::fs::write(path.join("validation.json"), &validation_bytes)?;
    std::fs::write(
        path.join("cache.json"),
        serde_json::to_vec_pretty(
            &json!({"schema_version":1,"pair_file_sha256":sha256(&pair_bytes),"validation_file_sha256":sha256(&validation_bytes),"start_input_index":start,"start_instruction":skip}),
        )?,
    )?;
    Ok((
        pair,
        json!({"schema_version":1,"status":"pass","complete":true,"action":if rejection.is_some(){"rebuilt"}else{"created"},"rejection":rejection,"validation":validation,"cache_path":path,"cache_creation_seconds":began.elapsed().as_secs_f64()}),
    ))
}
