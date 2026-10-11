use crate::{
    digest, family, iteration_increment as increment, lock, pattern_contract as contract, patterns, readiness,
    readiness_origins as origins,
    readiness_run::{Run, safe_artifact, source_inventory},
};
use anyhow::{Context, Result, ensure};
use pipeline_model::{
    Candidate, Complexity, InstructionProfile, KeyInventory, SelectedKey, SelectionAction, SelectionOutcome,
    timed_pattern::Declaration,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use toml::Value;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Proof {
    pub schema_version: u32,
    pub status: String,
    pub complete: bool,
    pub opcode: String,
    pub production_enabled: bool,
    pub source_sha256: String,
    pub instruction_sha256: String,
    pub timing_sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Promotion {
    pub schema_version: u32,
    pub opcode: String,
    pub proof_path: PathBuf,
    pub proof_sha256: String,
    pub prior_declaration_sha256: String,
    pub proposed_declaration_sha256: String,
}
pub(crate) fn prove(root: &Path, opcode: &str) -> Result<Value> {
    let mut run = Run::create(root, "pattern-proof", crate :: report ! ({ "prospective" : opcode , }), None)?;
    eprintln!("Run: {}", run.path.display());
    let result = (|| -> Result<()> {
        run.capture_source()?;
        let output = run.path.join("checks/patterns");
        fs::create_dir(&output)?;
        let request = crate :: report ! ({ "prospective" : opcode , "timing" : true , });
        let mut instruction = Value::Table(toml::Table::new());
        patterns::produce(&mut run, &output, &request, &mut instruction)?;
        patterns::authenticate(crate :: report ! ({ "output" : output , "run" : run . path , "timing" : true , }))?;
        let declaration: Declaration = pipeline_model::read(&root.join(contract::DECLARATION))?;
        ensure!(!declaration.patterns.iter().any(|p| p.opcode == opcode), "pattern is already enabled");
        let proof = Proof {
            schema_version: 1,
            status: "pass".into(),
            complete: true,
            opcode: opcode.into(),
            production_enabled: false,
            source_sha256: family::field(&run.manifest.identities["source"], "sha256")?.into(),
            instruction_sha256: digest(&output.join("instruction.toml"))?,
            timing_sha256: digest(&output.join("timing.toml"))?,
        };
        run.record_typed_check("prospective-proof", &proof)?;
        run.check_sources()
    })();
    match result {
        Ok(()) => run.finalize(
            "pass",
            Some("prospective-proof"),
            Some("complete independent proof; production remains disabled"),
            &["prospective-proof".into()],
        ),
        Err(error) => run.finalize("error", Some("prospective-proof"), Some(&format!("{error:#}")), &[]),
    }
}
pub(crate) fn verify_proof(owner: &Path) -> Result<Proof> {
    let owner_path = fs::canonicalize(owner)?;
    let owner = owner_path.as_path();
    let (manifest, _) = origins::snapshot(owner)?;
    origins::owned(owner, &manifest)?;
    let verdict = origins::document(&owner.join("verdict.toml"))?;
    ensure!(
        manifest["type"] == crate::readiness_metadata::value(&"pattern-proof").expect("validated native metadata")
            && manifest["state"] == crate::readiness_metadata::value(&"complete").expect("validated native metadata")
            && verdict["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
            && verdict["acceptance_eligible"]
                == crate::readiness_metadata::value(&false).expect("validated native metadata"),
        "proof is not completed independent evidence"
    );
    let proof: Proof = pipeline_model::read(&owner.join("checks/prospective-proof.toml"))?;
    ensure!(
        proof.schema_version == 1
            && proof.status == "pass"
            && proof.complete
            && !proof.production_enabled
            && crate::readiness_metadata::value(&proof.source_sha256).expect("validated native metadata")
                == manifest["identities"]["source"]["sha256"]
            && crate::readiness_metadata::value(&toml::Value::try_from(&proof)?).expect("validated native metadata")
                == verdict["checks"]["prospective-proof"],
        "prospective proof identity differs"
    );
    let output = owner.join("checks/patterns");
    ensure!(
        digest(&output.join("instruction.toml"))? == proof.instruction_sha256
            && digest(&output.join("timing.toml"))? == proof.timing_sha256,
        "proof checksums differ"
    );
    let instruction =
        patterns::authenticate(crate :: report ! ({ "output" : output , "run" : owner , "timing" : true , }))?;
    ensure!(
        instruction["prospective_opcode"]
            == crate::readiness_metadata::value(&proof.opcode).expect("validated native metadata"),
        "proved opcode differs"
    );
    Ok(proof)
}
fn promoted(mut declaration: Declaration, opcode: &str) -> Result<Declaration> {
    declaration.validate()?;
    ensure!(!declaration.patterns.iter().any(|p| p.opcode == opcode), "pattern already enabled");
    let selected = declaration.clone().select(Some(opcode))?.patterns.remove(0);
    declaration.prospective_patterns.retain(|p| p.opcode != opcode);
    for (target, additions) in [
        (&mut declaration.timing_tests, &selected.timing_tests),
        (&mut declaration.timing_oracles, &selected.timing_oracles),
        (&mut declaration.hardware_conditions, &selected.hardware_conditions),
    ] {
        for value in additions {
            if !target.contains(value) {
                target.push(value.clone());
            }
        }
    }
    declaration.patterns.push(selected);
    declaration.validate()?;
    Ok(declaration)
}
pub(crate) fn prepare(root: &Path, selection: &Path, proof_owner: &Path, timeout: u64) -> Result<PathBuf> {
    let selection = fs::canonicalize(selection)?;
    let proof_owner = fs::canonicalize(proof_owner)?;
    let mut run = Run::create(root, "increment-preparation", crate::readiness_contract::domain(), None)?;
    let result = (|| -> Result<()> {
        run.capture_source()?;
        let proof = verify_proof(&proof_owner)?;
        let selected = increment::selection_for(&selection, SelectionOutcome::NeedsTimedPattern)?;
        ensure!(
            selected.source_sha256 == proof.source_sha256
                && crate::readiness_metadata::value(&selected.source_sha256).expect("validated native metadata")
                    == run.manifest.identities["source"]["sha256"],
            "proof/selection/current sources differ"
        );
        let declaration: Declaration = pipeline_model::read(&root.join(contract::DECLARATION))?;
        let pattern = declaration.clone().select(Some(&proof.opcode))?.patterns.remove(0);
        let name = format!("{}/{}", pattern.mnemonic, pattern.mode);
        ensure!(
            selected.ranking.observed_unsupported_patterns.iter().any(|p| p.pattern == name && p.count > 0),
            "proved pattern was not observed in the validated domain"
        );
        let profile: InstructionProfile = pipeline_model::read(&selection.join("generated/selection-profile.toml"))?;
        profile.validate()?;
        let opcode = pattern.opcode_byte()?;
        let observed = profile
            .interpreted
            .iter()
            .filter(|r| r.opcode == opcode && !r.emulation && r.count > 0)
            .min_by_key(|r| (std::cmp::Reverse(r.count), r.address, r.mx))
            .context("proved pattern has no observed native key")?;
        ensure!(opcode == 0xad, "prospective instruction byte extent requires an explicit implementation");
        let rom = root.join("rom/sm.sfc");
        ensure!(digest(&rom)? == selected.identities.rom_sha256, "selection ROM identity changed");
        let offset = (((observed.address & 0x7f0000) >> 1) | (observed.address & 0x7fff)) as usize;
        let raw = fs::read(&rom)?;
        let bytes = raw.get(offset..offset + 3).context("observed instruction outside ROM")?.to_vec();
        ensure!(bytes[0] == opcode, "observed opcode differs from ROM");
        let chosen = Candidate {
            address: observed.address,
            mx: observed.mx,
            variant: format!("m{}x{}", observed.mx >> 1, observed.mx & 1),
            action: SelectionAction::AddRoutineWidth,
            estimated_compiled_step_gain: observed.count,
            complexity: Complexity { blocks: 1, instruction_keys: 1 },
            new_executed_keys: vec![SelectedKey {
                address: observed.address,
                mx: observed.mx,
                opcode,
                bytes,
                pattern: name,
                count: observed.count,
            }],
            deferred_keys: vec![],
            overlapping_keys: vec![],
        };
        run.manifest.accepted_parent = selected.baseline.run_id.into();
        run.save()?;
        let selection_hash = digest(&selection.join("selection.toml"))?;
        run.record_artifact("measured-selection", &selection.join("selection.toml"), Some("generated/selection.toml"))?;
        let baseline: KeyInventory =
            pipeline_model::read(&selection.join("generated/selection-baseline-inventory.toml"))?;
        let generator = increment::generator(&mut run, timeout)?;
        increment::propose_pattern(
            &mut run,
            &baseline,
            &chosen,
            &generator,
            &selection_hash,
            timeout,
            Some(&proof.opcode),
        )?;
        let proposal: increment::Proposal = pipeline_model::read(&run.path.join("checks/candidate-proposal.toml"))?;
        let target = run.path.join("generated/timed_patterns.toml");
        pipeline_model::write(&target, &promoted(declaration, &proof.opcode)?)?;
        run.record_artifact("proposed-declaration", &target, None)?;
        let promotion = Promotion {
            schema_version: 1,
            opcode: proof.opcode,
            proof_path: proof_owner.clone(),
            proof_sha256: digest(&proof_owner.join("checks/prospective-proof.toml"))?,
            prior_declaration_sha256: digest(&root.join(contract::DECLARATION))?,
            proposed_declaration_sha256: digest(&target)?,
        };
        run.record_typed_check("pattern-promotion", &promotion)?;
        let record = increment::Increment {
            schema_version: 1,
            kind: "timed-pattern-expansion".into(),
            baseline_path: selected.baseline.run_path,
            selection_path: selection,
            selection_sha256: selection_hash,
            preparation_path: run.path.clone(),
            chosen,
            deferred: proposal.deferred,
            pattern: Some(promotion),
            session_id: None,
            increment_id: None,
        };
        let path = run.path.join("increment.toml");
        pipeline_model::write(&path, &record)?;
        run.record_artifact("increment", &path, None)?;
        run.check_sources()
    })();
    match result {
        Ok(()) => {
            run.finalize(
                "pass",
                Some("pattern-promotion"),
                Some("one observed pattern key prepared; cold gates remain required"),
                &["candidate-proposal".into(), "pattern-promotion".into()],
            )?;
            Ok(run.path)
        }
        Err(error) => {
            run.finalize("error", Some("pattern-promotion"), Some(&format!("{error:#}")), &[])?;
            Err(error)
        }
    }
}
fn preparation(owner: &Path) -> Result<(Value, increment::Increment, increment::Proposal)> {
    let (manifest, inventory) = origins::snapshot(owner)?;
    origins::owned(owner, &manifest)?;
    let verdict = origins::document(&owner.join("verdict.toml"))?;
    ensure!(
        manifest["type"]
            == crate::readiness_metadata::value(&"increment-preparation").expect("validated native metadata")
            && manifest["state"] == crate::readiness_metadata::value(&"complete").expect("validated native metadata")
            && verdict["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
        "pattern preparation incomplete"
    );
    let record: increment::Increment = pipeline_model::read(&origins::artifact(owner, "increment", &manifest)?)?;
    ensure!(record.preparation_path == owner, "preparation owner differs");
    let promotion = record.pattern.as_ref().context("pattern promotion missing")?;
    let proof = verify_proof(&promotion.proof_path)?;
    ensure!(
        record.kind == "timed-pattern-expansion"
            && promotion.schema_version == 1
            && proof.opcode == promotion.opcode
            && crate::readiness_metadata::value(&proof.source_sha256).expect("validated native metadata")
                == manifest["identities"]["source"]["sha256"]
            && digest(&promotion.proof_path.join("checks/prospective-proof.toml"))? == promotion.proof_sha256
            && crate::readiness_metadata::value(&toml::Value::try_from(promotion)?).expect("validated native metadata")
                == verdict["checks"]["pattern-promotion"],
        "promotion proof identity differs"
    );
    let prior: Declaration = pipeline_model::read(&safe_artifact(&owner.join("source/files"), contract::DECLARATION)?)?;
    let proposed = origins::artifact(owner, "proposed-declaration", &manifest)?;
    ensure!(
        digest(&proposed)? == promotion.proposed_declaration_sha256
            && digest(&owner.join("source/files").join(contract::DECLARATION))? == promotion.prior_declaration_sha256
            && pipeline_model::read::<Declaration>(&proposed)? == promoted(prior, &promotion.opcode)?,
        "promotion differs from independently proved declaration"
    );
    let proposal: increment::Proposal = pipeline_model::read(&owner.join("checks/candidate-proposal.toml"))?;
    ensure!(
        proposal.status == "pass" && proposal.complete && proposal.selection_sha256 == record.selection_sha256,
        "pattern proposal identity differs"
    );
    Ok((inventory, record, proposal))
}
pub(crate) fn apply(root: &Path, owner: &Path, recovery: bool) -> Result<increment::Increment> {
    let (inventory, record, proposal) = preparation(owner)?;
    let promotion = record.pattern.as_ref().unwrap();
    let _guard = lock(&root.join("out/increment-apply.lock"))?;
    let pointer = origins::document(&root.join("out/accepted.toml"))?;
    let (manifest, _) = origins::snapshot(owner)?;
    ensure!(pointer["run_id"] == manifest["accepted_parent"], "accepted predecessor changed");
    let files = [
        (
            "config/functions.toml",
            "generated/functions.toml",
            &proposal.prior_config_sha256,
            &proposal.proposed_config_sha256,
        ),
        (
            contract::DECLARATION,
            "generated/timed_patterns.toml",
            &promotion.prior_declaration_sha256,
            &promotion.proposed_declaration_sha256,
        ),
    ];
    let expected: Vec<pipeline_model::readiness::SourceFile> = inventory["files"].clone().try_into()?;
    apply_files(root, owner, &record, expected, &files, recovery)?;
    Ok(record)
}
type ApplicationFile<'a> = (&'a str, &'a str, &'a String, &'a String);
fn apply_files(
    root: &Path,
    owner: &Path,
    record: &increment::Increment,
    mut expected: Vec<pipeline_model::readiness::SourceFile>,
    files: &[ApplicationFile<'_>],
    recovery: bool,
) -> Result<()> {
    let intent = root.join("out/pattern-application.toml");
    if recovery {
        let retained: increment::Increment =
            pipeline_model::read(&intent).context("sealed application intent missing")?;
        ensure!(retained == *record, "application recovery belongs to a different preparation");
    }
    for &(relative, artifact, prior, next) in files {
        ensure!(digest(&safe_artifact(owner, artifact)?)? == *next, "prepared promotion changed");
        let current = digest(&root.join(relative))?;
        ensure!(current == *prior || (recovery && current == *next), "source changed since pattern preparation");
        expected.iter_mut().find(|r| r.path == relative).context("promotion source missing")?.sha256 = current;
    }
    ensure!(source_inventory(root)? == expected, "source differs from prepared pattern promotion");
    let pending_intent = root.join(format!("out/.pattern-application-{}.pending", std::process::id()));
    let mut output = OpenOptions::new().create_new(true).write(true).open(&pending_intent)?;
    output.write_all(toml::to_string_pretty(record)?.as_bytes())?;
    output.sync_all()?;
    fs::rename(&pending_intent, &intent)?;
    File::open(intent.parent().unwrap())?.sync_all()?;
    for &(relative, artifact, _, next) in files {
        let target = root.join(relative);
        if digest(&target)? == *next {
            continue;
        }
        let pending = target.with_extension(format!("pending-{}", std::process::id()));
        let mut output = OpenOptions::new().create_new(true).write(true).open(&pending)?;
        output.write_all(&fs::read(safe_artifact(owner, artifact)?)?)?;
        output.sync_all()?;
        fs::set_permissions(&pending, target.metadata()?.permissions())?;
        ensure!(source_inventory(root)? == expected, "source changed during pattern application");
        fs::rename(pending, &target)?;
        File::open(target.parent().unwrap())?.sync_all()?;
        expected.iter_mut().find(|r| r.path == relative).unwrap().sha256 = next.clone();
    }
    Ok(())
}
pub(crate) fn verify_promotion(owner: &Path, verified: &Value, record: &increment::Increment) -> Result<()> {
    let (inventory, _, _) = preparation(&record.preparation_path)?;
    let promotion = record.pattern.as_ref().context("accepted promotion missing")?;
    ensure!(
        digest(&safe_artifact(&owner.join("source/files"), contract::DECLARATION)?)?
            == promotion.proposed_declaration_sha256,
        "accepted declaration differs from proved proposal"
    );
    for file in family::array(&inventory, "files")? {
        let relative = family::field(file, "path")?;
        if [contract::DECLARATION, "config/functions.toml"].contains(&relative) {
            continue;
        }
        ensure!(
            digest(&safe_artifact(&owner.join("source/files"), relative)?)? == family::field(file, "sha256")?,
            "promoted emitter/runtime differs from proved sources"
        );
    }
    ensure!(
        verified["verdict"]["checks"]["instruction"]["opcodes"].get(&promotion.opcode).is_some()
            && verified["verdict"]["checks"]["timing"]["source_sha256"]
                == verified["manifest"]["identities"]["source"]["sha256"],
        "promoted candidate lacks fresh complete instruction/timing proof"
    );
    Ok(())
}
pub(crate) fn operate(request: Value) -> Result<Value> {
    let root = request.get("checkout").and_then(Value::as_str).map(PathBuf::from).unwrap_or(std::env::current_dir()?);
    if request["command"] == crate::readiness_metadata::value(&"prove").expect("validated native metadata") {
        return prove(&root, request.get("opcode").and_then(Value::as_str).unwrap_or("ad"));
    }
    let timeout =
        request.get("timeout").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok()).unwrap_or(2400);
    let owner = if let Some(path) = request.get("preparation").and_then(Value::as_str) {
        PathBuf::from(path)
    } else {
        prepare(
            &root,
            Path::new(family::field(&request, "selection")?),
            Path::new(family::field(&request, "proof")?),
            timeout,
        )?
    };
    if request.get("prepare_only").and_then(toml::Value::as_bool) == Some(true) {
        return Ok(crate :: report ! ({ "status" : "pass" , "preparation_path" : owner , }));
    }
    let record = apply(&root, &owner, request.get("recover").and_then(toml::Value::as_bool) == Some(true))?;
    readiness::operate(
        crate :: report ! ({ "checkout" : root , "predecessor" : record . baseline_path , "increment" : record , "timeout" : timeout , "original_run" : request . get ("original_run") . and_then (Value :: as_str) , }),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Temporary(PathBuf);
    impl Drop for Temporary {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn application_fixture() -> (Temporary, PathBuf, increment::Increment) {
        let root = std::env::temp_dir().join(format!(
            "sm-pattern-recovery-{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(root.join("config")).unwrap();
        fs::create_dir_all(root.join("crates/recomp/src")).unwrap();
        fs::create_dir(root.join("out")).unwrap();
        assert!(std::process::Command::new("git").args(["init", "-q"]).arg(&root).status().unwrap().success());
        fs::write(root.join("config/functions.toml"), "old configuration\n").unwrap();
        fs::write(root.join(contract::DECLARATION), "old declaration\n").unwrap();
        let owner = root.join("out/preparation");
        fs::create_dir_all(owner.join("generated")).unwrap();
        fs::write(owner.join("generated/functions.toml"), "new configuration\n").unwrap();
        fs::write(owner.join("generated/timed_patterns.toml"), "new declaration\n").unwrap();
        let record = increment::Increment {
            schema_version: 1,
            kind: "timed-pattern-expansion".into(),
            baseline_path: "baseline".into(),
            selection_path: "selection".into(),
            selection_sha256: "selection".into(),
            preparation_path: owner.clone(),
            chosen: Candidate {
                address: 0x8000,
                mx: 3,
                variant: "m1x1".into(),
                action: SelectionAction::AddRoutineWidth,
                estimated_compiled_step_gain: 1,
                complexity: Complexity { blocks: 1, instruction_keys: 1 },
                new_executed_keys: vec![],
                deferred_keys: vec![],
                overlapping_keys: vec![],
            },
            deferred: vec![],
            pattern: None,
            session_id: None,
            increment_id: None,
        };
        (Temporary(root), owner, record)
    }
    #[test]
    fn sealed_application_recovers_partial_rename_and_rejects_unowned_edits() {
        let (root, owner, record) = application_fixture();
        let expected = source_inventory(&root.0).unwrap();
        let prior_config = digest(&root.0.join("config/functions.toml")).unwrap();
        let prior_pattern = digest(&root.0.join(contract::DECLARATION)).unwrap();
        let next_config = digest(&owner.join("generated/functions.toml")).unwrap();
        let next_pattern = digest(&owner.join("generated/timed_patterns.toml")).unwrap();
        let files = [
            ("config/functions.toml", "generated/functions.toml", &prior_config, &next_config),
            (contract::DECLARATION, "generated/timed_patterns.toml", &prior_pattern, &next_pattern),
        ];
        assert!(apply_files(&root.0, &owner, &record, expected.clone(), &files, true).is_err());
        pipeline_model::write(&root.0.join("out/pattern-application.toml"), &record).unwrap();
        fs::copy(owner.join("generated/functions.toml"), root.0.join("config/functions.toml")).unwrap();
        apply_files(&root.0, &owner, &record, expected.clone(), &files, true).unwrap();
        assert_eq!(digest(&root.0.join(contract::DECLARATION)).unwrap(), next_pattern);
        apply_files(&root.0, &owner, &record, expected.clone(), &files, true).unwrap();
        assert!(apply_files(&root.0, &owner, &record, expected.clone(), &files, false).is_err());
        fs::write(root.0.join("crates/unreviewed.rs"), "unexpected source edit\n").unwrap();
        assert!(apply_files(&root.0, &owner, &record, expected, &files, true).is_err());
    }
    #[test]
    fn altered_preparation_never_records_application_intent_or_changes_sources() {
        let (root, owner, record) = application_fixture();
        let expected = source_inventory(&root.0).unwrap();
        let prior = digest(&root.0.join("config/functions.toml")).unwrap();
        let next = digest(&owner.join("generated/functions.toml")).unwrap();
        fs::write(owner.join("generated/functions.toml"), "tampered proposal\n").unwrap();
        assert!(
            apply_files(
                &root.0,
                &owner,
                &record,
                expected,
                &[("config/functions.toml", "generated/functions.toml", &prior, &next)],
                false
            )
            .is_err()
        );
        assert_eq!(digest(&root.0.join("config/functions.toml")).unwrap(), prior);
        assert!(!root.0.join("out/pattern-application.toml").exists());
    }
    #[test]
    fn promotion_preserves_other_patterns_and_rejects_repeated_or_unknown_proof() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
        let declaration: Declaration = pipeline_model::read(&root.join(contract::DECLARATION)).unwrap();
        let changed = promoted(declaration.clone(), "ad").unwrap();
        assert_eq!(&changed.patterns[..declaration.patterns.len()], &declaration.patterns);
        assert_eq!(changed.patterns.last().unwrap().opcode, "ad");
        assert!(!changed.prospective_patterns.iter().any(|p| p.opcode == "ad"));
        assert!(changed.timing_tests.contains(&"prospective_lda_timing".into()));
        assert!(promoted(changed, "ad").is_err());
        assert!(promoted(declaration, "ff").is_err());
    }
}
