use crate::atomic_toml;
use crate::digest;
use crate::readiness_run::Run;
use crate::readiness_run::identity;
use crate::readiness_run::safe_artifact;
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use toml::Value;
fn document(path: &Path) -> Result<Value> {
    crate::readiness_origins::document(path)
}
fn key(row: &Value) -> Result<(u64, u64)> {
    let address =
        row["address"].as_integer().and_then(|n| u64::try_from(n).ok()).context("instruction address missing")?;
    let mx = row["mx"].as_integer().and_then(|n| u64::try_from(n).ok()).context("instruction width missing")?;
    ensure!(address <= 0xffffff && mx < 4, "invalid instruction key");
    Ok((address, mx))
}
fn symbol(text: &str) -> Result<(u64, u64)> {
    let (address, mx) = text
        .strip_prefix("step_")
        .context("instruction symbol prefix")?
        .split_once('_')
        .context("instruction symbol width")?;
    let address = u64::from_str_radix(address, 16)?;
    let mx = mx.parse::<u64>()?;
    ensure!(address <= 0xffffff && mx < 4, "invalid instruction symbol");
    Ok((address, mx))
}
pub(crate) fn table_rows(source: &str) -> Result<Value> {
    let mut bodies = BTreeMap::new();
    let mut remaining = source;
    while let Some((_, after)) = remaining.split_once("fn step_") {
        let (signature, body) = after.split_once("(m: &mut Machine) {").context("generated step signature")?;
        let key = symbol(&format!("step_{signature}"))?;
        let (body, after) = body.split_once("\n}").context("generated step body truncated")?;
        ensure!(
            bodies.insert(key, format!("{:x}", Sha256::digest(body.as_bytes()))).is_none(),
            "generated table duplicate body"
        );
        remaining = after;
    }
    let (_, table) = source.split_once("pub static STEP_TABLE:").context("generated step table missing")?;
    let (_, table) = table.split_once("&[").context("generated step table format")?;
    let table = if let Some((_, initializer)) = table.split_once("= &[") { initializer } else { table };
    if table.starts_with("];") {
        return Ok(crate::report!(vec![]));
    }
    let (table, _) = table.split_once("\n];").context("generated step table truncated")?;
    let mut rows = BTreeMap::new();
    for line in table.lines().map(str::trim).filter(|s| !s.is_empty()) {
        let line = line
            .strip_prefix("(0x")
            .context("generated table entry prefix")?
            .strip_suffix("),")
            .context("generated table entry ending")?;
        let (address, rest) = line.split_once(',').context("generated table address")?;
        let (mx, rest) = rest.split_once(',').context("generated table width")?;
        let key = (u64::from_str_radix(address.trim(), 16)?, mx.trim().parse::<u64>()?);
        let (bytes, sym) = rest
            .trim()
            .strip_prefix("&[")
            .context("generated instruction byte array")?
            .split_once("],")
            .context("generated instruction byte array ending")?;
        let bytes = bytes
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| Ok(s.parse::<u8>()?))
            .collect::<Result<Vec<_>>>()?;
        ensure!(
            !bytes.is_empty() && symbol(sym.trim())? == key && bodies.contains_key(&key),
            "generated table/body key mismatch or empty bytes"
        );
        let row = crate :: report ! ({ "address" : key . 0 , "mx" : key . 1 , "opcode" : bytes [0] , "bytes" : bytes , "body_sha256" : bodies [& key] , });
        ensure!(rows.insert(key, row).is_none(), "generated table duplicate keys");
    }
    Ok(Value::Array(rows.into_values().collect::<Vec<_>>()))
}
pub(crate) fn rom_identity(source: &str) -> Result<&str> {
    let (_, after) = source.split_once("ROM_SHA256:").context("generated table lacks ROM identity")?;
    let (_, after) = after.split_once("Some(\"").context("generated table ROM identity missing")?;
    let (hash, _) = after.split_once('"').context("generated table ROM identity malformed")?;
    ensure!(hash.len() == 64 && hash.bytes().all(|c| c.is_ascii_hexdigit()), "generated table ROM identity malformed");
    Ok(hash)
}
pub(crate) fn generated_inventory(run: &mut Run, steps: &Path) -> Result<PathBuf> {
    let source = fs::read_to_string(steps)?;
    let rows = table_rows(&source)?;
    let inventory = document(&run.path.join("source/inventory.toml"))?;
    let relevant = crate::documents::array(&inventory, "files")?
        .iter()
        .filter(|entry| {
            entry["path"].as_str().is_some_and(|p| {
                ["crates/recomp/src/", "crates/runtime/src/", "crates/snes/src/"]
                    .iter()
                    .any(|prefix| p.starts_with(prefix))
                    || ["Cargo.toml", "Cargo.lock", "crates/game/build.rs", "config/jump_tables.toml"].contains(&p)
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    let report = crate :: report ! ({ "schema_version" : 1 , "rom_sha256" : rom_identity (& source) ? , "steps_sha256" : digest (steps) ? , "steps_evidence" : steps . strip_prefix (& run . path) ? . to_str () . context ("steps path UTF-8") ? , "config_sha256" : digest (& run . path . join ("source/files/config/functions.toml")) ? , "execution_source_sha256" : identity (& relevant) ? , "source_sha256" : run . manifest . identities ["source"] ["sha256"] , "keys" : rows , });
    let path = run.path.join("generated/key-inventory.toml");
    pipeline_model::write(&path, &report)?;
    run.record_artifact("generated-key-inventory", &path, None)?;
    run.manifest.identities.get_mut("generated-key-inventory").context("identity missing")?["evidence"] =
        "generated/key-inventory.toml".into();
    run.save()?;
    run.guard(&path)?;
    Ok(path)
}
fn authenticate_inventory(owner: &Path, manifest: &Value, inventory: &Value) -> Result<()> {
    let path = safe_artifact(owner, crate::documents::field(inventory, "steps_evidence")?)?;
    let source = fs::read_to_string(&path)?;
    ensure!(
        digest(&path)? == crate::documents::field(inventory, "steps_sha256")?
            && inventory["keys"]
                == crate::readiness_metadata::value(&table_rows(&source)?).expect("validated native metadata"),
        "generated inventory differs from actual steps"
    );
    ensure!(
        inventory["source_sha256"] == manifest["identities"]["source"]["sha256"],
        "generated inventory source identity mismatch"
    );
    if let Some(hash) = manifest
        .get("identities")
        .and_then(|ids| ids.get("inputs"))
        .and_then(|inputs| inputs.get("rom_sha256"))
        .and_then(Value::as_str)
    {
        ensure!(
            inventory["rom_sha256"] == crate::readiness_metadata::value(&hash).expect("validated native metadata")
                && rom_identity(&source)? == hash,
            "generated inventory ROM identity mismatch"
        );
    }
    Ok(())
}
fn predecessor(owner: &Path) -> Result<(Value, Value)> {
    let manifest = document(&owner.join("manifest.toml"))?;
    let verdict = document(&owner.join("verdict.toml"))?;
    ensure!(
        verdict["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
            && manifest["state"] == crate::readiness_metadata::value(&"complete").expect("validated native metadata")
            && verdict["run_id"] == manifest["run_id"],
        "predecessor lacks completed passing verdict"
    );
    let ids = &manifest["identities"]["generated-key-inventory"];
    let path = safe_artifact(owner, crate::documents::field(ids, "evidence")?)?;
    ensure!(
        digest(&path)? == crate::documents::field(ids, "sha256")?,
        "predecessor generated-key inventory missing or corrupt"
    );
    let source: pipeline_model::readiness::SourceInventory =
        pipeline_model::read(&owner.join("source/inventory.toml"))?;
    ensure!(
        source.schema_version == pipeline_model::readiness::RUN_VERSION
            && identity(&source.files)? == source.sha256
            && Some(source.sha256.as_str()) == manifest["identities"]["source"]["sha256"].as_str(),
        "predecessor source snapshot identity mismatch"
    );
    for row in &source.files {
        ensure!(
            digest(&safe_artifact(&owner.join("source/files"), &row.path)?)? == row.sha256,
            "predecessor maintained source snapshot corrupt"
        );
    }
    let inventory = document(&path)?;
    authenticate_inventory(owner, &manifest, &inventory)?;
    Ok((manifest, inventory))
}
pub(crate) fn progress(candidate: &Value, previous: &Value) -> Result<Value> {
    if candidate.get("domain").is_none() || candidate.get("domain") != previous.get("domain") {
        return Ok(
            crate :: report ! ({ "comparable" : false , "reason" : "ROM/movie/profile/seed/input interval differs or is absent" , }),
        );
    }
    let mut result = crate :: report ! ({ "comparable" : true , });
    for (field, output) in [
        ("generated_instructions", "generated_delta"),
        ("interpreted_instructions", "interpreted_delta"),
        ("validated_distinct_keys", "validated_keys_delta"),
    ] {
        let count = |report: &Value| -> Result<i64> {
            if field == "validated_distinct_keys" {
                Ok(report.get(field).and_then(Value::as_integer).unwrap_or(0))
            } else {
                report[field].as_integer().context("progress count invalid")
            }
        };
        {
            let value: toml::Value = (count(candidate)? - count(previous)?).into();
            result.as_table_mut().context("native metadata table missing")?.insert((output).into(), value);
        };
    }
    Ok(result)
}
pub(crate) fn evaluate(
    run: &mut Run,
    profile_path: &Path,
    inventory_path: &Path,
    previous: Option<&Path>,
) -> Result<Value> {
    let report =
        derive(&run.path, &Value::try_from(&Value::try_from(&run.manifest)?)?, profile_path, inventory_path, previous)?;
    atomic_toml(&run.path.join("coverage.toml"), &report)?;
    run.record_check("changed-entry-coverage", report.clone())?;
    Ok(report)
}
pub(crate) fn derive(
    owner: &Path,
    manifest: &Value,
    profile_path: &Path,
    inventory_path: &Path,
    previous: Option<&Path>,
) -> Result<Value> {
    let profile = document(profile_path)?;
    let candidate = document(inventory_path)?;
    ensure!(
        profile["schema_version"] == crate::readiness_metadata::value(&1).expect("validated native metadata")
            && candidate["schema_version"] == crate::readiness_metadata::value(&1).expect("validated native metadata"),
        "unsupported coverage/inventory schema"
    );
    authenticate_inventory(owner, manifest, &candidate)?;
    ensure!(
        digest(inventory_path)?
            == crate::documents::field(&manifest["identities"]["generated-key-inventory"], "sha256")?,
        "candidate generated inventory artifact mismatch"
    );
    ensure!(
        profile["complete"] == crate::readiness_metadata::value(&true).expect("validated native metadata")
            && profile["expected_frames"] == manifest["scope"]["frames"]
            && profile["completed_frames"] == profile["expected_frames"]
            && profile["candidate_inventory_sha256"]
                == crate::readiness_metadata::value(&digest(inventory_path)?).expect("validated native metadata"),
        "instruction coverage lacks complete matching replay scope/identity"
    );
    let config = owner.join("source/files/config/functions.toml");
    if config.exists() {
        ensure!(
            candidate["config_sha256"]
                == crate::readiness_metadata::value(&digest(&config)?).expect("validated native metadata"),
            "candidate inventory config identity mismatch"
        );
    }
    let keys = crate::documents::array(&candidate, "keys")?;
    let mut current = BTreeMap::new();
    for row in keys {
        ensure!(current.insert(key(row)?, row).is_none(), "duplicate candidate key");
    }
    let mut counts = BTreeMap::new();
    let mut generated = 0u64;
    for row in crate::documents::array(&profile, "compiled")? {
        let key = key(row)?;
        let count = row["count"].as_integer().and_then(|n| u64::try_from(n).ok()).context("invalid compiled count")?;
        ensure!(
            current.get(&key).is_some_and(|entry| entry["opcode"] == row["opcode"])
                && counts.insert(key, count).is_none(),
            "coverage opcode/count/duplicate differs from inventory"
        );
        generated = generated.checked_add(count).context("compiled count overflow")?;
    }
    ensure!(
        counts.keys().eq(current.keys())
            && profile["generated_instructions"]
                == crate::readiness_metadata::value(&generated).expect("validated native metadata"),
        "executed table inventory/totals differ from candidate keys"
    );
    let mut interpreted = 0u64;
    for row in crate::documents::array(&profile, "interpreted")? {
        interpreted = interpreted
            .checked_add(
                row["count"].as_integer().and_then(|n| u64::try_from(n).ok()).context("invalid interpreted count")?,
            )
            .context("interpreted count overflow")?;
    }
    ensure!(
        profile["interpreted_instructions"]
            == crate::readiness_metadata::value(&interpreted).expect("validated native metadata"),
        "interpreted instruction totals differ"
    );
    let mut required = Vec::new();
    let mut parent_identity = Value::Table(toml::Table::new());
    let mut previous_manifest = None;
    if let Some(owner) = previous {
        let (prior_manifest, old) = predecessor(owner)?;
        ensure!(
            manifest.get("accepted_parent").is_none()
                || manifest.get("accepted_parent") == prior_manifest.get("run_id"),
            "predecessor differs from declared parent"
        );
        let old = crate::documents::array(&old, "keys")?
            .iter()
            .map(|row| Ok((key(row)?, row)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        let previous_inventory = document(&safe_artifact(
            owner,
            crate::documents::field(&prior_manifest["identities"]["generated-key-inventory"], "evidence")?,
        )?)?;
        let shared_change = candidate["execution_source_sha256"] != previous_inventory["execution_source_sha256"];
        for (key, row) in &current {
            let reason = if !old.contains_key(key) {
                Some("added")
            } else if **row != *old[key] {
                Some("body changed")
            } else if shared_change {
                Some("execution source changed")
            } else {
                None
            };
            if let Some(reason) = reason {
                required.push(crate :: report ! ({ "address" : key . 0 , "mx" : key . 1 , "reason" : reason , }));
            }
        }
        parent_identity = crate :: report ! ({ "run_id" : prior_manifest ["run_id"] , "run_path" : fs :: canonicalize (owner) ? , "inventory_sha256" : prior_manifest ["identities"] ["generated-key-inventory"] ["sha256"] , "source_sha256" : prior_manifest ["identities"] ["source"] ["sha256"] , });
        previous_manifest = Some(prior_manifest);
    }
    let uncovered = required
        .iter()
        .filter(|row| counts[&key(row).unwrap()] == 0)
        .map(|row| crate :: report ! ({ "address" : row ["address"] , "mx" : row ["mx"] , }))
        .collect::<Vec<_>>();
    let executed = crate::documents::array(&profile, "compiled")?
        .iter()
        .filter(|row| row["count"].as_integer().and_then(|n| u64::try_from(n).ok()).unwrap() > 0)
        .cloned()
        .collect::<Vec<_>>();
    let unexecuted = crate::documents::array(&profile, "compiled")?
        .iter()
        .filter(|row| row["count"] == crate::readiness_metadata::value(&0).expect("validated native metadata"))
        .map(|row| {
            let mut row = row.clone();
            row.as_table_mut().expect("validated profile row").insert(
                "validation".into(),
                if required.is_empty() { "unvalidated baseline" } else { "unvalidated" }.into(),
            );
            row
        })
        .collect::<Vec<_>>();
    let domain = manifest.get("identities").and_then(|ids| ids.get("inputs")).and_then(|inputs| {
        let fields = [
            ("rom", "rom_sha256"),
            ("movie", "movie_sha256"),
            ("input", "input_sha256"),
            ("profile", "profile"),
            ("seed", "seed"),
            ("controller", "controller"),
            ("reset", "reset"),
        ];
        let mut domain = toml::Table::new();
        for (output, field) in fields {
            domain.insert(output.into(), inputs.get(field)?.clone());
        }
        domain.insert("start".into(), 0.into());
        domain.insert("frames".into(), manifest.get("scope")?.get("frames")?.clone());
        Some(Value::Table(domain))
    });
    let status = if !uncovered.is_empty() || current.is_empty() || generated == 0 { "uncovered" } else { "pass" };
    let mut report = crate :: report ! ({ "schema_version" : 1 , "status" : status , "complete" : true , "required_keys" : required , "uncovered_keys" : uncovered , "executed" : executed , "unexecuted" : unexecuted , "unsupported" : family :: array (& profile , "interpreted") ? . iter () . filter (| r | r.get("supported").and_then(Value::as_bool) != Some(true)) . collect :: < Vec < _ > > () , "deferred" : family :: array (& profile , "interpreted") ? . iter () . filter (| r | r.get("supported").and_then(Value::as_bool) == Some(true)) . collect :: < Vec < _ > > () , "generated_instructions" : generated , "interpreted_instructions" : interpreted , "fallback_fraction" : if generated + interpreted > 0 { interpreted as f64 / (generated + interpreted) as f64 } else { 0.0 } , "validated_distinct_keys" : executed . len () , "predecessor" : parent_identity , "candidate_inventory_sha256" : digest (inventory_path) ? , "candidate_source_sha256" : manifest ["identities"] ["source"] ["sha256"] , "profile_sha256" : digest (profile_path) ? , "discovery" : "call-target misses are separate; not instruction counts" , });
    if let Some(domain) = domain {
        report.as_table_mut().expect("coverage report table").insert("domain".into(), domain);
    }
    if status == "uncovered" {
        {
            let value: toml::Value = if !uncovered.is_empty() {
                "changed compiled keys did not execute"
            } else {
                "empty table or no actual compiled execution"
            }
            .into();
            report.as_table_mut().context("native metadata table missing")?.insert(("reason").into(), value);
        };
    }
    let mut groups: BTreeMap<(u64, String, bool), Value> = BTreeMap::new();
    for (kind, field) in [("generated", "compiled"), ("interpreted", "interpreted")] {
        for row in crate::documents::array(&profile, field)? {
            let opcode =
                row["opcode"].as_integer().and_then(|n| u64::try_from(n).ok()).context("profile opcode missing")?;
            let pattern = row.get("pattern").and_then(Value::as_str).unwrap_or("unknown").to_owned();
            let emulation = row.get("emulation").and_then(Value::as_bool).unwrap_or(false);
            let group = groups . entry ((opcode , pattern . clone () , emulation)) . or_insert (crate :: report ! ({ "opcode" : opcode , "pattern" : pattern , "emulation" : emulation , "generated" : 0 , "interpreted" : 0 , }) ,) ;
            group[kind] = Value::Integer(i64::try_from(
                group[kind].as_integer().and_then(|n| u64::try_from(n).ok()).unwrap()
                    + row["count"].as_integer().and_then(|n| u64::try_from(n).ok()).unwrap(),
            )?);
        }
    }
    {
        let value: toml::Value = Value::Array(groups.into_values().collect::<Vec<_>>());
        report.as_table_mut().context("native metadata table missing")?.insert(("opcode_patterns").into(), value);
    };
    {
        let value: toml::Value = if let (Some(owner), Some(_)) = (previous, previous_manifest) {
            let verdict = document(&owner.join("verdict.toml"))?;
            let path = owner.join("coverage.toml");
            if path.exists()
                && verdict.get("checks").and_then(|checks| checks.get("changed-entry-coverage"))
                    == Some(&document(&path)?)
            {
                progress(&report, &document(&path)?)?
            } else {
                crate :: report ! ({ "comparable" : false , "reason" : "predecessor lacks instruction coverage" , })
            }
        } else {
            crate :: report ! ({ "comparable" : false , "reason" : "initial readiness baseline" , })
        };
        report.as_table_mut().context("native metadata table missing")?.insert(("progress").into(), value);
    };
    Ok(report)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };
    struct Temp(PathBuf);
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn project() -> Temp {
        let root = std::env::temp_dir()
            .join(format!("sm-coverage-controls-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(root.join("config")).unwrap();
        fs::write(root.join("config/functions.toml"), "# reviewed\n").unwrap();
        assert!(Command::new("git").args(["init", "-q"]).arg(&root).status().unwrap().success());
        assert!(Command::new("git").args(["add", "config"]).current_dir(&root).status().unwrap().success());
        Temp(root)
    }
    fn inventory(run: &mut Run, rows: &[(u64, &str)]) -> PathBuf {
        let mut source = String::new();
        for (address, body) in rows {
            source.push_str(&format!("fn step_{address:06x}_3(m: &mut Machine) {{\n{body}\n}}\n"));
        }
        source.push_str("pub static STEP_TABLE: &[StepEntry] = &[\n");
        for (address, _) in rows {
            source.push_str(&format!("(0x{address:06x}, 3, &[234], step_{address:06x}_3),\n"));
        }
        source . push_str ("];\npub const ROM_SHA256: Option<&str> = Some(\"0000000000000000000000000000000000000000000000000000000000000000\");\n") ;
        let steps = run.path.join("generated/steps.rs");
        fs::write(&steps, source).unwrap();
        generated_inventory(run, &steps).unwrap()
    }
    fn profile(run: &Run, inventory: &Path, rows: &[(u64, u64)]) -> PathBuf {
        let rows = rows . iter () . map (| (address , count) | crate :: report ! ({ "address" : address , "mx" : 3 , "opcode" : 234 , "count" : count , })) . collect :: < Vec < _ > > () ;
        let total =
            rows.iter().map(|row| row["count"].as_integer().and_then(|n| u64::try_from(n).ok()).unwrap()).sum::<u64>();
        let profile = crate :: report ! ({ "schema_version" : 1 , "complete" : true , "expected_frames" : 2 , "completed_frames" : 2 , "candidate_inventory_sha256" : digest (inventory) . unwrap () , "compiled" : rows , "generated_instructions" : total , "interpreted" : crate :: report ! (vec ! [crate :: report ! ({ "address" : 0x8003 , "mx" : 3 , "opcode" : 0x80 , "count" : 10 , })]) , "interpreted_instructions" : 10 , });
        let path = run.path.join("diagnostics/profile.toml");
        atomic_toml(&path, &profile).unwrap();
        path
    }
    fn run(root: &Path) -> Run {
        let mut run = Run::create(root, "parity", crate :: report ! ({ "frames" : 2 , }), None).unwrap();
        run.capture_source().unwrap();
        run
    }
    #[test]
    fn added_or_modified_unexecuted_keys_cannot_borrow_old_execution_and_corrupt_predecessors_fail() {
        let root = project();
        let mut parent = run(&root.0);
        let old = inventory(&mut parent, &[(0x8002, "body")]);
        let pp = profile(&parent, &old, &[(0x8002, 40)]);
        let old_report = evaluate(&mut parent, &pp, &old, None).unwrap();
        assert_eq!(old_report["status"], toml::Value::try_from("pass").unwrap());
        parent.finalize("pass", None, None, &[]).unwrap();
        let mut candidate = run(&root.0);
        let current = inventory(&mut candidate, &[(0x8002, "body"), (0x8020, "body")]);
        let p = profile(&candidate, &current, &[(0x8002, 40), (0x8020, 0)]);
        let report = evaluate(&mut candidate, &p, &current, Some(&parent.path)).unwrap();
        assert_eq!(report["status"], toml::Value::try_from("uncovered").unwrap());
        assert_eq!(
            report["required_keys"],
            toml::Value::try_from(crate::report!(vec![
                crate :: report ! ({ "address" : 0x8020 , "mx" : 3 , "reason" : "added" })
            ]))
            .unwrap()
        );
        let mut candidate = run(&root.0);
        let current = inventory(&mut candidate, &[(0x8002, "changed")]);
        let p = profile(&candidate, &current, &[(0x8002, 0)]);
        let report = evaluate(&mut candidate, &p, &current, Some(&parent.path)).unwrap();
        assert_eq!(report["status"], toml::Value::try_from("uncovered").unwrap());
        assert_eq!(report["required_keys"][0]["reason"], toml::Value::try_from("body changed").unwrap());
        let mut candidate = run(&root.0);
        let current = inventory(&mut candidate, &[(0x8002, "body"), (0x8020, "body")]);
        let p = profile(&candidate, &current, &[(0x8002, 40), (0x8020, 0)]);
        let report = evaluate(&mut candidate, &p, &current, None).unwrap();
        assert_eq!(report["status"], toml::Value::try_from("pass").unwrap());
        assert_eq!(report["unexecuted"][0]["validation"], toml::Value::try_from("unvalidated baseline").unwrap());
        assert_eq!(report["required_keys"], toml::Value::try_from(crate::report!(vec![])).unwrap());
        fs::write(old, "corrupt").unwrap();
        let mut candidate = run(&root.0);
        let current = inventory(&mut candidate, &[(0x8002, "body")]);
        let p = profile(&candidate, &current, &[(0x8002, 40)]);
        assert!(
            evaluate(&mut candidate, &p, &current, Some(&parent.path)).unwrap_err().to_string().contains("corrupt")
        );
    }
    #[test]
    fn actual_table_parser_rejects_duplicate_or_mismatched_symbols_and_profiles_bind_totals() {
        assert_eq!(
            table_rows("pub static STEP_TABLE: &[runtime::StepEntry] = &[];\n").unwrap(),
            crate::report!(vec![])
        );
        let root = project();
        let mut candidate = run(&root.0);
        let current = inventory(&mut candidate, &[(0x8002, "body")]);
        let source = fs::read_to_string(candidate.path.join("generated/steps.rs")).unwrap();
        assert_eq!(table_rows(&source).unwrap().as_array().unwrap().len(), 1);
        assert!(table_rows(&source.replace("&[234], step_008002_3", "&[234], step_008020_3")).is_err());
        let p = profile(&candidate, &current, &[(0x8002, 40)]);
        let mut report = document(&p).unwrap();
        {
            let value: toml::Value = 41.into();
            report
                .as_table_mut()
                .expect("native metadata table missing")
                .insert(("generated_instructions").into(), value);
        };
        atomic_toml(&p, &report).unwrap();
        assert!(evaluate(&mut candidate, &p, &current, None).is_err());
        let p = profile(&candidate, &current, &[(0x8002, 40)]);
        let mut report = document(&p).unwrap();
        let duplicate = report["compiled"][0].clone();
        report["compiled"].as_array_mut().unwrap().push(duplicate);
        atomic_toml(&p, &report).unwrap();
        assert!(evaluate(&mut candidate, &p, &current, None).is_err());
        let parent = crate :: report ! ({ "domain" : crate :: report ! ({ "frames" : 300 , }) , "generated_instructions" : 10 , "interpreted_instructions" : 20 , });
        let current = crate :: report ! ({ "domain" : crate :: report ! ({ "frames" : 200 , }) , "generated_instructions" : 30 , "interpreted_instructions" : 20 , });
        assert_eq!(progress(&current, &parent).unwrap()["comparable"], toml::Value::try_from(false).unwrap());
    }
}
